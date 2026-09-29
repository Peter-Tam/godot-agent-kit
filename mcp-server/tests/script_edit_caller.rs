//! Process-level edit stdin and request-boundary regressions. Real-editor stages live in
//! the stock caller acceptance runner; these assert no malformed payload can select an editor.
use godot_agent_kit::bridge::wire;
use godot_agent_kit::observation::*;
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

const SESSION: &str = "00112233445566778899aabbccddeeff";
const GODOT_VERSION: &str = "4.7.2.stable.official.ed1daf0bf";
const ENGINE_HASH: &str = "ed1daf0bf001b61586d9930840f2f1394092c079";
fn decimal(n: u64) -> DecimalCounter {
    DecimalCounter::new(n.to_string()).unwrap()
}
fn stamp(editor: bool, n: u64) -> CollectionStamp {
    CollectionStamp::new(
        if editor {
            ClockId::Editor(SessionId::new(SESSION).unwrap())
        } else {
            ClockId::Caller
        },
        decimal(n),
        decimal(n + 1),
        n + 1,
    )
    .unwrap()
}
fn basis(source: &str) -> Value {
    let request = ObservationRequest::new(
        RequestId::new("basis").unwrap(),
        ProjectRoot::new("/tmp/godot-agent-kit-edit-caller-test").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let target = ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(decimal(21), decimal(22)),
        SessionId::new(SESSION).unwrap(),
        EngineVersion::new(GODOT_VERSION, ENGINE_HASH).unwrap(),
    )
    .unwrap();
    let document = DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        request.script_path().clone(),
        Some(decimal(23)),
        Some(decimal(24)),
        Some(decimal(25)),
        Some(FileIdentity::new(decimal(26), decimal(27))),
    )
    .unwrap();
    let witness = |authority| {
        Witness::new(
            request.script_path().clone(),
            (authority != Authority::D).then(|| decimal(23)),
            (authority == Authority::B).then(|| decimal(24)),
            (authority == Authority::B).then(|| decimal(25)),
            (authority == Authority::D).then(|| FileIdentity::new(decimal(26), decimal(27))),
            (authority == Authority::B).then(|| decimal(28)),
        )
    };
    let observed = |authority, tick| {
        SourceObservation::observed(
            authority,
            source.to_owned(),
            stamp(authority != Authority::D, tick),
            witness(authority),
            Staleness::unknown(),
        )
    };
    let evidence = ObservationEvidence::new(
        target.clone(),
        DocumentState::new(
            Some(document),
            DocumentFact::observed(Validity::Valid, stamp(true, 20)),
            DocumentFact::observed(OpenState::Open, stamp(true, 30)),
        ),
        Sources::new(
            observed(Authority::D, 40),
            observed(Authority::R, 50),
            observed(Authority::B, 60),
        )
        .unwrap(),
        DirtyObservation::observed(DirtyState::Clean, stamp(true, 70), witness(Authority::B)),
        Recheck::performed(Vec::new()),
    );
    let prior = ObservationOutcome::classify(
        request,
        ObservationInterval::new(1, 2, 1000),
        Some(target),
        Some(evidence),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    assert_eq!(prior.outcome(), OutcomeKind::CompleteObservation);
    serde_json::from_slice(&wire::encode_outcome(&prior).unwrap()).unwrap()
}
fn invoke(input: &[u8], extra: &[&str]) -> (i32, Value, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_edit-gdscript"))
        .args([
            "--registry",
            "/tmp/godot-agent-kit-edit-caller-missing-registry",
            "--project",
            "/tmp/godot-agent-kit-edit-caller-test",
            "--session",
            SESSION,
            "--script",
            "res://subject.gd",
        ])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    // Oversize inputs can be rejected before the producer finishes writing.
    let _ = stdin.write_all(input);
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1, "one bounded result: {stdout:?}");
    let result: Value = serde_json::from_str(&stdout).unwrap();
    (
        output.status.code().unwrap(),
        result,
        String::from_utf8(output.stderr).unwrap(),
    )
}
#[test]
fn duplicate_unknown_and_trailing_input_refuse_before_editor_or_source() {
    let inputs=[
        br#"{"schema_version":1,"schema_version":1,"request_id":"edit","basis":{},"replacement_source":"secret"}"#.to_vec(),
        br#"{"schema_version":1,"request_id":"edit","basis":{},"replacement_source":"secret","unknown":true}"#.to_vec(),
        br#"{"schema_version":1,"request_id":"edit","basis":{},"replacement_source":"secret"} {}"#.to_vec(),
        br#"{"schema_version":1,"request_id":7,"basis":{},"replacement_source":"secret"}"#.to_vec(),
    ];
    for raw in inputs {
        let (code, result, stderr) = invoke(&raw, &[]);
        assert_eq!(code, 3);
        assert_eq!(result["outcome"], "refused");
        assert_eq!(result["application"], "not_applied");
        assert_eq!(result["reason"], "invalid_request");
        assert_eq!(result["resolved_target"], Value::Null);
        assert!(!stderr.contains("secret"));
    }
}
#[test]
fn stdin_limit_rejects_without_selecting_editor() {
    let raw = vec![b' '; 12 * 1024 * 1024 + 1];
    let (code, result, _) = invoke(&raw, &[]);
    assert_eq!(code, 3);
    assert_eq!(result["reason"], "invalid_request");
    assert_eq!(result["application"], "not_applied");
}
#[test]
fn source_limit_and_basis_binding_refuse_before_selection() {
    let prior = basis("extends Node\n");
    let oversized = json!({"schema_version":1,"request_id":"edit","basis":prior,
        "replacement_source":"x".repeat(SOURCE_LIMIT_BYTES+1)});
    let raw = serde_json::to_vec(&oversized).unwrap();
    let (code, result, _) = invoke(&raw, &[]);
    assert_eq!(code, 3);
    assert_eq!(result["reason"], "invalid_request");
    assert_eq!(result["request_id"], "edit");
    assert_eq!(
        result["requested_target"]["script_path"],
        "res://subject.gd"
    );
    assert_eq!(result["resolved_target"], Value::Null);

    let mut wrong = basis("extends Node\n");
    wrong["snapshot"]["sources"]["B"]["text"] = json!("unexpected modified basis\n");
    let raw = serde_json::to_vec(&json!({"schema_version":1,"request_id":"edit",
        "basis":wrong,"replacement_source":"extends Node\n"}))
    .unwrap();
    let (code, result, _) = invoke(&raw, &[]);
    assert_eq!(code, 3);
    assert_eq!(result["application"], "not_applied");
    assert_eq!(result["request_id"], "edit");
    assert_eq!(result["resolved_target"], Value::Null);
}
#[test]
fn internal_worker_cannot_be_started_with_ordinary_pipes() {
    let output = Command::new(env!("CARGO_BIN_EXE_edit-gdscript"))
        .arg("--internal-edit-worker")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("verified_changed"));
}

#[test]
fn nullable_basis_fields_must_be_present_and_semantic_refusals_keep_correlation() {
    let payload = json!({"schema_version":1,"request_id":"current-edit",
        "basis":basis("extends Node\n"),"replacement_source":"extends Node\n"});
    let decode = |value: &Value| {
        wire::edit::decode_request(
            &serde_json::to_vec(value).unwrap(),
            ProjectRoot::new("/tmp/godot-agent-kit-edit-caller-test").unwrap(),
            Some(SessionId::new(SESSION).unwrap()),
            ResourcePath::new("res://subject.gd").unwrap(),
        )
    };
    assert!(decode(&payload).unwrap().is_ok());
    for field in ["resolved_target", "snapshot", "selection"] {
        let mut missing = payload.clone();
        missing["basis"].as_object_mut().unwrap().remove(field);
        assert!(decode(&missing).is_err(), "missing {field}");
    }
    for text in ["extends Node\r\n", "# \0\n", "\u{feff}extends Node\n"] {
        let mut unsupported = payload.clone();
        unsupported["replacement_source"] = json!(text);
        let (code, result, stderr) = invoke(&serde_json::to_vec(&unsupported).unwrap(), &[]);
        assert_eq!(code, 3);
        assert_eq!(result["request_id"], "current-edit");
        assert_eq!(result["reason"], "unsupported_representation");
        assert_eq!(result["application"], "not_applied");
        assert_eq!(
            result["requested_target"]["script_path"],
            "res://subject.gd"
        );
        assert!(result["before"].is_null() && result["resolved_target"].is_null());
        assert!(stderr.is_empty());
    }
}
