use super::*;
use crate::script_close as core;
#[path = "channel.rs"]
mod channel;
use channel::Channel;
fn refusal_reason(reply: &codec::Reply) -> &'static str {
    if reply
        .sample
        .as_ref()
        .is_some_and(|s| s.dirty.state() == Some(DirtyState::Dirty))
    {
        return "dirty_conflict";
    }
    if reply.edited == Some(true) {
        return "resource_edited_conflict";
    }
    let reason = reply.reason.as_deref().unwrap_or("");
    if let Some(reason) = machine_reason(Some(reason)) {
        return reason;
    }
    match reason {
        "slot_busy" | "native_owner_busy" => "busy",
        "invalid_capture_or_basis" | "target_not_clean_or_basis_changed" => "revision_changed",
        "target_identity_changed" | "target_namespace_changed" => "identity_changed",
        "target_disk_changed" => "source_changed",
        "session_or_expiry_changed" => "session_changed",
        "native_validation_pending"
        | "required_signal_unavailable"
        | "completion_attribution_unavailable" => "native_revalidation_unavailable",
        "close_error" | "close_return_unavailable" => "native_failure",
        "wrong_stage"
        | "wrong_stage_or_authorization"
        | "wrong_verification_purpose"
        | "invalid_receipt_binding"
        | "receipt_binding_mismatch"
        | "receipt_source_length_mismatch" => "protocol_error",
        "document_association_unavailable" | "selection_unavailable" => "evidence_unavailable",
        "idle_parse_delay_exceeds_lease" => "unsafe_editor_context",
        r if r.ends_with("_limit") => "context_limit",
        r if r.ends_with("_changed") => "context_changed",
        r if r.contains("diverg") => "source_divergence",
        _ => "unsafe_editor_context",
    }
}
fn machine_reason(value: Option<&str>) -> Option<&'static str> {
    const REASONS: &[&str] = &[
        "complete",
        "already_closed",
        "invalid_request",
        "missing_basis",
        "invalid_basis",
        "revision_changed",
        "identity_changed",
        "ambiguous_session",
        "session_ended",
        "session_changed",
        "editor_unavailable",
        "disconnected",
        "denied_access",
        "outside_project",
        "missing_script",
        "invalid_document_kind",
        "unsupported_representation",
        "unsupported_capability",
        "open_state_unknown",
        "dirty_conflict",
        "resource_edited_conflict",
        "source_divergence",
        "stale_resource",
        "stale_buffer",
        "evidence_unavailable",
        "unsafe_editor_context",
        "context_limit",
        "context_parse_invalid",
        "context_validation_unavailable",
        "target_changed",
        "source_changed",
        "context_changed",
        "busy",
        "protocol_error",
        "native_failure",
        "native_revalidation_unavailable",
        "verification_incomplete",
        "verification_failed",
        "timeout",
        "cancelled",
    ];
    REASONS.iter().copied().find(|r| Some(*r) == value)
}
#[derive(Clone)]
struct DiskFacts {
    source: SourceObservation,
    file: FileIdentity,
    metadata: Option<(SourceReason, CollectionStamp)>,
}
impl std::ops::Deref for DiskFacts {
    type Target = SourceObservation;
    fn deref(&self) -> &SourceObservation {
        &self.source
    }
}
fn disk(
    channel: &mut Channel,
    request: &CloseRequest,
    target: &ResolvedTarget,
    cancelled: &AtomicBool,
) -> Result<DiskFacts, &'static str> {
    match channel.event(request, Some(target), cancelled)? {
        Event::Disk(source) => {
            let file = source
                .witness()
                .and_then(Witness::disk_file_id)
                .cloned()
                .ok_or("evidence_unavailable")?;
            Ok(DiskFacts {
                source,
                file,
                metadata: None,
            })
        }
        Event::DiskMetadata {
            reason,
            file,
            collection,
        } => {
            if reason == SourceReason::DiskMissing {
                return Err("missing_script");
            }
            let file = file.ok_or("evidence_unavailable")?;
            let source = SourceObservation::unavailable(Authority::D, reason)
                .map_err(|_| "protocol_error")?;
            Ok(DiskFacts {
                source,
                file,
                metadata: Some((reason, collection)),
            })
        }
        _ => Err("protocol_error"),
    }
}
fn binding(
    request: &CloseRequest,
    target: &ResolvedTarget,
    disk: &DiskFacts,
) -> Result<(), &'static str> {
    let file = &disk.file;
    if let Some(expected) = request.expected() {
        if expected.target().project_root() != target.project_root()
            || expected.target().project_file_id() != target.project_file_id()
            || expected.target().session_id() != target.session_id()
        {
            return Err("session_changed");
        }
        if expected.document().disk_file_id() != Some(file) {
            return Err("identity_changed");
        }
    }
    Ok(())
}
fn eligible(
    request: &CloseRequest,
    target: &ResolvedTarget,
    sample: &wire::EditorSample,
    disk: &DiskFacts,
    edited: Option<bool>,
) -> Result<(), &'static str> {
    binding(request, target, disk)?;
    let expected = request.expected().ok_or("missing_basis")?;
    let identity = sample.document.identity().ok_or("evidence_unavailable")?;
    let admitted = expected.document();
    if identity.kind() != admitted.kind()
        || identity.resource_path() != admitted.resource_path()
        || identity.script_instance_id() != admitted.script_instance_id()
        || identity.editor_instance_id() != admitted.editor_instance_id()
        || identity.buffer_instance_id() != admitted.buffer_instance_id()
        || identity
            .disk_file_id()
            .is_some_and(|file| Some(file) != admitted.disk_file_id())
    {
        return Err("identity_changed");
    }
    if sample.dirty.state() != Some(DirtyState::Clean) {
        return Err(if sample.dirty.state() == Some(DirtyState::Dirty) {
            "dirty_conflict"
        } else {
            "evidence_unavailable"
        });
    }
    if edited != Some(false) {
        return Err(if edited == Some(true) {
            "resource_edited_conflict"
        } else {
            "evidence_unavailable"
        });
    }
    if sample.buffer.witness().and_then(Witness::source_version) != Some(expected.current_version())
    {
        return Err("revision_changed");
    }
    if sample
        .resource
        .staleness()
        .is_some_and(|s| s.evidence().is_some())
    {
        return Err("stale_resource");
    }
    if sample
        .buffer
        .staleness()
        .is_some_and(|s| s.evidence().is_some())
    {
        return Err("stale_buffer");
    }
    let source = disk.text().ok_or("evidence_unavailable")?;
    if source.chars().any(|c| {
        matches!(c, '\0' | '\r' | '\u{feff}') || (c.is_control() && !matches!(c, '\n' | '\t'))
    }) {
        return Err("unsupported_representation");
    }
    if sample.resource.text() != Some(source) || sample.buffer.text() != Some(source) {
        return Err("source_divergence");
    }
    if crate::script_edit::SourceDigest::of(source) != *expected.source() {
        return Err("revision_changed");
    }
    Ok(())
}
fn validate_context(
    channel: &Channel,
    context: stock_validation::CloseContext,
    guard: &str,
    request: stock_validation::ValidationRequest,
    cancelled: &AtomicBool,
) -> Result<Value, &'static str> {
    let mut monitor = channel.socket.try_clone().map_err(|_| "protocol_error")?;
    let helper_cancel = AtomicBool::new(false);
    let stop = AtomicBool::new(false);
    let lost = AtomicBool::new(false);
    let remaining = (channel.clock.started + BUDGET)
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(9000) as u64;
    let result = thread::scope(|scope| {
        scope.spawn(|| {
            let mut probe = [0u8; 1];
            while !stop.load(Ordering::Relaxed) {
                if cancelled.load(Ordering::Relaxed) {
                    helper_cancel.store(true, Ordering::Relaxed);
                    break;
                }
                match monitor.read(&mut probe) {
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    _ => {
                        lost.store(true, Ordering::Relaxed);
                        helper_cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                thread::sleep(POLL_INTERVAL);
            }
        });
        let result = stock_validation::validate_close_context_owned(
            context,
            guard,
            request,
            remaining,
            channel.clock,
            &helper_cancel,
        );
        stop.store(true, Ordering::Relaxed);
        result
    });
    if cancelled.load(Ordering::Relaxed) {
        return Err("cancelled");
    }
    if channel.clock.started.elapsed() >= BUDGET {
        return Err("timeout");
    }
    if lost.load(Ordering::Relaxed) {
        return Err("disconnected");
    }
    serde_json::to_value(result).map_err(|_| "protocol_error")
}
fn snapshot(
    request: &CloseRequest,
    target: &ResolvedTarget,
    sample: wire::EditorSample,
    disk: DiskFacts,
    clock: AttemptClock,
) -> Result<ObservationSnapshot, &'static str> {
    let mut collected = Collected::new();
    let disk = if let Some((reason, collection)) = disk.metadata {
        Event::DiskMetadata {
            reason,
            file: Some(disk.file),
            collection,
        }
    } else {
        Event::Disk(disk.source)
    };
    for event in [
        Event::Selected(target.clone()),
        Event::Sample(Box::new(sample)),
        disk,
    ] {
        collected.accept(event).map_err(|_| "protocol_error")?;
    }
    collected
        .finish(request.observation().clone(), clock)
        .map_err(|_| "protocol_error")?
        .into_snapshot()
        .ok_or("evidence_unavailable")
}
fn prepare_arguments(
    request: &CloseRequest,
    target: &ResolvedTarget,
    disk: &SourceObservation,
) -> Result<Value, &'static str> {
    let e = request.expected().ok_or("missing_basis")?;
    let d = e.document();
    let file = disk
        .witness()
        .and_then(Witness::disk_file_id)
        .ok_or("evidence_unavailable")?;
    let source = disk.text().ok_or("evidence_unavailable")?;
    Ok(
        json!({"project_device":target.project_file_id().device().as_str(),"project_inode":target.project_file_id().inode().as_str(),"target_device":file.device().as_str(),"target_inode":file.inode().as_str(),"source_sha256":crate::project_fs_validation::hex_sha256(source.as_bytes()),"utf8_bytes":source.len(),"basis":{"script_instance_id":d.script_instance_id().ok_or("invalid_basis")?.as_str(),"editor_instance_id":d.editor_instance_id().ok_or("invalid_basis")?.as_str(),"buffer_instance_id":d.buffer_instance_id().ok_or("invalid_basis")?.as_str(),"current_version":e.current_version().as_str()}}),
    )
}
/// Runs the actual same-executable acquisition worker; parent alone authorizes and reduces.
pub fn run(
    request: CloseRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<ClosingOutcome, HostFailure> {
    let mut attempt = Attempt::new(request, clock.interval());
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= BUDGET {
        attempt.fail(if cancelled.load(Ordering::Relaxed) {
            "cancelled"
        } else {
            "timeout"
        });
        return Ok(attempt.finish(clock.interval()));
    }
    let (parent, child) = UnixStream::pair().map_err(|_| HostFailure)?;
    parent.set_nonblocking(true).map_err(|_| HostFailure)?;
    let read_fd: OwnedFd = child.try_clone().map_err(|_| HostFailure)?.into();
    let write_fd: OwnedFd = child.into();
    let worker = OwnedWorker(Some(
        Command::new(std::env::current_exe().map_err(|_| HostFailure)?)
            .arg(INTERNAL_WORKER_FLAG)
            .stdin(Stdio::from(read_fd))
            .stdout(Stdio::from(write_fd))
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| HostFailure)?,
    ));
    let mut channel = Channel::new(parent, clock);
    let result = (|| -> Result<(), &'static str> {
        let r = &attempt.request;
        let startup = serde_json::to_vec(&(
            6,
            r.request_id().as_str(),
            r.project_root().as_str(),
            r.session_id().map(SessionId::as_str),
            r.script_path().as_str(),
            registry.to_str().ok_or("invalid_request")?,
            clock.elapsed_us(),
        ))
        .map_err(|_| "protocol_error")?;
        if startup.len() > 4096 {
            return Err("invalid_request");
        }
        channel.send(&startup, cancelled)?;
        let target = match channel.event(&attempt.request, None, cancelled)? {
            Event::Selected(t) => t,
            _ => return Err("protocol_error"),
        };
        attempt.stage("selected");
        attempt.result.target = Some(target.clone());
        let bytes = channel.next(cancelled)?;
        let (v, kind, id, root): (u32, String, String, String) =
            serde_json::from_slice(&bytes).map_err(|_| "protocol_error")?;
        if v != 1 || kind != "close_binding" || id != attempt.request.request_id().as_str() {
            return Err("protocol_error");
        }
        ProjectRoot::new(root.clone()).map_err(|_| "protocol_error")?;
        let mut state =
            channel.reply(&mut attempt, &target, &root, "close_state", None, cancelled)?;
        attempt.stage("inspected");
        let initial_disk = disk(&mut channel, &attempt.request, &target, cancelled)?;
        binding(&attempt.request, &target, &initial_disk)?;
        let sample = state.sample.take().ok_or("evidence_unavailable")?;
        let recognition = match sample.document.open_state().value() {
            Some(OpenState::NotOpen) => true,
            Some(OpenState::Open) => false,
            None => return Err("open_state_unknown"),
        };
        if let Some(expected) = attempt.request.expected() {
            attempt.result.expected = Some(core::Expected {
                basis: expected.clone(),
                use_: if recognition {
                    "not_applied_to_closed_state"
                } else {
                    "matched"
                },
            });
        }
        let eligibility = if recognition {
            Ok(())
        } else {
            eligible(
                &attempt.request,
                &target,
                &sample,
                &initial_disk,
                state.edited,
            )
        };
        let current_version = sample
            .buffer
            .witness()
            .and_then(Witness::source_version)
            .cloned();
        let before = snapshot(
            &attempt.request,
            &target,
            sample,
            initial_disk.clone(),
            clock,
        )?;
        if before.document().validity().value() != Some(&Validity::Valid) {
            return Err("invalid_document_kind");
        }
        let summary = core::SnapshotSummary::of(&before);
        attempt.result.before = Some(core::Before {
            snapshot: summary.clone(),
            resource_edited: state.edited,
            edited_collection: Some(state.collection.clone()),
            current_version,
            saved_version: None,
        });
        attempt.result.observation = Some(core::ObservationSummary {
            purpose: "preparation",
            interval: clock.interval(),
            snapshot: summary,
        });
        if recognition {
            channel.control(&attempt.request, "recognize_close", &[], cancelled)?;
        } else {
            if let Err(reason) = eligibility {
                if let Some(expected) = &mut attempt.result.expected {
                    expected.use_ = "mismatched";
                }
                return Err(reason);
            }
            let arguments = prepare_arguments(&attempt.request, &target, &initial_disk)?;
            channel.control(&attempt.request, "prepare_close", &[arguments], cancelled)?;
            let mut prepared = channel.reply(
                &mut attempt,
                &target,
                &root,
                "close_prepared",
                None,
                cancelled,
            )?;
            attempt.stage("prepared");
            let prepared_disk = disk(&mut channel, &attempt.request, &target, cancelled)?;
            let prepared_sample = prepared.sample.take().ok_or("evidence_unavailable")?;
            eligible(
                &attempt.request,
                &target,
                &prepared_sample,
                &prepared_disk,
                prepared
                    .target_guard
                    .as_ref()
                    .and_then(|g| g["resource_edited"].as_bool()),
            )?;
            let original = prepared_sample.collection.clone();
            let context = prepared.context.take().ok_or("unsafe_editor_context")?;
            let guard = prepared.guard.take().ok_or("protocol_error")?;
            if !context.target_matches(
                attempt.request.expected().ok_or("missing_basis")?,
                &target,
                &prepared_disk,
            ) {
                return Err("target_changed");
            }
            if !context
                .target_guard_matches(prepared.target_guard.as_ref().ok_or("protocol_error")?)
            {
                return Err("protocol_error");
            }
            let target_guard = prepared.target_guard.as_ref().ok_or("protocol_error")?;
            if let Some(before) = &mut attempt.result.before {
                before.saved_version = Some(
                    DecimalCounter::new(
                        target_guard["saved_version"]
                            .as_str()
                            .ok_or("protocol_error")?,
                    )
                    .map_err(|_| "protocol_error")?,
                );
                before.edited_collection = Some(prepared.collection.clone());
            }
            let count = context.protected_count();
            let mut validators = prepared
                .validation
                .take()
                .ok_or("context_validation_unavailable")?;
            if validators.len() != count {
                return Err("protocol_error");
            }
            for (identity, validation) in context.admitted_documents().zip(&validators) {
                if identity
                    != (
                        validation.document_path.as_str(),
                        validation.script_id.as_str(),
                        validation.editor_id.as_str(),
                        validation.buffer_id.as_str(),
                    )
                {
                    return Err("protocol_error");
                }
            }
            channel.admit(
                attempt
                    .request
                    .expected()
                    .ok_or("missing_basis")?
                    .document()
                    .clone(),
                context
                    .admitted_documents()
                    .map(|(_, _, editor, _)| editor.to_owned())
                    .collect(),
                &prepared,
            )?;
            if let Some(first) = validators.first() {
                if validators.iter().any(|v| {
                    v.executable != first.executable
                        || v.warnings != first.warnings
                        || v.global_classes != first.global_classes
                }) {
                    return Err("context_changed");
                }
            }
            let environment = if count == 0 {
                stock_validation::ValidationRequest {
                    request_id: attempt.request.request_id().clone(),
                    session_id: target.session_id().clone(),
                    project_root: target.project_root().clone(),
                    root_path: attempt.request.script_path().clone(),
                    source: None,
                    purpose: stock_validation::Purpose::CloseContext,
                    open_context: None,
                    warnings: stock_validation::WarningSettings {
                        enable: false,
                        levels: Default::default(),
                        directory_rules: Default::default(),
                        provenance: stock_validation::WarningProvenance {
                            source: stock_validation::WarningOrigin::EditorProjectSettings,
                            project_root: target.project_root().as_str().into(),
                            session_id: target.session_id().as_str().into(),
                        },
                    },
                    global_classes: Vec::new(),
                    official_binary: std::env::current_exe()
                        .map_err(|_| "context_validation_unavailable")?,
                }
            } else {
                let v = validators.remove(0);
                stock_validation::ValidationRequest {
                    request_id: attempt.request.request_id().clone(),
                    session_id: target.session_id().clone(),
                    project_root: target.project_root().clone(),
                    root_path: attempt.request.script_path().clone(),
                    source: None,
                    purpose: stock_validation::Purpose::CloseContext,
                    open_context: None,
                    warnings: v.warnings,
                    global_classes: v.global_classes,
                    official_binary: v.executable.into(),
                }
            };
            let mut validation =
                validate_context(&channel, context, &guard, environment, cancelled)?;
            if validation["status"] != "valid" {
                return Err(if validation["status"] == "invalid" {
                    "context_parse_invalid"
                } else {
                    "context_validation_unavailable"
                });
            }
            attempt.stage("validated");
            channel.control(&attempt.request, "recheck_close", &[], cancelled)?;
            let checked = channel.reply(
                &mut attempt,
                &target,
                &root,
                "close_rechecked",
                Some(&original),
                cancelled,
            )?;
            if checked.purpose.as_deref() != Some("pre_close") {
                return Err("protocol_error");
            }
            if !matches!(checked.recheck,Some(Recheck::Performed{ref detected_changes}) if detected_changes.is_empty())
            {
                return Err("context_changed");
            }
            match channel.event(&attempt.request, Some(&target), cancelled)? {
                Event::DiskChecked(changes) if changes.is_empty() => {}
                _ => return Err("source_changed"),
            }
            let receipts = validation["receipt_bindings"].take();
            let args = [json!(guard), receipts];
            let control=serde_json::to_vec(&json!({"v":1,"kind":"authorize_close","request_id":attempt.request.request_id().as_str(),"arguments":args})).map_err(|_|"protocol_error")?;
            let native_tuple = json!([
                6,
                "close_advance",
                attempt.request.request_id().as_str(),
                target.session_id().as_str(),
                root,
                attempt.request.script_path().as_str(),
                args[0],
                args[1]
            ]);
            if control.len() > 4096
                || serde_json::to_vec(&native_tuple)
                    .map_err(|_| "protocol_error")?
                    .len()
                    > 4096
            {
                return Err("context_limit");
            }
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled");
            }
            if clock.started.elapsed() >= BUDGET {
                return Err("timeout");
            }
            attempt.authorize()?;
            channel.send(&control, cancelled)?;
            let progress = channel.reply(
                &mut attempt,
                &target,
                &root,
                "close_progress",
                None,
                cancelled,
            )?;
            attempt.stage("closing");
            if progress
                .native
                .as_ref()
                .is_none_or(|n| !n.entered || n.close_error != Some(0))
            {
                return Err("native_failure");
            }
            attempt.stage("settling");
            let waited = channel.reply(
                &mut attempt,
                &target,
                &root,
                "close_waited",
                None,
                cancelled,
            )?;
            if waited
                .continuation
                .as_ref()
                .is_none_or(|c| !["completed", "not_applicable"].contains(&c.state.as_str()))
            {
                return Err("native_revalidation_unavailable");
            }
        }
        attempt.stage("verifying");
        let acquisition = AttemptClock::start();
        let purpose = if recognition {
            "recognition"
        } else {
            "post_close"
        };
        let mut verified = channel.reply(
            &mut attempt,
            &target,
            &root,
            "close_sample",
            None,
            cancelled,
        )?;
        if verified.purpose.as_deref() != Some(purpose) {
            return Err("protocol_error");
        }
        let sample = verified.sample.take().ok_or("verification_incomplete")?;
        let original = sample.collection.clone();
        let mut collected = Collected::new();
        collected
            .accept(Event::Selected(target.clone()))
            .map_err(|_| "protocol_error")?;
        collected
            .accept(Event::Sample(Box::new(sample)))
            .map_err(|_| "protocol_error")?;
        let final_disk = disk(&mut channel, &attempt.request, &target, cancelled)?;
        binding(&attempt.request, &target, &final_disk)?;
        if final_disk.file != initial_disk.file
            || ((!recognition || final_disk.text().is_some() && initial_disk.text().is_some())
                && final_disk.text() != initial_disk.text())
        {
            return Err("source_changed");
        }
        let event = if let Some((reason, collection)) = final_disk.metadata {
            Event::DiskMetadata {
                reason,
                file: Some(final_disk.file),
                collection,
            }
        } else {
            Event::Disk(final_disk.source)
        };
        collected.accept(event).map_err(|_| "protocol_error")?;
        let mut checked = channel.reply(
            &mut attempt,
            &target,
            &root,
            "close_rechecked",
            Some(&original),
            cancelled,
        )?;
        if checked.purpose.as_deref() != Some(purpose) {
            return Err("protocol_error");
        }
        collected
            .accept(Event::Rechecked(
                checked.recheck.take().ok_or("verification_incomplete")?,
            ))
            .map_err(|_| "protocol_error")?;
        collected
            .accept(channel.event(&attempt.request, Some(&target), cancelled)?)
            .map_err(|_| "protocol_error")?;
        if !matches!(
            channel.event(&attempt.request, Some(&target), cancelled)?,
            Event::Done
        ) {
            return Err("protocol_error");
        }
        let ordinary = collected
            .finish(attempt.request.observation().clone(), clock)
            .map_err(|_| "protocol_error")?;
        let snapshot = ordinary.snapshot().ok_or("verification_incomplete")?;
        attempt.result.observation = Some(core::ObservationSummary {
            purpose: if recognition {
                "recognition"
            } else {
                "verification"
            },
            interval: acquisition.interval(),
            snapshot: core::SnapshotSummary::of(snapshot),
        });
        if snapshot.document().open_state().value() != Some(&OpenState::NotOpen)
            || snapshot.document().validity().value() != Some(&Validity::Valid)
            || snapshot.consistency().checks() != Checks::Performed
            || snapshot.consistency().stability() == Stability::Changed
        {
            return Err("verification_failed");
        }
        if !recognition {
            let native = verified.native.as_ref().ok_or("verification_incomplete")?;
            if native.old_document_removed != Some(true)
                || native.invalidated
                || native.protection.as_ref().is_none_or(|p| {
                    p.status != "preserved"
                        || !["completed", "not_applicable"].contains(&p.revalidation.as_str())
                })
            {
                return Err("verification_failed");
            }
            match verified.resource.as_ref().map(|r| r.state.as_str()) {
                Some("unloaded") => {}
                Some("retained") => {
                    let expected = attempt.request.expected().ok_or("invalid_basis")?;
                    let r = snapshot.sources().resource();
                    if r.witness().and_then(Witness::script_instance_id)
                        != expected.document().script_instance_id()
                        || r.text() != initial_disk.text()
                    {
                        return Err("verification_failed");
                    }
                }
                _ => return Err("verification_incomplete"),
            }
        }
        channel.reply(
            &mut attempt,
            &target,
            &root,
            "close_finished",
            None,
            cancelled,
        )?;
        attempt.complete(recognition);
        attempt.result.progress.verification.collection = Some(checked.collection);
        Ok(())
    })();
    if let Err(reason) = result {
        attempt.fail(reason);
    }
    drop(channel);
    drop(worker);
    Ok(attempt.finish(clock.interval()))
}
