//! Checked project-only discovery requests and immutable terminal observations.
//! Acquisition and authenticated transport remain private consumers of this domain.
use crate::observation::{
    EngineVersion, EvidenceError, FileIdentity, ObservationInterval, ProjectRoot, RequestId,
    ResourcePath, SessionId,
};
use serde::{Deserialize, Serialize};
mod attempt;
pub(crate) mod events;
pub(crate) use attempt::Attempt;
pub(crate) use events::{Binding, Context, Event, Gap, Stamp};

pub(crate) const ENTRY_LIMIT: usize = 1024;
pub(crate) const DIRECTORY_LIMIT: u32 = 1024;
pub(crate) const WORK_LIMIT: u32 = 16384;
pub(crate) const DEPTH_LIMIT: usize = 64;
pub(crate) const BATCH_LIMIT: usize = 64;
pub(crate) const FRAME_LIMIT: usize = 512 * 1024;
pub(crate) const RESULT_LIMIT: usize = 6 * 1024 * 1024;
pub(crate) const CUTOFF_US: u64 = 4_500_000;

#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    request_id: RequestId,
    project_root: ProjectRoot,
    session_id: Option<SessionId>,
}
impl DiscoveryRequest {
    pub fn new(
        request_id: RequestId,
        project_root: ProjectRoot,
        session_id: Option<SessionId>,
    ) -> Self {
        Self {
            request_id,
            project_root,
            session_id,
        }
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Identity {
    pub device: String,
    pub inode: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Version {
    pub version: String,
    pub hash: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DiscoveryTarget {
    pub request_id: String,
    pub project_root: String,
    pub project_file_id: Identity,
    pub session_id: String,
    pub godot_version: Version,
}
impl DiscoveryTarget {
    pub(crate) fn for_request(
        request: &DiscoveryRequest,
        project_root: ProjectRoot,
        identity: FileIdentity,
        session_id: SessionId,
        version: EngineVersion,
    ) -> Result<Self, EvidenceError> {
        if request
            .session_id
            .as_ref()
            .is_some_and(|s| s != &session_id)
        {
            return Err(EvidenceError::WrongTarget);
        }
        Ok(Self {
            request_id: request.request_id.as_str().into(),
            project_root: project_root.as_str().into(),
            project_file_id: Identity {
                device: identity.device().as_str().into(),
                inode: identity.inode().as_str().into(),
            },
            session_id: session_id.as_str().into(),
            godot_version: Version {
                version: version.version().into(),
                hash: version.hash().into(),
            },
        })
    }
    pub(crate) fn valid(&self, request: &DiscoveryRequest) -> bool {
        self.request_id == request.request_id.as_str()
            && ProjectRoot::new(self.project_root.clone()).is_ok()
            && request
                .session_id()
                .is_none_or(|s| s.as_str() == self.session_id)
            && SessionId::new(self.session_id.clone()).is_ok()
            && events::decimal(&self.project_file_id.device).is_some()
            && events::decimal(&self.project_file_id.inode).is_some()
            && EngineVersion::new(
                self.godot_version.version.clone(),
                self.godot_version.hash.clone(),
            )
            .is_ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Reason {
    DeniedAccess,
    UnsafeRegistry,
    AuthenticationFailed,
    OutOfProject,
    ProjectIdentityChanged,
    AmbiguousTarget,
    InvalidRequest,
    InvalidProject,
    UnsupportedVersion,
    UnsupportedCapability,
    UnsupportedVisibilityPolicy,
    EditorUnavailable,
    Busy,
    ProtocolError,
    Cancelled,
    SessionReplaced,
    DisconnectedEditor,
    Timeout,
    DirectoryUnreadable,
    EntryUnreadable,
    UnsafeEntry,
    UnsupportedPath,
    UnsupportedMarker,
    NamespaceChanged,
    ScopeChanged,
    EditorInventoryBusy,
    EditorInventoryChanged,
    RecheckUnavailable,
    EntryLimit,
    DirectoryLimit,
    DepthLimit,
    WorkLimit,
    ResultLimit,
    AdditionalGaps,
}
impl Reason {
    pub(crate) fn priority(self) -> u8 {
        match self {
            Self::DeniedAccess
            | Self::UnsafeRegistry
            | Self::AuthenticationFailed
            | Self::OutOfProject
            | Self::ProjectIdentityChanged => 0,
            Self::AmbiguousTarget => 1,
            Self::InvalidRequest
            | Self::InvalidProject
            | Self::UnsupportedVersion
            | Self::UnsupportedCapability
            | Self::UnsupportedVisibilityPolicy
            | Self::EditorUnavailable
            | Self::Busy => 2,
            Self::ProtocolError => 3,
            Self::Cancelled => 4,
            Self::SessionReplaced => 5,
            Self::DisconnectedEditor => 6,
            Self::Timeout => 7,
            _ => 8,
        }
    }
    pub(crate) fn suppress(self) -> bool {
        self.priority() <= 2 || matches!(self, Self::SessionReplaced | Self::ScopeChanged)
    }
    fn text(self) -> (&'static str, &'static str) {
        match self {
            Self::InvalidRequest => (
                "The discovery request is invalid.",
                "Provide a valid absolute project root and optional exact session.",
            ),
            Self::AmbiguousTarget => (
                "More than one authenticated editor matches.",
                "Select an exact session and submit a new request.",
            ),
            Self::DeniedAccess
            | Self::UnsafeRegistry
            | Self::AuthenticationFailed
            | Self::OutOfProject
            | Self::ProjectIdentityChanged => (
                "The requested project boundary could not be safely established.",
                "Restore the authorized project boundary before a new request.",
            ),
            Self::EntryLimit
            | Self::DirectoryLimit
            | Self::DepthLimit
            | Self::WorkLimit
            | Self::ResultLimit => (
                "Discovery exceeded a fixed resource bound.",
                "Do not infer absence from this limited listing.",
            ),
            Self::AdditionalGaps => (
                "Additional discovery gaps were omitted.",
                "Do not infer absence from this limited listing.",
            ),
            Self::NamespaceChanged | Self::ScopeChanged | Self::EditorInventoryChanged => (
                "The admitted discovery evidence changed during collection.",
                "Submit a new request for a fresh observation.",
            ),
            Self::ProtocolError => (
                "Discovery received invalid or contradictory evidence.",
                "Submit a new request after checking the matching installation.",
            ),
            Self::Cancelled | Self::Timeout | Self::DisconnectedEditor | Self::SessionReplaced => (
                "Discovery was interrupted before final verification.",
                "Treat retained entries only as earlier observations.",
            ),
            Self::UnsupportedVersion
            | Self::UnsupportedCapability
            | Self::UnsupportedVisibilityPolicy => (
                "The selected editor does not support this discovery profile.",
                "Use the supported matching editor and addon installation.",
            ),
            Self::EditorUnavailable | Self::Busy | Self::EditorInventoryBusy => (
                "The selected editor is unavailable or busy.",
                "Submit a new request when the selected editor is available.",
            ),
            _ => (
                "Part of the admitted discovery scope could not be verified.",
                "Do not infer absence from this limited listing.",
            ),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Stage {
    ValidateRequest,
    ResolveTarget,
    Authenticate,
    BeginScope,
    Enumerate,
    Recheck,
    Finalize,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Diagnostic {
    code: Reason,
    stage: Stage,
    scope: Option<String>,
    message: &'static str,
    action: &'static str,
    omitted_count: Option<u32>,
}
impl Diagnostic {
    pub(crate) fn new(reason: Reason, stage: Stage, scope: Option<String>) -> Self {
        let (message, action) = reason.text();
        Self {
            code: reason,
            stage,
            scope,
            message,
            action,
            omitted_count: None,
        }
    }
    pub(crate) fn aggregate(count: u32) -> Self {
        let mut d = Self::new(Reason::AdditionalGaps, Stage::Finalize, None);
        d.omitted_count = Some(count);
        d
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Scope {
    policy: &'static str,
    project_data_directory: String,
    exclusions: [&'static str; 7],
}
impl Scope {
    pub(crate) fn new(data: String) -> Self {
        Self {
            policy: "godot_project_files_v1",
            project_data_directory: data,
            exclusions: [
                "dot_names",
                "project_data_directory",
                "gdignore_subtrees",
                "nested_projects",
                "non_gdscript_files",
                "non_regular_files",
                "embedded_or_unsaved_documents",
            ],
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Consistency {
    pub recheck: &'static str,
    pub stability: &'static str,
    pub atomic: bool,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Inventory {
    pub scope: Scope,
    pub entries: Vec<String>,
    pub collection: Stamp,
    pub coverage: &'static str,
    pub validity: &'static str,
    pub consistency: Consistency,
    pub visited_entries: u32,
    pub visited_directories: u32,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RequestedTarget {
    project_root: String,
    session_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Selection {
    #[serde(deserialize_with = "events::sessions")]
    pub candidate_sessions: Vec<String>,
    pub missing_selector: String,
}
#[derive(Debug, Clone, Serialize)]
struct Interval {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryOutcome {
    schema_version: u32,
    request_id: String,
    outcome: &'static str,
    interval: Interval,
    requested_target: Option<RequestedTarget>,
    resolved_target: Option<DiscoveryTarget>,
    inventory: Option<Inventory>,
    diagnostics: Vec<Diagnostic>,
    selection: Option<Selection>,
}
impl DiscoveryOutcome {
    pub fn exit_code(&self) -> i32 {
        match self.outcome {
            "complete_listing" => 0,
            "limited_listing" => 2,
            "refused" => 3,
            _ => 4,
        }
    }
    pub fn outcome(&self) -> &str {
        self.outcome
    }
    /// Serialize one bounded terminal object followed by a newline.
    pub fn to_json_line(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec(self)?;
        if bytes.len() > RESULT_LIMIT {
            return Err(<serde_json::Error as serde::ser::Error>::custom(
                "discovery result bound",
            ));
        }
        bytes.push(b'\n');
        Ok(bytes)
    }
    pub fn invalid_request(request_id: RequestId, interval: ObservationInterval) -> Self {
        Self {
            schema_version: 1,
            request_id: request_id.as_str().into(),
            outcome: "refused",
            interval: Interval::from(interval),
            requested_target: None,
            resolved_target: None,
            inventory: None,
            diagnostics: vec![Diagnostic::new(
                Reason::InvalidRequest,
                Stage::ValidateRequest,
                None,
            )],
            selection: None,
        }
    }
    /// Safe structured boundary when the launcher cannot complete the normal contract.
    pub fn host_failure(request_id: RequestId, interval: ObservationInterval) -> Self {
        let mut value = Self::invalid_request(request_id, interval);
        value.outcome = "interrupted";
        value.diagnostics = vec![Diagnostic::new(
            Reason::ProtocolError,
            Stage::Finalize,
            None,
        )];
        value
    }
    pub(crate) fn bounded(mut self) -> Self {
        if within_result_limit(&self) {
            return self;
        }
        if self.outcome == "complete_listing" {
            self.outcome = "limited_listing";
        }
        self.diagnostics.truncate(62);
        self.diagnostics
            .push(Diagnostic::new(Reason::ResultLimit, Stage::Finalize, None));
        if let Some(inventory) = &mut self.inventory {
            inventory.coverage = "partial";
        }
        let entries = self
            .inventory
            .as_mut()
            .map(|i| std::mem::take(&mut i.entries));
        let mut size = serialized_size(&self).unwrap_or(RESULT_LIMIT);
        if let (Some(inventory), Some(entries)) = (&mut self.inventory, entries) {
            for entry in entries {
                let Some(length) = serialized_size(&entry) else {
                    break;
                };
                let separator = usize::from(!inventory.entries.is_empty());
                if length + separator > RESULT_LIMIT.saturating_sub(size) {
                    break;
                }
                size += length + separator;
                inventory.entries.push(entry);
            }
        }
        self
    }
}
fn within_result_limit(value: &DiscoveryOutcome) -> bool {
    serialized_size(value).is_some()
}
fn serialized_size<T: Serialize + ?Sized>(value: &T) -> Option<usize> {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > RESULT_LIMIT.saturating_sub(self.0) {
                return Err(std::io::Error::other("discovery result bound"));
            }
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    serde_json::to_writer(&mut count, value).ok()?;
    Some(count.0)
}
impl From<ObservationInterval> for Interval {
    fn from(value: ObservationInterval) -> Self {
        Self {
            started_unix_ms: value.started_unix_ms(),
            finished_unix_ms: value.finished_unix_ms(),
            elapsed_us: value.elapsed_us(),
        }
    }
}
pub(crate) fn eligible(path: &str, data: &str) -> bool {
    if ResourcePath::new(path.to_owned()).is_err() {
        return false;
    }
    let Some(relative) = path.strip_prefix("res://") else {
        return false;
    };
    !relative.contains(':')
        && relative.split('/').all(|p| !p.starts_with('.'))
        && path != data
        && !path
            .strip_prefix(data)
            .is_some_and(|tail| tail.starts_with('/'))
        && relative
            .rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("gd"))
}
