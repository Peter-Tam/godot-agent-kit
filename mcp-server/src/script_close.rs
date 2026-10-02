//! Checked safe-close intent and immutable, source-free terminal outcomes.
use crate::observation::*;
use crate::script_edit::{BasisError, ExpectedRevisionBasis};
mod attempt;
pub(crate) use attempt::Attempt;
mod outcome;
pub use outcome::ClosingOutcome;
pub(crate) use outcome::{
    Before, Expected, History, NativeEvidence, ObservationSummary, Progress, Protection,
    ResourceState, Selection, SnapshotSummary, SourceSummary, Step,
};

#[derive(Debug)]
pub struct CloseRequest {
    observation: ObservationRequest,
    expected: Option<ExpectedRevisionBasis>,
}
impl CloseRequest {
    /// Borrows prior evidence once; retained intent never contains source bodies.
    ///
    /// `None` supplies recognition-only intent. A supplied ordinary observation
    /// must be complete, clean and independently agreeing; construction does not
    /// authenticate or select an editor, or authorize a native close.
    ///
    /// # Errors
    /// Returns [`BasisError`] for incomplete, dirty, divergent or known-stale
    /// evidence; missing current-version or identity provenance; unsupported
    /// document/source representation; a reused prior request ID; or a prior
    /// target that disagrees with the requested project, path or explicit session.
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
        prior: Option<&ObservationOutcome>,
    ) -> Result<Self, BasisError> {
        let expected = prior
            .map(ExpectedRevisionBasis::from_observation)
            .transpose()?;
        if prior
            .and_then(ObservationOutcome::snapshot)
            .and_then(|s| s.sources().disk().text())
            .is_some_and(|text| {
                text.chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
            })
        {
            return Err(BasisError::UnsupportedRepresentation);
        }
        if script_path.kind() != Some(ScriptKind::ExternalGdscript) {
            return Err(BasisError::UnsupportedTarget);
        }
        if expected.as_ref().is_some_and(|e| {
            e.prior_request_id() == &request_id
                || e.target().project_root() != &project_root
                || e.target().script_path() != &script_path
                || session_id
                    .as_ref()
                    .is_some_and(|s| s != e.target().session_id())
        }) {
            return Err(BasisError::Missing);
        }
        Ok(Self {
            observation: ObservationRequest::new(request_id, project_root, session_id, script_path),
            expected,
        })
    }
    pub fn request_id(&self) -> &RequestId {
        self.observation.request_id()
    }
    pub fn project_root(&self) -> &ProjectRoot {
        self.observation.project_root()
    }
    pub fn session_id(&self) -> Option<&SessionId> {
        self.observation.session_id()
    }
    pub fn script_path(&self) -> &ResourcePath {
        self.observation.script_path()
    }
    pub(crate) fn observation(&self) -> &ObservationRequest {
        &self.observation
    }
    pub(crate) fn expected(&self) -> Option<&ExpectedRevisionBasis> {
        self.expected.as_ref()
    }
    pub(crate) fn boundary(observation: ObservationRequest) -> Self {
        Self {
            observation,
            expected: None,
        }
    }
}
