use crate::observation::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OpenStage {
    Accepted,
    Selected,
    Prepared,
    ContextValidating,
    Authorized,
    Binding,
    Compiling,
    Opening,
    Verifying,
    Verified,
}
impl OpenStage {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Selected => "selected",
            Self::Prepared => "prepared",
            Self::ContextValidating => "context_validating",
            Self::Authorized => "authorized",
            Self::Binding => "binding",
            Self::Compiling => "compiling",
            Self::Opening => "opening",
            Self::Verifying => "verifying",
            Self::Verified => "verified",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    VerifiedNewlyOpened,
    AlreadyOpenUnchanged,
    Refused,
    AppliedUnverified,
    EffectsUnknown,
}
impl Kind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::VerifiedNewlyOpened => "verified_newly_opened",
            Self::AlreadyOpenUnchanged => "already_open_unchanged",
            Self::Refused => "refused",
            Self::AppliedUnverified => "applied_unverified",
            Self::EffectsUnknown => "effects_unknown",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Application {
    NotApplied,
    Applied,
    PartlyApplied,
    Unknown,
}
impl Application {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::NotApplied => "not_applied",
            Self::Applied => "applied",
            Self::PartlyApplied => "partly_applied",
            Self::Unknown => "unknown",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reason {
    Complete,
    AlreadyOpen,
    InvalidRequest,
    AmbiguousSession,
    SessionEnded,
    SessionChanged,
    EditorUnavailable,
    Disconnected,
    DeniedAccess,
    OutsideProject,
    MissingScript,
    InvalidDocumentKind,
    UnsupportedRepresentation,
    UnsupportedCapability,
    OpenStateUnknown,
    CachedSourceConflict,
    DirtyConflict,
    StaleResource,
    StaleBuffer,
    EvidenceUnavailable,
    UnsafeEditorContext,
    CurrentParseInvalid,
    CurrentValidationUnavailable,
    TargetChanged,
    SourceChanged,
    RevisionChanged,
    ContextChanged,
    Busy,
    ProtocolError,
    NativeFailure,
    CompilationUnavailable,
    VerificationIncomplete,
    VerificationFailed,
    Timeout,
    Cancelled,
}
impl Reason {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::AlreadyOpen => "already_open",
            Self::InvalidRequest => "invalid_request",
            Self::AmbiguousSession => "ambiguous_session",
            Self::SessionEnded => "session_ended",
            Self::SessionChanged => "session_changed",
            Self::EditorUnavailable => "editor_unavailable",
            Self::Disconnected => "disconnected",
            Self::DeniedAccess => "denied_access",
            Self::OutsideProject => "outside_project",
            Self::MissingScript => "missing_script",
            Self::InvalidDocumentKind => "invalid_document_kind",
            Self::UnsupportedRepresentation => "unsupported_representation",
            Self::UnsupportedCapability => "unsupported_capability",
            Self::OpenStateUnknown => "open_state_unknown",
            Self::CachedSourceConflict => "cached_source_conflict",
            Self::DirtyConflict => "dirty_conflict",
            Self::StaleResource => "stale_resource",
            Self::StaleBuffer => "stale_buffer",
            Self::EvidenceUnavailable => "evidence_unavailable",
            Self::UnsafeEditorContext => "unsafe_editor_context",
            Self::CurrentParseInvalid => "current_parse_invalid",
            Self::CurrentValidationUnavailable => "current_validation_unavailable",
            Self::TargetChanged => "target_changed",
            Self::SourceChanged => "source_changed",
            Self::RevisionChanged => "revision_changed",
            Self::ContextChanged => "context_changed",
            Self::Busy => "busy",
            Self::ProtocolError => "protocol_error",
            Self::NativeFailure => "native_failure",
            Self::CompilationUnavailable => "compilation_unavailable",
            Self::VerificationIncomplete => "verification_incomplete",
            Self::VerificationFailed => "verification_failed",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepState {
    NotStarted,
    NotApplicable,
    Entered,
    Completed,
    Failed,
    Unknown,
}
impl StepState {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::NotApplicable => "not_applicable",
            Self::Entered => "entered",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Step {
    pub state: StepState,
    pub reason: Option<Reason>,
    pub collection: Option<CollectionStamp>,
    pub script: Option<DecimalCounter>,
    pub editor: Option<DecimalCounter>,
    pub buffer: Option<DecimalCounter>,
}
impl Default for Step {
    fn default() -> Self {
        Self {
            state: StepState::NotStarted,
            reason: None,
            collection: None,
            script: None,
            editor: None,
            buffer: None,
        }
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct Progress {
    pub binding: Step,
    pub mode: Option<&'static str>,
    pub compilation: Step,
    pub document: Step,
    pub verification: Step,
}
#[derive(Debug, Clone)]
pub(crate) struct ResourceEdited {
    pub availability: Availability,
    pub value: Option<bool>,
    pub collection: Option<CollectionStamp>,
    pub witness: Option<Witness>,
    pub reason: Option<Reason>,
}
impl Default for ResourceEdited {
    fn default() -> Self {
        Self {
            availability: Availability::Unavailable,
            value: None,
            collection: None,
            witness: None,
            reason: Some(Reason::EvidenceUnavailable),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct SourceSummary {
    pub authority: Authority,
    pub availability: Availability,
    pub digest: Option<(String, usize)>,
    pub reason: Option<SourceReason>,
    pub collection: Option<CollectionStamp>,
    pub witness: Option<Witness>,
    pub staleness: Option<Staleness>,
    pub invalidated: Option<(
        String,
        usize,
        CollectionStamp,
        Witness,
        Staleness,
        SourceReason,
    )>,
}
impl SourceSummary {
    pub(crate) fn of(s: &SourceObservation) -> Self {
        let digest = |text: &str| {
            (
                crate::project_fs_validation::hex_sha256(text.as_bytes()),
                text.len(),
            )
        };
        Self {
            authority: s.authority(),
            availability: s.availability(),
            digest: s.text().map(digest),
            reason: s.reason(),
            collection: s.collection().cloned(),
            witness: s.witness().cloned(),
            staleness: s.staleness().cloned(),
            invalidated: s.invalidated_evidence().map(|v| {
                let (h, n) = digest(v.text());
                (
                    h,
                    n,
                    v.collection().clone(),
                    v.witness().clone(),
                    v.staleness().clone(),
                    v.reason(),
                )
            }),
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Before {
    pub mode: &'static str,
    pub document: DocumentState,
    pub sources: [SourceSummary; 3],
    pub dirty: DirtyObservation,
    pub resource_edited: ResourceEdited,
}
#[derive(Debug, Clone)]
pub(crate) struct Observation {
    pub purpose: &'static str,
    pub interval: ObservationInterval,
    pub snapshot: ObservationSnapshot,
}
#[derive(Debug, Clone)]
pub(crate) struct TargetParse {
    pub state: &'static str,
    pub code: Option<i64>,
    pub collection: Option<CollectionStamp>,
}
impl Default for TargetParse {
    fn default() -> Self {
        Self {
            state: "not_collected",
            code: None,
            collection: None,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct SelectionEffect {
    pub before: &'static str,
    pub after: &'static str,
    pub effect: &'static str,
}
#[derive(Debug, Clone)]
pub(crate) struct OpenDiagnostic {
    pub stage: OpenStage,
    pub reason: Reason,
    pub code: Option<i64>,
}

/// Immutable opening evidence. This is not an edit basis or lifecycle permission.
#[derive(Debug)]
pub struct OpeningOutcome {
    pub(crate) request: Option<ObservationRequest>,
    pub(crate) request_id: RequestId,
    pub(crate) target: Option<ResolvedTarget>,
    pub(crate) interval: ObservationInterval,
    pub(crate) kind: Kind,
    pub(crate) reason: Reason,
    pub(crate) stage: OpenStage,
    pub(crate) application: Application,
    pub(crate) progress: Progress,
    pub(crate) before: Option<Before>,
    pub(crate) observation: Option<Observation>,
    pub(crate) resource_edited: ResourceEdited,
    pub(crate) parse: TargetParse,
    pub(crate) protection: (&'static str, Option<Reason>),
    pub(crate) history: (&'static str, Option<Reason>),
    pub(crate) selection: Option<SelectionEffect>,
    pub(crate) diagnostics: Vec<OpenDiagnostic>,
}
impl OpeningOutcome {
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn outcome(&self) -> &'static str {
        self.kind.name()
    }
    pub fn reason(&self) -> &'static str {
        self.reason.name()
    }
    pub fn application(&self) -> &'static str {
        self.application.name()
    }
    pub fn target(&self) -> Option<&ResolvedTarget> {
        self.target.as_ref()
    }
    pub fn snapshot(&self) -> Option<&ObservationSnapshot> {
        self.observation.as_ref().map(|o| &o.snapshot)
    }
    pub(crate) fn next_action(&self) -> &'static str {
        match self.kind {
        Kind::VerifiedNewlyOpened=>"Observe the explicitly selected target again before any edit",
        Kind::AlreadyOpenUnchanged=>"The observed dirty or limited state remains; obtain a new valid observation basis before editing",
        Kind::Refused=>"Resolve the reported refusal, then freshly observe the explicit original target before another intentional operation",
        Kind::AppliedUnverified|Kind::EffectsUnknown=>"Freshly observe the explicit original target and inspect human work before another intentional operation",
    }
    }
}
