use godot_agent_kit::observation::SOURCE_LIMIT_BYTES;
use godot_agent_kit::observation::*;
use godot_agent_kit::script_edit::ReplacementSource;
use godot_agent_kit::script_edit::{BasisError, EditRequest, ExpectedRevisionBasis, SourceDigest};

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
        EngineVersion::new("4.7.2", "hash").unwrap(),
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
fn prior_variant(
    disk: &str,
    resource: &str,
    buffer: &str,
    missing: Option<Authority>,
) -> ObservationOutcome {
    prior_sample(
        [disk, resource, buffer],
        DirtyState::Clean,
        Some("17"),
        "edit",
        missing,
        false,
        vec![],
    )
}

fn prior(text: &str, dirty: DirtyState, version: Option<&str>) -> ObservationOutcome {
    prior_with(text, dirty, version, "prior")
}

#[test]
fn basis_requires_attributable_clean_current_revision_not_a_saved_version() {
    assert_eq!(
        ExpectedRevisionBasis::from_observation(&prior("a", DirtyState::Dirty, Some("17"))),
        Err(BasisError::Dirty)
    );
    assert_eq!(
        ExpectedRevisionBasis::from_observation(&prior("a", DirtyState::Clean, None)),
        Err(BasisError::MissingVersion)
    );
    let original = prior("雪\n", DirtyState::Clean, Some("17"));
    let basis = ExpectedRevisionBasis::from_observation(&original).unwrap();
    assert_eq!(basis.current_version().as_str(), "17");
    assert_eq!(basis.source(), &SourceDigest::of("雪\n"));
    assert_eq!(basis.prior_request_id().as_str(), "prior");
    assert_eq!(basis.prior_interval(), original.interval());
    assert_eq!(
        basis.collections()[2],
        *original
            .snapshot()
            .unwrap()
            .sources()
            .buffer()
            .collection()
            .unwrap()
    );
    assert_eq!(basis.prior_dirty(), original.snapshot().unwrap().dirty());
}

#[test]
fn replacement_preserves_exact_utf8_and_supported_size_boundary() {
    for source in ["", "\t# 雪\n", "extends Node", "extends Node\n"] {
        assert_eq!(
            ReplacementSource::new(source.to_owned()).unwrap().as_str(),
            source
        );
    }
    let limit = "x".repeat(SOURCE_LIMIT_BYTES);
    assert_eq!(
        ReplacementSource::new(limit.clone()).unwrap().as_str(),
        limit
    );
    assert!(ReplacementSource::new("x".repeat(SOURCE_LIMIT_BYTES + 1)).is_err());
}

#[test]
fn replacement_refuses_unsupported_representation_without_normalization() {
    for source in ["\0", "a\r\nb", "a\rb", "\u{feff}extends Node"] {
        assert!(ReplacementSource::new(source.to_owned()).is_err());
    }
}

fn attempt(desired: &str) -> godot_agent_kit::script_edit::EditAttempt {
    use godot_agent_kit::script_edit::EditAttempt;
    let basis =
        ExpectedRevisionBasis::from_observation(&prior("old", DirtyState::Clean, Some("17")))
            .unwrap();
    let request = EditRequest::new(
        RequestId::new("edit").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
        basis,
        ReplacementSource::new(desired.to_owned()).unwrap(),
    )
    .unwrap();
    let target_request = ObservationRequest::new(
        RequestId::new("edit").unwrap(),
        request.project_root().clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    );
    let target = ResolvedTarget::for_request(
        &target_request,
        request.project_root().clone(),
        FileIdentity::new(decimal("1"), decimal("2")),
        SessionId::new(SESSION).unwrap(),
        EngineVersion::new("4.7.2", "hash").unwrap(),
    )
    .unwrap();
    let mut attempt = EditAttempt::new(request);
    attempt.select(target).unwrap();
    attempt
}

fn metadata() -> godot_agent_kit::script_edit::DiskMetadata {
    use godot_agent_kit::script_edit::DiskMetadata;
    DiskMetadata {
        identity: FileIdentity::new(decimal("6"), decimal("7")),
        mtime: decimal("111"),
        collection: stamp(false),
    }
}
fn saved(version: &str) -> godot_agent_kit::script_edit::SavedStateEvidence {
    use godot_agent_kit::script_edit::SavedStateEvidence;
    SavedStateEvidence {
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: prior("old", DirtyState::Clean, Some("17"))
            .snapshot()
            .unwrap()
            .document()
            .identity()
            .unwrap()
            .clone(),
        collection: stamp(true),
        current_version: decimal(version),
        saved_version: decimal(version),
        resource_edited: false,
        save_profile: [1; 32],
        original_preserved: true,
        desired_preserved: true,
    }
}
fn validation(
    purpose: godot_agent_kit::script_edit::ValidationPurpose,
    input: &str,
) -> godot_agent_kit::script_edit::ValidationResult {
    use godot_agent_kit::script_edit::*;
    ValidationResult {
        status: ValidationStatus::Valid,
        reason: None,
        purpose,
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: saved("17").document,
        source_path: ResourcePath::new("res://subject.gd").unwrap(),
        input: SourceDigest::of(input),
        collection: stamp(false),
        dependencies: vec![],
        context: [2; 32],
        context_current: true,
        dependencies_current: true,
        sources: vec![ValidationSourceFence {
            path: ResourcePath::new("res://subject.gd").unwrap(),
            source: SourceDigest::of(input),
            diagnostics_completed: true,
            symbols_completed: true,
        }],
        cleanup_confirmed: true,
        diagnostics: vec![],
    }
}
fn receipt(input: &str) -> godot_agent_kit::script_edit::PersistenceReceipt {
    use godot_agent_kit::script_edit::PersistenceReceipt;
    PersistenceReceipt {
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: saved("17").document,
        intended: SourceDigest::of(input),
        project_id: FileIdentity::new(decimal("1"), decimal("2")),
        file_id: FileIdentity::new(decimal("6"), decimal("7")),
        collection: stamp(true),
        write_started: true,
        bytes_written: input.len(),
        truncated: true,
        flushed: true,
        readback_matches: true,
        attached: true,
        descriptor_open: true,
        interference: false,
        original_mtime: Some(decimal("111")),
        mtime: Some(decimal("111")),
        restore_attempted: true,
        restore_errno: None,
        restored: true,
        reason: None,
    }
}
fn finalized() -> godot_agent_kit::script_edit::FinalizationResult {
    use godot_agent_kit::script_edit::*;
    FinalizationResult {
        status: FinalizationStatus::Complete,
        reason: None,
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: saved("17").document,
        collection: stamp(true),
        before_source: Some(SourceDigest::of("new")),
        after_source: Some(SourceDigest::of("new")),
        before_current: Some(decimal("18")),
        after_current: Some(decimal("18")),
        before_saved: Some(decimal("17")),
        after_saved: Some(decimal("18")),
        before_resource_edited: Some(true),
        after_resource_edited: Some(false),
        steps: Bookkeeping {
            resource_edited: true,
            saved_version: true,
        },
    }
}
fn timeline() -> ObservationInterval {
    ObservationInterval::new(1, 2, 100_000)
}
fn context() -> godot_agent_kit::script_edit::ContextRecheck {
    godot_agent_kit::script_edit::ContextRecheck {
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: saved("17").document,
        collection: stamp(false),
        context: [2; 32],
        dependencies: vec![],
        current: true,
    }
}
fn prepared(desired: &str) -> godot_agent_kit::script_edit::EditAttempt {
    let mut attempt = attempt(desired);
    attempt
        .prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    attempt
}
fn authorized(desired: &str) -> godot_agent_kit::script_edit::EditAttempt {
    use godot_agent_kit::script_edit::ValidationPurpose;
    let mut attempt = prepared(desired);
    attempt
        .validation(validation(ValidationPurpose::Preflight, desired))
        .unwrap();
    attempt
        .guard_application(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    assert!(attempt.ready_to_apply());
    attempt.authorize().unwrap();
    attempt
}

fn native_witness() -> godot_agent_kit::script_edit::NativeWitness {
    godot_agent_kit::script_edit::NativeWitness {
        request_id: RequestId::new("edit").unwrap(),
        session_id: SessionId::new(SESSION).unwrap(),
        document: saved("17").document,
        collection: stamp(true),
    }
}

#[test]
fn authorization_and_permanent_discard_do_not_equate_lost_reply_with_refusal() {
    use godot_agent_kit::script_edit::*;
    let no_dispatch = prepared("new").finish(timeline());
    assert_eq!(no_dispatch.outcome, EditOutcomeKind::Refused);
    assert_eq!(no_dispatch.application, Application::NotApplied);
    let mut lost = authorized("new");
    lost.fail(Reason::Deadline, None).unwrap();
    assert_eq!(
        lost.finish(timeline()).outcome,
        EditOutcomeKind::ApplicationUnknown
    );
    let mut discarded = authorized("new");
    discarded
        .discard_before_boundary(Reason::RevisionMismatch, native_witness())
        .unwrap();
    assert!(discarded.enter_application().is_err());
    let done = discarded.finish(timeline());
    assert_eq!(done.application, Application::NotApplied);
    assert_eq!(done.outcome, EditOutcomeKind::Refused);
    let mut entered = authorized("new");
    entered.enter_application().unwrap();
    entered.fail(Reason::Cancellation, None).unwrap();
    assert_eq!(
        entered.finish(timeline()).outcome,
        EditOutcomeKind::ApplicationUnknown
    );
}

#[test]
fn known_application_survives_partial_receipt_and_denial_suppresses_source() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    let mut partial = receipt("new");
    partial.attached = false;
    attempt.persistence(partial).unwrap();
    attempt.fail(Reason::DeniedAccess, None).unwrap();
    let done = attempt.finish(timeline());
    assert_eq!(done.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(done.application, Application::Applied);
    assert_eq!(done.reason, Reason::DeniedAccess);
    assert!(done.before.is_none() && done.expected.is_none() && done.validation.is_empty());
    assert!(done.resolved_target.is_none() && done.persistence.is_none());
}

#[test]
fn preparation_checks_dirty_equal_stale_version_and_save_profile_without_dispatch() {
    use godot_agent_kit::script_edit::*;
    let mut dirty = attempt("old");
    dirty
        .prepare(
            prior_with("old", DirtyState::Dirty, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(dirty.finish(timeline()).reason, Reason::DirtyConflict);
    let mut stale = attempt("old");
    stale
        .prepare(
            prior_with("old", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(stale.finish(timeline()).reason, Reason::RevisionMismatch);
    let mut unsupported = attempt("new");
    let mut profile = saved("17");
    profile.original_preserved = false;
    unsupported
        .prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(profile),
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(
        unsupported.finish(timeline()).reason,
        Reason::SaveWouldReformat
    );
}

#[test]
fn independent_verification_distinguishes_unchanged_and_changed_and_invalid_source() {
    use godot_agent_kit::script_edit::*;
    let mut unchanged = prepared("old");
    unchanged
        .validation(validation(ValidationPurpose::Unchanged, "old"))
        .unwrap();
    unchanged
        .verify(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    let done = unchanged.finish(timeline());
    assert_eq!(done.outcome, EditOutcomeKind::VerifiedUnchanged);
    assert_eq!(done.history, History::NotParticipated);
    let mut changed = authorized("new");
    changed.enter_application().unwrap();
    changed
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    changed.resource_synced(native_witness()).unwrap();
    changed.persistence(receipt("new")).unwrap();
    changed.finalization(finalized()).unwrap();
    changed
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    changed
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    assert_eq!(
        changed.finish(timeline()).outcome,
        EditOutcomeKind::VerifiedChanged
    );
    let mut invalid = prepared("old");
    let mut result = validation(ValidationPurpose::Unchanged, "old");
    result.status = ValidationStatus::Invalid;
    result.reason = Some(Reason::ParseError);
    invalid.validation(result).unwrap();
    assert_eq!(invalid.finish(timeline()).reason, Reason::ParseError);
}

#[test]
fn failed_t0_restore_after_content_write_retains_known_application_and_error() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    let mut proof = receipt("new");
    proof.mtime = None;
    proof.restored = false;
    proof.restore_errno = Some(1);
    proof.reason = Some(Reason::PersistenceFailure);
    request.persistence(proof).unwrap();
    assert_eq!(request.enter_finalization(), Err(EditError::OutOfOrder));
    let mut post = validation(ValidationPurpose::PostChange, "new");
    post.status = ValidationStatus::Invalid;
    post.reason = Some(Reason::ParseError);
    post.diagnostics.push(ValidationDiagnostic {
        origin: DiagnosticOrigin::Root,
        path: Some(ResourcePath::new("res://subject.gd").unwrap()),
        path_reason: None,
        source: Some(SourceDigest::of("new")),
        line: Some(1),
        column: Some(1),
        message: "actual source parse error".into(),
    });
    request.validation(post).unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    let result = request.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(result.application, Application::Applied);
    assert_eq!(result.reason, Reason::PersistenceFailure);
    assert_eq!(result.progress.persistence.state, StepState::Failed);
    assert_eq!(result.progress.finalization.state, StepState::NotStarted);
    assert_eq!(result.validation[1].status, ValidationStatus::Invalid);
    let proof = result.persistence.unwrap();
    assert_eq!(proof.restore_errno, Some(1));
    assert!(proof.restore_attempted && !proof.restored);
}

#[test]
fn matching_d_r_b_and_clean_saved_state_cannot_erase_wrong_t0_metadata() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    let mut proof = receipt("new");
    proof.original_mtime = Some(decimal("112"));
    proof.mtime = Some(decimal("112"));
    request.persistence(proof).unwrap();
    assert_eq!(request.enter_finalization(), Err(EditError::OutOfOrder));
    request
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    let result = request.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(result.progress.persistence.state, StepState::Failed);
}

#[test]
fn mutation_boundary_requires_fresh_original_revision_and_profile() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    request
        .validation(validation(ValidationPurpose::Preflight, "new"))
        .unwrap();
    assert_eq!(request.authorize(), Err(EditError::OutOfOrder));
    assert!(!request.ready_to_apply());
    assert_eq!(
        request.finish(timeline()).application,
        Application::NotApplied
    );
    let mut stale = prepared("new");
    stale
        .validation(validation(ValidationPurpose::Preflight, "new"))
        .unwrap();
    let mut updated = saved("17");
    updated.save_profile = [9; 32];
    stale
        .guard_application(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(updated),
            Some(metadata()),
        )
        .unwrap();
    assert!(!stale.ready_to_apply());
    assert_eq!(stale.authorize(), Err(EditError::OutOfOrder));
    let result = stale.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::Refused);
    assert_eq!(result.application, Application::NotApplied);
}

#[test]
fn reopened_guard_refuses_before_authority_without_erasing_human_work() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = prepared("new");
    attempt
        .validation(validation(ValidationPurpose::Preflight, "new"))
        .unwrap();
    attempt
        .guard_application(
            prior_sample_with_buffer(
                ["human", "human", "human"],
                DirtyState::Clean,
                (Some("19"), "55"),
                "edit",
                None,
                false,
                vec![],
            ),
            Some(saved("19")),
            Some(metadata()),
        )
        .unwrap();
    assert!(!attempt.ready_to_apply());
    let terminal = attempt.finish(timeline());
    assert_eq!(terminal.reason, Reason::IdentityChanged);
    assert_eq!(terminal.application, Application::NotApplied);
    assert_eq!(terminal.outcome, EditOutcomeKind::Refused);
    assert_eq!(terminal.history, History::NotParticipated);
    assert!(terminal.before.is_some());
}

#[test]
fn wrong_session_clock_and_source_validation_cannot_authorize() {
    use godot_agent_kit::script_edit::*;
    let mut wrong = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.session_id = SessionId::new("ffeeddccbbaa99887766554433221100").unwrap();
    assert_eq!(wrong.validation(result), Err(EditError::WrongTarget));
    assert_eq!(wrong.finish(timeline()).reason, Reason::ProtocolFailure);
    let mut wrong = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.collection = stamp(true);
    assert_eq!(wrong.validation(result), Err(EditError::WrongClock));
    let mut wrong = prepared("new");
    let result = validation(ValidationPurpose::Preflight, "different");
    assert_eq!(wrong.validation(result), Err(EditError::InvalidEvidence));
    assert!(wrong.authorize().is_err());
    assert_eq!(
        wrong.finish(timeline()).application,
        Application::NotApplied
    );
}

#[test]
fn latest_independent_human_post_sample_survives_helper_loss_without_verified_success() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    attempt.persistence(receipt("new")).unwrap();
    attempt.finalization(finalized()).unwrap();
    attempt
        .retain_after(
            prior_with("human", DirtyState::Dirty, Some("19"), "edit"),
            Some(saved("19")),
            Some(metadata()),
        )
        .unwrap();
    attempt.fail(Reason::Deadline, None).unwrap();
    let outcome = attempt.finish(timeline());
    assert_eq!(outcome.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(outcome.application, Application::Applied);
    assert_ne!(outcome.progress.verification.state, StepState::Completed);
    let after = outcome.after.unwrap();
    assert_eq!(after.sources[0].source, Some(SourceDigest::of("human")));
    assert_eq!(after.sources[1].source, Some(SourceDigest::of("human")));
    assert_eq!(after.sources[2].source, Some(SourceDigest::of("human")));
    assert_eq!(after.saved_state.unwrap().current_version, decimal("19"));
    assert_eq!(after.dirty.state(), Some(DirtyState::Dirty));
}

#[test]
fn partial_finalization_and_postchange_dependency_invalidation_retain_known_change() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    attempt.persistence(receipt("new")).unwrap();
    let mut partial = finalized();
    partial.status = FinalizationStatus::PartialOrUnknown;
    partial.reason = Some(Reason::Deadline);
    partial.after_saved = None;
    partial.steps.saved_version = false;
    attempt.finalization(partial).unwrap();
    let outcome = attempt.finish(timeline());
    assert_eq!(outcome.outcome, EditOutcomeKind::AppliedUnverified);
    let finalization = outcome.finalization.unwrap();
    assert_eq!(finalization.status, FinalizationStatus::PartialOrUnknown);
    assert_eq!(finalization.reason, Some(Reason::Deadline));
    assert!(finalization.steps.resource_edited);
    assert!(!finalization.steps.saved_version);
    assert_eq!(finalization.before_resource_edited, Some(true));
    assert_eq!(finalization.after_resource_edited, Some(false));
    assert_eq!(finalization.after_saved, None);
    assert_eq!(outcome.progress.finalization.state, StepState::Failed);
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    attempt.persistence(receipt("new")).unwrap();
    attempt.finalization(finalized()).unwrap();
    let mut post = validation(ValidationPurpose::PostChange, "new");
    post.status = ValidationStatus::Unavailable;
    post.dependencies_current = false;
    post.reason = Some(Reason::DependencyError);
    attempt.validation(post).unwrap();
    assert_eq!(attempt.finish(timeline()).reason, Reason::DependencyError);
}

#[test]
fn saved_readback_and_independent_sources_cannot_be_inferred_from_finalizer() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    attempt.persistence(receipt("new")).unwrap();
    attempt.finalization(finalized()).unwrap();
    attempt
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    attempt
        .verify(
            prior_with("old", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    assert_eq!(
        attempt.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    attempt.persistence(receipt("new")).unwrap();
    attempt.finalization(finalized()).unwrap();
    attempt
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    attempt
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            None,
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    assert_eq!(
        attempt.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn unavailable_authority_or_missing_saved_readback_never_supplies_a_basis() {
    for authority in [Authority::D, Authority::R, Authority::B] {
        let sample = prior_sample(
            ["old"; 3],
            DirtyState::Clean,
            Some("17"),
            "prior",
            Some(authority),
            false,
            vec![],
        );
        assert!(ExpectedRevisionBasis::from_observation(&sample).is_err());
    }
    let mut request = attempt("new");
    request
        .prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            None,
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(
        request.finish(timeline()).reason,
        godot_agent_kit::script_edit::Reason::UnavailableObservation
    );
}

#[test]
fn previously_observed_but_invalidated_source_and_known_staleness_refuse() {
    let invalidated = prior_sample(
        ["old"; 3],
        DirtyState::Clean,
        Some("17"),
        "prior",
        None,
        false,
        vec![DetectedChange::Source(Authority::R)],
    );
    assert!(ExpectedRevisionBasis::from_observation(&invalidated).is_err());
    let stale = prior_sample(
        ["old"; 3],
        DirtyState::Clean,
        Some("17"),
        "prior",
        None,
        true,
        vec![],
    );
    assert_eq!(
        ExpectedRevisionBasis::from_observation(&stale),
        Err(BasisError::KnownStaleResource)
    );
}

#[test]
fn postchange_validation_context_must_be_rechecked_after_native_result() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    request.finalization(finalized()).unwrap();
    request
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(ContextRecheck {
                current: false,
                ..context()
            }),
        )
        .unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(terminal.reason, Reason::IncompleteVerification);
}

#[test]
fn older_attributed_validation_event_is_rejected_without_erasing_authorization_risk() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = prepared("new");
    let mut validation = validation(ValidationPurpose::Preflight, "new");
    validation.collection =
        CollectionStamp::new(ClockId::Caller, decimal("1"), decimal("2"), 9).unwrap();
    assert_eq!(
        attempt.validation(validation),
        Err(EditError::InvalidTiming)
    );
    assert_eq!(attempt.finish(timeline()).outcome, EditOutcomeKind::Refused);
}

#[test]
fn invalidation_preserves_old_summary_but_not_current_source_or_success() {
    use godot_agent_kit::script_edit::*;
    let mut request = attempt("new");
    request
        .prepare(
            prior_sample(
                ["old"; 3],
                DirtyState::Clean,
                Some("17"),
                "edit",
                None,
                false,
                vec![DetectedChange::Source(Authority::R)],
            ),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    let result = request.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::Refused);
    let before = result.before.unwrap();
    let resource = &before.sources[1];
    assert_eq!(resource.source, None);
    assert_eq!(resource.availability, Availability::Unavailable);
    assert_eq!(
        resource.invalidated.as_ref().unwrap().source,
        SourceDigest::of("old")
    );
    assert_eq!(
        resource.invalidated.as_ref().unwrap().reason,
        SourceReason::SourceChanged
    );
}

#[test]
fn save_profile_checks_desired_as_well_as_original_and_receipt_identity_is_bound() {
    use godot_agent_kit::script_edit::*;
    let mut request = attempt("new");
    let mut profile = saved("17");
    profile.desired_preserved = false;
    request
        .prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(profile),
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(request.finish(timeline()).reason, Reason::SaveWouldReformat);
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    let mut receipt = receipt("new");
    receipt.file_id = FileIdentity::new(decimal("6"), decimal("8"));
    assert_eq!(
        request.persistence(receipt),
        Err(EditError::InvalidEvidence)
    );
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn unavailable_postparse_does_not_fall_back_to_valid_preflight() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    request.finalization(finalized()).unwrap();
    let mut post = validation(ValidationPurpose::PostChange, "new");
    post.status = ValidationStatus::Invalid;
    post.reason = Some(Reason::ParseError);
    request.validation(post).unwrap();
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn prior_unsupported_text_cannot_become_a_writable_revision_basis() {
    for source in ["a\r\nb", "\u{feff}old", "x\0y"] {
        assert_eq!(
            ExpectedRevisionBasis::from_observation(&prior(source, DirtyState::Clean, Some("17"))),
            Err(BasisError::UnsupportedRepresentation)
        );
    }
}

#[test]
fn missing_prior_basis_is_a_source_free_typed_refusal() {
    use godot_agent_kit::script_edit::*;
    let done = EditOutcome::refuse_without_basis(
        RequestId::new("edit").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
        timeline(),
        BasisError::Missing,
    );
    assert_eq!(done.outcome, EditOutcomeKind::Refused);
    assert_eq!(done.reason, Reason::MissingBasis);
    assert!(done.expected.is_none() && done.before.is_none() && done.after.is_none());
    assert_eq!(done.application, Application::NotApplied);
}

#[test]
fn divergence_and_missing_fresh_authority_prevent_mutation_despite_matching_disk_hash() {
    use godot_agent_kit::script_edit::*;
    let divergent = prior_variant("old", "different", "old", None);
    assert_eq!(
        ExpectedRevisionBasis::from_observation(&divergent),
        Err(BasisError::Divergent)
    );
    let mut divergent_attempt = attempt("new");
    divergent_attempt
        .prepare(
            prior_variant("old", "different", "old", None),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    assert_eq!(
        divergent_attempt.finish(timeline()).reason,
        Reason::Divergence
    );
    for missing in [Authority::D, Authority::R, Authority::B] {
        let mut attempt = attempt("new");
        attempt
            .prepare(
                prior_variant("old", "old", "old", Some(missing)),
                Some(saved("17")),
                Some(metadata()),
            )
            .unwrap();
        assert_eq!(
            attempt.finish(timeline()).application,
            Application::NotApplied
        );
    }
}

#[test]
fn empty_and_exact_limit_prior_sources_remain_eligible_without_saved_version_fabrication() {
    let limit = "x".repeat(SOURCE_LIMIT_BYTES);
    for text in ["", limit.as_str()] {
        let observed = prior(text, DirtyState::Clean, Some("17"));
        let basis = ExpectedRevisionBasis::from_observation(&observed).unwrap();
        assert_eq!(*basis.source(), SourceDigest::of(text));
    }
}

#[test]
fn incomplete_or_misattributed_validation_cannot_certify_parse_success() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    let mut invalid = validation(ValidationPurpose::Preflight, "new");
    invalid.status = ValidationStatus::Invalid;
    invalid.sources[0].symbols_completed = false;
    request.validation(invalid).unwrap();
    assert_eq!(
        request.finish(timeline()).reason,
        Reason::ValidationUnavailable
    );
    let mut request = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.diagnostics.push(ValidationDiagnostic {
        origin: DiagnosticOrigin::Root,
        path: Some(ResourcePath::new("res://other.gd").unwrap()),
        path_reason: None,
        source: Some(SourceDigest::of("new")),
        line: Some(1),
        column: Some(1),
        message: "unexpected token".into(),
    });
    assert_eq!(request.validation(result), Err(EditError::InvalidEvidence));
}

#[test]
fn every_captured_source_requires_both_fences_and_cleanup_even_for_invalid_results() {
    use godot_agent_kit::script_edit::*;
    let dependency_path = ResourcePath::new("res://dependency.gd").unwrap();
    let dependency_source = SourceDigest::of("class_name Dependency\n");
    for invalid in [false, true] {
        for missing in 0..7 {
            let mut request = prepared("new");
            let mut result = validation(ValidationPurpose::Preflight, "new");
            result.dependencies.push(DependencyWitness {
                path: dependency_path.clone(),
                identity: FileIdentity::new(decimal("1"), decimal("222")),
                source: dependency_source,
            });
            result.sources.push(ValidationSourceFence {
                path: dependency_path.clone(),
                source: dependency_source,
                diagnostics_completed: true,
                symbols_completed: true,
            });
            if invalid {
                result.status = ValidationStatus::Invalid;
                result.reason = Some(Reason::DependencyError);
                result.diagnostics.push(ValidationDiagnostic {
                    origin: DiagnosticOrigin::Dependency,
                    path: Some(dependency_path.clone()),
                    path_reason: None,
                    source: Some(dependency_source),
                    line: Some(1),
                    column: Some(1),
                    message: "dependency error".into(),
                });
            }
            match missing {
                0 => result.sources[0].diagnostics_completed = false,
                1 => result.sources[0].symbols_completed = false,
                2 => result.sources[1].diagnostics_completed = false,
                3 => result.sources[1].symbols_completed = false,
                4 => {
                    result.sources.remove(1);
                }
                5 => result.cleanup_confirmed = false,
                6 => {
                    result.sources.remove(0);
                }
                _ => unreachable!(),
            }
            request.validation(result).unwrap();
            let terminal = request.finish(timeline());
            assert_eq!(
                terminal.outcome,
                EditOutcomeKind::Refused,
                "{invalid} {missing}"
            );
            assert_eq!(
                terminal.reason,
                Reason::ValidationUnavailable,
                "{invalid} {missing}"
            );
            assert_eq!(terminal.validation[0].status, ValidationStatus::Unavailable);
            assert!(terminal.progress.validation.state != StepState::Completed);
        }
    }
}

#[test]
fn wrong_or_duplicate_dependency_fences_cannot_authorize() {
    use godot_agent_kit::script_edit::*;
    for duplicate in [false, true] {
        let mut request = prepared("new");
        let mut result = validation(ValidationPurpose::Preflight, "new");
        result.dependencies.push(DependencyWitness {
            path: ResourcePath::new("res://dependency.gd").unwrap(),
            identity: FileIdentity::new(decimal("1"), decimal("222")),
            source: SourceDigest::of("actual"),
        });
        result.sources.push(ValidationSourceFence {
            path: ResourcePath::new("res://dependency.gd").unwrap(),
            source: SourceDigest::of(if duplicate { "new" } else { "other" }),
            diagnostics_completed: true,
            symbols_completed: true,
        });
        if duplicate {
            result.sources[1].path = result.sources[0].path.clone();
        }
        assert_eq!(request.validation(result), Err(EditError::InvalidEvidence));
        assert_eq!(request.finish(timeline()).outcome, EditOutcomeKind::Refused);
    }
}

#[test]
fn completion_label_never_upgrades_unverified_or_preboundary_discarded_attempt() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = authorized("new");
    assert_eq!(
        attempt.discard_before_boundary(Reason::Complete, native_witness()),
        Err(EditError::InvalidEvidence)
    );
    assert_eq!(
        attempt.finish(timeline()).outcome,
        EditOutcomeKind::ApplicationUnknown
    );
    let mut attempt = authorized("new");
    attempt.enter_application().unwrap();
    attempt
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    attempt.resource_synced(native_witness()).unwrap();
    let mut receipt = receipt("new");
    receipt.attached = false;
    receipt.reason = Some(Reason::Complete);
    assert_eq!(
        attempt.persistence(receipt),
        Err(EditError::InvalidEvidence)
    );
    assert_eq!(
        attempt.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn newer_same_text_buffer_version_after_application_invalidates_success() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    request.finalization(finalized()).unwrap();
    request
        .validation(validation(ValidationPurpose::PostChange, "new"))
        .unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Clean, Some("19"), "edit"),
            Some(saved("19")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn out_of_interval_evidence_cannot_become_a_success_after_terminal_reduction() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("old");
    request
        .validation(validation(ValidationPurpose::Unchanged, "old"))
        .unwrap();
    request
        .verify(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    let terminal = request.finish(ObservationInterval::new(1, 2, 9));
    assert_eq!(terminal.outcome, EditOutcomeKind::Refused);
    assert_eq!(terminal.reason, Reason::ProtocolFailure);
}

#[test]
fn wrong_request_and_document_binding_cannot_contribute_acquired_evidence() {
    use godot_agent_kit::script_edit::*;
    let mut request = attempt("new");
    assert_eq!(
        request.prepare(
            prior("old", DirtyState::Clean, Some("17")),
            Some(saved("17")),
            Some(metadata())
        ),
        Err(EditError::WrongTarget)
    );
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::Refused);
    assert!(terminal.before.is_none());
    let mut request = attempt("new");
    let mut save = saved("17");
    save.session_id = SessionId::new("ffeeddccbbaa99887766554433221100").unwrap();
    assert_eq!(
        request.prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(save),
            None
        ),
        Err(EditError::WrongTarget)
    );
    assert_eq!(
        request.finish(timeline()).application,
        Application::NotApplied
    );
}

#[test]
fn entered_resource_sync_without_acknowledgment_keeps_known_buffer_application() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.enter_resource_sync().unwrap();
    request
        .resource_sync_unknown(Reason::Disconnection)
        .unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(terminal.application, Application::Applied);
    assert_eq!(terminal.progress.resource_sync.state, StepState::Unknown);
    assert_eq!(terminal.progress.persistence.state, StepState::NotStarted);
}

#[test]
fn lost_persistence_and_finalization_replies_remain_step_specific() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.enter_persistence().unwrap();
    request.persistence_unknown(Reason::Deadline).unwrap();
    let result = request.finish(timeline());
    assert_eq!(result.progress.persistence.state, StepState::Unknown);
    assert_eq!(result.progress.finalization.state, StepState::NotStarted);
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    request.enter_finalization().unwrap();
    request.finalization_unknown(Reason::Disconnection).unwrap();
    let result = request.finish(timeline());
    assert_eq!(result.progress.finalization.state, StepState::Unknown);
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
}

#[test]
fn busy_admission_exposes_no_prior_revision_or_source_and_no_queued_application() {
    use godot_agent_kit::script_edit::*;
    let mut request = attempt("new");
    request.fail(Reason::Busy, None).unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::Refused);
    assert_eq!(terminal.application, Application::NotApplied);
    assert!(terminal.resolved_target.is_none() && terminal.expected.is_none());
    assert!(terminal.before.is_none() && terminal.validation.is_empty());
}

#[test]
fn attributable_history_effect_without_confirmed_full_buffer_change_is_partly_applied() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .partial_application(Reason::PersistenceFailure, native_witness())
        .unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(terminal.application, Application::PartlyApplied);
    assert_eq!(
        terminal.progress.buffer_application.state,
        StepState::Failed
    );
    assert_eq!(terminal.history, History::NativeComplexEdit);
}

#[test]
fn cancellation_after_buffer_effect_cannot_start_new_mutating_stage() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.fail(Reason::Cancellation, None).unwrap();
    assert_eq!(request.enter_resource_sync(), Err(EditError::OutOfOrder));
    let terminal = request.finish(timeline());
    assert_eq!(terminal.reason, Reason::Cancellation);
    assert_eq!(terminal.outcome, EditOutcomeKind::AppliedUnverified);
}

#[test]
fn validation_uses_caller_clock_and_rejects_backwards_caller_ticks() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    let mut event = validation(ValidationPurpose::Preflight, "new");
    event.collection =
        CollectionStamp::new(ClockId::Caller, decimal("0"), decimal("0"), 11).unwrap();
    assert_eq!(request.validation(event), Err(EditError::InvalidTiming));
    assert_eq!(request.finish(timeline()).reason, Reason::ProtocolFailure);
}

#[test]
fn caller_validation_ticks_are_never_compared_with_editor_ticks() {
    use godot_agent_kit::script_edit::*;
    let mut request = attempt("new");
    let mut inspection = saved("17");
    inspection.collection = CollectionStamp::new(
        ClockId::Editor(SessionId::new(SESSION).unwrap()),
        decimal("999999"),
        decimal("1000000"),
        inspection.collection.received_elapsed_us(),
    )
    .unwrap();
    request
        .prepare(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(inspection),
            Some(metadata()),
        )
        .unwrap();
    request
        .validation(validation(ValidationPurpose::Preflight, "new"))
        .unwrap();
    assert_eq!(request.authorize(), Err(EditError::OutOfOrder));
    assert_eq!(
        request.finish(timeline()).application,
        Application::NotApplied
    );
}

#[test]
fn debug_does_not_expose_replacement_or_diagnostic_text() {
    use godot_agent_kit::script_edit::*;
    let secret = "secret script literal";
    assert!(!format!("{:?}", ReplacementSource::new(secret.into()).unwrap()).contains(secret));
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.status = ValidationStatus::Invalid;
    result.reason = Some(Reason::ParseError);
    result.diagnostics.push(ValidationDiagnostic {
        origin: DiagnosticOrigin::Root,
        path: Some(ResourcePath::new("res://subject.gd").unwrap()),
        path_reason: None,
        source: Some(SourceDigest::of("new")),
        line: Some(1),
        column: Some(2),
        message: secret.into(),
    });
    assert!(!format!("{result:?}").contains(secret));
}

#[test]
fn denial_remains_source_free_when_terminal_timing_is_malformed() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.fail(Reason::DeniedAccess, None).unwrap();
    let result = request.finish(ObservationInterval::new(1, 2, 0));
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(result.reason, Reason::DeniedAccess);
    assert!(result.expected.is_none() && result.before.is_none());
    assert!(result.validation.is_empty() && result.resolved_target.is_none());
    assert!(result.safe_next_action.contains("Observe"));
}

#[test]
fn incomplete_validation_is_reported_unavailable_not_valid() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.context_current = false;
    request.validation(result).unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.validation[0].status, ValidationStatus::Unavailable);
    assert_eq!(
        terminal.validation[0].reason,
        Some(Reason::ValidationUnavailable)
    );
}

#[test]
fn validation_cannot_overlap_the_preparation_collection() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.collection =
        CollectionStamp::new(ClockId::Caller, decimal("0"), decimal("99999"), 99999).unwrap();
    assert_eq!(request.validation(result), Err(EditError::InvalidTiming));
    assert_eq!(request.finish(timeline()).outcome, EditOutcomeKind::Refused);
}

#[test]
fn finalization_cannot_present_the_original_revision_as_its_entry_state() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    let mut result = finalized();
    result.before_source = Some(SourceDigest::of("old"));
    result.before_current = Some(decimal("17"));
    assert_eq!(
        request.finalization(result),
        Err(EditError::InvalidEvidence)
    );
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn partial_application_can_retain_independent_survivor_observations() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.fail(Reason::Disconnection, None).unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Dirty, Some("18"), "edit"),
            None,
            Some(metadata()),
            None,
        )
        .unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(terminal.reason, Reason::Disconnection);
    assert_eq!(
        terminal.after.unwrap().dirty.state(),
        Some(DirtyState::Dirty)
    );
}

#[test]
fn wrong_native_acknowledgments_cannot_prove_application_or_permanent_discard() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    let mut witness = native_witness();
    witness.request_id = RequestId::new("someone_else").unwrap();
    assert_eq!(
        request.discard_before_boundary(Reason::Cancellation, witness),
        Err(EditError::WrongTarget)
    );
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::ApplicationUnknown
    );
    let mut request = authorized("new");
    request.enter_application().unwrap();
    let mut witness = native_witness();
    witness.session_id = SessionId::new("ffeeddccbbaa99887766554433221100").unwrap();
    assert_eq!(
        request.buffer_changed(decimal("18"), witness),
        Err(EditError::WrongTarget)
    );
    assert_eq!(request.finish(timeline()).application, Application::Unknown);
}

#[test]
fn complete_finalization_and_preflight_still_require_postchange_parse() {
    use godot_agent_kit::script_edit::*;
    let mut request = authorized("new");
    request.enter_application().unwrap();
    request
        .buffer_changed(decimal("18"), native_witness())
        .unwrap();
    request.resource_synced(native_witness()).unwrap();
    request.persistence(receipt("new")).unwrap();
    request.finalization(finalized()).unwrap();
    request
        .verify(
            prior_with("new", DirtyState::Clean, Some("18"), "edit"),
            Some(saved("18")),
            Some(metadata()),
            Some(context()),
        )
        .unwrap();
    assert_eq!(
        request.finish(timeline()).outcome,
        EditOutcomeKind::AppliedUnverified
    );
}

#[test]
fn validation_limits_retain_bounded_unavailable_evidence() {
    use godot_agent_kit::script_edit::*;
    for (dependencies, bytes, diagnostics, message_bytes) in [
        (33, 1, 0, 0),
        (1, SOURCE_LIMIT_BYTES + 1, 0, 0),
        (9, SOURCE_LIMIT_BYTES, 0, 0),
        (0, 0, 65, 1),
        (0, 0, 1, 2049),
    ] {
        let mut request = prepared("new");
        let mut result = validation(ValidationPurpose::Preflight, "new");
        result.dependencies = (0..dependencies)
            .map(|i| DependencyWitness {
                path: ResourcePath::new(format!("res://dependency_{i}.gd")).unwrap(),
                identity: FileIdentity::new(decimal("1"), decimal(&(100 + i).to_string())),
                source: SourceDigest {
                    sha256: [1; 32],
                    utf8_bytes: bytes,
                },
            })
            .collect();
        result.diagnostics = (0..diagnostics)
            .map(|_| ValidationDiagnostic {
                origin: DiagnosticOrigin::Root,
                path: Some(ResourcePath::new("res://subject.gd").unwrap()),
                path_reason: None,
                source: Some(SourceDigest::of("new")),
                line: None,
                column: None,
                message: "x".repeat(message_bytes),
            })
            .collect();
        request.validation(result).unwrap();
        assert!(request.authorize().is_err());
        let terminal = request.finish(timeline());
        assert_eq!(terminal.reason, Reason::EvidenceLimit);
        assert_eq!(terminal.application, Application::NotApplied);
        assert_eq!(terminal.validation[0].status, ValidationStatus::Unavailable);
        assert!(terminal.validation[0].dependencies.len() <= 32);
        assert!(
            terminal.validation[0]
                .dependencies
                .iter()
                .map(|d| d.source.utf8_bytes)
                .sum::<usize>()
                <= 4 * 1024 * 1024
        );
        assert!(terminal.validation[0].diagnostics.len() <= 64);
        assert!(terminal.validation[0]
            .diagnostics
            .iter()
            .all(|d| d.message.len() <= 2048));
    }
}

#[test]
fn every_failure_cause_preserves_dispatch_and_application_certainty() {
    use godot_agent_kit::script_edit::*;
    for reason in [
        Reason::Busy,
        Reason::DirtyConflict,
        Reason::RevisionMismatch,
        Reason::IdentityChanged,
        Reason::SessionChanged,
        Reason::Divergence,
        Reason::KnownStaleResource,
        Reason::KnownStaleBuffer,
        Reason::UnavailableObservation,
        Reason::ClosedTarget,
        Reason::UnsupportedTarget,
        Reason::DeniedAccess,
        Reason::UnsupportedEngine,
        Reason::UnsupportedEffect,
        Reason::UnsupportedRepresentation,
        Reason::SaveWouldReformat,
        Reason::PersistenceFailure,
        Reason::PersistenceUnknown,
        Reason::PartialFinalization,
        Reason::ParseError,
        Reason::DependencyError,
        Reason::ValidationUnavailable,
        Reason::EvidenceLimit,
        Reason::Deadline,
        Reason::Cancellation,
        Reason::Disconnection,
        Reason::ProtocolFailure,
        Reason::MissingBasis,
        Reason::IncompleteVerification,
    ] {
        let mut not_applied = prepared("new");
        not_applied.fail(reason, None).unwrap();
        let terminal = not_applied.finish(timeline());
        assert_eq!(terminal.outcome, EditOutcomeKind::Refused, "{reason:?}");
        assert_eq!(terminal.application, Application::NotApplied);
        let mut uncertain = authorized("new");
        uncertain.fail(reason, None).unwrap();
        assert_eq!(
            uncertain.finish(timeline()).outcome,
            EditOutcomeKind::ApplicationUnknown,
            "{reason:?}"
        );
        let mut applied = authorized("new");
        applied.enter_application().unwrap();
        applied
            .buffer_changed(decimal("18"), native_witness())
            .unwrap();
        applied.fail(reason, None).unwrap();
        assert_eq!(
            applied.finish(timeline()).outcome,
            EditOutcomeKind::AppliedUnverified,
            "{reason:?}"
        );
    }
}

#[test]
fn a_bound_write_receipt_preserves_known_effects_when_earlier_acknowledgments_are_missing() {
    use godot_agent_kit::script_edit::*;
    for write_started_only in [false, true] {
        let mut request = authorized("new");
        request.enter_application().unwrap();
        let mut proof = receipt("new");
        if write_started_only {
            proof.bytes_written = 0;
            proof.truncated = false;
            proof.flushed = false;
            proof.readback_matches = false;
        }
        request.persistence(proof).unwrap();
        let terminal = request.finish(timeline());
        assert_eq!(
            terminal.outcome,
            if write_started_only {
                EditOutcomeKind::ApplicationUnknown
            } else {
                EditOutcomeKind::AppliedUnverified
            }
        );
        assert_eq!(terminal.progress.resource_sync.state, StepState::NotStarted);
        assert_eq!(
            terminal.application,
            if write_started_only {
                Application::Unknown
            } else {
                Application::PartlyApplied
            }
        );
    }
}

#[test]
fn validation_limit_failure_cannot_clear_a_denial() {
    use godot_agent_kit::script_edit::*;
    let mut request = prepared("new");
    let mut result = validation(ValidationPurpose::Preflight, "new");
    result.status = ValidationStatus::Unavailable;
    result.reason = Some(Reason::DeniedAccess);
    result.diagnostics.push(ValidationDiagnostic {
        origin: DiagnosticOrigin::Root,
        path: None,
        path_reason: Some(Reason::DeniedAccess),
        source: None,
        line: None,
        column: None,
        message: "x".repeat(2049),
    });
    request.validation(result).unwrap();
    let terminal = request.finish(timeline());
    assert_eq!(terminal.reason, Reason::DeniedAccess);
    assert!(
        terminal.expected.is_none() && terminal.before.is_none() && terminal.validation.is_empty()
    );
}

#[test]
fn isolated_native_effect_evidence_never_invents_earlier_stage_completion() {
    use godot_agent_kit::script_edit::*;
    let mut resource = authorized("new");
    resource.enter_application().unwrap();
    resource.resource_synced(native_witness()).unwrap();
    assert!(resource.enter_persistence().is_err());
    let result = resource.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(result.application, Application::PartlyApplied);
    assert_eq!(result.progress.buffer_application.state, StepState::Entered);
    assert_eq!(result.progress.resource_sync.state, StepState::Completed);
    let mut bookkeeping = authorized("new");
    bookkeeping.enter_application().unwrap();
    bookkeeping.finalization(finalized()).unwrap();
    let result = bookkeeping.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::AppliedUnverified);
    assert_eq!(result.application, Application::PartlyApplied);
    assert_eq!(result.progress.persistence.state, StepState::NotStarted);
    assert_eq!(result.progress.finalization.state, StepState::Completed);
}

#[test]
fn ambiguous_selection_keeps_only_source_free_disambiguation_metadata() {
    use godot_agent_kit::script_edit::*;
    let basis =
        ExpectedRevisionBasis::from_observation(&prior("old", DirtyState::Clean, Some("17")))
            .unwrap();
    let request = EditRequest::new(
        RequestId::new("edit").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        None,
        ResourcePath::new("res://subject.gd").unwrap(),
        basis,
        ReplacementSource::new("new".into()).unwrap(),
    )
    .unwrap();
    let mut attempt = EditAttempt::new(request);
    let selection = Selection::ambiguous(vec![
        SessionId::new(SESSION).unwrap(),
        SessionId::new("ffeeddccbbaa99887766554433221100").unwrap(),
    ])
    .unwrap();
    attempt
        .fail(Reason::AmbiguousTarget, Some(selection.clone()))
        .unwrap();
    let result = attempt.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::Refused);
    assert_eq!(result.selection, Some(selection));
    assert!(result.expected.is_none() && result.resolved_target.is_none());
    assert!(result.before.is_none() && result.after.is_none());
}

#[test]
fn unavailable_fresh_observation_denial_suppresses_an_earlier_valid_basis() {
    use godot_agent_kit::script_edit::*;
    let mut attempt = attempt("new");
    let request = ObservationRequest::new(
        RequestId::new("edit").unwrap(),
        ProjectRoot::new("/fixture/project").unwrap(),
        Some(SessionId::new(SESSION).unwrap()),
        ResourcePath::new("res://subject.gd").unwrap(),
    );
    let denied = ObservationOutcome::classify(
        request,
        timeline(),
        None,
        None,
        vec![TerminalFailure::DeniedAccess],
        vec![],
    )
    .unwrap();
    attempt.prepare(denied, None, None).unwrap();
    let result = attempt.finish(timeline());
    assert_eq!(result.reason, Reason::DeniedAccess);
    assert!(result.expected.is_none() && result.resolved_target.is_none());
}

#[test]
fn missing_verification_buffer_version_cannot_be_inferred_from_saved_state() {
    use godot_agent_kit::script_edit::*;
    for (desired, changed) in [("old", false), ("new", true)] {
        let mut request = if changed {
            authorized(desired)
        } else {
            prepared(desired)
        };
        if changed {
            request.enter_application().unwrap();
            request
                .buffer_changed(decimal("18"), native_witness())
                .unwrap();
            request.resource_synced(native_witness()).unwrap();
            request.persistence(receipt(desired)).unwrap();
            request.finalization(finalized()).unwrap();
        }
        request
            .validation(validation(
                if changed {
                    ValidationPurpose::PostChange
                } else {
                    ValidationPurpose::Unchanged
                },
                desired,
            ))
            .unwrap();
        request
            .verify(
                prior_with(desired, DirtyState::Clean, None, "edit"),
                Some(saved(if changed { "18" } else { "17" })),
                Some(metadata()),
                Some(context()),
            )
            .unwrap();
        let result = request.finish(timeline());
        assert_eq!(
            result.outcome,
            if changed {
                EditOutcomeKind::AppliedUnverified
            } else {
                EditOutcomeKind::Refused
            }
        );
        assert_eq!(
            result.application,
            if changed {
                Application::Applied
            } else {
                Application::NotApplied
            }
        );
        assert_eq!(result.reason, Reason::IncompleteVerification);
        let after = result.after.unwrap();
        assert_eq!(
            after.sources[2].identity.as_ref().unwrap().source_version(),
            None
        );
        assert_eq!(after.agreement, Agreement::Agree);
    }
}

#[test]
fn preboundary_discard_retires_authorization_but_cannot_erase_entered_work() {
    use godot_agent_kit::script_edit::*;
    let mut released = prepared("new");
    released
        .validation(validation(ValidationPurpose::Preflight, "new"))
        .unwrap();
    released
        .guard_application(
            prior_with("old", DirtyState::Clean, Some("17"), "edit"),
            Some(saved("17")),
            Some(metadata()),
        )
        .unwrap();
    released.authorize().unwrap();
    // Only an acknowledged native preboundary discard retires released authority.
    released
        .discard_before_boundary(Reason::Cancellation, native_witness())
        .unwrap();
    assert_eq!(released.authorize(), Err(EditError::OutOfOrder));
    assert_eq!(released.enter_application(), Err(EditError::OutOfOrder));
    let result = released.finish(timeline());
    assert_eq!(result.outcome, EditOutcomeKind::Refused);
    assert_eq!(result.application, Application::NotApplied);
    assert_eq!(result.history, History::NotParticipated);
    assert_eq!(result.reason, Reason::Cancellation);

    for known_effect in [false, true] {
        let mut entered = authorized("new");
        entered.enter_application().unwrap();
        if known_effect {
            entered
                .buffer_changed(decimal("18"), native_witness())
                .unwrap();
        }
        assert_eq!(
            entered.discard_before_boundary(Reason::Cancellation, native_witness()),
            Err(EditError::OutOfOrder)
        );
        let result = entered.finish(timeline());
        assert_eq!(
            result.outcome,
            if known_effect {
                EditOutcomeKind::AppliedUnverified
            } else {
                EditOutcomeKind::ApplicationUnknown
            }
        );
        assert_eq!(
            result.application,
            if known_effect {
                Application::Applied
            } else {
                Application::Unknown
            }
        );
    }
}
