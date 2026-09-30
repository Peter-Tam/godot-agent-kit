//! Protocol-independent known-path opening intent and immutable terminal interpretation.
//! Native receipts describe effects; only independent observation can establish success.
mod attempt;
mod outcome;
use crate::observation::{
    EvidenceError, ObservationRequest, ProjectRoot, RequestId, ResourcePath, ScriptKind, SessionId,
};
pub(crate) use attempt::{Attempt, NativeEvidence};
pub use outcome::OpeningOutcome;
pub(crate) use outcome::*;

/// One explicit external GDScript target. There is no source, focus or retry input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest(ObservationRequest);
impl OpenRequest {
    /// # Errors
    /// Rejects embedded and non-GDScript locators; filesystem authority is acquired later.
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
        script_path: ResourcePath,
    ) -> Result<Self, EvidenceError> {
        if script_path.kind() != Some(ScriptKind::ExternalGdscript) {
            return Err(EvidenceError::InvalidResourcePath);
        }
        Ok(Self(ObservationRequest::new(
            request_id,
            project_root,
            session_id,
            script_path,
        )))
    }
    pub fn request_id(&self) -> &RequestId {
        self.0.request_id()
    }
    pub fn project_root(&self) -> &ProjectRoot {
        self.0.project_root()
    }
    pub fn session_id(&self) -> Option<&SessionId> {
        self.0.session_id()
    }
    pub fn script_path(&self) -> &ResourcePath {
        self.0.script_path()
    }
    pub(crate) fn observation(&self) -> &ObservationRequest {
        &self.0
    }
}
