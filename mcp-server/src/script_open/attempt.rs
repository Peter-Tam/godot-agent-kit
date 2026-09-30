use super::*;
use crate::observation::*;

#[derive(Debug, Clone)]
pub(crate) struct NativeEvidence {
    pub stage: Option<String>,
    pub next: Option<String>,
    pub binding: Option<String>,
    pub compilation: Option<String>,
    pub document: Option<String>,
    pub parse: Option<i64>,
    pub script: Option<DecimalCounter>,
    pub editor: Option<DecimalCounter>,
    pub buffer: Option<DecimalCounter>,
    pub open: Option<bool>,
    pub discard: Option<bool>,
    pub entered: Option<bool>,
    pub mode: Option<String>,
    pub protection: Option<String>,
    pub selection: Option<&'static str>,
}
#[derive(Clone)]
enum Branch {
    Recognition(DocumentIdentity),
    Closed,
}
enum State {
    Accepted,
    Selected,
    Inspected(Branch),
    Preparing,
    Prepared { cached: bool },
    ContextValidating { cached: bool },
    Authorized { next: &'static str },
    Entered { step: &'static str },
    Ready { next: &'static str },
    Verifying(Branch),
    Verified(Branch),
    Terminal,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Effects {
    None,
    Possible,
    Discarded,
    CachePublished,
    SelectionChanged,
    DocumentAssociated,
}
pub(crate) struct Attempt {
    state: State,
    effects: Effects,
    result: OpeningOutcome,
    first_failure: Option<Reason>,
    last_native: Option<NativeEvidence>,
    admitted: Option<(String, usize, FileIdentity)>,
}
impl Attempt {
    pub(crate) fn new(request: OpenRequest, interval: ObservationInterval) -> Self {
        let id = request.request_id().clone();
        Self::initial(Some(request.0), id, interval)
    }
    pub(crate) fn invalid(id: RequestId, interval: ObservationInterval) -> OpeningOutcome {
        Self::initial(None, id, interval).finish(Some(Reason::InvalidRequest), None)
    }
    pub(crate) fn invalid_document_kind(
        request: ObservationRequest,
        interval: ObservationInterval,
    ) -> OpeningOutcome {
        let id = request.request_id().clone();
        Self::initial(Some(request), id, interval).finish(Some(Reason::InvalidDocumentKind), None)
    }
    fn initial(
        request: Option<ObservationRequest>,
        request_id: RequestId,
        interval: ObservationInterval,
    ) -> Self {
        Self {
            state: State::Accepted,
            effects: Effects::None,
            first_failure: None,
            last_native: None,
            admitted: None,
            result: OpeningOutcome {
                request,
                request_id,
                target: None,
                interval,
                kind: Kind::Refused,
                reason: Reason::EvidenceUnavailable,
                stage: OpenStage::Accepted,
                application: Application::NotApplied,
                progress: Progress::default(),
                before: None,
                observation: None,
                resource_edited: ResourceEdited::default(),
                parse: TargetParse::default(),
                protection: ("not_applicable", None),
                history: ("not_applicable", None),
                selection: None,
                diagnostics: Vec::new(),
            },
        }
    }
    pub(crate) fn request(&self) -> &ObservationRequest {
        self.result
            .request
            .as_ref()
            .unwrap_or_else(|| unreachable!("normal attempts have checked intent"))
    }
    pub(crate) fn target(&self) -> Option<&ResolvedTarget> {
        self.result.target.as_ref()
    }
    pub(crate) fn resource_value(&self) -> Option<bool> {
        self.result.resource_edited.value
    }
    pub(crate) fn script_identity(&self) -> Option<&DecimalCounter> {
        self.last_native.as_ref().and_then(|n| n.script.as_ref())
    }
    pub(crate) fn before_mode(&mut self, mode: &'static str) {
        if let Some(before) = &mut self.result.before {
            before.mode = mode;
        }
    }
    pub(crate) fn latest_observation(&mut self, value: Observation) {
        self.result.observation = Some(value);
    }
    pub(crate) fn no_source_context(&mut self) {
        self.result.protection = ("not_applicable", None);
        self.result.history = ("not_applicable", None);
    }
    pub(crate) fn selected(&mut self, target: ResolvedTarget) -> Result<(), Reason> {
        // The authenticated boundary has canonicalized the requested directory
        // and bound its file identity; a project-root alias is not retargeting.
        if !matches!(self.state, State::Accepted)
            || target.request_id() != self.request().request_id()
            || target.script_path() != self.request().script_path()
            || self
                .request()
                .session_id()
                .is_some_and(|s| s != target.session_id())
        {
            return Err(Reason::ProtocolError);
        }
        self.result.target = Some(target);
        self.result.stage = OpenStage::Selected;
        self.state = State::Selected;
        Ok(())
    }
    pub(crate) fn inspected(
        &mut self,
        open: bool,
        identity: Option<DocumentIdentity>,
    ) -> Result<(), Reason> {
        if !matches!(self.state, State::Selected) {
            return Err(Reason::ProtocolError);
        }
        let branch = if open {
            let id = identity.ok_or(Reason::OpenStateUnknown)?;
            if id.script_instance_id().is_none()
                || id.editor_instance_id().is_none()
                || id.buffer_instance_id().is_none()
            {
                return Err(Reason::OpenStateUnknown);
            }
            Branch::Recognition(id)
        } else {
            Branch::Closed
        };
        self.state = State::Inspected(branch);
        if open {
            self.result.stage = OpenStage::Prepared;
        }
        Ok(())
    }
    pub(crate) fn preparation_observation(&mut self, value: Observation) -> Result<(), Reason> {
        if !matches!(
            self.state,
            State::Inspected(_) | State::Prepared { .. } | State::ContextValidating { .. }
        ) {
            return Err(Reason::ProtocolError);
        }
        if self.result.before.is_none() {
            let s = &value.snapshot;
            self.result.before = Some(Before {
                mode: if s.document().open_state().value() == Some(&OpenState::Open) {
                    "open"
                } else if s.sources().resource().availability() == Availability::NotApplicable
                    || s.sources().resource().reason() == Some(SourceReason::ResourceNotLoaded)
                {
                    "closed_uncached"
                } else {
                    "closed_cached"
                },
                document: s.document().clone(),
                sources: [
                    SourceSummary::of(s.sources().disk()),
                    SourceSummary::of(s.sources().resource()),
                    SourceSummary::of(s.sources().buffer()),
                ],
                dirty: s.dirty().clone(),
                resource_edited: self.result.resource_edited.clone(),
            });
        }
        self.result.observation = Some(value);
        Ok(())
    }
    pub(crate) fn begin_preparation(&mut self) -> Result<(), Reason> {
        if !matches!(self.state, State::Inspected(Branch::Closed)) {
            return Err(Reason::ProtocolError);
        }
        self.state = State::Preparing;
        self.result.stage = OpenStage::Prepared;
        Ok(())
    }
    pub(crate) fn prepared(
        &mut self,
        cached: bool,
        hash: String,
        length: usize,
        file: FileIdentity,
    ) -> Result<(), Reason> {
        if !matches!(self.state, State::Preparing) {
            return Err(Reason::ProtocolError);
        }
        self.admitted = Some((hash, length, file));
        self.state = State::Prepared { cached };
        self.result.stage = OpenStage::Prepared;
        self.result.protection = ("unavailable", Some(Reason::EvidenceUnavailable));
        self.result.history = ("unavailable", Some(Reason::EvidenceUnavailable));
        if cached {
            self.result.progress.compilation.state = StepState::NotApplicable;
        }
        Ok(())
    }
    pub(crate) fn context_validating(&mut self) -> Result<(), Reason> {
        let State::Prepared { cached } = self.state else {
            return Err(Reason::ProtocolError);
        };
        self.state = State::ContextValidating { cached };
        self.result.stage = OpenStage::ContextValidating;
        Ok(())
    }
    /// Must be called before the supervisor writes the single effect authorization.
    pub(crate) fn authorize(&mut self) -> Result<(), Reason> {
        let cached = match self.state {
            State::Prepared { cached } | State::ContextValidating { cached } => cached,
            _ => return Err(Reason::ProtocolError),
        };
        if cached != (self.result.progress.compilation.state == StepState::NotApplicable) {
            return Err(Reason::ProtocolError);
        }
        self.state = State::Authorized { next: "bind" };
        self.effects = Effects::Possible;
        self.result.stage = OpenStage::Authorized;
        // Its purpose and original acquisition interval distinguish historical
        // preparation evidence from independently acquired post-open state.
        if let Some(o) = self.result.observation.as_mut() {
            o.purpose = "preparation";
        }
        Ok(())
    }
    pub(crate) fn enter(&mut self, step: &'static str) -> Result<(), Reason> {
        let next = match self.state {
            State::Authorized { next } | State::Ready { next } => next,
            _ => return Err(Reason::ProtocolError),
        };
        if next != step {
            return Err(Reason::ProtocolError);
        }
        self.state = State::Entered { step };
        let (stage, fact) = match step {
            "bind" => (OpenStage::Binding, &mut self.result.progress.binding),
            "compile" => (OpenStage::Compiling, &mut self.result.progress.compilation),
            "open" => (OpenStage::Opening, &mut self.result.progress.document),
            _ => return Err(Reason::ProtocolError),
        };
        fact.state = StepState::Entered;
        self.result.stage = stage;
        if step == "compile" {
            self.result.parse = TargetParse {
                state: "unavailable",
                code: None,
                collection: None,
            };
        }
        Ok(())
    }
    pub(crate) fn progress(
        &mut self,
        native: NativeEvidence,
        stamp: CollectionStamp,
        ready: bool,
    ) -> Result<(), Reason> {
        let State::Entered { step } = self.state else {
            return Err(Reason::ProtocolError);
        };
        let expected = match step {
            "bind" if native.mode.as_deref() == Some("cached") => "open",
            "bind" => "compile",
            "compile" => "open",
            "open" => "",
            _ => return Err(Reason::ProtocolError),
        };
        if ready {
            if native.next.as_deref() != Some(expected) || native.stage.as_deref() != Some(step) {
                return Err(Reason::ProtocolError);
            }
        } else {
            let stage = native.stage.as_deref().ok_or(Reason::ProtocolError)?;
            let prior = match step {
                "bind" => "prepared",
                "compile" => "bind",
                "open" if native.mode.as_deref() == Some("cached") => "bind",
                "open" => "compile",
                _ => return Err(Reason::ProtocolError),
            };
            if stage != step && stage != prior {
                return Err(Reason::ProtocolError);
            }
            if native.discard == Some(true) && native.next.as_deref() != Some("") {
                return Err(Reason::ProtocolError);
            }
        }
        self.record(native, stamp)?;
        if ready {
            self.state = State::Ready {
                next: if step == "open" { "verify" } else { expected },
            };
        }
        Ok(())
    }
    pub(crate) fn record(
        &mut self,
        native: NativeEvidence,
        stamp: CollectionStamp,
    ) -> Result<(), Reason> {
        if stamp.clock_id()
            != &ClockId::Editor(
                self.target()
                    .ok_or(Reason::ProtocolError)?
                    .session_id()
                    .clone(),
            )
        {
            return Err(Reason::ProtocolError);
        }
        if native.entered.is_none() {
            return Err(Reason::ProtocolError);
        }
        if let Some(old) = &self.last_native {
            for (before, after) in [
                (&old.script, &native.script),
                (&old.editor, &native.editor),
                (&old.buffer, &native.buffer),
            ] {
                if before.is_some() && after != before {
                    return Err(Reason::TargetChanged);
                }
            }
            if old.mode.is_some() && old.mode != native.mode {
                return Err(Reason::ProtocolError);
            }
            for (before, after, terminal) in [
                (
                    &old.binding,
                    &native.binding,
                    &[
                        "reused_existing",
                        "new_resource_published",
                        "failed_before_publication",
                    ][..],
                ),
                (
                    &old.compilation,
                    &native.compilation,
                    &[
                        "not_applicable",
                        "completed_valid",
                        "completed_invalid",
                        "unavailable_failed",
                    ][..],
                ),
                (
                    &old.document,
                    &native.document,
                    &["association_obtained", "failed_unverified"][..],
                ),
            ] {
                if before
                    .as_ref()
                    .is_some_and(|v| terminal.contains(&v.as_str()))
                    && after != before
                {
                    return Err(Reason::ProtocolError);
                }
            }
        }
        let fact = |state: StepState, document: bool| Step {
            state,
            reason: None,
            collection: Some(stamp.clone()),
            script: native.script.clone(),
            editor: if document {
                native.editor.clone()
            } else {
                None
            },
            buffer: if document {
                native.buffer.clone()
            } else {
                None
            },
        };
        match native.binding.as_deref() {
            Some("new_resource_published") | Some("reused_existing") => {
                if native.script.is_none() {
                    return Err(Reason::ProtocolError);
                }
                if native.binding.as_deref() == Some("new_resource_published")
                    && self.effects != Effects::DocumentAssociated
                {
                    self.effects = Effects::CachePublished;
                }
                if self.result.progress.binding.state != StepState::Completed {
                    self.result.progress.binding = fact(StepState::Completed, false);
                    self.result.progress.mode = Some(
                        if native.binding.as_deref() == Some("new_resource_published") {
                            "created"
                        } else {
                            "reused"
                        },
                    );
                }
            }
            Some("failed_before_publication") => {
                self.result.progress.binding = fact(StepState::Failed, false)
            }
            Some("unknown") => self.result.progress.binding = fact(StepState::Unknown, false),
            _ => {}
        }
        match native.compilation.as_deref() {
            Some("completed_valid") | Some("completed_invalid") => {
                let code = native.parse.ok_or(Reason::ProtocolError)?;
                if (code == 0) != (native.compilation.as_deref() == Some("completed_valid"))
                    || code != 0 && code != 43
                {
                    return Err(Reason::ProtocolError);
                }
                if self.result.progress.compilation.state != StepState::Completed {
                    self.result.progress.compilation = fact(StepState::Completed, false);
                    self.result.parse = TargetParse {
                        state: if code == 0 { "valid" } else { "invalid" },
                        code: Some(code),
                        collection: Some(stamp.clone()),
                    };
                }
            }
            Some("not_applicable") => {
                self.result.progress.compilation.state = StepState::NotApplicable
            }
            Some("unavailable_failed") | Some("unknown") => {
                self.result.progress.compilation = fact(
                    if native.compilation.as_deref() == Some("unavailable_failed") {
                        StepState::Failed
                    } else {
                        StepState::Unknown
                    },
                    false,
                );
                self.result.parse = TargetParse {
                    state: "unavailable",
                    code: native.parse,
                    collection: Some(stamp.clone()),
                };
            }
            _ => {}
        }
        match native.document.as_deref() {
            Some("association_obtained") => {
                if native.script.is_none()
                    || native.editor.is_none()
                    || native.buffer.is_none()
                    || native.open != Some(true)
                {
                    return Err(Reason::ProtocolError);
                }
                self.effects = Effects::DocumentAssociated;
                if self.result.progress.document.state != StepState::Completed {
                    self.result.progress.document = fact(StepState::Completed, true);
                }
            }
            Some("entered") => self.result.progress.document = fact(StepState::Entered, true),
            Some("failed_unverified") => {
                self.result.progress.document = fact(StepState::Failed, true)
            }
            Some("unknown") => self.result.progress.document = fact(StepState::Unknown, true),
            _ => {}
        }
        if native.discard == Some(true)
            && !matches!(
                self.effects,
                Effects::CachePublished | Effects::SelectionChanged | Effects::DocumentAssociated
            )
            && matches!(
                native.binding.as_deref(),
                Some("not_started" | "reused_existing" | "failed_before_publication")
            )
            && native.document.as_deref() == Some("not_started")
        {
            self.effects = Effects::Discarded;
        }
        self.last_native = Some(native);
        Ok(())
    }
    pub(crate) fn resource(
        &mut self,
        value: Option<bool>,
        stamp: CollectionStamp,
        script: Option<DecimalCounter>,
    ) {
        self.result.resource_edited = match (value, script) {
            (Some(v), Some(id)) => ResourceEdited {
                availability: Availability::Observed,
                value: Some(v),
                collection: Some(stamp),
                witness: Some(Witness::new(
                    self.request().script_path().clone(),
                    Some(id),
                    None,
                    None,
                    None,
                    None,
                )),
                reason: None,
            },
            _ => ResourceEdited::default(),
        };
    }
    pub(crate) fn no_resource(&mut self) {
        self.result.resource_edited = ResourceEdited {
            availability: Availability::NotApplicable,
            value: None,
            collection: None,
            witness: None,
            reason: None,
        };
    }
    pub(crate) fn selection(&mut self, after: &'static str, opening_call: bool) {
        let effect = if opening_call {
            if after == "target" {
                "selected_target"
            } else {
                "unknown"
            }
        } else {
            "none"
        };
        if effect == "selected_target"
            && !matches!(
                self.effects,
                Effects::CachePublished | Effects::DocumentAssociated
            )
        {
            self.effects = Effects::SelectionChanged;
        }
        match &mut self.result.selection {
            Some(s) => {
                s.after = after;
                if opening_call {
                    s.effect = effect;
                }
            }
            None => {
                self.result.selection = Some(SelectionEffect {
                    before: after,
                    after,
                    effect,
                })
            }
        }
    }
    pub(crate) fn closed_worker_before_entry(&mut self) {
        if matches!(self.state, State::Authorized { next: "bind" })
            && self.effects == Effects::Possible
        {
            self.effects = Effects::Discarded;
        }
    }
    pub(crate) fn start_verification(&mut self, purpose: &str) -> Result<(), Reason> {
        let branch = match (&self.state, purpose) {
            (State::Inspected(Branch::Recognition(id)), "recognition") => {
                Branch::Recognition(id.clone())
            }
            (State::Ready { next: "verify" }, "post_open") => Branch::Closed,
            _ => return Err(Reason::ProtocolError),
        };
        self.state = State::Verifying(branch);
        self.result.stage = OpenStage::Verifying;
        self.result.progress.verification.state = StepState::Entered;
        Ok(())
    }
    /// Remember conclusive facts as they arrive; absence waits for final verification.
    pub(crate) fn verification_sample(
        &mut self,
        document: &DocumentState,
        resource: &SourceObservation,
        buffer: &SourceObservation,
        dirty: &DirtyObservation,
    ) -> Result<(), Reason> {
        let State::Verifying(branch) = &self.state else {
            return Err(Reason::ProtocolError);
        };
        if self.first_failure.is_some() {
            return Ok(());
        }
        let identity_changed = match (branch, document.identity()) {
            (Branch::Recognition(old), Some(identity)) => {
                old.script_instance_id() != identity.script_instance_id()
                    || old.editor_instance_id() != identity.editor_instance_id()
                    || old.buffer_instance_id() != identity.buffer_instance_id()
            }
            (Branch::Closed, Some(identity)) => {
                let n = self.last_native.as_ref().ok_or(Reason::ProtocolError)?;
                n.script.as_ref() != identity.script_instance_id()
                    || n.editor.as_ref() != identity.editor_instance_id()
                    || n.buffer.as_ref() != identity.buffer_instance_id()
            }
            (_, None) => false,
        };
        let failure = if identity_changed
            || document.validity().value() == Some(&Validity::Invalid)
            || document.open_state().value() == Some(&OpenState::NotOpen)
            || document.validity().invalidated_evidence().is_some()
            || document.open_state().invalidated_evidence().is_some()
        {
            Some(Reason::TargetChanged)
        } else if matches!(branch, Branch::Recognition(_)) {
            None
        } else if resource.invalidated_evidence().is_some()
            || buffer.invalidated_evidence().is_some()
            || dirty.invalidated_evidence().is_some()
        {
            Some(Reason::SourceChanged)
        } else if resource
            .staleness()
            .and_then(Staleness::evidence)
            .is_some_and(|v| !v.is_empty())
        {
            Some(Reason::StaleResource)
        } else if buffer
            .staleness()
            .and_then(Staleness::evidence)
            .is_some_and(|v| !v.is_empty())
        {
            Some(Reason::StaleBuffer)
        } else if dirty.state() == Some(DirtyState::Dirty)
            || self.result.resource_edited.value == Some(true)
        {
            Some(Reason::DirtyConflict)
        } else if resource
            .text()
            .zip(buffer.text())
            .is_some_and(|(r, b)| r != b)
            || resource
                .text()
                .or(buffer.text())
                .is_some_and(|source| !self.admitted_source_matches(source))
        {
            Some(Reason::VerificationFailed)
        } else {
            None
        };
        if let Some(reason) = failure {
            self.verification_failure(reason);
        }
        Ok(())
    }
    fn admitted_source_matches(&self, source: &str) -> bool {
        self.admitted.as_ref().is_some_and(|(hash, length, _)| {
            if source.len() != *length || hash.len() != 64 {
                return false;
            }
            const HEX: &[u8; 16] = b"0123456789abcdef";
            let digest = ring::digest::digest(&ring::digest::SHA256, source.as_bytes());
            digest
                .as_ref()
                .iter()
                .zip(hash.as_bytes().as_chunks::<2>().0)
                .all(|(byte, pair)| {
                    pair[0] == HEX[(byte >> 4) as usize] && pair[1] == HEX[(byte & 15) as usize]
                })
        })
    }
    pub(crate) fn verification_disk(
        &mut self,
        source: &SourceObservation,
        file: Option<&FileIdentity>,
    ) {
        if !matches!(self.state, State::Verifying(Branch::Closed)) || self.first_failure.is_some() {
            return;
        }
        let file_changed = file
            .or_else(|| source.witness().and_then(Witness::disk_file_id))
            .zip(self.admitted.as_ref().map(|(_, _, file)| file))
            .is_some_and(|(observed, admitted)| observed != admitted);
        let failure = if file_changed {
            Some(Reason::TargetChanged)
        } else if source.invalidated_evidence().is_some() {
            Some(Reason::SourceChanged)
        } else if source
            .text()
            .is_some_and(|s| !self.admitted_source_matches(s))
        {
            Some(Reason::VerificationFailed)
        } else {
            None
        };
        if let Some(reason) = failure {
            self.verification_failure(reason);
        }
    }
    pub(crate) fn verification_recheck(&mut self, recheck: &Recheck) {
        let State::Verifying(branch) = &self.state else {
            return;
        };
        if self.first_failure.is_some() {
            return;
        }
        let changes: &[DetectedChange] = match recheck {
            Recheck::Performed { detected_changes }
            | Recheck::Partial {
                detected_changes, ..
            } => detected_changes,
            Recheck::Unavailable { .. } => &[],
        };
        let failure = changes
            .iter()
            .find_map(|change| match change {
                DetectedChange::DocumentClosed | DetectedChange::DocumentIdentityReplaced => {
                    Some(Reason::TargetChanged)
                }
                DetectedChange::DiskIdentityReplaced if matches!(branch, Branch::Closed) => {
                    Some(Reason::TargetChanged)
                }
                DetectedChange::SessionReplaced => Some(Reason::SessionChanged),
                DetectedChange::SessionEnded => Some(Reason::SessionEnded),
                DetectedChange::Source(_) | DetectedChange::Dirty
                    if matches!(branch, Branch::Closed) =>
                {
                    Some(Reason::SourceChanged)
                }
                _ => None,
            })
            .or_else(|| {
                (matches!(branch, Branch::Closed)
                    && self.result.resource_edited.value == Some(true))
                .then_some(Reason::DirtyConflict)
            });
        if let Some(reason) = failure {
            self.verification_failure(reason);
        }
    }
    fn verification_failure(&mut self, reason: Reason) {
        self.result.progress.verification.state = StepState::Failed;
        self.result
            .progress
            .verification
            .reason
            .get_or_insert(reason);
        self.fail(reason);
    }
    pub(crate) fn verification(
        &mut self,
        value: Observation,
        protection: Option<&str>,
        identity_rechecked: bool,
    ) -> Result<(), Reason> {
        let State::Verifying(branch) = &self.state else {
            return Err(Reason::ProtocolError);
        };
        let branch = branch.clone();
        let s = &value.snapshot;
        let Some(identity) = s.document().identity() else {
            self.result.observation = Some(value);
            self.result.progress.verification.state = StepState::Failed;
            self.fail(Reason::OpenStateUnknown);
            return Ok(());
        };
        let exact =
            s.document().open_state().value() == Some(&OpenState::Open) && identity_rechecked;
        let failure = match &branch {
            Branch::Recognition(old) => {
                if !exact
                    || old.script_instance_id() != identity.script_instance_id()
                    || old.editor_instance_id() != identity.editor_instance_id()
                    || old.buffer_instance_id() != identity.buffer_instance_id()
                {
                    Some(Reason::TargetChanged)
                } else {
                    None
                }
            }
            Branch::Closed => {
                let (_, _, file) = self.admitted.as_ref().ok_or(Reason::ProtocolError)?;
                let n = self.last_native.as_ref().ok_or(Reason::ProtocolError)?;
                if !exact
                    || n.script.as_ref() != identity.script_instance_id()
                    || n.editor.as_ref() != identity.editor_instance_id()
                    || n.buffer.as_ref() != identity.buffer_instance_id()
                    || identity.disk_file_id() != Some(file)
                {
                    Some(Reason::TargetChanged)
                } else if s.consistency().checks() != Checks::Performed {
                    Some(Reason::VerificationIncomplete)
                } else if !s.consistency().detected_changes().is_empty() {
                    Some(Reason::SourceChanged)
                } else if s
                    .sources()
                    .resource()
                    .staleness()
                    .and_then(Staleness::evidence)
                    .is_some_and(|v| !v.is_empty())
                {
                    Some(Reason::StaleResource)
                } else if s
                    .sources()
                    .buffer()
                    .staleness()
                    .and_then(Staleness::evidence)
                    .is_some_and(|v| !v.is_empty())
                {
                    Some(Reason::StaleBuffer)
                } else if [
                    s.sources().disk(),
                    s.sources().resource(),
                    s.sources().buffer(),
                ]
                .iter()
                .any(|source| source.availability() != Availability::Observed)
                {
                    Some(Reason::VerificationIncomplete)
                } else if s.agreement() != Agreement::Agree
                    || s.sources()
                        .disk()
                        .text()
                        .is_none_or(|text| !self.admitted_source_matches(text))
                {
                    Some(Reason::VerificationFailed)
                } else if s.dirty().state() == Some(DirtyState::Dirty)
                    || self.result.resource_edited.value == Some(true)
                {
                    Some(Reason::DirtyConflict)
                } else if s.dirty().state().is_none()
                    || self.result.resource_edited.value.is_none()
                    || protection != Some("unchanged")
                {
                    Some(Reason::VerificationIncomplete)
                } else {
                    None
                }
            }
        };
        self.result.observation = Some(value);
        if matches!(branch, Branch::Closed) && self.result.protection.0 != "invalidated" {
            if protection == Some("unchanged") {
                if self.result.protection.0 != "not_applicable" {
                    self.result.protection = ("preserved", None);
                    self.result.history = ("observed", None);
                }
            } else {
                self.result.protection = ("unavailable", Some(Reason::EvidenceUnavailable));
                self.result.history = ("unavailable", Some(Reason::EvidenceUnavailable));
            }
        }
        if let Some(reason) = failure {
            self.verification_failure(reason);
            return Ok(());
        }
        if self.first_failure.is_some() {
            return Ok(());
        }
        let observed = self
            .result
            .observation
            .as_ref()
            .ok_or(Reason::ProtocolError)?;
        let id = observed
            .snapshot
            .document()
            .identity()
            .ok_or(Reason::OpenStateUnknown)?;
        self.result.progress.verification = Step {
            state: StepState::Completed,
            reason: None,
            collection: observed
                .snapshot
                .document()
                .open_state()
                .collection()
                .cloned(),
            script: id.script_instance_id().cloned(),
            editor: id.editor_instance_id().cloned(),
            buffer: id.buffer_instance_id().cloned(),
        };
        self.result.stage = OpenStage::Verified;
        self.state = State::Verified(branch);
        Ok(())
    }
    pub(crate) fn fail(&mut self, reason: Reason) {
        if self.first_failure.is_none() {
            self.first_failure = Some(reason);
        }
        if reason == Reason::ContextChanged {
            self.result.protection = ("invalidated", Some(reason));
            self.result.history = ("invalidated", Some(reason));
        }
        let step = match self.result.stage {
            OpenStage::Binding => Some(&mut self.result.progress.binding),
            OpenStage::Compiling => Some(&mut self.result.progress.compilation),
            OpenStage::Opening => Some(&mut self.result.progress.document),
            OpenStage::Verifying => Some(&mut self.result.progress.verification),
            _ => None,
        };
        if let Some(step) = step {
            if step.state == StepState::Entered {
                step.state = if self
                    .last_native
                    .as_ref()
                    .is_some_and(|n| n.discard == Some(true))
                {
                    StepState::Failed
                } else {
                    StepState::Unknown
                };
            }
            if matches!(step.state, StepState::Failed | StepState::Unknown) {
                step.reason.get_or_insert(reason);
            }
        }
        if matches!(
            reason,
            Reason::TargetChanged
                | Reason::SessionChanged
                | Reason::SessionEnded
                | Reason::DeniedAccess
                | Reason::OutsideProject
        ) || (self.result.resource_edited.availability == Availability::NotApplicable
            && self.effects == Effects::Possible)
        {
            self.result.resource_edited = ResourceEdited {
                reason: Some(reason),
                ..ResourceEdited::default()
            };
        }
        if self.result.diagnostics.len() < 64 {
            self.result.diagnostics.push(OpenDiagnostic {
                stage: self.result.stage,
                reason,
                code: None,
            });
        }
    }
    pub(crate) fn finish(
        mut self,
        reason: Option<Reason>,
        interval: Option<ObservationInterval>,
    ) -> OpeningOutcome {
        if let Some(reason) = reason {
            self.fail(reason);
        }
        if let Some(interval) = interval {
            self.result.interval = interval;
        }
        if matches!(
            self.effects,
            Effects::Possible
                | Effects::CachePublished
                | Effects::SelectionChanged
                | Effects::DocumentAssociated
        ) {
            if let Some(s) = &mut self.result.selection {
                let no_open_proof = self.last_native.as_ref().is_some_and(|n| {
                    n.discard == Some(true) && n.document.as_deref() == Some("not_started")
                });
                if s.effect == "none" && !no_open_proof {
                    s.effect = "unknown";
                    s.after = "unknown";
                }
            }
        }
        let positive = if self.first_failure.is_none() {
            match &self.state {
                State::Verified(Branch::Recognition(_))
                    if matches!(self.effects, Effects::None | Effects::Discarded) =>
                {
                    Some((
                        Kind::AlreadyOpenUnchanged,
                        Reason::AlreadyOpen,
                        Application::NotApplied,
                    ))
                }
                State::Verified(Branch::Closed) if self.effects == Effects::DocumentAssociated => {
                    Some((
                        Kind::VerifiedNewlyOpened,
                        Reason::Complete,
                        Application::Applied,
                    ))
                }
                _ => None,
            }
        } else {
            None
        };
        let (kind, reason, application) = positive.unwrap_or_else(|| {
            let reason = self.first_failure.unwrap_or(Reason::VerificationIncomplete);
            match self.effects {
                Effects::CachePublished | Effects::SelectionChanged => {
                    (Kind::AppliedUnverified, reason, Application::PartlyApplied)
                }
                Effects::DocumentAssociated => {
                    (Kind::AppliedUnverified, reason, Application::Applied)
                }
                Effects::Possible => (Kind::EffectsUnknown, reason, Application::Unknown),
                Effects::None | Effects::Discarded => {
                    (Kind::Refused, reason, Application::NotApplied)
                }
            }
        });
        self.result.kind = kind;
        self.result.reason = reason;
        self.result.application = application;
        if matches!(
            reason,
            Reason::DeniedAccess | Reason::OutsideProject | Reason::AmbiguousSession
        ) {
            self.result.before = None;
            self.result.observation = None;
        }
        self.state = State::Terminal;
        self.result
    }
}

#[cfg(test)]
#[path = "attempt_tests.rs"]
mod tests;
