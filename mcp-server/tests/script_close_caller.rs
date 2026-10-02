use serde_json::Value;
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
const BASE: [&str; 6] = [
    "--registry",
    "/tmp/gak-close-missing-registry",
    "--project",
    "/tmp/gak-close-missing-project",
    "--script",
    "res://subject.gd",
];
fn command(args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_close-gdscript"));
    c.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    c
}
fn invoke(args: &[&str], input: &[u8]) -> (i32, Value, String, Duration) {
    let began = Instant::now();
    let mut child = command(args).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.ends_with('\n'));
    (
        output.status.code().unwrap(),
        serde_json::from_str(&stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        began.elapsed(),
    )
}
#[test]
fn malformed_flags_and_unowned_worker_entry_are_structured_and_no_effect() {
    for args in [
        vec![],
        vec!["--force", "true"],
        vec!["--discard", "true"],
        vec!["--retry", "1"],
        vec!["--source", "SOURCE_SENTINEL"],
        vec!["--internal-close-worker"],
        vec!["--internal-stock-validation-worker"],
    ] {
        let (code, value, stderr, elapsed) = invoke(&args, b"");
        assert_eq!(code, 2);
        assert_eq!(value["reason"], "invalid_request");
        assert_eq!(value["application"], "not_applied");
        assert!(value["resolved_target"].is_null());
        assert!(!value.to_string().contains("SOURCE_SENTINEL"));
        assert!(stderr.is_empty());
        assert!(elapsed < Duration::from_secs(10));
    }
}
#[test]
fn malformed_stdin_is_not_reflected_and_cannot_route() {
    for input in [b"{}".as_slice(),b"{} {}",b"\xff",b"{\"schema_version\":1,\"request_id\":\"close-test\",\"basis\":null,\"source\":\"SOURCE_SENTINEL\"}",b"{\"schema_version\":1,\"schema_version\":1,\"request_id\":\"close-test\",\"basis\":null}"]{let(code,value,stderr,_)=invoke(&BASE,input);assert_eq!(code,2);assert_eq!(value["reason"],"invalid_request");assert_eq!(value["application"],"not_applied");assert!(value["resolved_target"].is_null());assert!(value["observation"].is_null());assert!(!value.to_string().contains("SOURCE_SENTINEL"));assert!(stderr.is_empty());}
}
#[test]
fn blocked_real_stdin_uses_original_clock_and_terminal_delivery() {
    let began = Instant::now();
    let mut child = command(&BASE).spawn().unwrap();
    let held = child.stdin.take().unwrap();
    let output = child.wait_with_output().unwrap();
    drop(held);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(value["reason"], "timeout");
    assert_eq!(value["application"], "not_applied");
    assert!(value["resolved_target"].is_null());
    assert!(began.elapsed() < Duration::from_secs(10));
}
#[test]
fn source_oversize_is_rejected_without_waiting_for_eof() {
    let mut child = command(&BASE).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        let oversized = vec![b' '; 12 * 1024 * 1024 + 1];
        let _ = stdin.write_all(&oversized);
        stdin
    });
    let output = child.wait_with_output().unwrap();
    drop(writer.join().unwrap());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(value["reason"], "invalid_request");
    assert_eq!(value["application"], "not_applied");
}
#[test]
fn owned_worker_rejects_bad_framing_without_a_public_verdict() {
    for length in [0u32, 4097] {
        let (mut parent, child) = UnixStream::pair().unwrap();
        let read_fd: OwnedFd = child.try_clone().unwrap().into();
        let write_fd: OwnedFd = child.into();
        let mut c = Command::new(env!("CARGO_BIN_EXE_close-gdscript"));
        let mut child = c
            .arg("--internal-close-worker")
            .stdin(Stdio::from(read_fd))
            .stdout(Stdio::from(write_fd))
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(c); // The reusable Command otherwise retains the child's socket endpoint.
        parent
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        parent.write_all(&length.to_be_bytes()).unwrap();
        parent.shutdown(std::net::Shutdown::Write).unwrap();
        let mut bytes = Vec::new();
        parent.read_to_end(&mut bytes).unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(1));
        assert!(bytes.is_empty());
    }
}
#[test]
fn connected_worker_eof_cannot_emit_an_acquisition_success() {
    let (mut parent, child) = UnixStream::pair().unwrap();
    let read_fd: OwnedFd = child.try_clone().unwrap().into();
    let write_fd: OwnedFd = child.into();
    let mut child = Command::new(env!("CARGO_BIN_EXE_close-gdscript"))
        .arg("--internal-close-worker")
        .stdin(Stdio::from(read_fd))
        .stdout(Stdio::from(write_fd))
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    parent
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    parent.write_all(&10u32.to_be_bytes()).unwrap();
    parent.write_all(b"{\"v\":1}").unwrap();
    parent.shutdown(std::net::Shutdown::Write).unwrap();
    let mut bytes = Vec::new();
    parent.read_to_end(&mut bytes).unwrap();
    assert_eq!(child.wait().unwrap().code(), Some(1));
    assert!(bytes.is_empty());
}
#[test]
fn valid_null_input_preserves_exact_requested_target_on_routing_refusal() {
    let (code, value, stderr, elapsed) = invoke(
        &BASE,
        br#"{"schema_version":1,"request_id":"close-test","basis":null}"#,
    );
    assert_eq!(code, 3);
    assert_eq!(value["application"], "not_applied");
    assert_eq!(value["requested_target"]["script_path"], "res://subject.gd");
    assert!(value["observation"].is_null());
    assert!(stderr.is_empty());
    assert!(elapsed < Duration::from_secs(10));
}

#[test]
fn sigint_and_sigterm_cancel_blocked_input_without_effect_authority() {
    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    for signal in [2, 15] {
        let began = Instant::now();
        let mut child = command(&BASE).spawn().unwrap();
        let mut held = child.stdin.take().unwrap();
        // Filling beyond the supported host's pipe capacity establishes that
        // the stdin reader is running, after handler installation. Keep EOF held.
        for _ in 0..64 {
            held.write_all(&[b' '; 64 * 1024]).unwrap();
        }
        // SAFETY: This is the owned child PID and a standard interrupt/terminate signal.
        assert_eq!(unsafe { kill(child.id() as i32, signal) }, 0);
        let output = child.wait_with_output().unwrap();
        drop(held);
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(3));
        assert_eq!(value["reason"], "cancelled");
        assert_eq!(value["application"], "not_applied");
        assert!(value["resolved_target"].is_null());
        assert!(value["observation"].is_null());
        assert!(began.elapsed() < Duration::from_secs(10));
    }
}
