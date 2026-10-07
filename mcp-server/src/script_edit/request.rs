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

/// Bounded literal fragments; neither fragment is required to be a complete script.
pub(crate) struct ExactReplacement {
    old: String,
    new: String,
}
impl fmt::Debug for ExactReplacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExactReplacement([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExactMatchError {
    NoMatch,
    AmbiguousMatch,
    EmptyOldString,
    InvalidSource,
}
impl ExactMatchError {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NoMatch => "no_match",
            Self::AmbiguousMatch => "ambiguous_match",
            Self::EmptyOldString => "empty_old_string",
            Self::InvalidSource => "invalid_source",
        }
    }
}

impl ExactReplacement {
    pub(crate) fn new(old: String, new: String) -> Result<Self, EditError> {
        let old = ReplacementSource::new(old)?.0;
        let new = ReplacementSource::new(new)?.0;
        Ok(Self { old, new })
    }

    /// Resolve against the same complete admitted source used for the expected basis.
    pub(crate) fn derive(self, current: &str) -> Result<ReplacementSource, ExactMatchError> {
        if self.old.is_empty() {
            return if current.is_empty() {
                Ok(ReplacementSource(self.new))
            } else {
                Err(ExactMatchError::EmptyOldString)
            };
        }
        let start = current.find(&self.old).ok_or(ExactMatchError::NoMatch)?;
        // Advance one code point, not the matched span: overlapping starts count.
        let next = start
            + self
                .old
                .chars()
                .next()
                .expect("nonempty old text")
                .len_utf8();
        if current[next..].find(&self.old).is_some() {
            return Err(ExactMatchError::AmbiguousMatch);
        }
        let end = start + self.old.len();
        let length = current
            .len()
            .checked_sub(self.old.len())
            .and_then(|length| length.checked_add(self.new.len()))
            .filter(|length| *length <= SOURCE_LIMIT_BYTES)
            .ok_or(ExactMatchError::InvalidSource)?;
        if start == 0 && end == current.len() {
            return Ok(ReplacementSource(self.new));
        }
        let mut result = String::with_capacity(length);
        result.push_str(&current[..start]);
        result.push_str(&self.new);
        result.push_str(&current[end..]);
        ReplacementSource::new(result).map_err(|_| ExactMatchError::InvalidSource)
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

#[cfg(test)]
mod exact_tests {
    use super::*;

    fn derive(old: &str, new: &str, current: &str) -> Result<ReplacementSource, ExactMatchError> {
        ExactReplacement::new(old.into(), new.into())
            .unwrap()
            .derive(current)
    }

    #[test]
    fn counts_literal_starts_including_overlaps() {
        for (old, current, expected) in [
            ("old", "new", ExactMatchError::NoMatch),
            ("a", "aba", ExactMatchError::AmbiguousMatch),
            ("aa", "aaa", ExactMatchError::AmbiguousMatch),
            ("éé", "ééé", ExactMatchError::AmbiguousMatch),
            ("🦀🦀", "🦀🦀🦀", ExactMatchError::AmbiguousMatch),
        ] {
            assert_eq!(derive(old, "new", current), Err(expected));
        }
        assert_eq!(derive("aa", "b", "aa").unwrap().as_str(), "b");
        assert_eq!(derive("éé", "b", "éé").unwrap().as_str(), "b");
    }

    #[test]
    fn preserves_exact_surroundings_and_never_rescans_inserted_text() {
        for (old, new, current, expected) in [
            ("a", "ab", "abc", "abbc"),
            ("b", "bb", "abc", "abbc"),
            ("c", "cc", "abc", "abcc"),
            ("old", "old old", "before old after", "before old old after"),
            ("\tλ\n x", "\t🦀\n  y", "A\n\tλ\n x\nZ", "A\n\t🦀\n  y\nZ"),
            (" ", "\t", "a b", "a\tb"),
        ] {
            assert_eq!(derive(old, new, current).unwrap().as_str(), expected);
        }
        for (old, current) in [
            ("Old", "old"),
            ("a b", "a  b"),
            ("a\tb", "a b"),
            ("a\nb", "a b"),
            ("é", "e\u{301}"),
            ("e\u{301}", "é"),
        ] {
            assert_eq!(derive(old, "x", current), Err(ExactMatchError::NoMatch));
        }
    }

    #[test]
    fn deletion_equal_and_complete_empty_intents_remain_exact() {
        for (old, new, current, expected) in [
            ("b", "", "abc", "ac"),
            ("abc", "", "abc", ""),
            ("abc", "abc", "abc", "abc"),
            ("", "new\n", "", "new\n"),
            ("", "", "", ""),
            (" \t\n", "", " \t\n", ""),
        ] {
            assert_eq!(derive(old, new, current).unwrap().as_str(), expected);
        }
        for current in ["a", " ", "\t", "\n", " \t\n"] {
            assert_eq!(
                derive("", "x", current),
                Err(ExactMatchError::EmptyOldString)
            );
        }
        assert_eq!(derive("a", "a", "aa"), Err(ExactMatchError::AmbiguousMatch));
        assert_eq!(derive("old", "new", "new"), Err(ExactMatchError::NoMatch));
    }

    #[test]
    fn fragment_representation_and_utf8_byte_caps_are_checked() {
        let cap = "é".repeat(SOURCE_LIMIT_BYTES / 2);
        assert!(ExactReplacement::new(cap.clone(), cap.clone()).is_ok());
        assert!(ExactReplacement::new(String::new(), String::new()).is_ok());
        for invalid in [
            format!("{cap}a"),
            "\0".into(),
            "\r".into(),
            "\u{feff}".into(),
        ] {
            assert_eq!(
                ExactReplacement::new(invalid.clone(), "x".into()).unwrap_err(),
                EditError::InvalidSource
            );
            assert_eq!(
                ExactReplacement::new("x".into(), invalid).unwrap_err(),
                EditError::InvalidSource
            );
        }
        // Fragments are text, not independently valid scripts.
        assert!(ExactReplacement::new(")".into(), "(".into()).is_ok());
    }

    #[test]
    fn combined_source_limit_is_checked_before_construction() {
        let current = format!("a{}", "é".repeat((SOURCE_LIMIT_BYTES - 2) / 2));
        assert_eq!(
            derive("a", "ab", &current).unwrap().as_str().len(),
            SOURCE_LIMIT_BYTES
        );
        assert_eq!(
            derive("a", "abc", &current),
            Err(ExactMatchError::InvalidSource)
        );
        let new = "x".repeat(SOURCE_LIMIT_BYTES);
        assert_eq!(derive("a", &new, "a").unwrap().as_str(), new);
        assert_eq!(derive("a", &new, "ab"), Err(ExactMatchError::InvalidSource));
    }

    #[test]
    fn diagnostics_never_include_source_or_match_locations() {
        let intent = ExactReplacement::new("PRIVATE_OLD".into(), "PRIVATE_NEW".into()).unwrap();
        let debug = format!("{intent:?}");
        assert!(!debug.contains("PRIVATE_OLD"));
        assert!(!debug.contains("PRIVATE_NEW"));
        for (error, reason) in [
            (ExactMatchError::NoMatch, "no_match"),
            (ExactMatchError::AmbiguousMatch, "ambiguous_match"),
            (ExactMatchError::EmptyOldString, "empty_old_string"),
            (ExactMatchError::InvalidSource, "invalid_source"),
        ] {
            assert_eq!(error.as_str(), reason);
        }
    }
}
