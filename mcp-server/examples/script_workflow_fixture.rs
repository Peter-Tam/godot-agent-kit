//! Fixture-only supervised read/revision/edit consumer; not a product CLI.
use godot_agent_kit::observation::{
    ObservationRequest, ProjectRoot, RequestId, ResourcePath, SessionId,
};
use godot_agent_kit::runner::{self, AttemptClock};
use godot_agent_kit::script_edit::ReplacementSource;
use godot_agent_kit::script_read::{ScriptEditRequest, ScriptRevision};
use serde::Deserialize;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;
static CANCELLED: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: i32) {
    CANCELLED.store(true, Ordering::Relaxed);
}
unsafe extern "C" {
    fn signal(number: i32, handler: usize) -> usize;
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    Read {
        request_id: String,
    },
    Edit {
        request_id: String,
        revision: String,
        replacement_source: String,
    },
}
fn dispatch() {
    if let Some(code) = runner::closed_edit::worker_main() {
        std::process::exit(code);
    }
    if let Some(code) = runner::closed_edit::acquisition_worker_main() {
        std::process::exit(code);
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return;
    }
    let flag = args[1].to_str();
    let code = match flag {
        Some(runner::INTERNAL_WORKER_FLAG) => runner::worker_main(),
        Some(runner::edit::INTERNAL_WORKER_FLAG) => runner::edit::worker_main(),
        Some(runner::open::INTERNAL_WORKER_FLAG) => runner::open::worker_main(),
        Some(runner::close::INTERNAL_WORKER_FLAG) => runner::close::worker_main(),
        Some(runner::discovery::INTERNAL_WORKER_FLAG) => runner::discovery::worker_main(),
        Some(runner::stock_validation::INTERNAL_FLAG) => runner::stock_validation::worker_main(),
        _ => return,
    };
    std::process::exit(code.unwrap_or(1));
}
fn selectors() -> Result<(PathBuf, ProjectRoot, Option<SessionId>, ResourcePath), ()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (pairs, remainder) = args.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(());
    }
    let (mut registry, mut project, mut session, mut script) = (None, None, None, None);
    for pair in pairs {
        let value = pair[1].to_str().ok_or(())?.to_owned();
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
        || !Path::new(&registry).is_absolute()
        || registry.chars().any(char::is_control)
    {
        return Err(());
    }
    Ok((
        PathBuf::from(registry),
        ProjectRoot::new(project.ok_or(())?).map_err(|_| ())?,
        session.map(SessionId::new).transpose().map_err(|_| ())?,
        ResourcePath::new(script.ok_or(())?).map_err(|_| ())?,
    ))
}
fn fail(code: &str) -> ! {
    let application = if code == "host_failure" {
        "unknown"
    } else {
        "not_applied"
    };
    let bytes =
        serde_json::to_vec(&serde_json::json!({"error":{"code":code,"application":application}}))
            .unwrap_or_default();
    let _ = io::stdout().write_all(&bytes);
    let _ = io::stdout().write_all(b"\n");
    std::process::exit(2);
}
fn execute() -> Result<Vec<u8>, &'static str> {
    let clock = AttemptClock::start();
    // SAFETY: Both Darwin handlers only set the existing lock-free cancellation atomic.
    if unsafe {
        signal(2, cancel as *const () as usize) == usize::MAX
            || signal(15, cancel as *const () as usize) == usize::MAX
    } {
        return Err("host_failure");
    }
    let (registry, project, session, script) = selectors().map_err(|_| "invalid_request")?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("workflow-input".into())
        .spawn(move || {
            let mut bytes = Vec::new();
            let input = io::stdin()
                .lock()
                .take(16 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes);
            let decoded = if input.is_ok() && bytes.len() <= 16 * 1024 * 1024 {
                serde_json::from_slice::<Input>(&bytes).map_err(|_| "invalid_request")
            } else {
                Err("invalid_request")
            };
            let _ = sender.send(decoded);
        })
        .map_err(|_| "host_failure")?;
    let input = loop {
        if CANCELLED.load(Ordering::Relaxed) {
            return Err("cancellation");
        }
        if clock.elapsed_us() >= 9_500_000 {
            return Err("deadline");
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(value) => break value?,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Err("host_failure"),
        }
    };
    let is_edit = matches!(input, Input::Edit { .. });
    let (sender, receiver) = mpsc::sync_channel(1);
    let handle = std::thread::Builder::new()
        .name("workflow-supervisor".into())
        .spawn(move || {
            let result = (|| match input {
                Input::Read { request_id } => {
                    let request = ObservationRequest::new(
                        RequestId::new(request_id).map_err(|_| "invalid_request")?,
                        project,
                        session,
                        script,
                    );
                    runner::read::run(request, &registry, clock, &CANCELLED)
                        .and_then(|v| v.encode())
                        .map_err(|_| "host_failure")
                }
                Input::Edit {
                    request_id,
                    revision,
                    replacement_source,
                } => {
                    let request = ObservationRequest::new(
                        RequestId::new(request_id).map_err(|_| "invalid_request")?,
                        project,
                        session,
                        script,
                    );
                    let intent = ScriptEditRequest::new(
                        request,
                        ScriptRevision::new(revision).map_err(|_| "invalid_request")?,
                        ReplacementSource::new(replacement_source)
                            .map_err(|_| "invalid_request")?,
                    )
                    .map_err(|_| "invalid_request")?;
                    runner::read::edit(intent, &registry, clock, &CANCELLED)
                        .map_err(|_| "host_failure")
                }
            })();
            let _ = sender.send(result);
        })
        .map_err(|_| "host_failure")?;
    let cutoff = if is_edit { 9_500_000 } else { 4_500_000 };
    let result = loop {
        if clock.elapsed_us() >= cutoff {
            CANCELLED.store(true, Ordering::Relaxed);
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(value) => break value,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break Err("host_failure"),
        }
    };
    handle.join().map_err(|_| "host_failure")?;
    result
}
fn main() {
    dispatch();
    let bytes = execute().unwrap_or_else(|code| fail(code));
    if io::stdout()
        .lock()
        .write_all(&bytes)
        .and_then(|_| io::stdout().write_all(b"\n"))
        .is_err()
    {
        std::process::exit(1);
    }
}
