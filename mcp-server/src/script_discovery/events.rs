//! Closed, bounded worker evidence. No worker-supplied public outcome is accepted.
use super::*;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

pub(crate) fn decimal(s: &str) -> Option<u64> {
    if s.is_empty()
        || s.len() > 20
        || s.len() > 1 && s.starts_with('0')
        || !s.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    s.parse().ok()
}
pub(crate) fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    Option::deserialize(d)
}
fn bounded<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a bounded array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while v.len() < N {
                match a.next_element()? {
                    Some(x) => v.push(x),
                    None => return Ok(v),
                }
            }
            if a.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom("array bound"));
            }
            Ok(v)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn entries<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    bounded::<D, String, BATCH_LIMIT>(d)
}
fn gaps<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Gap>, D::Error> {
    bounded::<D, Gap, 63>(d)
}
pub(crate) fn sessions<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    bounded::<D, String, 64>(d)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Binding {
    pub request_id: String,
    pub project_root: String,
    #[serde(deserialize_with = "required_option")]
    pub session_id: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub project_file_id: Option<Identity>,
}
impl Binding {
    pub(crate) fn request(r: &DiscoveryRequest) -> Self {
        Self {
            request_id: r.request_id().as_str().into(),
            project_root: r.project_root().as_str().into(),
            session_id: r.session_id().map(|s| s.as_str().into()),
            project_file_id: None,
        }
    }
    pub(crate) fn target(t: &DiscoveryTarget) -> Self {
        Self {
            request_id: t.request_id.clone(),
            project_root: t.project_root.clone(),
            session_id: Some(t.session_id.clone()),
            project_file_id: Some(t.project_file_id.clone()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stamp {
    pub clock_id: String,
    pub started_tick_us: String,
    pub finished_tick_us: String,
    pub received_elapsed_us: u64,
}
impl Stamp {
    pub(crate) fn caller(start: u64, end: u64, receipt: u64) -> Self {
        Self {
            clock_id: "caller".into(),
            started_tick_us: start.to_string(),
            finished_tick_us: end.to_string(),
            received_elapsed_us: receipt,
        }
    }
    pub(crate) fn valid(&self, clock: &str, receipt: u64) -> bool {
        self.clock_id == clock
            && self.received_elapsed_us <= receipt
            && decimal(&self.started_tick_us)
                .zip(decimal(&self.finished_tick_us))
                .is_some_and(|(a, b)| a <= b)
            && (clock != "caller"
                || decimal(&self.finished_tick_us)
                    .is_some_and(|end| end <= self.received_elapsed_us))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContextStatus {
    Observed,
    Unavailable,
    Refused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContextReason {
    UnavailableEditorContext,
    UnsupportedVisibilityPolicy,
    UnsupportedDiscovery,
    Busy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScopeFacts {
    pub policy: String,
    pub project_data_directory: String,
    pub filesystem_epoch: String,
    pub scanning: bool,
    pub importing: bool,
}
impl ScopeFacts {
    pub(crate) fn valid(&self) -> bool {
        self.policy == "godot_project_files_v1"
            && ["res://.godot", "res://godot"].contains(&self.project_data_directory.as_str())
            && decimal(&self.filesystem_epoch).is_some()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Context {
    pub collection: Stamp,
    pub status: ContextStatus,
    #[serde(deserialize_with = "required_option")]
    pub reason: Option<ContextReason>,
    #[serde(deserialize_with = "required_option")]
    pub context: Option<ScopeFacts>,
    #[serde(deserialize_with = "required_option")]
    pub expiry_tick_us: Option<String>,
}
impl Context {
    pub(crate) fn valid(&self, session: &str, receipt: u64) -> bool {
        if !self.collection.valid(&format!("editor:{session}"), receipt) {
            return false;
        }
        let owned = self
            .expiry_tick_us
            .as_deref()
            .and_then(decimal)
            .zip(decimal(&self.collection.finished_tick_us))
            .is_some_and(|(expiry, end)| expiry > end);
        match (self.status, self.reason) {
            (ContextStatus::Observed, None) => {
                owned && self.context.as_ref().is_some_and(ScopeFacts::valid)
            }
            (ContextStatus::Unavailable, Some(ContextReason::UnavailableEditorContext))
            | (ContextStatus::Refused, Some(ContextReason::UnsupportedVisibilityPolicy)) => {
                owned && self.context.is_none()
            }
            (
                ContextStatus::Refused,
                Some(ContextReason::UnsupportedDiscovery | ContextReason::Busy),
            ) => self.context.is_none() && self.expiry_tick_us.is_none(),
            _ => false,
        }
    }
    pub(crate) fn failure(&self) -> Option<Reason> {
        match self.reason {
            Some(ContextReason::UnavailableEditorContext) => Some(Reason::RecheckUnavailable),
            Some(ContextReason::UnsupportedVisibilityPolicy) => {
                Some(Reason::UnsupportedVisibilityPolicy)
            }
            Some(ContextReason::UnsupportedDiscovery) => Some(Reason::UnsupportedCapability),
            Some(ContextReason::Busy) => Some(Reason::Busy),
            None => None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gap {
    pub code: Reason,
    pub stage: Stage,
    #[serde(deserialize_with = "required_option")]
    pub scope: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Event {
    Selected {
        binding: Binding,
        target: DiscoveryTarget,
        capability: bool,
    },
    Scope {
        binding: Binding,
        context: Context,
    },
    Entries {
        binding: Binding,
        ordinal: u32,
        collection: Stamp,
        #[serde(deserialize_with = "entries")]
        entries: Vec<String>,
        visited_entries: u32,
        visited_directories: u32,
    },
    Invalidate {
        binding: Binding,
        reason: Reason,
        #[serde(deserialize_with = "required_option")]
        prefix: Option<String>,
    },
    Rechecked {
        binding: Binding,
        context: Context,
        exhausted: bool,
        namespace_checked: bool,
        changed: bool,
        visited_entries: u32,
        visited_directories: u32,
        #[serde(deserialize_with = "gaps")]
        gaps: Vec<Gap>,
        omitted_gaps: u32,
    },
    Failed {
        binding: Binding,
        reason: Reason,
        #[serde(deserialize_with = "required_option")]
        selection: Option<Selection>,
    },
    Done {
        binding: Binding,
    },
}
impl Event {
    pub(crate) fn binding(&self) -> &Binding {
        match self {
            Self::Selected { binding, .. }
            | Self::Scope { binding, .. }
            | Self::Entries { binding, .. }
            | Self::Invalidate { binding, .. }
            | Self::Rechecked { binding, .. }
            | Self::Failed { binding, .. }
            | Self::Done { binding } => binding,
        }
    }
}
pub(crate) fn safe_scope(s: &str) -> bool {
    s == "res://"
        || s.len() <= 2048
            && !s.contains("::")
            && ResourcePath::new(s.trim_end_matches('/').to_owned()).is_ok()
}
