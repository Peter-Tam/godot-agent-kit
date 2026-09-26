//! Source-free registry bootstrap. Observation CLI follows when the supervised collector exists.
use std::env;
use std::path::Path;

use godot_agent_kit::project_fs;
use serde_json::json;

fn invalid_request() -> i32 {
    println!(
        "{}",
        json!({"schema_version": 1, "outcome": "invalid_request", "diagnostic": {"code":"invalid_request", "message":"Invalid bootstrap invocation"}})
    );
    3
}
fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let code = match args.as_slice() {
        [arg] if arg == "--help" => {
            println!("observe-gdscript init-registry --registry ABSOLUTE_PATH\nobserve-gdscript --help\nobserve-gdscript --version");
            0
        }
        [arg] if arg == "--version" => {
            println!("observe-gdscript {}", env!("CARGO_PKG_VERSION"));
            0
        }
        [command, flag, path] if command == "init-registry" && flag == "--registry" => {
            match project_fs::init_registry(Path::new(path)) {
                Ok(_) => {
                    println!("{}", json!({"schema_version": 1, "status":"ready"}));
                    0
                }
                Err(error) => {
                    println!(
                        "{}",
                        json!({"schema_version": 1, "outcome":"denied_access", "diagnostic":{"code":"unsafe_registry", "message":error.diagnostic.message(), "action":error.diagnostic.action()}})
                    );
                    3
                }
            }
        }
        _ => invalid_request(),
    };
    std::process::exit(code);
}
