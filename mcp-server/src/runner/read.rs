//! Fresh authenticated acquisition and one-time revision/lifecycle selection.
//! Blocking acquisition remains in existing owned observation/closed workers.
use super::{AttemptClock, HostFailure};
use crate::observation::*;
use crate::script_edit::{EditRequest, ExpectedRevisionBasis};
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
                    Some("missing_script") => Some(TerminalFailure::MissingTarget),
                    Some("invalid_target") => Some(TerminalFailure::InvalidTarget),
                    Some("invalid_request") => Some(TerminalFailure::InvalidRequest),
                    _ => None,
                };
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
    let observation = ObservationRequest::new(
        observation_id(intent.request.request_id())?,
        intent.request.project_root().clone(),
        intent.request.session_id().cloned(),
        intent.request.script_path().clone(),
    );
    let mut capture = acquire(observation, registry, clock, cancelled, true)?;
    let refused = if capture.revision().is_none() {
        Some("")
    } else if cancelled.load(Ordering::Relaxed) {
        Some("cancelled")
    } else if clock.elapsed_us() >= 9_500_000 {
        Some("timeout")
    } else if capture.revision() != Some(&intent.revision) {
        Some("revision_mismatch")
    } else {
        None
    };
    if let Some(reason) = refused {
        return serde_json::to_vec(&capture.refusal(
            intent.request.request_id(),
            (!reason.is_empty()).then_some(reason),
        ))
        .map_err(|_| HostFailure);
    }
    if let Some(basis) = capture.closed.take() {
        let request = crate::script_closed_edit::CheckedClosedRequest::new(
            intent.request.request_id().clone(),
            basis,
            intent.replacement,
        )
        .map_err(|_| HostFailure)?;
        let outcome = super::closed_edit::run(request, registry, clock, cancelled)?;
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
    let input = capture.matching_open_input(intent.request.request_id(), &intent.replacement)?;
    let request = EditRequest::new(
        intent.request.request_id().clone(),
        intent.request.project_root().clone(),
        intent.request.session_id().cloned(),
        intent.request.script_path().clone(),
        expected,
        intent.replacement,
    )
    .map_err(|_| HostFailure)?;
    let outcome = super::edit::run(request, input, registry, clock, cancelled)?;
    let outcome = crate::bridge::wire::edit::encode_outcome(&outcome).map_err(|_| HostFailure)?;
    let mut result = Vec::with_capacity(outcome.len() + 26);
    result.extend_from_slice(b"{\"mode\":\"open\",\"outcome\":");
    result.extend_from_slice(&outcome);
    result.push(b'}');
    Ok(result)
}
