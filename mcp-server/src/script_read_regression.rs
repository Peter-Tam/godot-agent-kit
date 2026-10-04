use crate::observation::*;
use crate::script_read::{ScriptReadResult, ScriptRevision};

fn decimal(v: &str) -> DecimalCounter {
    DecimalCounter::new(v).unwrap()
}
fn capture(
    id: &str,
    offset: u64,
    version: &str,
    buffer: &str,
    text: &str,
    dirty: DirtyState,
) -> ObservationOutcome {
    capture_variant(id, offset, version, buffer, text, dirty, None)
}
fn capture_variant(
    id: &str,
    offset: u64,
    version: &str,
    buffer: &str,
    text: &str,
    dirty: DirtyState,
    changed: Option<usize>,
) -> ObservationOutcome {
    let scalar = |index, original: &'static str| {
        if changed == Some(index) {
            "99"
        } else {
            original
        }
    };
    let session = SessionId::new(if changed == Some(0) {
        "ffeeddccbbaa99887766554433221100"
    } else {
        "00112233445566778899aabbccddeeff"
    })
    .unwrap();
    let request = ObservationRequest::new(
        RequestId::new(id).unwrap(),
        ProjectRoot::new(if changed == Some(1) {
            "/fixture/other"
        } else {
            "/fixture/project"
        })
        .unwrap(),
        Some(session.clone()),
        ResourcePath::new(if changed == Some(2) {
            "res://other.gd"
        } else {
            "res://subject.gd"
        })
        .unwrap(),
    );
    let target = ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(decimal(scalar(3, "1")), decimal(scalar(4, "2"))),
        session.clone(),
        EngineVersion::new(
            if changed == Some(10) {
                "4.7.3"
            } else {
                "4.7.2"
            },
            if changed == Some(11) { "other" } else { "hash" },
        )
        .unwrap(),
    )
    .unwrap();
    let stamp = |editor| {
        CollectionStamp::new(
            if editor {
                ClockId::Editor(session.clone())
            } else {
                ClockId::Caller
            },
            decimal(&offset.to_string()),
            decimal(&(offset + 1).to_string()),
            offset + 1,
        )
        .unwrap()
    };
    let witness = |a| {
        Witness::new(
            request.script_path().clone(),
            (a != Authority::D).then(|| decimal(scalar(5, "3"))),
            (a == Authority::B).then(|| decimal(scalar(6, "4"))),
            (a == Authority::B).then(|| decimal(buffer)),
            (a == Authority::D)
                .then(|| FileIdentity::new(decimal(scalar(7, "6")), decimal(scalar(8, "7")))),
            Some(decimal(if changed == Some(9) && a == Authority::D {
                "99"
            } else {
                version
            })),
        )
    };
    let source = |a| {
        SourceObservation::observed(
            a,
            text.to_owned(),
            stamp(a != Authority::D),
            witness(a),
            Staleness::unknown(),
        )
    };
    let identity = DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        request.script_path().clone(),
        Some(decimal(scalar(5, "3"))),
        Some(decimal(scalar(6, "4"))),
        Some(decimal(buffer)),
        Some(FileIdentity::new(
            decimal(scalar(7, "6")),
            decimal(scalar(8, "7")),
        )),
    )
    .unwrap();
    let evidence = ObservationEvidence::new(
        target.clone(),
        DocumentState::new(
            Some(identity),
            DocumentFact::observed(Validity::Valid, stamp(false)),
            DocumentFact::observed(OpenState::Open, stamp(true)),
        ),
        Sources::new(
            source(Authority::D),
            source(Authority::R),
            source(Authority::B),
        )
        .unwrap(),
        DirtyObservation::observed(dirty, stamp(true), witness(Authority::B)),
        Recheck::performed(vec![]),
    );
    ObservationOutcome::classify(
        request,
        ObservationInterval::new(offset, offset + 10, offset + 100),
        Some(target),
        Some(evidence),
        vec![],
        vec![],
    )
    .unwrap()
}
#[test]
fn revision_excludes_correlation_and_interval_but_binds_versions_identity_and_source() {
    let read = |id, tick, version, buffer, text| {
        ScriptReadResult::from_observation(capture(
            id,
            tick,
            version,
            buffer,
            text,
            DirtyState::Clean,
        ))
        .revision()
        .cloned()
        .unwrap()
    };
    let original = read("first", 10, "17", "5", "雪\n");
    assert_eq!(original, read("second", 20, "17", "5", "雪\n"));
    assert_ne!(original, read("first", 10, "18", "5", "雪\n"));
    assert_ne!(original, read("first", 10, "17", "8", "雪\n"));
    assert_ne!(original, read("first", 10, "17", "5", "other\n"));
}
#[test]
fn unsafe_read_keeps_source_but_has_no_revision() {
    let read = ScriptReadResult::from_observation(capture(
        "dirty",
        10,
        "17",
        "5",
        "human\n",
        DirtyState::Dirty,
    ));
    assert!(read.revision().is_none());
    let value: serde_json::Value = serde_json::from_slice(&read.encode().unwrap()).unwrap();
    assert_eq!(value["source"], "human\n");
    assert_eq!(value["state"]["dirty"]["buffer"]["state"], "dirty");
}
#[test]
fn projection_deduplicates_agreeing_text_and_hides_private_envelope() {
    let read = ScriptReadResult::from_observation(capture(
        "secret-id",
        10,
        "17",
        "5",
        "unique-source\n",
        DirtyState::Clean,
    ));
    let bytes = read.encode().unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert_eq!(text.matches("unique-source").count(), 1);
    assert!(!text.contains("secret-id"));
    assert!(!text.contains("script_instance_id"));
    assert!(!text.contains("collection"));
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert_eq!(value["state"]["sources"]["disk"]["equals_source"], true);
}
#[test]
fn revisions_are_exact_and_source_bounds_preserve_empty_observation() {
    for invalid in [
        "sr1:",
        "sr1:ABC",
        "sr2:0000000000000000000000000000000000000000000000000000000000000000",
    ] {
        assert!(ScriptRevision::new(invalid).is_err());
    }
    let empty =
        ScriptReadResult::from_observation(capture("empty", 10, "17", "5", "", DirtyState::Clean));
    assert!(empty.revision().is_some());
    let large = "x".repeat(SOURCE_LIMIT_BYTES + 1);
    assert!(ScriptReadResult::from_observation(capture(
        "large",
        10,
        "17",
        "5",
        &large,
        DirtyState::Clean
    ))
    .revision()
    .is_none());
}

#[test]
fn every_stable_open_target_and_witness_scalar_changes_the_commitment() {
    let original = ScriptReadResult::from_observation(capture_variant(
        "original",
        10,
        "17",
        "5",
        "same\n",
        DirtyState::Clean,
        None,
    ))
    .revision()
    .cloned()
    .unwrap();
    for index in 0..12 {
        let changed = ScriptReadResult::from_observation(capture_variant(
            "changed",
            10,
            "17",
            "5",
            "same\n",
            DirtyState::Clean,
            Some(index),
        ));
        assert_ne!(changed.revision(), Some(&original), "stable field {index}");
    }
}

#[test]
fn denied_selection_suppresses_source_target_and_revision() {
    let request = ObservationRequest::new(
        RequestId::new("denied").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        None,
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let observation = ObservationOutcome::classify(
        request,
        ObservationInterval::new(1, 2, 10),
        None,
        None,
        vec![TerminalFailure::DeniedAccess],
        vec![],
    )
    .unwrap();
    let result = ScriptReadResult::from_observation(observation);
    let payload: serde_json::Value = serde_json::from_slice(&result.encode().unwrap()).unwrap();
    assert!(payload["source"].is_null());
    assert!(payload["revision"].is_null());
    assert!(payload["state"]["target"].is_null());
    assert_eq!(payload["state"]["status"], "denied_access");
}
#[test]
fn unsupported_source_is_informational_not_a_revision() {
    for source in ["line\r\n", "\u{feff}line\n", "line\0"] {
        let result = ScriptReadResult::from_observation(capture(
            "unsupported",
            10,
            "17",
            "5",
            source,
            DirtyState::Clean,
        ));
        assert!(result.revision().is_none());
        let payload: serde_json::Value = serde_json::from_slice(&result.encode().unwrap()).unwrap();
        assert_eq!(payload["source"], source);
    }
}

#[test]
fn supplement_changes_invalidate_only_the_affected_authority() {
    let original = capture("read", 10, "17", "5", "old\n", DirtyState::Clean);
    let target = original.resolved_target().unwrap();
    let request = ObservationRequest::new(
        RequestId::new("read").unwrap(),
        target.project_root().clone(),
        Some(target.session_id().clone()),
        target.script_path().clone(),
    );
    let changed = original
        .supplement_recheck(
            request,
            original.interval().clone(),
            vec![DetectedChange::Source(Authority::R)],
            None,
            false,
        )
        .unwrap();
    let read = ScriptReadResult::from_observation(changed);
    let value: serde_json::Value = serde_json::from_slice(&read.encode().unwrap()).unwrap();
    assert!(read.revision().is_none());
    assert_eq!(value["source"], "old\n");
    assert_eq!(value["state"]["source_origin"], "editor_buffer");
    assert_eq!(
        value["state"]["sources"]["disk"]["availability"],
        "observed"
    );
    assert_eq!(
        value["state"]["sources"]["loaded_resource"]["availability"],
        "unavailable"
    );
    assert_eq!(
        value["state"]["sources"]["loaded_resource"]["invalidated"]["current"],
        false
    );
}

fn supplement_changes(
    original: &ObservationOutcome,
    changes: Vec<DetectedChange>,
) -> ObservationOutcome {
    let target = original.resolved_target().unwrap();
    let request = ObservationRequest::new(
        RequestId::new("read").unwrap(),
        target.project_root().clone(),
        Some(target.session_id().clone()),
        target.script_path().clone(),
    );
    original
        .supplement_recheck(request, original.interval().clone(), changes, None, false)
        .unwrap()
}

#[test]
fn detected_changes_encode_every_variant_as_stable_surface_code_objects() {
    for (change, surface, code) in [
        (DetectedChange::Source(Authority::D), "D", "source_changed"),
        (DetectedChange::Source(Authority::R), "R", "source_changed"),
        (DetectedChange::Source(Authority::B), "B", "source_changed"),
        (DetectedChange::Dirty, "dirty", "source_changed"),
        (
            DetectedChange::DocumentClosed,
            "document",
            "document_closed",
        ),
        (
            DetectedChange::DocumentIdentityReplaced,
            "document",
            "identity_changed",
        ),
        (
            DetectedChange::SessionReplaced,
            "session",
            "identity_changed",
        ),
        (DetectedChange::SessionEnded, "session", "session_ended"),
        (
            DetectedChange::DiskIdentityReplaced,
            "D",
            "identity_changed",
        ),
    ] {
        let original = capture("read", 10, "17", "5", "old\n", DirtyState::Clean);
        let read = ScriptReadResult::from_observation(supplement_changes(&original, vec![change]));
        let value: serde_json::Value = serde_json::from_slice(&read.encode().unwrap()).unwrap();
        assert!(read.revision().is_none());
        assert!(value["revision"].is_null());
        assert_eq!(
            value["state"]["consistency"]["detected_changes"],
            serde_json::json!([{"surface": surface, "code": code}]),
            "public change for {surface}/{code}",
        );
    }
}

#[test]
fn detected_changes_preserve_supplement_order_and_distinct_codes_without_debug_leaks() {
    let original = capture("read", 10, "17", "5", "old\n", DirtyState::Clean);
    let first = supplement_changes(
        &original,
        vec![DetectedChange::Source(Authority::B), DetectedChange::Dirty],
    );
    let changed = supplement_changes(
        &first,
        vec![
            DetectedChange::Source(Authority::D),
            DetectedChange::DiskIdentityReplaced,
        ],
    );
    let read = ScriptReadResult::from_observation(changed);
    let bytes = read.encode().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(read.revision().is_none());
    assert!(value["revision"].is_null());
    assert_eq!(
        value["state"]["consistency"]["detected_changes"],
        serde_json::json!([
            {"surface": "B", "code": "source_changed"},
            {"surface": "dirty", "code": "source_changed"},
            {"surface": "D", "code": "source_changed"},
            {"surface": "D", "code": "identity_changed"},
        ]),
    );
    let text = std::str::from_utf8(&bytes).unwrap();
    for debug_name in [
        "Source(",
        "\"Dirty\"",
        "DocumentClosed",
        "DocumentIdentityReplaced",
        "SessionReplaced",
        "SessionEnded",
        "DiskIdentityReplaced",
    ] {
        assert!(
            !text.contains(debug_name),
            "caller-visible Debug name: {debug_name}"
        );
    }
}

#[test]
fn source_free_refusal_preserves_failure_selection_correlation_and_clock() {
    let request = ObservationRequest::new(
        RequestId::new("capture").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        None,
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let selection = Selection::ambiguous(vec![
        SessionId::new("a".repeat(32)).unwrap(),
        SessionId::new("b".repeat(32)).unwrap(),
    ])
    .unwrap();
    for (failure, reason) in [
        (TerminalFailure::EditorUnavailable, "editor_unavailable"),
        (
            TerminalFailure::AmbiguousTarget(selection),
            "ambiguous_target",
        ),
        (TerminalFailure::DeniedAccess, "denied_access"),
        (TerminalFailure::MissingTarget, "missing_target"),
        (TerminalFailure::DisconnectedEditor, "disconnected_editor"),
        (TerminalFailure::Timeout, "timeout"),
        (
            TerminalFailure::UnsupportedObservation,
            "unsupported_observation",
        ),
    ] {
        let (target, evidence) = if failure == TerminalFailure::MissingTarget {
            let session = SessionId::new("a".repeat(32)).unwrap();
            let target = ResolvedTarget::for_request(
                &request,
                request.project_root().clone(),
                FileIdentity::new(decimal("1"), decimal("2")),
                session.clone(),
                EngineVersion::new(crate::bridge::GODOT_VERSION, crate::bridge::ENGINE_HASH)
                    .unwrap(),
            )
            .unwrap();
            let stamp = |clock| CollectionStamp::new(clock, decimal("1"), decimal("2"), 2).unwrap();
            let evidence = ObservationEvidence::new(
                target.clone(),
                DocumentState::new(
                    None,
                    DocumentFact::observed(Validity::Missing, stamp(ClockId::Caller)),
                    DocumentFact::observed(OpenState::NotOpen, stamp(ClockId::Editor(session))),
                ),
                Sources::new(
                    SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing)
                        .unwrap(),
                    SourceObservation::unavailable(Authority::R, SourceReason::ResourceNotLoaded)
                        .unwrap(),
                    SourceObservation::closed_buffer(),
                )
                .unwrap(),
                DirtyObservation::closed_document(),
                Recheck::performed(vec![]),
            );
            (Some(target), Some(evidence))
        } else {
            (None, None)
        };
        let observation = ObservationOutcome::classify(
            request.clone(),
            ObservationInterval::new(1, 2, 10),
            target,
            evidence,
            vec![failure],
            vec![],
        )
        .unwrap();
        let mut read = ScriptReadResult::from_observation(observation);
        read.requested = Some(request.clone());
        let refusal = read.refusal(&RequestId::new("edit").unwrap(), None);
        assert_eq!(refusal["outcome"]["request_id"], "edit");
        assert_eq!(refusal["outcome"]["reason"], reason);
        assert_eq!(refusal["outcome"]["interval"]["elapsed_us"], 10);
        assert!(!refusal["outcome"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(refusal["outcome"].get("source").is_none());
        if reason == "ambiguous_target" {
            assert_eq!(
                refusal["outcome"]["selection"]["candidate_sessions"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            assert_eq!(refusal["outcome"]["next_action"]["kind"], "specify_session");
            let payload: serde_json::Value =
                serde_json::from_slice(&read.encode().unwrap()).unwrap();
            assert_eq!(
                payload["state"]["selection"],
                refusal["outcome"]["selection"]
            );
        }
        if reason == "denied_access" {
            assert!(refusal["outcome"]["requested_target"].is_null());
        }
    }
}

#[test]
fn informative_closed_dirty_and_divergent_resource_survives_null_revision() {
    use crate::script_closed_edit::State;
    for (resource, edited, reason) in [
        ("disk\n", true, "dirty_resource"),
        ("human\n", true, "dirty_resource"),
        ("human\n", false, "divergent_resource"),
    ] {
        let open = capture("read", 10, "17", "5", "disk\n", DirtyState::Clean);
        let snapshot = open.snapshot().unwrap();
        let target = snapshot.target();
        let request = ObservationRequest::new(
            RequestId::new("read").unwrap(),
            target.project_root().clone(),
            Some(target.session_id().clone()),
            target.script_path().clone(),
        );
        let stamp = CollectionStamp::new(
            ClockId::Editor(target.session_id().clone()),
            decimal("10"),
            decimal("11"),
            11,
        )
        .unwrap();
        let r = SourceObservation::observed(
            Authority::R,
            resource.into(),
            stamp.clone(),
            snapshot.sources().resource().witness().unwrap().clone(),
            Staleness::unknown(),
        );
        let identity = DocumentIdentity::new(
            ScriptKind::ExternalGdscript,
            request.script_path().clone(),
            Some(decimal("3")),
            None,
            None,
            Some(FileIdentity::new(decimal("6"), decimal("7"))),
        )
        .unwrap();
        let evidence = ObservationEvidence::new(
            target.clone(),
            DocumentState::new(
                Some(identity),
                DocumentFact::observed(Validity::Valid, stamp.clone()),
                DocumentFact::observed(OpenState::NotOpen, stamp),
            ),
            Sources::new(
                snapshot.sources().disk().clone(),
                r,
                SourceObservation::closed_buffer(),
            )
            .unwrap(),
            DirtyObservation::closed_document(),
            Recheck::performed(vec![]),
        );
        let observation = ObservationOutcome::classify(
            request.clone(),
            open.interval().clone(),
            Some(target.clone()),
            Some(evidence),
            vec![],
            vec![],
        )
        .unwrap();
        let state: State = serde_json::from_value(serde_json::json!({"project_device":"1","project_inode":"2","close_epoch":"3","lifecycle":"closed","file_revision":{"device":"6","inode":"7","utf8_bytes":"5","sha256":crate::project_fs_validation::hex_sha256(b"disk\n"),"mtime":{"seconds":"0","nanoseconds":0},"ctime":{"seconds":"0","nanoseconds":0}},"resource":{"state":"present","instance_id":"3","path":"res://subject.gd","source":resource,"edited":edited,"profile_sha256":"a".repeat(64)}})).unwrap();
        assert!(
            crate::script_closed_edit::ClosedExpectedBasis::from_observation(&observation, &state)
                .is_none()
        );
        let mut read = ScriptReadResult::from_observation(observation);
        read.closed_observed = Some(state);
        read.suppress_revision(reason);
        let payload: serde_json::Value = serde_json::from_slice(&read.encode().unwrap()).unwrap();
        assert!(payload["revision"].is_null());
        assert_eq!(
            payload["state"]["dirty"]["loaded_resource"]["availability"],
            "observed"
        );
        assert_eq!(
            payload["state"]["dirty"]["loaded_resource"]["state"],
            if edited { "dirty" } else { "clean" }
        );
        assert_eq!(
            payload["state"]["sources"]["loaded_resource"]["availability"],
            "observed"
        );
        assert_eq!(
            payload["state"]["sources"]["loaded_resource"]["equals_source"],
            resource == "disk\n"
        );
        if resource != "disk\n" {
            assert_eq!(
                payload["state"]["sources"]["loaded_resource"]["text"],
                resource
            );
        }
        let lifecycle = read
            .observation
            .supplement_recheck(request, open.interval().clone(), vec![], None, true)
            .unwrap();
        assert!(lifecycle
            .snapshot()
            .unwrap()
            .document()
            .open_state()
            .value()
            .is_none());
        assert!(lifecycle
            .snapshot()
            .unwrap()
            .document()
            .open_state()
            .invalidated_evidence()
            .is_some());
        assert!(lifecycle
            .snapshot()
            .unwrap()
            .sources()
            .disk()
            .text()
            .is_some());
    }
}
