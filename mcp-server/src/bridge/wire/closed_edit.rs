//! Private v6 closed native exchange. No public method selector or raw channel.
use super::*;
use crate::runner::stock_validation::{ClosedContextProjection, WarningSettings};
use crate::script_closed_edit::{ExpectedState, NativeReceipt, State};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn nullable<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Validation {
    pub executable: PathBuf,
    pub warnings: WarningSettings,
    pub global_classes: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Context {
    pub projection: ClosedContextProjection,
    pub source: String,
    pub sha256: String,
    pub validation: Validation,
}
impl Context {
    pub(crate) fn checked(&self, request: &ObservationRequest, target: &ResolvedTarget) -> bool {
        self.projection.closed_context_matches(
            &self.source,
            &self.sha256,
            request,
            target,
            &self.validation.warnings,
            &self.validation.global_classes,
        )
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplyIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    status: String,
    #[serde(deserialize_with = "nullable")]
    reason: Option<String>,
    #[serde(deserialize_with = "nullable")]
    expiry_tick_us: Option<String>,
    #[serde(deserialize_with = "nullable")]
    state: Option<State>,
    #[serde(deserialize_with = "nullable")]
    native: Option<NativeReceipt>,
    #[serde(deserialize_with = "nullable")]
    context: Option<Context>,
}
pub(crate) struct Reply {
    pub status: String,
    pub reason: Option<String>,
    pub state: Option<State>,
    pub native: Option<NativeReceipt>,
    pub context: Option<Context>,
    pub collection: CollectionStamp,
}
fn reason(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'_' || b.is_ascii_digit())
}
fn checked(
    bytes: &[u8],
    request: &ObservationRequest,
    selected: &SelectedSession,
    kind: &str,
    started: Instant,
) -> Result<Reply, RoutingFailure> {
    checked_target(
        bytes,
        request,
        selected.target(),
        (
            &selected.advertised_project_root,
            selected.native_api_revision(),
            selected.native_build_id(),
        ),
        kind,
        started,
    )
}
fn checked_target(
    bytes: &[u8],
    request: &ObservationRequest,
    t: &ResolvedTarget,
    binding: (&str, u32, &str),
    kind: &str,
    started: Instant,
) -> Result<Reply, RoutingFailure> {
    let (advertised_project_root, native_api_revision, native_build_id) = binding;
    let stage = Stage::ReadEditor;
    json_depth(bytes, stage)?;
    let r: ReplyIn = decode(bytes, stage)?;
    if r.v != 6
        || r.kind != kind
        || r.request_id != request.request_id().as_str()
        || r.session_id != t.session_id().as_str()
        || r.project_root != advertised_project_root
        || r.script_path != t.script_path().as_str()
        || !matches!(
            r.status.as_str(),
            "inspected"
                | "prepared"
                | "applied"
                | "verified"
                | "rechecked"
                | "finished"
                | "aborted"
                | "refused"
                | "partial"
                | "unavailable"
        )
        || r.reason.as_deref().is_some_and(|s| !reason(s))
    {
        return Err(bad(stage));
    }
    let expected_status = match kind {
        "closed_state" => "inspected",
        "closed_prepared" => "prepared",
        "closed_applied" => "applied",
        "closed_verified" => "verified",
        "closed_rechecked" => "rechecked",
        "closed_finished" => "finished",
        "closed_aborted" => "aborted",
        _ => return Err(bad(stage)),
    };
    if r.status != expected_status
        && !matches!(r.status.as_str(), "refused" | "partial" | "unavailable")
    {
        return Err(bad(stage));
    }
    let elapsed = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let collection = r.collection.domain(t.session_id(), elapsed, stage, true)?;
    if r.expiry_tick_us
        .as_deref()
        .is_some_and(|s| !crate::script_closed_edit::model_unsigned(s))
    {
        return Err(bad(stage));
    }
    if let Some(s) = &r.state {
        if !s.file_revision.valid()
            || s.project_device != t.project_file_id().device().as_str()
            || s.project_inode != t.project_file_id().inode().as_str()
        {
            return Err(bad(stage));
        }
        if !matches!(s.lifecycle.as_str(), "closed" | "open" | "unknown")
            || !crate::script_closed_edit::model_unsigned(&s.close_epoch)
            || !s.valid(t.script_path().as_str())
        {
            return Err(bad(stage));
        }
    }
    if let Some(n) = &r.native {
        if n.request_id != request.request_id().as_str()
            || n.session_id != t.session_id().as_str()
            || n.script_path != t.script_path().as_str()
            || n.native_api_revision != native_api_revision
            || n.native_build_id != native_build_id
            || !crate::script_closed_edit::model_unsigned(&n.written_bytes)
            || n.reason.as_deref().is_some_and(|s| !reason(s))
            || !matches!(
                n.phase.as_str(),
                "prepared"
                    | "resource_applied"
                    | "content_persisted"
                    | "mtime_restored"
                    | "terminal"
            )
            || n.written_bytes
                .parse::<usize>()
                .is_ok_and(|n| n > SOURCE_LIMIT_BYTES)
            || (n.resource_changed && !n.resource_entered)
            || (n.written_bytes != "0" && !n.disk_entered)
            || (n.mtime_restored && !n.futimens_called)
        {
            return Err(bad(stage));
        }
    }
    let context = r.context.filter(|c| {
        c.checked(request, t)
            && r.state.as_ref().is_some_and(|s| {
                c.projection
                    .closed_resource_matches(s.resource.instance_id.as_deref(), s.resource.edited)
            })
    });
    Ok(Reply {
        status: r.status,
        reason: r.reason,
        state: r.state,
        native: r.native,
        context,
        collection,
    })
}
pub(crate) enum Operation<'a> {
    Inspect(u64),
    Prepare(&'a ExpectedState, &'a str, u64),
    Apply(&'a str, &'a str),
    Verify(&'a str),
    Recheck(&'a str),
    Finish,
    Abort,
}
pub(crate) fn exchange(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    op: Operation<'_>,
    started: Instant,
    deadline: Instant,
) -> Result<Reply, RoutingFailure> {
    let (name, kind) = match op {
        Operation::Inspect(_) => ("closed_inspect", "closed_state"),
        Operation::Prepare(..) => ("closed_prepare", "closed_prepared"),
        Operation::Apply(..) => ("closed_apply", "closed_applied"),
        Operation::Verify(_) => ("closed_verify", "closed_verified"),
        Operation::Recheck(_) => ("closed_recheck", "closed_rechecked"),
        Operation::Finish => ("closed_finish", "closed_finished"),
        Operation::Abort => ("closed_abort", "closed_aborted"),
    };
    let t = selected.target();
    let id = request.request_id().as_str();
    let session = t.session_id().as_str();
    let root = selected.advertised_project_root.as_str();
    let path = t.script_path().as_str();
    let bytes = match op {
        Operation::Inspect(n) => serde_json::to_vec(&(6, name, id, session, root, path, n)),
        Operation::Prepare(s, source, n) => {
            serde_json::to_vec(&(6, name, id, session, root, path, s, source, n))
        }
        Operation::Apply(a, b) => serde_json::to_vec(&(6, name, id, session, root, path, a, b)),
        Operation::Verify(p) | Operation::Recheck(p) => {
            serde_json::to_vec(&(6, name, id, session, root, path, p))
        }
        Operation::Finish | Operation::Abort => {
            serde_json::to_vec(&(6, name, id, session, root, path))
        }
    }
    .map_err(|_| bad(Stage::ReadEditor))?;
    if bytes.len() > 2 * SOURCE_LIMIT_BYTES + 16384 {
        return Err(bad(Stage::ReadEditor));
    }
    write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        Stage::ReadEditor,
    )?;
    write_deadline(&mut selected.socket, &bytes, deadline, Stage::ReadEditor)?;
    let bytes = receive_editor(&mut selected.socket, deadline, Stage::ReadEditor)?;
    checked(&bytes, request, selected, kind, started)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_codec_rejects_wrong_binding_operation_state_and_effects() {
        let request = ObservationRequest::new(
            RequestId::new("r").unwrap(),
            ProjectRoot::new("/p").unwrap(),
            None,
            ResourcePath::new("res://s.gd").unwrap(),
        );
        let session = SessionId::new("a".repeat(32)).unwrap();
        let target = ResolvedTarget::for_request(
            &request,
            request.project_root().clone(),
            FileIdentity::new(
                DecimalCounter::new("1").unwrap(),
                DecimalCounter::new("2").unwrap(),
            ),
            session.clone(),
            EngineVersion::new("4.7.2", "hash").unwrap(),
        )
        .unwrap();
        let base = serde_json::json!({"v":6,"kind":"closed_state","request_id":"r","session_id":session.as_str(),"project_root":"/p","script_path":"res://s.gd","collection":{"clock_id":format!("editor:{}",session.as_str()),"started_tick_us":"1","finished_tick_us":"2","received_elapsed_us":0},"status":"unavailable","reason":"cache_unavailable","expiry_tick_us":null,"state":null,"native":null,"context":null});
        let accepts = |v: &serde_json::Value| {
            checked_target(
                &serde_json::to_vec(v).unwrap(),
                &request,
                &target,
                ("/p", 4, "build"),
                "closed_state",
                Instant::now() - Duration::from_millis(1),
            )
            .is_ok()
        };
        let mut informative = base.clone();
        informative["status"] = "inspected".into();
        informative["reason"] = serde_json::Value::Null;
        informative["state"] = serde_json::json!({"project_device":"1","project_inode":"2","file_revision":{"device":"1","inode":"3","utf8_bytes":"0","sha256":crate::project_fs_validation::hex_sha256(b""),"mtime":{"seconds":"0","nanoseconds":0},"ctime":{"seconds":"0","nanoseconds":0}},"close_epoch":"1","lifecycle":"closed","resource":{"state":"present","instance_id":"4","path":"res://s.gd","source":"different R","edited":true,"profile_sha256":"a".repeat(64)}});
        assert!(accepts(&informative));
        informative["state"]["resource"]["edited"] = false.into();
        assert!(accepts(&informative));
        informative["state"]["resource"]["instance_id"] = "04".into();
        assert!(!accepts(&informative));
        informative["state"]["resource"]["instance_id"] = "4".into();
        informative["state"]["file_revision"]["mtime"]["nanoseconds"] = 1_000_000_000.into();
        assert!(!accepts(&informative));
        informative["state"]["file_revision"]["mtime"]["nanoseconds"] = 0.into();
        informative["state"]["close_epoch"] = "01".into();
        assert!(!accepts(&informative));
        informative["state"]["close_epoch"] = "1".into();
        informative["state"]["lifecycle"] = "open".into();
        assert!(accepts(&informative));
        informative["state"]["resource"]["source"] = "x".repeat(SOURCE_LIMIT_BYTES + 1).into();
        informative["status"] = "partial".into();
        assert!(!accepts(&informative));
        assert!(accepts(&base));
        for (key, value) in [
            ("request_id", "other"),
            ("session_id", "b"),
            ("script_path", "res://other.gd"),
            ("kind", "closed_prepared"),
            ("status", "prepared"),
            ("reason", "Unsafe"),
        ] {
            let mut bad = base.clone();
            bad[key] = value.into();
            assert!(!accepts(&bad), "{key}");
        }
        let mut bad = base.clone();
        bad["native"] = serde_json::json!({"request_id":"r","session_id":session.as_str(),"script_path":"res://s.gd","native_build_id":"build","native_api_revision":4,"phase":"terminal","resource_entered":false,"resource_changed":true,"disk_entered":false,"written_bytes":"0","truncate_done":false,"fsync_done":false,"pread_done":false,"futimens_called":false,"mtime_restored":false,"terminal_discard":true,"reason":null});
        assert!(!accepts(&bad));
        bad["native"]["resource_changed"] = false.into();
        assert!(accepts(&bad));
        bad["native"]["written_bytes"] = "1".into();
        assert!(!accepts(&bad));
        bad["native"]["written_bytes"] = "0".into();
        bad["native"]["mtime_restored"] = true.into();
        assert!(!accepts(&bad));
    }
    #[test]
    fn reply_rejects_omitted_nulls_and_unknown_fields() {
        let base = r#"{"v":6,"kind":"closed_state","request_id":"r","session_id":"s","project_root":"/p","script_path":"res://s.gd","collection":{"clock_id":"editor:s","started_tick_us":"1","finished_tick_us":"2","received_elapsed_us":0},"status":"unavailable","reason":null,"expiry_tick_us":null,"state":null,"native":null,"context":null}"#;
        assert!(serde_json::from_str::<ReplyIn>(base).is_ok());
        assert!(serde_json::from_str::<ReplyIn>(&base.replace(",\"native\":null", "")).is_err());
        assert!(
            serde_json::from_str::<ReplyIn>(&base.replace("\"v\":6", "\"v\":6,\"v\":6")).is_err()
        );
        assert!(
            serde_json::from_str::<ReplyIn>(&base.replace("\"v\":6", "\"v\":6,\"extra\":0"))
                .is_err()
        );
    }
}
