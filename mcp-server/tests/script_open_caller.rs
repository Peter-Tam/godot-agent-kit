use serde_json::Value;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
const BASE: [&str; 6] = [
    "--registry",
    "/tmp/gak-open-caller-missing-registry",
    "--project",
    "/tmp/gak-open-caller-missing-project",
    "--script",
    "res://subject.gd",
];
fn invoke(args: &[&str]) -> (i32, Value, String, Duration) {
    let began = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_open-gdscript"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let held_stdin = child.stdin.take().unwrap();
    let output = child.wait_with_output().unwrap();
    drop(held_stdin);
    let elapsed = began.elapsed();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.ends_with('\n'));
    let value = serde_json::from_str(&stdout).unwrap();
    (
        output.status.code().unwrap(),
        value,
        String::from_utf8(output.stderr).unwrap(),
        elapsed,
    )
}
#[test]
fn invalid_arguments_are_structured_no_effect_refusals_with_exit_two() {
    let cases = [
        vec![],
        vec!["--registry", "/tmp/registry"],
        vec!["--force", "true"],
        vec!["res://subject.gd"],
        vec!["--internal-open-worker"],
        vec!["--internal-stock-validation-worker"],
    ];
    for args in cases {
        let (code, value, stderr, elapsed) = invoke(&args);
        assert_eq!(code, 2, "{value}");
        assert_eq!(value["outcome"], "refused");
        assert_eq!(value["reason"], "invalid_request");
        assert_eq!(value["application"], "not_applied");
        assert_eq!(value["requested_target"], Value::Null);
        assert_eq!(value["resolved_target"], Value::Null);
        assert_eq!(value["observation"], Value::Null);
        assert!(stderr.is_empty());
        assert!(elapsed < Duration::from_secs(10));
    }
    for extra in [
        ["--script", "res://other.gd"],
        ["--focus", "true"],
        ["--retry", "1"],
        ["--source", "UNAUTHORIZED_STDIN_SENTINEL"],
        ["--reload", "true"],
    ] {
        let mut args = BASE.to_vec();
        args.extend(extra);
        let (code, value, stderr, _) = invoke(&args);
        assert_eq!(code, 2);
        assert_eq!(value["reason"], "invalid_request");
        assert!(!value.to_string().contains("UNAUTHORIZED_STDIN_SENTINEL"));
        assert!(!stderr.contains("UNAUTHORIZED_STDIN_SENTINEL"));
    }
}
#[test]
fn opener_never_waits_for_stdin_and_request_ids_are_fresh() {
    let (code, first, _, elapsed) = invoke(&BASE);
    assert_eq!(code, 3);
    assert_eq!(first["application"], "not_applied");
    assert_eq!(first["observation"], Value::Null);
    assert!(elapsed < Duration::from_secs(10));
    let (_, second, _, _) = invoke(&BASE);
    for value in [&first, &second] {
        let id = value["request_id"].as_str().unwrap();
        assert_eq!(id.len(), 32);
        assert!(id
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)));
    }
    assert_ne!(first["request_id"], second["request_id"]);
}
#[test]
fn well_formed_unsupported_document_kinds_preserve_target_and_refuse_before_routing() {
    const SESSION: &str = "00112233445566778899aabbccddeeff";
    for path in ["res://scripts/note.txt", "res://container.tscn::GDScript_x"] {
        for session in [None, Some(SESSION)] {
            let mut args = BASE.to_vec();
            args[5] = path;
            if let Some(session) = session {
                args.extend(["--session", session]);
            }
            let (code, value, stderr, elapsed) = invoke(&args);
            assert_eq!(code, 3, "{value}");
            assert_eq!(value["outcome"], "refused");
            assert_eq!(value["reason"], "invalid_document_kind");
            assert_eq!(value["stage"], "accepted");
            assert_eq!(value["application"], "not_applied");
            assert_eq!(value["requested_target"]["project_root"], BASE[3]);
            assert_eq!(
                value["requested_target"]["session_id"],
                serde_json::json!(session)
            );
            assert_eq!(value["requested_target"]["script_path"], path);
            assert_eq!(value["resolved_target"], Value::Null);
            assert_eq!(value["before"], Value::Null);
            assert_eq!(value["observation"], Value::Null);
            for stage in [
                "resource_binding",
                "initial_compilation",
                "document_open",
                "verification",
            ] {
                assert_eq!(value["progress"][stage]["state"], "not_started");
            }
            assert!(stderr.is_empty());
            assert!(elapsed < Duration::from_secs(10));
        }
    }
}
#[test]
fn malformed_target_or_session_remains_an_argument_error_even_for_an_unsupported_kind() {
    for (path, session) in [
        ("res://scripts/../note.txt", None),
        ("res://container.tscn::", None),
        ("res://scripts/note.txt", Some("invalid-session")),
    ] {
        let mut args = BASE.to_vec();
        args[5] = path;
        if let Some(session) = session {
            args.extend(["--session", session]);
        }
        let (code, value, stderr, _) = invoke(&args);
        assert_eq!(code, 2);
        assert_eq!(value["reason"], "invalid_request");
        assert_eq!(value["requested_target"], Value::Null);
        assert_eq!(value["resolved_target"], Value::Null);
        assert_eq!(value["observation"], Value::Null);
        assert!(stderr.is_empty());
    }
}
