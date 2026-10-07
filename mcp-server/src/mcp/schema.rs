//! Fixed task-oriented catalog. Checked Rust constructors enforce UTF-8 byte bounds.
use super::{handler::Operation, output};
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{json, Map, Value};
use std::sync::{Arc, LazyLock};

pub(super) fn tools() -> Vec<Tool> {
    static TOOLS: LazyLock<Vec<Tool>> = LazyLock::new(|| {
        [Operation::Discover, Operation::Read, Operation::Edit]
            .into_iter()
            .map(tool)
            .collect()
    });
    TOOLS.clone()
}
fn tool(operation: Operation) -> Tool {
    let description = match operation {
        Operation::Discover => "Find supported scripts in a project and report discovery coverage.",
        Operation::Read => "Read script source, current editor state and an edit revision without opening the script.",
        Operation::Edit => "Replace exact text in a script using its current read revision. The script stays open or closed as observed.",
    };
    let mut properties = Map::new();
    properties.insert(
        "project_root".into(),
        json!({"type":"string","minLength":1,"description":"Absolute project directory."}),
    );
    properties.insert("session_id".into(), json!({"type":"string","pattern":"^[0-9a-f]{32}$","description":"Editor session ID; omit only when selection is unambiguous."}));
    let mut required = vec!["project_root"];
    if operation != Operation::Discover {
        properties.insert("script_path".into(), json!({"type":"string","minLength":1,"description":"Exact project script locator, such as res://scripts/player.gd."}));
        required.push("script_path");
    }
    if operation == Operation::Edit {
        properties.insert("revision".into(), json!({"type":"string","pattern":"^sr1:[0-9a-f]{64}$","description":"Revision returned by read_script for this target."}));
        properties.insert(
            "old_string".into(),
            json!({"type":"string","description":"Exact text occurring once, including overlaps. Empty only to replace a completely empty script."}),
        );
        properties.insert(
            "new_string".into(),
            json!({"type":"string","description":"Replacement text; empty deletes the matched text."}),
        );
        required.extend(["revision", "old_string", "new_string"]);
    }
    let input = Map::from_iter([
        (
            "$schema".into(),
            json!("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("type".into(), json!("object")),
        ("additionalProperties".into(), json!(false)),
        ("properties".into(), Value::Object(properties)),
        ("required".into(), json!(required)),
    ]);
    let mut annotations = ToolAnnotations::new()
        .read_only(operation != Operation::Edit)
        .open_world(false);
    if operation == Operation::Edit {
        annotations = annotations.destructive(true).idempotent(false);
    }
    Tool::new(operation.name(), description, input)
        .with_annotations(annotations)
        .with_raw_output_schema(Arc::new(output::schema(operation)))
}
