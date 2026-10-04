//! One-shot, source-only official-stock GDScript validation. This is an internal
//! supervised worker, not a client-facing LSP or arbitrary-file operation.
use super::AttemptClock;
use crate::observation::{ProjectRoot, RequestId, ResourcePath, SessionId, SOURCE_LIMIT_BYTES};
use crate::project_fs;
use crate::project_fs_validation::{self as confined, Captured};
use cap_std::{
    ambient_authority,
    fs::{Dir, MetadataExt as _},
};
use ring::digest::{Context, SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub const INTERNAL_FLAG: &str = "--internal-stock-validation-worker";
const BUDGET: Duration = Duration::from_millis(9500);
const CLEANUP_RESERVE: Duration = Duration::from_millis(250);
const TICK: Duration = Duration::from_millis(5);
const CONTROL_MAX: usize = 4 * SOURCE_LIMIT_BYTES;
const RESULT_MAX: usize = SOURCE_LIMIT_BYTES;
const LSP_MAX: usize = 4 * SOURCE_LIMIT_BYTES;
const OFFICIAL_SHA256: &str = "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf";

// Exact source-only warning surface registered by the pinned official build.
// The three internal deprecated warnings are never produced; the separate
// renamed_in_godot_4_hint setting is not a warning severity.
const ACTIVE_WARNINGS: &[&str] = &[
    "unassigned_variable",
    "unassigned_variable_op_assign",
    "unused_variable",
    "unused_local_constant",
    "unused_private_class_variable",
    "unused_parameter",
    "unused_signal",
    "shadowed_variable",
    "shadowed_variable_base_class",
    "shadowed_global_identifier",
    "unreachable_code",
    "unreachable_pattern",
    "standalone_expression",
    "standalone_ternary",
    "incompatible_ternary",
    "untyped_declaration",
    "inferred_declaration",
    "unsafe_property_access",
    "unsafe_method_access",
    "unsafe_cast",
    "unsafe_call_argument",
    "unsafe_void_return",
    "return_value_discarded",
    "static_called_on_instance",
    "missing_tool",
    "redundant_static_unload",
    "redundant_await",
    "missing_await",
    "assert_always_true",
    "assert_always_false",
    "integer_division",
    "narrowing_conversion",
    "int_as_enum_without_cast",
    "int_as_enum_without_match",
    "enum_variable_without_default",
    "empty_file",
    "deprecated_keyword",
    "confusable_identifier",
    "confusable_local_declaration",
    "confusable_local_usage",
    "confusable_capture_reassignment",
    "confusable_temporary_modification",
    "inference_on_variant",
    "native_method_override",
    "get_node_default_without_onready",
    "onready_with_export",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningProvenance {
    pub source: WarningOrigin,
    pub project_root: String,
    pub session_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningOrigin {
    EditorProjectSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningSettings {
    pub enable: bool,
    pub levels: BTreeMap<String, u8>,
    pub directory_rules: BTreeMap<String, u8>,
    pub provenance: WarningProvenance,
}

/// `source=None` independently captures actual D for edit verification. Preflight
/// alone accepts a proposal; opening instead consumes a checked private context.
pub struct ValidationRequest {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub project_root: ProjectRoot,
    pub root_path: ResourcePath,
    pub source: Option<String>,
    pub purpose: Purpose,
    pub open_context: Option<OpeningContext>,
    pub warnings: WarningSettings,
    pub global_classes: Vec<String>,
    pub official_binary: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    Preflight,
    PostChange,
    Unchanged,
    OpenContext,
    CloseContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEvidence {
    pub path: String,
    pub sha256: String,
    pub utf8_bytes: usize,
    pub device: Option<u64>,
    pub inode: Option<u64>,
    pub diagnostics_completed: bool,
    pub symbols_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDiagnostic {
    pub path: String,
    pub source_sha256: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub status: String,
    pub reason: Option<String>,
    pub request_id: String,
    pub session_id: String,
    pub purpose: Purpose,
    pub root_path: String,
    pub sources: Vec<SourceEvidence>,
    pub diagnostics: Vec<ErrorDiagnostic>,
    pub context_sha256: Option<String>,
    /// Private opening attribution, issued only after valid completion and cleanup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening_binding: Option<OpeningValidationBinding>,
    /// Unknown when a worker was lost before returning its lifecycle evidence.
    pub child_spawned: Option<bool>,
    pub child_pid: Option<u32>,
    pub child_reaped: Option<bool>,
    pub clone_path: Option<String>,
    pub clone_extra_files: Option<bool>,
    pub clone_log_files: Option<bool>,
    pub cleanup_confirmed: bool,
    pub launch_args: Vec<String>,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
    pub elapsed_us: u64,
}
impl ValidationResult {
    fn new(request: &WireRequest) -> Self {
        Self {
            status: "unavailable".into(),
            reason: None,
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            purpose: request.purpose,
            root_path: request.root_path.clone(),
            sources: Vec::new(),
            diagnostics: Vec::new(),
            context_sha256: None,
            opening_binding: None,
            child_spawned: Some(false),
            child_pid: None,
            child_reaped: Some(false),
            clone_path: None,
            clone_extra_files: None,
            clone_log_files: None,
            cleanup_confirmed: true,
            launch_args: Vec::new(),
            started_unix_ms: 0,
            finished_unix_ms: 0,
            elapsed_us: 0,
        }
    }
    fn unavailable(&mut self, reason: &'static str) {
        self.status = "unavailable".into();
        self.reason = Some(reason.into());
        self.opening_binding = None;
    }
    fn stamp(&mut self, clock: &AttemptClock) {
        let interval = clock.interval();
        self.started_unix_ms = interval.started_unix_ms();
        self.finished_unix_ms = interval.finished_unix_ms();
        self.elapsed_us = interval.elapsed_us();
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRequest {
    request_id: String,
    session_id: String,
    project_root: String,
    root_path: String,
    source: Option<String>,
    purpose: Purpose,
    #[serde(serialize_with = "opening_context::serialize_context")]
    open_context: Option<OpeningContext>,
    warnings: WarningSettings,
    global_classes: Vec<String>,
    official_binary: String,
    private_dir: String,
    elapsed_us: u64,
}

fn wire_request(request: ValidationRequest, clock: &AttemptClock) -> WireRequest {
    WireRequest {
        request_id: request.request_id.as_str().into(),
        session_id: request.session_id.as_str().into(),
        project_root: request.project_root.as_str().into(),
        root_path: request.root_path.as_str().into(),
        source: request.source,
        purpose: request.purpose,
        open_context: request.open_context,
        global_classes: request.global_classes,
        warnings: request.warnings,
        official_binary: request.official_binary.to_string_lossy().into(),
        private_dir: String::new(),
        elapsed_us: clock.elapsed_us(),
    }
}

fn deadline(deadline: Instant) -> Result<(), &'static str> {
    if Instant::now() >= deadline {
        Err("deadline")
    } else {
        Ok(())
    }
}
#[path = "stock_validation/admission.rs"]
mod admission;
#[path = "stock_validation/closing_context.rs"]
mod closing_context;
#[path = "stock_validation/opening_context.rs"]
mod opening_context;
pub(crate) use closing_context::validate_close_context_owned;
pub use closing_context::{
    capture_close_target, recheck_close_target, validate_close_context, CloseContext,
    CloseValidation, TargetCapture,
};
#[path = "stock_validation/opening_literals.rs"]
mod opening_literals;
#[path = "stock_validation/ownership.rs"]
mod ownership;
#[path = "stock_validation/protocol.rs"]
mod protocol;
use admission::*;
pub(crate) use opening_context::Projection as ClosedContextProjection;
pub use opening_context::{OpeningContext, OpeningValidationBinding};
use ownership::*;
use protocol::*;

fn run_child(
    request: &WireRequest,
    result: &mut ValidationResult,
    deadline_at: Instant,
) -> Result<(), &'static str> {
    let closure = capture_closure(request, result, deadline_at)?;
    // Admission and binary provenance precede *any* engine process start.
    official(&request.official_binary, deadline_at)?;
    recheck(request, &closure, deadline_at)?;
    result.cleanup_confirmed = false;
    let private = private_clone_named(&request.private_dir)?;
    result.clone_path = Some(private.0.to_string_lossy().into());
    let project = stage(&closure, &private, &request.warnings)?;
    recheck(request, &closure, deadline_at)?;
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .map_err(|_| "endpoint_unverified")?;
    let port = listener
        .local_addr()
        .map_err(|_| "endpoint_unverified")?
        .port();
    drop(listener);
    let arguments = vec![
        "--headless".into(),
        "--editor".into(),
        "--path".into(),
        project.to_string_lossy().into_owned(),
        "--lsp-port".into(),
        port.to_string(),
    ];
    result.launch_args.clone_from(&arguments);
    let mut owned = OwnedGodot(None);
    let work = (|| -> Result<(), &'static str> {
        let child = Command::new(&request.official_binary)
            .args(&arguments)
            .current_dir(&project)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", private.0.join("home"))
            .env("XDG_CONFIG_HOME", private.0.join("config"))
            .env("XDG_DATA_HOME", private.0.join("data"))
            .env("XDG_CACHE_HOME", private.0.join("cache"))
            .env("TMPDIR", private.0.join("tmp"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "launch_failed")?;
        result.child_pid = Some(child.id());
        result.child_spawned = Some(true);
        owned.0 = Some(child);
        let mut stream = connect_owned(result.child_pid.expect("spawned"), port, deadline_at)?;
        let project_uri = uri(&project)?;
        send_lsp(
            &mut stream,
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize",
            "params":{"processId":std::process::id(),"rootUri":project_uri,
                "rootPath":project.to_str().ok_or("unsafe_path")?,
                "workspaceFolders":[{"uri":project_uri,"name":"Private GDScript Validation"}],
                "capabilities":{"textDocument":{"documentSymbol":{
                    "hierarchicalDocumentSymbolSupport":true}}}}}),
        )?;
        let initialized = response(&mut stream, 1, deadline_at)?;
        if !initialized.is_object() || initialized.get("capabilities").is_none() {
            return Err("initialize_incomplete");
        }
        send_lsp(
            &mut stream,
            &json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        )?;
        for (index, source) in closure.sources.iter().enumerate() {
            deadline(deadline_at)?;
            source_fence(&mut stream, source, index, &project, result, deadline_at)?;
        }
        drop(stream);
        recheck(request, &closure, deadline_at)?;
        missing_attributed(&closure, &result.diagnostics)?;
        result.status = if result.diagnostics.is_empty() {
            "valid"
        } else {
            "invalid"
        }
        .into();
        Ok(())
    })();
    result.child_reaped = Some(
        result.child_spawned == Some(true)
            && owned.reap(Instant::now() + Duration::from_millis(150)),
    );
    if result.child_spawned == Some(true) && result.child_reaped != Some(true) {
        return Err("reap_unavailable");
    }
    let inspected = inspect_clone(&private, &closure);
    let cleanup = fs::remove_dir_all(&private.0).map_err(|_| "cleanup_unavailable");
    // Retain only the *path* for the acceptance fixture to inspect post-cleanup.
    drop(private);
    let (extra, logs) = inspected?;
    result.clone_extra_files = Some(extra);
    result.clone_log_files = Some(logs);
    cleanup?;
    result.cleanup_confirmed = true;
    if logs {
        return Err("privacy_unverified");
    }
    if extra {
        return Err("unexpected_helper_effect");
    }
    work
}

fn collect(request: &WireRequest) -> ValidationResult {
    let now = Instant::now();
    let elapsed = Duration::from_micros(request.elapsed_us.min(9_500_000));
    let started = now.checked_sub(elapsed).unwrap_or(now);
    let clock = AttemptClock {
        started,
        started_unix_ms: super::unix_ms().saturating_sub(elapsed.as_millis() as u64),
    };
    let mut result = ValidationResult::new(request);
    // Leave 500 ms for child kill/reap and supervisor delivery.
    let work_deadline = started + BUDGET - Duration::from_millis(500);
    if let Err(reason) = run_child(request, &mut result, work_deadline) {
        result.unavailable(reason);
    }
    result.stamp(&clock);
    opening_context::complete(request, &mut result);
    result
}
fn send_frame(stream: &mut UnixStream, bytes: &[u8]) -> io::Result<()> {
    stream.write_all(&(bytes.len() as u32).to_be_bytes())?;
    stream.write_all(bytes)
}
fn send_request(
    stream: &mut UnixStream,
    bytes: &[u8],
    deadline_at: Instant,
    cancelled: &AtomicBool,
) -> Result<(), &'static str> {
    let header = (bytes.len() as u32).to_be_bytes();
    for part in [&header[..], bytes] {
        let mut used = 0;
        while used < part.len() {
            deadline(deadline_at)?;
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled");
            }
            match stream.write(&part[used..]) {
                Ok(0) => return Err("worker_lost"),
                Ok(n) => used += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::WouldBlock
                    ) => {}
                Err(_) => return Err("worker_lost"),
            }
        }
    }
    Ok(())
}
fn receive_frame(
    stream: &mut UnixStream,
    limit: usize,
    deadline_at: Instant,
    cancelled: Option<&AtomicBool>,
) -> Result<Vec<u8>, &'static str> {
    let mut header = [0u8; 4];
    receive_exact(stream, &mut header, deadline_at, cancelled)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > limit {
        return Err("protocol_limit");
    }
    let mut body = vec![0u8; length];
    receive_exact(stream, &mut body, deadline_at, cancelled)?;
    Ok(body)
}
fn receive_exact(
    stream: &mut UnixStream,
    output: &mut [u8],
    deadline_at: Instant,
    cancelled: Option<&AtomicBool>,
) -> Result<(), &'static str> {
    let mut used = 0;
    while used < output.len() {
        deadline(deadline_at)?;
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("cancelled");
        }
        match stream.read(&mut output[used..]) {
            Ok(0) => return Err("worker_lost"),
            Ok(n) => used += n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return Err("worker_lost"),
        }
    }
    Ok(())
}
/// Dispatch only for an inherited socket pair created by [`validate`]. Redirected
/// stdin/stdout pipes and ordinary public invocation cannot become a worker.
pub fn worker_main() -> Option<i32> {
    let input: OwnedFd = io::stdin().as_fd().try_clone_to_owned().ok()?;
    let output: OwnedFd = io::stdout().as_fd().try_clone_to_owned().ok()?;
    let mut input = UnixStream::from(input);
    let mut output = UnixStream::from(output);
    input.peer_addr().ok()?;
    output.peer_addr().ok()?;
    input
        .set_read_timeout(Some(Duration::from_millis(50)))
        .ok()?;
    output
        .set_write_timeout(Some(Duration::from_millis(250)))
        .ok()?;
    let request = receive_frame(&mut input, CONTROL_MAX, Instant::now() + BUDGET, None).ok()?;
    let request: WireRequest = serde_json::from_slice(&request).ok()?;
    if RequestId::new(request.request_id.clone()).is_err()
        || SessionId::new(request.session_id.clone()).is_err()
        || request.elapsed_us > 9_500_000
    {
        return Some(1);
    }
    let result = collect(&request);
    let body = serde_json::to_vec(&result).ok()?;
    if body.len() > RESULT_MAX {
        return Some(1);
    }
    Some(if send_frame(&mut output, &body).is_ok() {
        0
    } else {
        1
    })
}

/// Runs one private worker and one official-stock child. No general LSP endpoint
/// or arbitrary edit command is exposed. Invoke again with fresh actual capture
/// after native mutation; never reuse a preflight verdict for post-change.
pub fn validate(
    request: ValidationRequest,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> ValidationResult {
    validate_until(request, clock, cancelled, clock.started + BUDGET)
}

fn validate_until(
    request: ValidationRequest,
    clock: AttemptClock,
    cancelled: &AtomicBool,
    attempt_deadline: Instant,
) -> ValidationResult {
    let mut wire = wire_request(request, &clock);
    // The inherited worker's existing finite budget is shortened, never renewed.
    wire.elapsed_us = 9_500_000u64.saturating_sub(
        attempt_deadline
            .saturating_duration_since(Instant::now())
            .as_micros()
            .min(9_500_000) as u64,
    );
    let mut result = ValidationResult::new(&wire);
    let mut cleanup_path = None;
    let process = (|| -> Result<ValidationResult, &'static str> {
        deadline(attempt_deadline)?;
        if cancelled.load(Ordering::Relaxed) {
            return Err("cancelled");
        }
        if wire
            .source
            .as_ref()
            .is_some_and(|source| source.len() > SOURCE_LIMIT_BYTES)
        {
            return Err("source_limit");
        }
        wire.private_dir = private_name()?;
        // Generate an owned name without touching the temporary filesystem on
        // the supervisor deadline path. The worker creates/verifies that dir.
        cleanup_path = Some(std::env::temp_dir().join(&wire.private_dir));
        let body = serde_json::to_vec(&wire).map_err(|_| "invalid_request")?;
        if body.len() > CONTROL_MAX {
            return Err("source_limit");
        }
        let (mut parent, child_socket) = UnixStream::pair().map_err(|_| "worker_unavailable")?;
        parent
            .set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(|_| "worker_unavailable")?;
        parent
            .set_write_timeout(Some(Duration::from_millis(50)))
            .map_err(|_| "worker_unavailable")?;
        let input: OwnedFd = child_socket
            .try_clone()
            .map_err(|_| "worker_unavailable")?
            .into();
        let output: OwnedFd = child_socket.into();
        let mut command = Command::new(std::env::current_exe().map_err(|_| "worker_unavailable")?);
        command
            .arg(INTERNAL_FLAG)
            .stdin(Stdio::from(input))
            .stdout(Stdio::from(output))
            .stderr(Stdio::null());
        // Kill the entire private worker/engine process group on supervisor loss.
        // Normal completion still reaps the engine in its owning worker.
        unsafe {
            command.pre_exec(|| {
                if setpgid(0, 0) == 0 {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error())
                }
            });
        }
        let worker = command.spawn().map_err(|_| "worker_unavailable")?;
        // Command retains its configured Stdio descriptors after spawn. Drop
        // those worker-side duplicates so worker loss produces EOF promptly.
        drop(command);
        // Once the worker can receive this request, silence cannot prove that
        // it never spawned Godot or that its scratch data has been removed.
        result.child_spawned = None;
        result.child_reaped = None;
        result.cleanup_confirmed = false;
        let receipt = (|| -> Result<ValidationResult, &'static str> {
            let reply_deadline = attempt_deadline - CLEANUP_RESERVE;
            send_request(&mut parent, &body, reply_deadline, cancelled)?;
            let reply = receive_frame(&mut parent, RESULT_MAX, reply_deadline, Some(cancelled))?;
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled");
            }
            let response: ValidationResult =
                serde_json::from_slice(&reply).map_err(|_| "worker_lost")?;
            if response.request_id != wire.request_id
                || response.session_id != wire.session_id
                || response.root_path != wire.root_path
                || response.purpose != wire.purpose
            {
                return Err("worker_lost");
            }
            Ok(response)
        })();
        drop(parent);
        // Kill the owned group even if the worker leader already exited: its
        // editor child may still have inherited the group and be running.
        kill_owned_group(worker.id());
        let confirmed = if receipt.as_ref().is_ok_and(|value| value.cleanup_confirmed) {
            let _ = super::reap_detached(worker, None);
            true
        } else {
            super::reap_detached(worker, cleanup_path.take())
                .and_then(|receipt| {
                    receipt
                        .recv_timeout(attempt_deadline.saturating_duration_since(Instant::now()))
                        .ok()
                })
                .unwrap_or(false)
        };
        result.cleanup_confirmed = confirmed;
        match receipt {
            Ok(mut received) => {
                received.cleanup_confirmed = confirmed;
                if !confirmed {
                    received.unavailable("cleanup_unavailable");
                }
                Ok(received)
            }
            Err(reason) => Err(reason),
        }
    })();
    match process {
        Ok(received) => result = received,
        Err(reason) => {
            result.unavailable(reason);
            // Interrupted worker clone cleanup belongs to the detached reaper.
        }
    }
    // Only the supervisor's clock defines the acceptance interval.
    result.stamp(&clock);
    opening_context::check_receipt(&wire, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn stock_warning_levels() -> BTreeMap<String, u8> {
        ACTIVE_WARNINGS
            .iter()
            .map(|key| {
                let level = if matches!(
                    *key,
                    "untyped_declaration"
                        | "inferred_declaration"
                        | "unsafe_property_access"
                        | "unsafe_method_access"
                        | "unsafe_cast"
                        | "unsafe_call_argument"
                        | "return_value_discarded"
                        | "missing_await"
                ) {
                    0
                } else if matches!(
                    *key,
                    "inference_on_variant"
                        | "native_method_override"
                        | "get_node_default_without_onready"
                        | "onready_with_export"
                ) {
                    2
                } else {
                    1
                };
                ((*key).to_owned(), level)
            })
            .collect()
    }

    pub(super) fn project() -> (PrivateClone, WireRequest) {
        let private = private_clone().unwrap();
        let project = private.0.join("selected");
        mkdir(&project).unwrap();
        mkdir(&project.join("scripts")).unwrap();
        fs::write(
            project.join("project.godot"),
            "config_version=5\n\n[application]\nconfig/name=\"fixture\"\n",
        )
        .unwrap();
        fs::write(project.join("scripts/main.gd"), "extends RefCounted\n").unwrap();
        let canonical = fs::canonicalize(project).unwrap();
        let request = WireRequest {
            request_id: "test".into(),
            session_id: "0123456789abcdef0123456789abcdef".into(),
            project_root: canonical.to_str().unwrap().into(),
            root_path: "res://scripts/main.gd".into(),
            source: None,
            purpose: Purpose::Unchanged,
            open_context: None,
            warnings: WarningSettings {
                enable: true,
                levels: stock_warning_levels(),
                directory_rules: BTreeMap::from([("res://addons".into(), 0)]),
                provenance: WarningProvenance {
                    source: WarningOrigin::EditorProjectSettings,
                    project_root: canonical.to_str().unwrap().into(),
                    session_id: "0123456789abcdef0123456789abcdef".into(),
                },
            },
            global_classes: Vec::new(),
            official_binary: "/no/engine".into(),
            private_dir: String::new(),
            elapsed_us: 0,
        };
        (private, request)
    }
    pub(super) fn admission(request: &WireRequest) -> Result<Closure, &'static str> {
        capture_closure(
            request,
            &mut ValidationResult::new(request),
            Instant::now() + Duration::from_secs(2),
        )
    }
    #[test]
    fn source_only_and_path_lexing_are_conservative_without_becoming_a_parser() {
        assert_eq!(
            resolve("res://scripts/nested/main.gd", "../base.gd").unwrap(),
            "res://scripts/base.gd"
        );
        assert_eq!(
            resolve("res://scripts/main.gd", "./native/level_one.gd").unwrap(),
            "res://scripts/native/level_one.gd"
        );
        assert_eq!(
            resolve("res://scripts/native/deep/child.gd", "./../sibling.gd").unwrap(),
            "res://scripts/native/sibling.gd"
        );
        assert!(resolve("res://scripts/main.gd", "../../outside.gd").is_err());
        assert!(resolve("res://scripts/main.gd", "uid://abc").is_err());
        assert!(resolve("res://scripts/main.gd", "/private/outside.gd").is_err());
        assert!(tokens(
            "# preload(\"hidden.tres\")\n@tool\nstatic var a = \"\"\nfunc _static_init(): pass\n"
        )
        .is_ok());
        assert!(tokens("const X = \"a\\\"b\"").is_err());
        assert!(tokens("const X = \"\"\"raw\"\"\"").is_err());
    }
    #[test]
    fn transitive_dependencies_resolve_from_each_referring_source() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        fs::write(root.join("scripts/main.gd"), "extends \"parts/child.gd\"\n").unwrap();
        mkdir(&root.join("scripts/parts")).unwrap();
        fs::write(
            root.join("scripts/parts/child.gd"),
            "extends \"../base.gd\"\nstatic var value = preload(\"../base.gd\").new()\n",
        )
        .unwrap();
        fs::write(root.join("scripts/base.gd"), "extends RefCounted\n").unwrap();
        request.source = Some("@tool\nextends \"parts/child.gd\"\n".into());
        request.purpose = Purpose::Preflight;
        let closure = admission(&request).unwrap();
        assert_eq!(
            closure
                .sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<Vec<_>>(),
            [
                "res://scripts/main.gd",
                "res://scripts/parts/child.gd",
                "res://scripts/base.gd"
            ]
        );
        assert!(closure.missing.is_empty());
        assert!(recheck(&request, &closure, Instant::now() + Duration::from_secs(2)).is_ok());
        fs::write(root.join("scripts/base.gd"), "extends Resource\n").unwrap();
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("source_changed")
        );
    }
    #[test]
    fn unsafe_links_and_effectful_resources_refuse_before_launch() {
        let (_tmp, mut request) = project();
        for (source, reason) in [
            ("extends \"../../outside.gd\"\n", "unsafe_link"),
            (
                "const X = preload(\"res://payload.tres\")\n",
                "non_gd_dependency",
            ),
            ("const X = preload(computed)\n", "computed_dependency"),
            ("const X = preload(\"uid://special\")\n", "unsafe_link"),
            ("const X = load(\"helper.gd\")\n", "unsupported_context"),
            ("class_name ProjectGlobal\n", "unsupported_context"),
            (
                "const X = ProjectSpecific.new()\n",
                "unreproducible_global_class",
            ),
            ("const X = \"file:///private/outside\"\n", "unsafe_link"),
        ] {
            request.source = Some(source.into());
            request.purpose = Purpose::Preflight;
            assert_eq!(admission(&request).err(), Some(reason), "{source}");
        }
    }
    #[test]
    fn symlink_and_source_limits_cannot_be_staged() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        symlink(root.join("project.godot"), root.join("scripts/linked.gd")).unwrap();
        request.source = Some("const X = preload(\"linked.gd\")\n".into());
        request.purpose = Purpose::Preflight;
        assert_eq!(admission(&request).err(), Some("unsafe_path"));
        request.source = Some(" ".repeat(SOURCE_LIMIT_BYTES + 1));
        assert_eq!(admission(&request).err(), Some("source_limit"));
    }
    #[test]
    fn missing_dependency_is_referrer_bound_and_context_changes_invalidate() {
        let (_tmp, mut request) = project();
        request.source = Some("extends \"missing.gd\"\n".into());
        request.purpose = Purpose::Preflight;
        let closure = admission(&request).unwrap();
        assert_eq!(closure.missing.len(), 1);
        assert_eq!(closure.missing[0].referring, request.root_path);
        assert_eq!(closure.missing[0].path, "res://scripts/missing.gd");
        let root = Path::new(&request.project_root);
        fs::write(
            root.join("project.godot"),
            "config_version=5\n\n[application]\nconfig/name=\"changed\"\n",
        )
        .unwrap();
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("context_changed")
        );
    }
    #[test]
    fn owner_fence_needs_matching_diagnostics_and_shaped_symbols() {
        let (_tmp, request) = project();
        let mut result = ValidationResult::new(&request);
        let closure = admission(&request).unwrap();
        result.sources.push(SourceEvidence {
            path: request.root_path.clone(),
            sha256: closure.sources[0].captured.sha256.clone(),
            utf8_bytes: closure.sources[0].captured.bytes,
            device: None,
            inode: None,
            diagnostics_completed: false,
            symbols_completed: false,
        });
        let project_path = PathBuf::from(&request.project_root);
        let expected_uri = uri(&project_path.join("scripts/main.gd")).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            let limit = Instant::now() + Duration::from_secs(2);
            let open = read_lsp(&mut connection, limit).unwrap();
            assert_eq!(open["params"]["textDocument"]["uri"], expected_uri);
            assert_eq!(
                open["params"]["textDocument"]["text"],
                "extends RefCounted\n"
            );
            assert_eq!(open["params"]["textDocument"]["version"], 1);
            let symbols = read_lsp(&mut connection, limit).unwrap();
            assert_eq!(symbols["method"], "textDocument/documentSymbol");
            send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                "params":{"uri":expected_uri,"diagnostics":[{"severity":1,
                    "range":{"start":{"line":1,"character":3},
                             "end":{"line":1,"character":14}},"message":"typed error"}]}}),
            )
            .unwrap();
            send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0","id":2,"result":[
                {"name":"main.gd","kind":5,"range":{"start":{"line":0},"end":{"line":1}}}]}),
            )
            .unwrap();
        });
        let mut connection = TcpStream::connect(address).unwrap();
        source_fence(
            &mut connection,
            &closure.sources[0],
            0,
            &project_path,
            &mut result,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
        server.join().unwrap();
        assert!(result.sources[0].diagnostics_completed && result.sources[0].symbols_completed);
        assert_eq!(
            result.diagnostics[0].source_sha256,
            closure.sources[0].captured.sha256
        );
        assert_eq!(
            (result.diagnostics[0].line, result.diagnostics[0].column),
            (1, 3)
        );
    }
    #[test]
    fn symbols_without_owner_uri_diagnostics_cannot_complete() {
        let (_tmp, request) = project();
        let closure = admission(&request).unwrap();
        let mut result = ValidationResult::new(&request);
        attribute(&mut result, &closure.sources[0]);
        let project_path = PathBuf::from(&request.project_root);
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            let limit = Instant::now() + Duration::from_secs(2);
            read_lsp(&mut connection, limit).unwrap(); // exact didOpen
            read_lsp(&mut connection, limit).unwrap(); // symbol request
            send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                "params":{"uri":"file:///private/other.gd","diagnostics":[]}}),
            )
            .unwrap();
            send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0","id":2,"result":[
                {"name":"main.gd","kind":5,"range":{"start":{"line":0},"end":{"line":1}}}]}),
            )
            .unwrap();
        });
        let mut client = TcpStream::connect(address).unwrap();
        assert_eq!(
            source_fence(
                &mut client,
                &closure.sources[0],
                0,
                &project_path,
                &mut result,
                Instant::now() + Duration::from_secs(2)
            ),
            Err("protocol_loss")
        );
        server.join().unwrap();
        assert!(!result.sources[0].symbols_completed);
        assert!(!result.sources[0].diagnostics_completed);
        assert!(result.diagnostics.is_empty());
    }
    #[test]
    fn early_symbols_cannot_be_completed_by_later_owner_diagnostics() {
        let (_tmp, request) = project();
        let closure = admission(&request).unwrap();
        let mut result = ValidationResult::new(&request);
        attribute(&mut result, &closure.sources[0]);
        let project_path = PathBuf::from(&request.project_root);
        let source_uri = uri(&project_path.join("scripts/main.gd")).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            read_lsp(&mut connection, deadline).unwrap();
            read_lsp(&mut connection, deadline).unwrap();
            send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0","id":2,"result":[
                {"name":"main.gd","kind":5,"range":{"start":{"line":0},"end":{"line":1}}}]}),
            )
            .unwrap();
            let _ = send_lsp(
                &mut connection,
                &json!({"jsonrpc":"2.0",
                "method":"textDocument/publishDiagnostics",
                "params":{"uri":source_uri,"diagnostics":[]}}),
            );
        });
        let mut connection = TcpStream::connect(address).unwrap();
        assert_eq!(
            source_fence(
                &mut connection,
                &closure.sources[0],
                0,
                &project_path,
                &mut result,
                Instant::now() + Duration::from_secs(2)
            ),
            Err("protocol_loss")
        );
        server.join().unwrap();
        assert!(!result.sources[0].diagnostics_completed);
        assert!(!result.sources[0].symbols_completed);
    }

    #[test]
    fn malformed_diagnostics_cannot_turn_into_empty_valid_fence() {
        let (_tmp, request) = project();
        let closure = admission(&request).unwrap();
        let project_path = PathBuf::from(&request.project_root);
        let uri = uri(&project_path.join("scripts/main.gd")).unwrap();
        for entry in [
            json!(null),
            json!({}),
            json!({"severity":"2","message":"warning",
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}),
            json!({"severity":5,"message":"warning",
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}}}),
            json!({"severity":2,"message":"warning",
                "range":{"start":{"line":0,"character":0}}}),
        ] {
            let mut result = ValidationResult::new(&request);
            attribute(&mut result, &closure.sources[0]);
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            let address = listener.local_addr().unwrap();
            let owned_uri = uri.clone();
            let server = thread::spawn(move || {
                let (mut connection, _) = listener.accept().unwrap();
                let deadline = Instant::now() + Duration::from_secs(2);
                read_lsp(&mut connection, deadline).unwrap();
                read_lsp(&mut connection, deadline).unwrap();
                send_lsp(
                    &mut connection,
                    &json!({"jsonrpc":"2.0",
                    "method":"textDocument/publishDiagnostics",
                    "params":{"uri":owned_uri,"diagnostics":[entry]}}),
                )
                .unwrap();
                let _ = send_lsp(
                    &mut connection,
                    &json!({"jsonrpc":"2.0","id":2,"result":[
                    {"name":"main.gd","kind":5,
                     "range":{"start":{"line":0},"end":{"line":1}}}]}),
                );
            });
            let mut connection = TcpStream::connect(address).unwrap();
            assert_eq!(
                source_fence(
                    &mut connection,
                    &closure.sources[0],
                    0,
                    &project_path,
                    &mut result,
                    Instant::now() + Duration::from_secs(2)
                ),
                Err("protocol_loss")
            );
            server.join().unwrap();
            assert!(!result.sources[0].diagnostics_completed);
            assert!(!result.sources[0].symbols_completed);
        }
    }
    #[test]
    fn dependency_count_and_changed_missing_parent_are_unavailable() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        let mut script = String::from("extends RefCounted\n");
        for i in 0..33 {
            fs::write(
                root.join(format!("scripts/d{i}.gd")),
                "extends RefCounted\n",
            )
            .unwrap();
            script.push_str(&format!("const D{i} = preload(\"d{i}.gd\")\n"));
        }
        request.purpose = Purpose::Preflight;
        request.source = Some(script);
        let mut partial = ValidationResult::new(&request);
        assert_eq!(
            capture_closure(
                &request,
                &mut partial,
                Instant::now() + Duration::from_secs(3)
            )
            .err(),
            Some("dependency_limit")
        );
        assert_eq!(partial.sources.len(), 33); // root and first 32, never false valid
        assert!(partial.sources.iter().all(|s| !s.diagnostics_completed));
        assert_eq!(partial.child_spawned, Some(false));
        mkdir(&root.join("other")).unwrap();
        request.source = Some("extends \"../other/missing.gd\"\n".into());
        let missing = admission(&request).unwrap();
        fs::rename(root.join("other"), root.join("old-other")).unwrap();
        mkdir(&root.join("other")).unwrap();
        assert_eq!(
            recheck(&request, &missing, Instant::now() + Duration::from_secs(2)),
            Err("dependency_changed")
        );
    }
    #[test]
    fn aggregate_dependency_bytes_limit_retains_bounded_partial_witness() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        let mut script = String::from("extends RefCounted\n");
        let dependency = format!("extends RefCounted\n#{}\n", "x".repeat(510 * 1024));
        assert!(dependency.len() <= SOURCE_LIMIT_BYTES);
        for index in 0..9 {
            fs::write(root.join(format!("scripts/large{index}.gd")), &dependency).unwrap();
            script.push_str(&format!("const D{index} = preload(\"large{index}.gd\")\n"));
        }
        request.purpose = Purpose::Preflight;
        request.source = Some(script);
        let mut partial = ValidationResult::new(&request);
        assert_eq!(
            capture_closure(
                &request,
                &mut partial,
                Instant::now() + Duration::from_secs(5)
            )
            .err(),
            Some("dependency_limit")
        );
        assert_eq!(partial.sources.len(), 9); // root plus eight accepted dependencies
        assert!(partial
            .sources
            .iter()
            .skip(1)
            .all(|s| s.utf8_bytes == dependency.len()));
    }
    #[test]
    fn effective_warning_profile_and_selected_config_must_remain_bound() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        fs::write(
            root.join("project.godot"),
            "config_version=5\n\n[debug]\ngdscript/warnings/enable=false\n\
             gdscript/warnings/enable.editor=true\n\
             gdscript/warnings/integer_division=1\n\
             gdscript/warnings/integer_division.editor=2\n",
        )
        .unwrap();
        request.warnings.levels.insert("integer_division".into(), 2);
        let closure = admission(&request).unwrap();
        let staged = private_clone().unwrap();
        let cloned = stage(&closure, &staged, &request.warnings).unwrap();
        let reproduced = fs::read_to_string(cloned.join("project.godot")).unwrap();
        assert!(reproduced.contains("gdscript/warnings/enable=true\n"));
        assert!(reproduced.contains("gdscript/warnings/integer_division=2\n"));
        assert!(!reproduced.contains("gdscript/warnings/enable.editor="));
        request
            .warnings
            .directory_rules
            .insert("res://scripts".into(), 0);
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("context_changed")
        );
        request.warnings.directory_rules.remove("res://scripts");
        fs::write(
            root.join("project.godot"),
            "config_version=5\n\n[debug]\ngdscript/warnings/integer_division=2\n\
             gdscript/warnings/directory_rules={\"res://scripts\": 0}\n",
        )
        .unwrap();
        assert_eq!(
            recheck(&request, &closure, Instant::now() + Duration::from_secs(2)),
            Err("context_changed")
        );
    }
    #[test]
    fn missing_or_cross_session_editor_warning_witness_refuses_prelaunch() {
        let (_tmp, mut request) = project();
        request.warnings.levels.remove("unused_variable");
        assert_eq!(admission(&request).err(), Some("incomplete_editor_context"));
        request.warnings.levels = stock_warning_levels();
        request.warnings.provenance.session_id = "fedcba9876543210fedcba9876543210".into();
        assert_eq!(admission(&request).err(), Some("context_mismatch"));
    }
    #[test]
    fn unrelated_editor_plugin_configuration_is_not_staged_or_a_source_dependency() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        fs::write(
            root.join("project.godot"),
            "config_version=5\n\n[editor_plugins]\n\
             enabled=PackedStringArray(\"res://addons/integration/plugin.cfg\")\n",
        )
        .unwrap();
        let closure = admission(&request).unwrap();
        assert_eq!(closure.sources.len(), 1);
        assert!(closure.missing.is_empty());
        request.source = Some("const X = PluginProvidedClass.new()\n".into());
        request.purpose = Purpose::Preflight;
        assert_eq!(
            admission(&request).err(),
            Some("unreproducible_global_class")
        );
    }
    #[test]
    fn disposable_clone_detects_effects_and_requested_logs_without_exposing_source() {
        let (_tmp, request) = project();
        let closure = admission(&request).unwrap();
        let disposable = private_clone().unwrap();
        let project = stage(&closure, &disposable, &request.warnings).unwrap();
        assert_eq!(inspect_clone(&disposable, &closure), Ok((false, false)));
        fs::write(project.join("scripts/main.gd.uid"), "uid").unwrap();
        assert_eq!(inspect_clone(&disposable, &closure), Ok((false, false)));
        fs::write(project.join("runtime-marker.txt"), "private sentinel").unwrap();
        assert_eq!(inspect_clone(&disposable, &closure), Ok((true, false)));
        fs::write(disposable.0.join("home/engine.log"), "raw stderr").unwrap();
        assert_eq!(inspect_clone(&disposable, &closure), Ok((true, true)));
    }
    #[test]
    fn source_identity_and_exclusive_stage_preserve_proposed_bytes() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        fs::hard_link(root.join("scripts/main.gd"), root.join("scripts/hard.gd")).unwrap();
        request.purpose = Purpose::Preflight;
        request.source = Some("extends RefCounted\nconst A = preload(\"hard.gd\")\n".into());
        assert_eq!(admission(&request).err(), Some("source_alias"));

        let case = root.join("scripts/MAIN.gd");
        if case.metadata().is_ok()
            && fs::metadata(&case).unwrap().ino()
                == fs::metadata(root.join("scripts/main.gd")).unwrap().ino()
        {
            request.source = Some("extends RefCounted\nconst A = preload(\"MAIN.gd\")\n".into());
            assert_eq!(admission(&request).err(), Some("source_alias"));
        }

        fs::write(root.join("scripts/cafe\u{301}.gd"), "extends RefCounted\n").unwrap();
        if let Ok(composed) = fs::metadata(root.join("scripts/café.gd")) {
            if composed.ino()
                == fs::metadata(root.join("scripts/cafe\u{301}.gd"))
                    .unwrap()
                    .ino()
            {
                request.source = Some("extends RefCounted\nconst A = preload(\"cafe\u{301}.gd\")\nconst B = preload(\"café.gd\")\n".into());
                assert_eq!(admission(&request).err(), Some("source_alias"));
            }
        }

        request.source = Some("extends RefCounted\n".into());
        let mut closure = admission(&request).unwrap();
        let proposed = closure.sources[0].captured.text.clone();
        let mut second = closure.sources[0].clone();
        second.captured.text = "extends Resource\n".into();
        closure.sources.push(second);
        let disposable = private_clone().unwrap();
        assert_eq!(
            stage(&closure, &disposable, &request.warnings).err(),
            Some("source_alias")
        );
        assert_eq!(
            fs::read_to_string(disposable.0.join("project/scripts/main.gd")).unwrap(),
            proposed
        );
    }

    #[test]
    fn exited_worker_leader_does_not_release_owned_editor_group() {
        let (private, _) = project();
        let marker = private.0.join("escaped-editor-marker");
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("sleep 1 && touch \"$MARKER\" &")
            .env("MARKER", &marker)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(|| {
                if setpgid(0, 0) == 0 {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error())
                }
            });
        }
        let mut leader = command.spawn().unwrap();
        let limit = Instant::now() + Duration::from_secs(2);
        while leader.try_wait().unwrap().is_none() && Instant::now() < limit {
            thread::sleep(TICK);
        }
        assert!(leader.try_wait().unwrap().is_some());
        kill_owned_group(leader.id());
        let _ = super::super::reap_detached(leader, None);
        thread::sleep(Duration::from_millis(1100));
        assert!(
            !marker.exists(),
            "orphaned editor escaped its exited worker group"
        );
    }

    #[test]
    fn detached_worker_reaper_owns_interrupted_clone_cleanup() {
        let mut private = private_clone().unwrap();
        let path = std::mem::take(&mut private.0);
        drop(private); // Simulate a worker killed before its Drop can remove this path.
        let child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 2")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let returned_at = Instant::now();
        let receipt = super::super::reap_detached(child, Some(path.clone())).unwrap();
        let returned_in = returned_at.elapsed();
        assert_eq!(receipt.recv_timeout(Duration::from_secs(3)), Ok(true));
        let cleaned = !path.exists();
        if !cleaned {
            let _ = fs::remove_dir_all(&path);
        }
        assert!(
            returned_in < Duration::from_secs(1),
            "delivery waited for worker or filesystem cleanup"
        );
        assert!(cleaned, "detached reaper left an owned clone behind");
    }

    #[test]
    fn selected_global_class_names_include_lowercase_and_collisions() {
        let (_tmp, mut request) = project();
        request.purpose = Purpose::Preflight;
        request.global_classes = vec!["ExistingGlobal".into(), "existing_global".into()];
        request.source = Some(
            "extends RefCounted\nclass ExistingGlobal extends RefCounted:\n\tvar value: int = 0\n"
                .into(),
        );
        assert_eq!(
            admission(&request).err(),
            Some("unreproducible_global_class")
        );
        request.source = Some("extends RefCounted\nvar item: existing_global\n".into());
        assert_eq!(
            admission(&request).err(),
            Some("unreproducible_global_class")
        );
        request.source = Some("extends RefCounted\nvar item: int = 1\n".into());
        assert!(admission(&request).is_ok());
        request.global_classes = vec!["x".into(); 257];
        assert_eq!(admission(&request).err(), Some("incomplete_editor_context"));
    }

    #[test]
    fn missing_literals_require_distinct_full_references_and_lines() {
        let (_tmp, mut request) = project();
        let root = Path::new(&request.project_root);
        mkdir(&root.join("scripts/a")).unwrap();
        mkdir(&root.join("scripts/b")).unwrap();
        request.purpose = Purpose::Preflight;
        request.source =
            Some("extends \"a/missing.gd\"\nconst B = preload(\"b/missing.gd\")\n".into());
        let closure = admission(&request).unwrap();
        assert_eq!(closure.missing.len(), 2);
        let hash = closure.sources[0].captured.sha256.clone();
        let mut diagnostics = vec![ErrorDiagnostic {
            path: request.root_path.clone(),
            source_sha256: hash.clone(),
            line: 0,
            column: 0,
            message: "Could not resolve script \"a/missing.gd\"".into(),
        }];
        assert_eq!(
            missing_attributed(&closure, &diagnostics),
            Err("missing_unattributed")
        );
        diagnostics.push(ErrorDiagnostic {
            path: request.root_path.clone(),
            source_sha256: hash,
            line: 1,
            column: 10,
            message: "Could not resolve script \"b/missing.gd\"".into(),
        });
        assert_eq!(missing_attributed(&closure, &diagnostics), Ok(()));
        diagnostics[1].line = 0;
        assert_eq!(
            missing_attributed(&closure, &diagnostics),
            Err("missing_unattributed")
        );
    }
    #[test]
    fn wrong_binary_refuses_before_launch_without_touching_selected_source() {
        let (_tmp, request) = project();
        let source = Path::new(&request.project_root).join("scripts/main.gd");
        let before = fs::read(&source).unwrap();
        let result = collect(&request);
        assert_eq!(result.status, "unavailable");
        assert_eq!(result.reason.as_deref(), Some("wrong_binary"));
        assert_eq!(result.child_spawned, Some(false));
        assert!(result.clone_path.is_none());
        assert_eq!(fs::read(source).unwrap(), before);
    }
}
