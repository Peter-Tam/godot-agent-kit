use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
#[cfg(target_os = "macos")]
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{symlink, DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use godot_agent_kit::observation::{
    DiagnosticCode, ObservationRequest, OutcomeKind, ProjectRoot, RequestId, ResourcePath,
};
use godot_agent_kit::{project_fs, target};
use serde_json::json;

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
        serde_json::to_vec(&json!({"v":1,"session_id":id,"project_root":self.project,"godot_version":"4.7.2.stable.official.ed1daf0bf","engine_hash":"ed1daf0bf001b61586d9930840f2f1394092c079","host":"127.0.0.1","port":self.listener.local_addr().unwrap().port(),"token":"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"})).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.home).unwrap();
    }
}
const ID: &str = "00112233445566778899aabbccddeeff";
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
    for failure in ["symlink", "mode", "oversized", "unknown", "duplicate"] {
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
