use super::*;
use crate::target::SelectedSession;
type Failure = (&'static str, bool);
fn disclosure_failure(cause: Failure, current: Result<(), Failure>) -> Failure {
    (cause.0, cause.1 || current.err().is_some_and(|e| e.1))
}
fn disclose(request: &ObservationRequest, at: Instant) -> Result<(), Failure> {
    budget(at)?;
    confined::source(request.project_root(), request.script_path())
        .map(|_| ())
        .map_err(|e| {
            (
                "disclosure_unavailable",
                matches!(
                    e,
                    confined::CaptureError::Unsafe
                        | confined::CaptureError::Unreadable
                        | confined::CaptureError::Changed
                ),
            )
        })
}
fn disclose_selected(
    selected: &SelectedSession,
    request: &ObservationRequest,
    at: Instant,
) -> Result<(), Failure> {
    use std::os::unix::fs::MetadataExt;
    budget(at)?;
    let identity = selected.target().project_file_id();
    let metadata = std::fs::symlink_metadata(request.project_root().as_str())
        .map_err(|_| ("identity_changed", true))?;
    if identity.device().as_str().parse::<u64>().ok() != Some(metadata.dev())
        || identity.inode().as_str().parse::<u64>().ok() != Some(metadata.ino())
    {
        return Err(("identity_changed", true));
    }
    disclose(request, at)
}
fn transferred_elapsed(elapsed_us: u64, accepted_ms: u64, now_ms: u64) -> u64 {
    elapsed_us.max(
        now_ms
            .saturating_sub(accepted_ms)
            .saturating_add(1)
            .saturating_mul(1000),
    )
}
fn err(e: RoutingFailure) -> Failure {
    (failure(&e), e.outcome == OutcomeKind::DeniedAccess)
}
fn capture(request: &ObservationRequest) -> Result<confined::Captured, Failure> {
    confined::source(request.project_root(), request.script_path())
        .map_err(|e| {
            if matches!(
                e,
                confined::CaptureError::Unsafe | confined::CaptureError::Unreadable
            ) {
                ("denied_access", true)
            } else {
                ("disk_unavailable", false)
            }
        })?
        .ok_or(("missing_script", false))
}
fn budget(at: Instant) -> Result<u64, Failure> {
    let ms = at
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(9000) as u64;
    if ms == 0 {
        Err(("timeout", false))
    } else {
        Ok(ms)
    }
}
fn native(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    op: codec::Operation<'_>,
    started: Instant,
    at: Instant,
) -> Result<codec::Reply, Failure> {
    codec::exchange(selected, request, op, started, at).map_err(err)
}
fn admitted(reply: &codec::Reply, status: &str) -> Result<(), Failure> {
    if reply.status == status && reply.reason.is_none() {
        return Ok(());
    }
    let reason = match reply.reason.as_deref() {
        Some("slot_busy") => "slot_busy",
        Some("validator_settings_changed") => "validator_settings_changed",
        Some("target_open") => "lifecycle_changed",
        Some("target_unsaved") => "dirty_target",
        Some("resource_edited") => "dirty_resource",
        Some("resource_disk_divergent") => "divergent_resource",
        Some(
            "cache_unavailable"
            | "roster_unavailable"
            | "unsaved_roster_unavailable"
            | "resource_unavailable",
        ) => "unavailable_observation",
        Some("close_epoch_changed" | "close_epoch_unavailable") => "lifecycle_changed",
        Some("session_or_expiry_changed") => "session_or_expiry_changed",
        Some(
            "expected_revision_changed"
            | "expected_resource_changed"
            | "file_revision_changed"
            | "cache_branch_or_identity_changed",
        ) => "revision_mismatch",
        Some("namespace_changed") => "identity_changed",
        Some("save_would_change_source" | "save_profile_changed" | "save_profile_unavailable") => {
            "unsupported_save_profile"
        }
        Some("closed_context_unavailable" | "effective_or_compiled_profile_changed") => {
            "unsupported_context"
        }
        Some("denied_access") => "denied_access",
        _ => "native_refused",
    };
    Err((
        reason,
        matches!(reason, "denied_access" | "identity_changed"),
    ))
}
fn state_disk(state: &State, disk: &confined::Captured, path: &str) -> bool {
    state.valid(path) && FileRevision::capture(disk).as_ref() == Some(&state.file_revision)
}
fn inspect_inner(
    request: &ObservationRequest,
    registry: &Path,
    started: Instant,
    at: Instant,
) -> Result<(State, CaptureChanges), Failure> {
    let mut selected = target::resolve(request, registry, at).map_err(err)?;
    let work = (|| {
        if !selected.capabilities().edit_closed_gdscript {
            return Err(("unsupported_capability", false));
        }
        let before = capture(request)?;
        let a = native(
            &mut selected,
            request,
            codec::Operation::Inspect(budget(at)?),
            started,
            at,
        )?;
        admitted(&a, "inspected")?;
        let context = a.context.as_ref();
        let state = a.state.ok_or(("unavailable_closed_state", false))?;
        if !state.valid(request.script_path().as_str())
            || context.is_some_and(|c| {
                if state.resource.state == "present" {
                    state.resource.source.as_deref() != Some(c.source.as_str())
                } else {
                    confined::hex_sha256(c.source.as_bytes()) != state.file_revision.sha256
                        || c.source.len().to_string() != state.file_revision.utf8_bytes
                }
            })
        {
            return Err(("protocol_error", false));
        }
        // Inspection has no retained owner and closes its channel. A new authenticated
        // selection samples the same explicit session, never a convenient replacement.
        let mut selected2 = target::resolve(request, registry, at).map_err(err)?;
        if selected2.target().session_id() != selected.target().session_id() {
            return Err(("session_changed", false));
        }
        let after = capture(request)?;
        let b = native(
            &mut selected2,
            request,
            codec::Operation::Inspect(budget(at)?),
            started,
            at,
        )?;
        admitted(&b, "inspected")?;
        let next = b.state.ok_or(("unavailable_closed_state", false))?;
        if !state_disk(&next, &after, request.script_path().as_str())
            || a.collection.finished_tick_us() > b.collection.started_tick_us()
        {
            return Err(("revision_mismatch", false));
        }
        let changes = CaptureChanges {
            disk: state.file_revision != next.file_revision
                || FileRevision::capture(&before).as_ref() != Some(&state.file_revision),
            resource: state.resource != next.resource,
            lifecycle: state.close_epoch != next.close_epoch || state.lifecycle != next.lifecycle,
            context_unavailable: context.is_none() || b.context.is_none(),
            context: match (context, b.context.as_ref()) {
                (Some(a), Some(b)) => {
                    a.sha256 != b.sha256
                        || a.source != b.source
                        || a.validation.executable != b.validation.executable
                        || a.validation.warnings != b.validation.warnings
                        || a.validation.global_classes != b.validation.global_classes
                }
                _ => true,
            },
        };
        Ok((next, changes))
    })();
    work.map_err(|cause| disclosure_failure(cause, disclose_selected(&selected, request, at)))
}
fn inspect(
    request: &ObservationRequest,
    registry: &Path,
    started: Instant,
    at: Instant,
) -> Result<(State, CaptureChanges), Failure> {
    let result = inspect_inner(request, registry, started, at);
    let current = budget(at).and_then(|_| capture(request));
    match result {
        Err(cause) => Err(disclosure_failure(cause, current.map(|_| ()))),
        Ok((state, changes)) => {
            use std::os::unix::fs::MetadataExt;
            let root = std::fs::symlink_metadata(request.project_root().as_str())
                .map_err(|_| ("identity_changed", true))?;
            if state.project_device.parse::<u64>().ok() != Some(root.dev())
                || state.project_inode.parse::<u64>().ok() != Some(root.ino())
            {
                return Err(("identity_changed", true));
            }
            let disk = current?;
            if !state_disk(&state, &disk, request.script_path().as_str()) {
                return Err(("revision_mismatch", false));
            }
            Ok((state, changes))
        }
    }
}
fn current_context(selected: &SelectedSession, c: &codec::Context) -> Option<String> {
    let path = ResourcePath::new("res://project.godot").ok()?;
    let config = confined::capture(selected.target().project_root(), &path, 64 * 1024).ok()??;
    let warnings = serde_json::to_string(&c.validation.warnings).ok()?;
    let classes = serde_json::to_string(&c.validation.global_classes).ok()?;
    Some(confined::hex_sha256(
        format!("{}:{}:{}", config.sha256, warnings, classes).as_bytes(),
    ))
}
fn dependencies(selected: &SelectedSession, r: &stock_validation::ValidationResult) -> bool {
    !r.sources.is_empty()
        && r.sources.len() <= 33
        && r.sources.iter().skip(1).all(|s| {
            let Ok(p) = ResourcePath::new(s.path.clone()) else {
                return false;
            };
            confined::source(selected.target().project_root(), &p)
                .ok()
                .flatten()
                .is_some_and(|c| {
                    c.sha256 == s.sha256
                        && c.bytes == s.utf8_bytes
                        && Some(c.device) == s.device
                        && Some(c.inode) == s.inode
                })
        })
}
#[derive(Serialize)]
struct HelperFrame<'a> {
    kind: &'static str,
    role: &'static str,
    context: &'a codec::Context,
    source: Option<&'a str>,
}
fn preflight(
    io: (&mut UnixStream, &mut UnixStream),
    selected: &SelectedSession,
    request: &ObservationRequest,
    context: &codec::Context,
    desired: &str,
    at: Instant,
) -> Result<
    (
        stock_validation::ValidationResult,
        stock_validation::ValidationResult,
    ),
    Failure,
> {
    let (input, output) = io;
    send(
        output,
        &HelperFrame {
            kind: "Helper",
            role: "preflight",
            context,
            source: Some(desired),
        },
        at,
        None,
    )
    .map_err(|e| (e, false))?;
    let control: Control = receive(input, at).map_err(|e| (e, false))?;
    let Control::Preflight {
        original,
        desired: proposed,
    } = control
    else {
        return Err(("cancelled", false));
    };
    check_validation(
        &original,
        selected,
        request,
        context,
        &confined::hex_sha256(context.source.as_bytes()),
    )?;
    check_validation(
        &proposed,
        selected,
        request,
        context,
        &confined::hex_sha256(desired.as_bytes()),
    )?;
    Ok((*original, *proposed))
}
fn post_validation(
    io: (&mut UnixStream, &mut UnixStream),
    selected: &SelectedSession,
    request: &ObservationRequest,
    context: &codec::Context,
    at: Instant,
) -> Result<stock_validation::ValidationResult, Failure> {
    let (input, output) = io;
    send(
        output,
        &HelperFrame {
            kind: "Helper",
            role: "post",
            context,
            source: None,
        },
        at,
        None,
    )
    .map_err(|e| (e, false))?;
    let control: Control = receive(input, at).map_err(|e| (e, false))?;
    let Control::Validation { result } = control else {
        return Err(("cancelled", false));
    };
    check_validation(
        &result,
        selected,
        request,
        context,
        &confined::hex_sha256(context.source.as_bytes()),
    )?;
    Ok(*result)
}
fn check_validation(
    result: &stock_validation::ValidationResult,
    selected: &SelectedSession,
    request: &ObservationRequest,
    context: &codec::Context,
    expected: &str,
) -> Result<(), Failure> {
    if result.status != "valid" {
        return Err((
            if result.status == "invalid" {
                "parse_error"
            } else if result.reason.as_deref() == Some("cancelled") {
                "cancelled"
            } else if result.reason.as_deref() == Some("deadline") {
                "timeout"
            } else {
                "validation_unavailable"
            },
            matches!(
                result.reason.as_deref(),
                Some("denied_access" | "unsafe_project" | "unsafe_path" | "source_unreadable")
            ),
        ));
    }
    if !valid(
        result,
        request.request_id().as_str(),
        selected.target().session_id().as_str(),
        request.script_path().as_str(),
        expected,
    ) || current_context(selected, context).as_ref() != result.context_sha256.as_ref()
        || !dependencies(selected, result)
    {
        return Err(("validation_failed", false));
    }
    Ok(())
}
fn sample_inner(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    started: Instant,
    at: Instant,
) -> Result<(State, codec::Context), Failure> {
    let disk = capture(request)?;
    let a = native(
        selected,
        request,
        codec::Operation::Verify(purpose),
        started,
        at,
    )?;
    admitted(&a, "verified")?;
    let state = a.state.ok_or(("unavailable_closed_state", false))?;
    let context = a.context.ok_or(("validation_context_unavailable", false))?;
    if !state_disk(&state, &disk, request.script_path().as_str()) || context.source != disk.text {
        return Err(("revision_mismatch", false));
    }
    let b = native(
        selected,
        request,
        codec::Operation::Recheck(purpose),
        started,
        at,
    )?;
    admitted(&b, "rechecked")?;
    let after = capture(request)?;
    if b.state.as_ref() != Some(&state)
        || !state_disk(&state, &after, request.script_path().as_str())
        || b.context.as_ref().is_none_or(|c| {
            c.sha256 != context.sha256
                || c.source != context.source
                || c.validation.executable != context.validation.executable
                || c.validation.warnings != context.validation.warnings
                || c.validation.global_classes != context.validation.global_classes
        })
        || a.collection.finished_tick_us() > b.collection.started_tick_us()
    {
        return Err(("revision_mismatch", false));
    }
    Ok((state, context))
}
fn sample(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    started: Instant,
    at: Instant,
) -> Result<(State, codec::Context), Failure> {
    sample_inner(selected, request, purpose, started, at)
        .map_err(|cause| disclosure_failure(cause, disclose_selected(selected, request, at)))
}
fn edit(
    startup: Startup,
    request: &ObservationRequest,
    input: &mut UnixStream,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<(), Failure> {
    let basis = startup.basis.ok_or(("missing_basis", false))?;
    let replacement = crate::script_edit::ReplacementSource::new(
        startup.source.ok_or(("missing_source", false))?,
    )
    .map_err(|_| ("unsupported_representation", false))?;
    let desired = replacement.as_str();
    if !basis.state.valid(&basis.path)
        || basis.project != request.project_root().as_str()
        || basis.path != request.script_path().as_str()
        || Some(basis.session.as_str()) != request.session_id().map(|s| s.as_str())
    {
        return Err(("missing_basis", false));
    }
    let mut selected = target::resolve(request, &startup.registry, at).map_err(err)?;
    if !selected.capabilities().edit_closed_gdscript {
        return Err(("unsupported_capability", false));
    }
    let initial = capture(request)?;
    if FileRevision::capture(&initial).as_ref() != Some(&basis.state.file_revision) {
        return Err(("revision_mismatch", false));
    }
    let work = (|| {
        let prepared = native(
            &mut selected,
            request,
            codec::Operation::Prepare(&basis.state, desired, budget(at)?),
            started,
            at,
        )?;
        admitted(&prepared, "prepared")?;
        if prepared
            .state
            .as_ref()
            .is_none_or(|s| !basis.state.matches(s))
        {
            return Err(("revision_mismatch", false));
        }
        let context = prepared
            .context
            .ok_or(("validation_context_unavailable", false))?;
        if context.source != initial.text {
            return Err(("revision_mismatch", false));
        }
        drop(initial);
        let (original, desired_result) = preflight(
            (&mut *input, &mut *output),
            &selected,
            request,
            &context,
            desired,
            at,
        )?;
        let (guard, guard_context) = sample(&mut selected, request, "preflight", started, at)?;
        if !basis.state.matches(&guard)
            || guard_context.sha256 != context.sha256
            || current_context(&selected, &guard_context).as_ref()
                != desired_result.context_sha256.as_ref()
            || !dependencies(&selected, &original)
            || !dependencies(&selected, &desired_result)
        {
            return Err(("revision_mismatch", false));
        }
        let unchanged = desired == context.source;
        if !unchanged {
            send(output, &Message::Ready {}, at, None).map_err(|e| (e, false))?;
            match receive::<Control>(input, at).map_err(|e| (e, false))? {
                Control::Authorize {} => {}
                _ => return Err(("cancelled", false)),
            }
            let applied = native(
                &mut selected,
                request,
                codec::Operation::Apply(&confined::hex_sha256(desired.as_bytes()), &context.sha256),
                started,
                at,
            )?;
            if let Some(mut receipt) = applied.native {
                // The authenticated envelope's failure is causal even if the
                // effect receipt has no separate reason; never drop it.
                if receipt.reason.is_none() {
                    receipt.reason = applied.reason.clone();
                }
                send(output, &Message::Native { receipt }, at, None).map_err(|e| (e, false))?;
            } else {
                return Err(("application_unknown", false));
            }
            if applied.status != "applied" || applied.reason.is_some() {
                // Preserve a fresh safe survivor sample when the native terminal
                // owner allows it, but never treat it as successful completion.
                let survivor = native(
                    &mut selected,
                    request,
                    codec::Operation::Verify("post_change"),
                    started,
                    at,
                );
                let current = capture(request);
                let denied = current.as_ref().err().is_some_and(|e| e.1)
                    || survivor.as_ref().err().is_some_and(|e| e.1)
                    || survivor.as_ref().is_ok_and(|r| {
                        matches!(
                            r.reason.as_deref(),
                            Some("denied_access" | "namespace_changed")
                        )
                    })
                    || matches!(
                        applied.reason.as_deref(),
                        Some("denied_access" | "namespace_changed")
                    );
                if let (Ok(reply), Ok(disk)) = (survivor, current) {
                    if let Some(state) = reply.state {
                        if !denied && state_disk(&state, &disk, request.script_path().as_str()) {
                            send(output, &Message::Survivor { state }, at, None)
                                .map_err(|e| (e, denied))?;
                        }
                    }
                }
                return Err(("native_partial_application", denied));
            }
        }
        let purpose = if unchanged {
            "unchanged"
        } else {
            "post_change"
        };
        let (post, post_context) = sample(&mut selected, request, purpose, started, at)?;
        if post.file_revision.sha256 != confined::hex_sha256(desired.as_bytes())
            || post_context.source != desired
        {
            return Err(("source_changed", false));
        }
        let post_validation = post_validation(
            (&mut *input, &mut *output),
            &selected,
            request,
            &post_context,
            at,
        )?;
        let (final_state, final_context) = sample(&mut selected, request, purpose, started, at)?;
        let ctx = current_context(&selected, &final_context)
            .ok_or(("validation_context_unavailable", false))?;
        let current = final_state == post
            && final_context.sha256 == post_context.sha256
            && Some(&ctx) == post_validation.context_sha256.as_ref()
            && dependencies(&selected, &post_validation);
        let finished = native(
            &mut selected,
            request,
            codec::Operation::Finish,
            started,
            at,
        )?;
        admitted(&finished, "finished")?;
        send(
            output,
            &Message::Verified {
                state: final_state,
                context_sha256: ctx,
                valid: current,
            },
            at,
            None,
        )
        .map_err(|e| (e, false))?;
        Ok(())
    })();
    if let Err(cause) = work {
        let mut failure = disclosure_failure(cause, disclose_selected(&selected, request, at));
        let aborted = native(&mut selected, request, codec::Operation::Abort, started, at);
        failure.1 |= aborted.as_ref().err().is_some_and(|e| e.1)
            || aborted.as_ref().is_ok_and(|r| {
                matches!(
                    r.reason.as_deref(),
                    Some("denied_access" | "namespace_changed")
                )
            });
        return Err(disclosure_failure(
            failure,
            disclose_selected(&selected, request, at),
        ));
    }
    work
}
fn dispatch(acquisition: bool) -> Option<i32> {
    let mut input = UnixStream::from(io::stdin().as_fd().try_clone_to_owned().ok()?);
    let mut output = UnixStream::from(io::stdout().as_fd().try_clone_to_owned().ok()?);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let startup: Startup = match receive(&mut input, Instant::now() + EDIT_BUDGET) {
        Ok(s) => s,
        Err(_) => return Some(1),
    };
    if startup.v != 6
        || !matches!(startup.budget_us, 4_500_000 | 9_500_000)
        || startup.elapsed_us >= startup.budget_us
        || startup.registry.to_str().is_none_or(|s| s.len() > 1024)
        || !startup.registry.is_absolute()
    {
        return Some(1);
    }
    let request = match (
        RequestId::new(startup.id.clone()),
        ProjectRoot::new(startup.project.clone()),
        startup.session.clone().map(SessionId::new).transpose(),
        ResourcePath::new(startup.path.clone()),
    ) {
        (Ok(id), Ok(project), Ok(session), Ok(path)) => {
            ObservationRequest::new(id, project, session, path)
        }
        _ => return Some(1),
    };
    // The parent owns the original monotonic cutoff. Conservatively consume
    // launch/channel latency as well; a backward wall adjustment cannot renew
    // the elapsed startup budget.
    let elapsed = transferred_elapsed(
        startup.elapsed_us,
        startup.accepted_unix_ms,
        crate::runner::unix_ms(),
    );
    if elapsed >= startup.budget_us {
        return Some(1);
    }
    let now = Instant::now();
    let Some(started) = now.checked_sub(Duration::from_micros(elapsed)) else {
        return Some(1);
    };
    let at = started + Duration::from_micros(startup.budget_us);
    let result = if acquisition {
        inspect(&request, &startup.registry, started, at).and_then(|(state, changes)| {
            send(&mut output, &Message::Acquired { state, changes }, at, None)
                .map_err(|e| (e, false))
        })
    } else {
        edit(startup, &request, &mut input, &mut output, started, at)
    };
    match result {
        Ok(()) => Some(0),
        Err((reason, denied)) => {
            let _ = send(
                &mut output,
                &Message::Failed {
                    reason: reason.to_owned(),
                    denied,
                },
                at,
                None,
            );
            Some(0)
        }
    }
}
pub fn worker_main() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(INTERNAL_WORKER_FLAG))
        || args.next().is_some()
    {
        return None;
    }
    dispatch(false)
}
pub fn acquisition_worker_main() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(INTERNAL_ACQUISITION_WORKER_FLAG))
        || args.next().is_some()
    {
        return None;
    }
    dispatch(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disclosure_denial_is_sticky_without_replacing_cause() {
        assert_eq!(
            disclosure_failure(("revision_mismatch", false), Err(("denied_access", true))),
            ("revision_mismatch", true)
        );
        assert_eq!(
            disclosure_failure(("mtime_failure", false), Ok(())),
            ("mtime_failure", false)
        );
    }
    #[test]
    fn worker_transfer_consumes_launch_latency_without_renewal() {
        assert_eq!(transferred_elapsed(100_000, 1_000, 1_250), 251_000);
        assert_eq!(transferred_elapsed(500_000, 1_000, 1_100), 500_000);
        assert_eq!(transferred_elapsed(500_000, 1_000, 900), 500_000);
        assert!(transferred_elapsed(1, 1_000, 10_500) >= 9_500_000);
    }
    #[test]
    fn authorization_control_has_one_strict_private_shape() {
        for kind in ["Authorize", "Abort"] {
            let frame = format!(r#"{{"kind":"{kind}"}}"#);
            assert!(serde_json::from_str::<Control>(&frame).is_ok());
            let extra = format!(r#"{{"kind":"{kind}","source":"extra"}}"#);
            assert!(serde_json::from_str::<Control>(&extra).is_err());
        }
        assert!(serde_json::from_str::<Control>(r#"{"kind":"Authorize","kind":"Abort"}"#).is_err());
        assert!(serde_json::from_str::<Message>(r#"{"kind":"Ready","source":"extra"}"#).is_err());
    }
}
