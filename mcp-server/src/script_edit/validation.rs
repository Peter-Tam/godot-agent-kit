//! Source-attributed validation records, bounds and diagnostic consistency.

use super::{EditError, Reason, SourceDigest};
use crate::observation::{
    CollectionStamp, DocumentIdentity, FileIdentity, RequestId, ResourcePath, ScriptKind,
    SessionId, SOURCE_LIMIT_BYTES,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationPurpose {
    Preflight,
    PostChange,
    Unchanged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Unavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticOrigin {
    Root,
    Dependency,
}
#[derive(Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub origin: DiagnosticOrigin,
    pub path: Option<ResourcePath>,
    pub path_reason: Option<Reason>,
    pub source: Option<SourceDigest>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
}
impl fmt::Debug for ValidationDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationDiagnostic")
            .field("origin", &self.origin)
            .field("path", &self.path)
            .field("source", &self.source)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("message", &"[redacted]")
            .finish()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyWitness {
    pub path: ResourcePath,
    pub identity: FileIdentity,
    pub source: SourceDigest,
}
/// Both stock LSP fences for one owner-attributed captured source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSourceFence {
    pub path: ResourcePath,
    pub source: SourceDigest,
    pub diagnostics_completed: bool,
    pub symbols_completed: bool,
}
/// Validation is attributed to exact source, invocation, dependency and context witnesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub reason: Option<Reason>,
    pub purpose: ValidationPurpose,
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub source_path: ResourcePath,
    pub input: SourceDigest,
    pub collection: CollectionStamp,
    pub dependencies: Vec<DependencyWitness>,
    pub context: [u8; 32],
    pub context_current: bool,
    pub dependencies_current: bool,
    pub sources: Vec<ValidationSourceFence>,
    pub cleanup_confirmed: bool,
    pub diagnostics: Vec<ValidationDiagnostic>,
}
/// Fresh read-only project mapping and dependency witnesses, obtained after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRecheck {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub context: [u8; 32],
    pub dependencies: Vec<DependencyWitness>,
    pub current: bool,
}

/// Retain bounded, attributable diagnostics without upgrading unavailable evidence.
pub(super) fn bound_evidence(result: &mut ValidationResult) {
    let mut dependency_bytes = 0usize;
    let mut over_limit = result.dependencies.len() > 32
        || result.sources.len() > 33
        || result.diagnostics.len() > 64;
    result.dependencies.truncate(32);
    result.dependencies.retain(|dependency| {
        let fits = dependency.source.utf8_bytes <= SOURCE_LIMIT_BYTES
            && dependency.source.utf8_bytes <= 4 * 1024 * 1024 - dependency_bytes;
        if fits {
            dependency_bytes += dependency.source.utf8_bytes;
        }
        over_limit |= !fits;
        fits
    });
    result.sources.truncate(33);
    result.diagnostics.truncate(64);
    result.diagnostics.retain(|diagnostic| {
        let fits = diagnostic.message.len() <= 2048;
        over_limit |= !fits;
        fits
    });
    if over_limit {
        result.status = ValidationStatus::Unavailable;
        result.reason = Some(
            result
                .reason
                .filter(|r| r.suppresses_source())
                .unwrap_or(Reason::EvidenceLimit),
        );
        result.cleanup_confirmed = false;
        result.sources.retain(|fence| {
            fence.path == result.source_path
                || result
                    .dependencies
                    .iter()
                    .any(|d| d.path == fence.path && d.source == fence.source)
        });
        // A discarded dependency cannot authorize retaining its source-attributed error.
        result.diagnostics.retain(|d| {
            d.origin == DiagnosticOrigin::Root
                || d.source.is_none()
                || result
                    .dependencies
                    .iter()
                    .any(|w| Some(&w.path) == d.path.as_ref() && Some(w.source) == d.source)
        });
    }
}
/// Complete owner fences must cover the root and every admitted dependency exactly once.
pub(super) fn complete_fences(result: &ValidationResult) -> bool {
    result.cleanup_confirmed
        && result.sources.len() == result.dependencies.len() + 1
        && result
            .sources
            .iter()
            .all(|fence| fence.diagnostics_completed && fence.symbols_completed)
}

pub(super) fn check_attribution(
    result: &ValidationResult,
    source_path: &ResourcePath,
    intended: &SourceDigest,
) -> Result<(), EditError> {
    if result.source_path != *source_path
        || result.input != *intended
        || result.dependencies.iter().enumerate().any(|(i, d)| {
            d.path == *source_path
                || d.path.kind() != Some(ScriptKind::ExternalGdscript)
                || result.dependencies[..i]
                    .iter()
                    .any(|old| old.path == d.path)
        })
        || result.sources.iter().enumerate().any(|(i, fence)| {
            result.sources[..i].iter().any(|old| old.path == fence.path)
                || (fence.path == *source_path && fence.source != *intended)
                || (fence.path != *source_path
                    && !result.dependencies.iter().any(|dependency| {
                        dependency.path == fence.path && dependency.source == fence.source
                    }))
        })
        || result.diagnostics.iter().any(|d| {
            d.message.len() > 2048
                || d.path.as_ref().is_some_and(|p| p.as_str().len() > 2048)
                || d.path.is_none() != d.path_reason.is_some()
                || d.origin == DiagnosticOrigin::Root
                    && (d.path.as_ref() != Some(source_path) || d.source != Some(*intended))
                    && result.status != ValidationStatus::Unavailable
                || d.origin == DiagnosticOrigin::Dependency
                    && d.source.is_some()
                    && !result
                        .dependencies
                        .iter()
                        .any(|w| Some(&w.path) == d.path.as_ref() && Some(w.source) == d.source)
        })
        || result.status == ValidationStatus::Valid && !result.diagnostics.is_empty()
    {
        return Err(EditError::InvalidEvidence);
    }
    Ok(())
}
