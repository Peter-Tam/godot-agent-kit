use super::*;
use crate::script_close::{CloseRequest, ClosingOutcome};
use crate::script_edit::BasisError;
use serde::Deserialize;
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
fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    T::deserialize(d)
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
    #[serde(deserialize_with = "present")]
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
    basis: RequiredOption<BasisIn>,
}
pub type DecodedRequest = Result<CloseRequest, (RequestId, &'static str)>;
#[derive(Debug)]
pub struct InputError {
    pub request_id: Option<RequestId>,
}
pub fn input_limit() -> usize {
    INPUT_LIMIT
}
pub fn decode_request(
    bytes: &[u8],
    project: ProjectRoot,
    session: Option<SessionId>,
    script: ResourcePath,
) -> Result<DecodedRequest, InputError> {
    strict::check_request(bytes, INPUT_STAGE).map_err(|_| InputError { request_id: None })?;
    let mut input: InputIn =
        serde_json::from_slice(bytes).map_err(|_| InputError { request_id: None })?;
    let id = RequestId::new(std::mem::take(&mut input.request_id))
        .map_err(|_| InputError { request_id: None })?;
    let bad_input = || InputError {
        request_id: Some(id.clone()),
    };
    if input.schema_version != 1 {
        return Err(bad_input());
    }
    let supplied = input.basis.0.is_some();
    let prior = input
        .basis
        .0
        .map(|basis| prior(basis, project.clone(), session.clone(), script.clone()))
        .transpose()
        .map_err(|_| bad_input())?
        .flatten();
    if supplied && prior.is_none() {
        return Ok(Err((id, "invalid_basis")));
    }
    if let Some(target) = prior.as_ref().and_then(ObservationOutcome::resolved_target) {
        let reason = if target.project_root() != &project {
            Some("target_changed")
        } else if session
            .as_ref()
            .is_some_and(|value| value != target.session_id())
        {
            Some("session_changed")
        } else if target.script_path() != &script {
            Some("identity_changed")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Ok(Err((id, reason)));
        }
    }
    Ok(
        CloseRequest::new(id.clone(), project, session, script, prior.as_ref()).map_err(|error| {
            (
                id,
                match error {
                    BasisError::Dirty => "dirty_conflict",
                    BasisError::Divergent => "source_divergence",
                    BasisError::KnownStaleResource => "stale_resource",
                    BasisError::KnownStaleBuffer => "stale_buffer",
                    BasisError::UnsupportedTarget => "invalid_document_kind",
                    BasisError::UnsupportedRepresentation => "unsupported_representation",
                    _ => "invalid_basis",
                },
            )
        }),
    )
}
fn prior(
    basis: BasisIn,
    project: ProjectRoot,
    session: Option<SessionId>,
    script: ResourcePath,
) -> Result<Option<ObservationOutcome>, RoutingFailure> {
    if basis.schema_version != 1 {
        return Err(bad(INPUT_STAGE));
    }
    let prior_id = RequestId::new(basis.request_id).map_err(|_| bad(INPUT_STAGE))?;
    // Historical public evidence is decoded in its own target/session domain.
    // Binding it to this request is a separate refusal, not malformed evidence.
    let (project, session, script) = if let Some(target) = &basis.resolved_target.0 {
        (
            ProjectRoot::new(target.project_root.clone()).map_err(|_| bad(INPUT_STAGE))?,
            Some(SessionId::new(target.session_id.clone()).map_err(|_| bad(INPUT_STAGE))?),
            ResourcePath::new(target.script_path.clone()).map_err(|_| bad(INPUT_STAGE))?,
        )
    } else {
        (project, session, script)
    };
    let prior_request =
        ObservationRequest::new(prior_id, project.clone(), session.clone(), script.clone());
    let interval = ObservationInterval::new(
        basis.interval.started_unix_ms,
        basis.interval.finished_unix_ms,
        basis.interval.elapsed_us,
    );
    let prior = if let Some(snapshot) = basis.snapshot.0 {
        if basis.selection.0.is_some() || basis.resolved_target.0.is_none() {
            return Err(bad(INPUT_STAGE));
        }
        let expected_target = basis
            .resolved_target
            .0
            .ok_or_else(|| bad(INPUT_STAGE))?
            .domain(&prior_request)?;
        let target = snapshot.target.domain(&prior_request)?;
        if target != expected_target || snapshot.consistency.atomic {
            return Err(bad(INPUT_STAGE));
        }
        let receipt = interval.elapsed_us();
        let document = basis::document(snapshot.document, &target, receipt)?;
        let disk = basis::source(snapshot.sources.disk, Authority::D, &target, receipt)?;
        let resource = basis::source(snapshot.sources.resource, Authority::R, &target, receipt)?;
        let buffer = basis::source(snapshot.sources.buffer, Authority::B, &target, receipt)?;
        let dirty = basis::dirty(snapshot.dirty, &target, receipt)?;
        let parsed_diagnostics = snapshot
            .diagnostics
            .0
            .into_iter()
            .map(|d| d.domain(INPUT_STAGE))
            .collect::<Result<Vec<_>, _>>()?;
        let changes = snapshot
            .consistency
            .detected_changes
            .0
            .into_iter()
            .map(|v| v.domain(INPUT_STAGE))
            .collect::<Result<Vec<_>, _>>()?;
        let recheck = match (
            snapshot.consistency.checks.as_str(),
            snapshot.consistency.recheck_reason,
        ) {
            ("performed", None) => Recheck::performed(changes),
            ("unavailable", Some(reason)) => Recheck::partial(reason, changes),
            _ => return Err(bad(INPUT_STAGE)),
        };
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
        if classified.outcome() != basis.outcome
            || Stability(s.consistency().stability()) != snapshot.consistency.stability
            || Agreement(s.agreement()) != snapshot.agreement
            || Comparison(c.disk_resource()) != snapshot.comparisons.disk_resource
            || Comparison(c.disk_buffer()) != snapshot.comparisons.disk_buffer
            || Comparison(c.resource_buffer()) != snapshot.comparisons.resource_buffer
            || s.diagnostics() != parsed_diagnostics
        {
            return Err(bad(INPUT_STAGE));
        }
        let top = basis
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
        if basis.resolved_target.0.is_some() || basis.outcome == OutcomeKind::CompleteObservation {
            return Err(bad(INPUT_STAGE));
        }
        if let Some(selection) = basis.selection.0 {
            if selection.missing_selector != "session_id"
                || basis.outcome != OutcomeKind::AmbiguousTarget
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
        return Ok(None);
    };
    Ok(Some(prior))
}
pub fn encode_outcome(value: &ClosingOutcome) -> Result<Vec<u8>, RoutingFailure> {
    output::encode(value)
}
pub fn exit_code(value: &ClosingOutcome) -> i32 {
    match value.outcome() {
        "verified_newly_closed" | "already_closed_unchanged" => 0,
        "refused" if value.reason() == "invalid_request" => 2,
        "refused" => 3,
        _ => 4,
    }
}
mod basis;
mod evidence;
pub(crate) use evidence::*;
pub(crate) mod domain;
pub(crate) fn call(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    opcode: &str,
    args: &[serde_json::Value],
    deadline: Instant,
) -> Result<Vec<u8>, RoutingFailure> {
    use serde_json::json;
    let arity = match opcode {
        "close_begin" | "close_recheck" | "close_verify" => 1,
        "close_prepare" | "close_advance" => 2,
        "close_wait" | "close_finish" | "close_abort" => 0,
        _ => return Err(bad(Stage::ReadEditor)),
    };
    if args.len() != arity
        || !selected.capabilities().close_gdscript
        || selected.native_api_revision() != 4
        || selected.target().request_id() != request.request_id()
    {
        return Err(bad(Stage::ReadEditor));
    }
    let mut tuple = vec![
        json!(6),
        json!(opcode),
        json!(request.request_id().as_str()),
        json!(selected.target().session_id().as_str()),
        json!(selected.advertised_project_root),
        json!(request.script_path().as_str()),
    ];
    tuple.extend_from_slice(args);
    let bytes = serde_json::to_vec(&tuple).map_err(|_| bad(Stage::ReadEditor))?;
    let limit = if opcode == "close_prepare" {
        4 * 1024 * 1024
    } else {
        4096
    };
    if bytes.len() > limit {
        return Err(bad(Stage::ReadEditor));
    }
    write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        Stage::ReadEditor,
    )?;
    write_deadline(&mut selected.socket, &bytes, deadline, Stage::ReadEditor)?;
    receive_editor(&mut selected.socket, deadline, Stage::ReadEditor)
}
mod output;
pub fn refusal(
    id: RequestId,
    interval: ObservationInterval,
    reason: &'static str,
    selectors: Option<(ProjectRoot, Option<SessionId>, ResourcePath)>,
) -> ClosingOutcome {
    let known = selectors.is_some();
    let (project, session, path) = selectors.unwrap_or_else(|| {
        (
            ProjectRoot::new("/").expect("root selector"),
            None,
            ResourcePath::new("res://unknown.gd").expect("constant path"),
        )
    });
    let request = CloseRequest::boundary(ObservationRequest::new(id, project, session, path));
    let mut attempt = crate::script_close::Attempt::new(request, interval.clone());
    if !known {
        attempt.result.request = None;
    }
    attempt.fail(reason);
    attempt.finish(interval)
}
mod strict;
pub(crate) fn decode<'a, T: Deserialize<'a>>(
    bytes: &'a [u8],
    stage: Stage,
) -> Result<T, RoutingFailure> {
    strict::check(bytes, stage)?;
    serde_json::from_slice(bytes).map_err(|_| bad(stage))
}
#[cfg(test)]
mod tests;
