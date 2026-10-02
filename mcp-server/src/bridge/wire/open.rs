//! Opening-v1 output and strict private-v4 native/ordinary evidence exchange.
use super::*;
use crate::runner::stock_validation::{OpeningContext, WarningSettings};
use crate::script_open::{self, Reason};
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
mod output;
pub use output::{encode_outcome, exit_code, invalid_document_kind, invalid_request};

struct Nullable<T>(Option<T>);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Present<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Present<T> {
            type Value = Nullable<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a required typed value or explicit null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Nullable(None))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                T::deserialize(serde::de::value::StrDeserializer::<E>::new(v))
                    .map(|v| Nullable(Some(v)))
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                T::deserialize(serde::de::value::BoolDeserializer::<E>::new(v))
                    .map(|v| Nullable(Some(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                T::deserialize(serde::de::value::I64Deserializer::<E>::new(v))
                    .map(|v| Nullable(Some(v)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                T::deserialize(serde::de::value::U64Deserializer::<E>::new(v))
                    .map(|v| Nullable(Some(v)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                v: A,
            ) -> Result<Self::Value, A::Error> {
                T::deserialize(serde::de::value::MapAccessDeserializer::new(v))
                    .map(|v| Nullable(Some(v)))
            }
        }
        d.deserialize_any(Present(std::marker::PhantomData))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIn {
    request_id: Nullable<String>,
    stage: Nullable<String>,
    next_stage: Nullable<String>,
    cache_binding: Nullable<String>,
    initial_compilation: Nullable<String>,
    document_open: Nullable<String>,
    target_parse_code: Nullable<i64>,
    script_instance_id: Nullable<String>,
    editor_instance_id: Nullable<String>,
    buffer_instance_id: Nullable<String>,
    target_open: Nullable<bool>,
    terminal_discard: Nullable<bool>,
    entered: Nullable<bool>,
    mode: Nullable<String>,
    protection: Nullable<String>,
    selection: Nullable<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheIn {
    state: String,
    script_instance_id: Nullable<String>,
    sha256: Nullable<String>,
    utf8_bytes: Nullable<String>,
    reason: Nullable<String>,
}
#[derive(Debug)]
pub(crate) struct Cache {
    pub state: &'static str,
    pub script: Option<DecimalCounter>,
    pub hash: Option<String>,
    pub length: Option<usize>,
    pub reason: Option<Reason>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Validation {
    pub executable: String,
    #[serde(deserialize_with = "checked_warnings")]
    pub warnings: WarningSettings,
    pub global_classes: Vec<String>,
}
fn unique_levels<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<std::collections::BTreeMap<String, u8>, D::Error> {
    struct Levels;
    impl<'de> serde::de::Visitor<'de> for Levels {
        type Value = std::collections::BTreeMap<String, u8>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded unique warning levels")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = std::collections::BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, u8>()? {
                if out.len() == 128 || key.len() > 2048 || out.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("invalid warning map"));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_map(Levels)
}
pub(super) fn checked_warnings<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<WarningSettings, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Warnings {
        enable: bool,
        #[serde(deserialize_with = "unique_levels")]
        levels: std::collections::BTreeMap<String, u8>,
        #[serde(deserialize_with = "unique_levels")]
        directory_rules: std::collections::BTreeMap<String, u8>,
        provenance: crate::runner::stock_validation::WarningProvenance,
    }
    let w = Warnings::deserialize(d)?;
    Ok(WarningSettings {
        enable: w.enable,
        levels: w.levels,
        directory_rules: w.directory_rules,
        provenance: w.provenance,
    })
}

macro_rules! response {
    ($name:ident { $($key:ident:$ty:ty),* $(,)? }) => {
        #[derive(Deserialize)] #[serde(deny_unknown_fields)]
        struct $name {
            v:u32,kind:String,request_id:String,session_id:String,project_root:String,script_path:String,
            collection:StampIn,status:String,reason:String,native:Nullable<NativeIn>,resource_edited:Nullable<bool>,expiry_tick_us:Nullable<String>,
            $($key:$ty),*
        }
    }
}
response!(StateIn {sample:Nullable<SampleIn>,cache:Nullable<CacheIn>});
response!(PreparedIn {mode:Nullable<String>,sample:Nullable<SampleIn>,context:Nullable<OpeningContext>,validation:Nullable<Validation>,project_device:Nullable<String>,project_inode:Nullable<String>,target_device:Nullable<String>,target_inode:Nullable<String>});
response!(ProgressIn {});
response!(SampleReplyIn {purpose:String,sample:Nullable<SampleIn>,protection:Nullable<String>,selection:String});
response!(RecheckedIn {purpose:String,recheck:RecheckIn,protection:Nullable<String>,selection:String});
response!(TerminalIn {
    terminal_discard: bool
});

pub(crate) struct Reply {
    pub status: String,
    pub reason: Option<Reason>,
    pub collection: CollectionStamp,
    pub native: Option<script_open::NativeEvidence>,
    pub sample: Option<EditorSample>,
    pub cache: Option<Cache>,
    pub edited: Option<bool>,
    pub expiry: Option<DecimalCounter>,
    pub mode: Option<String>,
    pub context: Option<OpeningContext>,
    pub validation: Option<Validation>,
    pub file: Option<(FileIdentity, FileIdentity)>,
    pub purpose: Option<String>,
    pub recheck: Option<Recheck>,
    pub protection: Option<String>,
    pub selection: Option<&'static str>,
}
fn native(v: NativeIn, id: &RequestId) -> Result<script_open::NativeEvidence, RoutingFailure> {
    let stage = Stage::ReadEditor;
    if v.request_id.0.as_deref() != Some(id.as_str()) {
        return Err(bad(stage));
    }
    if v.stage.0.is_none()
        || v.next_stage.0.is_none()
        || v.cache_binding.0.is_none()
        || v.initial_compilation.0.is_none()
        || v.document_open.0.is_none()
        || v.terminal_discard.0.is_none()
        || v.entered.0.is_none()
    {
        return Err(bad(stage));
    }
    for (actual, allowed) in [
        (
            &v.stage.0,
            &[
                "inspected",
                "prepared",
                "bind",
                "compile",
                "open",
                "unknown",
            ][..],
        ),
        (
            &v.next_stage.0,
            &["", "prepare", "bind", "compile", "open"][..],
        ),
        (
            &v.cache_binding.0,
            &[
                "not_started",
                "reused_existing",
                "new_resource_published",
                "failed_before_publication",
                "unknown",
            ][..],
        ),
        (
            &v.initial_compilation.0,
            &[
                "not_started",
                "not_applicable",
                "completed_valid",
                "completed_invalid",
                "unavailable_failed",
                "unknown",
            ][..],
        ),
        (
            &v.document_open.0,
            &[
                "not_started",
                "entered",
                "association_obtained",
                "failed_unverified",
                "unknown",
            ][..],
        ),
        (&v.mode.0, &["cold", "cached"][..]),
        (&v.protection.0, &["not_applicable", "unchanged"][..]),
    ] {
        if actual
            .as_ref()
            .is_some_and(|s| !allowed.contains(&s.as_str()))
        {
            return Err(bad(stage));
        }
    }
    if v.target_parse_code
        .0
        .is_some_and(|c| c < 0 || c > i64::from(i32::MAX))
    {
        return Err(bad(stage));
    }
    let counter = |v: Nullable<String>| v.0.map(|s| checked_id(s, stage)).transpose();
    Ok(script_open::NativeEvidence {
        stage: v.stage.0,
        next: v.next_stage.0,
        binding: v.cache_binding.0,
        compilation: v.initial_compilation.0,
        document: v.document_open.0,
        parse: v.target_parse_code.0,
        script: counter(v.script_instance_id)?,
        editor: counter(v.editor_instance_id)?,
        buffer: counter(v.buffer_instance_id)?,
        open: v.target_open.0,
        discard: v.terminal_discard.0,
        entered: v.entered.0,
        mode: v.mode.0,
        protection: v.protection.0,
        selection: v.selection.0.as_deref().map(relation).transpose()?,
    })
}
fn checked_id(v: String, stage: Stage) -> Result<DecimalCounter, RoutingFailure> {
    let d = decimal(v, stage)?;
    if d.as_str().parse::<u64>().ok().filter(|n| *n != 0).is_none() {
        return Err(bad(stage));
    }
    Ok(d)
}
fn natural(v: String) -> Result<DecimalCounter, RoutingFailure> {
    let d = decimal(v, Stage::ReadEditor)?;
    d.as_str()
        .parse::<u64>()
        .map_err(|_| bad(Stage::ReadEditor))?;
    Ok(d)
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn cache(v: CacheIn) -> Result<Cache, RoutingFailure> {
    let mut result = Cache {
        state: match v.state.as_str() {
            "absent" => "absent",
            "present" => "present",
            "unavailable" => "unavailable",
            _ => return Err(bad(Stage::ReadEditor)),
        },
        script: v
            .script_instance_id
            .0
            .map(|v| checked_id(v, Stage::ReadEditor))
            .transpose()?,
        hash: v.sha256.0,
        length: v
            .utf8_bytes
            .0
            .map(|v| {
                natural(v).and_then(|d| {
                    d.as_str()
                        .parse::<usize>()
                        .map_err(|_| bad(Stage::ReadEditor))
                })
            })
            .transpose()?,
        reason: v.reason.0.as_deref().map(machine_reason),
    };
    if result.hash.as_ref().is_some_and(|v| !digest(v))
        || result.length.is_some_and(|n| n > SOURCE_LIMIT_BYTES)
        || result.hash.is_some() != result.length.is_some()
    {
        return Err(bad(Stage::ReadEditor));
    }
    match result.state {
        "absent" if result.script.is_none() && result.hash.is_none() && result.reason.is_none() => {
        }
        "present" if result.script.is_some() => {}
        "unavailable"
            if result.script.is_none() && result.hash.is_none() && result.reason.is_some() => {}
        _ => return Err(bad(Stage::ReadEditor)),
    }
    if result.state == "present" && result.hash.is_none() && result.reason.is_none() {
        result.reason = Some(Reason::EvidenceUnavailable);
    }
    Ok(result)
}
pub(crate) fn machine_reason(v: &str) -> Reason {
    match v {
        "slot_busy" | "busy" | "active_native_stage" | "closing_after_active_stage" => Reason::Busy,
        "cached_source_conflict" => Reason::CachedSourceConflict,
        "resource_edited" => Reason::DirtyConflict,
        "disk_content_changed" | "resource_source_changed" => Reason::SourceChanged,
        "namespace_or_descriptor_changed"
        | "target_opened_during_attempt"
        | "cache_identity_changed"
        | "cache_state_changed"
        | "cache_document_identity_mismatch"
        | "document_association_changed"
        | "cache_path_mismatch" => Reason::TargetChanged,
        "current_context_changed"
        | "effective_context_changed"
        | "target_class_bindings_changed"
        | "current_document_association_changed"
        | "current_namespace_changed" => Reason::ContextChanged,
        "session_closed" => Reason::SessionEnded,
        "session_or_expiry_changed" | "invalid_expiry" => Reason::Timeout,
        "cancelled" => Reason::Cancelled,
        "wrong_type_cache" => Reason::InvalidDocumentKind,
        "native_unavailable" | "unavailable" | "capability_unavailable" => {
            Reason::UnsupportedCapability
        }
        "document_association_unavailable" => Reason::OpenStateUnknown,
        "resource_source_unavailable"
        | "cached_source_unavailable"
        | "resource_edited_unavailable"
        | "cache_unavailable"
        | "source_unavailable"
        | "evidence_limit"
        | "remap_state_unavailable" => Reason::EvidenceUnavailable,
        "initial_compilation_unavailable"
        | "initial_compilation_failed"
        | "initial_compilation_result_unavailable" => Reason::CompilationUnavailable,
        "document_open_unverified" => Reason::VerificationIncomplete,
        "file_missing" | "missing_script" => Reason::MissingScript,
        "file_denied" | "denied_access" | "unsafe_scope" => Reason::DeniedAccess,
        "outside_project" => Reason::OutsideProject,
        "wrong_attempt"
        | "invalid_binding"
        | "invalid_capture_binding"
        | "wrong_attempt_or_stage"
        | "wrong_stage_or_context_validation_binding"
        | "wrong_verification_purpose"
        | "existing_resource_compilation_forbidden" => Reason::ProtocolError,
        "project_source_dependency" => Reason::UnsupportedRepresentation,
        "revision_changed" => Reason::RevisionChanged,
        "session_changed" => Reason::SessionChanged,
        "unsupported_current_editor"
        | "unsupported_current_script"
        | "unsupported_current_path"
        | "unsafe_source_profile"
        | "unsafe_compiled_property"
        | "unsafe_compiled_method"
        | "duplicate_compiled_property"
        | "duplicate_compiled_method"
        | "duplicate_effective_name"
        | "external_editor_enabled"
        | "external_editor_unavailable"
        | "warning_rules_unavailable"
        | "warning_context_unavailable_or_limit"
        | "warning_context_unavailable"
        | "global_context_unavailable"
        | "autoload_context_unavailable_or_limit" => Reason::UnsafeEditorContext,
        s if s.starts_with("current_")
            || s.starts_with("compiled_")
            || s.starts_with("effective_") =>
        {
            Reason::UnsafeEditorContext
        }
        s if s.contains("profile")
            || s.contains("limit")
            || s.contains("unsupported")
            || s.contains("external_editor")
            || s.contains("binding") =>
        {
            Reason::UnsupportedRepresentation
        }
        _ => Reason::NativeFailure,
    }
}
fn relation(v: &str) -> Result<&'static str, RoutingFailure> {
    match v {
        "requested_target" => Ok("target"),
        "other" => Ok("other"),
        "no_source" => Ok("no_source_editor"),
        "unknown" => Ok("unknown"),
        _ => Err(bad(Stage::ReadEditor)),
    }
}
macro_rules! base {
    ($v:ident,$kind:expr,$request:expr,$target:expr,$advertised:expr,$receipt:expr) => {{
        if $v.v != 5
            || $v.kind != $kind
            || $v.request_id != $request.request_id().as_str()
            || $v.session_id != $target.session_id().as_str()
            || $v.project_root != $advertised
            || $v.script_path != $request.script_path().as_str()
        {
            return Err(bad(Stage::ReadEditor));
        }
        let collection =
            $v.collection
                .domain($target.session_id(), $receipt, Stage::ReadEditor, true)?;
        if ![
            "inspected",
            "prepared",
            "ready",
            "observed",
            "unchanged",
            "discarded",
            "refused",
            "partial",
        ]
        .contains(&$v.status.as_str())
            || $v.reason.len() > 128
            || $v.reason.chars().any(char::is_control)
        {
            return Err(bad(Stage::ReadEditor));
        }
        let success = match $kind {
            "open_state" => "inspected",
            "open_prepared" => "prepared",
            "open_progress" => "ready",
            "open_sample" => "observed",
            "open_rechecked" => "unchanged",
            _ => "discarded",
        };
        if $v.status != success && !["refused", "partial"].contains(&$v.status.as_str()) {
            return Err(bad(Stage::ReadEditor));
        }
        if $v.status == success && success != "discarded" && !$v.reason.is_empty() {
            return Err(bad(Stage::ReadEditor));
        }
        let expiry = $v.expiry_tick_us.0.map(natural).transpose()?;
        if expiry
            .as_ref()
            .is_some_and(|e| e <= collection.finished_tick_us())
            && !["refused", "partial", "discarded"].contains(&$v.status.as_str())
        {
            return Err(bad(Stage::ReadEditor));
        }
        if $v.status == success && success != "discarded" && expiry.is_none() {
            return Err(bad(Stage::ReadEditor));
        }
        Reply {
            status: $v.status,
            reason: if $v.reason.is_empty() || $v.reason == "finished" {
                None
            } else {
                Some(machine_reason(&$v.reason))
            },
            collection,
            native: $v
                .native
                .0
                .map(|v| native(v, $request.request_id()))
                .transpose()?,
            sample: None,
            cache: None,
            edited: $v.resource_edited.0,
            expiry,
            mode: None,
            context: None,
            validation: None,
            file: None,
            purpose: None,
            recheck: None,
            protection: None,
            selection: None,
        }
    }};
}
/// The same decoder runs in worker and supervisor, using one receipt for nested facts.
fn typed_reply(
    bytes: &[u8],
    kind: &'static str,
    request: &ObservationRequest,
    target: &ResolvedTarget,
    advertised: &str,
    receipt: u64,
    original: Option<&CollectionStamp>,
) -> Result<Reply, RoutingFailure> {
    match kind {
        "open_state" => {
            let v: StateIn = decode(bytes, Stage::ReadEditor)?;
            let sample = v.sample.0;
            let c = v.cache.0;
            let mut r = base!(v, kind, request, target, advertised, receipt);
            r.cache = c.map(cache).transpose()?;
            r.sample = sample
                .map(|s| s.domain(request, target, advertised, receipt, Stage::ReadEditor))
                .transpose()?;
            Ok(r)
        }
        "open_prepared" => {
            let v: PreparedIn = decode(bytes, Stage::ReadEditor)?;
            let sample = v.sample.0;
            let context = v.context.0;
            let validation = v.validation.0;
            let mode = v.mode.0;
            let file = match (
                v.project_device.0,
                v.project_inode.0,
                v.target_device.0,
                v.target_inode.0,
            ) {
                (Some(pd), Some(pi), Some(td), Some(ti)) => Some((
                    FileIdentity::new(natural(pd)?, natural(pi)?),
                    FileIdentity::new(natural(td)?, natural(ti)?),
                )),
                (None, None, None, None) => None,
                _ => return Err(bad(Stage::ReadEditor)),
            };
            let mut r = base!(v, kind, request, target, advertised, receipt);
            if mode
                .as_ref()
                .is_some_and(|m| !["cold", "cached"].contains(&m.as_str()))
            {
                return Err(bad(Stage::ReadEditor));
            }
            if let Some(v) = &validation {
                if v.executable.len() > 4096
                    || !std::path::Path::new(&v.executable).is_absolute()
                    || v.global_classes.len() > 256
                    || v.global_classes.iter().any(|s| s.len() > 256)
                    || v.warnings.levels.len() > 128
                    || v.warnings.directory_rules.len() > 128
                {
                    return Err(bad(Stage::ReadEditor));
                }
            }
            r.mode = mode;
            r.file = file;
            r.context = context;
            r.validation = validation;
            r.sample = sample
                .map(|s| s.domain(request, target, advertised, receipt, Stage::ReadEditor))
                .transpose()?;
            Ok(r)
        }
        "open_progress" => {
            let v: ProgressIn = decode(bytes, Stage::ReadEditor)?;
            Ok(base!(v, kind, request, target, advertised, receipt))
        }
        "open_sample" => {
            let v: SampleReplyIn = decode(bytes, Stage::ReadEditor)?;
            let purpose = v.purpose;
            let sample = v.sample.0;
            let protection = v.protection.0;
            let selection = relation(&v.selection)?;
            let mut r = base!(v, kind, request, target, advertised, receipt);
            check_purpose(&purpose, &protection)?;
            r.purpose = Some(purpose);
            r.protection = protection;
            r.selection = Some(selection);
            r.sample = sample
                .map(|s| s.domain(request, target, advertised, receipt, Stage::ReadEditor))
                .transpose()?;
            Ok(r)
        }
        "open_rechecked" => {
            let v: RecheckedIn = decode(bytes, Stage::Recheck)?;
            let purpose = v.purpose;
            let recheck = v.recheck;
            let protection = v.protection.0;
            let selection = relation(&v.selection)?;
            let mut r = base!(v, kind, request, target, advertised, receipt);
            check_purpose(&purpose, &protection)?;
            r.purpose = Some(purpose);
            r.protection = protection;
            r.selection = Some(selection);
            r.recheck = Some(recheck.domain(
                request,
                target,
                advertised,
                receipt,
                Stage::Recheck,
                original.ok_or_else(|| bad(Stage::Recheck))?,
            )?);
            Ok(r)
        }
        "open_finished" | "open_aborted" => {
            let v: TerminalIn = decode(bytes, Stage::Finalize)?;
            let discard = v.terminal_discard;
            let r = base!(v, kind, request, target, advertised, receipt);
            if discard && r.native.as_ref().is_some_and(|n| n.discard != Some(true)) {
                return Err(bad(Stage::Finalize));
            }
            Ok(r)
        }
        _ => Err(bad(Stage::ReadEditor)),
    }
}
pub(crate) fn decode_reply(
    bytes: &[u8],
    kind: &'static str,
    request: &ObservationRequest,
    target: &ResolvedTarget,
    advertised: &str,
    receipt: u64,
    original: Option<&CollectionStamp>,
) -> Result<Reply, RoutingFailure> {
    let reply = typed_reply(bytes, kind, request, target, advertised, receipt, original)?;
    if let Some(sample) = &reply.sample {
        within_editor_interval(&sample.collection, &reply.collection, Stage::ReadEditor)?;
    }
    if let Some(n) = &reply.native {
        if reply.selection.is_some() && reply.selection != n.selection {
            return Err(bad(Stage::ReadEditor));
        }
        if reply.mode.is_some() && reply.mode != n.mode {
            return Err(bad(Stage::ReadEditor));
        }
    }
    Ok(reply)
}
fn check_purpose(p: &str, protection: &Option<String>) -> Result<(), RoutingFailure> {
    if !["recognition", "post_open"].contains(&p)
        || protection
            .as_ref()
            .is_some_and(|p| !["not_applicable", "unchanged"].contains(&p.as_str()))
    {
        return Err(bad(Stage::ReadEditor));
    }
    Ok(())
}
fn exchange(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    bytes: &[u8],
    deadline: Instant,
) -> Result<Vec<u8>, RoutingFailure> {
    if selected.target().request_id() != request.request_id()
        || selected.target().script_path() != request.script_path()
        || !selected.capabilities().open_gdscript
        || selected.native_api_revision() != 3
    {
        return Err(bad(Stage::ReadEditor));
    }
    if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
        return Err(bad(Stage::ReadEditor));
    }
    write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        Stage::ReadEditor,
    )?;
    write_deadline(&mut selected.socket, bytes, deadline, Stage::ReadEditor)?;
    receive_editor(&mut selected.socket, deadline, Stage::ReadEditor)
}
pub(crate) fn call(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    opcode: &str,
    arguments: &[&str],
    budget: Option<u64>,
    deadline: Instant,
) -> Result<Vec<u8>, RoutingFailure> {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Item<'a> {
        Number(u64),
        Text(&'a str),
    }
    let mut tuple = vec![
        Item::Number(5),
        Item::Text(opcode),
        Item::Text(request.request_id().as_str()),
        Item::Text(selected.target().session_id().as_str()),
        Item::Text(selected.advertised_project_root.as_str()),
        Item::Text(request.script_path().as_str()),
    ];
    if let Some(budget) = budget {
        if !(1..=9000).contains(&budget) {
            return Err(bad(Stage::ReadEditor));
        }
        tuple.push(Item::Number(budget));
    }
    tuple.extend(arguments.iter().map(|s| Item::Text(s)));
    let bytes = serde_json::to_vec(&tuple).map_err(|_| bad(Stage::ReadEditor))?;
    if opcode != "open_prepare" && bytes.len() > 4096 {
        return Err(bad(Stage::ReadEditor));
    }
    exchange(selected, request, &bytes, deadline)
}
pub(crate) fn encode_disk(
    id: &RequestId,
    disk: &SourceObservation,
) -> Result<Vec<u8>, RoutingFailure> {
    encode_ipc(envelope(
        id,
        "disk",
        DiskPayload {
            disk: source_out(disk),
            file_identity: None,
            file_collection: None,
        },
    ))
}
pub(crate) fn validation_request(
    context: OpeningContext,
    v: Validation,
    request: &ObservationRequest,
    target: &ResolvedTarget,
) -> Option<crate::runner::stock_validation::ValidationRequest> {
    context.validation_request(
        request,
        target,
        v.warnings,
        v.global_classes,
        PathBuf::from(v.executable),
    )
}

/// Best effort only: the supervisor never waits for a cleanup acknowledgment.
pub(crate) fn release(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    abort: bool,
    at: Instant,
) {
    let local = Instant::now() + Duration::from_millis(20);
    let deadline = at.min(local);
    let Ok(bytes) = serde_json::to_vec(&(
        5,
        if abort { "open_abort" } else { "open_finish" },
        request.request_id().as_str(),
        selected.target().session_id().as_str(),
        selected.advertised_project_root.as_str(),
        request.script_path().as_str(),
    )) else {
        return;
    };
    let _ = write_deadline(
        &mut selected.socket,
        &(bytes.len() as u32).to_be_bytes(),
        deadline,
        Stage::Finalize,
    )
    .and_then(|_| write_deadline(&mut selected.socket, &bytes, deadline, Stage::Finalize));
}

#[cfg(test)]
mod tests;
