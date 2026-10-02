//! One bounded safe-close attempt; optional prior evidence arrives only on stdin.
use godot_agent_kit::{
    bridge::wire::close as codec,
    observation::{ProjectRoot, RequestId, ResourcePath, SessionId},
    runner::{self, AttemptClock},
};
use ring::rand::{SecureRandom, SystemRandom};
use std::{
    env,
    ffi::OsString,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
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
        .name("bounded-close-stdin".into())
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
            return Err("cancelled");
        }
        let left =
            Duration::from_millis(9500).saturating_sub(Duration::from_micros(clock.elapsed_us()));
        if left.is_zero() {
            return Err("timeout");
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
) -> Result<Result<codec::DecodedRequest, codec::InputError>, &'static str> {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("bounded-close-decode".into())
        .spawn(move || {
            let decoded = codec::decode_request(&bytes, project, session, script);
            drop(bytes);
            let _ = sender.send(decoded);
        })
        .map_err(|_| "host")?;
    loop {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err("cancelled");
        }
        let left =
            Duration::from_millis(9500).saturating_sub(Duration::from_micros(clock.elapsed_us()));
        if left.is_zero() {
            return Err("timeout");
        }
        match receiver.recv_timeout(left.min(Duration::from_millis(10))) {
            Ok(result) => return Ok(result),
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("host"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}
fn deliver(value: godot_agent_kit::script_close::ClosingOutcome) -> i32 {
    let code = codec::exit_code(&value);
    match codec::encode_outcome(&value) {
        Ok(bytes) if emit(&bytes) => code,
        _ => 1,
    }
}
fn main() -> ExitCode {
    let clock = AttemptClock::start();
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.as_slice(),[arg] if arg==runner::close::INTERNAL_WORKER_FLAG) {
        if let Some(code) = runner::close::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg==runner::stock_validation::INTERNAL_FLAG) {
        if let Some(code) = runner::stock_validation::worker_main() {
            return ExitCode::from(code as u8);
        }
    }
    if matches!(args.as_slice(),[arg] if arg=="--help") {
        println!("close-gdscript --registry ABSOLUTE_PATH --project ABSOLUTE_PATH [--session SESSION_ID] --script res://PATH < payload.json");
        return ExitCode::SUCCESS;
    }
    let Some(generated) = request_id() else {
        eprintln!("stage=accepted code=host_randomness_unavailable");
        return ExitCode::from(1);
    };
    let code = match parse(&args) {
        Err(()) => deliver(codec::refusal(
            generated,
            clock.interval(),
            "invalid_request",
            None,
        )),
        Ok((project, session, script, registry)) => {
            let selectors = Some((project.clone(), session.clone(), script.clone()));
            if !install_cancellation() {
                return ExitCode::from(1);
            }
            match stdin_until(clock) {
                Err("host") => 1,
                Err(reason) => deliver(codec::refusal(
                    generated,
                    clock.interval(),
                    reason,
                    selectors,
                )),
                Ok(bytes) => match decode_until(bytes, project, session, script, clock) {
                    Err("host") => 1,
                    Err(reason) => deliver(codec::refusal(
                        generated,
                        clock.interval(),
                        reason,
                        selectors,
                    )),
                    Ok(Err(error)) => deliver(codec::refusal(
                        error.request_id.unwrap_or(generated),
                        clock.interval(),
                        "invalid_request",
                        selectors,
                    )),
                    Ok(Ok(Err((id, reason)))) => {
                        deliver(codec::refusal(id, clock.interval(), reason, selectors))
                    }
                    Ok(Ok(Ok(request))) => {
                        match runner::close::run(request, &registry, clock, &CANCELLED) {
                            Ok(value) => deliver(value),
                            Err(_) => 1,
                        }
                    }
                },
            }
        }
    };
    if code == 1 {
        eprintln!("stage=terminal code=host_or_delivery_failure");
    }
    ExitCode::from(code as u8)
}
