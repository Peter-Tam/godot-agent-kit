//! Real public caller / authenticated editor / confined walker integration.
use super::*;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;

fn authenticate_peer(listener: TcpListener) -> (TcpStream, Value) {
    let (mut socket, _) = listener.accept().unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(6)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let hello = read_frame(&mut socket);
    let caps = json!({"observe_gdscript":true,"open_enumeration":true,"buffer_attribution":false,"unsaved_paths":false,"cached_resource_lookup":false,"edit_open_gdscript":false,"open_gdscript":false,"discover_gdscripts":true});
    frame(
        &mut socket,
        &json!({"v":4,"kind":"challenge","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":0,"native_build_id":"","client_nonce":hello[5],"server_nonce":SERVER_NONCE,"server_proof":proof(b"server",&hello,&caps,SECRET)}),
    );
    let authentication = read_frame(&mut socket);
    assert_eq!(authentication[1], "authenticate");
    assert_eq!(authentication[7], proof(b"client", &hello, &caps, SECRET));
    frame(
        &mut socket,
        &json!({"v":4,"kind":"hello","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":0,"native_build_id":"","client_nonce":hello[5],"server_nonce":SERVER_NONCE,"finish_proof":proof(b"finish",&hello,&caps,SECRET)}),
    );
    (socket, hello)
}
fn context(hello: &Value, kind: &str, tick: u64) -> Value {
    json!({"v":4,"kind":kind,"request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"collection":{"clock_id":format!("editor:{}",hello[3].as_str().unwrap()),"started_tick_us":tick.to_string(),"finished_tick_us":(tick+1).to_string(),"received_elapsed_us":0},"status":"observed","reason":null,"context":{"policy":"godot_project_files_v1","project_data_directory":"res://.godot","filesystem_epoch":"0","scanning":false,"importing":false},"expiry_tick_us":"9000000000"})
}
fn spawn_caller(fixture: &Fixture) -> Child {
    Command::new(env!("CARGO_BIN_EXE_discover-gdscripts"))
        .args([
            "--registry",
            fixture.registry.to_str().unwrap(),
            "--project",
            fixture.project.to_str().unwrap(),
            "--session",
            ID1,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}
fn result(child: Child) -> (i32, Value) {
    let output = child.wait_with_output().unwrap();
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
fn admit(socket: &mut TcpStream, hello: &Value) {
    let begin = read_frame(socket);
    assert_eq!(begin.as_array().unwrap().len(), 6);
    assert_eq!(begin[1], "discover_begin");
    assert_eq!(begin[2], hello[2]);
    assert_eq!(begin[3], hello[3]);
    assert_eq!(begin[4], hello[4]);
    assert!((1..=4500).contains(&begin[5].as_u64().unwrap()));
    frame(socket, &context(hello, "discover_state", 100));
}

#[test]
fn public_discovery_actually_exhausts_metadata_without_source_acquisition() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("actors")).unwrap();
    fs::create_dir(fixture.project.join("ui")).unwrap();
    fs::write(
        fixture.project.join("actors/player.gd"),
        "NOT VALID GDSCRIPT\nPRIVATE_SOURCE_SENTINEL",
    )
    .unwrap();
    fs::write(fixture.project.join("ui/player.GD"), "").unwrap();
    fs::write(
        fixture.project.join("project.godot"),
        "root marker does not exclude root",
    )
    .unwrap();
    fs::write(
        fixture.project.join(".gdignore"),
        "root marker does not exclude root",
    )
    .unwrap();
    let listener = listener();
    fixture.descriptor(ID1, listener.local_addr().unwrap().port());
    let server = thread::spawn(move || {
        let (mut socket, hello) = authenticate_peer(listener);
        admit(&mut socket, &hello);
        assert_eq!(
            read_frame(&mut socket),
            json!([4, "discover_recheck", hello[2], hello[3], hello[4]])
        );
        frame(&mut socket, &context(&hello, "discover_rechecked", 200));
        let finish = read_frame(&mut socket);
        assert_eq!(
            finish,
            json!([4, "discover_finish", hello[2], hello[3], hello[4]])
        );
        let mut ack = context(&hello, "discover_finished", 300);
        for key in ["status", "reason", "context", "expiry_tick_us"] {
            ack.as_object_mut().unwrap().remove(key);
        }
        ack["terminal_discard"] = json!(false);
        frame(&mut socket, &ack);
    });
    let began = Instant::now();
    let (exit, value) = result(spawn_caller(&fixture));
    assert_eq!(exit, 0, "{value}");
    assert_eq!(value["outcome"], "complete_listing");
    assert_eq!(
        value["inventory"]["entries"],
        json!(["res://actors/player.gd", "res://ui/player.GD"])
    );
    assert_eq!(value["inventory"]["coverage"], "complete");
    assert_eq!(
        value["inventory"]["consistency"],
        json!({"recheck":"completed","stability":"unknown","atomic":false})
    );
    assert!(!value.to_string().contains("PRIVATE_SOURCE_SENTINEL"));
    assert!(began.elapsed() < Duration::from_secs(5));
    server.join().unwrap();
}

#[test]
fn stalled_owned_worker_timeout_and_cancellation_preserve_only_checked_earlier_batches() {
    for cancelled in [false, true] {
        let fixture = Fixture::new();
        fs::write(
            fixture.project.join("subject.gd"),
            "PRIVATE_SOURCE_SENTINEL",
        )
        .unwrap();
        let listener = listener();
        fixture.descriptor(ID1, listener.local_addr().unwrap().port());
        let (ready_send, ready_receive) = mpsc::sync_channel(1);
        let (release_send, release_receive) = mpsc::sync_channel(1);
        let server = thread::spawn(move || {
            let (mut socket, hello) = authenticate_peer(listener);
            admit(&mut socket, &hello);
            assert_eq!(read_frame(&mut socket)[1], "discover_recheck");
            ready_send.send(()).unwrap();
            release_receive
                .recv_timeout(Duration::from_secs(6))
                .unwrap();
            // The deliberately late control frame cannot replay or upgrade delivery.
            let bytes = serde_json::to_vec(&context(&hello, "discover_rechecked", 200)).unwrap();
            let _ = socket.write_all(&(bytes.len() as u32).to_be_bytes());
            let _ = socket.write_all(&bytes);
        });
        let began = Instant::now();
        let child = spawn_caller(&fixture);
        ready_receive.recv_timeout(Duration::from_secs(3)).unwrap();
        let children = Command::new("pgrep")
            .args(["-P", &child.id().to_string()])
            .output()
            .unwrap();
        assert!(children.status.success());
        let children = String::from_utf8(children.stdout).unwrap();
        let owned: Vec<u32> = children
            .split_whitespace()
            .map(|id| id.parse().unwrap())
            .collect();
        assert_eq!(owned.len(), 1);
        // SIGSTOP proves actual process supervision, not an in-memory timeout model.
        assert!(Command::new("kill")
            .args(["-STOP", &owned[0].to_string()])
            .status()
            .unwrap()
            .success());
        if cancelled {
            assert!(Command::new("kill")
                .args(["-INT", &child.id().to_string()])
                .status()
                .unwrap()
                .success());
        }
        let (exit, value) = result(child);
        assert_eq!(exit, 4, "{value}");
        assert_eq!(value["outcome"], "interrupted");
        assert_eq!(value["inventory"]["entries"], json!(["res://subject.gd"]));
        assert_eq!(value["inventory"]["validity"], "earlier_observation");
        assert_eq!(value["inventory"]["coverage"], "partial");
        assert_eq!(value["inventory"]["consistency"]["recheck"], "unavailable");
        assert!(value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == if cancelled { "cancelled" } else { "timeout" }));
        assert!(
            began.elapsed() < Duration::from_secs(5),
            "{:?}; result interval={:?}",
            began.elapsed(),
            value["interval"]
        );
        assert!(!value.to_string().contains("PRIVATE_SOURCE_SENTINEL"));
        release_send.send(()).unwrap();
        server.join().unwrap();
        // Owned child is killed/reaped asynchronously, without waiting in delivery.
        let reap_deadline = Instant::now() + Duration::from_secs(2);
        while Command::new("kill")
            .args(["-0", &owned[0].to_string()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
        {
            assert!(
                Instant::now() < reap_deadline,
                "owned worker was not reaped"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[test]
fn scope_loss_busy_and_malformed_control_never_become_complete_empty() {
    for case in [
        "scope_changed",
        "epoch_changed",
        "scanning",
        "busy",
        "foreign_session",
        "foreign_recheck",
        "foreign_ack",
        "unknown_field",
        "duplicate_field",
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.project.join("subject.gd"), "SOURCE_SENTINEL").unwrap();
        let listener = listener();
        fixture.descriptor(ID1, listener.local_addr().unwrap().port());
        let server = thread::spawn(move || {
            let (mut socket, hello) = authenticate_peer(listener);
            assert_eq!(read_frame(&mut socket)[1], "discover_begin");
            let mut begin = context(&hello, "discover_state", 100);
            match case {
                "scanning" => begin["context"]["scanning"] = json!(true),
                "busy" => {
                    begin["status"] = json!("refused");
                    begin["reason"] = json!("busy");
                    begin["context"] = Value::Null;
                    begin["expiry_tick_us"] = Value::Null;
                }
                "foreign_session" => {
                    begin["session_id"] = json!("ffffffffffffffffffffffffffffffff")
                }
                "unknown_field" => begin["extra"] = json!(true),
                _ => {}
            }
            if case == "duplicate_field" {
                let text = serde_json::to_string(&begin).unwrap();
                let text = format!("{},\"status\":\"observed\"}}", &text[..text.len() - 1]);
                socket
                    .write_all(&(text.len() as u32).to_be_bytes())
                    .unwrap();
                socket.write_all(text.as_bytes()).unwrap();
            } else {
                frame(&mut socket, &begin);
            }
            if matches!(
                case,
                "busy" | "foreign_session" | "unknown_field" | "duplicate_field"
            ) {
                return;
            }
            assert_eq!(read_frame(&mut socket)[1], "discover_recheck");
            let mut recheck = context(&hello, "discover_rechecked", 200);
            if case == "scope_changed" {
                recheck["context"]["project_data_directory"] = json!("res://godot");
            }
            if case == "epoch_changed" {
                recheck["context"]["filesystem_epoch"] = json!("1");
            }
            if case == "foreign_recheck" {
                recheck["session_id"] = json!("ffffffffffffffffffffffffffffffff");
            }
            frame(&mut socket, &recheck);
            if case == "foreign_recheck" {
                return;
            }
            assert_eq!(read_frame(&mut socket)[1], "discover_finish");
            let mut ack = json!({"v":4,"kind":"discover_finished","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"collection":{"clock_id":format!("editor:{}",hello[3].as_str().unwrap()),"started_tick_us":"300","finished_tick_us":"301","received_elapsed_us":0},"terminal_discard":false});
            if case == "foreign_ack" {
                ack["session_id"] = json!("ffffffffffffffffffffffffffffffff");
            }
            frame(&mut socket, &ack);
        });
        let began = Instant::now();
        let (exit, value) = result(spawn_caller(&fixture));
        assert!(began.elapsed() < Duration::from_secs(5));
        assert!(!value.to_string().contains("SOURCE_SENTINEL"));
        match case {
            "scope_changed" => {
                assert_eq!(exit, 2, "{value}");
                assert_eq!(value["inventory"], Value::Null);
                assert!(value["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["code"] == "scope_changed"));
            }
            "epoch_changed" => {
                assert_eq!(exit, 2, "{value}");
                assert_eq!(value["inventory"]["entries"], json!(["res://subject.gd"]));
                assert_eq!(value["inventory"]["consistency"]["stability"], "changed");
            }
            "scanning" => {
                assert_eq!(exit, 2, "{value}");
                assert_eq!(value["inventory"]["entries"], json!([]));
                assert_eq!(value["inventory"]["coverage"], "not_started");
            }
            "busy" => {
                assert_eq!(exit, 3, "{value}");
                assert_eq!(value["inventory"], Value::Null);
            }
            _ => {
                assert_eq!(exit, 4, "{value}");
                assert_eq!(value["inventory"], Value::Null);
            }
        }
        server.join().unwrap();
    }
}
