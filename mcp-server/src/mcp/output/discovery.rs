//! Preserve the established discovery-v1 record, including partial coverage.
use super::{Identity, Interval, Selection, Stamp};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
pub(super) struct DiscoveryResult<'a> {
    schema_version: u8,
    request_id: &'a str,
    outcome: Outcome,
    interval: Interval,
    requested_target: Option<Requested<'a>>,
    resolved_target: Option<Resolved<'a>>,
    inventory: Option<Inventory<'a>>,
    diagnostics: Vec<Diagnostic<'a>>,
    selection: Option<Selection<'a>>,
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Outcome {
    CompleteListing,
    LimitedListing,
    Refused,
    Interrupted,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Requested<'a> {
    project_root: &'a str,
    session_id: Option<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Resolved<'a> {
    request_id: &'a str,
    project_root: &'a str,
    project_file_id: Identity<'a>,
    session_id: &'a str,
    godot_version: Version<'a>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Version<'a> {
    version: &'a str,
    hash: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields, bound(deserialize = "'de: 'a"))]
struct Inventory<'a> {
    scope: Scope<'a>,
    entries: Vec<&'a str>,
    collection: Stamp<'a>,
    coverage: Coverage,
    validity: Validity,
    consistency: Consistency,
    visited_entries: u32,
    visited_directories: u32,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Scope<'a> {
    policy: &'a str,
    project_data_directory: &'a str,
    exclusions: Vec<&'a str>,
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Coverage {
    Complete,
    Partial,
    NotStarted,
}
#[derive(Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Validity {
    Observed,
    EarlierObservation,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Consistency {
    recheck: Recheck,
    stability: Stability,
    atomic: bool,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Recheck {
    Completed,
    Unavailable,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Stability {
    Changed,
    Unchanged,
    Unknown,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Diagnostic<'a> {
    code: &'a str,
    stage: &'a str,
    scope: Option<&'a str>,
    message: &'a str,
    action: &'a str,
    omitted_count: Option<u32>,
}

pub(super) fn validate(value: &Value, request_id: &str) -> Result<bool, ()> {
    let result = DiscoveryResult::deserialize(value).map_err(|_| ())?;
    if result.schema_version != 1
        || result.request_id != request_id
        || result
            .requested_target
            .as_ref()
            .is_some_and(|t| !super::valid_selectors(t.project_root, t.session_id, None))
        || result.resolved_target.as_ref().is_some_and(|t| {
            t.request_id != request_id
                || !super::valid_selectors(t.project_root, Some(t.session_id), None)
        })
        || result.selection.as_ref().is_some_and(|s| !s.valid())
    {
        return Err(());
    }
    if let Some(inventory) = &result.inventory {
        if inventory.entries.len() > 1024
            || inventory.consistency.atomic
            || inventory
                .entries
                .iter()
                .any(|entry| crate::observation::ResourcePath::new(*entry).is_err())
            || result.outcome == Outcome::CompleteListing
                && (inventory.coverage != Coverage::Complete
                    || inventory.validity != Validity::Observed)
        {
            return Err(());
        }
    } else if result.outcome == Outcome::CompleteListing {
        return Err(());
    }
    Ok(matches!(
        result.outcome,
        Outcome::Refused | Outcome::Interrupted
    ))
}
