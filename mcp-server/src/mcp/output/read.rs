//! The read projection's public shape; borrow source while checking the SDK object.
use super::{Diagnostic, Interval, Selection, Target};
use crate::observation::SOURCE_LIMIT_BYTES;
use crate::script_read::ScriptRevision;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
pub(super) struct ReadResult<'a> {
    source: Option<&'a str>,
    revision: Option<&'a str>,
    state: State<'a>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct State<'a> {
    status: Status,
    target: Option<ReadTarget<'a>>,
    requested_target: Option<Target<'a>>,
    document: Document<'a>,
    source_origin: Option<Origin>,
    sources: Option<Sources<'a>>,
    dirty: Option<Dirty<'a>>,
    consistency: Option<Consistency<'a>>,
    interval: Interval,
    diagnostics: Vec<Diagnostic<'a>>,
    limitations: Vec<&'a str>,
    revision_unavailable_reason: Option<&'a str>,
    next_action: Action,
    selection: Option<Selection<'a>>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Status {
    CompleteObservation,
    LimitedObservation,
    NotOpen,
    AmbiguousTarget,
    MissingTarget,
    InvalidTarget,
    EditorUnavailable,
    DisconnectedEditor,
    Timeout,
    UnsupportedObservation,
    DeniedAccess,
    InvalidRequest,
    ProtocolError,
    Cancelled,
}
#[derive(Deserialize, serde::Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Origin {
    Disk,
    LoadedResource,
    EditorBuffer,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Action {
    EditWithCurrentRevision,
    SpecifySession,
    CheckAccessAndTarget,
    CheckSelectedEditor,
    FreshRead,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct ReadTarget<'a> {
    project_root: &'a str,
    script_path: &'a str,
    session_id: &'a str,
    kind: Option<Kind>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Kind {
    ExternalGdscript,
    BuiltinGdscript,
}
#[derive(Deserialize, serde::Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Lifecycle {
    Open,
    Closed,
    Unknown,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Document<'a> {
    lifecycle: Lifecycle,
    identity: Option<&'a str>,
    validity: Option<Validity>,
    validity_reason: Option<&'a str>,
    invalidated_validity: Option<Invalidated<'a, Validity>>,
    open_state_reason: Option<&'a str>,
    invalidated_lifecycle: Option<Invalidated<'a, OpenState>>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Validity {
    Valid,
    Missing,
    Invalid,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum OpenState {
    Open,
    NotOpen,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Invalidated<'a, T> {
    value: T,
    reason: &'a str,
}
#[derive(Deserialize, serde::Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Availability {
    Observed,
    Unavailable,
    NotApplicable,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Sources<'a> {
    disk: Source<'a>,
    loaded_resource: Source<'a>,
    editor_buffer: Source<'a>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Source<'a> {
    availability: Availability,
    current: bool,
    equals_source: Option<bool>,
    text: Option<&'a str>,
    text_origin: Option<Origin>,
    reason: Option<&'a str>,
    invalidated: Option<Historical<'a>>,
    staleness: Option<Staleness<'a>>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Historical<'a> {
    text: Option<&'a str>,
    equals_source: Option<bool>,
    reason: &'a str,
    current: bool,
    staleness: Staleness<'a>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Staleness<'a> {
    state: StaleState,
    evidence: Option<Vec<StaleEvidence<'a>>>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum StaleState {
    KnownStale,
    Unknown,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct StaleEvidence<'a> {
    incorporated_version: &'a str,
    changed_authority: Authority,
    changed_version: &'a str,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
enum Authority {
    D,
    R,
    B,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Dirty<'a> {
    buffer: BufferDirty<'a>,
    loaded_resource: ResourceDirty,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct BufferDirty<'a> {
    availability: Availability,
    state: Option<DirtyState>,
    reason: Option<&'a str>,
    invalidated: Option<OldDirty<'a>>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct ResourceDirty {
    availability: Availability,
    state: Option<DirtyState>,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct OldDirty<'a> {
    state: DirtyState,
    reason: &'a str,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum DirtyState {
    Dirty,
    Clean,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Consistency<'a> {
    atomic: bool,
    checks: Checks,
    stability: Stability,
    recheck_reason: Option<&'a str>,
    detected_changes: Vec<Change<'a>>,
    comparisons: Comparisons,
    agreement: Agreement,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Checks {
    Performed,
    Unavailable,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Stability {
    Changed,
    Unchanged,
    Unknown,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Change<'a> {
    surface: &'a str,
    code: &'a str,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Comparisons {
    disk_resource: Comparison,
    disk_buffer: Comparison,
    resource_buffer: Comparison,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Comparison {
    Equal,
    Different,
    Unknown,
}
#[derive(Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Agreement {
    Agree,
    Divergent,
    Unknown,
}

pub(super) fn validate(value: &Value) -> Result<bool, ()> {
    let read = ReadResult::deserialize(value).map_err(|_| ())?;
    let state = &read.state;
    if read.source.is_some_and(|s| s.len() > SOURCE_LIMIT_BYTES)
        || read
            .revision
            .is_some_and(|r| ScriptRevision::new(r).is_err())
        || read.revision.is_some() == state.revision_unavailable_reason.is_some()
        || read.source.is_some() != state.source_origin.is_some()
        || state.consistency.as_ref().is_some_and(|c| c.atomic)
        || !state.interval.valid()
        || state.selection.as_ref().is_some_and(|s| !s.valid())
        || state.requested_target.as_ref().is_some_and(|t| !t.valid())
        || state.target.as_ref().is_some_and(|t| {
            !super::valid_selectors(t.project_root, Some(t.session_id), Some(t.script_path))
        })
        || state.document.identity.is_some_and(|s| !super::hex(s, 64))
        || state.document.lifecycle != Lifecycle::Open && state.document.identity.is_some()
    {
        return Err(());
    }
    if let Some(sources) = &state.sources {
        for source in [
            &sources.disk,
            &sources.loaded_resource,
            &sources.editor_buffer,
        ] {
            if source.current != (source.availability == Availability::Observed)
                || source
                    .text
                    .is_some_and(|t| t.len() > SOURCE_LIMIT_BYTES || Some(t) == read.source)
                || source.invalidated.as_ref().is_some_and(|v| {
                    v.current || v.text.is_some_and(|t| t.len() > SOURCE_LIMIT_BYTES)
                })
                || source.equals_source == Some(true) && read.source.is_none()
            {
                return Err(());
            }
        }
    }
    // This check is presentation-only; dirty/limited/closed information remains successful.
    Ok(!matches!(
        state.status,
        Status::CompleteObservation | Status::LimitedObservation | Status::NotOpen
    ))
}
