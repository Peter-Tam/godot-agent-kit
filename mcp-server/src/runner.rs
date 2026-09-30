//! Same-binary, read-only acquisition worker and bounded observation supervisor.
//!
//! Potentially blocking filesystem/editor work stays in the owned child. The parent only
//! consumes bounded, validated events and reduces them through the common observation core.
//! No executable, timeout override, editor control, or source bypass is a caller option.
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, LazyLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::bridge::wire::{self, EditorSample, Event};
use crate::observation::*;
use crate::target::RoutingFailure;
use crate::{project_fs, target};

pub const INTERNAL_WORKER_FLAG: &str = "--internal-observation-worker";
const COLLECTION_BUDGET: Duration = Duration::from_millis(4500);
const POLL_INTERVAL: Duration = Duration::from_millis(1);
const CONTROL_LIMIT: usize = 4096;

#[path = "runner/stock_validation.rs"]
pub mod stock_validation;

#[path = "runner/edit.rs"]
pub mod edit;

#[path = "runner/open.rs"]
pub mod open;

/// The caller creates this before parsing flags or doing any filesystem/selection work.
#[derive(Clone, Copy)]
pub struct AttemptClock {
    started: Instant,
    started_unix_ms: u64,
}
impl Default for AttemptClock {
    fn default() -> Self {
        Self::start()
    }
}
impl AttemptClock {
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
            started_unix_ms: unix_ms(),
        }
    }
    pub fn elapsed_us(&self) -> u64 {
        self.started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
    }
    pub fn interval(&self) -> ObservationInterval {
        ObservationInterval::new(self.started_unix_ms, unix_ms(), self.elapsed_us())
    }
}
fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

/// An unexpected host failure, not an editor outcome. No OS error/payload is retained.
#[derive(Debug)]
pub struct HostFailure;
impl std::fmt::Display for HostFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The owned observation worker could not be launched or supervised")
    }
}
impl std::error::Error for HostFailure {}

pub fn exit_code(outcome: OutcomeKind) -> i32 {
    match outcome {
        OutcomeKind::CompleteObservation | OutcomeKind::NotOpen => 0,
        OutcomeKind::LimitedObservation | OutcomeKind::UnsupportedObservation => 2,
        OutcomeKind::AmbiguousTarget
        | OutcomeKind::MissingTarget
        | OutcomeKind::InvalidTarget
        | OutcomeKind::EditorUnavailable
        | OutcomeKind::DeniedAccess
        | OutcomeKind::InvalidRequest => 3,
        OutcomeKind::DisconnectedEditor
        | OutcomeKind::Timeout
        | OutcomeKind::ProtocolError
        | OutcomeKind::Cancelled => 4,
    }
}

fn failure(outcome: OutcomeKind, code: DiagnosticCode, stage: Stage) -> RoutingFailure {
    RoutingFailure {
        outcome,
        diagnostic: Diagnostic::new(code, stage, None),
        selection: None,
    }
}
fn protocol_failure() -> RoutingFailure {
    failure(
        OutcomeKind::ProtocolError,
        DiagnosticCode::InvalidFrame,
        Stage::Finalize,
    )
}
fn timeout_failure(stage: Stage) -> RoutingFailure {
    failure(
        OutcomeKind::Timeout,
        DiagnosticCode::DeadlineExceeded,
        stage,
    )
}
fn signal(error: RoutingFailure) -> Result<(TerminalFailure, Diagnostic), RoutingFailure> {
    let terminal = match error.outcome {
        OutcomeKind::AmbiguousTarget => {
            TerminalFailure::AmbiguousTarget(error.selection.ok_or_else(protocol_failure)?)
        }
        OutcomeKind::MissingTarget => TerminalFailure::MissingTarget,
        OutcomeKind::InvalidTarget => TerminalFailure::InvalidTarget,
        OutcomeKind::EditorUnavailable => TerminalFailure::EditorUnavailable,
        OutcomeKind::DisconnectedEditor => TerminalFailure::DisconnectedEditor,
        OutcomeKind::Timeout => TerminalFailure::Timeout,
        OutcomeKind::UnsupportedObservation => TerminalFailure::UnsupportedObservation,
        OutcomeKind::DeniedAccess => TerminalFailure::DeniedAccess,
        OutcomeKind::InvalidRequest => TerminalFailure::InvalidRequest,
        OutcomeKind::ProtocolError => TerminalFailure::ProtocolError,
        OutcomeKind::Cancelled => TerminalFailure::Cancelled,
        _ => return Err(protocol_failure()),
    };
    Ok((terminal, error.diagnostic))
}

#[derive(PartialEq, Eq)]
enum Expected {
    Selected,
    Sample,
    Disk,
    Recheck,
    DiskCheck,
    Done,
}
struct Collected {
    target: Option<ResolvedTarget>,
    sample: Option<Box<EditorSample>>,
    disk: Option<SourceObservation>,
    disk_metadata: Option<project_fs::DiskMetadata>,
    recheck: Recheck,
    disk_checked: bool,
    expected: Expected,
    signals: Vec<TerminalFailure>,
    diagnostics: Vec<Diagnostic>,
}
impl Collected {
    fn new() -> Self {
        Self {
            target: None,
            sample: None,
            disk: None,
            disk_metadata: None,
            recheck: Recheck::unavailable(RecheckReason::Unavailable),
            expected: Expected::Selected,
            disk_checked: false,
            signals: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
    fn fail(&mut self, error: RoutingFailure) {
        let (terminal, diagnostic) = signal(error).unwrap_or_else(|_| {
            (
                TerminalFailure::ProtocolError,
                Diagnostic::new(DiagnosticCode::InvalidFrame, Stage::Finalize, None),
            )
        });
        self.signals.push(terminal);
        self.diagnostics.push(diagnostic);
    }
    // Ordering is a boundary invariant: no disk before selected/sample, no duplicated
    // source, and no success before both rechecks. A terminal event never gets upgraded.
    fn accept(&mut self, event: Event) -> Result<bool, RoutingFailure> {
        match event {
            Event::Selected(target) if self.expected == Expected::Selected => {
                self.target = Some(target);
                self.expected = Expected::Sample;
            }
            Event::Sample(mut sample) if self.expected == Expected::Sample => {
                self.diagnostics.append(&mut sample.diagnostics);
                self.sample = Some(sample);
                self.expected = Expected::Disk;
            }
            Event::Disk(source) if self.expected == Expected::Disk => {
                self.disk = Some(source);
                self.expected = Expected::Recheck;
            }
            Event::DiskMetadata {
                reason,
                file,
                collection,
            } if self.expected == Expected::Disk => {
                self.disk = Some(
                    SourceObservation::unavailable(Authority::D, reason)
                        .map_err(|_| protocol_failure())?,
                );
                self.disk_metadata = Some(project_fs::DiskMetadata { file, collection });
                self.expected = Expected::Recheck;
            }
            Event::Rechecked(recheck) if self.expected == Expected::Recheck => {
                self.recheck = recheck;
                self.expected = Expected::DiskCheck;
            }
            Event::DiskChecked(mut changes) if self.expected == Expected::DiskCheck => {
                match &mut self.recheck {
                    Recheck::Performed { detected_changes }
                    | Recheck::Partial {
                        detected_changes, ..
                    } => detected_changes.append(&mut changes),
                    Recheck::Unavailable { reason } => {
                        self.recheck = Recheck::partial(*reason, changes)
                    }
                }
                self.disk_checked = true;
                self.expected = Expected::Done;
            }
            Event::Failed(error) => {
                self.fail(error);
                return Ok(true);
            }
            Event::Done if self.expected == Expected::Done => return Ok(true),
            _ => return Err(protocol_failure()),
        }
        Ok(false)
    }
    fn finish(
        mut self,
        request: ObservationRequest,
        clock: AttemptClock,
    ) -> Result<ObservationOutcome, HostFailure> {
        let request = match &self.target {
            Some(target) => ObservationRequest::new(
                request.request_id().clone(),
                target.project_root().clone(),
                request.session_id().cloned(),
                request.script_path().clone(),
            ),
            None => request,
        };
        let interval = clock.interval();
        if !self.disk_checked {
            let reason = if self.signals.contains(&TerminalFailure::DisconnectedEditor) {
                RecheckReason::SessionEnded
            } else if self.signals.contains(&TerminalFailure::Timeout) {
                RecheckReason::DeadlineExceeded
            } else {
                RecheckReason::Unavailable
            };
            self.recheck = match self.recheck {
                Recheck::Performed { detected_changes }
                | Recheck::Partial {
                    detected_changes, ..
                } => Recheck::partial(reason, detected_changes),
                Recheck::Unavailable { reason: original } => {
                    Recheck::unavailable(if reason == RecheckReason::Unavailable {
                        original
                    } else {
                        reason
                    })
                }
            };
        }
        let evidence = match (self.sample, self.target.as_ref()) {
            (Some(sample), Some(target)) => {
                let reason = if request.script_path().kind() == Some(ScriptKind::BuiltinGdscript) {
                    SourceReason::NoStandaloneDiskSource
                } else if self.signals.contains(&TerminalFailure::Timeout) {
                    SourceReason::DeadlineExceeded
                } else {
                    SourceReason::DiskUnreadable
                };
                let disk = self.disk.unwrap_or_else(|| {
                    SourceObservation::unavailable(Authority::D, reason).expect("valid D reason")
                });
                let document =
                    bind_disk_identity(sample.document, &disk, self.disk_metadata.as_ref(), target)
                        .map_err(|_| HostFailure)?;
                let sources =
                    Sources::new(disk, sample.resource, sample.buffer).map_err(|_| HostFailure)?;
                Some(ObservationEvidence::new(
                    target.clone(),
                    document,
                    sources,
                    sample.dirty,
                    self.recheck,
                ))
            }
            _ => None,
        };
        // All received evidence is validated before retention. A final invariant failure
        // is still fail-closed, never a panic or disclosure of the rejected snapshot.
        let fallback_request = request.clone();
        let fallback_interval = interval.clone();
        let fallback_target = self.target.clone();
        match ObservationOutcome::classify(
            request,
            interval,
            self.target,
            evidence,
            self.signals,
            self.diagnostics,
        ) {
            Ok(outcome) => Ok(outcome),
            Err(_) => {
                self.diagnostics = vec![Diagnostic::new(
                    DiagnosticCode::InvalidFrame,
                    Stage::Finalize,
                    None,
                )];
                ObservationOutcome::classify(
                    fallback_request,
                    fallback_interval,
                    fallback_target,
                    None,
                    vec![TerminalFailure::ProtocolError],
                    self.diagnostics,
                )
                .map_err(|_| HostFailure)
            }
        }
    }
}
fn bind_disk_identity(
    document: DocumentState,
    disk: &SourceObservation,
    disk_identity: Option<&project_fs::DiskMetadata>,
    target: &ResolvedTarget,
) -> Result<DocumentState, EvidenceError> {
    let disk_id = disk_identity
        .and_then(|metadata| metadata.file.as_ref())
        .or_else(|| {
            disk.witness().and_then(Witness::disk_file_id).or_else(|| {
                disk.invalidated_evidence()
                    .and_then(|old| old.witness().disk_file_id())
            })
        });
    let identity = if let Some(id) = document.identity() {
        Some(DocumentIdentity::new(
            id.kind(),
            id.resource_path().clone(),
            id.script_instance_id().cloned(),
            id.editor_instance_id().cloned(),
            id.buffer_instance_id().cloned(),
            disk_id.cloned(),
        )?)
    } else if disk_id.is_some() {
        Some(DocumentIdentity::new(
            ScriptKind::ExternalGdscript,
            target.script_path().clone(),
            None,
            None,
            None,
            disk_id.cloned(),
        )?)
    } else {
        None
    };
    let validity = if document.open_state().value() == Some(&OpenState::NotOpen)
        && document.validity().value().is_none()
        && target.script_path().kind() == Some(ScriptKind::ExternalGdscript)
    {
        if let Some(metadata) = disk_identity {
            let validity = if metadata.file.is_some() {
                Validity::Valid
            } else {
                Validity::Missing
            };
            DocumentFact::observed(validity, metadata.collection.clone())
        } else {
            disk.collection().map_or_else(
                || document.validity().clone(),
                |stamp| DocumentFact::observed(Validity::Valid, stamp.clone()),
            )
        }
    } else {
        document.validity().clone()
    };
    Ok(DocumentState::new(
        identity,
        validity,
        document.open_state().clone(),
    ))
}

struct OwnedWorker(Option<Child>);

// Reaping (and stock worker's private-dir destruction after an interrupted
// worker) must never run in a request delivery thread.
fn reap_detached(mut child: Child, cleanup: Option<PathBuf>) -> Option<mpsc::Receiver<bool>> {
    if cleanup.is_none() && matches!(child.try_wait(), Ok(Some(_))) {
        return None;
    }
    let _ = child.kill();
    struct ReapJob {
        child: Child,
        cleanup: Option<(PathBuf, mpsc::Sender<bool>)>,
    }
    static REAPER: LazyLock<Option<mpsc::Sender<ReapJob>>> = LazyLock::new(|| {
        let (send, receive) = mpsc::channel::<ReapJob>();
        thread::Builder::new()
            .name("worker-reaper".into())
            .spawn(move || {
                let mut children: Vec<ReapJob> = Vec::new();
                loop {
                    match receive.recv_timeout(Duration::from_millis(10)) {
                        Ok(job) => children.push(job),
                        Err(mpsc::RecvTimeoutError::Disconnected) if children.is_empty() => break,
                        Err(_) => {}
                    }
                    children.retain_mut(|job| {
                        if !matches!(job.child.try_wait(), Ok(Some(_))) {
                            return true;
                        }
                        if let Some((path, notify)) = job.cleanup.take() {
                            let removed = match std::fs::remove_dir_all(path) {
                                Ok(()) => true,
                                Err(error) => error.kind() == io::ErrorKind::NotFound,
                            };
                            let _ = notify.send(removed);
                        }
                        false
                    });
                }
            })
            .ok()
            .map(|_| send)
    });
    let (cleanup, receipt) = match cleanup {
        Some(path) => {
            let (notify, receipt) = mpsc::channel();
            (Some((path, notify)), Some(receipt))
        }
        None => (None, None),
    };
    if let Some(reaper) = REAPER.as_ref() {
        let _ = reaper.send(ReapJob { child, cleanup });
    }
    // A killed child is adopted on caller exit if the host cannot start the reaper.
    receipt
}
impl Drop for OwnedWorker {
    fn drop(&mut self) {
        if let Some(child) = self.0.take() {
            let _ = reap_detached(child, None);
        }
    }
}

/// Execute one observation with the same trusted executable's private worker mode.
///
/// The executable must dispatch `INTERNAL_WORKER_FLAG` to [`worker_main`]. The
/// caller must drain the result and keep `cancelled` alive for this call. No source
/// acquisition takes place in the supervisor, including on the timeout path.
///
/// # Errors
/// Returns `HostFailure` when the fixed worker/socket cannot be launched or the
/// host cannot represent a terminal result. Normal refusals are structured outcomes.
pub fn run(
    request: ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<ObservationOutcome, HostFailure> {
    if cancelled.load(Ordering::Relaxed) || clock.started.elapsed() >= COLLECTION_BUDGET {
        let mut state = Collected::new();
        state.fail(if cancelled.load(Ordering::Relaxed) {
            failure(
                OutcomeKind::Cancelled,
                DiagnosticCode::Cancelled,
                Stage::Finalize,
            )
        } else {
            timeout_failure(Stage::ResolveTarget)
        });
        return state.finish(request, clock);
    }
    let (mut parent, child_socket) = UnixStream::pair().map_err(|_| HostFailure)?;
    parent.set_nonblocking(true).map_err(|_| HostFailure)?;
    let input: OwnedFd = child_socket.try_clone().map_err(|_| HostFailure)?.into();
    let output: OwnedFd = child_socket.into();
    let child = Command::new(std::env::current_exe().map_err(|_| HostFailure)?)
        .arg(INTERNAL_WORKER_FLAG)
        .stdin(Stdio::from(input))
        .stdout(Stdio::from(output))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| HostFailure)?;
    let worker = OwnedWorker(Some(child));
    // The tuple avoids optional/duplicate object-key ambiguity on private startup.
    let payload = serde_json::to_vec(&(
        1u32,
        request.request_id().as_str(),
        request.project_root().as_str(),
        request.session_id().map(SessionId::as_str),
        request.script_path().as_str(),
        registry.to_str().ok_or(HostFailure)?,
        clock.elapsed_us(),
    ))
    .map_err(|_| HostFailure)?;
    if payload.len() > CONTROL_LIMIT {
        let mut state = Collected::new();
        state.fail(failure(
            OutcomeKind::InvalidRequest,
            DiagnosticCode::InvalidRequest,
            Stage::ValidateRequest,
        ));
        return state.finish(request, clock);
    }
    let header = (payload.len() as u32).to_be_bytes();
    let mut sent = 0;
    let deadline = clock.started + COLLECTION_BUDGET;
    while sent < 4 + payload.len()
        && Instant::now() < deadline
        && !cancelled.load(Ordering::Relaxed)
    {
        let part = if sent < 4 {
            &header[sent..]
        } else {
            &payload[sent - 4..]
        };
        match parent.write(part) {
            Ok(0) => return Err(HostFailure),
            Ok(count) => sent += count,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL_INTERVAL),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(HostFailure),
        }
    }
    supervise(parent, worker, request, clock, cancelled)
}

#[derive(Default)]
struct Frames {
    header: [u8; 4],
    header_used: usize,
    body: Vec<u8>,
    used: usize,
}
impl Frames {
    fn next(&mut self, input: &mut UnixStream) -> Result<Option<Vec<u8>>, RoutingFailure> {
        let mut budget = 64 * 1024;
        while budget > 0 {
            let buffer = if self.header_used < 4 {
                &mut self.header[self.header_used..]
            } else {
                let end = (self.used + budget).min(self.body.len());
                &mut self.body[self.used..end]
            };
            match input.read(buffer) {
                Ok(0) => return Err(protocol_failure()),
                Ok(count) => {
                    budget = budget.saturating_sub(count);
                    if self.header_used < 4 {
                        self.header_used += count;
                        if self.header_used == 4 {
                            let length = u32::from_be_bytes(self.header) as usize;
                            if length == 0 || length > wire::RESULT_LIMIT {
                                return Err(protocol_failure());
                            }
                            self.body.resize(length, 0);
                        }
                    } else {
                        self.used += count;
                        if self.used == self.body.len() {
                            self.header_used = 0;
                            self.used = 0;
                            return Ok(Some(std::mem::take(&mut self.body)));
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => return Err(protocol_failure()),
            }
        }
        Ok(None)
    }
}
fn supervise(
    mut input: UnixStream,
    worker: OwnedWorker,
    request: ObservationRequest,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<ObservationOutcome, HostFailure> {
    let mut state = Collected::new();
    let mut frames = Frames::default();
    loop {
        if cancelled.load(Ordering::Relaxed) {
            state.fail(failure(
                OutcomeKind::Cancelled,
                DiagnosticCode::Cancelled,
                Stage::Finalize,
            ));
            break;
        }
        if clock.started.elapsed() >= COLLECTION_BUDGET {
            state.fail(timeout_failure(if state.target.is_none() {
                Stage::ResolveTarget
            } else {
                Stage::Finalize
            }));
            break;
        }
        match frames.next(&mut input) {
            Ok(Some(bytes)) => {
                // Bytes arriving after the collection cutoff never become evidence.
                if clock.started.elapsed() >= COLLECTION_BUDGET {
                    state.fail(timeout_failure(Stage::Finalize));
                    break;
                }
                let event =
                    wire::decode_event(&bytes, &request, state.target.as_ref(), clock.elapsed_us());
                if clock.started.elapsed() >= COLLECTION_BUDGET {
                    state.fail(timeout_failure(Stage::Finalize));
                    break;
                }
                if cancelled.load(Ordering::Relaxed) {
                    state.fail(failure(
                        OutcomeKind::Cancelled,
                        DiagnosticCode::Cancelled,
                        Stage::Finalize,
                    ));
                    break;
                }
                match event.and_then(|event| state.accept(event)) {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(error) => {
                        state.fail(error);
                        break;
                    }
                }
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                state.fail(error);
                break;
            }
        }
    }
    drop(input);
    drop(worker);
    state.finish(request, clock)
}

fn send_event(
    output: &mut impl Write,
    event: &Event,
    id: &RequestId,
) -> Result<(), RoutingFailure> {
    let bytes = wire::encode_event(event, id)?;
    if bytes.len() > wire::RESULT_LIMIT {
        return Err(protocol_failure());
    }
    output
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| output.write_all(&bytes))
        .and_then(|_| output.flush())
        .map_err(|_| protocol_failure())
}
fn collect(
    request: &ObservationRequest,
    registry: &Path,
    started: Instant,
    deadline: Instant,
    output: &mut impl Write,
) -> Result<(), RoutingFailure> {
    let mut selected = target::resolve(request, registry, deadline)?;
    send_event(
        output,
        &Event::Selected(selected.target().clone()),
        request.request_id(),
    )?;
    if !selected.capabilities().observe_gdscript {
        return Err(failure(
            OutcomeKind::UnsupportedObservation,
            DiagnosticCode::UnsupportedCapability,
            Stage::ReadEditor,
        ));
    }
    let (sample_collection, sample_validity) = {
        let event = Event::Sample(Box::new(wire::observe(
            &mut selected,
            request,
            started,
            deadline,
        )?));
        send_event(output, &event, request.request_id())?;
        let Event::Sample(sample) = event else {
            unreachable!()
        };
        // Retain only attribution and interval; release source strings before D.
        (
            sample.collection,
            sample.document.validity().value().copied(),
        )
    };
    if Instant::now() >= deadline {
        return Err(timeout_failure(Stage::ReadDisk));
    }
    let non_gd_validity = sample_validity.filter(|v| {
        selected.target().script_path().kind().is_none()
            && matches!(v, Validity::Missing | Validity::Invalid)
    });
    let disk = if let Some(validity) = non_gd_validity {
        project_fs::non_gd_document_disk(&selected, validity, Stage::ReadDisk)?
    } else {
        project_fs::read_disk_with_metadata(&selected, started)?
    };
    let event = match disk.metadata {
        Some(project_fs::DiskMetadata { file, collection }) => Event::DiskMetadata {
            reason: disk.source.reason().ok_or_else(protocol_failure)?,
            file,
            collection,
        },
        None => Event::Disk(disk.source),
    };
    send_event(output, &event, request.request_id())?;
    let recheck = wire::recheck(
        &mut selected,
        request,
        &sample_collection,
        started,
        deadline,
    )?;
    send_event(output, &Event::Rechecked(recheck), request.request_id())?;
    if Instant::now() >= deadline {
        return Err(timeout_failure(Stage::Recheck));
    }
    let disk = match event {
        Event::Disk(source) => project_fs::DiskRead {
            source,
            metadata: None,
            mtime: None,
            access_denied: false,
        },
        Event::DiskMetadata {
            reason,
            file,
            collection,
        } => project_fs::DiskRead {
            source: SourceObservation::unavailable(Authority::D, reason)
                .map_err(|_| protocol_failure())?,
            metadata: Some(project_fs::DiskMetadata { file, collection }),
            mtime: None,
            access_denied: false,
        },
        _ => unreachable!(),
    };
    let disk_changes = if let Some(validity) = non_gd_validity {
        project_fs::non_gd_document_disk(&selected, validity, Stage::Recheck)?;
        Vec::new()
    } else {
        project_fs::recheck_disk_read(&selected, &disk, started)?
    };
    if Instant::now() >= deadline {
        return Err(timeout_failure(Stage::Recheck));
    }
    send_event(
        output,
        &Event::DiskChecked(disk_changes),
        request.request_id(),
    )?;
    send_event(output, &Event::Done, request.request_id())
}

/// Dispatch the private worker only when stdin/stdout are connected socket descriptors.
/// A normal shell invocation, including redirected pipes, is not a product operation.
/// Returns `None` for a non-private invocation, otherwise the internal worker exit code.
pub fn worker_main() -> Option<i32> {
    let input_fd = io::stdin().as_fd().try_clone_to_owned().ok()?;
    let output_fd = io::stdout().as_fd().try_clone_to_owned().ok()?;
    let mut input = UnixStream::from(input_fd);
    let mut output = UnixStream::from(output_fd);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    let mut receive = || -> Result<_, ()> {
        input
            .set_read_timeout(Some(COLLECTION_BUDGET))
            .map_err(|_| ())?;
        let mut length = [0; 4];
        input.read_exact(&mut length).map_err(|_| ())?;
        let length = u32::from_be_bytes(length) as usize;
        if length == 0 || length > CONTROL_LIMIT {
            return Err(());
        }
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes).map_err(|_| ())?;
        type Startup = (u32, String, String, Option<String>, String, String, u64);
        let (version, id, root, session, locator, registry, elapsed): Startup =
            serde_json::from_slice(&bytes).map_err(|_| ())?;
        if version != 1
            || elapsed > 4_500_000
            || registry.len() > 1024
            || !Path::new(&registry).is_absolute()
        {
            return Err(());
        }
        let request = ObservationRequest::new(
            RequestId::new(id).map_err(|_| ())?,
            ProjectRoot::new(root).map_err(|_| ())?,
            session.map(SessionId::new).transpose().map_err(|_| ())?,
            ResourcePath::new(locator).map_err(|_| ())?,
        );
        Ok((request, registry, elapsed))
    };
    let Ok((request, registry, elapsed)) = receive() else {
        return Some(1);
    };
    // The offset establishes this worker's request-local caller tick origin. Supervisor
    // receipt stamps, not worker timing or wall time, govern acceptance and finalization.
    let now = Instant::now();
    let started = now
        .checked_sub(Duration::from_micros(elapsed))
        .unwrap_or(now);
    let deadline = started + COLLECTION_BUDGET;
    let _ = output.set_write_timeout(Some(COLLECTION_BUDGET));
    match collect(
        &request,
        Path::new(&registry),
        started,
        deadline,
        &mut output,
    ) {
        Ok(()) => Some(0),
        Err(error) => Some(
            if send_event(&mut output, &Event::Failed(error), request.request_id()).is_ok() {
                0
            } else {
                1
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ObservationRequest {
        ObservationRequest::new(
            RequestId::new("owned_worker_test").unwrap(),
            ProjectRoot::new("/fixture").unwrap(),
            None,
            ResourcePath::new("res://subject.gd").unwrap(),
        )
    }
    fn blocked() -> (UnixStream, UnixStream, OwnedWorker) {
        let (parent, peer) = UnixStream::pair().unwrap();
        parent.set_nonblocking(true).unwrap();
        // Test-only owned stalled process. No product caller accepts this executable.
        let child = Command::new("/bin/sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        (parent, peer, OwnedWorker(Some(child)))
    }
    #[test]
    fn blocked_owned_worker_cannot_hold_the_terminal_result() {
        let clock = AttemptClock::start();
        let (parent, _peer, worker) = blocked();
        let outcome = supervise(parent, worker, request(), clock, &AtomicBool::new(false)).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::Timeout);
        assert!(clock.started.elapsed() >= COLLECTION_BUDGET);
        assert!(clock.started.elapsed() < Duration::from_secs(5));
    }
    #[test]
    fn cancellation_is_terminal_without_waiting_for_blocked_work() {
        let clock = AttemptClock::start();
        let (parent, _peer, worker) = blocked();
        let outcome = supervise(parent, worker, request(), clock, &AtomicBool::new(true)).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::Cancelled);
        assert!(clock.started.elapsed() < Duration::from_secs(1));
    }
    #[test]
    fn malformed_or_premature_worker_frames_cannot_become_success() {
        for bytes in [
            vec![0xff; 4],
            vec![0, 0, 0, 2, b'{'],
            vec![0, 0, 0, 2, b'{', b'}'],
        ] {
            let clock = AttemptClock::start();
            let (parent, mut peer, worker) = blocked();
            peer.write_all(&bytes).unwrap();
            drop(peer);
            let outcome =
                supervise(parent, worker, request(), clock, &AtomicBool::new(false)).unwrap();
            assert_eq!(outcome.outcome(), OutcomeKind::ProtocolError);
            assert!(outcome.snapshot().is_none());
        }
    }
    #[test]
    fn late_terminal_frames_are_not_promoted_after_cutoff() {
        let mut clock = AttemptClock::start();
        clock.started -= COLLECTION_BUDGET;
        let (parent, mut peer, worker) = blocked();
        send_event(
            &mut peer,
            &Event::Failed(failure(
                OutcomeKind::EditorUnavailable,
                DiagnosticCode::EditorUnavailable,
                Stage::ResolveTarget,
            )),
            request().request_id(),
        )
        .unwrap();
        let outcome = supervise(parent, worker, request(), clock, &AtomicBool::new(false)).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::Timeout);
    }
    #[test]
    fn queued_evidence_after_terminal_loss_cannot_upgrade_an_attributed_snapshot() {
        let clock = AttemptClock::start();
        let (parent, mut peer, worker) = blocked();
        let (target, sample) = selected_sample();
        for event in [
            Event::Selected(target),
            Event::Sample(Box::new(sample)),
            Event::Disk(
                SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing).unwrap(),
            ),
            Event::Failed(failure(
                OutcomeKind::DisconnectedEditor,
                DiagnosticCode::SessionEnded,
                Stage::Recheck,
            )),
            // Already queued in the socket, but never validated or retained.
            Event::Rechecked(Recheck::performed(vec![])),
            Event::DiskChecked(vec![]),
            Event::Done,
        ] {
            send_event(&mut peer, &event, request().request_id()).unwrap();
        }
        let outcome = supervise(parent, worker, request(), clock, &AtomicBool::new(false)).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::DisconnectedEditor);
        let snapshot = outcome.snapshot().unwrap();
        assert_eq!(snapshot.consistency().checks(), Checks::Unavailable);
        assert_eq!(
            snapshot.consistency().recheck_reason(),
            Some(RecheckReason::SessionEnded)
        );
        assert!(snapshot.sources().resource().text().is_none());
        assert_eq!(
            snapshot
                .sources()
                .resource()
                .invalidated_evidence()
                .unwrap()
                .text(),
            "R evidence"
        );
        assert_eq!(
            snapshot.sources().disk().reason(),
            Some(SourceReason::DiskMissing)
        );
    }
    fn selected_sample() -> (ResolvedTarget, EditorSample) {
        // Boundary evidence only, not a Godot observability oracle.
        let session = SessionId::new("00112233445566778899aabbccddeeff").unwrap();
        let decimal = |n: &str| DecimalCounter::new(n).unwrap();
        let target = ResolvedTarget::for_request(
            &request(),
            request().project_root().clone(),
            FileIdentity::new(decimal("1"), decimal("2")),
            session.clone(),
            EngineVersion::new(
                "4.7.2.stable.official.ed1daf0bf",
                "ed1daf0bf001b61586d9930840f2f1394092c079",
            )
            .unwrap(),
        )
        .unwrap();
        let stamp =
            CollectionStamp::new(ClockId::Editor(session), decimal("1"), decimal("2"), 0).unwrap();
        let id = DocumentIdentity::new(
            ScriptKind::ExternalGdscript,
            request().script_path().clone(),
            Some(decimal("10")),
            Some(decimal("11")),
            Some(decimal("12")),
            None,
        )
        .unwrap();
        let resource_witness = Witness::new(
            request().script_path().clone(),
            Some(decimal("10")),
            None,
            None,
            None,
            None,
        );
        let buffer_witness = Witness::new(
            request().script_path().clone(),
            Some(decimal("10")),
            Some(decimal("11")),
            Some(decimal("12")),
            None,
            Some(decimal("3")),
        );
        (
            target,
            EditorSample {
                collection: stamp.clone(),
                document: DocumentState::new(
                    Some(id),
                    DocumentFact::observed(Validity::Valid, stamp.clone()),
                    DocumentFact::observed(OpenState::Open, stamp.clone()),
                ),
                resource: SourceObservation::observed(
                    Authority::R,
                    "R evidence".into(),
                    stamp.clone(),
                    resource_witness,
                    Staleness::unknown(),
                ),
                buffer: SourceObservation::observed(
                    Authority::B,
                    "B evidence".into(),
                    stamp.clone(),
                    buffer_witness.clone(),
                    Staleness::unknown(),
                ),
                dirty: DirtyObservation::observed(DirtyState::Dirty, stamp, buffer_witness),
                diagnostics: vec![],
            },
        )
    }
    #[test]
    fn unexpected_worker_stage_retains_only_previously_validated_evidence() {
        let clock = AttemptClock::start();
        let (target, sample) = selected_sample();
        let mut state = Collected::new();
        state.accept(Event::Selected(target)).unwrap();
        state.accept(Event::Sample(Box::new(sample))).unwrap();
        let error = state
            .accept(Event::Rechecked(Recheck::performed(vec![])))
            .unwrap_err();
        state.fail(error);
        let outcome = state.finish(request(), clock).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::ProtocolError);
        let snapshot = outcome.snapshot().unwrap();
        assert_eq!(snapshot.sources().resource().text(), Some("R evidence"));
        assert_eq!(snapshot.sources().buffer().text(), Some("B evidence"));
        assert_eq!(snapshot.sources().disk().text(), None);
        assert_eq!(snapshot.consistency().checks(), Checks::Unavailable);
    }
    #[test]
    fn scope_denial_after_sample_discards_every_source() {
        let clock = AttemptClock::start();
        let (target, sample) = selected_sample();
        let mut state = Collected::new();
        state.accept(Event::Selected(target)).unwrap();
        state.accept(Event::Sample(Box::new(sample))).unwrap();
        state
            .accept(Event::Failed(failure(
                OutcomeKind::DeniedAccess,
                DiagnosticCode::OutOfProject,
                Stage::ReadDisk,
            )))
            .unwrap();
        let outcome = state.finish(request(), clock).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::DeniedAccess);
        assert!(outcome.snapshot().is_none());
        assert!(outcome.resolved_target().is_none());
    }
    #[test]
    fn editor_changes_survive_an_interrupted_independent_disk_recheck() {
        let clock = AttemptClock::start();
        let (target, sample) = selected_sample();
        let mut state = Collected::new();
        state.accept(Event::Selected(target)).unwrap();
        state.accept(Event::Sample(Box::new(sample))).unwrap();
        state
            .accept(Event::Disk(
                SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing).unwrap(),
            ))
            .unwrap();
        state
            .accept(Event::Rechecked(Recheck::performed(vec![
                DetectedChange::Source(Authority::B),
            ])))
            .unwrap();
        state.fail(timeout_failure(Stage::Recheck));
        let outcome = state.finish(request(), clock).unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::Timeout);
        let snapshot = outcome.snapshot().unwrap();
        assert_eq!(snapshot.consistency().checks(), Checks::Unavailable);
        assert_eq!(snapshot.sources().resource().text(), Some("R evidence"));
        assert_eq!(snapshot.sources().buffer().text(), None);
        assert_eq!(
            snapshot
                .sources()
                .buffer()
                .invalidated_evidence()
                .unwrap()
                .text(),
            "B evidence"
        );
    }
    #[test]
    fn selected_lifetime_loss_at_each_validated_stage_never_claims_live_completion() {
        let disk = || {
            SourceObservation::observed(
                Authority::D,
                "D evidence".into(),
                CollectionStamp::new(
                    ClockId::Caller,
                    DecimalCounter::new("0").unwrap(),
                    DecimalCounter::new("0").unwrap(),
                    0,
                )
                .unwrap(),
                Witness::new(
                    request().script_path().clone(),
                    None,
                    None,
                    None,
                    Some(FileIdentity::new(
                        DecimalCounter::new("3").unwrap(),
                        DecimalCounter::new("4").unwrap(),
                    )),
                    None,
                ),
                Staleness::unknown(),
            )
        };
        // Before sample, after sample, after D, after a validated editor change,
        // and after both checks but before Done. No stage can convert known loss
        // into complete or a confirmed closed-document result.
        for stage in 0..5 {
            let clock = AttemptClock::start();
            let (target, sample) = selected_sample();
            let mut state = Collected::new();
            state.accept(Event::Selected(target)).unwrap();
            if stage >= 1 {
                state.accept(Event::Sample(Box::new(sample))).unwrap();
            }
            if stage >= 2 {
                state.accept(Event::Disk(disk())).unwrap();
            }
            if stage >= 3 {
                state
                    .accept(Event::Rechecked(Recheck::performed(vec![
                        DetectedChange::Source(Authority::B),
                    ])))
                    .unwrap();
            }
            if stage >= 4 {
                state.accept(Event::DiskChecked(vec![])).unwrap();
            }
            assert!(state
                .accept(Event::Failed(failure(
                    OutcomeKind::DisconnectedEditor,
                    DiagnosticCode::SessionEnded,
                    if stage == 0 {
                        Stage::ReadEditor
                    } else {
                        Stage::Recheck
                    },
                )))
                .unwrap());
            let outcome = state.finish(request(), clock).unwrap();
            assert_eq!(
                outcome.outcome(),
                OutcomeKind::DisconnectedEditor,
                "stage {stage}"
            );
            assert_eq!(
                outcome.resolved_target().unwrap().session_id().as_str(),
                "00112233445566778899aabbccddeeff"
            );
            if stage == 0 {
                assert!(outcome.snapshot().is_none());
                continue;
            }
            let snapshot = outcome.snapshot().unwrap();
            assert_eq!(
                snapshot.sources().disk().text(),
                (stage >= 2).then_some("D evidence"),
                "stage {stage}"
            );
            assert_eq!(snapshot.sources().resource().text(), None, "stage {stage}");
            assert_eq!(snapshot.sources().buffer().text(), None, "stage {stage}");
            assert_eq!(
                snapshot
                    .sources()
                    .resource()
                    .invalidated_evidence()
                    .unwrap()
                    .text(),
                "R evidence"
            );
            assert_eq!(
                snapshot
                    .sources()
                    .buffer()
                    .invalidated_evidence()
                    .unwrap()
                    .text(),
                "B evidence"
            );
            assert_eq!(
                snapshot.comparisons().disk_resource(),
                crate::observation::Comparison::Unknown
            );
            assert_eq!(snapshot.document().open_state().value(), None);
            assert!(snapshot
                .consistency()
                .detected_changes()
                .contains(&DetectedChange::SessionEnded));
            if stage == 3 {
                assert_eq!(snapshot.consistency().checks(), Checks::Unavailable);
                assert!(snapshot
                    .consistency()
                    .detected_changes()
                    .contains(&DetectedChange::Source(Authority::B)));
            }
        }
    }
}
