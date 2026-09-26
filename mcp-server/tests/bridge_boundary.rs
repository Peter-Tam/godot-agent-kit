use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use godot_agent_kit::bridge::wire::{self, Event};
use godot_agent_kit::observation::{
    DiagnosticCode, ObservationRequest, OutcomeKind, ProjectRoot, RequestId, ResourcePath,
    SessionId,
};
use godot_agent_kit::{project_fs, target};
use ring::hmac;
use serde_json::{json, Value};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const VERSION: &str = "4.7.2.stable.official.ed1daf0bf";
const HASH: &str = "ed1daf0bf001b61586d9930840f2f1394092c079";
const SECRET: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const SERVER_NONCE: &str = "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f";

struct Fixture {
    home: PathBuf,
    project: PathBuf,
    registry: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = fs::canonicalize(std::env::temp_dir()).unwrap();
        let home = base.join(format!(
            "gak-bridge-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&home).unwrap();
        let project = home.join("project");
        let registry = home.join("registry");
        DirBuilder::new().mode(0o700).create(&project).unwrap();
        project_fs::init_registry(&registry).unwrap();
        Self {
            home,
            project,
            registry,
        }
    }
    fn descriptor(&self, id: &str, port: u16) {
        let name = self.registry.join(format!("{id}.json"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(name)
            .unwrap();
        write!(file, "{}", json!({"v":1,"session_id":id,"project_root":self.project,"godot_version":VERSION,"engine_hash":HASH,"host":"127.0.0.1","port":port,"token":SECRET})).unwrap();
    }
    fn request(&self, id: Option<&str>) -> ObservationRequest {
        ObservationRequest::new(
            RequestId::new("try_1").unwrap(),
            ProjectRoot::new(self.project.to_str().unwrap()).unwrap(),
            id.map(|s| SessionId::new(s).unwrap()),
            ResourcePath::new("res://scripts/subject.gd").unwrap(),
        )
    }
    fn resolve(
        &self,
        id: Option<&str>,
        duration: Duration,
    ) -> Result<target::SelectedSession, target::RoutingFailure> {
        target::resolve(&self.request(id), &self.registry, Instant::now() + duration)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.home).unwrap();
    }
}
fn listener() -> TcpListener {
    for attempt in 0..1000 {
        let port = 50000
            + ((std::process::id() as usize * 97 + NEXT.fetch_add(1, Ordering::Relaxed) + attempt)
                % 15000) as u16;
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
            return listener;
        }
    }
    panic!("no private test port available")
}
fn frame(stream: &mut TcpStream, message: &Value) {
    let bytes = serde_json::to_vec(message).unwrap();
    stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&bytes).unwrap();
}
fn read_frame(stream: &mut TcpStream) -> Value {
    let mut length = [0; 4];
    stream.read_exact(&mut length).unwrap();
    let mut bytes = vec![0; u32::from_be_bytes(length) as usize];
    assert!(bytes.len() <= 4096);
    stream.read_exact(&mut bytes).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
fn hex_decode(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|part| u8::from_str_radix(std::str::from_utf8(part).unwrap(), 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn field(buffer: &mut Vec<u8>, field: &[u8]) {
    buffer.extend_from_slice(&(field.len() as u32).to_be_bytes());
    buffer.extend_from_slice(field);
}
fn proof(role: &[u8], hello: &Value, caps: &Value, key: &str) -> String {
    let mut bytes = Vec::new();
    field(&mut bytes, role);
    field(&mut bytes, b"godot-agent-kit/observation-bridge/v1");
    field(&mut bytes, hello[2].as_str().unwrap().as_bytes());
    field(&mut bytes, &hex_decode(hello[3].as_str().unwrap()));
    field(&mut bytes, hello[4].as_str().unwrap().as_bytes());
    field(&mut bytes, VERSION.as_bytes());
    field(&mut bytes, HASH.as_bytes());
    for name in [
        "observe_gdscript",
        "open_enumeration",
        "buffer_attribution",
        "unsaved_paths",
        "cached_resource_lookup",
    ] {
        field(&mut bytes, &[u8::from(caps[name].as_bool().unwrap())]);
    }
    field(&mut bytes, &hex_decode(hello[5].as_str().unwrap()));
    field(&mut bytes, &hex_decode(SERVER_NONCE));
    let key = hmac::Key::new(hmac::HMAC_SHA256, &hex_decode(key));
    hex(hmac::sign(&key, &bytes).as_ref())
}
fn editor_stamp(hello: &Value) -> Value {
    json!({"clock_id":format!("editor:{}",hello[3].as_str().unwrap()),"started_tick_us":"9007199254740993","finished_tick_us":"9007199254740994","received_elapsed_us":0})
}

fn observation_sample(hello: &Value) -> Value {
    let stamp = editor_stamp(hello);
    let path = "res://scripts/subject.gd";
    let resource_witness = json!({"resource_path":path,"script_instance_id":"9007199254740993","editor_instance_id":null,"buffer_instance_id":null,"disk_file_id":null,"source_version":"9007199254740994"});
    let buffer_witness = json!({"resource_path":path,"script_instance_id":"9007199254740993","editor_instance_id":"9007199254740994","buffer_instance_id":"9007199254740995","disk_file_id":null,"source_version":"9007199254740996"});
    json!({
        "v":1,"kind":"sample","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":path,
        "collection":stamp,
        "document":{"identity":{"kind":"external_gdscript","resource_path":path,"script_instance_id":"9007199254740993","editor_instance_id":"9007199254740994","buffer_instance_id":"9007199254740995","disk_file_id":null},
            "validity":{"value":"valid","collection":stamp,"reason":null,"invalidated_evidence":null},
            "open_state":{"value":"open","collection":stamp,"reason":null,"invalidated_evidence":null}},
        "R":{"authority":"R","availability":"observed","text":"\u{feff}x\r\n","collection":stamp,"witness":resource_witness,"staleness":{"state":"unknown"},"reason":null,"invalidated_evidence":null},
        "B":{"authority":"B","availability":"observed","text":"x\n","collection":stamp,"witness":buffer_witness,"staleness":{"state":"unknown"},"reason":null,"invalidated_evidence":null},
        "dirty":{"availability":"observed","state":"dirty","collection":stamp,"witness":buffer_witness,"reason":null,"invalidated_evidence":null},
        "diagnostics":[]
    })
}

#[derive(Clone, Copy)]
enum Mode {
    Observation(ObservationMode),
    Valid,
    WrongSecret,
    Reflection,
    Replay,
    ChangedTranscript,
    MalformedProof,
    DuplicateField,
    NullProof,
    ChangedFinish,
    NoFinish,
    SilentFinish,
    Oversized,
    Silence,
}
#[derive(Clone, Copy)]
enum ObservationMode {
    Valid,
    Stable,
    WrongIdentity,
    DuplicateRequired,
    Nested,
    WrongWitness,
    WrongClock,
    SecretUnknownField,
    ExcessDiagnostics,
    ExcessSource,
    ExcessSourceWrongWitness,
    OverlargeFrame,
    Disconnect,
    Silent,
}
fn serve(listener: TcpListener, mode: Mode) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "owned peer was not contacted before deadline"
                    );
                    thread::park_timeout(Duration::from_millis(1));
                }
                Err(error) => panic!("owned peer accept failed: {error}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let hello = read_frame(&mut socket);
        assert_eq!(hello.as_array().unwrap().len(), 6);
        assert_eq!(hello[1], "hello");
        assert_eq!(hello[5].as_str().unwrap().len(), 64);
        assert!(!hello.to_string().contains(SECRET));
        if matches!(mode, Mode::Silence) {
            thread::sleep(Duration::from_millis(350));
            return;
        }
        if matches!(mode, Mode::Oversized) {
            socket.write_all(&4097u32.to_be_bytes()).unwrap();
            return;
        }
        let caps = json!({"observe_gdscript":matches!(mode, Mode::Observation(_)),"open_enumeration":true,"buffer_attribution":false,"unsaved_paths":false,"cached_resource_lookup":false});
        let key = if matches!(mode, Mode::WrongSecret) {
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        } else {
            SECRET
        };
        let server_proof = match mode {
            Mode::Replay => {
                "31cd95cdc00afe815371791207ddedda98cb79e1d6bf181bd4fb8ac3cc8d2ed1".to_owned()
            }
            Mode::MalformedProof => "XYZ".to_owned(),
            _ => proof(
                if matches!(mode, Mode::Reflection) {
                    b"client"
                } else {
                    b"server"
                },
                &hello,
                &caps,
                key,
            ),
        };
        let mut challenge = json!({"v":1,"kind":"challenge","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"server_proof":server_proof});
        if matches!(mode, Mode::ChangedTranscript) {
            challenge["request_id"] = json!("other-request");
        }
        if matches!(mode, Mode::NullProof) {
            challenge["server_proof"] = Value::Null;
        }
        if matches!(mode, Mode::DuplicateField) {
            let mut encoded = challenge.to_string();
            encoded.pop();
            encoded.push_str(",\"server_nonce\":\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"}");
            socket
                .write_all(&(encoded.len() as u32).to_be_bytes())
                .unwrap();
            socket.write_all(encoded.as_bytes()).unwrap();
        } else {
            frame(&mut socket, &challenge);
        }
        if matches!(
            mode,
            Mode::WrongSecret
                | Mode::Reflection
                | Mode::Replay
                | Mode::ChangedTranscript
                | Mode::MalformedProof
                | Mode::DuplicateField
                | Mode::NullProof
        ) {
            return;
        }
        let authentication = read_frame(&mut socket);
        assert_eq!(authentication[1], "authenticate");
        assert_eq!(authentication[7], proof(b"client", &hello, &caps, SECRET));
        assert!(!authentication.to_string().contains(SECRET));
        if matches!(mode, Mode::NoFinish) {
            return;
        }
        if matches!(mode, Mode::SilentFinish) {
            thread::sleep(Duration::from_millis(350));
            return;
        }
        let finish = json!({"v":1,"kind":"hello","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"finish_proof":proof(b"finish", &hello, &caps, SECRET)});
        let mut finish = finish;
        if matches!(mode, Mode::ChangedFinish) {
            finish["capabilities"]["unsaved_paths"] = json!(true);
        }
        frame(&mut socket, &finish);
        if let Mode::Observation(reply) = mode {
            let observe = read_frame(&mut socket);
            assert_eq!(
                observe,
                json!([
                    1,
                    "observe",
                    hello[2],
                    hello[3],
                    hello[4],
                    "res://scripts/subject.gd"
                ])
            );
            match reply {
                ObservationMode::Disconnect => return,
                ObservationMode::Silent => {
                    thread::sleep(Duration::from_millis(350));
                    return;
                }
                ObservationMode::OverlargeFrame => {
                    socket
                        .write_all(&((wire::RESULT_LIMIT + 1) as u32).to_be_bytes())
                        .unwrap();
                    return;
                }
                _ => {}
            }
            let mut sample = observation_sample(&hello);
            if matches!(reply, ObservationMode::WrongIdentity) {
                sample["session_id"] = json!(ID2);
            }
            if matches!(reply, ObservationMode::WrongWitness) {
                sample["R"]["witness"]["script_instance_id"] = json!("123");
            }
            if matches!(reply, ObservationMode::WrongClock) {
                sample["B"]["collection"]["clock_id"] = json!("caller");
            }
            if matches!(reply, ObservationMode::SecretUnknownField) {
                sample["leaked_secret"] = json!(SECRET);
            }
            if matches!(reply, ObservationMode::ExcessDiagnostics) {
                sample["diagnostics"] = json!(vec![
                    json!({"code":"too_large","stage":"read_editor","surface":"R","message":"Source exceeded its independent observation limit","action":"Inspect this source through a separately authorized bounded workflow"});
                    65
                ]);
            }
            if matches!(
                reply,
                ObservationMode::ExcessSource | ObservationMode::ExcessSourceWrongWitness
            ) {
                sample["R"]["text"] =
                    json!("Q".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES + 1));
            }
            if matches!(reply, ObservationMode::ExcessSourceWrongWitness) {
                sample["R"]["witness"]["script_instance_id"] = json!("123");
            }
            if matches!(reply, ObservationMode::Nested) {
                let mut nested = json!(0);
                for _ in 0..33 {
                    nested = json!([nested]);
                }
                sample["extension"] = nested;
            }
            if matches!(reply, ObservationMode::DuplicateRequired) {
                let mut encoded = sample.to_string();
                encoded.pop();
                encoded.push_str(",\"session_id\":\"00112233445566778899aabbccddeeff\"}");
                socket
                    .write_all(&(encoded.len() as u32).to_be_bytes())
                    .unwrap();
                socket.write_all(encoded.as_bytes()).unwrap();
                return;
            }
            if matches!(reply, ObservationMode::Stable) {
                // Deliberately split both the length header and UTF-8/JSON body.
                // No delay or assumption about TCP packet coalescing is required.
                socket.set_nodelay(true).unwrap();
                let bytes = serde_json::to_vec(&sample).unwrap();
                let header = (bytes.len() as u32).to_be_bytes();
                socket.write_all(&header[..1]).unwrap();
                socket.write_all(&header[1..]).unwrap();
                let (chunks, remainder) = bytes.as_chunks::<7>();
                for chunk in chunks {
                    socket.write_all(chunk).unwrap();
                }
                socket.write_all(remainder).unwrap();
            } else {
                frame(&mut socket, &sample);
            }
            if matches!(reply, ObservationMode::Valid | ObservationMode::Stable) {
                let recheck = read_frame(&mut socket);
                assert_eq!(
                    recheck,
                    json!([
                        1,
                        "recheck",
                        hello[2],
                        hello[3],
                        hello[4],
                        "res://scripts/subject.gd"
                    ])
                );
                let changes = if matches!(reply, ObservationMode::Stable) {
                    json!([])
                } else {
                    json!([{"surface":"B","code":"source_changed"}])
                };
                frame(
                    &mut socket,
                    &json!({"v":1,"kind":"recheck","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":"res://scripts/subject.gd","collection":editor_stamp(&hello),"checks":"performed","detected_changes":changes,"reason":null}),
                );
            }
            return;
        }
        // A selected session must not receive a source-bearing frame during T002 routing.
        socket
            .set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        let mut extra = [0; 1];
        assert!(matches!(socket.read(&mut extra), Ok(0) | Err(_)));
    })
}
fn attach(fixture: &Fixture, id: &str, mode: Mode) -> thread::JoinHandle<()> {
    let socket = listener();
    fixture.descriptor(id, socket.local_addr().unwrap().port());
    serve(socket, mode)
}
const ID1: &str = "00112233445566778899aabbccddeeff";
const ID2: &str = "11112233445566778899aabbccddeeff";
const ID3: &str = "22222233445566778899aabbccddeeff";

#[test]
fn two_authenticated_sessions_outweigh_later_protocol_and_timeout() {
    for mode in [Mode::DuplicateField, Mode::Silence] {
        let fixture = Fixture::new();
        let first = attach(&fixture, ID1, Mode::Valid);
        let second = attach(&fixture, ID2, Mode::Valid);
        let third = attach(&fixture, ID3, mode);
        let deadline = if matches!(mode, Mode::Silence) {
            Duration::from_millis(120)
        } else {
            Duration::from_secs(2)
        };
        let error = fixture.resolve(None, deadline).err().unwrap();
        assert_eq!(error.outcome, OutcomeKind::AmbiguousTarget);
        let selection = error.selection.unwrap();
        assert_eq!(
            selection
                .candidate_sessions()
                .iter()
                .map(SessionId::as_str)
                .collect::<Vec<_>>(),
            [ID1, ID2]
        );
        first.join().unwrap();
        second.join().unwrap();
        third.join().unwrap();
    }
}

#[test]
fn two_authenticated_sessions_outweigh_later_unsupported_version() {
    let fixture = Fixture::new();
    let first = attach(&fixture, ID1, Mode::Valid);
    let second = attach(&fixture, ID2, Mode::Valid);
    let third = listener();
    fixture.descriptor(ID3, third.local_addr().unwrap().port());
    let path = fixture.registry.join(format!("{ID3}.json"));
    let mut descriptor: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    descriptor["godot_version"] = json!("4.8.0.unsupported");
    fs::write(path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
    let error = fixture.resolve(None, Duration::from_secs(2)).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::AmbiguousTarget);
    assert_eq!(
        error
            .selection
            .unwrap()
            .candidate_sessions()
            .iter()
            .map(SessionId::as_str)
            .collect::<Vec<_>>(),
        [ID1, ID2]
    );
    third.set_nonblocking(true).unwrap();
    assert_eq!(
        third.accept().err().unwrap().kind(),
        std::io::ErrorKind::WouldBlock
    );
    first.join().unwrap();
    second.join().unwrap();
}

#[test]
fn later_authentication_denial_outweighs_two_authenticated_sessions() {
    let fixture = Fixture::new();
    let first = attach(&fixture, ID1, Mode::Valid);
    let second = attach(&fixture, ID2, Mode::Valid);
    let third = attach(&fixture, ID3, Mode::WrongSecret);
    let error = fixture.resolve(None, Duration::from_secs(2)).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert!(error.selection.is_none());
    first.join().unwrap();
    second.join().unwrap();
    third.join().unwrap();
}

#[test]
fn uncertain_advertised_project_identity_cannot_select_another_session() {
    let fixture = Fixture::new();
    let first = attach(&fixture, ID1, Mode::Valid);
    let second = listener();
    fixture.descriptor(ID2, second.local_addr().unwrap().port());
    let path = fixture.registry.join(format!("{ID2}.json"));
    let mut descriptor: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    descriptor["project_root"] = json!(fixture.home.join("inaccessible-project"));
    fs::write(path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
    let error = fixture.resolve(None, Duration::from_secs(2)).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert!(error.selection.is_none());
    second.set_nonblocking(true).unwrap();
    assert_eq!(
        second.accept().err().unwrap().kind(),
        std::io::ErrorKind::WouldBlock
    );
    first.join().unwrap();
}

#[test]
fn live_mutual_proof_and_exact_selection() {
    let fixture = Fixture::new();
    let first = attach(&fixture, ID1, Mode::Valid);
    let second = attach(&fixture, ID2, Mode::Valid);
    let error = fixture.resolve(None, Duration::from_secs(2)).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::AmbiguousTarget);
    assert_eq!(error.selection.unwrap().candidate_sessions().len(), 2);
    first.join().unwrap();
    second.join().unwrap();
    // A previously advertised but ended exact session cannot be substituted by another.
    assert_eq!(
        fixture
            .resolve(Some(ID1), Duration::from_millis(200))
            .err()
            .unwrap()
            .outcome,
        OutcomeKind::EditorUnavailable
    );
}

#[test]
fn ended_exact_id_does_not_connect_to_a_new_live_session() {
    let fixture = Fixture::new();
    fixture.descriptor(ID1, listener().local_addr().unwrap().port());
    let replacement = listener();
    fixture.descriptor(ID2, replacement.local_addr().unwrap().port());
    let failure = fixture
        .resolve(Some(ID1), Duration::from_millis(200))
        .err()
        .unwrap();
    assert_eq!(failure.outcome, OutcomeKind::EditorUnavailable);
    replacement.set_nonblocking(true).unwrap();
    assert_eq!(
        replacement.accept().err().unwrap().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn selected_connection_requires_finish_and_never_sends_source() {
    let fixture = Fixture::new();
    let peer = attach(&fixture, ID1, Mode::Valid);
    let selected = fixture
        .resolve(Some(ID1), Duration::from_secs(2))
        .ok()
        .unwrap();
    assert_eq!(selected.target().session_id().as_str(), ID1);
    assert!(!selected.capabilities().observe_gdscript);
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn aliased_project_root_is_bound_to_canonical_filesystem_identity() {
    let fixture = Fixture::new();
    let alias = fixture.home.join("project-alias");
    std::os::unix::fs::symlink(&fixture.project, &alias).unwrap();
    let peer = attach(&fixture, ID1, Mode::Valid);
    let request = ObservationRequest::new(
        RequestId::new("alias_1").unwrap(),
        ProjectRoot::new(alias.to_str().unwrap()).unwrap(),
        None,
        ResourcePath::new("res://scripts/subject.gd").unwrap(),
    );
    let selected = target::resolve(
        &request,
        &fixture.registry,
        Instant::now() + Duration::from_secs(2),
    )
    .ok()
    .unwrap();
    assert_eq!(
        selected.target().project_root().as_str(),
        fixture.project.to_str().unwrap()
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn forged_reflected_or_changed_proofs_are_not_liveness() {
    for (mode, outcome) in [
        (Mode::WrongSecret, OutcomeKind::DeniedAccess),
        (Mode::Reflection, OutcomeKind::DeniedAccess),
        (Mode::Replay, OutcomeKind::DeniedAccess),
        (Mode::ChangedTranscript, OutcomeKind::DeniedAccess),
        (Mode::MalformedProof, OutcomeKind::DeniedAccess),
        (Mode::DuplicateField, OutcomeKind::ProtocolError),
        (Mode::NullProof, OutcomeKind::ProtocolError),
        (Mode::ChangedFinish, OutcomeKind::ProtocolError),
        (Mode::NoFinish, OutcomeKind::DeniedAccess),
        (Mode::SilentFinish, OutcomeKind::Timeout),
        (Mode::Oversized, OutcomeKind::ProtocolError),
        (Mode::Silence, OutcomeKind::Timeout),
    ] {
        let fixture = Fixture::new();
        let peer = attach(&fixture, ID1, mode);
        let error = fixture
            .resolve(None, Duration::from_millis(100))
            .err()
            .unwrap();
        assert_eq!(error.outcome, outcome);
        if matches!(
            mode,
            Mode::WrongSecret
                | Mode::Reflection
                | Mode::Replay
                | Mode::MalformedProof
                | Mode::NoFinish
        ) {
            assert_eq!(
                error.diagnostic.code(),
                DiagnosticCode::AuthenticationFailed
            );
        }
        assert!(!format!("{error:?}").contains(SECRET));
        peer.join().unwrap();
    }
}

#[test]
fn live_plus_unresolved_is_timeout_not_unique() {
    let fixture = Fixture::new();
    let first = attach(&fixture, ID1, Mode::Valid);
    let second = attach(&fixture, ID2, Mode::Silence);
    let error = fixture
        .resolve(None, Duration::from_millis(120))
        .err()
        .unwrap();
    assert_eq!(error.outcome, OutcomeKind::Timeout);
    first.join().unwrap();
    second.join().unwrap();
}

#[test]
fn controlled_peer_framing_preserves_exact_editor_text_and_independent_recheck() {
    use godot_agent_kit::observation::{
        Authority, Comparison, DetectedChange, ObservationEvidence, ObservationInterval,
        ObservationOutcome, Recheck, SourceObservation, SourceReason, Sources,
    };
    let fixture = Fixture::new();
    let peer = attach(&fixture, ID1, Mode::Observation(ObservationMode::Valid));
    let request = fixture.request(None);
    let mut selected = fixture.resolve(None, Duration::from_secs(2)).unwrap();
    let started = Instant::now();
    let sample = wire::observe(
        &mut selected,
        &request,
        started,
        started + Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(sample.resource.text(), Some("\u{feff}x\r\n"));
    assert_eq!(sample.buffer.text(), Some("x\n"));
    assert_eq!(
        sample
            .document
            .identity()
            .unwrap()
            .script_instance_id()
            .unwrap()
            .as_str(),
        "9007199254740993"
    );
    assert!(
        matches!(wire::recheck(&mut selected, &request, started, started + Duration::from_secs(2)).unwrap(), Recheck::Performed { detected_changes } if detected_changes == vec![DetectedChange::Source(Authority::B)])
    );
    let target = selected.target().clone();
    drop(selected);
    peer.join().unwrap();
    let worker_sample = wire::EditorSample {
        document: sample.document.clone(),
        resource: sample.resource.clone(),
        buffer: sample.buffer.clone(),
        dirty: sample.dirty.clone(),
        diagnostics: sample.diagnostics.clone(),
    };
    let encoded_sample = wire::encode_event(
        &Event::Sample(Box::new(worker_sample)),
        request.request_id(),
    )
    .unwrap();
    let receipt = started.elapsed().as_micros() as u64 + 1_000_000;
    assert!(
        matches!(wire::decode_event(&encoded_sample, &request, Some(&target), receipt).unwrap(),
        Event::Sample(s) if s.resource.text() == Some("\u{feff}x\r\n") && s.resource.collection().unwrap().received_elapsed_us() == receipt)
    );
    let identity_change = ObservationEvidence::new(
        target.clone(),
        sample.document.clone(),
        Sources::new(
            SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing).unwrap(),
            sample.resource.clone(),
            sample.buffer.clone(),
        )
        .unwrap(),
        sample.dirty.clone(),
        Recheck::performed(vec![DetectedChange::DocumentIdentityReplaced]),
    );
    let invalidated = ObservationOutcome::classify(
        request.clone(),
        ObservationInterval::new(1, 2, started.elapsed().as_micros() as u64 + 1000),
        Some(target.clone()),
        Some(identity_change),
        vec![],
        vec![],
    )
    .unwrap();
    let invalidated_json: Value =
        serde_json::from_slice(&wire::encode_outcome(&invalidated).unwrap()).unwrap();
    assert!(invalidated_json["snapshot"]["document"]["validity"]["value"].is_null());
    assert_eq!(
        invalidated_json["snapshot"]["document"]["validity"]["invalidated_evidence"]["value"],
        "valid"
    );
    assert_eq!(
        invalidated_json["snapshot"]["document"]["open_state"]["invalidated_evidence"]
            ["collection"]["clock_id"],
        format!("editor:{ID1}")
    );
    assert!(invalidated_json["snapshot"]["sources"]["R"]["text"].is_null());
    assert_eq!(
        invalidated_json["snapshot"]["sources"]["R"]["invalidated_evidence"]["text"],
        "\u{feff}x\r\n"
    );
    assert_eq!(
        invalidated_json["snapshot"]["sources"]["R"]["invalidated_evidence"]["witness"]
            ["source_version"],
        "9007199254740994"
    );
    assert_eq!(invalidated_json["snapshot"]["dirty"]["state"], "unknown");
    assert_eq!(
        invalidated_json["snapshot"]["dirty"]["invalidated_evidence"]["state"],
        "dirty"
    );
    assert_eq!(
        invalidated_json["snapshot"]["dirty"]["invalidated_evidence"]["reason"]["code"],
        "identity_changed"
    );

    // This controlled peer proves only the protocol boundary, not live-editor collection.
    let evidence = ObservationEvidence::new(
        target.clone(),
        sample.document,
        Sources::new(
            SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing).unwrap(),
            sample.resource,
            sample.buffer,
        )
        .unwrap(),
        sample.dirty,
        Recheck::performed(vec![DetectedChange::Source(Authority::B)]),
    );
    let outcome = ObservationOutcome::classify(
        request,
        ObservationInterval::new(1, 2, started.elapsed().as_micros() as u64 + 1000),
        Some(target),
        Some(evidence),
        vec![],
        sample.diagnostics,
    )
    .unwrap();
    assert_eq!(outcome.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(
        outcome.snapshot().unwrap().comparisons().resource_buffer(),
        Comparison::Unknown
    );
    let serialized = wire::encode_outcome(&outcome).unwrap();
    let value: Value = serde_json::from_slice(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["resolved_target"]["session_id"], ID1);
    assert_eq!(
        value["snapshot"]["document"]["identity"]["script_instance_id"],
        "9007199254740993"
    );
    assert_eq!(value["snapshot"]["sources"]["R"]["text"], "\u{feff}x\r\n");
    assert!(value["snapshot"]["sources"]["B"]["text"].is_null());
    assert_eq!(
        value["snapshot"]["sources"]["B"]["invalidated_evidence"]["text"],
        "x\n"
    );
    assert_eq!(
        value["snapshot"]["sources"]["B"]["reason"]["code"],
        "source_changed"
    );
    assert_eq!(value["snapshot"]["consistency"]["atomic"], false);
    assert_eq!(
        value["snapshot"]["comparisons"]["resource_buffer"],
        "unknown"
    );
    assert_eq!(value["snapshot"]["dirty"]["state"], "dirty");
    assert!(value["selection"].is_null());
    assert!(!String::from_utf8(serialized).unwrap().contains(SECRET));
}

#[test]
fn controlled_peer_rejects_wrong_identity_duplicates_depth_and_oversized_frames() {
    for (mode, expected) in [
        (ObservationMode::WrongIdentity, OutcomeKind::ProtocolError),
        (
            ObservationMode::DuplicateRequired,
            OutcomeKind::ProtocolError,
        ),
        (ObservationMode::WrongWitness, OutcomeKind::ProtocolError),
        (ObservationMode::WrongClock, OutcomeKind::ProtocolError),
        (
            ObservationMode::SecretUnknownField,
            OutcomeKind::ProtocolError,
        ),
        (
            ObservationMode::ExcessDiagnostics,
            OutcomeKind::ProtocolError,
        ),
        (
            ObservationMode::ExcessSourceWrongWitness,
            OutcomeKind::ProtocolError,
        ),
        (ObservationMode::Nested, OutcomeKind::ProtocolError),
        (ObservationMode::OverlargeFrame, OutcomeKind::ProtocolError),
        (ObservationMode::Disconnect, OutcomeKind::DisconnectedEditor),
        (ObservationMode::Silent, OutcomeKind::Timeout),
    ] {
        let fixture = Fixture::new();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let request = fixture.request(Some(ID1));
        let mut selected = fixture.resolve(Some(ID1), Duration::from_secs(2)).unwrap();
        let started = Instant::now();
        let error = wire::observe(
            &mut selected,
            &request,
            started,
            started + Duration::from_millis(140),
        )
        .err()
        .unwrap();
        assert_eq!(error.outcome, expected);
        assert_eq!(
            error.diagnostic.code(),
            if expected == OutcomeKind::ProtocolError {
                DiagnosticCode::InvalidFrame
            } else if expected == OutcomeKind::Timeout {
                DiagnosticCode::DeadlineExceeded
            } else {
                DiagnosticCode::SessionEnded
            }
        );
        assert!(!format!("{error:?}").contains(SECRET));
        drop(selected);
        peer.join().unwrap();
    }
}

#[test]
fn excess_resource_text_limits_only_r_and_retains_independent_b() {
    use godot_agent_kit::observation::{Availability, SourceReason};
    let fixture = Fixture::new();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::ExcessSource),
    );
    let request = fixture.request(Some(ID1));
    let mut selected = fixture.resolve(Some(ID1), Duration::from_secs(2)).unwrap();
    let started = Instant::now();
    let sample = wire::observe(
        &mut selected,
        &request,
        started,
        started + Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(sample.resource.availability(), Availability::Unavailable);
    assert_eq!(sample.resource.reason(), Some(SourceReason::TooLarge));
    assert!(sample.resource.text().is_none());
    assert_eq!(sample.buffer.text(), Some("x\n"));
    assert!(sample.document.identity().is_some());
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn ipc_events_validate_target_identity_and_reject_claimed_success() {
    use godot_agent_kit::observation::{
        DecimalCounter, DetectedChange, Diagnostic, EngineVersion, FileIdentity, Recheck,
        RecheckReason, Stage, Surface,
    };
    let fixture = Fixture::new();
    let request = fixture.request(Some(ID1));
    let target = godot_agent_kit::observation::ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(
            DecimalCounter::new("9007199254740993").unwrap(),
            DecimalCounter::new("9007199254740994").unwrap(),
        ),
        SessionId::new(ID1).unwrap(),
        EngineVersion::new(VERSION, HASH).unwrap(),
    )
    .unwrap();
    let event = wire::encode_event(&Event::Selected(target.clone()), request.request_id()).unwrap();
    assert!(
        matches!(wire::decode_event(&event, &request, None, 1).unwrap(), Event::Selected(t) if t.project_file_id().inode().as_str() == "9007199254740994")
    );
    let mut wrong: Value = serde_json::from_slice(&event).unwrap();
    wrong["target"]["session_id"] = json!(ID2);
    assert_eq!(
        wire::decode_event(wrong.to_string().as_bytes(), &request, None, 1)
            .err()
            .unwrap()
            .outcome,
        OutcomeKind::ProtocolError
    );
    let mut wrong = serde_json::from_slice::<Value>(&event).unwrap();
    wrong["target"]["script_path"] = json!("res://scripts/other.gd");
    assert!(wire::decode_event(wrong.to_string().as_bytes(), &request, None, 1).is_err());
    let failed = json!({"v":1,"request_id":request.request_id().as_str(),"kind":"failed",
        "failure":{"outcome":"complete_observation","diagnostic":{"code":"invalid_frame","stage":"read_editor","surface":"session","message":"Bridge response was invalid","action":"Inspect the affected stage and authority before a new observation"},"selection":null}});
    assert!(wire::decode_event(failed.to_string().as_bytes(), &request, Some(&target), 1).is_err());
    let diagnostic = Diagnostic::new(
        DiagnosticCode::DeadlineExceeded,
        Stage::Recheck,
        Some(Surface::Session),
    );
    let failure = target::RoutingFailure {
        outcome: OutcomeKind::Timeout,
        diagnostic,
        selection: None,
    };
    let event = wire::encode_event(&Event::Failed(failure), request.request_id()).unwrap();
    assert!(
        matches!(wire::decode_event(&event, &request, Some(&target), 1).unwrap(), Event::Failed(f) if f.outcome == OutcomeKind::Timeout)
    );
    let partial = wire::encode_event(
        &Event::Rechecked(Recheck::partial(
            RecheckReason::Unavailable,
            vec![DetectedChange::Source(
                godot_agent_kit::observation::Authority::B,
            )],
        )),
        request.request_id(),
    )
    .unwrap();
    assert!(
        matches!(wire::decode_event(&partial, &request, Some(&target), 1).unwrap(),
        Event::Rechecked(Recheck::Partial { reason: RecheckReason::Unavailable, detected_changes }) if detected_changes == vec![DetectedChange::Source(godot_agent_kit::observation::Authority::B)])
    );
    let valid = wire::encode_event(
        &Event::DiskChecked(vec![DetectedChange::DiskIdentityReplaced]),
        request.request_id(),
    )
    .unwrap();
    assert!(
        matches!(wire::decode_event(&valid, &request, Some(&target), 1).unwrap(), Event::DiskChecked(changes) if changes == vec![DetectedChange::DiskIdentityReplaced])
    );
    let mut wrong = serde_json::from_slice::<Value>(&valid).unwrap();
    wrong["disk_checked"][0]["surface"] = json!("B");
    assert!(wire::decode_event(wrong.to_string().as_bytes(), &request, Some(&target), 1).is_err());
}

#[test]
fn denied_result_is_source_free_with_required_nullables_and_safe_diagnostics() {
    use godot_agent_kit::observation::{
        Diagnostic, ObservationInterval, ObservationOutcome, Stage, Surface, TerminalFailure,
    };
    let fixture = Fixture::new();
    let request = fixture.request(Some(ID1));
    let diagnostic = Diagnostic::new(
        DiagnosticCode::DeniedAccess,
        Stage::ResolveTarget,
        Some(Surface::Session),
    );
    let outcome = ObservationOutcome::classify(
        request,
        ObservationInterval::new(1, 2, 3),
        None,
        None,
        vec![TerminalFailure::DeniedAccess],
        vec![diagnostic],
    )
    .unwrap();
    let bytes = wire::encode_outcome(&outcome).unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 8);
    assert_eq!(value["outcome"], "denied_access");
    assert!(value["resolved_target"].is_null());
    assert!(value["snapshot"].is_null());
    assert!(value["selection"].is_null());
    assert_eq!(value["diagnostics"][0]["code"], "denied_access");
    assert_eq!(
        value["diagnostics"][0]["message"],
        "Access to target was denied"
    );
    assert_eq!(
        value["diagnostics"][0]["action"],
        "Check access and explicit target selection"
    );
    assert!(!String::from_utf8(bytes).unwrap().contains(SECRET));
}

#[test]
fn invalidated_worker_evidence_is_preserved_without_becoming_current_text() {
    use godot_agent_kit::observation::{
        DecimalCounter, DirtyReason, EngineVersion, FactReason, FileIdentity, SourceReason,
    };
    let fixture = Fixture::new();
    let request = fixture.request(Some(ID1));
    let selected = godot_agent_kit::observation::ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(
            DecimalCounter::new("1").unwrap(),
            DecimalCounter::new("2").unwrap(),
        ),
        SessionId::new(ID1).unwrap(),
        EngineVersion::new(VERSION, HASH).unwrap(),
    )
    .unwrap();
    let mut sample = observation_sample(&json!([
        1,
        "hello",
        request.request_id().as_str(),
        ID1,
        request.project_root().as_str()
    ]));
    let source_reason =
        json!({"code":"source_changed","action":SourceReason::SourceChanged.action()});
    let dirty_reason =
        json!({"code":"source_changed","action":DirtyReason::SourceChanged.action()});
    let fact_reason = json!({"code":"source_changed","action":FactReason::SourceChanged.action()});
    let resource = sample["R"].take();
    sample["R"] = json!({"authority":"R","availability":"unavailable","text":null,"collection":null,"witness":null,"staleness":null,
        "reason":source_reason,"invalidated_evidence":{"text":resource["text"],"collection":resource["collection"],"witness":resource["witness"],"staleness":resource["staleness"],"reason":source_reason}});
    let dirty = sample["dirty"].take();
    sample["dirty"] = json!({"availability":"unavailable","state":"unknown","collection":null,"witness":null,"reason":dirty_reason,
        "invalidated_evidence":{"state":dirty["state"],"collection":dirty["collection"],"witness":dirty["witness"],"reason":dirty_reason}});
    let validity = sample["document"]["validity"].take();
    sample["document"]["validity"] = json!({"value":null,"collection":null,"reason":fact_reason,
        "invalidated_evidence":{"value":validity["value"],"collection":validity["collection"],"reason":fact_reason}});
    let event = json!({"v":1,"request_id":request.request_id().as_str(),"kind":"sample","sample":{
        "document":sample["document"],"R":sample["R"],"B":sample["B"],"dirty":sample["dirty"],"diagnostics":[]}});
    let decoded =
        wire::decode_event(event.to_string().as_bytes(), &request, Some(&selected), 100).unwrap();
    let Event::Sample(sample) = decoded else {
        panic!("expected validated partial sample")
    };
    assert_eq!(sample.resource.text(), None);
    assert_eq!(
        sample.resource.invalidated_evidence().unwrap().text(),
        "\u{feff}x\r\n"
    );
    assert_eq!(sample.buffer.text(), Some("x\n"));
    assert!(sample.dirty.state().is_none());
    assert!(sample.dirty.invalidated_evidence().is_some());
    assert!(sample.document.validity().value().is_none());
    assert!(sample.document.validity().invalidated_evidence().is_some());
    let mut wrong = event;
    wrong["sample"]["R"]["invalidated_evidence"]["witness"]["script_instance_id"] = json!("999");
    assert!(
        wire::decode_event(wrong.to_string().as_bytes(), &request, Some(&selected), 100).is_err()
    );
}

#[test]
fn actual_caller_completes_supported_peer_and_preserves_rechecked_partial_evidence() {
    // These peers test the real caller/worker pipeline, not Godot's R/B observability.
    for (mode, expected, exit) in [
        (ObservationMode::Stable, "complete_observation", 0),
        (ObservationMode::Valid, "limited_observation", 2),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        let sentinel = "# T003_INTENTIONAL_DISK_RESULT_ONLY\n";
        fs::write(fixture.project.join("scripts/subject.gd"), sentinel).unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let began = Instant::now();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_observe-gdscript"))
            .args([
                "--registry",
                fixture.registry.to_str().unwrap(),
                "--project",
                fixture.project.to_str().unwrap(),
                "--script",
                "res://scripts/subject.gd",
            ])
            .output()
            .unwrap();
        assert!(began.elapsed() < Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(exit));
        assert!(output.stderr.is_empty());
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], expected);
        assert_eq!(result["resolved_target"]["session_id"], ID1);
        assert_eq!(result["snapshot"]["sources"]["D"]["text"], sentinel);
        assert_eq!(result["snapshot"]["sources"]["R"]["text"], "\u{feff}x\r\n");
        assert_eq!(result["snapshot"]["agreement"], "divergent");
        if matches!(mode, ObservationMode::Valid) {
            assert!(result["snapshot"]["sources"]["B"].get("text").is_none());
            assert_eq!(
                result["snapshot"]["sources"]["B"]["invalidated_evidence"]["text"],
                "x\n"
            );
        } else {
            assert_eq!(result["snapshot"]["sources"]["B"]["text"], "x\n");
        }
        assert!(!String::from_utf8(output.stdout).unwrap().contains(SECRET));
        peer.join().unwrap();
    }
}

#[test]
fn selected_channel_cannot_be_reused_for_another_request_or_project() {
    let fixture = Fixture::new();
    let peer = attach(&fixture, ID1, Mode::Valid);
    let mut selected = fixture.resolve(None, Duration::from_secs(2)).unwrap();
    for (id, root) in [
        ("another_request", fixture.project.to_str().unwrap()),
        ("try_1", "/different_project"),
    ] {
        let request = ObservationRequest::new(
            RequestId::new(id).unwrap(),
            ProjectRoot::new(root).unwrap(),
            None,
            ResourcePath::new("res://scripts/subject.gd").unwrap(),
        );
        let start = Instant::now();
        let failure = wire::observe(
            &mut selected,
            &request,
            start,
            start + Duration::from_secs(1),
        )
        .err()
        .unwrap();
        assert_eq!(failure.outcome, OutcomeKind::ProtocolError);
    }
    drop(selected);
    peer.join().unwrap();
}
