//! Independent observation eligibility, revision guards and postcondition verification.

use super::{EditAttempt, MutationGate};
use crate::observation::{
    Agreement, Availability, Checks, ClockId, DecimalCounter, DirtyState, ObservationOutcome,
    OutcomeKind, SourceObservation, Stability, Witness,
};
use crate::script_edit::{
    evidence::surface, ContextRecheck, DiskMetadata, EditError, EditEvidence, History, Reason,
    SavedStateEvidence, SourceDigest, Stage, Step, StepState, ValidationPurpose, ValidationStatus,
};

impl EditAttempt {
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
    fn check_disk(&mut self, disk: &DiskMetadata) -> Result<(), EditError> {
        self.check_stamp(&disk.collection, false)?;
        if disk.identity
            != *self
                .request
                .expected
                .document()
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
            && saved.as_ref().zip(disk.as_ref()).is_some_and(|(s, d)| {
                Some(&s.current_version) == b_version
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
        self.require(self.gate == MutationGate::Authorized)?;
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
            self.gate = MutationGate::Guarded;
        } else {
            self.record_failure(observed_reason.unwrap_or(Reason::RevisionMismatch));
        }
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
        self.require(
            self.after.is_none()
                && self.before.is_some()
                && (self.gate == MutationGate::Entered || !changed),
        )?;
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
                        && Some(&a.current_version) == version
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
                self.gate == MutationGate::Preparing
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
