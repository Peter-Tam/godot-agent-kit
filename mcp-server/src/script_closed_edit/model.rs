use super::*;
use crate::observation::{Checks, ObservationOutcome, OpenState, Stability, SOURCE_LIMIT_BYTES};
use crate::project_fs_validation as confined;
fn nullable<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

pub(crate) fn unsigned(s: &str) -> bool {
    !s.is_empty()
        && (s == "0" || !s.starts_with('0'))
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.parse::<u64>().is_ok()
}
pub(crate) fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Timestamp {
    pub seconds: String,
    pub nanoseconds: u32,
}
impl Timestamp {
    fn valid(&self) -> bool {
        self.nanoseconds < 1_000_000_000
            && self
                .seconds
                .parse::<i64>()
                .is_ok_and(|v| v.to_string() == self.seconds)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileRevision {
    pub device: String,
    pub inode: String,
    pub utf8_bytes: String,
    pub sha256: String,
    pub mtime: Timestamp,
    pub ctime: Timestamp,
}
impl FileRevision {
    pub(crate) fn valid(&self) -> bool {
        unsigned(&self.device)
            && unsigned(&self.inode)
            && unsigned(&self.utf8_bytes)
            && self
                .utf8_bytes
                .parse::<usize>()
                .is_ok_and(|n| n <= SOURCE_LIMIT_BYTES)
            && hash(&self.sha256)
            && self.mtime.valid()
            && self.ctime.valid()
    }
    pub(crate) fn capture(c: &confined::Captured) -> Option<Self> {
        let mtime = Timestamp {
            seconds: c.metadata.2.to_string(),
            nanoseconds: u32::try_from(c.metadata.3).ok()?,
        };
        let ctime = Timestamp {
            seconds: c.metadata.4.to_string(),
            nanoseconds: u32::try_from(c.metadata.5).ok()?,
        };
        let r = Self {
            device: c.device.to_string(),
            inode: c.inode.to_string(),
            utf8_bytes: c.bytes.to_string(),
            sha256: c.sha256.clone(),
            mtime,
            ctime,
        };
        r.valid().then_some(r)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceState {
    pub state: String,
    #[serde(deserialize_with = "nullable")]
    pub instance_id: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub path: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub source: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub edited: Option<bool>,
    #[serde(deserialize_with = "nullable")]
    pub profile_sha256: Option<String>,
}
impl ResourceState {
    pub(crate) fn valid(&self, path: &str) -> bool {
        match self.state.as_str() {
            "absent" | "unavailable" => {
                self.instance_id.is_none()
                    && self.path.is_none()
                    && self.source.is_none()
                    && self.edited.is_none()
                    && self.profile_sha256.is_none()
            }
            "present" => {
                self.instance_id
                    .as_deref()
                    .is_some_and(|s| unsigned(s) && s != "0")
                    && self.path.as_deref() == Some(path)
                    && self
                        .source
                        .as_ref()
                        .is_some_and(|s| s.len() <= SOURCE_LIMIT_BYTES)
                    && self.edited.is_some()
                    && self.profile_sha256.as_deref().is_some_and(hash)
            }
            _ => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct State {
    pub project_device: String,
    pub project_inode: String,
    pub file_revision: FileRevision,
    pub close_epoch: String,
    pub lifecycle: String,
    pub resource: ResourceState,
}
impl State {
    pub(crate) fn valid(&self, path: &str) -> bool {
        unsigned(&self.project_device)
            && unsigned(&self.project_inode)
            && unsigned(&self.close_epoch)
            && matches!(self.lifecycle.as_str(), "closed" | "open" | "unknown")
            && self.file_revision.valid()
            && self.resource.valid(path)
    }
    pub(crate) fn expected(&self) -> ExpectedState {
        let r = &self.resource;
        ExpectedState {
            project_device: self.project_device.clone(),
            project_inode: self.project_inode.clone(),
            file_revision: self.file_revision.clone(),
            close_epoch: self.close_epoch.clone(),
            resource: ExpectedResource {
                state: r.state.clone(),
                instance_id: r.instance_id.clone(),
                path: r.path.clone(),
                source_sha256: r
                    .source
                    .as_ref()
                    .map(|s| confined::hex_sha256(s.as_bytes())),
                utf8_bytes: r.source.as_ref().map(|s| s.len().to_string()),
                edited: r.edited,
                profile_sha256: r.profile_sha256.clone(),
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedResource {
    pub state: String,
    #[serde(deserialize_with = "nullable")]
    pub instance_id: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub path: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub source_sha256: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub utf8_bytes: Option<String>,
    #[serde(deserialize_with = "nullable")]
    pub edited: Option<bool>,
    #[serde(deserialize_with = "nullable")]
    pub profile_sha256: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedState {
    pub project_device: String,
    pub project_inode: String,
    pub file_revision: FileRevision,
    pub close_epoch: String,
    pub resource: ExpectedResource,
}
impl ExpectedState {
    pub(crate) fn valid(&self, path: &str) -> bool {
        let r = &self.resource;
        unsigned(&self.project_device)
            && unsigned(&self.project_inode)
            && unsigned(&self.close_epoch)
            && self.file_revision.valid()
            && match r.state.as_str() {
                "absent" => {
                    r.instance_id.is_none()
                        && r.path.is_none()
                        && r.source_sha256.is_none()
                        && r.utf8_bytes.is_none()
                        && r.edited.is_none()
                        && r.profile_sha256.is_none()
                }
                "present" => {
                    r.instance_id
                        .as_deref()
                        .is_some_and(|s| unsigned(s) && s != "0")
                        && r.path.as_deref() == Some(path)
                        && r.source_sha256.as_ref() == Some(&self.file_revision.sha256)
                        && r.utf8_bytes.as_ref() == Some(&self.file_revision.utf8_bytes)
                        && r.edited == Some(false)
                        && r.profile_sha256.as_deref().is_some_and(hash)
                }
                _ => false,
            }
    }
    pub(crate) fn matches(&self, s: &State) -> bool {
        s.lifecycle == "closed" && s.expected() == *self
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeReceipt {
    pub request_id: String,
    pub session_id: String,
    pub script_path: String,
    pub native_build_id: String,
    pub native_api_revision: u32,
    pub phase: String,
    pub resource_entered: bool,
    pub resource_changed: bool,
    pub disk_entered: bool,
    pub written_bytes: String,
    pub truncate_done: bool,
    pub fsync_done: bool,
    pub pread_done: bool,
    pub futimens_called: bool,
    pub mtime_restored: bool,
    pub terminal_discard: bool,
    #[serde(deserialize_with = "nullable")]
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClosedExpectedBasis {
    pub(crate) prior_request: String,
    pub(crate) project: String,
    pub(crate) session: String,
    pub(crate) path: String,
    pub(crate) state: ExpectedState,
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum BasisError {
    Ineligible,
    Correlation,
}
impl ClosedExpectedBasis {
    pub(crate) fn from_observation(o: &ObservationOutcome, state: &State) -> Option<Self> {
        let s = o.snapshot()?;
        let t = s.target();
        if s.document().open_state().value() != Some(&OpenState::NotOpen)
            || s.consistency().stability() == Stability::Changed
            || s.consistency().checks() != Checks::Performed
            || s.consistency().recheck_reason().is_some()
            || !state.valid(t.script_path().as_str())
            || state.lifecycle != "closed"
            || state.resource.state == "unavailable"
            || state.resource.edited == Some(true)
        {
            return None;
        }
        let d = s.sources().disk().text()?;
        let file = s.sources().disk().witness()?.disk_file_id()?;
        if file.device().as_str() != state.file_revision.device
            || file.inode().as_str() != state.file_revision.inode
        {
            return None;
        }
        if state.project_device != t.project_file_id().device().as_str()
            || state.project_inode != t.project_file_id().inode().as_str()
            || state.file_revision.sha256 != confined::hex_sha256(d.as_bytes())
            || state.file_revision.utf8_bytes != d.len().to_string()
        {
            return None;
        }
        if state.resource.state == "present"
            && (state.resource.source.as_deref() != Some(d)
                || s.sources().resource().text() != Some(d))
        {
            return None;
        }
        if state.resource.state == "present"
            && s.sources()
                .resource()
                .witness()?
                .script_instance_id()?
                .as_str()
                != state.resource.instance_id.as_deref()?
        {
            return None;
        }
        if state.resource.state == "absent"
            && s.sources().resource().reason()
                != Some(crate::observation::SourceReason::ResourceNotLoaded)
        {
            return None;
        }
        Some(Self {
            prior_request: o.request_id().as_str().to_owned(),
            project: t.project_root().as_str().to_owned(),
            session: t.session_id().as_str().to_owned(),
            path: t.script_path().as_str().to_owned(),
            state: state.expected(),
        })
    }
    pub(crate) fn stable_commitment(&self, c: &mut ring::digest::Context) {
        fn field(c: &mut ring::digest::Context, s: &str) {
            c.update(&(s.len() as u64).to_be_bytes());
            c.update(s.as_bytes());
        }
        field(c, "godot-agent-kit/closed-basis/v1");
        for (tag, value) in [
            &self.project,
            &self.session,
            &self.path,
            &self.state.project_device,
            &self.state.project_inode,
            &self.state.close_epoch,
            &self.state.file_revision.device,
            &self.state.file_revision.inode,
            &self.state.file_revision.utf8_bytes,
            &self.state.file_revision.sha256,
            &self.state.file_revision.mtime.seconds,
            &self.state.file_revision.ctime.seconds,
            &self.state.resource.state,
        ]
        .into_iter()
        .enumerate()
        {
            c.update(&[tag as u8]);
            field(c, value);
        }
        c.update(&[13]);
        c.update(&self.state.file_revision.mtime.nanoseconds.to_be_bytes());
        c.update(&[14]);
        c.update(&self.state.file_revision.ctime.nanoseconds.to_be_bytes());
        if self.state.resource.state == "present" {
            for (tag, value) in [
                self.state.resource.instance_id.as_deref().unwrap_or(""),
                self.state.resource.path.as_deref().unwrap_or(""),
                self.state.resource.source_sha256.as_deref().unwrap_or(""),
                self.state.resource.utf8_bytes.as_deref().unwrap_or(""),
                self.state.resource.profile_sha256.as_deref().unwrap_or(""),
            ]
            .into_iter()
            .enumerate()
            {
                c.update(&[15 + tag as u8]);
                field(c, value);
            }
            c.update(&[20, u8::from(self.state.resource.edited == Some(true))]);
        }
    }
    pub(crate) fn resource_present(&self) -> bool {
        self.state.resource.state == "present"
    }
}
pub(crate) struct CheckedClosedRequest {
    pub(crate) id: RequestId,
    pub(crate) basis: ClosedExpectedBasis,
    pub(crate) replacement: ReplacementSource,
}
impl CheckedClosedRequest {
    pub(crate) fn new(
        id: RequestId,
        basis: ClosedExpectedBasis,
        replacement: ReplacementSource,
    ) -> Result<Self, BasisError> {
        if !basis.state.valid(&basis.path) {
            return Err(BasisError::Ineligible);
        }
        if id.as_str() == basis.prior_request {
            return Err(BasisError::Correlation);
        }
        Ok(Self {
            id,
            basis,
            replacement,
        })
    }
    pub(crate) fn observation(&self) -> Result<crate::observation::ObservationRequest, BasisError> {
        Ok(crate::observation::ObservationRequest::new(
            self.id.clone(),
            ProjectRoot::new(self.basis.project.clone()).map_err(|_| BasisError::Ineligible)?,
            Some(SessionId::new(self.basis.session.clone()).map_err(|_| BasisError::Ineligible)?),
            ResourcePath::new(self.basis.path.clone()).map_err(|_| BasisError::Ineligible)?,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dirty_resource_is_information_not_mutation_authority() {
        let r = ResourceState {
            state: "present".into(),
            instance_id: Some("1".into()),
            path: Some("res://a.gd".into()),
            source: Some("extends Node\n".into()),
            edited: Some(true),
            profile_sha256: Some("a".repeat(64)),
        };
        assert!(r.valid("res://a.gd"));
        assert!(!r.valid("res://other.gd"));
        let mut missing = r;
        missing.edited = None;
        assert!(!missing.valid("res://a.gd"));
    }
    #[test]
    fn canonical_scalars_are_strict() {
        for s in ["", "00", "01", "-1", "18446744073709551616"] {
            assert!(!unsigned(s));
        }
        assert!(unsigned("0"));
    }
    #[test]
    fn timestamp_bounds_and_spelling_are_strict() {
        assert!(!Timestamp {
            seconds: "-0".into(),
            nanoseconds: 0
        }
        .valid());
        assert!(!Timestamp {
            seconds: "0".into(),
            nanoseconds: 1_000_000_000
        }
        .valid());
    }
    #[test]
    fn missing_cache_facts_are_not_absence() {
        let mut r = ResourceState {
            state: "absent".into(),
            instance_id: None,
            path: None,
            source: None,
            edited: None,
            profile_sha256: None,
        };
        assert!(r.valid("res://a.gd"));
        r.edited = Some(false);
        assert!(!r.valid("res://a.gd"));
        r.state = "unavailable".into();
        assert!(!r.valid("res://a.gd"));
        r.edited = None;
        assert!(r.valid("res://a.gd"));
        assert_eq!(r.state, "unavailable");
    }
}

#[cfg(test)]
#[path = "regression.rs"]
mod regression;
