use godot_agent_kit::observation::*;

const SESSION: &str = "00112233445566778899aabbccddeeff";
const REPLACEMENT_SESSION: &str = "ffeeddccbbaa99887766554433221100";
const SCRIPT: &str = "res://scripts/subject.gd";

fn decimal(value: &str) -> DecimalCounter {
    DecimalCounter::new(value).unwrap()
}
fn request(session: Option<&str>) -> ObservationRequest {
    ObservationRequest::new(
        RequestId::new("example-1").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        session.map(|s| SessionId::new(s).unwrap()),
        ResourcePath::new(SCRIPT).unwrap(),
    )
}
fn target(request: &ObservationRequest) -> ResolvedTarget {
    ResolvedTarget::for_request(
        request,
        ProjectRoot::new("/fixture/project").unwrap(),
        FileIdentity::new(decimal("184467440737095516160"), decimal("7")),
        SessionId::new(SESSION).unwrap(),
        EngineVersion::new("4.7.2.stable.official", "ed1daf0bf").unwrap(),
    )
    .unwrap()
}
fn interval() -> ObservationInterval {
    ObservationInterval::new(1_750_000_000_000, 1_750_000_000_002, 100)
}
fn caller_stamp(tick: &str, received: u64) -> CollectionStamp {
    CollectionStamp::new(ClockId::Caller, decimal(tick), decimal(tick), received).unwrap()
}
fn editor_stamp(tick: &str, received: u64) -> CollectionStamp {
    CollectionStamp::new(
        ClockId::Editor(SessionId::new(SESSION).unwrap()),
        decimal(tick),
        decimal(tick),
        received,
    )
    .unwrap()
}
fn identity(open: Option<OpenState>) -> DocumentIdentity {
    DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        ResourcePath::new(SCRIPT).unwrap(),
        Some(decimal("18446744073709551618")),
        if open == Some(OpenState::Open) {
            Some(decimal("1001"))
        } else {
            None
        },
        if open == Some(OpenState::Open) {
            Some(decimal("2001"))
        } else {
            None
        },
        Some(FileIdentity::new(decimal("12"), decimal("345"))),
    )
    .unwrap()
}
fn witness(id: &DocumentIdentity, surface: Authority) -> Witness {
    Witness::new(
        id.resource_path().clone(),
        if surface != Authority::D {
            id.script_instance_id().cloned()
        } else {
            None
        },
        if surface == Authority::B {
            id.editor_instance_id().cloned()
        } else {
            None
        },
        if surface == Authority::B {
            id.buffer_instance_id().cloned()
        } else {
            None
        },
        if surface == Authority::D {
            id.disk_file_id().cloned()
        } else {
            None
        },
        Some(decimal("17")),
    )
}
fn observed(id: &DocumentIdentity, surface: Authority, text: &str) -> SourceObservation {
    let stamp = match surface {
        Authority::D => caller_stamp("100000000000000000000000000000000000001", 11),
        Authority::R => editor_stamp("200000000000000000000000000000000000007", 17),
        Authority::B => editor_stamp("200000000000000000000000000000000000011", 29),
    };
    SourceObservation::observed(
        surface,
        text.to_owned(),
        stamp,
        witness(id, surface),
        Staleness::unknown(),
    )
}
fn source(
    id: &DocumentIdentity,
    surface: Authority,
    text: Option<&str>,
    open: Option<OpenState>,
) -> SourceObservation {
    match (surface, text, open) {
        (_, Some(text), _) => observed(id, surface, text),
        (Authority::B, None, Some(OpenState::NotOpen)) => SourceObservation::closed_buffer(),
        (Authority::D, None, _) => {
            SourceObservation::unavailable(surface, SourceReason::DiskMissing).unwrap()
        }
        (Authority::R, None, _) => {
            SourceObservation::unavailable(surface, SourceReason::ResourceNotLoaded).unwrap()
        }
        (Authority::B, None, None) => {
            SourceObservation::unavailable(surface, SourceReason::OpenStateUnknown).unwrap()
        }
        (Authority::B, None, _) => {
            SourceObservation::unavailable(surface, SourceReason::BufferUnreadable).unwrap()
        }
    }
}
fn evidence(
    open: Option<OpenState>,
    disk: Option<&str>,
    resource: Option<&str>,
    buffer: Option<&str>,
    dirty: Option<DirtyState>,
    changes: Vec<DetectedChange>,
) -> (ObservationRequest, ResolvedTarget, ObservationEvidence) {
    let request = request(Some(SESSION));
    let target = target(&request);
    let id = identity(open);
    let document = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(
            Validity::Valid,
            caller_stamp("100000000000000000000000000000000000002", 13),
        ),
        match open {
            Some(state) => DocumentFact::observed(
                state,
                editor_stamp("200000000000000000000000000000000000003", 15),
            ),
            None => DocumentFact::unknown(FactReason::OpenStateUnknown),
        },
    );
    let sources = Sources::new(
        source(&id, Authority::D, disk, open),
        source(&id, Authority::R, resource, open),
        source(&id, Authority::B, buffer, open),
    )
    .unwrap();
    let dirty = match (open, dirty) {
        (Some(OpenState::NotOpen), _) => DirtyObservation::closed_document(),
        (_, Some(state)) => DirtyObservation::observed(
            state,
            editor_stamp("200000000000000000000000000000000000012", 31),
            witness(&id, Authority::B),
        ),
        (None, None) => DirtyObservation::unavailable(DirtyReason::OpenStateUnknown).unwrap(),
        _ => DirtyObservation::unavailable(DirtyReason::DirtyAttributionUnavailable).unwrap(),
    };
    (
        request,
        target.clone(),
        ObservationEvidence::new(
            target,
            document,
            sources,
            dirty,
            Recheck::performed(changes),
        ),
    )
}
fn classify(
    open: Option<OpenState>,
    disk: Option<&str>,
    resource: Option<&str>,
    buffer: Option<&str>,
    dirty: Option<DirtyState>,
    changes: Vec<DetectedChange>,
    signals: Vec<TerminalFailure>,
) -> ObservationOutcome {
    let (request, target, evidence) = evidence(open, disk, resource, buffer, dirty, changes);
    ObservationOutcome::classify(
        request,
        interval(),
        Some(target),
        Some(evidence),
        signals,
        vec![],
    )
    .unwrap()
}
fn pairs(result: &ObservationOutcome) -> [Comparison; 3] {
    let pairs = result.snapshot().unwrap().comparisons();
    [
        pairs.disk_resource(),
        pairs.disk_buffer(),
        pairs.resource_buffer(),
    ]
}

#[test]
fn all_eleven_semantic_vectors_use_attributed_independent_full_evidence() {
    use Agreement::{Agree, Divergent, Unknown as AgreementUnknown};
    use Comparison::{Different, Equal, Unknown as ComparisonUnknown};
    use DirtyState::*;
    use OpenState::*;
    use OutcomeKind::{CompleteObservation, LimitedObservation, NotOpen as NotOpenOutcome};
    let cases = [
        (
            "clean",
            Some(Open),
            Some("S"),
            Some("S"),
            Some("S"),
            Some(Clean),
            None,
            CompleteObservation,
            [Equal, Equal, Equal],
            Agree,
        ),
        (
            "dirty-divergent",
            Some(Open),
            Some("S"),
            Some("S"),
            Some("U"),
            Some(Dirty),
            None,
            CompleteObservation,
            [Equal, Different, Different],
            Divergent,
        ),
        (
            "dirty-equal",
            Some(Open),
            Some("S"),
            Some("S"),
            Some("S"),
            Some(Dirty),
            None,
            CompleteObservation,
            [Equal, Equal, Equal],
            Agree,
        ),
        (
            "dirty-unknown",
            Some(Open),
            Some("S"),
            Some("S"),
            Some("S"),
            None,
            None,
            LimitedObservation,
            [Equal, Equal, Equal],
            Agree,
        ),
        (
            "missing-R-with-known-divergence",
            Some(Open),
            Some("S"),
            None,
            Some("U"),
            Some(Dirty),
            None,
            LimitedObservation,
            [ComparisonUnknown, Different, ComparisonUnknown],
            Divergent,
        ),
        (
            "closed-unloaded",
            Some(NotOpen),
            Some("S"),
            None,
            None,
            None,
            None,
            NotOpenOutcome,
            [ComparisonUnknown, ComparisonUnknown, ComparisonUnknown],
            AgreementUnknown,
        ),
        (
            "closed-cached",
            Some(NotOpen),
            Some("S"),
            Some("S"),
            None,
            None,
            None,
            NotOpenOutcome,
            [Equal, ComparisonUnknown, ComparisonUnknown],
            AgreementUnknown,
        ),
        (
            "empty-observed",
            Some(Open),
            Some(""),
            Some(""),
            Some(""),
            Some(Clean),
            None,
            CompleteObservation,
            [Equal, Equal, Equal],
            Agree,
        ),
        (
            "line-endings",
            Some(Open),
            Some("x\r\n"),
            Some("x\n"),
            Some("x\n"),
            Some(Clean),
            None,
            CompleteObservation,
            [Different, Different, Equal],
            Divergent,
        ),
        (
            "invalidated-buffer",
            Some(Open),
            Some("S"),
            Some("S"),
            Some("U"),
            Some(Dirty),
            Some(Authority::B),
            LimitedObservation,
            [Equal, ComparisonUnknown, ComparisonUnknown],
            AgreementUnknown,
        ),
        (
            "open-unknown",
            None,
            Some("S"),
            None,
            None,
            None,
            None,
            LimitedObservation,
            [ComparisonUnknown, ComparisonUnknown, ComparisonUnknown],
            AgreementUnknown,
        ),
    ];
    for (
        name,
        open,
        d,
        r,
        b,
        dirty,
        invalidated,
        expected_outcome,
        expected_pairs,
        expected_agreement,
    ) in cases
    {
        let result = classify(
            open,
            d,
            r,
            b,
            dirty,
            invalidated.map_or_else(Vec::new, |surface| vec![DetectedChange::Source(surface)]),
            vec![],
        );
        let snapshot = result.snapshot().unwrap();
        assert_eq!(result.outcome(), expected_outcome, "{name}");
        assert_eq!(pairs(&result), expected_pairs, "{name}");
        assert_eq!(snapshot.agreement(), expected_agreement, "{name}");
        assert_eq!(snapshot.sources().disk().text(), d, "{name}");
        assert_eq!(snapshot.sources().resource().text(), r, "{name}");
        assert_eq!(
            snapshot.sources().buffer().text(),
            if invalidated.is_some() { None } else { b },
            "{name}"
        );
        assert_eq!(snapshot.dirty().state(), dirty, "{name}");
        assert_eq!(
            snapshot.document().open_state().value(),
            open.as_ref(),
            "{name}"
        );
        assert!(!snapshot.consistency().atomic(), "{name}");
        assert_eq!(snapshot.consistency().checks(), Checks::Performed, "{name}");
        assert_eq!(
            snapshot.consistency().stability(),
            if invalidated.is_some() {
                Stability::Changed
            } else {
                Stability::Unknown
            },
            "{name}"
        );
        assert_eq!(
            snapshot.sources().disk().collection().unwrap().clock_id(),
            &ClockId::Caller,
            "{name}"
        );
        if let Some(invalidated) = invalidated {
            assert_eq!(invalidated, Authority::B);
            let old = snapshot.sources().buffer().invalidated_evidence().unwrap();
            assert_eq!(old.text(), "U");
            assert_eq!(old.reason(), SourceReason::SourceChanged);
            assert_eq!(old.collection().received_elapsed_us(), 29);
            assert_eq!(old.witness().buffer_instance_id().unwrap().as_str(), "2001");
        } else if r.is_some() {
            assert_eq!(
                snapshot
                    .sources()
                    .resource()
                    .collection()
                    .unwrap()
                    .clock_id(),
                &ClockId::Editor(SessionId::new(SESSION).unwrap()),
                "{name}"
            );
        }
        if open == Some(NotOpen) {
            assert_eq!(
                snapshot.sources().buffer().availability(),
                Availability::NotApplicable
            );
            assert_eq!(
                snapshot.sources().buffer().reason(),
                Some(SourceReason::DocumentNotOpen)
            );
            assert_eq!(
                snapshot.dirty().reason(),
                Some(DirtyReason::DocumentNotOpen)
            );
            assert!(snapshot
                .document()
                .identity()
                .unwrap()
                .editor_instance_id()
                .is_none());
        }
        if r.is_none() {
            assert_eq!(
                snapshot.sources().resource().reason(),
                Some(SourceReason::ResourceNotLoaded)
            );
        }
    }
}

#[test]
fn every_pair_and_partial_divergence_preserve_exact_values() {
    for (d, r, b, expected) in [
        (
            "a",
            "a",
            "b",
            [
                Comparison::Equal,
                Comparison::Different,
                Comparison::Different,
            ],
        ),
        (
            "a",
            "b",
            "a",
            [
                Comparison::Different,
                Comparison::Equal,
                Comparison::Different,
            ],
        ),
        (
            "b",
            "a",
            "a",
            [
                Comparison::Different,
                Comparison::Different,
                Comparison::Equal,
            ],
        ),
        ("a", "b", "c", [Comparison::Different; 3]),
        ("x", "x", "x", [Comparison::Equal; 3]),
        ("x", "x ", "x\n", [Comparison::Different; 3]),
        (
            "\u{feff}λ\r\n",
            "\u{feff}λ\r\n",
            "\u{feff}λ\r\n",
            [Comparison::Equal; 3],
        ),
    ] {
        let result = classify(
            Some(OpenState::Open),
            Some(d),
            Some(r),
            Some(b),
            Some(DirtyState::Clean),
            vec![],
            vec![],
        );
        assert_eq!(pairs(&result), expected);
        assert_eq!(result.outcome(), OutcomeKind::CompleteObservation);
        assert_eq!(
            result.snapshot().unwrap().agreement(),
            if d == r && r == b {
                Agreement::Agree
            } else {
                Agreement::Divergent
            }
        );
    }
    for absent in [Authority::D, Authority::R, Authority::B] {
        let result = classify(
            Some(OpenState::Open),
            (absent != Authority::D).then_some("a"),
            (absent != Authority::R).then_some("b"),
            (absent != Authority::B).then_some("a"),
            Some(DirtyState::Clean),
            vec![],
            vec![],
        );
        assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
        assert_eq!(
            result.snapshot().unwrap().agreement(),
            if absent == Authority::B {
                Agreement::Divergent
            } else if absent == Authority::R {
                Agreement::Unknown
            } else {
                Agreement::Divergent
            }
        );
    }
}

#[test]
fn source_limit_is_per_authority_and_inclusive_in_utf8_bytes() {
    let exact = "λ".repeat(SOURCE_LIMIT_BYTES / 2);
    let over = format!("{exact}x");
    let result = classify(
        Some(OpenState::Open),
        Some(&exact),
        Some(&exact),
        Some(&exact),
        Some(DirtyState::Clean),
        vec![],
        vec![],
    );
    assert_eq!(result.outcome(), OutcomeKind::CompleteObservation);
    assert_eq!(
        result.snapshot().unwrap().sources().disk().text(),
        Some(exact.as_str())
    );
    for authority in [Authority::D, Authority::R, Authority::B] {
        let (d, r, b) = match authority {
            Authority::D => (over.as_str(), "R", "B"),
            Authority::R => ("D", over.as_str(), "B"),
            Authority::B => ("D", "R", over.as_str()),
        };
        let result = classify(
            Some(OpenState::Open),
            Some(d),
            Some(r),
            Some(b),
            Some(DirtyState::Clean),
            vec![],
            vec![],
        );
        let snapshot = result.snapshot().unwrap();
        assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
        let limited = match authority {
            Authority::D => snapshot.sources().disk(),
            Authority::R => snapshot.sources().resource(),
            Authority::B => snapshot.sources().buffer(),
        };
        assert_eq!(limited.availability(), Availability::Unavailable);
        assert_eq!(limited.reason(), Some(SourceReason::TooLarge));
        assert_eq!(limited.text(), None);
        assert_eq!(limited.invalidated_evidence(), None);
        assert_eq!(snapshot.dirty().state(), Some(DirtyState::Clean));
        assert!(snapshot
            .diagnostics()
            .iter()
            .any(|d| d.code() == DiagnosticCode::TooLarge));
        for (other, value) in [(Authority::D, d), (Authority::R, r), (Authority::B, b)] {
            if other != authority {
                let fact = match other {
                    Authority::D => snapshot.sources().disk(),
                    Authority::R => snapshot.sources().resource(),
                    Authority::B => snapshot.sources().buffer(),
                };
                assert_eq!(fact.text(), Some(value));
            }
        }
    }
    let result = classify(
        Some(OpenState::NotOpen),
        Some(&over),
        None,
        None,
        None,
        vec![],
        vec![],
    );
    assert_eq!(result.outcome(), OutcomeKind::NotOpen);
    assert_eq!(
        result.snapshot().unwrap().sources().disk().reason(),
        Some(SourceReason::TooLarge)
    );
}

#[test]
fn known_stale_requires_real_version_evidence_and_is_not_divergence() {
    assert_eq!(
        Staleness::known_stale(vec![]),
        Err(EvidenceError::InvalidStaleness)
    );
    let request = request(Some(SESSION));
    let target = target(&request);
    let doc = identity(Some(OpenState::Open));
    let newer_witness = Witness::new(
        doc.resource_path().clone(),
        None,
        None,
        None,
        doc.disk_file_id().cloned(),
        Some(decimal("18")),
    );
    assert_eq!(
        StalenessEvidence::unapplied_change(
            decimal("9"),
            Authority::D,
            decimal("9"),
            caller_stamp("45", 19),
            newer_witness.clone()
        ),
        Err(EvidenceError::InvalidStaleness)
    );
    assert_eq!(
        StalenessEvidence::unapplied_change(
            decimal("10"),
            Authority::D,
            decimal("9"),
            caller_stamp("45", 19),
            newer_witness.clone()
        ),
        Err(EvidenceError::InvalidStaleness)
    );
    let later = StalenessEvidence::unapplied_change(
        decimal("17"),
        Authority::D,
        decimal("18"),
        caller_stamp("40", 16),
        newer_witness,
    )
    .unwrap();
    let sources = Sources::new(
        observed(&doc, Authority::D, "S"),
        SourceObservation::observed(
            Authority::R,
            "S".into(),
            editor_stamp("44", 18),
            witness(&doc, Authority::R),
            Staleness::known_stale(vec![later]).unwrap(),
        ),
        observed(&doc, Authority::B, "S"),
    )
    .unwrap();
    let document = DocumentState::new(
        Some(doc.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 2)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 3)),
    );
    let dirty = DirtyObservation::observed(
        DirtyState::Clean,
        editor_stamp("3", 4),
        witness(&doc, Authority::B),
    );
    let result = ObservationOutcome::classify(
        request,
        interval(),
        Some(target.clone()),
        Some(ObservationEvidence::new(
            target,
            document,
            sources,
            dirty,
            Recheck::performed(vec![]),
        )),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::CompleteObservation);
    assert_eq!(result.snapshot().unwrap().agreement(), Agreement::Agree);
    assert_eq!(
        result
            .snapshot()
            .unwrap()
            .sources()
            .resource()
            .staleness()
            .unwrap()
            .evidence()
            .unwrap()
            .len(),
        1
    );
    let divergent = classify(
        Some(OpenState::Open),
        Some("S"),
        Some("U"),
        Some("S"),
        Some(DirtyState::Clean),
        vec![],
        vec![],
    );
    assert_eq!(
        divergent
            .snapshot()
            .unwrap()
            .sources()
            .resource()
            .staleness(),
        Some(&Staleness::unknown())
    );
    assert_eq!(
        divergent.snapshot().unwrap().agreement(),
        Agreement::Divergent
    );
}

#[test]
fn same_document_rechecks_invalidate_only_affected_facts() {
    let result = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![DetectedChange::Source(Authority::R), DetectedChange::Dirty],
        vec![],
    );
    let snapshot = result.snapshot().unwrap();
    assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(snapshot.sources().resource().text(), None);
    assert_eq!(
        snapshot
            .sources()
            .resource()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "R"
    );
    assert_eq!(snapshot.dirty().state(), None);
    assert_eq!(
        snapshot.dirty().invalidated_evidence().unwrap().state(),
        DirtyState::Dirty
    );
    assert_eq!(snapshot.sources().disk().text(), Some("D"));
    assert_eq!(snapshot.sources().buffer().text(), Some("B"));
    assert_eq!(
        pairs(&result),
        [
            Comparison::Unknown,
            Comparison::Different,
            Comparison::Unknown
        ]
    );
    assert_eq!(snapshot.agreement(), Agreement::Divergent);
    assert_eq!(snapshot.consistency().stability(), Stability::Changed);
}

#[test]
fn replacement_and_close_never_become_a_new_current_document() {
    let result = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![DetectedChange::DocumentIdentityReplaced],
        vec![],
    );
    let snapshot = result.snapshot().unwrap();
    assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(snapshot.agreement(), Agreement::Unknown);
    for (source, text) in [
        (snapshot.sources().disk(), "D"),
        (snapshot.sources().resource(), "R"),
        (snapshot.sources().buffer(), "B"),
    ] {
        assert_eq!(source.text(), None);
        assert_eq!(source.invalidated_evidence().unwrap().text(), text);
        assert_eq!(source.reason(), Some(SourceReason::IdentityChanged));
    }
    assert_eq!(snapshot.document().open_state().value(), None);
    assert_eq!(
        snapshot
            .document()
            .open_state()
            .invalidated_evidence()
            .unwrap()
            .value(),
        &OpenState::Open
    );
    assert_eq!(snapshot.target().session_id().as_str(), SESSION);
    let closing = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Clean),
        vec![DetectedChange::DocumentClosed],
        vec![],
    );
    assert_eq!(closing.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(
        closing.snapshot().unwrap().document().open_state().value(),
        None
    );
    assert_eq!(
        closing.snapshot().unwrap().sources().disk().text(),
        Some("D")
    );
    assert_eq!(
        closing.snapshot().unwrap().sources().resource().text(),
        Some("R")
    );
    assert_eq!(
        closing
            .snapshot()
            .unwrap()
            .sources()
            .buffer()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "B"
    );
    let session_replaced = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Clean),
        vec![DetectedChange::SessionReplaced],
        vec![TerminalFailure::DisconnectedEditor],
    );
    assert_eq!(session_replaced.outcome(), OutcomeKind::DisconnectedEditor);
    assert_eq!(
        session_replaced
            .snapshot()
            .unwrap()
            .target()
            .session_id()
            .as_str(),
        SESSION
    );
    assert_eq!(
        session_replaced
            .snapshot()
            .unwrap()
            .sources()
            .resource()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "R"
    );
}

#[test]
fn recheck_unavailable_never_claims_complete_or_closed_success() {
    for open in [OpenState::Open, OpenState::NotOpen] {
        let (request, target, _) = evidence(
            Some(open),
            Some("S"),
            Some("S"),
            (open == OpenState::Open).then_some("S"),
            Some(DirtyState::Clean),
            vec![],
        );
        let id = identity(Some(open));
        let facts = DocumentState::new(
            Some(id.clone()),
            DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
            DocumentFact::observed(open, editor_stamp("1", 2)),
        );
        let sources = Sources::new(
            observed(&id, Authority::D, "S"),
            observed(&id, Authority::R, "S"),
            if open == OpenState::Open {
                observed(&id, Authority::B, "S")
            } else {
                SourceObservation::closed_buffer()
            },
        )
        .unwrap();
        let dirty = if open == OpenState::Open {
            DirtyObservation::observed(
                DirtyState::Clean,
                editor_stamp("2", 3),
                witness(&id, Authority::B),
            )
        } else {
            DirtyObservation::closed_document()
        };
        let outcome = ObservationOutcome::classify(
            request,
            interval(),
            Some(target.clone()),
            Some(ObservationEvidence::new(
                target,
                facts,
                sources,
                dirty,
                Recheck::unavailable(RecheckReason::DeadlineExceeded),
            )),
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(outcome.outcome(), OutcomeKind::Timeout);
        assert_eq!(
            outcome.snapshot().unwrap().consistency().checks(),
            Checks::Unavailable
        );
        assert_eq!(
            outcome.snapshot().unwrap().consistency().recheck_reason(),
            Some(RecheckReason::DeadlineExceeded)
        );
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| d.code() == DiagnosticCode::RecheckUnavailable));
    }
}

#[test]
fn terminal_precedence_and_source_suppression_hold_even_for_invalidated_text() {
    let full = || {
        evidence(
            Some(OpenState::Open),
            Some("private-D"),
            Some("private-R"),
            Some("private-B"),
            Some(DirtyState::Dirty),
            vec![DetectedChange::Source(Authority::B)],
        )
    };
    let (request, target, evidence) = full();
    let denied = ObservationOutcome::classify(
        request,
        interval(),
        Some(target),
        Some(evidence),
        vec![
            TerminalFailure::Timeout,
            TerminalFailure::DeniedAccess,
            TerminalFailure::DisconnectedEditor,
        ],
        vec![],
    )
    .unwrap();
    assert_eq!(denied.outcome(), OutcomeKind::DeniedAccess);
    assert!(denied.snapshot().is_none());
    assert!(denied.resolved_target().is_none());
    assert!(denied.selection().is_none());
    assert!(denied
        .diagnostics()
        .iter()
        .all(|d| !d.message().contains("private") && !d.action().contains("private")));
    let (request, target, evidence) = full();
    let disconnect = ObservationOutcome::classify(
        request,
        interval(),
        Some(target),
        Some(evidence),
        vec![
            TerminalFailure::Timeout,
            TerminalFailure::DisconnectedEditor,
        ],
        vec![],
    )
    .unwrap();
    assert_eq!(disconnect.outcome(), OutcomeKind::DisconnectedEditor);
    assert_eq!(
        disconnect.snapshot().unwrap().sources().disk().text(),
        Some("private-D")
    );
    assert_eq!(
        disconnect
            .snapshot()
            .unwrap()
            .sources()
            .buffer()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "private-B"
    );
    for failure in [
        TerminalFailure::Timeout,
        TerminalFailure::Cancelled,
        TerminalFailure::ProtocolError,
        TerminalFailure::EditorUnavailable,
    ] {
        let (request, target, evidence) = full();
        let result = ObservationOutcome::classify(
            request,
            interval(),
            Some(target),
            Some(evidence),
            vec![failure.clone()],
            vec![],
        )
        .unwrap();
        assert_eq!(result.outcome(), failure_kind(failure));
        assert_ne!(result.outcome(), OutcomeKind::CompleteObservation);
        if result.outcome() == OutcomeKind::EditorUnavailable {
            assert!(result.snapshot().is_none());
        }
    }
    let selection = Selection::ambiguous(vec![
        SessionId::new(SESSION).unwrap(),
        SessionId::new(REPLACEMENT_SESSION).unwrap(),
    ])
    .unwrap();
    let (mut request, target, evidence) = full();
    request = ObservationRequest::new(
        request.request_id().clone(),
        request.project_root().clone(),
        None,
        request.script_path().clone(),
    );
    let ambiguous = ObservationOutcome::classify(
        request,
        interval(),
        Some(target),
        Some(evidence),
        vec![TerminalFailure::AmbiguousTarget(selection)],
        vec![],
    )
    .unwrap();
    assert_eq!(ambiguous.outcome(), OutcomeKind::AmbiguousTarget);
    assert!(ambiguous.snapshot().is_none());
    assert!(ambiguous.resolved_target().is_none());
    assert_eq!(ambiguous.selection().unwrap().candidate_sessions().len(), 2);
    assert_eq!(
        ambiguous.selection().unwrap().missing_selector(),
        MissingSelector::SessionId
    );
}
fn failure_kind(failure: TerminalFailure) -> OutcomeKind {
    match failure {
        TerminalFailure::Timeout => OutcomeKind::Timeout,
        TerminalFailure::Cancelled => OutcomeKind::Cancelled,
        TerminalFailure::ProtocolError => OutcomeKind::ProtocolError,
        TerminalFailure::EditorUnavailable => OutcomeKind::EditorUnavailable,
        _ => unreachable!(),
    }
}

#[test]
fn invalid_identity_session_clock_timing_and_reason_are_rejected() {
    assert_eq!(RequestId::new(""), Err(EvidenceError::InvalidRequestId));
    assert_eq!(
        RequestId::new("a".repeat(65)),
        Err(EvidenceError::InvalidRequestId)
    );
    assert_eq!(
        RequestId::new("bad value"),
        Err(EvidenceError::InvalidRequestId)
    );
    assert_eq!(
        SessionId::new(SESSION.to_uppercase()),
        Err(EvidenceError::InvalidSessionId)
    );
    assert_eq!(SessionId::new("abc"), Err(EvidenceError::InvalidSessionId));
    for path in [
        "relative",
        "/tmp/../project",
        "/tmp//project",
        "/tmp/",
        "/tmp/\0bad",
    ] {
        assert_eq!(
            ProjectRoot::new(path),
            Err(EvidenceError::InvalidProjectRoot),
            "{path:?}"
        );
    }
    assert_eq!(
        ProjectRoot::new(format!("/{}", "a".repeat(1024))),
        Err(EvidenceError::InvalidProjectRoot)
    );
    for path in [
        "res://",
        "res://../escape.gd",
        "res://a//b.gd",
        "res://a/./b.gd",
        "res://b.gd?x",
        "res://b.gd#x",
        "res://b\\c.gd",
        "user://b.gd",
        "/tmp/b.gd",
        "res://scene.tscn::Other_1",
        "res://scene.tscn::GDScript_",
        "res://b.gd\n",
    ] {
        assert_eq!(
            ResourcePath::new(path),
            Err(EvidenceError::InvalidResourcePath),
            "{path:?}"
        );
    }
    assert_eq!(
        ResourcePath::new(format!("res://{}.gd", "a".repeat(2048))),
        Err(EvidenceError::InvalidResourcePath)
    );
    assert_eq!(
        ResourcePath::new("res://屋/Σ.gd").unwrap().as_str(),
        "res://屋/Σ.gd"
    );
    assert_eq!(
        ResourcePath::new("res://scene.tscn::GDScript_abc")
            .unwrap()
            .kind(),
        Some(ScriptKind::BuiltinGdscript)
    );
    for number in ["", "01", "-1", "+2", "2.0", "1e3", "１２", "1\n"] {
        assert_eq!(
            DecimalCounter::new(number),
            Err(EvidenceError::InvalidDecimal),
            "{number:?}"
        );
    }
    assert!(
        decimal("99999999999999999999999999999999999999999999999999999")
            > decimal("10000000000000000000000000000000000000000000000000000")
    );
    assert_eq!(
        CollectionStamp::new(ClockId::Caller, decimal("2"), decimal("1"), 0),
        Err(EvidenceError::InvalidTiming)
    );
    assert_eq!(
        SourceObservation::unavailable(Authority::D, SourceReason::ResourceNotLoaded),
        Err(EvidenceError::InvalidReason)
    );
    assert_eq!(
        SourceObservation::unavailable(Authority::B, SourceReason::DocumentNotOpen),
        Err(EvidenceError::InvalidReason)
    );
    assert_eq!(
        DirtyObservation::unavailable(DirtyReason::DocumentNotOpen),
        Err(EvidenceError::InvalidReason)
    );
    assert_eq!(
        Selection::ambiguous(vec![SessionId::new(SESSION).unwrap()]),
        Err(EvidenceError::InvalidSelection)
    );
    assert_eq!(
        Selection::ambiguous(vec![SessionId::new(SESSION).unwrap(); 2]),
        Err(EvidenceError::InvalidSelection)
    );
    let (request, target, _) = evidence(
        Some(OpenState::Open),
        Some("S"),
        Some("S"),
        Some("S"),
        Some(DirtyState::Clean),
        vec![],
    );
    assert_eq!(
        ResolvedTarget::for_request(
            &request,
            ProjectRoot::new("/other/project").unwrap(),
            FileIdentity::new(decimal("1"), decimal("2")),
            SessionId::new(SESSION).unwrap(),
            EngineVersion::new("4.7", "abcd").unwrap()
        ),
        Err(EvidenceError::WrongTarget)
    );
    assert_eq!(
        ResolvedTarget::for_request(
            &request,
            target.project_root().clone(),
            target.project_file_id().clone(),
            SessionId::new(REPLACEMENT_SESSION).unwrap(),
            target.godot_version().clone()
        ),
        Err(EvidenceError::WrongTarget)
    );
}

#[test]
fn unrelated_clock_domains_are_not_compared_but_wrong_session_and_late_facts_fail() {
    let result = classify(
        Some(OpenState::Open),
        Some("a"),
        Some("a"),
        Some("a"),
        Some(DirtyState::Clean),
        vec![],
        vec![],
    );
    assert_eq!(result.outcome(), OutcomeKind::CompleteObservation);
    let (request, target, _) = evidence(
        Some(OpenState::Open),
        Some("S"),
        Some("S"),
        Some("S"),
        Some(DirtyState::Clean),
        vec![],
    );
    let id = identity(Some(OpenState::Open));
    let create = |stamp: CollectionStamp| {
        ObservationEvidence::new(
            target.clone(),
            DocumentState::new(
                Some(id.clone()),
                DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
                DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
            ),
            Sources::new(
                observed(&id, Authority::D, "S"),
                SourceObservation::observed(
                    Authority::R,
                    "S".into(),
                    stamp,
                    witness(&id, Authority::R),
                    Staleness::unknown(),
                ),
                observed(&id, Authority::B, "S"),
            )
            .unwrap(),
            DirtyObservation::observed(
                DirtyState::Clean,
                editor_stamp("3", 3),
                witness(&id, Authority::B),
            ),
            Recheck::performed(vec![]),
        )
    };
    let wrong = CollectionStamp::new(
        ClockId::Editor(SessionId::new(REPLACEMENT_SESSION).unwrap()),
        decimal("1"),
        decimal("2"),
        5,
    )
    .unwrap();
    assert_eq!(
        ObservationOutcome::classify(
            request.clone(),
            interval(),
            Some(target.clone()),
            Some(create(wrong)),
            vec![],
            vec![]
        ),
        Err(EvidenceError::WrongClock)
    );
    let late = CollectionStamp::new(
        ClockId::Editor(SessionId::new(SESSION).unwrap()),
        decimal("1"),
        decimal("2"),
        101,
    )
    .unwrap();
    let late_evidence = create(late);
    assert_eq!(
        ObservationOutcome::classify(
            request,
            interval(),
            Some(target),
            Some(late_evidence),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidTiming)
    );
}

#[test]
fn invalid_document_attribution_and_nullable_fields_cannot_fake_a_complete_result() {
    let req = request(Some(SESSION));
    let selected = target(&req);
    let id = identity(Some(OpenState::Open));
    let wrong_path = ResourcePath::new("res://other.gd").unwrap();
    let wrong_id = DocumentIdentity::new(
        ScriptKind::ExternalGdscript,
        wrong_path,
        id.script_instance_id().cloned(),
        id.editor_instance_id().cloned(),
        id.buffer_instance_id().cloned(),
        id.disk_file_id().cloned(),
    )
    .unwrap();
    let facts = DocumentState::new(
        Some(wrong_id),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    let sources = Sources::new(
        observed(&id, Authority::D, "D"),
        observed(&id, Authority::R, "R"),
        observed(&id, Authority::B, "B"),
    )
    .unwrap();
    let dirty = DirtyObservation::observed(
        DirtyState::Clean,
        editor_stamp("3", 3),
        witness(&id, Authority::B),
    );
    assert_eq!(
        ObservationOutcome::classify(
            req.clone(),
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected.clone(),
                facts,
                sources.clone(),
                dirty.clone(),
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::WrongTarget)
    );
    let facts = DocumentState::new(
        None,
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    assert_eq!(
        ObservationOutcome::classify(
            req.clone(),
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected.clone(),
                facts,
                sources.clone(),
                dirty.clone(),
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidDocument)
    );
    let facts = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::NotOpen, editor_stamp("2", 2)),
    );
    assert_eq!(
        ObservationOutcome::classify(
            req.clone(),
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected.clone(),
                facts,
                sources.clone(),
                dirty.clone(),
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidAvailability)
    );
    assert_eq!(
        Sources::new(
            sources.resource().clone(),
            sources.disk().clone(),
            sources.buffer().clone()
        ),
        Err(EvidenceError::WrongAuthority)
    );
    let altered = Witness::new(
        id.resource_path().clone(),
        Some(decimal("9999")),
        None,
        None,
        None,
        Some(decimal("17")),
    );
    let replaced = Sources::new(
        sources.disk().clone(),
        SourceObservation::observed(
            Authority::R,
            "R".into(),
            editor_stamp("5", 4),
            altered,
            Staleness::unknown(),
        ),
        sources.buffer().clone(),
    )
    .unwrap();
    let facts = DocumentState::new(
        Some(id),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    assert_eq!(
        ObservationOutcome::classify(
            req.clone(),
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected.clone(),
                facts,
                replaced,
                dirty,
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::WrongTarget)
    );
    assert_eq!(
        ObservationOutcome::classify(req, interval(), None, None, vec![], vec![]),
        Err(EvidenceError::InvalidDocument)
    );
}

#[test]
fn open_missing_disk_remains_valid_but_missing_closed_target_and_invalid_kind_are_refusals() {
    let open = classify(
        Some(OpenState::Open),
        None,
        Some("loaded"),
        Some("human"),
        Some(DirtyState::Dirty),
        vec![],
        vec![],
    );
    assert_eq!(open.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(
        open.snapshot().unwrap().document().validity().value(),
        Some(&Validity::Valid)
    );
    assert_eq!(
        open.snapshot().unwrap().sources().disk().reason(),
        Some(SourceReason::DiskMissing)
    );
    assert_eq!(
        open.snapshot().unwrap().sources().buffer().text(),
        Some("human")
    );
    for (validity, expected) in [
        (Validity::Missing, OutcomeKind::MissingTarget),
        (Validity::Invalid, OutcomeKind::InvalidTarget),
    ] {
        let req = request(Some(SESSION));
        let selected = target(&req);
        let doc = DocumentState::new(
            None,
            DocumentFact::observed(validity, caller_stamp("1", 1)),
            DocumentFact::observed(OpenState::NotOpen, editor_stamp("2", 2)),
        );
        let sources = Sources::new(
            SourceObservation::unavailable(Authority::D, SourceReason::DiskMissing).unwrap(),
            SourceObservation::unavailable(Authority::R, SourceReason::ResourceNotLoaded).unwrap(),
            SourceObservation::closed_buffer(),
        )
        .unwrap();
        let result = ObservationOutcome::classify(
            req,
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected,
                doc,
                sources,
                DirtyObservation::closed_document(),
                Recheck::performed(vec![]),
            )),
            vec![TerminalFailure::Timeout],
            vec![],
        )
        .unwrap();
        assert_eq!(result.outcome(), expected);
        assert!(result.snapshot().is_none());
        assert!(result.resolved_target().is_some());
        assert!(result.diagnostics().iter().any(|d| matches!(
            d.code(),
            DiagnosticCode::MissingTarget | DiagnosticCode::InvalidTarget
        )));
    }
}

#[test]
fn builtin_script_cannot_acquire_scene_container_as_disk_source() {
    let req = ObservationRequest::new(
        RequestId::new("builtin-1").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://scene.tscn::GDScript_abc").unwrap(),
    );
    let selected = target(&req);
    let id = DocumentIdentity::new(
        ScriptKind::BuiltinGdscript,
        req.script_path().clone(),
        Some(decimal("44")),
        Some(decimal("55")),
        Some(decimal("66")),
        None,
    )
    .unwrap();
    let doc = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, editor_stamp("2", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("3", 2)),
    );
    let valid_sources = Sources::new(
        SourceObservation::unavailable(Authority::D, SourceReason::NoStandaloneDiskSource).unwrap(),
        observed(&id, Authority::R, "extends Node\n"),
        observed(&id, Authority::B, "extends Node\n"),
    )
    .unwrap();
    let dirty = DirtyObservation::observed(
        DirtyState::Clean,
        editor_stamp("3", 3),
        witness(&id, Authority::B),
    );
    let result = ObservationOutcome::classify(
        req.clone(),
        interval(),
        Some(selected.clone()),
        Some(ObservationEvidence::new(
            selected.clone(),
            doc.clone(),
            valid_sources,
            dirty.clone(),
            Recheck::performed(vec![]),
        )),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(
        result.snapshot().unwrap().sources().disk().reason(),
        Some(SourceReason::NoStandaloneDiskSource)
    );
    assert_eq!(
        result.snapshot().unwrap().sources().resource().text(),
        Some("extends Node\n")
    );
    let container = Sources::new(
        observed(&id, Authority::D, "[gd_scene]\n"),
        observed(&id, Authority::R, "extends Node\n"),
        observed(&id, Authority::B, "extends Node\n"),
    )
    .unwrap();
    assert_eq!(
        ObservationOutcome::classify(
            req,
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected,
                doc,
                container,
                dirty,
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidAvailability)
    );
}

#[test]
fn dirty_attribution_and_absent_identity_are_never_inferred_from_equal_text() {
    let (req, selected, _) = evidence(
        Some(OpenState::Open),
        Some("S"),
        Some("S"),
        Some("S"),
        Some(DirtyState::Clean),
        vec![],
    );
    let id = identity(Some(OpenState::Open));
    let sources = Sources::new(
        observed(&id, Authority::D, "S"),
        observed(&id, Authority::R, "S"),
        observed(&id, Authority::B, "S"),
    )
    .unwrap();
    let doc = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    let wrong_dirty_witness = Witness::new(
        id.resource_path().clone(),
        id.script_instance_id().cloned(),
        Some(decimal("999")),
        id.buffer_instance_id().cloned(),
        None,
        None,
    );
    assert_eq!(
        ObservationOutcome::classify(
            req.clone(),
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected.clone(),
                doc.clone(),
                sources.clone(),
                DirtyObservation::observed(
                    DirtyState::Clean,
                    editor_stamp("4", 3),
                    wrong_dirty_witness
                ),
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::WrongTarget)
    );
    let result = ObservationOutcome::classify(
        req,
        interval(),
        Some(selected.clone()),
        Some(ObservationEvidence::new(
            selected,
            doc,
            sources,
            DirtyObservation::unavailable(DirtyReason::DirtyAttributionUnavailable).unwrap(),
            Recheck::performed(vec![]),
        )),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(result.snapshot().unwrap().agreement(), Agreement::Agree);
    assert_eq!(
        result.snapshot().unwrap().dirty().availability(),
        Availability::Unavailable
    );
    assert_eq!(
        result.snapshot().unwrap().dirty().reason(),
        Some(DirtyReason::DirtyAttributionUnavailable)
    );
}

#[test]
fn later_surface_limit_does_not_erase_earlier_invalidated_evidence_or_other_sources() {
    let req = request(Some(SESSION));
    let selected = target(&req);
    let id = identity(Some(OpenState::Open));
    let mut buffer = observed(&id, Authority::B, "earlier human text");
    buffer.invalidate(SourceReason::SourceChanged).unwrap();
    buffer.invalidate(SourceReason::TooLarge).unwrap();
    let doc = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    let sources = Sources::new(
        observed(&id, Authority::D, "disk"),
        observed(&id, Authority::R, "resource"),
        buffer,
    )
    .unwrap();
    let dirty = DirtyObservation::observed(
        DirtyState::Dirty,
        editor_stamp("3", 3),
        witness(&id, Authority::B),
    );
    let result = ObservationOutcome::classify(
        req,
        interval(),
        Some(selected.clone()),
        Some(ObservationEvidence::new(
            selected,
            doc,
            sources,
            dirty,
            Recheck::performed(vec![]),
        )),
        vec![],
        vec![],
    )
    .unwrap();
    let snapshot = result.snapshot().unwrap();
    assert_eq!(result.outcome(), OutcomeKind::LimitedObservation);
    assert_eq!(snapshot.consistency().stability(), Stability::Changed);
    assert!(snapshot
        .consistency()
        .detected_changes()
        .contains(&DetectedChange::Source(Authority::B)));
    assert_eq!(
        snapshot.sources().buffer().reason(),
        Some(SourceReason::TooLarge)
    );
    assert_eq!(snapshot.sources().buffer().text(), None);
    assert_eq!(
        snapshot
            .sources()
            .buffer()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "earlier human text"
    );
    assert_eq!(snapshot.sources().disk().text(), Some("disk"));
    assert_eq!(snapshot.sources().resource().text(), Some("resource"));
    assert_eq!(snapshot.dirty().state(), Some(DirtyState::Dirty));
}

#[test]
fn closed_document_losing_its_session_is_not_misreported_closed_or_inapplicable() {
    let result = classify(
        Some(OpenState::NotOpen),
        Some("disk"),
        Some("cached"),
        None,
        None,
        vec![],
        vec![
            TerminalFailure::DisconnectedEditor,
            TerminalFailure::Timeout,
        ],
    );
    let snapshot = result.snapshot().unwrap();
    assert_eq!(result.outcome(), OutcomeKind::DisconnectedEditor);
    assert_eq!(snapshot.document().open_state().value(), None);
    assert_eq!(
        snapshot
            .document()
            .open_state()
            .invalidated_evidence()
            .unwrap()
            .value(),
        &OpenState::NotOpen
    );
    assert_eq!(snapshot.sources().disk().text(), Some("disk"));
    assert_eq!(
        snapshot
            .sources()
            .resource()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "cached"
    );
    assert_eq!(
        snapshot.sources().buffer().availability(),
        Availability::Unavailable
    );
    assert_eq!(
        snapshot.sources().buffer().reason(),
        Some(SourceReason::SessionEnded)
    );
    assert_eq!(snapshot.dirty().availability(), Availability::Unavailable);
    assert_eq!(snapshot.dirty().reason(), Some(DirtyReason::SessionEnded));
    assert_eq!(snapshot.consistency().stability(), Stability::Changed);
}

#[test]
fn independently_known_refusal_precedes_deadline_expiry() {
    for (known, expected) in [
        (
            TerminalFailure::UnsupportedObservation,
            OutcomeKind::UnsupportedObservation,
        ),
        (
            TerminalFailure::EditorUnavailable,
            OutcomeKind::EditorUnavailable,
        ),
    ] {
        let (request, target, evidence) = evidence(
            Some(OpenState::Open),
            Some("D"),
            Some("R"),
            Some("B"),
            Some(DirtyState::Clean),
            vec![],
        );
        let result = ObservationOutcome::classify(
            request,
            interval(),
            Some(target),
            Some(evidence),
            vec![TerminalFailure::Timeout, known],
            vec![],
        )
        .unwrap();
        assert_eq!(result.outcome(), expected);
        assert_ne!(result.outcome(), OutcomeKind::CompleteObservation);
    }
}

#[test]
fn same_authority_changes_are_not_evidence_of_lagging_another_authority() {
    let req = request(Some(SESSION));
    let selected = target(&req);
    let id = identity(Some(OpenState::Open));
    let newer_witness = Witness::new(
        id.resource_path().clone(),
        id.script_instance_id().cloned(),
        None,
        None,
        None,
        Some(decimal("18")),
    );
    let evidence = StalenessEvidence::unapplied_change(
        decimal("17"),
        Authority::R,
        decimal("18"),
        editor_stamp("43", 19),
        newer_witness,
    )
    .unwrap();
    let sources = Sources::new(
        observed(&id, Authority::D, "S"),
        SourceObservation::observed(
            Authority::R,
            "S".into(),
            editor_stamp("44", 18),
            witness(&id, Authority::R),
            Staleness::known_stale(vec![evidence]).unwrap(),
        ),
        observed(&id, Authority::B, "S"),
    )
    .unwrap();
    let doc = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    let dirty = DirtyObservation::observed(
        DirtyState::Clean,
        editor_stamp("3", 3),
        witness(&id, Authority::B),
    );
    assert_eq!(
        ObservationOutcome::classify(
            req,
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected,
                doc,
                sources,
                dirty,
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidStaleness)
    );
}

#[test]
fn integration_rejects_prior_attempt_evidence_in_the_same_session() {
    let (first, selected, facts) = evidence(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![],
    );
    let second = ObservationRequest::new(
        RequestId::new("example-2").unwrap(),
        first.project_root().clone(),
        first.session_id().cloned(),
        first.script_path().clone(),
    );
    assert_eq!(
        ObservationOutcome::classify(
            second,
            interval(),
            Some(selected),
            Some(facts),
            vec![],
            vec![]
        ),
        Err(EvidenceError::WrongTarget)
    );
}

#[test]
fn integration_protocol_failure_retains_validated_partial_evidence() {
    let result = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![DetectedChange::Source(Authority::B)],
        vec![TerminalFailure::ProtocolError],
    );
    assert_eq!(result.outcome(), OutcomeKind::ProtocolError);
    let snapshot = result
        .snapshot()
        .expect("earlier validated facts must survive");
    assert_eq!(snapshot.sources().disk().text(), Some("D"));
    assert_eq!(snapshot.sources().resource().text(), Some("R"));
    assert_eq!(
        snapshot
            .sources()
            .buffer()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "B"
    );
    assert_eq!(
        pairs(&result),
        [
            Comparison::Different,
            Comparison::Unknown,
            Comparison::Unknown
        ]
    );
}

#[test]
fn integration_known_loss_in_recheck_is_disconnection_without_a_duplicate_signal() {
    let result = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![DetectedChange::SessionEnded],
        vec![TerminalFailure::Timeout],
    );
    assert_eq!(result.outcome(), OutcomeKind::DisconnectedEditor);
    assert_eq!(
        result.snapshot().unwrap().sources().disk().text(),
        Some("D")
    );
    assert_eq!(result.snapshot().unwrap().sources().resource().text(), None);
}

#[test]
fn integration_cancellation_does_not_hide_known_live_evidence_invalidation() {
    let result = classify(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![],
        vec![
            TerminalFailure::Cancelled,
            TerminalFailure::DisconnectedEditor,
        ],
    );
    assert_eq!(result.outcome(), OutcomeKind::Cancelled);
    let snapshot = result.snapshot().unwrap();
    assert_eq!(snapshot.sources().resource().text(), None);
    assert_eq!(
        snapshot
            .sources()
            .resource()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "R"
    );
    assert_eq!(snapshot.sources().disk().text(), Some("D"));
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code() == DiagnosticCode::SessionEnded));
}

#[test]
fn integration_public_dirty_variant_cannot_bypass_reason_validation() {
    let req = request(Some(SESSION));
    let selected = target(&req);
    let id = identity(Some(OpenState::Open));
    let doc = DocumentState::new(
        Some(id.clone()),
        DocumentFact::observed(Validity::Valid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::Open, editor_stamp("2", 2)),
    );
    let sources = Sources::new(
        observed(&id, Authority::D, "D"),
        observed(&id, Authority::R, "R"),
        observed(&id, Authority::B, "B"),
    )
    .unwrap();
    let dirty = DirtyObservation::Unavailable {
        reason: DirtyReason::DocumentNotOpen,
        invalidated: None,
    };
    assert_eq!(
        ObservationOutcome::classify(
            req,
            interval(),
            Some(selected.clone()),
            Some(ObservationEvidence::new(
                selected,
                doc,
                sources,
                dirty,
                Recheck::performed(vec![])
            )),
            vec![],
            vec![]
        ),
        Err(EvidenceError::InvalidReason)
    );
}

#[test]
fn integration_wall_clock_adjustments_do_not_override_monotonic_collection_time() {
    let (req, selected, facts) = evidence(
        Some(OpenState::Open),
        Some("D"),
        Some("R"),
        Some("B"),
        Some(DirtyState::Dirty),
        vec![],
    );
    let adjusted = ObservationInterval::new(2000, 1000, 100);
    let result =
        ObservationOutcome::classify(req, adjusted, Some(selected), Some(facts), vec![], vec![])
            .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::CompleteObservation);
    assert_eq!(result.interval().elapsed_us(), 100);
    assert_eq!(result.interval().finished_unix_ms(), 1000);
}

#[test]
fn integration_non_gdscript_locator_can_receive_an_attributed_invalid_target_result() {
    let req = ObservationRequest::new(
        RequestId::new("invalid-target").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://notes.txt")
            .expect("lexical locator is valid; document type is evidence"),
    );
    let selected = target(&req);
    let doc = DocumentState::new(
        None,
        DocumentFact::observed(Validity::Invalid, caller_stamp("1", 1)),
        DocumentFact::observed(OpenState::NotOpen, editor_stamp("2", 2)),
    );
    let sources = Sources::new(
        SourceObservation::unavailable(Authority::D, SourceReason::DiskUnreadable).unwrap(),
        SourceObservation::unavailable(Authority::R, SourceReason::ResourceNotLoaded).unwrap(),
        SourceObservation::closed_buffer(),
    )
    .unwrap();
    let result = ObservationOutcome::classify(
        req,
        interval(),
        Some(selected.clone()),
        Some(ObservationEvidence::new(
            selected,
            doc,
            sources,
            DirtyObservation::closed_document(),
            Recheck::performed(vec![]),
        )),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(result.outcome(), OutcomeKind::InvalidTarget);
    assert!(result.snapshot().is_none());
    assert_eq!(
        result.resolved_target().unwrap().script_path().as_str(),
        "res://notes.txt"
    );
}

#[test]
fn closed_source_change_does_not_invalidate_the_observed_closed_state() {
    let result = classify(
        Some(OpenState::NotOpen),
        Some("D"),
        Some("R"),
        None,
        None,
        vec![DetectedChange::Source(Authority::D)],
        vec![],
    );
    assert_eq!(result.outcome(), OutcomeKind::NotOpen);
    let snapshot = result.snapshot().unwrap();
    assert_eq!(
        snapshot.document().open_state().value(),
        Some(&OpenState::NotOpen)
    );
    assert_eq!(snapshot.sources().disk().text(), None);
    assert_eq!(
        snapshot
            .sources()
            .disk()
            .invalidated_evidence()
            .unwrap()
            .text(),
        "D"
    );
    assert_eq!(snapshot.sources().resource().text(), Some("R"));
    assert_eq!(
        snapshot.sources().buffer().availability(),
        Availability::NotApplicable
    );
}

#[test]
fn target_failure_requires_attributable_document_evidence_not_an_override_flag() {
    for failure in [
        TerminalFailure::MissingTarget,
        TerminalFailure::InvalidTarget,
    ] {
        let (req, selected, facts) = evidence(
            Some(OpenState::Open),
            Some("D"),
            Some("R"),
            Some("B"),
            Some(DirtyState::Dirty),
            vec![],
        );
        assert_eq!(
            ObservationOutcome::classify(
                req,
                interval(),
                Some(selected),
                Some(facts),
                vec![failure],
                vec![]
            ),
            Err(EvidenceError::InvalidDocument)
        );
    }
}
