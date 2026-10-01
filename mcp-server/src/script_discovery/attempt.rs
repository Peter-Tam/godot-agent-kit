//! Parent-owned evidence ordering, invalidation and terminal precedence.
use super::events::{decimal, safe_scope, ContextStatus};
use super::*;
use std::collections::BTreeSet;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Starting,
    Selected,
    Scoped,
    Enumerating,
    Rechecked,
    Failed,
    Terminal,
}
pub(crate) struct Attempt {
    request: DiscoveryRequest,
    state: State,
    target: Option<DiscoveryTarget>,
    begin: Option<Context>,
    entries: BTreeSet<String>,
    invalidated: Vec<String>,
    diagnostics: Vec<Diagnostic>,
    omitted: u32,
    reason: Option<Reason>,
    selection: Option<Selection>,
    suppressed: bool,
    changed: bool,
    checked: bool,
    exhausted: bool,
    visited_entries: u32,
    visited_directories: u32,
    ordinal: u32,
    accepted_entries: usize,
    last_receipt: u64,
    collection: Option<Stamp>,
}
impl Attempt {
    pub(crate) fn new(request: DiscoveryRequest) -> Self {
        Self {
            request,
            state: State::Starting,
            target: None,
            begin: None,
            entries: BTreeSet::new(),
            invalidated: Vec::new(),
            diagnostics: Vec::new(),
            omitted: 0,
            reason: None,
            selection: None,
            suppressed: false,
            changed: false,
            checked: false,
            exhausted: false,
            visited_entries: 0,
            visited_directories: 0,
            ordinal: 0,
            accepted_entries: 0,
            last_receipt: 0,
            collection: None,
        }
    }
    pub(crate) fn request(&self) -> &DiscoveryRequest {
        &self.request
    }
    pub(crate) fn is_done(&self) -> bool {
        self.state == State::Terminal
    }
    fn add(&mut self, reason: Reason, stage: Stage, scope: Option<String>) {
        if self.reason.is_none_or(|r| reason.priority() < r.priority()) {
            self.reason = Some(reason);
        }
        if reason.suppress() {
            self.suppressed = true;
            self.entries.clear();
        }
        if self.diagnostics.len() < 63 {
            self.diagnostics.push(Diagnostic::new(reason, stage, scope));
        } else {
            self.omitted = self.omitted.saturating_add(1);
        }
    }
    pub(crate) fn fail(&mut self, reason: Reason) {
        self.add(reason, Stage::Finalize, None);
        self.state = State::Failed;
    }
    fn binding(&self) -> Binding {
        self.target
            .as_ref()
            .map_or_else(|| Binding::request(&self.request), Binding::target)
    }
    fn counters(&self, entries: u32, dirs: u32) -> bool {
        entries >= self.visited_entries
            && dirs >= self.visited_directories
            && entries <= WORK_LIMIT
            && dirs <= DIRECTORY_LIMIT
            && dirs > 0
    }
    pub(crate) fn accept(&mut self, event: Event, receipt: u64) -> Result<(), Reason> {
        let result = self.accept_checked(event, receipt);
        if let Err(reason) = result {
            self.fail(reason);
        }
        result
    }
    fn accept_checked(&mut self, event: Event, receipt: u64) -> Result<(), Reason> {
        if self.state == State::Terminal || receipt < self.last_receipt {
            return Err(Reason::ProtocolError);
        }
        if receipt >= CUTOFF_US {
            return Err(Reason::Timeout);
        }
        let expected = self.binding();
        if event.binding() != &expected {
            // Reject the payload in full. Only already-bound lifetime/root loss suppresses
            // previously authorized paths; a foreign request cannot add any evidence.
            if self.target.is_some()
                && event.binding().request_id == expected.request_id
                && (event.binding().session_id != expected.session_id
                    || event.binding().project_root != expected.project_root
                    || event.binding().project_file_id != expected.project_file_id)
            {
                self.suppressed = true;
                self.entries.clear();
            }
            return Err(Reason::ProtocolError);
        }
        match event {
            Event::Selected {
                target, capability, ..
            } => {
                if self.state != State::Starting || !target.valid(&self.request) {
                    return Err(Reason::ProtocolError);
                }
                let supported = target.godot_version.version == crate::bridge::GODOT_VERSION
                    && target.godot_version.hash == crate::bridge::ENGINE_HASH;
                self.target = Some(target);
                if !supported {
                    self.add(Reason::UnsupportedVersion, Stage::Authenticate, None);
                    self.state = State::Failed;
                } else if !capability {
                    self.add(Reason::UnsupportedCapability, Stage::Authenticate, None);
                    self.state = State::Failed;
                } else {
                    self.state = State::Selected;
                }
            }
            Event::Scope { context, .. } => {
                let target = self.target.as_ref().ok_or(Reason::ProtocolError)?;
                if self.state != State::Selected || !context.valid(&target.session_id, receipt) {
                    return Err(Reason::ProtocolError);
                }
                if let Some(mut reason) = context.failure() {
                    if reason == Reason::RecheckUnavailable {
                        reason = Reason::UnsupportedVisibilityPolicy;
                    }
                    self.add(reason, Stage::BeginScope, None);
                }
                if context.status == ContextStatus::Observed {
                    let start = context.collection.received_elapsed_us;
                    self.collection = Some(Stamp::caller(start, start, receipt));
                    if context
                        .context
                        .as_ref()
                        .is_some_and(|f| f.scanning || f.importing)
                    {
                        self.add(
                            Reason::EditorInventoryBusy,
                            Stage::BeginScope,
                            Some("res://".into()),
                        );
                    }
                }
                self.begin = Some(context);
                self.state = State::Scoped;
            }
            Event::Entries {
                ordinal,
                collection,
                entries,
                visited_entries,
                visited_directories,
                ..
            } => {
                let facts = self
                    .begin
                    .as_ref()
                    .and_then(|c| c.context.as_ref())
                    .ok_or(Reason::ProtocolError)?;
                if !matches!(self.state, State::Scoped | State::Enumerating)
                    || self.suppressed
                    || facts.scanning
                    || facts.importing
                    || ordinal != self.ordinal
                    || entries.is_empty()
                    || entries.len() > BATCH_LIMIT
                    || !self.counters(visited_entries, visited_directories)
                    || entries.len() + self.accepted_entries > ENTRY_LIMIT
                    || !collection.valid("caller", receipt)
                {
                    return Err(Reason::ProtocolError);
                }
                let begin = self.collection.as_ref().ok_or(Reason::ProtocolError)?;
                if decimal(&collection.started_tick_us) < decimal(&begin.finished_tick_us) {
                    return Err(Reason::ProtocolError);
                }
                let mut batch = BTreeSet::new();
                for path in &entries {
                    if !eligible(path, &facts.project_data_directory)
                        || path[6..].split('/').count().saturating_sub(1) > DEPTH_LIMIT
                        || self.entries.contains(path)
                        || !batch.insert(path.as_str())
                        || self.invalidated.iter().any(|prefix| affected(path, prefix))
                    {
                        return Err(Reason::ProtocolError);
                    }
                }
                if visited_entries < (self.accepted_entries + entries.len()) as u32 {
                    return Err(Reason::ProtocolError);
                }
                self.accepted_entries += entries.len();
                self.entries.extend(entries);
                self.visited_entries = visited_entries;
                self.visited_directories = visited_directories;
                self.ordinal += 1;
                if let Some(total) = &mut self.collection {
                    total.finished_tick_us = collection.finished_tick_us;
                    total.received_elapsed_us = receipt;
                }
                self.state = State::Enumerating;
            }
            Event::Invalidate { reason, prefix, .. } => {
                let binding_loss = reason == Reason::ProtocolError
                    && prefix.is_none()
                    && matches!(
                        self.state,
                        State::Selected | State::Scoped | State::Enumerating | State::Rechecked
                    );
                if !binding_loss
                    && (!matches!(self.state, State::Scoped | State::Enumerating)
                        || !matches!(
                            reason,
                            Reason::NamespaceChanged
                                | Reason::ScopeChanged
                                | Reason::ProjectIdentityChanged
                                | Reason::DeniedAccess
                                | Reason::OutOfProject
                        ))
                    || prefix.as_ref().is_some_and(|p| !safe_scope(p))
                    || self.invalidated.len() >= WORK_LIMIT as usize
                {
                    return Err(Reason::ProtocolError);
                }
                if let Some(prefix) = &prefix {
                    self.entries.retain(|path| !affected(path, prefix));
                    self.invalidated.push(prefix.clone());
                } else {
                    self.entries.clear();
                    self.suppressed = true;
                }
                self.changed = true;
                self.add(reason, Stage::Recheck, prefix);
            }
            Event::Rechecked {
                context,
                exhausted,
                namespace_checked,
                changed,
                visited_entries,
                visited_directories,
                gaps,
                omitted_gaps,
                ..
            } => {
                let target = self.target.as_ref().ok_or(Reason::ProtocolError)?;
                let begin = self.begin.as_ref().ok_or(Reason::ProtocolError)?;
                let zero_work = visited_entries == 0
                    && visited_directories == 0
                    && self.visited_directories == 0
                    && self.ordinal == 0
                    && !exhausted
                    && !namespace_checked;
                if !matches!(self.state, State::Scoped | State::Enumerating)
                    || !context.valid(&target.session_id, receipt)
                    || !(self.counters(visited_entries, visited_directories) || zero_work)
                    || gaps.len() > 63
                    || omitted_gaps > WORK_LIMIT + DIRECTORY_LIMIT
                    || gaps.iter().any(|g| {
                        g.code.priority() != 8
                            || g.code == Reason::AdditionalGaps
                            || g.scope.as_ref().is_some_and(|s| !safe_scope(s))
                    })
                    || decimal(&context.collection.started_tick_us)
                        < decimal(&begin.collection.finished_tick_us)
                    || context.collection.received_elapsed_us < begin.collection.received_elapsed_us
                    || context.expiry_tick_us != begin.expiry_tick_us
                {
                    return Err(Reason::ProtocolError);
                }
                let old = begin.context.clone();
                if old
                    .as_ref()
                    .zip(context.context.as_ref())
                    .is_some_and(|(a, b)| {
                        decimal(&b.filesystem_epoch) < decimal(&a.filesystem_epoch)
                    })
                {
                    return Err(Reason::ProtocolError);
                }
                if let Some(mut reason) = context.failure() {
                    if reason == Reason::UnsupportedVisibilityPolicy && old.is_some() {
                        reason = Reason::ScopeChanged;
                        self.changed = true;
                    }
                    self.add(reason, Stage::Recheck, Some("res://".into()));
                }
                match (&old, &context.context) {
                    (Some(a), Some(b)) => {
                        if a.policy != b.policy
                            || a.project_data_directory != b.project_data_directory
                        {
                            self.add(Reason::ScopeChanged, Stage::Recheck, Some("res://".into()));
                            self.changed = true;
                        }
                        if a.filesystem_epoch != b.filesystem_epoch {
                            self.add(
                                Reason::EditorInventoryChanged,
                                Stage::Recheck,
                                Some("res://".into()),
                            );
                            self.changed = true;
                        }
                        if a.scanning || a.importing || b.scanning || b.importing {
                            self.add(
                                Reason::EditorInventoryBusy,
                                Stage::Recheck,
                                Some("res://".into()),
                            );
                        }
                    }
                    _ => self.add(
                        Reason::RecheckUnavailable,
                        Stage::Recheck,
                        Some("res://".into()),
                    ),
                }
                for gap in gaps {
                    if gap.code == Reason::NamespaceChanged {
                        self.changed = true;
                        if let Some(prefix) = &gap.scope {
                            self.entries.retain(|path| !affected(path, prefix));
                        } else {
                            self.entries.clear();
                            self.suppressed = true;
                        }
                    }
                    self.add(gap.code, gap.stage, gap.scope);
                }
                self.omitted = self.omitted.saturating_add(omitted_gaps);
                if omitted_gaps > 0 && self.reason.is_none() {
                    self.reason = Some(Reason::AdditionalGaps);
                }
                if changed {
                    self.changed = true;
                    if self.reason.is_none() {
                        self.add(
                            Reason::NamespaceChanged,
                            Stage::Recheck,
                            Some("res://".into()),
                        );
                    }
                }
                self.checked = namespace_checked && context.status == ContextStatus::Observed;
                if !namespace_checked {
                    self.add(
                        Reason::RecheckUnavailable,
                        Stage::Recheck,
                        Some("res://".into()),
                    );
                }
                self.exhausted = exhausted;
                if !exhausted && self.reason.is_none() {
                    self.add(
                        Reason::RecheckUnavailable,
                        Stage::Enumerate,
                        Some("res://".into()),
                    );
                }
                self.visited_entries = visited_entries;
                self.visited_directories = visited_directories;
                if let Some(total) = &mut self.collection {
                    total.finished_tick_us = receipt.to_string();
                    total.received_elapsed_us = receipt;
                }
                self.state = State::Rechecked;
            }
            Event::Failed {
                reason, selection, ..
            } => {
                if self.state == State::Failed
                    || reason == Reason::AdditionalGaps
                    || selection.as_ref().is_some_and(|s| {
                        reason != Reason::AmbiguousTarget
                            || self.request.session_id().is_some()
                            || s.missing_selector != "session_id"
                            || s.candidate_sessions.len() < 2
                            || s.candidate_sessions.len() > 64
                            || s.candidate_sessions
                                .iter()
                                .any(|id| SessionId::new(id.clone()).is_err())
                            || s.candidate_sessions.windows(2).any(|w| w[0] >= w[1])
                    })
                    || (reason == Reason::AmbiguousTarget && selection.is_none())
                {
                    return Err(Reason::ProtocolError);
                }
                self.selection = selection;
                self.add(reason, Stage::Finalize, None);
                self.state = State::Failed;
            }
            Event::Done { .. } => {
                if !matches!(self.state, State::Rechecked | State::Failed)
                    && !(self.state == State::Scoped && self.reason.is_some())
                {
                    return Err(Reason::ProtocolError);
                }
                self.state = State::Terminal;
            }
        }
        self.last_receipt = receipt;
        Ok(())
    }
    pub(crate) fn finish(mut self, interval: ObservationInterval) -> DiscoveryOutcome {
        if self.state != State::Terminal && self.reason.is_none() {
            self.add(Reason::RecheckUnavailable, Stage::Finalize, None);
        }
        let priority = self.reason.map(Reason::priority);
        let outcome = match priority {
            Some(0..=2) => "refused",
            Some(3..=7) => "interrupted",
            Some(_) => "limited_listing",
            None if self.checked && self.exhausted && !self.changed => "complete_listing",
            None => "limited_listing",
        };
        if self.omitted > 0 {
            self.diagnostics.push(Diagnostic::aggregate(self.omitted));
        }
        if self.suppressed {
            let untrusted =
                priority.is_some_and(|p| p <= 2) || self.reason == Some(Reason::SessionReplaced);
            for d in &mut self.diagnostics {
                if untrusted
                    || d.code != Reason::ScopeChanged
                    || d.scope.as_deref() != Some("res://")
                {
                    d.scope = None;
                }
            }
        }
        let inventory = if self.suppressed {
            None
        } else {
            self.begin
                .as_ref()
                .and_then(|c| c.context.as_ref())
                .zip(self.collection)
                .map(|(facts, collection)| Inventory {
                    scope: Scope::new(facts.project_data_directory.clone()),
                    entries: self.entries.into_iter().collect(),
                    collection,
                    coverage: if outcome == "complete_listing" {
                        "complete"
                    } else if self.visited_directories == 0 {
                        "not_started"
                    } else {
                        "partial"
                    },
                    validity: if outcome == "interrupted" {
                        "earlier_observation"
                    } else {
                        "observed"
                    },
                    consistency: Consistency {
                        recheck: if outcome != "interrupted" && self.checked {
                            "completed"
                        } else {
                            "unavailable"
                        },
                        stability: if self.changed { "changed" } else { "unknown" },
                        atomic: false,
                    },
                    visited_entries: self.visited_entries,
                    visited_directories: self.visited_directories,
                })
        };
        DiscoveryOutcome {
            schema_version: 1,
            request_id: self.request.request_id().as_str().into(),
            outcome,
            interval: interval.into(),
            requested_target: Some(RequestedTarget {
                project_root: self.request.project_root().as_str().into(),
                session_id: self.request.session_id().map(|s| s.as_str().into()),
            }),
            resolved_target: if priority.is_some_and(|p| p <= 1) {
                None
            } else {
                self.target
            },
            inventory,
            diagnostics: self.diagnostics,
            selection: if self.reason == Some(Reason::AmbiguousTarget) {
                self.selection
            } else {
                None
            },
        }
        .bounded()
    }
}
fn affected(path: &str, prefix: &str) -> bool {
    prefix == "res://"
        || path == prefix.trim_end_matches('/')
        || path
            .strip_prefix(prefix.trim_end_matches('/'))
            .is_some_and(|tail| tail.starts_with('/'))
}
#[cfg(test)]
mod tests;
