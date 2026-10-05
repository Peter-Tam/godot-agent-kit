//! Local stdio MCP entry point; same-binary workers retain private socket I/O.
use godot_agent_kit::{mcp, runner};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
extern "C" fn cancel(_: i32) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}
extern "C" {
    fn signal(number: i32, handler: usize) -> usize;
}
fn install_cancellation() -> bool {
    // SAFETY: supported Darwin handlers only store to a lock-free atomic.
    unsafe {
        signal(2, cancel as *const () as usize) != usize::MAX
            && signal(15, cancel as *const () as usize) != usize::MAX
    }
}
fn worker(args: &[OsString]) -> Option<i32> {
    let [flag] = args else {
        return None;
    };
    let flag = flag.as_os_str();
    if flag == OsStr::new(runner::INTERNAL_WORKER_FLAG) {
        return Some(runner::worker_main().unwrap_or(1));
    }
    if flag == OsStr::new(runner::discovery::INTERNAL_WORKER_FLAG) {
        return Some(runner::discovery::worker_main().unwrap_or(1));
    }
    if flag == OsStr::new(runner::edit::INTERNAL_WORKER_FLAG) {
        return Some(runner::edit::worker_main().unwrap_or(1));
    }
    if flag == OsStr::new(runner::closed_edit::INTERNAL_WORKER_FLAG) {
        return Some(runner::closed_edit::worker_main().unwrap_or(1));
    }
    if flag == OsStr::new(runner::closed_edit::INTERNAL_ACQUISITION_WORKER_FLAG) {
        return Some(runner::closed_edit::acquisition_worker_main().unwrap_or(1));
    }
    if flag == OsStr::new(runner::stock_validation::INTERNAL_FLAG) {
        return Some(runner::stock_validation::worker_main().unwrap_or(1));
    }
    None
}
fn registry(args: &[OsString]) -> Option<PathBuf> {
    let [flag, path] = args else {
        return None;
    };
    if flag != "--registry" {
        return None;
    }
    let text = path.to_str()?;
    if text.is_empty()
        || text.len() > 1024
        || text.chars().any(char::is_control)
        || !Path::new(text).is_absolute()
    {
        return None;
    }
    Some(PathBuf::from(path))
}
fn main() -> ExitCode {
    // Neither SDK message tracing nor panic payloads may expose tool source.
    std::panic::set_hook(Box::new(|_| eprintln!("mcp host failure")));
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Some(code) = worker(&args) {
        return ExitCode::from(code as u8);
    }
    if matches!(args.as_slice(), [arg] if arg == "--help") {
        eprintln!("godot-agent-kit-mcp --registry ABSOLUTE_PATH");
        return ExitCode::SUCCESS;
    }
    if matches!(args.as_slice(), [arg] if arg == "--version") {
        eprintln!(
            "godot-agent-kit-mcp {} (MCP 2025-11-25)",
            env!("CARGO_PKG_VERSION")
        );
        return ExitCode::SUCCESS;
    }
    let Some(registry) = registry(&args) else {
        eprintln!("mcp invalid configuration");
        return ExitCode::from(2);
    };
    if !install_cancellation() {
        eprintln!("mcp host failure");
        return ExitCode::FAILURE;
    }
    match mcp::run(registry, &SHUTDOWN) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("mcp host failure");
            ExitCode::FAILURE
        }
    }
}
