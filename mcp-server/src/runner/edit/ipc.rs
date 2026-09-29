//! Same-binary private edit IPC: bounded typed details supplement the existing observation events.
use super::*;
use crate::runner::stock_validation;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{json, Value};

#[derive(Deserialize)]
pub(super) struct KindProbe<'a> {
    pub kind: &'a str,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StampIn {
    clock_id: String,
    started_tick_us: String,
    finished_tick_us: String,
    received_elapsed_us: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedIn {
    request_id: String,
    session_id: String,
    resource_path: String,
    script_instance_id: String,
    editor_instance_id: String,
    buffer_instance_id: String,
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
pub(super) struct DetailIn {
    v: u32,
    kind: String,
    request_id: String,
    phase: String,
    saved: Option<SavedIn>,
    mtime: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValidationIn {
    v: u32,
    kind: String,
    request_id: String,
    purpose: String,
    result: stock_validation::ValidationResult,
    started_tick_us: u64,
    finished_tick_us: u64,
    context_current: bool,
    dependencies_current: bool,
}
pub(super) const HELPER_REQUEST_LIMIT: usize = 256 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HelperRequestIn {
    v: u32,
    kind: String,
    request_id: String,
    purpose: stock_validation::Purpose,
    warnings: stock_validation::WarningSettings,
    global_classes: Vec<String>,
    official_binary: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HelperReplyIn {
    v: u32,
    kind: String,
    request_id: String,
    purpose: stock_validation::Purpose,
    result: stock_validation::ValidationResult,
}
pub(super) fn helper_request(
    id: &RequestId,
    purpose: stock_validation::Purpose,
    context: &wire::edit::EditContext,
) -> Result<Vec<u8>, RoutingFailure> {
    let bytes = serde_json::to_vec(&json!({"v":1,"kind":"edit_helper_request",
        "request_id":id.as_str(),"purpose":purpose,"warnings":context.warnings,
        "global_classes":context.global_classes,
        "official_binary":context.executable.to_str().ok_or_else(protocol_failure)?}))
    .map_err(|_| protocol_failure())?;
    if bytes.len() > HELPER_REQUEST_LIMIT {
        return Err(protocol_failure());
    }
    Ok(bytes)
}
impl HelperRequestIn {
    pub(super) fn domain(
        self,
        request: &EditRequest,
        purpose: stock_validation::Purpose,
    ) -> Result<stock_validation::ValidationRequest, RoutingFailure> {
        let target = request.expected().target();
        if self.v != 1
            || self.kind != "edit_helper_request"
            || self.request_id != request.request_id().as_str()
            || self.purpose != purpose
            || self.global_classes.len() > 256
            || self.global_classes.iter().any(|v| v.len() > 2048)
            || self.warnings.levels.len() > 128
            || self.warnings.directory_rules.len() > 128
            || self.warnings.provenance.project_root != target.project_root().as_str()
            || self.warnings.provenance.session_id != target.session_id().as_str()
            || self.official_binary.len() > 4096
            || !std::path::Path::new(&self.official_binary).is_absolute()
        {
            return Err(protocol_failure());
        }
        Ok(stock_validation::ValidationRequest {
            request_id: request.request_id().clone(),
            session_id: target.session_id().clone(),
            project_root: target.project_root().clone(),
            root_path: request.script_path().clone(),
            source: (purpose == stock_validation::Purpose::Preflight)
                .then(|| request.replacement_source().as_str().to_owned()),
            purpose,
            warnings: self.warnings,
            global_classes: self.global_classes,
            official_binary: self.official_binary.into(),
        })
    }
}
pub(super) fn helper_reply(
    id: &RequestId,
    purpose: stock_validation::Purpose,
    result: &stock_validation::ValidationResult,
) -> Value {
    json!({"v":1,"kind":"edit_helper_reply","request_id":id.as_str(),
        "purpose":purpose,"result":result})
}
impl HelperReplyIn {
    pub(super) fn domain(
        self,
        id: &RequestId,
        purpose: stock_validation::Purpose,
    ) -> Result<stock_validation::ValidationResult, RoutingFailure> {
        if self.v != 1
            || self.kind != "edit_helper_reply"
            || self.request_id != id.as_str()
            || self.purpose != purpose
            || self.result.request_id != id.as_str()
            || self.result.purpose != purpose
        {
            return Err(protocol_failure());
        }
        Ok(self.result)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FreshDependency {
    pub path: String,
    pub device: String,
    pub inode: String,
    pub sha256: String,
    pub utf8_bytes: usize,
}
fn bounded_dependencies<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<FreshDependency>, D::Error> {
    struct Bounded;
    impl<'de> Visitor<'de> for Bounded {
        type Value = Vec<FreshDependency>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most 32 captured dependencies")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(value) = input.next_element()? {
                if out.len() == 32 {
                    return Err(de::Error::custom("dependency limit"));
                }
                out.push(value);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Bounded)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ContextIn {
    v: u32,
    kind: String,
    request_id: String,
    started_tick_us: u64,
    finished_tick_us: u64,
    current: bool,
    context_sha256: String,
    #[serde(deserialize_with = "bounded_dependencies")]
    dependencies: Vec<FreshDependency>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIn {
    stage: String,
    status: String,
    reason: String,
    expected_hash: String,
    desired_hash: String,
    project_device: String,
    project_inode: String,
    file_device: String,
    file_inode: String,
    prepared_current: String,
    prepared_saved: String,
    frozen_current: Option<String>,
    buffer_changed: bool,
    resource_changed: bool,
    write_started: bool,
    write_calls: usize,
    written_bytes: usize,
    write_errno: i32,
    truncated: bool,
    truncate_errno: i32,
    flushed: bool,
    fsync_errno: i32,
    readback: bool,
    pread_errno: i32,
    restore_attempted: bool,
    restore_errno: i32,
    restored: bool,
    edited_attempted: bool,
    edited_cleared: bool,
    tag_attempted: bool,
    tagged: bool,
    t0: String,
    after_restore: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleBindingIn {
    resource_path: String,
    script_instance_id: Option<String>,
    editor_instance_id: Option<String>,
    buffer_instance_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateIn {
    document: Option<SampleBindingIn>,
    source: Option<HashIn>,
    current: Option<String>,
    saved: Option<String>,
    resource_edited: Option<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HashIn {
    sha256: String,
    utf8_bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProgressIn {
    v: u32,
    kind: String,
    request_id: String,
    stage: String,
    collection: StampIn,
    native: NativeIn,
    before: Option<StateIn>,
    after: Option<StateIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerminalIn {
    v: u32,
    kind: String,
    request_id: String,
    reason: String,
    terminal_before_boundary: bool,
    collection: Option<StampIn>,
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out
}
fn digest(text: &str) -> Value {
    let value = script_edit::SourceDigest::of(text);
    json!({"sha256":hex(&value.sha256),"utf8_bytes":value.utf8_bytes})
}
fn state(sample: &wire::edit::EditSample) -> Value {
    let src = sample.sample.buffer.text().map(digest);
    let document = sample.sample.document.identity();
    let saved = sample
        .saved
        .as_ref()
        .filter(|s| Some(&s.document) == document);
    json!({"document":document.map(|doc|json!({
            "resource_path":doc.resource_path().as_str(),
            "script_instance_id":doc.script_instance_id().map(DecimalCounter::as_str),
            "editor_instance_id":doc.editor_instance_id().map(DecimalCounter::as_str),
            "buffer_instance_id":doc.buffer_instance_id().map(DecimalCounter::as_str)})),
        "source":src,
        "current":sample.sample.buffer.witness().and_then(Witness::source_version).map(DecimalCounter::as_str),
        "saved":saved.map(|s|s.saved_version.as_str()),
        "resource_edited":saved.map(|s|s.resource_edited)})
}
fn stamp(value: &CollectionStamp) -> Value {
    json!({"clock_id":match value.clock_id(){ClockId::Caller=>"caller".to_owned(),ClockId::Editor(s)=>format!("editor:{}",s.as_str())},
        "started_tick_us":value.started_tick_us().as_str(),
        "finished_tick_us":value.finished_tick_us().as_str(),
        "received_elapsed_us":value.received_elapsed_us()})
}
pub(super) fn detail(
    id: &RequestId,
    phase: &str,
    saved: Option<&script_edit::SavedStateEvidence>,
    mtime: Option<&DecimalCounter>,
) -> Value {
    json!({"v":1,"kind":"edit_detail","request_id":id.as_str(),"phase":phase,
        "saved":saved.map(|s|json!({"request_id":s.request_id.as_str(),"session_id":s.session_id.as_str(),
            "resource_path":s.document.resource_path().as_str(),
            "script_instance_id":s.document.script_instance_id().map(DecimalCounter::as_str),
            "editor_instance_id":s.document.editor_instance_id().map(DecimalCounter::as_str),
            "buffer_instance_id":s.document.buffer_instance_id().map(DecimalCounter::as_str),
            "current_version":s.current_version.as_str(),"saved_version":s.saved_version.as_str(),
            "resource_edited":s.resource_edited,"save_profile":hex(&s.save_profile),
            "original_preserved":s.original_preserved,"desired_preserved":s.desired_preserved,
            "collection":stamp(&s.collection)})),
        "mtime":mtime.map(DecimalCounter::as_str)})
}
pub(super) fn validation(
    id: &RequestId,
    purpose: &str,
    result: &stock_validation::ValidationResult,
    start: u64,
    finish: u64,
    context_current: bool,
    dependencies_current: bool,
) -> Value {
    json!({"v":1,"kind":"edit_validation","request_id":id.as_str(),"purpose":purpose,
        "result":result,"started_tick_us":start,"finished_tick_us":finish,
        "context_current":context_current,"dependencies_current":dependencies_current})
}
pub(super) fn context(
    id: &RequestId,
    started_tick_us: u64,
    finished_tick_us: u64,
    current: bool,
    context_sha256: &str,
    dependencies: &[FreshDependency],
) -> Value {
    json!({"v":1,"kind":"edit_context","request_id":id.as_str(),
        "started_tick_us":started_tick_us,"finished_tick_us":finished_tick_us,
        "current":current,"context_sha256":context_sha256,"dependencies":dependencies})
}
pub(super) fn progress(id: &RequestId, value: &wire::edit::AppliedEvent) -> Value {
    let n = &value.native;
    json!({"v":1,"kind":"edit_progress","request_id":id.as_str(),"stage":value.stage,
        "collection":stamp(&value.collection),
        "native":{"stage":n.stage,"status":n.status,"reason":format!("{:?}",n.reason),
            "expected_hash":hex(&n.expected_hash),"desired_hash":hex(&n.desired_hash),
            "project_device":n.project.device().as_str(),"project_inode":n.project.inode().as_str(),
            "file_device":n.file.device().as_str(),"file_inode":n.file.inode().as_str(),
            "prepared_current":n.prepared_current.as_str(),"prepared_saved":n.prepared_saved.as_str(),
            "frozen_current":n.frozen_current.as_ref().map(DecimalCounter::as_str),
            "buffer_changed":n.buffer_changed,"resource_changed":n.resource_changed,
            "write_started":n.write_started,"write_calls":n.write_calls,"written_bytes":n.written_bytes,
            "write_errno":n.write_errno,"truncated":n.truncated,"truncate_errno":n.truncate_errno,
            "flushed":n.flushed,"fsync_errno":n.fsync_errno,"readback":n.readback,
            "pread_errno":n.pread_errno,"restore_attempted":n.restore_attempted,
            "restore_errno":n.restore_errno,"restored":n.restored,
            "edited_attempted":n.edited_attempted,"edited_cleared":n.edited_cleared,
            "tag_attempted":n.tag_attempted,"tagged":n.tagged,"t0":n.t0.as_str(),
            "after_restore":n.after_restore.as_ref().map(DecimalCounter::as_str)},
        "before":value.before.as_ref().map(state),"after":value.after.as_ref().map(state)})
}
pub(super) fn terminal(
    id: &RequestId,
    reason: script_edit::Reason,
    preboundary: bool,
    collection: Option<&CollectionStamp>,
) -> Value {
    json!({"v":1,"kind":"edit_terminal","request_id":id.as_str(),
        "reason":format!("{:?}",reason),"terminal_before_boundary":preboundary,
        "collection":collection.map(stamp)})
}
fn number(value: String) -> Result<DecimalCounter, RoutingFailure> {
    if value.len() > 64 {
        return Err(protocol_failure());
    }
    DecimalCounter::new(value).map_err(|_| protocol_failure())
}
fn hash(value: &str) -> Result<[u8; 32], RoutingFailure> {
    if value.len() != 64 {
        return Err(protocol_failure());
    }
    let mut out = [0; 32];
    for (index, part) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        out[index] = digit(part[0])
            .zip(digit(part[1]))
            .map(|(a, b)| a << 4 | b)
            .ok_or_else(protocol_failure)?;
    }
    Ok(out)
}
fn source(value: HashIn) -> Result<script_edit::SourceDigest, RoutingFailure> {
    if value.utf8_bytes > SOURCE_LIMIT_BYTES {
        return Err(protocol_failure());
    }
    Ok(script_edit::SourceDigest {
        sha256: hash(&value.sha256)?,
        utf8_bytes: value.utf8_bytes,
    })
}
fn reason(value: &str) -> Result<script_edit::Reason, RoutingFailure> {
    use script_edit::Reason::*;
    Ok(match value {
        "Complete" => Complete,
        "Busy" => Busy,
        "DirtyConflict" => DirtyConflict,
        "RevisionMismatch" => RevisionMismatch,
        "IdentityChanged" => IdentityChanged,
        "SessionChanged" => SessionChanged,
        "Divergence" => Divergence,
        "KnownStaleResource" => KnownStaleResource,
        "KnownStaleBuffer" => KnownStaleBuffer,
        "UnavailableObservation" => UnavailableObservation,
        "ClosedTarget" => ClosedTarget,
        "UnsupportedTarget" => UnsupportedTarget,
        "AmbiguousTarget" => AmbiguousTarget,
        "DeniedAccess" => DeniedAccess,
        "UnsupportedEngine" => UnsupportedEngine,
        "UnsupportedEffect" => UnsupportedEffect,
        "UnsupportedRepresentation" => UnsupportedRepresentation,
        "SaveWouldReformat" => SaveWouldReformat,
        "PersistenceFailure" => PersistenceFailure,
        "PersistenceUnknown" => PersistenceUnknown,
        "PartialFinalization" => PartialFinalization,
        "ParseError" => ParseError,
        "DependencyError" => DependencyError,
        "ValidationUnavailable" => ValidationUnavailable,
        "EvidenceLimit" => EvidenceLimit,
        "Deadline" => Deadline,
        "Cancellation" => Cancellation,
        "Disconnection" => Disconnection,
        "ProtocolFailure" => ProtocolFailure,
        "MissingBasis" => MissingBasis,
        "IncompleteVerification" => IncompleteVerification,
        _ => return Err(protocol_failure()),
    })
}
impl DetailIn {
    pub(super) fn domain(
        self,
        request: &ObservationRequest,
        target: &ResolvedTarget,
        observed: &ObservationOutcome,
        receipt: u64,
    ) -> Result<
        (
            Option<script_edit::SavedStateEvidence>,
            Option<script_edit::DiskMetadata>,
        ),
        RoutingFailure,
    > {
        if self.v != 1
            || self.kind != "edit_detail"
            || self.request_id != request.request_id().as_str()
            || !matches!(
                self.phase.as_str(),
                "prepared" | "guard" | "post" | "verified"
            )
        {
            return Err(protocol_failure());
        }
        let document = observed.snapshot().and_then(|s| s.document().identity());
        let saved = self
            .saved
            .map(|s| {
                if s.request_id != request.request_id().as_str()
                    || s.session_id != target.session_id().as_str()
                    || s.resource_path != request.script_path().as_str()
                {
                    return Err(protocol_failure());
                }
                let doc = document.ok_or_else(protocol_failure)?;
                let ids = [
                    Some(number(s.script_instance_id)?),
                    Some(number(s.editor_instance_id)?),
                    Some(number(s.buffer_instance_id)?),
                ];
                let identity = DocumentIdentity::new(
                    ScriptKind::ExternalGdscript,
                    request.script_path().clone(),
                    ids[0].clone(),
                    ids[1].clone(),
                    ids[2].clone(),
                    doc.disk_file_id().cloned(),
                )
                .map_err(|_| protocol_failure())?;
                let stamp = s.collection.domain(target.session_id(), receipt)?;
                Ok(script_edit::SavedStateEvidence {
                    request_id: request.request_id().clone(),
                    session_id: target.session_id().clone(),
                    document: identity,
                    collection: stamp,
                    current_version: number(s.current_version)?,
                    saved_version: number(s.saved_version)?,
                    resource_edited: s.resource_edited,
                    save_profile: hash(&s.save_profile)?,
                    original_preserved: s.original_preserved,
                    desired_preserved: s.desired_preserved,
                })
            })
            .transpose()?;
        let disk = self
            .mtime
            .map(|mtime| {
                let source = observed
                    .snapshot()
                    .map(|s| s.sources().disk())
                    .ok_or_else(protocol_failure)?;
                Ok(script_edit::DiskMetadata {
                    identity: source
                        .witness()
                        .and_then(Witness::disk_file_id)
                        .cloned()
                        .ok_or_else(protocol_failure)?,
                    mtime: number(mtime)?,
                    collection: source.collection().cloned().ok_or_else(protocol_failure)?,
                })
            })
            .transpose()?;
        Ok((saved, disk))
    }
    pub(super) fn phase(&self) -> &str {
        &self.phase
    }
}
impl StampIn {
    fn domain(self, session: &SessionId, receipt: u64) -> Result<CollectionStamp, RoutingFailure> {
        if self.clock_id != format!("editor:{}", session.as_str())
            || self.received_elapsed_us > receipt
        {
            return Err(protocol_failure());
        }
        CollectionStamp::new(
            ClockId::Editor(session.clone()),
            number(self.started_tick_us)?,
            number(self.finished_tick_us)?,
            receipt,
        )
        .map_err(|_| protocol_failure())
    }
}
impl ValidationIn {
    pub(super) fn domain(
        self,
        request: &EditRequest,
        purpose: script_edit::ValidationPurpose,
        receipt: u64,
    ) -> Result<script_edit::ValidationResult, RoutingFailure> {
        if self.v != 1
            || self.kind != "edit_validation"
            || self.request_id != request.request_id().as_str()
            || self.purpose
                != match purpose {
                    script_edit::ValidationPurpose::Preflight => "preflight",
                    script_edit::ValidationPurpose::PostChange => "post_change",
                    script_edit::ValidationPurpose::Unchanged => "unchanged",
                }
            || self.result.request_id != request.request_id().as_str()
            || self.result.session_id != request.expected().target().session_id().as_str()
            || self.result.root_path != request.script_path().as_str()
            || self.started_tick_us > self.finished_tick_us
            || self.finished_tick_us > receipt
            || self.result.sources.len() > 33
            || self.result.diagnostics.len() > 64
        {
            return Err(protocol_failure());
        }
        let input = if purpose == script_edit::ValidationPurpose::Preflight {
            script_edit::SourceDigest::of(request.replacement_source().as_str())
        } else if let Some(root) = self.result.sources.first() {
            script_edit::SourceDigest {
                sha256: hash(&root.sha256)?,
                utf8_bytes: root.utf8_bytes,
            }
        } else if self.result.status == "unavailable" {
            script_edit::SourceDigest::of(request.replacement_source().as_str())
        } else {
            return Err(protocol_failure());
        };
        let mut fences = Vec::new();
        let mut deps = Vec::new();
        for (index, item) in self.result.sources.iter().enumerate() {
            let path = ResourcePath::new(item.path.clone()).map_err(|_| protocol_failure())?;
            let digest = script_edit::SourceDigest {
                sha256: hash(&item.sha256)?,
                utf8_bytes: item.utf8_bytes,
            };
            if item.utf8_bytes > SOURCE_LIMIT_BYTES {
                return Err(protocol_failure());
            }
            fences.push(script_edit::ValidationSourceFence {
                path: path.clone(),
                source: digest,
                diagnostics_completed: item.diagnostics_completed,
                symbols_completed: item.symbols_completed,
            });
            if index != 0 {
                deps.push(script_edit::DependencyWitness {
                    path,
                    identity: FileIdentity::new(
                        number(item.device.ok_or_else(protocol_failure)?.to_string())?,
                        number(item.inode.ok_or_else(protocol_failure)?.to_string())?,
                    ),
                    source: digest,
                });
            }
        }
        let status = match self.result.status.as_str() {
            "valid" => script_edit::ValidationStatus::Valid,
            "invalid" => script_edit::ValidationStatus::Invalid,
            "unavailable" => script_edit::ValidationStatus::Unavailable,
            _ => return Err(protocol_failure()),
        };
        let mut diagnostics = Vec::new();
        for item in self.result.diagnostics {
            if item.message.len() > 2048 {
                return Err(protocol_failure());
            }
            let path = ResourcePath::new(item.path).map_err(|_| protocol_failure())?;
            let digest = hash(&item.source_sha256)?;
            let source = fences
                .iter()
                .find(|s| s.path == path && s.source.sha256 == digest)
                .map(|f| f.source);
            if source.is_none() && status != script_edit::ValidationStatus::Unavailable {
                return Err(protocol_failure());
            }
            diagnostics.push(script_edit::ValidationDiagnostic {
                origin: if path == *request.script_path() {
                    script_edit::DiagnosticOrigin::Root
                } else {
                    script_edit::DiagnosticOrigin::Dependency
                },
                path: Some(path),
                path_reason: None,
                source,
                line: Some(item.line),
                column: Some(item.column),
                message: item.message,
            });
        }
        let context = self
            .result
            .context_sha256
            .as_deref()
            .map(hash)
            .transpose()?
            .unwrap_or([0; 32]);
        let reason = match status {
            script_edit::ValidationStatus::Invalid
                if diagnostics
                    .iter()
                    .any(|d| d.origin == script_edit::DiagnosticOrigin::Dependency) =>
            {
                Some(script_edit::Reason::DependencyError)
            }
            script_edit::ValidationStatus::Invalid => Some(script_edit::Reason::ParseError),
            script_edit::ValidationStatus::Unavailable => {
                Some(script_edit::Reason::ValidationUnavailable)
            }
            script_edit::ValidationStatus::Valid => None,
        };
        let stamp = CollectionStamp::new(
            ClockId::Caller,
            number(self.started_tick_us.to_string())?,
            number(self.finished_tick_us.to_string())?,
            receipt,
        )
        .map_err(|_| protocol_failure())?;
        Ok(script_edit::ValidationResult {
            status,
            reason,
            purpose,
            request_id: request.request_id().clone(),
            session_id: request.expected().target().session_id().clone(),
            document: request.expected().document().clone(),
            source_path: request.script_path().clone(),
            input,
            collection: stamp,
            dependencies: deps,
            context,
            context_current: self.context_current,
            dependencies_current: self.dependencies_current,
            sources: fences,
            cleanup_confirmed: self.result.cleanup_confirmed,
            diagnostics,
        })
    }
}
impl ProgressIn {
    pub(super) fn domain(
        self,
        request: &EditRequest,
        receipt: u64,
        expected_kind: &str,
    ) -> Result<(script_edit::NativeWitness, NativeState), RoutingFailure> {
        if self.v != 1
            || self.kind != expected_kind
            || self.request_id != request.request_id().as_str()
            || self.stage != self.native.stage
        {
            return Err(protocol_failure());
        }
        let sid = request.expected().target().session_id();
        let stamp = self.collection.domain(sid, receipt)?;
        let n = self.native;
        let expected = &request.expected().source().sha256;
        let intended = script_edit::SourceDigest::of(request.replacement_source().as_str());
        if hash(&n.expected_hash)? != *expected
            || hash(&n.desired_hash)? != intended.sha256
            || FileIdentity::new(number(n.project_device)?, number(n.project_inode)?)
                != *request.expected().target().project_file_id()
            || FileIdentity::new(number(n.file_device)?, number(n.file_inode)?)
                != *request
                    .expected()
                    .document()
                    .disk_file_id()
                    .ok_or_else(protocol_failure)?
            || number(n.prepared_current)? != *request.expected().current_version()
            || number(n.prepared_saved)? != *request.expected().current_version()
            || n.written_bytes > SOURCE_LIMIT_BYTES
            || n.written_bytes > 0 && n.write_calls == 0
            || !n.write_started
                && (n.write_calls != 0 || n.written_bytes != 0 || n.truncated || n.flushed)
        {
            return Err(protocol_failure());
        }
        let before = self
            .before
            .map(|v| v.domain(request.expected().document()))
            .transpose()?;
        let after = self
            .after
            .map(|v| v.domain(request.expected().document()))
            .transpose()?;
        Ok((
            script_edit::NativeWitness {
                request_id: request.request_id().clone(),
                session_id: sid.clone(),
                document: request.expected().document().clone(),
                collection: stamp,
            },
            NativeState {
                stage: self.stage,
                status: n.status,
                reason: reason(&n.reason)?,
                frozen_current: n.frozen_current.map(number).transpose()?,
                buffer_changed: n.buffer_changed,
                resource_changed: n.resource_changed,
                write_started: n.write_started,
                written_bytes: n.written_bytes,
                write_errno: n.write_errno,
                truncated: n.truncated,
                truncate_errno: n.truncate_errno,
                flushed: n.flushed,
                fsync_errno: n.fsync_errno,
                readback: n.readback,
                pread_errno: n.pread_errno,
                restore_attempted: n.restore_attempted,
                restore_errno: n.restore_errno,
                restored: n.restored,
                edited_attempted: n.edited_attempted,
                edited_cleared: n.edited_cleared,
                tag_attempted: n.tag_attempted,
                tagged: n.tagged,
                t0: number(n.t0)?,
                after_restore: n.after_restore.map(number).transpose()?,
                before,
                after,
            },
        ))
    }
}
#[derive(Clone)]
pub(super) struct ActualState {
    pub source: Option<script_edit::SourceDigest>,
    pub current: Option<DecimalCounter>,
    pub saved: Option<DecimalCounter>,
    pub edited: Option<bool>,
}
impl StateIn {
    fn domain(self, expected: &DocumentIdentity) -> Result<ActualState, RoutingFailure> {
        // Editor samples own R/B identity, not D identity. The native receipt and
        // independent disk reader check D separately; never invent it here.
        let bound = self
            .document
            .map(|doc| {
                let identity = DocumentIdentity::new(
                    ScriptKind::ExternalGdscript,
                    ResourcePath::new(doc.resource_path).map_err(|_| protocol_failure())?,
                    doc.script_instance_id.map(number).transpose()?,
                    doc.editor_instance_id.map(number).transpose()?,
                    doc.buffer_instance_id.map(number).transpose()?,
                    None,
                )
                .map_err(|_| protocol_failure())?;
                Ok::<bool, RoutingFailure>(
                    identity.kind() == expected.kind()
                        && identity.resource_path() == expected.resource_path()
                        && identity.script_instance_id() == expected.script_instance_id()
                        && identity.editor_instance_id() == expected.editor_instance_id()
                        && identity.buffer_instance_id() == expected.buffer_instance_id(),
                )
            })
            .transpose()?
            .unwrap_or(false);
        if !bound {
            return Ok(ActualState {
                source: None,
                current: None,
                saved: None,
                edited: None,
            });
        }
        Ok(ActualState {
            source: self.source.map(source).transpose()?,
            current: self.current.map(number).transpose()?,
            saved: self.saved.map(number).transpose()?,
            edited: self.resource_edited,
        })
    }
}
pub(super) struct NativeState {
    pub stage: String,
    pub status: String,
    pub reason: script_edit::Reason,
    pub frozen_current: Option<DecimalCounter>,
    pub buffer_changed: bool,
    pub resource_changed: bool,
    pub write_started: bool,
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
    pub before: Option<ActualState>,
    pub after: Option<ActualState>,
}
impl TerminalIn {
    pub(super) fn domain(
        self,
        id: &RequestId,
        session: &SessionId,
        receipt: u64,
    ) -> Result<(script_edit::Reason, bool, Option<CollectionStamp>), RoutingFailure> {
        if self.v != 1 || self.kind != "edit_terminal" || self.request_id != id.as_str() {
            return Err(protocol_failure());
        }
        Ok((
            reason(&self.reason)?,
            self.terminal_before_boundary,
            self.collection
                .map(|s| s.domain(session, receipt))
                .transpose()?,
        ))
    }
}
impl ContextIn {
    pub(super) fn domain(
        self,
        request: &EditRequest,
        receipt: u64,
    ) -> Result<script_edit::ContextRecheck, RoutingFailure> {
        if self.v != 1
            || self.kind != "edit_context"
            || self.request_id != request.request_id().as_str()
            || self.started_tick_us > self.finished_tick_us
            || self.finished_tick_us > receipt
        {
            return Err(protocol_failure());
        }
        let mut dependencies = Vec::with_capacity(self.dependencies.len());
        for item in self.dependencies {
            let path = ResourcePath::new(item.path).map_err(|_| protocol_failure())?;
            if path.kind() != Some(ScriptKind::ExternalGdscript)
                || dependencies
                    .iter()
                    .any(|d: &script_edit::DependencyWitness| d.path == path)
                || item.utf8_bytes > SOURCE_LIMIT_BYTES
            {
                return Err(protocol_failure());
            }
            dependencies.push(script_edit::DependencyWitness {
                path,
                identity: FileIdentity::new(number(item.device)?, number(item.inode)?),
                source: script_edit::SourceDigest {
                    sha256: hash(&item.sha256)?,
                    utf8_bytes: item.utf8_bytes,
                },
            });
        }
        Ok(script_edit::ContextRecheck {
            request_id: request.request_id().clone(),
            session_id: request.expected().target().session_id().clone(),
            document: request.expected().document().clone(),
            collection: CollectionStamp::new(
                ClockId::Caller,
                number(self.started_tick_us.to_string())?,
                number(self.finished_tick_us.to_string())?,
                receipt,
            )
            .map_err(|_| protocol_failure())?,
            context: hash(&self.context_sha256)?,
            dependencies,
            current: self.current,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_binding_preserves_facts_without_disk_identity_but_rejects_replaced_buffers() {
        let counter = |value: &str| DecimalCounter::new(value.to_owned()).unwrap();
        let expected = DocumentIdentity::new(
            ScriptKind::ExternalGdscript,
            ResourcePath::new("res://subject.gd").unwrap(),
            Some(counter("1")),
            Some(counter("2")),
            Some(counter("3")),
            Some(FileIdentity::new(counter("4"), counter("5"))),
        )
        .unwrap();
        let observed = script_edit::SourceDigest::of("new");
        let state = |buffer: &str| StateIn {
            document: Some(SampleBindingIn {
                resource_path: "res://subject.gd".into(),
                script_instance_id: Some("1".into()),
                editor_instance_id: Some("2".into()),
                buffer_instance_id: Some(buffer.into()),
            }),
            source: Some(HashIn {
                sha256: hex(&observed.sha256),
                utf8_bytes: observed.utf8_bytes,
            }),
            current: Some("12".into()),
            saved: Some("12".into()),
            resource_edited: Some(false),
        };
        let same_editor = state("3").domain(&expected).unwrap();
        assert_eq!(same_editor.source, Some(observed));
        assert_eq!(same_editor.current, Some(counter("12")));
        assert_eq!(same_editor.saved, Some(counter("12")));
        assert_eq!(same_editor.edited, Some(false));

        let replacement = state("9").domain(&expected).unwrap();
        assert!(replacement.source.is_none());
        assert!(replacement.current.is_none());
        assert!(replacement.saved.is_none());
        assert!(replacement.edited.is_none());
    }
}
