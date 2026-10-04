//! Source-free edit DTOs; internal bases, object handles and collections never escape.
mod projection;
use super::{Application, Diagnostic, Interval, NextAction, Selection, Target};
pub(super) use projection::project;
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum EditResult<'a> {
    Open { outcome: &'a OpenOutcome<'a> },
    Closed { outcome: &'a ClosedOutcome<'a> },
    Undetermined { outcome: &'a Refusal<'a> },
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Outcome {
    VerifiedChanged,
    VerifiedUnchanged,
    Refused,
    AppliedUnverified,
    ApplicationUnknown,
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum History {
    NotParticipated,
    NativeComplexEdit,
    NotApplicableClosed,
    Unknown,
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum LifecycleState {
    Open,
    Closed,
    Unknown,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Lifecycle {
    admitted: Option<LifecycleState>,
    observed_final: LifecycleState,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
pub(super) struct Refusal<'a> {
    request_id: &'a str,
    interval: Interval,
    requested_target: Option<Target<'a>>,
    target: Option<Target<'a>>,
    outcome: Outcome,
    application: Application,
    reason: &'a str,
    stage: &'a str,
    diagnostics: Vec<Diagnostic<'a>>,
    selection: Option<Selection<'a>>,
    history: History,
    lifecycle: Lifecycle,
    revision: Option<&'a str>,
    next_action: NextAction<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
pub(super) struct ClosedOutcome<'a> {
    request_id: &'a str,
    interval: Interval,
    target: Option<Target<'a>>,
    outcome: Outcome,
    application: Application,
    reason: &'a str,
    stage: &'a str,
    history: History,
    lifecycle: Lifecycle,
    revision: Option<&'a str>,
    effects: Effects,
    evidence: Option<ClosedEvidence<'a>>,
    limitations: Vec<&'a str>,
    next_action: NextAction<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Effects {
    authorized: bool,
    entry_refused: bool,
    resource_entered: bool,
    resource_changed: bool,
    disk_entered: bool,
    written_bytes: u64,
    truncated: bool,
    flushed: bool,
    readback: bool,
    mtime_restored: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct ClosedEvidence<'a> {
    before: ClosedSources<'a>,
    after: Option<ClosedSources<'a>>,
    validation: ClosedValidation,
    atomic: bool,
    independently_verified: bool,
    buffer: &'a str,
    resource: ResourceApplicability<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct ClosedValidation {
    original: bool,
    desired: bool,
    actual: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct ClosedSources<'a> {
    disk: Digest<'a>,
    resource: Resource<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Digest<'a> {
    sha256: &'a str,
    utf8_bytes: usize,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Resource<'a> {
    state: &'a str,
    edited: Option<bool>,
    sha256: Option<&'a str>,
    utf8_bytes: Option<usize>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct ResourceApplicability<'a> {
    admitted: &'a str,
    availability: &'a str,
    state: Option<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
pub(super) struct OpenOutcome<'a> {
    request_id: &'a str,
    requested_target: Target<'a>,
    resolved_target: Option<Target<'a>>,
    interval: Interval,
    outcome: Outcome,
    reason: &'a str,
    stage: &'a str,
    application: Application,
    progress: Progress<'a>,
    revision: Option<&'a str>,
    before: Option<Evidence<'a>>,
    after: Option<Evidence<'a>>,
    persistence: Option<Persistence<'a>>,
    finalization: Option<Finalization<'a>>,
    validation: Vec<Validation<'a>>,
    context_recheck: Option<Context<'a>>,
    history: History,
    diagnostics: Vec<EditDiagnostic<'a>>,
    selection: Option<Selection<'a>>,
    safe_next_action: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct EditDiagnostic<'a> {
    stage: &'a str,
    reason: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Progress<'a> {
    buffer_application: Step<'a>,
    resource_sync: Step<'a>,
    persistence: Step<'a>,
    finalization: Step<'a>,
    validation: Step<'a>,
    verification: Step<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Step<'a> {
    state: StepState,
    reason: Option<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum StepState {
    NotStarted,
    Entered,
    Completed,
    Failed,
    Unknown,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Evidence<'a> {
    document: Document<'a>,
    sources: Sources<'a>,
    dirty: Dirty<'a>,
    saved_state: Option<Saved<'a>>,
    saved_state_reason: Option<&'a str>,
    comparisons: Comparisons<'a>,
    agreement: &'a str,
    consistency: Consistency<'a>,
    disk_metadata: Option<Disk<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Document<'a> {
    identity: Option<&'a str>,
    validity: Fact<'a>,
    open_state: Fact<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Fact<'a> {
    value: Option<&'a str>,
    reason: Option<Reason<'a>>,
    invalidated_evidence: Option<OldFact<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct OldFact<'a> {
    value: &'a str,
    reason: Reason<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Reason<'a> {
    code: &'a str,
    action: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Sources<'a> {
    #[serde(rename = "D")]
    disk: Surface<'a>,
    #[serde(rename = "R")]
    resource: Surface<'a>,
    #[serde(rename = "B")]
    buffer: Surface<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Surface<'a> {
    authority: &'a str,
    availability: &'a str,
    source_sha256: Option<&'a str>,
    utf8_bytes: Option<usize>,
    staleness: Option<Staleness<'a>>,
    reason: Option<&'a str>,
    invalidated_evidence: Option<OldSurface<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct OldSurface<'a> {
    source_sha256: &'a str,
    utf8_bytes: usize,
    staleness: Staleness<'a>,
    reason: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Staleness<'a> {
    state: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<Vec<StaleEvidence<'a>>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct StaleEvidence<'a> {
    incorporated_version: &'a str,
    changed_authority: &'a str,
    changed_version: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Dirty<'a> {
    availability: &'a str,
    state: &'a str,
    reason: Option<Reason<'a>>,
    invalidated_evidence: Option<OldDirty<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct OldDirty<'a> {
    state: &'a str,
    reason: Reason<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Saved<'a> {
    current_version: &'a str,
    saved_version: &'a str,
    resource_edited: bool,
    save_profile: &'a str,
    original_preserved: bool,
    desired_preserved: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Comparisons<'a> {
    disk_resource: &'a str,
    disk_buffer: &'a str,
    resource_buffer: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Consistency<'a> {
    atomic: bool,
    checks: &'a str,
    stability: &'a str,
    detected_changes: Vec<Change<'a>>,
    recheck_reason: Option<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Change<'a> {
    surface: &'a str,
    code: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Disk<'a> {
    mtime: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Persistence<'a> {
    intended: Digest<'a>,
    write_started: bool,
    bytes_written: u64,
    truncated: bool,
    flushed: bool,
    readback_matches: bool,
    attached: bool,
    descriptor_open: bool,
    interference: bool,
    original_mtime: Option<&'a str>,
    mtime: Option<&'a str>,
    restore_attempted: bool,
    restore_errno: Option<i32>,
    restored: bool,
    reason: Option<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Finalization<'a> {
    status: &'a str,
    reason: Option<&'a str>,
    before_source: Option<Digest<'a>>,
    after_source: Option<Digest<'a>>,
    before_current: Option<&'a str>,
    after_current: Option<&'a str>,
    before_saved: Option<&'a str>,
    after_saved: Option<&'a str>,
    before_resource_edited: Option<bool>,
    after_resource_edited: Option<bool>,
    steps: FinalizationSteps,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct FinalizationSteps {
    resource_edited: bool,
    saved_version: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Validation<'a> {
    status: &'a str,
    reason: Option<&'a str>,
    purpose: &'a str,
    source_path: &'a str,
    input: Digest<'a>,
    context: &'a str,
    context_current: bool,
    dependencies_current: bool,
    cleanup_confirmed: bool,
    sources: Vec<Fence<'a>>,
    dependencies: Vec<Dependency<'a>>,
    diagnostics: Vec<ValidationDiagnostic<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Fence<'a> {
    path: &'a str,
    source: Digest<'a>,
    diagnostics_completed: bool,
    symbols_completed: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Dependency<'a> {
    path: &'a str,
    source: Digest<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct ValidationDiagnostic<'a> {
    origin: &'a str,
    path: Option<&'a str>,
    path_reason: Option<&'a str>,
    source: Option<Digest<'a>>,
    line: Option<u32>,
    column: Option<u32>,
    message: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Context<'a> {
    context: &'a str,
    current: bool,
    dependencies: Vec<Dependency<'a>>,
}

fn terminal(outcome: &Outcome, application: Application) -> bool {
    match outcome {
        Outcome::VerifiedChanged => application == Application::Applied,
        Outcome::VerifiedUnchanged | Outcome::Refused => application == Application::NotApplied,
        Outcome::ApplicationUnknown => application == Application::Unknown,
        Outcome::AppliedUnverified => matches!(
            application,
            Application::Applied | Application::PartlyApplied
        ),
    }
}
pub(super) fn retained_application(value: &Value) -> Application {
    let outcome = &value["outcome"];
    let application =
        Application::deserialize(&outcome["application"]).unwrap_or(Application::Unknown);
    let effects = &outcome["effects"];
    if application == Application::NotApplied {
        if effects["resource_changed"] == true
            || effects["written_bytes"].as_u64().is_some_and(|n| n > 0)
            || effects["truncated"] == true
        {
            return Application::PartlyApplied;
        }
        if effects["resource_entered"] == true
            || effects["disk_entered"] == true
            || effects["authorized"] == true && effects["entry_refused"] != true
        {
            return Application::Unknown;
        }
    }
    application
}

pub(super) fn validate(value: &Value, id: &str) -> Result<bool, ()> {
    let record = value.as_object().ok_or(())?;
    if record.len() != 2 {
        return Err(());
    }
    let outcome = record.get("outcome").ok_or(())?;
    match record.get("mode").and_then(Value::as_str) {
        Some("open") => {
            let outcome = OpenOutcome::deserialize(outcome).map_err(|_| ())?;
            validate_typed(EditResult::Open { outcome: &outcome }, id)
        }
        Some("closed") => {
            let outcome = ClosedOutcome::deserialize(outcome).map_err(|_| ())?;
            validate_typed(EditResult::Closed { outcome: &outcome }, id)
        }
        Some("undetermined") => {
            let outcome = Refusal::deserialize(outcome).map_err(|_| ())?;
            validate_typed(EditResult::Undetermined { outcome: &outcome }, id)
        }
        _ => Err(()),
    }
}

fn validate_typed(result: EditResult<'_>, id: &str) -> Result<bool, ()> {
    let (request_id, interval, outcome, application) = match &result {
        EditResult::Open { outcome: o } => (&o.request_id, &o.interval, &o.outcome, o.application),
        EditResult::Closed { outcome: o } => {
            (&o.request_id, &o.interval, &o.outcome, o.application)
        }
        EditResult::Undetermined { outcome: o } => {
            (&o.request_id, &o.interval, &o.outcome, o.application)
        }
    };
    if *request_id != id || !interval.valid() || !terminal(outcome, application) {
        return Err(());
    }
    let success = matches!(
        outcome,
        Outcome::VerifiedChanged | Outcome::VerifiedUnchanged
    );
    match result {
        EditResult::Closed { outcome: o } => {
            if o.lifecycle.admitted != Some(LifecycleState::Closed)
                || o.lifecycle.observed_final != LifecycleState::Closed
                    && o.history == History::NotApplicableClosed
                || o.evidence.as_ref().is_some_and(|e| e.atomic)
                || o.outcome == Outcome::VerifiedUnchanged
                    && (o.effects.authorized
                        || o.effects.resource_entered
                        || o.effects.resource_changed
                        || o.effects.disk_entered
                        || o.effects.written_bytes != 0
                        || o.effects.truncated
                        || o.effects.flushed
                        || o.effects.readback
                        || o.effects.mtime_restored)
                || success
                    && (o.lifecycle.observed_final != LifecycleState::Closed
                        || !o.evidence.as_ref().is_some_and(|e| {
                            e.independently_verified
                                && e.validation.original
                                && e.validation.desired
                                && e.validation.actual
                                && e.buffer == "not_applicable"
                        }))
            {
                return Err(());
            }
        }
        EditResult::Open { outcome: o } => {
            if success
                && !o.after.as_ref().is_some_and(|a| {
                    a.document.open_state.value == Some("open")
                        && a.agreement == "agree"
                        && !a.consistency.atomic
                })
            {
                return Err(());
            }
        }
        EditResult::Undetermined { outcome: o } => {
            if o.outcome != Outcome::Refused {
                return Err(());
            }
        }
    }
    Ok(!success)
}
