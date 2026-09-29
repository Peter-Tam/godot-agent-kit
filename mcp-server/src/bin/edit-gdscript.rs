//! One bounded edit attempt; source and observation basis arrive only through stdin.
use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use godot_agent_kit::bridge::wire::edit as codec;
use godot_agent_kit::observation::{ProjectRoot, RequestId, ResourcePath, SessionId};
use godot_agent_kit::runner::{self, AttemptClock};
use godot_agent_kit::script_edit::EditOutcome;
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::json;

static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: i32) {
    CANCELLED.store(true, Ordering::Relaxed);
}
extern "C" {
    fn signal(number: i32, handler: usize) -> usize;
}
fn install_cancellation() -> bool {
    // SAFETY: Darwin SIGINT/SIGTERM handlers only set the lock-free atomic.
    unsafe {
        signal(2, cancel as *const () as usize) != usize::MAX
            && signal(15, cancel as *const () as usize) != usize::MAX
    }
}
fn request_id() -> Option<RequestId> {
    let mut bytes = [0; 16];
    SystemRandom::new().fill(&mut bytes).ok()?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut id = String::with_capacity(32);
    for byte in bytes {
        id.push(HEX[(byte >> 4) as usize] as char);
        id.push(HEX[(byte & 15) as usize] as char);
    }
    RequestId::new(id).ok()
}
fn emit(bytes: &[u8]) -> bool {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .and_then(|_| stdout.write_all(b"\n"))
        .is_ok()
}
#[derive(Clone, Copy, serde::Serialize)]
struct Requested<'a> {
    project_root: &'a str,
    session_id: Option<&'a str>,
    script_path: &'a str,
}
fn boundary(
    id: &RequestId,
    clock: AttemptClock,
    reason: &str,
    host: bool,
    requested: Option<Requested<'_>>,
) -> i32 {
    if host {
        eprintln!(
            "request_id={} stage=finalize code=host_failure",
            id.as_str()
        );
        return 1;
    }
    let interval = clock.interval();
    let bytes = serde_json::to_vec(&json!({
        "schema_version":1,"operation":"edit_open_gdscript","request_id":id.as_str(),
        "requested_target":requested,"resolved_target":null,
        "interval":{"started_unix_ms":interval.started_unix_ms(),
            "finished_unix_ms":interval.finished_unix_ms(),"elapsed_us":interval.elapsed_us()},
        "outcome":"refused","reason":reason,"stage":"accepted","application":"not_applied",
        "progress":{"buffer_application":{"state":"not_started","reason":null,"collection":null},
            "resource_sync":{"state":"not_started","reason":null,"collection":null},
            "persistence":{"state":"not_started","reason":null,"collection":null},
            "finalization":{"state":"not_started","reason":null,"collection":null},
            "validation":{"state":"not_started","reason":null,"collection":null},
            "verification":{"state":"not_started","reason":null,"collection":null}},
        "expected":null,"before":null,"after":null,"persistence":null,"finalization":null,
        "validation":[],"context_recheck":null,"history":"not_participated",
        "diagnostics":[{"stage":"accepted","reason":reason}],"selection":null,
        "safe_next_action":"Observe the explicitly selected target again before any new edit"
    }))
    .expect("constant boundary JSON");
    if !emit(&bytes) {
        1
    } else if matches!(
        reason,
        "deadline" | "cancellation" | "disconnection" | "protocol_failure"
    ) {
        4
    } else {
        3
    }
}
fn parse(args: &[OsString]) -> Result<(ProjectRoot, Option<SessionId>, ResourcePath, PathBuf), ()> {
    let (mut registry, mut project, mut session, mut script) = (None, None, None, None);
    let (pairs, remainder) = args.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(());
    }
    for pair in pairs {
        let value = pair[1].to_str().ok_or(())?;
        let slot = match pair[0].to_str().ok_or(())? {
            "--registry" => &mut registry,
            "--project" => &mut project,
            "--session" => &mut session,
            "--script" => &mut script,
            _ => return Err(()),
        };
        if slot.replace(value).is_some() {
            return Err(());
        }
    }
    let registry = registry.ok_or(())?;
    if registry.len() > 1024
        || !Path::new(registry).is_absolute()
        || registry.chars().any(char::is_control)
    {
        return Err(());
    }
    Ok((
        ProjectRoot::new(project.ok_or(())?).map_err(|_| ())?,
        session.map(SessionId::new).transpose().map_err(|_| ())?,
        ResourcePath::new(script.ok_or(())?).map_err(|_| ())?,
        PathBuf::from(registry),
    ))
}
fn stdin_until(clock: AttemptClock) -> Result<Vec<u8>, &'static str> {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("bounded-edit-stdin".into())
        .spawn(move || {
            let mut bytes = Vec::new();
            let read = io::stdin()
                .lock()
                .take((codec::input_limit() + 1) as u64)
                .read_to_end(&mut bytes);
            let _ = sender.send((read, bytes));
        })
        .map_err(|_| "host")?;
    loop {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err("cancellation");
        }
        let left =
            Duration::from_millis(9500).saturating_sub(Duration::from_micros(clock.elapsed_us()));
        if left.is_zero() {
            return Err("deadline");
        }
        match receiver.recv_timeout(left.min(Duration::from_millis(10))) {
            Ok((Ok(_), bytes)) if bytes.len() <= codec::input_limit() => return Ok(bytes),
            Ok(_) => return Err("invalid_request"),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("host"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}
fn decode_until(
    bytes: Vec<u8>,
    project: ProjectRoot,
    session: Option<SessionId>,
    script: ResourcePath,
    clock: AttemptClock,
) -> Result<(Vec<u8>, Result<codec::DecodedRequest, codec::InputError>), &'static str> {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("bounded-edit-decode".into())
        .spawn(move || {
            let decoded = codec::decode_request(&bytes, project, session, script);
            let _ = sender.send((bytes, decoded));
        })
        .map_err(|_| "host")?;
    loop {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err("cancellation");
        }
        let left =
            Duration::from_millis(9500).saturating_sub(Duration::from_micros(clock.elapsed_us()));
        if left.is_zero() {
            return Err("deadline");
        }
        match receiver.recv_timeout(left.min(Duration::from_millis(10))) {
            Ok(result) => return Ok(result),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("host"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}
fn main() -> ExitCode {
    let clock = AttemptClock::start();
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.as_slice(),[arg] if arg==runner::edit::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::edit::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg==runner::stock_validation::INTERNAL_FLAG) {
        if let Some(code) = runner::stock_validation::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg=="--help") {
        println!("edit-gdscript --registry ABSOLUTE_PATH --project ABSOLUTE_PATH [--session SESSION_ID] --script res://PATH < payload.json");
        return ExitCode::SUCCESS;
    }
    let Some(generated) = request_id() else {
        eprintln!("stage=validate_request code=host_randomness_unavailable");
        return ExitCode::from(1);
    };
    let code = match parse(&args) {
        Err(()) => boundary(&generated, clock, "invalid_request", false, None),
        Ok((project, session, script, registry)) => {
            let requested = Some(Requested {
                project_root: project.as_str(),
                session_id: session.as_ref().map(SessionId::as_str),
                script_path: script.as_str(),
            });
            if !install_cancellation() {
                return ExitCode::from(boundary(
                    &generated,
                    clock,
                    "protocol_failure",
                    true,
                    requested,
                ) as u8);
            }
            match stdin_until(clock) {
                Err(reason) => boundary(&generated, clock, reason, reason == "host", requested),
                Ok(bytes) => match decode_until(
                    bytes,
                    project.clone(),
                    session.clone(),
                    script.clone(),
                    clock,
                ) {
                    Err(reason) => boundary(&generated, clock, reason, reason == "host", requested),
                    Ok((bytes, decoded)) => match decoded {
                        Err(error) => boundary(
                            error.request_id.as_ref().unwrap_or(&generated),
                            clock,
                            "invalid_request",
                            false,
                            requested,
                        ),
                        Ok(Err((id, error))) => {
                            let outcome = EditOutcome::refuse_without_basis(
                                id,
                                project,
                                session,
                                script,
                                clock.interval(),
                                error,
                            );
                            match codec::encode_outcome(&outcome) {
                                Ok(bytes) if emit(&bytes) => 3,
                                _ => 1,
                            }
                        }
                        Ok(Ok(request)) => {
                            let id = request.request_id().clone();
                            match runner::edit::run(request, bytes, &registry, clock, &CANCELLED) {
                                Ok(outcome) => match codec::encode_outcome(&outcome) {
                                    Ok(bytes) if emit(&bytes) => codec::exit_code(&outcome),
                                    _ => boundary(&id, clock, "protocol_failure", true, requested),
                                },
                                Err(_) => boundary(&id, clock, "protocol_failure", true, requested),
                            }
                        }
                    },
                },
            }
        }
    };
    ExitCode::from(code as u8)
}
