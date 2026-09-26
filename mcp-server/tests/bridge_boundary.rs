use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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
#[derive(Clone, Copy)]
enum Mode {
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
        let caps = json!({"observe_gdscript":false,"open_enumeration":true,"buffer_attribution":false,"unsaved_paths":false,"cached_resource_lookup":false});
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
