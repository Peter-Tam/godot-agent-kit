use ring::digest::{Context, SHA256};
use serde_json::{json, Map, Value};

fn object(value: &mut Value) -> Result<&mut Map<String, Value>, ()> {
    value.as_object_mut().ok_or(())
}
fn remove_context(value: &mut Value) {
    if let Some(record) = value.as_object_mut() {
        for key in [
            "request_id",
            "session_id",
            "document",
            "collection",
            "project_id",
            "file_id",
        ] {
            record.remove(key);
        }
    }
}
fn remove_private_evidence(value: &mut Value) {
    match value {
        Value::Object(record) => {
            record.remove("collection");
            record.remove("witness");
            record.remove("identity");
            for value in record.values_mut() {
                remove_private_evidence(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                remove_private_evidence(value);
            }
        }
        _ => {}
    }
}
fn document_identity(document: &Value, session: &str) -> Result<String, ()> {
    use crate::script_read::field;
    let mut context = Context::new(&SHA256);
    field(&mut context, "domain", b"godot-agent-kit/document/1");
    field(&mut context, "session", session.as_bytes());
    for name in ["kind", "resource_path"] {
        field(
            &mut context,
            name,
            document[name].as_str().ok_or(())?.as_bytes(),
        );
    }
    for (tag, key) in [
        ("script", "script_instance_id"),
        ("editor", "editor_instance_id"),
        ("buffer", "buffer_instance_id"),
    ] {
        let value = document.get(key).ok_or(())?;
        field(
            &mut context,
            tag,
            if value.is_null() {
                b"absent"
            } else {
                b"present"
            },
        );
        if !value.is_null() {
            field(&mut context, "value", value.as_str().ok_or(())?.as_bytes());
        }
    }
    let file = document.get("disk_file_id").ok_or(())?;
    field(
        &mut context,
        "file",
        if file.is_null() {
            b"absent"
        } else {
            b"present"
        },
    );
    if !file.is_null() {
        for name in ["device", "inode"] {
            field(
                &mut context,
                name,
                file[name].as_str().ok_or(())?.as_bytes(),
            );
        }
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(64);
    for byte in context.finish().as_ref() {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 15) as usize] as char);
    }
    Ok(text)
}
fn open(outcome: &mut Map<String, Value>) -> Result<(), ()> {
    outcome.remove("schema_version");
    outcome.remove("operation");
    outcome.remove("expected");
    let session = outcome
        .get("resolved_target")
        .and_then(|t| t["session_id"].as_str())
        .map(str::to_owned);
    if let Some(target) = outcome
        .get_mut("resolved_target")
        .and_then(Value::as_object_mut)
    {
        target
            .retain(|key, _| matches!(key.as_str(), "project_root" | "script_path" | "session_id"));
    }
    for name in ["before", "after"] {
        if let Some(evidence) = outcome.get_mut(name).filter(|v| !v.is_null()) {
            let (identity_key, mut identity) = object(evidence.get_mut("document").ok_or(())?)?
                .remove_entry("identity")
                .ok_or(())?;
            if !identity.is_null() {
                identity =
                    Value::String(document_identity(&identity, session.as_deref().ok_or(())?)?);
            }
            if let Some(saved) = evidence.get_mut("saved_state") {
                remove_context(saved);
            }
            remove_private_evidence(evidence);
            object(evidence.get_mut("document").ok_or(())?)?.insert(identity_key, identity);
        }
    }
    for name in ["persistence", "finalization", "context_recheck"] {
        if let Some(value) = outcome.get_mut(name) {
            remove_context(value);
            remove_private_evidence(value);
        }
    }
    if let Some(values) = outcome.get_mut("validation").and_then(Value::as_array_mut) {
        for value in values {
            remove_context(value);
            remove_private_evidence(value);
        }
    }
    if let Some(progress) = outcome.get_mut("progress") {
        remove_private_evidence(progress);
    }
    Ok(())
}

pub(in crate::mcp::output) fn project(
    result: &mut Value,
    revision: Option<&str>,
    id: &str,
) -> Result<(), ()> {
    let mode = result
        .get("mode")
        .and_then(Value::as_str)
        .ok_or(())?
        .to_owned();
    let outcome = object(result.get_mut("outcome").ok_or(())?)?;
    if outcome.get("request_id").and_then(Value::as_str) != Some(id) {
        return Err(());
    }
    // Never reintroduce source-derived preconditions after the core denies disclosure.
    let permitted = match mode.as_str() {
        "open" => outcome.get("expected").is_some_and(Value::is_object),
        "closed" => outcome.get("evidence").is_some_and(Value::is_object),
        "undetermined" => outcome.get("target").is_some_and(Value::is_object),
        _ => false,
    };
    outcome.insert(
        "revision".into(),
        if permitted {
            json!(revision)
        } else {
            Value::Null
        },
    );
    match mode.as_str() {
        "open" => open(outcome),
        "closed" => {
            let lifecycle = object(outcome.get_mut("lifecycle").ok_or(())?)?;
            let observed = lifecycle.remove("final_state").ok_or(())?;
            let closed = observed.as_str() == Some("closed");
            lifecycle.insert("observed_final".into(), observed);
            if !closed {
                outcome.insert("history".into(), json!("unknown"));
            }
            Ok(())
        }
        "undetermined" => {
            outcome.insert(
                "lifecycle".into(),
                json!({"admitted":null,"observed_final":"unknown"}),
            );
            let kind = outcome
                .get("next_action")
                .and_then(|v| v["kind"].as_str())
                .ok_or(())?;
            let action = match kind {
                "specify_session" => {
                    json!({"kind":"select_session","candidate_sessions":outcome.get("selection").and_then(|v|v.get("candidate_sessions")).ok_or(())?})
                }
                "check_access_and_target" | "check_selected_editor" => {
                    json!({"kind":"check_setup"})
                }
                "fresh_read" => json!({"kind":"fresh_read"}),
                _ => return Err(()),
            };
            outcome.insert("next_action".into(), action);
            Ok(())
        }
        _ => Err(()),
    }
}
