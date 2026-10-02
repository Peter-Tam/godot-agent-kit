use godot_agent_kit::{
    bridge::wire::{self, close as codec},
    observation::*,
    script_close::CloseRequest,
    script_edit::BasisError,
};
use serde_json::json;
const SESSION: &str = "00112233445566778899aabbccddeeff";

fn decimal(s: &str) -> DecimalCounter {
    DecimalCounter::new(s).unwrap()
}

thread_local! {
    static TICK: std::cell::Cell<u64> = const { std::cell::Cell::new(100) };
}

fn stamp(editor: bool) -> CollectionStamp {
    let tick = TICK.with(|clock| {
        let tick = clock.get() + 2;
        clock.set(tick);
        tick
    });
    CollectionStamp::new(
        if editor {
            ClockId::Editor(SessionId::new(SESSION).unwrap())
        } else {
            ClockId::Caller
        },
        decimal(&tick.to_string()),
        decimal(&(tick + 1).to_string()),
        tick + 1,
    )
    .unwrap()
}

fn prior_sample_with_buffer(
    texts: [&str; 3],
    dirty: DirtyState,
    (version, buffer_id): (Option<&str>, &str),
    request_id: &str,
    missing: Option<Authority>,
    stale: bool,
    changes: Vec<DetectedChange>,
) -> ObservationOutcome {
    let request = ObservationRequest::new(
        RequestId::new(request_id).unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let target = ResolvedTarget::for_request(
        &request,
        request.project_root().clone(),
        FileIdentity::new(decimal("1"), decimal("2")),
        SessionId::new(SESSION).unwrap(),
        EngineVersion::new(
            "4.7.2.stable.official.ed1daf0bf",
            "ed1daf0bf001b61586d9930840f2f1394092c079",
        )
        .unwrap(),
    )
    .unwrap();
    let identity = DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        request.script_path().clone(),
        Some(decimal("3")),
        Some(decimal("4")),
        Some(decimal(buffer_id)),
        Some(FileIdentity::new(decimal("6"), decimal("7"))),
    )
    .unwrap();
    let witness = |authority| {
        Witness::new(
            request.script_path().clone(),
            (authority != Authority::D).then(|| decimal("3")),
            (authority == Authority::B).then(|| decimal("4")),
            (authority == Authority::B).then(|| decimal(buffer_id)),
            (authority == Authority::D).then(|| FileIdentity::new(decimal("6"), decimal("7"))),
            if authority == Authority::B {
                version.map(decimal)
            } else {
                Some(decimal("17"))
            },
        )
    };
    let source = |authority: Authority| {
        if missing == Some(authority) {
            SourceObservation::unavailable(
                authority,
                match authority {
                    Authority::D => SourceReason::DiskUnreadable,
                    Authority::R => SourceReason::ResourceUnreadable,
                    Authority::B => SourceReason::BufferUnreadable,
                },
            )
            .unwrap()
        } else {
            let staleness = if stale && authority == Authority::R {
                Staleness::known_stale(vec![StalenessEvidence::unapplied_change(
                    decimal("16"),
                    Authority::D,
                    decimal("17"),
                    stamp(false),
                    witness(Authority::D),
                )
                .unwrap()])
                .unwrap()
            } else {
                Staleness::unknown()
            };
            let actual = match authority {
                Authority::D => texts[0],
                Authority::R => texts[1],
                Authority::B => texts[2],
            };
            SourceObservation::observed(
                authority,
                actual.to_owned(),
                stamp(authority != Authority::D),
                witness(authority),
                staleness,
            )
        }
    };
    let sources = Sources::new(
        source(Authority::D),
        source(Authority::R),
        source(Authority::B),
    )
    .unwrap();
    let evidence = ObservationEvidence::new(
        target.clone(),
        DocumentState::new(
            Some(identity),
            DocumentFact::observed(Validity::Valid, stamp(false)),
            DocumentFact::observed(OpenState::Open, stamp(true)),
        ),
        sources,
        DirtyObservation::observed(dirty, stamp(true), witness(Authority::B)),
        Recheck::performed(changes),
    );
    ObservationOutcome::classify(
        request,
        timeline(),
        Some(target),
        Some(evidence),
        vec![],
        vec![],
    )
    .unwrap()
}

fn prior_sample(
    texts: [&str; 3],
    dirty: DirtyState,
    version: Option<&str>,
    request_id: &str,
    missing: Option<Authority>,
    stale: bool,
    changes: Vec<DetectedChange>,
) -> ObservationOutcome {
    prior_sample_with_buffer(
        texts,
        dirty,
        (version, "5"),
        request_id,
        missing,
        stale,
        changes,
    )
}

fn prior_with(
    text: &str,
    dirty: DirtyState,
    version: Option<&str>,
    request_id: &str,
) -> ObservationOutcome {
    prior_sample([text; 3], dirty, version, request_id, None, false, vec![])
}
fn timeline() -> ObservationInterval {
    ObservationInterval::new(1, 2, 1_000_000)
}
fn request(prior: Option<&ObservationOutcome>) -> Result<CloseRequest, BasisError> {
    CloseRequest::new(
        RequestId::new("close-fresh").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
        prior,
    )
}
#[test]
fn null_basis_is_recognition_only_intent_and_selectors_are_checked() {
    assert!(request(None).is_ok());
    for path in ["res://scene.tscn::GDScript_1", "res://scene.tscn"] {
        assert_eq!(
            CloseRequest::new(
                RequestId::new("close-fresh").unwrap(),
                ProjectRoot::new("/fixture/project").unwrap(),
                None,
                ResourcePath::new(path).unwrap(),
                None
            )
            .unwrap_err(),
            BasisError::UnsupportedTarget
        );
    }
}
#[test]
fn eligible_empty_and_safe_syntax_error_prior_do_not_create_a_parse_gate() {
    for text in ["", "extends Node\nfunc broken(\n"] {
        let prior = prior_with(text, DirtyState::Clean, Some("3"), "prior");
        assert!(request(Some(&prior)).is_ok());
    }
}
#[test]
fn dirty_equal_text_and_divergent_or_stale_prior_never_become_null() {
    let dirty = prior_with("extends Node\n", DirtyState::Dirty, Some("3"), "prior");
    assert_eq!(request(Some(&dirty)).unwrap_err(), BasisError::Dirty);
    let divergent = prior_sample(
        ["a", "b", "a"],
        DirtyState::Clean,
        Some("3"),
        "prior",
        None,
        false,
        vec![],
    );
    assert!(request(Some(&divergent)).is_err());
    let stale = prior_sample(
        ["a"; 3],
        DirtyState::Clean,
        Some("3"),
        "prior",
        None,
        true,
        vec![],
    );
    assert_eq!(
        request(Some(&stale)).unwrap_err(),
        BasisError::KnownStaleResource
    );
}
#[test]
fn prior_cannot_choose_another_selector_or_reuse_request_id() {
    let prior = prior_with("a", DirtyState::Clean, Some("3"), "prior");
    for (id, root, session, path) in [
        ("prior", "/fixture/project", SESSION, "res://subject.gd"),
        ("fresh", "/other", SESSION, "res://subject.gd"),
        (
            "fresh",
            "/fixture/project",
            "ffeeddccbbaa99887766554433221100",
            "res://subject.gd",
        ),
        ("fresh", "/fixture/project", SESSION, "res://other.gd"),
    ] {
        assert!(CloseRequest::new(
            RequestId::new(id).unwrap(),
            ProjectRoot::new(root).unwrap(),
            Some(SessionId::new(session).unwrap()),
            ResourcePath::new(path).unwrap(),
            Some(&prior)
        )
        .is_err());
    }
}
#[test]
fn request_debug_retains_no_prior_source() {
    let sentinel = "extends Node\nconst SOURCE_SENTINEL = 42\n";
    let prior = prior_with(sentinel, DirtyState::Clean, Some("3"), "prior");
    assert!(!format!("{:?}", request(Some(&prior)).unwrap()).contains("SOURCE_SENTINEL"));
}
#[test]
fn public_close_input_decodes_real_observation_and_rejects_nested_duplicate() {
    let prior = prior_with("a", DirtyState::Clean, Some("3"), "prior");
    let basis: serde_json::Value =
        serde_json::from_slice(&wire::encode_outcome(&prior).unwrap()).unwrap();
    let payload = json!({"schema_version":1,"request_id":"close-fresh","basis":basis});
    let bytes = serde_json::to_vec(&payload).unwrap();
    assert!(codec::decode_request(
        &bytes,
        ProjectRoot::new("/fixture/project").unwrap(),
        None,
        ResourcePath::new("res://subject.gd").unwrap()
    )
    .unwrap()
    .is_ok());
    let duplicate = String::from_utf8(bytes).unwrap().replace(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
    );
    assert!(codec::decode_request(
        duplicate.as_bytes(),
        ProjectRoot::new("/fixture/project").unwrap(),
        None,
        ResourcePath::new("res://subject.gd").unwrap()
    )
    .is_err());
}
#[test]
fn strict_input_accepts_exact_bound_and_rejects_one_over_and_depth() {
    let valid = br#"{"schema_version":1,"request_id":"close-fresh","basis":null}"#;
    let mut exact = valid.to_vec();
    exact.resize(codec::input_limit(), b' ');
    let decode = |b: &[u8]| {
        codec::decode_request(
            b,
            ProjectRoot::new("/fixture/project").unwrap(),
            None,
            ResourcePath::new("res://subject.gd").unwrap(),
        )
    };
    assert!(decode(&exact).unwrap().is_ok());
    exact.push(b' ');
    assert!(decode(&exact).is_err());
    for bad in [
        b"\xff".as_slice(),
        b"{} {}",
        b"{\"schema_version\":1,\"request_id\":\"close-fresh\",\"basis\":null,\"force\":true}",
    ] {
        assert!(decode(bad).is_err());
    }
    let deep = format!("{}null{}", "[".repeat(33), "]".repeat(33));
    assert!(decode(deep.as_bytes()).is_err());
}

#[test]
fn real_unavailable_and_partial_rechecks_are_basis_refusals_not_malformed_input() {
    let complete = prior_with("a", DirtyState::Clean, Some("3"), "prior");
    let snapshot = complete.snapshot().unwrap();
    for changes in [vec![], vec![DetectedChange::Source(Authority::R)]] {
        let prior = ObservationOutcome::classify(
            ObservationRequest::new(
                RequestId::new("prior").unwrap(),
                ProjectRoot::new("/fixture/project").unwrap(),
                Some(SessionId::new(SESSION).unwrap()),
                ResourcePath::new("res://subject.gd").unwrap(),
            ),
            timeline(),
            Some(snapshot.target().clone()),
            Some(ObservationEvidence::new(
                snapshot.target().clone(),
                snapshot.document().clone(),
                snapshot.sources().clone(),
                snapshot.dirty().clone(),
                Recheck::partial(RecheckReason::Unavailable, changes),
            )),
            vec![],
            vec![],
        )
        .unwrap();
        let basis: serde_json::Value =
            serde_json::from_slice(&wire::encode_outcome(&prior).unwrap()).unwrap();
        assert_eq!(basis["snapshot"]["consistency"]["checks"], "unavailable");
        let payload = json!({"schema_version":1,"request_id":"close-fresh","basis":basis});
        let decode = |value: &serde_json::Value| {
            codec::decode_request(
                &serde_json::to_vec(value).unwrap(),
                ProjectRoot::new("/fixture/project").unwrap(),
                Some(SessionId::new(SESSION).unwrap()),
                ResourcePath::new("res://subject.gd").unwrap(),
            )
        };
        assert!(decode(&payload).unwrap().is_err());
        let mut unsupported = payload.clone();
        unsupported["basis"]["snapshot"]["consistency"]["checks"] = json!("partial");
        assert!(decode(&unsupported).is_err());
        let mut contradictory = payload;
        contradictory["basis"]["snapshot"]["consistency"]["recheck_reason"] =
            serde_json::Value::Null;
        assert!(decode(&contradictory).is_err());
    }
}

#[test]
fn a_valid_prior_from_another_session_is_not_malformed_json() {
    let prior = prior_with("a", DirtyState::Clean, Some("3"), "prior");
    let basis: serde_json::Value =
        serde_json::from_slice(&wire::encode_outcome(&prior).unwrap()).unwrap();
    let payload = json!({"schema_version":1,"request_id":"close-fresh","basis":basis});
    let result = codec::decode_request(
        &serde_json::to_vec(&payload).unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new("ffeeddccbbaa99887766554433221100").unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    assert!(result.unwrap().is_err());
}

#[test]
fn every_explicit_nullable_prior_field_must_remain_present() {
    fn nullable_paths(value: &serde_json::Value, path: String, paths: &mut Vec<(String, String)>) {
        if let Some(object) = value.as_object() {
            for (key, value) in object {
                if value.is_null() {
                    paths.push((path.clone(), key.clone()));
                } else {
                    nullable_paths(value, format!("{path}/{key}"), paths);
                }
            }
        } else if let Some(array) = value.as_array() {
            for (index, value) in array.iter().enumerate() {
                nullable_paths(value, format!("{path}/{index}"), paths);
            }
        }
    }
    let prior = prior_with("a", DirtyState::Clean, Some("3"), "prior");
    let basis: serde_json::Value =
        serde_json::from_slice(&wire::encode_outcome(&prior).unwrap()).unwrap();
    let payload = json!({"schema_version":1,"request_id":"close-fresh","basis":basis});
    let mut paths = Vec::new();
    nullable_paths(&payload, String::new(), &mut paths);
    for (path, key) in paths {
        let mut missing = payload.clone();
        missing
            .pointer_mut(&path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(&key);
        assert!(
            codec::decode_request(
                &serde_json::to_vec(&missing).unwrap(),
                ProjectRoot::new("/fixture/project").unwrap(),
                Some(SessionId::new(SESSION).unwrap()),
                ResourcePath::new("res://subject.gd").unwrap(),
            )
            .is_err(),
            "{path}/{key}"
        );
    }
}
