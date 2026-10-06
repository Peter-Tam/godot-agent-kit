//! Strict public edit input and selected-session edit exchange.
use super::*;
use crate::observation::{
    ObservationEvidence, ObservationInterval, ObservationOutcome, Recheck, Sources,
};
use crate::script_edit::{self, EditRequest, ExpectedRevisionBasis, ReplacementSource};
use serde::Deserialize;

#[path = "edit/editor.rs"]
mod editor;
pub(crate) use editor::{
    apply, prepare, recheck, terminal, verify, AppliedEvent, EditContext, EditSample,
};
#[path = "edit/output.rs"]
mod output;
pub(crate) use output::encode_resolved_outcome;
pub use output::{encode_outcome, exit_code};

// Unlike an ordinary serde Option, this accepts explicit null but requires the
// field to exist in the complete observation-v1 envelope.
struct RequiredOption<T>(Option<T>);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for RequiredOption<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Present<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Present<T> {
            type Value = RequiredOption<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a required object or explicit null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(RequiredOption(None))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(|value| RequiredOption(Some(value)))
            }
        }
        d.deserialize_any(Present(std::marker::PhantomData))
    }
}
const INPUT_LIMIT: usize = 12 * 1024 * 1024;
const INPUT_STAGE: Stage = Stage::ValidateRequest;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalIn {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComparisonsIn {
    disk_resource: String,
    disk_buffer: String,
    resource_buffer: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsistencyIn {
    atomic: bool,
    stability: String,
    checks: String,
    detected_changes: Bounded<ChangeIn>,
    recheck_reason: Option<RecheckReason>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcesIn {
    #[serde(rename = "D")]
    disk: SourceIn,
    #[serde(rename = "R")]
    resource: SourceIn,
    #[serde(rename = "B")]
    buffer: SourceIn,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotIn {
    target: TargetIn,
    document: DocumentIn,
    sources: SourcesIn,
    dirty: DirtyIn,
    comparisons: ComparisonsIn,
    agreement: String,
    consistency: ConsistencyIn,
    diagnostics: Bounded<DiagnosticIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionInPublic {
    candidate_sessions: Bounded<String>,
    missing_selector: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BasisIn {
    schema_version: u32,
    request_id: String,
    outcome: OutcomeKind,
    interval: IntervalIn,
    resolved_target: RequiredOption<TargetIn>,
    snapshot: RequiredOption<SnapshotIn>,
    diagnostics: Bounded<DiagnosticIn>,
    selection: RequiredOption<SelectionInPublic>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputIn {
    schema_version: u32,
    request_id: String,
    basis: BasisIn,
    replacement_source: String,
}

/// Correlation is retained only after the complete input shape and ID have decoded.
#[derive(Debug)]
pub struct InputError {
    pub request_id: Option<RequestId>,
}
impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid edit input")
    }
}
impl std::error::Error for InputError {}
/// Either eligible checked intent or a correlated, source-free basis refusal.
pub type DecodedRequest = Result<EditRequest, (RequestId, script_edit::BasisError)>;

/// Decode a complete public observation, not a source digest supplied as an authorization token.
/// The ordinary observation reducer verifies all three exact source strings and attribution.
///
/// # Errors
/// Rejects malformed, oversized or inconsistent input. A checked request ID is
/// retained for semantic errors after strict decoding, without retaining source.
pub fn decode_request(
    bytes: &[u8],
    project: ProjectRoot,
    session: Option<SessionId>,
    script: ResourcePath,
) -> Result<DecodedRequest, InputError> {
    let unknown = || InputError { request_id: None };
    if bytes.len() > INPUT_LIMIT || bytes.is_empty() {
        return Err(unknown());
    }
    json_depth(bytes, INPUT_STAGE).map_err(|_| unknown())?;
    let mut input: InputIn = decode(bytes, INPUT_STAGE).map_err(|_| unknown())?;
    let id = RequestId::new(std::mem::take(&mut input.request_id)).map_err(|_| unknown())?;
    validate_input(input, project, session, script, &id).map_err(|_| InputError {
        request_id: Some(id),
    })
}

fn validate_input(
    input: InputIn,
    project: ProjectRoot,
    session: Option<SessionId>,
    script: ResourcePath,
    id: &RequestId,
) -> Result<DecodedRequest, RoutingFailure> {
    if input.schema_version != 1 || input.basis.schema_version != 1 {
        return Err(bad(INPUT_STAGE));
    }
    let prior_id = RequestId::new(input.basis.request_id).map_err(|_| bad(INPUT_STAGE))?;
    if input.replacement_source.len() > SOURCE_LIMIT_BYTES {
        return Err(bad(INPUT_STAGE));
    }
    let replacement = match ReplacementSource::new(input.replacement_source) {
        Ok(source) => source,
        Err(_) => {
            return Ok(Err((
                id.clone(),
                script_edit::BasisError::UnsupportedRepresentation,
            )))
        }
    };
    let prior_request =
        ObservationRequest::new(prior_id, project.clone(), session.clone(), script.clone());
    let interval = ObservationInterval::new(
        input.basis.interval.started_unix_ms,
        input.basis.interval.finished_unix_ms,
        input.basis.interval.elapsed_us,
    );
    let prior = if let Some(snapshot) = input.basis.snapshot.0 {
        if input.basis.selection.0.is_some() || input.basis.resolved_target.0.is_none() {
            return Err(bad(INPUT_STAGE));
        }
        let expected_target = input
            .basis
            .resolved_target
            .0
            .unwrap()
            .domain(&prior_request)?;
        let target = snapshot.target.domain(&prior_request)?;
        if target != expected_target
            || !snapshot.consistency.detected_changes.0.is_empty()
            || snapshot.consistency.recheck_reason.is_some()
            || snapshot.consistency.checks != "performed"
            || snapshot.consistency.stability != "unknown"
            || snapshot.consistency.atomic
        {
            return Err(bad(INPUT_STAGE));
        }
        let receipt = interval.elapsed_us();
        let path = target.script_path();
        let sid = target.session_id();
        let document = snapshot.document.domain(path, sid, receipt, INPUT_STAGE)?;
        let disk = snapshot.sources.disk.domain(
            Authority::D,
            &target,
            receipt,
            INPUT_STAGE,
            document.identity(),
            None,
        )?;
        let resource = snapshot.sources.resource.domain(
            Authority::R,
            &target,
            receipt,
            INPUT_STAGE,
            document.identity(),
            None,
        )?;
        let buffer = snapshot.sources.buffer.domain(
            Authority::B,
            &target,
            receipt,
            INPUT_STAGE,
            document.identity(),
            None,
        )?;
        let dirty = snapshot.dirty.domain(path, sid, receipt, INPUT_STAGE)?;
        let parsed_diagnostics = snapshot
            .diagnostics
            .0
            .into_iter()
            .map(|d| d.domain(INPUT_STAGE))
            .collect::<Result<Vec<_>, _>>()?;
        let recheck = Recheck::performed(Vec::new());
        let sources = Sources::new(disk, resource, buffer).map_err(|_| bad(INPUT_STAGE))?;
        let classified = ObservationOutcome::classify(
            prior_request,
            interval,
            Some(target.clone()),
            Some(ObservationEvidence::new(
                target, document, sources, dirty, recheck,
            )),
            Vec::new(),
            Vec::new(),
        )
        .map_err(|_| bad(INPUT_STAGE))?;
        let s = classified.snapshot().ok_or_else(|| bad(INPUT_STAGE))?;
        let c = s.comparisons();
        if classified.outcome() != input.basis.outcome
            || Agreement(s.agreement()) != snapshot.agreement
            || Comparison(c.disk_resource()) != snapshot.comparisons.disk_resource
            || Comparison(c.disk_buffer()) != snapshot.comparisons.disk_buffer
            || Comparison(c.resource_buffer()) != snapshot.comparisons.resource_buffer
            || s.diagnostics() != parsed_diagnostics
        {
            return Err(bad(INPUT_STAGE));
        }
        let top = input
            .basis
            .diagnostics
            .0
            .into_iter()
            .map(|d| d.domain(INPUT_STAGE))
            .collect::<Result<Vec<_>, _>>()?;
        if classified.diagnostics() != top {
            return Err(bad(INPUT_STAGE));
        }
        classified
    } else {
        if input.basis.resolved_target.0.is_some()
            || input.basis.outcome == OutcomeKind::CompleteObservation
        {
            return Err(bad(INPUT_STAGE));
        }
        if let Some(selection) = input.basis.selection.0 {
            if selection.missing_selector != "session_id"
                || input.basis.outcome != OutcomeKind::AmbiguousTarget
            {
                return Err(bad(INPUT_STAGE));
            }
            let _ = selection
                .candidate_sessions
                .0
                .into_iter()
                .map(SessionId::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| bad(INPUT_STAGE))?;
        }
        return Ok(Err((id.clone(), script_edit::BasisError::Missing)));
    };
    let basis = match ExpectedRevisionBasis::from_observation(&prior) {
        Ok(basis) => basis,
        Err(error) => return Ok(Err((id.clone(), error))),
    };
    EditRequest::new(id.clone(), project, session, script, basis, replacement)
        .map(Ok)
        .map_err(|_| bad(INPUT_STAGE))
}

pub fn input_limit() -> usize {
    INPUT_LIMIT
}
