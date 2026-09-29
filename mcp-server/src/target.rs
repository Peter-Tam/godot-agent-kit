//! Source-free target resolution: inspect private metadata, authenticate, then bind one session.
use std::fmt;
use std::path::Path;
use std::time::Instant;

use crate::bridge::{self, Capabilities};
use crate::observation::{
    Diagnostic, DiagnosticCode, ObservationRequest, OutcomeKind, ResolvedTarget, Selection,
    SessionId, Stage,
};
use crate::project_fs;

pub struct SelectedSession {
    target: ResolvedTarget,
    capabilities: Capabilities,
    native_api_revision: u32,
    native_build_id: String,
    requested_project_root: crate::observation::ProjectRoot,
    // The authenticated channel and rooted directory stay attached to this unique selection.
    pub(crate) socket: std::net::TcpStream,
    pub(crate) project_directory: cap_std::fs::Dir,
    pub(crate) advertised_project_root: String,
}
impl fmt::Debug for SelectedSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedSession")
            .field("target", &self.target)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}
impl SelectedSession {
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }
    pub(crate) fn native_api_revision(&self) -> u32 {
        self.native_api_revision
    }
    pub(crate) fn native_build_id(&self) -> &str {
        &self.native_build_id
    }
    pub(crate) fn matches_request(&self, request: &ObservationRequest) -> bool {
        self.target.request_id() == request.request_id()
            && &self.requested_project_root == request.project_root()
            && self.target.script_path() == request.script_path()
            && request
                .session_id()
                .is_none_or(|id| id == self.target.session_id())
    }
    pub(crate) fn observation_channel(
        &mut self,
    ) -> (&mut std::net::TcpStream, &ResolvedTarget, &str) {
        (
            &mut self.socket,
            &self.target,
            &self.advertised_project_root,
        )
    }
}

/// A source-free routing refusal with only fixed diagnostics and, for established
/// ambiguity, authenticated public session IDs. Private endpoints and proofs stay internal.
pub struct RoutingFailure {
    pub outcome: OutcomeKind,
    pub diagnostic: Diagnostic,
    pub selection: Option<Selection>,
}
impl RoutingFailure {
    pub(crate) fn new(outcome: OutcomeKind, code: DiagnosticCode, stage: Stage) -> Self {
        Self {
            outcome,
            diagnostic: Diagnostic::new(code, stage, Some(crate::observation::Surface::Session)),
            selection: None,
        }
    }
}
impl fmt::Debug for RoutingFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutingFailure")
            .field("outcome", &self.outcome)
            .field("diagnostic", &self.diagnostic)
            .field("selection", &self.selection)
            .finish()
    }
}
impl fmt::Display for RoutingFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.diagnostic.message())
    }
}
impl std::error::Error for RoutingFailure {}

fn fail(outcome: OutcomeKind, code: DiagnosticCode) -> RoutingFailure {
    RoutingFailure::new(outcome, code, Stage::ResolveTarget)
}

/// No script/container source is opened during resolution. Authentication denial is
/// terminal; unresolved candidates cannot establish implicit unique selection.
///
/// # Errors
/// Refuses unsafe project or registry metadata, uncertain matching candidates,
/// authentication/bridge failures, deadline expiry, and ambiguous live sessions.
pub fn resolve(
    request: &ObservationRequest,
    registry: &Path,
    deadline: Instant,
) -> Result<SelectedSession, RoutingFailure> {
    if Instant::now() >= deadline {
        return Err(fail(OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded));
    }
    let project = project_fs::project(request)?;
    let directory = project_fs::open_registry(registry)?;
    let registry_root = std::fs::canonicalize(registry)
        .map_err(|_| fail(OutcomeKind::DeniedAccess, DiagnosticCode::UnsafeRegistry))?;
    if registry_root.starts_with(Path::new(project.root.as_str())) {
        return Err(fail(
            OutcomeKind::DeniedAccess,
            DiagnosticCode::UnsafeRegistry,
        ));
    }
    let mut filenames = Vec::new();
    for entry in directory
        .read_dir(".")
        .map_err(|_| fail(OutcomeKind::DeniedAccess, DiagnosticCode::UnsafeRegistry))?
    {
        let entry =
            entry.map_err(|_| fail(OutcomeKind::DeniedAccess, DiagnosticCode::UnsafeRegistry))?;
        let name = entry.file_name();
        let filename = name
            .to_str()
            .ok_or_else(|| fail(OutcomeKind::DeniedAccess, DiagnosticCode::UnsafeRegistry))?;
        if filename.ends_with(".json") {
            if filenames.len() == 32 {
                return Err(fail(
                    OutcomeKind::UnsupportedObservation,
                    DiagnosticCode::UnsupportedObservation,
                ));
            }
            filenames.push(filename.to_owned());
        }
        if Instant::now() >= deadline {
            return Err(fail(OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded));
        }
    }
    let mut descriptors = Vec::with_capacity(filenames.len());
    for filename in filenames {
        let bytes = project_fs::descriptor_bytes(&directory, &filename)?;
        descriptors.push(bridge::Descriptor::parse(&bytes, &filename)?);
        if Instant::now() >= deadline {
            return Err(fail(OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded));
        }
    }
    // Sort by public session ID for deterministic diagnostics; never select by this order.
    descriptors.sort_by(|a, b| a.session_id.cmp(&b.session_id));
    let mut confirmed: Vec<(
        SessionId,
        bridge::Authenticated,
        crate::observation::EngineVersion,
        String,
    )> = Vec::new();
    let mut pending = false;
    let mut deferred_failure: Option<RoutingFailure> = None;
    for descriptor in &descriptors {
        if request
            .session_id()
            .is_some_and(|id| id.as_str() != descriptor.session_id)
        {
            continue;
        }
        if !project_fs::matching_project(&descriptor.project_root, &project)? {
            continue;
        }
        if !descriptor.supported() {
            let error = fail(
                OutcomeKind::UnsupportedObservation,
                DiagnosticCode::UnsupportedVersion,
            );
            deferred_failure.get_or_insert(error);
            continue;
        }
        let session = match SessionId::new(descriptor.session_id.clone()) {
            Ok(session) => session,
            Err(_) => {
                deferred_failure = Some(fail(
                    OutcomeKind::ProtocolError,
                    DiagnosticCode::InvalidFrame,
                ));
                continue;
            }
        };
        match bridge::authenticate(descriptor, request.request_id().as_str(), deadline) {
            Ok(authenticated) => {
                let version = match crate::observation::EngineVersion::new(
                    &descriptor.godot_version,
                    &descriptor.engine_hash,
                ) {
                    Ok(version) => version,
                    Err(_) => {
                        deferred_failure = Some(fail(
                            OutcomeKind::ProtocolError,
                            DiagnosticCode::InvalidFrame,
                        ));
                        continue;
                    }
                };
                confirmed.push((
                    session,
                    authenticated,
                    version,
                    descriptor.project_root.clone(),
                ));
            }
            Err(error) if error.outcome == OutcomeKind::EditorUnavailable => {}
            Err(error) if error.outcome == OutcomeKind::Timeout => {
                pending = true;
                break;
            }
            Err(error) if error.outcome == OutcomeKind::DeniedAccess => return Err(error),
            Err(error) if error.outcome == OutcomeKind::ProtocolError => {
                deferred_failure = Some(error);
            }
            Err(error) => {
                deferred_failure.get_or_insert(error);
            }
        }
    }
    if confirmed.len() > 1 {
        let sessions = confirmed.into_iter().map(|(id, _, _, _)| id).collect();
        let selection = Selection::ambiguous(sessions)
            .map_err(|_| fail(OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame))?;
        let mut error = fail(
            OutcomeKind::AmbiguousTarget,
            DiagnosticCode::AmbiguousTarget,
        );
        error.selection = Some(selection);
        return Err(error);
    }
    if let Some(error) = deferred_failure {
        return Err(error);
    }
    if pending || Instant::now() >= deadline {
        return Err(fail(OutcomeKind::Timeout, DiagnosticCode::DeadlineExceeded));
    }
    let Some((id, authenticated, version, advertised_project_root)) = confirmed.pop() else {
        return Err(fail(
            OutcomeKind::EditorUnavailable,
            DiagnosticCode::EditorUnavailable,
        ));
    };
    let normalized = ObservationRequest::new(
        request.request_id().clone(),
        project.root.clone(),
        request.session_id().cloned(),
        request.script_path().clone(),
    );
    let target =
        ResolvedTarget::for_request(&normalized, project.root, project.file_id, id, version)
            .map_err(|_| fail(OutcomeKind::ProtocolError, DiagnosticCode::InvalidFrame))?;
    Ok(SelectedSession {
        target,
        capabilities: authenticated.capabilities,
        native_api_revision: authenticated.native_api_revision,
        native_build_id: authenticated.native_build_id,
        requested_project_root: request.project_root().clone(),
        socket: authenticated.socket,
        project_directory: project.directory,
        advertised_project_root,
    })
}
