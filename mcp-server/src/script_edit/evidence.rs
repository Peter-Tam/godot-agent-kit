//! Independent surface/save evidence and separately attributed native effect records.

use super::{Reason, SourceDigest};
use crate::observation::{
    Agreement, Authority, Availability, CollectionStamp, Comparisons, Consistency, DecimalCounter,
    DirtyObservation, DocumentIdentity, DocumentState, FileIdentity, RequestId, SessionId,
    SourceObservation, SourceReason, Witness,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceEvidence {
    pub authority: Authority,
    pub availability: Availability,
    pub source: Option<SourceDigest>,
    pub collection: Option<CollectionStamp>,
    pub identity: Option<Witness>,
    pub staleness: Option<crate::observation::Staleness>,
    pub reason: Option<SourceReason>,
    pub invalidated: Option<InvalidatedSurface>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidatedSurface {
    pub source: SourceDigest,
    pub collection: CollectionStamp,
    pub identity: Witness,
    pub staleness: crate::observation::Staleness,
    pub reason: SourceReason,
}
pub(super) fn surface(source: &SourceObservation, agreed: Option<SourceDigest>) -> SurfaceEvidence {
    SurfaceEvidence {
        authority: source.authority(),
        availability: source.availability(),
        source: source
            .text()
            .map(|text| agreed.unwrap_or_else(|| SourceDigest::of(text))),
        collection: source.collection().cloned(),
        identity: source.witness().cloned(),
        staleness: source.staleness().cloned(),
        reason: source.reason(),
        invalidated: source.invalidated_evidence().map(|old| InvalidatedSurface {
            source: SourceDigest::of(old.text()),
            collection: old.collection().clone(),
            identity: old.witness().clone(),
            staleness: old.staleness().clone(),
            reason: old.reason(),
        }),
    }
}

/// Independent caller-clock disk identity/mtime read, separate from the writer's receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskMetadata {
    pub identity: FileIdentity,
    pub mtime: DecimalCounter,
    pub collection: CollectionStamp,
}
/// Independent native inspection, not the finalizer's returned step flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedStateEvidence {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub current_version: DecimalCounter,
    pub saved_version: DecimalCounter,
    pub resource_edited: bool,
    pub resource_mtime: DecimalCounter,
    pub document_mtime: DecimalCounter,
    pub save_profile: [u8; 32],
    pub original_preserved: bool,
    pub desired_preserved: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditEvidence {
    pub document: DocumentState,
    pub sources: [SurfaceEvidence; 3],
    pub dirty: DirtyObservation,
    pub saved_state: Option<SavedStateEvidence>,
    pub comparisons: Comparisons,
    /// Why a required independent inspection is missing; never an inferred clean state.
    pub saved_state_reason: Option<Reason>,
    pub agreement: Agreement,
    pub consistency: Consistency,
    pub disk_metadata: Option<DiskMetadata>,
}

/// Attributable native acknowledgment. Only a trusted integration may supply it.
/// A discard witness additionally attests that the native context is permanently consumed
/// before any source/history entry; an abort request or transport EOF is not this evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeWitness {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
}
/// Summary of attempt-local native descriptor evidence; never accepted as independent D readback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceReceipt {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub intended: SourceDigest,
    pub project_id: FileIdentity,
    pub file_id: FileIdentity,
    pub collection: CollectionStamp,
    pub write_started: bool,
    pub bytes_written: usize,
    pub truncated: bool,
    pub flushed: bool,
    pub readback_matches: bool,
    pub attached: bool,
    pub descriptor_open: bool,
    pub interference: bool,
    pub mtime: Option<DecimalCounter>,
    pub reason: Option<Reason>,
}
impl PersistenceReceipt {
    pub(super) fn complete(&self) -> bool {
        self.write_started
            && self.bytes_written == self.intended.utf8_bytes
            && self.truncated
            && self.flushed
            && self.readback_matches
            && self.attached
            && self.descriptor_open
            && !self.interference
            && self.mtime.is_some()
            && self.reason.is_none()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalizationStatus {
    Complete,
    Rejected,
    PartialOrUnknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bookkeeping {
    pub resource_mtime: bool,
    pub document_mtime: bool,
    pub resource_edited: bool,
    pub saved_version: bool,
    pub display: bool,
}
impl Bookkeeping {
    pub(super) fn any(self) -> bool {
        self.resource_mtime
            || self.document_mtime
            || self.resource_edited
            || self.saved_version
            || self.display
    }
    pub(super) fn all(self) -> bool {
        self.resource_mtime
            && self.document_mtime
            && self.resource_edited
            && self.saved_version
            && self.display
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizationResult {
    pub status: FinalizationStatus,
    pub reason: Option<Reason>,
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub document: DocumentIdentity,
    pub collection: CollectionStamp,
    pub before_source: Option<SourceDigest>,
    pub after_source: Option<SourceDigest>,
    pub before_current: Option<DecimalCounter>,
    pub after_current: Option<DecimalCounter>,
    pub before_saved: Option<DecimalCounter>,
    pub after_saved: Option<DecimalCounter>,
    pub before_resource_mtime: Option<DecimalCounter>,
    pub after_resource_mtime: Option<DecimalCounter>,
    pub before_document_mtime: Option<DecimalCounter>,
    pub after_document_mtime: Option<DecimalCounter>,
    pub before_resource_edited: Option<bool>,
    pub after_resource_edited: Option<bool>,
    pub steps: Bookkeeping,
}
