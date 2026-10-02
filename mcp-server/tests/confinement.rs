use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(target_os = "macos")]
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{symlink, DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use godot_agent_kit::observation::{
    Authority, Availability, DetectedChange, DiagnosticCode, ObservationRequest, OutcomeKind,
    ProjectRoot, RequestId, ResourcePath, SourceReason, Stage, SOURCE_LIMIT_BYTES,
};
use godot_agent_kit::{project_fs, target};
use ring::hmac;
use serde_json::{json, Value};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    home: PathBuf,
    project: PathBuf,
    registry: PathBuf,
    listener: TcpListener,
}
impl Fixture {
    fn new() -> Self {
        let base = fs::canonicalize(std::env::temp_dir()).unwrap();
        let home = base.join(format!(
            "gak-confinement-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&home).unwrap();
        let project = home.join("project");
        DirBuilder::new().mode(0o700).create(&project).unwrap();
        let registry = home.join("registry");
        project_fs::init_registry(&registry).unwrap();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        assert!((49152..=65535).contains(&listener.local_addr().unwrap().port()));
        Self {
            home,
            project,
            registry,
            listener,
        }
    }
    fn request(&self, locator: &str) -> ObservationRequest {
        ObservationRequest::new(
            RequestId::new("safe_1").unwrap(),
            ProjectRoot::new(self.project.to_str().unwrap()).unwrap(),
            None,
            ResourcePath::new(locator).unwrap(),
        )
    }
    fn result(&self, locator: &str) -> Result<target::SelectedSession, target::RoutingFailure> {
        target::resolve(
            &self.request(locator),
            &self.registry,
            Instant::now() + Duration::from_millis(300),
        )
    }
    fn descriptor(&self, id: &str, bytes: &[u8]) {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.registry.join(format!("{id}.json")))
            .unwrap();
        file.write_all(bytes).unwrap();
    }
    fn valid_descriptor(&self, id: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"v":5,"session_id":id,"project_root":self.project,"godot_version":"4.7.2.stable.official.ed1daf0bf","engine_hash":"ed1daf0bf001b61586d9930840f2f1394092c079","host":"127.0.0.1","port":self.listener.local_addr().unwrap().port(),"token":"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"})).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.home).unwrap();
    }
}
const ID: &str = "00112233445566778899aabbccddeeff";

const VERSION: &str = "4.7.2.stable.official.ed1daf0bf";
const HASH: &str = "ed1daf0bf001b61586d9930840f2f1394092c079";
const SECRET: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const SERVER_NONCE: &str = "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f";

fn frame(stream: &mut TcpStream, value: &Value) {
    let bytes = serde_json::to_vec(value).unwrap();
    stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&bytes).unwrap();
}

fn receive(stream: &mut TcpStream) -> Value {
    let mut length = [0; 4];
    stream.read_exact(&mut length).unwrap();
    let mut bytes = vec![0; u32::from_be_bytes(length) as usize];
    assert!(bytes.len() <= 4096);
    stream.read_exact(&mut bytes).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn decoded(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn field(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value);
}

fn proof(role: &[u8], hello: &Value, capabilities: &Value) -> String {
    let mut transcript = Vec::new();
    field(&mut transcript, role);
    field(&mut transcript, b"godot-agent-kit/editor-bridge/v5");
    field(&mut transcript, hello[2].as_str().unwrap().as_bytes());
    field(&mut transcript, &decoded(hello[3].as_str().unwrap()));
    field(&mut transcript, hello[4].as_str().unwrap().as_bytes());
    field(&mut transcript, VERSION.as_bytes());
    field(&mut transcript, HASH.as_bytes());
    for name in [
        "observe_gdscript",
        "open_enumeration",
        "buffer_attribution",
        "unsaved_paths",
        "cached_resource_lookup",
        "edit_open_gdscript",
        "open_gdscript",
        "discover_gdscripts",
        "close_gdscript",
    ] {
        field(
            &mut transcript,
            &[u8::from(capabilities[name].as_bool().unwrap())],
        );
    }
    field(&mut transcript, &0u32.to_be_bytes());
    field(&mut transcript, b"");
    field(&mut transcript, &decoded(hello[5].as_str().unwrap()));
    field(&mut transcript, &decoded(SERVER_NONCE));
    let key = hmac::Key::new(hmac::HMAC_SHA256, &decoded(SECRET));
    hmac::sign(&key, &transcript)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// The real selection gate is exercised once per fixture, not bypassed with a
// forged SelectedSession. The synthetic peer proves only authentication, not R/B.
fn select(f: &Fixture, locator: &str) -> (target::SelectedSession, thread::JoinHandle<()>) {
    f.descriptor(ID, &f.valid_descriptor(ID));
    let listener = f.listener.try_clone().unwrap();
    let peer = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let hello = receive(&mut socket);
        let capabilities = json!({"observe_gdscript":false,"open_enumeration":true,"buffer_attribution":false,"unsaved_paths":false,"cached_resource_lookup":false,"edit_open_gdscript":false,"open_gdscript":false,"discover_gdscripts":false,"close_gdscript":false});
        frame(
            &mut socket,
            &json!({
                "v":5,"kind":"challenge","request_id":hello[2],"session_id":hello[3],
                "project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,
                "capabilities":capabilities,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,
                "native_api_revision":0,"native_build_id":"",
                "server_proof":proof(b"server",&hello,&capabilities)
            }),
        );
        let authenticate = receive(&mut socket);
        assert_eq!(authenticate[1], "authenticate");
        assert_eq!(authenticate[7], proof(b"client", &hello, &capabilities));
        frame(
            &mut socket,
            &json!({
                "v":5,"kind":"hello","request_id":hello[2],"session_id":hello[3],
                "project_root":hello[4],"godot_version":VERSION,"engine_hash":HASH,
                "capabilities":capabilities,"client_nonce":hello[5],"server_nonce":SERVER_NONCE,
                "native_api_revision":0,"native_build_id":"",
                "finish_proof":proof(b"finish",&hello,&capabilities)
            }),
        );
        let mut byte = [0];
        assert!(matches!(socket.read(&mut byte), Ok(0) | Err(_)));
    });
    let selected = target::resolve(
        &f.request(locator),
        &f.registry,
        Instant::now() + Duration::from_secs(2),
    )
    .unwrap();
    (selected, peer)
}
#[cfg(target_os = "macos")]
fn acl_listing(path: &Path) -> String {
    let output = Command::new("/bin/ls")
        .arg("-lde")
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[cfg(target_os = "macos")]
fn grant_acl(path: &Path, entry: &str, mode: u32) -> String {
    let before_mode = fs::metadata(path).unwrap().mode() & 0o7777;
    assert_eq!(before_mode, mode);
    let status = Command::new("/bin/chmod")
        .arg("+a")
        .arg(entry)
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(fs::metadata(path).unwrap().mode() & 0o7777, mode);
    acl_listing(path)
}

#[cfg(target_os = "macos")]
#[test]
fn access_expanding_acl_on_registry_is_rejected_without_repair() {
    let f = Fixture::new();
    let acl = grant_acl(
        &f.registry,
        "everyone allow read,search,readattr,readextattr,readsecurity",
        0o700,
    );
    let error = project_fs::init_registry(&f.registry).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::UnsafeRegistry);
    assert_eq!(acl_listing(&f.registry), acl);
}

#[cfg(target_os = "macos")]
#[test]
fn access_expanding_acl_on_registry_ancestor_is_rejected_without_repair() {
    let f = Fixture::new();
    let acl = grant_acl(
        &f.home,
        "everyone allow read,search,readattr,readextattr,readsecurity",
        0o700,
    );
    let error = project_fs::init_registry(&f.registry).err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::UnsafeRegistry);
    assert_eq!(acl_listing(&f.home), acl);
}

#[cfg(target_os = "macos")]
#[test]
fn access_expanding_acl_on_descriptor_is_rejected_without_repair() {
    let f = Fixture::new();
    f.descriptor(ID, &f.valid_descriptor(ID));
    let path = f.registry.join(format!("{ID}.json"));
    let acl = grant_acl(
        &path,
        "everyone allow read,readattr,readextattr,readsecurity",
        0o600,
    );
    let error = f.result("res://safe.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::UnsafeRegistry);
    assert_eq!(acl_listing(&path), acl);
}

#[cfg(target_os = "macos")]
#[test]
fn inherited_allow_acl_on_descriptor_is_rejected() {
    let f = Fixture::new();
    f.descriptor(ID, &f.valid_descriptor(ID));
    let path = f.registry.join(format!("{ID}.json"));
    assert!(Command::new("/bin/chmod")
        .arg("+ai")
        .arg("everyone allow read")
        .arg(&path)
        .status()
        .unwrap()
        .success());
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o7777, 0o600);
    let acl = acl_listing(&path);
    assert!(acl.contains("inherited allow read"));
    let error = f.result("res://safe.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::UnsafeRegistry);
    assert_eq!(acl_listing(&path), acl);
}

#[cfg(target_os = "macos")]
#[test]
fn deny_only_acl_on_registry_ancestor_remains_accepted() {
    let f = Fixture::new();
    let acl = grant_acl(&f.home, "everyone deny delete", 0o700);
    assert_eq!(
        project_fs::init_registry(&f.registry).unwrap(),
        fs::canonicalize(&f.registry).unwrap()
    );
    assert_eq!(acl_listing(&f.home), acl);
    // Test-owned cleanup only; the production verifier must never repair this ACL.
    assert!(Command::new("/bin/chmod")
        .arg("-N")
        .arg(&f.home)
        .status()
        .unwrap()
        .success());
}

#[cfg(target_os = "macos")]
#[test]
fn deny_only_acl_on_descriptor_remains_accepted() {
    let f = Fixture::new();
    f.descriptor(ID, &f.valid_descriptor(ID));
    let path = f.registry.join(format!("{ID}.json"));
    let acl = grant_acl(&path, "everyone deny write", 0o600);
    let listener = f.listener.try_clone().unwrap();
    listener.set_nonblocking(true).unwrap();
    let peer = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    drop(socket);
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "safe descriptor did not reach its owned peer"
                    );
                    std::thread::park_timeout(Duration::from_millis(1));
                }
                Err(error) => panic!("owned peer accept failed: {error}"),
            }
        }
    });
    let error = f.result("res://safe.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::EditorUnavailable);
    assert_eq!(acl_listing(&path), acl);
    peer.join().unwrap();
}

#[test]
fn private_bootstrap_refuses_modes_symlinks_and_unsafe_ancestors() {
    let f = Fixture::new();
    assert_eq!(
        project_fs::init_registry(&f.registry).unwrap(),
        fs::canonicalize(&f.registry).unwrap()
    );
    let aliased = f.home.join("registry-alias");
    symlink(&f.registry, &aliased).unwrap();
    assert_eq!(
        project_fs::init_registry(&aliased)
            .err()
            .unwrap()
            .diagnostic
            .code(),
        DiagnosticCode::UnsafeRegistry
    );
    fs::set_permissions(&f.registry, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        f.result("res://safe.gd").err().unwrap().outcome,
        OutcomeKind::DeniedAccess
    );
    fs::set_permissions(&f.registry, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&f.home, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(
        project_fs::init_registry(&f.registry)
            .err()
            .unwrap()
            .diagnostic
            .code(),
        DiagnosticCode::UnsafeRegistry
    );
    fs::set_permissions(&f.home, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn registry_within_requested_project_is_not_private_state() {
    let f = Fixture::new();
    let nested = f.project.join("metadata");
    project_fs::init_registry(&nested).unwrap();
    let failure = target::resolve(
        &f.request("res://safe.gd"),
        &nested,
        Instant::now() + Duration::from_millis(300),
    )
    .err()
    .unwrap();
    assert_eq!(failure.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(failure.diagnostic.code(), DiagnosticCode::UnsafeRegistry);
}

#[test]
fn bad_descriptors_never_route_or_leak_payloads() {
    for failure in ["symlink", "mode", "oversized", "unknown", "duplicate", "v3"] {
        let f = Fixture::new();
        let path = f.registry.join(format!("{ID}.json"));
        match failure {
            "symlink" => {
                symlink(f.home.join("outside"), &path).unwrap();
            }
            "mode" => {
                f.descriptor(ID, &f.valid_descriptor(ID));
                fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            }
            "oversized" => {
                f.descriptor(ID, &vec![b'Z'; 4097]);
            }
            "unknown" => {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&f.valid_descriptor(ID)).unwrap();
                value["source"] = json!("SYNTHETIC_SOURCE_SENTINEL");
                f.descriptor(ID, &serde_json::to_vec(&value).unwrap());
            }
            "duplicate" => {
                let bytes = f.valid_descriptor(ID);
                let mut value = String::from_utf8(bytes).unwrap();
                value.pop();
                value.push_str(",\"token\":\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"}");
                f.descriptor(ID, value.as_bytes());
            }
            "v3" => {
                let mut value: Value = serde_json::from_slice(&f.valid_descriptor(ID)).unwrap();
                value["v"] = json!(3);
                f.descriptor(ID, &serde_json::to_vec(&value).unwrap());
            }
            _ => unreachable!(),
        }
        let result = f.result("res://safe.gd").err().unwrap();
        assert!(matches!(
            result.outcome,
            OutcomeKind::DeniedAccess | OutcomeKind::ProtocolError
        ));
        let diagnostic = format!("{result:?} {result}");
        assert!(!diagnostic.contains("SYNTHETIC_SOURCE_SENTINEL"));
        assert!(!diagnostic.contains("0001020304050607"));
        f.listener.set_nonblocking(true).unwrap();
        assert_eq!(
            f.listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
}

#[test]
fn excess_descriptors_do_not_select_a_convenient_subset() {
    let f = Fixture::new();
    for i in 0..33 {
        let id = format!("{i:032x}");
        f.descriptor(&id, &f.valid_descriptor(&id));
    }
    assert_eq!(
        f.result("res://safe.gd").err().unwrap().outcome,
        OutcomeKind::UnsupportedObservation
    );
}

#[test]
fn locator_validation_refuses_symlink_escapes_and_preserves_missing_source() {
    let f = Fixture::new();
    let outside = f.home.join("outside.gd");
    fs::write(&outside, b"SYNTHETIC_SOURCE_SENTINEL").unwrap();
    symlink(&outside, f.project.join("outside.gd")).unwrap();
    assert_eq!(
        f.result("res://outside.gd")
            .err()
            .unwrap()
            .diagnostic
            .code(),
        DiagnosticCode::OutOfProject
    );
    fs::create_dir(f.project.join("scripts")).unwrap();
    symlink(&f.home, f.project.join("scripts/link")).unwrap();
    assert_eq!(
        f.result("res://scripts/link/outside.gd")
            .err()
            .unwrap()
            .outcome,
        OutcomeKind::DeniedAccess
    );
    assert_eq!(
        f.result("res://missing.gd").err().unwrap().outcome,
        OutcomeKind::EditorUnavailable
    );
    assert_eq!(
        f.result("res://missing.tscn::GDScript_abc")
            .err()
            .unwrap()
            .outcome,
        OutcomeKind::EditorUnavailable
    );
    for invalid in [
        "/tmp/source.gd",
        "user://source.gd",
        "res://../outside.gd",
        "res://scripts//x.gd",
        "res://x.gd?fake",
        "res://x\\y.gd",
    ] {
        assert!(
            ResourcePath::new(invalid).is_err(),
            "invalid resource locator"
        );
    }
}

#[test]
fn cross_user_writable_project_root_or_locator_component_is_refused() {
    let f = Fixture::new();
    fs::set_permissions(&f.project, fs::Permissions::from_mode(0o777)).unwrap();
    let error = f.result("res://missing.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
    fs::set_permissions(&f.project, fs::Permissions::from_mode(0o700)).unwrap();

    let scripts = f.project.join("scripts");
    DirBuilder::new().mode(0o700).create(&scripts).unwrap();
    fs::set_permissions(&scripts, fs::Permissions::from_mode(0o777)).unwrap();
    let error = f.result("res://scripts/missing.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
}

#[cfg(target_os = "macos")]
#[test]
fn acl_grant_on_project_root_refuses_cross_user_writable_identity() {
    let f = Fixture::new();
    let acl = grant_acl(
        &f.project,
        "everyone allow write,append,add_file,add_subdirectory,delete_child",
        0o700,
    );
    let error = f.result("res://missing.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
    assert_eq!(acl_listing(&f.project), acl);
}

#[cfg(target_os = "macos")]
#[test]
fn acl_grant_on_locator_directory_refuses_unsafe_component() {
    let f = Fixture::new();
    let scripts = f.project.join("scripts");
    DirBuilder::new().mode(0o700).create(&scripts).unwrap();
    let acl = grant_acl(
        &scripts,
        "everyone allow add_file,add_subdirectory,delete_child,search",
        0o700,
    );
    let error = f.result("res://scripts/missing.gd").err().unwrap();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
    assert_eq!(acl_listing(&scripts), acl);
}

#[test]
fn bootstrap_accepts_only_private_owner_directory() {
    let f = Fixture::new();
    let file = f.home.join("regular");
    fs::write(&file, b"x").unwrap();
    assert!(project_fs::init_registry(&file).is_err());
    assert!(project_fs::init_registry(Path::new("relative/registry")).is_err());
}

#[test]
fn selected_disk_reads_preserve_exact_text_and_independent_limits() {
    let f = Fixture::new();
    let path = f.project.join("subject.gd");
    fs::write(&path, b"").unwrap();
    let (selected, peer) = select(&f, "res://subject.gd");
    let started = Instant::now();
    let empty = project_fs::read_disk(&selected, started).unwrap();
    assert_eq!(empty.authority(), Authority::D);
    assert_eq!(empty.availability(), Availability::Observed);
    assert_eq!(empty.text(), Some(""));
    assert!(empty.collection().is_some());
    assert!(empty.witness().unwrap().disk_file_id().is_some());
    assert_eq!(
        empty.collection().unwrap().clock_id(),
        &godot_agent_kit::observation::ClockId::Caller
    );

    let exact = "\u{feff}first\r\nsecond\n  \r\n";
    fs::write(&path, exact.as_bytes()).unwrap();
    let text = project_fs::read_disk(&selected, started).unwrap();
    assert_eq!(text.text(), Some(exact));
    assert_eq!(
        project_fs::recheck_disk(&selected, &text, started).unwrap(),
        Vec::<DetectedChange>::new()
    );

    fs::write(&path, vec![b'x'; SOURCE_LIMIT_BYTES]).unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started)
            .unwrap()
            .text()
            .unwrap()
            .len(),
        SOURCE_LIMIT_BYTES
    );
    fs::write(&path, vec![b'x'; SOURCE_LIMIT_BYTES + 1]).unwrap();
    let too_large = project_fs::read_disk(&selected, started).unwrap();
    assert_eq!(too_large.reason(), Some(SourceReason::TooLarge));
    assert!(too_large.text().is_none());
    assert_eq!(too_large.collection(), None);
    assert_eq!(too_large.witness(), None);
    assert!(project_fs::recheck_disk(&selected, &too_large, started)
        .unwrap()
        .is_empty());

    fs::write(&path, b"\xff").unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started).unwrap().reason(),
        Some(SourceReason::InvalidUtf8)
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn selected_disk_recheck_detects_content_identity_and_availability_transitions() {
    let f = Fixture::new();
    let path = f.project.join("subject.gd");
    fs::write(&path, b"before").unwrap();
    let (selected, peer) = select(&f, "res://subject.gd");
    let started = Instant::now();
    let initial = project_fs::read_disk(&selected, started).unwrap();
    fs::write(&path, b"after").unwrap();
    assert_eq!(
        project_fs::recheck_disk(&selected, &initial, started).unwrap(),
        vec![DetectedChange::Source(Authority::D)]
    );
    let current = project_fs::read_disk(&selected, started).unwrap();
    fs::write(f.project.join("replacement"), b"after").unwrap();
    fs::rename(f.project.join("replacement"), &path).unwrap();
    assert_eq!(
        project_fs::recheck_disk(&selected, &current, started).unwrap(),
        vec![DetectedChange::DiskIdentityReplaced]
    );
    let current = project_fs::read_disk(&selected, started).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(
        project_fs::recheck_disk(&selected, &current, started).unwrap(),
        vec![DetectedChange::Source(Authority::D)]
    );
    let missing = project_fs::read_disk(&selected, started).unwrap();
    assert_eq!(missing.reason(), Some(SourceReason::DiskMissing));
    assert!(project_fs::recheck_disk(&selected, &missing, started)
        .unwrap()
        .is_empty());
    fs::write(&path, b"new").unwrap();
    assert_eq!(
        project_fs::recheck_disk(&selected, &missing, started).unwrap(),
        vec![DetectedChange::Source(Authority::D)]
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn selected_disk_refuses_post_selection_symlinks_and_writable_ancestors() {
    let f = Fixture::new();
    let scripts = f.project.join("scripts");
    DirBuilder::new().mode(0o700).create(&scripts).unwrap();
    fs::write(scripts.join("subject.gd"), b"safe").unwrap();
    let outside = f.home.join("outside.gd");
    fs::write(&outside, b"PRIVATE_SOURCE_SENTINEL").unwrap();
    let (selected, peer) = select(&f, "res://scripts/subject.gd");
    let started = Instant::now();
    let initial = project_fs::read_disk(&selected, started).unwrap();
    fs::remove_file(scripts.join("subject.gd")).unwrap();
    symlink(&outside, scripts.join("subject.gd")).unwrap();
    let error = project_fs::read_disk(&selected, started).unwrap_err();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert_eq!(error.diagnostic.stage(), Stage::ReadDisk);
    assert_eq!(error.diagnostic.code(), DiagnosticCode::OutOfProject);
    assert!(!format!("{error:?}").contains("PRIVATE_SOURCE_SENTINEL"));
    assert_eq!(
        project_fs::recheck_disk(&selected, &initial, started)
            .unwrap_err()
            .outcome,
        OutcomeKind::DeniedAccess
    );

    fs::remove_file(scripts.join("subject.gd")).unwrap();
    fs::write(scripts.join("subject.gd"), b"safe").unwrap();
    fs::set_permissions(&scripts, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started)
            .unwrap_err()
            .outcome,
        OutcomeKind::DeniedAccess
    );
    fs::set_permissions(&scripts, fs::Permissions::from_mode(0o700)).unwrap();
    let moved = f.home.join("original-scripts");
    fs::rename(&scripts, &moved).unwrap();
    symlink(&f.home, &scripts).unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started)
            .unwrap_err()
            .outcome,
        OutcomeKind::DeniedAccess
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn selected_disk_handles_nonregular_unreadable_and_non_script_locators() {
    let f = Fixture::new();
    let path = f.project.join("subject.gd");
    fs::write(&path, b"source").unwrap();
    let (selected, peer) = select(&f, "res://subject.gd");
    let started = Instant::now();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started).unwrap().reason(),
        Some(SourceReason::DiskUnreadable)
    );
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert_eq!(
        project_fs::read_disk(&selected, started).unwrap().reason(),
        Some(SourceReason::DiskUnreadable)
    );
    drop(selected);
    peer.join().unwrap();

    let built_in = Fixture::new();
    fs::write(
        built_in.project.join("level.tscn"),
        b"PRIVATE_CONTAINER_SENTINEL",
    )
    .unwrap();
    let (selected, peer) = select(&built_in, "res://level.tscn::GDScript_abc");
    let disk = project_fs::read_disk(&selected, Instant::now()).unwrap();
    assert_eq!(disk.reason(), Some(SourceReason::NoStandaloneDiskSource));
    assert_eq!(disk.text(), None);
    assert!(project_fs::recheck_disk(&selected, &disk, Instant::now())
        .unwrap()
        .is_empty());
    drop(selected);
    peer.join().unwrap();

    let unsupported = Fixture::new();
    fs::write(
        unsupported.project.join("notes.txt"),
        b"PRIVATE_TEXT_SENTINEL",
    )
    .unwrap();
    let (selected, peer) = select(&unsupported, "res://notes.txt");
    let error = project_fs::read_disk(&selected, Instant::now()).unwrap_err();
    assert_eq!(error.outcome, OutcomeKind::UnsupportedObservation);
    assert!(!format!("{error:?}").contains("PRIVATE_TEXT_SENTINEL"));
    drop(selected);
    peer.join().unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn selected_disk_refuses_acl_added_after_authentication() {
    let f = Fixture::new();
    let path = f.project.join("subject.gd");
    fs::write(&path, b"PRIVATE_SOURCE_SENTINEL").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let (selected, peer) = select(&f, "res://subject.gd");
    let acl = grant_acl(&path, "everyone allow read", 0o600);
    let error = project_fs::read_disk(&selected, Instant::now()).unwrap_err();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert!(!format!("{error:?}").contains("PRIVATE_SOURCE_SENTINEL"));
    assert_eq!(acl_listing(&path), acl);
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn selected_disk_rejects_project_root_rebinding_without_reading_replacement() {
    let f = Fixture::new();
    fs::write(f.project.join("subject.gd"), b"original").unwrap();
    let (selected, peer) = select(&f, "res://subject.gd");
    let started = Instant::now();
    let original = project_fs::read_disk(&selected, started).unwrap();
    let moved = f.home.join("formerly-selected-project");
    fs::rename(&f.project, &moved).unwrap();
    DirBuilder::new().mode(0o700).create(&f.project).unwrap();
    fs::write(
        f.project.join("subject.gd"),
        b"PRIVATE_REPLACEMENT_SENTINEL",
    )
    .unwrap();
    let error = project_fs::read_disk(&selected, started).unwrap_err();
    assert_eq!(error.outcome, OutcomeKind::DeniedAccess);
    assert!(!format!("{error:?}").contains("PRIVATE_REPLACEMENT_SENTINEL"));
    assert_eq!(
        project_fs::recheck_disk(&selected, &original, started)
            .unwrap_err()
            .outcome,
        OutcomeKind::DeniedAccess
    );
    drop(selected);
    peer.join().unwrap();
}

#[test]
fn opening_caller_refuses_leaf_and_directory_symlinks_before_authentication() {
    for directory in [false, true] {
        let f = Fixture::new();
        let outside = f.home.join("outside.gd");
        fs::write(&outside, b"OPEN_OUTSIDE_SOURCE_SENTINEL").unwrap();
        fs::create_dir(f.project.join("scripts")).unwrap();
        let locator = if directory {
            symlink(&f.home, f.project.join("scripts/escape")).unwrap();
            "res://scripts/escape/outside.gd"
        } else {
            symlink(&outside, f.project.join("scripts/escape.gd")).unwrap();
            "res://scripts/escape.gd"
        };
        f.descriptor(ID, &f.valid_descriptor(ID));
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_open-gdscript"))
            .args([
                "--registry",
                f.registry.to_str().unwrap(),
                "--project",
                f.project.to_str().unwrap(),
                "--session",
                ID,
                "--script",
                locator,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3));
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["reason"], "outside_project");
        assert_eq!(result["application"], "not_applied");
        assert_eq!(result["resolved_target"], Value::Null);
        assert_eq!(result["observation"], Value::Null);
        for bytes in [&output.stdout, &output.stderr] {
            assert!(!String::from_utf8_lossy(bytes).contains("OPEN_OUTSIDE_SOURCE_SENTINEL"));
        }
        f.listener.set_nonblocking(true).unwrap();
        assert_eq!(
            f.listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        assert_eq!(fs::read(&outside).unwrap(), b"OPEN_OUTSIDE_SOURCE_SENTINEL");
    }
}
