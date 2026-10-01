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
const NATIVE_BUILD_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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
        write!(file, "{}", json!({"v":4,"session_id":id,"project_root":self.project,"godot_version":VERSION,"engine_hash":HASH,"host":"127.0.0.1","port":port,"token":SECRET})).unwrap();
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
    for _ in 0..1000 {
        // Reserve one distinct port per attempt across concurrent fixture tests.
        let port = 50000
            + ((std::process::id() as usize * 97 + NEXT.fetch_add(1, Ordering::Relaxed)) % 15000)
                as u16;
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
    field(&mut bytes, b"godot-agent-kit/editor-bridge/v4");
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
        "edit_open_gdscript",
        "open_gdscript",
        "discover_gdscripts",
    ] {
        field(&mut bytes, &[u8::from(caps[name].as_bool().unwrap())]);
    }
    let native = caps["edit_open_gdscript"].as_bool().unwrap();
    field(&mut bytes, &(if native { 2u32 } else { 0 }).to_be_bytes());
    field(
        &mut bytes,
        if native {
            NATIVE_BUILD_ID.as_bytes()
        } else {
            b""
        },
    );
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
        "v":4,"kind":"sample","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":path,
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
fn unavailable_source(
    authority: &str,
    code: &str,
    reason: godot_agent_kit::observation::SourceReason,
) -> Value {
    json!({"authority":authority,"availability":"unavailable","text":null,"collection":null,
        "witness":null,"staleness":null,"reason":{"code":code,"action":reason.action()},
        "invalidated_evidence":null})
}

#[derive(Clone, Copy)]
enum Mode {
    ObservationAt(&'static str, ObservationMode),
    Observation(ObservationMode),
    SourceCapableNoObserve,
    Valid,
    ValidNative,
    ChangedNativeBuild,
    ChangedCapability(&'static str),
    MissingDiscovery,
    ExtraCapability,
    WrongDiscoveryType,
    DuplicateDiscovery,
    ChangedIdentity(&'static str),
    ReflectedFinish,
    ChangedFinishCapability,
    OldHandshake,
    ChangedNativeRevision,
    ChangedFinishBuild,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservationMode {
    Valid,
    PartialDivergence,
    WhitespaceDivergence,
    DirtyEqual,
    DirtyUnknown,
    BufferAndDirtyChanged,
    DocumentClosed,
    DocumentReplaced,
    PartialChanged,
    PartialClosed,
    ChangedResource,
    DenyObserve,
    DenyRecheck,
    WrongRequestDenial,
    FactOutsideSample,
    RecheckBeforeSample,
    WrongRecheckIdentity,
    ClosedCached,
    ClosedUnloaded,
    ClosedUnloadedUnknown,
    ClosedDenyRecheck,
    ClosedDisconnectRecheck,
    ClosedSilentRecheck,
    ClosedReplaceDisk,
    MissingClosed,
    InvalidUnknown,
    InvalidSyntax,
    MissingDisk,
    UnreadableDisk,
    ResourceUnavailable,
    BufferUnavailable,
    OpenUnknown,
    ExactResource,
    ExactBuffer,
    ExcessClosedResource,
    Builtin,
    BuiltinUnknown,
    Stable,
    Empty,
    WrongIdentity,
    WrongRequest,
    WrongDocument,
    DuplicateRequired,
    Nested,
    WrongWitness,
    WrongClock,
    SecretUnknownField,
    ExcessDiagnostics,
    ExcessSource,
    ExcessBuffer,
    ExcessSourceWrongWitness,
    OverlargeFrame,
    Disconnect,
    Silent,
    DisconnectRecheck,
    SilentRecheck,
    WrongRecheckRequest,
    MalformedRecheck,
    UnavailableRecheck,
}
fn serve(
    listener: TcpListener,
    mode: Mode,
    replacement: Option<PathBuf>,
) -> thread::JoinHandle<()> {
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
        assert_eq!(hello[0], 4);
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
        let native = matches!(
            mode,
            Mode::ValidNative
                | Mode::ChangedNativeBuild
                | Mode::ChangedNativeRevision
                | Mode::ChangedFinishBuild
        );
        let caps = json!({"observe_gdscript":matches!(mode, Mode::Observation(_) | Mode::ObservationAt(_, _) | Mode::SourceCapableNoObserve),"open_enumeration":true,"buffer_attribution":false,"unsaved_paths":false,"cached_resource_lookup":false,"edit_open_gdscript":native,"open_gdscript":false,"discover_gdscripts":false});
        let key = if matches!(mode, Mode::WrongSecret) {
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        } else {
            SECRET
        };
        let server_proof = match mode {
            Mode::Replay => {
                "59f272feb95a7ebc4c16d7ea642b0d8640bda97370abbba2a6e1a339bfaa5f02".to_owned()
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
        let mut challenge = json!({"v":4,"kind":"challenge","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":if native {2} else {0},"native_build_id":if native {NATIVE_BUILD_ID} else {""},"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"server_proof":server_proof});
        match mode {
            Mode::ChangedNativeBuild => challenge["native_build_id"] = json!("b".repeat(64)),
            Mode::ChangedCapability(name) => {
                challenge["capabilities"][name] = json!(!caps[name].as_bool().unwrap());
            }
            Mode::ChangedNativeRevision => challenge["native_api_revision"] = json!(1),
            Mode::OldHandshake => challenge["v"] = json!(3),
            Mode::MissingDiscovery => {
                challenge["capabilities"]
                    .as_object_mut()
                    .unwrap()
                    .remove("discover_gdscripts");
            }
            Mode::ExtraCapability => challenge["capabilities"]["extra"] = json!(false),
            Mode::WrongDiscoveryType => challenge["capabilities"]["discover_gdscripts"] = json!(0),
            Mode::ChangedIdentity(name) => challenge[name] = json!("different-identity"),
            _ => {}
        }
        if matches!(mode, Mode::ChangedTranscript) {
            challenge["request_id"] = json!("other-request");
        }
        if matches!(mode, Mode::NullProof) {
            challenge["server_proof"] = Value::Null;
        }
        if matches!(mode, Mode::DuplicateField | Mode::DuplicateDiscovery) {
            let mut encoded = challenge.to_string();
            if matches!(mode, Mode::DuplicateDiscovery) {
                encoded = encoded.replace(
                    "\"discover_gdscripts\":false",
                    "\"discover_gdscripts\":false,\"discover_gdscripts\":false",
                );
            } else {
                encoded.pop();
                encoded.push_str(",\"server_nonce\":\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"}");
            }
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
                | Mode::ChangedNativeBuild
                | Mode::ChangedCapability(_)
                | Mode::MissingDiscovery
                | Mode::ExtraCapability
                | Mode::WrongDiscoveryType
                | Mode::DuplicateDiscovery
                | Mode::ChangedIdentity(_)
                | Mode::ChangedNativeRevision
                | Mode::OldHandshake
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
        let finish = json!({"v":4,"kind":"hello","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,"capabilities":caps,"native_api_revision":if native {2} else {0},"native_build_id":if native {NATIVE_BUILD_ID} else {""},"client_nonce":hello[5],"server_nonce":SERVER_NONCE,"finish_proof":proof(b"finish", &hello, &caps, SECRET)});
        let mut finish = finish;
        if matches!(mode, Mode::ChangedFinish) {
            finish["capabilities"]["unsaved_paths"] = json!(true);
        }
        if matches!(mode, Mode::ChangedFinishBuild) {
            finish["native_build_id"] = json!("b".repeat(64));
        }
        if matches!(mode, Mode::ChangedFinishCapability) {
            finish["capabilities"]["open_gdscript"] = json!(true);
        }
        if matches!(mode, Mode::ReflectedFinish) {
            finish["finish_proof"] = json!(proof(b"server", &hello, &caps, SECRET));
        }
        frame(&mut socket, &finish);
        if let Mode::Observation(reply) | Mode::ObservationAt(_, reply) = mode {
            let path = if let Mode::ObservationAt(path, _) = mode {
                path
            } else {
                "res://scripts/subject.gd"
            };
            let observe = read_frame(&mut socket);
            assert_eq!(
                observe,
                json!([4, "observe", hello[2], hello[3], hello[4], path])
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
            if matches!(
                reply,
                ObservationMode::DenyObserve | ObservationMode::WrongRequestDenial
            ) {
                frame(
                    &mut socket,
                    &json!({"v":4,"kind":"failure","request_id":if matches!(reply, ObservationMode::WrongRequestDenial) { json!("different-request") } else { hello[2].clone() },"session_id":hello[3],"project_root":hello[4],"script_path":"res://scripts/subject.gd","code":"out_of_project","stage":"read_editor"}),
                );
                return;
            }
            if matches!(reply, ObservationMode::BuiltinUnknown) {
                frame(
                    &mut socket,
                    &json!({"v":4,"kind":"failure","request_id":hello[2],
                    "session_id":hello[3],"project_root":hello[4],"script_path":path,
                    "code":"unsupported_observation","stage":"read_editor"}),
                );
                return;
            }
            let mut sample = observation_sample(&hello);
            sample["script_path"] = json!(path);
            sample["document"]["identity"]["resource_path"] = json!(path);
            sample["R"]["witness"]["resource_path"] = json!(path);
            sample["B"]["witness"]["resource_path"] = json!(path);
            sample["dirty"]["witness"]["resource_path"] = json!(path);
            if matches!(reply, ObservationMode::Builtin) {
                sample["document"]["identity"]["kind"] = json!("builtin_gdscript");
            }
            if matches!(reply, ObservationMode::InvalidSyntax) {
                let syntax = "extends Node\nfunc broken(\n";
                sample["R"]["text"] = json!(syntax);
                sample["B"]["text"] = json!(syntax);
                sample["dirty"]["state"] = json!("clean");
            }
            if matches!(
                reply,
                ObservationMode::ClosedCached
                    | ObservationMode::ClosedUnloaded
                    | ObservationMode::ClosedUnloadedUnknown
                    | ObservationMode::ClosedDenyRecheck
                    | ObservationMode::ClosedDisconnectRecheck
                    | ObservationMode::ClosedSilentRecheck
                    | ObservationMode::ClosedReplaceDisk
                    | ObservationMode::MissingClosed
                    | ObservationMode::ExcessClosedResource
            ) {
                use godot_agent_kit::observation::{DirtyReason, SourceReason};
                let stamp = editor_stamp(&hello);
                sample["document"]["open_state"] = json!({"value":"not_open",
                    "collection":stamp,"reason":null,"invalidated_evidence":null});
                sample["document"]["identity"]["editor_instance_id"] = Value::Null;
                sample["document"]["identity"]["buffer_instance_id"] = Value::Null;
                sample["B"] =
                    unavailable_source("B", "document_not_open", SourceReason::DocumentNotOpen);
                sample["B"]["availability"] = json!("not_applicable");
                sample["dirty"] = json!({"availability":"not_applicable","state":"not_applicable",
                    "collection":null,"witness":null,
                    "reason":{"code":"document_not_open","action":DirtyReason::DocumentNotOpen.action()},
                    "invalidated_evidence":null});
                if matches!(
                    reply,
                    ObservationMode::ClosedUnloaded
                        | ObservationMode::ClosedUnloadedUnknown
                        | ObservationMode::ClosedDenyRecheck
                        | ObservationMode::ClosedDisconnectRecheck
                        | ObservationMode::ClosedSilentRecheck
                        | ObservationMode::ClosedReplaceDisk
                        | ObservationMode::MissingClosed
                ) {
                    sample["R"] = unavailable_source(
                        "R",
                        "resource_not_loaded",
                        SourceReason::ResourceNotLoaded,
                    );
                }
                if matches!(reply, ObservationMode::ExcessClosedResource) {
                    sample["R"]["text"] = json!(
                        "λ".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES / 2) + "!"
                    );
                }
                if matches!(
                    reply,
                    ObservationMode::ClosedUnloadedUnknown
                        | ObservationMode::ClosedDenyRecheck
                        | ObservationMode::ClosedDisconnectRecheck
                        | ObservationMode::ClosedSilentRecheck
                        | ObservationMode::ClosedReplaceDisk
                        | ObservationMode::MissingClosed
                ) {
                    sample["document"]["identity"] = Value::Null;
                    sample["document"]["validity"] = json!({"value":null,"collection":null,
                        "reason":{"code":"unavailable","action":godot_agent_kit::observation::FactReason::Unavailable.action()},
                        "invalidated_evidence":null});
                }
            }
            if matches!(reply, ObservationMode::ResourceUnavailable) {
                use godot_agent_kit::observation::SourceReason;
                sample["R"] = unavailable_source(
                    "R",
                    "resource_unreadable",
                    SourceReason::ResourceUnreadable,
                );
            }
            if matches!(
                reply,
                ObservationMode::BufferUnavailable
                    | ObservationMode::OpenUnknown
                    | ObservationMode::InvalidUnknown
            ) {
                use godot_agent_kit::observation::{DirtyReason, SourceReason};
                let (code, reason) = if matches!(
                    reply,
                    ObservationMode::OpenUnknown | ObservationMode::InvalidUnknown
                ) {
                    ("open_state_unknown", SourceReason::OpenStateUnknown)
                } else {
                    ("buffer_unreadable", SourceReason::BufferUnreadable)
                };
                sample["B"] = unavailable_source("B", code, reason);
                sample["dirty"] = json!({"availability":"unavailable","state":"unknown",
                    "collection":null,"witness":null,"reason":{"code":if matches!(reply, ObservationMode::OpenUnknown | ObservationMode::InvalidUnknown) { "open_state_unknown" } else { "dirty_attribution_unavailable" },
                    "action":if matches!(reply, ObservationMode::OpenUnknown | ObservationMode::InvalidUnknown) { DirtyReason::OpenStateUnknown.action() } else { DirtyReason::DirtyAttributionUnavailable.action() }},
                    "invalidated_evidence":null});
                if matches!(
                    reply,
                    ObservationMode::OpenUnknown | ObservationMode::InvalidUnknown
                ) {
                    sample["document"]["identity"]["editor_instance_id"] = Value::Null;
                    sample["document"]["identity"]["buffer_instance_id"] = Value::Null;
                    sample["document"]["open_state"] = json!({"value":null,"collection":null,
                        "reason":{"code":"open_state_unknown","action":godot_agent_kit::observation::FactReason::OpenStateUnknown.action()},
                        "invalidated_evidence":null});
                }
                if matches!(reply, ObservationMode::InvalidUnknown) {
                    sample["document"]["identity"] = Value::Null;
                    sample["document"]["validity"]["value"] = json!("invalid");
                    sample["R"] = unavailable_source(
                        "R",
                        "resource_not_loaded",
                        SourceReason::ResourceNotLoaded,
                    );
                }
            }
            if matches!(reply, ObservationMode::ExactResource) {
                sample["R"]["text"] =
                    json!("λ".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES / 2));
            }
            if matches!(reply, ObservationMode::ExactBuffer) {
                sample["B"]["text"] =
                    json!("λ".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES / 2));
            }
            if matches!(reply, ObservationMode::WrongRequest) {
                sample["request_id"] = json!("another-request");
            }
            if matches!(reply, ObservationMode::WrongDocument) {
                sample["document"]["identity"]["resource_path"] = json!("res://scripts/other.gd");
            }
            if matches!(reply, ObservationMode::WrongIdentity) {
                sample["session_id"] = json!(ID2);
            }
            if matches!(reply, ObservationMode::WrongWitness) {
                sample["R"]["witness"]["script_instance_id"] = json!("123");
            }
            if matches!(reply, ObservationMode::FactOutsideSample) {
                sample["R"]["collection"]["finished_tick_us"] = json!("9007199254740995");
            }
            if matches!(reply, ObservationMode::WrongClock) {
                sample["B"]["collection"]["clock_id"] = json!("caller");
            }
            if matches!(reply, ObservationMode::Empty) {
                sample["R"]["text"] = json!("");
                sample["B"]["text"] = json!("");
                sample["dirty"]["state"] = json!("clean");
            }
            if matches!(
                reply,
                ObservationMode::PartialDivergence
                    | ObservationMode::WhitespaceDivergence
                    | ObservationMode::DirtyEqual
                    | ObservationMode::DirtyUnknown
                    | ObservationMode::BufferAndDirtyChanged
                    | ObservationMode::DocumentClosed
                    | ObservationMode::DocumentReplaced
                    | ObservationMode::PartialChanged
                    | ObservationMode::PartialClosed
            ) {
                sample["R"]["text"] = json!(if matches!(
                    reply,
                    ObservationMode::PartialDivergence | ObservationMode::WhitespaceDivergence
                ) {
                    "x\r\n"
                } else {
                    "x\n"
                });
                sample["B"]["text"] =
                    json!(if matches!(reply, ObservationMode::PartialDivergence) {
                        "x\r\n"
                    } else if matches!(reply, ObservationMode::WhitespaceDivergence) {
                        "x \n"
                    } else {
                        "x\n"
                    });
                if matches!(reply, ObservationMode::BufferAndDirtyChanged) {
                    sample["dirty"]["state"] = json!("clean");
                }
                if matches!(reply, ObservationMode::DirtyUnknown) {
                    use godot_agent_kit::observation::DirtyReason;
                    sample["dirty"] = json!({"availability":"unavailable","state":"unknown",
                        "reason":{"code":"dirty_attribution_unavailable",
                            "action":DirtyReason::DirtyAttributionUnavailable.action()}});
                }
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
            if matches!(reply, ObservationMode::ExcessBuffer) {
                sample["B"]["text"] =
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
            if matches!(
                reply,
                ObservationMode::Valid
                    | ObservationMode::Stable
                    | ObservationMode::Empty
                    | ObservationMode::ChangedResource
                    | ObservationMode::DenyRecheck
                    | ObservationMode::RecheckBeforeSample
                    | ObservationMode::WrongRecheckIdentity
                    | ObservationMode::PartialDivergence
                    | ObservationMode::WhitespaceDivergence
                    | ObservationMode::DirtyEqual
                    | ObservationMode::DirtyUnknown
                    | ObservationMode::BufferAndDirtyChanged
                    | ObservationMode::DocumentClosed
                    | ObservationMode::DocumentReplaced
                    | ObservationMode::PartialChanged
                    | ObservationMode::PartialClosed
                    | ObservationMode::DisconnectRecheck
                    | ObservationMode::SilentRecheck
                    | ObservationMode::WrongRecheckRequest
                    | ObservationMode::MalformedRecheck
                    | ObservationMode::UnavailableRecheck
                    | ObservationMode::ClosedCached
                    | ObservationMode::ClosedUnloaded
                    | ObservationMode::ClosedUnloadedUnknown
                    | ObservationMode::ClosedDenyRecheck
                    | ObservationMode::ClosedDisconnectRecheck
                    | ObservationMode::ClosedSilentRecheck
                    | ObservationMode::ClosedReplaceDisk
                    | ObservationMode::MissingClosed
                    | ObservationMode::InvalidUnknown
                    | ObservationMode::InvalidSyntax
                    | ObservationMode::MissingDisk
                    | ObservationMode::UnreadableDisk
                    | ObservationMode::ResourceUnavailable
                    | ObservationMode::BufferUnavailable
                    | ObservationMode::OpenUnknown
                    | ObservationMode::ExactResource
                    | ObservationMode::ExactBuffer
                    | ObservationMode::ExcessClosedResource
                    | ObservationMode::ExcessSource
                    | ObservationMode::ExcessBuffer
                    | ObservationMode::Builtin
            ) {
                // A direct sample consumer may close without asking for recheck.
                // Full caller cases below still assert their terminal evidence.
                if socket.peek(&mut [0u8; 1]).unwrap() == 0 {
                    return;
                }
                let recheck = read_frame(&mut socket);
                assert_eq!(
                    recheck,
                    json!([4, "recheck", hello[2], hello[3], hello[4], path])
                );
                if matches!(
                    reply,
                    ObservationMode::DisconnectRecheck | ObservationMode::ClosedDisconnectRecheck
                ) {
                    return;
                }
                if matches!(
                    reply,
                    ObservationMode::SilentRecheck | ObservationMode::ClosedSilentRecheck
                ) {
                    thread::sleep(Duration::from_secs(5));
                    return;
                }
                if matches!(reply, ObservationMode::MalformedRecheck) {
                    socket.write_all(&2u32.to_be_bytes()).unwrap();
                    socket.write_all(b"{}").unwrap();
                    return;
                }
                if matches!(
                    reply,
                    ObservationMode::DenyRecheck | ObservationMode::ClosedDenyRecheck
                ) {
                    frame(
                        &mut socket,
                        &json!({"v":4,"kind":"failure","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":"res://scripts/subject.gd","code":"out_of_project","stage":"recheck"}),
                    );
                    return;
                }
                if matches!(reply, ObservationMode::ClosedReplaceDisk) {
                    let original = replacement.as_ref().expect("test controls this disk file");
                    let next = original.with_extension("next");
                    fs::write(
                        &next,
                        "λ".repeat(godot_agent_kit::observation::SOURCE_LIMIT_BYTES / 2) + "!",
                    )
                    .unwrap();
                    fs::rename(next, original).unwrap();
                }
                let changes = match reply {
                    ObservationMode::Stable
                    | ObservationMode::Empty
                    | ObservationMode::UnavailableRecheck
                    | ObservationMode::RecheckBeforeSample
                    | ObservationMode::ClosedCached
                    | ObservationMode::ClosedUnloaded
                    | ObservationMode::ClosedUnloadedUnknown
                    | ObservationMode::ClosedDenyRecheck
                    | ObservationMode::ClosedDisconnectRecheck
                    | ObservationMode::ClosedSilentRecheck
                    | ObservationMode::ClosedReplaceDisk
                    | ObservationMode::MissingClosed
                    | ObservationMode::InvalidUnknown
                    | ObservationMode::InvalidSyntax
                    | ObservationMode::MissingDisk
                    | ObservationMode::UnreadableDisk
                    | ObservationMode::ResourceUnavailable
                    | ObservationMode::BufferUnavailable
                    | ObservationMode::OpenUnknown
                    | ObservationMode::ExactResource
                    | ObservationMode::ExactBuffer
                    | ObservationMode::ExcessClosedResource
                    | ObservationMode::ExcessSource
                    | ObservationMode::ExcessBuffer
                    | ObservationMode::Builtin => json!([]),
                    ObservationMode::ChangedResource => {
                        json!([{"surface":"R","code":"source_changed"}])
                    }
                    ObservationMode::PartialDivergence
                    | ObservationMode::WhitespaceDivergence
                    | ObservationMode::DirtyEqual
                    | ObservationMode::DirtyUnknown => json!([]),
                    ObservationMode::BufferAndDirtyChanged | ObservationMode::PartialChanged => {
                        json!([{"surface":"B","code":"source_changed"},
                               {"surface":"dirty","code":"source_changed"}])
                    }
                    ObservationMode::DocumentClosed | ObservationMode::PartialClosed => {
                        json!([{"surface":"document","code":"document_closed"}])
                    }
                    ObservationMode::DocumentReplaced => {
                        json!([{"surface":"document","code":"identity_changed"}])
                    }
                    _ => json!([{"surface":"B","code":"source_changed"}]),
                };
                let mut stamp = editor_stamp(&hello);
                stamp["started_tick_us"] =
                    json!(if matches!(reply, ObservationMode::RecheckBeforeSample) {
                        "9007199254740993"
                    } else {
                        "9007199254740994"
                    });
                stamp["finished_tick_us"] = json!("9007199254740995");
                let mut rechecked = json!({"v":4,"kind":"recheck","request_id":hello[2],"session_id":hello[3],"project_root":hello[4],"script_path":path,"collection":stamp,"checks":"performed","detected_changes":changes,"reason":null});
                if matches!(reply, ObservationMode::WrongRecheckRequest) {
                    rechecked["request_id"] = json!("another-request");
                }
                if matches!(reply, ObservationMode::UnavailableRecheck) {
                    rechecked["checks"] = json!("unavailable");
                    rechecked["reason"] = json!("unavailable");
                }
                if matches!(
                    reply,
                    ObservationMode::PartialChanged | ObservationMode::PartialClosed
                ) {
                    rechecked["checks"] = json!("unavailable");
                    rechecked["reason"] = json!("unavailable");
                }
                if matches!(reply, ObservationMode::WrongRecheckIdentity) {
                    rechecked["session_id"] = json!(ID2);
                }
                frame(&mut socket, &rechecked);
            }
            return;
        }
        // Source-capable ambiguity must not issue observe; the source-free
        // T002 modes also close without requesting any document authority.
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
    serve(
        socket,
        mode,
        if matches!(mode, Mode::Observation(ObservationMode::ClosedReplaceDisk)) {
            Some(fixture.project.join("scripts/subject.gd"))
        } else {
            None
        },
    )
}
const ID1: &str = "00112233445566778899aabbccddeeff";
const ID2: &str = "11112233445566778899aabbccddeeff";
const ID3: &str = "22222233445566778899aabbccddeeff";
fn invoke(
    fixture: &Fixture,
    project: &std::path::Path,
    session: Option<&str>,
) -> std::process::Output {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_observe-gdscript"));
    command.args([
        "--registry",
        fixture.registry.to_str().unwrap(),
        "--project",
        project.to_str().unwrap(),
        "--script",
        "res://scripts/subject.gd",
    ]);
    if let Some(session) = session {
        command.args(["--session", session]);
    }
    command.output().unwrap()
}
fn invoke_at(fixture: &Fixture, locator: &str) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_observe-gdscript"))
        .args([
            "--registry",
            fixture.registry.to_str().unwrap(),
            "--project",
            fixture.project.to_str().unwrap(),
            "--script",
            locator,
        ])
        .output()
        .unwrap()
}

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
fn actual_caller_refuses_source_until_unique_selection_and_binds_exact_session() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    let disk = "# selected project and session only\n";
    fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
    let first = attach(&fixture, ID1, Mode::SourceCapableNoObserve);
    let second = attach(&fixture, ID2, Mode::SourceCapableNoObserve);
    let started = Instant::now();
    let output = invoke(&fixture, &fixture.project, None);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "ambiguous_target");
    assert_eq!(result["selection"]["missing_selector"], "session_id");
    assert_eq!(result["selection"]["candidate_sessions"], json!([ID1, ID2]));
    assert!(result["resolved_target"].is_null());
    assert!(result["snapshot"].is_null());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(disk));
    first.join().unwrap();
    second.join().unwrap();

    // Two advertisements exist, but only the exact session's authenticated
    // channel may receive observe/recheck or contribute project source.
    fs::remove_file(fixture.registry.join(format!("{ID1}.json"))).unwrap();
    let unused = listener();
    fixture.descriptor(ID1, unused.local_addr().unwrap().port());
    fs::remove_file(fixture.registry.join(format!("{ID2}.json"))).unwrap();
    let selected = attach(&fixture, ID2, Mode::Observation(ObservationMode::Stable));
    let started = Instant::now();
    let output = invoke(&fixture, &fixture.project, Some(ID2));
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(output.status.code(), Some(0));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "complete_observation");
    assert_eq!(result["resolved_target"]["session_id"], ID2);
    assert_eq!(
        result["resolved_target"]["project_root"],
        fixture.project.to_str().unwrap()
    );
    assert_eq!(
        result["snapshot"]["target"]["script_path"],
        "res://scripts/subject.gd"
    );
    assert_eq!(result["snapshot"]["sources"]["D"]["text"], disk);
    assert_eq!(
        result["snapshot"]["sources"]["R"]["collection"]["clock_id"],
        format!("editor:{ID2}")
    );
    assert_eq!(
        result["snapshot"]["sources"]["B"]["witness"]["resource_path"],
        "res://scripts/subject.gd"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
    unused.set_nonblocking(true).unwrap();
    assert_eq!(
        unused.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    selected.join().unwrap();
}

#[test]
fn absent_and_ended_exact_sessions_never_substitute_a_replacement_or_read_disk() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    let disk = "# source that absence cannot disclose\n";
    fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
    let absent = invoke(&fixture, &fixture.project, Some(ID1));
    assert_eq!(absent.status.code(), Some(3));
    let first: Value = serde_json::from_slice(&absent.stdout).unwrap();
    assert_eq!(first["outcome"], "editor_unavailable");
    assert!(first["snapshot"].is_null());

    let ended = listener();
    let old_port = ended.local_addr().unwrap().port();
    drop(ended);
    fixture.descriptor(ID1, old_port);
    let replacement = listener();
    fixture.descriptor(ID2, replacement.local_addr().unwrap().port());
    let output = invoke(&fixture, &fixture.project, Some(ID1));
    assert_eq!(output.status.code(), Some(3));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "editor_unavailable");
    assert!(result["snapshot"].is_null());
    assert!(result["resolved_target"].is_null());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(disk));
    replacement.set_nonblocking(true).unwrap();
    assert_eq!(
        replacement.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
#[test]
fn same_named_projects_with_the_same_script_path_keep_disk_and_editor_lifetimes_separate() {
    // A controlled peer supplies R/B shapes; only the independently confined D
    // is a filesystem observation here. The live GUI scenario proves actual R/B.
    let fixture = Fixture::new();
    let other_parent = fixture.home.join("other");
    DirBuilder::new().mode(0o700).create(&other_parent).unwrap();
    let other = other_parent.join("project");
    DirBuilder::new().mode(0o700).create(&other).unwrap();
    for (project, source) in [
        (&fixture.project, "# project A only\n"),
        (&other, "# project B only\n"),
    ] {
        fs::create_dir(project.join("scripts")).unwrap();
        fs::write(project.join("scripts/subject.gd"), source).unwrap();
    }
    let first = attach(&fixture, ID1, Mode::Observation(ObservationMode::Stable));
    let second_listener = listener();
    fixture.descriptor(ID2, second_listener.local_addr().unwrap().port());
    let descriptor = fixture.registry.join(format!("{ID2}.json"));
    let mut metadata: Value = serde_json::from_slice(&fs::read(&descriptor).unwrap()).unwrap();
    metadata["project_root"] = json!(other);
    fs::write(descriptor, serde_json::to_vec(&metadata).unwrap()).unwrap();
    let second = serve(
        second_listener,
        Mode::Observation(ObservationMode::Stable),
        None,
    );
    for (project, id, selected, unselected) in [
        (
            &fixture.project,
            ID1,
            "# project A only\n",
            "# project B only\n",
        ),
        (&other, ID2, "# project B only\n", "# project A only\n"),
    ] {
        let began = Instant::now();
        let output = invoke(&fixture, project, None);
        assert!(began.elapsed() < Duration::from_secs(5));
        assert_eq!(output.status.code(), Some(0));
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "complete_observation");
        assert_eq!(result["resolved_target"]["session_id"], id);
        assert_eq!(
            result["resolved_target"]["project_root"],
            project.to_str().unwrap()
        );
        assert_eq!(result["snapshot"]["sources"]["D"]["text"], selected);
        assert_eq!(
            result["snapshot"]["sources"]["R"]["collection"]["clock_id"],
            format!("editor:{id}")
        );
        assert_eq!(
            result["snapshot"]["document"]["identity"]["resource_path"],
            "res://scripts/subject.gd"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains(unselected));
    }
    first.join().unwrap();
    second.join().unwrap();
}

#[test]
fn rebound_ended_port_without_the_original_secret_cannot_release_project_source() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    let disk = "# source never exposed to an unproved listener\n";
    fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
    let old_listener = listener();
    let port = old_listener.local_addr().unwrap().port();
    fixture.descriptor(ID1, port);
    drop(old_listener);
    let rebound = TcpListener::bind(("127.0.0.1", port)).unwrap();
    let impostor = serve(rebound, Mode::WrongSecret, None);
    let began = Instant::now();
    let output = invoke(&fixture, &fixture.project, Some(ID1));
    assert!(began.elapsed() < Duration::from_secs(5));
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "denied_access");
    assert_eq!(result["diagnostics"][0]["code"], "authentication_failed");
    assert!(result["resolved_target"].is_null());
    assert!(result["snapshot"].is_null());
    for source in [disk, SECRET] {
        assert!(!String::from_utf8_lossy(&output.stdout).contains(source));
    }
    impostor.join().unwrap();
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
        matches!(wire::recheck(&mut selected, &request, &sample.collection, started, started + Duration::from_secs(2)).unwrap(), Recheck::Performed { detected_changes } if detected_changes == vec![DetectedChange::Source(Authority::B)])
    );
    let target = selected.target().clone();
    drop(selected);
    peer.join().unwrap();
    let worker_sample = wire::EditorSample {
        collection: sample.collection.clone(),
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
        (ObservationMode::WrongRequest, OutcomeKind::ProtocolError),
        (ObservationMode::WrongDocument, OutcomeKind::ProtocolError),
        (
            ObservationMode::FactOutsideSample,
            OutcomeKind::ProtocolError,
        ),
        (
            ObservationMode::WrongRequestDenial,
            OutcomeKind::ProtocolError,
        ),
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
fn recheck_cannot_claim_success_with_a_pre_sample_editor_interval() {
    let fixture = Fixture::new();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::RecheckBeforeSample),
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
    let failure = wire::recheck(
        &mut selected,
        &request,
        &sample.collection,
        started,
        started + Duration::from_secs(2),
    )
    .unwrap_err();
    assert_eq!(failure.outcome, OutcomeKind::ProtocolError);
    assert_eq!(failure.diagnostic.code(), DiagnosticCode::InvalidFrame);
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn authenticated_scope_denial_before_or_after_sampling_exposes_no_source() {
    // Controlled peers exercise the actual caller and worker, not Godot getter evidence.
    for mode in [ObservationMode::DenyObserve, ObservationMode::DenyRecheck] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        let sentinel = "# confidential disk evidence\n";
        fs::write(fixture.project.join("scripts/subject.gd"), sentinel).unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
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
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stderr.is_empty());
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "denied_access");
        assert_eq!(result["diagnostics"][0]["code"], "out_of_project");
        assert_eq!(
            result["diagnostics"][0]["stage"],
            if matches!(mode, ObservationMode::DenyObserve) {
                "read_editor"
            } else {
                "recheck"
            }
        );
        assert!(result["resolved_target"].is_null());
        assert!(result["snapshot"].is_null());
        for source in [sentinel, "\u{feff}x\r\n", "x\n", SECRET] {
            assert!(!String::from_utf8_lossy(&output.stdout).contains(source));
        }
        peer.join().unwrap();
    }
}

#[test]
fn mismatched_recheck_cannot_complete_and_keeps_only_earlier_validated_facts() {
    // Actual caller and simulated editor; the wrong-session recheck is never evidence.
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    let disk = "# independent disk\n";
    fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::WrongRecheckIdentity),
    );
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
    assert_eq!(output.status.code(), Some(4));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "protocol_error");
    assert_eq!(result["snapshot"]["sources"]["D"]["text"], disk);
    assert_eq!(result["snapshot"]["sources"]["R"]["text"], "\u{feff}x\r\n");
    assert_eq!(result["snapshot"]["sources"]["B"]["text"], "x\n");
    assert_eq!(result["snapshot"]["consistency"]["checks"], "unavailable");
    assert!(!String::from_utf8(output.stdout).unwrap().contains(SECRET));
    peer.join().unwrap();
}

#[test]
fn mismatched_or_malformed_pre_sample_evidence_never_triggers_a_disk_read() {
    for mode in [
        ObservationMode::WrongIdentity,
        ObservationMode::WrongRequest,
        ObservationMode::WrongDocument,
        ObservationMode::DuplicateRequired,
        ObservationMode::OverlargeFrame,
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        let disk = "# pre-sample protocol failure is source-free\n";
        fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let started = Instant::now();
        let output = invoke(&fixture, &fixture.project, Some(ID1));
        assert!(started.elapsed() < Duration::from_secs(5), "{mode:?}");
        assert_eq!(output.status.code(), Some(4), "{mode:?}");
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "protocol_error", "{mode:?}");
        assert_eq!(result["resolved_target"]["session_id"], ID1);
        assert!(result["snapshot"].is_null(), "{mode:?}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(disk));
        assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
        peer.join().unwrap();
    }
}

#[test]
fn actual_caller_preserves_attributed_partial_facts_across_live_interruptions() {
    // Controlled authenticated peers test transport/consumer transitions, not
    // whether Godot can really observe R, B or document-specific dirty state.
    use ObservationMode::*;
    for (mode, expected, exit, after_sample) in [
        (Disconnect, "disconnected_editor", 4, false),
        (DisconnectRecheck, "disconnected_editor", 4, true),
        (SilentRecheck, "timeout", 4, true),
        (WrongRecheckRequest, "protocol_error", 4, true),
        (MalformedRecheck, "protocol_error", 4, true),
        (UnavailableRecheck, "limited_observation", 2, true),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        let disk = "# independent selected disk source\n";
        fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let started = Instant::now();
        let output = invoke(&fixture, &fixture.project, Some(ID1));
        assert!(started.elapsed() < Duration::from_secs(5), "{mode:?}");
        assert_eq!(output.status.code(), Some(exit), "{mode:?}");
        assert!(output.stderr.is_empty(), "{mode:?}");
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], expected, "{mode:?}");
        assert_eq!(result["resolved_target"]["session_id"], ID1, "{mode:?}");
        assert_eq!(
            result["resolved_target"]["project_root"],
            fixture.project.to_str().unwrap(),
            "{mode:?}"
        );
        assert_eq!(
            result["resolved_target"]["script_path"], "res://scripts/subject.gd",
            "{mode:?}"
        );
        assert!(
            result["interval"]["elapsed_us"].as_u64().unwrap() < 5_000_000,
            "{mode:?}"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
        if !after_sample {
            assert!(result["snapshot"].is_null(), "{mode:?}");
            assert!(!String::from_utf8_lossy(&output.stdout).contains(disk));
        } else {
            let snapshot = &result["snapshot"];
            assert_eq!(snapshot["target"]["session_id"], ID1, "{mode:?}");
            assert_eq!(
                snapshot["document"]["identity"]["resource_path"], "res://scripts/subject.gd",
                "{mode:?}"
            );
            assert_eq!(snapshot["sources"]["D"]["text"], disk, "{mode:?}");
            assert_eq!(
                snapshot["sources"]["D"]["collection"]["clock_id"], "caller",
                "{mode:?}"
            );
            assert_eq!(snapshot["consistency"]["checks"], "unavailable", "{mode:?}");
            assert_eq!(
                snapshot["sources"]["R"]["collection"]["clock_id"],
                if matches!(mode, DisconnectRecheck) {
                    Value::Null
                } else {
                    json!(format!("editor:{ID1}"))
                },
                "{mode:?}"
            );
            if matches!(mode, DisconnectRecheck) {
                for surface in ["R", "B"] {
                    assert!(
                        snapshot["sources"][surface].get("text").is_none(),
                        "{mode:?}"
                    );
                    assert_eq!(
                        snapshot["sources"][surface]["reason"]["code"], "session_ended",
                        "{mode:?}"
                    );
                    assert_eq!(
                        snapshot["sources"][surface]["invalidated_evidence"]["collection"]
                            ["clock_id"],
                        format!("editor:{ID1}"),
                        "{mode:?}"
                    );
                }
                assert_eq!(
                    snapshot["sources"]["R"]["invalidated_evidence"]["text"],
                    "\u{feff}x\r\n"
                );
                assert_eq!(
                    snapshot["sources"]["B"]["invalidated_evidence"]["text"],
                    "x\n"
                );
                assert_eq!(snapshot["dirty"]["invalidated_evidence"]["state"], "dirty");
                assert!(snapshot["document"]["open_state"]["value"].is_null());
                assert_eq!(snapshot["comparisons"]["disk_resource"], "unknown");
                assert_eq!(snapshot["consistency"]["recheck_reason"], "session_ended");
            } else {
                assert_eq!(
                    snapshot["sources"]["R"]["text"], "\u{feff}x\r\n",
                    "{mode:?}"
                );
                assert_eq!(snapshot["sources"]["B"]["text"], "x\n", "{mode:?}");
                assert_eq!(snapshot["dirty"]["state"], "dirty", "{mode:?}");
                assert_eq!(
                    snapshot["consistency"]["recheck_reason"],
                    if matches!(mode, SilentRecheck) {
                        "deadline_exceeded"
                    } else {
                        "unavailable"
                    },
                    "{mode:?}"
                );
            }
        }
        peer.join().unwrap();
    }
}

#[test]
fn changed_resource_only_invalidates_r_and_preserves_independent_b() {
    // This peer simulates editor evidence. It does not establish real R/B visibility.
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    let disk = "independent disk\n";
    fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::ChangedResource),
    );
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
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(result["outcome"], "limited_observation");
    assert_eq!(result["snapshot"]["sources"]["D"]["text"], disk);
    assert!(result["snapshot"]["sources"]["R"].get("text").is_none());
    assert_eq!(
        result["snapshot"]["sources"]["R"]["invalidated_evidence"]["text"],
        "\u{feff}x\r\n"
    );
    assert_eq!(result["snapshot"]["sources"]["B"]["text"], "x\n");
    assert_eq!(result["snapshot"]["dirty"]["state"], "dirty");
    assert_eq!(
        result["snapshot"]["comparisons"]["disk_buffer"],
        "different"
    );
    assert_eq!(
        result["snapshot"]["comparisons"]["resource_buffer"],
        "unknown"
    );
    peer.join().unwrap();
}

#[test]
fn dirty_divergence_and_document_transition_events_reach_the_caller() {
    use ObservationMode::*;
    for (mode, expected) in [
        (PartialDivergence, "complete_observation"),
        (WhitespaceDivergence, "complete_observation"),
        (DirtyEqual, "complete_observation"),
        (DirtyUnknown, "limited_observation"),
        (BufferAndDirtyChanged, "limited_observation"),
        (DocumentClosed, "limited_observation"),
        (DocumentReplaced, "limited_observation"),
        (PartialChanged, "limited_observation"),
        (PartialClosed, "limited_observation"),
    ] {
        let disk = "x\n";
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(fixture.project.join("scripts/subject.gd"), disk).unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
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
        peer.join().unwrap();
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], expected, "{mode:?}");
        assert_eq!(
            output.status.code(),
            Some(if expected == "complete_observation" {
                0
            } else {
                2
            }),
            "{mode:?}"
        );
        assert_eq!(
            result["snapshot"]["document"]["identity"]["script_instance_id"], "9007199254740993",
            "{mode:?}"
        );
        let snapshot = &result["snapshot"];
        match mode {
            PartialDivergence => {
                assert_eq!(snapshot["sources"]["D"]["text"], disk);
                assert_eq!(snapshot["comparisons"]["disk_resource"], "different");
                assert_eq!(snapshot["comparisons"]["disk_buffer"], "different");
                assert_eq!(snapshot["comparisons"]["resource_buffer"], "equal");
                assert_eq!(snapshot["sources"]["R"]["text"], "x\r\n");
                assert_eq!(snapshot["sources"]["B"]["text"], "x\r\n");
            }
            WhitespaceDivergence => {
                for pair in ["disk_resource", "disk_buffer", "resource_buffer"] {
                    assert_eq!(snapshot["comparisons"][pair], "different");
                }
                assert_eq!(snapshot["sources"]["B"]["text"], "x \n");
            }
            DirtyEqual => {
                assert_eq!(snapshot["agreement"], "agree");
                assert_eq!(snapshot["dirty"]["state"], "dirty");
            }
            DirtyUnknown => {
                assert_eq!(snapshot["agreement"], "agree");
                assert_eq!(snapshot["document"]["open_state"]["value"], "open");
                assert_eq!(snapshot["dirty"]["state"], "unknown");
                assert_eq!(
                    snapshot["dirty"]["reason"]["code"],
                    "dirty_attribution_unavailable"
                );
                assert_eq!(snapshot["sources"]["B"]["text"], disk);
            }
            BufferAndDirtyChanged | PartialChanged => {
                assert_eq!(snapshot["sources"]["D"]["text"], disk);
                assert_eq!(snapshot["sources"]["R"]["text"], disk);
                assert_eq!(
                    snapshot["sources"]["B"]["invalidated_evidence"]["text"],
                    disk
                );
                assert_eq!(snapshot["sources"]["B"]["reason"]["code"], "source_changed");
                assert_eq!(
                    snapshot["dirty"]["invalidated_evidence"]["state"],
                    if matches!(mode, BufferAndDirtyChanged) {
                        "clean"
                    } else {
                        "dirty"
                    }
                );
                assert_eq!(snapshot["comparisons"]["disk_resource"], "equal");
                assert_eq!(snapshot["comparisons"]["disk_buffer"], "unknown");
                assert_eq!(
                    snapshot["consistency"]["checks"],
                    if matches!(mode, PartialChanged) {
                        "unavailable"
                    } else {
                        "performed"
                    }
                );
                assert_eq!(snapshot["consistency"]["stability"], "changed");
            }
            DocumentClosed | PartialClosed => {
                assert_eq!(snapshot["sources"]["D"]["text"], disk);
                assert_eq!(snapshot["document"]["open_state"]["value"], Value::Null);
                assert_eq!(
                    snapshot["document"]["open_state"]["reason"]["code"],
                    "document_closed"
                );
                assert_eq!(snapshot["sources"]["R"]["text"], disk);
                assert_eq!(
                    snapshot["sources"]["B"]["invalidated_evidence"]["text"],
                    disk
                );
                assert_eq!(snapshot["dirty"]["invalidated_evidence"]["state"], "dirty");
                assert_eq!(
                    snapshot["consistency"]["checks"],
                    if matches!(mode, PartialClosed) {
                        "unavailable"
                    } else {
                        "performed"
                    }
                );
                assert_eq!(snapshot["comparisons"]["disk_resource"], "equal");
            }
            DocumentReplaced => {
                for authority in ["D", "R", "B"] {
                    assert_eq!(snapshot["sources"][authority]["text"], Value::Null);
                    assert_eq!(
                        snapshot["sources"][authority]["invalidated_evidence"]["text"],
                        disk
                    );
                    assert_eq!(
                        snapshot["sources"][authority]["reason"]["code"],
                        "identity_changed"
                    );
                }
                assert_eq!(snapshot["agreement"], "unknown");
                assert_eq!(snapshot["dirty"]["invalidated_evidence"]["state"], "dirty");
            }
            _ => unreachable!(),
        }
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
fn excess_buffer_text_limits_only_b_without_substituting_r() {
    use godot_agent_kit::observation::{Availability, SourceReason};
    let fixture = Fixture::new();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::ExcessBuffer),
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
    assert_eq!(sample.buffer.availability(), Availability::Unavailable);
    assert_eq!(sample.buffer.reason(), Some(SourceReason::TooLarge));
    assert_eq!(sample.buffer.text(), None);
    assert_eq!(sample.resource.text(), Some("\u{feff}x\r\n"));
    assert_eq!(
        sample.dirty.state(),
        Some(godot_agent_kit::observation::DirtyState::Dirty)
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn actual_caller_distinguishes_observed_empty_sources_from_unavailable() {
    // Simulated peer text and dirty evidence; not an independent Godot oracle.
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    fs::write(fixture.project.join("scripts/subject.gd"), "").unwrap();
    let peer = attach(&fixture, ID1, Mode::Observation(ObservationMode::Empty));
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
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "complete_observation");
    for source in ["D", "R", "B"] {
        assert_eq!(
            result["snapshot"]["sources"][source]["availability"],
            "observed"
        );
        assert_eq!(result["snapshot"]["sources"][source]["text"], "");
    }
    assert_eq!(result["snapshot"]["dirty"]["state"], "clean");
    assert_eq!(result["snapshot"]["comparisons"]["disk_resource"], "equal");
    assert_eq!(result["snapshot"]["comparisons"]["disk_buffer"], "equal");
    assert_eq!(
        result["snapshot"]["comparisons"]["resource_buffer"],
        "equal"
    );
    let elapsed = result["interval"]["elapsed_us"].as_u64().unwrap();
    assert!(elapsed > 0);
    for (source, clock) in [
        ("D", "caller".to_owned()),
        ("R", format!("editor:{ID1}")),
        ("B", format!("editor:{ID1}")),
    ] {
        let stamp = &result["snapshot"]["sources"][source]["collection"];
        assert_eq!(stamp["clock_id"], clock);
        assert!(stamp["received_elapsed_us"].as_u64().unwrap() <= elapsed);
    }
    assert!(!String::from_utf8(output.stdout).unwrap().contains(SECRET));
    peer.join().unwrap();
}

#[test]
fn caller_keeps_other_sources_when_one_editor_authority_exceeds_its_limit() {
    for (mode, unavailable, intact, intact_text) in [
        (ObservationMode::ExcessSource, "R", "B", "x\n"),
        (ObservationMode::ExcessBuffer, "B", "R", "\u{feff}x\r\n"),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(fixture.project.join("scripts/subject.gd"), "separate disk").unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "limited_observation", "{mode:?}");
        let snapshot = &result["snapshot"];
        assert_eq!(snapshot["sources"]["D"]["text"], "separate disk");
        assert_eq!(
            snapshot["sources"][unavailable]["availability"],
            "unavailable"
        );
        assert_eq!(
            snapshot["sources"][unavailable]["reason"]["code"],
            "too_large"
        );
        assert!(snapshot["sources"][unavailable]["text"].is_null());
        assert_eq!(snapshot["sources"][intact]["text"], intact_text);
        assert_eq!(snapshot["dirty"]["state"], "dirty");
        assert_eq!(snapshot["agreement"], "divergent");
        peer.join().unwrap();
    }
}

#[test]
fn closed_cached_unloaded_missing_and_non_gdscript_have_distinct_caller_outcomes() {
    use ObservationMode::*;
    for (mode, expected_r) in [
        (ClosedCached, Some("\u{feff}x\r\n")),
        (ClosedUnloaded, None),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(fixture.project.join("scripts/subject.gd"), "disk\n").unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(result["outcome"], "not_open");
        let snapshot = &result["snapshot"];
        assert_eq!(snapshot["document"]["validity"]["value"], "valid");
        assert_eq!(snapshot["document"]["open_state"]["value"], "not_open");
        assert_eq!(snapshot["sources"]["D"]["text"], "disk\n");
        assert_eq!(snapshot["sources"]["R"]["text"].as_str(), expected_r);
        if expected_r.is_none() {
            assert_eq!(
                snapshot["sources"]["R"]["reason"]["code"],
                "resource_not_loaded"
            );
        }
        assert_eq!(snapshot["sources"]["B"]["availability"], "not_applicable");
        assert_eq!(snapshot["dirty"]["availability"], "not_applicable");
        assert_eq!(snapshot["comparisons"]["disk_buffer"], "unknown");
        peer.join().unwrap();
    }
    for (locator, mode, expected) in [
        ("res://scripts/subject.gd", MissingClosed, "missing_target"),
        ("res://scripts/notes.txt", InvalidUnknown, "invalid_target"),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        if mode == InvalidUnknown {
            fs::write(
                fixture.project.join("scripts/notes.txt"),
                "PRIVATE_NON_SCRIPT",
            )
            .unwrap();
        }
        let peer = attach(&fixture, ID1, Mode::ObservationAt(locator, mode));
        let output = invoke_at(&fixture, locator);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(3), "{mode:?} {result}");
        assert_eq!(result["outcome"], expected);
        assert!(result["snapshot"].is_null());
        assert_eq!(result["resolved_target"]["script_path"], locator);
        assert!(!String::from_utf8_lossy(&output.stdout).contains("PRIVATE_NON_SCRIPT"));
        peer.join().unwrap();
    }
}

#[test]
fn syntax_invalid_gdscript_is_still_observable_and_open_missing_disk_retains_editor_facts() {
    for (mode, disk_reason) in [
        (ObservationMode::InvalidSyntax, None),
        (ObservationMode::MissingDisk, Some("disk_missing")),
        (ObservationMode::UnreadableDisk, Some("disk_unreadable")),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        let source = fixture.project.join("scripts/subject.gd");
        match mode {
            ObservationMode::InvalidSyntax => {
                fs::write(&source, "extends Node\nfunc broken(\n").unwrap()
            }
            ObservationMode::UnreadableDisk => {
                fs::write(&source, "PRIVATE_UNREADABLE_DISK").unwrap();
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).unwrap();
            }
            _ => {}
        }
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            result["outcome"],
            if disk_reason.is_some() {
                "limited_observation"
            } else {
                "complete_observation"
            },
            "{mode:?} {result}"
        );
        assert_eq!(result["snapshot"]["document"]["validity"]["value"], "valid");
        assert_eq!(
            result["snapshot"]["document"]["open_state"]["value"],
            "open"
        );
        assert_eq!(
            result["snapshot"]["sources"]["D"]["reason"]["code"].as_str(),
            disk_reason
        );
        assert!(result["snapshot"]["sources"]["R"]["text"]
            .as_str()
            .is_some());
        assert!(result["snapshot"]["sources"]["B"]["text"]
            .as_str()
            .is_some());
        assert!(
            result["snapshot"]["comparisons"]["disk_resource"] == "unknown"
                || disk_reason.is_none()
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("PRIVATE_UNREADABLE_DISK"));
        peer.join().unwrap();
    }
}

#[test]
fn unavailable_editor_surfaces_and_unknown_open_never_infer_another_authority() {
    for (mode, expected_source, reason) in [
        (
            ObservationMode::ResourceUnavailable,
            "R",
            "resource_unreadable",
        ),
        (ObservationMode::BufferUnavailable, "B", "buffer_unreadable"),
        (ObservationMode::OpenUnknown, "B", "open_state_unknown"),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(fixture.project.join("scripts/subject.gd"), "independent D").unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        let snapshot = &result["snapshot"];
        assert_eq!(
            result["outcome"], "limited_observation",
            "{mode:?} {result}"
        );
        assert_eq!(snapshot["sources"]["D"]["text"], "independent D");
        assert_eq!(
            snapshot["sources"][expected_source]["availability"],
            "unavailable"
        );
        assert_eq!(
            snapshot["sources"][expected_source]["reason"]["code"],
            reason
        );
        assert!(snapshot["sources"][expected_source]["text"].is_null());
        if mode == ObservationMode::OpenUnknown {
            assert!(snapshot["document"]["open_state"]["value"].is_null());
            assert_eq!(
                snapshot["document"]["open_state"]["reason"]["code"],
                "open_state_unknown"
            );
            assert_eq!(snapshot["dirty"]["state"], "unknown");
        } else {
            assert_eq!(snapshot["document"]["open_state"]["value"], "open");
        }
        peer.join().unwrap();
    }
}

#[test]
fn oversized_disk_keeps_independent_open_or_unknown_open_editor_facts() {
    use godot_agent_kit::observation::SOURCE_LIMIT_BYTES;
    for (mode, expected_open) in [
        (ObservationMode::Stable, Some("open")),
        (ObservationMode::OpenUnknown, None),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(
            fixture.project.join("scripts/subject.gd"),
            vec![b'z'; SOURCE_LIMIT_BYTES + 1],
        )
        .unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        let snapshot = &result["snapshot"];
        assert_eq!(result["outcome"], "limited_observation", "{mode:?}");
        assert_eq!(
            snapshot["document"]["open_state"]["value"].as_str(),
            expected_open
        );
        assert_eq!(snapshot["sources"]["D"]["reason"]["code"], "too_large");
        assert!(snapshot["sources"]["D"]["text"].is_null());
        assert_eq!(snapshot["sources"]["R"]["text"], "\u{feff}x\r\n");
        if mode == ObservationMode::OpenUnknown {
            assert_eq!(
                snapshot["sources"]["B"]["reason"]["code"],
                "open_state_unknown"
            );
            assert_eq!(snapshot["dirty"]["state"], "unknown");
        } else {
            assert_eq!(snapshot["sources"]["B"]["text"], "x\n");
            assert_eq!(snapshot["dirty"]["state"], "dirty");
        }
        peer.join().unwrap();
    }
}

#[test]
fn builtin_identity_never_reads_container_and_absent_identity_is_unsupported() {
    let locator = "res://scripts/scene.tscn::GDScript_abc";
    for (mode, expected) in [
        (ObservationMode::Builtin, "limited_observation"),
        (ObservationMode::BuiltinUnknown, "unsupported_observation"),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(
            fixture.project.join("scripts/scene.tscn"),
            "PRIVATE_CONTAINER_SOURCE",
        )
        .unwrap();
        let peer = attach(&fixture, ID1, Mode::ObservationAt(locator, mode));
        let output = invoke_at(&fixture, locator);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], expected, "{mode:?} {result}");
        if mode == ObservationMode::Builtin {
            assert_eq!(
                result["snapshot"]["document"]["identity"]["kind"],
                "builtin_gdscript"
            );
            assert_eq!(
                result["snapshot"]["sources"]["D"]["reason"]["code"],
                "no_standalone_disk_source"
            );
            assert_eq!(result["snapshot"]["sources"]["R"]["text"], "\u{feff}x\r\n");
            assert_eq!(result["snapshot"]["sources"]["B"]["text"], "x\n");
        } else {
            assert!(result["snapshot"].is_null());
        }
        assert!(!String::from_utf8_lossy(&output.stdout).contains("PRIVATE_CONTAINER_SOURCE"));
        peer.join().unwrap();
    }
}

#[test]
fn utf8_limit_at_exact_boundary_is_observed_per_editor_source() {
    use godot_agent_kit::observation::{Availability, SOURCE_LIMIT_BYTES};
    let exact = "λ".repeat(SOURCE_LIMIT_BYTES / 2);
    for (mode, source) in [
        (ObservationMode::ExactResource, "R"),
        (ObservationMode::ExactBuffer, "B"),
    ] {
        let fixture = Fixture::new();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
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
        let observed = if source == "R" {
            &sample.resource
        } else {
            &sample.buffer
        };
        assert_eq!(observed.availability(), Availability::Observed);
        assert_eq!(observed.text(), Some(exact.as_str()));
        assert_eq!(observed.text().unwrap().len(), SOURCE_LIMIT_BYTES);
        assert!(observed.collection().is_some());
        drop(selected);
        peer.join().unwrap();
    }
}

#[test]
fn closed_disk_and_resource_limits_are_independent_of_not_open_status() {
    use godot_agent_kit::observation::SOURCE_LIMIT_BYTES;
    for (mode, disk_bytes, expected_disk, expected_resource) in [
        (
            ObservationMode::ClosedCached,
            SOURCE_LIMIT_BYTES,
            "observed",
            "observed",
        ),
        (
            ObservationMode::ClosedCached,
            SOURCE_LIMIT_BYTES + 1,
            "too_large",
            "observed",
        ),
        (
            ObservationMode::ExcessClosedResource,
            SOURCE_LIMIT_BYTES,
            "observed",
            "too_large",
        ),
        (
            ObservationMode::ClosedUnloaded,
            SOURCE_LIMIT_BYTES + 1,
            "too_large",
            "resource_not_loaded",
        ),
        (
            ObservationMode::ClosedUnloadedUnknown,
            SOURCE_LIMIT_BYTES + 1,
            "too_large",
            "resource_not_loaded",
        ),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(
            fixture.project.join("scripts/subject.gd"),
            format!(
                "{}{}",
                "λ".repeat(SOURCE_LIMIT_BYTES / 2),
                if disk_bytes > SOURCE_LIMIT_BYTES {
                    "!"
                } else {
                    ""
                }
            ),
        )
        .unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], "not_open", "{mode:?}");
        let snapshot = &result["snapshot"];
        assert_eq!(
            snapshot["sources"]["D"]["availability"],
            if expected_disk == "observed" {
                "observed"
            } else {
                "unavailable"
            }
        );
        assert_eq!(
            snapshot["sources"]["D"]["reason"]["code"],
            if expected_disk == "observed" {
                Value::Null
            } else {
                json!(expected_disk)
            }
        );
        if expected_disk == "observed" {
            assert_eq!(
                snapshot["sources"]["D"]["text"].as_str().unwrap().len(),
                SOURCE_LIMIT_BYTES
            );
        } else {
            assert!(snapshot["sources"]["D"]["text"].is_null());
        }
        assert_eq!(
            snapshot["sources"]["R"]["reason"]["code"],
            if expected_resource == "observed" {
                Value::Null
            } else {
                json!(expected_resource)
            }
        );
        assert_eq!(snapshot["sources"]["B"]["availability"], "not_applicable");
        assert_eq!(snapshot["dirty"]["availability"], "not_applicable");
        if mode == ObservationMode::ClosedUnloadedUnknown {
            assert_eq!(snapshot["document"]["validity"]["value"], "valid");
            assert_eq!(
                snapshot["document"]["validity"]["collection"]["clock_id"],
                "caller"
            );
            assert!(snapshot["document"]["identity"]["disk_file_id"]["inode"]
                .as_str()
                .is_some());
            assert!(snapshot["document"]["identity"]["script_instance_id"].is_null());
            assert!(snapshot["sources"]["D"]["witness"].is_null());
        }
        peer.join().unwrap();
    }
}

#[test]
fn closed_unloaded_invalid_utf8_disk_still_has_verified_identity_without_d_text() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    fs::write(fixture.project.join("scripts/subject.gd"), [0xff]).unwrap();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::ClosedUnloadedUnknown),
    );
    let output = invoke(&fixture, &fixture.project, None);
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "not_open");
    assert_eq!(result["snapshot"]["document"]["validity"]["value"], "valid");
    assert_eq!(
        result["snapshot"]["sources"]["D"]["reason"]["code"],
        "invalid_utf8"
    );
    assert!(result["snapshot"]["sources"]["D"]["text"].is_null());
    assert_eq!(
        result["snapshot"]["sources"]["R"]["reason"]["code"],
        "resource_not_loaded"
    );
    assert_eq!(
        result["snapshot"]["sources"]["B"]["availability"],
        "not_applicable"
    );
    peer.join().unwrap();
}

#[test]
fn closed_oversized_disk_identity_change_invalidates_only_stale_disk_facts() {
    use godot_agent_kit::observation::SOURCE_LIMIT_BYTES;
    let fixture = Fixture::new();
    fs::create_dir(fixture.project.join("scripts")).unwrap();
    fs::write(
        fixture.project.join("scripts/subject.gd"),
        "λ".repeat(SOURCE_LIMIT_BYTES / 2) + "!",
    )
    .unwrap();
    let peer = attach(
        &fixture,
        ID1,
        Mode::Observation(ObservationMode::ClosedReplaceDisk),
    );
    let output = invoke(&fixture, &fixture.project, None);
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"], "limited_observation");
    let snapshot = &result["snapshot"];
    assert_eq!(
        snapshot["sources"]["D"]["reason"]["code"],
        "identity_changed"
    );
    assert_eq!(
        snapshot["document"]["validity"]["reason"]["code"],
        "identity_changed"
    );
    assert_eq!(
        snapshot["document"]["validity"]["invalidated_evidence"]["value"],
        "valid"
    );
    assert_eq!(snapshot["document"]["open_state"]["value"], "not_open");
    assert_eq!(
        snapshot["sources"]["R"]["reason"]["code"],
        "resource_not_loaded"
    );
    assert_eq!(
        snapshot["consistency"]["detected_changes"][0]["code"],
        "identity_changed"
    );
    peer.join().unwrap();
}

#[test]
fn closed_disk_limit_never_outweighs_denial_known_disconnect_or_deadline() {
    use godot_agent_kit::observation::SOURCE_LIMIT_BYTES;
    for (mode, expected) in [
        (ObservationMode::ClosedDenyRecheck, "denied_access"),
        (
            ObservationMode::ClosedDisconnectRecheck,
            "disconnected_editor",
        ),
        (ObservationMode::ClosedSilentRecheck, "timeout"),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.project.join("scripts")).unwrap();
        fs::write(
            fixture.project.join("scripts/subject.gd"),
            "λ".repeat(SOURCE_LIMIT_BYTES / 2) + "!",
        )
        .unwrap();
        let peer = attach(&fixture, ID1, Mode::Observation(mode));
        let output = invoke(&fixture, &fixture.project, None);
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["outcome"], expected, "{mode:?}");
        assert_ne!(result["outcome"], "not_open");
        if expected == "denied_access" {
            assert!(result["snapshot"].is_null());
            assert!(result["resolved_target"].is_null());
        } else {
            assert_eq!(
                result["snapshot"]["sources"]["D"]["reason"]["code"],
                "too_large"
            );
            assert_eq!(
                result["snapshot"]["sources"]["R"]["reason"]["code"],
                if expected == "disconnected_editor" {
                    "session_ended"
                } else {
                    "resource_not_loaded"
                }
            );
            assert_eq!(result["snapshot"]["consistency"]["checks"], "unavailable");
        }
        assert!(output.stderr.is_empty());
        assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
        peer.join().unwrap();
    }
}

#[test]
fn ipc_events_validate_target_identity_and_reject_claimed_success() {
    use godot_agent_kit::observation::{
        ClockId, CollectionStamp, DecimalCounter, DetectedChange, Diagnostic, EngineVersion,
        FileIdentity, Recheck, RecheckReason, SourceReason, Stage, Surface,
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
    let file = FileIdentity::new(
        DecimalCounter::new("22").unwrap(),
        DecimalCounter::new("33").unwrap(),
    );
    let stamp = CollectionStamp::new(
        ClockId::Caller,
        DecimalCounter::new("0").unwrap(),
        DecimalCounter::new("1").unwrap(),
        1,
    )
    .unwrap();
    let limited_disk = wire::encode_event(
        &Event::DiskMetadata {
            reason: SourceReason::TooLarge,
            file: Some(file.clone()),
            collection: stamp,
        },
        request.request_id(),
    )
    .unwrap();
    assert!(matches!(
        wire::decode_event(&limited_disk, &request, Some(&target), 2).unwrap(),
        Event::DiskMetadata { reason: SourceReason::TooLarge, file: Some(id), .. }
            if id == file
    ));
    let mut counterfeit: Value = serde_json::from_slice(&limited_disk).unwrap();
    counterfeit["disk"]["reason"]["code"] = json!("disk_missing");
    counterfeit["disk"]["reason"]["action"] = json!(SourceReason::DiskMissing.action());
    assert_eq!(
        wire::decode_event(
            counterfeit.to_string().as_bytes(),
            &request,
            Some(&target),
            2
        )
        .err()
        .unwrap()
        .outcome,
        OutcomeKind::ProtocolError
    );
    let mut counterfeit: Value = serde_json::from_slice(&limited_disk).unwrap();
    counterfeit["file_collection"]["clock_id"] = json!(format!("editor:{ID1}"));
    assert!(wire::decode_event(
        counterfeit.to_string().as_bytes(),
        &request,
        Some(&target),
        2
    )
    .is_err());
    let failed = json!({"v":4,"request_id":request.request_id().as_str(),"kind":"failed",
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
    let event = json!({"v":4,"request_id":request.request_id().as_str(),"kind":"sample","sample":{
        "collection":sample["collection"],"document":sample["document"],"R":sample["R"],"B":sample["B"],"dirty":sample["dirty"],"diagnostics":[]}});
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
fn false_discovery_capability_preserves_caller_observation_and_rechecked_partial_evidence() {
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
fn selected_channel_cannot_be_reused_for_another_request_project_session_or_document() {
    let fixture = Fixture::new();
    let peer = attach(&fixture, ID1, Mode::Valid);
    let mut selected = fixture.resolve(None, Duration::from_secs(2)).unwrap();
    for (id, root, session, script) in [
        (
            "another_request",
            fixture.project.to_str().unwrap(),
            None,
            "res://scripts/subject.gd",
        ),
        (
            "try_1",
            "/different_project",
            None,
            "res://scripts/subject.gd",
        ),
        (
            "try_1",
            fixture.project.to_str().unwrap(),
            Some(ID2),
            "res://scripts/subject.gd",
        ),
        (
            "try_1",
            fixture.project.to_str().unwrap(),
            None,
            "res://scripts/other.gd",
        ),
    ] {
        let request = ObservationRequest::new(
            RequestId::new(id).unwrap(),
            ProjectRoot::new(root).unwrap(),
            session.map(|value| SessionId::new(value).unwrap()),
            ResourcePath::new(script).unwrap(),
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

#[test]
fn native_capability_and_build_are_authenticated_before_selection() {
    let fixture = Fixture::new();
    let peer = attach(&fixture, ID1, Mode::ValidNative);
    let selected = fixture.resolve(Some(ID1), Duration::from_secs(2)).unwrap();
    assert!(selected.capabilities().edit_open_gdscript);
    assert!(!selected.capabilities().discover_gdscripts);
    assert!(!selected.capabilities().open_gdscript);
    drop(selected);
    peer.join().unwrap();
    for mode in [
        Mode::ChangedNativeBuild,
        Mode::ChangedCapability("observe_gdscript"),
        Mode::ChangedCapability("open_enumeration"),
        Mode::ChangedCapability("buffer_attribution"),
        Mode::ChangedCapability("unsaved_paths"),
        Mode::ChangedCapability("cached_resource_lookup"),
        Mode::ChangedCapability("edit_open_gdscript"),
        Mode::ChangedCapability("open_gdscript"),
        Mode::ChangedCapability("discover_gdscripts"),
        Mode::MissingDiscovery,
        Mode::ExtraCapability,
        Mode::WrongDiscoveryType,
        Mode::DuplicateDiscovery,
        Mode::ChangedIdentity("session_id"),
        Mode::ChangedIdentity("project_root"),
        Mode::ChangedIdentity("godot_version"),
        Mode::ChangedIdentity("engine_hash"),
        Mode::ChangedIdentity("client_nonce"),
        Mode::ChangedIdentity("server_nonce"),
        Mode::ReflectedFinish,
        Mode::ChangedFinishCapability,
        Mode::OldHandshake,
        Mode::ChangedNativeRevision,
        Mode::ChangedFinishBuild,
    ] {
        let fixture = Fixture::new();
        let peer = attach(&fixture, ID1, mode);
        let failure = fixture
            .resolve(Some(ID1), Duration::from_secs(2))
            .unwrap_err();
        assert!(matches!(
            failure.outcome,
            OutcomeKind::DeniedAccess | OutcomeKind::ProtocolError
        ));
        assert!(failure.selection.is_none());
        peer.join().unwrap();
    }
}

#[test]
fn old_private_descriptor_cannot_negotiate_a_downgrade() {
    let fixture = Fixture::new();
    let socket = listener();
    fixture.descriptor(ID1, socket.local_addr().unwrap().port());
    let path = fixture.registry.join(format!("{ID1}.json"));
    let mut descriptor: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    descriptor["v"] = json!(3);
    fs::write(path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
    let failure = fixture
        .resolve(Some(ID1), Duration::from_secs(2))
        .unwrap_err();
    assert_eq!(failure.outcome, OutcomeKind::ProtocolError);
    socket.set_nonblocking(true).unwrap();
    assert_eq!(
        socket.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[path = "bridge_boundary/open.rs"]
mod script_open;
