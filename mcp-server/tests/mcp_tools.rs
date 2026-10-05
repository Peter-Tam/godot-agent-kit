//! Actual MCP process boundaries; no editor fixture or synthetic success responses.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

struct Client {
    child: Child,
    input: ChildStdin,
    output: Receiver<Value>,
}
impl Client {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_godot-agent-kit-mcp"))
            .args(["--registry", "/tmp/gak-mcp-test-absent-registry"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (send, output) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Ok(value) = serde_json::from_str(&line) else {
                    break;
                };
                if send.send(value).is_err() {
                    break;
                }
            }
        });
        let mut client = Self {
            child,
            input,
            output,
        };
        client.send(json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{
            "protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"boundary-test","version":"1"}
        }}));
        assert_eq!(client.receive()["result"]["protocolVersion"], "2025-11-25");
        client.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        client
    }
    fn send(&mut self, value: Value) {
        serde_json::to_writer(&mut self.input, &value).unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
    }
    fn receive(&self) -> Value {
        self.output.recv_timeout(Duration::from_secs(12)).unwrap()
    }
    fn call(&mut self, name: &str, arguments: Value) -> Value {
        self.send(json!({"jsonrpc":"2.0","id":"consumer-call","method":"tools/call","params":{"name":name,"arguments":arguments}}));
        let response = self.receive();
        assert_eq!(response["id"], "consumer-call");
        assert!(response.get("error").is_none(), "{response}");
        response["result"].clone()
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn malformed_semantic_arguments_are_source_free_tool_errors_without_dispatch() {
    let mut client = Client::new();
    let valid = json!({"project_root":"/tmp/kit-absent-project","script_path":"res://target.gd"});
    let mut cases = vec![
        json!({}),
        json!({"project_root":"relative"}),
        json!({"project_root":"/tmp/project","session_id":null}),
        json!({"project_root":"/tmp/project","session_id":"not-a-session"}),
    ];
    for (key, value) in [
        ("basis", json!({"source":"PRIVATE_BASIS_SENTINEL"})),
        ("force", json!(true)),
        ("script_path", json!("res://../PRIVATE_PATH_SENTINEL.gd")),
        ("project_root", json!(format!("/{}", "é".repeat(512)))),
    ] {
        let mut args = valid.clone();
        args[key] = value;
        cases.push(args);
    }
    for args in cases {
        let result = client.call("read_script", args);
        let object = &result["structuredContent"];
        assert_eq!(result["isError"], true);
        assert_eq!(object["result"], Value::Null);
        assert_eq!(object["error"]["category"], "input");
        assert_eq!(object["error"]["application"], "not_applied");
        assert_eq!(object["error"]["requested_target"], Value::Null);
        assert_eq!(object["error"]["next_action"]["kind"], "correct_request");
        assert!(!result.to_string().contains("SENTINEL"));
    }
}

#[test]
fn replacement_and_revision_validation_precede_editor_selection() {
    let mut client = Client::new();
    let base = json!({"project_root":"/tmp/kit-absent-project","script_path":"res://target.gd","revision":format!("sr1:{}", "a".repeat(64)),"replacement_source":""});
    for replacement in [
        json!(null),
        json!("bad\r\n"),
        json!("bad\0"),
        json!("\u{feff}"),
        json!("x".repeat(512 * 1024 + 1)),
    ] {
        let mut arguments = base.clone();
        arguments["replacement_source"] = replacement;
        let result = client.call("edit_script", arguments);
        assert_eq!(result["structuredContent"]["error"]["category"], "input");
    }
    let mut invalid_revision = base.clone();
    invalid_revision["revision"] = json!(format!("sr1:{}", "A".repeat(64)));
    assert_eq!(
        client.call("edit_script", invalid_revision)["structuredContent"]["error"]["category"],
        "input"
    );
    let result = client.call("edit_script", base);
    assert_eq!(result["structuredContent"]["error"], Value::Null);
    assert_eq!(
        result["structuredContent"]["result"]["outcome"]["outcome"],
        "refused"
    );
    assert_eq!(
        result["structuredContent"]["result"]["outcome"]["application"],
        "not_applied"
    );
}

#[test]
fn real_unavailable_operations_remain_distinct_from_adapter_input_errors() {
    let mut client = Client::new();
    let discover = client.call(
        "discover_scripts",
        json!({"project_root":"/tmp/kit-absent-project"}),
    );
    let read = client.call(
        "read_script",
        json!({"project_root":"/tmp/kit-absent-project","script_path":"res://target.gd"}),
    );
    for (name, response) in [("discover_scripts", &discover), ("read_script", &read)] {
        let object = &response["structuredContent"];
        assert_eq!(object["operation"], name);
        assert_eq!(object["schema_version"], 1);
        assert_eq!(object["error"], Value::Null);
        assert_eq!(response["isError"], true);
        assert_ne!(object["request_id"], "consumer-call");
    }
    assert_eq!(read["structuredContent"]["result"]["source"], Value::Null);
    assert_eq!(read["structuredContent"]["result"]["revision"], Value::Null);
    assert_eq!(
        discover["structuredContent"]["result"]["inventory"],
        Value::Null
    );
    assert_ne!(
        discover["structuredContent"]["request_id"],
        read["structuredContent"]["request_id"]
    );
}
