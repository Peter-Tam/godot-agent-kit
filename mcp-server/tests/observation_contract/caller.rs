//! Actual-process checks; positive editor observations are deliberately not simulated here.
use std::fs::{self, DirBuilder};
use std::os::unix::fs::DirBuilderExt;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn caller(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_observe-gdscript"))
        .args(args)
        .output()
        .unwrap()
}

fn result(output: &Output, exit: i32, outcome: &str) -> Value {
    assert_eq!(output.status.code(), Some(exit));
    assert!(
        output.stderr.is_empty(),
        "normal outcomes do not need incidental logs"
    );
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["outcome"], outcome);
    assert!(value["request_id"]
        .as_str()
        .is_some_and(|id| !id.is_empty()));
    for field in ["resolved_target", "snapshot", "selection"] {
        assert!(value.get(field).is_some_and(Value::is_null));
    }
    assert!(value["interval"]["started_unix_ms"].is_u64());
    assert!(value["interval"]["finished_unix_ms"].is_u64());
    assert!(value["interval"]["elapsed_us"].as_u64().unwrap() < 5_000_000);
    assert!(value["diagnostics"].as_array().is_some());
    value
}

#[test]
fn malformed_selectors_and_flags_have_correlated_source_free_results() {
    let source_sentinel = "T003_PRIVATE_MALFORMED_ARGUMENT";
    for args in [
        vec![],
        vec![source_sentinel],
        vec![
            "--registry",
            "/missing",
            "--project",
            "/fixture",
            "--script",
        ],
        vec![
            "--registry",
            "/missing",
            "--project",
            "/fixture",
            "--script",
            "res://subject.gd",
            "--project",
            "/second",
        ],
        vec![
            "--registry",
            "/missing",
            "--project",
            "relative",
            "--script",
            "res://subject.gd",
        ],
        vec![
            "--registry",
            "/missing",
            "--project",
            "/fixture",
            "--script",
            "res://../secret.gd",
        ],
        vec![
            "--registry",
            "/missing",
            "--project",
            "/fixture",
            "--script",
            "res://subject.gd",
            "--session",
            "bad",
        ],
        vec!["--help", "--project", "/fixture"],
        vec!["--internal-observation-worker"],
    ] {
        let output = caller(&args);
        result(&output, 3, "invalid_request");
        assert!(!String::from_utf8(output.stdout)
            .unwrap()
            .contains(source_sentinel));
    }
    let first = result(&caller(&[]), 3, "invalid_request");
    let second = result(&caller(&[]), 3, "invalid_request");
    assert_ne!(first["request_id"], second["request_id"]);
}

#[test]
fn actual_bootstrap_and_absent_session_never_read_source_or_start_editor() {
    let root = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!(
            "gak-caller-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    DirBuilder::new().mode(0o700).create(&root).unwrap();
    let project = root.join("project");
    DirBuilder::new().mode(0o700).create(&project).unwrap();
    let script = project.join("subject.gd");
    let source = "extends RefCounted\n# T003_DISK_NOT_AUTHORIZED\n";
    fs::write(&script, source).unwrap();
    let before = fs::metadata(&script).unwrap().modified().unwrap();
    let registry = root.join("registry");
    let bootstrap = caller(&["init-registry", "--registry", registry.to_str().unwrap()]);
    assert_eq!(bootstrap.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&bootstrap.stdout).unwrap()["status"],
        "ready"
    );
    let started = Instant::now();
    let output = caller(&[
        "--registry",
        registry.to_str().unwrap(),
        "--project",
        project.to_str().unwrap(),
        "--script",
        "res://subject.gd",
    ]);
    result(&output, 3, "editor_unavailable");
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_eq!(fs::read_to_string(&script).unwrap(), source);
    assert_eq!(fs::metadata(&script).unwrap().modified().unwrap(), before);
    assert_eq!(fs::read_dir(&registry).unwrap().count(), 0);
    assert!(!String::from_utf8(output.stdout)
        .unwrap()
        .contains("T003_DISK_NOT_AUTHORIZED"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn interrupted_rechecks_preserve_earlier_changes_without_claiming_full_checks() {
    use godot_agent_kit::observation::*;
    let request = super::request(Some(super::SESSION));
    let target = super::target(&request);
    let id = super::identity(Some(OpenState::Open));
    let document = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, super::editor_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, super::editor_stamp("2", 2)),
    );
    let evidence = ObservationEvidence::new(
        target.clone(),
        document,
        Sources::new(
            super::observed(&id, Authority::D, "D"),
            super::observed(&id, Authority::R, "R"),
            super::observed(&id, Authority::B, "old B"),
        )
        .unwrap(),
        DirtyObservation::observed(
            DirtyState::Dirty,
            super::editor_stamp("3", 3),
            super::witness(&id, Authority::B),
        ),
        Recheck::partial(
            RecheckReason::DeadlineExceeded,
            vec![DetectedChange::Source(Authority::B)],
        ),
    );
    let result = ObservationOutcome::classify(
        request,
        super::interval(),
        Some(target),
        Some(evidence),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::Timeout);
    let snapshot = result.snapshot().unwrap();
    assert_eq!(snapshot.consistency().checks(), Checks::Unavailable);
    assert_eq!(snapshot.consistency().stability(), Stability::Changed);
    assert!(snapshot
        .consistency()
        .detected_changes()
        .contains(&DetectedChange::Source(Authority::B)));
    assert_eq!(snapshot.sources().disk().text(), Some("D"));
    assert_eq!(snapshot.sources().resource().text(), Some("R"));
    assert_eq!(snapshot.sources().buffer().text(), None);
    assert_eq!(
        snapshot
            .sources()
            .buffer()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "old B"
    );
    assert_eq!(snapshot.comparisons().disk_buffer(), Comparison::Unknown);
}

#[test]
fn valid_individual_selectors_exceeding_control_budget_are_a_structured_refusal() {
    let root = format!("/{}", "p".repeat(1023));
    let registry = format!("/{}", "r".repeat(1023));
    let script = format!("res://{}.gd", "s".repeat(2039));
    let output = caller(&[
        "--registry",
        &registry,
        "--project",
        &root,
        "--script",
        &script,
    ]);
    result(&output, 3, "invalid_request");
}
