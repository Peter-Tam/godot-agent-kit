//! Checked stateless script revisions and source-deduplicated observation projection.
use crate::bridge::wire;
use crate::observation::*;
use crate::script_edit::{ExpectedRevisionBasis, ReplacementSource};
use ring::digest::{Context, SHA256};

trait Name {
    fn as_str(&self) -> &'static str;
}
macro_rules! names {
    ($($ty:ident),+ $(,)?) => {
        $(impl Name for $ty { fn as_str(&self)->&'static str {wire::$ty(*self)} })+
    };
}
names!(
    Authority,
    ScriptKind,
    Availability,
    DirtyState,
    Validity,
    OpenState,
    Checks,
    Stability,
    Comparison,
    Agreement,
    RecheckReason,
    FactReason,
    DirtyReason,
    SourceReason,
    OutcomeKind,
    Stage,
    Surface,
    DiagnosticCode
);
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRevision(String);
impl ScriptRevision {
    /// Accept only the fixed, versioned lowercase commitment representation.
    pub fn new(value: impl Into<String>) -> Result<Self, EvidenceError> {
        let value = value.into();
        if value.len() != 68
            || !value.starts_with("sr1:")
            || !value.as_bytes()[4..]
                .iter()
                .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(v))
        {
            return Err(EvidenceError::WrongTarget);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
pub(crate) fn field(context: &mut Context, tag: &str, value: &[u8]) {
    context.update(&(tag.len() as u64).to_be_bytes());
    context.update(tag.as_bytes());
    context.update(&(value.len() as u64).to_be_bytes());
    context.update(value);
}
fn optional(context: &mut Context, tag: &str, value: Option<&DecimalCounter>) {
    field(
        context,
        tag,
        if value.is_some() {
            b"present"
        } else {
            b"absent"
        },
    );
    if let Some(value) = value {
        field(context, "value", value.as_str().as_bytes());
    }
}
fn file(context: &mut Context, tag: &str, value: Option<&FileIdentity>) {
    field(
        context,
        tag,
        if value.is_some() {
            b"present"
        } else {
            b"absent"
        },
    );
    if let Some(value) = value {
        field(context, "device", value.device().as_str().as_bytes());
        field(context, "inode", value.inode().as_str().as_bytes());
    }
}
fn identity(context: &mut Context, document: &DocumentIdentity) {
    field(context, "kind", document.kind().as_str().as_bytes());
    field(
        context,
        "resource_path",
        document.resource_path().as_str().as_bytes(),
    );
    optional(context, "script", document.script_instance_id());
    optional(context, "editor", document.editor_instance_id());
    optional(context, "buffer", document.buffer_instance_id());
    file(context, "file", document.disk_file_id());
}
fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}
fn commitment(basis: &ExpectedRevisionBasis) -> ScriptRevision {
    let mut context = Context::new(&SHA256);
    field(
        &mut context,
        "domain",
        b"godot-agent-kit/script-revision/sr1",
    );
    field(&mut context, "lifecycle", b"open");
    let target = basis.target();
    field(
        &mut context,
        "project",
        target.project_root().as_str().as_bytes(),
    );
    file(
        &mut context,
        "project_identity",
        Some(target.project_file_id()),
    );
    field(
        &mut context,
        "session",
        target.session_id().as_str().as_bytes(),
    );
    field(
        &mut context,
        "script_path",
        target.script_path().as_str().as_bytes(),
    );
    identity(&mut context, basis.document());
    field(
        &mut context,
        "engine_version",
        target.godot_version().version().as_bytes(),
    );
    field(
        &mut context,
        "engine_hash",
        target.godot_version().hash().as_bytes(),
    );
    for (tag, witness) in ["D", "R", "B"].into_iter().zip(basis.witnesses()) {
        field(&mut context, "authority", tag.as_bytes());
        field(&mut context, "availability", b"observed");
        field(
            &mut context,
            "path",
            witness.resource_path().as_str().as_bytes(),
        );
        optional(&mut context, "script", witness.script_instance_id());
        optional(&mut context, "editor", witness.editor_instance_id());
        optional(&mut context, "buffer", witness.buffer_instance_id());
        file(&mut context, "file", witness.disk_file_id());
        optional(&mut context, "version", witness.source_version());
    }
    field(&mut context, "dirty", b"clean");
    field(&mut context, "validity", b"valid");
    if let Some(witness) = basis.prior_dirty().witness() {
        field(&mut context, "authority", b"dirty_buffer");
        field(
            &mut context,
            "path",
            witness.resource_path().as_str().as_bytes(),
        );
        optional(&mut context, "script", witness.script_instance_id());
        optional(&mut context, "editor", witness.editor_instance_id());
        optional(&mut context, "buffer", witness.buffer_instance_id());
        file(&mut context, "file", witness.disk_file_id());
        optional(&mut context, "version", witness.source_version());
    }
    field(&mut context, "source_sha256", &basis.source().sha256);
    field(
        &mut context,
        "utf8_bytes",
        &(basis.source().utf8_bytes as u64).to_be_bytes(),
    );
    field(
        &mut context,
        "current_version",
        basis.current_version().as_str().as_bytes(),
    );
    ScriptRevision(format!("sr1:{}", hex(context.finish().as_ref())))
}

/// Checked selectors and revision intent; never accepts an observation or native basis.
#[derive(Debug)]
pub struct ScriptEditRequest {
    pub(crate) request: ObservationRequest,
    pub(crate) revision: ScriptRevision,
    pub(crate) replacement: ReplacementSource,
}
impl ScriptEditRequest {
    pub fn new(
        request: ObservationRequest,
        revision: ScriptRevision,
        replacement: ReplacementSource,
    ) -> Result<Self, EvidenceError> {
        if request.script_path().kind() != Some(ScriptKind::ExternalGdscript) {
            return Err(EvidenceError::WrongTarget);
        }
        Ok(Self {
            request,
            revision,
            replacement,
        })
    }
}

/// Useful read projection. Private captures and bases remain owned within this call.
pub struct ScriptReadResult {
    pub(crate) requested: Option<ObservationRequest>,
    pub(crate) observation: ObservationOutcome,
    pub(crate) closed: Option<crate::script_closed_edit::ClosedExpectedBasis>,
    pub(crate) closed_observed: Option<crate::script_closed_edit::State>,
    revision: Option<ScriptRevision>,
    unavailable: Option<String>,
    pub(crate) interval: ObservationInterval,
}
impl ScriptReadResult {
    pub(crate) fn from_observation(observation: ObservationOutcome) -> Self {
        let basis = ExpectedRevisionBasis::from_observation(&observation);
        let revision = basis.as_ref().ok().map(commitment);
        let unavailable = basis.err().map(|reason| match reason {
            crate::script_edit::BasisError::Dirty => "dirty_buffer",
            crate::script_edit::BasisError::Divergent => "divergent_sources",
            crate::script_edit::BasisError::KnownStaleResource => "stale_resource",
            crate::script_edit::BasisError::KnownStaleBuffer => "stale_buffer",
            _ if !matches!(
                observation.outcome(),
                OutcomeKind::CompleteObservation
                    | OutcomeKind::LimitedObservation
                    | OutcomeKind::NotOpen
            ) =>
            {
                wire::OutcomeKind(observation.outcome())
            }
            _ => "required_state_unavailable",
        });
        let interval = observation.interval().clone();
        Self {
            observation,
            closed: None,
            closed_observed: None,
            revision,
            unavailable: unavailable.map(str::to_owned),
            interval,
            requested: None,
        }
    }
    pub(crate) fn with_closed(
        mut self,
        basis: crate::script_closed_edit::ClosedExpectedBasis,
    ) -> Self {
        let mut context = Context::new(&SHA256);
        field(
            &mut context,
            "domain",
            b"godot-agent-kit/script-revision/sr1",
        );
        field(&mut context, "lifecycle", b"closed");
        basis.stable_commitment(&mut context);
        self.revision = Some(ScriptRevision(format!(
            "sr1:{}",
            hex(context.finish().as_ref())
        )));
        self.unavailable = None;
        self.closed = Some(basis);
        self
    }
    pub fn revision(&self) -> Option<&ScriptRevision> {
        self.revision.as_ref()
    }
    pub(crate) fn suppress_revision(&mut self, reason: &str) {
        self.revision = None;
        self.unavailable = Some(reason.to_owned());
        self.closed = None;
    }
    pub(crate) fn refusal(
        &self,
        request_id: &RequestId,
        reason: Option<&str>,
    ) -> serde_json::Value {
        let reason = reason
            .or(self.unavailable.as_deref())
            .unwrap_or("required_state_unavailable");
        let denied = self.observation.outcome() == OutcomeKind::DeniedAccess;
        let diagnostics: Vec<_> = self.observation.diagnostics().iter().map(|d|serde_json::json!({"code":d.code().as_str(),"stage":d.stage().as_str(),"surface":d.surface().map(|s|s.as_str())})).collect();
        let selection = self.observation.selection().map(|s|serde_json::json!({"candidate_sessions":s.candidate_sessions().iter().map(SessionId::as_str).collect::<Vec<_>>(),"missing_selector":"session_id"}));
        let action = if selection.is_some() {
            "specify_session"
        } else if denied {
            "check_access_and_target"
        } else if matches!(
            self.observation.outcome(),
            OutcomeKind::EditorUnavailable | OutcomeKind::DisconnectedEditor
        ) {
            "check_selected_editor"
        } else {
            "fresh_read"
        };
        serde_json::json!({"mode":"undetermined","outcome":{"outcome":"refused","application":"not_applied","request_id":request_id.as_str(),"reason":reason,"stage":self.observation.diagnostics().last().map(|d|d.stage().as_str()).unwrap_or("preflight"),"diagnostics":diagnostics,"interval":IntervalView::new(&self.interval),"requested_target":if denied {None} else {self.requested.as_ref().map(|r|serde_json::json!({"project_root":r.project_root().as_str(),"script_path":r.script_path().as_str(),"session_id":r.session_id().map(SessionId::as_str)}))},"target":if denied {None} else {self.observation.resolved_target().map(|t|serde_json::json!({"project_root":t.project_root().as_str(),"script_path":t.script_path().as_str(),"session_id":t.session_id().as_str()}))},"selection":selection,"history":"not_participated","lifecycle":{"admitted":null,"final":null},"next_action":{"kind":action}}})
    }
    /// Serialize only the public projection, bounded independently of private worker data.
    pub fn encode(&self) -> Result<Vec<u8>, crate::runner::HostFailure> {
        let snapshot = self.observation.snapshot();
        let current = snapshot.and_then(|s| {
            s.sources()
                .buffer()
                .text()
                .map(|v| (v, "editor_buffer"))
                .or_else(|| s.sources().disk().text().map(|v| (v, "disk")))
                .or_else(|| {
                    s.sources()
                        .resource()
                        .text()
                        .map(|v| (v, "loaded_resource"))
                })
        });
        let source = current.map(|v| v.0);
        let sources = snapshot.map(|s| SourceSet {
            disk: SourceView::new(s.sources().disk(), source, None),
            loaded_resource: SourceView::new(
                s.sources().resource(),
                source,
                s.sources().disk().text().map(|text| (text, "disk")),
            ),
            editor_buffer: SourceView::new(
                s.sources().buffer(),
                source,
                s.sources()
                    .disk()
                    .text()
                    .map(|text| (text, "disk"))
                    .or_else(|| {
                        s.sources()
                            .resource()
                            .text()
                            .map(|text| (text, "loaded_resource"))
                    }),
            ),
        });
        let document_id = snapshot
            .and_then(|s| s.document().identity())
            .filter(|_| {
                snapshot
                    .is_some_and(|s| s.document().open_state().value() == Some(&OpenState::Open))
            })
            .map(|d| {
                let mut context = Context::new(&SHA256);
                field(&mut context, "domain", b"godot-agent-kit/document/1");
                if let Some(t) = self.observation.resolved_target() {
                    field(&mut context, "session", t.session_id().as_str().as_bytes());
                }
                identity(&mut context, d);
                hex(context.finish().as_ref())
            });
        let lifecycle = snapshot
            .and_then(|s| s.document().open_state().value())
            .map_or("unknown", |v| {
                if *v == OpenState::Open {
                    "open"
                } else {
                    "closed"
                }
            });
        let target = self.observation.resolved_target().map(|t| serde_json::json!({"project_root":t.project_root().as_str(),"script_path":t.script_path().as_str(),"session_id":t.session_id().as_str(),"kind":t.script_path().kind().map(|k|k.as_str())}));
        let resource_dirty = self.closed_observed.as_ref().map(|s| serde_json::json!({"availability":match s.resource.state.as_str() {"present"=>"observed","absent"=>"not_applicable",_=>"unavailable"},"state":s.resource.edited.map(|edited|if edited{"dirty"}else{"clean"})})).unwrap_or_else(||serde_json::json!({"availability":"unavailable","state":null}));
        let dirty = snapshot.map(|s| serde_json::json!({"buffer":{"availability":s.dirty().availability().as_str(),"state":s.dirty().state().map(|v|v.as_str()),"reason":s.dirty().reason().map(|v|v.as_str()),"invalidated":s.dirty().invalidated_evidence().map(|v|serde_json::json!({"state":v.state().as_str(),"reason":v.reason().as_str()}))},"loaded_resource":resource_dirty}));
        let consistency = snapshot.map(|s| {
            let detected_changes: Vec<_> = s
                .consistency()
                .detected_changes()
                .iter()
                .map(|change| {
                    let (surface, code) = match *change {
                        DetectedChange::Source(Authority::D) => ("D", "source_changed"),
                        DetectedChange::Source(Authority::R) => ("R", "source_changed"),
                        DetectedChange::Source(Authority::B) => ("B", "source_changed"),
                        DetectedChange::Dirty => ("dirty", "source_changed"),
                        DetectedChange::DocumentClosed => ("document", "document_closed"),
                        DetectedChange::DocumentIdentityReplaced => ("document", "identity_changed"),
                        DetectedChange::SessionReplaced => ("session", "identity_changed"),
                        DetectedChange::SessionEnded => ("session", "session_ended"),
                        DetectedChange::DiskIdentityReplaced => ("D", "identity_changed"),
                    };
                    serde_json::json!({"surface": surface, "code": code})
                })
                .collect();
            serde_json::json!({"atomic":false,"checks":s.consistency().checks().as_str(),"stability":s.consistency().stability().as_str(),"recheck_reason":s.consistency().recheck_reason().map(|v|v.as_str()),"detected_changes":detected_changes,"comparisons":{"disk_resource":s.comparisons().disk_resource().as_str(),"disk_buffer":s.comparisons().disk_buffer().as_str(),"resource_buffer":s.comparisons().resource_buffer().as_str()},"agreement":s.agreement().as_str()})
        });
        let diagnostics: Vec<_> = self.observation.diagnostics().iter().map(|d|serde_json::json!({"code":d.code().as_str(),"stage":d.stage().as_str(),"surface":d.surface().map(|s|s.as_str())})).collect();
        let mut limitations = vec!["non_atomic_observation"];
        if snapshot.is_some_and(|s| s.sources().resource().text().is_some()) {
            limitations.push("loaded_class_not_reloaded");
        }
        let document = serde_json::json!({"lifecycle":lifecycle,"identity":document_id,"validity":snapshot.and_then(|s|s.document().validity().value()).map(|v|v.as_str()),"validity_reason":snapshot.and_then(|s|s.document().validity().reason()).map(|v|v.as_str()),"invalidated_validity":snapshot.and_then(|s|s.document().validity().invalidated_evidence()).map(|v|serde_json::json!({"value":v.value().as_str(),"reason":v.reason().as_str()})),"open_state_reason":snapshot.and_then(|s|s.document().open_state().reason()).map(|v|v.as_str()),"invalidated_lifecycle":snapshot.and_then(|s|s.document().open_state().invalidated_evidence()).map(|v|serde_json::json!({"value":v.value().as_str(),"reason":v.reason().as_str()}))});
        let requested_target=self.requested.as_ref().map(|r|serde_json::json!({"project_root":r.project_root().as_str(),"script_path":r.script_path().as_str(),"session_id":r.session_id().map(|s|s.as_str())}));
        let next_action = if self.revision.is_some() {
            "edit_with_current_revision"
        } else {
            match self.observation.outcome() {
                OutcomeKind::AmbiguousTarget => "specify_session",
                OutcomeKind::DeniedAccess => "check_access_and_target",
                OutcomeKind::EditorUnavailable | OutcomeKind::DisconnectedEditor => {
                    "check_selected_editor"
                }
                _ => "fresh_read",
            }
        };
        let state = State {
            status: self.observation.outcome().as_str(),
            target,
            requested_target,
            document,
            source_origin: current.map(|v| v.1),
            sources,
            dirty,
            consistency,
            interval: IntervalView::new(&self.interval),
            diagnostics,
            limitations,
            revision_unavailable_reason: self.unavailable.as_deref(),
            next_action,
            selection: self.observation.selection().map(|s|serde_json::json!({"candidate_sessions":s.candidate_sessions().iter().map(SessionId::as_str).collect::<Vec<_>>(),"missing_selector":"session_id"})),
        };
        let bytes = serde_json::to_vec(&ReadView {
            source,
            revision: self.revision.as_ref().map(ScriptRevision::as_str),
            state,
        })
        .map_err(|_| crate::runner::HostFailure)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(crate::runner::HostFailure);
        }
        Ok(bytes)
    }
    pub(crate) fn matching_open_input(
        &self,
        id: &RequestId,
        replacement: &ReplacementSource,
    ) -> Result<Vec<u8>, crate::runner::HostFailure> {
        let capture =
            wire::encode_outcome(&self.observation).map_err(|_| crate::runner::HostFailure)?;
        // Embed the one exact frozen observation without decoding source.
        let mut bytes = Vec::with_capacity(capture.len() + replacement.as_str().len() + 256);
        bytes.extend_from_slice(b"{\"schema_version\":1,\"request_id\":");
        serde_json::to_writer(&mut bytes, id.as_str()).map_err(|_| crate::runner::HostFailure)?;
        bytes.extend_from_slice(b",\"basis\":");
        bytes.extend_from_slice(&capture);
        bytes.extend_from_slice(b",\"replacement_source\":");
        serde_json::to_writer(&mut bytes, replacement.as_str())
            .map_err(|_| crate::runner::HostFailure)?;
        bytes.push(b'}');
        if bytes.len() > wire::edit::input_limit() {
            return Err(crate::runner::HostFailure);
        }
        Ok(bytes)
    }
}
#[derive(Serialize)]
struct ReadView<'a> {
    source: Option<&'a str>,
    revision: Option<&'a str>,
    state: State<'a>,
}
#[derive(Serialize)]
struct State<'a> {
    status: &'static str,
    target: Option<serde_json::Value>,
    requested_target: Option<serde_json::Value>,
    document: serde_json::Value,
    source_origin: Option<&'static str>,
    sources: Option<SourceSet<'a>>,
    dirty: Option<serde_json::Value>,
    consistency: Option<serde_json::Value>,
    interval: IntervalView,
    diagnostics: Vec<serde_json::Value>,
    limitations: Vec<&'static str>,
    revision_unavailable_reason: Option<&'a str>,
    next_action: &'static str,
    selection: Option<serde_json::Value>,
}
#[derive(Serialize)]
struct IntervalView {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
impl IntervalView {
    fn new(v: &ObservationInterval) -> Self {
        Self {
            started_unix_ms: v.started_unix_ms(),
            finished_unix_ms: v.finished_unix_ms(),
            elapsed_us: v.elapsed_us(),
        }
    }
}
#[derive(Serialize)]
struct SourceSet<'a> {
    disk: SourceView<'a>,
    loaded_resource: SourceView<'a>,
    editor_buffer: SourceView<'a>,
}
#[derive(Serialize)]
struct SourceView<'a> {
    availability: &'static str,
    current: bool,
    equals_source: Option<bool>,
    text: Option<&'a str>,
    text_origin: Option<&'static str>,
    reason: Option<&'static str>,
    invalidated: Option<HistoricalSource<'a>>,
    staleness: Option<serde_json::Value>,
}
#[derive(Serialize)]
struct HistoricalSource<'a> {
    text: Option<&'a str>,
    equals_source: Option<bool>,
    reason: &'static str,
    current: bool,
    staleness: serde_json::Value,
}
fn staleness(v: &Staleness) -> serde_json::Value {
    serde_json::json!({"state":if v.evidence().is_some(){"known_stale"}else{"unknown"},"evidence":v.evidence().map(|e|e.iter().map(|v|serde_json::json!({"incorporated_version":v.incorporated_version().as_str(),"changed_authority":v.changed_authority().as_str(),"changed_version":v.changed_version().as_str()})).collect::<Vec<_>>())})
}
impl<'a> SourceView<'a> {
    fn new(
        v: &'a SourceObservation,
        source: Option<&str>,
        previous: Option<(&str, &'static str)>,
    ) -> Self {
        let duplicate = previous.filter(|(text, _)| Some(*text) == v.text());
        Self {
            availability: v.availability().as_str(),
            current: v.text().is_some(),
            equals_source: v.text().zip(source).map(|(a, b)| a == b),
            text: v
                .text()
                .filter(|v| Some(*v) != source && duplicate.is_none()),
            text_origin: duplicate.map(|(_, origin)| origin),
            reason: v.reason().map(|v| v.as_str()),
            invalidated: v.invalidated_evidence().map(|v| HistoricalSource {
                text: (Some(v.text()) != source).then(|| v.text()),
                equals_source: source.map(|s| s == v.text()),
                reason: v.reason().as_str(),
                current: false,
                staleness: staleness(v.staleness()),
            }),
            staleness: v.staleness().map(staleness),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tagged_lengths_prevent_boundary_aliasing() {
        let mut first = Context::new(&SHA256);
        field(&mut first, "ab", b"c");
        field(&mut first, "d", b"e");
        let mut second = Context::new(&SHA256);
        field(&mut second, "a", b"bc");
        field(&mut second, "d", b"e");
        assert_ne!(first.finish().as_ref(), second.finish().as_ref());
    }
}

#[cfg(test)]
#[path = "script_read_regression.rs"]
mod regression;
