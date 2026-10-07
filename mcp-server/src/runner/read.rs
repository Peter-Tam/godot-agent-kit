//! Fresh authenticated acquisition and one-time revision/lifecycle selection.
//! Blocking acquisition remains in existing owned observation/closed workers.
use super::{AttemptClock, HostFailure};
use crate::observation::*;
use crate::script_edit::{
    Application, EditOutcome, EditOutcomeKind, EditRequest, ExactMatchError, ExpectedRevisionBasis,
    History, Reason, ReplacementSource, StepState,
};
use crate::script_read::{ScriptEditRequest, ScriptReadResult};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// Acquire ordinary read evidence and independently checked closed supplement.
pub fn run(
    request: ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<ScriptReadResult, HostFailure> {
    acquire(request, registry, clock, cancelled, false)
}
fn acquire(
    request: ObservationRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
    editing: bool,
) -> Result<ScriptReadResult, HostFailure> {
    let observation = if editing {
        super::run_for_edit(request.clone(), registry, clock, cancelled)?
    } else {
        super::run(request.clone(), registry, clock, cancelled)?
    };
    let mut result = ScriptReadResult::from_observation(observation);
    if result
        .observation
        .snapshot()
        .is_some_and(|s| s.document().open_state().value() == Some(&OpenState::NotOpen))
    {
        let supplement = if editing {
            super::closed_edit::acquire_for_edit(
                &request,
                registry,
                clock,
                cancelled,
                &result.observation,
            )
        } else {
            super::closed_edit::acquire(&request, registry, clock, cancelled, &result.observation)
        };
        match supplement {
            Ok(acquisition) => {
                let failure = match acquisition.reason.as_deref() {
                    Some("editor_unavailable") => Some(TerminalFailure::EditorUnavailable),
                    Some("disconnected" | "session_or_expiry_changed") => {
                        Some(TerminalFailure::DisconnectedEditor)
                    }
                    Some("timeout") => Some(TerminalFailure::Timeout),
                    Some("cancelled") => Some(TerminalFailure::Cancelled),
                    Some("protocol_error") => Some(TerminalFailure::ProtocolError),
                    Some("unsupported_capability") => Some(TerminalFailure::UnsupportedObservation),
                    Some("invalid_target") => Some(TerminalFailure::InvalidTarget),
                    Some("invalid_request") => Some(TerminalFailure::InvalidRequest),
                    _ => None,
                };
                // A missing D file can coexist with a valid retained Script.
                // Preserve that observation; missing_target still refuses the edit below.
                if !acquisition.changes.is_empty()
                    || acquisition.lifecycle_changed
                    || failure.is_some()
                {
                    result.observation = result
                        .observation
                        .supplement_recheck(
                            request.clone(),
                            clock.interval(),
                            acquisition.changes,
                            failure.clone(),
                            acquisition.lifecycle_changed,
                        )
                        .map_err(|_| HostFailure)?;
                }
                result.closed_observed = acquisition.observed;
                if let Some(basis) = acquisition.eligible {
                    result = result.with_closed(basis);
                } else {
                    let reason = failure
                        .as_ref()
                        .map(|_| crate::bridge::wire::OutcomeKind(result.observation.outcome()))
                        .or(acquisition.reason.as_deref())
                        .unwrap_or("closed_basis_unavailable");
                    result.suppress_revision(reason);
                }
            }
            Err(super::closed_edit::AcquisitionFailure::DeniedAccess) => {
                result = ScriptReadResult::from_observation(
                    ObservationOutcome::classify(
                        request.clone(),
                        clock.interval(),
                        None,
                        None,
                        vec![TerminalFailure::DeniedAccess],
                        vec![],
                    )
                    .map_err(|_| HostFailure)?,
                );
                result.suppress_revision("denied_access");
            }
            Err(super::closed_edit::AcquisitionFailure::Host) => return Err(HostFailure),
        }
    }
    result.interval = clock.interval();
    result.requested = Some(request);
    if cancelled.load(Ordering::Relaxed) && result.revision().is_some() {
        result.suppress_revision("cancelled");
    }
    if clock.elapsed_us() >= (if editing { 9_500_000 } else { 4_500_000 })
        && result.revision().is_some()
    {
        result.suppress_revision("timeout");
    }
    Ok(result)
}
fn observation_id(edit: &RequestId) -> Result<RequestId, HostFailure> {
    use ring::rand::{SecureRandom, SystemRandom};
    let mut bytes = [0; 16];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| HostFailure)?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut id = String::with_capacity(33);
    id.push('o');
    for byte in bytes {
        id.push(HEX[(byte >> 4) as usize] as char);
        id.push(HEX[(byte & 15) as usize] as char);
    }
    // A fixed prefix and random suffix do not reuse caller correlation.
    if id == edit.as_str() {
        id.replace_range(..1, "p");
    }
    RequestId::new(id).map_err(|_| HostFailure)
}
/// Compare one fresh capture, freeze it, and dispatch exactly one existing effect path.
pub fn edit(
    intent: ScriptEditRequest,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, HostFailure> {
    let (selectors, revision, source) = intent.into_parts();
    let observation = ObservationRequest::new(
        observation_id(selectors.request_id())?,
        selectors.project_root().clone(),
        selectors.session_id().cloned(),
        selectors.script_path().clone(),
    );
    let mut capture = acquire(observation, registry, clock, cancelled, true)?;
    let refused = if capture.revision().is_none() {
        Some("")
    } else if cancelled.load(Ordering::Relaxed) {
        Some("cancelled")
    } else if clock.elapsed_us() >= 9_500_000 {
        Some("timeout")
    } else if capture.revision() != Some(&revision) {
        Some("revision_mismatch")
    } else {
        None
    };
    if let Some(reason) = refused {
        return serde_json::to_vec(&capture.refusal(
            selectors.request_id(),
            (!reason.is_empty()).then_some(reason),
        ))
        .map_err(|_| HostFailure);
    }
    // Eligibility has checked this D against every applicable authority and the
    // same basis whose revision just matched. Never use the read presentation fallback.
    let current = capture
        .observation
        .snapshot()
        .and_then(|snapshot| snapshot.sources().disk().text())
        .ok_or(HostFailure)?;
    let (replacement, pending) = match source.resolve(current) {
        Ok(replacement) => (replacement, None),
        Err(reason) => match ReplacementSource::new(current.to_owned()) {
            Ok(unchanged) => (unchanged, Some(reason)),
            Err(_) => {
                return serde_json::to_vec(
                    &capture.refusal(selectors.request_id(), Some("unsupported_representation")),
                )
                .map_err(|_| HostFailure);
            }
        },
    };
    if let Some(reason) = interrupted(clock, cancelled) {
        capture.interval = clock.interval();
        return serde_json::to_vec(&capture.refusal(selectors.request_id(), Some(reason)))
            .map_err(|_| HostFailure);
    }
    if let Some(basis) = capture.closed.take() {
        let request = crate::script_closed_edit::CheckedClosedRequest::new(
            selectors.request_id().clone(),
            basis,
            replacement,
        )
        .map_err(|_| HostFailure)?;
        let mut outcome = super::closed_edit::run(request, registry, clock, cancelled)?;
        if let Some(reason) = pending {
            reduce_closed_match(&mut outcome, reason, interrupted(clock, cancelled));
        }
        #[derive(serde::Serialize)]
        struct ResultView<T> {
            mode: &'static str,
            outcome: T,
        }
        return serde_json::to_vec(&ResultView {
            mode: "closed",
            outcome,
        })
        .map_err(|_| HostFailure);
    }
    let expected =
        ExpectedRevisionBasis::from_observation(&capture.observation).map_err(|_| HostFailure)?;
    let input = capture.matching_open_input(selectors.request_id(), &replacement)?;
    let request = EditRequest::new(
        selectors.request_id().clone(),
        selectors.project_root().clone(),
        selectors.session_id().cloned(),
        selectors.script_path().clone(),
        expected,
        replacement,
    )
    .map_err(|_| HostFailure)?;
    let mut outcome = super::edit::run(request, input, registry, clock, cancelled)?;
    let matching =
        pending.filter(|_| reduce_open_match(&mut outcome, interrupted(clock, cancelled)));
    let outcome = crate::bridge::wire::edit::encode_resolved_outcome(&outcome, matching)
        .map_err(|_| HostFailure)?;
    let mut result = Vec::with_capacity(outcome.len() + 26);
    result.extend_from_slice(b"{\"mode\":\"open\",\"outcome\":");
    result.extend_from_slice(&outcome);
    result.push(b'}');
    Ok(result)
}

fn interrupted(clock: AttemptClock, cancelled: &AtomicBool) -> Option<&'static str> {
    if cancelled.load(Ordering::Relaxed) {
        Some("cancelled")
    } else if clock.elapsed_us() >= 9_500_000 {
        Some("timeout")
    } else {
        None
    }
}

/// Only the existing reducer's independently verified, zero-effect unchanged
/// result can expose a retained match diagnosis. Every real failure wins.
fn reduce_open_match(outcome: &mut EditOutcome, interruption: Option<&str>) -> bool {
    if !matches!(
        outcome.outcome,
        EditOutcomeKind::VerifiedUnchanged | EditOutcomeKind::VerifiedChanged
    ) {
        return false;
    }
    let zero_effects = outcome.application == Application::NotApplied
        && outcome.history == History::NotParticipated
        && [
            &outcome.progress.buffer_application,
            &outcome.progress.resource_sync,
            &outcome.progress.persistence,
            &outcome.progress.finalization,
        ]
        .iter()
        .all(|step| step.state == StepState::NotStarted)
        && outcome.persistence.is_none()
        && outcome.finalization.is_none();
    if outcome.outcome != EditOutcomeKind::VerifiedUnchanged || !zero_effects {
        outcome.outcome = match outcome.application {
            Application::Applied | Application::PartlyApplied => EditOutcomeKind::AppliedUnverified,
            _ => {
                outcome.application = Application::Unknown;
                EditOutcomeKind::ApplicationUnknown
            }
        };
        outcome.reason = Reason::ProtocolFailure;
        outcome.safe_next_action =
            "Observe the explicitly selected target again before any new edit";
        return false;
    }
    if let Some(reason) = interruption {
        outcome.outcome = EditOutcomeKind::Refused;
        outcome.reason = if reason == "cancelled" {
            Reason::Cancellation
        } else {
            Reason::Deadline
        };
        outcome.safe_next_action =
            "Observe the explicitly selected target again before any new edit";
        return false;
    }
    // These are the checked reducer's verification/disclosure invariants, not
    // an acknowledgment or a successful serialization interpreted as proof.
    let verified = outcome.expected.is_some()
        && outcome.before.is_some()
        && outcome.after.is_some()
        && outcome.progress.validation.state == StepState::Completed
        && outcome.progress.verification.state == StepState::Completed;
    if !verified {
        outcome.outcome = EditOutcomeKind::Refused;
        outcome.reason = Reason::IncompleteVerification;
        outcome.safe_next_action =
            "Observe the explicitly selected target again before any new edit";
    }
    verified
}

fn reduce_closed_match(
    outcome: &mut crate::script_closed_edit::ClosedOutcome,
    reason: ExactMatchError,
    interruption: Option<&str>,
) {
    if !matches!(outcome.outcome, "verified_unchanged" | "verified_changed") {
        return;
    }
    let effects = &outcome.effects;
    let zero_effects = outcome.application == "not_applied"
        && !effects.authorized
        && !effects.resource_entered
        && !effects.resource_changed
        && !effects.disk_entered
        && effects.written_bytes == 0
        && !effects.truncated
        && !effects.flushed
        && !effects.readback
        && !effects.mtime_restored;
    if outcome.outcome != "verified_unchanged" || !zero_effects {
        outcome.outcome = match outcome.application {
            "applied" | "partly_applied" => "applied_unverified",
            _ => {
                outcome.application = "unknown";
                "application_unknown"
            }
        };
        outcome.reason = "protocol_error".into();
        outcome.next_action = serde_json::json!({"kind":"fresh_read"});
        return;
    }
    let verified = outcome.lifecycle.final_state == "closed"
        && outcome.history == "not_applicable_closed"
        && outcome.evidence.as_ref().is_some_and(|evidence| {
            evidence["independently_verified"] == true
                && evidence["validation"]["original"] == true
                && evidence["validation"]["desired"] == true
                && evidence["validation"]["actual"] == true
        });
    outcome.outcome = "refused";
    outcome.next_action = serde_json::json!({"kind":"fresh_read"});
    outcome.reason = interruption
        .unwrap_or(if verified {
            reason.as_str()
        } else {
            "incomplete_verification"
        })
        .into();
    outcome.stage = if interruption.is_none() && verified {
        "matching"
    } else {
        "verification"
    };
}

#[cfg(test)]
mod matching_tests {
    use super::*;
    use crate::script_closed_edit::{ClosedOutcome, Effects};

    fn closed_unchanged() -> ClosedOutcome {
        let mut outcome = Effects::default().outcome(true, true, "complete", false, false);
        outcome.evidence = Some(serde_json::json!({
            "independently_verified":true,
            "validation":{"original":true,"desired":true,"actual":true}
        }));
        outcome
    }

    #[test]
    fn closed_match_diagnosis_requires_independent_unchanged_validation() {
        for reason in [
            ExactMatchError::NoMatch,
            ExactMatchError::AmbiguousMatch,
            ExactMatchError::EmptyOldString,
            ExactMatchError::InvalidSource,
        ] {
            let mut outcome = closed_unchanged();
            reduce_closed_match(&mut outcome, reason, None);
            assert_eq!(outcome.outcome, "refused");
            assert_eq!(outcome.application, "not_applied");
            assert_eq!(outcome.reason, reason.as_str());
            assert_eq!(outcome.stage, "matching");
            assert_eq!(outcome.lifecycle.final_state, "closed");
            assert_eq!(outcome.history, "not_applicable_closed");
            assert_eq!(outcome.next_action["kind"], "fresh_read");
        }
        for field in ["original", "desired", "actual"] {
            let mut outcome = closed_unchanged();
            outcome.evidence.as_mut().unwrap()["validation"][field] = false.into();
            reduce_closed_match(&mut outcome, ExactMatchError::NoMatch, None);
            assert_eq!(outcome.outcome, "refused");
            assert_eq!(outcome.reason, "incomplete_verification");
        }
    }

    #[test]
    fn retained_match_error_never_erases_closed_uncertainty_or_denial() {
        for (kind, application, reason) in [
            ("refused", "not_applied", "dirty_resource"),
            ("application_unknown", "unknown", "disconnected"),
            ("applied_unverified", "partly_applied", "denied_access"),
        ] {
            let mut outcome = closed_unchanged();
            outcome.outcome = kind;
            outcome.application = application;
            outcome.reason = reason.into();
            outcome.evidence = None;
            reduce_closed_match(&mut outcome, ExactMatchError::NoMatch, Some("cancelled"));
            assert_eq!(outcome.outcome, kind);
            assert_eq!(outcome.application, application);
            assert_eq!(outcome.reason, reason);
            assert!(outcome.evidence.is_none());
        }
    }

    #[test]
    fn diagnostic_effects_cannot_become_a_matching_refusal() {
        let mut outcome = closed_unchanged();
        outcome.effects.authorized = true;
        reduce_closed_match(&mut outcome, ExactMatchError::NoMatch, None);
        assert_eq!(outcome.outcome, "application_unknown");
        assert_eq!(outcome.application, "unknown");
        assert_eq!(outcome.reason, "protocol_error");
        assert!(outcome.effects.authorized);
        let mut outcome = closed_unchanged();
        outcome.outcome = "verified_changed";
        outcome.application = "applied";
        outcome.effects.disk_entered = true;
        outcome.effects.written_bytes = 7;
        reduce_closed_match(&mut outcome, ExactMatchError::NoMatch, None);
        assert_eq!(outcome.outcome, "applied_unverified");
        assert_eq!(outcome.application, "applied");
        assert_eq!(outcome.effects.written_bytes, 7);
    }

    #[test]
    fn cancellation_or_deadline_after_diagnostic_verification_wins() {
        for reason in ["cancelled", "timeout"] {
            let mut outcome = closed_unchanged();
            reduce_closed_match(&mut outcome, ExactMatchError::AmbiguousMatch, Some(reason));
            assert_eq!(outcome.outcome, "refused");
            assert_eq!(outcome.application, "not_applied");
            assert_eq!(outcome.reason, reason);
            assert_eq!(outcome.stage, "verification");
        }
    }

    fn open_refusal() -> EditOutcome {
        EditOutcome::refuse_without_basis(
            RequestId::new("matching-reduction").unwrap(),
            ProjectRoot::new("/fixture").unwrap(),
            None,
            ResourcePath::new("res://target.gd").unwrap(),
            ObservationInterval::new(1, 1, 0),
            crate::script_edit::BasisError::Missing,
        )
    }

    #[test]
    fn open_failures_missing_evidence_and_effects_never_publish_matching_success() {
        let mut outcome = open_refusal();
        outcome.reason = Reason::DeniedAccess;
        assert!(!reduce_open_match(&mut outcome, Some("cancelled")));
        assert_eq!(outcome.reason, Reason::DeniedAccess);
        assert!(outcome.expected.is_none());

        let mut outcome = open_refusal();
        outcome.outcome = EditOutcomeKind::VerifiedUnchanged;
        assert!(!reduce_open_match(&mut outcome, None));
        assert_eq!(outcome.outcome, EditOutcomeKind::Refused);
        assert_eq!(outcome.reason, Reason::IncompleteVerification);

        let mut outcome = open_refusal();
        outcome.outcome = EditOutcomeKind::VerifiedChanged;
        outcome.application = Application::PartlyApplied;
        outcome.progress.buffer_application.state = StepState::Completed;
        assert!(!reduce_open_match(&mut outcome, None));
        assert_eq!(outcome.outcome, EditOutcomeKind::AppliedUnverified);
        assert_eq!(outcome.application, Application::PartlyApplied);
        assert_eq!(
            outcome.progress.buffer_application.state,
            StepState::Completed
        );
    }
}
