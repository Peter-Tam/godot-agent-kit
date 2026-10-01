use super::*;
use crate::script_open::Observation;
#[derive(serde::Deserialize)]
struct Probe {
    kind: String,
}
enum Phase {
    Selected,
    State,
    InspectDisk {
        cache: codec::Cache,
        open: bool,
    },
    Prepare,
    PreparedDisk {
        context: Box<stock_validation::OpeningContext>,
        validation: Option<codec::Validation>,
    },
    Entry,
    Progress {
        step: &'static str,
    },
    Sample {
        purpose: &'static str,
    },
    VerifyDisk {
        purpose: &'static str,
    },
    Recheck {
        purpose: &'static str,
    },
    DiskCheck {
        purpose: &'static str,
        protection: Option<String>,
        identity: bool,
    },
    Done {
        purpose: &'static str,
        protection: Option<String>,
        identity: bool,
    },
}
fn send_bytes(
    socket: &mut UnixStream,
    bytes: &[u8],
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), Reason> {
    let header = (bytes.len() as u32).to_be_bytes();
    for part in [&header[..], bytes] {
        let mut offset = 0;
        while offset < part.len() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(Reason::Cancelled);
            }
            if clock.started.elapsed() >= BUDGET {
                return Err(Reason::Timeout);
            }
            match socket.write(&part[offset..]) {
                Ok(0) => return Err(Reason::Disconnected),
                Ok(n) => offset += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL_INTERVAL),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => return Err(Reason::Disconnected),
            }
        }
    }
    Ok(())
}
fn control(
    socket: &mut UnixStream,
    id: &RequestId,
    kind: &str,
    hash: &str,
    guard: &str,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), Reason> {
    let bytes = serde_json::to_vec(&(1, kind, id.as_str(), hash, guard))
        .map_err(|_| Reason::ProtocolError)?;
    if bytes.len() > CONTROL_LIMIT {
        return Err(Reason::ProtocolError);
    }
    send_bytes(socket, &bytes, clock, cancelled)
}
fn run_helper(
    socket: &UnixStream,
    request: stock_validation::ValidationRequest,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<stock_validation::ValidationResult, Reason> {
    let mut monitor = socket.try_clone().map_err(|_| Reason::ProtocolError)?;
    monitor
        .set_nonblocking(true)
        .map_err(|_| Reason::ProtocolError)?;
    let stop = AtomicBool::new(false);
    let interrupted = AtomicBool::new(false);
    let helper_cancel = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            let mut probe = [0; 1];
            while !stop.load(Ordering::Relaxed) {
                if cancelled.load(Ordering::Relaxed) {
                    helper_cancel.store(true, Ordering::Relaxed);
                    break;
                }
                match monitor.read(&mut probe) {
                    Ok(_) => {
                        interrupted.store(true, Ordering::Relaxed);
                        helper_cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        interrupted.store(true, Ordering::Relaxed);
                        helper_cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                thread::sleep(POLL_INTERVAL);
            }
        });
        let result = stock_validation::validate(request, clock, &helper_cancel);
        stop.store(true, Ordering::Relaxed);
        result
    });
    if cancelled.load(Ordering::Relaxed) {
        Err(Reason::Cancelled)
    } else if clock.started.elapsed() >= BUDGET {
        Err(Reason::Timeout)
    } else if interrupted.load(Ordering::Relaxed) {
        Err(Reason::Disconnected)
    } else {
        Ok(result)
    }
}
fn observation(
    collected: Collected,
    request: &ObservationRequest,
    clock: AttemptClock,
    acquisition: AttemptClock,
    purpose: &'static str,
) -> Result<Observation, Reason> {
    let interval = acquisition.interval();
    // Stamps keep their original operation-relative caller ticks and receipts.
    // Validate them against that clock, not the shorter acquisition interval.
    let value = collected
        .finish(request.clone(), clock)
        .map_err(|_| Reason::ProtocolError)?;
    let unavailable = match value.outcome() {
        OutcomeKind::MissingTarget => Reason::MissingScript,
        OutcomeKind::InvalidTarget => Reason::InvalidDocumentKind,
        OutcomeKind::InvalidRequest => Reason::InvalidRequest,
        OutcomeKind::DeniedAccess => Reason::DeniedAccess,
        OutcomeKind::AmbiguousTarget => Reason::AmbiguousSession,
        OutcomeKind::ProtocolError => Reason::ProtocolError,
        OutcomeKind::Timeout => Reason::Timeout,
        OutcomeKind::DisconnectedEditor => Reason::Disconnected,
        OutcomeKind::EditorUnavailable => Reason::EditorUnavailable,
        OutcomeKind::UnsupportedObservation => Reason::UnsupportedCapability,
        OutcomeKind::Cancelled => Reason::Cancelled,
        _ => Reason::EvidenceUnavailable,
    };
    let snapshot = value.into_snapshot().ok_or(unavailable)?;
    Ok(Observation {
        purpose,
        interval,
        snapshot,
    })
}
fn new_collection(
    target: &ResolvedTarget,
    sample: wire::EditorSample,
) -> Result<Collected, Reason> {
    let mut c = Collected::new();
    c.accept(Event::Selected(target.clone()))
        .map_err(|_| Reason::ProtocolError)?;
    c.accept(Event::Sample(Box::new(sample)))
        .map_err(|_| Reason::ProtocolError)?;
    Ok(c)
}
fn update_resource(
    attempt: &mut Attempt,
    r: &codec::Reply,
    script: Option<DecimalCounter>,
    entered: bool,
) {
    if ["inspected", "prepared", "ready", "observed", "unchanged"].contains(&r.status.as_str())
        || entered
    {
        if script.is_none()
            && r.edited.is_none()
            && (r.cache.as_ref().is_some_and(|c| c.state == "absent")
                || r.mode.as_deref() == Some("cold"))
        {
            attempt.no_resource();
        } else {
            attempt.resource(r.edited, r.collection.clone(), script);
        }
    } else if r.reason == Some(Reason::DirtyConflict) && r.edited == Some(true) {
        attempt.resource(r.edited, r.collection.clone(), script);
    }
}
fn accept_native(attempt: &mut Attempt, r: &codec::Reply) -> Result<(), Reason> {
    if let Some(n) = &r.native {
        if r.protection.is_some() && r.protection != n.protection {
            return Err(Reason::ProtocolError);
        }
        attempt.record(n.clone(), r.collection.clone())?;
        update_resource(attempt, r, n.script.clone(), false);
        if ["inspected", "observed", "unchanged"].contains(&r.status.as_str()) {
            if let Some(relation) = n.selection {
                attempt.selection(relation, false);
            }
        }
    }
    Ok(())
}
/// One checked invocation; no caller-controlled executable, effects or deadline.
/// # Errors
/// Returns a source-free host failure if the trusted worker cannot be launched.
pub fn run(
    request: OpenRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<OpeningOutcome, HostFailure> {
    let attempt = Attempt::new(request, clock.interval());
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= BUDGET {
        return Ok(attempt.finish(
            Some(if cancelled.load(Ordering::Relaxed) {
                Reason::Cancelled
            } else {
                Reason::Timeout
            }),
            Some(clock.interval()),
        ));
    }
    let (mut parent, child) = UnixStream::pair().map_err(|_| HostFailure)?;
    parent.set_nonblocking(true).map_err(|_| HostFailure)?;
    let read_fd: OwnedFd = child.try_clone().map_err(|_| HostFailure)?.into();
    let write_fd: OwnedFd = child.into();
    let worker = Command::new(std::env::current_exe().map_err(|_| HostFailure)?)
        .arg(INTERNAL_WORKER_FLAG)
        .stdin(Stdio::from(read_fd))
        .stdout(Stdio::from(write_fd))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| HostFailure)?;
    let worker = OwnedWorker(Some(worker));
    let r = attempt.request();
    let startup = serde_json::to_vec(&(
        4,
        r.request_id().as_str(),
        r.project_root().as_str(),
        r.session_id().map(SessionId::as_str),
        r.script_path().as_str(),
        registry.to_str().ok_or(HostFailure)?,
        clock.elapsed_us(),
    ))
    .map_err(|_| HostFailure)?;
    if startup.len() > CONTROL_LIMIT {
        return Ok(attempt.finish(Some(Reason::InvalidRequest), Some(clock.interval())));
    }
    if let Err(reason) = send_bytes(&mut parent, &startup, clock, cancelled) {
        return Ok(attempt.finish(Some(reason), Some(clock.interval())));
    }
    Ok(supervise(parent, worker, attempt, clock, cancelled))
}
fn supervise(
    mut socket: UnixStream,
    worker: OwnedWorker,
    mut attempt: Attempt,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> OpeningOutcome {
    let mut phase = Phase::Selected;
    let mut frames = Frames::default();
    let mut collected: Option<Collected> = None;
    let mut acquisition: Option<AttemptClock> = None;
    let mut original: Option<CollectionStamp> = None;
    let mut admitted: Option<(String, usize, FileIdentity, bool)> = None;
    let mut previous: Option<CollectionStamp> = None;
    let mut advertised: Option<String> = None;
    let mut expiry: Option<DecimalCounter> = None;
    let result = (|| -> Result<(), Reason> {
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err(Reason::Cancelled);
            }
            if clock.started.elapsed() >= BUDGET {
                return Err(Reason::Timeout);
            }
            let bytes = match frames.next(&mut socket) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    thread::sleep(POLL_INTERVAL);
                    continue;
                }
                Err(e) => return Err(reason(&e)),
            };
            if bytes.first() == Some(&b'[') {
                if bytes.len() > CONTROL_LIMIT {
                    return Err(Reason::ProtocolError);
                }
                let (v, kind, id, stage): (u32, String, String, String) =
                    serde_json::from_slice(&bytes).map_err(|_| Reason::ProtocolError)?;
                if v != 1 || id != attempt.request().request_id().as_str() {
                    return Err(Reason::ProtocolError);
                }
                if kind == "open_binding" && matches!(phase, Phase::State) && advertised.is_none() {
                    ProjectRoot::new(stage.clone()).map_err(|_| Reason::ProtocolError)?;
                    advertised = Some(stage);
                    acquisition = Some(AttemptClock::start());
                    continue;
                }
                if kind == "open_entering" && stage == "prepare" && matches!(phase, Phase::Prepare)
                {
                    attempt.begin_preparation()?;
                    acquisition = Some(AttemptClock::start());
                    continue;
                }
                if kind != "open_entering" || !matches!(phase, Phase::Entry) {
                    return Err(Reason::ProtocolError);
                }
                match stage.as_str() {
                    "bind" => {
                        attempt.enter("bind")?;
                        phase = Phase::Progress { step: "bind" };
                    }
                    "compile" => {
                        attempt.enter("compile")?;
                        phase = Phase::Progress { step: "compile" };
                    }
                    "open" => {
                        attempt.enter("open")?;
                        phase = Phase::Progress { step: "open" };
                    }
                    "verify:recognition" => {
                        attempt.start_verification("recognition")?;
                        acquisition = Some(AttemptClock::start());
                        phase = Phase::Sample {
                            purpose: "recognition",
                        };
                    }
                    "verify:post_open" => {
                        attempt.start_verification("post_open")?;
                        acquisition = Some(AttemptClock::start());
                        phase = Phase::Sample {
                            purpose: "post_open",
                        };
                    }
                    _ => return Err(Reason::ProtocolError),
                }
                continue;
            }
            let probe: Probe = serde_json::from_slice(&bytes).map_err(|_| Reason::ProtocolError)?;
            if probe.kind.starts_with("open_") {
                let kind = match (&phase, probe.kind.as_str()) {
                    (Phase::State, "open_state") => "open_state",
                    (Phase::Prepare, "open_prepared") => "open_prepared",
                    (Phase::Progress { .. }, "open_progress") => "open_progress",
                    (Phase::Sample { .. }, "open_sample") => "open_sample",
                    (Phase::Recheck { .. }, "open_rechecked") => "open_rechecked",
                    _ => return Err(Reason::ProtocolError),
                };
                let target = attempt.target().ok_or(Reason::ProtocolError)?.clone();
                let mut r = codec::decode_reply(
                    &bytes,
                    kind,
                    attempt.request(),
                    &target,
                    advertised.as_deref().ok_or(Reason::ProtocolError)?,
                    clock.elapsed_us(),
                    original.as_ref(),
                )
                .map_err(|e| reason(&e))?;
                if previous
                    .as_ref()
                    .is_some_and(|p| r.collection.started_tick_us() < p.finished_tick_us())
                {
                    return Err(Reason::ProtocolError);
                }
                previous = Some(r.collection.clone());
                if expiry.is_some() && r.expiry.is_some() && expiry != r.expiry {
                    return Err(Reason::ProtocolError);
                }
                if expiry.is_none() {
                    expiry = r.expiry.clone();
                }
                match &phase {
                    Phase::State => {
                        let native = r.native.as_ref();
                        if native.is_some_and(|n| {
                            n.stage.as_deref() != Some("inspected")
                                || n.mode.is_some()
                                || n.binding.as_deref() != Some("not_started")
                                || n.compilation.as_deref() != Some("not_started")
                                || n.document.as_deref() != Some("not_started")
                        }) {
                            return Err(Reason::ProtocolError);
                        }
                        accept_native(&mut attempt, &r)?;
                        if r.status != "inspected" {
                            return Err(r.reason.unwrap_or(Reason::EvidenceUnavailable));
                        }
                        let n = r.native.as_ref().ok_or(Reason::ProtocolError)?;
                        let open = n.open.ok_or(Reason::OpenStateUnknown)?;
                        if let Some(sample) = r.sample.take() {
                            if sample.document.open_state().value()
                                != Some(&if open {
                                    OpenState::Open
                                } else {
                                    OpenState::NotOpen
                                })
                            {
                                return Err(Reason::OpenStateUnknown);
                            }
                            if open {
                                let i =
                                    sample.document.identity().ok_or(Reason::OpenStateUnknown)?;
                                if i.script_instance_id() != n.script.as_ref()
                                    || i.editor_instance_id() != n.editor.as_ref()
                                    || i.buffer_instance_id() != n.buffer.as_ref()
                                {
                                    return Err(Reason::TargetChanged);
                                }
                            }
                            attempt.inspected(open, sample.document.identity().cloned())?;
                            collected = Some(new_collection(&target, sample)?);
                        }
                        phase = Phase::InspectDisk {
                            cache: r.cache.take().ok_or(Reason::EvidenceUnavailable)?,
                            open,
                        };
                    }
                    Phase::Prepare => {
                        if r.native.as_ref().is_some_and(|n| {
                            n.binding.as_deref() != Some("not_started")
                                || n.document.as_deref() != Some("not_started")
                        }) {
                            return Err(Reason::ProtocolError);
                        }
                        accept_native(&mut attempt, &r)?;
                        if r.status != "prepared" {
                            return Err(r.reason.unwrap_or(Reason::UnsafeEditorContext));
                        }
                        let (hash, length, file, cached) =
                            admitted.as_ref().ok_or(Reason::ProtocolError)?;
                        let n = r.native.as_ref().ok_or(Reason::ProtocolError)?;
                        if n.stage.as_deref() != Some("prepared")
                            || n.next.as_deref() != Some("bind")
                            || n.open != Some(false)
                            || n.compilation.as_deref()
                                != Some(if *cached {
                                    "not_applicable"
                                } else {
                                    "not_started"
                                })
                        {
                            return Err(Reason::ProtocolError);
                        }
                        if r.mode.as_deref() != Some(if *cached { "cached" } else { "cold" })
                            || r.file.as_ref()
                                != Some(&(target.project_file_id().clone(), file.clone()))
                        {
                            return Err(Reason::TargetChanged);
                        }
                        attempt.prepared(*cached, hash.clone(), *length, file.clone())?;
                        collected = Some(new_collection(
                            &target,
                            r.sample.take().ok_or(Reason::EvidenceUnavailable)?,
                        )?);
                        phase = Phase::PreparedDisk {
                            context: Box::new(
                                r.context
                                    .take()
                                    .ok_or(Reason::CurrentValidationUnavailable)?,
                            ),
                            validation: r.validation.take(),
                        };
                    }
                    Phase::Progress { step } => {
                        let n = r.native.take().ok_or(Reason::ProtocolError)?;
                        let entered = match *step {
                            "bind" => matches!(
                                n.binding.as_deref(),
                                Some("new_resource_published" | "reused_existing")
                            ),
                            "compile" => !matches!(
                                n.compilation.as_deref(),
                                Some("not_started" | "not_applicable")
                            ),
                            "open" => n.document.as_deref() != Some("not_started"),
                            _ => false,
                        };
                        let relation = n.selection;
                        let script = n.script.clone();
                        attempt.progress(n, r.collection.clone(), r.status == "ready")?;
                        update_resource(&mut attempt, &r, script, entered);
                        if r.status == "ready" || *step == "open" && entered {
                            if let Some(relation) = relation {
                                attempt.selection(relation, *step == "open");
                            }
                        }
                        if r.status != "ready" {
                            return Err(r.reason.unwrap_or(if *step == "compile" {
                                Reason::CompilationUnavailable
                            } else {
                                Reason::NativeFailure
                            }));
                        }
                        phase = Phase::Entry;
                    }
                    Phase::Sample { purpose } => {
                        accept_native(&mut attempt, &r)?;
                        if r.purpose.as_deref() != Some(*purpose) {
                            return Err(Reason::ProtocolError);
                        }
                        if r.status != "observed" {
                            return Err(r.reason.unwrap_or(Reason::VerificationIncomplete));
                        }
                        let sample = r.sample.take().ok_or(Reason::VerificationIncomplete)?;
                        original = Some(sample.collection.clone());
                        attempt.verification_sample(
                            &sample.document,
                            &sample.resource,
                            &sample.buffer,
                            &sample.dirty,
                        )?;
                        collected = Some(new_collection(&target, sample)?);
                        phase = Phase::VerifyDisk { purpose };
                    }
                    Phase::Recheck { purpose } => {
                        if r.purpose.as_deref() != Some(*purpose) {
                            return Err(Reason::ProtocolError);
                        }
                        let recheck = r.recheck.take().ok_or(Reason::VerificationIncomplete)?;
                        let c = collected.as_mut().ok_or(Reason::ProtocolError)?;
                        c.accept(Event::Rechecked(recheck))
                            .map_err(|_| Reason::ProtocolError)?;
                        if r.status != "unchanged" {
                            if let Some(reason) = r.reason {
                                attempt.fail(reason);
                            }
                        }
                        accept_native(&mut attempt, &r)?;
                        attempt.verification_recheck(&c.recheck);
                        if r.status != "unchanged" {
                            return Err(r.reason.unwrap_or(Reason::VerificationIncomplete));
                        }
                        phase = Phase::DiskCheck {
                            purpose,
                            protection: r.protection.take(),
                            identity: r.native.as_ref().and_then(|n| n.open) == Some(true),
                        };
                    }
                    _ => return Err(Reason::ProtocolError),
                }
                continue;
            }
            let event = wire::decode_event(
                &bytes,
                attempt.request(),
                attempt.target(),
                clock.elapsed_us(),
            )
            .map_err(|e| reason(&e))?;
            if let Event::Failed(e) = event {
                attempt.closed_worker_before_entry();
                return Err(
                    if matches!(phase, Phase::Selected)
                        && attempt.request().session_id().is_some()
                        && e.outcome == OutcomeKind::EditorUnavailable
                    {
                        Reason::SessionEnded
                    } else {
                        reason(&e)
                    },
                );
            }
            match (&mut phase, event) {
                (Phase::Selected, Event::Selected(target)) => {
                    attempt.selected(target)?;
                    phase = Phase::State;
                }
                (
                    Phase::InspectDisk { cache, open },
                    e @ (Event::Disk(_) | Event::DiskMetadata { .. }),
                ) => {
                    let Some(c) = collected.as_mut() else {
                        // A collector cannot supply a missing closed document.
                        // Still consume independently confined D so missing or
                        // denied source is not hidden by absent editor evidence.
                        return Err(match &e {
                            Event::Disk(source)
                                if !*open && source.reason() == Some(SourceReason::DiskMissing) =>
                            {
                                Reason::MissingScript
                            }
                            Event::DiskMetadata {
                                reason: SourceReason::DiskMissing,
                                ..
                            } if !*open => Reason::MissingScript,
                            _ => Reason::EvidenceUnavailable,
                        });
                    };
                    c.accept(e).map_err(|_| Reason::ProtocolError)?;
                    let value = observation(
                        collected.take().ok_or(Reason::ProtocolError)?,
                        attempt.request(),
                        clock,
                        acquisition.take().ok_or(Reason::ProtocolError)?,
                        "preparation",
                    )?;
                    let admission = (|| -> Result<_, Reason> {
                        if *open {
                            return Ok(None);
                        }
                        let d = value.snapshot.sources().disk();
                        let source =
                            d.text()
                                .ok_or(if d.reason() == Some(SourceReason::DiskMissing) {
                                    Reason::MissingScript
                                } else {
                                    Reason::EvidenceUnavailable
                                })?;
                        let file = d
                            .witness()
                            .and_then(Witness::disk_file_id)
                            .ok_or(Reason::EvidenceUnavailable)?
                            .clone();
                        let hash = crate::project_fs_validation::hex_sha256(source.as_bytes());
                        if cache.state == "unavailable" {
                            return Err(cache.reason.unwrap_or(Reason::EvidenceUnavailable));
                        }
                        let cached = cache.state == "present";
                        if cached {
                            let r = value.snapshot.sources().resource();
                            if cache.hash.is_none() || cache.length.is_none() || r.text().is_none()
                            {
                                return Err(Reason::EvidenceUnavailable);
                            }
                            if r.staleness()
                                .and_then(Staleness::evidence)
                                .is_some_and(|v| !v.is_empty())
                            {
                                return Err(Reason::StaleResource);
                            }
                            if cache.hash.as_deref() != Some(hash.as_str())
                                || cache.length != Some(source.len())
                                || r.text() != Some(source)
                            {
                                return Err(Reason::CachedSourceConflict);
                            }
                            if cache.script.as_ref()
                                != value
                                    .snapshot
                                    .document()
                                    .identity()
                                    .and_then(DocumentIdentity::script_instance_id)
                            {
                                return Err(Reason::TargetChanged);
                            }
                            match attempt.resource_value() {
                                Some(false) => {}
                                Some(true) => return Err(Reason::DirtyConflict),
                                None => return Err(Reason::EvidenceUnavailable),
                            }
                        }
                        Ok(Some((hash, source.len(), file, cached)))
                    })();
                    attempt.preparation_observation(value)?;
                    attempt.before_mode(if *open {
                        "open"
                    } else if cache.state == "present" {
                        "closed_cached"
                    } else {
                        "closed_uncached"
                    });
                    admitted = admission?;
                    phase = if *open { Phase::Entry } else { Phase::Prepare };
                }
                (Phase::PreparedDisk { .. }, e @ (Event::Disk(_) | Event::DiskMetadata { .. })) => {
                    collected
                        .as_mut()
                        .ok_or(Reason::ProtocolError)?
                        .accept(e)
                        .map_err(|_| Reason::ProtocolError)?;
                    let value = observation(
                        collected.take().ok_or(Reason::ProtocolError)?,
                        attempt.request(),
                        clock,
                        acquisition.take().ok_or(Reason::ProtocolError)?,
                        "preparation",
                    )?;
                    let (hash, length, file, cached) =
                        admitted.as_ref().ok_or(Reason::ProtocolError)?;
                    let d = value.snapshot.sources().disk();
                    let r = value.snapshot.sources().resource();
                    let doc = value.snapshot.document();
                    let invalid = if doc.open_state().value() != Some(&OpenState::NotOpen)
                        || d.witness().and_then(Witness::disk_file_id) != Some(file)
                    {
                        Some(Reason::TargetChanged)
                    } else if d.text().is_none_or(|s| {
                        s.len() != *length
                            || crate::project_fs_validation::hex_sha256(s.as_bytes()) != *hash
                    }) {
                        Some(Reason::SourceChanged)
                    } else if *cached
                        && doc
                            .identity()
                            .and_then(DocumentIdentity::script_instance_id)
                            != attempt.script_identity()
                    {
                        Some(Reason::TargetChanged)
                    } else if *cached && r.text().is_none() {
                        Some(Reason::EvidenceUnavailable)
                    } else if *cached && r.text() != d.text() {
                        Some(Reason::SourceChanged)
                    } else if *cached
                        && r.staleness()
                            .and_then(Staleness::evidence)
                            .is_some_and(|v| !v.is_empty())
                    {
                        Some(Reason::StaleResource)
                    } else if !*cached && r.availability() == Availability::Observed {
                        Some(Reason::TargetChanged)
                    } else if *cached && attempt.resource_value() != Some(false) {
                        Some(if attempt.resource_value() == Some(true) {
                            Reason::DirtyConflict
                        } else {
                            Reason::EvidenceUnavailable
                        })
                    } else {
                        None
                    };
                    attempt.preparation_observation(value)?;
                    if let Some(reason) = invalid {
                        return Err(reason);
                    }
                    let old = std::mem::replace(&mut phase, Phase::Entry);
                    let Phase::PreparedDisk {
                        context,
                        validation,
                        ..
                    } = old
                    else {
                        return Err(Reason::ProtocolError);
                    };
                    let hashes = if context.no_source() {
                        if validation.is_some() {
                            return Err(Reason::ProtocolError);
                        }
                        attempt.no_source_context();
                        (String::new(), String::new())
                    } else {
                        let binding = context
                            .checked_binding()
                            .ok_or(Reason::UnsafeEditorContext)?;
                        let v = validation.ok_or(Reason::CurrentValidationUnavailable)?;
                        let request = codec::validation_request(
                            *context,
                            v,
                            attempt.request(),
                            attempt.target().ok_or(Reason::ProtocolError)?,
                        )
                        .ok_or(Reason::UnsafeEditorContext)?;
                        attempt.context_validating()?;
                        let result = run_helper(&socket, request, clock, cancelled)?;
                        if result.status == "invalid" {
                            return Err(if binding.invalid_result(&result) {
                                Reason::CurrentParseInvalid
                            } else {
                                Reason::CurrentValidationUnavailable
                            });
                        }
                        let (source, guard) = binding
                            .validated_hashes(&result)
                            .ok_or(Reason::CurrentValidationUnavailable)?;
                        (source.to_owned(), guard.to_owned())
                    };
                    if cancelled.load(Ordering::Relaxed) {
                        return Err(Reason::Cancelled);
                    }
                    if clock.started.elapsed() >= BUDGET {
                        return Err(Reason::Timeout);
                    }
                    attempt.authorize()?;
                    control(
                        &mut socket,
                        attempt.request().request_id(),
                        "authorize_open",
                        &hashes.0,
                        &hashes.1,
                        clock,
                        cancelled,
                    )?;
                }
                (
                    Phase::VerifyDisk { purpose },
                    e @ (Event::Disk(_) | Event::DiskMetadata { .. }),
                ) => {
                    let c = collected.as_mut().ok_or(Reason::ProtocolError)?;
                    c.accept(e).map_err(|_| Reason::ProtocolError)?;
                    if let Some(source) = &c.disk {
                        attempt.verification_disk(
                            source,
                            c.disk_metadata.as_ref().and_then(|m| m.file.as_ref()),
                        );
                    }
                    phase = Phase::Recheck { purpose };
                }
                (
                    Phase::DiskCheck {
                        purpose,
                        protection,
                        identity,
                    },
                    e @ Event::DiskChecked(_),
                ) => {
                    let c = collected.as_mut().ok_or(Reason::ProtocolError)?;
                    c.accept(e).map_err(|_| Reason::ProtocolError)?;
                    attempt.verification_recheck(&c.recheck);
                    phase = Phase::Done {
                        purpose,
                        protection: protection.take(),
                        identity: *identity,
                    };
                }
                (
                    Phase::Done {
                        purpose,
                        protection,
                        identity,
                    },
                    Event::Done,
                ) => {
                    let value = observation(
                        collected.take().ok_or(Reason::ProtocolError)?,
                        attempt.request(),
                        clock,
                        acquisition.take().ok_or(Reason::ProtocolError)?,
                        if *purpose == "recognition" {
                            "recognition"
                        } else {
                            "verification"
                        },
                    )?;
                    attempt.verification(value, protection.as_deref(), *identity)?;
                    if clock.started.elapsed() >= BUDGET {
                        return Err(Reason::Timeout);
                    }
                    return Ok(());
                }
                _ => return Err(Reason::ProtocolError),
            }
        }
    })();
    if let Some(mut c) = collected {
        if let Err(r) = &result {
            c.fail(failure(*r));
        }
        let purpose = match phase {
            Phase::VerifyDisk { purpose }
            | Phase::Recheck { purpose }
            | Phase::DiskCheck { purpose, .. }
            | Phase::Done { purpose, .. } => {
                if purpose == "recognition" {
                    "recognition"
                } else {
                    "verification"
                }
            }
            _ => "preparation",
        };
        if let Some(acquisition) = acquisition {
            if let Ok(value) = observation(c, attempt.request(), clock, acquisition, purpose) {
                attempt.latest_observation(value);
            }
        }
    }
    if result.is_err() {
        let _ = control(
            &mut socket,
            attempt.request().request_id(),
            "abort",
            "",
            "",
            clock,
            cancelled,
        );
    }
    // Delivery never waits for an editor cleanup acknowledgment or an entered native call.
    drop(socket);
    drop(worker);
    attempt.finish(result.err(), Some(clock.interval()))
}
