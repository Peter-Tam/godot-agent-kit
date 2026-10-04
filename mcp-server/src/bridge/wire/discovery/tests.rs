use super::*;
use crate::observation::{ProjectRoot, RequestId, SessionId};
use crate::script_discovery::{Binding, Gap, Identity, Selection, Stage, Version};
fn request() -> DiscoveryRequest {
    DiscoveryRequest::new(
        RequestId::new("discovery-wire").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new("00112233445566778899aabbccddeeff").unwrap()),
    )
}
fn target() -> DiscoveryTarget {
    DiscoveryTarget {
        request_id: "discovery-wire".into(),
        project_root: "/fixture/project".into(),
        project_file_id: Identity {
            device: "1".into(),
            inode: "2".into(),
        },
        session_id: "00112233445566778899aabbccddeeff".into(),
        godot_version: Version {
            version: crate::bridge::GODOT_VERSION.into(),
            hash: crate::bridge::ENGINE_HASH.into(),
        },
    }
}
fn context_reply() -> serde_json::Value {
    serde_json::json!({"v":6,"kind":"discover_state","request_id":"discovery-wire","session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","collection":{"clock_id":"editor:00112233445566778899aabbccddeeff","started_tick_us":"10","finished_tick_us":"11","received_elapsed_us":0},"status":"observed","reason":null,"context":{"policy":"godot_project_files_v1","project_data_directory":"res://.godot","filesystem_epoch":"0","scanning":false,"importing":false},"expiry_tick_us":"4500010"})
}
#[test]
fn private_v5_reply_is_rejected() {
    let mut old = context_reply();
    old["v"] = serde_json::json!(5);
    assert!(decode(&old).is_err());
}
fn decode(v: &serde_json::Value) -> Result<Context, Reason> {
    decode_context(
        &serde_json::to_vec(v).unwrap(),
        "discover_state",
        &request(),
        &target(),
        "/fixture/project",
        100,
    )
}
#[test]
fn controls_are_real_path_free_bound_exact_tuples() {
    let r = request();
    let t = target();
    for (control, opcode) in [
        (Control::Begin, "discover_begin"),
        (Control::Recheck, "discover_recheck"),
        (Control::Finish, "discover_finish"),
        (Control::Abort, "discover_abort"),
    ] {
        let bytes = encode_control(control, &r, &t, "/fixture/project", 4500).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let expected = if control == Control::Begin {
            serde_json::json!([
                6,
                opcode,
                "discovery-wire",
                "00112233445566778899aabbccddeeff",
                "/fixture/project",
                4500
            ])
        } else {
            serde_json::json!([
                6,
                opcode,
                "discovery-wire",
                "00112233445566778899aabbccddeeff",
                "/fixture/project"
            ])
        };
        assert_eq!(v, expected);
    }
    assert!(encode_control(Control::Begin, &r, &t, "/fixture/project", 0).is_err());
    assert!(encode_control(Control::Begin, &r, &t, "/fixture/project", 4501).is_err());
    let mut wrong = t.clone();
    wrong.request_id = "foreign".into();
    assert!(encode_control(Control::Recheck, &r, &wrong, "/fixture/project", 1).is_err());
    let mut wrong = t;
    wrong.session_id = "ffeeddccbbaa99887766554433221100".into();
    assert!(encode_control(Control::Begin, &r, &wrong, "/fixture/project", 1).is_err());
}
#[test]
fn editor_receipt_is_assigned_once_in_the_caller_domain() {
    let v = context_reply();
    let c = decode(&v).unwrap();
    assert_eq!(c.collection.received_elapsed_us, 100);
    assert_eq!(c.collection.started_tick_us, "10");
    assert_eq!(c.collection.finished_tick_us, "11");
    assert_eq!(c.context.unwrap().project_data_directory, "res://.godot");
    let mut v = v;
    v["collection"]["received_elapsed_us"] = serde_json::json!(101);
    assert!(decode(&v).is_err());
}
#[test]
fn context_status_reason_ownership_and_nullability_are_closed() {
    for (status, reason, owned) in [
        ("unavailable", "unavailable_editor_context", true),
        ("refused", "unsupported_visibility_policy", true),
        ("refused", "unsupported_discovery", false),
        ("refused", "busy", false),
    ] {
        let mut v = context_reply();
        v["status"] = status.into();
        v["reason"] = reason.into();
        v["context"] = serde_json::Value::Null;
        if !owned {
            v["expiry_tick_us"] = serde_json::Value::Null;
        }
        assert!(decode(&v).is_ok());
        v["context"] = context_reply()["context"].clone();
        assert!(decode(&v).is_err());
    }
    for field in ["reason", "context", "expiry_tick_us"] {
        let mut v = context_reply();
        v.as_object_mut().unwrap().remove(field);
        assert!(decode(&v).is_err(), "missing {field}");
    }
    for (field, value) in [
        ("reason", serde_json::json!("busy")),
        ("status", serde_json::json!("success")),
        ("expiry_tick_us", serde_json::json!("11")),
        ("context", serde_json::Value::Null),
    ] {
        let mut v = context_reply();
        v[field] = value;
        assert!(decode(&v).is_err(), "{field}");
    }
}
#[test]
fn context_rejects_unknown_duplicate_copied_or_wrong_primitive_facts() {
    for field in ["v", "kind", "request_id", "session_id", "project_root"] {
        let mut v = context_reply();
        v[field] = if field == "v" {
            serde_json::json!(3)
        } else {
            serde_json::json!("foreign")
        };
        assert!(decode(&v).is_err());
    }
    for field in [
        "policy",
        "project_data_directory",
        "filesystem_epoch",
        "scanning",
        "importing",
    ] {
        let mut v = context_reply();
        v["context"][field] = match field {
            "policy" => "future_policy".into(),
            "project_data_directory" => "res://arbitrary".into(),
            "filesystem_epoch" => "00".into(),
            _ => serde_json::json!(0),
        };
        assert!(decode(&v).is_err());
    }
    let mut v = context_reply();
    v["collection"]["clock_id"] = "caller".into();
    assert!(decode(&v).is_err());
    for parent in ["", "context", "collection"] {
        let mut v = context_reply();
        let object = if parent.is_empty() {
            &mut v
        } else {
            &mut v[parent]
        };
        object["extra"] = true.into();
        assert!(decode(&v).is_err());
    }
    let raw = serde_json::to_string(&context_reply()).unwrap();
    for (old, new) in [
        ("\"v\":6", "\"v\":6,\"v\":6"),
        (
            "\"policy\":\"godot_project_files_v1\"",
            "\"policy\":\"godot_project_files_v1\",\"policy\":\"godot_project_files_v1\"",
        ),
        (
            "\"started_tick_us\":\"10\"",
            "\"started_tick_us\":\"10\",\"started_tick_us\":\"10\"",
        ),
    ] {
        let duplicate = raw.replace(old, new);
        assert_ne!(raw, duplicate);
        assert!(decode_context(
            duplicate.as_bytes(),
            "discover_state",
            &request(),
            &target(),
            "/fixture/project",
            100
        )
        .is_err());
    }
}
#[test]
fn finished_and_aborted_acknowledgments_cannot_be_exchanged_or_rebound() {
    for (control, kind, discard) in [
        (Control::Finish, "discover_finished", false),
        (Control::Abort, "discover_aborted", true),
    ] {
        let mut v = context_reply();
        let object = v.as_object_mut().unwrap();
        for field in ["status", "reason", "context", "expiry_tick_us"] {
            object.remove(field);
        }
        object.insert("kind".into(), kind.into());
        object.insert("terminal_discard".into(), discard.into());
        let bytes = serde_json::to_vec(&v).unwrap();
        let ack = decode_ack(
            &bytes,
            control,
            &request(),
            &target(),
            "/fixture/project",
            100,
        )
        .unwrap();
        assert_eq!(ack.collection.received_elapsed_us, 100);
        assert_eq!(ack.terminal_discard, discard);
        v["terminal_discard"] = (!discard).into();
        assert!(decode_ack(
            &serde_json::to_vec(&v).unwrap(),
            control,
            &request(),
            &target(),
            "/fixture/project",
            100
        )
        .is_err());
        v["terminal_discard"] = discard.into();
        v["session_id"] = "ffeeddccbbaa99887766554433221100".into();
        assert!(decode_ack(
            &serde_json::to_vec(&v).unwrap(),
            control,
            &request(),
            &target(),
            "/fixture/project",
            100
        )
        .is_err());
    }
}
#[test]
fn editor_control_size_and_depth_bounds_reject_before_model_acquisition() {
    let raw = serde_json::to_vec(&context_reply()).unwrap();
    let mut exact = raw.clone();
    exact.resize(CONTROL_LIMIT, b' ');
    assert!(decode_context(
        &exact,
        "discover_state",
        &request(),
        &target(),
        "/fixture/project",
        100
    )
    .is_ok());
    exact.push(b' ');
    assert!(decode_context(
        &exact,
        "discover_state",
        &request(),
        &target(),
        "/fixture/project",
        100
    )
    .is_err());
    let deep = format!("{}0{}", "[".repeat(33), "]".repeat(33));
    assert!(decode_event(deep.as_bytes()).is_err());
    let mut v = context_reply();
    v["v"] = serde_json::json!(6.0);
    assert!(decode(&v).is_err());
}
fn worker_entries(n: usize) -> Event {
    Event::Entries {
        binding: Binding::target(&target()),
        ordinal: 0,
        collection: Stamp::caller(20, 25, 25),
        entries: (0..n).map(|n| format!("res://{n}.gd")).collect(),
        visited_entries: n as u32,
        visited_directories: 1,
    }
}
#[test]
fn worker_progress_bounds_are_enforced_during_deserialization() {
    assert!(decode_event(&encode_event(&worker_entries(64)).unwrap()).is_ok());
    assert!(decode_event(&encode_event(&worker_entries(65)).unwrap()).is_err());
    let bytes = encode_event(&worker_entries(1)).unwrap();
    let mut exact = bytes.clone();
    exact.resize(script_discovery::FRAME_LIMIT, b' ');
    assert!(decode_event(&exact).is_ok());
    exact.push(b' ');
    assert!(decode_event(&exact).is_err());
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["ordinal"] = serde_json::json!(-1);
    assert!(decode_event(&serde_json::to_vec(&v).unwrap()).is_err());
    v["ordinal"] = serde_json::json!(0.0);
    assert!(decode_event(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn worker_nested_fields_are_strict_and_nullable_fields_are_required() {
    let selected = Event::Selected {
        binding: Binding::request(&request()),
        target: target(),
        capability: true,
    };
    let bytes = encode_event(&selected).unwrap();
    for pointer in [
        "/",
        "/binding",
        "/target",
        "/target/project_file_id",
        "/target/godot_version",
    ] {
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let object = if pointer == "/" {
            &mut v
        } else {
            v.pointer_mut(pointer).unwrap()
        };
        object["extra"] = true.into();
        assert!(
            decode_event(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["binding"]
        .as_object_mut()
        .unwrap()
        .remove("project_file_id");
    assert!(decode_event(&serde_json::to_vec(&v).unwrap()).is_err());
    let raw = String::from_utf8(bytes).unwrap();
    for (old, new) in [
        (
            "\"kind\":\"selected\"",
            "\"kind\":\"selected\",\"kind\":\"selected\"",
        ),
        ("\"device\":\"1\"", "\"device\":\"1\",\"device\":\"1\""),
        (
            "\"capability\":true",
            "\"capability\":true,\"capability\":true",
        ),
    ] {
        let duplicate = raw.replace(old, new);
        assert_ne!(raw, duplicate);
        assert!(decode_event(duplicate.as_bytes()).is_err());
    }
    let failed = Event::Failed {
        binding: Binding::request(&request()),
        reason: Reason::Cancelled,
        selection: None,
    };
    let mut v = serde_json::to_value(failed).unwrap();
    v.as_object_mut().unwrap().remove("selection");
    assert!(decode_event(&serde_json::to_vec(&v).unwrap()).is_err());
    let invalidated = Event::Invalidate {
        binding: Binding::target(&target()),
        reason: Reason::NamespaceChanged,
        prefix: None,
    };
    let mut v = serde_json::to_value(invalidated).unwrap();
    v.as_object_mut().unwrap().remove("prefix");
    assert!(decode_event(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn worker_gap_and_selection_arrays_reject_one_over_before_growth() {
    for n in [63, 64] {
        let event = Event::Rechecked {
            binding: Binding::target(&target()),
            context: decode(&context_reply()).unwrap(),
            exhausted: false,
            namespace_checked: false,
            changed: false,
            visited_entries: 0,
            visited_directories: 0,
            gaps: (0..n)
                .map(|_| Gap {
                    code: Reason::UnsupportedPath,
                    stage: Stage::Enumerate,
                    scope: Some("res://".into()),
                })
                .collect(),
            omitted_gaps: 0,
        };
        assert_eq!(
            decode_event(&encode_event(&event).unwrap()).is_err(),
            n == 64
        );
    }
    for n in [64, 65] {
        let event = Event::Failed {
            binding: Binding::request(&request()),
            reason: Reason::AmbiguousTarget,
            selection: Some(Selection {
                candidate_sessions: (0..n).map(|n| format!("{n:032x}")).collect(),
                missing_selector: "session_id".into(),
            }),
        };
        assert_eq!(
            decode_event(&encode_event(&event).unwrap()).is_err(),
            n == 65
        );
    }
}
#[test]
fn known_same_attempt_binding_loss_is_not_confused_with_copied_or_malformed_frames() {
    for field in ["session_id", "project_root"] {
        let mut v = context_reply();
        v[field] = if field == "session_id" {
            "ffeeddccbbaa99887766554433221100".into()
        } else {
            "/other/project".into()
        };
        let bytes = serde_json::to_vec(&v).unwrap();
        assert!(decode_context(
            &bytes,
            "discover_state",
            &request(),
            &target(),
            "/fixture/project",
            100
        )
        .is_err());
        assert!(
            binding_changed(
                &bytes,
                "discover_state",
                &request(),
                &target(),
                "/fixture/project"
            ),
            "field={field}"
        );
        v["request_id"] = "copied-request".into();
        assert!(!binding_changed(
            &serde_json::to_vec(&v).unwrap(),
            "discover_state",
            &request(),
            &target(),
            "/fixture/project"
        ));
    }
    let mut v = context_reply();
    v["project_root"] = "/other/project".into();
    v["extra"] = true.into();
    assert!(!binding_changed(
        &serde_json::to_vec(&v).unwrap(),
        "discover_state",
        &request(),
        &target(),
        "/fixture/project"
    ));
}
