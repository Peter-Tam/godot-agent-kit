//! Strict, bounded editor frames and private worker evidence events.
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

use crate::observation::*;
use crate::target::{RoutingFailure, SelectedSession};

pub const RESULT_LIMIT: usize = 12 * 1024 * 1024;
const COLLECTION_LIMIT: usize = 64;

pub mod edit;
pub mod open;

/// Inclusive editor interval enclosing every editor fact in this sample.
/// Retained through private IPC so the supervisor can validate worker evidence.
pub struct EditorSample {
    pub collection: CollectionStamp,
    pub document: DocumentState,
    pub resource: SourceObservation,
    pub buffer: SourceObservation,
    pub dirty: DirtyObservation,
    pub diagnostics: Vec<Diagnostic>,
}

pub enum Event {
    Selected(ResolvedTarget),
    Sample(Box<EditorSample>),
    Disk(SourceObservation),
    /// Verified presence/absence metadata when standalone D text was unavailable.
    DiskMetadata {
        reason: SourceReason,
        file: Option<FileIdentity>,
        collection: CollectionStamp,
    },
    Rechecked(Recheck),
    DiskChecked(Vec<DetectedChange>),
    Failed(RoutingFailure),
    Done,
}

fn bad(stage: Stage) -> RoutingFailure {
    RoutingFailure::new(
        OutcomeKind::ProtocolError,
        DiagnosticCode::InvalidFrame,
        stage,
    )
}
fn timed(stage: Stage) -> RoutingFailure {
    RoutingFailure::new(
        OutcomeKind::Timeout,
        DiagnosticCode::DeadlineExceeded,
        stage,
    )
}
fn disconnected(stage: Stage) -> RoutingFailure {
    RoutingFailure::new(
        OutcomeKind::DisconnectedEditor,
        DiagnosticCode::SessionEnded,
        stage,
    )
}

// Serde's recursion limit is 128. Enforce our smaller envelope before constructing
// any model record, including a malicious array/object inside an unknown field.
fn json_depth(bytes: &[u8], stage: Stage) -> Result<(), RoutingFailure> {
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    for &c in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                quoted = false;
            }
        } else {
            match c {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > 32 {
                        return Err(bad(stage));
                    }
                }
                b'}' | b']' => {
                    depth = depth.checked_sub(1).ok_or_else(|| bad(stage))?;
                }
                _ => {}
            }
        }
    }
    if quoted || depth != 0 {
        return Err(bad(stage));
    }
    Ok(())
}
fn decode<'a, T: Deserialize<'a>>(bytes: &'a [u8], stage: Stage) -> Result<T, RoutingFailure> {
    if bytes.is_empty() || bytes.len() > RESULT_LIMIT {
        return Err(bad(stage));
    }
    json_depth(bytes, stage)?;
    serde_json::from_slice(bytes).map_err(|_| bad(stage))
}

#[derive(Debug)]
struct Bounded<T, const LIMIT: usize = COLLECTION_LIMIT>(Vec<T>);
impl<'de, T: Deserialize<'de>, const LIMIT: usize> Deserialize<'de> for Bounded<T, LIMIT> {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Limited<T, const LIMIT: usize>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>, const LIMIT: usize> Visitor<'de> for Limited<T, LIMIT> {
            type Value = Bounded<T, LIMIT>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bounded array")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut result = Vec::new();
                while result.len() < LIMIT {
                    match seq.next_element()? {
                        Some(item) => result.push(item),
                        None => return Ok(Bounded(result)),
                    }
                }
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom("collection limit"));
                }
                Ok(Bounded(result))
            }
        }
        d.deserialize_seq(Limited::<T, LIMIT>(std::marker::PhantomData))
    }
}

// Too-large text is consumed and discarded *for this source alone*. Never retain
// a truncated prefix or treat it as an observed empty string.
#[derive(Debug)]
enum TextIn {
    Within(String),
    TooLarge,
}
impl<'de> Deserialize<'de> for TextIn {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct TextVisitor;
        impl<'de> Visitor<'de> for TextVisitor {
            type Value = TextIn;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("UTF-8 source text")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                if s.len() > SOURCE_LIMIT_BYTES {
                    Ok(TextIn::TooLarge)
                } else {
                    Ok(TextIn::Within(s.to_owned()))
                }
            }
            fn visit_borrowed_str<E: de::Error>(self, s: &'de str) -> Result<Self::Value, E> {
                self.visit_str(s)
            }
            fn visit_string<E: de::Error>(self, s: String) -> Result<Self::Value, E> {
                if s.len() > SOURCE_LIMIT_BYTES {
                    Ok(TextIn::TooLarge)
                } else {
                    Ok(TextIn::Within(s))
                }
            }
        }
        d.deserialize_string(TextVisitor)
    }
}

macro_rules! names {
    ($ty:ident { $($variant:ident => $name:literal),+ $(,)? }) => {
        #[allow(non_snake_case)]
        fn $ty(v: $ty) -> &'static str { match v { $($ty::$variant => $name),+ } }
    };
}
// Enum strings are explicit wire values, not derived from Rust Debug spelling.
names!(Authority { D => "D", R => "R", B => "B" });
names!(Availability { Observed => "observed", Unavailable => "unavailable", NotApplicable => "not_applicable" });
names!(ScriptKind { ExternalGdscript => "external_gdscript", BuiltinGdscript => "builtin_gdscript" });
names!(SourceReason { TooLarge => "too_large", DiskMissing => "disk_missing", DiskUnreadable => "disk_unreadable", InvalidUtf8 => "invalid_utf8", NoStandaloneDiskSource => "no_standalone_disk_source", ResourceNotLoaded => "resource_not_loaded", ResourceUnreadable => "resource_unreadable", BufferUnreadable => "buffer_unreadable", BufferAttributionUnavailable => "buffer_attribution_unavailable", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", UnsupportedCapability => "unsupported_capability", DocumentNotOpen => "document_not_open" });
names!(DirtyReason { DirtyAttributionUnavailable => "dirty_attribution_unavailable", UnsupportedCapability => "unsupported_capability", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", DocumentNotOpen => "document_not_open" });
names!(FactReason { Unavailable => "unavailable", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded" });
names!(Validity { Valid => "valid", Missing => "missing", Invalid => "invalid" });
names!(OpenState { Open => "open", NotOpen => "not_open" });
names!(DirtyState { Dirty => "dirty", Clean => "clean" });
names!(RecheckReason { Unavailable => "unavailable", DeadlineExceeded => "deadline_exceeded", SessionEnded => "session_ended", UnsupportedCapability => "unsupported_capability" });
names!(Stability { Unknown => "unknown", Changed => "changed" });
names!(Checks { Performed => "performed", Unavailable => "unavailable" });
names!(Comparison { Equal => "equal", Different => "different", Unknown => "unknown" });
names!(Agreement { Agree => "agree", Divergent => "divergent", Unknown => "unknown" });
names!(OutcomeKind { CompleteObservation => "complete_observation", LimitedObservation => "limited_observation", NotOpen => "not_open", AmbiguousTarget => "ambiguous_target", MissingTarget => "missing_target", InvalidTarget => "invalid_target", EditorUnavailable => "editor_unavailable", DisconnectedEditor => "disconnected_editor", Timeout => "timeout", UnsupportedObservation => "unsupported_observation", DeniedAccess => "denied_access", InvalidRequest => "invalid_request", ProtocolError => "protocol_error", Cancelled => "cancelled" });
names!(Stage { ValidateRequest => "validate_request", ResolveTarget => "resolve_target", Authenticate => "authenticate", AttributeDocument => "attribute_document", ReadDisk => "read_disk", ReadEditor => "read_editor", Recheck => "recheck", Finalize => "finalize" });
names!(Surface { D => "D", R => "R", B => "B", Dirty => "dirty", Document => "document", Session => "session" });
names!(DiagnosticCode { DirtyAttributionUnavailable => "dirty_attribution_unavailable", BufferAttributionUnavailable => "buffer_attribution_unavailable", BufferUnreadable => "buffer_unreadable", ResourceNotLoaded => "resource_not_loaded", ResourceUnreadable => "resource_unreadable", DiskMissing => "disk_missing", DiskUnreadable => "disk_unreadable", InvalidUtf8 => "invalid_utf8", TooLarge => "too_large", NoStandaloneDiskSource => "no_standalone_disk_source", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", OpenStateUnknown => "open_state_unknown", DocumentStateUnavailable => "document_state_unavailable", UnsupportedVersion => "unsupported_version", UnsupportedCapability => "unsupported_capability", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", OutOfProject => "out_of_project", UnsafeRegistry => "unsafe_registry", AuthenticationFailed => "authentication_failed", InvalidFrame => "invalid_frame", Cancelled => "cancelled", AmbiguousTarget => "ambiguous_target", MissingTarget => "missing_target", InvalidTarget => "invalid_target", EditorUnavailable => "editor_unavailable", UnsupportedObservation => "unsupported_observation", DeniedAccess => "denied_access", InvalidRequest => "invalid_request", ProtocolError => "protocol_error", RecheckUnavailable => "recheck_unavailable" });

macro_rules! wire_enum {
    ($type:ident { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = <&str>::deserialize(d)?;
                match s { $($name => Ok(Self::$variant),)+ _ => Err(de::Error::custom("invalid enum")) }
            }
        }
    };
}
wire_enum!(Authority { D => "D", R => "R", B => "B" });
wire_enum!(ScriptKind { ExternalGdscript => "external_gdscript", BuiltinGdscript => "builtin_gdscript" });
wire_enum!(SourceReason { TooLarge => "too_large", DiskMissing => "disk_missing", DiskUnreadable => "disk_unreadable", InvalidUtf8 => "invalid_utf8", NoStandaloneDiskSource => "no_standalone_disk_source", ResourceNotLoaded => "resource_not_loaded", ResourceUnreadable => "resource_unreadable", BufferUnreadable => "buffer_unreadable", BufferAttributionUnavailable => "buffer_attribution_unavailable", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", UnsupportedCapability => "unsupported_capability", DocumentNotOpen => "document_not_open" });
wire_enum!(DirtyReason { DirtyAttributionUnavailable => "dirty_attribution_unavailable", UnsupportedCapability => "unsupported_capability", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", DocumentNotOpen => "document_not_open" });
wire_enum!(FactReason { Unavailable => "unavailable", OpenStateUnknown => "open_state_unknown", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded" });
wire_enum!(Validity { Valid => "valid", Missing => "missing", Invalid => "invalid" });
wire_enum!(OpenState { Open => "open", NotOpen => "not_open" });
wire_enum!(DirtyState { Dirty => "dirty", Clean => "clean" });
wire_enum!(RecheckReason { Unavailable => "unavailable", DeadlineExceeded => "deadline_exceeded", SessionEnded => "session_ended", UnsupportedCapability => "unsupported_capability" });
wire_enum!(OutcomeKind { CompleteObservation => "complete_observation", LimitedObservation => "limited_observation", NotOpen => "not_open", AmbiguousTarget => "ambiguous_target", MissingTarget => "missing_target", InvalidTarget => "invalid_target", EditorUnavailable => "editor_unavailable", DisconnectedEditor => "disconnected_editor", Timeout => "timeout", UnsupportedObservation => "unsupported_observation", DeniedAccess => "denied_access", InvalidRequest => "invalid_request", ProtocolError => "protocol_error", Cancelled => "cancelled" });
wire_enum!(Stage { ValidateRequest => "validate_request", ResolveTarget => "resolve_target", Authenticate => "authenticate", AttributeDocument => "attribute_document", ReadDisk => "read_disk", ReadEditor => "read_editor", Recheck => "recheck", Finalize => "finalize" });
wire_enum!(Surface { D => "D", R => "R", B => "B", Dirty => "dirty", Document => "document", Session => "session" });
wire_enum!(DiagnosticCode { DirtyAttributionUnavailable => "dirty_attribution_unavailable", BufferAttributionUnavailable => "buffer_attribution_unavailable", BufferUnreadable => "buffer_unreadable", ResourceNotLoaded => "resource_not_loaded", ResourceUnreadable => "resource_unreadable", DiskMissing => "disk_missing", DiskUnreadable => "disk_unreadable", InvalidUtf8 => "invalid_utf8", TooLarge => "too_large", NoStandaloneDiskSource => "no_standalone_disk_source", IdentityChanged => "identity_changed", SourceChanged => "source_changed", DocumentClosed => "document_closed", OpenStateUnknown => "open_state_unknown", DocumentStateUnavailable => "document_state_unavailable", UnsupportedVersion => "unsupported_version", UnsupportedCapability => "unsupported_capability", SessionEnded => "session_ended", DeadlineExceeded => "deadline_exceeded", OutOfProject => "out_of_project", UnsafeRegistry => "unsafe_registry", AuthenticationFailed => "authentication_failed", InvalidFrame => "invalid_frame", Cancelled => "cancelled", AmbiguousTarget => "ambiguous_target", MissingTarget => "missing_target", InvalidTarget => "invalid_target", EditorUnavailable => "editor_unavailable", UnsupportedObservation => "unsupported_observation", DeniedAccess => "denied_access", InvalidRequest => "invalid_request", ProtocolError => "protocol_error", RecheckUnavailable => "recheck_unavailable" });

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileIn {
    device: String,
    inode: String,
}
impl FileIn {
    fn domain(self, stage: Stage) -> Result<FileIdentity, RoutingFailure> {
        Ok(FileIdentity::new(
            decimal(self.device, stage)?,
            decimal(self.inode, stage)?,
        ))
    }
}
fn decimal(s: String, stage: Stage) -> Result<DecimalCounter, RoutingFailure> {
    if s.len() > 64 {
        return Err(bad(stage));
    }
    DecimalCounter::new(s).map_err(|_| bad(stage))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StampIn {
    clock_id: String,
    started_tick_us: String,
    finished_tick_us: String,
    received_elapsed_us: u64,
}
impl StampIn {
    fn domain(
        self,
        session: &SessionId,
        receipt: u64,
        stage: Stage,
        editor: bool,
    ) -> Result<CollectionStamp, RoutingFailure> {
        let clock_matches = if editor {
            self.clock_id.strip_prefix("editor:") == Some(session.as_str())
        } else {
            self.clock_id == "caller"
        };
        if !clock_matches || self.received_elapsed_us > receipt {
            return Err(bad(stage));
        }
        CollectionStamp::new(
            if editor {
                ClockId::Editor(session.clone())
            } else {
                ClockId::Caller
            },
            decimal(self.started_tick_us, stage)?,
            decimal(self.finished_tick_us, stage)?,
            receipt,
        )
        .map_err(|_| bad(stage))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WitnessIn {
    resource_path: String,
    script_instance_id: Option<String>,
    editor_instance_id: Option<String>,
    buffer_instance_id: Option<String>,
    disk_file_id: Option<FileIn>,
    source_version: Option<String>,
}
impl WitnessIn {
    fn domain(self, path: &ResourcePath, stage: Stage) -> Result<Witness, RoutingFailure> {
        if self.resource_path != path.as_str() {
            return Err(bad(stage));
        }
        Ok(Witness::new(
            path.clone(),
            self.script_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.editor_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.buffer_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.disk_file_id.map(|v| v.domain(stage)).transpose()?,
            self.source_version.map(|v| decimal(v, stage)).transpose()?,
        ))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StalenessIn {
    state: String,
    evidence: Option<Bounded<StaleEvidenceIn>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StaleEvidenceIn {
    incorporated_version: String,
    changed_authority: Authority,
    changed_version: String,
    collection: StampIn,
    witness: WitnessIn,
}
impl StalenessIn {
    fn domain(
        self,
        path: &ResourcePath,
        session: &SessionId,
        receipt: u64,
        stage: Stage,
        source_authority: Authority,
    ) -> Result<Staleness, RoutingFailure> {
        match (self.state.as_str(), self.evidence) {
            ("unknown", None) => Ok(Staleness::unknown()),
            ("known_stale", Some(items)) => {
                let evidence = items
                    .0
                    .into_iter()
                    .map(|e| {
                        if e.changed_authority == source_authority
                            || (source_authority != Authority::D
                                && e.changed_authority == Authority::D)
                        {
                            return Err(bad(stage));
                        }
                        let editor = e.changed_authority != Authority::D;
                        StalenessEvidence::unapplied_change(
                            decimal(e.incorporated_version, stage)?,
                            e.changed_authority,
                            decimal(e.changed_version, stage)?,
                            e.collection.domain(session, receipt, stage, editor)?,
                            e.witness.domain(path, stage)?,
                        )
                        .map_err(|_| bad(stage))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Staleness::known_stale(evidence).map_err(|_| bad(stage))
            }
            _ => Err(bad(stage)),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceIn {
    authority: Authority,
    availability: AvailabilityIn,
    text: Option<TextIn>,
    collection: Option<StampIn>,
    witness: Option<WitnessIn>,
    staleness: Option<StalenessIn>,
    reason: Option<ReasonIn<SourceReason>>,
    invalidated_evidence: Option<InvalidatedSourceIn>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AvailabilityIn {
    Observed,
    Unavailable,
    NotApplicable,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReasonIn<T> {
    code: T,
    action: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidatedSourceIn {
    text: TextIn,
    collection: StampIn,
    witness: WitnessIn,
    staleness: StalenessIn,
    reason: ReasonIn<SourceReason>,
}
fn source_reason(r: ReasonIn<SourceReason>, stage: Stage) -> Result<SourceReason, RoutingFailure> {
    if r.action != r.code.action() {
        return Err(bad(stage));
    }
    Ok(r.code)
}
impl SourceIn {
    fn domain(
        self,
        authority: Authority,
        target: &ResolvedTarget,
        receipt: u64,
        stage: Stage,
        identity: Option<&DocumentIdentity>,
        interval: Option<&CollectionStamp>,
    ) -> Result<SourceObservation, RoutingFailure> {
        let path = target.script_path();
        let session = target.session_id();
        if self.authority != authority {
            return Err(bad(stage));
        }
        let editor = authority != Authority::D;
        if editor
            && identity.is_none()
            && (matches!(self.availability, AvailabilityIn::Observed)
                || self.invalidated_evidence.is_some())
        {
            return Err(bad(stage));
        }
        let current = match self.availability {
            AvailabilityIn::Observed => {
                if self.invalidated_evidence.is_some() || self.reason.is_some() {
                    return Err(bad(stage));
                }
                let (Some(text), Some(stamp), Some(witness), Some(stale)) =
                    (self.text, self.collection, self.witness, self.staleness)
                else {
                    return Err(bad(stage));
                };
                let stamp = stamp.domain(session, receipt, stage, editor)?;
                if let Some(interval) = interval {
                    within_editor_interval(&stamp, interval, stage)?;
                }
                let witness = witness.domain(path, stage)?;
                let stale = stale.domain(path, session, receipt, stage, authority)?;
                if let Some(interval) = interval {
                    for evidence in stale.evidence().into_iter().flatten() {
                        if evidence.changed_authority() != Authority::D {
                            within_editor_interval(evidence.collection(), interval, stage)?;
                        }
                    }
                }
                if !editor && stale.evidence().is_some() {
                    return Err(bad(stage));
                }
                if matches!(&text, TextIn::TooLarge) {
                    if let Some(identity) = identity {
                        check_witness(&witness, identity, authority, stage)?;
                        if let Some(evidence) = stale.evidence() {
                            for e in evidence {
                                check_witness(e.witness(), identity, e.changed_authority(), stage)?;
                            }
                        }
                    } else if witness.disk_file_id().is_none()
                        || witness.script_instance_id().is_some()
                        || witness.editor_instance_id().is_some()
                        || witness.buffer_instance_id().is_some()
                        || stale.evidence().is_some()
                    {
                        return Err(bad(stage));
                    }
                }
                match text {
                    TextIn::Within(text) => {
                        SourceObservation::observed(authority, text, stamp, witness, stale)
                    }
                    TextIn::TooLarge => {
                        SourceObservation::unavailable(authority, SourceReason::TooLarge)
                            .map_err(|_| bad(stage))?
                    }
                }
            }
            AvailabilityIn::Unavailable => {
                if self.text.is_some()
                    || self.collection.is_some()
                    || self.witness.is_some()
                    || self.staleness.is_some()
                {
                    return Err(bad(stage));
                }
                let reason = source_reason(self.reason.ok_or_else(|| bad(stage))?, stage)?;
                let unavailable =
                    SourceObservation::unavailable(authority, reason).map_err(|_| bad(stage))?;
                if let Some(old) = self.invalidated_evidence {
                    let stamp = old.collection.domain(session, receipt, stage, editor)?;
                    if let Some(interval) = interval {
                        within_editor_interval(&stamp, interval, stage)?;
                    }
                    let witness = old.witness.domain(path, stage)?;
                    let stale = old
                        .staleness
                        .domain(path, session, receipt, stage, authority)?;
                    if let Some(interval) = interval {
                        for evidence in stale.evidence().into_iter().flatten() {
                            if evidence.changed_authority() != Authority::D {
                                within_editor_interval(evidence.collection(), interval, stage)?;
                            }
                        }
                    }
                    let old_reason = source_reason(old.reason, stage)?;
                    if !editor && stale.evidence().is_some() {
                        return Err(bad(stage));
                    }
                    if let Some(identity) = identity {
                        check_witness(&witness, identity, authority, stage)?;
                        for evidence in stale.evidence().into_iter().flatten() {
                            check_witness(
                                evidence.witness(),
                                identity,
                                evidence.changed_authority(),
                                stage,
                            )?;
                        }
                    } else if witness.disk_file_id().is_none()
                        || witness.script_instance_id().is_some()
                        || witness.editor_instance_id().is_some()
                        || witness.buffer_instance_id().is_some()
                    {
                        return Err(bad(stage));
                    }
                    match old.text {
                        TextIn::Within(text) => {
                            let mut source =
                                SourceObservation::observed(authority, text, stamp, witness, stale);
                            source.invalidate(old_reason).map_err(|_| bad(stage))?;
                            source.invalidate(reason).map_err(|_| bad(stage))?;
                            source
                        }
                        TextIn::TooLarge => {
                            SourceObservation::unavailable(authority, SourceReason::TooLarge)
                                .map_err(|_| bad(stage))?
                        }
                    }
                } else {
                    unavailable
                }
            }
            AvailabilityIn::NotApplicable => {
                if authority != Authority::B
                    || self.text.is_some()
                    || self.collection.is_some()
                    || self.witness.is_some()
                    || self.staleness.is_some()
                    || self.invalidated_evidence.is_some()
                    || source_reason(self.reason.ok_or_else(|| bad(stage))?, stage)?
                        != SourceReason::DocumentNotOpen
                {
                    return Err(bad(stage));
                }
                SourceObservation::closed_buffer()
            }
        };
        Ok(current)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityIn {
    kind: ScriptKind,
    resource_path: String,
    script_instance_id: Option<String>,
    editor_instance_id: Option<String>,
    buffer_instance_id: Option<String>,
    disk_file_id: Option<FileIn>,
}
impl IdentityIn {
    fn domain(self, path: &ResourcePath, stage: Stage) -> Result<DocumentIdentity, RoutingFailure> {
        if self.resource_path != path.as_str() {
            return Err(bad(stage));
        }
        DocumentIdentity::new(
            self.kind,
            path.clone(),
            self.script_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.editor_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.buffer_instance_id
                .map(|v| decimal(v, stage))
                .transpose()?,
            self.disk_file_id.map(|v| v.domain(stage)).transpose()?,
        )
        .map_err(|_| bad(stage))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactIn<T> {
    value: Option<T>,
    collection: Option<StampIn>,
    reason: Option<ReasonIn<FactReason>>,
    invalidated_evidence: Option<InvalidatedFactIn<T>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidatedFactIn<T> {
    value: T,
    collection: StampIn,
    reason: ReasonIn<FactReason>,
}
fn fact<T>(
    value: FactIn<T>,
    session: &SessionId,
    receipt: u64,
    stage: Stage,
    editor: bool,
) -> Result<DocumentFact<T>, RoutingFailure> {
    match (
        value.value,
        value.collection,
        value.reason,
        value.invalidated_evidence,
    ) {
        (Some(v), Some(stamp), None, None) => Ok(DocumentFact::observed(
            v,
            stamp.domain(session, receipt, stage, editor)?,
        )),
        (None, None, Some(reason), old) if reason.action == reason.code.action() => {
            if let Some(old) = old {
                if old.reason.action != old.reason.code.action() {
                    return Err(bad(stage));
                }
                let mut fact = DocumentFact::observed(
                    old.value,
                    old.collection.domain(session, receipt, stage, editor)?,
                );
                fact.invalidate(old.reason.code);
                fact.invalidate(reason.code);
                Ok(fact)
            } else {
                Ok(DocumentFact::unknown(reason.code))
            }
        }
        _ => Err(bad(stage)),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentIn {
    identity: Option<IdentityIn>,
    validity: FactIn<Validity>,
    open_state: FactIn<OpenState>,
}
impl DocumentIn {
    fn domain(
        self,
        path: &ResourcePath,
        session: &SessionId,
        receipt: u64,
        stage: Stage,
    ) -> Result<DocumentState, RoutingFailure> {
        Ok(DocumentState::new(
            self.identity.map(|v| v.domain(path, stage)).transpose()?,
            fact(self.validity, session, receipt, stage, true)?,
            fact(self.open_state, session, receipt, stage, true)?,
        ))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DirtyIn {
    availability: AvailabilityIn,
    state: String,
    collection: Option<StampIn>,
    witness: Option<WitnessIn>,
    reason: Option<ReasonIn<DirtyReason>>,
    invalidated_evidence: Option<InvalidatedDirtyIn>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidatedDirtyIn {
    state: DirtyState,
    collection: StampIn,
    witness: WitnessIn,
    reason: ReasonIn<DirtyReason>,
}
impl DirtyIn {
    fn domain(
        self,
        path: &ResourcePath,
        session: &SessionId,
        receipt: u64,
        stage: Stage,
    ) -> Result<DirtyObservation, RoutingFailure> {
        match self.availability {
            AvailabilityIn::Observed
                if self.reason.is_none() && self.invalidated_evidence.is_none() =>
            {
                let state = match self.state.as_str() {
                    "dirty" => DirtyState::Dirty,
                    "clean" => DirtyState::Clean,
                    _ => return Err(bad(stage)),
                };
                Ok(DirtyObservation::observed(
                    state,
                    self.collection
                        .ok_or_else(|| bad(stage))?
                        .domain(session, receipt, stage, true)?,
                    self.witness
                        .ok_or_else(|| bad(stage))?
                        .domain(path, stage)?,
                ))
            }
            AvailabilityIn::Unavailable
                if self.state == "unknown"
                    && self.collection.is_none()
                    && self.witness.is_none() =>
            {
                let reason = self.reason.ok_or_else(|| bad(stage))?;
                if reason.action != reason.code.action() {
                    return Err(bad(stage));
                }
                if let Some(old) = self.invalidated_evidence {
                    if old.reason.action != old.reason.code.action() {
                        return Err(bad(stage));
                    }
                    let mut dirty = DirtyObservation::observed(
                        old.state,
                        old.collection.domain(session, receipt, stage, true)?,
                        old.witness.domain(path, stage)?,
                    );
                    dirty.invalidate(old.reason.code).map_err(|_| bad(stage))?;
                    dirty.invalidate(reason.code).map_err(|_| bad(stage))?;
                    Ok(dirty)
                } else {
                    DirtyObservation::unavailable(reason.code).map_err(|_| bad(stage))
                }
            }
            AvailabilityIn::NotApplicable
                if self.state == "not_applicable"
                    && self.collection.is_none()
                    && self.witness.is_none()
                    && self.invalidated_evidence.is_none()
                    && self.reason.is_some_and(|r| {
                        r.code == DirtyReason::DocumentNotOpen && r.action == r.code.action()
                    }) =>
            {
                Ok(DirtyObservation::closed_document())
            }
            _ => Err(bad(stage)),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticIn {
    code: DiagnosticCode,
    stage: Stage,
    surface: Option<Surface>,
    message: String,
    action: String,
}
impl DiagnosticIn {
    fn domain(self, stage: Stage) -> Result<Diagnostic, RoutingFailure> {
        let d = Diagnostic::new(self.code, self.stage, self.surface);
        if self.message != d.message() || self.action != d.action() {
            return Err(bad(stage));
        }
        Ok(d)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    document: DocumentIn,
    #[serde(rename = "R")]
    resource: SourceIn,
    #[serde(rename = "B")]
    buffer: SourceIn,
    dirty: DirtyIn,
    diagnostics: Bounded<DiagnosticIn>,
}
impl SampleIn {
    fn domain(
        self,
        request: &ObservationRequest,
        target: &ResolvedTarget,
        advertised_root: &str,
        receipt: u64,
        stage: Stage,
    ) -> Result<EditorSample, RoutingFailure> {
        if self.v != 4
            || self.kind != "sample"
            || self.request_id != request.request_id().as_str()
            || self.session_id != target.session_id().as_str()
            || self.project_root != advertised_root
            || self.script_path != request.script_path().as_str()
        {
            return Err(bad(stage));
        }
        let collection = self
            .collection
            .domain(target.session_id(), receipt, stage, true)?;
        let document =
            self.document
                .domain(target.script_path(), target.session_id(), receipt, stage)?;
        let resource = self.resource.domain(
            Authority::R,
            target,
            receipt,
            stage,
            document.identity(),
            Some(&collection),
        )?;
        let buffer = self.buffer.domain(
            Authority::B,
            target,
            receipt,
            stage,
            document.identity(),
            Some(&collection),
        )?;
        let sample = EditorSample {
            collection,
            document,
            resource,
            buffer,
            dirty: self
                .dirty
                .domain(target.script_path(), target.session_id(), receipt, stage)?,
            diagnostics: self
                .diagnostics
                .0
                .into_iter()
                .map(|d| d.domain(stage))
                .collect::<Result<Vec<_>, _>>()?,
        };
        validate_sample(&sample, target, receipt, stage)?;
        Ok(sample)
    }
}

fn check_stamp(
    stamp: &CollectionStamp,
    session: &SessionId,
    receipt: u64,
    editor: bool,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    if stamp.received_elapsed_us() > receipt
        || !match stamp.clock_id() {
            ClockId::Caller => !editor,
            ClockId::Editor(s) => editor && s == session,
        }
    {
        return Err(bad(stage));
    }
    Ok(())
}
fn within_editor_interval(
    stamp: &CollectionStamp,
    interval: &CollectionStamp,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    if stamp.clock_id() != interval.clock_id()
        || stamp.started_tick_us() < interval.started_tick_us()
        || stamp.finished_tick_us() > interval.finished_tick_us()
        || stamp.received_elapsed_us() != interval.received_elapsed_us()
    {
        return Err(bad(stage));
    }
    Ok(())
}
fn check_witness(
    w: &Witness,
    identity: &DocumentIdentity,
    authority: Authority,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    if w.resource_path() != identity.resource_path()
        || match authority {
            Authority::D => {
                w.disk_file_id() != identity.disk_file_id()
                    || w.script_instance_id().is_some()
                    || w.editor_instance_id().is_some()
                    || w.buffer_instance_id().is_some()
            }
            Authority::R => {
                w.script_instance_id() != identity.script_instance_id()
                    || w.disk_file_id().is_some()
                    || w.editor_instance_id().is_some()
                    || w.buffer_instance_id().is_some()
            }
            Authority::B => {
                w.editor_instance_id() != identity.editor_instance_id()
                    || w.buffer_instance_id() != identity.buffer_instance_id()
                    || w.disk_file_id().is_some()
                    || w.script_instance_id()
                        .zip(identity.script_instance_id())
                        .is_some_and(|(a, b)| a != b)
            }
        }
    {
        return Err(bad(stage));
    }
    Ok(())
}
fn check_source(
    s: &SourceObservation,
    identity: &DocumentIdentity,
    session: &SessionId,
    receipt: u64,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    let current = s
        .collection()
        .zip(s.witness())
        .map(|(stamp, witness)| (stamp, witness, s.staleness()));
    let old = s
        .invalidated_evidence()
        .map(|old| (old.collection(), old.witness(), Some(old.staleness())));
    for (stamp, witness, stale) in current.into_iter().chain(old) {
        check_stamp(
            stamp,
            session,
            receipt,
            s.authority() != Authority::D,
            stage,
        )?;
        check_witness(witness, identity, s.authority(), stage)?;
        for evidence in stale.and_then(Staleness::evidence).into_iter().flatten() {
            check_stamp(
                evidence.collection(),
                session,
                receipt,
                evidence.changed_authority() != Authority::D,
                stage,
            )?;
            check_witness(
                evidence.witness(),
                identity,
                evidence.changed_authority(),
                stage,
            )?;
        }
    }
    Ok(())
}
fn validate_sample(
    s: &EditorSample,
    target: &ResolvedTarget,
    receipt: u64,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    check_stamp(&s.collection, target.session_id(), receipt, true, stage)?;
    let identity = s.document.identity();
    if identity.is_some_and(|i| i.resource_path() != target.script_path()) {
        return Err(bad(stage));
    }
    for stamp in [
        s.document.validity().collection(),
        s.document.open_state().collection(),
        s.document
            .validity()
            .invalidated_evidence()
            .map(|old| old.collection()),
        s.document
            .open_state()
            .invalidated_evidence()
            .map(|old| old.collection()),
    ]
    .into_iter()
    .flatten()
    {
        check_stamp(stamp, target.session_id(), receipt, true, stage)?;
        within_editor_interval(stamp, &s.collection, stage)?;
    }
    if let Some(identity) = identity {
        if identity.disk_file_id().is_some() {
            return Err(bad(stage));
        }
        check_source(&s.resource, identity, target.session_id(), receipt, stage)?;
        check_source(&s.buffer, identity, target.session_id(), receipt, stage)?;
        if let (Some(stamp), Some(w)) = (s.dirty.collection(), s.dirty.witness()) {
            check_stamp(stamp, target.session_id(), receipt, true, stage)?;
            check_witness(w, identity, Authority::B, stage)?;
            within_editor_interval(stamp, &s.collection, stage)?;
        }
        if let Some(old) = s.dirty.invalidated_evidence() {
            check_stamp(old.collection(), target.session_id(), receipt, true, stage)?;
            check_witness(old.witness(), identity, Authority::B, stage)?;
            within_editor_interval(old.collection(), &s.collection, stage)?;
        }
    } else if s.resource.availability() == Availability::Observed
        || s.buffer.availability() == Availability::Observed
        || s.dirty.availability() == Availability::Observed
        || s.resource.invalidated_evidence().is_some()
        || s.buffer.invalidated_evidence().is_some()
        || s.dirty.invalidated_evidence().is_some()
    {
        return Err(bad(stage));
    }
    if matches!(
        s.document.validity().value(),
        Some(Validity::Missing | Validity::Invalid)
    ) && s.document.open_state().value() == Some(&OpenState::Open)
    {
        return Err(bad(stage));
    }
    match s.document.open_state().value() {
        Some(OpenState::NotOpen)
            if s.buffer.availability() != Availability::NotApplicable
                || s.dirty.availability() != Availability::NotApplicable
                || identity.is_some_and(|i| {
                    i.editor_instance_id().is_some() || i.buffer_instance_id().is_some()
                }) =>
        {
            return Err(bad(stage))
        }
        Some(OpenState::Open) | None
            if s.buffer.availability() == Availability::NotApplicable
                || s.dirty.availability() == Availability::NotApplicable =>
        {
            return Err(bad(stage))
        }
        None if s.buffer.availability() == Availability::Observed
            || s.dirty.availability() == Availability::Observed =>
        {
            return Err(bad(stage))
        }
        _ => (),
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecheckIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    collection: StampIn,
    checks: String,
    detected_changes: Bounded<ChangeIn>,
    reason: Option<RecheckReason>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeIn {
    surface: Surface,
    code: DiagnosticCode,
}
impl ChangeIn {
    fn domain(self, stage: Stage) -> Result<DetectedChange, RoutingFailure> {
        match (self.surface, self.code) {
            (Surface::D, DiagnosticCode::SourceChanged) => Ok(DetectedChange::Source(Authority::D)),
            (Surface::R, DiagnosticCode::SourceChanged) => Ok(DetectedChange::Source(Authority::R)),
            (Surface::B, DiagnosticCode::SourceChanged) => Ok(DetectedChange::Source(Authority::B)),
            (Surface::Dirty, DiagnosticCode::SourceChanged) => Ok(DetectedChange::Dirty),
            (Surface::Document, DiagnosticCode::DocumentClosed) => {
                Ok(DetectedChange::DocumentClosed)
            }
            (Surface::Document, DiagnosticCode::IdentityChanged) => {
                Ok(DetectedChange::DocumentIdentityReplaced)
            }
            (Surface::Session, DiagnosticCode::IdentityChanged) => {
                Ok(DetectedChange::SessionReplaced)
            }
            (Surface::Session, DiagnosticCode::SessionEnded) => Ok(DetectedChange::SessionEnded),
            (Surface::D, DiagnosticCode::IdentityChanged) => {
                Ok(DetectedChange::DiskIdentityReplaced)
            }
            _ => Err(bad(stage)),
        }
    }
}
fn editor_changes(changes: &[DetectedChange], stage: Stage) -> Result<(), RoutingFailure> {
    if changes.iter().any(|change| {
        matches!(
            change,
            DetectedChange::Source(Authority::D) | DetectedChange::DiskIdentityReplaced
        )
    }) {
        return Err(bad(stage));
    }
    Ok(())
}
impl RecheckIn {
    fn domain(
        self,
        request: &ObservationRequest,
        target: &ResolvedTarget,
        advertised_root: &str,
        receipt: u64,
        stage: Stage,
        original: &CollectionStamp,
    ) -> Result<Recheck, RoutingFailure> {
        if self.v != 4
            || self.kind != "recheck"
            || self.request_id != request.request_id().as_str()
            || self.session_id != target.session_id().as_str()
            || self.project_root != advertised_root
            || self.script_path != request.script_path().as_str()
        {
            return Err(bad(stage));
        }
        let collection = self
            .collection
            .domain(target.session_id(), receipt, stage, true)?;
        if collection.started_tick_us() < original.finished_tick_us()
            || collection.received_elapsed_us() < original.received_elapsed_us()
        {
            return Err(bad(stage));
        }
        let changes = self
            .detected_changes
            .0
            .into_iter()
            .map(|c| c.domain(stage))
            .collect::<Result<Vec<_>, _>>()?;
        editor_changes(&changes, stage)?;
        match self.checks.as_str() {
            "performed" if self.reason.is_none() => Ok(Recheck::performed(changes)),
            "unavailable" => {
                let reason = self.reason.ok_or_else(|| bad(stage))?;
                if changes.is_empty() {
                    Ok(Recheck::unavailable(reason))
                } else {
                    Ok(Recheck::partial(reason, changes))
                }
            }
            _ => Err(bad(stage)),
        }
    }
}

fn network(err: std::io::Error, stage: Stage) -> RoutingFailure {
    if matches!(
        err.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        timed(stage)
    } else {
        disconnected(stage)
    }
}
fn remaining(deadline: Instant, stage: Stage) -> Result<Duration, RoutingFailure> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        Err(timed(stage))
    } else {
        Ok(left)
    }
}
fn write_deadline(
    stream: &mut std::net::TcpStream,
    bytes: &[u8],
    deadline: Instant,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    let mut offset = 0;
    while offset < bytes.len() {
        stream
            .set_write_timeout(Some(remaining(deadline, stage)?))
            .map_err(|e| network(e, stage))?;
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err(disconnected(stage)),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(network(e, stage)),
        }
    }
    Ok(())
}
fn read_deadline(
    stream: &mut std::net::TcpStream,
    bytes: &mut [u8],
    deadline: Instant,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    let mut offset = 0;
    while offset < bytes.len() {
        stream
            .set_read_timeout(Some(remaining(deadline, stage)?))
            .map_err(|e| network(e, stage))?;
        match stream.read(&mut bytes[offset..]) {
            Ok(0) => return Err(disconnected(stage)),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(network(e, stage)),
        }
    }
    Ok(())
}
fn send_editor<T: Serialize>(
    stream: &mut std::net::TcpStream,
    frame: T,
    deadline: Instant,
    stage: Stage,
) -> Result<(), RoutingFailure> {
    let bytes = serde_json::to_vec(&frame).map_err(|_| bad(stage))?;
    if bytes.len() > 4096 {
        return Err(bad(stage));
    }
    write_deadline(stream, &(bytes.len() as u32).to_be_bytes(), deadline, stage)?;
    write_deadline(stream, &bytes, deadline, stage)
}
fn receive_editor(
    stream: &mut std::net::TcpStream,
    deadline: Instant,
    stage: Stage,
) -> Result<Vec<u8>, RoutingFailure> {
    let mut prefix = [0; 4];
    read_deadline(stream, &mut prefix, deadline, stage)?;
    let length = u32::from_be_bytes(prefix) as usize;
    if length == 0 || length > RESULT_LIMIT {
        return Err(bad(stage));
    }
    let mut bytes = vec![0; length];
    read_deadline(stream, &mut bytes, deadline, stage)?;
    Ok(bytes)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditorFailureIn {
    v: u32,
    kind: String,
    request_id: String,
    session_id: String,
    project_root: String,
    script_path: String,
    code: DiagnosticCode,
    stage: Stage,
}
impl EditorFailureIn {
    fn domain(
        self,
        request: &ObservationRequest,
        target: &ResolvedTarget,
        advertised_root: &str,
        stage: Stage,
    ) -> Result<RoutingFailure, RoutingFailure> {
        if self.v != 4
            || self.kind != "failure"
            || self.request_id != request.request_id().as_str()
            || self.session_id != target.session_id().as_str()
            || self.project_root != advertised_root
            || self.script_path != request.script_path().as_str()
            || self.stage != stage
        {
            return Err(bad(stage));
        }
        let outcome = match self.code {
            DiagnosticCode::OutOfProject => OutcomeKind::DeniedAccess,
            DiagnosticCode::UnsupportedObservation if stage == Stage::ReadEditor => {
                OutcomeKind::UnsupportedObservation
            }
            _ => return Err(bad(stage)),
        };
        Ok(RoutingFailure::new(outcome, self.code, stage))
    }
}
fn editor_reply<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    request: &ObservationRequest,
    target: &ResolvedTarget,
    advertised_root: &str,
    stage: Stage,
) -> Result<T, RoutingFailure> {
    match decode(bytes, stage) {
        Ok(reply) => Ok(reply),
        Err(_) => match decode::<EditorFailureIn>(bytes, stage) {
            Ok(failure) => Err(failure.domain(request, target, advertised_root, stage)?),
            Err(_) => Err(bad(stage)),
        },
    }
}

/// Collect one editor sample on the uniquely selected, request-bound channel.
/// No disk acquisition or editor mutation is performed by this boundary.
///
/// # Errors
/// Refuses another request/target, absent capability, invalid or oversized frames,
/// inconsistent evidence, known disconnection, or deadline expiry. Errors are redacted.
pub fn observe(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    started: Instant,
    deadline: Instant,
) -> Result<EditorSample, RoutingFailure> {
    let stage = Stage::ReadEditor;
    if !selected.matches_request(request) {
        return Err(bad(stage));
    }
    if !selected.capabilities().observe_gdscript {
        return Err(RoutingFailure::new(
            OutcomeKind::UnsupportedObservation,
            DiagnosticCode::UnsupportedCapability,
            stage,
        ));
    }
    let (socket, target, advertised_root) = selected.observation_channel();
    send_editor(
        socket,
        (
            4,
            "observe",
            request.request_id().as_str(),
            target.session_id().as_str(),
            advertised_root,
            request.script_path().as_str(),
        ),
        deadline,
        stage,
    )?;
    let bytes = receive_editor(&mut selected.socket, deadline, stage)?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    editor_reply::<SampleIn>(
        &bytes,
        request,
        selected.target(),
        &selected.advertised_project_root,
        stage,
    )?
    .domain(
        request,
        selected.target(),
        &selected.advertised_project_root,
        receipt,
        stage,
    )
}
/// Independently recheck the selected editor after the original sample interval.
///
/// # Errors
/// Returns a redacted identity/protocol, disconnection, or deadline failure. A
/// limited recheck can retain detected changes without claiming all checks ran.
pub fn recheck(
    selected: &mut SelectedSession,
    request: &ObservationRequest,
    original: &CollectionStamp,
    started: Instant,
    deadline: Instant,
) -> Result<Recheck, RoutingFailure> {
    let stage = Stage::Recheck;
    if !selected.matches_request(request) {
        return Err(bad(stage));
    }
    let (socket, target, advertised_root) = selected.observation_channel();
    send_editor(
        socket,
        (
            4,
            "recheck",
            request.request_id().as_str(),
            target.session_id().as_str(),
            advertised_root,
            request.script_path().as_str(),
        ),
        deadline,
        stage,
    )?;
    let bytes = receive_editor(&mut selected.socket, deadline, stage)?;
    let receipt = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
    check_stamp(
        original,
        selected.target().session_id(),
        receipt,
        true,
        stage,
    )?;
    editor_reply::<RecheckIn>(
        &bytes,
        request,
        selected.target(),
        &selected.advertised_project_root,
        stage,
    )?
    .domain(
        request,
        selected.target(),
        &selected.advertised_project_root,
        receipt,
        stage,
        original,
    )
}

// Borrowed result records: source strings are never copied into a generic JSON Value.
#[derive(Serialize)]
struct FileOut<'a> {
    device: &'a str,
    inode: &'a str,
}
fn file_out(f: &FileIdentity) -> FileOut<'_> {
    FileOut {
        device: f.device().as_str(),
        inode: f.inode().as_str(),
    }
}
struct EditorClock<'a>(&'a str);
impl std::fmt::Display for EditorClock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "editor:{}", self.0)
    }
}
struct ClockOut<'a>(&'a ClockId);
impl Serialize for ClockOut<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            ClockId::Caller => serializer.serialize_str("caller"),
            ClockId::Editor(session) => serializer.collect_str(&EditorClock(session.as_str())),
        }
    }
}
#[derive(Serialize)]
struct StampOut<'a> {
    clock_id: ClockOut<'a>,
    started_tick_us: &'a str,
    finished_tick_us: &'a str,
    received_elapsed_us: u64,
}
fn stamp_out(s: &CollectionStamp) -> StampOut<'_> {
    StampOut {
        clock_id: ClockOut(s.clock_id()),
        started_tick_us: s.started_tick_us().as_str(),
        finished_tick_us: s.finished_tick_us().as_str(),
        received_elapsed_us: s.received_elapsed_us(),
    }
}
#[derive(Serialize)]
struct WitnessOut<'a> {
    resource_path: &'a str,
    script_instance_id: Option<&'a str>,
    editor_instance_id: Option<&'a str>,
    buffer_instance_id: Option<&'a str>,
    disk_file_id: Option<FileOut<'a>>,
    source_version: Option<&'a str>,
}
fn witness_out(w: &Witness) -> WitnessOut<'_> {
    WitnessOut {
        resource_path: w.resource_path().as_str(),
        script_instance_id: w.script_instance_id().map(DecimalCounter::as_str),
        editor_instance_id: w.editor_instance_id().map(DecimalCounter::as_str),
        buffer_instance_id: w.buffer_instance_id().map(DecimalCounter::as_str),
        disk_file_id: w.disk_file_id().map(file_out),
        source_version: w.source_version().map(DecimalCounter::as_str),
    }
}
#[derive(Serialize)]
struct ReasonOut {
    code: &'static str,
    action: &'static str,
}
fn source_reason_out(reason: SourceReason) -> ReasonOut {
    ReasonOut {
        code: SourceReason(reason),
        action: reason.action(),
    }
}
fn dirty_reason_out(reason: DirtyReason) -> ReasonOut {
    ReasonOut {
        code: DirtyReason(reason),
        action: reason.action(),
    }
}
fn fact_reason_out(reason: FactReason) -> ReasonOut {
    ReasonOut {
        code: FactReason(reason),
        action: reason.action(),
    }
}
#[derive(Serialize)]
struct StaleOut<'a> {
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<StaleEvidenceList<'a>>,
}
#[derive(Serialize)]
struct StaleEvidenceOut<'a> {
    incorporated_version: &'a str,
    changed_authority: &'static str,
    changed_version: &'a str,
    collection: StampOut<'a>,
    witness: WitnessOut<'a>,
}
struct StaleEvidenceList<'a>(&'a [StalenessEvidence]);
impl Serialize for StaleEvidenceList<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for e in self.0 {
            seq.serialize_element(&StaleEvidenceOut {
                incorporated_version: e.incorporated_version().as_str(),
                changed_authority: Authority(e.changed_authority()),
                changed_version: e.changed_version().as_str(),
                collection: stamp_out(e.collection()),
                witness: witness_out(e.witness()),
            })?;
        }
        seq.end()
    }
}
fn stale_out(s: &Staleness) -> StaleOut<'_> {
    match s.evidence() {
        None => StaleOut {
            state: "unknown",
            evidence: None,
        },
        Some(items) => StaleOut {
            state: "known_stale",
            evidence: Some(StaleEvidenceList(items)),
        },
    }
}
#[derive(Serialize)]
struct InvalidSourceOut<'a> {
    text: &'a str,
    collection: StampOut<'a>,
    witness: WitnessOut<'a>,
    staleness: StaleOut<'a>,
    reason: ReasonOut,
}
#[derive(Serialize)]
struct SourceOut<'a> {
    authority: &'static str,
    availability: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    collection: Option<StampOut<'a>>,
    witness: Option<WitnessOut<'a>>,
    staleness: Option<StaleOut<'a>>,
    reason: Option<ReasonOut>,
    invalidated_evidence: Option<InvalidSourceOut<'a>>,
}
fn source_out(s: &SourceObservation) -> SourceOut<'_> {
    SourceOut {
        authority: Authority(s.authority()),
        availability: Availability(s.availability()),
        text: s.text(),
        collection: s.collection().map(stamp_out),
        witness: s.witness().map(witness_out),
        staleness: s.staleness().map(stale_out),
        reason: s.reason().map(source_reason_out),
        invalidated_evidence: s.invalidated_evidence().map(|old| InvalidSourceOut {
            text: old.text(),
            collection: stamp_out(old.collection()),
            witness: witness_out(old.witness()),
            staleness: stale_out(old.staleness()),
            reason: source_reason_out(old.reason()),
        }),
    }
}
#[derive(Serialize)]
struct IdentityOut<'a> {
    kind: &'static str,
    resource_path: &'a str,
    script_instance_id: Option<&'a str>,
    editor_instance_id: Option<&'a str>,
    buffer_instance_id: Option<&'a str>,
    disk_file_id: Option<FileOut<'a>>,
}
fn identity_out(i: &DocumentIdentity) -> IdentityOut<'_> {
    IdentityOut {
        kind: ScriptKind(i.kind()),
        resource_path: i.resource_path().as_str(),
        script_instance_id: i.script_instance_id().map(DecimalCounter::as_str),
        editor_instance_id: i.editor_instance_id().map(DecimalCounter::as_str),
        buffer_instance_id: i.buffer_instance_id().map(DecimalCounter::as_str),
        disk_file_id: i.disk_file_id().map(file_out),
    }
}
#[derive(Serialize)]
struct InvalidFactOut<'a> {
    value: &'static str,
    collection: StampOut<'a>,
    reason: ReasonOut,
}
#[derive(Serialize)]
struct FactOut<'a> {
    value: Option<&'static str>,
    collection: Option<StampOut<'a>>,
    reason: Option<ReasonOut>,
    invalidated_evidence: Option<InvalidFactOut<'a>>,
}
fn fact_out<T: Copy>(fact: &DocumentFact<T>, render: fn(T) -> &'static str) -> FactOut<'_> {
    FactOut {
        value: fact.value().copied().map(render),
        collection: fact.collection().map(stamp_out),
        reason: fact.reason().map(fact_reason_out),
        invalidated_evidence: fact.invalidated_evidence().map(|old| InvalidFactOut {
            value: render(*old.value()),
            collection: stamp_out(old.collection()),
            reason: fact_reason_out(old.reason()),
        }),
    }
}
#[derive(Serialize)]
struct DocumentOut<'a> {
    identity: Option<IdentityOut<'a>>,
    validity: FactOut<'a>,
    open_state: FactOut<'a>,
}
fn document_out(d: &DocumentState) -> DocumentOut<'_> {
    DocumentOut {
        identity: d.identity().map(identity_out),
        validity: fact_out(d.validity(), Validity),
        open_state: fact_out(d.open_state(), OpenState),
    }
}
#[derive(Serialize)]
struct InvalidDirtyOut<'a> {
    state: &'static str,
    collection: StampOut<'a>,
    witness: WitnessOut<'a>,
    reason: ReasonOut,
}
#[derive(Serialize)]
struct DirtyOut<'a> {
    availability: &'static str,
    state: &'static str,
    collection: Option<StampOut<'a>>,
    witness: Option<WitnessOut<'a>>,
    reason: Option<ReasonOut>,
    invalidated_evidence: Option<InvalidDirtyOut<'a>>,
}
fn dirty_out(d: &DirtyObservation) -> DirtyOut<'_> {
    DirtyOut {
        availability: Availability(d.availability()),
        state: d.state().map_or_else(
            || {
                if d.availability() == Availability::NotApplicable {
                    "not_applicable"
                } else {
                    "unknown"
                }
            },
            DirtyState,
        ),
        collection: d.collection().map(stamp_out),
        witness: d.witness().map(witness_out),
        reason: d.reason().map(dirty_reason_out),
        invalidated_evidence: d.invalidated_evidence().map(|old| InvalidDirtyOut {
            state: DirtyState(old.state()),
            collection: stamp_out(old.collection()),
            witness: witness_out(old.witness()),
            reason: dirty_reason_out(old.reason()),
        }),
    }
}
#[derive(Serialize)]
struct DiagnosticOut {
    code: &'static str,
    stage: &'static str,
    surface: Option<&'static str>,
    message: &'static str,
    action: &'static str,
}
fn diagnostic_out(d: &Diagnostic) -> DiagnosticOut {
    DiagnosticOut {
        code: DiagnosticCode(d.code()),
        stage: Stage(d.stage()),
        surface: d.surface().map(Surface),
        message: d.message(),
        action: d.action(),
    }
}
struct DiagnosticsOut<'a>(&'a [Diagnostic]);
impl Serialize for DiagnosticsOut<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for d in self.0 {
            seq.serialize_element(&diagnostic_out(d))?;
        }
        seq.end()
    }
}
#[derive(Serialize)]
struct TargetOut<'a> {
    project_root: &'a str,
    project_file_id: FileOut<'a>,
    session_id: &'a str,
    godot_version: &'a str,
    engine_hash: &'a str,
    script_path: &'a str,
}
fn target_out(t: &ResolvedTarget) -> TargetOut<'_> {
    TargetOut {
        project_root: t.project_root().as_str(),
        project_file_id: file_out(t.project_file_id()),
        session_id: t.session_id().as_str(),
        godot_version: t.godot_version().version(),
        engine_hash: t.godot_version().hash(),
        script_path: t.script_path().as_str(),
    }
}
#[derive(Serialize)]
struct SelectionOut<'a> {
    candidate_sessions: SessionList<'a>,
    missing_selector: &'static str,
}
struct SessionList<'a>(&'a [SessionId]);
impl Serialize for SessionList<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for session in self.0 {
            seq.serialize_element(session.as_str())?;
        }
        seq.end()
    }
}
fn selection_out(s: &Selection) -> SelectionOut<'_> {
    SelectionOut {
        candidate_sessions: SessionList(s.candidate_sessions()),
        missing_selector: "session_id",
    }
}
#[derive(Serialize)]
struct SourcesOut<'a> {
    #[serde(rename = "D")]
    disk: SourceOut<'a>,
    #[serde(rename = "R")]
    resource: SourceOut<'a>,
    #[serde(rename = "B")]
    buffer: SourceOut<'a>,
}
#[derive(Serialize)]
struct ComparisonsOut {
    disk_resource: &'static str,
    disk_buffer: &'static str,
    resource_buffer: &'static str,
}
#[derive(Serialize)]
struct ChangeOut {
    surface: &'static str,
    code: &'static str,
}
fn change_out(change: DetectedChange) -> ChangeOut {
    let (surface, code) = match change {
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
    ChangeOut { surface, code }
}
struct ChangesOut<'a>(&'a [DetectedChange]);
impl Serialize for ChangesOut<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for change in self.0 {
            seq.serialize_element(&change_out(*change))?;
        }
        seq.end()
    }
}
#[derive(Serialize)]
struct ConsistencyOut<'a> {
    atomic: bool,
    stability: &'static str,
    checks: &'static str,
    detected_changes: ChangesOut<'a>,
    recheck_reason: Option<&'static str>,
}
#[derive(Serialize)]
struct SnapshotOut<'a> {
    target: TargetOut<'a>,
    document: DocumentOut<'a>,
    sources: SourcesOut<'a>,
    dirty: DirtyOut<'a>,
    comparisons: ComparisonsOut,
    agreement: &'static str,
    consistency: ConsistencyOut<'a>,
    diagnostics: DiagnosticsOut<'a>,
}
fn snapshot_out(s: &ObservationSnapshot) -> SnapshotOut<'_> {
    let c = s.comparisons();
    let consistency = s.consistency();
    SnapshotOut {
        target: target_out(s.target()),
        document: document_out(s.document()),
        sources: SourcesOut {
            disk: source_out(s.sources().disk()),
            resource: source_out(s.sources().resource()),
            buffer: source_out(s.sources().buffer()),
        },
        dirty: dirty_out(s.dirty()),
        comparisons: ComparisonsOut {
            disk_resource: Comparison(c.disk_resource()),
            disk_buffer: Comparison(c.disk_buffer()),
            resource_buffer: Comparison(c.resource_buffer()),
        },
        agreement: Agreement(s.agreement()),
        consistency: ConsistencyOut {
            atomic: consistency.atomic(),
            stability: Stability(consistency.stability()),
            checks: Checks(consistency.checks()),
            detected_changes: ChangesOut(consistency.detected_changes()),
            recheck_reason: consistency.recheck_reason().map(RecheckReason),
        },
        diagnostics: DiagnosticsOut(s.diagnostics()),
    }
}
#[derive(Serialize)]
struct IntervalOut {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
#[derive(Serialize)]
struct OutcomeOut<'a> {
    schema_version: u32,
    request_id: &'a str,
    outcome: &'static str,
    interval: IntervalOut,
    resolved_target: Option<TargetOut<'a>>,
    snapshot: Option<SnapshotOut<'a>>,
    diagnostics: DiagnosticsOut<'a>,
    selection: Option<SelectionOut<'a>>,
}
/// Serialize the common reducer's result without copying source into generic JSON values.
/// The CLI, not this encoder, supplies the single trailing newline.
///
/// # Errors
/// Returns a redacted boundary failure if serialization or the result-size bound fails.
pub fn encode_outcome(outcome: &ObservationOutcome) -> Result<Vec<u8>, RoutingFailure> {
    let interval = outcome.interval();
    let dto = OutcomeOut {
        schema_version: outcome.schema_version(),
        request_id: outcome.request_id().as_str(),
        outcome: OutcomeKind(outcome.outcome()),
        interval: IntervalOut {
            started_unix_ms: interval.started_unix_ms(),
            finished_unix_ms: interval.finished_unix_ms(),
            elapsed_us: interval.elapsed_us(),
        },
        resolved_target: outcome.resolved_target().map(target_out),
        snapshot: outcome.snapshot().map(snapshot_out),
        diagnostics: DiagnosticsOut(outcome.diagnostics()),
        selection: outcome.selection().map(selection_out),
    };
    let bytes = serde_json::to_vec(&dto).map_err(|_| bad(Stage::Finalize))?;
    if bytes.len() > RESULT_LIMIT {
        return Err(bad(Stage::Finalize));
    }
    Ok(bytes)
}

#[derive(Serialize)]
struct SampleOut<'a> {
    collection: StampOut<'a>,
    document: DocumentOut<'a>,
    #[serde(rename = "R")]
    resource: SourceOut<'a>,
    #[serde(rename = "B")]
    buffer: SourceOut<'a>,
    dirty: DirtyOut<'a>,
    diagnostics: DiagnosticsOut<'a>,
}
fn sample_out(s: &EditorSample) -> SampleOut<'_> {
    SampleOut {
        collection: stamp_out(&s.collection),
        document: document_out(&s.document),
        resource: source_out(&s.resource),
        buffer: source_out(&s.buffer),
        dirty: dirty_out(&s.dirty),
        diagnostics: DiagnosticsOut(&s.diagnostics),
    }
}
#[derive(Serialize)]
struct RecheckOut<'a> {
    checks: &'static str,
    detected_changes: ChangesOut<'a>,
    reason: Option<&'static str>,
}
fn recheck_out(r: &Recheck) -> RecheckOut<'_> {
    match r {
        Recheck::Performed { detected_changes } => RecheckOut {
            checks: "performed",
            detected_changes: ChangesOut(detected_changes),
            reason: None,
        },
        Recheck::Unavailable { reason } => RecheckOut {
            checks: "unavailable",
            detected_changes: ChangesOut(&[]),
            reason: Some(RecheckReason(*reason)),
        },
        Recheck::Partial {
            reason,
            detected_changes,
        } => RecheckOut {
            checks: "unavailable",
            detected_changes: ChangesOut(detected_changes),
            reason: Some(RecheckReason(*reason)),
        },
    }
}
#[derive(Serialize)]
struct FailureOut<'a> {
    outcome: &'static str,
    diagnostic: DiagnosticOut,
    selection: Option<SelectionOut<'a>>,
}
#[derive(Serialize)]
struct EventOut<'a, T> {
    v: u32,
    request_id: &'a str,
    kind: &'static str,
    #[serde(flatten)]
    payload: T,
}
#[derive(Serialize)]
struct SelectedPayload<'a> {
    target: TargetOut<'a>,
}
#[derive(Serialize)]
struct SamplePayload<'a> {
    sample: SampleOut<'a>,
}
#[derive(Serialize)]
struct DiskPayload<'a> {
    disk: SourceOut<'a>,
    file_identity: Option<FileOut<'a>>,
    file_collection: Option<StampOut<'a>>,
}
#[derive(Serialize)]
struct RecheckedPayload<'a> {
    recheck: RecheckOut<'a>,
}
#[derive(Serialize)]
struct DiskCheckedPayload<'a> {
    disk_checked: ChangesOut<'a>,
}
#[derive(Serialize)]
struct FailedPayload<'a> {
    failure: FailureOut<'a>,
}
#[derive(Serialize)]
struct DoneOut<'a> {
    v: u32,
    request_id: &'a str,
    kind: &'static str,
}
fn encode_ipc<T: Serialize>(dto: T) -> Result<Vec<u8>, RoutingFailure> {
    let bytes = serde_json::to_vec(&dto).map_err(|_| bad(Stage::Finalize))?;
    if bytes.is_empty() || bytes.len() > RESULT_LIMIT {
        return Err(bad(Stage::Finalize));
    }
    Ok(bytes)
}
fn envelope<'a, T>(request_id: &'a RequestId, kind: &'static str, payload: T) -> EventOut<'a, T> {
    EventOut {
        v: 4,
        request_id: request_id.as_str(),
        kind,
        payload,
    }
}
/// Encode one bounded, correlated private worker event; this is not a public protocol.
///
/// # Errors
/// Refuses invalid terminal kinds, changes attributed to the wrong recheck stage,
/// serialization failure, or an oversized event. Receivers must still validate it.
pub fn encode_event(event: &Event, request_id: &RequestId) -> Result<Vec<u8>, RoutingFailure> {
    // Each event has its own payload type; the shared envelope is generic.
    match event {
        Event::Selected(target) => encode_ipc(envelope(
            request_id,
            "selected",
            SelectedPayload {
                target: target_out(target),
            },
        )),
        Event::Sample(sample) => encode_ipc(envelope(
            request_id,
            "sample",
            SamplePayload {
                sample: sample_out(sample),
            },
        )),
        Event::Disk(source) => encode_ipc(envelope(
            request_id,
            "disk",
            DiskPayload {
                disk: source_out(source),
                file_identity: None,
                file_collection: None,
            },
        )),
        Event::DiskMetadata {
            reason,
            file,
            collection,
        } => {
            let source = SourceObservation::unavailable(Authority::D, *reason)
                .map_err(|_| bad(Stage::ReadDisk))?;
            encode_ipc(envelope(
                request_id,
                "disk",
                DiskPayload {
                    disk: source_out(&source),
                    file_identity: file.as_ref().map(file_out),
                    file_collection: Some(stamp_out(collection)),
                },
            ))
        }
        Event::Rechecked(recheck) => {
            let changes = match recheck {
                Recheck::Performed { detected_changes }
                | Recheck::Partial {
                    detected_changes, ..
                } => detected_changes.as_slice(),
                Recheck::Unavailable { .. } => &[],
            };
            editor_changes(changes, Stage::Recheck)?;
            encode_ipc(envelope(
                request_id,
                "rechecked",
                RecheckedPayload {
                    recheck: recheck_out(recheck),
                },
            ))
        }
        Event::DiskChecked(changes) => {
            if changes.len() > COLLECTION_LIMIT
                || changes.iter().any(|change| {
                    !matches!(
                        change,
                        DetectedChange::Source(Authority::D) | DetectedChange::DiskIdentityReplaced
                    )
                })
            {
                return Err(bad(Stage::Recheck));
            }
            encode_ipc(envelope(
                request_id,
                "disk_checked",
                DiskCheckedPayload {
                    disk_checked: ChangesOut(changes),
                },
            ))
        }
        Event::Failed(failure) => {
            if matches!(
                failure.outcome,
                OutcomeKind::CompleteObservation
                    | OutcomeKind::LimitedObservation
                    | OutcomeKind::NotOpen
            ) || (failure.outcome == OutcomeKind::AmbiguousTarget) != failure.selection.is_some()
            {
                return Err(bad(Stage::Finalize));
            }
            encode_ipc(envelope(
                request_id,
                "failed",
                FailedPayload {
                    failure: FailureOut {
                        outcome: OutcomeKind(failure.outcome),
                        diagnostic: diagnostic_out(&failure.diagnostic),
                        selection: failure.selection.as_ref().map(selection_out),
                    },
                },
            ))
        }
        Event::Done => encode_ipc(DoneOut {
            v: 4,
            request_id: request_id.as_str(),
            kind: "done",
        }),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetIn {
    project_root: String,
    project_file_id: FileIn,
    session_id: String,
    godot_version: String,
    engine_hash: String,
    script_path: String,
}
impl TargetIn {
    fn domain(self, request: &ObservationRequest) -> Result<ResolvedTarget, RoutingFailure> {
        let stage = Stage::ResolveTarget;
        let root = ProjectRoot::new(self.project_root).map_err(|_| bad(stage))?;
        let session = SessionId::new(self.session_id).map_err(|_| bad(stage))?;
        if self.script_path != request.script_path().as_str()
            || request
                .session_id()
                .is_some_and(|wanted| wanted != &session)
            || self.godot_version != super::GODOT_VERSION
            || self.engine_hash != super::ENGINE_HASH
        {
            return Err(bad(stage));
        }
        // Canonicalization is performed by the authenticated worker, not by the supervisor.
        // Bind a normalized request to the worker's already-verified canonical root.
        let normalized = ObservationRequest::new(
            request.request_id().clone(),
            root.clone(),
            request.session_id().cloned(),
            request.script_path().clone(),
        );
        ResolvedTarget::for_request(
            &normalized,
            root,
            self.project_file_id.domain(stage)?,
            session,
            EngineVersion::new(self.godot_version, self.engine_hash).map_err(|_| bad(stage))?,
        )
        .map_err(|_| bad(stage))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SampleEventIn {
    collection: StampIn,
    document: DocumentIn,
    #[serde(rename = "R")]
    resource: SourceIn,
    #[serde(rename = "B")]
    buffer: SourceIn,
    dirty: DirtyIn,
    diagnostics: Bounded<DiagnosticIn>,
}
impl SampleEventIn {
    fn domain(self, target: &ResolvedTarget, receipt: u64) -> Result<EditorSample, RoutingFailure> {
        let stage = Stage::ReadEditor;
        let collection = self
            .collection
            .domain(target.session_id(), receipt, stage, true)?;
        let document =
            self.document
                .domain(target.script_path(), target.session_id(), receipt, stage)?;
        let resource = self.resource.domain(
            Authority::R,
            target,
            receipt,
            stage,
            document.identity(),
            Some(&collection),
        )?;
        let buffer = self.buffer.domain(
            Authority::B,
            target,
            receipt,
            stage,
            document.identity(),
            Some(&collection),
        )?;
        let sample = EditorSample {
            collection,
            document,
            resource,
            buffer,
            dirty: self
                .dirty
                .domain(target.script_path(), target.session_id(), receipt, stage)?,
            diagnostics: self
                .diagnostics
                .0
                .into_iter()
                .map(|d| d.domain(stage))
                .collect::<Result<Vec<_>, _>>()?,
        };
        validate_sample(&sample, target, receipt, stage)?;
        Ok(sample)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecheckEventIn {
    checks: String,
    detected_changes: Bounded<ChangeIn>,
    reason: Option<RecheckReason>,
}
impl RecheckEventIn {
    fn domain(self, stage: Stage) -> Result<Recheck, RoutingFailure> {
        let changes = self
            .detected_changes
            .0
            .into_iter()
            .map(|c| c.domain(stage))
            .collect::<Result<Vec<_>, _>>()?;
        editor_changes(&changes, stage)?;
        match self.checks.as_str() {
            "performed" if self.reason.is_none() => Ok(Recheck::performed(changes)),
            "unavailable" => {
                let reason = self.reason.ok_or_else(|| bad(stage))?;
                if changes.is_empty() {
                    Ok(Recheck::unavailable(reason))
                } else {
                    Ok(Recheck::partial(reason, changes))
                }
            }
            _ => Err(bad(stage)),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionIn {
    candidate_sessions: Bounded<String>,
    missing_selector: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FailureIn {
    outcome: OutcomeKind,
    diagnostic: DiagnosticIn,
    selection: Option<SelectionIn>,
}
impl FailureIn {
    fn domain(self, request: &ObservationRequest) -> Result<RoutingFailure, RoutingFailure> {
        let stage = Stage::Finalize;
        if matches!(
            self.outcome,
            OutcomeKind::CompleteObservation
                | OutcomeKind::LimitedObservation
                | OutcomeKind::NotOpen
        ) {
            return Err(bad(stage));
        }
        let diagnostic = self.diagnostic.domain(stage)?;
        let selection = self
            .selection
            .map(|s| {
                if s.missing_selector != "session_id" || request.session_id().is_some() {
                    return Err(bad(stage));
                }
                Selection::ambiguous(
                    s.candidate_sessions
                        .0
                        .into_iter()
                        .map(|id| SessionId::new(id).map_err(|_| bad(stage)))
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|_| bad(stage))
            })
            .transpose()?;
        if (self.outcome == OutcomeKind::AmbiguousTarget) != selection.is_some() {
            return Err(bad(stage));
        }
        Ok(RoutingFailure {
            outcome: self.outcome,
            diagnostic,
            selection,
        })
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum EventIn {
    Selected {
        v: u32,
        request_id: String,
        target: TargetIn,
    },
    Sample {
        v: u32,
        request_id: String,
        sample: Box<SampleEventIn>,
    },
    Disk {
        v: u32,
        request_id: String,
        disk: Box<SourceIn>,
        file_identity: Option<FileIn>,
        file_collection: Option<StampIn>,
    },
    Rechecked {
        v: u32,
        request_id: String,
        recheck: RecheckEventIn,
    },
    DiskChecked {
        v: u32,
        request_id: String,
        disk_checked: Bounded<ChangeIn>,
    },
    Failed {
        v: u32,
        request_id: String,
        failure: FailureIn,
    },
    Done {
        v: u32,
        request_id: String,
    },
}
/// Validate a private worker event before retention and stamp its supervisor receipt.
/// The supervisor separately enforces stage ordering and its collection deadline.
///
/// # Errors
/// Refuses malformed/duplicate fields, wrong identity or clock, invalid evidence,
/// incompatible versions, excess nesting/collections, and oversized frames.
pub fn decode_event(
    bytes: &[u8],
    request: &ObservationRequest,
    target: Option<&ResolvedTarget>,
    received_elapsed_us: u64,
) -> Result<Event, RoutingFailure> {
    let stage = Stage::Finalize;
    let event: EventIn = decode(bytes, stage)?;
    let (version, id) = match &event {
        EventIn::Selected { v, request_id, .. }
        | EventIn::Sample { v, request_id, .. }
        | EventIn::Disk { v, request_id, .. }
        | EventIn::Rechecked { v, request_id, .. }
        | EventIn::DiskChecked { v, request_id, .. }
        | EventIn::Failed { v, request_id, .. }
        | EventIn::Done { v, request_id } => (*v, request_id),
    };
    if version != 4 || id != request.request_id().as_str() {
        return Err(bad(stage));
    }
    if let Some(target) = target {
        if target.script_path() != request.script_path()
            || request
                .session_id()
                .is_some_and(|id| target.session_id() != id)
        {
            return Err(bad(stage));
        }
    }
    match event {
        EventIn::Selected {
            target: selected, ..
        } if target.is_none() => Ok(Event::Selected(selected.domain(request)?)),
        EventIn::Sample { sample, .. } => Ok(Event::Sample(Box::new(
            sample.domain(target.ok_or_else(|| bad(stage))?, received_elapsed_us)?,
        ))),
        EventIn::Disk {
            disk,
            file_identity,
            file_collection,
            ..
        } => {
            let t = target.ok_or_else(|| bad(stage))?;
            let disk = disk.domain(
                Authority::D,
                t,
                received_elapsed_us,
                Stage::ReadDisk,
                None,
                None,
            )?;
            if let Some(stamp) = disk.collection() {
                check_stamp(
                    stamp,
                    t.session_id(),
                    received_elapsed_us,
                    false,
                    Stage::ReadDisk,
                )?;
            }
            if let Some(w) = disk.witness() {
                if w.resource_path() != t.script_path()
                    || w.disk_file_id().is_none()
                    || w.editor_instance_id().is_some()
                    || w.script_instance_id().is_some()
                    || w.buffer_instance_id().is_some()
                {
                    return Err(bad(Stage::ReadDisk));
                }
            }
            match (file_identity, file_collection) {
                (file, Some(stamp)) => {
                    if t.script_path().kind() != Some(ScriptKind::ExternalGdscript) {
                        return Err(bad(Stage::ReadDisk));
                    }
                    let file = file.map(|file| file.domain(Stage::ReadDisk)).transpose()?;
                    let stamp = stamp.domain(
                        t.session_id(),
                        received_elapsed_us,
                        Stage::ReadDisk,
                        false,
                    )?;
                    let reason = disk.reason().ok_or_else(|| bad(Stage::ReadDisk))?;
                    if !matches!(
                        (&file, reason),
                        (
                            Some(_),
                            SourceReason::TooLarge
                                | SourceReason::InvalidUtf8
                                | SourceReason::DiskUnreadable
                        ) | (None, SourceReason::DiskMissing)
                    ) {
                        return Err(bad(Stage::ReadDisk));
                    }
                    Ok(Event::DiskMetadata {
                        reason,
                        file,
                        collection: stamp,
                    })
                }
                (None, None) => Ok(Event::Disk(disk)),
                _ => Err(bad(Stage::ReadDisk)),
            }
        }
        EventIn::Rechecked { recheck, .. } if target.is_some() => {
            Ok(Event::Rechecked(recheck.domain(Stage::Recheck)?))
        }
        EventIn::DiskChecked { disk_checked, .. } if target.is_some() => Ok(Event::DiskChecked(
            disk_checked
                .0
                .into_iter()
                .map(|c| match (c.surface, c.code) {
                    (Surface::D, DiagnosticCode::SourceChanged) => {
                        Ok(DetectedChange::Source(Authority::D))
                    }
                    (Surface::D, DiagnosticCode::IdentityChanged) => {
                        Ok(DetectedChange::DiskIdentityReplaced)
                    }
                    _ => Err(bad(Stage::Recheck)),
                })
                .collect::<Result<Vec<_>, _>>()?,
        )),
        EventIn::Failed { failure, .. } => Ok(Event::Failed(failure.domain(request)?)),
        EventIn::Done { .. } => Ok(Event::Done),
        _ => Err(bad(stage)),
    }
}
