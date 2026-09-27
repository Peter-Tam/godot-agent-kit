//! Protocol-independent eligibility and terminal interpretation for one open GDScript edit.
//!
//! An adapter must independently acquire and authenticate every supplied fact. In particular,
//! neither a receipt nor a finalizer response is a readback of D, R, B or saved state. This
//! module does not issue native operations, own a filesystem capability or enforce a deadline.
//!
//! A typed caller extracts [`ExpectedRevisionBasis::from_observation`] from a prior complete
//! observation, constructs [`EditRequest`], and supplies fresh evidence to [`EditAttempt`].
//! Changed intent requires preflight validation, authorization before dispatch, fresh guards
//! before native entry, and separately recorded application, persistence, finalization,
//! post-change validation and independent verification. Unchanged intent uses only unchanged
//! validation and verification. [`EditAttempt::finish`] consumes the attempt.
//!
//! The checked request and basis expose intent and observation provenance to external Rust
//! callers. Evidence records and their fields are public so trusted integrations can supply
//! actual observations and consumers can inspect missing, invalidated or partial facts.
//! [`Stage`], [`AttemptProgress`], [`Application`] and [`EditOutcomeKind`] describe distinct
//! facts; the public outcome vocabulary is not an authorization or mutation API.
//! All of these types are available here; the responsibility modules are private.

mod attempt;
mod evidence;
mod outcome;
mod request;
mod validation;

pub use attempt::EditAttempt;
pub use evidence::{
    Bookkeeping, DiskMetadata, EditEvidence, FinalizationResult, FinalizationStatus,
    InvalidatedSurface, NativeWitness, PersistenceReceipt, SavedStateEvidence, SurfaceEvidence,
};
pub use outcome::{
    Application, AttemptProgress, EditDiagnostic, EditOutcome, EditOutcomeKind, History, Reason,
    Stage, Step, StepState,
};
pub use request::{
    BasisError, EditRequest, ExpectedRevisionBasis, ReplacementSource, SourceDigest,
};
pub use validation::{
    ContextRecheck, DependencyWitness, DiagnosticOrigin, ValidationDiagnostic, ValidationPurpose,
    ValidationResult, ValidationStatus,
};

use std::fmt;

const SCHEMA_VERSION: u32 = 1;
const OPERATION: &str = "edit_open_gdscript";

/// Checked request or integration-evidence failure returned to typed callers.
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
