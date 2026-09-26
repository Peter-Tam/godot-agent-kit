//! Protocol-independent, read-only observation evidence and classification.
//!
//! An adapter supplies independently acquired facts. This module checks their shape,
//! attribution and collection clocks, applies detected invalidations, and computes a
//! terminal interpretation. It does not acquire facts, authenticate sessions, enforce
//! the deadline, or claim that an interval is an atomic editor snapshot.

use std::cmp::Ordering;
use std::fmt;

pub const SCHEMA_VERSION: u32 = 1;
pub const SOURCE_LIMIT_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidRequestId,
    InvalidSessionId,
    InvalidProjectRoot,
    InvalidResourcePath,
    InvalidDecimal,
    InvalidVersion,
    InvalidTiming,
    WrongClock,
    WrongTarget,
    WrongAuthority,
    InvalidDocument,
    InvalidAvailability,
    InvalidReason,
    InvalidStaleness,
    InvalidSelection,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid observation evidence: {self:?}")
    }
}
impl std::error::Error for EvidenceError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);
impl RequestId {
    /// # Errors
    /// Rejects empty, over-64-character, or non-ASCII-safe correlation IDs.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(EvidenceError::InvalidRequestId);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);
impl SessionId {
    /// # Errors
    /// Requires exactly 32 lowercase hexadecimal characters.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        if value.len() != 32
            || !value
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(EvidenceError::InvalidSessionId);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An exact, already-canonicalized absolute local root. The adapter must still establish
/// filesystem identity and confinement; lexical validation is not symlink verification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectRoot(String);
impl ProjectRoot {
    /// # Errors
    /// Rejects non-absolute or non-normal lexical paths and roots over 1024 UTF-8 bytes.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        if value.len() > 1024
            || !value.starts_with('/')
            || value.contains('\\')
            || value.chars().any(char::is_control)
            || (value != "/" && !valid_components(&value[1..], false))
        {
            return Err(EvidenceError::InvalidProjectRoot);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_components(value: &str, forbid_decorations: bool) -> bool {
    value.split('/').all(|part| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && !part.contains('\\')
            && !part.chars().any(char::is_control)
            && (!forbid_decorations || !part.bytes().any(|c| matches!(c, b'?' | b'#' | b':')))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
    ExternalGdscript,
    BuiltinGdscript,
}

/// An exact project-relative script locator, without case folding or Unicode normalization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourcePath {
    value: String,
    kind: Option<ScriptKind>,
}
impl ResourcePath {
    /// # Errors
    /// Rejects non-`res://`, non-normalized or decorated locators. A non-GDScript
    /// locator remains representable for an attributable `invalid_target` result.
    /// Symlink components and actual document type require adapter-side verification.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        let path = value
            .strip_prefix("res://")
            .ok_or(EvidenceError::InvalidResourcePath)?;
        let (container, kind) = if let Some((container, subresource)) = path.split_once("::") {
            if !subresource.starts_with("GDScript_")
                || subresource.len() == "GDScript_".len()
                || !subresource
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || ![".tscn", ".tres", ".scn", ".res"]
                    .iter()
                    .any(|suffix| container.ends_with(suffix))
            {
                return Err(EvidenceError::InvalidResourcePath);
            }
            (container, Some(ScriptKind::BuiltinGdscript))
        } else {
            (
                path,
                path.ends_with(".gd")
                    .then_some(ScriptKind::ExternalGdscript),
            )
        };
        if value.len() > 2048 || !valid_components(container, true) {
            return Err(EvidenceError::InvalidResourcePath);
        }
        Ok(Self { value, kind })
    }
    pub fn as_str(&self) -> &str {
        &self.value
    }
    /// Syntactic candidate kind, not proof of an existing GDScript.
    pub fn kind(&self) -> Option<ScriptKind> {
        self.kind
    }
}

/// Decimal digits stay decimal, including values beyond the precision of a JSON number.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecimalCounter(String);
impl DecimalCounter {
    /// # Errors
    /// Rejects signs, leading zeroes and non-decimal digits; no lossy integer conversion.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        if value.is_empty()
            || (value.len() > 1 && value.starts_with('0'))
            || !value.bytes().all(|c| c.is_ascii_digit())
        {
            return Err(EvidenceError::InvalidDecimal);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Ord for DecimalCounter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.cmp(&other.0))
    }
}
impl PartialOrd for DecimalCounter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileIdentity {
    device: DecimalCounter,
    inode: DecimalCounter,
}
impl FileIdentity {
    pub fn new(device: DecimalCounter, inode: DecimalCounter) -> Self {
        Self { device, inode }
    }
    pub fn device(&self) -> &DecimalCounter {
        &self.device
    }
    pub fn inode(&self) -> &DecimalCounter {
        &self.inode
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationRequest {
    request_id: RequestId,
    project_root: ProjectRoot,
    session_id: Option<SessionId>,
    script_path: ResourcePath,
}
impl ObservationRequest {
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
    ) -> Self {
        Self {
            request_id,
            project_root,
            session_id,
            script_path,
        }
    }
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn project_root(&self) -> &ProjectRoot {
        &self.project_root
    }
    pub fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    pub fn script_path(&self) -> &ResourcePath {
        &self.script_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineVersion {
    version: String,
    hash: String,
}
impl EngineVersion {
    /// # Errors
    /// Version/hash must be nonempty safe identifiers, never arbitrary diagnostic text.
    pub fn new(version: impl Into<String>, hash: impl Into<String>) -> Result<Self, EvidenceError> {
        let (version, hash) = (version.into(), hash.into());
        if ![&version, &hash].iter().all(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-' | b'+'))
        }) {
            return Err(EvidenceError::InvalidVersion);
        }
        Ok(Self { version, hash })
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    request_id: RequestId,
    project_root: ProjectRoot,
    project_file_id: FileIdentity,
    session_id: SessionId,
    godot_version: EngineVersion,
    script_path: ResourcePath,
}
impl ResolvedTarget {
    /// Build only after an adapter has proven a unique authenticated matching session.
    /// # Errors
    /// Refuses to bind a different project or a different explicitly selected session.
    pub fn for_request(
        request: &ObservationRequest,
        project_root: ProjectRoot,
        project_file_id: FileIdentity,
        session_id: SessionId,
        godot_version: EngineVersion,
    ) -> Result<Self, EvidenceError> {
        if project_root != request.project_root
            || request
                .session_id
                .as_ref()
                .is_some_and(|s| s != &session_id)
        {
            return Err(EvidenceError::WrongTarget);
        }
        Ok(Self {
            request_id: request.request_id.clone(),
            project_root,
            project_file_id,
            session_id,
            godot_version,
            script_path: request.script_path.clone(),
        })
    }
    pub(crate) fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn project_root(&self) -> &ProjectRoot {
        &self.project_root
    }
    pub fn project_file_id(&self) -> &FileIdentity {
        &self.project_file_id
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn godot_version(&self) -> &EngineVersion {
        &self.godot_version
    }
    pub fn script_path(&self) -> &ResourcePath {
        &self.script_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationInterval {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
impl ObservationInterval {
    /// Wall-clock endpoints may move backwards after a clock adjustment. Only the
    /// independently measured monotonic elapsed time bounds collection.
    pub fn new(started_unix_ms: u64, finished_unix_ms: u64, elapsed_us: u64) -> Self {
        Self {
            started_unix_ms,
            finished_unix_ms,
            elapsed_us,
        }
    }
    pub fn started_unix_ms(&self) -> u64 {
        self.started_unix_ms
    }
    pub fn finished_unix_ms(&self) -> u64 {
        self.finished_unix_ms
    }
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed_us
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClockId {
    Caller,
    Editor(SessionId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionStamp {
    clock_id: ClockId,
    started_tick_us: DecimalCounter,
    finished_tick_us: DecimalCounter,
    received_elapsed_us: u64,
}
impl CollectionStamp {
    /// # Errors
    /// Tick ordering compares only ticks on this one clock. Received time is checked
    /// separately against the enclosing attempt, never against an editor tick.
    pub fn new(
        clock_id: ClockId,
        started_tick_us: DecimalCounter,
        finished_tick_us: DecimalCounter,
        received_elapsed_us: u64,
    ) -> Result<Self, EvidenceError> {
        if started_tick_us > finished_tick_us {
            return Err(EvidenceError::InvalidTiming);
        }
        Ok(Self {
            clock_id,
            started_tick_us,
            finished_tick_us,
            received_elapsed_us,
        })
    }
    pub fn clock_id(&self) -> &ClockId {
        &self.clock_id
    }
    pub fn started_tick_us(&self) -> &DecimalCounter {
        &self.started_tick_us
    }
    pub fn finished_tick_us(&self) -> &DecimalCounter {
        &self.finished_tick_us
    }
    pub fn received_elapsed_us(&self) -> u64 {
        self.received_elapsed_us
    }
    fn validate(
        &self,
        target: &ResolvedTarget,
        interval: &ObservationInterval,
        editor: bool,
    ) -> Result<(), EvidenceError> {
        if self.received_elapsed_us > interval.elapsed_us {
            return Err(EvidenceError::InvalidTiming);
        }
        match (&self.clock_id, editor) {
            (ClockId::Caller, false) => Ok(()),
            (ClockId::Editor(id), true) if id == target.session_id() => Ok(()),
            _ => Err(EvidenceError::WrongClock),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentIdentity {
    kind: ScriptKind,
    resource_path: ResourcePath,
    script_instance_id: Option<DecimalCounter>,
    editor_instance_id: Option<DecimalCounter>,
    buffer_instance_id: Option<DecimalCounter>,
    disk_file_id: Option<FileIdentity>,
}
impl DocumentIdentity {
    /// # Errors
    /// A path must describe the declared kind; built-in script containers have no script D.
    pub fn new(
        kind: ScriptKind,
        resource_path: ResourcePath,
        script_instance_id: Option<DecimalCounter>,
        editor_instance_id: Option<DecimalCounter>,
        buffer_instance_id: Option<DecimalCounter>,
        disk_file_id: Option<FileIdentity>,
    ) -> Result<Self, EvidenceError> {
        if Some(kind) != resource_path.kind()
            || (kind == ScriptKind::BuiltinGdscript && disk_file_id.is_some())
        {
            return Err(EvidenceError::InvalidDocument);
        }
        Ok(Self {
            kind,
            resource_path,
            script_instance_id,
            editor_instance_id,
            buffer_instance_id,
            disk_file_id,
        })
    }
    pub fn kind(&self) -> ScriptKind {
        self.kind
    }
    pub fn resource_path(&self) -> &ResourcePath {
        &self.resource_path
    }
    pub fn script_instance_id(&self) -> Option<&DecimalCounter> {
        self.script_instance_id.as_ref()
    }
    pub fn editor_instance_id(&self) -> Option<&DecimalCounter> {
        self.editor_instance_id.as_ref()
    }
    pub fn buffer_instance_id(&self) -> Option<&DecimalCounter> {
        self.buffer_instance_id.as_ref()
    }
    pub fn disk_file_id(&self) -> Option<&FileIdentity> {
        self.disk_file_id.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authority {
    D,
    R,
    B,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Observed,
    Unavailable,
    NotApplicable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceReason {
    TooLarge,
    DiskMissing,
    DiskUnreadable,
    InvalidUtf8,
    NoStandaloneDiskSource,
    ResourceNotLoaded,
    ResourceUnreadable,
    BufferUnreadable,
    BufferAttributionUnavailable,
    OpenStateUnknown,
    IdentityChanged,
    SourceChanged,
    DocumentClosed,
    SessionEnded,
    DeadlineExceeded,
    UnsupportedCapability,
    DocumentNotOpen,
}
impl SourceReason {
    pub fn action(self) -> &'static str {
        use SourceReason::*;
        match self {
            TooLarge => "Inspect this source using a separately authorized bounded workflow",
            DiskMissing | DiskUnreadable | InvalidUtf8 => {
                "Check the selected standalone disk source"
            }
            NoStandaloneDiskSource => {
                "Inspect the already-loaded built-in script without substituting container text"
            }
            ResourceNotLoaded | ResourceUnreadable => {
                "Inspect already-loaded resource state without force-loading"
            }
            BufferUnreadable | BufferAttributionUnavailable | OpenStateUnknown => {
                "Establish document-specific live editor evidence"
            }
            IdentityChanged | SourceChanged | DocumentClosed | SessionEnded | DeadlineExceeded => {
                "Start a new observation of the intended editor and script"
            }
            UnsupportedCapability => "Use an explicitly supported observation capability",
            DocumentNotOpen => "Observe an open document to obtain buffer text",
        }
    }
    fn applicable(self, authority: Authority) -> bool {
        use SourceReason::*;
        match self {
            DiskMissing | DiskUnreadable | InvalidUtf8 | NoStandaloneDiskSource => {
                authority == Authority::D
            }
            ResourceNotLoaded | ResourceUnreadable => authority == Authority::R,
            BufferUnreadable
            | BufferAttributionUnavailable
            | OpenStateUnknown
            | DocumentNotOpen
            | DocumentClosed => authority == Authority::B,
            SessionEnded => authority != Authority::D,
            _ => true,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyReason {
    DirtyAttributionUnavailable,
    UnsupportedCapability,
    OpenStateUnknown,
    IdentityChanged,
    SourceChanged,
    DocumentClosed,
    SessionEnded,
    DeadlineExceeded,
    DocumentNotOpen,
}
impl DirtyReason {
    pub fn action(self) -> &'static str {
        use DirtyReason::*;
        match self {
            DirtyAttributionUnavailable | OpenStateUnknown => {
                "Establish document-specific editor unsaved-work evidence"
            }
            UnsupportedCapability => "Use an editor with attributable dirty-state support",
            IdentityChanged | SourceChanged | DocumentClosed | SessionEnded | DeadlineExceeded => {
                "Start a new observation of the intended document"
            }
            DocumentNotOpen => "Observe an open document to obtain buffer dirty state",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactReason {
    Unavailable,
    OpenStateUnknown,
    IdentityChanged,
    SourceChanged,
    DocumentClosed,
    SessionEnded,
    DeadlineExceeded,
}
impl FactReason {
    pub fn action(self) -> &'static str {
        match self {
            Self::Unavailable | Self::OpenStateUnknown => {
                "Establish the requested document identity and open state"
            }
            Self::IdentityChanged
            | Self::SourceChanged
            | Self::DocumentClosed
            | Self::SessionEnded
            | Self::DeadlineExceeded => "Start a new observation of the intended document",
        }
    }
}

/// Independent evidence that a source has not incorporated an identified change in
/// another authority. Both versions belong to the referenced authority's sequence:
/// the adapter must establish the last incorporated version, not compare unrelated
/// D/R/B version counters or infer lag from text disagreement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StalenessEvidence {
    incorporated_version: DecimalCounter,
    changed_authority: Authority,
    changed_version: DecimalCounter,
    collection: CollectionStamp,
    witness: Witness,
}
impl StalenessEvidence {
    /// # Errors
    /// Requires an independently identified newer revision of the referenced authority,
    /// with a witness naming that exact revision.
    pub fn unapplied_change(
        incorporated_version: DecimalCounter,
        changed_authority: Authority,
        changed_version: DecimalCounter,
        collection: CollectionStamp,
        witness: Witness,
    ) -> Result<Self, EvidenceError> {
        if incorporated_version >= changed_version
            || witness.source_version() != Some(&changed_version)
        {
            return Err(EvidenceError::InvalidStaleness);
        }
        Ok(Self {
            incorporated_version,
            changed_authority,
            changed_version,
            collection,
            witness,
        })
    }
    pub fn incorporated_version(&self) -> &DecimalCounter {
        &self.incorporated_version
    }
    pub fn changed_authority(&self) -> Authority {
        self.changed_authority
    }
    pub fn changed_version(&self) -> &DecimalCounter {
        &self.changed_version
    }
    pub fn collection(&self) -> &CollectionStamp {
        &self.collection
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staleness(Option<Vec<StalenessEvidence>>);
impl Staleness {
    pub fn unknown() -> Self {
        Self(None)
    }
    /// # Errors
    /// Empty evidence cannot establish known staleness.
    pub fn known_stale(evidence: Vec<StalenessEvidence>) -> Result<Self, EvidenceError> {
        if evidence.is_empty() {
            return Err(EvidenceError::InvalidStaleness);
        }
        Ok(Self(Some(evidence)))
    }
    pub fn evidence(&self) -> Option<&[StalenessEvidence]> {
        self.0.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    resource_path: ResourcePath,
    script_instance_id: Option<DecimalCounter>,
    editor_instance_id: Option<DecimalCounter>,
    buffer_instance_id: Option<DecimalCounter>,
    disk_file_id: Option<FileIdentity>,
    source_version: Option<DecimalCounter>,
}
impl Witness {
    pub fn new(
        resource_path: ResourcePath,
        script_instance_id: Option<DecimalCounter>,
        editor_instance_id: Option<DecimalCounter>,
        buffer_instance_id: Option<DecimalCounter>,
        disk_file_id: Option<FileIdentity>,
        source_version: Option<DecimalCounter>,
    ) -> Self {
        Self {
            resource_path,
            script_instance_id,
            editor_instance_id,
            buffer_instance_id,
            disk_file_id,
            source_version,
        }
    }
    pub fn resource_path(&self) -> &ResourcePath {
        &self.resource_path
    }
    pub fn script_instance_id(&self) -> Option<&DecimalCounter> {
        self.script_instance_id.as_ref()
    }
    pub fn editor_instance_id(&self) -> Option<&DecimalCounter> {
        self.editor_instance_id.as_ref()
    }
    pub fn buffer_instance_id(&self) -> Option<&DecimalCounter> {
        self.buffer_instance_id.as_ref()
    }
    pub fn disk_file_id(&self) -> Option<&FileIdentity> {
        self.disk_file_id.as_ref()
    }
    pub fn source_version(&self) -> Option<&DecimalCounter> {
        self.source_version.as_ref()
    }
    fn validate(
        &self,
        identity: &DocumentIdentity,
        authority: Authority,
    ) -> Result<(), EvidenceError> {
        if self.resource_path != identity.resource_path
            || (authority == Authority::D
                && (self.disk_file_id != identity.disk_file_id
                    || self.script_instance_id.is_some()
                    || self.editor_instance_id.is_some()
                    || self.buffer_instance_id.is_some()))
            || (authority == Authority::R
                && (self.script_instance_id != identity.script_instance_id
                    || self.disk_file_id.is_some()
                    || self.editor_instance_id.is_some()
                    || self.buffer_instance_id.is_some()))
            || (authority == Authority::B
                && (self.editor_instance_id != identity.editor_instance_id
                    || self.buffer_instance_id != identity.buffer_instance_id
                    || self.disk_file_id.is_some()))
            || (authority == Authority::B
                && self
                    .script_instance_id
                    .as_ref()
                    .zip(identity.script_instance_id.as_ref())
                    .is_some_and(|(a, b)| a != b))
        {
            return Err(EvidenceError::WrongTarget);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedSource {
    text: String,
    collection: CollectionStamp,
    witness: Witness,
    staleness: Staleness,
    reason: SourceReason,
}
impl InvalidatedSource {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn collection(&self) -> &CollectionStamp {
        &self.collection
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
    pub fn staleness(&self) -> &Staleness {
        &self.staleness
    }
    pub fn reason(&self) -> SourceReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceObservation {
    authority: Authority,
    state: SourceState,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceState {
    Observed {
        text: String,
        collection: CollectionStamp,
        witness: Witness,
        staleness: Staleness,
    },
    Unavailable {
        reason: SourceReason,
        invalidated: Option<InvalidatedSource>,
    },
    NotApplicable,
}
impl SourceObservation {
    /// Consumes exact UTF-8, without normalization or a copy. Over-limit text is discarded
    /// for this authority only and becomes `unavailable/too_large`, not a whole-attempt error.
    pub fn observed(
        authority: Authority,
        text: String,
        collection: CollectionStamp,
        witness: Witness,
        staleness: Staleness,
    ) -> Self {
        if text.len() > SOURCE_LIMIT_BYTES {
            return Self {
                authority,
                state: SourceState::Unavailable {
                    reason: SourceReason::TooLarge,
                    invalidated: None,
                },
            };
        }
        Self {
            authority,
            state: SourceState::Observed {
                text,
                collection,
                witness,
                staleness,
            },
        }
    }
    /// # Errors
    /// A reason must apply to this surface; `document_not_open` is reserved for B's
    /// not-applicable state, never an unavailable result.
    pub fn unavailable(authority: Authority, reason: SourceReason) -> Result<Self, EvidenceError> {
        if !reason.applicable(authority) || reason == SourceReason::DocumentNotOpen {
            return Err(EvidenceError::InvalidReason);
        }
        Ok(Self {
            authority,
            state: SourceState::Unavailable {
                reason,
                invalidated: None,
            },
        })
    }
    pub fn closed_buffer() -> Self {
        Self {
            authority: Authority::B,
            state: SourceState::NotApplicable,
        }
    }
    pub fn authority(&self) -> Authority {
        self.authority
    }
    pub fn availability(&self) -> Availability {
        match self.state {
            SourceState::Observed { .. } => Availability::Observed,
            SourceState::Unavailable { .. } => Availability::Unavailable,
            SourceState::NotApplicable => Availability::NotApplicable,
        }
    }
    pub fn text(&self) -> Option<&str> {
        match &self.state {
            SourceState::Observed { text, .. } => Some(text),
            _ => None,
        }
    }
    pub fn collection(&self) -> Option<&CollectionStamp> {
        match &self.state {
            SourceState::Observed { collection, .. } => Some(collection),
            _ => None,
        }
    }
    pub fn witness(&self) -> Option<&Witness> {
        match &self.state {
            SourceState::Observed { witness, .. } => Some(witness),
            _ => None,
        }
    }
    pub fn staleness(&self) -> Option<&Staleness> {
        match &self.state {
            SourceState::Observed { staleness, .. } => Some(staleness),
            _ => None,
        }
    }
    pub fn reason(&self) -> Option<SourceReason> {
        match &self.state {
            SourceState::Unavailable { reason, .. } => Some(*reason),
            SourceState::NotApplicable => Some(SourceReason::DocumentNotOpen),
            _ => None,
        }
    }
    pub fn invalidated_evidence(&self) -> Option<&InvalidatedSource> {
        match &self.state {
            SourceState::Unavailable { invalidated, .. } => invalidated.as_ref(),
            _ => None,
        }
    }
    /// Move formerly observed text into explicitly invalidated evidence. Never changes
    /// a separate authority's current text or loses a previous invalidated value.
    ///
    /// # Errors
    /// Rejects reasons belonging to another authority or reserved for inapplicability.
    /// Only identity/session loss can invalidate a previously not-applicable buffer.
    pub fn invalidate(&mut self, reason: SourceReason) -> Result<(), EvidenceError> {
        if !reason.applicable(self.authority) || reason == SourceReason::DocumentNotOpen {
            return Err(EvidenceError::InvalidReason);
        }
        if matches!(&self.state, SourceState::NotApplicable)
            && !matches!(
                reason,
                SourceReason::IdentityChanged | SourceReason::SessionEnded
            )
        {
            return Err(EvidenceError::InvalidAvailability);
        }
        let old = std::mem::replace(
            &mut self.state,
            SourceState::Unavailable {
                reason,
                invalidated: None,
            },
        );
        self.state = match old {
            SourceState::Observed {
                text,
                collection,
                witness,
                staleness,
            } => SourceState::Unavailable {
                reason,
                invalidated: Some(InvalidatedSource {
                    text,
                    collection,
                    witness,
                    staleness,
                    reason,
                }),
            },
            SourceState::Unavailable { invalidated, .. } => SourceState::Unavailable {
                reason,
                invalidated,
            },
            SourceState::NotApplicable => SourceState::Unavailable {
                reason,
                invalidated: None,
            },
        };
        Ok(())
    }
    fn validate(
        &self,
        target: &ResolvedTarget,
        identity: &DocumentIdentity,
        interval: &ObservationInterval,
    ) -> Result<(), EvidenceError> {
        let check = |collection: &CollectionStamp,
                     witness: &Witness,
                     staleness: &Staleness|
         -> Result<(), EvidenceError> {
            collection.validate(target, interval, self.authority != Authority::D)?;
            witness.validate(identity, self.authority)?;
            if let Some(evidence) = staleness.evidence() {
                for change in evidence {
                    if change.changed_authority == self.authority {
                        return Err(EvidenceError::InvalidStaleness);
                    }
                    change.collection.validate(
                        target,
                        interval,
                        change.changed_authority != Authority::D,
                    )?;
                    change
                        .witness
                        .validate(identity, change.changed_authority)?;
                }
            }
            Ok(())
        };
        match &self.state {
            SourceState::Observed {
                collection,
                witness,
                staleness,
                ..
            } => check(collection, witness, staleness),
            SourceState::Unavailable {
                invalidated: Some(old),
                ..
            } => check(&old.collection, &old.witness, &old.staleness),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sources {
    disk: SourceObservation,
    resource: SourceObservation,
    buffer: SourceObservation,
}
impl Sources {
    /// # Errors
    /// Every authority must occur in its own slot exactly once.
    pub fn new(
        disk: SourceObservation,
        resource: SourceObservation,
        buffer: SourceObservation,
    ) -> Result<Self, EvidenceError> {
        if disk.authority != Authority::D
            || resource.authority != Authority::R
            || buffer.authority != Authority::B
        {
            return Err(EvidenceError::WrongAuthority);
        }
        Ok(Self {
            disk,
            resource,
            buffer,
        })
    }
    pub fn disk(&self) -> &SourceObservation {
        &self.disk
    }
    pub fn resource(&self) -> &SourceObservation {
        &self.resource
    }
    pub fn buffer(&self) -> &SourceObservation {
        &self.buffer
    }
    fn get_mut(&mut self, authority: Authority) -> &mut SourceObservation {
        match authority {
            Authority::D => &mut self.disk,
            Authority::R => &mut self.resource,
            Authority::B => &mut self.buffer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validity {
    Valid,
    Missing,
    Invalid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenState {
    Open,
    NotOpen,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedFact<T> {
    value: T,
    collection: CollectionStamp,
    reason: FactReason,
}
impl<T> InvalidatedFact<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn collection(&self) -> &CollectionStamp {
        &self.collection
    }
    pub fn reason(&self) -> FactReason {
        self.reason
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentFact<T> {
    Observed {
        value: T,
        collection: CollectionStamp,
    },
    Unknown {
        reason: FactReason,
        invalidated: Option<InvalidatedFact<T>>,
    },
}
impl<T> DocumentFact<T> {
    pub fn observed(value: T, collection: CollectionStamp) -> Self {
        Self::Observed { value, collection }
    }
    pub fn unknown(reason: FactReason) -> Self {
        Self::Unknown {
            reason,
            invalidated: None,
        }
    }
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Observed { value, .. } => Some(value),
            Self::Unknown { .. } => None,
        }
    }
    pub fn collection(&self) -> Option<&CollectionStamp> {
        match self {
            Self::Observed { collection, .. } => Some(collection),
            _ => None,
        }
    }
    pub fn reason(&self) -> Option<FactReason> {
        match self {
            Self::Unknown { reason, .. } => Some(*reason),
            _ => None,
        }
    }
    pub fn invalidated_evidence(&self) -> Option<&InvalidatedFact<T>> {
        match self {
            Self::Unknown { invalidated, .. } => invalidated.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn invalidate(&mut self, reason: FactReason) {
        let old = std::mem::replace(
            self,
            Self::Unknown {
                reason,
                invalidated: None,
            },
        );
        *self = match old {
            Self::Observed { value, collection } => Self::Unknown {
                reason,
                invalidated: Some(InvalidatedFact {
                    value,
                    collection,
                    reason,
                }),
            },
            Self::Unknown { invalidated, .. } => Self::Unknown {
                reason,
                invalidated,
            },
        };
    }
    fn validate(
        &self,
        target: &ResolvedTarget,
        interval: &ObservationInterval,
        editor_only: bool,
    ) -> Result<(), EvidenceError> {
        let check = |collection: &CollectionStamp| {
            let editor = editor_only || matches!(collection.clock_id(), ClockId::Editor(_));
            collection.validate(target, interval, editor)
        };
        match self {
            Self::Observed { collection, .. } => check(collection),
            Self::Unknown {
                invalidated: Some(old),
                ..
            } => check(&old.collection),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentState {
    identity: Option<DocumentIdentity>,
    validity: DocumentFact<Validity>,
    open_state: DocumentFact<OpenState>,
}
impl DocumentState {
    pub fn new(
        identity: Option<DocumentIdentity>,
        validity: DocumentFact<Validity>,
        open_state: DocumentFact<OpenState>,
    ) -> Self {
        Self {
            identity,
            validity,
            open_state,
        }
    }
    pub fn identity(&self) -> Option<&DocumentIdentity> {
        self.identity.as_ref()
    }
    pub fn validity(&self) -> &DocumentFact<Validity> {
        &self.validity
    }
    pub fn open_state(&self) -> &DocumentFact<OpenState> {
        &self.open_state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyState {
    Dirty,
    Clean,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedDirty {
    state: DirtyState,
    collection: CollectionStamp,
    witness: Witness,
    reason: DirtyReason,
}
impl InvalidatedDirty {
    pub fn state(&self) -> DirtyState {
        self.state
    }
    pub fn collection(&self) -> &CollectionStamp {
        &self.collection
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
    pub fn reason(&self) -> DirtyReason {
        self.reason
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirtyObservation {
    Observed {
        state: DirtyState,
        collection: CollectionStamp,
        witness: Witness,
    },
    Unavailable {
        reason: DirtyReason,
        invalidated: Option<InvalidatedDirty>,
    },
    NotApplicable,
}
impl DirtyObservation {
    pub fn observed(state: DirtyState, collection: CollectionStamp, witness: Witness) -> Self {
        Self::Observed {
            state,
            collection,
            witness,
        }
    }
    /// # Errors
    /// Closed-document inapplicability is not an unavailable dirty state.
    pub fn unavailable(reason: DirtyReason) -> Result<Self, EvidenceError> {
        if reason == DirtyReason::DocumentNotOpen {
            return Err(EvidenceError::InvalidReason);
        }
        Ok(Self::Unavailable {
            reason,
            invalidated: None,
        })
    }
    pub fn closed_document() -> Self {
        Self::NotApplicable
    }
    pub fn availability(&self) -> Availability {
        match self {
            Self::Observed { .. } => Availability::Observed,
            Self::Unavailable { .. } => Availability::Unavailable,
            Self::NotApplicable => Availability::NotApplicable,
        }
    }
    pub fn state(&self) -> Option<DirtyState> {
        match self {
            Self::Observed { state, .. } => Some(*state),
            _ => None,
        }
    }
    pub fn collection(&self) -> Option<&CollectionStamp> {
        match self {
            Self::Observed { collection, .. } => Some(collection),
            _ => None,
        }
    }
    pub fn witness(&self) -> Option<&Witness> {
        match self {
            Self::Observed { witness, .. } => Some(witness),
            _ => None,
        }
    }
    pub fn reason(&self) -> Option<DirtyReason> {
        match self {
            Self::Unavailable { reason, .. } => Some(*reason),
            Self::NotApplicable => Some(DirtyReason::DocumentNotOpen),
            _ => None,
        }
    }
    pub fn invalidated_evidence(&self) -> Option<&InvalidatedDirty> {
        match self {
            Self::Unavailable { invalidated, .. } => invalidated.as_ref(),
            _ => None,
        }
    }
    /// Preserve an earlier dirty observation as explicitly invalidated evidence.
    ///
    /// # Errors
    /// Rejects `DocumentNotOpen` as an unavailable reason. A previously not-applicable
    /// dirty state can be invalidated only by identity/session loss.
    pub fn invalidate(&mut self, reason: DirtyReason) -> Result<(), EvidenceError> {
        if reason == DirtyReason::DocumentNotOpen {
            return Err(EvidenceError::InvalidReason);
        }
        if matches!(self, Self::NotApplicable)
            && !matches!(
                reason,
                DirtyReason::IdentityChanged | DirtyReason::SessionEnded
            )
        {
            return Err(EvidenceError::InvalidAvailability);
        }
        let old = std::mem::replace(
            self,
            Self::Unavailable {
                reason,
                invalidated: None,
            },
        );
        *self = match old {
            Self::Observed {
                state,
                collection,
                witness,
            } => Self::Unavailable {
                reason,
                invalidated: Some(InvalidatedDirty {
                    state,
                    collection,
                    witness,
                    reason,
                }),
            },
            Self::Unavailable { invalidated, .. } => Self::Unavailable {
                reason,
                invalidated,
            },
            Self::NotApplicable => Self::Unavailable {
                reason,
                invalidated: None,
            },
        };
        Ok(())
    }
    fn validate(
        &self,
        target: &ResolvedTarget,
        identity: Option<&DocumentIdentity>,
        interval: &ObservationInterval,
    ) -> Result<(), EvidenceError> {
        if matches!(
            self,
            Self::Unavailable {
                reason: DirtyReason::DocumentNotOpen,
                ..
            }
        ) {
            return Err(EvidenceError::InvalidReason);
        }
        match self {
            Self::Observed {
                collection,
                witness,
                ..
            } => {
                collection.validate(target, interval, true)?;
                witness.validate(
                    identity.ok_or(EvidenceError::InvalidDocument)?,
                    Authority::B,
                )
            }
            Self::Unavailable {
                invalidated: Some(old),
                ..
            } => {
                old.collection.validate(target, interval, true)?;
                old.witness.validate(
                    identity.ok_or(EvidenceError::InvalidDocument)?,
                    Authority::B,
                )
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedChange {
    Source(Authority),
    Dirty,
    DocumentClosed,
    DocumentIdentityReplaced,
    SessionReplaced,
    SessionEnded,
    DiskIdentityReplaced,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecheckReason {
    Unavailable,
    DeadlineExceeded,
    SessionEnded,
    UnsupportedCapability,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recheck {
    Performed {
        detected_changes: Vec<DetectedChange>,
    },
    Unavailable {
        reason: RecheckReason,
    },
    /// Some independent checks found changes before another required check failed.
    Partial {
        reason: RecheckReason,
        detected_changes: Vec<DetectedChange>,
    },
}
impl Recheck {
    pub fn performed(detected_changes: Vec<DetectedChange>) -> Self {
        Self::Performed { detected_changes }
    }
    pub fn unavailable(reason: RecheckReason) -> Self {
        Self::Unavailable { reason }
    }
    pub fn partial(reason: RecheckReason, detected_changes: Vec<DetectedChange>) -> Self {
        Self::Partial {
            reason,
            detected_changes,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    Unknown,
    Changed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checks {
    Performed,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consistency {
    checks: Checks,
    stability: Stability,
    detected_changes: Vec<DetectedChange>,
    recheck_reason: Option<RecheckReason>,
}
impl Consistency {
    pub fn atomic(&self) -> bool {
        false
    }
    pub fn checks(&self) -> Checks {
        self.checks
    }
    pub fn stability(&self) -> Stability {
        self.stability
    }
    pub fn detected_changes(&self) -> &[DetectedChange] {
        &self.detected_changes
    }
    pub fn recheck_reason(&self) -> Option<RecheckReason> {
        self.recheck_reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    Different,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agreement {
    Agree,
    Divergent,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comparisons {
    disk_resource: Comparison,
    disk_buffer: Comparison,
    resource_buffer: Comparison,
}
impl Comparisons {
    pub fn disk_resource(&self) -> Comparison {
        self.disk_resource
    }
    pub fn disk_buffer(&self) -> Comparison {
        self.disk_buffer
    }
    pub fn resource_buffer(&self) -> Comparison {
        self.resource_buffer
    }
}
fn compare(a: &SourceObservation, b: &SourceObservation) -> Comparison {
    match (a.text(), b.text()) {
        (Some(left), Some(right)) if left == right => Comparison::Equal,
        (Some(_), Some(_)) => Comparison::Different,
        _ => Comparison::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    ValidateRequest,
    ResolveTarget,
    Authenticate,
    AttributeDocument,
    ReadDisk,
    ReadEditor,
    Recheck,
    Finalize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    D,
    R,
    B,
    Dirty,
    Document,
    Session,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    DirtyAttributionUnavailable,
    BufferAttributionUnavailable,
    BufferUnreadable,
    ResourceNotLoaded,
    ResourceUnreadable,
    DiskMissing,
    DiskUnreadable,
    InvalidUtf8,
    TooLarge,
    NoStandaloneDiskSource,
    IdentityChanged,
    SourceChanged,
    DocumentClosed,
    OpenStateUnknown,
    DocumentStateUnavailable,
    UnsupportedVersion,
    UnsupportedCapability,
    SessionEnded,
    DeadlineExceeded,
    OutOfProject,
    UnsafeRegistry,
    AuthenticationFailed,
    InvalidFrame,
    Cancelled,
    AmbiguousTarget,
    MissingTarget,
    InvalidTarget,
    EditorUnavailable,
    UnsupportedObservation,
    DeniedAccess,
    InvalidRequest,
    ProtocolError,
    RecheckUnavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    stage: Stage,
    surface: Option<Surface>,
}
impl Diagnostic {
    pub fn new(code: DiagnosticCode, stage: Stage, surface: Option<Surface>) -> Self {
        Self {
            code,
            stage,
            surface,
        }
    }
    pub fn code(&self) -> DiagnosticCode {
        self.code
    }
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn surface(&self) -> Option<Surface> {
        self.surface
    }
    /// Fixed safe metadata; never interpolates source, paths, secrets or malformed payloads.
    pub fn message(&self) -> &'static str {
        use DiagnosticCode::*;
        match self.code {
            DirtyAttributionUnavailable => "Document-specific dirty state was not established",
            BufferAttributionUnavailable => "Buffer attribution was not established",
            BufferUnreadable => "Editor buffer could not be read",
            ResourceNotLoaded => "Script resource was not already loaded",
            ResourceUnreadable => "Loaded script resource could not be read",
            DiskMissing => "Standalone disk source was not found",
            DiskUnreadable => "Standalone disk source could not be read",
            InvalidUtf8 => "Source was not valid UTF-8",
            TooLarge => "Source exceeded its independent observation limit",
            NoStandaloneDiskSource => "No attributable standalone disk source exists",
            IdentityChanged => "Observed identity changed during collection",
            SourceChanged => "Observed source changed during collection",
            DocumentClosed => "Document closed during collection",
            OpenStateUnknown => "Document open state was not established",
            DocumentStateUnavailable => "Document state was not established",
            UnsupportedVersion => "This observation capability is unsupported",
            UnsupportedCapability => "Required surface capability is unsupported",
            SessionEnded => "Selected editor session ended",
            DeadlineExceeded => "Observation deadline expired",
            OutOfProject => "Target is outside the selected project",
            UnsafeRegistry => "Private registry is unsafe",
            AuthenticationFailed => "Session authentication failed",
            InvalidFrame => "Bridge response was invalid",
            Cancelled => "Observation was cancelled",
            AmbiguousTarget => "Multiple intended editor sessions match",
            MissingTarget => "Requested script was not found",
            InvalidTarget => "Requested target is not a GDScript",
            EditorUnavailable => "Requested editor session is unavailable",
            UnsupportedObservation => "Observation cannot be performed safely",
            DeniedAccess => "Access to target was denied",
            InvalidRequest => "Observation request is invalid",
            ProtocolError => "Observation protocol failed",
            RecheckUnavailable => "Required post-collection checks were unavailable",
        }
    }
    pub fn action(&self) -> &'static str {
        use DiagnosticCode::*;
        match self.code {
            AmbiguousTarget => "Specify the exact editor session",
            DirtyAttributionUnavailable | BufferAttributionUnavailable | OpenStateUnknown => {
                "Obtain document-specific editor evidence"
            }
            TooLarge => "Inspect this source through a separately authorized bounded workflow",
            SessionEnded | EditorUnavailable | DeadlineExceeded => {
                "Start a new observation after checking the intended editor"
            }
            AuthenticationFailed | UnsafeRegistry | DeniedAccess | OutOfProject => {
                "Check access and explicit target selection"
            }
            _ => "Inspect the affected stage and authority before a new observation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationEvidence {
    target: ResolvedTarget,
    document: DocumentState,
    sources: Sources,
    dirty: DirtyObservation,
    recheck: Recheck,
}
impl ObservationEvidence {
    pub fn new(
        target: ResolvedTarget,
        document: DocumentState,
        sources: Sources,
        dirty: DirtyObservation,
        recheck: Recheck,
    ) -> Self {
        Self {
            target,
            document,
            sources,
            dirty,
            recheck,
        }
    }
    fn terminal_failure(&self) -> Option<TerminalFailure> {
        let session_ended = matches!(
            &self.recheck,
            Recheck::Unavailable {
                reason: RecheckReason::SessionEnded
            } | Recheck::Partial {
                reason: RecheckReason::SessionEnded,
                ..
            }
        ) || matches!(&self.recheck,
                Recheck::Performed { detected_changes } | Recheck::Partial { detected_changes, .. }
                if detected_changes.iter().any(|change| matches!(change, DetectedChange::SessionEnded | DetectedChange::SessionReplaced)))
            || self.document.open_state.reason() == Some(FactReason::SessionEnded)
            || self.sources.resource.reason() == Some(SourceReason::SessionEnded)
            || self.sources.buffer.reason() == Some(SourceReason::SessionEnded)
            || self.dirty.reason() == Some(DirtyReason::SessionEnded);
        if session_ended {
            Some(TerminalFailure::DisconnectedEditor)
        } else if matches!(
            self.recheck,
            Recheck::Unavailable {
                reason: RecheckReason::DeadlineExceeded
            } | Recheck::Partial {
                reason: RecheckReason::DeadlineExceeded,
                ..
            }
        ) {
            Some(TerminalFailure::Timeout)
        } else {
            None
        }
    }
    fn finish(
        mut self,
        interval: &ObservationInterval,
        disconnected: bool,
    ) -> Result<ObservationSnapshot, EvidenceError> {
        let target = &self.target;
        if self
            .document
            .identity
            .as_ref()
            .is_some_and(|i| i.resource_path() != target.script_path())
        {
            return Err(EvidenceError::WrongTarget);
        }
        self.document.validity.validate(target, interval, false)?;
        self.document.open_state.validate(target, interval, true)?;
        self.dirty
            .validate(target, self.document.identity.as_ref(), interval)?;
        if let Some(identity) = self.document.identity.as_ref() {
            for source in [
                &self.sources.disk,
                &self.sources.resource,
                &self.sources.buffer,
            ] {
                source.validate(target, identity, interval)?;
            }
        } else if [
            &self.sources.disk,
            &self.sources.resource,
            &self.sources.buffer,
        ]
        .iter()
        .any(|s| s.availability() == Availability::Observed || s.invalidated_evidence().is_some())
            || self.dirty.availability() == Availability::Observed
            || self.dirty.invalidated_evidence().is_some()
        {
            return Err(EvidenceError::InvalidDocument);
        }
        // Disk disappearance alone does not make an independently observed open document missing.
        if matches!(
            self.document.validity.value(),
            Some(Validity::Missing | Validity::Invalid)
        ) && self.document.open_state.value() == Some(&OpenState::Open)
        {
            return Err(EvidenceError::InvalidDocument);
        }
        match self.document.open_state.value() {
            Some(OpenState::NotOpen)
                if self.sources.buffer.availability() != Availability::NotApplicable
                    || self.dirty.availability() != Availability::NotApplicable =>
            {
                return Err(EvidenceError::InvalidAvailability)
            }
            Some(OpenState::Open) | None
                if self.sources.buffer.availability() == Availability::NotApplicable
                    || self.dirty.availability() == Availability::NotApplicable =>
            {
                return Err(EvidenceError::InvalidAvailability)
            }
            _ => {}
        }
        if self.document.open_state.value().is_none()
            && (self.sources.buffer.availability() == Availability::Observed
                || self.dirty.availability() == Availability::Observed)
        {
            return Err(EvidenceError::InvalidDocument);
        }
        if self
            .document
            .identity
            .as_ref()
            .is_some_and(|i| i.kind() == ScriptKind::BuiltinGdscript)
            && (self.sources.disk.availability() == Availability::Observed
                || self
                    .sources
                    .disk
                    .reason()
                    .is_some_and(|reason| reason != SourceReason::NoStandaloneDiskSource))
        {
            return Err(EvidenceError::InvalidAvailability);
        }
        if self.document.open_state.value() == Some(&OpenState::NotOpen)
            && self.document.identity.as_ref().is_some_and(|i| {
                i.editor_instance_id().is_some() || i.buffer_instance_id().is_some()
            })
        {
            return Err(EvidenceError::InvalidDocument);
        }
        let (checks, recheck_reason, detected_changes) = match self.recheck {
            Recheck::Unavailable { reason } => (Checks::Unavailable, Some(reason), Vec::new()),
            Recheck::Performed { detected_changes } => (Checks::Performed, None, detected_changes),
            Recheck::Partial {
                reason,
                detected_changes,
            } => (Checks::Unavailable, Some(reason), detected_changes),
        };
        for &change in &detected_changes {
            match change {
                DetectedChange::Source(authority) => {
                    self.sources
                        .get_mut(authority)
                        .invalidate(SourceReason::SourceChanged)?;
                }
                DetectedChange::Dirty => self.dirty.invalidate(DirtyReason::SourceChanged)?,
                DetectedChange::DocumentClosed => {
                    self.document
                        .open_state
                        .invalidate(FactReason::DocumentClosed);
                    self.sources
                        .buffer
                        .invalidate(SourceReason::DocumentClosed)?;
                    self.dirty.invalidate(DirtyReason::DocumentClosed)?;
                }
                DetectedChange::DiskIdentityReplaced => {
                    self.sources
                        .disk
                        .invalidate(SourceReason::IdentityChanged)?;
                    self.document
                        .validity
                        .invalidate(FactReason::IdentityChanged);
                }
                DetectedChange::DocumentIdentityReplaced | DetectedChange::SessionReplaced => {
                    for authority in [Authority::D, Authority::R, Authority::B] {
                        self.sources
                            .get_mut(authority)
                            .invalidate(SourceReason::IdentityChanged)?;
                    }
                    self.dirty.invalidate(DirtyReason::IdentityChanged)?;
                    self.document
                        .validity
                        .invalidate(FactReason::IdentityChanged);
                    self.document
                        .open_state
                        .invalidate(FactReason::IdentityChanged);
                }
                DetectedChange::SessionEnded => {
                    self.sources
                        .resource
                        .invalidate(SourceReason::SessionEnded)?;
                    self.sources.buffer.invalidate(SourceReason::SessionEnded)?;
                    self.dirty.invalidate(DirtyReason::SessionEnded)?;
                    self.document
                        .open_state
                        .invalidate(FactReason::SessionEnded);
                    if self
                        .document
                        .validity
                        .collection()
                        .is_some_and(|stamp| matches!(stamp.clock_id(), ClockId::Editor(_)))
                    {
                        self.document.validity.invalidate(FactReason::SessionEnded);
                    }
                }
            }
        }
        let mut consistency = Consistency {
            checks,
            stability: if detected_changes.is_empty() {
                Stability::Unknown
            } else {
                Stability::Changed
            },
            detected_changes,
            recheck_reason,
        };
        if disconnected
            && !consistency
                .detected_changes
                .contains(&DetectedChange::SessionEnded)
            && !consistency
                .detected_changes
                .contains(&DetectedChange::SessionReplaced)
        {
            self.sources
                .resource
                .invalidate(SourceReason::SessionEnded)?;
            self.sources.buffer.invalidate(SourceReason::SessionEnded)?;
            self.dirty.invalidate(DirtyReason::SessionEnded)?;
            self.document
                .open_state
                .invalidate(FactReason::SessionEnded);
            if self
                .document
                .validity
                .collection()
                .is_some_and(|stamp| matches!(stamp.clock_id(), ClockId::Editor(_)))
            {
                self.document.validity.invalidate(FactReason::SessionEnded);
            }
            consistency.stability = Stability::Changed;
            consistency
                .detected_changes
                .push(DetectedChange::SessionEnded);
        }
        // A later limitation must not conceal a change already established during collection.
        for source in [
            &self.sources.disk,
            &self.sources.resource,
            &self.sources.buffer,
        ] {
            if source
                .invalidated_evidence()
                .is_some_and(|old| old.reason() == SourceReason::SourceChanged)
            {
                let change = DetectedChange::Source(source.authority());
                if !consistency.detected_changes.contains(&change) {
                    consistency.detected_changes.push(change);
                }
                consistency.stability = Stability::Changed;
            }
        }
        if self
            .dirty
            .invalidated_evidence()
            .is_some_and(|old| old.reason() == DirtyReason::SourceChanged)
        {
            if !consistency
                .detected_changes
                .contains(&DetectedChange::Dirty)
            {
                consistency.detected_changes.push(DetectedChange::Dirty);
            }
            consistency.stability = Stability::Changed;
        }
        if self.document.open_state.value().is_none()
            && (self.sources.buffer.availability() == Availability::NotApplicable
                || self.dirty.availability() == Availability::NotApplicable)
        {
            return Err(EvidenceError::InvalidAvailability);
        }
        let comparisons = Comparisons {
            disk_resource: compare(&self.sources.disk, &self.sources.resource),
            disk_buffer: compare(&self.sources.disk, &self.sources.buffer),
            resource_buffer: compare(&self.sources.resource, &self.sources.buffer),
        };
        let agreement = if [
            comparisons.disk_resource,
            comparisons.disk_buffer,
            comparisons.resource_buffer,
        ]
        .contains(&Comparison::Different)
        {
            Agreement::Divergent
        } else if comparisons
            == (Comparisons {
                disk_resource: Comparison::Equal,
                disk_buffer: Comparison::Equal,
                resource_buffer: Comparison::Equal,
            })
        {
            Agreement::Agree
        } else {
            Agreement::Unknown
        };
        let mut diagnostics = Vec::new();
        for (surface, source) in [
            (Surface::D, &self.sources.disk),
            (Surface::R, &self.sources.resource),
            (Surface::B, &self.sources.buffer),
        ] {
            if source.availability() == Availability::Unavailable {
                let code = match source.reason().expect("unavailable has reason") {
                    SourceReason::TooLarge => DiagnosticCode::TooLarge,
                    SourceReason::DiskMissing => DiagnosticCode::DiskMissing,
                    SourceReason::DiskUnreadable => DiagnosticCode::DiskUnreadable,
                    SourceReason::InvalidUtf8 => DiagnosticCode::InvalidUtf8,
                    SourceReason::NoStandaloneDiskSource => DiagnosticCode::NoStandaloneDiskSource,
                    SourceReason::ResourceNotLoaded => DiagnosticCode::ResourceNotLoaded,
                    SourceReason::BufferUnreadable => DiagnosticCode::BufferUnreadable,
                    SourceReason::BufferAttributionUnavailable => {
                        DiagnosticCode::BufferAttributionUnavailable
                    }
                    SourceReason::OpenStateUnknown => DiagnosticCode::OpenStateUnknown,
                    SourceReason::IdentityChanged => DiagnosticCode::IdentityChanged,
                    SourceReason::SourceChanged => DiagnosticCode::SourceChanged,
                    SourceReason::DocumentClosed => DiagnosticCode::DocumentClosed,
                    SourceReason::SessionEnded => DiagnosticCode::SessionEnded,
                    SourceReason::DeadlineExceeded => DiagnosticCode::DeadlineExceeded,
                    SourceReason::ResourceUnreadable => DiagnosticCode::ResourceUnreadable,
                    SourceReason::UnsupportedCapability => DiagnosticCode::UnsupportedCapability,
                    SourceReason::DocumentNotOpen => DiagnosticCode::DocumentClosed,
                };
                let stage = if matches!(
                    code,
                    DiagnosticCode::IdentityChanged
                        | DiagnosticCode::SourceChanged
                        | DiagnosticCode::DocumentClosed
                ) {
                    Stage::Recheck
                } else if surface == Surface::D {
                    Stage::ReadDisk
                } else {
                    Stage::ReadEditor
                };
                diagnostics.push(Diagnostic::new(code, stage, Some(surface)));
            }
        }
        if let DirtyObservation::Unavailable { reason, .. } = &self.dirty {
            let code = match reason {
                DirtyReason::DirtyAttributionUnavailable => {
                    DiagnosticCode::DirtyAttributionUnavailable
                }
                DirtyReason::OpenStateUnknown => DiagnosticCode::OpenStateUnknown,
                DirtyReason::IdentityChanged => DiagnosticCode::IdentityChanged,
                DirtyReason::SourceChanged => DiagnosticCode::SourceChanged,
                DirtyReason::DocumentClosed => DiagnosticCode::DocumentClosed,
                DirtyReason::SessionEnded => DiagnosticCode::SessionEnded,
                DirtyReason::DeadlineExceeded => DiagnosticCode::DeadlineExceeded,
                DirtyReason::UnsupportedCapability => DiagnosticCode::UnsupportedCapability,
                DirtyReason::DocumentNotOpen => DiagnosticCode::DocumentClosed,
            };
            let stage = if matches!(
                code,
                DiagnosticCode::IdentityChanged
                    | DiagnosticCode::SourceChanged
                    | DiagnosticCode::DocumentClosed
            ) {
                Stage::Recheck
            } else {
                Stage::ReadEditor
            };
            diagnostics.push(Diagnostic::new(code, stage, Some(Surface::Dirty)));
        }
        for reason in [
            self.document.validity.reason(),
            self.document.open_state.reason(),
        ]
        .into_iter()
        .flatten()
        {
            let code = match reason {
                FactReason::Unavailable => DiagnosticCode::DocumentStateUnavailable,
                FactReason::OpenStateUnknown => DiagnosticCode::OpenStateUnknown,
                FactReason::IdentityChanged => DiagnosticCode::IdentityChanged,
                FactReason::SourceChanged => DiagnosticCode::SourceChanged,
                FactReason::DocumentClosed => DiagnosticCode::DocumentClosed,
                FactReason::SessionEnded => DiagnosticCode::SessionEnded,
                FactReason::DeadlineExceeded => DiagnosticCode::DeadlineExceeded,
            };
            let stage = if matches!(
                code,
                DiagnosticCode::IdentityChanged
                    | DiagnosticCode::SourceChanged
                    | DiagnosticCode::DocumentClosed
            ) {
                Stage::Recheck
            } else {
                Stage::AttributeDocument
            };
            diagnostics.push(Diagnostic::new(code, stage, Some(Surface::Document)));
        }
        if consistency.checks == Checks::Unavailable {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::RecheckUnavailable,
                Stage::Recheck,
                Some(Surface::Document),
            ));
        }
        Ok(ObservationSnapshot {
            target: self.target,
            document: self.document,
            sources: self.sources,
            dirty: self.dirty,
            comparisons,
            agreement,
            consistency,
            diagnostics,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationSnapshot {
    target: ResolvedTarget,
    document: DocumentState,
    sources: Sources,
    dirty: DirtyObservation,
    comparisons: Comparisons,
    agreement: Agreement,
    consistency: Consistency,
    diagnostics: Vec<Diagnostic>,
}
impl ObservationSnapshot {
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    pub fn document(&self) -> &DocumentState {
        &self.document
    }
    pub fn sources(&self) -> &Sources {
        &self.sources
    }
    pub fn dirty(&self) -> &DirtyObservation {
        &self.dirty
    }
    pub fn comparisons(&self) -> Comparisons {
        self.comparisons
    }
    pub fn agreement(&self) -> Agreement {
        self.agreement
    }
    pub fn consistency(&self) -> &Consistency {
        &self.consistency
    }
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    fn classification(&self) -> OutcomeKind {
        if self.document.validity.value() != Some(&Validity::Valid) {
            return OutcomeKind::LimitedObservation;
        }
        if self.consistency.checks != Checks::Performed {
            return OutcomeKind::LimitedObservation;
        }
        match self.document.open_state.value() {
            Some(OpenState::NotOpen) if self.document.identity.is_some() => OutcomeKind::NotOpen,
            Some(OpenState::Open)
                if self.document.identity.is_some()
                    && self.consistency.stability != Stability::Changed
                    && self.sources.disk.availability() == Availability::Observed
                    && self.sources.resource.availability() == Availability::Observed
                    && self.sources.buffer.availability() == Availability::Observed
                    && self.dirty.availability() == Availability::Observed =>
            {
                OutcomeKind::CompleteObservation
            }
            _ => OutcomeKind::LimitedObservation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingSelector {
    SessionId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    candidate_sessions: Vec<SessionId>,
    missing_selector: MissingSelector,
}
impl Selection {
    /// # Errors
    /// Only multiple distinct confirmed candidates permit ambiguous selection metadata.
    pub fn ambiguous(mut candidate_sessions: Vec<SessionId>) -> Result<Self, EvidenceError> {
        candidate_sessions.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        if candidate_sessions.len() < 2 || candidate_sessions.windows(2).any(|w| w[0] == w[1]) {
            return Err(EvidenceError::InvalidSelection);
        }
        Ok(Self {
            candidate_sessions,
            missing_selector: MissingSelector::SessionId,
        })
    }
    pub fn candidate_sessions(&self) -> &[SessionId] {
        &self.candidate_sessions
    }
    pub fn missing_selector(&self) -> MissingSelector {
        self.missing_selector
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeKind {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalFailure {
    AmbiguousTarget(Selection),
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
impl TerminalFailure {
    // Fail closed; a known result (especially disconnection) beats deadline expiry.
    // Refusals and interruptions beat open/closed classification, never the other way round.
    fn priority(&self) -> u8 {
        match self {
            Self::DeniedAccess => 0,
            Self::AmbiguousTarget(_) => 1,
            Self::InvalidRequest => 2,
            Self::ProtocolError => 3,
            Self::Cancelled => 4,
            Self::DisconnectedEditor => 5,
            Self::MissingTarget => 6,
            Self::InvalidTarget => 7,
            Self::UnsupportedObservation => 8,
            Self::EditorUnavailable => 9,
            Self::Timeout => 10,
        }
    }
    fn kind(&self) -> OutcomeKind {
        match self {
            Self::AmbiguousTarget(_) => OutcomeKind::AmbiguousTarget,
            Self::MissingTarget => OutcomeKind::MissingTarget,
            Self::InvalidTarget => OutcomeKind::InvalidTarget,
            Self::EditorUnavailable => OutcomeKind::EditorUnavailable,
            Self::DisconnectedEditor => OutcomeKind::DisconnectedEditor,
            Self::Timeout => OutcomeKind::Timeout,
            Self::UnsupportedObservation => OutcomeKind::UnsupportedObservation,
            Self::DeniedAccess => OutcomeKind::DeniedAccess,
            Self::InvalidRequest => OutcomeKind::InvalidRequest,
            Self::ProtocolError => OutcomeKind::ProtocolError,
            Self::Cancelled => OutcomeKind::Cancelled,
        }
    }
    fn source_free(&self) -> bool {
        matches!(
            self,
            Self::DeniedAccess
                | Self::AmbiguousTarget(_)
                | Self::InvalidRequest
                | Self::EditorUnavailable
        )
    }
    fn diagnostic(&self) -> Diagnostic {
        let (code, stage) = match self {
            Self::AmbiguousTarget(_) => (DiagnosticCode::AmbiguousTarget, Stage::ResolveTarget),
            Self::MissingTarget => (DiagnosticCode::MissingTarget, Stage::AttributeDocument),
            Self::InvalidTarget => (DiagnosticCode::InvalidTarget, Stage::AttributeDocument),
            Self::EditorUnavailable => (DiagnosticCode::EditorUnavailable, Stage::ResolveTarget),
            Self::DisconnectedEditor => (DiagnosticCode::SessionEnded, Stage::ReadEditor),
            Self::Timeout => (DiagnosticCode::DeadlineExceeded, Stage::Finalize),
            Self::UnsupportedObservation => (
                DiagnosticCode::UnsupportedObservation,
                Stage::AttributeDocument,
            ),
            Self::DeniedAccess => (DiagnosticCode::DeniedAccess, Stage::ResolveTarget),
            Self::InvalidRequest => (DiagnosticCode::InvalidRequest, Stage::ValidateRequest),
            Self::ProtocolError => (DiagnosticCode::ProtocolError, Stage::ReadEditor),
            Self::Cancelled => (DiagnosticCode::Cancelled, Stage::Finalize),
        };
        Diagnostic::new(code, stage, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationOutcome {
    request_id: RequestId,
    outcome: OutcomeKind,
    interval: ObservationInterval,
    resolved_target: Option<ResolvedTarget>,
    snapshot: Option<ObservationSnapshot>,
    diagnostics: Vec<Diagnostic>,
    selection: Option<Selection>,
}
impl ObservationOutcome {
    /// Reduce one attempt's trusted evidence and explicit terminal signals. Signals are
    /// ordered by safety precedence, not arrival order. Denial and ambiguity suppress
    /// *all* source, including previously invalidated evidence. An interrupted attempt
    /// retains separately attributed partial evidence but cannot become a success.
    ///
    /// # Errors
    /// Refuses mismatched request/target/session, mixed clocks, out-of-interval facts,
    /// contradictions of closed/open applicability, or unbound evidence. The caller must
    /// convert this error to a safe boundary failure, not publish the rejected evidence.
    pub fn classify(
        request: ObservationRequest,
        interval: ObservationInterval,
        target: Option<ResolvedTarget>,
        evidence: Option<ObservationEvidence>,
        signals: Vec<TerminalFailure>,
        mut diagnostics: Vec<Diagnostic>,
    ) -> Result<Self, EvidenceError> {
        let inferred = evidence
            .as_ref()
            .and_then(ObservationEvidence::terminal_failure);
        let mut disconnected = false;
        let mut failure: Option<TerminalFailure> = None;
        for signal in signals.into_iter().chain(inferred) {
            disconnected |= signal == TerminalFailure::DisconnectedEditor;
            let diagnostic = signal.diagnostic();
            if !diagnostics.contains(&diagnostic) {
                diagnostics.push(diagnostic);
            }
            if failure
                .as_ref()
                .is_none_or(|known| signal.priority() < known.priority())
            {
                failure = Some(signal);
            }
        }
        let source_free = failure.as_ref().is_some_and(TerminalFailure::source_free);
        if source_free {
            let failure = failure.expect("source-free failure");
            let outcome = failure.kind();
            let selection = match failure {
                TerminalFailure::AmbiguousTarget(selection) => {
                    if request.session_id.is_some() {
                        return Err(EvidenceError::InvalidSelection);
                    }
                    Some(selection)
                }
                _ => None,
            };
            return Ok(Self {
                request_id: request.request_id,
                outcome,
                interval,
                resolved_target: None,
                snapshot: None,
                diagnostics,
                selection,
            });
        }
        if evidence.is_none() && failure.is_none() {
            return Err(EvidenceError::InvalidDocument);
        }
        let mut resolved_target = target;
        if let Some(target) = &resolved_target {
            validate_target(&request, target)?;
        }
        let snapshot = if let Some(evidence) = evidence {
            validate_target(&request, &evidence.target)?;
            if resolved_target
                .as_ref()
                .is_some_and(|t| t != &evidence.target)
            {
                return Err(EvidenceError::WrongTarget);
            }
            if resolved_target.is_none() {
                resolved_target = Some(evidence.target.clone());
            }
            Some(evidence.finish(&interval, disconnected)?)
        } else {
            None
        };
        let inferred = snapshot.as_ref().and_then(|snapshot| {
            match (
                snapshot.document.validity.value(),
                snapshot.document.open_state.value(),
            ) {
                (Some(Validity::Missing), Some(OpenState::NotOpen)) => {
                    Some(TerminalFailure::MissingTarget)
                }
                (Some(Validity::Invalid), Some(OpenState::NotOpen)) => {
                    Some(TerminalFailure::InvalidTarget)
                }
                _ => None,
            }
        });
        if matches!(
            failure,
            Some(TerminalFailure::MissingTarget | TerminalFailure::InvalidTarget)
        ) && failure != inferred
        {
            return Err(EvidenceError::InvalidDocument);
        }
        if let Some(inferred) = inferred {
            if failure
                .as_ref()
                .is_none_or(|known| inferred.priority() <= known.priority())
            {
                let diagnostic = inferred.diagnostic();
                if !diagnostics.contains(&diagnostic) {
                    diagnostics.push(diagnostic);
                }
                return Ok(Self {
                    request_id: request.request_id,
                    outcome: inferred.kind(),
                    interval,
                    resolved_target,
                    snapshot: None,
                    diagnostics,
                    selection: None,
                });
            }
        }
        let outcome = match failure {
            Some(failure) => failure.kind(),
            None => snapshot.as_ref().map_or(
                OutcomeKind::LimitedObservation,
                ObservationSnapshot::classification,
            ),
        };
        if let Some(snapshot) = &snapshot {
            diagnostics.extend_from_slice(snapshot.diagnostics());
        }
        Ok(Self {
            request_id: request.request_id,
            outcome,
            interval,
            resolved_target,
            snapshot,
            diagnostics,
            selection: None,
        })
    }
    pub fn schema_version(&self) -> u32 {
        SCHEMA_VERSION
    }
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn outcome(&self) -> OutcomeKind {
        self.outcome
    }
    pub fn interval(&self) -> &ObservationInterval {
        &self.interval
    }
    pub fn resolved_target(&self) -> Option<&ResolvedTarget> {
        self.resolved_target.as_ref()
    }
    pub fn snapshot(&self) -> Option<&ObservationSnapshot> {
        self.snapshot.as_ref()
    }
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }
}
fn validate_target(
    request: &ObservationRequest,
    target: &ResolvedTarget,
) -> Result<(), EvidenceError> {
    if request.request_id != target.request_id
        || request.project_root() != target.project_root()
        || request.script_path() != target.script_path()
        || request
            .session_id()
            .is_some_and(|s| s != target.session_id())
    {
        return Err(EvidenceError::WrongTarget);
    }
    Ok(())
}
