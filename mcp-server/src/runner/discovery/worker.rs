use super::{codec, BUDGET, CONTROL_LIMIT, EVENT_LIMIT};
use crate::observation::{DiagnosticCode, OutcomeKind, ProjectRoot, RequestId, SessionId};
use crate::project_fs::discovery::{self as filesystem, Progress};
use crate::script_discovery::events::ContextStatus;
use crate::script_discovery::{
    Binding, Context, DiscoveryRequest, DiscoveryTarget, Event, Gap, Reason, Selection, Stage,
    Stamp,
};
use crate::target::{self, RoutingFailure, SelectedEditor};
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

fn reason(error: &RoutingFailure) -> Reason {
    match error.diagnostic.code() {
        DiagnosticCode::UnsafeRegistry => Reason::UnsafeRegistry,
        DiagnosticCode::AuthenticationFailed => Reason::AuthenticationFailed,
        DiagnosticCode::OutOfProject => Reason::OutOfProject,
        DiagnosticCode::IdentityChanged => Reason::ProjectIdentityChanged,
        DiagnosticCode::UnsupportedVersion => Reason::UnsupportedVersion,
        DiagnosticCode::UnsupportedCapability => Reason::UnsupportedCapability,
        DiagnosticCode::DeniedAccess => Reason::DeniedAccess,
        _ => match error.outcome {
            OutcomeKind::AmbiguousTarget => Reason::AmbiguousTarget,
            OutcomeKind::EditorUnavailable => Reason::EditorUnavailable,
            OutcomeKind::Timeout => Reason::Timeout,
            OutcomeKind::Cancelled => Reason::Cancelled,
            OutcomeKind::DisconnectedEditor => Reason::DisconnectedEditor,
            OutcomeKind::DeniedAccess => Reason::DeniedAccess,
            OutcomeKind::InvalidRequest => Reason::InvalidRequest,
            OutcomeKind::InvalidTarget | OutcomeKind::MissingTarget => Reason::InvalidProject,
            OutcomeKind::UnsupportedObservation => Reason::UnsupportedCapability,
            _ => Reason::ProtocolError,
        },
    }
}
fn routing(reason: Reason) -> RoutingFailure {
    let (outcome, code) = match reason {
        Reason::Timeout => (OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded),
        Reason::DisconnectedEditor => (
            OutcomeKind::DisconnectedEditor,
            DiagnosticCode::SessionEnded,
        ),
        Reason::ProjectIdentityChanged => {
            (OutcomeKind::DeniedAccess, DiagnosticCode::IdentityChanged)
        }
        Reason::DeniedAccess => (OutcomeKind::DeniedAccess, DiagnosticCode::DeniedAccess),
        Reason::OutOfProject => (OutcomeKind::DeniedAccess, DiagnosticCode::OutOfProject),
        _ => (OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame),
    };
    RoutingFailure::new(outcome, code, crate::observation::Stage::Finalize)
}
fn deadline(at: Instant) -> Result<Duration, Reason> {
    let remaining = at.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err(Reason::Timeout)
    } else {
        Ok(remaining)
    }
}
fn elapsed(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
fn receive(input: &mut impl Read, limit: usize) -> Result<Vec<u8>, Reason> {
    let mut header = [0; 4];
    input
        .read_exact(&mut header)
        .map_err(|_| Reason::DisconnectedEditor)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > limit {
        return Err(Reason::ProtocolError);
    }
    let mut bytes = vec![0; length];
    input
        .read_exact(&mut bytes)
        .map_err(|_| Reason::DisconnectedEditor)?;
    Ok(bytes)
}
fn send(output: &mut UnixStream, event: &Event, at: Instant) -> Result<(), Reason> {
    let bytes = codec::encode_event(event)?;
    if bytes.len() > EVENT_LIMIT {
        return Err(Reason::ProtocolError);
    }
    output
        .set_write_timeout(Some(deadline(at)?))
        .map_err(|_| Reason::ProtocolError)?;
    output
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| output.write_all(&bytes))
        .map_err(|_| Reason::ProtocolError)
}
fn control(
    selected: &mut SelectedEditor,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    kind: codec::Control,
    at: Instant,
) -> Result<Vec<u8>, Reason> {
    let remaining = deadline(at)?;
    let milliseconds = remaining.as_millis().min(4500) as u64;
    if kind == codec::Control::Begin && milliseconds == 0 {
        return Err(Reason::Timeout);
    }
    let bytes = codec::encode_control(
        kind,
        request,
        target,
        &selected.advertised_project_root,
        milliseconds,
    )?;
    selected
        .socket
        .set_write_timeout(Some(remaining))
        .map_err(|_| Reason::ProtocolError)?;
    selected
        .socket
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| selected.socket.write_all(&bytes))
        .map_err(|_| {
            if Instant::now() >= at {
                Reason::Timeout
            } else {
                Reason::DisconnectedEditor
            }
        })?;
    selected
        .socket
        .set_read_timeout(Some(deadline(at)?))
        .map_err(|_| Reason::ProtocolError)?;
    let bytes = receive(&mut selected.socket, CONTROL_LIMIT).map_err(|error| {
        if Instant::now() >= at {
            Reason::Timeout
        } else {
            error
        }
    })?;
    deadline(at)?;
    Ok(bytes)
}
fn context(
    selected: &mut SelectedEditor,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    kind: codec::Control,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<Context, Reason> {
    let bytes = control(selected, request, target, kind, at)?;
    let expected = if kind == codec::Control::Begin {
        "discover_state"
    } else {
        "discover_rechecked"
    };
    let decoded = codec::decode_context(
        &bytes,
        expected,
        request,
        target,
        &selected.advertised_project_root,
        elapsed(started),
    );
    if decoded.is_err()
        && codec::binding_changed(
            &bytes,
            expected,
            request,
            target,
            &selected.advertised_project_root,
        )
    {
        send(
            output,
            &Event::Invalidate {
                binding: Binding::target(target),
                reason: Reason::ProtocolError,
                prefix: None,
            },
            at,
        )?;
    }
    decoded
}
fn release(
    selected: &mut SelectedEditor,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    abort: bool,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<(), Reason> {
    let kind = if abort {
        codec::Control::Abort
    } else {
        codec::Control::Finish
    };
    let bytes = control(selected, request, target, kind, at)?;
    let decoded = codec::decode_ack(
        &bytes,
        kind,
        request,
        target,
        &selected.advertised_project_root,
        elapsed(started),
    );
    let expected = if abort {
        "discover_aborted"
    } else {
        "discover_finished"
    };
    if decoded.is_err()
        && codec::binding_changed(
            &bytes,
            expected,
            request,
            target,
            &selected.advertised_project_root,
        )
    {
        send(
            output,
            &Event::Invalidate {
                binding: Binding::target(target),
                reason: Reason::ProtocolError,
                prefix: None,
            },
            at,
        )?;
    }
    decoded?;
    Ok(())
}
fn gap_code(code: &str) -> Result<Reason, Reason> {
    Ok(match code {
        "directory_unreadable" => Reason::DirectoryUnreadable,
        "entry_unreadable" => Reason::EntryUnreadable,
        "unsafe_entry" => Reason::UnsafeEntry,
        "unsupported_path" => Reason::UnsupportedPath,
        "unsupported_marker" => Reason::UnsupportedMarker,
        "namespace_changed" => Reason::NamespaceChanged,
        "recheck_unavailable" => Reason::RecheckUnavailable,
        "entry_limit" => Reason::EntryLimit,
        "directory_limit" => Reason::DirectoryLimit,
        "depth_limit" => Reason::DepthLimit,
        "work_limit" => Reason::WorkLimit,
        _ => return Err(Reason::ProtocolError),
    })
}
fn acquire(
    selected: &mut SelectedEditor,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    output: &mut UnixStream,
    started: Instant,
    at: Instant,
) -> Result<(), Reason> {
    let binding = Binding::target(target);
    let begin = context(
        selected,
        request,
        target,
        codec::Control::Begin,
        output,
        started,
        at,
    )?;
    send(
        output,
        &Event::Scope {
            binding: binding.clone(),
            context: begin.clone(),
        },
        at,
    )?;
    if begin.status != ContextStatus::Observed {
        // No names were examined; unsupported/unavailable admission cannot be a disk fallback.
        if begin.expiry_tick_us.is_some() {
            release(selected, request, target, false, output, started, at)?;
        }
        return Ok(());
    }
    let facts = begin.context.as_ref().ok_or(Reason::ProtocolError)?;
    let mut ordinal = 0;
    let mut batch_started = elapsed(started);
    let collection = if facts.scanning || facts.importing {
        None
    } else {
        Some(
            filesystem::collect(selected, &facts.project_data_directory, at, |progress| {
                match progress {
                    Progress::Entries {
                        entries,
                        visited_entries,
                        visited_directories,
                    } => {
                        let finished = elapsed(started);
                        send(
                            output,
                            &Event::Entries {
                                binding: binding.clone(),
                                ordinal,
                                collection: Stamp::caller(batch_started, finished, finished),
                                entries: entries
                                    .into_iter()
                                    .map(|path| path.as_str().to_owned())
                                    .collect(),
                                visited_entries,
                                visited_directories,
                            },
                            at,
                        )
                        .map_err(routing)?;
                        ordinal += 1;
                        batch_started = elapsed(started);
                    }
                    Progress::Invalidate { scope } => {
                        send(
                            output,
                            &Event::Invalidate {
                                binding: binding.clone(),
                                reason: Reason::NamespaceChanged,
                                prefix: Some(scope),
                            },
                            at,
                        )
                        .map_err(routing)?;
                    }
                }
                Ok(())
            })
            .map_err(|error| reason(&error))?,
        )
    };
    let rechecked = context(
        selected,
        request,
        target,
        codec::Control::Recheck,
        output,
        started,
        at,
    )?;
    let (
        exhausted,
        namespace_checked,
        changed,
        visited_entries,
        visited_directories,
        gaps,
        omitted_gaps,
    ) = if let Some(collection) = collection {
        let gaps = collection
            .gaps
            .into_iter()
            .map(|gap| {
                Ok(Gap {
                    code: gap_code(gap.code)?,
                    stage: match gap.stage {
                        "enumerate" => Stage::Enumerate,
                        "recheck" => Stage::Recheck,
                        _ => return Err(Reason::ProtocolError),
                    },
                    scope: Some(gap.scope),
                })
            })
            .collect::<Result<Vec<_>, Reason>>()?;
        (
            collection.exhausted,
            collection.recheck_completed,
            collection.changed,
            collection.visited_entries,
            collection.visited_directories,
            gaps,
            collection.omitted_gaps,
        )
    } else {
        (false, false, false, 0, 0, Vec::new(), 0)
    };
    send(
        output,
        &Event::Rechecked {
            binding,
            context: rechecked,
            exhausted,
            namespace_checked,
            changed,
            visited_entries,
            visited_directories,
            gaps,
            omitted_gaps,
        },
        at,
    )?;
    release(selected, request, target, false, output, started, at)
}

/// The same executable's private worker rejects ordinary shell pipes/terminal FDs.
pub fn worker_main() -> Option<i32> {
    let mut input = UnixStream::from(io::stdin().as_fd().try_clone_to_owned().ok()?);
    let mut output = UnixStream::from(io::stdout().as_fd().try_clone_to_owned().ok()?);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    input.set_read_timeout(Some(BUDGET)).ok()?;
    let startup = receive(&mut input, CONTROL_LIMIT).ok()?;
    type Startup = (u32, String, String, Option<String>, String, u64);
    let (version, id, root, session, registry, prior_elapsed): Startup =
        serde_json::from_slice(&startup).ok()?;
    if version != 5
        || prior_elapsed >= 4_500_000
        || registry.len() > 1024
        || !Path::new(&registry).is_absolute()
        || registry.chars().any(char::is_control)
    {
        return Some(1);
    }
    let request = DiscoveryRequest::new(
        RequestId::new(id).ok()?,
        ProjectRoot::new(root).ok()?,
        session.map(SessionId::new).transpose().ok()?,
    );
    let now = Instant::now();
    let started = now.checked_sub(Duration::from_micros(prior_elapsed))?;
    let at = started + BUDGET;
    let mut binding = Binding::request(&request);
    let mut selection = None;
    let result = match target::resolve_project(
        request.request_id(),
        request.project_root(),
        request.session_id(),
        Path::new(&registry),
        at,
    ) {
        Ok(mut selected) => {
            if &selected.request_id != request.request_id() {
                Err(Reason::ProtocolError)
            } else {
                match DiscoveryTarget::for_request(
                    &request,
                    selected.project_root.clone(),
                    selected.project_file_id.clone(),
                    selected.session_id.clone(),
                    selected.godot_version.clone(),
                ) {
                    Ok(target) => {
                        let result = send(
                            &mut output,
                            &Event::Selected {
                                binding: binding.clone(),
                                target: target.clone(),
                                capability: selected.capabilities.discover_gdscripts,
                            },
                            at,
                        );
                        binding = Binding::target(&target);
                        if let Err(error) = result {
                            Err(error)
                        } else if !selected.capabilities.discover_gdscripts {
                            Ok(())
                        } else {
                            match acquire(
                                &mut selected,
                                &request,
                                &target,
                                &mut output,
                                started,
                                at,
                            ) {
                                Ok(()) => Ok(()),
                                Err(error) => {
                                    // Publish known denial/loss before any cleanup I/O. The parent
                                    // terminates on this failure and never waits for an addon ack.
                                    let published = send(
                                        &mut output,
                                        &Event::Failed {
                                            binding: binding.clone(),
                                            reason: error,
                                            selection: None,
                                        },
                                        at,
                                    );
                                    if published.is_ok() {
                                        let _ = release(
                                            &mut selected,
                                            &request,
                                            &target,
                                            true,
                                            &mut output,
                                            started,
                                            at,
                                        );
                                        Ok(())
                                    } else {
                                        Err(error)
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => Err(Reason::ProtocolError),
                }
            }
        }
        Err(error) => {
            selection = error.selection.as_ref().map(|value| Selection {
                candidate_sessions: value
                    .candidate_sessions()
                    .iter()
                    .map(|id| id.as_str().to_owned())
                    .collect(),
                missing_selector: "session_id".into(),
            });
            Err(reason(&error))
        }
    };
    if let Err(reason) = result {
        if send(
            &mut output,
            &Event::Failed {
                binding: binding.clone(),
                reason,
                selection,
            },
            at,
        )
        .is_err()
        {
            return Some(1);
        }
    }
    Some(if send(&mut output, &Event::Done { binding }, at).is_ok() {
        0
    } else {
        1
    })
}
