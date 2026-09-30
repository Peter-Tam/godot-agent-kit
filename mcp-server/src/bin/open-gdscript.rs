//! One known-path opening attempt; stdin is deliberately unused.
use godot_agent_kit::bridge::wire::open as codec;
use godot_agent_kit::observation::{
    ObservationRequest, ProjectRoot, RequestId, ResourcePath, ScriptKind, SessionId,
};
use godot_agent_kit::runner::{self, AttemptClock};
use godot_agent_kit::script_open::{OpenRequest, OpeningOutcome};
use ring::rand::{SecureRandom, SystemRandom};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: i32) {
    CANCELLED.store(true, Ordering::Relaxed);
}
extern "C" {
    fn signal(number: i32, handler: usize) -> usize;
}
fn install_cancellation() -> bool {
    // SAFETY: the handler has C ABI, static lifetime and performs only a
    // lock-free atomic store. These are the supported platform's SIGINT/SIGTERM.
    unsafe {
        signal(2, cancel as *const () as usize) != usize::MAX
            && signal(15, cancel as *const () as usize) != usize::MAX
    }
}
fn request_id() -> Option<RequestId> {
    let mut bytes = [0; 16];
    SystemRandom::new().fill(&mut bytes).ok()?;
    let mut id = String::with_capacity(32);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        id.push(HEX[(byte >> 4) as usize] as char);
        id.push(HEX[(byte & 15) as usize] as char);
    }
    RequestId::new(id).ok()
}
enum ParseError {
    Syntax,
    InvalidDocumentKind(ObservationRequest),
}
fn parse(args: &[OsString], id: RequestId) -> Result<(OpenRequest, PathBuf), ParseError> {
    let (mut registry, mut project, mut session, mut script) = (None, None, None, None);
    let (pairs, remainder) = args.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(ParseError::Syntax);
    }
    for pair in pairs {
        let value = pair[1].to_str().ok_or(ParseError::Syntax)?;
        let slot = match pair[0].to_str().ok_or(ParseError::Syntax)? {
            "--registry" => &mut registry,
            "--project" => &mut project,
            "--session" => &mut session,
            "--script" => &mut script,
            _ => return Err(ParseError::Syntax),
        };
        if slot.replace(value).is_some() {
            return Err(ParseError::Syntax);
        }
    }
    let registry = registry.ok_or(ParseError::Syntax)?;
    if registry.len() > 1024
        || !Path::new(registry).is_absolute()
        || registry.chars().any(char::is_control)
    {
        return Err(ParseError::Syntax);
    }
    let project =
        ProjectRoot::new(project.ok_or(ParseError::Syntax)?).map_err(|_| ParseError::Syntax)?;
    let session = session
        .map(SessionId::new)
        .transpose()
        .map_err(|_| ParseError::Syntax)?;
    let script =
        ResourcePath::new(script.ok_or(ParseError::Syntax)?).map_err(|_| ParseError::Syntax)?;
    if script.kind() != Some(ScriptKind::ExternalGdscript) {
        return Err(ParseError::InvalidDocumentKind(ObservationRequest::new(
            id, project, session, script,
        )));
    }
    let request = OpenRequest::new(id, project, session, script).map_err(|_| ParseError::Syntax)?;
    Ok((request, PathBuf::from(registry)))
}
fn deliver(outcome: OpeningOutcome, clock: AttemptClock) -> i32 {
    let remaining =
        Duration::from_secs(10).saturating_sub(Duration::from_micros(clock.elapsed_us()));
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let code = codec::exit_code(&outcome);
        let result = codec::encode_outcome(&outcome)
            .map_err(|_| ())
            .and_then(|bytes| {
                let mut output = io::stdout().lock();
                output
                    .write_all(&bytes)
                    .and_then(|_| output.write_all(b"\n"))
                    .and_then(|_| output.flush())
                    .map_err(|_| ())
            });
        let _ = send.send(if result.is_ok() { code } else { 1 });
    });
    receive.recv_timeout(remaining).unwrap_or(1)
}
fn main() -> ExitCode {
    let clock = AttemptClock::start();
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.as_slice(),[arg] if arg==runner::open::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::open::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg==runner::stock_validation::INTERNAL_FLAG) {
        if let Some(code) = runner::stock_validation::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg=="--help") {
        println!("open-gdscript --registry ABSOLUTE_PATH --project ABSOLUTE_PATH [--session SESSION_ID] --script res://PATH");
        return ExitCode::SUCCESS;
    }
    let Some(id) = request_id() else {
        eprintln!("stage=accepted code=host_randomness_unavailable");
        return ExitCode::from(1);
    };
    if !install_cancellation() {
        eprintln!(
            "request_id={} stage=accepted code=host_failure",
            id.as_str()
        );
        return ExitCode::from(1);
    }
    let outcome = match parse(&args, id.clone()) {
        Err(ParseError::Syntax) => codec::invalid_request(id, clock.interval()),
        Err(ParseError::InvalidDocumentKind(request)) => {
            codec::invalid_document_kind(request, clock.interval())
        }
        Ok((request, registry)) => match runner::open::run(request, &registry, clock, &CANCELLED) {
            Ok(value) => value,
            Err(_) => {
                eprintln!(
                    "request_id={} stage=accepted code=host_failure",
                    id.as_str()
                );
                return ExitCode::from(1);
            }
        },
    };
    ExitCode::from(deliver(outcome, clock) as u8)
}
