//! MCP DTO validation and deliberate public projections; no editor safety decisions.
mod discovery;
mod edit;
mod read;

use super::handler::Operation;
use crate::observation::{ProjectRoot, RequestId, ResourcePath, SessionId};
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::{generate::SchemaSettings, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

// Wall endpoints locate acquisition and may move backwards; elapsed_us is monotonic.
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Interval {
    started_unix_ms: u64,
    finished_unix_ms: u64,
    elapsed_us: u64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Target<'a> {
    project_root: &'a str,
    script_path: &'a str,
    session_id: Option<&'a str>,
}
impl Target<'_> {
    fn valid(&self) -> bool {
        valid_selectors(self.project_root, self.session_id, Some(self.script_path))
    }
}
fn valid_selectors(project: &str, session: Option<&str>, path: Option<&str>) -> bool {
    ProjectRoot::new(project).is_ok()
        && session.is_none_or(|s| SessionId::new(s).is_ok())
        && path.is_none_or(|p| ResourcePath::new(p).is_ok())
}
fn hex(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Identity<'a> {
    device: &'a str,
    inode: &'a str,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Stamp<'a> {
    clock_id: &'a str,
    started_tick_us: &'a str,
    finished_tick_us: &'a str,
    received_elapsed_us: u64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Selection<'a> {
    candidate_sessions: Vec<&'a str>,
    missing_selector: &'a str,
}
impl Selection<'_> {
    fn valid(&self) -> bool {
        self.missing_selector == "session_id"
            && self.candidate_sessions.len() <= 64
            && self
                .candidate_sessions
                .iter()
                .all(|s| SessionId::new(*s).is_ok())
    }
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Diagnostic<'a> {
    code: &'a str,
    stage: &'a str,
    surface: Option<&'a str>,
}

#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct Envelope<'a, T> {
    schema_version: u8,
    operation: &'a str,
    request_id: &'a str,
    result: Option<T>,
    error: Option<AdapterError<'a>>,
}
#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(deny_unknown_fields)]
struct AdapterError<'a> {
    category: Category,
    code: ErrorCode,
    stage: ErrorStage,
    application: Application,
    requested_target: Option<Target<'a>>,
    resolved_target: Option<Target<'a>>,
    next_action: NextAction<'a>,
}
#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum Category {
    Input,
    Admission,
    Host,
}
#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum ErrorCode {
    InvalidArguments,
    ServerBusy,
    HostFailure,
    InvalidOutput,
}
#[derive(Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
enum ErrorStage {
    ValidateRequest,
    Admission,
    Execute,
    Deliver,
}
#[derive(Deserialize, Serialize, JsonSchema, Copy, Clone, PartialEq, Eq)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub(super) enum Application {
    NotApplied,
    Applied,
    PartlyApplied,
    Unknown,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    deny_unknown_fields,
    bound(deserialize = "'de: 'a")
)]
enum NextAction<'a> {
    CorrectRequest,
    SelectSession { candidate_sessions: Vec<&'a str> },
    FreshRead,
    CheckSetup,
    Unsupported,
    None,
}

pub(super) enum Failure {
    InvalidArguments,
    Busy,
    HostBeforeDispatch,
    HostAfterDispatch,
    InvalidOutput(Application),
}

fn unavailable_result_guidance(operation: Operation) -> (NextAction<'static>, &'static str) {
    match operation {
        Operation::Discover => (
            NextAction::CheckSetup,
            "Discovery result unavailable. Check setup before trying again.",
        ),
        Operation::Read => (
            NextAction::FreshRead,
            "Read result unavailable. Read the target again.",
        ),
        Operation::Edit => (
            NextAction::FreshRead,
            "Edit result unavailable. Read the original target before another edit.",
        ),
    }
}

pub(super) fn failure(operation: Operation, id: &RequestId, failure: Failure) -> CallToolResult {
    let (category, code, stage, application, next_action, summary) = match failure {
        Failure::InvalidArguments => (
            Category::Input,
            ErrorCode::InvalidArguments,
            ErrorStage::ValidateRequest,
            Application::NotApplied,
            NextAction::CorrectRequest,
            "Invalid arguments. Correct the request.",
        ),
        Failure::Busy => (
            Category::Admission,
            ErrorCode::ServerBusy,
            ErrorStage::Admission,
            Application::NotApplied,
            NextAction::None,
            "Server busy; this call was not queued.",
        ),
        Failure::HostBeforeDispatch => (
            Category::Host,
            ErrorCode::HostFailure,
            ErrorStage::Admission,
            Application::NotApplied,
            NextAction::CheckSetup,
            "Host unavailable before dispatch. Check setup.",
        ),
        Failure::HostAfterDispatch => {
            let (next_action, summary) = unavailable_result_guidance(operation);
            (
                Category::Host,
                ErrorCode::HostFailure,
                ErrorStage::Execute,
                if operation == Operation::Edit {
                    Application::Unknown
                } else {
                    Application::NotApplied
                },
                next_action,
                summary,
            )
        }
        Failure::InvalidOutput(application) => {
            let (next_action, summary) = unavailable_result_guidance(operation);
            (
                Category::Host,
                ErrorCode::InvalidOutput,
                ErrorStage::Deliver,
                application,
                next_action,
                summary,
            )
        }
    };
    let envelope: Envelope<'_, ()> = Envelope {
        schema_version: 2,
        operation: operation.name(),
        request_id: id.as_str(),
        result: None,
        error: Some(AdapterError {
            category,
            code,
            stage,
            application,
            requested_target: None,
            resolved_target: None,
            next_action,
        }),
    };
    // This concrete record contains only checked scalars, enums and null targets.
    let value = json!(envelope);
    carrier(value, true, summary)
}

pub(super) fn complete(
    operation: Operation,
    id: &RequestId,
    mut result: Value,
    revision: Option<&str>,
) -> CallToolResult {
    let retained = if operation == Operation::Edit {
        edit::retained_application(&result)
    } else {
        Application::NotApplied
    };
    let valid = match operation {
        Operation::Discover => discovery::validate(&result, id.as_str()),
        Operation::Read => read::validate(&result),
        Operation::Edit => edit::project(&mut result, revision, id.as_str())
            .and_then(|()| edit::validate(&result, id.as_str())),
    };
    let Ok(is_error) = valid else {
        return failure(operation, id, Failure::InvalidOutput(retained));
    };
    let summary = match (operation, is_error) {
        (Operation::Discover, false) => {
            "Script discovery returned; check coverage before inferring absence."
        }
        (Operation::Read, false) => {
            "Current script state returned; only a non-null revision supplies an edit precondition."
        }
        (Operation::Edit, false) => "Edit verified. The admitted document lifecycle was preserved.",
        (Operation::Edit, true) => edit::guidance(&result),
        (_, true) => "Operation did not succeed. Follow the structured result's next action.",
    };
    let envelope = json!({"schema_version":2,"operation":operation.name(),"request_id":id.as_str(),"result":result,"error":null});
    if !within_limit(&envelope, 16 * 1024 * 1024) {
        return failure(operation, id, Failure::InvalidOutput(retained));
    }
    carrier(envelope, is_error, summary)
}

fn carrier(value: Value, is_error: bool, summary: &str) -> CallToolResult {
    let mut result = CallToolResult::default();
    result.result_type = None;
    result.content = vec![ContentBlock::text(summary)];
    result.structured_content = Some(value);
    result.is_error = Some(is_error);
    result
}
fn within_limit(value: &Value, limit: usize) -> bool {
    struct Count {
        used: usize,
        limit: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.used) {
                return Err(std::io::Error::other("result bound"));
            }
            self.used += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count { used: 0, limit }, value).is_ok()
}
pub(super) fn schema(operation: Operation) -> Map<String, Value> {
    fn make<T: JsonSchema>() -> Map<String, Value> {
        let Value::Object(schema) = Value::from(
            SchemaSettings::draft2020_12()
                .for_serialize()
                .into_generator()
                .into_root_schema_for::<Envelope<'_, T>>(),
        ) else {
            unreachable!("the concrete envelope always generates an object schema");
        };
        schema
    }
    let mut schema = match operation {
        Operation::Discover => make::<discovery::DiscoveryResult<'_>>(),
        Operation::Read => make::<read::ReadResult<'_>>(),
        Operation::Edit => make::<edit::EditResult<'_>>(),
    };
    schema.remove("title");
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.insert("schema_version".into(), json!({"const":2}));
        properties.insert("operation".into(), json!({"const":operation.name()}));
    }
    schema.insert(
        "oneOf".into(),
        json!([
            {"properties":{"result":{"type":"object"},"error":{"type":"null"}}},
            {"properties":{"result":{"type":"null"},"error":{"type":"object"}}}
        ]),
    );
    schema
}

#[cfg(test)]
mod tests;
