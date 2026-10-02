use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Effects {
    None,
    Possible,
    Discarded,
    Selection,
    Removed,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Accepted,
    Selected,
    Inspected,
    Prepared,
    Validated,
    Authorized,
    Closing,
    Settling,
    Verifying,
    Verified(bool),
}
pub(crate) struct Attempt {
    pub request: CloseRequest,
    pub result: ClosingOutcome,
    effects: Effects,
    failure: Option<&'static str>,
    source_scope_invalid: bool,
    state: State,
}
impl Attempt {
    pub fn new(request: CloseRequest, interval: ObservationInterval) -> Self {
        let result = ClosingOutcome {
            request: Some(request.observation().clone()),
            request_id: request.request_id().clone(),
            target: None,
            interval,
            kind: "refused",
            reason: "evidence_unavailable",
            stage: "accepted",
            application: "not_applied",
            expected: None,
            progress: Progress::default(),
            before: None,
            observation: None,
            resource: ResourceState::default(),
            protection: Protection::default(),
            selection: None,
            history: History::default(),
            diagnostics: Vec::new(),
        };
        Self {
            request,
            result,
            effects: Effects::None,
            failure: None,
            source_scope_invalid: false,
            state: State::Accepted,
        }
    }
    pub fn fail(&mut self, reason: &'static str) {
        self.source_scope_invalid |= matches!(
            reason,
            "denied_access"
                | "outside_project"
                | "ambiguous_session"
                | "session_changed"
                | "target_changed"
        );
        if self.failure.is_none() {
            self.failure = Some(reason);
        }
    }
    pub fn stage(&mut self, stage: &'static str) {
        let next = match stage {
            "selected" => State::Selected,
            "inspected" => State::Inspected,
            "prepared" => State::Prepared,
            "validated" => State::Validated,
            "authorized" => State::Authorized,
            "closing" => State::Closing,
            "settling" => State::Settling,
            "verifying" => State::Verifying,
            _ => {
                self.fail("protocol_error");
                return;
            }
        };
        if self.state != next
            && !matches!(
                (self.state, next),
                (State::Accepted, State::Selected)
                    | (State::Selected, State::Inspected)
                    | (State::Inspected, State::Prepared)
                    | (State::Prepared, State::Validated)
                    | (State::Validated, State::Authorized)
                    | (State::Authorized, State::Closing)
                    | (State::Closing, State::Settling)
                    | (State::Settling, State::Verifying)
                    | (State::Inspected, State::Verifying)
            )
        {
            self.fail("protocol_error");
            return;
        }
        self.state = next;
        self.result.stage = stage;
        let step = match stage {
            "closing" => Some(&mut self.result.progress.closing),
            "settling" => Some(&mut self.result.progress.native_revalidation),
            "verifying" => Some(&mut self.result.progress.verification),
            _ => None,
        };
        if let Some(step) = step {
            if step.state == "not_started" {
                step.state = "entered";
            }
        }
    }
    pub fn authorize(&mut self) -> Result<(), &'static str> {
        if self.state != State::Validated || self.failure.is_some() {
            return Err("protocol_error");
        }
        self.effects = Effects::Possible;
        // Preparation remains available under `before`, not as a post-effect
        // assertion when the authorization reply may be lost.
        self.result.resource = ResourceState::default();
        self.result.protection = Protection {
            status: "unavailable",
            revalidation: "unavailable",
            required_count: None,
            completed_count: None,
            reason: Some("verification_incomplete"),
        };
        self.result.history.target_buffer = "unavailable";
        self.result.history.unrelated = "unavailable";
        self.result.history.reason = Some("verification_incomplete");
        if let Some(selection) = &mut self.result.selection {
            selection.after = "unknown";
            selection.request_effect = "unknown";
        }
        for step in [
            &mut self.result.progress.closing,
            &mut self.result.progress.native_revalidation,
        ] {
            *step = Step {
                state: "unknown",
                reason: Some("evidence_unavailable"),
                collection: None,
            };
        }
        self.stage("authorized");
        Ok(())
    }
    pub fn native(&mut self, native: NativeEvidence) {
        if native.entered && self.state == State::Authorized {
            self.stage("closing");
        }
        if native.removed == Some(true) || native.target_buffer == "disposed" {
            self.effects = Effects::Removed;
        } else if native
            .selection
            .as_ref()
            .is_some_and(|s| s.request_effect == "native_fallback")
            && self.effects != Effects::Removed
        {
            self.effects = Effects::Selection;
        } else if native.discard
            && !native.entered
            && matches!(self.effects, Effects::None | Effects::Possible)
        {
            self.effects = Effects::Discarded;
            self.result.progress.closing = Step::default();
        }
        if native.invalidated {
            self.fail("verification_failed");
        }
        if let Some(selection) = native.selection {
            self.result.selection = Some(selection);
        }
        if let Some(protection) = native.protection {
            if protection.status == "preserved" {
                self.result.history.unrelated = "observed";
            }
            if protection.status == "invalidated" {
                self.result.history.unrelated = "invalidated";
            }
            self.result.protection = protection;
        }
        self.result.history.target_buffer = native.target_buffer;
        self.result.history.reason = if self.result.history.target_buffer == "unavailable"
            || self.result.history.unrelated == "unavailable"
        {
            Some("evidence_unavailable")
        } else {
            None
        };
        if native.entered {
            let step = Step {
                state: if native.returned.is_some() {
                    "completed"
                } else {
                    "entered"
                },
                reason: None,
                collection: native.returned.or(native.entry),
            };
            record_step(&mut self.result.progress.closing, step);
        }
        if let Some(step) = native.continuation {
            record_step(&mut self.result.progress.native_revalidation, step);
        }
    }
    pub fn complete(&mut self, recognition: bool) {
        if self.state != State::Verifying {
            self.fail("protocol_error");
            return;
        }
        self.state = State::Verified(recognition);
        self.result.progress.verification.state = "completed";
        if recognition {
            self.result.progress.closing.state = "not_applicable";
            self.result.progress.native_revalidation.state = "not_applicable";
        }
    }
    pub fn finish(mut self, interval: ObservationInterval) -> ClosingOutcome {
        let (kind, application, positive) = match self.effects {
            Effects::None | Effects::Discarded
                if self.state == State::Verified(true) && self.failure.is_none() =>
            {
                (
                    "already_closed_unchanged",
                    "not_applied",
                    Some("already_closed"),
                )
            }
            Effects::Removed if self.state == State::Verified(false) && self.failure.is_none() => {
                ("verified_newly_closed", "applied", Some("complete"))
            }
            Effects::Removed => ("applied_unverified", "applied", None),
            Effects::Selection => ("applied_unverified", "partly_applied", None),
            Effects::Possible => ("effects_unknown", "unknown", None),
            _ => ("refused", "not_applied", None),
        };
        self.result.kind = kind;
        self.result.application = application;
        self.result.reason = self
            .failure
            .or(positive)
            .unwrap_or("verification_incomplete");
        self.result.interval = interval;
        if let Some(reason) = self.failure {
            self.result
                .diagnostics
                .push((self.result.stage, reason, None));
            for step in [
                &mut self.result.progress.closing,
                &mut self.result.progress.native_revalidation,
                &mut self.result.progress.verification,
            ] {
                if step.state == "entered" {
                    step.state = "failed";
                    step.reason = Some(reason);
                }
            }
        }
        if self.source_scope_invalid {
            self.result.expected = None;
            self.result.before = None;
            self.result.observation = None;
        }
        self.result
    }
}

fn record_step(current: &mut Step, incoming: Step) {
    let same_event = current.state == incoming.state
        && current.reason == incoming.reason
        && current
            .collection
            .as_ref()
            .zip(incoming.collection.as_ref())
            .is_some_and(|(old, new)| {
                old.clock_id() == new.clock_id()
                    && old.started_tick_us() == new.started_tick_us()
                    && old.finished_tick_us() == new.finished_tick_us()
                    && old.received_elapsed_us() <= new.received_elapsed_us()
            });
    // Later envelopes repeat historical native events. Retain their first real
    // receipt instead of making a close appear later than its own verification.
    if !same_event {
        *current = incoming;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn attempt() -> Attempt {
        let mut a = Attempt::new(
            CloseRequest::new(
                RequestId::new("close-test").unwrap(),
                ProjectRoot::new("/tmp/close-test").unwrap(),
                None,
                ResourcePath::new("res://subject.gd").unwrap(),
                None,
            )
            .unwrap(),
            ObservationInterval::new(1, 1, 0),
        );
        for stage in ["selected", "inspected", "prepared", "validated"] {
            a.stage(stage);
        }
        a
    }
    fn native() -> NativeEvidence {
        NativeEvidence {
            entered: true,
            removed: None,
            discard: false,
            invalidated: false,
            target_buffer: "unavailable",
            selection: None,
            protection: None,
            continuation: None,
            entry: None,
            returned: None,
        }
    }
    #[test]
    fn authorization_without_ack_is_unknown() {
        let mut a = attempt();
        a.result.resource.state = "retained";
        a.result.protection.status = "preserved";
        a.result.history.target_buffer = "retained";
        a.result.selection = Some(Selection {
            before: "target",
            after: "target",
            request_effect: "none",
        });
        a.authorize().unwrap();
        a.fail("timeout");
        let outcome = a.finish(ObservationInterval::new(1, 2, 1000));
        assert_eq!(outcome.application(), "unknown");
        assert_eq!(outcome.resource.state, "not_collected");
        assert_eq!(outcome.protection.status, "unavailable");
        assert_eq!(outcome.history.target_buffer, "unavailable");
        assert_eq!(outcome.selection.as_ref().unwrap().after, "unknown");
        assert_eq!(
            outcome.selection.as_ref().unwrap().request_effect,
            "unknown"
        );
        assert_eq!(outcome.progress.closing.state, "unknown");
    }
    #[test]
    fn known_removal_survives_protocol_loss() {
        let mut a = attempt();
        a.authorize().unwrap();
        let mut n = native();
        n.removed = Some(true);
        a.native(n);
        a.fail("protocol_error");
        let result = a.finish(ObservationInterval::new(1, 2, 1000));
        assert_eq!(result.outcome(), "applied_unverified");
        assert_eq!(result.application(), "applied");
    }
    #[test]
    fn invalidation_prevents_late_upgrade() {
        let mut a = attempt();
        a.authorize().unwrap();
        let mut n = native();
        n.removed = Some(true);
        n.invalidated = true;
        a.native(n);
        a.stage("settling");
        a.stage("verifying");
        a.complete(false);
        assert_eq!(
            a.finish(ObservationInterval::new(1, 2, 1000)).outcome(),
            "applied_unverified"
        );
    }
    #[test]
    fn first_cause_is_sticky() {
        let mut a = attempt();
        a.fail("dirty_conflict");
        a.fail("timeout");
        assert_eq!(
            a.finish(ObservationInterval::new(1, 2, 1000)).reason(),
            "dirty_conflict"
        );
    }
    #[test]
    fn discard_cannot_erase_known_selection() {
        let mut a = attempt();
        a.authorize().unwrap();
        let mut n = native();
        n.selection = Some(Selection {
            before: "target",
            after: "other",
            request_effect: "native_fallback",
        });
        a.native(n);
        let mut n = native();
        n.entered = false;
        n.discard = true;
        a.native(n);
        assert_eq!(
            a.finish(ObservationInterval::new(1, 2, 1000)).application(),
            "partly_applied"
        );
    }
    #[test]
    fn attributable_preentry_discard_establishes_no_effect() {
        let mut a = attempt();
        a.authorize().unwrap();
        let mut n = native();
        n.entered = false;
        n.discard = true;
        a.native(n);
        a.fail("cancelled");
        assert_eq!(
            a.finish(ObservationInterval::new(1, 2, 1000)).application(),
            "not_applied"
        );
    }
    #[test]
    fn native_removal_alone_never_establishes_verified_closure() {
        let mut a = attempt();
        a.authorize().unwrap();
        let mut n = native();
        n.removed = Some(true);
        a.native(n);
        assert_eq!(
            a.finish(ObservationInterval::new(1, 2, 1000)).outcome(),
            "applied_unverified"
        );
    }
    #[test]
    fn recognition_cannot_follow_effect_authorization() {
        let mut a = attempt();
        a.authorize().unwrap();
        a.stage("verifying");
        a.complete(true);
        let out = a.finish(ObservationInterval::new(1, 2, 1000));
        assert_eq!(out.outcome(), "effects_unknown");
        assert_eq!(out.reason(), "protocol_error");
    }
    #[test]
    fn a_preentry_failure_prevents_authorization() {
        let mut a = attempt();
        a.fail("dirty_conflict");
        assert!(a.authorize().is_err());
        let out = a.finish(ObservationInterval::new(1, 2, 1000));
        assert_eq!(out.application(), "not_applied");
        assert_eq!(out.reason(), "dirty_conflict");
    }
    #[test]
    fn repeated_native_facts_preserve_receipt_chronology_but_new_events_advance() {
        let collection = |tick: &str, receipt| {
            CollectionStamp::new(
                ClockId::Editor(SessionId::new("00112233445566778899aabbccddeeff").unwrap()),
                DecimalCounter::new(tick).unwrap(),
                DecimalCounter::new(tick).unwrap(),
                receipt,
            )
            .unwrap()
        };
        let event = |receipt, completion| {
            let mut n = native();
            n.entry = Some(collection("10", receipt));
            n.returned = Some(collection("20", receipt));
            n.continuation = Some(Step {
                state: "completed",
                reason: None,
                collection: Some(collection(completion, receipt)),
            });
            n
        };
        let mut a = attempt();
        a.authorize().unwrap();
        a.native(event(50, "30"));
        a.native(event(80, "30"));
        assert_eq!(
            a.result
                .progress
                .closing
                .collection
                .as_ref()
                .unwrap()
                .received_elapsed_us(),
            50
        );
        assert_eq!(
            a.result
                .progress
                .native_revalidation
                .collection
                .as_ref()
                .unwrap()
                .received_elapsed_us(),
            50
        );
        a.native(event(90, "40"));
        assert_eq!(
            a.result
                .progress
                .closing
                .collection
                .as_ref()
                .unwrap()
                .received_elapsed_us(),
            50
        );
        assert_eq!(
            a.result
                .progress
                .native_revalidation
                .collection
                .as_ref()
                .unwrap()
                .received_elapsed_us(),
            90
        );
    }
}
