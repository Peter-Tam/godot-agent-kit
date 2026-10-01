use super::*;
fn receive(input: &mut UnixStream, limit: usize, at: Instant) -> Result<Vec<u8>, RoutingFailure> {
    deadline(at)?;
    input
        .set_read_timeout(Some(at.saturating_duration_since(Instant::now())))
        .map_err(|_| protocol_failure())?;
    let mut h = [0; 4];
    input.read_exact(&mut h).map_err(|_| {
        failure(if Instant::now() >= at {
            Reason::Timeout
        } else {
            Reason::Disconnected
        })
    })?;
    let n = u32::from_be_bytes(h) as usize;
    if n == 0 || n > limit {
        return Err(protocol_failure());
    }
    let mut bytes = vec![0; n];
    input
        .set_read_timeout(Some(at.saturating_duration_since(Instant::now())))
        .map_err(|_| protocol_failure())?;
    input.read_exact(&mut bytes).map_err(|_| {
        failure(if Instant::now() >= at {
            Reason::Timeout
        } else {
            Reason::Disconnected
        })
    })?;
    deadline(at)?;
    Ok(bytes)
}
fn entering(output: &mut UnixStream, id: &RequestId, step: &str) -> Result<(), RoutingFailure> {
    let bytes = serde_json::to_vec(&(1, "open_entering", id.as_str(), step))
        .map_err(|_| protocol_failure())?;
    send(output, &bytes)
}
fn disk(
    output: &mut UnixStream,
    id: &RequestId,
    read: &project_fs::DiskRead,
) -> Result<(), RoutingFailure> {
    if read.access_denied {
        return Err(failure(Reason::DeniedAccess));
    }
    if let Some(meta) = &read.metadata {
        event(
            output,
            id,
            &Event::DiskMetadata {
                reason: read.source.reason().ok_or_else(protocol_failure)?,
                file: meta.file.clone(),
                collection: meta.collection.clone(),
            },
        )
    } else {
        send(output, &codec::encode_disk(id, &read.source)?)
    }
}
fn rpc(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    opcode: &str,
    args: &[&str],
    original: Option<&CollectionStamp>,
    timing: (Instant, Instant),
    output: &mut UnixStream,
) -> Result<codec::Reply, RoutingFailure> {
    let (started, at) = timing;
    let kind = match opcode {
        "open_begin" => "open_state",
        "open_prepare" => "open_prepared",
        "open_advance" => "open_progress",
        "open_verify" => "open_sample",
        "open_recheck" => "open_rechecked",
        _ => return Err(protocol_failure()),
    };
    let budget = (opcode == "open_begin").then(|| {
        at.saturating_duration_since(Instant::now())
            .as_millis()
            .min(9000) as u64
    });
    let bytes = codec::call(selected, request, opcode, args, budget, at)?;
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
    send(output, &bytes)?;
    Ok(reply)
}
fn verify(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    started: Instant,
    at: Instant,
    output: &mut UnixStream,
) -> Result<(), RoutingFailure> {
    entering(
        output,
        request.request_id(),
        if purpose == "recognition" {
            "verify:recognition"
        } else {
            "verify:post_open"
        },
    )?;
    let sample = rpc(
        selected,
        request,
        "open_verify",
        &[purpose],
        None,
        (started, at),
        output,
    )?;
    if sample.status != "observed" {
        return Ok(());
    }
    let original = sample
        .sample
        .as_ref()
        .ok_or_else(protocol_failure)?
        .collection
        .clone();
    drop(sample);
    let read = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, request.request_id(), &read)?;
    let rechecked = rpc(
        selected,
        request,
        "open_recheck",
        &[purpose],
        Some(&original),
        (started, at),
        output,
    )?;
    if rechecked.status != "unchanged" {
        return Ok(());
    }
    let (changes, denied) = project_fs::recheck_edit_disk_read(selected, &read, started)?;
    if denied {
        return Err(failure(Reason::DeniedAccess));
    }
    deadline(at)?;
    event(output, request.request_id(), &Event::DiskChecked(changes))?;
    event(output, request.request_id(), &Event::Done)
}
fn attempt(
    selected: &mut SelectedSession,
    request: &OpenRequest,
    input: &mut UnixStream,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<(), RoutingFailure> {
    let observation = request.observation();
    let id = request.request_id();
    event(output, id, &Event::Selected(selected.target().clone()))?;
    let binding = serde_json::to_vec(&(
        1,
        "open_binding",
        id.as_str(),
        selected.advertised_project_root.as_str(),
    ))
    .map_err(|_| protocol_failure())?;
    send(output, &binding)?;
    if !selected.capabilities().open_gdscript || selected.native_api_revision() != 2 {
        return Err(failure(Reason::UnsupportedCapability));
    }
    let mut state = rpc(
        selected,
        observation,
        "open_begin",
        &[],
        None,
        (started, at),
        output,
    )?;
    if state.status != "inspected" {
        return Ok(());
    }
    let open = state
        .native
        .as_ref()
        .and_then(|n| n.open)
        .ok_or_else(protocol_failure)?;
    drop(state.sample.take());
    let read = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, id, &read)?;
    if open {
        return verify(selected, observation, "recognition", started, at, output);
    }
    let Some(source) = read.source.text() else {
        return Err(RoutingFailure::new(
            if read.source.reason() == Some(SourceReason::DiskMissing) {
                OutcomeKind::MissingTarget
            } else {
                OutcomeKind::UnsupportedObservation
            },
            match read.source.reason() {
                Some(SourceReason::DiskMissing) => DiagnosticCode::DiskMissing,
                Some(SourceReason::TooLarge) => DiagnosticCode::TooLarge,
                _ => DiagnosticCode::DiskUnreadable,
            },
            Stage::ReadDisk,
        ));
    };
    let cache = state.cache.ok_or_else(protocol_failure)?;
    if !["absent", "present"].contains(&cache.state) {
        return Err(failure(Reason::UnsupportedCapability));
    }
    let file = read
        .source
        .witness()
        .and_then(Witness::disk_file_id)
        .ok_or_else(protocol_failure)?;
    let project = selected.target().project_file_id().clone();
    let hash = crate::project_fs_validation::hex_sha256(source.as_bytes());
    let length = source.len().to_string();
    entering(output, id, "prepare")?;
    let prepared = rpc(
        selected,
        observation,
        "open_prepare",
        &[
            project.device().as_str(),
            project.inode().as_str(),
            file.device().as_str(),
            file.inode().as_str(),
            cache.state,
            cache.script.as_ref().map_or("", DecimalCounter::as_str),
            &hash,
            &length,
            source,
        ],
        None,
        (started, at),
        output,
    )?;
    if prepared.status != "prepared" {
        return Ok(());
    }
    let cached = prepared.mode.as_deref() == Some("cached");
    drop(prepared);
    // A second independently confined D read accompanies the preparation sample.
    let current = project_fs::read_disk_with_metadata(selected, started)?;
    disk(output, id, &current)?;
    let control = receive(input, CONTROL_LIMIT, at)?;
    let (v, kind, request_id, source_hash, context_hash): (u32, String, String, String, String) =
        serde_json::from_slice(&control).map_err(|_| protocol_failure())?;
    if v != 1 || request_id != id.as_str() {
        return Err(protocol_failure());
    }
    if kind == "abort" && source_hash.is_empty() && context_hash.is_empty() {
        return Err(failure(Reason::Cancelled));
    }
    if kind != "authorize_open" || source_hash.len() > 64 || context_hash.len() > 64 {
        return Err(protocol_failure());
    }
    let changes = project_fs::recheck_disk_read(selected, &read, started)?;
    if !changes.is_empty() {
        return Err(RoutingFailure::new(
            OutcomeKind::ProtocolError,
            DiagnosticCode::SourceChanged,
            Stage::Recheck,
        ));
    }
    for step in if cached {
        &["bind", "open"][..]
    } else {
        &["bind", "compile", "open"][..]
    } {
        deadline(at)?;
        entering(output, id, step)?;
        let progress = rpc(
            selected,
            observation,
            "open_advance",
            &[step, &source_hash, &context_hash],
            None,
            (started, at),
            output,
        )?;
        if progress.status != "ready" {
            return Ok(());
        }
    }
    verify(selected, observation, "post_open", started, at, output)
}
/// Private same-binary mode requires inherited connected sockets, never shell pipes.
pub fn worker_main() -> Option<i32> {
    let mut input = UnixStream::from(io::stdin().as_fd().try_clone_to_owned().ok()?);
    let mut output = UnixStream::from(io::stdout().as_fd().try_clone_to_owned().ok()?);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let startup = receive(&mut input, CONTROL_LIMIT, Instant::now() + BUDGET).ok()?;
    type Startup = (u32, String, String, Option<String>, String, String, u64);
    let (v, id, root, session, path, registry, elapsed): Startup =
        serde_json::from_slice(&startup).ok()?;
    if v != 4
        || elapsed >= 9_500_000
        || registry.len() > 1024
        || !Path::new(&registry).is_absolute()
    {
        return Some(1);
    }
    let request = OpenRequest::new(
        RequestId::new(id).ok()?,
        ProjectRoot::new(root).ok()?,
        session.map(SessionId::new).transpose().ok()?,
        ResourcePath::new(path).ok()?,
    )
    .ok()?;
    let now = Instant::now();
    let started = now
        .checked_sub(Duration::from_micros(elapsed))
        .unwrap_or(now);
    let at = started + BUDGET;
    let result = match target::resolve(request.observation(), Path::new(&registry), at) {
        Ok(mut selected) => {
            let result = attempt(
                &mut selected,
                &request,
                &mut input,
                &mut output,
                started,
                at,
            );
            codec::release(&mut selected, request.observation(), result.is_err(), at);
            result
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
