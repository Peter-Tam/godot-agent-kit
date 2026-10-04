use super::*;
use crate::runner::stock_validation::{CloseContext, WarningSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub(super) struct Nullable<T>(pub Option<T>);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(d).map(Self)
    }
}
fn required<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    T::deserialize(d)
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceState {
    pub state: String,
    #[serde(deserialize_with = "required")]
    pub resource_edited: Option<bool>,
    #[serde(deserialize_with = "required")]
    pub reason: Option<String>,
    #[serde(deserialize_with = "required")]
    pub collection: Option<Value>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Protection {
    pub status: String,
    pub revalidation: String,
    #[serde(deserialize_with = "required")]
    pub required_count: Option<usize>,
    #[serde(deserialize_with = "required")]
    pub completed_count: Option<usize>,
    #[serde(deserialize_with = "required")]
    pub reason: Option<String>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {
    pub before: String,
    pub after: String,
    pub request_effect: String,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Continuation {
    pub state: String,
    #[serde(deserialize_with = "required")]
    pub reason: Option<String>,
    #[serde(deserialize_with = "seven")]
    pub required_editor_ids: Vec<String>,
    #[serde(deserialize_with = "seven")]
    pub completed_editor_ids: Vec<String>,
    #[serde(deserialize_with = "seven")]
    pub collections: Vec<ContinuationCollection>,
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContinuationCollection {
    pub editor_id: String,
    #[serde(deserialize_with = "required")]
    pub visit: Option<Value>,
    #[serde(deserialize_with = "required")]
    pub completion: Option<Value>,
}
fn seven<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Vec<T>, D::Error> {
    Bounded::<T, 7>::deserialize(d).map(|v| v.0)
}
fn globals<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    let values = Bounded::<String, 256>::deserialize(d)?.0;
    if values.iter().any(|v| {
        v.is_empty() || v.len() > 256 || !v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    }) || values
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != values.len()
    {
        return Err(serde::de::Error::custom("invalid global classes"));
    }
    Ok(values)
}
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Native {
    pub request_id: String,
    pub session_id: String,
    pub script_path: String,
    pub native_build_id: String,
    pub native_api_revision: u32,
    pub phase: String,
    #[serde(deserialize_with = "required")]
    pub script_instance_id: Option<String>,
    #[serde(deserialize_with = "required")]
    pub editor_instance_id: Option<String>,
    #[serde(deserialize_with = "required")]
    pub buffer_instance_id: Option<String>,
    #[serde(deserialize_with = "required")]
    pub entry_collection: Option<Value>,
    #[serde(deserialize_with = "required")]
    pub return_collection: Option<Value>,
    pub entered: bool,
    #[serde(deserialize_with = "required")]
    pub close_error: Option<i64>,
    #[serde(deserialize_with = "required")]
    pub old_document_removed: Option<bool>,
    #[serde(deserialize_with = "required")]
    pub selection: Option<Selection>,
    pub target_buffer: String,
    #[serde(deserialize_with = "required")]
    pub protection: Option<Protection>,
    #[serde(deserialize_with = "required")]
    pub continuation: Option<Continuation>,
    pub invalidated: bool,
    #[serde(deserialize_with = "required")]
    pub reason: Option<String>,
    pub terminal_discard: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Validation {
    pub document_path: String,
    pub script_id: String,
    pub editor_id: String,
    pub buffer_id: String,
    pub executable: String,
    #[serde(deserialize_with = "crate::bridge::wire::open::checked_warnings")]
    pub warnings: WarningSettings,
    #[serde(deserialize_with = "globals")]
    pub global_classes: Vec<String>,
}
macro_rules! response {($name:ident {$($field:ident:$ty:ty),*})=>{#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct $name {v:u32,kind:String,request_id:String,session_id:String,project_root:String,script_path:String,collection:StampIn,status:String,#[serde(deserialize_with="present")]reason:Nullable<String>,#[serde(deserialize_with="present")]native:Nullable<Native>,#[serde(deserialize_with="present")]expiry_tick_us:Nullable<String>,$(#[serde(deserialize_with="present")]$field:$ty),*}};}
response!(State {sample:Nullable<SampleIn>,resource_edited:Nullable<bool>,resource_state:Nullable<ResourceState>,selection:Nullable<Selection>});
response!(Prepared {sample:Nullable<SampleIn>,target_guard:Nullable<Value>,context:Nullable<CloseContext>,guard_sha256:Nullable<String>,validation:Nullable<Bounded<Validation,7>>});
response!(Rechecked {purpose:String,recheck:Nullable<RecheckIn>,protection:Nullable<Protection>,selection:Nullable<Selection>,resource_state:Nullable<ResourceState>});
response!(Progress {});
response!(Waited {continuation:Continuation,protection:Nullable<Protection>});
response!(Sample {purpose:String,sample:Nullable<SampleIn>,resource_edited:Nullable<bool>,resource_state:Nullable<ResourceState>,protection:Nullable<Protection>,selection:Nullable<Selection>});
response!(Terminal {
    terminal_discard: bool
});
pub(crate) struct Reply {
    pub status: String,
    pub reason: Option<String>,
    pub collection: CollectionStamp,
    pub native: Option<Native>,
    pub sample: Option<EditorSample>,
    pub context: Option<CloseContext>,
    pub guard: Option<String>,
    pub validation: Option<Vec<Validation>>,
    pub target_guard: Option<Value>,
    pub recheck: Option<Recheck>,
    pub purpose: Option<String>,
    pub continuation: Option<Continuation>,
    pub protection: Option<Protection>,
    pub resource: Option<ResourceState>,
    pub selection: Option<Selection>,
    pub edited: Option<bool>,
    pub discard: Option<bool>,
    pub expiry: Option<DecimalCounter>,
}
pub(crate) fn decode_reply(
    bytes: &[u8],
    kind: &str,
    request: &ObservationRequest,
    target: &ResolvedTarget,
    root: &str,
    receipt: u64,
    original: Option<&CollectionStamp>,
) -> Result<Reply, RoutingFailure> {
    let stage = Stage::ReadEditor;
    macro_rules! base {
        ($r:expr) => {{
            let r = $r;
            if r.v != 6
                || r.kind != kind
                || r.request_id != request.request_id().as_str()
                || r.session_id != target.session_id().as_str()
                || r.project_root != root
                || r.script_path != request.script_path().as_str()
            {
                return Err(bad(stage));
            }
            if r.collection.received_elapsed_us != 0 {
                return Err(bad(stage));
            }
            let expiry = r.expiry_tick_us.0.map(|v| decimal(v, stage)).transpose()?;
            let native = r.native.0;
            if let Some(n) = &native {
                if n.request_id != request.request_id().as_str()
                    || n.session_id != target.session_id().as_str()
                    || n.script_path != request.script_path().as_str()
                    || n.native_api_revision != 4
                    || n.native_build_id.is_empty()
                    || n.native_build_id.len() > 128
                    || ![
                        "inspected",
                        "prepared",
                        "entered",
                        "returned",
                        "settling",
                        "settled",
                        "terminal",
                    ]
                    .contains(&n.phase.as_str())
                {
                    return Err(bad(stage));
                }
                for id in [
                    &n.script_instance_id,
                    &n.editor_instance_id,
                    &n.buffer_instance_id,
                ]
                .into_iter()
                .flatten()
                {
                    if decimal(id.clone(), stage)?.as_str() == "0" {
                        return Err(bad(stage));
                    }
                }
                if n.close_error.is_some_and(|v| !(0..=255).contains(&v)) {
                    return Err(bad(stage));
                }
            }
            Reply {
                status: r.status,
                reason: r.reason.0,
                collection: r
                    .collection
                    .domain(target.session_id(), receipt, stage, true)?,
                native,
                sample: None,
                context: None,
                guard: None,
                validation: None,
                target_guard: None,
                recheck: None,
                purpose: None,
                continuation: None,
                protection: None,
                resource: None,
                selection: None,
                edited: None,
                discard: None,
                expiry,
            }
        }};
    }
    macro_rules! sample {
        ($s:expr) => {
            $s.map(|s| s.domain(request, target, root, receipt, stage))
                .transpose()?
        };
    }
    let reply = match kind {
        "close_state" => {
            let mut r: State = decode(bytes, stage)?;
            let s = sample!(r.sample.0.take());
            let resource = r.resource_state.0.take();
            let selection = r.selection.0.take();
            let edited = r.resource_edited.0;
            let mut out = base!(r);
            out.sample = s;
            out.resource = resource;
            out.selection = selection;
            out.edited = edited;
            out
        }
        "close_prepared" => {
            let mut r: Prepared = decode(bytes, stage)?;
            let s = sample!(r.sample.0.take());
            let context = r.context.0.take();
            let guard = r.guard_sha256.0.take();
            let validation = r.validation.0.take().map(|v| v.0);
            let target_guard = r.target_guard.0.take();
            let mut out = base!(r);
            out.sample = s;
            out.context = context;
            out.guard = guard;
            out.validation = validation;
            out.target_guard = target_guard;
            out
        }
        "close_rechecked" => {
            let mut r: Rechecked = decode(bytes, stage)?;
            let purpose = std::mem::take(&mut r.purpose);
            let check = r.recheck.0.take().ok_or_else(|| bad(stage))?.domain(
                request,
                target,
                root,
                receipt,
                stage,
                original.ok_or_else(|| bad(stage))?,
            )?;
            let protection = r.protection.0.take();
            let selection = r.selection.0.take();
            let resource = r.resource_state.0.take();
            let mut out = base!(r);
            out.purpose = Some(purpose);
            out.recheck = Some(check);
            out.protection = protection;
            out.selection = selection;
            out.resource = resource;
            out
        }
        "close_progress" => base!(decode::<Progress>(bytes, stage)?),
        "close_waited" => {
            let mut r: Waited = decode(bytes, stage)?;
            let continuation = std::mem::replace(
                &mut r.continuation,
                Continuation {
                    state: String::new(),
                    reason: None,
                    required_editor_ids: Vec::new(),
                    completed_editor_ids: Vec::new(),
                    collections: Vec::new(),
                },
            );
            let protection = r.protection.0.take();
            let mut out = base!(r);
            out.continuation = Some(continuation);
            out.protection = protection;
            out
        }
        "close_sample" => {
            let mut r: Sample = decode(bytes, stage)?;
            let s = sample!(r.sample.0.take());
            let purpose = std::mem::take(&mut r.purpose);
            let edited = r.resource_edited.0;
            let resource = r.resource_state.0.take();
            let protection = r.protection.0.take();
            let selection = r.selection.0.take();
            let mut out = base!(r);
            out.sample = s;
            out.purpose = Some(purpose);
            out.edited = edited;
            out.resource = resource;
            out.protection = protection;
            out.selection = selection;
            out
        }
        "close_finished" | "close_aborted" => {
            let r: Terminal = decode(bytes, stage)?;
            let discard = r.terminal_discard;
            let mut out = base!(r);
            out.discard = Some(discard);
            out
        }
        _ => return Err(bad(stage)),
    };
    let statuses: &[&str] = match kind {
        "close_state" => &["inspected", "refused"],
        "close_prepared" => &["prepared", "refused"],
        "close_rechecked" => &["rechecked", "refused"],
        "close_progress" => &["returned", "refused", "unavailable"],
        "close_waited" => &["completed", "invalidated", "unavailable", "expired"],
        "close_sample" => &["observed", "unavailable", "refused"],
        "close_finished" | "close_aborted" => &["released", "refused"],
        _ => return Err(bad(stage)),
    };
    if !statuses.contains(&reply.status.as_str()) {
        return Err(bad(stage));
    }
    validate(&reply, target.session_id(), receipt)?;
    Ok(reply)
}
fn validate(reply: &Reply, session: &SessionId, receipt: u64) -> Result<(), RoutingFailure> {
    let stage = Stage::ReadEditor;
    fn reason(value: Option<&str>) -> bool {
        value.is_none_or(|s| {
            s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
    }
    fn selection(value: &Selection) -> bool {
        ["target", "other", "no_source_editor", "unknown"].contains(&value.before.as_str())
            && ["target", "other", "no_source_editor", "unknown"].contains(&value.after.as_str())
            && ["none", "native_fallback", "unknown"].contains(&value.request_effect.as_str())
    }
    fn protection(value: &Protection) -> bool {
        ["not_applicable", "preserved", "unavailable", "invalidated"]
            .contains(&value.status.as_str())
            && [
                "not_applicable",
                "pending",
                "completed",
                "unavailable",
                "invalidated",
            ]
            .contains(&value.revalidation.as_str())
            && value.required_count.is_none_or(|n| n <= 7)
            && value
                .completed_count
                .is_none_or(|n| n <= 7 && value.required_count.is_some_and(|r| n <= r))
            && reason(value.reason.as_deref())
    }
    fn continuation(value: &Continuation) -> bool {
        [
            "not_applicable",
            "pending",
            "completed",
            "unavailable",
            "invalidated",
        ]
        .contains(&value.state.as_str())
            && value.required_editor_ids.len() <= 7
            && value.completed_editor_ids.len() <= 7
            && value.collections.len() <= 7
            && value
                .required_editor_ids
                .windows(2)
                .all(|w| w[0].parse::<u64>().ok() < w[1].parse::<u64>().ok())
            && value
                .completed_editor_ids
                .windows(2)
                .all(|w| w[0].parse::<u64>().ok() < w[1].parse::<u64>().ok())
            && value
                .completed_editor_ids
                .iter()
                .all(|id| value.required_editor_ids.contains(id))
            && (value.state != "completed"
                || value.required_editor_ids == value.completed_editor_ids)
            && (value.state != "not_applicable" || value.required_editor_ids.is_empty())
            && reason(value.reason.as_deref())
    }
    if !reason(reply.reason.as_deref())
        || reply.selection.as_ref().is_some_and(|v| !selection(v))
        || reply.protection.as_ref().is_some_and(|v| !protection(v))
        || reply
            .continuation
            .as_ref()
            .is_some_and(|v| !continuation(v))
    {
        return Err(bad(stage));
    }
    let stamp = |v: &Value| -> Result<CollectionStamp, RoutingFailure> {
        let value = StampIn::deserialize(v).map_err(|_| bad(stage))?;
        if value.received_elapsed_us != 0 {
            return Err(bad(stage));
        }
        value.domain(session, receipt, stage, true)
    };
    if let Some(v) = &reply.resource {
        if ![
            "retained",
            "unloaded",
            "unavailable",
            "invalidated",
            "not_collected",
        ]
        .contains(&v.state.as_str())
            || !reason(v.reason.as_deref())
        {
            return Err(bad(stage));
        }
        if let Some(collection) = &v.collection {
            stamp(collection)?;
        }
    }
    if let Some(n) = &reply.native {
        if !["retained", "disposed", "unavailable", "not_applicable"]
            .contains(&n.target_buffer.as_str())
            || !reason(n.reason.as_deref())
            || n.selection.as_ref().is_some_and(|v| !selection(v))
            || n.protection.as_ref().is_some_and(|v| !protection(v))
            || n.continuation.as_ref().is_some_and(|v| !continuation(v))
        {
            return Err(bad(stage));
        }
        let entry = n.entry_collection.as_ref().map(&stamp).transpose()?;
        let returned = n.return_collection.as_ref().map(&stamp).transpose()?;
        if n.entered != entry.is_some()
            || returned.as_ref().is_some_and(|returned| {
                entry
                    .as_ref()
                    .is_none_or(|entry| returned.started_tick_us() < entry.finished_tick_us())
            })
            || entry
                .as_ref()
                .is_some_and(|entry| entry.finished_tick_us() > reply.collection.finished_tick_us())
            || returned.as_ref().is_some_and(|returned| {
                returned.finished_tick_us() > reply.collection.finished_tick_us()
            })
            || (n.close_error.is_some() && returned.is_none())
            || (!n.entered
                && (n.old_document_removed == Some(true)
                    || n.target_buffer == "disposed"
                    || n.selection
                        .as_ref()
                        .is_some_and(|s| s.request_effect == "native_fallback")))
            || (n.terminal_discard
                && (n.entered
                    || returned.is_some()
                    || n.close_error.is_some()
                    || n.old_document_removed == Some(true)
                    || n.target_buffer == "disposed"
                    || n.selection
                        .as_ref()
                        .is_some_and(|s| s.request_effect == "native_fallback")))
        {
            return Err(bad(stage));
        }
        if let Some(c) = &n.continuation {
            validate_ledger(c, session, receipt, entry.as_ref(), &reply.collection)?;
        }
    }
    if let Some(c) = &reply.continuation {
        let entry = reply
            .native
            .as_ref()
            .and_then(|n| n.entry_collection.as_ref())
            .map(&stamp)
            .transpose()?;
        validate_ledger(c, session, receipt, entry.as_ref(), &reply.collection)?;
    }
    if let Some(native) = &reply.native {
        if reply
            .continuation
            .as_ref()
            .is_some_and(|v| native.continuation.as_ref() != Some(v))
            || reply
                .protection
                .as_ref()
                .is_some_and(|v| native.protection.as_ref() != Some(v))
            || reply.selection.as_ref().is_some_and(|v| {
                if reply.purpose.as_deref() == Some("survivor") {
                    native.selection.as_ref().is_none_or(|old| {
                        old.before != v.before || old.request_effect != v.request_effect
                    })
                } else {
                    native.selection.as_ref() != Some(v)
                }
            })
            || reply.discard.is_some_and(|v| v != native.terminal_discard)
        {
            return Err(bad(stage));
        }
    }
    if reply.validation.as_ref().is_some_and(|v| v.len() > 7) {
        return Err(bad(stage));
    }
    if let Some(purpose) = &reply.purpose {
        let allowed: &[&str] = if reply.recheck.is_some() {
            &["pre_close", "recognition", "post_close"]
        } else {
            &["recognition", "post_close", "survivor"]
        };
        if !allowed.contains(&purpose.as_str()) {
            return Err(bad(stage));
        }
    }
    Ok(())
}
fn validate_ledger(
    ledger: &Continuation,
    session: &SessionId,
    receipt: u64,
    entry: Option<&CollectionStamp>,
    envelope: &CollectionStamp,
) -> Result<(), RoutingFailure> {
    let stage = Stage::ReadEditor;
    if ledger.collections.len() != ledger.required_editor_ids.len() {
        return Err(bad(stage));
    }
    for id in ledger
        .required_editor_ids
        .iter()
        .chain(&ledger.completed_editor_ids)
    {
        if decimal(id.clone(), stage)?.as_str() == "0" {
            return Err(bad(stage));
        }
    }
    for (id, row) in ledger.required_editor_ids.iter().zip(&ledger.collections) {
        if &row.editor_id != id {
            return Err(bad(stage));
        }
        let visit = StampIn::deserialize(row.visit.as_ref().ok_or_else(|| bad(stage))?)
            .map_err(|_| bad(stage))?;
        if visit.received_elapsed_us != 0 {
            return Err(bad(stage));
        }
        let visit = visit.domain(session, receipt, stage, true)?;
        if entry.is_none_or(|entry| visit.started_tick_us() < entry.finished_tick_us())
            || visit.finished_tick_us() > envelope.finished_tick_us()
        {
            return Err(bad(stage));
        }
        if ledger.completed_editor_ids.contains(id) {
            let completion =
                StampIn::deserialize(row.completion.as_ref().ok_or_else(|| bad(stage))?)
                    .map_err(|_| bad(stage))?;
            if completion.received_elapsed_us != 0 {
                return Err(bad(stage));
            }
            let completion = completion.domain(session, receipt, stage, true)?;
            if completion.started_tick_us() < visit.finished_tick_us()
                || completion.finished_tick_us() > envelope.finished_tick_us()
            {
                return Err(bad(stage));
            }
        } else if row.completion.is_some() {
            return Err(bad(stage));
        }
    }
    Ok(())
}
