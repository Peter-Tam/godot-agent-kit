//! Parent-owned cutoff and reduction for read-only discovery acquisition.
use super::{AttemptClock, Frames, HostFailure, OwnedWorker, POLL_INTERVAL};
use crate::bridge::wire::discovery as codec;
use crate::script_discovery::{Attempt, DiscoveryOutcome, DiscoveryRequest, Event, Reason};
use std::io::{self, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[path = "discovery/worker.rs"]
mod worker;
#[doc(hidden)]
pub use worker::worker_main;

// This mode is dispatched only with connected owned stdin/stdout Unix sockets.
#[doc(hidden)]
pub const INTERNAL_WORKER_FLAG: &str = "--internal-discovery-worker";
const BUDGET: Duration = Duration::from_millis(4500);
const CONTROL_LIMIT: usize = 4096;
const EVENT_LIMIT: usize = 512 * 1024;

fn interrupted(clock: AttemptClock, cancelled: &AtomicBool) -> Option<Reason> {
    if cancelled.load(Ordering::Relaxed) {
        Some(Reason::Cancelled)
    } else if clock.started.elapsed() >= BUDGET {
        Some(Reason::Timeout)
    } else {
        None
    }
}

fn send_startup(
    socket: &mut UnixStream,
    bytes: &[u8],
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<(), Reason> {
    if bytes.is_empty() || bytes.len() > CONTROL_LIMIT {
        return Err(Reason::InvalidRequest);
    }
    let header = (bytes.len() as u32).to_be_bytes();
    for part in [&header[..], bytes] {
        let mut offset = 0;
        while offset < part.len() {
            if let Some(reason) = interrupted(clock, cancelled) {
                return Err(reason);
            }
            match socket.write(&part[offset..]) {
                Ok(0) => return Err(Reason::ProtocolError),
                Ok(count) => offset += count,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(POLL_INTERVAL);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(_) => return Err(Reason::ProtocolError),
            }
        }
    }
    Ok(())
}

/// Perform one fresh discovery using this executable's owned, killable worker.
/// The caller starts `clock` before parsing and drains the bounded final result.
/// No filesystem/editor call that can block takes place in this supervisor.
///
/// # Errors
/// Returns a redacted host failure if the owned process/channel cannot be launched.
pub fn run(
    request: DiscoveryRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<DiscoveryOutcome, HostFailure> {
    let mut attempt = Attempt::new(request);
    if let Some(reason) = interrupted(clock, cancelled) {
        attempt.fail(reason);
        return Ok(attempt.finish(clock.interval()));
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
    let request = attempt.request();
    let startup = serde_json::to_vec(&(
        4,
        request.request_id().as_str(),
        request.project_root().as_str(),
        request
            .session_id()
            .map(crate::observation::SessionId::as_str),
        registry.to_str().ok_or(HostFailure)?,
        clock.elapsed_us(),
    ))
    .map_err(|_| HostFailure)?;
    if let Err(reason) = send_startup(&mut parent, &startup, clock, cancelled) {
        attempt.fail(reason);
        // Drop always kills/reaps only this child without blocking delivery.
        drop(parent);
        drop(worker);
        return Ok(attempt.finish(clock.interval()));
    }
    Ok(supervise(parent, worker, attempt, clock, cancelled))
}

fn supervise(
    mut socket: UnixStream,
    worker: OwnedWorker,
    mut attempt: Attempt,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> DiscoveryOutcome {
    let mut frames = Frames::default();
    loop {
        if let Some(reason) = interrupted(clock, cancelled) {
            attempt.fail(reason);
            break;
        }
        let bytes = match frames.next_limited(&mut socket, EVENT_LIMIT) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            Err(_) => {
                attempt.fail(Reason::ProtocolError);
                break;
            }
        };
        // The parent assigns one receipt after framing; no late bytes can be evidence.
        let receipt = clock.elapsed_us();
        if let Some(reason) = interrupted(clock, cancelled) {
            attempt.fail(reason);
            break;
        }
        let accepted = codec::decode_event(&bytes).and_then(|event| {
            let failed = matches!(event, Event::Failed { .. });
            attempt.accept(event, receipt)?;
            Ok(failed || attempt.is_done())
        });
        match accepted {
            Ok(done) => {
                if let Some(reason) = interrupted(clock, cancelled) {
                    attempt.fail(reason);
                    break;
                }
                if done {
                    break;
                }
            }
            Err(reason) => {
                attempt.fail(reason);
                break;
            }
        }
    }
    // Closing the channel and killing the owned worker cannot await addon cleanup.
    drop(socket);
    drop(worker);
    attempt.finish(clock.interval())
}
