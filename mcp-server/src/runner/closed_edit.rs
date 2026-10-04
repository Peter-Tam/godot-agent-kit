//! Bounded owned acquisition/edit workers; parent alone releases irreversible authority.
use super::stock_validation;
use super::{AttemptClock, HostFailure, OwnedWorker};
use crate::bridge::wire::closed_edit as codec;
use crate::observation::*;
use crate::project_fs_validation as confined;
use crate::script_closed_edit::{
    CheckedClosedRequest, ClosedExpectedBasis, ClosedOutcome, Effects, FileRevision, NativeReceipt,
    State,
};
use crate::target::{self, RoutingFailure};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
#[path = "closed_edit/supervisor.rs"]
mod supervisor;
#[path = "closed_edit/worker.rs"]
mod worker;
pub(crate) use supervisor::run;
pub(crate) use supervisor::{acquire, acquire_for_edit};
pub use worker::{acquisition_worker_main, worker_main};
pub const INTERNAL_WORKER_FLAG: &str = "--internal-closed-edit-worker";
pub const INTERNAL_ACQUISITION_WORKER_FLAG: &str = "--internal-closed-acquisition-worker";
const EDIT_BUDGET: Duration = Duration::from_millis(9500);
const READ_BUDGET: Duration = Duration::from_millis(4500);
const LIMIT: usize = 4 * SOURCE_LIMIT_BYTES + 262144;
#[derive(Debug)]
pub(crate) enum AcquisitionFailure {
    Host,
    DeniedAccess,
}
pub(crate) struct Acquisition {
    pub observed: Option<State>,
    pub eligible: Option<ClosedExpectedBasis>,
    pub reason: Option<String>,
    pub changes: Vec<DetectedChange>,
    pub lifecycle_changed: bool,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureChanges {
    disk: bool,
    resource: bool,
    lifecycle: bool,
    context: bool,
    context_unavailable: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Startup {
    v: u32,
    id: String,
    project: String,
    session: Option<String>,
    path: String,
    registry: PathBuf,
    accepted_unix_ms: u64,
    elapsed_us: u64,
    budget_us: u64,
    basis: Option<ClosedExpectedBasis>,
    source: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Message {
    Acquired {
        state: State,
        changes: CaptureChanges,
    },
    Helper {
        role: String,
        context: Box<codec::Context>,
        source: Option<String>,
    },
    Ready {},
    Native {
        receipt: NativeReceipt,
    },
    Verified {
        state: State,
        context_sha256: String,
        valid: bool,
    },
    Survivor {
        state: State,
    },
    Failed {
        reason: String,
        denied: bool,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Control {
    Authorize {},
    Abort {},
    Preflight {
        original: Box<stock_validation::ValidationResult>,
        desired: Option<Box<stock_validation::ValidationResult>>,
    },
    Validation {
        result: Box<stock_validation::ValidationResult>,
    },
}
// Match the stock validator's bounded blocking channel. Kernel readiness wakes
// promptly for large frames; no per-chunk polling sleep consumes the edit budget.
fn io_timeout(at: Instant, cancel: Option<&AtomicBool>) -> Result<Duration, &'static str> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err("cancelled");
    }
    let remaining = at.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err("timeout");
    }
    Ok(remaining.min(Duration::from_millis(50)))
}

fn send<T: Serialize>(
    s: &mut UnixStream,
    v: &T,
    at: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), &'static str> {
    let bytes = serde_json::to_vec(v).map_err(|_| "protocol_error")?;
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err("protocol_error");
    }
    let header = (bytes.len() as u32).to_be_bytes();
    for part in [&header[..], &bytes[..]] {
        let mut off = 0;
        while off < part.len() {
            s.set_write_timeout(Some(io_timeout(at, cancel)?))
                .map_err(|_| "disconnected")?;
            match s.write(&part[off..]) {
                Ok(0) => return Err("disconnected"),
                Ok(n) => off += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return Err("disconnected"),
            }
        }
    }
    Ok(())
}
fn receive<T: for<'de> Deserialize<'de>>(
    s: &mut UnixStream,
    at: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<T, &'static str> {
    let mut h = [0; 4];
    receive_exact(s, &mut h, at, cancel)?;
    let len = u32::from_be_bytes(h) as usize;
    if len == 0 || len > LIMIT {
        return Err("protocol_error");
    }
    let mut bytes = vec![0; len];
    receive_exact(s, &mut bytes, at, cancel)?;
    let value = serde_json::from_slice(&bytes).map_err(|_| "protocol_error")?;
    io_timeout(at, cancel)?;
    Ok(value)
}

fn receive_exact(
    s: &mut UnixStream,
    bytes: &mut [u8],
    at: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), &'static str> {
    let mut used = 0;
    while used < bytes.len() {
        s.set_read_timeout(Some(io_timeout(at, cancel)?))
            .map_err(|_| "disconnected")?;
        match s.read(&mut bytes[used..]) {
            Ok(0) => return Err("disconnected"),
            Ok(count) => used += count,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return Err("disconnected"),
        }
    }
    Ok(())
}

fn failure(e: &RoutingFailure) -> &'static str {
    match e.outcome {
        OutcomeKind::DeniedAccess => "denied_access",
        OutcomeKind::Timeout => "timeout",
        OutcomeKind::Cancelled => "cancelled",
        OutcomeKind::AmbiguousTarget => "ambiguous_session",
        OutcomeKind::UnsupportedObservation => "unsupported_capability",
        OutcomeKind::DisconnectedEditor => "disconnected",
        OutcomeKind::ProtocolError => "protocol_error",
        OutcomeKind::MissingTarget => "missing_target",
        OutcomeKind::InvalidTarget => "invalid_target",
        OutcomeKind::InvalidRequest => "invalid_request",
        _ => "editor_unavailable",
    }
}
fn valid(
    r: &stock_validation::ValidationResult,
    id: &str,
    session: &str,
    path: &str,
    source_hash: &str,
) -> bool {
    r.status == "valid"
        && r.reason.is_none()
        && r.request_id == id
        && r.session_id == session
        && r.root_path == path
        && r.context_sha256.is_some()
        && r.cleanup_confirmed
        && r.diagnostics.is_empty()
        && !r.sources.is_empty()
        && r.sources[0].sha256 == source_hash
        && r.sources
            .iter()
            .all(|s| s.diagnostics_completed && s.symbols_completed)
}

#[cfg(test)]
mod channel_tests {
    use super::*;
    use std::sync::{mpsc, Arc};
    use std::thread;

    #[test]
    fn source_frames_are_complete_and_keep_the_next_frame_separate() {
        let source = "𝛌\n".repeat(SOURCE_LIMIT_BYTES / 5) + "abc";
        assert_eq!(source.len(), SOURCE_LIMIT_BYTES);
        let (mut sender, mut receiver) = UnixStream::pair().unwrap();
        let at = Instant::now() + Duration::from_secs(5);
        thread::scope(|scope| {
            let writer = scope.spawn(|| {
                send(&mut sender, &source, at, None).unwrap();
                send(&mut sender, &7u8, at, None).unwrap();
            });
            assert_eq!(receive::<String>(&mut receiver, at, None).unwrap(), source);
            assert_eq!(receive::<u8>(&mut receiver, at, None).unwrap(), 7);
            writer.join().unwrap();
        });
    }

    #[test]
    fn incomplete_live_frame_exhausts_the_original_deadline() {
        let (mut sender, mut receiver) = UnixStream::pair().unwrap();
        sender.write_all(&3u32.to_be_bytes()).unwrap();
        sender.write_all(b"\"").unwrap();
        assert_eq!(
            receive::<String>(
                &mut receiver,
                Instant::now() + Duration::from_millis(30),
                None,
            ),
            Err("timeout"),
        );
    }

    #[test]
    fn stalled_source_delivery_remains_cancellable_after_bytes_are_sent() {
        let (mut sender, mut receiver) = UnixStream::pair().unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        let (done, result) = mpsc::channel();
        let writer = thread::spawn(move || {
            let source = "x".repeat(2 * SOURCE_LIMIT_BYTES);
            let outcome = send(
                &mut sender,
                &source,
                Instant::now() + Duration::from_secs(5),
                Some(&signal),
            );
            done.send(outcome).unwrap();
        });
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut prefix = [0; 5];
        receiver.read_exact(&mut prefix).unwrap();
        cancelled.store(true, Ordering::Relaxed);
        let outcome = result.recv_timeout(Duration::from_secs(1));
        drop(receiver);
        writer.join().unwrap();
        assert_eq!(outcome.unwrap(), Err("cancelled"));
    }

    #[test]
    fn invalid_frame_lengths_are_rejected_without_waiting_for_a_body() {
        for length in [0, LIMIT as u32 + 1] {
            let (mut sender, mut receiver) = UnixStream::pair().unwrap();
            sender.write_all(&length.to_be_bytes()).unwrap();
            assert_eq!(
                receive::<String>(&mut receiver, Instant::now() + Duration::from_secs(1), None,),
                Err("protocol_error"),
            );
        }
    }
}
