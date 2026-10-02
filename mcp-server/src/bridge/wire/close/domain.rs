use super::evidence::Selection;
use super::*;
use crate::script_close as core;
fn fixed(value: &str, values: &[&'static str]) -> Result<&'static str, RoutingFailure> {
    values
        .iter()
        .copied()
        .find(|v| *v == value)
        .ok_or_else(|| bad(Stage::ReadEditor))
}
pub(crate) fn stamp(
    value: &serde_json::Value,
    session: &SessionId,
    receipt: u64,
) -> Result<CollectionStamp, RoutingFailure> {
    StampIn::deserialize(value)
        .map_err(|_| bad(Stage::ReadEditor))?
        .domain(session, receipt, Stage::ReadEditor, true)
}
pub(crate) fn selection(value: &Selection) -> Result<core::Selection, RoutingFailure> {
    Ok(core::Selection {
        before: fixed(
            &value.before,
            &["target", "other", "no_source_editor", "unknown"],
        )?,
        after: fixed(
            &value.after,
            &["target", "other", "no_source_editor", "unknown"],
        )?,
        request_effect: fixed(
            &value.request_effect,
            &["none", "native_fallback", "unknown"],
        )?,
    })
}
pub(crate) fn protection(value: &Protection) -> Result<core::Protection, RoutingFailure> {
    Ok(core::Protection {
        status: fixed(
            &value.status,
            &["not_applicable", "preserved", "unavailable", "invalidated"],
        )?,
        revalidation: fixed(
            &value.revalidation,
            &[
                "not_applicable",
                "pending",
                "completed",
                "unavailable",
                "invalidated",
            ],
        )?,
        required_count: value.required_count,
        completed_count: value.completed_count,
        reason: value.reason.as_ref().map(|_| "context_changed"),
    })
}
pub(crate) fn resource(
    value: &ResourceState,
    session: &SessionId,
    receipt: u64,
) -> Result<core::ResourceState, RoutingFailure> {
    Ok(core::ResourceState {
        state: fixed(
            &value.state,
            &[
                "retained",
                "unloaded",
                "unavailable",
                "invalidated",
                "not_collected",
            ],
        )?,
        resource_edited: value.resource_edited,
        reason: value.reason.as_ref().map(|_| {
            if value.state == "invalidated" {
                "verification_failed"
            } else {
                "evidence_unavailable"
            }
        }),
        collection: value
            .collection
            .as_ref()
            .map(|c| stamp(c, session, receipt))
            .transpose()?,
    })
}
pub(crate) fn native(
    value: &Native,
    session: &SessionId,
    receipt: u64,
) -> Result<core::NativeEvidence, RoutingFailure> {
    let continuation = value
        .continuation
        .as_ref()
        .map(|c| -> Result<core::Step, RoutingFailure> {
            let state = match c.state.as_str() {
                "not_applicable" => "not_applicable",
                "pending" => "entered",
                "completed" => "completed",
                "invalidated" => "failed",
                "unavailable" => "unknown",
                _ => return Err(bad(Stage::ReadEditor)),
            };
            let mut latest: Option<CollectionStamp> = None;
            for row in &c.collections {
                if let Some(value) = &row.completion {
                    let current = stamp(value, session, receipt)?;
                    if latest
                        .as_ref()
                        .is_none_or(|old| current.finished_tick_us() > old.finished_tick_us())
                    {
                        latest = Some(current);
                    }
                }
            }
            Ok(core::Step {
                state,
                reason: c.reason.as_ref().map(|_| "native_revalidation_unavailable"),
                collection: latest,
            })
        })
        .transpose()?;
    Ok(core::NativeEvidence {
        entered: value.entered,
        removed: value.old_document_removed,
        discard: value.terminal_discard,
        invalidated: value.invalidated,
        target_buffer: fixed(
            &value.target_buffer,
            &["retained", "disposed", "unavailable", "not_applicable"],
        )?,
        selection: value.selection.as_ref().map(selection).transpose()?,
        protection: value.protection.as_ref().map(protection).transpose()?,
        continuation,
        entry: value
            .entry_collection
            .as_ref()
            .map(|v| stamp(v, session, receipt))
            .transpose()?,
        returned: value
            .return_collection
            .as_ref()
            .map(|v| stamp(v, session, receipt))
            .transpose()?,
    })
}
