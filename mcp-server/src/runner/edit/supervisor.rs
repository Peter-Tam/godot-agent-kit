//! The only authorization release; the reducer retains every attributable partial fact.
use super::*;
use crate::script_edit::{
    Bookkeeping, FinalizationResult, FinalizationStatus, NativeWitness, PersistenceReceipt, Reason,
    SourceDigest, ValidationPurpose,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Prepared,
    PreparedDetail,
    Preflight,
    Guard,
    GuardDetail,
    AbortAwait,
    Applying,
    Post,
    PostDetail,
    PostValidation,
    Verified,
    VerifiedDetail,
    Context,
    Terminal,
}
fn observation(request: &EditRequest) -> ObservationRequest {
    ObservationRequest::new(
        request.request_id().clone(),
        request.project_root().clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    )
}
fn send_bytes(
    stream: &mut UnixStream,
    bytes: &[u8],
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), Reason> {
    if bytes.is_empty() || bytes.len() > WORKER_LIMIT {
        return Err(Reason::ProtocolFailure);
    }
    let header = (bytes.len() as u32).to_be_bytes();
    for part in [&header[..], bytes] {
        let mut offset = 0;
        while offset < part.len() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(Reason::Cancellation);
            }
            if clock.started.elapsed() >= BUDGET {
                return Err(Reason::Deadline);
            }
            match stream.write(&part[offset..(offset + 65536).min(part.len())]) {
                Ok(0) => return Err(Reason::Disconnection),
                Ok(n) => offset += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL_INTERVAL),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => return Err(Reason::Disconnection),
            }
        }
    }
    Ok(())
}
fn send_control(
    stream: &mut UnixStream,
    id: &RequestId,
    name: &str,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), Reason> {
    let bytes = serde_json::to_vec(&(1, name, id.as_str())).map_err(|_| Reason::ProtocolFailure)?;
    if bytes.len() > CONTROL_LIMIT {
        return Err(Reason::ProtocolFailure);
    }
    send_bytes(stream, &bytes, clock, cancelled)
}
fn finish(mut attempt: EditAttempt, reason: Option<Reason>, clock: AttemptClock) -> EditOutcome {
    if let Some(reason) = reason {
        let _ = attempt.fail(reason, None);
    }
    attempt.finish(clock.interval())
}
fn unknown_stage(reason: Reason) -> Reason {
    if reason == Reason::Complete {
        Reason::IncompleteVerification
    } else {
        reason
    }
}
fn receipt(
    attempt: &EditAttempt,
    witness: &NativeWitness,
    native: &ipc::NativeState,
) -> PersistenceReceipt {
    let request = attempt.request();
    let intended = SourceDigest::of(request.replacement_source().as_str());
    let restored = native.restore_attempted && native.restored && native.restore_errno == 0;
    PersistenceReceipt {
        request_id: witness.request_id.clone(),
        session_id: witness.session_id.clone(),
        document: witness.document.clone(),
        intended,
        project_id: request.expected().target().project_file_id().clone(),
        file_id: request
            .expected()
            .document()
            .disk_file_id()
            .expect("eligible basis has D identity")
            .clone(),
        collection: witness.collection.clone(),
        write_started: native.write_started,
        bytes_written: native.written_bytes,
        truncated: native.truncated,
        flushed: native.flushed,
        readback_matches: native.readback,
        attached: native.status == "ready" || native.status == "complete",
        descriptor_open: native.status == "ready" || native.status == "complete",
        interference: matches!(
            native.reason,
            Reason::RevisionMismatch | Reason::IdentityChanged
        ),
        original_mtime: Some(native.t0.clone()),
        mtime: native.after_restore.clone(),
        restore_attempted: native.restore_attempted,
        restore_errno: (native.restore_errno != 0).then_some(native.restore_errno),
        restored,
        reason: if restored
            && native.readback
            && native.flushed
            && native.truncated
            && native.written_bytes == intended.utf8_bytes
            && native.write_errno == 0
            && native.truncate_errno == 0
            && native.fsync_errno == 0
            && native.pread_errno == 0
            && (native.status == "ready" || native.status == "complete")
        {
            None
        } else if native.stage == "content_persisted"
            && native.status == "ready"
            && !native.restore_attempted
        {
            Some(Reason::PersistenceUnknown)
        } else {
            Some(if native.reason == Reason::Complete {
                Reason::PersistenceFailure
            } else {
                native.reason
            })
        },
    }
}
fn finalization_result(
    witness: NativeWitness,
    before: Option<ipc::ActualState>,
    after: Option<ipc::ActualState>,
    edited_cleared: bool,
    tagged: bool,
    status: FinalizationStatus,
    reason: Option<Reason>,
) -> FinalizationResult {
    FinalizationResult {
        status,
        reason,
        request_id: witness.request_id,
        session_id: witness.session_id,
        document: witness.document,
        collection: witness.collection,
        before_source: before.as_ref().and_then(|s| s.source),
        after_source: after.as_ref().and_then(|s| s.source),
        before_current: before.as_ref().and_then(|s| s.current.clone()),
        after_current: after.as_ref().and_then(|s| s.current.clone()),
        before_saved: before.as_ref().and_then(|s| s.saved.clone()),
        after_saved: after.as_ref().and_then(|s| s.saved.clone()),
        before_resource_edited: before.as_ref().and_then(|s| s.edited),
        after_resource_edited: after.as_ref().and_then(|s| s.edited),
        steps: Bookkeeping {
            resource_edited: edited_cleared,
            saved_version: tagged,
        },
    }
}
struct NativeProgress {
    prior: usize,
    pending_content: Option<(NativeWitness, ipc::NativeState)>,
    partial_native: Option<(NativeWitness, Reason)>,
    before_a: Option<ipc::ActualState>,
    cleared: Option<(NativeWitness, Option<ipc::ActualState>, bool)>,
    applied: bool,
    content_complete: bool,
    persistence_received: bool,
    entered_a: bool,
    final_received: bool,
}
impl NativeProgress {
    fn new() -> Self {
        Self {
            prior: 0,
            pending_content: None,
            partial_native: None,
            before_a: None,
            cleared: None,
            applied: false,
            content_complete: false,
            persistence_received: false,
            entered_a: false,
            final_received: false,
        }
    }
    fn consume(
        &mut self,
        attempt: &mut EditAttempt,
        witness: NativeWitness,
        native: ipc::NativeState,
    ) -> Result<(), RoutingFailure> {
        let step = match native.stage.as_str() {
            "prepared" => 0,
            "buffer_applied" => 1,
            "resource_applied" => 2,
            "content_persisted" => 3,
            "mtime_restored" => 4,
            "edited_cleared" => 5,
            "saved_tagged" => 6,
            _ => return Err(protocol_failure()),
        };
        if step <= self.prior && !(step == 0 && self.prior == 0) {
            return Err(protocol_failure());
        }
        if step > self.prior + 1 || (step == 0 && self.prior != 0) {
            return Err(protocol_failure());
        }
        self.prior = step;
        if step == 0 {
            return Ok(());
        }
        if !self.applied {
            attempt
                .enter_application()
                .map_err(|_| protocol_failure())?;
            self.applied = true;
        }
        match step {
            1 => {
                if native.buffer_changed {
                    let current = native.frozen_current.ok_or_else(protocol_failure)?;
                    attempt
                        .buffer_changed(current, witness)
                        .map_err(|_| protocol_failure())?;
                } else if native.status == "partial" && native.resource_changed {
                    attempt
                        .partial_application(unknown_stage(native.reason), witness)
                        .map_err(|_| protocol_failure())?;
                } else {
                    attempt
                        .fail(unknown_stage(native.reason), None)
                        .map_err(|_| protocol_failure())?;
                }
            }
            2 => {
                if native.resource_changed {
                    let _ = attempt.enter_resource_sync();
                    attempt
                        .resource_synced(witness)
                        .map_err(|_| protocol_failure())?;
                } else {
                    let _ = attempt.enter_resource_sync();
                    let _ = attempt.resource_sync_unknown(unknown_stage(native.reason));
                }
            }
            3 => {
                let _ = attempt.enter_persistence();
                self.pending_content = Some((witness, native));
            }
            4 => {
                let (previous, content) =
                    self.pending_content.take().ok_or_else(protocol_failure)?;
                if content.written_bytes > native.written_bytes
                    || !content.write_started && native.write_started
                {
                    return Err(protocol_failure());
                }
                let full = receipt(attempt, &witness, &native);
                self.content_complete = full.reason.is_none();
                // A completed metadata stage belongs to the retained descriptor,
                // not a fresh independent Rust D read.
                attempt.persistence(full).map_err(|_| protocol_failure())?;
                self.persistence_received = true;
                drop(previous);
            }
            5 => {
                if self.content_complete {
                    attempt
                        .enter_finalization()
                        .map_err(|_| protocol_failure())?;
                    self.entered_a = true;
                }
                self.before_a = native.before;
                self.cleared = Some((witness, native.after, native.edited_cleared));
                if !native.edited_cleared {
                    let _ = attempt.fail(
                        if native.reason == Reason::Complete {
                            Reason::PartialFinalization
                        } else {
                            native.reason
                        },
                        None,
                    );
                }
            }
            6 => {
                if !self.entered_a {
                    let _ = attempt.fail(Reason::PartialFinalization, None);
                    return Ok(());
                }
                let before = self.before_a.take();
                let (clear_after, clear_edited) = self
                    .cleared
                    .take()
                    .map(|(_, after, edited)| (after, edited))
                    .unwrap_or((None, false));
                let after = native.after.or(clear_after);
                let edited = native.edited_cleared || clear_edited;
                let complete = native.tagged && edited && before.is_some() && after.is_some();
                let result = finalization_result(
                    witness,
                    before,
                    after,
                    edited,
                    native.tagged,
                    if complete {
                        FinalizationStatus::Complete
                    } else {
                        FinalizationStatus::PartialOrUnknown
                    },
                    (!complete).then_some(if native.reason == Reason::Complete {
                        Reason::PartialFinalization
                    } else {
                        native.reason
                    }),
                );
                attempt
                    .finalization(result)
                    .map_err(|_| protocol_failure())?;
                self.entered_a = false;
            }
            _ => return Err(protocol_failure()),
        }
        Ok(())
    }
    fn native_final(
        &mut self,
        attempt: &mut EditAttempt,
        witness: NativeWitness,
        native: ipc::NativeState,
    ) -> Result<(), RoutingFailure> {
        if self.final_received {
            return Err(protocol_failure());
        }
        self.final_received = true;
        if native.status == "complete" {
            if self.prior != 6 || !native.tagged || !native.restored {
                return Err(protocol_failure());
            }
            return Ok(());
        }
        if native.status != "partial" && native.status != "refused" {
            return Err(protocol_failure());
        }
        if native.status == "refused" && self.prior != 0 {
            return Err(protocol_failure());
        }
        let reason = unknown_stage(native.reason);
        let original = attempt.request().expected();
        let proven = native.after.as_ref().is_some_and(|after| {
            native.before.as_ref().is_none_or(|before| {
                before.source == Some(*original.source())
                    || before.current.as_ref() == Some(original.current_version())
            }) && (after
                .source
                .is_some_and(|source| source != *original.source())
                || after
                    .current
                    .as_ref()
                    .is_some_and(|v| v != original.current_version()))
        });
        if native.status == "partial"
            && !self.applied
            && (native.buffer_changed
                || native.resource_changed
                || native.write_started
                || native.edited_attempted
                || native.tag_attempted
                || proven)
        {
            attempt
                .enter_application()
                .map_err(|_| protocol_failure())?;
            self.applied = true;
            if proven {
                attempt
                    .partial_application(reason, witness.clone())
                    .map_err(|_| protocol_failure())?;
            } else {
                self.partial_native = Some((witness.clone(), reason));
            }
        }
        if let Some((held, _)) = self.pending_content.as_ref() {
            if native.stage == "content_persisted" && held.document == witness.document {
                // This is the terminal metadata attempt: no later restore
                // progress can complete the held content receipt.
                let value = receipt(attempt, &witness, &native);
                attempt.persistence(value).map_err(|_| protocol_failure())?;
                self.pending_content = None;
                self.persistence_received = true;
                let _ = attempt.fail(reason, None);
                return Ok(());
            }
        }
        if native.write_started
            && self.prior >= 2
            && self.pending_content.is_none()
            && !self.persistence_received
        {
            let _ = attempt.enter_persistence();
            let value = receipt(attempt, &witness, &native);
            attempt.persistence(value).map_err(|_| protocol_failure())?;
            self.persistence_received = true;
        }
        if !self.entered_a
            && self.content_complete
            && matches!(native.stage.as_str(), "edited_cleared" | "saved_tagged")
        {
            attempt
                .enter_finalization()
                .map_err(|_| protocol_failure())?;
            self.entered_a = true;
        }
        if self.entered_a {
            let before = self.before_a.take().or(native.before);
            let (clear_after, clear_edited) = self
                .cleared
                .take()
                .map(|(_, after, edited)| (after, edited))
                .unwrap_or((None, false));
            let result = finalization_result(
                witness,
                before,
                native.after.or(clear_after),
                native.edited_cleared || clear_edited,
                native.tagged,
                FinalizationStatus::PartialOrUnknown,
                Some(reason),
            );
            attempt
                .finalization(result)
                .map_err(|_| protocol_failure())?;
            self.entered_a = false;
        }
        if reason != Reason::Complete {
            let _ = attempt.fail(reason, None);
        }
        Ok(())
    }
    fn independent_partial(&mut self, attempt: &mut EditAttempt, observed: &ObservationOutcome) {
        let Some((witness, reason)) = self.partial_native.take() else {
            return;
        };
        let Some(snapshot) = observed
            .snapshot()
            .filter(|s| s.document().identity() == Some(attempt.request().expected().document()))
        else {
            return;
        };
        let source = snapshot.sources().buffer().text();
        let changed = source
            .is_some_and(|s| SourceDigest::of(s) != *attempt.request().expected().source())
            || snapshot
                .sources()
                .buffer()
                .witness()
                .and_then(Witness::source_version)
                .is_some_and(|v| v != attempt.request().expected().current_version());
        if changed {
            let _ = attempt.partial_application(reason, witness);
        }
    }
    fn retain_partial(&mut self, attempt: &mut EditAttempt, reason: Option<Reason>) {
        if let Some((witness, native)) = self.pending_content.take() {
            let value = receipt(attempt, &witness, &native);
            let _ = attempt.persistence(value);
        }
        if self.entered_a {
            if let Some((witness, after, edited)) = self.cleared.take() {
                let result = finalization_result(
                    witness,
                    self.before_a.take(),
                    after,
                    edited,
                    false,
                    FinalizationStatus::PartialOrUnknown,
                    Some(reason.unwrap_or(Reason::PartialFinalization)),
                );
                let _ = attempt.finalization(result);
            } else {
                let _ = attempt.finalization_unknown(reason.unwrap_or(Reason::PartialFinalization));
            }
            self.entered_a = false;
        }
    }
}
fn monitor_helper(
    mut monitor: UnixStream,
    cancelled: &AtomicBool,
    stop: &AtomicBool,
    interrupted: &AtomicBool,
    helper_cancel: &AtomicBool,
) {
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
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => {
                interrupted.store(true, Ordering::Relaxed);
                helper_cancel.store(true, Ordering::Relaxed);
                break;
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
}
fn run_helper(
    socket: &mut UnixStream,
    bytes: &[u8],
    attempt: &EditAttempt,
    purpose: stock_validation::Purpose,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), RoutingFailure> {
    if matches!(
        purpose,
        stock_validation::Purpose::OpenContext | stock_validation::Purpose::CloseContext
    ) || bytes.len() > ipc::HELPER_REQUEST_LIMIT
    {
        return Err(protocol_failure());
    }
    let request: ipc::HelperRequestIn =
        serde_json::from_slice(bytes).map_err(|_| protocol_failure())?;
    let request = request.domain(attempt.request(), purpose)?;
    let monitor = socket.try_clone().map_err(|_| protocol_failure())?;
    monitor
        .set_nonblocking(true)
        .map_err(|_| protocol_failure())?;
    let stop = AtomicBool::new(false);
    let interrupted = AtomicBool::new(false);
    let helper_cancel = AtomicBool::new(false);
    let result = thread::scope(|scope| {
        scope.spawn(|| monitor_helper(monitor, cancelled, &stop, &interrupted, &helper_cancel));
        let result = stock_validation::validate(request, clock, &helper_cancel);
        stop.store(true, Ordering::Relaxed);
        result
    });
    if interrupted.load(Ordering::Relaxed) {
        return Err(error(Reason::Disconnection));
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(Reason::Cancellation));
    }
    let bytes = serde_json::to_vec(&ipc::helper_reply(
        attempt.request().request_id(),
        purpose,
        &result,
    ))
    .map_err(|_| protocol_failure())?;
    if bytes.len() > SOURCE_LIMIT_BYTES + 4096 {
        return Err(protocol_failure());
    }
    send_bytes(socket, &bytes, clock, cancelled).map_err(super::error)
}
fn send_abort(
    stream: &mut UnixStream,
    id: &RequestId,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) {
    if clock.started.elapsed() < BUDGET {
        let _ = send_control(stream, id, "abort", clock, cancelled);
    }
}
/// Owned worker launched only from the same binary; no user executable/timeout override.
pub fn run(
    request: EditRequest,
    input_bytes: Vec<u8>,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<EditOutcome, HostFailure> {
    let attempt = EditAttempt::new(request);
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= BUDGET {
        return Ok(finish(
            attempt,
            Some(if cancelled.load(Ordering::Relaxed) {
                Reason::Cancellation
            } else {
                Reason::Deadline
            }),
            clock,
        ));
    }
    let (mut parent, child_socket) = UnixStream::pair().map_err(|_| HostFailure)?;
    parent.set_nonblocking(true).map_err(|_| HostFailure)?;
    let read_fd: OwnedFd = child_socket.try_clone().map_err(|_| HostFailure)?.into();
    let write_fd: OwnedFd = child_socket.into();
    let worker = Command::new(std::env::current_exe().map_err(|_| HostFailure)?)
        .arg(INTERNAL_WORKER_FLAG)
        .stdin(Stdio::from(read_fd))
        .stdout(Stdio::from(write_fd))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| HostFailure)?;
    let worker = OwnedWorker(Some(worker));
    let request = observation(attempt.request());
    let id = request.request_id().clone();
    let original = attempt.request();
    let startup = serde_json::to_vec(&(
        6,
        id.as_str(),
        original.project_root().as_str(),
        original.session_id().map(SessionId::as_str),
        original.script_path().as_str(),
        registry.to_str().ok_or(HostFailure)?,
        clock.elapsed_us(),
    ))
    .map_err(|_| HostFailure)?;
    let sent = if startup.len() <= STARTUP_LIMIT && input_bytes.len() <= wire::edit::input_limit() {
        send_bytes(&mut parent, &startup, clock, cancelled)
            .and_then(|_| send_bytes(&mut parent, &input_bytes, clock, cancelled))
    } else {
        Err(Reason::ProtocolFailure)
    };
    if let Err(reason) = sent {
        drop(parent);
        drop(worker);
        return Ok(finish(attempt, Some(reason), clock));
    }
    let outcome = supervise(parent, worker, attempt, clock, cancelled);
    Ok(outcome)
}
fn supervise(
    mut socket: UnixStream,
    worker: OwnedWorker,
    mut attempt: EditAttempt,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> EditOutcome {
    let request = observation(attempt.request());
    let id = request.request_id().clone();
    let mut frames = Frames::default();
    let mut phase = Phase::Prepared;
    let mut collected = super::super::Collected::new();
    let mut observed: Option<ObservationOutcome> = None;
    let mut post_observed: Option<(
        ObservationOutcome,
        Option<script_edit::SavedStateEvidence>,
        Option<script_edit::DiskMetadata>,
    )> = None;
    let mut changed = false;
    let mut progress = NativeProgress::new();
    let mut helper_pending = false;
    let mut final_saved: Option<script_edit::SavedStateEvidence> = None;
    let mut final_disk: Option<script_edit::DiskMetadata> = None;
    let mut context: Option<script_edit::ContextRecheck> = None;
    let mut reason = None;
    let mut finished = false;
    let mut verified_sample = false;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            reason = Some(Reason::Cancellation);
            break;
        }
        if clock.started.elapsed() >= BUDGET {
            reason = Some(Reason::Deadline);
            break;
        }
        let bytes = match frames.next(&mut socket) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            Err(_) => {
                reason = Some(Reason::Disconnection);
                break;
            }
        };
        if clock.started.elapsed() >= BUDGET {
            reason = Some(Reason::Deadline);
            break;
        }
        let current: ipc::KindProbe<'_> = match serde_json::from_slice(&bytes) {
            Ok(current) => current,
            Err(_) => {
                reason = Some(Reason::ProtocolFailure);
                break;
            }
        };
        let result = (|| -> Result<(), RoutingFailure> {
            let receipt = clock.elapsed_us();
            match current.kind {
                "selected" | "sample" | "disk" | "rechecked" | "disk_checked" | "done"
                | "failed" => {
                    if current.kind == "failed" {
                        let event = wire::decode_event(
                            &bytes,
                            &request,
                            collected.target.as_ref(),
                            receipt,
                        )?;
                        if let Event::Failed(failure) = event {
                            attempt
                                .fail(super::reason(&failure), failure.selection)
                                .map_err(|_| protocol_failure())?;
                            finished = true;
                            return Ok(());
                        }
                        return Err(protocol_failure());
                    }
                    if !matches!(
                        phase,
                        Phase::Prepared
                            | Phase::Guard
                            | Phase::Post
                            | Phase::Verified
                            | Phase::Applying
                    ) {
                        return Err(protocol_failure());
                    }
                    if phase == Phase::Applying {
                        if current.kind != "selected" || !progress.final_received {
                            return Err(protocol_failure());
                        }
                        phase = Phase::Post;
                    }
                    let event =
                        wire::decode_event(&bytes, &request, collected.target.as_ref(), receipt)?;
                    if let Event::Selected(target) = &event {
                        if !matches!(phase, Phase::Prepared)
                            && attempt.request().expected().target().session_id()
                                != target.session_id()
                        {
                            return Err(protocol_failure());
                        }
                        if phase == Phase::Prepared {
                            attempt
                                .select(target.clone())
                                .map_err(|_| protocol_failure())?;
                        }
                    }
                    let done = collected.accept(event)?;
                    if done {
                        let state =
                            std::mem::replace(&mut collected, super::super::Collected::new());
                        let outcome = state
                            .finish(request.clone(), clock)
                            .map_err(|_| protocol_failure())?;
                        if phase == Phase::Prepared {
                            changed = outcome
                                .snapshot()
                                .and_then(|s| s.sources().disk().text())
                                .is_some_and(|text| {
                                    text != attempt.request().replacement_source().as_str()
                                });
                        }
                        observed = Some(outcome);
                        phase = match phase {
                            Phase::Prepared => Phase::PreparedDetail,
                            Phase::Guard => Phase::GuardDetail,
                            Phase::Post => Phase::PostDetail,
                            Phase::Verified => Phase::VerifiedDetail,
                            _ => return Err(protocol_failure()),
                        };
                    }
                }
                "edit_detail" => {
                    let detail: ipc::DetailIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    let expected = match phase {
                        Phase::PreparedDetail => "prepared",
                        Phase::GuardDetail => "guard",
                        Phase::PostDetail => "post",
                        Phase::VerifiedDetail => "verified",
                        _ => return Err(protocol_failure()),
                    };
                    if detail.phase() != expected {
                        return Err(protocol_failure());
                    }
                    let snapshot = observed.take().ok_or_else(protocol_failure)?;
                    let target = snapshot
                        .resolved_target()
                        .ok_or_else(protocol_failure)?
                        .clone();
                    let (saved, disk) = detail.domain(&request, &target, &snapshot, receipt)?;
                    match phase {
                        Phase::PreparedDetail => {
                            attempt
                                .prepare(snapshot, saved, disk)
                                .map_err(|_| protocol_failure())?;
                            phase = Phase::Preflight;
                        }
                        Phase::GuardDetail => {
                            attempt
                                .guard_application(snapshot, saved, disk)
                                .map_err(|_| protocol_failure())?;
                            if !attempt.ready_to_apply() {
                                send_abort(&mut socket, &id, clock, cancelled);
                                phase = Phase::AbortAwait;
                                return Ok(());
                            }
                            attempt.authorize().map_err(|_| protocol_failure())?;
                            // may_apply is recorded inside authorize() before exactly one release.
                            send_control(&mut socket, &id, "authorize_apply", clock, cancelled)
                                .map_err(super::error)?;
                            phase = Phase::Applying;
                        }
                        Phase::PostDetail => {
                            progress.independent_partial(&mut attempt, &snapshot);
                            if snapshot.snapshot().is_some_and(|s| {
                                s.dirty().state() != Some(DirtyState::Clean)
                                    || [
                                        s.sources().disk(),
                                        s.sources().resource(),
                                        s.sources().buffer(),
                                    ]
                                    .iter()
                                    .any(|v| {
                                        v.text()
                                            != Some(attempt.request().replacement_source().as_str())
                                    })
                            }) {
                                let _ = attempt.fail(Reason::RevisionMismatch, None);
                            }
                            if saved.is_none() || disk.is_none() {
                                let _ = attempt.fail(Reason::UnavailableObservation, None);
                            }
                            post_observed = Some((snapshot, saved, disk));
                            phase = Phase::PostValidation;
                        }
                        Phase::VerifiedDetail => {
                            observed = Some(snapshot);
                            final_saved = saved;
                            final_disk = disk;
                            phase = Phase::Context;
                        }
                        _ => return Err(protocol_failure()),
                    }
                }
                "edit_helper_request" => {
                    if helper_pending || !matches!(phase, Phase::Preflight | Phase::PostValidation)
                    {
                        return Err(protocol_failure());
                    }
                    let purpose = if phase == Phase::PostValidation {
                        stock_validation::Purpose::PostChange
                    } else if changed {
                        stock_validation::Purpose::Preflight
                    } else {
                        stock_validation::Purpose::Unchanged
                    };
                    run_helper(&mut socket, &bytes, &attempt, purpose, clock, cancelled)?;
                    helper_pending = true;
                }
                "edit_validation" => {
                    if !helper_pending || !matches!(phase, Phase::Preflight | Phase::PostValidation)
                    {
                        return Err(protocol_failure());
                    }
                    helper_pending = false;
                    let value: ipc::ValidationIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    let purpose = if phase == Phase::PostValidation {
                        ValidationPurpose::PostChange
                    } else if changed {
                        ValidationPurpose::Preflight
                    } else {
                        ValidationPurpose::Unchanged
                    };
                    let validation = value.domain(attempt.request(), purpose, receipt)?;
                    let valid = validation.status == script_edit::ValidationStatus::Valid
                        && validation.reason.is_none()
                        && validation.context_current
                        && validation.dependencies_current;
                    attempt
                        .validation(validation)
                        .map_err(|_| protocol_failure())?;
                    if phase == Phase::Preflight {
                        if !valid {
                            finished = true;
                        } else {
                            phase = if changed {
                                Phase::Guard
                            } else {
                                Phase::Verified
                            };
                        }
                    } else {
                        phase = Phase::Verified;
                    }
                }
                "edit_progress" => {
                    if phase != Phase::Applying {
                        return Err(protocol_failure());
                    }
                    let value: ipc::ProgressIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    let (witness, native) =
                        value.domain(attempt.request(), receipt, "edit_progress")?;
                    progress.consume(&mut attempt, witness, native)?;
                }
                "edit_native_final" => {
                    if phase != Phase::Applying {
                        return Err(protocol_failure());
                    }
                    let value: ipc::ProgressIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    let (witness, native) =
                        value.domain(attempt.request(), receipt, "edit_native_final")?;
                    progress.native_final(&mut attempt, witness, native)?;
                }
                "edit_context" => {
                    if phase != Phase::Context {
                        return Err(protocol_failure());
                    }
                    let value: ipc::ContextIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    context = Some(value.domain(attempt.request(), receipt)?);
                    phase = Phase::Terminal;
                }
                "edit_terminal" => {
                    if matches!(
                        phase,
                        Phase::Guard
                            | Phase::Post
                            | Phase::Verified
                            | Phase::PreparedDetail
                            | Phase::GuardDetail
                            | Phase::PostDetail
                            | Phase::VerifiedDetail
                            | Phase::Context
                    ) {
                        return Err(protocol_failure());
                    }
                    let value: ipc::TerminalIn =
                        serde_json::from_slice(&bytes).map_err(|_| protocol_failure())?;
                    let (why, preboundary, stamp) = value.domain(
                        &id,
                        attempt.request().expected().target().session_id(),
                        receipt,
                    )?;
                    progress.retain_partial(&mut attempt, (why != Reason::Complete).then_some(why));
                    if preboundary && changed && phase == Phase::Applying && !progress.applied {
                        let stamp = stamp.ok_or_else(protocol_failure)?;
                        let witness = NativeWitness {
                            request_id: id.clone(),
                            session_id: attempt.request().expected().target().session_id().clone(),
                            document: attempt.request().expected().document().clone(),
                            collection: stamp,
                        };
                        attempt
                            .discard_before_boundary(unknown_stage(why), witness)
                            .map_err(|_| protocol_failure())?;
                    } else if why != Reason::Complete {
                        attempt.fail(why, None).map_err(|_| protocol_failure())?;
                    }
                    if phase == Phase::Terminal {
                        let observed = observed.take().ok_or_else(protocol_failure)?;
                        attempt
                            .verify(
                                observed,
                                final_saved.take(),
                                final_disk.take(),
                                context.take(),
                            )
                            .map_err(|_| protocol_failure())?;
                        verified_sample = true;
                    }
                    finished = true;
                }
                _ => return Err(protocol_failure()),
            }
            Ok(())
        })();
        if let Err(error) = result {
            reason = Some(super::reason(&error));
            break;
        }
        if finished {
            break;
        }
    }
    if !finished {
        progress.retain_partial(&mut attempt, reason);
        send_abort(&mut socket, &id, clock, cancelled);
    }
    if !verified_sample && progress.applied {
        let latest = observed
            .take()
            .and_then(|sample| {
                if sample.snapshot().and_then(|s| s.document().identity())
                    == Some(attempt.request().expected().document())
                {
                    Some((sample, final_saved.take(), final_disk.take()))
                } else {
                    let _ = attempt.fail(
                        match sample.outcome() {
                            OutcomeKind::DeniedAccess => Reason::DeniedAccess,
                            OutcomeKind::NotOpen => Reason::ClosedTarget,
                            OutcomeKind::DisconnectedEditor => Reason::Disconnection,
                            OutcomeKind::Timeout => Reason::Deadline,
                            _ if sample.snapshot().is_some() => Reason::IdentityChanged,
                            _ => Reason::UnavailableObservation,
                        },
                        None,
                    );
                    None
                }
            })
            .or(post_observed);
        if let Some((sample, saved, disk)) = latest {
            let _ = attempt.retain_after(sample, saved, disk);
        }
    }
    drop(socket);
    drop(worker);
    finish(attempt, reason, clock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::OwnedFd;

    #[test]
    fn live_helper_monitor_cancels_on_actual_worker_process_exit() {
        let (parent, peer) = UnixStream::pair().unwrap();
        let monitor = parent.try_clone().unwrap();
        monitor.set_nonblocking(true).unwrap();
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 0.08")
            .stdin(Stdio::from(OwnedFd::from(peer)))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let cancelled = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        let interrupted = AtomicBool::new(false);
        let helper_cancel = AtomicBool::new(false);
        thread::scope(|scope| {
            let watch = scope
                .spawn(|| monitor_helper(monitor, &cancelled, &stop, &interrupted, &helper_cancel));
            assert!(child.wait().unwrap().success());
            watch.join().unwrap();
        });
        assert!(interrupted.load(Ordering::Relaxed));
        assert!(helper_cancel.load(Ordering::Relaxed));
    }

    #[test]
    fn live_helper_monitor_cancels_on_parent_cancellation_without_worker_loss() {
        let (parent, peer) = UnixStream::pair().unwrap();
        let monitor = parent.try_clone().unwrap();
        monitor.set_nonblocking(true).unwrap();
        let mut child = Command::new("/bin/sleep")
            .arg("5")
            .stdin(Stdio::from(OwnedFd::from(peer)))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let cancelled = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        let interrupted = AtomicBool::new(false);
        let helper_cancel = AtomicBool::new(false);
        thread::scope(|scope| {
            let watch = scope
                .spawn(|| monitor_helper(monitor, &cancelled, &stop, &interrupted, &helper_cancel));
            cancelled.store(true, Ordering::Relaxed);
            watch.join().unwrap();
        });
        assert!(helper_cancel.load(Ordering::Relaxed));
        assert!(!interrupted.load(Ordering::Relaxed));
        child.kill().unwrap();
        child.wait().unwrap();
    }
}
