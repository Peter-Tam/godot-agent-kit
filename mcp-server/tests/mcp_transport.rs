use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn wait_exit(child: &mut Child, bound: Duration) -> ExitStatus {
    let deadline = Instant::now() + bound;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("MCP exit exceeded bound");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

struct Peer {
    child: Child,
    input: Option<ChildStdin>,
    output: mpsc::Receiver<Value>,
}
impl Peer {
    fn start() -> Self {
        Self::start_at(std::path::Path::new("/nonexistent/mcp-regression-registry"))
    }
    fn start_at(registry: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_godot-agent-kit-mcp"))
            .arg("--registry")
            .arg(registry)
            .env("RUST_LOG", "trace")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (sender, output) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                if sender.send(serde_json::from_str(&line).unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            output,
        }
    }
    fn raw(&mut self, text: &str) {
        self.input
            .as_mut()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }
    fn send(&mut self, value: Value) {
        self.raw(&format!("{value}\n"));
    }
    fn read(&mut self) -> Value {
        self.output
            .recv_timeout(Duration::from_secs(3))
            .expect("bounded MCP response")
    }
    fn initialize(&mut self) {
        self.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"regression","version":"1"}}}));
        let result = self.read();
        assert_eq!(result["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(result["result"]["capabilities"], json!({"tools":{}}));
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn negotiation_gating_and_catalog() {
    let mut peer = Peer::start();
    peer.send(json!({"jsonrpc":"2.0","id":0,"method":"tools/list"}));
    assert_eq!(peer.read()["error"]["code"], -32600);
    peer.send(json!({"jsonrpc":"2.0","id":"ping","method":"ping"}));
    assert_eq!(peer.read()["id"], "ping");
    peer.initialize();
    peer.send(json!({"jsonrpc":"2.0","id":"negotiated-ping","method":"ping"}));
    let ping = peer.read();
    assert_eq!(ping["id"], "negotiated-ping");
    assert_eq!(ping["result"], json!({}));
    assert!(ping.get("error").is_none());
    peer.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let response = peer.read();
    let names: Vec<_> = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["discover_scripts", "read_script", "edit_script"]);
    assert!(response["result"].get("nextCursor").is_none());
    peer.send(json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{"cursor":"anything"}}));
    assert_eq!(peer.read()["error"]["code"], -32602);
    peer.send(json!({"jsonrpc":"2.0","id":4,"method":"resources/list"}));
    assert_eq!(peer.read()["error"]["code"], -32601);
}

#[test]
fn malformed_frames_ids_and_privacy() {
    let mut peer = Peer::start();
    peer.raw("{\"sentinel_private_source\":\n");
    let parse = peer.read();
    assert_eq!(parse["error"]["code"], -32700);
    assert!(parse["id"].is_null());
    peer.raw("{\"jsonrpc\":\"2.0\",\"id\":1,\"id\":2,\"method\":\"ping\"}\n");
    assert_eq!(peer.read()["error"]["code"], -32700);
    for id in [
        json!(1.5),
        json!("x".repeat(129)),
        json!(9223372036854775808u64),
    ] {
        peer.send(json!({"jsonrpc":"2.0","id":id,"method":"ping"}));
        assert_eq!(peer.read()["error"]["code"], -32600);
    }
    for id in [json!(i64::MIN), json!(i64::MAX), json!("é".repeat(64))] {
        peer.send(json!({"jsonrpc":"2.0","id":id,"method":"ping"}));
        assert_eq!(peer.read()["id"], id);
    }
    peer.input.take();
    let status = wait_exit(&mut peer.child, Duration::from_secs(2));
    assert!(status.success());
    use std::io::Read;
    let mut stderr = String::new();
    peer.child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(!stderr.contains("sentinel_private_source"));
}

#[test]
fn unknown_tools_and_invalid_arguments_are_distinct() {
    let mut peer = Peer::start();
    peer.initialize();
    for params in [
        json!({"name":"unknown","arguments":{}}),
        json!({"name":"read_script","arguments":null}),
        json!({"name":"read_script","extra":1}),
    ] {
        peer.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":params}));
        assert_eq!(peer.read()["error"]["code"], -32602);
    }
    peer.send(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_script","arguments":{"project_root":42}}}));
    let response = peer.read();
    assert_eq!(response["result"]["isError"], true);
    assert_eq!(
        response["result"]["structuredContent"]["error"]["category"],
        "input"
    );
}

#[test]
fn cli_is_no_effect_and_strict() {
    let exe = env!("CARGO_BIN_EXE_godot-agent-kit-mcp");
    for arg in ["--help", "--version"] {
        let output = Command::new(exe).arg(arg).output().unwrap();
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
    }
    for args in [
        vec![],
        vec!["--registry"],
        vec!["--registry", "relative"],
        vec!["--registry", "/tmp/a", "--registry", "/tmp/b"],
        vec!["--validator-engine", "/tmp/PRIVATE_ENGINE_SENTINEL"],
        vec!["--registry", "/tmp/a", "--validator-engine"],
        vec![
            "--registry",
            "/tmp/a",
            "--validator-engine",
            "PRIVATE_ENGINE_SENTINEL",
        ],
        vec![
            "--registry",
            "/tmp/a",
            "--validator-engine",
            "/tmp/PRIVATE_ENGINE_SENTINEL\n",
        ],
        vec![
            "--registry",
            "/tmp/a",
            "--validator-engine",
            "/tmp/a",
            "--validator-engine",
            "/tmp/PRIVATE_ENGINE_SENTINEL",
        ],
        vec![
            "--registry",
            "/tmp/a",
            "--unknown",
            "/tmp/PRIVATE_ENGINE_SENTINEL",
        ],
        vec!["--unknown"],
        vec!["--help", "--registry", "/tmp/a"],
        vec![
            "--version",
            "--validator-engine",
            "/tmp/PRIVATE_ENGINE_SENTINEL",
        ],
    ] {
        let output = Command::new(exe).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("PRIVATE_ENGINE_SENTINEL"));
    }
}

#[test]
fn configured_engine_path_rejects_oversize_and_non_utf8_without_disclosure() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let exe = env!("CARGO_BIN_EXE_godot-agent-kit-mcp");
    for path in [
        OsString::from(format!("/PRIVATE_ENGINE_SENTINEL{}", "x".repeat(1024))),
        OsString::from_vec(b"/PRIVATE_ENGINE_SENTINEL_\xff".to_vec()),
    ] {
        let output = Command::new(exe)
            .args(["--registry", "/tmp/a", "--validator-engine"])
            .arg(path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("PRIVATE_ENGINE_SENTINEL"));
    }
}

#[test]
fn configured_wrong_engine_fails_before_protocol_without_disclosure_or_cold_fallback() {
    let exe = env!("CARGO_BIN_EXE_godot-agent-kit-mcp");
    for engine in [exe, "/nonexistent/PRIVATE_ENGINE_SENTINEL"] {
        for engine_first in [false, true] {
            let registry_args = ["--registry", "/nonexistent/mcp-regression-registry"];
            let engine_args = ["--validator-engine", engine];
            let mut command = Command::new(exe);
            if engine_first {
                command.args(engine_args).args(registry_args);
            } else {
                command.args(registry_args).args(engine_args);
            }
            let mut child = command
                .env("RUST_LOG", "trace")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let _ = child.stdin.as_mut().unwrap().write_all(
                b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"PRIVATE_SOURCE_SENTINEL\",\"version\":\"1\"}}}\n",
            );
            // Keep stdin open: configured startup failure must not depend on EOF.
            let status = wait_exit(&mut child, Duration::from_secs(12));
            assert_eq!(status.code(), Some(1));
            drop(child.stdin.take());
            let output = child.wait_with_output().unwrap();
            assert!(output.stdout.is_empty());
            let diagnostics = String::from_utf8_lossy(&output.stderr);
            assert!(!diagnostics.contains("PRIVATE_ENGINE_SENTINEL"));
            assert!(!diagnostics.contains("PRIVATE_SOURCE_SENTINEL"));
            assert!(!diagnostics.contains(engine));
        }
    }
}

#[test]
fn private_workers_reject_ordinary_pipe_authority_without_disclosure() {
    let exe = env!("CARGO_BIN_EXE_godot-agent-kit-mcp");
    for flag in [
        "--internal-observation-worker",
        "--internal-discovery-worker",
        "--internal-edit-worker",
        "--internal-closed-edit-worker",
        "--internal-closed-acquisition-worker",
        "--internal-stock-validation-worker",
        "--internal-validator-readiness-worker",
    ] {
        let mut child = Command::new(exe)
            .arg(flag)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Pipe dispatch has no worker authority, even when it carries a request.
        let _ = child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{\"private_source\":\"PRIVATE_SOURCE_SENTINEL\"}\n");
        assert_eq!(
            wait_exit(&mut child, Duration::from_secs(2)).code(),
            Some(1)
        );
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn malformed_initialize_does_not_unlock_tools() {
    let mut peer = Peer::start();
    peer.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{}}}));
    assert_eq!(peer.read()["error"]["code"], -32602);
    peer.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    peer.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    assert_eq!(peer.read()["error"]["code"], -32600);
    peer.initialize();
    peer.send(json!({"jsonrpc":"2.0","id":3,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"regression","version":"1"}}}));
    assert_eq!(peer.read()["error"]["code"], -32600);
}

#[test]
fn discovery_probe_and_unknown_cancellation_do_not_dispatch() {
    let mut peer = Peer::start();
    peer.send(json!({"jsonrpc":"2.0","id":"probe","method":"server/discover"}));
    let response = peer.read();
    assert_eq!(
        response["result"]["supportedVersions"],
        json!(["2025-11-25"])
    );
    peer.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"unknown","reason":"sentinel_private_source"}}));
    peer.send(json!({"jsonrpc":"2.0","id":2,"method":"ping"}));
    assert_eq!(peer.read()["result"], json!({}));
}

#[test]
fn invalid_utf8_and_unterminated_frames_close_without_payload_echo() {
    for bytes in [vec![0xff, b'\n'], b"{\"sentinel_private_source\":".to_vec()] {
        let mut peer = Peer::start();
        peer.input.as_mut().unwrap().write_all(&bytes).unwrap();
        peer.input.take();
        wait_exit(&mut peer.child, Duration::from_secs(2));
        use std::io::Read;
        let mut stderr = String::new();
        peer.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        assert!(stderr.contains("mcp connection closed"));
        assert!(!stderr.contains("sentinel_private_source"));
    }
}

extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}
#[test]
fn signals_exit_with_stdin_still_open() {
    for signal in [2, 15] {
        let mut peer = Peer::start();
        peer.send(json!({"jsonrpc":"2.0","id":1,"method":"ping"}));
        peer.read(); // Handlers are installed; no startup race.
                     // SAFETY: this is the owned test child, never an editor.
        assert_eq!(unsafe { kill(peer.child.id() as i32, signal) }, 0);
        assert!(wait_exit(&mut peer.child, Duration::from_secs(2)).success());
        assert!(peer.input.is_some());
    }
}

#[test]
fn unconsumed_stdout_does_not_prevent_signal_teardown() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_godot-agent-kit-mcp"))
        .args(["--registry", "/nonexistent/mcp-regression-registry"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(input, "{}", json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"regression","version":"1"}}})).unwrap();
    let (handshake, received) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        handshake.send(stdout).unwrap();
    });
    let stdout = received
        .recv_timeout(Duration::from_secs(3))
        .expect("bounded handshake");
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","method":"notifications/initialized"})
    )
    .unwrap();
    for id in 2..66 {
        writeln!(
            input,
            "{}",
            json!({"jsonrpc":"2.0","id":id,"method":"tools/list"})
        )
        .unwrap();
    }
    // A read end stays open, but the complete catalog responses cannot all fit.
    assert_eq!(unsafe { kill(child.id() as i32, 15) }, 0);
    assert!(wait_exit(&mut child, Duration::from_secs(2)).success());
    drop(stdout);
}

#[test]
fn oversized_input_closes_instead_of_dispatching_a_truncated_frame() {
    let mut peer = Peer::start();
    peer.send(json!({"jsonrpc":"2.0","id":1,"method":"ping"}));
    peer.read();
    let mut input = peer.input.take().unwrap();
    let (completed, written) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let bytes = vec![b'x'; 16 * 1024 * 1024 + 1];
        let _ = input.write_all(&bytes);
        let _ = completed.send(());
    });
    wait_exit(&mut peer.child, Duration::from_secs(3));
    written.recv_timeout(Duration::from_secs(1)).unwrap();
    use std::io::Read;
    let mut stderr = String::new();
    peer.child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stderr.contains("mcp connection closed"));
    assert!(!stderr.contains("xxxxx"));
}

#[test]
fn depth_overflow_and_nested_duplicate_arguments_never_dispatch() {
    let mut peer = Peer::start();
    peer.initialize();
    let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    peer.raw(&format!("{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{{\"name\":\"read_script\",\"arguments\":{{\"extra\":{deep}}}}}}}\n"));
    assert_eq!(peer.read()["error"]["code"], -32700);
    peer.raw("{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"read_script\",\"arguments\":{\"project_root\":\"/first\",\"project_root\":\"/second\"}}}\n");
    assert_eq!(peer.read()["error"]["code"], -32700);
    peer.send(json!({"jsonrpc":"2.0","id":4,"method":"ping","params":{"_meta":{}}}));
    assert_eq!(peer.read()["id"], 4);
}

#[test]
fn unfinished_handshake_and_partial_frame_expire_with_stdin_open() {
    for partial in [false, true] {
        let mut peer = Peer::start();
        if partial {
            peer.initialize();
            peer.input
                .as_mut()
                .unwrap()
                .write_all(b"{\"private_partial_frame\":")
                .unwrap();
            peer.input.as_mut().unwrap().flush().unwrap();
        }
        assert!(wait_exit(&mut peer.child, Duration::from_secs(12)).success());
        assert!(peer.input.is_some(), "the peer did not supply EOF");
        use std::io::Read;
        let mut stderr = String::new();
        peer.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        assert!(!stderr.contains("private_partial_frame"));
    }
}

#[derive(Clone, Copy)]
enum AcquisitionInterruption {
    Cancel,
    Eof,
    Signal(i32),
}

#[test]
fn active_call_cancel_and_eof_close_only_the_owned_acquisition() {
    for interruption in [
        AcquisitionInterruption::Cancel,
        AcquisitionInterruption::Eof,
    ] {
        interrupted_acquisition(interruption);
    }
}

#[test]
fn active_call_sigint_and_sigterm_close_only_the_owned_acquisition() {
    for signal in [2, 15] {
        interrupted_acquisition(AcquisitionInterruption::Signal(signal));
    }
}

fn interrupted_acquisition(interruption: AcquisitionInterruption) {
    use std::fs::{self, DirBuilder, OpenOptions};
    use std::io::Read;
    use std::net::TcpListener;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
    use std::time::{SystemTime, UNIX_EPOCH};
    const SESSION: &str = "00112233445566778899aabbccddeeff";
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("mcp-owned-{}-{unique}", std::process::id()));
    DirBuilder::new().mode(0o700).create(&home).unwrap();
    let project = home.join("project");
    DirBuilder::new().mode(0o700).create(&project).unwrap();
    let source = project.join("target.gd");
    fs::write(&source, "extends Node\n").unwrap();
    let registry = home.join("registry");
    godot_agent_kit::project_fs::init_registry(&registry).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut descriptor = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(registry.join(format!("{SESSION}.json")))
        .unwrap();
    write!(
        descriptor,
        "{}",
        json!({
            "v":6,"session_id":SESSION,"project_root":project,
            "godot_version":"4.7.2.stable.official.ed1daf0bf",
            "engine_hash":"ed1daf0bf001b61586d9930840f2f1394092c079",
            "host":"127.0.0.1","port":listener.local_addr().unwrap().port(),
            "token":"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
        })
    )
    .unwrap();
    drop(descriptor);
    let (entered, started) = mpsc::sync_channel(1);
    let (closed, finished) = mpsc::sync_channel(1);
    let boundary = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "acquisition did not reach the selected endpoint"
                    );
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("{error}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut size = [0; 4];
        socket.read_exact(&mut size).unwrap();
        let size = u32::from_be_bytes(size) as usize;
        assert!(size <= 4096);
        let mut hello = vec![0; size];
        socket.read_exact(&mut hello).unwrap();
        entered.send(()).unwrap();
        // No authentication response: the actual bounded worker is waiting,
        // and has acquired no mutation authority.
        assert_eq!(
            socket.read(&mut [0]).unwrap(),
            0,
            "owned worker kept the endpoint alive after cancellation"
        );
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "a rejected edit was queued or replayed at the selected endpoint"
        );
        closed.send(()).unwrap();
    });
    let mut peer = Peer::start_at(&registry);
    peer.initialize();
    peer.send(
        json!({"jsonrpc":"2.0","id":"active","method":"tools/call","params":{
            "name":"edit_script","arguments":{"project_root":project,"session_id":SESSION,
                "script_path":"res://target.gd","revision":format!("sr1:{}", "a".repeat(64)),
                "replacement_source":"extends Node\n# must not apply\n"}
        }}),
    );
    started.recv_timeout(Duration::from_secs(3)).unwrap();
    // Neither a duplicate edit nor a different owner's rejected edit may
    // acquire a second private connection or replace this cancellation owner.
    for id in ["active", "rejected"] {
        peer.send(
            json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{
                "name":"edit_script","arguments":{"project_root":project,"session_id":SESSION,
                    "script_path":"res://target.gd","revision":format!("sr1:{}", "a".repeat(64)),
                    "replacement_source":"extends Node\n# rejected private source\n"}
            }}),
        );
        let response = peer.read();
        assert_eq!(response["id"], id);
        if id == "active" {
            assert_eq!(response["error"]["code"], -32600);
        } else {
            let object = &response["result"]["structuredContent"];
            assert_eq!(object["error"]["code"], "server_busy");
            assert_eq!(object["error"]["application"], "not_applied");
        }
        assert!(!response.to_string().contains("rejected private source"));
    }
    for id in ["unknown", "rejected"] {
        peer.send(
            json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":id}}),
        );
    }
    peer.send(json!({"jsonrpc":"2.0","id":"owner-isolated","method":"ping"}));
    assert_eq!(peer.read()["id"], "owner-isolated");
    assert!(
        finished.try_recv().is_err(),
        "another ID cancelled the original owner"
    );
    match interruption {
        AcquisitionInterruption::Eof => {
            peer.input.take();
            assert!(wait_exit(&mut peer.child, Duration::from_secs(2)).success());
        }
        AcquisitionInterruption::Signal(signal) => {
            // SAFETY: this is the owned MCP test process, not the endpoint.
            assert_eq!(unsafe { kill(peer.child.id() as i32, signal) }, 0);
            assert!(wait_exit(&mut peer.child, Duration::from_secs(2)).success());
            assert!(peer.input.is_some());
        }
        AcquisitionInterruption::Cancel => {
            peer.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"active"}}));
            peer.send(json!({"jsonrpc":"2.0","id":"responsive","method":"ping"}));
            assert_eq!(peer.read()["id"], "responsive");
        }
    }
    finished.recv_timeout(Duration::from_secs(2)).unwrap();
    boundary.join().unwrap();
    assert_eq!(fs::read_to_string(source).unwrap(), "extends Node\n");
    drop(peer);
    fs::remove_dir_all(home).unwrap();
}
