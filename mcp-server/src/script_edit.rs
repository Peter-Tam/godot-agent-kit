//! Protocol-independent eligibility and terminal interpretation for one open GDScript edit.
//!
//! An adapter must independently acquire and authenticate every supplied fact. In particular,
//! neither a receipt nor a finalizer response is a readback of D, R, B or saved state. This
//! module does not issue native operations, own a filesystem capability or enforce a deadline.
//!
//! A typed caller extracts `ExpectedRevisionBasis::from_observation` from a prior complete
//! observation, constructs `EditRequest`, and calls `EditAttempt::select` and `prepare` with
//! freshly checked evidence. Changed intent requires preflight validation, `authorize` before
//! dispatch, `guard_application` before native entry, and separately recorded application,
//! persistence, finalization, post-change validation and independent `verify`. Unchanged intent
//! uses only unchanged validation and verification. Finally `finish` consumes the attempt.
//! The caller must never infer authorization or a saved version from prior observation fields.

use crate::observation::{
    Agreement, Authority, Availability, Checks, ClockId, CollectionStamp, Comparisons, Consistency,
    DecimalCounter, DirtyObservation, DirtyState, DocumentIdentity, DocumentState, FileIdentity,
    ObservationInterval, ObservationOutcome, OutcomeKind, ProjectRoot, RequestId, ResolvedTarget,
    ResourcePath, ScriptKind, Selection, SessionId, SourceObservation, SourceReason, Stability,
    Witness, SOURCE_LIMIT_BYTES,
};
use ring::digest::{digest, SHA256};
use std::fmt;

pub const SCHEMA_VERSION: u32 = 1;
pub const OPERATION: &str = "edit_open_gdscript";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    InvalidSource,
    WrongTarget,
    WrongClock,
    InvalidTiming,
    InvalidEvidence,
    OutOfOrder,
}
impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid edit evidence: {self:?}")
    }
}
impl std::error::Error for EditError {}

/// Exact, bounded LF UTF-8. Source bytes are deliberately absent from `Debug` output.
#[derive(Clone, PartialEq, Eq)]
pub struct ReplacementSource(String);
impl fmt::Debug for ReplacementSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReplacementSource([redacted])")
    }
}
impl ReplacementSource {
    /// # Errors
    /// Rejects more than 512 KiB, NUL, CR, or any UTF-8 BOM; never normalizes input.
    pub fn new(value: String) -> Result<Self, EditError> {
        if value.len() > SOURCE_LIMIT_BYTES
            || value.chars().any(|c| matches!(c, '\0' | '\r' | '\u{feff}'))
        {
            return Err(EditError::InvalidSource);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Digest is a summary, never authority to write or a substitute for exact byte comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceDigest {
    pub sha256: [u8; 32],
    pub utf8_bytes: usize,
}
impl SourceDigest {
    pub fn of(text: &str) -> Self {
        let mut sha256 = [0; 32];
        sha256.copy_from_slice(digest(&SHA256, text.as_bytes()).as_ref());
        Self {
            sha256,
            utf8_bytes: text.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasisError {
    Missing,
    Dirty,
    Divergent,
    KnownStaleResource,
    KnownStaleBuffer,
    MissingVersion,
    UnsupportedTarget,
    UnsupportedRepresentation,
}

/// Provenance from an eligible *prior* observation. Its source and version are not authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRevisionBasis {
    prior_request_id: RequestId,
    prior_interval: ObservationInterval,
    target: ResolvedTarget,
    document: DocumentIdentity,
    witnesses: [Witness; 3],
    collections: [CollectionStamp; 3],
    prior_document: DocumentState,
    prior_dirty: DirtyObservation,
    source: SourceDigest,
    current_version: DecimalCounter,
}
impl ExpectedRevisionBasis {
    /// # Errors
    /// Only a complete, stable, clean, agreeing open standalone script with all three actual
    /// sources and an attributable buffer current version supplies a basis. A saved version is
    /// deliberately not inferred from observation v1.
    pub fn from_observation(prior: &ObservationOutcome) -> Result<Self, BasisError> {
        let snapshot = prior.snapshot().ok_or(BasisError::Missing)?;
        if prior.outcome() != OutcomeKind::CompleteObservation
            || snapshot.consistency().checks() != Checks::Performed
            || snapshot.consistency().stability() == Stability::Changed
        {
            return Err(BasisError::Missing);
        }
        let target = snapshot.target();
        let document = snapshot.document().identity().ok_or(BasisError::Missing)?;
        if document.kind() != ScriptKind::ExternalGdscript
            || document.disk_file_id().is_none()
            || document.script_instance_id().is_none()
            || document.editor_instance_id().is_none()
            || document.buffer_instance_id().is_none()
        {
            return Err(BasisError::UnsupportedTarget);
        }
        if snapshot.dirty().state() != Some(DirtyState::Clean) {
            return Err(BasisError::Dirty);
        }
        let [d, r, b] = [
            snapshot.sources().disk(),
            snapshot.sources().resource(),
            snapshot.sources().buffer(),
        ];
        if r.staleness().is_some_and(|v| v.evidence().is_some()) {
            return Err(BasisError::KnownStaleResource);
        }
        if b.staleness().is_some_and(|v| v.evidence().is_some()) {
            return Err(BasisError::KnownStaleBuffer);
        }
        if snapshot.agreement() != Agreement::Agree {
            return Err(BasisError::Divergent);
        }
        let source = d.text().ok_or(BasisError::Missing)?;
        if r.text() != Some(source) || b.text() != Some(source) {
            return Err(BasisError::Divergent);
        }
        if source
            .chars()
            .any(|c| matches!(c, '\0' | '\r' | '\u{feff}'))
        {
            return Err(BasisError::UnsupportedRepresentation);
        }
        let disk = d.witness().cloned().ok_or(BasisError::Missing)?;
        let resource = r.witness().cloned().ok_or(BasisError::Missing)?;
        let buffer = b.witness().cloned().ok_or(BasisError::Missing)?;
        let current_version = buffer
            .source_version()
            .cloned()
            .ok_or(BasisError::MissingVersion)?;
        Ok(Self {
            prior_request_id: prior.request_id().clone(),
            prior_interval: prior.interval().clone(),
            target: target.clone(),
            document: document.clone(),
            witnesses: [disk, resource, buffer],
            collections: [d, r, b].map(|s| s.collection().expect("observed source").clone()),
            prior_document: snapshot.document().clone(),
            prior_dirty: snapshot.dirty().clone(),
            source: SourceDigest::of(source),
            current_version,
        })
    }
    pub fn prior_request_id(&self) -> &RequestId {
        &self.prior_request_id
    }
    pub fn prior_interval(&self) -> &ObservationInterval {
        &self.prior_interval
    }
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    pub fn document(&self) -> &DocumentIdentity {
        &self.document
    }
    pub fn witnesses(&self) -> &[Witness; 3] {
        &self.witnesses
    }
    pub fn collections(&self) -> &[CollectionStamp; 3] {
        &self.collections
    }
    pub fn prior_document(&self) -> &DocumentState {
        &self.prior_document
    }
    pub fn prior_dirty(&self) -> &DirtyObservation {
        &self.prior_dirty
    }
    pub fn source(&self) -> &SourceDigest {
        &self.source
    }
    pub fn current_version(&self) -> &DecimalCounter {
        &self.current_version
    }
}

/// Caller intent; the selected target must still be freshly authenticated.
#[derive(Debug)]
pub struct EditRequest {
    request_id: RequestId,
    project_root: ProjectRoot,
    session_id: Option<SessionId>,
    script_path: ResourcePath,
    expected: ExpectedRevisionBasis,
    replacement_source: ReplacementSource,
}
impl EditRequest {
    /// # Errors
    /// Rejects a non-standalone selector, mismatched prior selectors, or a reused prior ID.
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
        expected: ExpectedRevisionBasis,
        replacement_source: ReplacementSource,
    ) -> Result<Self, EditError> {
        if script_path.kind() != Some(ScriptKind::ExternalGdscript)
            || expected.target.project_root() != &project_root
            || expected.target.script_path() != &script_path
            || session_id
                .as_ref()
                .is_some_and(|s| s != expected.target.session_id())
            || request_id == expected.prior_request_id
        {
            return Err(EditError::WrongTarget);
        }
        Ok(Self {
            request_id,
            project_root,
            session_id,
            script_path,
            expected,
            replacement_source,
        })
    }
    pub fn schema_version(&self) -> u32 {
        SCHEMA_VERSION
    }
    pub fn operation(&self) -> &'static str {
        OPERATION
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
    pub fn expected(&self) -> &ExpectedRevisionBasis {
        &self.expected
    }
    pub fn replacement_source(&self) -> &ReplacementSource {
        &self.replacement_source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceEvidence {
    pub authority: Authority,
    pub availability: Availability,
    pub source: Option<SourceDigest>,
    pub collection: Option<CollectionStamp>,
    pub identity: Option<Witness>,
    pub staleness: Option<crate::observation::Staleness>,
    pub reason: Option<SourceReason>,
    pub invalidated: Option<InvalidatedSurface>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedSurface {
    pub source: SourceDigest,
    pub collection: CollectionStamp,
    pub identity: Witness,
    pub staleness: crate::observation::Staleness,
    pub reason: SourceReason,
}
fn surface(source: &SourceObservation, agreed: Option<SourceDigest>) -> SurfaceEvidence {
    SurfaceEvidence {
        authority: source.authority(),
        availability: source.availability(),
        source: source
            .text()
            .map(|text| agreed.unwrap_or_else(|| SourceDigest::of(text))),
        collection: source.collection().cloned(),
        identity: source.witness().cloned(),
        staleness: source.staleness().cloned(),
        reason: source.reason(),
        invalidated: source.invalidated_evidence().map(|old| InvalidatedSurface {
            source: SourceDigest::of(old.text()),
            collection: old.collection().clone(),
            identity: old.witness().clone(),
            staleness: old.staleness().clone(),
            reason: old.reason(),
        }),
    }
}

/// Independent caller-clock disk identity/mtime read, separate from the writer's receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskMetadata {
    pub identity: FileIdentity,
    pub mtime: DecimalCounter,
    pub collection: CollectionStamp,
}
/// Independent native inspection, not the finalizer's returned step flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedStateEvidence {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub current_version: DecimalCounter,
    pub saved_version: DecimalCounter,
    pub resource_edited: bool,
    pub resource_mtime: DecimalCounter,
    pub document_mtime: DecimalCounter,
    pub save_profile: [u8; 32],
    pub original_preserved: bool,
    pub desired_preserved: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditEvidence {
    pub document: DocumentState,
    pub sources: [SurfaceEvidence; 3],
    pub dirty: DirtyObservation,
    pub saved_state: Option<SavedStateEvidence>,
    pub comparisons: Comparisons,
    /// Why a required independent inspection is missing; never an inferred clean state.
    pub saved_state_reason: Option<Reason>,
    pub agreement: Agreement,
    pub consistency: Consistency,
    pub disk_metadata: Option<DiskMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Accepted,
    Selected,
    Prepared,
    Preflight,
    Authorized,
    Applying,
    Persisting,
    Finalizing,
    Validating,
    Verifying,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    NotStarted,
    Entered,
    Completed,
    Failed,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub state: StepState,
    pub reason: Option<Reason>,
    /// Actual native evidence timing, not the dispatch/authorization time.
    pub collection: Option<CollectionStamp>,
}
impl Default for Step {
    fn default() -> Self {
        Self {
            state: StepState::NotStarted,
            reason: None,
            collection: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttemptProgress {
    pub buffer_application: Step,
    pub resource_sync: Step,
    pub persistence: Step,
    pub finalization: Step,
    pub validation: Step,
    pub verification: Step,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Application {
    NotApplied,
    Applied,
    PartlyApplied,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditOutcomeKind {
    VerifiedChanged,
    VerifiedUnchanged,
    Refused,
    AppliedUnverified,
    ApplicationUnknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Complete,
    Busy,
    DirtyConflict,
    RevisionMismatch,
    IdentityChanged,
    SessionChanged,
    Divergence,
    KnownStaleResource,
    KnownStaleBuffer,
    UnavailableObservation,
    ClosedTarget,
    UnsupportedTarget,
    AmbiguousTarget,
    DeniedAccess,
    ApprovalRequired,
    UnsupportedEngine,
    UnsupportedEffect,
    UnsupportedRepresentation,
    SaveWouldReformat,
    PersistenceFailure,
    PersistenceUnknown,
    PartialFinalization,
    ParseError,
    DependencyError,
    ValidationUnavailable,
    EvidenceLimit,
    Deadline,
    Cancellation,
    Disconnection,
    ProtocolFailure,
    MissingBasis,
    IncompleteVerification,
    RuntimeUnavailable,
}
impl Reason {
    fn suppresses_source(self) -> bool {
        matches!(
            self,
            Self::DeniedAccess | Self::AmbiguousTarget | Self::Busy
        )
    }
    fn action(self) -> &'static str {
        match self {
            Self::Complete => "No recovery action required",
            Self::AmbiguousTarget => "Select exactly one session and observe it again",
            Self::DeniedAccess => "Check access without disclosing source",
            _ => "Observe the explicitly selected target again before any new edit",
        }
    }
}
fn observation_reason(kind: OutcomeKind) -> Option<Reason> {
    match kind {
        OutcomeKind::CompleteObservation => None,
        OutcomeKind::DisconnectedEditor => Some(Reason::Disconnection),
        OutcomeKind::Timeout => Some(Reason::Deadline),
        OutcomeKind::Cancelled => Some(Reason::Cancellation),
        OutcomeKind::ProtocolError => Some(Reason::ProtocolFailure),
        OutcomeKind::DeniedAccess => Some(Reason::DeniedAccess),
        OutcomeKind::AmbiguousTarget => Some(Reason::AmbiguousTarget),
        OutcomeKind::NotOpen => Some(Reason::ClosedTarget),
        OutcomeKind::InvalidTarget | OutcomeKind::MissingTarget => Some(Reason::UnsupportedTarget),
        _ => Some(Reason::UnavailableObservation),
    }
}
/// Bounded source-free cause; detailed source-attributed errors remain in validation records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditDiagnostic {
    pub stage: Stage,
    pub reason: Reason,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    NotParticipated,
    NativeComplexEdit,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationPurpose {
    Preflight,
    PostChange,
    Unchanged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Unavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticOrigin {
    Root,
    Dependency,
}
#[derive(Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub origin: DiagnosticOrigin,
    pub path: Option<ResourcePath>,
    pub path_reason: Option<Reason>,
    pub source: Option<SourceDigest>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub category: String,
    pub message: String,
}
impl fmt::Debug for ValidationDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationDiagnostic")
            .field("origin", &self.origin)
            .field("path", &self.path)
            .field("source", &self.source)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("category", &"[redacted]")
            .field("message", &"[redacted]")
            .finish()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyWitness {
    pub path: ResourcePath,
    pub identity: FileIdentity,
    pub source: SourceDigest,
}
/// Validation is attributed to exact source, invocation, dependency and context witnesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub reason: Option<Reason>,
    pub purpose: ValidationPurpose,
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub source_path: ResourcePath,
    pub input: SourceDigest,
    pub collection: CollectionStamp,
    pub dependencies: Vec<DependencyWitness>,
    pub context: [u8; 32],
    pub context_current: bool,
    pub dependencies_current: bool,
    pub diagnostics_complete: bool,
    pub diagnostics: Vec<ValidationDiagnostic>,
}
/// Fresh read-only project mapping and dependency witnesses, obtained after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRecheck {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub context: [u8; 32],
    pub dependencies: Vec<DependencyWitness>,
    pub current: bool,
}

/// Attributable native acknowledgment. Only a trusted integration may supply it.
/// A discard witness additionally attests that the native context is permanently consumed
/// before any source/history entry; an abort request or transport EOF is not this evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWitness {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
}
/// Summary of attempt-local native descriptor evidence; never accepted as independent D readback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceReceipt {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub intended: SourceDigest,
    pub project_id: FileIdentity,
    pub file_id: FileIdentity,
    pub collection: CollectionStamp,
    pub write_started: bool,
    pub bytes_written: usize,
    pub truncated: bool,
    pub flushed: bool,
    pub readback_matches: bool,
    pub attached: bool,
    pub descriptor_open: bool,
    pub interference: bool,
    pub mtime: Option<DecimalCounter>,
    pub reason: Option<Reason>,
}
impl PersistenceReceipt {
    pub fn complete(&self) -> bool {
        self.write_started
            && self.bytes_written == self.intended.utf8_bytes
            && self.truncated
            && self.flushed
            && self.readback_matches
            && self.attached
            && self.descriptor_open
            && !self.interference
            && self.mtime.is_some()
            && self.reason.is_none()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizationStatus {
    Complete,
    Rejected,
    PartialOrUnknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bookkeeping {
    pub resource_mtime: bool,
    pub document_mtime: bool,
    pub resource_edited: bool,
    pub saved_version: bool,
    pub display: bool,
}
impl Bookkeeping {
    fn any(self) -> bool {
        self.resource_mtime
            || self.document_mtime
            || self.resource_edited
            || self.saved_version
            || self.display
    }
    fn all(self) -> bool {
        self.resource_mtime
            && self.document_mtime
            && self.resource_edited
            && self.saved_version
            && self.display
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizationResult {
    pub status: FinalizationStatus,
    pub reason: Option<Reason>,
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub before_source: Option<SourceDigest>,
    pub after_source: Option<SourceDigest>,
    pub before_current: Option<DecimalCounter>,
    pub after_current: Option<DecimalCounter>,
    pub before_saved: Option<DecimalCounter>,
    pub after_saved: Option<DecimalCounter>,
    pub before_resource_mtime: Option<DecimalCounter>,
    pub after_resource_mtime: Option<DecimalCounter>,
    pub before_document_mtime: Option<DecimalCounter>,
    pub after_document_mtime: Option<DecimalCounter>,
    pub before_resource_edited: Option<bool>,
    pub after_resource_edited: Option<bool>,
    pub steps: Bookkeeping,
}

/// Terminal public result has summaries only, no copy of the supplied or acquired source.
#[derive(Debug)]
pub struct EditOutcome {
    pub request_id: RequestId,
    pub requested_target: (ProjectRoot, Option<SessionId>, ResourcePath),
    pub resolved_target: Option<ResolvedTarget>,
    pub interval: ObservationInterval,
    pub outcome: EditOutcomeKind,
    pub reason: Reason,
    pub stage: Stage,
    pub application: Application,
    pub progress: AttemptProgress,
    pub expected: Option<ExpectedRevisionBasis>,
    pub before: Option<EditEvidence>,
    pub after: Option<EditEvidence>,
    pub persistence: Option<PersistenceReceipt>,
    pub finalization: Option<FinalizationResult>,
    pub validation: Vec<ValidationResult>,
    pub context_recheck: Option<ContextRecheck>,
    pub history: History,
    pub diagnostics: Vec<EditDiagnostic>,
    pub selection: Option<Selection>,
    pub safe_next_action: &'static str,
}
impl EditOutcome {
    pub fn schema_version(&self) -> u32 {
        SCHEMA_VERSION
    }
    pub fn operation(&self) -> &'static str {
        OPERATION
    }
    /// A checked caller that cannot extract an eligible prior basis never gains authorization.
    /// This constructor publishes no prior source, version or resolved target.
    pub fn refuse_without_basis(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
        interval: ObservationInterval,
        error: BasisError,
    ) -> Self {
        let reason = match error {
            BasisError::Dirty => Reason::DirtyConflict,
            BasisError::Divergent => Reason::Divergence,
            BasisError::KnownStaleResource => Reason::KnownStaleResource,
            BasisError::KnownStaleBuffer => Reason::KnownStaleBuffer,
            BasisError::UnsupportedTarget => Reason::UnsupportedTarget,
            BasisError::UnsupportedRepresentation => Reason::UnsupportedRepresentation,
            BasisError::Missing | BasisError::MissingVersion => Reason::MissingBasis,
        };
        Self {
            request_id,
            requested_target: (project_root, session_id, script_path),
            resolved_target: None,
            interval,
            outcome: EditOutcomeKind::Refused,
            reason,
            stage: Stage::Accepted,
            application: Application::NotApplied,
            progress: AttemptProgress::default(),
            expected: None,
            before: None,
            after: None,
            persistence: None,
            finalization: None,
            validation: Vec::new(),
            context_recheck: None,
            history: History::NotParticipated,
            diagnostics: vec![EditDiagnostic {
                stage: Stage::Accepted,
                reason,
            }],
            selection: None,
            safe_next_action: reason.action(),
        }
    }
}

/// Consuming one-attempt reducer. Methods reject out-of-order evidence rather than retargeting.
/// An error makes the attempt a sticky protocol failure; call `finish` to classify retained facts.
pub struct EditAttempt {
    request: EditRequest,
    target: Option<ResolvedTarget>,
    stage: Stage,
    progress: AttemptProgress,
    before: Option<EditEvidence>,
    after: Option<EditEvidence>,
    persistence: Option<PersistenceReceipt>,
    finalization: Option<FinalizationResult>,
    validation: Vec<ValidationResult>,
    context_recheck: Option<ContextRecheck>,
    intent_changed: Option<bool>,
    intended: SourceDigest,
    applied_version: Option<DecimalCounter>,
    authorized: bool,
    guarded: bool,
    entered: bool,
    discarded: bool,
    application: Application,
    history: History,
    failure: Option<Reason>,
    selection: Option<Selection>,
    last_received_us: u64,
    last_editor_tick: Option<DecimalCounter>,
    last_caller_tick: Option<DecimalCounter>,
}
impl EditAttempt {
    pub fn new(request: EditRequest) -> Self {
        let intended = SourceDigest::of(request.replacement_source.as_str());
        Self {
            request,
            target: None,
            stage: Stage::Accepted,
            progress: AttemptProgress::default(),
            before: None,
            after: None,
            persistence: None,
            finalization: None,
            validation: Vec::new(),
            context_recheck: None,
            intent_changed: None,
            intended,
            applied_version: None,
            authorized: false,
            guarded: false,
            entered: false,
            discarded: false,
            application: Application::NotApplied,
            history: History::NotParticipated,
            failure: None,
            selection: None,
            last_received_us: 0,
            last_editor_tick: None,
            last_caller_tick: None,
        }
    }
    fn record_failure(&mut self, reason: Reason) {
        if self.failure.is_none() || reason.suppresses_source() {
            self.failure = Some(reason);
        }
    }
    fn terminal_observation(&mut self, observed: &ObservationOutcome) -> Result<bool, EditError> {
        if observed.request_id() != self.request.request_id() {
            return Err(self.reject(EditError::WrongTarget));
        }
        let reason = observation_reason(observed.outcome());
        if reason.is_some_and(Reason::suppresses_source)
            || observed.snapshot().is_none()
            || observed.outcome() == OutcomeKind::NotOpen
        {
            self.fail(
                reason.unwrap_or(Reason::UnavailableObservation),
                observed.selection().cloned(),
            )?;
            return Ok(true);
        }
        Ok(false)
    }
    fn reject(&mut self, error: EditError) -> EditError {
        self.record_failure(Reason::ProtocolFailure);
        error
    }
    fn require(&mut self, condition: bool) -> Result<(), EditError> {
        if condition {
            Ok(())
        } else {
            Err(self.reject(EditError::OutOfOrder))
        }
    }
    fn target(&self) -> Result<&ResolvedTarget, EditError> {
        self.target.as_ref().ok_or(EditError::WrongTarget)
    }
    fn check_stamp(&mut self, stamp: &CollectionStamp, editor: bool) -> Result<(), EditError> {
        let clock = match stamp.clock_id() {
            ClockId::Caller if !editor => true,
            ClockId::Editor(id) if editor => {
                self.target.as_ref().is_some_and(|t| t.session_id() == id)
            }
            _ => false,
        };
        if !clock {
            return Err(self.reject(EditError::WrongClock));
        }
        Ok(())
    }
    fn check_order(&mut self, stamp: &CollectionStamp) -> Result<(), EditError> {
        let previous = match stamp.clock_id() {
            ClockId::Caller => self.last_caller_tick.as_ref(),
            ClockId::Editor(_) => self.last_editor_tick.as_ref(),
        };
        if stamp.received_elapsed_us() < self.last_received_us
            || previous.is_some_and(|old| stamp.started_tick_us() < old)
        {
            return Err(self.reject(EditError::InvalidTiming));
        }
        if previous.is_none_or(|old| stamp.finished_tick_us() > old) {
            let tick = stamp.finished_tick_us().clone();
            match stamp.clock_id() {
                ClockId::Caller => self.last_caller_tick = Some(tick),
                ClockId::Editor(_) => self.last_editor_tick = Some(tick),
            }
        }
        self.last_received_us = stamp.received_elapsed_us();
        Ok(())
    }
    fn check_native(&mut self, witness: &NativeWitness) -> Result<(), EditError> {
        self.check_binding(
            &witness.request_id,
            &witness.session_id,
            &witness.document,
            &witness.collection,
            true,
        )?;
        self.check_order(&witness.collection)
    }
    fn check_binding(
        &mut self,
        request_id: &RequestId,
        session_id: &SessionId,
        document: &DocumentIdentity,
        stamp: &CollectionStamp,
        editor: bool,
    ) -> Result<(), EditError> {
        if request_id != self.request.request_id()
            || self.target()?.session_id() != session_id
            || document != self.request.expected.document()
        {
            return Err(self.reject(EditError::WrongTarget));
        }
        self.check_stamp(stamp, editor)
    }
    /// # Errors
    /// The selected target must match fresh request, explicit session, and the prior exact
    /// project/session identity. The adapter establishes authentication before calling this.
    pub fn select(&mut self, target: ResolvedTarget) -> Result<(), EditError> {
        self.require(self.stage == Stage::Accepted)?;
        if target.project_root() != self.request.project_root()
            || target.script_path() != self.request.script_path()
            || self
                .request
                .session_id()
                .is_some_and(|s| s != target.session_id())
            || target.project_file_id() != self.request.expected.target().project_file_id()
            || target.session_id() != self.request.expected.target().session_id()
            || target.godot_version() != self.request.expected.target().godot_version()
            || target.request_id() != self.request.request_id()
        {
            if target.request_id() == self.request.request_id()
                && target.project_root() == self.request.project_root()
                && target.script_path() == self.request.script_path()
            {
                self.record_failure(
                    if target.session_id() != self.request.expected.target().session_id() {
                        Reason::SessionChanged
                    } else {
                        Reason::IdentityChanged
                    },
                );
            }
            return Err(self.reject(EditError::WrongTarget));
        }
        self.target = Some(target);
        self.stage = Stage::Selected;
        Ok(())
    }
    fn check_disk(&mut self, disk: &DiskMetadata) -> Result<(), EditError> {
        self.check_stamp(&disk.collection, false)?;
        if disk.identity
            != *self
                .request
                .expected
                .document
                .disk_file_id()
                .ok_or(EditError::WrongTarget)?
        {
            return Err(self.reject(EditError::WrongTarget));
        }
        Ok(())
    }
    fn convert(
        &mut self,
        observed: ObservationOutcome,
        saved: Option<SavedStateEvidence>,
        disk: Option<DiskMetadata>,
        after: bool,
    ) -> Result<(EditEvidence, bool, bool), EditError> {
        if observed.request_id() != self.request.request_id()
            || observed.resolved_target() != self.target.as_ref()
        {
            return Err(self.reject(EditError::WrongTarget));
        }
        let snapshot = observed
            .snapshot()
            .ok_or_else(|| self.reject(EditError::InvalidEvidence))?;
        let id = snapshot
            .document()
            .identity()
            .ok_or_else(|| self.reject(EditError::InvalidEvidence))?;
        if id != self.request.expected.document() {
            self.record_failure(Reason::IdentityChanged);
            return Err(self.reject(EditError::WrongTarget));
        }
        if let Some(saved) = &saved {
            self.check_binding(
                &saved.request_id,
                &saved.session_id,
                &saved.document,
                &saved.collection,
                true,
            )?;
        }
        if let Some(disk) = &disk {
            self.check_disk(disk)?;
        }
        let sources = snapshot.sources();
        let stamps = [
            snapshot.sources().disk(),
            snapshot.sources().resource(),
            snapshot.sources().buffer(),
        ]
        .into_iter()
        .filter_map(SourceObservation::collection)
        .chain(snapshot.document().validity().collection())
        .chain(snapshot.document().open_state().collection())
        .chain(snapshot.dirty().collection())
        .chain(saved.iter().map(|s| &s.collection))
        .chain(disk.iter().map(|d| &d.collection));
        let mut earliest = u64::MAX;
        let mut latest = self.last_received_us;
        let (mut min_editor, mut max_editor) = (None::<&DecimalCounter>, None::<&DecimalCounter>);
        let (mut min_caller, mut max_caller) = (None::<&DecimalCounter>, None::<&DecimalCounter>);
        for stamp in stamps {
            earliest = earliest.min(stamp.received_elapsed_us());
            latest = latest.max(stamp.received_elapsed_us());
            let (low, high) = match stamp.clock_id() {
                ClockId::Caller => (&mut min_caller, &mut max_caller),
                ClockId::Editor(_) => (&mut min_editor, &mut max_editor),
            };
            let start = stamp.started_tick_us();
            let finish = stamp.finished_tick_us();
            if low.is_none_or(|old| start < old) {
                *low = Some(start);
            }
            if high.is_none_or(|old| finish > old) {
                *high = Some(finish);
            }
        }
        if earliest < self.last_received_us
            || self
                .last_editor_tick
                .as_ref()
                .zip(min_editor)
                .is_some_and(|(old, now)| now < old)
            || self
                .last_caller_tick
                .as_ref()
                .zip(min_caller)
                .is_some_and(|(old, now)| now < old)
        {
            return Err(self.reject(EditError::InvalidTiming));
        }
        if max_editor
            .is_some_and(|tick| self.last_editor_tick.as_ref().is_none_or(|old| tick > old))
        {
            self.last_editor_tick = max_editor.cloned();
        }
        if max_caller
            .is_some_and(|tick| self.last_caller_tick.as_ref().is_none_or(|old| tick > old))
        {
            self.last_caller_tick = max_caller.cloned();
        }
        self.last_received_us = latest;
        let current = sources.disk().text();
        let expected = if after {
            self.request.replacement_source.as_str()
        } else {
            current.unwrap_or("")
        };
        let exact = current.is_some()
            && [sources.disk(), sources.resource(), sources.buffer()]
                .iter()
                .all(|s| s.text() == Some(expected));
        let changed = current.is_some_and(|s| s != self.request.replacement_source.as_str());
        let b_version = sources.buffer().witness().and_then(Witness::source_version);
        let eligible = observed.outcome() == OutcomeKind::CompleteObservation
            && snapshot.agreement() == Agreement::Agree
            && snapshot.consistency().checks() == Checks::Performed
            && snapshot.consistency().stability() != Stability::Changed
            && snapshot.dirty().state() == Some(DirtyState::Clean)
            && [sources.resource(), sources.buffer()]
                .iter()
                .all(|s| s.staleness().is_some_and(|v| v.evidence().is_none()))
            && exact
            && b_version.is_some()
            && saved.as_ref().zip(disk.as_ref()).is_some_and(|(s, d)| {
                s.current_version == *b_version.expect("eligible version")
                    && !s.resource_edited
                    && s.saved_version == s.current_version
                    && s.resource_mtime == d.mtime
                    && s.document_mtime == d.mtime
                    && s.original_preserved
                    && s.desired_preserved
            });
        let agreed = exact.then(|| {
            if after {
                self.intended
            } else {
                SourceDigest::of(expected)
            }
        });
        let evidence = EditEvidence {
            document: snapshot.document().clone(),
            sources: [
                surface(sources.disk(), agreed),
                surface(sources.resource(), agreed),
                surface(sources.buffer(), agreed),
            ],
            saved_state_reason: saved.is_none().then_some(Reason::UnavailableObservation),
            dirty: snapshot.dirty().clone(),
            saved_state: saved,
            comparisons: snapshot.comparisons(),
            agreement: snapshot.agreement(),
            consistency: snapshot.consistency().clone(),
            disk_metadata: disk,
        };
        Ok((evidence, eligible, changed))
    }
    /// # Errors
    /// Refuses misbound evidence. An ineligible but well-formed sample is retained with an
    /// explicit refusal reason; no authorization can follow it.
    pub fn prepare(
        &mut self,
        observed: ObservationOutcome,
        saved: Option<SavedStateEvidence>,
        disk: Option<DiskMetadata>,
    ) -> Result<(), EditError> {
        self.require(self.stage == Stage::Selected)?;
        if self.terminal_observation(&observed)? {
            return Ok(());
        }
        let observed_reason = observation_reason(observed.outcome());
        let (evidence, eligible, changed) = self.convert(observed, saved, disk, false)?;
        let actual = evidence.sources[0].source;
        let prior = self.request.expected.source();
        let version = evidence.sources[2]
            .identity
            .as_ref()
            .and_then(Witness::source_version);
        let witness_changed = evidence
            .sources
            .iter()
            .zip(self.request.expected.witnesses())
            .any(|(now, old)| {
                now.identity.as_ref().is_some_and(|w| {
                    old.source_version().is_some() && w.source_version() != old.source_version()
                })
            });
        let bad = if matches!(
            observed_reason,
            Some(
                Reason::Deadline
                    | Reason::Cancellation
                    | Reason::Disconnection
                    | Reason::ProtocolFailure
            )
        ) {
            observed_reason
        } else if evidence.dirty.state() == Some(DirtyState::Dirty) {
            Some(Reason::DirtyConflict)
        } else if evidence.consistency.stability() == Stability::Changed {
            Some(
                if evidence
                    .consistency
                    .detected_changes()
                    .iter()
                    .any(|c| matches!(c, crate::observation::DetectedChange::Dirty))
                {
                    Reason::DirtyConflict
                } else if evidence
                    .consistency
                    .detected_changes()
                    .iter()
                    .any(|c| matches!(c, crate::observation::DetectedChange::Source(_)))
                {
                    Reason::RevisionMismatch
                } else {
                    Reason::IdentityChanged
                },
            )
        } else if evidence.sources[1]
            .staleness
            .as_ref()
            .is_some_and(|v| v.evidence().is_some())
        {
            Some(Reason::KnownStaleResource)
        } else if evidence.sources[2]
            .staleness
            .as_ref()
            .is_some_and(|v| v.evidence().is_some())
        {
            Some(Reason::KnownStaleBuffer)
        } else if witness_changed
            || version.is_some_and(|v| v != self.request.expected.current_version())
            || actual.is_some_and(|s| s != *prior)
        {
            Some(Reason::RevisionMismatch)
        } else if evidence.agreement == Agreement::Divergent {
            Some(Reason::Divergence)
        } else if evidence
            .sources
            .iter()
            .any(|s| s.availability != Availability::Observed)
            || evidence.dirty.state().is_none()
        {
            Some(Reason::UnavailableObservation)
        } else if evidence
            .saved_state
            .as_ref()
            .is_some_and(|s| !s.original_preserved || !s.desired_preserved)
        {
            Some(Reason::SaveWouldReformat)
        } else if evidence
            .saved_state
            .as_ref()
            .zip(evidence.disk_metadata.as_ref())
            .is_some_and(|(s, d)| {
                s.current_version != *self.request.expected.current_version()
                    || s.saved_version != s.current_version
                    || s.resource_mtime != d.mtime
                    || s.document_mtime != d.mtime
            })
        {
            Some(Reason::RevisionMismatch)
        } else if !eligible {
            observed_reason.or(Some(Reason::UnavailableObservation))
        } else {
            None
        };
        self.intent_changed = Some(changed);
        self.before = Some(evidence);
        self.stage = Stage::Prepared;
        if let Some(reason) = bad {
            self.record_failure(reason);
        }
        Ok(())
    }
    /// Changed requests require valid preflight. Unchanged requests use a single validation
    /// after preparation and never enter the mutation branch.
    pub fn validation(&mut self, mut result: ValidationResult) -> Result<(), EditError> {
        let changed = self
            .intent_changed
            .ok_or_else(|| self.reject(EditError::OutOfOrder))?;
        let purpose = if changed && !self.authorized {
            ValidationPurpose::Preflight
        } else if changed {
            ValidationPurpose::PostChange
        } else {
            ValidationPurpose::Unchanged
        };
        self.require(
            result.purpose == purpose
                && self.stage >= Stage::Prepared
                && (purpose != ValidationPurpose::PostChange || self.stage >= Stage::Finalizing)
                && (purpose != ValidationPurpose::Preflight || self.stage == Stage::Prepared)
                && self.validation.iter().all(|r| r.purpose != purpose),
        )?;
        if result.reason == Some(Reason::Complete) {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        let mut dependency_bytes = 0usize;
        let mut over_limit = result.dependencies.len() > 32 || result.diagnostics.len() > 64;
        result.dependencies.truncate(32);
        result.dependencies.retain(|dependency| {
            let fits = dependency.source.utf8_bytes <= SOURCE_LIMIT_BYTES
                && dependency.source.utf8_bytes <= 4 * 1024 * 1024 - dependency_bytes;
            if fits {
                dependency_bytes += dependency.source.utf8_bytes;
            }
            over_limit |= !fits;
            fits
        });
        result.diagnostics.truncate(64);
        result.diagnostics.retain(|diagnostic| {
            let fits = diagnostic.message.len() <= 2048 && diagnostic.category.len() <= 128;
            over_limit |= !fits;
            fits
        });
        if over_limit {
            result.status = ValidationStatus::Unavailable;
            result.reason = Some(
                result
                    .reason
                    .filter(|r| r.suppresses_source())
                    .unwrap_or(Reason::EvidenceLimit),
            );
            result.diagnostics_complete = false;
            // A discarded dependency cannot authorize retaining its source-attributed error.
            result.diagnostics.retain(|d| {
                d.origin == DiagnosticOrigin::Root
                    || d.source.is_none()
                    || result
                        .dependencies
                        .iter()
                        .any(|w| Some(&w.path) == d.path.as_ref() && Some(w.source) == d.source)
            });
        }
        self.check_binding(
            &result.request_id,
            &result.session_id,
            &result.document,
            &result.collection,
            true,
        )?;
        if result.source_path != *self.request.script_path()
            || result.input != self.intended
            || result.dependencies.iter().enumerate().any(|(i, d)| {
                d.path.kind() != Some(ScriptKind::ExternalGdscript)
                    || result.dependencies[..i]
                        .iter()
                        .any(|old| old.path == d.path)
            })
            || result.diagnostics.iter().any(|d| {
                d.message.len() > 2048
                    || d.category.len() > 128
                    || d.path.as_ref().is_some_and(|p| p.as_str().len() > 2048)
                    || d.path.is_none() != d.path_reason.is_some()
                    || d.origin == DiagnosticOrigin::Root
                        && (d.path.as_ref() != Some(self.request.script_path())
                            || d.source != Some(self.intended))
                        && result.status != ValidationStatus::Unavailable
                    || d.origin == DiagnosticOrigin::Dependency
                        && d.source.is_some()
                        && !result
                            .dependencies
                            .iter()
                            .any(|w| Some(&w.path) == d.path.as_ref() && Some(w.source) == d.source)
            })
            || result.status == ValidationStatus::Valid && !result.diagnostics.is_empty()
        {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.check_order(&result.collection)?;
        let complete =
            result.diagnostics_complete && result.context_current && result.dependencies_current;
        if !complete {
            result.status = ValidationStatus::Unavailable;
            result.reason = Some(result.reason.unwrap_or(Reason::ValidationUnavailable));
        }
        let good = result.status == ValidationStatus::Valid && result.reason.is_none();
        if !good {
            let reason = result.reason.unwrap_or(match result.status {
                ValidationStatus::Invalid => Reason::ParseError,
                _ => Reason::ValidationUnavailable,
            });
            result.reason = Some(reason);
            self.record_failure(reason);
            self.progress.validation = Step {
                state: if complete && result.status == ValidationStatus::Invalid {
                    StepState::Failed
                } else {
                    StepState::Unknown
                },
                reason: self.failure,
                ..Step::default()
            };
        } else {
            self.progress.validation = Step {
                state: StepState::Completed,
                reason: None,
                ..Step::default()
            };
        }
        self.progress.validation.collection = Some(result.collection.clone());
        self.validation.push(result);
        self.stage = if purpose == ValidationPurpose::Preflight {
            Stage::Preflight
        } else {
            Stage::Validating
        };
        Ok(())
    }
    /// Supervisor MUST call before releasing one-shot worker authorization.
    pub fn authorize(&mut self) -> Result<(), EditError> {
        self.require(
            self.failure.is_none()
                && self.stage >= Stage::Prepared
                && self.intent_changed == Some(true)
                && !self.authorized
                && !self.entered
                && self.validation.iter().any(|r| {
                    r.purpose == ValidationPurpose::Preflight && r.status == ValidationStatus::Valid
                }),
        )?;
        self.authorized = true;
        self.application = Application::Unknown;
        self.stage = Stage::Authorized;
        Ok(())
    }
    /// Irreversible native rejection before source/history boundary. A timeout or abort is NOT
    /// a discard acknowledgment; this must come from the slot-owning native attempt.
    pub fn discard_before_boundary(
        &mut self,
        reason: Reason,
        witness: NativeWitness,
    ) -> Result<(), EditError> {
        if reason == Reason::Complete {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.require(
            self.authorized
                && !self.entered
                && !self.discarded
                && self.application == Application::Unknown,
        )?;
        self.check_native(&witness)?;
        self.discarded = true;
        self.application = Application::NotApplied;
        self.record_failure(reason);
        Ok(())
    }
    /// Rechecks the unchanged original source, version, save profile and target immediately
    /// before the first effectful native command. Failure after authorization remains unknown
    /// until the native owner irreversibly discards the command before the boundary.
    ///
    /// # Errors
    /// Rejects misbound/clock-invalid evidence; this is a sticky protocol failure.
    pub fn guard_application(
        &mut self,
        observed: ObservationOutcome,
        saved: Option<SavedStateEvidence>,
        disk: Option<DiskMetadata>,
    ) -> Result<(), EditError> {
        self.require(self.authorized && !self.guarded && !self.entered && !self.discarded)?;
        if self.terminal_observation(&observed)? {
            return Ok(());
        }
        let observed_reason = observation_reason(observed.outcome());
        let (evidence, eligible, _) = self.convert(observed, saved, disk, false)?;
        let Some(before) = self.before.as_ref() else {
            return Err(self.reject(EditError::OutOfOrder));
        };
        let original = self.request.expected.source();
        let current = evidence.sources[0].source;
        let before_saved = before.saved_state.as_ref();
        let guard_saved = evidence.saved_state.as_ref();
        let matched = eligible
            && current == Some(*original)
            && evidence.sources[2]
                .identity
                .as_ref()
                .and_then(Witness::source_version)
                == Some(self.request.expected.current_version())
            && before_saved.zip(guard_saved).is_some_and(|(b, a)| {
                a.save_profile == b.save_profile
                    && a.saved_version == b.saved_version
                    && a.current_version == b.current_version
            })
            && before
                .disk_metadata
                .as_ref()
                .zip(evidence.disk_metadata.as_ref())
                .is_some_and(|(b, a)| a.identity == b.identity && a.mtime == b.mtime);
        if matched && self.failure.is_none() {
            self.guarded = true;
        } else {
            self.record_failure(observed_reason.unwrap_or(Reason::RevisionMismatch));
        }
        Ok(())
    }
    /// Record entry immediately BEFORE first native source/history operation.
    pub fn enter_application(&mut self) -> Result<(), EditError> {
        self.require(
            self.failure.is_none()
                && self.authorized
                && self.guarded
                && !self.discarded
                && !self.entered,
        )?;
        self.entered = true;
        self.stage = Stage::Applying;
        self.progress.buffer_application = Step {
            state: StepState::Entered,
            reason: None,
            ..Step::default()
        };
        self.history = History::Unknown;
        Ok(())
    }
    /// A trusted native witness established an actual history or source effect but cannot
    /// establish complete B application. Mere entry, lost reply or a native command's intent
    /// is NOT such evidence.
    pub fn partial_application(
        &mut self,
        reason: Reason,
        witness: NativeWitness,
    ) -> Result<(), EditError> {
        if reason == Reason::Complete {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.require(self.entered && self.progress.buffer_application.state == StepState::Entered)?;
        self.check_native(&witness)?;
        self.application = Application::PartlyApplied;
        self.history = History::NativeComplexEdit;
        self.progress.buffer_application = Step {
            state: StepState::Failed,
            reason: Some(reason),
            collection: Some(witness.collection),
        };
        self.record_failure(reason);
        Ok(())
    }
    /// Native acknowledged a real complex edit (not merely an attempted command).
    pub fn buffer_changed(
        &mut self,
        current_version: DecimalCounter,
        witness: NativeWitness,
    ) -> Result<(), EditError> {
        self.require(self.entered && self.progress.buffer_application.state == StepState::Entered)?;
        self.check_native(&witness)?;
        if current_version == *self.request.expected.current_version() {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.applied_version = Some(current_version);
        self.application = Application::Applied;
        self.history = History::NativeComplexEdit;
        self.progress.buffer_application = Step {
            state: StepState::Completed,
            reason: None,
            collection: Some(witness.collection),
        };
        Ok(())
    }
    /// Mark the resource setter as entered before calling the native operation. A lost
    /// acknowledgment cannot erase the earlier known buffer change.
    pub fn enter_resource_sync(&mut self) -> Result<(), EditError> {
        self.require(
            self.failure.is_none()
                && self.progress.buffer_application.state == StepState::Completed
                && self.progress.resource_sync.state == StepState::NotStarted,
        )?;
        self.progress.resource_sync = Step {
            state: StepState::Entered,
            reason: None,
            ..Step::default()
        };
        Ok(())
    }
    /// Record a lost or indeterminate resource setter result without claiming R readback.
    pub fn resource_sync_unknown(&mut self, reason: Reason) -> Result<(), EditError> {
        if reason == Reason::Complete {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.require(self.progress.resource_sync.state == StepState::Entered)?;
        self.progress.resource_sync = Step {
            state: StepState::Unknown,
            reason: Some(reason),
            ..Step::default()
        };
        self.record_failure(reason);
        Ok(())
    }
    /// Record a completed setter even if an earlier acknowledgment was lost. This supplies
    /// application knowledge, not permission to dispatch a later stage or independent R proof.
    pub fn resource_synced(&mut self, witness: NativeWitness) -> Result<(), EditError> {
        self.require(
            self.entered
                && self.stage <= Stage::Applying
                && self.progress.resource_sync.state != StepState::Completed
                && self.after.is_none(),
        )?;
        self.check_native(&witness)?;
        if self.application != Application::Applied {
            self.application = Application::PartlyApplied;
        }
        self.progress.resource_sync = Step {
            state: StepState::Completed,
            reason: None,
            collection: Some(witness.collection),
        };
        Ok(())
    }
    /// Mark descriptor I/O entered before dispatch, without assuming any bytes changed.
    pub fn enter_persistence(&mut self) -> Result<(), EditError> {
        self.require(
            self.failure.is_none()
                && self.progress.resource_sync.state == StepState::Completed
                && self.progress.buffer_application.state == StepState::Completed
                && self.progress.persistence.state == StepState::NotStarted,
        )?;
        self.stage = Stage::Persisting;
        self.progress.persistence = Step {
            state: StepState::Entered,
            reason: None,
            ..Step::default()
        };
        Ok(())
    }
    /// A lost descriptor reply cannot be treated as a complete persistence receipt.
    pub fn persistence_unknown(&mut self, reason: Reason) -> Result<(), EditError> {
        if reason == Reason::Complete {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.require(self.progress.persistence.state == StepState::Entered)?;
        self.progress.persistence = Step {
            state: StepState::Unknown,
            reason: Some(reason),
            ..Step::default()
        };
        self.record_failure(reason);
        Ok(())
    }
    /// # Errors
    /// Receipt binding and clock must be attributable; incomplete or uncertain writes cannot
    /// justify finalization or success. A write start alone is only potential application.
    /// Receiving evidence is not dispatching another stage: missing earlier acknowledgments
    /// must not erase a subsequently established write/truncate effect.
    pub fn persistence(&mut self, receipt: PersistenceReceipt) -> Result<(), EditError> {
        self.require(self.entered && self.persistence.is_none() && self.after.is_none())?;
        self.check_binding(
            &receipt.request_id,
            &receipt.session_id,
            &receipt.document,
            &receipt.collection,
            true,
        )?;
        if receipt.project_id != *self.target()?.project_file_id()
            || receipt.file_id
                != *self
                    .request
                    .expected
                    .document
                    .disk_file_id()
                    .ok_or(EditError::WrongTarget)?
            || receipt.intended != self.intended
            || receipt.bytes_written > receipt.intended.utf8_bytes
            || !receipt.write_started
                && (receipt.bytes_written != 0 || receipt.truncated || receipt.flushed)
            || receipt.reason == Some(Reason::Complete)
        {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.check_order(&receipt.collection)?;
        self.stage = Stage::Persisting;
        let complete = receipt.complete();
        if (receipt.bytes_written != 0 || receipt.truncated)
            && self.application != Application::Applied
        {
            self.application = Application::PartlyApplied;
        }
        let reason = if complete {
            None
        } else {
            Some(receipt.reason.unwrap_or(if receipt.interference {
                Reason::RevisionMismatch
            } else if !receipt.attached {
                Reason::IdentityChanged
            } else {
                Reason::PersistenceUnknown
            }))
        };
        self.progress.persistence = Step {
            state: if complete {
                StepState::Completed
            } else if reason == Some(Reason::PersistenceUnknown) {
                StepState::Unknown
            } else {
                StepState::Failed
            },
            reason,
            collection: Some(receipt.collection.clone()),
        };
        if let Some(reason) = reason {
            self.record_failure(reason);
        }
        self.persistence = Some(receipt);
        Ok(())
    }
    /// Mark finalization entered; its return is still not a saved-state readback.
    pub fn enter_finalization(&mut self) -> Result<(), EditError> {
        self.require(
            self.failure.is_none()
                && self.progress.persistence.state == StepState::Completed
                && self.progress.buffer_application.state == StepState::Completed
                && self.progress.resource_sync.state == StepState::Completed
                && self.progress.finalization.state == StepState::NotStarted,
        )?;
        self.stage = Stage::Finalizing;
        self.progress.finalization = Step {
            state: StepState::Entered,
            reason: None,
            ..Step::default()
        };
        Ok(())
    }
    pub fn finalization_unknown(&mut self, reason: Reason) -> Result<(), EditError> {
        if reason == Reason::Complete {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.require(self.progress.finalization.state == StepState::Entered)?;
        self.progress.finalization = Step {
            state: StepState::Unknown,
            reason: Some(reason),
            ..Step::default()
        };
        self.record_failure(reason);
        Ok(())
    }
    /// # Errors
    /// Finalization flags describe bookkeeping only, never D/R/B or saved readbacks.
    pub fn finalization(&mut self, result: FinalizationResult) -> Result<(), EditError> {
        self.require(
            self.entered
                && self.finalization.is_none()
                && self.after.is_none()
                && self.stage <= Stage::Finalizing,
        )?;
        self.check_binding(
            &result.request_id,
            &result.session_id,
            &result.document,
            &result.collection,
            true,
        )?;
        if result.status == FinalizationStatus::Rejected && result.steps.any()
            || result.reason == Some(Reason::Complete)
            || result.status == FinalizationStatus::Complete
                && (!result.steps.all()
                    || result.reason.is_some()
                    || result.before_resource_edited.is_none()
                    || result.before_source != Some(self.intended)
                    || result.after_source != Some(self.intended)
                    || result.before_current.is_none()
                    || result.before_current.as_ref()
                        == Some(self.request.expected.current_version())
                    || result.before_current != result.after_current
                    || self
                        .applied_version
                        .as_ref()
                        .is_some_and(|v| result.after_current.as_ref() != Some(v))
                    || result.before_saved.as_ref()
                        != self
                            .before
                            .as_ref()
                            .and_then(|b| b.saved_state.as_ref())
                            .map(|s| &s.saved_version)
                    || result.after_saved != result.after_current
                    || result.before_resource_mtime.as_ref()
                        != self
                            .before
                            .as_ref()
                            .and_then(|b| b.saved_state.as_ref())
                            .map(|s| &s.resource_mtime)
                    || result.before_document_mtime.as_ref()
                        != self
                            .before
                            .as_ref()
                            .and_then(|b| b.saved_state.as_ref())
                            .map(|s| &s.document_mtime)
                    || result.after_resource_mtime.is_none()
                    || result.after_resource_mtime != result.after_document_mtime
                    || self
                        .persistence
                        .as_ref()
                        .is_some_and(|p| result.after_resource_mtime.as_ref() != p.mtime.as_ref())
                    || result.after_resource_edited != Some(false))
        {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        self.check_order(&result.collection)?;
        self.stage = Stage::Finalizing;
        if result.steps.any() && self.application == Application::Unknown {
            self.application = Application::PartlyApplied;
        }
        let complete = result.status == FinalizationStatus::Complete && result.steps.all();
        self.progress.finalization = Step {
            state: if complete {
                StepState::Completed
            } else {
                StepState::Failed
            },
            reason: result.reason,
            ..Step::default()
        };
        if !complete {
            self.record_failure(result.reason.unwrap_or(Reason::PartialFinalization));
        }
        self.progress.finalization.collection = Some(result.collection.clone());
        self.finalization = Some(result);
        Ok(())
    }
    /// Independent final D/R/B and saved inspection. Incomplete evidence is retained as a
    /// nullable/qualified result but cannot satisfy success.
    pub fn verify(
        &mut self,
        observed: ObservationOutcome,
        saved: Option<SavedStateEvidence>,
        disk: Option<DiskMetadata>,
        context: Option<ContextRecheck>,
    ) -> Result<(), EditError> {
        let changed = self
            .intent_changed
            .ok_or_else(|| self.reject(EditError::OutOfOrder))?;
        // Read-only survivor collection is useful even when a preceding effect failed.
        // It never authorizes another effect or substitutes for absent validation.
        self.require(self.after.is_none() && self.before.is_some() && (self.entered || !changed))?;
        if self.terminal_observation(&observed)? {
            return Ok(());
        }
        let observed_reason = observation_reason(observed.outcome());
        let (evidence, eligible, _) = self.convert(observed, saved, disk, true)?;
        if let Some(c) = &context {
            self.check_binding(
                &c.request_id,
                &c.session_id,
                &c.document,
                &c.collection,
                true,
            )?;
            if c.dependencies.len() > 32 {
                return Err(self.reject(EditError::InvalidEvidence));
            }
            self.check_order(&c.collection)?;
        }
        if let Some(reason) = observed_reason {
            self.record_failure(reason);
        }
        self.stage = Stage::Verifying;
        // `convert` compared exact D/R/B bytes before constructing digest summaries.
        let version = evidence.sources[2]
            .identity
            .as_ref()
            .and_then(Witness::source_version);
        let before_saved = self.before.as_ref().and_then(|b| b.saved_state.as_ref());
        let after_saved = evidence.saved_state.as_ref();
        let metadata = evidence.disk_metadata.as_ref();
        let good = eligible
            && self.failure.is_none()
            && self.validation.last().is_some_and(|v| {
                v.status == ValidationStatus::Valid
                    && v.diagnostics_complete
                    && v.purpose
                        == if changed {
                            ValidationPurpose::PostChange
                        } else {
                            ValidationPurpose::Unchanged
                        }
                    && v.context_current
                    && v.dependencies_current
                    && context.as_ref().is_some_and(|c| {
                        c.current && c.context == v.context && c.dependencies == v.dependencies
                    })
                    && (!changed
                        || self.validation.first().is_some_and(|p| {
                            p.purpose == ValidationPurpose::Preflight
                                && p.status == ValidationStatus::Valid
                                && p.context == v.context
                                && p.dependencies == v.dependencies
                        }))
            })
            && before_saved
                .zip(after_saved)
                .zip(metadata)
                .is_some_and(|((b, a), d)| {
                    a.save_profile == b.save_profile
                        && a.saved_version == a.current_version
                        && a.current_version == *version.expect("eligible version")
                        && a.resource_mtime == d.mtime
                        && a.document_mtime == d.mtime
                        && (!changed || a.current_version != b.current_version)
                        && (changed
                            || a.current_version == b.current_version
                                && self
                                    .before
                                    .as_ref()
                                    .and_then(|e| e.disk_metadata.as_ref())
                                    .is_some_and(|old| {
                                        d.identity == old.identity && d.mtime == old.mtime
                                    }))
                })
            && (if changed {
                self.progress.buffer_application.state == StepState::Completed
                    && self.progress.resource_sync.state == StepState::Completed
                    && self.progress.persistence.state == StepState::Completed
                    && self.progress.finalization.state == StepState::Completed
                    && self.history == History::NativeComplexEdit
                    && self.applied_version.as_ref() == version
                    && self.persistence.as_ref().and_then(|p| p.mtime.as_ref())
                        == metadata.map(|d| &d.mtime)
            } else {
                !self.authorized
                    && !self.entered
                    && self.persistence.is_none()
                    && self.finalization.is_none()
                    && self.history == History::NotParticipated
            });
        self.progress.verification = Step {
            state: if good {
                StepState::Completed
            } else {
                StepState::Failed
            },
            reason: (!good).then_some(Reason::IncompleteVerification),
            ..Step::default()
        };
        self.progress.verification.collection = context.as_ref().map(|c| c.collection.clone());
        if !good {
            self.record_failure(Reason::IncompleteVerification);
        }
        self.after = Some(evidence);
        self.context_recheck = context;
        Ok(())
    }
    /// Sticky terminal signal. Denial/ambiguity suppress source-derived result fields even if
    /// known application had already occurred; they do not erase application knowledge.
    pub fn fail(&mut self, reason: Reason, selection: Option<Selection>) -> Result<(), EditError> {
        if reason == Reason::Complete
            || selection.is_some()
                && (reason != Reason::AmbiguousTarget || self.request.session_id.is_some())
        {
            return Err(self.reject(EditError::InvalidEvidence));
        }
        if reason.suppresses_source() || self.failure.is_none() {
            self.failure = Some(reason);
        }
        if reason == Reason::AmbiguousTarget {
            self.selection = selection;
        }
        Ok(())
    }
    /// Consumes state; no late frame can upgrade a terminal result.
    pub fn finish(mut self, interval: ObservationInterval) -> EditOutcome {
        // All stamps, including receipt and validator stamps, must have arrived during this
        // attempt. Clock ticks from the editor are never compared to caller ticks.
        let stamps = self
            .before
            .iter()
            .chain(self.after.iter())
            .flat_map(|e| {
                e.sources
                    .iter()
                    .filter_map(|s| s.collection.as_ref())
                    .chain(e.saved_state.iter().map(|s| &s.collection))
                    .chain(e.disk_metadata.iter().map(|d| &d.collection))
            })
            .chain(self.validation.iter().map(|v| &v.collection))
            .chain(self.context_recheck.iter().map(|c| &c.collection))
            .chain(self.persistence.iter().map(|p| &p.collection))
            .chain(self.finalization.iter().map(|f| &f.collection));
        if self.last_received_us > interval.elapsed_us()
            || stamps
                .into_iter()
                .any(|s| s.received_elapsed_us() > interval.elapsed_us())
        {
            self.record_failure(Reason::ProtocolFailure);
        }
        let verified =
            self.failure.is_none() && self.progress.verification.state == StepState::Completed;
        let reason = if verified {
            Reason::Complete
        } else {
            self.failure
                .filter(|reason| *reason != Reason::Complete)
                .unwrap_or(Reason::IncompleteVerification)
        };
        let outcome = if verified {
            if self.intent_changed == Some(true) {
                EditOutcomeKind::VerifiedChanged
            } else {
                EditOutcomeKind::VerifiedUnchanged
            }
        } else if self.application == Application::Applied
            || self.application == Application::PartlyApplied
        {
            EditOutcomeKind::AppliedUnverified
        } else if self.authorized && !self.discarded {
            EditOutcomeKind::ApplicationUnknown
        } else {
            EditOutcomeKind::Refused
        };
        let suppress = reason.suppresses_source();
        let request = self.request;
        EditOutcome {
            request_id: request.request_id,
            requested_target: (
                request.project_root,
                request.session_id,
                request.script_path,
            ),
            resolved_target: if suppress { None } else { self.target },
            interval,
            outcome,
            reason,
            stage: self.stage,
            application: self.application,
            progress: self.progress,
            expected: (!suppress).then_some(request.expected),
            before: if suppress { None } else { self.before },
            after: if suppress { None } else { self.after },
            persistence: if suppress { None } else { self.persistence },
            finalization: if suppress { None } else { self.finalization },
            validation: if suppress {
                Vec::new()
            } else {
                self.validation
            },
            context_recheck: if suppress { None } else { self.context_recheck },
            history: self.history,
            diagnostics: if reason == Reason::Complete {
                Vec::new()
            } else {
                vec![EditDiagnostic {
                    stage: self.stage,
                    reason,
                }]
            },
            selection: self.selection,
            safe_next_action: if matches!(
                outcome,
                EditOutcomeKind::AppliedUnverified | EditOutcomeKind::ApplicationUnknown
            ) {
                "Observe the explicitly selected target again and inspect human work before any new edit"
            } else {
                reason.action()
            },
        }
    }
}
