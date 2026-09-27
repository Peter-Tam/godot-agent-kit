//! Attempt lifecycle, evidence ordering and terminal reduction; never native execution.

mod observations;

use super::validation::{bound_evidence, check_attribution};
use super::{
    Application, AttemptProgress, ContextRecheck, EditDiagnostic, EditError, EditEvidence,
    EditOutcome, EditOutcomeKind, EditRequest, FinalizationResult, FinalizationStatus, History,
    NativeWitness, PersistenceReceipt, Reason, SourceDigest, Stage, Step, StepState,
    ValidationPurpose, ValidationResult, ValidationStatus,
};
use crate::observation::{
    ClockId, CollectionStamp, DecimalCounter, DocumentIdentity, ObservationInterval, RequestId,
    ResolvedTarget, Selection, SessionId,
};

/// Dispatch/entry lifecycle, independent of observed application and per-step progress.
/// Only an attributable pre-entry discard can retire released authorization as not applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationGate {
    Preparing,
    Authorized,
    Guarded,
    Entered,
    Discarded,
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
    gate: MutationGate,
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
            gate: MutationGate::Preparing,
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
    /// Changed requests require valid preflight. Unchanged requests use a single validation
    /// after preparation and never enter the mutation branch.
    pub fn validation(&mut self, mut result: ValidationResult) -> Result<(), EditError> {
        let changed = self
            .intent_changed
            .ok_or_else(|| self.reject(EditError::OutOfOrder))?;
        let purpose = if changed && self.gate == MutationGate::Preparing {
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
        bound_evidence(&mut result);
        self.check_binding(
            &result.request_id,
            &result.session_id,
            &result.document,
            &result.collection,
            true,
        )?;
        check_attribution(&result, self.request.script_path(), &self.intended)
            .map_err(|error| self.reject(error))?;
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
                && self.gate == MutationGate::Preparing
                && self.validation.iter().any(|r| {
                    r.purpose == ValidationPurpose::Preflight && r.status == ValidationStatus::Valid
                }),
        )?;
        self.gate = MutationGate::Authorized;
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
            matches!(self.gate, MutationGate::Authorized | MutationGate::Guarded)
                && self.application == Application::Unknown,
        )?;
        self.check_native(&witness)?;
        self.gate = MutationGate::Discarded;
        self.application = Application::NotApplied;
        self.record_failure(reason);
        Ok(())
    }
    /// Record entry immediately BEFORE first native source/history operation.
    pub fn enter_application(&mut self) -> Result<(), EditError> {
        self.require(self.failure.is_none() && self.gate == MutationGate::Guarded)?;
        self.gate = MutationGate::Entered;
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
        self.require(
            self.gate == MutationGate::Entered
                && self.progress.buffer_application.state == StepState::Entered,
        )?;
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
        self.require(
            self.gate == MutationGate::Entered
                && self.progress.buffer_application.state == StepState::Entered,
        )?;
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
            self.gate == MutationGate::Entered
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
        self.require(
            self.gate == MutationGate::Entered
                && self.persistence.is_none()
                && self.after.is_none(),
        )?;
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
                    .document()
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
            self.gate == MutationGate::Entered
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
        } else if matches!(
            self.gate,
            MutationGate::Authorized | MutationGate::Guarded | MutationGate::Entered
        ) {
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
