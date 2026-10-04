use super::*;
use serde_json::{json, Value};
fn request() -> ObservationRequest {
    ObservationRequest::new(
        RequestId::new("open_boundary").unwrap(),
        ProjectRoot::new("/project").unwrap(),
        Some(SessionId::new("00112233445566778899aabbccddeeff").unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    )
}
fn target() -> ResolvedTarget {
    let r = request();
    ResolvedTarget::for_request(
        &r,
        r.project_root().clone(),
        FileIdentity::new(
            DecimalCounter::new("1").unwrap(),
            DecimalCounter::new("2").unwrap(),
        ),
        r.session_id().unwrap().clone(),
        EngineVersion::new(
            "4.7.2.stable.official.ed1daf0bf",
            "ed1daf0bf001b61586d9930840f2f1394092c079",
        )
        .unwrap(),
    )
    .unwrap()
}
fn refused() -> Value {
    json!({"v":6,"kind":"open_state","request_id":"open_boundary","session_id":"00112233445566778899aabbccddeeff","project_root":"/project","script_path":"res://subject.gd","collection":{"clock_id":"editor:00112233445566778899aabbccddeeff","started_tick_us":"1","finished_tick_us":"2","received_elapsed_us":0},"status":"refused","reason":"slot_busy","native":null,"sample":null,"cache":null,"resource_edited":null,"expiry_tick_us":null})
}
#[test]
fn private_v5_reply_is_rejected() {
    let mut old = refused();
    old["v"] = json!(5);
    assert!(parse(&old).is_err());
}
fn parse(v: &Value) -> Result<Reply, RoutingFailure> {
    decode_reply(
        &serde_json::to_vec(v).unwrap(),
        "open_state",
        &request(),
        &target(),
        "/project",
        100,
        None,
    )
}
#[test]
fn missing_nullable_fields_wrong_scope_unknown_status_and_collection_clock_are_protocol_failures() {
    for key in [
        "native",
        "sample",
        "cache",
        "resource_edited",
        "expiry_tick_us",
    ] {
        let mut v = refused();
        v.as_object_mut().unwrap().remove(key);
        assert!(parse(&v).is_err(), "missing {key}");
    }
    for (key, value) in [
        ("session_id", json!("11112233445566778899aabbccddeeff")),
        ("script_path", json!("res://unrelated.gd")),
        ("status", json!("done")),
        ("v", json!(2)),
        ("resource_edited", json!("false")),
    ] {
        let mut v = refused();
        v[key] = value;
        assert!(parse(&v).is_err(), "{key}");
    }
    let mut v = refused();
    v["collection"]["clock_id"] = json!("caller");
    assert!(parse(&v).is_err());
    let mut v = refused();
    v["collection"]["received_elapsed_us"] = json!(101);
    assert!(parse(&v).is_err());
}
#[test]
fn cache_absence_cannot_hide_unavailable_or_hash_bearing_state() {
    let mut v = refused();
    v["cache"] = json!({"state":"absent","script_instance_id":null,"sha256":null,"utf8_bytes":null,"reason":"cache_unavailable"});
    assert!(parse(&v).is_err());
    v["cache"] = json!({"state":"absent","script_instance_id":null,"sha256":"a".repeat(64),"utf8_bytes":"1","reason":null});
    assert!(parse(&v).is_err());
    v["cache"] = json!({"state":"unavailable","script_instance_id":null,"sha256":null,"utf8_bytes":null,"reason":"cache_unavailable"});
    let r = parse(&v).unwrap();
    assert_eq!(r.cache.unwrap().state, "unavailable");
}
#[test]
fn native_ids_and_parse_codes_reject_unsafe_numeric_conversion() {
    let mut v = refused();
    v["native"] = json!({"request_id":"open_boundary","stage":"inspected","next_stage":"prepare","cache_binding":"not_started","initial_compilation":"not_started","document_open":"not_started","target_parse_code":null,"script_instance_id":"3","editor_instance_id":null,"buffer_instance_id":null,"target_open":false,"terminal_discard":false,"entered":true,"mode":null,"protection":null,"selection":"no_source"});
    for id in [
        json!(9007199254740993u64),
        json!("18446744073709551616"),
        json!("03"),
        json!("0"),
    ] {
        let mut changed = v.clone();
        changed["native"]["script_instance_id"] = id;
        assert!(parse(&changed).is_err());
    }
    for code in [json!(-1), json!(0.5), json!(2147483648u64)] {
        let mut changed = v.clone();
        changed["native"]["target_parse_code"] = code;
        assert!(parse(&changed).is_err());
    }
}
#[test]
fn duplicate_keys_and_excessive_depth_never_become_defaulted_evidence() {
    let mut bytes = serde_json::to_vec(&refused()).unwrap();
    bytes.pop();
    bytes.extend_from_slice(b",\"resource_edited\":false}");
    assert!(decode_reply(
        &bytes,
        "open_state",
        &request(),
        &target(),
        "/project",
        100,
        None
    )
    .is_err());
    let mut v = refused();
    let mut nested = json!(null);
    for _ in 0..33 {
        nested = json!([nested]);
    }
    v["unknown"] = nested;
    assert!(parse(&v).is_err());
    let raw=br#"{"enable":true,"levels":{"unused_variable":1,"unused_variable":2},"directory_rules":{},"provenance":{"source":"editor_project_settings","project_root":"/project","session_id":"00112233445566778899aabbccddeeff"}}"#;
    let mut deserializer = serde_json::Deserializer::from_slice(raw);
    assert!(checked_warnings(&mut deserializer).is_err());
}
