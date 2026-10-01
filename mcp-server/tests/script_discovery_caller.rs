use serde_json::Value;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn invoke(args: &[&str]) -> (i32, Value, String, Duration) {
    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_discover-gdscripts"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A caller does not consume stdin, including its inaccessible private-worker mode.
    let held_stdin = child.stdin.take().unwrap();
    let output = child.wait_with_output().unwrap();
    drop(held_stdin);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.ends_with('\n'));
    assert!(stdout.len() <= 6 * 1024 * 1024 + 1);
    (
        output.status.code().unwrap(),
        serde_json::from_str(&stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
        start.elapsed(),
    )
}

#[test]
fn invalid_arguments_never_reflect_input_or_acquire_inventory() {
    const SENTINEL: &str = "PRIVATE_UNVALIDATED_INPUT_SENTINEL";
    let base = [
        "--registry",
        "/tmp/private-registry",
        "--project",
        "/tmp/project",
    ];
    let mut cases = vec![
        vec![],
        vec!["--registry", "/tmp/private-registry"],
        vec![SENTINEL],
        vec!["--internal-discovery-worker"],
        vec!["--project", "relative"],
        vec!["--registry", "relative", "--project", "/tmp/project"],
    ];
    for extra in [
        ["--project", SENTINEL],
        ["--registry", SENTINEL],
        ["--session", SENTINEL],
        ["--script", SENTINEL],
        ["--prefix", SENTINEL],
        ["--timeout", "9999"],
        ["--force", "true"],
        ["--offline", "true"],
        ["--executable", SENTINEL],
    ] {
        let mut args = base.to_vec();
        args.extend(extra);
        cases.push(args);
    }
    for args in cases {
        let (code, value, stderr, elapsed) = invoke(&args);
        assert_eq!(code, 3, "{value}");
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["outcome"], "refused");
        assert_eq!(value["diagnostics"][0]["code"], "invalid_request");
        assert_eq!(value["requested_target"], Value::Null);
        assert_eq!(value["resolved_target"], Value::Null);
        assert_eq!(value["inventory"], Value::Null);
        assert_eq!(value["selection"], Value::Null);
        assert!(!value.to_string().contains(SENTINEL));
        assert!(stderr.is_empty(), "{stderr}");
        assert!(elapsed < Duration::from_secs(5), "{elapsed:?}");
        assert_eq!(value.as_object().unwrap().len(), 9);
        for key in [
            "interval",
            "requested_target",
            "resolved_target",
            "inventory",
            "selection",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing required nullable field {key}"
            );
        }
    }
}

#[test]
fn inaccessible_project_is_not_a_complete_empty_project_and_ids_are_fresh() {
    let args = [
        "--registry",
        "/tmp/gak-discovery-no-registry",
        "--project",
        "/tmp/gak-discovery-no-project",
    ];
    let (_, first, stderr, elapsed) = invoke(&args);
    let (_, second, _, _) = invoke(&args);
    assert_eq!(first["outcome"], "refused");
    assert_eq!(first["inventory"], Value::Null);
    assert_eq!(first["requested_target"]["project_root"], args[3]);
    assert_eq!(first["requested_target"]["session_id"], Value::Null);
    assert!(stderr.is_empty());
    assert!(elapsed < Duration::from_secs(5));
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
fn help_and_version_do_not_launch_discovery() {
    for arg in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_discover-gdscripts"))
            .arg(arg)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.starts_with("discover-gdscripts "));
        assert!(!text.contains("internal-discovery"));
    }
}

#[test]
fn non_unicode_arguments_and_non_normal_project_roots_are_not_lossy_aliases() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let output = Command::new(env!("CARGO_BIN_EXE_discover-gdscripts"))
        .args(["--registry", "/tmp/registry", "--project"])
        .arg(OsString::from_vec(b"/tmp/PRIVATE_NAME_\xff".to_vec()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["requested_target"], Value::Null);
    assert_eq!(value["inventory"], Value::Null);
    assert!(!String::from_utf8(output.stdout)
        .unwrap()
        .contains("PRIVATE_NAME"));
    assert!(output.stderr.is_empty());
    for root in [
        "/tmp/project/../other",
        "/tmp//project",
        "/tmp/project\\alias",
        "/tmp/project\ncontrol",
    ] {
        let (code, value, stderr, _) = invoke(&["--registry", "/tmp/registry", "--project", root]);
        assert_eq!(code, 3);
        assert_eq!(value["diagnostics"][0]["code"], "invalid_request");
        assert_eq!(value["requested_target"], Value::Null);
        assert!(stderr.is_empty());
    }
}

#[test]
fn exact_root_byte_bound_routes_but_one_over_never_echoes_the_target() {
    let exact = format!("/tmp/{}end", "a/".repeat(508));
    assert_eq!(exact.len(), 1024);
    let (code, value, stderr, _) =
        invoke(&["--registry", "/tmp/gak-no-registry", "--project", &exact]);
    assert_eq!(code, 3);
    assert_eq!(value["requested_target"]["project_root"], exact);
    assert_ne!(value["diagnostics"][0]["code"], "invalid_request");
    assert!(stderr.is_empty());
    let over = format!("{exact}x");
    let (code, value, stderr, _) =
        invoke(&["--registry", "/tmp/gak-no-registry", "--project", &over]);
    assert_eq!(code, 3);
    assert_eq!(value["requested_target"], Value::Null);
    assert_eq!(value["diagnostics"][0]["code"], "invalid_request");
    assert!(stderr.is_empty());
}
