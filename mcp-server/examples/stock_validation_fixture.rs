//! Test-only JSON fixture for the private stock-validation worker. Not a product CLI.
use godot_agent_kit::observation::{ProjectRoot, RequestId, ResourcePath, SessionId};
use godot_agent_kit::runner::stock_validation::{
    self, OpeningContext, Purpose, ValidationRequest, WarningSettings,
};
use godot_agent_kit::runner::AttemptClock;
use serde::Deserialize;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    binary: PathBuf,
    project: String,
    script: String,
    source: Option<String>,
    purpose: Purpose,
    request_id: String,
    open_context: Option<OpeningContext>,
    session_id: String,
    warnings: WarningSettings,
    global_classes: Vec<String>,
}

#[derive(Deserialize)]
struct OperationProbe {
    operation: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseValidationFixture {
    operation: CloseValidateOperation,
    binary: PathBuf,
    project: String,
    script: String,
    request_id: String,
    session_id: String,
    warnings: WarningSettings,
    global_classes: Vec<String>,
    remaining_budget_ms: u64,
    context: stock_validation::CloseContext,
    guard_sha256: String,
}

#[derive(Deserialize)]
enum CloseValidateOperation {
    #[serde(rename = "close_validate")]
    Validate,
}

#[derive(Deserialize)]
enum CaptureOperation {
    #[serde(rename = "close_capture")]
    Capture,
}
#[derive(Deserialize)]
enum RecheckOperation {
    #[serde(rename = "close_recheck")]
    Recheck,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureFixture {
    operation: CaptureOperation,
    project: String,
    script: String,
    request_id: String,
    session_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecheckFixture {
    operation: RecheckOperation,
    project: String,
    script: String,
    request_id: String,
    session_id: String,
    capture: stock_validation::TargetCapture,
}
enum CloseFixture {
    Capture(CaptureFixture),
    Recheck(RecheckFixture),
}
fn close_validation_fixture(input: CloseValidationFixture, clock: AttemptClock) {
    let CloseValidationFixture {
        operation: CloseValidateOperation::Validate,
        binary,
        project,
        script,
        request_id,
        session_id,
        warnings,
        global_classes,
        remaining_budget_ms,
        context,
        guard_sha256,
    } = input;
    let remaining_budget_ms = if (1..=9000).contains(&remaining_budget_ms) {
        remaining_budget_ms.saturating_sub(clock.elapsed_us().saturating_add(999) / 1000)
    } else {
        remaining_budget_ms
    };
    let result = stock_validation::validate_close_context(
        context,
        &guard_sha256,
        ValidationRequest {
            request_id: RequestId::new(request_id).expect("request_id"),
            session_id: SessionId::new(session_id).expect("session_id"),
            project_root: ProjectRoot::new(project).expect("project"),
            root_path: ResourcePath::new(script).expect("script"),
            source: None,
            purpose: Purpose::CloseContext,
            open_context: None,
            warnings,
            global_classes,
            official_binary: binary,
        },
        remaining_budget_ms,
    );
    println!(
        "{}",
        serde_json::to_string(&result).expect("close validation result")
    );
}

fn close_fixture(input: CloseFixture) {
    let result = match input {
        CloseFixture::Capture(CaptureFixture {
            operation: CaptureOperation::Capture,
            project,
            script,
            request_id,
            session_id,
        }) => {
            RequestId::new(request_id).expect("request_id");
            SessionId::new(session_id).expect("session_id");
            match stock_validation::capture_close_target(
                &ProjectRoot::new(project).expect("project"),
                &ResourcePath::new(script).expect("script"),
            ) {
                Ok(capture) => serde_json::to_value(capture).expect("capture"),
                Err(reason) => serde_json::json!({"status":"unavailable","reason":reason}),
            }
        }
        CloseFixture::Recheck(RecheckFixture {
            operation: RecheckOperation::Recheck,
            project,
            script,
            request_id,
            session_id,
            capture,
        }) => {
            RequestId::new(request_id).expect("request_id");
            SessionId::new(session_id).expect("session_id");
            match stock_validation::recheck_close_target(
                &ProjectRoot::new(project).expect("project"),
                &ResourcePath::new(script).expect("script"),
                &capture,
            ) {
                Ok((unchanged, capture)) => {
                    serde_json::json!({"unchanged":unchanged,"capture":capture})
                }
                Err(reason) => {
                    serde_json::json!({"unchanged":false,"status":"unavailable","reason":reason})
                }
            }
        }
    };
    println!(
        "{}",
        serde_json::to_string(&result).expect("private close fixture result")
    );
}
fn main() {
    let clock = AttemptClock::start();
    if std::env::args().len() == 2
        && std::env::args().nth(1).as_deref() == Some(stock_validation::INTERNAL_FLAG)
    {
        std::process::exit(stock_validation::worker_main().unwrap_or(1));
    }
    if std::env::args().len() != 1 {
        std::process::exit(2);
    }
    let mut bytes = String::new();
    io::stdin()
        .take(12 * 1024 * 1024 + 1)
        .read_to_string(&mut bytes)
        .expect("fixture JSON input");
    assert!(
        bytes.len() <= 12 * 1024 * 1024,
        "bounded private fixture input"
    );
    // Probe only the tiny discriminator; untagged deserialization would buffer
    // and duplicate every private source body before applying source bounds.
    let operation: OperationProbe = serde_json::from_str(&bytes).expect("fixture operation");
    if operation.operation.as_deref() == Some("close_validate") {
        close_validation_fixture(
            serde_json::from_str(&bytes).expect("typed close validation input"),
            clock,
        );
        return;
    }
    if let Some(operation) = operation.operation {
        let input = match operation.as_str() {
            "close_capture" => CloseFixture::Capture(
                serde_json::from_str(&bytes).expect("typed close capture input"),
            ),
            "close_recheck" => CloseFixture::Recheck(
                serde_json::from_str(&bytes).expect("typed close recheck input"),
            ),
            _ => panic!("unknown fixture operation"),
        };
        close_fixture(input);
        return;
    }
    let request: Fixture = serde_json::from_str(&bytes).expect("typed fixture input");
    assert!(
        request.purpose != Purpose::CloseContext,
        "close requires bound aggregate fixture"
    );
    // Retain only checked source-free attribution while validation consumes the
    // private current-source record; never duplicate or emit its source body.
    let expected_opening = request
        .open_context
        .as_ref()
        .and_then(OpeningContext::checked_binding);
    let input = ValidationRequest {
        request_id: RequestId::new(request.request_id).expect("request_id"),
        session_id: SessionId::new(request.session_id).expect("session_id"),
        project_root: ProjectRoot::new(request.project).expect("project"),
        root_path: ResourcePath::new(request.script).expect("script"),
        source: request.source,
        purpose: request.purpose,
        open_context: request.open_context,
        warnings: request.warnings,
        global_classes: request.global_classes,
        official_binary: request.binary,
    };
    let mut result =
        stock_validation::validate(input, AttemptClock::start(), &AtomicBool::new(false));
    if result.purpose == Purpose::OpenContext
        && result.status == "valid"
        && !expected_opening
            .as_ref()
            .is_some_and(|binding| binding.validated_hashes(&result).is_some())
    {
        result.status = "unavailable".into();
        result.reason = Some("opening_receipt_mismatch".into());
        result.opening_binding = None;
    }
    println!(
        "{}",
        serde_json::to_string(&result).expect("bounded result")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_validation_decodes_bounded_context_before_the_unread_envelope_tail() {
        // An internally tagged enum must buffer through the malformed tail;
        // direct decoding must instead reject this source at its own boundary.
        let input = format!(
            r#"{{"operation":"close_validate","context":{{"documents":[{{"source":"{}","projection":"#,
            "x".repeat(512 * 1024 + 1),
        );
        let error = serde_json::from_str::<CloseValidationFixture>(&input)
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"), "{error}");
    }

    #[test]
    fn capture_recheck_bounds_source_before_the_unread_tail() {
        let input = format!(
            r#"{{"operation":"close_recheck","capture":{{"source":"{}","project_device":"#,
            "x".repeat(512 * 1024 + 1),
        );
        let error = serde_json::from_str::<RecheckFixture>(&input)
            .err()
            .unwrap();
        assert!(error.to_string().contains("close_context_limit"), "{error}");
    }
}
