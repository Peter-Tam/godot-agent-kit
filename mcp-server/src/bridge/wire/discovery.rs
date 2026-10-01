//! Strict path-free editor controls and bounded private worker evidence frames.
use crate::script_discovery::events::{self, ContextReason, ContextStatus, ScopeFacts};
use crate::script_discovery::{
    self, Context, DiscoveryRequest, DiscoveryTarget, Event, Reason, Stamp,
};
use serde::{Deserialize, Serialize};
const CONTROL_LIMIT: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Control {
    Begin,
    Recheck,
    Finish,
    Abort,
}
impl Control {
    fn opcode(self) -> &'static str {
        match self {
            Self::Begin => "discover_begin",
            Self::Recheck => "discover_recheck",
            Self::Finish => "discover_finish",
            Self::Abort => "discover_abort",
        }
    }
}
fn checked_json<'a, T: Deserialize<'a>>(bytes: &'a [u8], limit: usize) -> Result<T, Reason> {
    if bytes.is_empty() || bytes.len() > limit {
        return Err(Reason::ProtocolError);
    }
    super::json_depth(bytes, crate::observation::Stage::ReadEditor)
        .map_err(|_| Reason::ProtocolError)?;
    serde_json::from_slice(bytes).map_err(|_| Reason::ProtocolError)
}
pub(crate) fn encode_event(event: &Event) -> Result<Vec<u8>, Reason> {
    let bytes = serde_json::to_vec(event).map_err(|_| Reason::ProtocolError)?;
    if bytes.len() > script_discovery::FRAME_LIMIT {
        return Err(Reason::ProtocolError);
    }
    Ok(bytes)
}
pub(crate) fn decode_event(bytes: &[u8]) -> Result<Event, Reason> {
    checked_json(bytes, script_discovery::FRAME_LIMIT)
}
pub(crate) fn encode_control(
    control: Control,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    advertised: &str,
    remaining_ms: u64,
) -> Result<Vec<u8>, Reason> {
    if !target.valid(request)
        || crate::observation::ProjectRoot::new(advertised.to_owned()).is_err()
        || control == Control::Begin && !(1..=4500).contains(&remaining_ms)
    {
        return Err(Reason::ProtocolError);
    }
    let bytes = if control == Control::Begin {
        serde_json::to_vec(&(
            4,
            control.opcode(),
            request.request_id().as_str(),
            &target.session_id,
            advertised,
            remaining_ms,
        ))
    } else {
        serde_json::to_vec(&(
            4,
            control.opcode(),
            request.request_id().as_str(),
            &target.session_id,
            advertised,
        ))
    }
    .map_err(|_| Reason::ProtocolError)?;
    if bytes.len() > CONTROL_LIMIT {
        return Err(Reason::ProtocolError);
    }
    Ok(bytes)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    collection: Stamp,
    status: ContextStatus,
    #[serde(deserialize_with = "events::required_option")]
    reason: Option<ContextReason>,
    #[serde(deserialize_with = "events::required_option")]
    context: Option<ScopeFacts>,
    #[serde(deserialize_with = "events::required_option")]
    expiry_tick_us: Option<String>,
}
fn bound(
    (v, kind, id, session, root): (u32, &str, &str, &str, &str),
    expected: &str,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    advertised: &str,
) -> bool {
    v == 4
        && kind == expected
        && id == request.request_id().as_str()
        && session == target.session_id
        && root == advertised
        && target.valid(request)
}
pub(crate) fn decode_context(
    bytes: &[u8],
    kind: &str,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    advertised: &str,
    receipt: u64,
) -> Result<Context, Reason> {
    if !["discover_state", "discover_rechecked"].contains(&kind)
        || receipt >= script_discovery::CUTOFF_US
    {
        return Err(Reason::ProtocolError);
    }
    let mut v: ContextIn = checked_json(bytes, CONTROL_LIMIT)?;
    if !bound(
        (v.v, &v.kind, &v.request_id, &v.session_id, &v.project_root),
        kind,
        request,
        target,
        advertised,
    ) || !v
        .collection
        .valid(&format!("editor:{}", target.session_id), receipt)
    {
        return Err(Reason::ProtocolError);
    }
    v.collection.received_elapsed_us = receipt;
    let context = Context {
        collection: v.collection,
        status: v.status,
        reason: v.reason,
        context: v.context,
        expiry_tick_us: v.expiry_tick_us,
    };
    if !context.valid(&target.session_id, receipt) {
        return Err(Reason::ProtocolError);
    }
    Ok(context)
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ack {
    pub v: u32,
    pub kind: String,
    pub request_id: String,
    pub session_id: String,
    pub project_root: String,
    pub collection: Stamp,
    pub terminal_discard: bool,
}
pub(crate) fn decode_ack(
    bytes: &[u8],
    control: Control,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    advertised: &str,
    receipt: u64,
) -> Result<Ack, Reason> {
    let kind = match control {
        Control::Finish => "discover_finished",
        Control::Abort => "discover_aborted",
        _ => return Err(Reason::ProtocolError),
    };
    let mut ack: Ack = checked_json(bytes, CONTROL_LIMIT)?;
    if !bound(
        (
            ack.v,
            &ack.kind,
            &ack.request_id,
            &ack.session_id,
            &ack.project_root,
        ),
        kind,
        request,
        target,
        advertised,
    ) || ack.terminal_discard != (control == Control::Abort)
        || !ack
            .collection
            .valid(&format!("editor:{}", target.session_id), receipt)
        || receipt >= script_discovery::CUTOFF_US
    {
        return Err(Reason::ProtocolError);
    }
    ack.collection.received_elapsed_us = receipt;
    Ok(ack)
}
/// Distinguish negative lifetime/root evidence from a merely malformed or copied
/// payload. Used only after strict decoding fails; never admits positive facts.
pub(crate) fn binding_changed(
    bytes: &[u8],
    expected_kind: &str,
    request: &DiscoveryRequest,
    target: &DiscoveryTarget,
    advertised: &str,
) -> bool {
    let fields = match expected_kind {
        "discover_state" | "discover_rechecked" => checked_json::<ContextIn>(bytes, CONTROL_LIMIT)
            .ok()
            .map(|v| (v.v, v.kind, v.request_id, v.session_id, v.project_root)),
        "discover_finished" | "discover_aborted" => checked_json::<Ack>(bytes, CONTROL_LIMIT)
            .ok()
            .map(|v| (v.v, v.kind, v.request_id, v.session_id, v.project_root)),
        _ => None,
    };
    fields.is_some_and(|(v, kind, id, session, root)| {
        v == 4
            && kind == expected_kind
            && id == request.request_id().as_str()
            && target.valid(request)
            && crate::observation::SessionId::new(session.clone()).is_ok()
            && crate::observation::ProjectRoot::new(root.clone()).is_ok()
            && (session != target.session_id || root != advertised)
    })
}
#[cfg(test)]
mod tests;
