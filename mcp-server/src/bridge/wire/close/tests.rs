use super::*;
use serde_json::{json, Value};
fn fixture() -> (ObservationRequest, ResolvedTarget) {
    let request = ObservationRequest::new(
        RequestId::new("close-test").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new("00112233445566778899aabbccddeeff").unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let target = ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(
            DecimalCounter::new("1").unwrap(),
            DecimalCounter::new("2").unwrap(),
        ),
        request.session_id().unwrap().clone(),
        EngineVersion::new("4.7.2", "hash").unwrap(),
    )
    .unwrap();
    (request, target)
}
fn stamp(start: &str, end: &str) -> Value {
    json!({"clock_id":"editor:00112233445566778899aabbccddeeff","started_tick_us":start,"finished_tick_us":end,"received_elapsed_us":0})
}
fn native() -> Value {
    json!({"request_id":"close-test","session_id":"00112233445566778899aabbccddeeff","script_path":"res://subject.gd","native_build_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","native_api_revision":3,"phase":"returned","script_instance_id":"3","editor_instance_id":"4","buffer_instance_id":"5","entry_collection":stamp("1","1"),"return_collection":stamp("2","2"),"entered":true,"close_error":0,"old_document_removed":true,"selection":{"before":"target","after":"no_source_editor","request_effect":"native_fallback"},"target_buffer":"disposed","protection":{"status":"preserved","revalidation":"not_applicable","required_count":0,"completed_count":0,"reason":null},"continuation":{"state":"not_applicable","reason":null,"required_editor_ids":[],"completed_editor_ids":[],"collections":[]},"invalidated":false,"reason":null,"terminal_discard":false})
}
fn progress() -> Value {
    json!({"v":5,"kind":"close_progress","request_id":"close-test","session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","script_path":"res://subject.gd","collection":stamp("3","4"),"status":"returned","reason":null,"native":native(),"expiry_tick_us":"100"})
}
fn decode_progress(value: &Value) -> bool {
    let (r, t) = fixture();
    decode_reply(
        &serde_json::to_vec(value).unwrap(),
        "close_progress",
        &r,
        &t,
        "/fixture/project",
        1000,
        None,
    )
    .is_ok()
}
#[test]
fn native_effect_evidence_requires_every_nullable_and_binding_field() {
    let exact = progress();
    assert!(decode_progress(&exact));
    for key in exact["native"].as_object().unwrap().keys() {
        let mut missing = exact.clone();
        missing["native"].as_object_mut().unwrap().remove(key);
        assert!(!decode_progress(&missing), "missing {key}");
    }
    for key in exact.as_object().unwrap().keys() {
        let mut missing = exact.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(!decode_progress(&missing), "missing {key}");
    }
}
#[test]
fn wrong_scope_revision_phase_and_noncanonical_ids_cannot_establish_effects() {
    let exact = progress();
    for (key, value) in [
        ("request_id", json!("other")),
        ("session_id", json!("ffeeddccbbaa99887766554433221100")),
        ("script_path", json!("res://other.gd")),
        ("native_api_revision", json!(2)),
        ("phase", json!("verified_newly_closed")),
        ("buffer_instance_id", json!("05")),
        ("editor_instance_id", json!("0")),
        ("close_error", json!(0.5)),
    ] {
        let mut wrong = exact.clone();
        wrong["native"][key] = value;
        assert!(!decode_progress(&wrong), "{key}");
    }
    let mut extra = exact;
    extra["native"]["source"] = json!("PRIVATE_SOURCE");
    assert!(!decode_progress(&extra));
}
#[test]
fn private_frames_reject_duplicate_deep_oversize_and_wrong_kind() {
    let (r, t) = fixture();
    let exact = serde_json::to_string(&progress()).unwrap();
    let duplicate = exact.replace("\"entered\":true", "\"entered\":true,\"entered\":false");
    assert!(decode_reply(
        duplicate.as_bytes(),
        "close_progress",
        &r,
        &t,
        "/fixture/project",
        1000,
        None
    )
    .is_err());
    assert!(decode_reply(
        exact.as_bytes(),
        "close_waited",
        &r,
        &t,
        "/fixture/project",
        1000,
        None
    )
    .is_err());
    let deep = format!("{}null{}", "[".repeat(33), "]".repeat(33));
    assert!(decode_reply(
        deep.as_bytes(),
        "close_progress",
        &r,
        &t,
        "/fixture/project",
        1000,
        None
    )
    .is_err());
    let large = vec![b' '; RESULT_LIMIT + 1];
    assert!(decode_reply(
        &large,
        "close_progress",
        &r,
        &t,
        "/fixture/project",
        1000,
        None
    )
    .is_err());
}
#[test]
fn continuation_completion_requires_exact_admitted_ids_and_ordered_actual_stamps() {
    let mut frame = progress();
    let ledger = json!({"state":"completed","reason":null,"required_editor_ids":["6"],"completed_editor_ids":["6"],"collections":[{"editor_id":"6","visit":stamp("1","1"),"completion":stamp("2","2")}]});
    frame["native"]["continuation"] = ledger.clone();
    assert!(decode_progress(&frame));
    for bad in [
        json!({"state":"completed","reason":null,"required_editor_ids":["6"],"completed_editor_ids":[],"collections":[]}),
        json!({"state":"completed","reason":null,"required_editor_ids":["6"],"completed_editor_ids":["7"],"collections":[{"editor_id":"6","visit":stamp("1","1"),"completion":stamp("2","2")}]}),
    ] {
        frame["native"]["continuation"] = bad;
        assert!(!decode_progress(&frame));
    }
    let mut stale = ledger;
    stale["collections"][0]["visit"] = stamp("3", "3");
    frame["native"]["continuation"] = stale;
    assert!(!decode_progress(&frame));
}
#[test]
fn source_free_projection_does_not_turn_native_ok_into_a_public_verdict() {
    let (r, t) = fixture();
    let reply = decode_reply(
        &serde_json::to_vec(&progress()).unwrap(),
        "close_progress",
        &r,
        &t,
        "/fixture/project",
        1000,
        None,
    )
    .unwrap();
    assert_eq!(reply.native.unwrap().close_error, Some(0));
    assert_eq!(reply.status, "returned");
}

fn public_prior() -> ObservationOutcome {
    let (request, _) = fixture();
    let request = ObservationRequest::new(
        RequestId::new("prior").unwrap(),
        request.project_root().clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    );
    let target = ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(
            DecimalCounter::new("1").unwrap(),
            DecimalCounter::new("2").unwrap(),
        ),
        request.session_id().unwrap().clone(),
        EngineVersion::new(
            "4.7.2.stable.official.ed1daf0bf",
            "ed1daf0bf001b61586d9930840f2f1394092c079",
        )
        .unwrap(),
    )
    .unwrap();
    let counter = |s| DecimalCounter::new(s).unwrap();
    let collected = |editor, receipt| {
        CollectionStamp::new(
            if editor {
                ClockId::Editor(target.session_id().clone())
            } else {
                ClockId::Caller
            },
            counter("10"),
            counter("11"),
            receipt,
        )
        .unwrap()
    };
    let identity = DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        target.script_path().clone(),
        Some(counter("3")),
        Some(counter("4")),
        Some(counter("5")),
        Some(FileIdentity::new(counter("6"), counter("7"))),
    )
    .unwrap();
    let witness = |authority| {
        Witness::new(
            target.script_path().clone(),
            (authority != Authority::D).then(|| counter("3")),
            (authority == Authority::B).then(|| counter("4")),
            (authority == Authority::B).then(|| counter("5")),
            (authority == Authority::D).then(|| FileIdentity::new(counter("6"), counter("7"))),
            Some(counter("17")),
        )
    };
    let source = |authority, receipt| {
        SourceObservation::observed(
            authority,
            "extends Node\n".into(),
            collected(authority != Authority::D, receipt),
            witness(authority),
            Staleness::unknown(),
        )
    };
    let evidence = ObservationEvidence::new(
        target.clone(),
        DocumentState::new(
            Some(identity),
            DocumentFact::observed(Validity::Valid, collected(false, 12)),
            DocumentFact::observed(OpenState::Open, collected(true, 13)),
        ),
        Sources::new(
            source(Authority::D, 14),
            source(Authority::R, 15),
            source(Authority::B, 16),
        )
        .unwrap(),
        DirtyObservation::observed(
            DirtyState::Clean,
            collected(true, 17),
            witness(Authority::B),
        ),
        Recheck::performed(vec![]),
    );
    ObservationOutcome::classify(
        request,
        ObservationInterval::new(1, 2, 100),
        Some(target),
        Some(evidence),
        vec![],
        vec![],
    )
    .unwrap()
}

#[test]
fn serialized_basis_preserves_original_authority_and_document_provenance() {
    let prior = public_prior();
    let expected = crate::script_edit::ExpectedRevisionBasis::from_observation(&prior).unwrap();
    let basis: Value =
        serde_json::from_slice(&super::super::encode_outcome(&prior).unwrap()).unwrap();
    let payload = json!({"schema_version":1,"request_id":"close-test","basis":basis});
    let (request, _) = fixture();
    let decoded = decode_request(
        &serde_json::to_vec(&payload).unwrap(),
        request.project_root().clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(decoded.expected(), Some(&expected));
    for (pointer, wrong) in [
        (
            "/basis/snapshot/sources/R/collection/clock_id",
            json!("caller"),
        ),
        (
            "/basis/snapshot/sources/D/collection/received_elapsed_us",
            json!(101),
        ),
        (
            "/basis/snapshot/document/validity/collection/clock_id",
            json!("editor:ffeeddccbbaa99887766554433221100"),
        ),
    ] {
        let mut bad = payload.clone();
        *bad.pointer_mut(pointer).unwrap() = wrong;
        assert!(
            decode_request(
                &serde_json::to_vec(&bad).unwrap(),
                request.project_root().clone(),
                request.session_id().cloned(),
                request.script_path().clone()
            )
            .is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn native_and_callback_stamps_must_belong_to_actual_entry_and_envelope() {
    let mut exact = progress();
    exact["native"]["entry_collection"] = stamp("20", "20");
    exact["native"]["return_collection"] = stamp("30", "30");
    exact["collection"] = stamp("40", "50");
    exact["native"]["continuation"] = json!({"state":"completed","reason":null,"required_editor_ids":["6"],"completed_editor_ids":["6"],"collections":[{"editor_id":"6","visit":stamp("21","22"),"completion":stamp("23","24")}]});
    assert!(decode_progress(&exact));
    for (pointer, wrong) in [
        ("/native/return_collection", stamp("19", "19")),
        ("/native/return_collection", stamp("51", "51")),
        ("/native/entry_collection", stamp("51", "51")),
        ("/native/continuation/collections/0/visit", stamp("1", "2")),
        (
            "/native/continuation/collections/0/completion",
            stamp("51", "52"),
        ),
        (
            "/native/continuation/collections/0/completion",
            stamp("20", "21"),
        ),
        (
            "/native/continuation/collections/0/visit/clock_id",
            json!("caller"),
        ),
    ] {
        let mut wrong_frame = exact.clone();
        *wrong_frame.pointer_mut(pointer).unwrap() = wrong;
        assert!(!decode_progress(&wrong_frame), "{pointer}");
    }
}

#[test]
fn pre_entry_or_discard_facts_cannot_establish_native_effects() {
    let mut pre_entry = progress();
    pre_entry["status"] = json!("refused");
    pre_entry["native"]["phase"] = json!("prepared");
    pre_entry["native"]["entered"] = json!(false);
    pre_entry["native"]["entry_collection"] = Value::Null;
    pre_entry["native"]["return_collection"] = Value::Null;
    pre_entry["native"]["close_error"] = Value::Null;
    pre_entry["native"]["old_document_removed"] = Value::Null;
    pre_entry["native"]["target_buffer"] = json!("retained");
    pre_entry["native"]["selection"]["request_effect"] = json!("none");
    assert!(decode_progress(&pre_entry));
    for (key, value) in [
        ("old_document_removed", json!(true)),
        ("target_buffer", json!("disposed")),
        ("return_collection", stamp("2", "2")),
        ("close_error", json!(0)),
        ("entry_collection", stamp("1", "1")),
    ] {
        let mut wrong = pre_entry.clone();
        wrong["native"][key] = value;
        assert!(!decode_progress(&wrong), "{key}");
    }
    pre_entry["native"]["selection"]["request_effect"] = json!("native_fallback");
    assert!(!decode_progress(&pre_entry));
    let mut discarded = progress();
    discarded["native"]["terminal_discard"] = json!(true);
    assert!(!decode_progress(&discarded));
}

fn decode_sample(frame: &Value) -> Result<Reply, RoutingFailure> {
    let (request, target) = fixture();
    decode_reply(
        &serde_json::to_vec(frame).unwrap(),
        "close_sample",
        &request,
        &target,
        "/fixture/project",
        1000,
        None,
    )
}
fn survivor() -> Value {
    let mut frame = progress();
    frame["kind"] = json!("close_sample");
    frame["status"] = json!("unavailable");
    frame["purpose"] = json!("survivor");
    for key in ["sample", "resource_edited", "resource_state", "protection"] {
        frame[key] = Value::Null;
    }
    frame["selection"] =
        json!({"before":"target","after":"target","request_effect":"native_fallback"});
    frame
}
#[test]
fn survivor_keeps_new_selection_and_historical_native_effect_independently() {
    let exact = survivor();
    let decoded = decode_sample(&exact).unwrap();
    assert_eq!(decoded.selection.unwrap().after, "target");
    assert_eq!(
        decoded.native.unwrap().selection.unwrap().after,
        "no_source_editor"
    );
    for (pointer, wrong) in [
        ("/purpose", json!("post_close")),
        ("/purpose", json!("pre_close")),
        ("/purpose", json!("preparation")),
        ("/selection/before", json!("other")),
        ("/selection/request_effect", json!("none")),
        ("/selection/after", json!("reopened")),
    ] {
        let mut bad = exact.clone();
        *bad.pointer_mut(pointer).unwrap() = wrong;
        assert!(decode_sample(&bad).is_err(), "{pointer}");
    }
    for key in exact.as_object().unwrap().keys() {
        let mut missing = exact.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(decode_sample(&missing).is_err(), "missing {key}");
    }
}

#[test]
fn reopened_survivor_sample_does_not_replace_historical_buffer_effects() {
    let prior: Value =
        serde_json::from_slice(&super::super::encode_outcome(&public_prior()).unwrap()).unwrap();
    let snapshot = &prior["snapshot"];
    let mut sample = json!({"v":5,"kind":"sample","request_id":"close-test","session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","script_path":"res://subject.gd","collection":stamp("20","21"),"document":snapshot["document"],"R":snapshot["sources"]["R"],"B":snapshot["sources"]["B"],"dirty":snapshot["dirty"],"diagnostics":[]});
    // Live editor acquisition does not claim Rust's independently read D identity.
    sample["document"]["identity"]["disk_file_id"] = Value::Null;
    for pointer in [
        "/document/validity/collection",
        "/document/open_state/collection",
        "/R/collection",
        "/B/collection",
        "/dirty/collection",
    ] {
        *sample.pointer_mut(pointer).unwrap() = stamp("20", "21");
    }
    for pointer in [
        "/document/identity/buffer_instance_id",
        "/B/witness/buffer_instance_id",
        "/dirty/witness/buffer_instance_id",
    ] {
        *sample.pointer_mut(pointer).unwrap() = json!("9");
    }
    let mut frame = survivor();
    frame["status"] = json!("observed");
    frame["sample"] = sample;
    frame["collection"] = stamp("22", "23");
    let reply = decode_sample(&frame).unwrap();
    assert_eq!(
        reply
            .sample
            .unwrap()
            .document
            .identity()
            .unwrap()
            .buffer_instance_id()
            .unwrap()
            .as_str(),
        "9"
    );
    let native = reply.native.unwrap();
    assert_eq!(native.buffer_instance_id.as_deref(), Some("5"));
    assert_eq!(native.old_document_removed, Some(true));
    assert_eq!(native.target_buffer, "disposed");
}

#[test]
fn required_nullable_stage_keys_and_seven_document_bound_are_enforced() {
    let (request, target) = fixture();
    for (kind, fields) in [
        (
            "close_state",
            vec!["sample", "resource_edited", "resource_state", "selection"],
        ),
        (
            "close_prepared",
            vec![
                "sample",
                "target_guard",
                "context",
                "guard_sha256",
                "validation",
            ],
        ),
    ] {
        let mut exact = progress();
        exact["kind"] = json!(kind);
        exact["status"] = json!("refused");
        for key in &fields {
            exact[*key] = Value::Null;
        }
        let decode = |v: &Value| {
            decode_reply(
                &serde_json::to_vec(v).unwrap(),
                kind,
                &request,
                &target,
                "/fixture/project",
                1000,
                None,
            )
        };
        assert!(decode(&exact).is_ok());
        for key in fields {
            let mut missing = exact.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(decode(&missing).is_err(), "{kind} missing {key}");
        }
    }
    for count in [7, 8] {
        let ids: Vec<String> = (6..6 + count).map(|id| id.to_string()).collect();
        let rows: Vec<Value> = ids
            .iter()
            .map(|id| json!({"editor_id":id,"visit":stamp("1","1"),"completion":stamp("2","2")}))
            .collect();
        let mut frame = progress();
        frame["native"]["continuation"] = json!({"state":"completed","reason":null,"required_editor_ids":ids,"completed_editor_ids":ids,"collections":rows});
        assert_eq!(decode_progress(&frame), count == 7);
    }
}

#[test]
fn recheck_purpose_is_not_interchangeable_with_survivor_collection() {
    let (request, target) = fixture();
    let original = StampIn::deserialize(stamp("1", "1"))
        .unwrap()
        .domain(target.session_id(), 10, Stage::ReadEditor, true)
        .unwrap();
    let mut frame = progress();
    frame["kind"] = json!("close_rechecked");
    frame["status"] = json!("rechecked");
    frame["purpose"] = json!("pre_close");
    frame["recheck"] = json!({"v":5,"kind":"recheck","request_id":"close-test","session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","script_path":"res://subject.gd","collection":stamp("3","4"),"checks":"performed","detected_changes":[],"reason":null});
    for key in ["protection", "selection", "resource_state"] {
        frame[key] = Value::Null;
    }
    let decode = |v: &Value| {
        decode_reply(
            &serde_json::to_vec(v).unwrap(),
            "close_rechecked",
            &request,
            &target,
            "/fixture/project",
            1000,
            Some(&original),
        )
    };
    for purpose in ["pre_close", "recognition", "post_close"] {
        frame["purpose"] = json!(purpose);
        assert!(decode(&frame).is_ok(), "{purpose}");
    }
    frame["purpose"] = json!("survivor");
    assert!(decode(&frame).is_err());
    frame["purpose"] = json!("pre_close");
    for key in [
        "purpose",
        "recheck",
        "protection",
        "selection",
        "resource_state",
    ] {
        let mut missing = frame.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(decode(&missing).is_err(), "missing {key}");
    }
}

#[test]
fn late_scope_denial_redacts_sources_without_erasing_known_effects_or_first_failure() {
    use crate::script_close as core;
    let prior = public_prior();
    let snapshot = core::SnapshotSummary::of(prior.snapshot().unwrap());
    let (selectors, _) = fixture();
    let request = CloseRequest::new(
        selectors.request_id().clone(),
        selectors.project_root().clone(),
        selectors.session_id().cloned(),
        selectors.script_path().clone(),
        Some(&prior),
    )
    .unwrap();
    let expected = core::Expected {
        basis: request.expected().unwrap().clone(),
        use_: "matched",
    };
    let interval = ObservationInterval::new(1, 2, 1000);
    let mut attempt = core::Attempt::new(request, interval.clone());
    for stage in ["selected", "inspected", "prepared", "validated"] {
        attempt.stage(stage);
    }
    attempt.authorize().unwrap();
    attempt.native(core::NativeEvidence {
        entered: true,
        removed: Some(true),
        discard: false,
        invalidated: false,
        target_buffer: "disposed",
        selection: None,
        protection: None,
        continuation: None,
        entry: None,
        returned: None,
    });
    attempt.fail("protocol_error");
    attempt.fail("denied_access");
    // A later acquired envelope cannot restore disclosure authority.
    attempt.result.expected = Some(expected);
    attempt.result.before = Some(core::Before {
        snapshot: snapshot.clone(),
        resource_edited: Some(false),
        edited_collection: None,
        current_version: None,
        saved_version: None,
    });
    attempt.result.observation = Some(core::ObservationSummary {
        purpose: "survivor",
        interval: interval.clone(),
        snapshot,
    });
    let outcome = attempt.finish(interval);
    let result: Value = serde_json::from_slice(&encode_outcome(&outcome).unwrap()).unwrap();
    assert_eq!(result["outcome"], "applied_unverified");
    assert_eq!(result["application"], "applied");
    assert_eq!(result["reason"], "protocol_error");
    assert_eq!(result["history"]["target_buffer"], "disposed");
    for field in ["expected", "before", "observation"] {
        assert!(result[field].is_null(), "{field}");
    }
}
