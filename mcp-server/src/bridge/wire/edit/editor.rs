//! Strict selected-session edit exchange; transport and DTO facts stay out of policy.
use super::*;
use crate::runner::stock_validation;
use crate::script_edit::{Reason, SavedStateEvidence};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

fn unique_levels<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, u8>, D::Error> {
    struct Unique;
    impl<'de> Visitor<'de> for Unique {
        type Value = BTreeMap<String, u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded unique warning settings")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
            let mut output = BTreeMap::new();
            while let Some((key, value)) = input.next_entry::<String, u8>()? {
                if output.insert(key, value).is_some() || output.len() > 128 {
                    return Err(de::Error::custom("invalid warning settings"));
                }
            }
            Ok(output)
        }
    }
    d.deserialize_map(Unique)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedIn {
    document: IdentityIn,
    current_version: String,
    saved_version: String,
    resource_edited: bool,
    save_profile: String,
    original_preserved: bool,
    desired_preserved: bool,
    collection: StampIn,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WarningIn {
    enable: bool,
    #[serde(deserialize_with = "unique_levels")]
    levels: BTreeMap<String, u8>,
    #[serde(deserialize_with = "unique_levels")]
    directory_rules: BTreeMap<String, u8>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextIn {
    executable_path: String,
    warning_profile: WarningIn,
    global_classes: Bounded<String, 256>,
    collection: StampIn,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeIn {
    seconds: String,
    nanoseconds: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactsIn {
    expected_sha256: String,
    desired_sha256: String,
    project_device: String,
    project_inode: String,
    target_device: String,
    target_inode: String,
    prepared_current: String,
    prepared_saved: String,
    frozen_current: Option<String>,
    buffer_changed: bool,
    resource_changed: bool,
    write_started: bool,
    write_calls: String,
    written_bytes: String,
    write_errno: String,
    truncate_done: bool,
    truncate_errno: String,
    fsync_done: bool,
    fsync_errno: String,
    pread_done: bool,
    pread_errno: String,
    futimens_called: bool,
    futimens_errno: String,
    mtime_restored: bool,
    edited_clear_attempted: bool,
    edited_cleared: bool,
    tag_attempted: bool,
    tagged: bool,
    t0: TimeIn,
    before_restore: Option<TimeIn>,
    after_restore: Option<TimeIn>,
    injected_fault: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIn {
    status: String,
    reason: String,
    stage: String,
    facts: FactsIn,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeEventIn {
    stage: String,
    native: NativeIn,
    collection: StampIn,
    before_sample: Option<SampleIn>,
    after_sample: Option<SampleIn>,
    before_saved_state: Option<SavedIn>,
    after_saved_state: Option<SavedIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparedIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    status: String,
    reason: String,
    intent: Option<String>,
    native: Option<NativeIn>,
    sample: Option<SampleIn>,
    saved_state: Option<SavedIn>,
    context: Option<ContextIn>,
    expiry_tick_us: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AppliedIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    status: String,
    reason: String,
    native: Option<NativeIn>,
    events: Bounded<NativeEventIn, 1>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    event: NativeEventIn,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleReplyIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    purpose: String,
    sample: Option<SampleIn>,
    saved_state: Option<SavedIn>,
    context: Option<ContextIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecheckedIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    purpose: String,
    recheck: RecheckIn,
    saved_state: Option<SavedIn>,
    context: Option<ContextIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    status: String,
    reason: String,
    terminal_before_boundary: bool,
    native: Option<NativeIn>,
}

pub struct EditContext {
    pub executable: PathBuf,
    pub warnings: stock_validation::WarningSettings,
    pub global_classes: Vec<String>,
    pub collection: CollectionStamp,
}
pub struct NativeFacts {
    pub stage: &'static str,
    pub status: &'static str,
    pub reason: Reason,
    pub expected_hash: [u8; 32],
    pub desired_hash: [u8; 32],
    pub project: FileIdentity,
    pub file: FileIdentity,
    pub prepared_current: DecimalCounter,
    pub prepared_saved: DecimalCounter,
    pub frozen_current: Option<DecimalCounter>,
    pub buffer_changed: bool,
    pub resource_changed: bool,
    pub write_started: bool,
    pub write_calls: usize,
    pub written_bytes: usize,
    pub write_errno: i32,
    pub truncated: bool,
    pub truncate_errno: i32,
    pub flushed: bool,
    pub fsync_errno: i32,
    pub readback: bool,
    pub pread_errno: i32,
    pub restore_attempted: bool,
    pub restore_errno: i32,
    pub restored: bool,
    pub edited_attempted: bool,
    pub edited_cleared: bool,
    pub tag_attempted: bool,
    pub tagged: bool,
    pub t0: DecimalCounter,
    pub after_restore: Option<DecimalCounter>,
}
pub struct EditSample {
    pub collection: CollectionStamp,
    pub sample: EditorSample,
    pub saved: Option<SavedStateEvidence>,
    pub context: Option<EditContext>,
}
pub struct Prepared {
    pub status: &'static str,
    pub reason: Reason,
    pub intent_changed: Option<bool>,
    pub native: Option<NativeFacts>,
    pub sample: Option<EditSample>,
}
pub struct AppliedEvent {
    pub collection: CollectionStamp,
    pub stage: &'static str,
    pub native: NativeFacts,
    pub before: Option<EditSample>,
    pub after: Option<EditSample>,
}
pub struct Applied {
    pub collection: CollectionStamp,
    pub status: &'static str,
    pub reason: Reason,
    pub native: Option<NativeFacts>,
    pub terminal_event: Option<AppliedEvent>,
}
pub struct Rechecked {
    pub recheck: Recheck,
    pub saved: Option<SavedStateEvidence>,
    pub context: Option<EditContext>,
}
pub struct Terminal {
    pub collection: CollectionStamp,
    pub terminal_before_boundary: bool,
    pub reason: Reason,
}

fn bound(
    (v, kind, id, sid, root, path): (u32, &str, &str, &str, &str, &str),
    expected: &str,
    stamp: StampIn,
    selected: &SelectedSession,
    request: &ObservationRequest,
    receipt: u64,
    stage: Stage,
) -> Result<CollectionStamp, RoutingFailure> {
    if v != 3
        || kind != expected
        || id != request.request_id().as_str()
        || sid != selected.target().session_id().as_str()
        || root != selected.advertised_project_root
        || path != request.script_path().as_str()
        || !selected.matches_request(request)
    {
        return Err(bad(stage));
    }
    stamp.domain(selected.target().session_id(), receipt, stage, true)
}
fn number(value: String, stage: Stage) -> Result<usize, RoutingFailure> {
    let canonical = decimal(value, stage)?;
    canonical.as_str().parse().map_err(|_| bad(stage))
}
fn errno(value: String, stage: Stage) -> Result<i32, RoutingFailure> {
    let value = decimal(value, stage)?;
    i32::try_from(value.as_str().parse::<u32>().map_err(|_| bad(stage))?).map_err(|_| bad(stage))
}
fn hex(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut output = [0; 32];
    for (i, chunk) in s.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        fn nibble(c: u8) -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                _ => None,
            }
        }
        output[i] = nibble(chunk[0])? << 4 | nibble(chunk[1])?;
    }
    Some(output)
}
fn encode_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 15) as usize] as char);
    }
    out
}
fn mtime(value: TimeIn, stage: Stage) -> Result<DecimalCounter, RoutingFailure> {
    let seconds = number(value.seconds, stage)?;
    let nanoseconds = number(value.nanoseconds, stage)?;
    if nanoseconds >= 1_000_000_000 {
        return Err(bad(stage));
    }
    DecimalCounter::new(format!(
        "{}",
        (seconds as u128) * 1_000_000_000 + nanoseconds as u128
    ))
    .map_err(|_| bad(stage))
}
fn reason(value: &str) -> Reason {
    match value {
        "slot_busy" | "busy" => Reason::Busy,
        "dirty" | "dirty_conflict" => Reason::DirtyConflict,
        "closed_target" => Reason::ClosedTarget,
        "unavailable_observation" => Reason::UnavailableObservation,
        "identity_changed" => Reason::IdentityChanged,
        "divergence" => Reason::Divergence,
        "stale"
        | "revision_mismatch"
        | "source_changed"
        | "buffer_version_changed"
        | "version_changed"
        | "edited_state_changed"
        | "disk_content_changed"
        | "mtime_changed"
        | "intermediate_source_or_version_changed" => Reason::RevisionMismatch,
        "document_association_changed"
        | "namespace_or_descriptor_changed"
        | "target_changed"
        | "invalid_binding"
        | "disk_or_namespace_changed"
        | "document_closed_during_edit"
        | "post_tag_document_changed" => Reason::IdentityChanged,
        "session_or_expiry_changed" | "invalid_expiry" | "cancelled_or_expired_during_edit" => {
            Reason::Deadline
        }
        "cancelled_during_edit" => Reason::Cancellation,
        "denied_access" => Reason::DeniedAccess,
        "evidence_limit" => Reason::EvidenceLimit,
        "unsupported_effect" | "unsafe_editor_effects" => Reason::UnsupportedEffect,
        "unsupported_representation" | "invalid_buffer" => Reason::UnsupportedRepresentation,
        "save_profile_changed" | "save_would_reformat" => Reason::SaveWouldReformat,
        "write_failed"
        | "truncate_failed"
        | "fsync_failed"
        | "pread_failed"
        | "futimens_failed"
        | "pread_or_namespace_changed"
        | "metadata_preflight_failed"
        | "mtime_restore_failed"
        | "mtime_readback_failed"
        | "incomplete_content_receipt" => Reason::PersistenceFailure,
        "edited_readback_failed" | "post_tag_unverified" => Reason::PartialFinalization,
        "version_unavailable"
        | "source_unavailable"
        | "edited_state_unavailable"
        | "buffer_readback_failed"
        | "resource_readback_failed" => Reason::UnavailableObservation,
        "validation_unavailable" => Reason::ValidationUnavailable,
        "parse_error" => Reason::ParseError,
        "" | "ok" | "ready" | "prepared" | "complete" => Reason::Complete,
        _ => Reason::ProtocolFailure,
    }
}
fn phase(s: &str) -> Option<&'static str> {
    match s {
        "prepared" => Some("prepared"),
        "buffer_applied" => Some("buffer_applied"),
        "resource_applied" => Some("resource_applied"),
        "content_persisted" => Some("content_persisted"),
        "mtime_restored" => Some("mtime_restored"),
        "edited_cleared" => Some("edited_cleared"),
        "saved_tagged" => Some("saved_tagged"),
        _ => None,
    }
}
fn native(value: NativeIn, stage: Stage) -> Result<NativeFacts, RoutingFailure> {
    let f = value.facts;
    if f.injected_fault.as_ref().is_some_and(|name| {
        name.len() > 64 || name.bytes().any(|c| !c.is_ascii_lowercase() && c != b'_')
    }) {
        return Err(bad(stage));
    }
    let status = match value.status.as_str() {
        "prepared" => "prepared",
        "ready" => "ready",
        "complete" => "complete",
        "refused" => "refused",
        "partial" => "partial",
        "busy" => "busy",
        _ => return Err(bad(stage)),
    };
    let phase = phase(&value.stage).ok_or_else(|| bad(stage))?;
    let project = FileIdentity::new(
        decimal(f.project_device, stage)?,
        decimal(f.project_inode, stage)?,
    );
    let file = FileIdentity::new(
        decimal(f.target_device, stage)?,
        decimal(f.target_inode, stage)?,
    );
    let expected_hash = hex(&f.expected_sha256).ok_or_else(|| bad(stage))?;
    let desired_hash = hex(&f.desired_sha256).ok_or_else(|| bad(stage))?;
    let prepared_current = decimal(f.prepared_current, stage)?;
    let prepared_saved = decimal(f.prepared_saved, stage)?;
    let frozen_current = f.frozen_current.map(|s| decimal(s, stage)).transpose()?;
    let after_restore = f.after_restore.map(|v| mtime(v, stage)).transpose()?;
    let _ = f.before_restore.map(|v| mtime(v, stage)).transpose()?;
    Ok(NativeFacts {
        stage: phase,
        status,
        reason: reason(&value.reason),
        expected_hash,
        desired_hash,
        project,
        file,
        prepared_current,
        prepared_saved,
        frozen_current,
        buffer_changed: f.buffer_changed,
        resource_changed: f.resource_changed,
        write_started: f.write_started,
        write_calls: number(f.write_calls, stage)?,
        written_bytes: number(f.written_bytes, stage)?,
        write_errno: errno(f.write_errno, stage)?,
        truncated: f.truncate_done,
        truncate_errno: errno(f.truncate_errno, stage)?,
        flushed: f.fsync_done,
        fsync_errno: errno(f.fsync_errno, stage)?,
        readback: f.pread_done,
        pread_errno: errno(f.pread_errno, stage)?,
        restore_attempted: f.futimens_called,
        restore_errno: errno(f.futimens_errno, stage)?,
        restored: f.mtime_restored,
        edited_attempted: f.edited_clear_attempted,
        edited_cleared: f.edited_cleared,
        tag_attempted: f.tag_attempted,
        tagged: f.tagged,
        t0: mtime(f.t0, stage)?,
        after_restore,
    })
}
fn saved(
    value: SavedIn,
    selected: &SelectedSession,
    request: &ObservationRequest,
    receipt: u64,
    stage: Stage,
) -> Result<SavedStateEvidence, RoutingFailure> {
    let document = value.document.domain(request.script_path(), stage)?;
    if document.kind() != ScriptKind::ExternalGdscript {
        return Err(bad(stage));
    }
    Ok(SavedStateEvidence {
        request_id: request.request_id().clone(),
        session_id: selected.target().session_id().clone(),
        document,
        collection: value.collection.domain(
            selected.target().session_id(),
            receipt,
            stage,
            true,
        )?,
        current_version: decimal(value.current_version, stage)?,
        saved_version: decimal(value.saved_version, stage)?,
        resource_edited: value.resource_edited,
        save_profile: hex(&value.save_profile).ok_or_else(|| bad(stage))?,
        original_preserved: value.original_preserved,
        desired_preserved: value.desired_preserved,
    })
}
fn context(
    value: ContextIn,
    selected: &SelectedSession,
    receipt: u64,
    stage: Stage,
) -> Result<EditContext, RoutingFailure> {
    let executable = PathBuf::from(value.executable_path);
    if !executable.is_absolute()
        || value.global_classes.0.iter().any(|s| s.len() > 2048)
        || value.warning_profile.levels.len() > 128
        || value.warning_profile.directory_rules.len() > 128
    {
        return Err(bad(stage));
    }
    let sid = selected.target().session_id().as_str().to_owned();
    Ok(EditContext {
        executable,
        warnings: stock_validation::WarningSettings {
            enable: value.warning_profile.enable,
            levels: value.warning_profile.levels,
            directory_rules: value.warning_profile.directory_rules,
            provenance: stock_validation::WarningProvenance {
                source: stock_validation::WarningOrigin::EditorProjectSettings,
                project_root: selected.target().project_root().as_str().into(),
                session_id: sid,
            },
        },
        global_classes: value.global_classes.0,
        collection: value.collection.domain(
            selected.target().session_id(),
            receipt,
            stage,
            true,
        )?,
    })
}
fn sample(
    value: SampleIn,
    saved_in: Option<SavedIn>,
    context_in: Option<ContextIn>,
    selected: &SelectedSession,
    request: &ObservationRequest,
    receipt: u64,
    stage: Stage,
) -> Result<EditSample, RoutingFailure> {
    let editor = value.domain(
        request,
        selected.target(),
        &selected.advertised_project_root,
        receipt,
        stage,
    )?;
    let saved = saved_in
        .map(|v| saved(v, selected, request, receipt, stage))
        .transpose()?;
    let context = context_in
        .map(|v| context(v, selected, receipt, stage))
        .transpose()?;
    Ok(EditSample {
        collection: editor.collection.clone(),
        sample: editor,
        saved,
        context,
    })
}
fn frame(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    bytes: &[u8],
    started: Instant,
    deadline: Instant,
    stage: Stage,
    large: bool,
) -> Result<Vec<u8>, RoutingFailure> {
    if !selected.matches_request(request) {
        return Err(bad(stage));
    }
    if Instant::now() >= deadline {
        return Err(timed(stage));
    }
    if bytes.len() > (if large { 4 * 1024 * 1024 } else { 4096 }) {
        return Err(bad(stage));
    }
    write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        stage,
    )?;
    write_deadline(&mut selected.socket, bytes, deadline, stage)?;
    let response = receive_editor(&mut selected.socket, deadline, stage)?;
    if Instant::now() >= deadline || started.elapsed().as_micros() > 9_500_000 {
        return Err(timed(stage));
    }
    Ok(response)
}
fn answer<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    selected: &SelectedSession,
    request: &ObservationRequest,
    stage: Stage,
) -> Result<T, RoutingFailure> {
    editor_reply(
        bytes,
        request,
        selected.target(),
        &selected.advertised_project_root,
        stage,
    )
}
/// One immutable prepare, with source only on the uniquely selected authenticated channel.
pub fn prepare(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    basis: &ExpectedRevisionBasis,
    replacement: &ReplacementSource,
    started: Instant,
    deadline: Instant,
) -> Result<Prepared, RoutingFailure> {
    let stage = Stage::ReadEditor;
    let budget = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(9000) as u64;
    if budget == 0 {
        return Err(timed(stage));
    }
    let target = selected.target();
    let id = basis.document().disk_file_id().ok_or_else(|| bad(stage))?;
    let doc = basis.document();
    let [script, editor, buffer] = [
        doc.script_instance_id(),
        doc.editor_instance_id(),
        doc.buffer_instance_id(),
    ];
    let (Some(script), Some(editor), Some(buffer)) = (script, editor, buffer) else {
        return Err(bad(stage));
    };
    let hash = encode_hex(&basis.source().sha256);
    #[derive(serde::Serialize)]
    #[serde(untagged)]
    enum Argument<'a> {
        Number(u64),
        Text(&'a str),
    }
    use Argument::{Number, Text};
    let source_bytes = basis.source().utf8_bytes.to_string();
    let tuple = [
        Number(3),
        Text("edit_prepare"),
        Text(request.request_id().as_str()),
        Text(target.session_id().as_str()),
        Text(selected.advertised_project_root.as_str()),
        Text(request.script_path().as_str()),
        Number(budget),
        Text(basis.prior_request_id().as_str()),
        Text(target.project_file_id().device().as_str()),
        Text(target.project_file_id().inode().as_str()),
        Text(id.device().as_str()),
        Text(id.inode().as_str()),
        Text(script.as_str()),
        Text(editor.as_str()),
        Text(buffer.as_str()),
        Text(basis.current_version().as_str()),
        Text(&hash),
        Text(&source_bytes),
        Text(replacement.as_str()),
    ];
    let bytes = serde_json::to_vec(&tuple).map_err(|_| bad(stage))?;
    let reply: PreparedIn = answer(
        &frame(selected, request, &bytes, started, deadline, stage, true)?,
        selected,
        request,
        stage,
    )?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let stamp = bound(
        (
            reply.v,
            &reply.kind,
            &reply.request_id,
            &reply.session_id,
            &reply.project_root,
            &reply.script_path,
        ),
        "edit_prepared",
        reply.collection,
        selected,
        request,
        receipt,
        stage,
    )?;
    let status = match reply.status.as_str() {
        "prepared" => "prepared",
        "busy" => "busy",
        "refused" => "refused",
        _ => return Err(bad(stage)),
    };
    let intent_changed = match reply.intent.as_deref() {
        Some("changed") => Some(true),
        Some("unchanged") => Some(false),
        None => None,
        _ => return Err(bad(stage)),
    };
    if status == "busy"
        && (reply.sample.is_some()
            || reply.native.is_some()
            || reply.saved_state.is_some()
            || reply.context.is_some()
            || intent_changed.is_some())
    {
        return Err(bad(stage));
    }
    let native = reply.native.map(|v| native(v, stage)).transpose()?;
    let sample = reply
        .sample
        .map(|v| {
            sample(
                v,
                reply.saved_state,
                reply.context,
                selected,
                request,
                receipt,
                stage,
            )
        })
        .transpose()?;
    if let Some(sample) = &sample {
        within_editor_interval(&sample.collection, &stamp, stage)?;
        if let Some(saved) = &sample.saved {
            within_editor_interval(&saved.collection, &stamp, stage)?;
        }
        if let Some(context) = &sample.context {
            within_editor_interval(&context.collection, &stamp, stage)?;
        }
    }
    if status == "prepared"
        && (sample.is_none() || intent_changed.is_none() || reply.expiry_tick_us.is_none())
    {
        return Err(bad(stage));
    }
    if let Some(expiry) = reply.expiry_tick_us {
        let _ = decimal(expiry, stage)?;
    }
    Ok(Prepared {
        status,
        reason: reason(&reply.reason),
        intent_changed,
        native,
        sample,
    })
}
fn control(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    opcode: &str,
    purpose: Option<&str>,
    started: Instant,
    deadline: Instant,
) -> Result<Vec<u8>, RoutingFailure> {
    let stage = Stage::ReadEditor;
    let target = selected.target();
    let prefix = (
        3,
        opcode,
        request.request_id().as_str(),
        target.session_id().as_str(),
        selected.advertised_project_root.as_str(),
        request.script_path().as_str(),
    );
    let bytes = match purpose {
        Some(p) => serde_json::to_vec(&(
            prefix.0, prefix.1, prefix.2, prefix.3, prefix.4, prefix.5, p,
        )),
        None => serde_json::to_vec(&prefix),
    }
    .map_err(|_| bad(stage))?;
    frame(selected, request, &bytes, started, deadline, stage, false)
}
/// Apply consumes the one prepared intent; never sends the source again.
pub fn apply<F>(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    started: Instant,
    deadline: Instant,
    mut progress: F,
) -> Result<Applied, RoutingFailure>
where
    F: FnMut(&AppliedEvent) -> Result<(), RoutingFailure>,
{
    let stage = Stage::ReadEditor;
    if !selected.matches_request(request) {
        return Err(bad(stage));
    }
    let target = selected.target();
    let bytes = serde_json::to_vec(&(
        3,
        "edit_apply",
        request.request_id().as_str(),
        target.session_id().as_str(),
        selected.advertised_project_root.as_str(),
        request.script_path().as_str(),
    ))
    .map_err(|_| bad(stage))?;
    if bytes.len() > 4096 {
        return Err(bad(stage));
    }
    write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        stage,
    )?;
    write_deadline(&mut selected.socket, &bytes, deadline, stage)?;
    let mut last_progress: Option<(usize, CollectionStamp)> = None;
    loop {
        let bytes = receive_editor(&mut selected.socket, deadline, stage)?;
        if Instant::now() >= deadline {
            return Err(timed(stage));
        }
        let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        #[derive(Deserialize)]
        struct KindProbe<'a> {
            kind: &'a str,
        }
        let probe: KindProbe<'_> = decode(&bytes, stage)?;
        match probe.kind {
            "edit_progress" => {
                let reply: ProgressIn = answer(&bytes, selected, request, stage)?;
                let _ = bound(
                    (
                        reply.v,
                        &reply.kind,
                        &reply.request_id,
                        &reply.session_id,
                        &reply.project_root,
                        &reply.script_path,
                    ),
                    "edit_progress",
                    reply.collection,
                    selected,
                    request,
                    receipt,
                    stage,
                )?;
                let event = applied_event(reply.event, selected, request, receipt, stage)?;
                let step = phase_index(event.stage);
                if last_progress.as_ref().is_some_and(|(previous, stamp)| {
                    step != previous + 1
                        || event.collection.started_tick_us() < stamp.finished_tick_us()
                }) || last_progress.is_none() && step != 1
                {
                    return Err(bad(stage));
                }
                progress(&event)?;
                last_progress = Some((step, event.collection));
            }
            "edit_applied" => {
                let reply: AppliedIn = answer(&bytes, selected, request, stage)?;
                let collection = bound(
                    (
                        reply.v,
                        &reply.kind,
                        &reply.request_id,
                        &reply.session_id,
                        &reply.project_root,
                        &reply.script_path,
                    ),
                    "edit_applied",
                    reply.collection,
                    selected,
                    request,
                    receipt,
                    stage,
                )?;
                let status = match reply.status.as_str() {
                    "ready" => "ready",
                    "complete" => "complete",
                    "refused" => "refused",
                    "partial" => "partial",
                    _ => return Err(bad(stage)),
                };
                let native = reply.native.map(|v| native(v, stage)).transpose()?;
                let previous = last_progress.as_ref().map_or(0, |(step, _)| *step);
                let mut terminal_events = reply.events.0.into_iter();
                let terminal_event = terminal_events
                    .next()
                    .map(|event| applied_event(event, selected, request, receipt, stage))
                    .transpose()?;
                if matches!(status, "ready" | "complete") {
                    if terminal_event.is_some()
                        || previous != 6
                        || native
                            .as_ref()
                            .is_none_or(|n| n.stage != "saved_tagged" || n.status != "complete")
                    {
                        return Err(bad(stage));
                    }
                } else if let Some(event) = &terminal_event {
                    let last = native.as_ref().ok_or_else(|| bad(stage))?;
                    let step = phase_index(event.stage);
                    if event.stage != last.stage
                        || event.native.status != last.status
                        || !matches!(last.status, "partial" | "refused")
                        || step < previous
                        || step > previous + 1
                        || last_progress.as_ref().is_some_and(|(_, stamp)| {
                            event.collection.started_tick_us() < stamp.finished_tick_us()
                        })
                    {
                        return Err(bad(stage));
                    }
                } else if native.is_some() || previous != 0 {
                    return Err(bad(stage));
                }
                return Ok(Applied {
                    collection,
                    status,
                    reason: reason(&reply.reason),
                    native,
                    terminal_event,
                });
            }
            _ => return Err(bad(stage)),
        }
    }
}
fn phase_index(stage: &str) -> usize {
    match stage {
        "prepared" => 0,
        "buffer_applied" => 1,
        "resource_applied" => 2,
        "content_persisted" => 3,
        "mtime_restored" => 4,
        "edited_cleared" => 5,
        "saved_tagged" => 6,
        _ => usize::MAX,
    }
}
fn applied_event(
    event: NativeEventIn,
    selected: &SelectedSession,
    request: &ObservationRequest,
    receipt: u64,
    stage: Stage,
) -> Result<AppliedEvent, RoutingFailure> {
    let label = phase(&event.stage).ok_or_else(|| bad(stage))?;
    let stamp = event
        .collection
        .domain(selected.target().session_id(), receipt, stage, true)?;
    let native = native(event.native, stage)?;
    if native.stage != label {
        return Err(bad(stage));
    }
    let before = event
        .before_sample
        .map(|v| {
            sample(
                v,
                event.before_saved_state,
                None,
                selected,
                request,
                receipt,
                stage,
            )
        })
        .transpose()?;
    let after = event
        .after_sample
        .map(|v| {
            sample(
                v,
                event.after_saved_state,
                None,
                selected,
                request,
                receipt,
                stage,
            )
        })
        .transpose()?;
    Ok(AppliedEvent {
        collection: stamp,
        stage: label,
        native,
        before,
        after,
    })
}
pub fn verify(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    started: Instant,
    deadline: Instant,
) -> Result<EditSample, RoutingFailure> {
    let stage = Stage::ReadEditor;
    let reply: SampleReplyIn = answer(
        &control(
            selected,
            request,
            "edit_verify",
            Some(purpose),
            started,
            deadline,
        )?,
        selected,
        request,
        stage,
    )?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let _ = bound(
        (
            reply.v,
            &reply.kind,
            &reply.request_id,
            &reply.session_id,
            &reply.project_root,
            &reply.script_path,
        ),
        "edit_sample",
        reply.collection,
        selected,
        request,
        receipt,
        stage,
    )?;
    if reply.purpose != purpose {
        return Err(bad(stage));
    }
    sample(
        reply.sample.ok_or_else(|| bad(stage))?,
        reply.saved_state,
        reply.context,
        selected,
        request,
        receipt,
        stage,
    )
}
pub fn recheck(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    purpose: &str,
    original: &CollectionStamp,
    started: Instant,
    deadline: Instant,
) -> Result<Rechecked, RoutingFailure> {
    let stage = Stage::Recheck;
    let reply: RecheckedIn = answer(
        &control(
            selected,
            request,
            "edit_recheck",
            Some(purpose),
            started,
            deadline,
        )?,
        selected,
        request,
        stage,
    )?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let _ = bound(
        (
            reply.v,
            &reply.kind,
            &reply.request_id,
            &reply.session_id,
            &reply.project_root,
            &reply.script_path,
        ),
        "edit_rechecked",
        reply.collection,
        selected,
        request,
        receipt,
        stage,
    )?;
    if reply.purpose != purpose {
        return Err(bad(stage));
    }
    Ok(Rechecked {
        recheck: reply.recheck.domain(
            request,
            selected.target(),
            &selected.advertised_project_root,
            receipt,
            stage,
            original,
        )?,
        saved: reply
            .saved_state
            .map(|v| saved(v, selected, request, receipt, stage))
            .transpose()?,
        context: reply
            .context
            .map(|v| context(v, selected, receipt, stage))
            .transpose()?,
    })
}
pub fn terminal(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    abort: bool,
    started: Instant,
    deadline: Instant,
) -> Result<Terminal, RoutingFailure> {
    let stage = Stage::Finalize;
    let reply: TerminalIn = answer(
        &control(
            selected,
            request,
            if abort { "edit_abort" } else { "edit_finish" },
            None,
            started,
            deadline,
        )?,
        selected,
        request,
        stage,
    )?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    let collection = bound(
        (
            reply.v,
            &reply.kind,
            &reply.request_id,
            &reply.session_id,
            &reply.project_root,
            &reply.script_path,
        ),
        if abort {
            "edit_aborted"
        } else {
            "edit_finished"
        },
        reply.collection,
        selected,
        request,
        receipt,
        stage,
    )?;
    if !matches!(reply.status.as_str(), "terminal" | "busy") {
        return Err(bad(stage));
    }
    let native = reply.native.map(|v| native(v, stage)).transpose()?;
    if reply.terminal_before_boundary
        && (reply.status != "terminal"
            || native.as_ref().is_some_and(|n| {
                n.buffer_changed
                    || n.resource_changed
                    || n.write_started
                    || n.edited_attempted
                    || n.tag_attempted
            }))
    {
        return Err(bad(stage));
    }
    Ok(Terminal {
        collection,
        terminal_before_boundary: reply.terminal_before_boundary,
        reason: reason(&reply.reason),
    })
}
