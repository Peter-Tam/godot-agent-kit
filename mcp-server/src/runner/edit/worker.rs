//! Owned child acquires all project/editor/helper evidence; the supervisor alone authorizes apply.
use super::*;
use crate::script_edit::Reason;
use serde_json::Value;

fn receive(
    stream: &mut UnixStream,
    limit: usize,
    deadline_at: Instant,
) -> Result<Vec<u8>, RoutingFailure> {
    deadline(deadline_at)?;
    let mut header = [0; 4];
    stream
        .set_read_timeout(Some(deadline_at.saturating_duration_since(Instant::now())))
        .map_err(|_| protocol_failure())?;
    stream.read_exact(&mut header).map_err(|_| {
        error(if Instant::now() >= deadline_at {
            Reason::Deadline
        } else {
            Reason::Disconnection
        })
    })?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 || len > limit {
        return Err(protocol_failure());
    }
    let mut bytes = vec![0; len];
    deadline(deadline_at)?;
    stream
        .set_read_timeout(Some(deadline_at.saturating_duration_since(Instant::now())))
        .map_err(|_| protocol_failure())?;
    stream.read_exact(&mut bytes).map_err(|_| {
        error(if Instant::now() >= deadline_at {
            Reason::Deadline
        } else {
            Reason::Disconnection
        })
    })?;
    deadline(deadline_at)?;
    Ok(bytes)
}
fn control(
    input: &mut UnixStream,
    id: &RequestId,
    deadline_at: Instant,
) -> Result<bool, RoutingFailure> {
    let bytes = receive(input, CONTROL_LIMIT, deadline_at)?;
    let tuple: (u32, String, String) =
        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
    if tuple.0 != 1 || tuple.2 != id.as_str() {
        return Err(protocol_failure());
    }
    match tuple.1.as_str() {
        "authorize_apply" => Ok(true),
        "abort" => Ok(false),
        _ => Err(protocol_failure()),
    }
}
const HELPER_REPLY_LIMIT: usize = SOURCE_LIMIT_BYTES + 4096;
fn helper(
    request: &EditRequest,
    context: &wire::edit::EditContext,
    purpose: stock_validation::Purpose,
    input: &mut UnixStream,
    output: &mut UnixStream,
    started: Instant,
    deadline_at: Instant,
) -> Result<(stock_validation::ValidationResult, u64, u64), RoutingFailure> {
    if matches!(
        purpose,
        stock_validation::Purpose::OpenContext | stock_validation::Purpose::CloseContext
    ) {
        return Err(protocol_failure());
    }
    let begin = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    send(
        output,
        &ipc::helper_request(request.request_id(), purpose, context)?,
    )?;
    let bytes = receive(input, HELPER_REPLY_LIMIT, deadline_at)?;
    let end = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let reply: ipc::HelperReplyIn =
        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
    Ok((reply.domain(request.request_id(), purpose)?, begin, end))
}
fn actual_context(selected: &SelectedSession, context: &wire::edit::EditContext) -> Option<String> {
    let path = ResourcePath::new("res://project.godot").ok()?;
    let config = confined::capture(selected.target().project_root(), &path, 64 * 1024).ok()??;
    let warnings = serde_json::to_string(&context.warnings).ok()?;
    let classes = serde_json::to_string(&context.global_classes).ok()?;
    Some(confined::hex_sha256(
        format!("{}:{}:{}", config.sha256, warnings, classes).as_bytes(),
    ))
}
fn context_current(
    selected: &SelectedSession,
    context: &wire::edit::EditContext,
    result: &stock_validation::ValidationResult,
) -> bool {
    actual_context(selected, context).as_ref() == result.context_sha256.as_ref()
        && result.context_sha256.is_some()
}
fn capture_dependencies(
    selected: &SelectedSession,
    result: &stock_validation::ValidationResult,
) -> Option<Vec<ipc::FreshDependency>> {
    if result.sources.is_empty() || result.sources.len() > 33 {
        return None;
    }
    let mut out = Vec::with_capacity(result.sources.len() - 1);
    for src in result.sources.iter().skip(1) {
        let path = ResourcePath::new(src.path.clone()).ok()?;
        let captured =
            confined::capture(selected.target().project_root(), &path, SOURCE_LIMIT_BYTES)
                .ok()??;
        out.push(ipc::FreshDependency {
            path: src.path.clone(),
            device: captured.device.to_string(),
            inode: captured.inode.to_string(),
            sha256: captured.sha256,
            utf8_bytes: captured.bytes,
        });
    }
    Some(out)
}
fn dependencies_current(
    selected: &SelectedSession,
    result: &stock_validation::ValidationResult,
) -> bool {
    let Some(fresh) = capture_dependencies(selected, result) else {
        return false;
    };
    fresh
        .iter()
        .zip(result.sources.iter().skip(1))
        .all(|(actual, expected)| {
            actual.sha256 == expected.sha256
                && actual.utf8_bytes == expected.utf8_bytes
                && actual.device.parse::<u64>().ok() == expected.device
                && actual.inode.parse::<u64>().ok() == expected.inode
        })
}
fn same_context(a: &wire::edit::EditContext, b: &wire::edit::EditContext) -> bool {
    a.executable == b.executable
        && a.warnings == b.warnings
        && a.global_classes == b.global_classes
        && a.collection.clock_id() == b.collection.clock_id()
        && a.collection.finished_tick_us() <= b.collection.started_tick_us()
}
fn same_saved(
    a: Option<&script_edit::SavedStateEvidence>,
    b: Option<&script_edit::SavedStateEvidence>,
) -> bool {
    a.zip(b).is_some_and(|(left, right)| {
        left.document == right.document
            && left.current_version == right.current_version
            && left.saved_version == right.saved_version
            && left.resource_edited == right.resource_edited
            && left.save_profile == right.save_profile
            && left.original_preserved == right.original_preserved
            && left.desired_preserved == right.desired_preserved
    })
}

/// Consume one authenticated readonly editor sample, independently read/recheck D,
/// publish the ordinary bounded observation event sequence, then separate saved/mtime.
fn sample(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    evidence: wire::edit::EditSample,
    (purpose, phase): (&str, &str),
    initial_disk: Option<project_fs::DiskRead>,
    (started, deadline_at): (Instant, Instant),
    output: &mut UnixStream,
) -> Result<Option<wire::edit::EditContext>, RoutingFailure> {
    deadline(deadline_at)?;
    let disk = match initial_disk {
        Some(disk) => disk,
        None => project_fs::read_disk_with_metadata(selected, started)?,
    };
    if disk.access_denied {
        return Err(error(Reason::DeniedAccess));
    }
    deadline(deadline_at)?;
    let checked = wire::edit::recheck(
        selected,
        request,
        purpose,
        &evidence.collection,
        started,
        deadline_at,
    )?;
    let (changes, denied) = project_fs::recheck_edit_disk_read(selected, &disk, started)?;
    if denied {
        return Err(error(Reason::DeniedAccess));
    }
    deadline(deadline_at)?;
    let mtime = if disk.source.availability() == Availability::Observed {
        disk.mtime.clone()
    } else {
        None
    };
    let id = request.request_id();
    if phase != "prepared" {
        event(output, id, &Event::Selected(selected.target().clone()))?;
    }
    event(output, id, &Event::Sample(Box::new(evidence.sample)))?;
    let disk_event = match disk.metadata {
        Some(project_fs::DiskMetadata { file, collection }) => Event::DiskMetadata {
            reason: disk.source.reason().ok_or_else(protocol_failure)?,
            file,
            collection,
        },
        None => Event::Disk(disk.source),
    };
    event(output, id, &disk_event)?;
    event(output, id, &Event::Rechecked(checked.recheck))?;
    event(output, id, &Event::DiskChecked(changes))?;
    event(output, id, &Event::Done)?;
    extra(
        output,
        ipc::detail(id, phase, evidence.saved.as_ref(), mtime.as_ref()),
    )?;
    // A different context across the same independent sample's recheck is
    // unavailable, not a copied positive witness.
    let result = evidence.context.filter(|before| {
        checked
            .context
            .as_ref()
            .is_some_and(|after| same_context(before, after))
            && same_saved(evidence.saved.as_ref(), checked.saved.as_ref())
    });
    Ok(result)
}
fn valid(result: &stock_validation::ValidationResult) -> bool {
    result.status == "valid"
        && !matches!(
            result.purpose,
            stock_validation::Purpose::OpenContext | stock_validation::Purpose::CloseContext
        )
        && result.opening_binding.is_none()
        && result.cleanup_confirmed
        && result
            .sources
            .iter()
            .all(|src| src.diagnostics_completed && src.symbols_completed)
        && !result.sources.is_empty()
        && result.diagnostics.is_empty()
}
fn run_worker(
    request: EditRequest,
    registry: &Path,
    input: &mut UnixStream,
    output: &mut UnixStream,
    started: Instant,
    deadline_at: Instant,
) -> Result<(), RoutingFailure> {
    let selected_req = ObservationRequest::new(
        request.request_id().clone(),
        request.project_root().clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    );
    let mut selected = target::resolve(&selected_req, registry, deadline_at)?;
    let actual = selected.target();
    let expected = request.expected().target();
    if actual.session_id() != expected.session_id()
        || actual.project_file_id() != expected.project_file_id()
        || actual.project_root() != expected.project_root()
        || actual.script_path() != expected.script_path()
        || actual.godot_version() != expected.godot_version()
    {
        extra(
            output,
            ipc::terminal(
                request.request_id(),
                if actual.session_id() != expected.session_id() {
                    Reason::SessionChanged
                } else {
                    Reason::IdentityChanged
                },
                true,
                None,
            ),
        )?;
        return Ok(());
    }
    event(
        output,
        request.request_id(),
        &Event::Selected(actual.clone()),
    )?;
    // No source-bearing preparation before unique authentication and exact native family.
    if !selected.capabilities().edit_open_gdscript
        || selected.native_api_revision() != 3
        || selected.native_build_id().len() != 64
    {
        return Err(error(Reason::UnsupportedEngine));
    }
    // Confined read/access precedes any source-bearing preparation refusal.
    // Reuse its D witness for the first independent editor sample.
    let initial_disk = project_fs::read_disk_with_metadata(&selected, started)?;
    if initial_disk.access_denied {
        return Err(error(Reason::DeniedAccess));
    }
    deadline(deadline_at)?;
    let prepared = wire::edit::prepare(
        &mut selected,
        &selected_req,
        request.expected(),
        request.replacement_source(),
        started,
        deadline_at,
    )?;
    if prepared.status == "busy" {
        extra(
            output,
            ipc::terminal(request.request_id(), Reason::Busy, true, None),
        )?;
        return Ok(());
    }
    if prepared.status != "prepared" {
        extra(
            output,
            ipc::terminal(request.request_id(), prepared.reason, true, None),
        )?;
        return Ok(());
    }
    let changed = prepared.intent_changed.ok_or_else(protocol_failure)?;
    if changed {
        let native = prepared.native.as_ref().ok_or_else(protocol_failure)?;
        if native.status != "prepared"
            || native.stage != "prepared"
            || native.expected_hash != request.expected().source().sha256
            || native.desired_hash
                != script_edit::SourceDigest::of(request.replacement_source().as_str()).sha256
            || native.project != *selected.target().project_file_id()
            || Some(&native.file) != request.expected().document().disk_file_id()
            || native.prepared_current != *request.expected().current_version()
            || native.prepared_saved != native.prepared_current
            || native.buffer_changed
            || native.resource_changed
            || native.write_started
            || native.edited_attempted
            || native.tag_attempted
        {
            return Err(protocol_failure());
        }
    } else if prepared.native.is_some() {
        return Err(protocol_failure());
    }
    let Some(prepared_sample) = prepared.sample else {
        extra(
            output,
            ipc::terminal(request.request_id(), prepared.reason, true, None),
        )?;
        return Ok(());
    };
    let initial_purpose = if changed { "preflight" } else { "unchanged" };
    let Some(initial_context) = sample(
        &mut selected,
        &selected_req,
        prepared_sample,
        (initial_purpose, "prepared"),
        Some(initial_disk),
        (started, deadline_at),
        output,
    )?
    else {
        let terminal =
            wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at).ok();
        extra(
            output,
            ipc::terminal(
                request.request_id(),
                Reason::UnavailableObservation,
                terminal
                    .as_ref()
                    .is_some_and(|t| t.terminal_before_boundary),
                terminal.as_ref().map(|t| &t.collection),
            ),
        )?;
        return Ok(());
    };
    let (result, begin, end) = helper(
        &request,
        &initial_context,
        if changed {
            stock_validation::Purpose::Preflight
        } else {
            stock_validation::Purpose::Unchanged
        },
        input,
        output,
        started,
        deadline_at,
    )?;
    deadline(deadline_at)?;
    let good_context = context_current(&selected, &initial_context, &result);
    let good_deps = dependencies_current(&selected, &result);
    extra(
        output,
        ipc::validation(
            request.request_id(),
            if changed { "preflight" } else { "unchanged" },
            &result,
            begin,
            end,
            good_context,
            good_deps,
        ),
    )?;
    if !valid(&result) || !good_context || !good_deps {
        let terminal =
            wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at).ok();
        extra(
            output,
            ipc::terminal(
                request.request_id(),
                Reason::ValidationUnavailable,
                terminal
                    .as_ref()
                    .is_some_and(|t| t.terminal_before_boundary),
                terminal.as_ref().map(|t| &t.collection),
            ),
        )?;
        return Ok(());
    }
    let mut last_result = result;
    if changed {
        // Helper preflight is not a mutation guard; acquire fresh actual editor/D facts.
        let guarded = wire::edit::verify(
            &mut selected,
            &selected_req,
            "preflight",
            started,
            deadline_at,
        )?;
        let guard_context = sample(
            &mut selected,
            &selected_req,
            guarded,
            ("preflight", "guard"),
            None,
            (started, deadline_at),
            output,
        )?;
        if !guard_context
            .as_ref()
            .is_some_and(|c| same_context(&initial_context, c))
        {
            let terminal =
                wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at).ok();
            extra(
                output,
                ipc::terminal(
                    request.request_id(),
                    Reason::RevisionMismatch,
                    terminal
                        .as_ref()
                        .is_some_and(|t| t.terminal_before_boundary),
                    terminal.as_ref().map(|t| &t.collection),
                ),
            )?;
            return Ok(());
        }
        // No edit_apply can be sent until the parent has reduced the fresh guard,
        // recorded may_apply and released exactly one authorization control.
        if !control(input, request.request_id(), deadline_at)? {
            let terminal =
                wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at).ok();
            extra(
                output,
                ipc::terminal(
                    request.request_id(),
                    Reason::Cancellation,
                    terminal
                        .as_ref()
                        .is_some_and(|t| t.terminal_before_boundary),
                    terminal.as_ref().map(|t| &t.collection),
                ),
            )?;
            return Ok(());
        }
        let applied = wire::edit::apply(
            &mut selected,
            &selected_req,
            started,
            deadline_at,
            |stage| extra(output, ipc::progress(request.request_id(), stage)),
        )?;
        let status = applied.status;
        let applied_reason = applied.reason;
        if let Some(last) = applied.terminal_event {
            let mut record = ipc::progress(request.request_id(), &last);
            if let Some(map) = record.as_object_mut() {
                map.insert("kind".into(), Value::String("edit_native_final".into()));
            }
            extra(output, record)?;
        } else if let Some(native) = applied.native {
            let last = wire::edit::AppliedEvent {
                stage: native.stage,
                collection: applied.collection,
                native,
                before: None,
                after: None,
            };
            let mut record = ipc::progress(request.request_id(), &last);
            if let Some(map) = record.as_object_mut() {
                map.insert("kind".into(), Value::String("edit_native_final".into()));
            }
            extra(output, record)?;
        }
        if status == "refused" {
            let terminal =
                wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at)?;
            extra(
                output,
                ipc::terminal(
                    request.request_id(),
                    applied_reason,
                    terminal.terminal_before_boundary,
                    Some(&terminal.collection),
                ),
            )?;
            return Ok(());
        }
        // Immediate independent actual-source sample before invoking a new stock helper.
        let immediate = wire::edit::verify(
            &mut selected,
            &selected_req,
            "post_change",
            started,
            deadline_at,
        )?;
        let post_context = sample(
            &mut selected,
            &selected_req,
            immediate,
            ("post_change", "post"),
            None,
            (started, deadline_at),
            output,
        )?;
        let Some(post_context) = post_context else {
            let _ = wire::edit::terminal(&mut selected, &selected_req, true, started, deadline_at);
            return Err(error(Reason::UnavailableObservation));
        };
        let (result, begin, end) = helper(
            &request,
            &post_context,
            stock_validation::Purpose::PostChange,
            input,
            output,
            started,
            deadline_at,
        )?;
        deadline(deadline_at)?;
        let matching_context = context_current(&selected, &post_context, &result)
            && same_context(&initial_context, &post_context);
        let matching_deps = dependencies_current(&selected, &result);
        extra(
            output,
            ipc::validation(
                request.request_id(),
                "post_change",
                &result,
                begin,
                end,
                matching_context,
                matching_deps,
            ),
        )?;
        last_result = result;
    }
    // A NEW observation after helper (also unchanged path's only follow-up sample).
    let purpose = if changed { "post_change" } else { "unchanged" };
    let final_sample =
        wire::edit::verify(&mut selected, &selected_req, purpose, started, deadline_at)?;
    let final_context = sample(
        &mut selected,
        &selected_req,
        final_sample,
        (purpose, "verified"),
        None,
        (started, deadline_at),
        output,
    )?;
    let context_start = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let captured = final_context
        .as_ref()
        .and_then(|c| actual_context(&selected, c));
    let dependencies = capture_dependencies(&selected, &last_result);
    let actual = dependencies.as_ref().is_some_and(|deps| {
        deps.iter()
            .zip(last_result.sources.iter().skip(1))
            .all(|(a, b)| {
                a.sha256 == b.sha256
                    && a.utf8_bytes == b.utf8_bytes
                    && a.device.parse::<u64>().ok() == b.device
                    && a.inode.parse::<u64>().ok() == b.inode
            })
    });
    let current = final_context
        .as_ref()
        .is_some_and(|c| same_context(&initial_context, c))
        && captured.as_ref() == last_result.context_sha256.as_ref()
        && captured.is_some()
        && actual;
    let context_end = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    extra(
        output,
        ipc::context(
            request.request_id(),
            context_start,
            context_end,
            current,
            captured
                .as_deref()
                .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000"),
            dependencies.as_deref().unwrap_or(&[]),
        ),
    )?;
    let terminal = wire::edit::terminal(&mut selected, &selected_req, false, started, deadline_at)?;
    extra(
        output,
        ipc::terminal(
            request.request_id(),
            terminal.reason,
            terminal.terminal_before_boundary,
            Some(&terminal.collection),
        ),
    )?;
    Ok(())
}
/// An internal mode must have inherited bidirectional Unix sockets; shell pipes cannot dispatch it.
pub fn worker_main() -> Option<i32> {
    let input_fd = io::stdin().as_fd().try_clone_to_owned().ok()?;
    let output_fd = io::stdout().as_fd().try_clone_to_owned().ok()?;
    let mut input = UnixStream::from(input_fd);
    let mut output = UnixStream::from(output_fd);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let mut receive_startup = || -> Result<_, RoutingFailure> {
        let deadline_at = Instant::now() + BUDGET;
        let frame = receive(&mut input, STARTUP_LIMIT, deadline_at)?;
        type Startup = (u32, String, String, Option<String>, String, String, u64);
        let (v, id, root, session, path, registry, elapsed): Startup =
            serde_json::from_slice(&frame).map_err(|_| protocol_failure())?;
        if v != 5
            || elapsed >= 9_500_000
            || registry.len() > 1024
            || !Path::new(&registry).is_absolute()
        {
            return Err(protocol_failure());
        }
        let now = Instant::now();
        let started = now
            .checked_sub(Duration::from_micros(elapsed))
            .unwrap_or(now);
        let request_id = RequestId::new(id).map_err(|_| protocol_failure())?;
        let project = ProjectRoot::new(root).map_err(|_| protocol_failure())?;
        let sid = session
            .map(SessionId::new)
            .transpose()
            .map_err(|_| protocol_failure())?;
        let path = ResourcePath::new(path).map_err(|_| protocol_failure())?;
        let input_bytes = receive(&mut input, wire::edit::input_limit(), started + BUDGET)?;
        let request = wire::edit::decode_request(&input_bytes, project, sid, path)
            .map_err(|_| protocol_failure())?
            .map_err(|_| protocol_failure())?;
        if request.request_id() != &request_id {
            return Err(protocol_failure());
        }
        Ok((request, PathBuf::from(registry), started))
    };
    let Ok((request, registry, started)) = receive_startup() else {
        return Some(1);
    };
    let id = request.request_id().clone();
    let deadline_at = started + BUDGET;
    let code = match run_worker(
        request,
        &registry,
        &mut input,
        &mut output,
        started,
        deadline_at,
    ) {
        Ok(()) => 0,
        Err(failure) => {
            if event(&mut output, &id, &Event::Failed(failure)).is_ok() {
                0
            } else {
                1
            }
        }
    };
    Some(code)
}
