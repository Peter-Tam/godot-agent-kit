use super::*;
#[derive(Clone)]
pub(crate) struct Step {
    pub state: &'static str,
    pub reason: Option<&'static str>,
    pub collection: Option<CollectionStamp>,
}
impl Default for Step {
    fn default() -> Self {
        Self {
            state: "not_started",
            reason: None,
            collection: None,
        }
    }
}
#[derive(Default)]
pub(crate) struct Progress {
    pub closing: Step,
    pub native_revalidation: Step,
    pub verification: Step,
}
#[derive(Clone)]
pub(crate) struct ResourceState {
    pub state: &'static str,
    pub resource_edited: Option<bool>,
    pub reason: Option<&'static str>,
    pub collection: Option<CollectionStamp>,
}
impl Default for ResourceState {
    fn default() -> Self {
        Self {
            state: "not_collected",
            resource_edited: None,
            reason: None,
            collection: None,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Protection {
    pub status: &'static str,
    pub revalidation: &'static str,
    pub required_count: Option<usize>,
    pub completed_count: Option<usize>,
    pub reason: Option<&'static str>,
}
impl Default for Protection {
    fn default() -> Self {
        Self {
            status: "not_applicable",
            revalidation: "not_applicable",
            required_count: None,
            completed_count: None,
            reason: None,
        }
    }
}
#[derive(Clone)]
pub(crate) struct Selection {
    pub before: &'static str,
    pub after: &'static str,
    pub request_effect: &'static str,
}
pub(crate) struct History {
    pub target_buffer: &'static str,
    pub unrelated: &'static str,
    pub reason: Option<&'static str>,
}
impl Default for History {
    fn default() -> Self {
        Self {
            target_buffer: "not_applicable",
            unrelated: "not_applicable",
            reason: None,
        }
    }
}
pub(crate) struct Expected {
    pub basis: ExpectedRevisionBasis,
    pub use_: &'static str,
}
#[derive(Clone)]
pub(crate) struct SourceSummary {
    pub authority: Authority,
    pub availability: Availability,
    pub digest: Option<(String, usize)>,
    pub reason: Option<SourceReason>,
    pub collection: Option<CollectionStamp>,
    pub witness: Option<Witness>,
    pub staleness: Option<Staleness>,
    pub invalidated: Option<(
        String,
        usize,
        CollectionStamp,
        Witness,
        Staleness,
        SourceReason,
    )>,
}
impl SourceSummary {
    fn of(s: &SourceObservation) -> Self {
        let digest = |text: &str| {
            (
                crate::project_fs_validation::hex_sha256(text.as_bytes()),
                text.len(),
            )
        };
        Self {
            authority: s.authority(),
            availability: s.availability(),
            digest: s.text().map(digest),
            reason: s.reason(),
            collection: s.collection().cloned(),
            witness: s.witness().cloned(),
            staleness: s.staleness().cloned(),
            invalidated: s.invalidated_evidence().map(|v| {
                let (h, n) = digest(v.text());
                (
                    h,
                    n,
                    v.collection().clone(),
                    v.witness().clone(),
                    v.staleness().clone(),
                    v.reason(),
                )
            }),
        }
    }
}
#[derive(Clone)]
pub(crate) struct SnapshotSummary {
    pub target: ResolvedTarget,
    pub document: DocumentState,
    pub sources: [SourceSummary; 3],
    pub dirty: DirtyObservation,
    pub comparisons: Comparisons,
    pub agreement: Agreement,
    pub consistency: Consistency,
    pub diagnostics: Vec<Diagnostic>,
}
impl SnapshotSummary {
    pub fn of(value: &ObservationSnapshot) -> Self {
        Self {
            target: value.target().clone(),
            document: value.document().clone(),
            sources: [
                SourceSummary::of(value.sources().disk()),
                SourceSummary::of(value.sources().resource()),
                SourceSummary::of(value.sources().buffer()),
            ],
            dirty: value.dirty().clone(),
            comparisons: value.comparisons(),
            agreement: value.agreement(),
            consistency: value.consistency().clone(),
            diagnostics: value.diagnostics().to_vec(),
        }
    }
}
pub(crate) struct Before {
    pub snapshot: SnapshotSummary,
    pub resource_edited: Option<bool>,
    pub edited_collection: Option<CollectionStamp>,
    pub current_version: Option<DecimalCounter>,
    pub saved_version: Option<DecimalCounter>,
}
pub(crate) struct ObservationSummary {
    pub purpose: &'static str,
    pub interval: ObservationInterval,
    pub snapshot: SnapshotSummary,
}
pub(crate) struct NativeEvidence {
    pub entered: bool,
    pub removed: Option<bool>,
    pub discard: bool,
    pub invalidated: bool,
    pub target_buffer: &'static str,
    pub selection: Option<Selection>,
    pub protection: Option<Protection>,
    pub continuation: Option<Step>,
    pub entry: Option<CollectionStamp>,
    pub returned: Option<CollectionStamp>,
}

/// Immutable terminal interpretation, with only source-free authority summaries.
pub struct ClosingOutcome {
    pub(crate) request: Option<ObservationRequest>,
    pub(crate) request_id: RequestId,
    pub(crate) target: Option<ResolvedTarget>,
    pub(crate) interval: ObservationInterval,
    pub(crate) kind: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) stage: &'static str,
    pub(crate) application: &'static str,
    pub(crate) expected: Option<Expected>,
    pub(crate) progress: Progress,
    pub(crate) before: Option<Before>,
    pub(crate) observation: Option<ObservationSummary>,
    pub(crate) resource: ResourceState,
    pub(crate) protection: Protection,
    pub(crate) selection: Option<Selection>,
    pub(crate) history: History,
    pub(crate) diagnostics: Vec<(&'static str, &'static str, Option<i64>)>,
}
impl ClosingOutcome {
    pub fn outcome(&self) -> &str {
        self.kind
    }
    pub fn reason(&self) -> &str {
        self.reason
    }
    pub fn application(&self) -> &str {
        self.application
    }
}
