//! Bounded local observation caller; no editor startup, mutation, or public worker controls.
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use godot_agent_kit::bridge::wire;
use godot_agent_kit::observation::*;
use godot_agent_kit::project_fs;
use godot_agent_kit::runner::{self, AttemptClock};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::json;

static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: i32) {
    // AtomicBool is lock-free on the supported macOS arm64 target; no allocation,
    // I/O, locks, or non-signal-safe runtime calls occur in the handler.
    CANCELLED.store(true, Ordering::Relaxed);
}
extern "C" {
    // Darwin signal(3): sig_t is a pointer-sized function pointer. SIG_ERR is -1.
    fn signal(number: i32, handler: usize) -> usize;
}
fn install_cancellation() -> bool {
    // SAFETY: both signal numbers are Darwin/POSIX SIGINT/SIGTERM. The handler has
    // the C ABI and lifetime of the process; it only sets the lock-free flag.
    unsafe {
        signal(2, cancel as *const () as usize) != usize::MAX
            && signal(15, cancel as *const () as usize) != usize::MAX
    }
}

fn request_id() -> Result<RequestId, ()> {
    let mut bytes = [0u8; 16];
    SystemRandom::new().fill(&mut bytes).map_err(|_| ())?;
    let mut id = String::with_capacity(32);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        id.push(HEX[(byte >> 4) as usize] as char);
        id.push(HEX[(byte & 15) as usize] as char);
    }
    RequestId::new(id).map_err(|_| ())
}
fn emit(bytes: &[u8]) -> bool {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .and_then(|_| stdout.write_all(b"\n"))
        .is_ok()
}
fn boundary(id: &RequestId, clock: AttemptClock, host: bool) -> i32 {
    let interval = clock.interval();
    let diagnostic = Diagnostic::new(
        if host {
            DiagnosticCode::ProtocolError
        } else {
            DiagnosticCode::InvalidRequest
        },
        if host {
            Stage::Finalize
        } else {
            Stage::ValidateRequest
        },
        None,
    );
    let bytes = serde_json::to_vec(&json!({
        "schema_version": 1, "request_id": id.as_str(),
        "outcome": if host { "protocol_error" } else { "invalid_request" },
        "interval": {"started_unix_ms": interval.started_unix_ms(),
            "finished_unix_ms": interval.finished_unix_ms(), "elapsed_us": interval.elapsed_us()},
        "resolved_target": null, "snapshot": null, "selection": null,
        "diagnostics": [{"code": if host { "protocol_error" } else { "invalid_request" },
            "stage": if host { "finalize" } else { "validate_request" },
            "surface": null, "message": diagnostic.message(), "action": diagnostic.action()}]
    }))
    .expect("fixed boundary fields serialize");
    if host {
        eprintln!(
            "request_id={} stage=finalize code=host_failure",
            id.as_str()
        );
    }
    if !emit(&bytes) || host {
        1
    } else {
        3
    }
}
fn parse(args: &[OsString], id: &RequestId) -> Result<(ObservationRequest, PathBuf), ()> {
    let (mut registry, mut project, mut session, mut script) = (None, None, None, None);
    let (pairs, remainder) = args.as_chunks::<2>();
    for pair in pairs {
        let flag = pair[0].to_str().ok_or(())?;
        let value = pair[1].to_str().ok_or(())?;
        let slot = match flag {
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
    if !remainder.is_empty() {
        return Err(());
    }
    let registry = registry.ok_or(())?;
    if registry.len() > 1024
        || !Path::new(registry).is_absolute()
        || registry.chars().any(char::is_control)
    {
        return Err(());
    }
    let request = ObservationRequest::new(
        id.clone(),
        ProjectRoot::new(project.ok_or(())?).map_err(|_| ())?,
        session.map(SessionId::new).transpose().map_err(|_| ())?,
        ResourcePath::new(script.ok_or(())?).map_err(|_| ())?,
    );
    Ok((request, PathBuf::from(registry)))
}
fn main() -> ExitCode {
    let clock = AttemptClock::start();
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.as_slice(), [arg] if arg == runner::closed_edit::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::closed_edit::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(), [arg] if arg == runner::closed_edit::INTERNAL_ACQUISITION_WORKER_FLAG)
    {
        if let Some(code) = runner::closed_edit::acquisition_worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(), [arg] if arg == runner::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    let Ok(id) = request_id() else {
        eprintln!("stage=validate_request code=host_randomness_unavailable");
        return ExitCode::from(1);
    };
    let code = match args.as_slice() {
        [arg] if arg == "--help" => {
            println!("observe-gdscript --registry ABSOLUTE_PATH --project ABSOLUTE_PATH [--session SESSION_ID] --script res://PATH\nobserve-gdscript init-registry --registry ABSOLUTE_PATH\nobserve-gdscript --help\nobserve-gdscript --version");
            0
        }
        [arg] if arg == "--version" => {
            println!("observe-gdscript {}", env!("CARGO_PKG_VERSION"));
            0
        }
        [command, flag, path] if command == "init-registry" && flag == "--registry" => {
            let value = match project_fs::init_registry(Path::new(path)) {
                Ok(_) => json!({"schema_version": 1, "status":"ready"}),
                Err(error) => json!({"schema_version": 1, "outcome":"denied_access",
                    "diagnostic":{"code":"unsafe_registry", "message":error.diagnostic.message(), "action":error.diagnostic.action()}}),
            };
            let code = if value["status"] == "ready" { 0 } else { 3 };
            if emit(&serde_json::to_vec(&value).expect("fixed bootstrap fields serialize")) {
                code
            } else {
                1
            }
        }
        _ => match parse(&args, &id) {
            Err(()) => boundary(&id, clock, false),
            Ok((request, registry)) => {
                if !install_cancellation() {
                    return ExitCode::from(boundary(&id, clock, true) as u8);
                }
                match runner::run(request, &registry, clock, &CANCELLED) {
                    Ok(outcome) => match wire::encode_outcome(&outcome) {
                        Ok(bytes) => {
                            if emit(&bytes) {
                                runner::exit_code(outcome.outcome())
                            } else {
                                eprintln!(
                                    "request_id={} stage=finalize code=output_unavailable",
                                    id.as_str()
                                );
                                1
                            }
                        }
                        Err(_) => boundary(&id, clock, true),
                    },
                    Err(_) => boundary(&id, clock, true),
                }
            }
        },
    };
    ExitCode::from(code as u8)
}
