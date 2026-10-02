//! Parent-owned close verdict and one-shot authority; worker only acquires facts.
use super::stock_validation;
use super::{
    protocol_failure, AttemptClock, Collected, Frames, HostFailure, OwnedWorker, POLL_INTERVAL,
};
use crate::bridge::wire::{self, close as codec, Event};
use crate::observation::*;
use crate::project_fs;
use crate::script_close::{Attempt, CloseRequest, ClosingOutcome};
use crate::target::{self, RoutingFailure, SelectedSession};
use serde_json::{json, Value};
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
#[path = "close/supervisor.rs"]
mod supervisor;
#[path = "close/worker.rs"]
mod worker;
pub use supervisor::run;
pub use worker::worker_main;
pub const INTERNAL_WORKER_FLAG: &str = "--internal-close-worker";
const BUDGET: Duration = Duration::from_millis(9500);

// A fresh survivor read cannot renew an expired editor lease. Both acquisition
// and reduction use this exact condition for the additional failure-only frames.
fn survivor_available(reply: &codec::Reply) -> bool {
    reply.native.as_ref().is_some_and(|native| native.entered)
        && reply
            .expiry
            .as_ref()
            .is_some_and(|expiry| reply.collection.finished_tick_us() < expiry)
        && reply.purpose.as_deref() != Some("survivor")
}
fn reason(e: &RoutingFailure) -> &'static str {
    match e.diagnostic.code() {
        DiagnosticCode::DiskMissing => return "missing_script",
        DiagnosticCode::SourceChanged => return "source_changed",
        DiagnosticCode::IdentityChanged => return "identity_changed",
        DiagnosticCode::OutOfProject => return "outside_project",
        _ => {}
    }
    match e.outcome {
        OutcomeKind::AmbiguousTarget => "ambiguous_session",
        OutcomeKind::Timeout => "timeout",
        OutcomeKind::Cancelled => "cancelled",
        OutcomeKind::DeniedAccess => "denied_access",
        OutcomeKind::EditorUnavailable => "editor_unavailable",
        OutcomeKind::DisconnectedEditor => "disconnected",
        OutcomeKind::MissingTarget => "missing_script",
        OutcomeKind::InvalidTarget => "invalid_document_kind",
        OutcomeKind::UnsupportedObservation => "unsupported_capability",
        _ => "protocol_error",
    }
}
fn send(output: &mut impl Write, bytes: &[u8]) -> Result<(), RoutingFailure> {
    if bytes.is_empty() || bytes.len() > wire::RESULT_LIMIT {
        return Err(protocol_failure());
    }
    output
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| output.write_all(bytes))
        .map_err(|_| protocol_failure())
}
fn event(output: &mut impl Write, id: &RequestId, event: &Event) -> Result<(), RoutingFailure> {
    send(output, &wire::encode_event(event, id)?)
}
fn deadline(at: Instant) -> Result<(), RoutingFailure> {
    if Instant::now() >= at {
        Err(RoutingFailure::new(
            OutcomeKind::Timeout,
            DiagnosticCode::DeadlineExceeded,
            Stage::Finalize,
        ))
    } else {
        Ok(())
    }
}
fn receive(input: &mut UnixStream, limit: usize, at: Instant) -> Result<Vec<u8>, RoutingFailure> {
    deadline(at)?;
    input
        .set_read_timeout(Some(at.saturating_duration_since(Instant::now())))
        .map_err(|_| protocol_failure())?;
    let mut h = [0; 4];
    input.read_exact(&mut h).map_err(|_| protocol_failure())?;
    let n = u32::from_be_bytes(h) as usize;
    if n == 0 || n > limit {
        return Err(protocol_failure());
    }
    let mut bytes = vec![0; n];
    input
        .read_exact(&mut bytes)
        .map_err(|_| protocol_failure())?;
    deadline(at)?;
    Ok(bytes)
}
