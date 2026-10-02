//! Private supervised channel, frame ordering and attributable failure evidence.
use super::*;
pub(super) struct Channel {
    pub(super) socket: UnixStream,
    frames: Frames,
    pub(super) clock: AttemptClock,
    expiry: Option<DecimalCounter>,
    previous: Option<CollectionStamp>,
    entered: bool,
    removed: bool,
    invalidated: bool,
    protected_editors: Option<std::collections::BTreeSet<String>>,
    admitted_target: Option<DocumentIdentity>,
    visits: std::collections::BTreeMap<String, (CollectionStamp, Option<CollectionStamp>)>,
}
impl Channel {
    pub(super) fn new(socket: UnixStream, clock: AttemptClock) -> Self {
        Self {
            socket,
            clock,
            frames: Frames::default(),
            expiry: None,
            previous: None,
            entered: false,
            removed: false,
            invalidated: false,
            protected_editors: None,
            admitted_target: None,
            visits: Default::default(),
        }
    }

    pub(super) fn admit(
        &mut self,
        target: DocumentIdentity,
        editors: std::collections::BTreeSet<String>,
        prepared: &codec::Reply,
    ) -> Result<(), &'static str> {
        if self.admitted_target.is_some()
            || prepared
                .native
                .as_ref()
                .is_none_or(|native| !same_document(native, &target))
        {
            return Err("protocol_error");
        }
        self.admitted_target = Some(target);
        self.protected_editors = Some(editors);
        Ok(())
    }
    pub(super) fn next(&mut self, cancelled: &AtomicBool) -> Result<Vec<u8>, &'static str> {
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err("cancelled");
            }
            if self.clock.started.elapsed() >= BUDGET {
                return Err("timeout");
            }
            match self.frames.next(&mut self.socket) {
                Ok(Some(bytes)) => return Ok(bytes),
                Ok(None) => thread::sleep(POLL_INTERVAL),
                Err(e) => return Err(reason(&e)),
            }
        }
    }
    pub(super) fn send(
        &mut self,
        bytes: &[u8],
        cancelled: &AtomicBool,
    ) -> Result<(), &'static str> {
        let h = (bytes.len() as u32).to_be_bytes();
        for part in [&h[..], bytes] {
            let mut offset = 0;
            while offset < part.len() {
                if cancelled.load(Ordering::Relaxed) {
                    return Err("cancelled");
                }
                if self.clock.started.elapsed() >= BUDGET {
                    return Err("timeout");
                }
                match self.socket.write(&part[offset..]) {
                    Ok(0) => return Err("disconnected"),
                    Ok(n) => offset += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL_INTERVAL),
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(_) => return Err("disconnected"),
                }
            }
        }
        Ok(())
    }
    pub(super) fn control(
        &mut self,
        request: &CloseRequest,
        kind: &str,
        args: &[Value],
        cancelled: &AtomicBool,
    ) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(
            &json!({"v":1,"kind":kind,"request_id":request.request_id().as_str(),"arguments":args}),
        )
        .map_err(|_| "protocol_error")?;
        if bytes.len() > 4096 {
            return Err("protocol_error");
        }
        self.send(&bytes, cancelled)
    }
    pub(super) fn event(
        &mut self,
        request: &CloseRequest,
        target: Option<&ResolvedTarget>,
        cancelled: &AtomicBool,
    ) -> Result<Event, &'static str> {
        let bytes = self.next(cancelled)?;
        let event = wire::decode_event(
            &bytes,
            request.observation(),
            target,
            self.clock.elapsed_us(),
        )
        .map_err(|e| reason(&e))?;
        if let Event::Failed(e) = event {
            return Err(reason(&e));
        }
        Ok(event)
    }
    fn receive_reply(
        &mut self,
        attempt: &mut Attempt,
        target: &ResolvedTarget,
        root: &str,
        kind: &str,
        original: Option<&CollectionStamp>,
        cancelled: &AtomicBool,
    ) -> Result<codec::Reply, &'static str> {
        let bytes = self.next(cancelled)?;
        if let Ok(Event::Failed(e)) = wire::decode_event(
            &bytes,
            attempt.request.observation(),
            Some(target),
            self.clock.elapsed_us(),
        ) {
            return Err(reason(&e));
        }
        let reply = match codec::decode_reply(
            &bytes,
            kind,
            attempt.request.observation(),
            target,
            root,
            self.clock.elapsed_us(),
            original,
        ) {
            Ok(reply) => reply,
            Err(error) => {
                // A worker may retire ownership after an acquisition failure. It supplies
                // no verdict, but a separately valid retirement can preserve effect truth.
                if let Ok(aborted) = codec::decode_reply(
                    &bytes,
                    "close_aborted",
                    attempt.request.observation(),
                    target,
                    root,
                    self.clock.elapsed_us(),
                    None,
                ) {
                    self.validate_sequence(&aborted, target.session_id())?;
                    attempt.fail("protocol_error");
                    if let Some(native) = &aborted.native {
                        attempt.native(
                            codec::domain::native(
                                native,
                                target.session_id(),
                                self.clock.elapsed_us(),
                            )
                            .map_err(|_| "protocol_error")?,
                        );
                    }
                }
                return Err(reason(&error));
            }
        };
        self.validate_sequence(&reply, target.session_id())?;
        let success = match kind {
            "close_state" => "inspected",
            "close_prepared" => "prepared",
            "close_rechecked" => "rechecked",
            "close_progress" => "returned",
            "close_waited" => "completed",
            "close_sample" => "observed",
            _ => "released",
        };
        if reply.status != success {
            attempt.fail(refusal_reason(&reply));
        }
        if let Some(native) = &reply.native {
            attempt.native(
                codec::domain::native(native, target.session_id(), self.clock.elapsed_us())
                    .map_err(|_| "protocol_error")?,
            );
        }
        if let Some(resource) = &reply.resource {
            attempt.result.resource =
                codec::domain::resource(resource, target.session_id(), self.clock.elapsed_us())
                    .map_err(|_| "protocol_error")?;
        }
        if let Some(selection) = &reply.selection {
            attempt.result.selection =
                Some(codec::domain::selection(selection).map_err(|_| "protocol_error")?);
        }
        if let Some(protection) = &reply.protection {
            attempt.result.protection =
                codec::domain::protection(protection).map_err(|_| "protocol_error")?;
        }
        Ok(reply)
    }

    fn validate_sequence(
        &mut self,
        reply: &codec::Reply,
        session: &SessionId,
    ) -> Result<(), &'static str> {
        if self.previous.as_ref().is_some_and(|previous| {
            reply.collection.started_tick_us() < previous.finished_tick_us()
        }) || self
            .expiry
            .as_ref()
            .is_some_and(|expiry| reply.expiry.as_ref() != Some(expiry))
        {
            return Err("protocol_error");
        }
        if let Some(native) = &reply.native {
            if self.entered && !native.entered
                || self.removed && native.old_document_removed == Some(false)
                || self.invalidated && !native.invalidated
            {
                return Err("protocol_error");
            }
            if native.entered
                && self
                    .admitted_target
                    .as_ref()
                    .is_none_or(|target| !same_document(native, target))
            {
                return Err("protocol_error");
            }
            if let Some(ledger) = &native.continuation {
                if ledger.required_editor_ids.iter().any(|id| {
                    self.protected_editors
                        .as_ref()
                        .is_none_or(|set| !set.contains(id))
                }) || native.protection.as_ref().is_some_and(|p| {
                    p.required_count != Some(ledger.required_editor_ids.len())
                        || p.completed_count != Some(ledger.completed_editor_ids.len())
                }) {
                    return Err("protocol_error");
                }
                if self
                    .visits
                    .keys()
                    .any(|id| !ledger.required_editor_ids.contains(id))
                {
                    return Err("protocol_error");
                }
                for row in &ledger.collections {
                    let visit = codec::domain::stamp(
                        row.visit.as_ref().ok_or("protocol_error")?,
                        session,
                        self.clock.elapsed_us(),
                    )
                    .map_err(|_| "protocol_error")?;
                    let completion = row
                        .completion
                        .as_ref()
                        .map(|value| codec::domain::stamp(value, session, self.clock.elapsed_us()))
                        .transpose()
                        .map_err(|_| "protocol_error")?;
                    if let Some((previous_visit, previous_completion)) =
                        self.visits.get(&row.editor_id)
                    {
                        if visit.started_tick_us() < previous_visit.started_tick_us()
                            || visit.finished_tick_us() < previous_visit.finished_tick_us()
                            || previous_completion.as_ref().is_some_and(|previous| {
                                completion.as_ref().map_or_else(
                                    || visit.finished_tick_us() < previous.finished_tick_us(),
                                    |current| {
                                        current.finished_tick_us() < previous.finished_tick_us()
                                    },
                                )
                            })
                        {
                            return Err("protocol_error");
                        }
                    }
                    if let Some(previous) = self.visits.get_mut(&row.editor_id) {
                        *previous = (visit, completion);
                    } else {
                        self.visits
                            .insert(row.editor_id.clone(), (visit, completion));
                    }
                }
            } else if !self.visits.is_empty() {
                return Err("protocol_error");
            }
            self.entered |= native.entered;
            self.removed |= native.old_document_removed == Some(true);
            self.invalidated |= native.invalidated;
        }
        if self.expiry.is_none() {
            self.expiry = reply.expiry.clone();
        }
        self.previous = Some(reply.collection.clone());
        Ok(())
    }

    pub(super) fn reply(
        &mut self,
        attempt: &mut Attempt,
        target: &ResolvedTarget,
        root: &str,
        kind: &str,
        original: Option<&CollectionStamp>,
        cancelled: &AtomicBool,
    ) -> Result<codec::Reply, &'static str> {
        let reply = self.receive_reply(attempt, target, root, kind, original, cancelled)?;
        let success = match kind {
            "close_state" => "inspected",
            "close_prepared" => "prepared",
            "close_rechecked" => "rechecked",
            "close_progress" => "returned",
            "close_waited" => "completed",
            "close_sample" => "observed",
            _ => "released",
        };
        if reply.status == success {
            return Ok(reply);
        }
        let cause = refusal_reason(&reply);
        if (matches!(kind, "close_progress" | "close_waited")
            || matches!(kind, "close_sample" | "close_rechecked")
                && reply.purpose.as_deref() == Some("post_close"))
            && survivor_available(&reply)
        {
            // This is fresh evidence, never a second authorization or a
            // recovery of successful-close authority.
            let _ = self.observe_survivor(attempt, target, root, cancelled);
        }
        Err(cause)
    }

    fn observe_survivor(
        &mut self,
        attempt: &mut Attempt,
        target: &ResolvedTarget,
        root: &str,
        cancelled: &AtomicBool,
    ) -> Result<(), &'static str> {
        let mut reply =
            self.receive_reply(attempt, target, root, "close_sample", None, cancelled)?;
        if reply.purpose.as_deref() != Some("survivor") {
            return Err("protocol_error");
        }
        if let Some(sample) = reply.sample.take() {
            let disk = disk(self, &attempt.request, target, cancelled)?;
            let observed = snapshot(&attempt.request, target, sample, disk, self.clock)?;
            attempt.result.observation = Some(core::ObservationSummary {
                purpose: "survivor",
                interval: self.clock.interval(),
                snapshot: core::SnapshotSummary::of(&observed),
            });
        }
        self.receive_reply(attempt, target, root, "close_finished", None, cancelled)?;
        Ok(())
    }
}

fn same_document(native: &codec::Native, target: &DocumentIdentity) -> bool {
    native.script_instance_id.as_deref() == target.script_instance_id().map(|id| id.as_str())
        && native.editor_instance_id.as_deref() == target.editor_instance_id().map(|id| id.as_str())
        && native.buffer_instance_id.as_deref() == target.buffer_instance_id().map(|id| id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SESSION: &str = "00112233445566778899aabbccddeeff";

    fn counter(value: u64) -> DecimalCounter {
        DecimalCounter::new(value.to_string()).unwrap()
    }

    fn stamp(tick: u64) -> Value {
        json!({"clock_id":format!("editor:{SESSION}"), "started_tick_us":tick.to_string(),
               "finished_tick_us":tick.to_string(), "received_elapsed_us":0})
    }

    fn reply(tick: u64, visit: u64, completed: Option<u64>) -> codec::Reply {
        let native = serde_json::from_value(json!({
            "request_id":"close-test", "session_id":SESSION, "script_path":"res://subject.gd",
            "native_build_id":"a".repeat(64), "native_api_revision":3, "phase":"settling",
            "script_instance_id":"3", "editor_instance_id":"4", "buffer_instance_id":"5",
            "entry_collection":stamp(1), "return_collection":stamp(10), "entered":true,
            "close_error":0, "old_document_removed":true, "selection":null,
            "target_buffer":"disposed", "invalidated":false, "reason":null, "terminal_discard":false,
            "protection":{"status":"preserved",
                "revalidation":if completed.is_some() {"completed"} else {"pending"},
                "required_count":1, "completed_count":usize::from(completed.is_some()), "reason":null},
            "continuation":{"state":if completed.is_some() {"completed"} else {"pending"}, "reason":null,
                "required_editor_ids":["7"], "completed_editor_ids":if completed.is_some() {vec!["7"]} else {vec![]},
                "collections":[{"editor_id":"7","visit":stamp(visit),"completion":completed.map(stamp)}]}
        })).unwrap();
        codec::Reply {
            status: "returned".into(),
            reason: None,
            collection: CollectionStamp::new(
                ClockId::Editor(SessionId::new(SESSION).unwrap()),
                counter(tick),
                counter(tick),
                0,
            )
            .unwrap(),
            native: Some(native),
            sample: None,
            context: None,
            guard: None,
            validation: None,
            target_guard: None,
            recheck: None,
            purpose: None,
            continuation: None,
            protection: None,
            resource: None,
            selection: None,
            edited: None,
            discard: None,
            expiry: Some(counter(1000)),
        }
    }

    fn channel() -> Channel {
        let (socket, _peer) = UnixStream::pair().unwrap();
        let mut channel = Channel::new(socket, AttemptClock::start());
        channel.admitted_target = Some(
            DocumentIdentity::new(
                ScriptKind::ExternalGdscript,
                ResourcePath::new("res://subject.gd").unwrap(),
                Some(counter(3)),
                Some(counter(4)),
                Some(counter(5)),
                None,
            )
            .unwrap(),
        );
        channel.protected_editors = Some(["7".to_owned()].into_iter().collect());
        channel
    }

    #[test]
    fn later_visit_can_clear_completion_but_cannot_reuse_earlier_visit() {
        let session = SessionId::new(SESSION).unwrap();
        let mut channel = channel();
        channel
            .validate_sequence(&reply(30, 5, Some(20)), &session)
            .unwrap();
        channel
            .validate_sequence(&reply(40, 35, None), &session)
            .unwrap();
        channel
            .validate_sequence(&reply(60, 35, Some(50)), &session)
            .unwrap();
        assert!(channel
            .validate_sequence(&reply(70, 5, Some(20)), &session)
            .is_err());
    }

    #[test]
    fn established_required_editors_cannot_disappear_from_later_evidence() {
        let session = SessionId::new(SESSION).unwrap();
        let mut channel = channel();
        channel
            .validate_sequence(&reply(30, 5, Some(20)), &session)
            .unwrap();
        let mut missing = reply(40, 5, Some(20));
        let native = missing.native.as_mut().unwrap();
        let ledger = native.continuation.as_mut().unwrap();
        ledger.required_editor_ids.clear();
        ledger.completed_editor_ids.clear();
        ledger.collections.clear();
        let protection = native.protection.as_mut().unwrap();
        protection.required_count = Some(0);
        protection.completed_count = Some(0);
        assert!(channel.validate_sequence(&missing, &session).is_err());
    }

    #[test]
    fn replacement_buffer_cannot_supply_old_document_removal_evidence() {
        let session = SessionId::new(SESSION).unwrap();
        let mut channel = channel();
        let mut replacement = reply(30, 5, Some(20));
        replacement.native.as_mut().unwrap().buffer_instance_id = Some("9".into());
        assert!(channel.validate_sequence(&replacement, &session).is_err());
        assert!(!channel.removed);
    }

    #[test]
    fn retirement_cannot_renew_lease_or_erase_known_removal() {
        let session = SessionId::new(SESSION).unwrap();
        let mut channel = channel();
        channel
            .validate_sequence(&reply(30, 5, Some(20)), &session)
            .unwrap();
        let mut retired = reply(40, 5, Some(20));
        retired.expiry = Some(counter(1001));
        assert!(channel.validate_sequence(&retired, &session).is_err());
        retired.expiry = Some(counter(1000));
        retired.native.as_mut().unwrap().old_document_removed = Some(false);
        assert!(channel.validate_sequence(&retired, &session).is_err());
        assert!(channel.removed);
    }
}
