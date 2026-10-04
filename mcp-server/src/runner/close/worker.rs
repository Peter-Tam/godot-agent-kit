use super::*;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    v: u32,
    kind: String,
    request_id: String,
    arguments: Vec<Value>,
}
fn control(
    input: &mut UnixStream,
    id: &RequestId,
    kind: &str,
    arity: usize,
    at: Instant,
) -> Result<Vec<Value>, RoutingFailure> {
    let bytes = receive(input, 4096, at)?;
    let c: Control = codec::decode(&bytes, Stage::ValidateRequest)?;
    if c.v != 1 || c.request_id != id.as_str() || c.kind != kind || c.arguments.len() != arity {
        return Err(protocol_failure());
    }
    Ok(c.arguments)
}
fn disk(
    output: &mut UnixStream,
    id: &RequestId,
    read: &project_fs::DiskRead,
) -> Result<(), RoutingFailure> {
    if read.access_denied {
        return Err(RoutingFailure::new(
            OutcomeKind::DeniedAccess,
            DiagnosticCode::DeniedAccess,
            Stage::ReadDisk,
        ));
    }
    if let Some(m) = &read.metadata {
        event(
            output,
            id,
            &Event::DiskMetadata {
                reason: read.source.reason().ok_or_else(protocol_failure)?,
                file: m.file.clone(),
                collection: m.collection.clone(),
            },
        )
    } else {
        event(output, id, &Event::Disk(read.source.clone()))
    }
}
fn rpc(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    opcode: &str,
    args: &[Value],
    original: Option<&CollectionStamp>,
    started: Instant,
    output: &mut UnixStream,
) -> Result<codec::Reply, RoutingFailure> {
    let kind = match opcode {
        "close_begin" => "close_state",
        "close_prepare" => "close_prepared",
        "close_recheck" => "close_rechecked",
        "close_advance" => "close_progress",
        "close_wait" => "close_waited",
        "close_verify" => "close_sample",
        "close_finish" => "close_finished",
        "close_abort" => "close_aborted",
        _ => return Err(protocol_failure()),
    };
    let at = started + BUDGET;
    let at = if opcode == "close_abort" {
        at.min(Instant::now() + Duration::from_millis(20))
    } else {
        at
    };
    let bytes = codec::call(selected, request, opcode, args, at)?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let reply = codec::decode_reply(
        &bytes,
        kind,
        request,
        selected.target(),
        &selected.advertised_project_root,
        receipt,
        original,
    )?;
    if reply
        .native
        .as_ref()
        .is_some_and(|n| n.native_build_id != selected.native_build_id())
    {
        return Err(protocol_failure());
    }
    send(output, &bytes)?;
    Ok(reply)
}
fn survivor(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    started: Instant,
    at: Instant,
    output: &mut UnixStream,
) -> Result<(), RoutingFailure> {
    deadline(at)?;
    let reply = rpc(
        selected,
        request,
        "close_verify",
        &[json!("survivor")],
        None,
        started,
        output,
    )?;
    if reply.sample.is_some() {
        let read = project_fs::read_disk_with_metadata(selected, started)?;
        disk(output, request.request_id(), &read)?;
    }
    // Survivor acquisition has no recheck/authorization continuation. The
    // native guard is retired and its earlier failure remains sticky.
    Ok(())
}

fn verify(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    started: Instant,
    at: Instant,
    output: &mut UnixStream,
) -> Result<(), RoutingFailure> {
    let reply = rpc(
        selected,
        request,
        "close_verify",
        &[json!(purpose)],
        None,
        started,
        output,
    )?;
    if reply.status != "observed" {
        if purpose == "post_close" && survivor_available(&reply) {
            return survivor(selected, request, started, at, output);
        }
        return Ok(());
    }
    let original = reply
        .sample
        .as_ref()
        .ok_or_else(protocol_failure)?
        .collection
        .clone();
    let read = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, request.request_id(), &read)?;
    let reply = rpc(
        selected,
        request,
        "close_recheck",
        &[json!(purpose)],
        Some(&original),
        started,
        output,
    )?;
    if reply.status != "rechecked" {
        if purpose == "post_close" && survivor_available(&reply) {
            return survivor(selected, request, started, at, output);
        }
        return Ok(());
    }
    let (changes, denied) = project_fs::recheck_edit_disk_read(selected, &read, started)?;
    if denied {
        return Err(RoutingFailure::new(
            OutcomeKind::DeniedAccess,
            DiagnosticCode::DeniedAccess,
            Stage::Recheck,
        ));
    }
    event(output, request.request_id(), &Event::DiskChecked(changes))?;
    event(output, request.request_id(), &Event::Done)
}
fn attempt(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    input: &mut UnixStream,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<(), RoutingFailure> {
    let id = request.request_id();
    event(output, id, &Event::Selected(selected.target().clone()))?;
    send(
        output,
        &serde_json::to_vec(&(
            1,
            "close_binding",
            id.as_str(),
            selected.advertised_project_root.as_str(),
        ))
        .map_err(|_| protocol_failure())?,
    )?;
    if !selected.capabilities().close_gdscript || selected.native_api_revision() != 4 {
        return Err(RoutingFailure::new(
            OutcomeKind::UnsupportedObservation,
            DiagnosticCode::UnsupportedCapability,
            Stage::ReadEditor,
        ));
    }
    let budget = at
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(9000) as u64;
    let mut state = rpc(
        selected,
        request,
        "close_begin",
        &[json!(budget)],
        None,
        started,
        output,
    )?;
    if state.status != "inspected" {
        return Ok(());
    }
    let read = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, id, &read)?;
    let open = state
        .sample
        .as_ref()
        .and_then(|s| s.document.open_state().value())
        .copied();
    drop(state.sample.take());
    if open == Some(OpenState::NotOpen) {
        control(input, id, "recognize_close", 0, at)?;
        return verify(selected, request, "recognition", started, at, output);
    }
    if open != Some(OpenState::Open) {
        return Err(protocol_failure());
    }
    let mut args = control(input, id, "prepare_close", 1, at)?;
    let source = read.source.text().ok_or_else(protocol_failure)?;
    args.insert(0, json!(source));
    let mut prepared = rpc(
        selected,
        request,
        "close_prepare",
        &args,
        None,
        started,
        output,
    )?;
    if prepared.status != "prepared" {
        return Ok(());
    }
    let original = prepared
        .sample
        .as_ref()
        .ok_or_else(protocol_failure)?
        .collection
        .clone();
    drop(prepared.sample.take());
    drop(prepared.context.take());
    let current = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, id, &current)?;
    control(input, id, "recheck_close", 0, at)?;
    let reply = rpc(
        selected,
        request,
        "close_recheck",
        &[json!("pre_close")],
        Some(&original),
        started,
        output,
    )?;
    if reply.status != "rechecked" {
        return Ok(());
    }
    let (changes, denied) = project_fs::recheck_edit_disk_read(selected, &read, started)?;
    if denied {
        return Err(RoutingFailure::new(
            OutcomeKind::DeniedAccess,
            DiagnosticCode::DeniedAccess,
            Stage::Recheck,
        ));
    }
    let unchanged = changes.is_empty();
    event(output, id, &Event::DiskChecked(changes))?;
    if !unchanged {
        return Ok(());
    }
    let args = control(input, id, "authorize_close", 2, at)?;
    deadline(at)?;
    let reply = rpc(
        selected,
        request,
        "close_advance",
        &args,
        None,
        started,
        output,
    )?;
    if reply.status != "returned" {
        if survivor_available(&reply) {
            return survivor(selected, request, started, at, output);
        }
        return Ok(());
    }
    let waited = rpc(selected, request, "close_wait", &[], None, started, output)?;
    if waited.status != "completed" {
        if survivor_available(&waited) {
            return survivor(selected, request, started, at, output);
        }
        return Ok(());
    }
    verify(selected, request, "post_close", started, at, output)
}
/// Rejects shell entry: both inherited descriptors must be connected Unix sockets.
pub fn worker_main() -> Option<i32> {
    let mut input = UnixStream::from(io::stdin().as_fd().try_clone_to_owned().ok()?);
    let mut output = UnixStream::from(io::stdout().as_fd().try_clone_to_owned().ok()?);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let bytes = match receive(&mut input, 4096, Instant::now() + BUDGET) {
        Ok(bytes) => bytes,
        Err(_) => return Some(1),
    };
    let (v, id, root, session, path, registry, elapsed): (
        u32,
        String,
        String,
        Option<String>,
        String,
        String,
        u64,
    ) = match codec::decode(&bytes, Stage::ValidateRequest) {
        Ok(startup) => startup,
        Err(_) => return Some(1),
    };
    if v != 6
        || elapsed >= 9_500_000
        || registry.len() > 1024
        || !Path::new(&registry).is_absolute()
    {
        return Some(1);
    }
    let request = (|| {
        Some(ObservationRequest::new(
            RequestId::new(id).ok()?,
            ProjectRoot::new(root).ok()?,
            session.map(SessionId::new).transpose().ok()?,
            ResourcePath::new(path).ok()?,
        ))
    })();
    let Some(request) = request else {
        return Some(1);
    };
    let now = Instant::now();
    let started = now
        .checked_sub(Duration::from_micros(elapsed))
        .unwrap_or(now);
    let at = started + BUDGET;
    let result = match target::resolve(&request, Path::new(&registry), at) {
        Ok(mut selected) => {
            let result = attempt(
                &mut selected,
                &request,
                &mut input,
                &mut output,
                started,
                at,
            );
            match result {
                Ok(()) => rpc(
                    &mut selected,
                    &request,
                    "close_finish",
                    &[],
                    None,
                    started,
                    &mut output,
                )
                .map(|_| ()),
                Err(error) => {
                    let _ = rpc(
                        &mut selected,
                        &request,
                        "close_abort",
                        &[],
                        None,
                        started,
                        &mut output,
                    );
                    Err(error)
                }
            }
        }
        Err(e) => Err(e),
    };
    Some(match result {
        Ok(()) => 0,
        Err(e) => {
            if event(&mut output, request.request_id(), &Event::Failed(e)).is_ok() {
                0
            } else {
                1
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_control_rejects_duplicate_authority_at_any_depth() {
        let id = RequestId::new("close-control").unwrap();
        for arguments in [
            r#"[{"basis":{"current_version":"1","current_version":"2"}}]"#,
            r#"[{"source_sha256":"a","source_sha256":"b"}]"#,
            r#"[{"basis":{"current_version":"1"},"basis":{"current_version":"2"}}]"#,
        ] {
            let (mut parent, mut child) = UnixStream::pair().unwrap();
            let bytes = format!(
                r#"{{"v":1,"kind":"prepare_close","request_id":"close-control","arguments":{arguments}}}"#
            );
            send(&mut parent, bytes.as_bytes()).unwrap();
            assert!(control(
                &mut child,
                &id,
                "prepare_close",
                1,
                Instant::now() + Duration::from_secs(1)
            )
            .is_err());
        }
    }
}
