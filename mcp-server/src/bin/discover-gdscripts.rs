//! One bounded, source-free project discovery; stdin is deliberately unused.
use godot_agent_kit::observation::{ProjectRoot, RequestId, SessionId};
use godot_agent_kit::runner::{self, AttemptClock};
use godot_agent_kit::script_discovery::{DiscoveryOutcome, DiscoveryRequest};
use ring::rand::{SecureRandom, SystemRandom};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
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
    // SAFETY: this C-ABI, static handler performs only a lock-free atomic store;
    // SIGINT and SIGTERM are 2 and 15 on the supported Unix platform.
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
fn parse(args: &[OsString], id: RequestId) -> Result<(DiscoveryRequest, PathBuf), ()> {
    let (mut registry, mut project, mut session) = (None, None, None);
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
        || Path::new(registry)
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(());
    }
    let project = ProjectRoot::new(project.ok_or(())?).map_err(|_| ())?;
    let session = session.map(SessionId::new).transpose().map_err(|_| ())?;
    Ok((
        DiscoveryRequest::new(id, project, session),
        PathBuf::from(registry),
    ))
}
fn deliver(outcome: DiscoveryOutcome, clock: AttemptClock, host_failure: bool) -> i32 {
    let remaining =
        Duration::from_secs(5).saturating_sub(Duration::from_micros(clock.elapsed_us()));
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let code = if host_failure { 1 } else { outcome.exit_code() };
        let written = outcome.to_json_line().map_err(|_| ()).and_then(|bytes| {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(&bytes)
                .and_then(|_| stdout.flush())
                .map_err(|_| ())
        });
        let _ = send.send(if written.is_ok() { code } else { 1 });
    });
    receive.recv_timeout(remaining).unwrap_or(1)
}
fn main() -> ExitCode {
    let clock = AttemptClock::start();
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.as_slice(), [arg] if arg == runner::discovery::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::discovery::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(), [arg] if arg == "--help") {
        println!("discover-gdscripts --registry ABSOLUTE_PATH --project ABSOLUTE_PATH [--session SESSION_ID]");
        return ExitCode::SUCCESS;
    }
    if matches!(args.as_slice(), [arg] if arg == "--version") {
        println!("discover-gdscripts {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let Some(id) = request_id() else {
        eprintln!("stage=validate_request code=host_randomness_unavailable");
        return ExitCode::from(1);
    };
    if !install_cancellation() {
        eprintln!(
            "request_id={} stage=validate_request code=host_failure",
            id.as_str()
        );
        return ExitCode::from(deliver(
            DiscoveryOutcome::host_failure(id, clock.interval()),
            clock,
            true,
        ) as u8);
    }
    let (outcome, host_failure) = match parse(&args, id.clone()) {
        Err(()) => (
            DiscoveryOutcome::invalid_request(id, clock.interval()),
            false,
        ),
        Ok((request, registry)) => {
            match runner::discovery::run(request, &registry, clock, &CANCELLED) {
                Ok(outcome) => (outcome, false),
                Err(_) => {
                    eprintln!(
                        "request_id={} stage=resolve_target code=host_failure",
                        id.as_str()
                    );
                    (DiscoveryOutcome::host_failure(id, clock.interval()), true)
                }
            }
        }
    };
    ExitCode::from(deliver(outcome, clock, host_failure) as u8)
}
