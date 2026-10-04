use super::*;
fn launch(
    startup: &Startup,
    flag: &str,
    at: Instant,
    cancelled: &AtomicBool,
) -> Result<(UnixStream, OwnedWorker), HostFailure> {
    let (mut parent, child) = UnixStream::pair().map_err(|_| HostFailure)?;
    let input: OwnedFd = child.try_clone().map_err(|_| HostFailure)?.into();
    let output: OwnedFd = child.into();
    let process = Command::new(std::env::current_exe().map_err(|_| HostFailure)?)
        .arg(flag)
        .env_clear()
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(output))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| HostFailure)?;
    let owner = OwnedWorker(Some(process));
    parent.set_nonblocking(true).map_err(|_| HostFailure)?;
    send(&mut parent, startup, at, Some(cancelled)).map_err(|_| HostFailure)?;
    Ok((parent, owner))
}
fn startup(
    request: &ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    budget: Duration,
) -> Startup {
    Startup {
        v: 6,
        id: request.request_id().as_str().to_owned(),
        project: request.project_root().as_str().to_owned(),
        session: request.session_id().map(|s| s.as_str().to_owned()),
        path: request.script_path().as_str().to_owned(),
        registry: registry.to_path_buf(),
        accepted_unix_ms: clock.started_unix_ms,
        elapsed_us: clock.elapsed_us(),
        budget_us: budget.as_micros() as u64,
        basis: None,
        source: None,
    }
}
fn next(
    s: &mut UnixStream,
    frames: &mut Frames,
    at: Instant,
    cancelled: &AtomicBool,
) -> Result<Message, &'static str> {
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err("cancelled");
        }
        if Instant::now() >= at {
            return Err("timeout");
        }
        match frames.next_limited(s, LIMIT) {
            Ok(Some(b)) => return serde_json::from_slice(&b).map_err(|_| "protocol_error"),
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(_) => return Err("disconnected"),
        }
    }
}
pub(crate) fn acquire(
    request: &ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
    observation: &ObservationOutcome,
) -> Result<Acquisition, AcquisitionFailure> {
    acquire_until(
        request,
        registry,
        clock,
        cancelled,
        observation,
        READ_BUDGET,
    )
}
pub(crate) fn acquire_for_edit(
    request: &ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
    observation: &ObservationOutcome,
) -> Result<Acquisition, AcquisitionFailure> {
    acquire_until(
        request,
        registry,
        clock,
        cancelled,
        observation,
        EDIT_BUDGET,
    )
}
fn acquire_until(
    request: &ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
    observation: &ObservationOutcome,
    budget: Duration,
) -> Result<Acquisition, AcquisitionFailure> {
    let unavailable = |reason: &str, changes| Acquisition {
        observed: None,
        eligible: None,
        reason: Some(reason.to_owned()),
        changes,
        lifecycle_changed: false,
    };
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= budget {
        return Ok(unavailable(
            if cancelled.load(Ordering::Relaxed) {
                "cancelled"
            } else {
                "timeout"
            },
            vec![],
        ));
    }
    let Some(target) = observation.resolved_target() else {
        return Ok(unavailable("required_state_unavailable", vec![]));
    };
    let mut input = startup(request, registry, clock, budget);
    input.session = Some(target.session_id().as_str().to_owned());
    let (mut socket, _owner) = launch(
        &input,
        INTERNAL_ACQUISITION_WORKER_FLAG,
        clock.started + budget,
        cancelled,
    )
    .map_err(|_| AcquisitionFailure::Host)?;
    match next(
        &mut socket,
        &mut Frames::default(),
        clock.started + budget,
        cancelled,
    ) {
        Ok(Message::Acquired {
            state,
            changes: captured,
        }) => {
            let mut changes = Vec::new();
            let lifecycle_changed = captured.lifecycle || state.lifecycle != "closed";
            if captured.disk {
                changes.push(DetectedChange::Source(Authority::D));
            }
            if captured.resource {
                changes.push(DetectedChange::Source(Authority::R));
            }
            if let Some(s) = observation.snapshot() {
                if s.sources().disk().text().is_some_and(|d| {
                    confined::hex_sha256(d.as_bytes()) != state.file_revision.sha256
                }) {
                    changes.push(DetectedChange::Source(Authority::D));
                }
                if s.sources()
                    .disk()
                    .witness()
                    .and_then(|w| w.disk_file_id())
                    .is_some_and(|f| {
                        f.device().as_str() != state.file_revision.device
                            || f.inode().as_str() != state.file_revision.inode
                    })
                {
                    changes.push(DetectedChange::DiskIdentityReplaced);
                }
                if s.sources().resource().text() != state.resource.source.as_deref() {
                    changes.push(DetectedChange::Source(Authority::R));
                }
                if s.sources()
                    .resource()
                    .witness()
                    .and_then(|w| w.script_instance_id())
                    .map(|id| id.as_str())
                    != state.resource.instance_id.as_deref()
                {
                    changes.push(DetectedChange::Source(Authority::R));
                }
            }
            let eligible = if changes.is_empty() && !lifecycle_changed && !captured.context {
                ClosedExpectedBasis::from_observation(observation, &state)
            } else {
                None
            };
            let reason =
                if captured.context_unavailable {
                    Some("validation_context_unavailable")
                } else if !changes.is_empty() || captured.context {
                    Some("revision_mismatch")
                } else if lifecycle_changed {
                    Some("lifecycle_changed")
                } else if state.resource.edited == Some(true) {
                    Some("dirty_resource")
                } else if state.resource.source.as_ref().is_some_and(|r| {
                    confined::hex_sha256(r.as_bytes()) != state.file_revision.sha256
                }) {
                    Some("divergent_resource")
                } else if eligible.is_none() {
                    Some("closed_basis_unavailable")
                } else {
                    None
                };
            Ok(Acquisition {
                observed: Some(state),
                eligible,
                reason: reason.map(str::to_owned),
                changes,
                lifecycle_changed,
            })
        }
        Ok(Message::Failed { denied: true, .. }) => Err(AcquisitionFailure::DeniedAccess),
        Ok(Message::Failed { reason, .. }) => {
            let changes = match reason.as_str() {
                "session_changed" | "session_or_expiry_changed" => {
                    vec![DetectedChange::SessionReplaced]
                }
                "disconnected" | "editor_unavailable" => vec![DetectedChange::SessionEnded],
                "identity_changed" => vec![DetectedChange::DocumentIdentityReplaced],
                "unavailable_observation" => vec![DetectedChange::Source(Authority::R)],
                "revision_mismatch" | "validation_context_changed" => vec![
                    DetectedChange::Source(Authority::D),
                    DetectedChange::Source(Authority::R),
                ],
                _ => vec![],
            };
            let mut acquisition = unavailable(&reason, changes);
            acquisition.lifecycle_changed = matches!(
                reason.as_str(),
                "lifecycle_changed" | "unavailable_observation"
            );
            Ok(acquisition)
        }
        Err(reason) => Ok(unavailable(reason, vec![])),
        _ => Ok(unavailable("protocol_error", vec![])),
    }
}
fn preflight(
    original: stock_validation::ValidationRequest,
    desired: stock_validation::ValidationRequest,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<
    (
        stock_validation::ValidationResult,
        stock_validation::ValidationResult,
    ),
    &'static str,
> {
    std::thread::scope(|scope| {
        let original = std::thread::Builder::new()
            .name("closed-original-validation".into())
            .spawn_scoped(scope, || {
                stock_validation::validate(original, clock, cancelled)
            })
            .map_err(|_| "validation_unavailable")?;
        let desired = stock_validation::validate(desired, clock, cancelled);
        let original = original.join().map_err(|_| "validation_unavailable")?;
        Ok((original, desired))
    })
}

pub(crate) fn run(
    request: CheckedClosedRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<ClosedOutcome, HostFailure> {
    let mut effects = Effects::default();
    let present = request.basis.resource_present();
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= EDIT_BUDGET {
        let mut out = effects.outcome(
            false,
            false,
            if cancelled.load(Ordering::Relaxed) {
                "cancelled"
            } else {
                "timeout"
            },
            false,
            present,
        );
        out.request_id = Some(request.id.as_str().to_owned());
        let interval = clock.interval();
        out.interval = Some(
            serde_json::json!({"started_unix_ms":interval.started_unix_ms(),"finished_unix_ms":interval.finished_unix_ms(),"elapsed_us":interval.elapsed_us()}),
        );
        out.target = Some(
            serde_json::json!({"project_root":request.basis.project,"session_id":request.basis.session,"script_path":request.basis.path}),
        );
        return Ok(out);
    }
    let observation = request.observation().map_err(|_| HostFailure)?;
    let at = clock.started + EDIT_BUDGET;
    let session = observation.session_id().cloned().ok_or(HostFailure)?;
    let desired_hash = confined::hex_sha256(request.replacement.as_str().as_bytes());
    let unchanged = desired_hash == request.basis.state.file_revision.sha256
        && request.replacement.as_str().len().to_string()
            == request.basis.state.file_revision.utf8_bytes;
    let mut input = startup(&observation, registry, clock, EDIT_BUDGET);
    input.basis = Some(request.basis.clone());
    input.source = Some(request.replacement.as_str().to_owned());
    let (mut socket, _owner) = launch(&input, INTERNAL_WORKER_FLAG, at, cancelled)?;
    let mut frames = Frames::default();
    let mut helper_phase = 0;
    let mut original_valid = false;
    let mut desired_valid = false;
    let mut post_valid = false;
    let mut post_context = None;
    let mut final_state = None;
    let mut reason = "incomplete_verification".to_owned();
    let mut denied = false;
    let mut verified = false;
    loop {
        let message = match next(&mut socket, &mut frames, at, cancelled) {
            Ok(m) => m,
            Err(e) => {
                if reason == "incomplete_verification" {
                    reason = e.to_owned();
                }
                break;
            }
        };
        match message {
            Message::Helper {
                role,
                context,
                source,
            } => {
                let preflight = role == "preflight";
                if context.source.len() > SOURCE_LIMIT_BYTES
                    || if preflight {
                        helper_phase != 0
                            || effects.authorized
                            || source.as_deref() != Some(request.replacement.as_str())
                            || confined::hex_sha256(context.source.as_bytes())
                                != request.basis.state.file_revision.sha256
                    } else {
                        role != "post"
                            || helper_phase != 2
                            || !original_valid
                            || !desired_valid
                            || !(effects.authorized || unchanged)
                            || source.is_some()
                    }
                {
                    reason = "protocol_error".into();
                    break;
                }
                let make_request = |source, purpose, warnings, global_classes, official_binary| {
                    stock_validation::ValidationRequest {
                        request_id: request.id.clone(),
                        session_id: session.clone(),
                        project_root: observation.project_root().clone(),
                        root_path: observation.script_path().clone(),
                        source,
                        purpose,
                        open_context: None,
                        warnings,
                        global_classes,
                        official_binary,
                    }
                };
                let control = if preflight {
                    let original = make_request(
                        Some(context.source),
                        stock_validation::Purpose::Preflight,
                        context.validation.warnings.clone(),
                        context.validation.global_classes.clone(),
                        context.validation.executable.clone(),
                    );
                    let desired = make_request(
                        source,
                        stock_validation::Purpose::Preflight,
                        context.validation.warnings,
                        context.validation.global_classes,
                        context.validation.executable,
                    );
                    let (original, desired) =
                        match self::preflight(original, desired, clock, cancelled) {
                            Ok(results) => results,
                            Err(error) => {
                                reason = error.into();
                                break;
                            }
                        };
                    original_valid = valid(
                        &original,
                        request.id.as_str(),
                        &request.basis.session,
                        &request.basis.path,
                        &request.basis.state.file_revision.sha256,
                    );
                    desired_valid = valid(
                        &desired,
                        request.id.as_str(),
                        &request.basis.session,
                        &request.basis.path,
                        &desired_hash,
                    );
                    denied |= [&original, &desired].iter().any(|result| {
                        matches!(
                            result.reason.as_deref(),
                            Some(
                                "denied_access"
                                    | "unsafe_project"
                                    | "unsafe_path"
                                    | "source_unreadable"
                            )
                        )
                    });
                    helper_phase = 2;
                    Control::Preflight {
                        original: Box::new(original),
                        desired: Box::new(desired),
                    }
                } else {
                    let result = stock_validation::validate(
                        make_request(
                            None,
                            if unchanged {
                                stock_validation::Purpose::Unchanged
                            } else {
                                stock_validation::Purpose::PostChange
                            },
                            context.validation.warnings,
                            context.validation.global_classes,
                            context.validation.executable,
                        ),
                        clock,
                        cancelled,
                    );
                    post_valid = valid(
                        &result,
                        request.id.as_str(),
                        &request.basis.session,
                        &request.basis.path,
                        &desired_hash,
                    );
                    post_context = result.context_sha256.clone();
                    denied |= matches!(
                        result.reason.as_deref(),
                        Some(
                            "denied_access"
                                | "unsafe_project"
                                | "unsafe_path"
                                | "source_unreadable"
                        )
                    );
                    helper_phase = 3;
                    Control::Validation {
                        result: Box::new(result),
                    }
                };
                if send(&mut socket, &control, at, Some(cancelled)).is_err() {
                    reason = "disconnected".into();
                    break;
                }
            }
            Message::Ready {} => {
                if !original_valid || !desired_valid || effects.authorized || unchanged {
                    reason = "protocol_error".into();
                    break;
                }
                // Sticky potential application is recorded before any authorization bytes.
                effects.authorized = true;
                if send(&mut socket, &Control::Authorize {}, at, Some(cancelled)).is_err() {
                    reason = "disconnected".into();
                    break;
                }
            }
            Message::Native { receipt } => {
                if !effects.authorized
                    || receipt.request_id != request.id.as_str()
                    || receipt.session_id != request.basis.session
                    || receipt.script_path != request.basis.path
                {
                    reason = "protocol_error".into();
                    break;
                }
                effects.merge(&receipt);
                if let Some(r) = receipt.reason {
                    reason = r;
                }
            }
            Message::Survivor { state } => {
                if !effects.authorized
                    || !state.valid(&request.basis.path)
                    || state.project_device != request.basis.state.project_device
                    || state.project_inode != request.basis.state.project_inode
                {
                    reason = "protocol_error".into();
                    break;
                }
                final_state = Some(state);
            }
            Message::Verified {
                state,
                context_sha256,
                valid: current,
            } => {
                let before = &request.basis.state;
                let resource = if present {
                    state.resource.state == "present"
                        && state.resource.instance_id == before.resource.instance_id
                        && state.resource.path == before.resource.path
                        && state.resource.edited == Some(false)
                        && state.resource.profile_sha256 == before.resource.profile_sha256
                        && state.resource.source.as_deref() == Some(request.replacement.as_str())
                } else {
                    state.resource.state == "absent"
                };
                let receipt = unchanged
                    || effects.disk_entered
                        && effects.written_bytes == request.replacement.as_str().len() as u64
                        && effects.truncated
                        && effects.flushed
                        && effects.readback
                        && effects.mtime_restored
                        && (!present || effects.resource_entered);
                verified = current
                    && state.lifecycle == "closed"
                    && original_valid
                    && desired_valid
                    && post_valid
                    && post_context.as_deref() == Some(context_sha256.as_str())
                    && receipt
                    && state.valid(&request.basis.path)
                    && state.project_device == before.project_device
                    && state.project_inode == before.project_inode
                    && state.file_revision.device == before.file_revision.device
                    && state.file_revision.inode == before.file_revision.inode
                    && state.file_revision.sha256 == desired_hash
                    && state.file_revision.utf8_bytes
                        == request.replacement.as_str().len().to_string()
                    && state.file_revision.mtime == before.file_revision.mtime
                    && state.close_epoch == before.close_epoch
                    && resource
                    && (!unchanged || before.matches(&state));
                final_state = Some(state);
                if verified {
                    reason = "complete".into();
                }
                break;
            }
            Message::Failed {
                reason: r,
                denied: d,
            } => {
                denied |= d;
                if reason == "incomplete_verification" {
                    reason = r;
                }
                break;
            }
            _ => {
                reason = "protocol_error".into();
                break;
            }
        }
    }
    verified &= !denied;
    if verified && present && !unchanged {
        effects.resource_changed = true;
    }
    if !verified {
        let _ = send(&mut socket, &Control::Abort {}, at, None);
    }
    let mut outcome = effects.outcome(verified, unchanged, &reason, denied, present);
    outcome.request_id = Some(request.id.as_str().to_owned());
    let interval = clock.interval();
    outcome.interval = Some(
        serde_json::json!({"started_unix_ms":interval.started_unix_ms(),"finished_unix_ms":interval.finished_unix_ms(),"elapsed_us":interval.elapsed_us()}),
    );
    if !denied {
        outcome.target = Some(
            serde_json::json!({"project_root":request.basis.project,"session_id":request.basis.session,"script_path":request.basis.path}),
        );
        let summary = |s: &State| serde_json::json!({"disk":{"sha256":s.file_revision.sha256,"utf8_bytes":s.file_revision.utf8_bytes.parse::<usize>().ok()},"resource":{"state":s.resource.state,"edited":s.resource.edited,"sha256":s.resource.source.as_ref().map(|s|confined::hex_sha256(s.as_bytes())),"utf8_bytes":s.resource.source.as_ref().map(String::len)}});
        let before = &request.basis.state;
        let before_summary = serde_json::json!({"disk":{"sha256":before.file_revision.sha256,"utf8_bytes":before.file_revision.utf8_bytes.parse::<usize>().ok()},"resource":{"state":before.resource.state,"edited":before.resource.edited,"sha256":before.resource.source_sha256,"utf8_bytes":before.resource.utf8_bytes.as_deref().and_then(|s|s.parse::<usize>().ok())}});
        let applicability =
            final_state
                .as_ref()
                .map_or("unavailable", |s| match s.resource.state.as_str() {
                    "present" => "observed",
                    "absent" => "not_applicable",
                    _ => "unavailable",
                });
        outcome.evidence = Some(
            serde_json::json!({"before":before_summary,"after":final_state.as_ref().map(summary),"validation":{"original":original_valid,"desired":desired_valid,"actual":post_valid},"atomic":false,"independently_verified":verified,"buffer":if final_state.as_ref().is_some_and(|s|s.lifecycle=="closed") {"not_applicable"}else{"unavailable"},"resource":{"admitted":if present {"present"}else{"absent"},"availability":applicability,"state":final_state.as_ref().map(|s|s.resource.state.as_str())}}),
        );
        if let Some(state) = final_state.as_ref() {
            outcome.lifecycle.final_state = match state.lifecycle.as_str() {
                "closed" => "closed",
                "open" => "open",
                _ => "unknown",
            };
        }
    }
    Ok(outcome)
}
