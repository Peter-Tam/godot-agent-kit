//! Same-binary edit worker and supervisor with one irreversible authorization handoff.
use super::{protocol_failure, AttemptClock, Frames, HostFailure, OwnedWorker, POLL_INTERVAL};
use crate::bridge::wire::{self, Event};
use crate::observation::*;
use crate::project_fs;
use crate::project_fs_validation as confined;
use crate::runner::stock_validation;
use crate::script_edit::{self, EditAttempt, EditOutcome, EditRequest};
use crate::target::{self, RoutingFailure, SelectedSession};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub const INTERNAL_WORKER_FLAG: &str = "--internal-edit-worker";
const BUDGET: Duration = Duration::from_millis(9500);
const STARTUP_LIMIT: usize = 4096;
const CONTROL_LIMIT: usize = 4096;
const WORKER_LIMIT: usize = wire::RESULT_LIMIT;

#[path = "edit/ipc.rs"]
mod ipc;
#[path = "edit/supervisor.rs"]
mod supervisor;
#[path = "edit/worker.rs"]
mod worker;
pub use supervisor::run;
pub use worker::worker_main;

fn error(reason: script_edit::Reason) -> RoutingFailure {
    let (outcome, code) = match reason {
        script_edit::Reason::Deadline => (OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded),
        script_edit::Reason::Cancellation => (OutcomeKind::Cancelled, DiagnosticCode::Cancelled),
        script_edit::Reason::Disconnection => (
            OutcomeKind::DisconnectedEditor,
            DiagnosticCode::SessionEnded,
        ),
        script_edit::Reason::DeniedAccess => {
            (OutcomeKind::DeniedAccess, DiagnosticCode::DeniedAccess)
        }
        script_edit::Reason::AmbiguousTarget => (
            OutcomeKind::AmbiguousTarget,
            DiagnosticCode::AmbiguousTarget,
        ),
        script_edit::Reason::Busy | script_edit::Reason::UnsupportedEngine => (
            OutcomeKind::UnsupportedObservation,
            DiagnosticCode::UnsupportedCapability,
        ),
        script_edit::Reason::UnavailableObservation => (
            OutcomeKind::EditorUnavailable,
            DiagnosticCode::EditorUnavailable,
        ),
        _ => (OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame),
    };
    RoutingFailure::new(outcome, code, Stage::Finalize)
}
fn reason(error: &RoutingFailure) -> script_edit::Reason {
    match error.outcome {
        OutcomeKind::AmbiguousTarget => script_edit::Reason::AmbiguousTarget,
        OutcomeKind::DeniedAccess => script_edit::Reason::DeniedAccess,
        OutcomeKind::Timeout => script_edit::Reason::Deadline,
        OutcomeKind::Cancelled => script_edit::Reason::Cancellation,
        OutcomeKind::DisconnectedEditor => script_edit::Reason::Disconnection,
        OutcomeKind::EditorUnavailable => script_edit::Reason::UnavailableObservation,
        OutcomeKind::UnsupportedObservation => script_edit::Reason::UnsupportedEngine,
        OutcomeKind::MissingTarget | OutcomeKind::InvalidTarget | OutcomeKind::NotOpen => {
            script_edit::Reason::ClosedTarget
        }
        OutcomeKind::InvalidRequest => script_edit::Reason::MissingBasis,
        _ => script_edit::Reason::ProtocolFailure,
    }
}
fn send(output: &mut impl Write, bytes: &[u8]) -> Result<(), RoutingFailure> {
    if bytes.len() > WORKER_LIMIT || bytes.is_empty() {
        return Err(protocol_failure());
    }
    output
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| output.write_all(bytes))
        .map_err(|_| protocol_failure())
}
fn event(output: &mut impl Write, id: &RequestId, value: &Event) -> Result<(), RoutingFailure> {
    send(output, &wire::encode_event(value, id)?)
}
fn extra(output: &mut impl Write, value: serde_json::Value) -> Result<(), RoutingFailure> {
    send(
        output,
        &serde_json::to_vec(&value).map_err(|_| protocol_failure())?,
    )
}
fn deadline(at: Instant) -> Result<(), RoutingFailure> {
    if Instant::now() >= at {
        Err(error(script_edit::Reason::Deadline))
    } else {
        Ok(())
    }
}
