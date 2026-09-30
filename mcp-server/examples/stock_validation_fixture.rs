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
fn main() {
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
        .take(2_200_000)
        .read_to_string(&mut bytes)
        .expect("fixture JSON input");
    let request: Fixture = serde_json::from_str(&bytes).expect("typed fixture input");
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
