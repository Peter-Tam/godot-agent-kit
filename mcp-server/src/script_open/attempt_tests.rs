use super::*;
fn request() -> OpenRequest {
    OpenRequest::new(
        RequestId::new("opening").unwrap(),
        ProjectRoot::new("/project").unwrap(),
        None,
        ResourcePath::new("res://test.gd").unwrap(),
    )
    .unwrap()
}
#[test]
fn authorization_loss_is_unknown_and_cannot_authorize_twice() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 1, 1));
    a.state = State::Prepared { cached: false };
    a.authorize().unwrap();
    assert_eq!(a.authorize(), Err(Reason::ProtocolError));
    let result = a.finish(Some(Reason::Timeout), None);
    assert_eq!(result.kind, Kind::EffectsUnknown);
    assert_eq!(result.application, Application::Unknown);
}
#[test]
fn known_cache_effect_survives_protocol_loss() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 1, 1));
    a.effects = Effects::CachePublished;
    let result = a.finish(Some(Reason::ProtocolError), None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.application, Application::PartlyApplied);
}
#[test]
fn first_conflict_survives_later_missing_evidence() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 1, 1));
    a.fail(Reason::ContextChanged);
    a.effects = Effects::DocumentAssociated;
    let result = a.finish(Some(Reason::Disconnected), None);
    assert_eq!(result.reason, Reason::ContextChanged);
    assert_eq!(result.application, Application::Applied);
}
#[test]
fn pre_authorization_interruption_is_not_application() {
    let result = Attempt::new(request(), ObservationInterval::new(1, 1, 1))
        .finish(Some(Reason::Cancelled), None);
    assert_eq!(result.kind, Kind::Refused);
    assert_eq!(result.application, Application::NotApplied);
}
fn decimal(n: u64) -> DecimalCounter {
    DecimalCounter::new(n.to_string()).unwrap()
}
fn target() -> ResolvedTarget {
    let r = request();
    ResolvedTarget::for_request(
        r.observation(),
        r.project_root().clone(),
        FileIdentity::new(decimal(10), decimal(11)),
        SessionId::new("00112233445566778899aabbccddeeff").unwrap(),
        EngineVersion::new(
            "4.7.2.stable.official.ed1daf0bf",
            "ed1daf0bf001b61586d9930840f2f1394092c079",
        )
        .unwrap(),
    )
    .unwrap()
}
fn stamp() -> CollectionStamp {
    CollectionStamp::new(
        ClockId::Editor(target().session_id().clone()),
        decimal(20),
        decimal(21),
        100,
    )
    .unwrap()
}
fn file() -> FileIdentity {
    FileIdentity::new(decimal(4), decimal(5))
}
fn identity() -> DocumentIdentity {
    DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        request().script_path().clone(),
        Some(decimal(1)),
        Some(decimal(2)),
        Some(decimal(3)),
        Some(file()),
    )
    .unwrap()
}
fn snapshot(
    source: &str,
    dirty: DirtyState,
    changes: Vec<DetectedChange>,
    limited_disk: bool,
) -> Observation {
    let t = target();
    let witness = |authority| {
        Witness::new(
            request().script_path().clone(),
            (authority != Authority::D).then(|| decimal(1)),
            (authority == Authority::B).then(|| decimal(2)),
            (authority == Authority::B).then(|| decimal(3)),
            (authority == Authority::D).then(file),
            (authority == Authority::B).then(|| decimal(9)),
        )
    };
    let observed = |authority| {
        SourceObservation::observed(
            authority,
            source.to_owned(),
            if authority == Authority::D {
                CollectionStamp::new(ClockId::Caller, decimal(10), decimal(11), 100).unwrap()
            } else {
                stamp()
            },
            witness(authority),
            Staleness::unknown(),
        )
    };
    let d = if limited_disk {
        SourceObservation::unavailable(Authority::D, SourceReason::TooLarge).unwrap()
    } else {
        observed(Authority::D)
    };
    let evidence = ObservationEvidence::new(
        t.clone(),
        DocumentState::new(
            Some(identity()),
            DocumentFact::observed(Validity::Valid, stamp()),
            DocumentFact::observed(OpenState::Open, stamp()),
        ),
        Sources::new(d, observed(Authority::R), observed(Authority::B)).unwrap(),
        DirtyObservation::observed(dirty, stamp(), witness(Authority::B)),
        Recheck::performed(changes),
    );
    let interval = ObservationInterval::new(1, 2, 1000);
    let value = ObservationOutcome::classify(
        request().observation().clone(),
        interval.clone(),
        Some(t),
        Some(evidence),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    Observation {
        purpose: "verification",
        interval,
        snapshot: value.into_snapshot().unwrap(),
    }
}
fn native(
    stage: &str,
    next: &str,
    binding: &str,
    compilation: &str,
    document: &str,
    code: Option<i64>,
) -> NativeEvidence {
    NativeEvidence {
        stage: Some(stage.into()),
        next: Some(next.into()),
        binding: Some(binding.into()),
        compilation: Some(compilation.into()),
        document: Some(document.into()),
        parse: code,
        script: Some(decimal(1)),
        editor: (document == "association_obtained").then(|| decimal(2)),
        buffer: (document == "association_obtained").then(|| decimal(3)),
        open: Some(document == "association_obtained"),
        discard: Some(false),
        entered: Some(true),
        mode: Some("cold".into()),
        protection: None,
        selection: Some(if document == "association_obtained" {
            "target"
        } else {
            "other"
        }),
    }
}
fn opened(source: &str, parse_code: i64) -> Attempt {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.selected(target()).unwrap();
    a.inspected(false, None).unwrap();
    a.begin_preparation().unwrap();
    a.prepared(
        false,
        crate::project_fs_validation::hex_sha256(source.as_bytes()),
        source.len(),
        file(),
    )
    .unwrap();
    a.no_source_context();
    a.authorize().unwrap();
    a.enter("bind").unwrap();
    a.progress(
        native(
            "bind",
            "compile",
            "new_resource_published",
            "not_started",
            "not_started",
            None,
        ),
        stamp(),
        true,
    )
    .unwrap();
    a.enter("compile").unwrap();
    a.progress(
        native(
            "compile",
            "open",
            "new_resource_published",
            if parse_code == 0 {
                "completed_valid"
            } else {
                "completed_invalid"
            },
            "not_started",
            Some(parse_code),
        ),
        stamp(),
        true,
    )
    .unwrap();
    a.enter("open").unwrap();
    a.progress(
        native(
            "open",
            "",
            "new_resource_published",
            if parse_code == 0 {
                "completed_valid"
            } else {
                "completed_invalid"
            },
            "association_obtained",
            Some(parse_code),
        ),
        stamp(),
        true,
    )
    .unwrap();
    a
}
#[test]
fn native_open_acknowledgment_is_not_independent_verification() {
    let result = opened("extends Node\n", 0).finish(None, None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.reason, Reason::VerificationIncomplete);
}
#[test]
fn parse_invalid_and_empty_sources_can_be_verified_new_opens() {
    for (source, code) in [("", 0), ("extends Node\nfunc broken(:\n", 43)] {
        let mut a = opened(source, code);
        a.resource(Some(false), stamp(), Some(decimal(1)));
        a.start_verification("post_open").unwrap();
        a.verification(
            snapshot(source, DirtyState::Clean, Vec::new(), false),
            Some("unchanged"),
            true,
        )
        .unwrap();
        let result = a.finish(None, None);
        assert_eq!(result.kind, Kind::VerifiedNewlyOpened);
        assert_eq!(
            result.parse.state,
            if code == 0 { "valid" } else { "invalid" }
        );
        assert_eq!(result.history.0, "not_applicable");
    }
}
#[test]
fn equality_does_not_infer_the_separate_resource_edited_flag() {
    let source = "extends Node\n";
    let mut a = opened(source, 0);
    a.start_verification("post_open").unwrap();
    a.verification(
        snapshot(source, DirtyState::Clean, Vec::new(), false),
        Some("unchanged"),
        true,
    )
    .unwrap();
    let result = a.finish(None, None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.reason, Reason::VerificationIncomplete);
}
#[test]
fn dirty_equal_and_source_limited_recognition_do_not_authorize_effects() {
    for limited in [false, true] {
        let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
        a.selected(target()).unwrap();
        a.inspected(true, Some(identity())).unwrap();
        assert_eq!(a.authorize(), Err(Reason::ProtocolError));
        a.resource(Some(true), stamp(), Some(decimal(1)));
        a.start_verification("recognition").unwrap();
        a.verification(
            snapshot("same", DirtyState::Dirty, Vec::new(), limited),
            Some("not_applicable"),
            true,
        )
        .unwrap();
        let result = a.finish(None, None);
        assert_eq!(result.kind, Kind::AlreadyOpenUnchanged);
        assert_eq!(result.application, Application::NotApplied);
        assert_eq!(
            result.snapshot().unwrap().dirty().state(),
            Some(DirtyState::Dirty)
        );
        if limited {
            assert_eq!(
                result.snapshot().unwrap().sources().disk().reason(),
                Some(SourceReason::TooLarge)
            );
        }
    }
}
#[test]
fn same_text_with_a_newer_observed_buffer_version_cannot_restore_new_open_success() {
    let mut a = opened("same", 0);
    a.resource(Some(false), stamp(), Some(decimal(1)));
    a.start_verification("post_open").unwrap();
    a.verification(
        snapshot(
            "same",
            DirtyState::Clean,
            vec![DetectedChange::Source(Authority::B)],
            false,
        ),
        Some("unchanged"),
        true,
    )
    .unwrap();
    let result = a.finish(None, None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.reason, Reason::SourceChanged);
    assert!(result
        .snapshot()
        .unwrap()
        .sources()
        .buffer()
        .invalidated_evidence()
        .is_some());
}
#[test]
fn irreversible_pre_effect_discard_restores_not_applied_but_not_known_cache_effects() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.selected(target()).unwrap();
    a.state = State::Prepared { cached: false };
    a.authorize().unwrap();
    a.enter("bind").unwrap();
    let mut discarded = native(
        "prepared",
        "",
        "failed_before_publication",
        "not_started",
        "not_started",
        None,
    );
    discarded.discard = Some(true);
    a.progress(discarded, stamp(), false).unwrap();
    let result = a.finish(Some(Reason::ContextChanged), None);
    assert_eq!(result.kind, Kind::Refused);
    assert_eq!(result.application, Application::NotApplied);
    let mut a = opened("same", 0);
    let mut late = a.last_native.clone().unwrap();
    late.discard = Some(true);
    a.record(late, stamp()).unwrap();
    let result = a.finish(Some(Reason::Timeout), None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.application, Application::Applied);
}
#[test]
fn cached_reuse_neither_compiles_nor_by_itself_proves_application() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.selected(target()).unwrap();
    a.inspected(false, None).unwrap();
    a.begin_preparation().unwrap();
    a.prepared(
        true,
        crate::project_fs_validation::hex_sha256(b"same"),
        4,
        file(),
    )
    .unwrap();
    a.no_source_context();
    a.authorize().unwrap();
    a.enter("bind").unwrap();
    let mut reused = native(
        "bind",
        "open",
        "reused_existing",
        "not_applicable",
        "not_started",
        None,
    );
    reused.mode = Some("cached".into());
    a.progress(reused, stamp(), true).unwrap();
    assert_eq!(a.enter("compile"), Err(Reason::ProtocolError));
    let result = a.finish(Some(Reason::ProtocolError), None);
    assert_eq!(result.kind, Kind::EffectsUnknown);
    assert_eq!(result.progress.binding.state, StepState::Completed);
    assert_eq!(result.progress.compilation.state, StepState::NotApplicable);
    assert_eq!(result.parse.state, "not_collected");
}
#[test]
fn a_closed_owned_worker_before_dispatch_proves_no_effect_but_loss_after_entry_does_not() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.state = State::Prepared { cached: false };
    a.authorize().unwrap();
    a.closed_worker_before_entry();
    assert_eq!(
        a.finish(Some(Reason::SourceChanged), None).kind,
        Kind::Refused
    );
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.state = State::Prepared { cached: false };
    a.authorize().unwrap();
    a.enter("bind").unwrap();
    a.closed_worker_before_entry();
    assert_eq!(
        a.finish(Some(Reason::Disconnected), None).kind,
        Kind::EffectsUnknown
    );
}
#[test]
fn out_of_order_receipts_cannot_introduce_or_erase_effect_knowledge() {
    let mut a = Attempt::new(request(), ObservationInterval::new(1, 2, 1000));
    a.selected(target()).unwrap();
    a.state = State::Prepared { cached: false };
    a.authorize().unwrap();
    a.enter("bind").unwrap();
    assert_eq!(
        a.progress(
            native(
                "compile",
                "open",
                "new_resource_published",
                "completed_valid",
                "not_started",
                Some(0)
            ),
            stamp(),
            true
        ),
        Err(Reason::ProtocolError)
    );
    assert_eq!(
        a.finish(Some(Reason::ProtocolError), None).kind,
        Kind::EffectsUnknown
    );
    let mut a = opened("same", 0);
    let mut wrong = a.last_native.clone().unwrap();
    wrong.script = Some(decimal(999));
    assert_eq!(a.record(wrong, stamp()), Err(Reason::TargetChanged));
    let result = a.finish(Some(Reason::ProtocolError), None);
    assert_eq!(result.kind, Kind::AppliedUnverified);
    assert_eq!(result.application, Application::Applied);
}
