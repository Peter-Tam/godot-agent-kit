//! Checked caller intent and provenance from a prior observation.

use super::{EditError, OPERATION, SCHEMA_VERSION};
use crate::observation::{
    Agreement, Checks, CollectionStamp, DecimalCounter, DirtyObservation, DirtyState,
    DocumentIdentity, DocumentState, ObservationInterval, ObservationOutcome, OutcomeKind,
    ProjectRoot, RequestId, ResolvedTarget, ResourcePath, ScriptKind, SessionId, Stability,
    Witness, SOURCE_LIMIT_BYTES,
};
use ring::digest::{digest, SHA256};
use std::fmt;

/// Exact, bounded LF UTF-8. Source bytes are deliberately absent from `Debug` output.
#[derive(Clone, PartialEq, Eq)]
pub struct ReplacementSource(String);
impl fmt::Debug for ReplacementSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReplacementSource([redacted])")
    }
}
impl ReplacementSource {
    /// # Errors
    /// Rejects more than 512 KiB, NUL, CR, or any UTF-8 BOM; never normalizes input.
    pub fn new(value: String) -> Result<Self, EditError> {
        if value.len() > SOURCE_LIMIT_BYTES
            || value.chars().any(|c| matches!(c, '\0' | '\r' | '\u{feff}'))
        {
            return Err(EditError::InvalidSource);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Digest is a summary, never authority to write or a substitute for exact byte comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceDigest {
    pub sha256: [u8; 32],
    pub utf8_bytes: usize,
}
impl SourceDigest {
    pub fn of(text: &str) -> Self {
        let mut sha256 = [0; 32];
        sha256.copy_from_slice(digest(&SHA256, text.as_bytes()).as_ref());
        Self {
            sha256,
            utf8_bytes: text.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasisError {
    Missing,
    Dirty,
    Divergent,
    KnownStaleResource,
    KnownStaleBuffer,
    MissingVersion,
    UnsupportedTarget,
    UnsupportedRepresentation,
}

/// Provenance from an eligible *prior* observation. Its source and version are not authorization.
/// Read-only accessors expose the checked target, revision and original observation provenance
/// carried by [`super::EditOutcome::expected`] to external consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRevisionBasis {
    prior_request_id: RequestId,
    prior_interval: ObservationInterval,
    target: ResolvedTarget,
    document: DocumentIdentity,
    witnesses: [Witness; 3],
    collections: [CollectionStamp; 3],
    prior_document: DocumentState,
    prior_dirty: DirtyObservation,
    source: SourceDigest,
    current_version: DecimalCounter,
}
impl ExpectedRevisionBasis {
    /// # Errors
    /// Only a complete, stable, clean, agreeing open standalone script with all three actual
    /// sources and an attributable buffer current version supplies a basis. A saved version is
    /// deliberately not inferred from observation v1.
    pub fn from_observation(prior: &ObservationOutcome) -> Result<Self, BasisError> {
        let snapshot = prior.snapshot().ok_or(BasisError::Missing)?;
        if prior.outcome() != OutcomeKind::CompleteObservation
            || snapshot.consistency().checks() != Checks::Performed
            || snapshot.consistency().stability() == Stability::Changed
        {
            return Err(BasisError::Missing);
        }
        let target = snapshot.target();
        let document = snapshot.document().identity().ok_or(BasisError::Missing)?;
        if document.kind() != ScriptKind::ExternalGdscript
            || document.disk_file_id().is_none()
            || document.script_instance_id().is_none()
            || document.editor_instance_id().is_none()
            || document.buffer_instance_id().is_none()
        {
            return Err(BasisError::UnsupportedTarget);
        }
        if snapshot.dirty().state() != Some(DirtyState::Clean) {
            return Err(BasisError::Dirty);
        }
        let [d, r, b] = [
            snapshot.sources().disk(),
            snapshot.sources().resource(),
            snapshot.sources().buffer(),
        ];
        if r.staleness().is_some_and(|v| v.evidence().is_some()) {
            return Err(BasisError::KnownStaleResource);
        }
        if b.staleness().is_some_and(|v| v.evidence().is_some()) {
            return Err(BasisError::KnownStaleBuffer);
        }
        if snapshot.agreement() != Agreement::Agree {
            return Err(BasisError::Divergent);
        }
        let source = d.text().ok_or(BasisError::Missing)?;
        if r.text() != Some(source) || b.text() != Some(source) {
            return Err(BasisError::Divergent);
        }
        if source
            .chars()
            .any(|c| matches!(c, '\0' | '\r' | '\u{feff}'))
        {
            return Err(BasisError::UnsupportedRepresentation);
        }
        let disk = d.witness().cloned().ok_or(BasisError::Missing)?;
        let resource = r.witness().cloned().ok_or(BasisError::Missing)?;
        let buffer = b.witness().cloned().ok_or(BasisError::Missing)?;
        let current_version = buffer
            .source_version()
            .cloned()
            .ok_or(BasisError::MissingVersion)?;
        Ok(Self {
            prior_request_id: prior.request_id().clone(),
            prior_interval: prior.interval().clone(),
            target: target.clone(),
            document: document.clone(),
            witnesses: [disk, resource, buffer],
            collections: [
                d.collection().cloned().ok_or(BasisError::Missing)?,
                r.collection().cloned().ok_or(BasisError::Missing)?,
                b.collection().cloned().ok_or(BasisError::Missing)?,
            ],
            prior_document: snapshot.document().clone(),
            prior_dirty: snapshot.dirty().clone(),
            source: SourceDigest::of(source),
            current_version,
        })
    }
    pub fn prior_request_id(&self) -> &RequestId {
        &self.prior_request_id
    }
    pub fn prior_interval(&self) -> &ObservationInterval {
        &self.prior_interval
    }
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    pub fn document(&self) -> &DocumentIdentity {
        &self.document
    }
    pub fn witnesses(&self) -> &[Witness; 3] {
        &self.witnesses
    }
    pub fn collections(&self) -> &[CollectionStamp; 3] {
        &self.collections
    }
    pub fn prior_document(&self) -> &DocumentState {
        &self.prior_document
    }
    pub fn prior_dirty(&self) -> &DirtyObservation {
        &self.prior_dirty
    }
    pub fn source(&self) -> &SourceDigest {
        &self.source
    }
    pub fn current_version(&self) -> &DecimalCounter {
        &self.current_version
    }
}

/// Caller intent; the selected target must still be freshly authenticated.
#[derive(Debug)]
pub struct EditRequest {
    pub(super) request_id: RequestId,
    pub(super) project_root: ProjectRoot,
    pub(super) session_id: Option<SessionId>,
    pub(super) script_path: ResourcePath,
    pub(super) expected: ExpectedRevisionBasis,
    pub(super) replacement_source: ReplacementSource,
}
impl EditRequest {
    /// # Errors
    /// Rejects a non-standalone selector, mismatched prior selectors, or a reused prior ID.
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
        expected: ExpectedRevisionBasis,
        replacement_source: ReplacementSource,
    ) -> Result<Self, EditError> {
        if script_path.kind() != Some(ScriptKind::ExternalGdscript)
            || expected.target.project_root() != &project_root
            || expected.target.script_path() != &script_path
            || session_id
                .as_ref()
                .is_some_and(|s| s != expected.target.session_id())
            || request_id == expected.prior_request_id
        {
            return Err(EditError::WrongTarget);
        }
        Ok(Self {
            request_id,
            project_root,
            session_id,
            script_path,
            expected,
            replacement_source,
        })
    }
    pub fn schema_version(&self) -> u32 {
        SCHEMA_VERSION
    }
    pub fn operation(&self) -> &'static str {
        OPERATION
    }
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn project_root(&self) -> &ProjectRoot {
        &self.project_root
    }
    pub fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    pub fn script_path(&self) -> &ResourcePath {
        &self.script_path
    }
    pub fn expected(&self) -> &ExpectedRevisionBasis {
        &self.expected
    }
    pub fn replacement_source(&self) -> &ReplacementSource {
        &self.replacement_source
    }
}
