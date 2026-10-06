//! Validate caller intent, then use exactly one existing supervised operation.
use super::output;
use crate::observation::{ObservationRequest, ProjectRoot, RequestId, ResourcePath, SessionId};
use crate::runner::{self, AttemptClock};
use crate::script_discovery::DiscoveryRequest;
use crate::script_edit::ExactReplacement;
use crate::script_read::{ScriptEditRequest, ScriptRevision};
use ring::rand::{SecureRandom, SystemRandom};
use rmcp::model::CallToolResult;
use serde_json::{Map, Value};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Operation {
    Discover,
    Read,
    Edit,
}
impl Operation {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "discover_scripts" => Some(Self::Discover),
            "read_script" => Some(Self::Read),
            "edit_script" => Some(Self::Edit),
            _ => None,
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Discover => "discover_scripts",
            Self::Read => "read_script",
            Self::Edit => "edit_script",
        }
    }
    pub(super) fn budget(self) -> Duration {
        Duration::from_secs(if self == Self::Edit { 10 } else { 5 })
    }
}
pub(super) enum Failure {
    Busy,
    HostBeforeDispatch,
    HostAfterDispatch,
}
pub(super) fn failure(operation: Operation, id: &RequestId, reason: Failure) -> CallToolResult {
    output::failure(
        operation,
        id,
        match reason {
            Failure::Busy => output::Failure::Busy,
            Failure::HostBeforeDispatch => output::Failure::HostBeforeDispatch,
            Failure::HostAfterDispatch => output::Failure::HostAfterDispatch,
        },
    )
}
pub(super) fn new_request_id() -> Result<RequestId, ()> {
    let mut random = [0; 16];
    SystemRandom::new().fill(&mut random).map_err(|_| ())?;
    let mut text = String::with_capacity(32);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for b in random {
        text.push(HEX[(b >> 4) as usize] as char);
        text.push(HEX[(b & 15) as usize] as char);
    }
    RequestId::new(text).map_err(|_| ())
}

enum Request {
    Discover(DiscoveryRequest),
    Read(ObservationRequest),
    Edit(ScriptEditRequest),
}
pub(super) struct Invocation {
    request: Request,
}
fn take_string(arguments: &mut Map<String, Value>, name: &str) -> Result<String, ()> {
    match arguments.remove(name) {
        Some(Value::String(text)) => Ok(text),
        _ => Err(()),
    }
}
fn checked(
    operation: Operation,
    arguments: Option<Map<String, Value>>,
    id: RequestId,
) -> Result<Invocation, ()> {
    let mut args = arguments.ok_or(())?;
    let project = ProjectRoot::new(take_string(&mut args, "project_root")?).map_err(|_| ())?;
    let session = if args.contains_key("session_id") {
        Some(SessionId::new(take_string(&mut args, "session_id")?).map_err(|_| ())?)
    } else {
        None
    };
    let request = if operation == Operation::Discover {
        Request::Discover(DiscoveryRequest::new(id, project, session))
    } else {
        let path = ResourcePath::new(take_string(&mut args, "script_path")?).map_err(|_| ())?;
        let request = ObservationRequest::new(id, project, session, path);
        if operation == Operation::Read {
            Request::Read(request)
        } else {
            let revision =
                ScriptRevision::new(take_string(&mut args, "revision")?).map_err(|_| ())?;
            let replacement = ExactReplacement::new(
                take_string(&mut args, "old_string")?,
                take_string(&mut args, "new_string")?,
            )
            .map_err(|_| ())?;
            Request::Edit(ScriptEditRequest::exact(request, revision, replacement).map_err(|_| ())?)
        }
    };
    if !args.is_empty() {
        return Err(());
    }
    Ok(Invocation { request })
}
pub(super) fn prepare(
    operation: Operation,
    arguments: Option<Map<String, Value>>,
    request_id: RequestId,
) -> Result<Invocation, CallToolResult> {
    checked(operation, arguments, request_id.clone())
        .map_err(|()| output::failure(operation, &request_id, output::Failure::InvalidArguments))
}
pub(super) fn execute(
    invocation: Invocation,
    registry: &Path,
    clock: AttemptClock,
    cancelled: &AtomicBool,
) -> CallToolResult {
    let (operation, id, revision) = match &invocation.request {
        Request::Discover(request) => (Operation::Discover, request.request_id().clone(), None),
        Request::Read(request) => (Operation::Read, request.request_id().clone(), None),
        Request::Edit(request) => (
            Operation::Edit,
            request.request.request_id().clone(),
            Some(request.revision.as_str().to_owned()),
        ),
    };
    let result = match invocation.request {
        Request::Discover(request) => runner::discovery::run(request, registry, clock, cancelled)
            .and_then(|result| serde_json::to_value(result).map_err(|_| runner::HostFailure)),
        Request::Read(request) => runner::read::run(request, registry, clock, cancelled)
            .and_then(|result| result.encode())
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|_| runner::HostFailure)),
        Request::Edit(request) => runner::read::edit(request, registry, clock, cancelled)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|_| runner::HostFailure)),
    };
    match result {
        Ok(result) => output::complete(operation, &id, result, revision.as_deref()),
        Err(_) => output::failure(operation, &id, output::Failure::HostAfterDispatch),
    }
}
