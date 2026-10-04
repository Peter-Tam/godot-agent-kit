//! Bounded owned acquisition/edit workers; parent alone releases irreversible authority.
use super::stock_validation;
use super::{AttemptClock, Frames, HostFailure, OwnedWorker, POLL_INTERVAL};
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
use std::thread;
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
        desired: Box<stock_validation::ValidationResult>,
    },
    Validation {
        result: Box<stock_validation::ValidationResult>,
    },
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
            if Instant::now() >= at {
                return Err("timeout");
            }
            if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                return Err("cancelled");
            }
            match s.write(&part[off..]) {
                Ok(0) => return Err("disconnected"),
                Ok(n) => off += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL_INTERVAL),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => return Err("disconnected"),
            }
        }
    }
    Ok(())
}
fn receive<T: for<'de> Deserialize<'de>>(
    s: &mut UnixStream,
    at: Instant,
) -> Result<T, &'static str> {
    if Instant::now() >= at {
        return Err("timeout");
    }
    s.set_read_timeout(Some(at.saturating_duration_since(Instant::now())))
        .map_err(|_| "disconnected")?;
    let mut h = [0; 4];
    s.read_exact(&mut h).map_err(|_| "disconnected")?;
    let len = u32::from_be_bytes(h) as usize;
    if len == 0 || len > LIMIT {
        return Err("protocol_error");
    }
    let mut bytes = vec![0; len];
    s.read_exact(&mut bytes).map_err(|_| "disconnected")?;
    serde_json::from_slice(&bytes).map_err(|_| "protocol_error")
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
        OutcomeKind::MissingTarget => "missing_script",
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
