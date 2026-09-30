//! Parent-owned validator and exactly-once authorization, with killable acquisition worker.
use super::stock_validation;
use super::{
    protocol_failure, AttemptClock, Collected, Frames, HostFailure, OwnedWorker, POLL_INTERVAL,
};
use crate::bridge::wire::open as codec;
use crate::bridge::wire::{self, Event};
use crate::observation::*;
use crate::project_fs;
use crate::script_open::{Attempt, OpenRequest, OpeningOutcome, Reason};
use crate::target::{self, RoutingFailure, SelectedSession};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
#[path = "open/supervisor.rs"]
mod supervisor;
#[path = "open/worker.rs"]
mod worker;
pub use supervisor::run;
pub use worker::worker_main;
pub const INTERNAL_WORKER_FLAG: &str = "--internal-open-worker";
const BUDGET: Duration = Duration::from_millis(9500);
const CONTROL_LIMIT: usize = 4096;
fn failure(reason: Reason) -> RoutingFailure {
    let (kind, code) = match reason {
        Reason::Timeout => (OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded),
        Reason::Cancelled => (OutcomeKind::Cancelled, DiagnosticCode::Cancelled),
        Reason::Disconnected => (
            OutcomeKind::DisconnectedEditor,
            DiagnosticCode::SessionEnded,
        ),
        Reason::DeniedAccess => (OutcomeKind::DeniedAccess, DiagnosticCode::DeniedAccess),
        Reason::OutsideProject => (OutcomeKind::DeniedAccess, DiagnosticCode::OutOfProject),
        Reason::UnsupportedCapability => (
            OutcomeKind::UnsupportedObservation,
            DiagnosticCode::UnsupportedCapability,
        ),
        _ => (OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame),
    };
    RoutingFailure::new(kind, code, Stage::Finalize)
}
fn reason(e: &RoutingFailure) -> Reason {
    match e.diagnostic.code() {
        DiagnosticCode::SourceChanged => return Reason::SourceChanged,
        DiagnosticCode::IdentityChanged => return Reason::TargetChanged,
        DiagnosticCode::DiskMissing => return Reason::MissingScript,
        DiagnosticCode::TooLarge | DiagnosticCode::DiskUnreadable | DiagnosticCode::InvalidUtf8 => {
            return Reason::EvidenceUnavailable
        }
        _ => {}
    }
    match e.outcome {
        OutcomeKind::AmbiguousTarget => Reason::AmbiguousSession,
        OutcomeKind::Timeout => Reason::Timeout,
        OutcomeKind::Cancelled => Reason::Cancelled,
        OutcomeKind::DisconnectedEditor => Reason::Disconnected,
        OutcomeKind::DeniedAccess if e.diagnostic.code() == DiagnosticCode::OutOfProject => {
            Reason::OutsideProject
        }
        OutcomeKind::DeniedAccess => Reason::DeniedAccess,
        OutcomeKind::EditorUnavailable => Reason::EditorUnavailable,
        OutcomeKind::MissingTarget => Reason::MissingScript,
        OutcomeKind::InvalidTarget => Reason::InvalidDocumentKind,
        OutcomeKind::UnsupportedObservation => Reason::UnsupportedCapability,
        _ => Reason::ProtocolError,
    }
}
fn send(output: &mut impl Write, bytes: &[u8]) -> Result<(), RoutingFailure> {
    if bytes.is_empty() || bytes.len() > wire::RESULT_LIMIT {
        return Err(protocol_failure());
    }
    output
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| output.write_all(bytes))
        .map_err(|_| failure(Reason::Disconnected))
}
fn event(output: &mut impl Write, id: &RequestId, event: &Event) -> Result<(), RoutingFailure> {
    send(output, &wire::encode_event(event, id)?)
}
fn deadline(at: Instant) -> Result<(), RoutingFailure> {
    if Instant::now() >= at {
        Err(failure(Reason::Timeout))
    } else {
        Ok(())
    }
}
