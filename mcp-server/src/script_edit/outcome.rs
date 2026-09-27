//! Public progress, application knowledge and terminal outcome vocabulary.

use super::{
    BasisError, ContextRecheck, EditEvidence, ExpectedRevisionBasis, FinalizationResult,
    PersistenceReceipt, ValidationResult, OPERATION, SCHEMA_VERSION,
};
use crate::observation::{
    CollectionStamp, ObservationInterval, ProjectRoot, RequestId, ResolvedTarget, ResourcePath,
    Selection, SessionId,
};

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
}
impl Reason {
    pub(super) fn suppresses_source(self) -> bool {
        matches!(
            self,
            Self::DeniedAccess | Self::AmbiguousTarget | Self::Busy
        )
    }
    pub(super) fn action(self) -> &'static str {
        match self {
            Self::Complete => "No recovery action required",
            Self::AmbiguousTarget => "Select exactly one session and observe it again",
            Self::DeniedAccess => "Check access without disclosing source",
            _ => "Observe the explicitly selected target again before any new edit",
        }
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
