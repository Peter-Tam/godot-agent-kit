use super::*;

fn id() -> RequestId {
    RequestId::new("result-boundary").unwrap()
}
fn closed(outcome: &str, application: &str) -> Value {
    json!({"mode":"closed","outcome":{
        "request_id":"result-boundary","interval":{"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000},
        "target":{"project_root":"/fixture/project","script_path":"res://target.gd","session_id":"00112233445566778899aabbccddeeff"},
        "outcome":outcome,"application":application,"reason":"complete","stage":"verification",
        "history":"not_applicable_closed","lifecycle":{"admitted":"closed","final_state":"closed"},
        "effects":{"authorized":false,"entry_refused":false,"resource_entered":false,"resource_changed":false,"disk_entered":false,"written_bytes":0,"truncated":false,"flushed":false,"readback":false,"mtime_restored":false},
        "evidence":{"before":{"disk":{"sha256":"a".repeat(64),"utf8_bytes":0},"resource":{"state":"absent","edited":null,"sha256":null,"utf8_bytes":null}},
            "after":{"disk":{"sha256":"a".repeat(64),"utf8_bytes":0},"resource":{"state":"absent","edited":null,"sha256":null,"utf8_bytes":null}},
            "validation":{"original":true,"desired":true,"actual":true},"atomic":false,"independently_verified":true,
            "buffer":"not_applicable","resource":{"admitted":"absent","availability":"not_applicable","state":"absent"}},
        "limitations":[],"next_action":{"kind":"none"}
    }})
}
#[test]
fn invalid_output_after_known_effects_cannot_become_not_applied() {
    for application in ["applied", "partly_applied", "unknown"] {
        let result = complete(
            Operation::Edit,
            &id(),
            json!({"mode":"closed","outcome":{"application":application}}),
            None,
        );
        assert_eq!(result.is_error, Some(true));
        let object = result.structured_content.unwrap();
        assert_eq!(object["result"], Value::Null);
        assert_eq!(object["error"]["code"], "invalid_output");
        assert_eq!(object["error"]["application"], application);
        assert_eq!(object["error"]["next_action"]["kind"], "fresh_read");
    }
}
#[test]
fn contradictory_unchanged_receipt_is_not_published_as_success() {
    let mut result = closed("verified_unchanged", "not_applied");
    result["outcome"]["effects"]["resource_changed"] = json!(true);
    let response = complete(Operation::Edit, &id(), result, None);
    assert_eq!(response.is_error, Some(true));
    let object = response.structured_content.unwrap();
    assert_eq!(object["error"]["code"], "invalid_output");
    assert_eq!(object["error"]["application"], "partly_applied");
}
#[test]
fn uncertain_final_lifecycle_does_not_claim_closed_history_applicability() {
    let mut result = closed("application_unknown", "unknown");
    result["outcome"]["lifecycle"]["final_state"] = json!("unknown");
    result["outcome"]["evidence"] = Value::Null;
    result["outcome"]["next_action"] = json!({"kind":"fresh_read"});
    let response = complete(Operation::Edit, &id(), result, None);
    let object = response.structured_content.unwrap();
    assert_eq!(object["error"], Value::Null);
    assert_eq!(object["result"]["outcome"]["history"], "unknown");
    assert_eq!(
        object["result"]["outcome"]["lifecycle"]["observed_final"],
        "unknown"
    );
}
#[test]
fn closed_noop_has_one_authoritative_object_without_a_duplicate_revision() {
    let revision = format!("sr1:{}", "a".repeat(64));
    let response = complete(
        Operation::Edit,
        &id(),
        closed("verified_unchanged", "not_applied"),
        Some(&revision),
    );
    assert_eq!(response.is_error, Some(false));
    assert_eq!(
        response.structured_content.as_ref().unwrap()["result"]["outcome"]["revision"],
        revision
    );
    assert!(!serde_json::to_string(&response.content)
        .unwrap()
        .contains(&revision));
}

#[test]
fn late_disclosure_denial_does_not_echo_preconditions_after_an_earlier_failure() {
    let revision = format!("sr1:{}", "b".repeat(64));
    let mut result = closed("applied_unverified", "partly_applied");
    result["outcome"]["reason"] = json!("timeout");
    result["outcome"]["target"] = Value::Null;
    result["outcome"]["evidence"] = Value::Null;
    result["outcome"]["lifecycle"]["final_state"] = json!("unknown");
    result["outcome"]["effects"]["resource_changed"] = json!(true);
    result["outcome"]["next_action"] = json!({"kind":"fresh_read"});
    let response = complete(Operation::Edit, &id(), result, Some(&revision));
    let object = response.structured_content.unwrap();
    assert_eq!(object["error"], Value::Null);
    assert_eq!(object["result"]["outcome"]["application"], "partly_applied");
    assert_eq!(object["result"]["outcome"]["revision"], Value::Null);
    assert!(!object.to_string().contains(&revision));
}

fn open_unavailable(outcome: &str, application: &str) -> Value {
    let progress: serde_json::Map<String, Value> = [
        "buffer_application",
        "resource_sync",
        "persistence",
        "finalization",
        "validation",
        "verification",
    ]
    .into_iter()
    .map(|name| {
        (
            name.to_owned(),
            json!({"state":"not_started","reason":null,"collection":null}),
        )
    })
    .collect();
    let sources: serde_json::Map<String, Value> = [
        ("D", "disk_unreadable"),
        ("R", "resource_unreadable"),
        ("B", "buffer_unreadable"),
    ]
    .into_iter()
    .map(|(authority, reason)| {
        (
            authority.to_owned(),
            json!({
                "authority":authority,"availability":"unavailable","source_sha256":null,
                "utf8_bytes":null,"collection":null,"identity":null,"staleness":null,
                "reason":reason,"invalidated_evidence":null,
            }),
        )
    })
    .collect();
    json!({"mode":"open","outcome":{
        "schema_version":1,"operation":"edit_open_script","request_id":"result-boundary",
        "requested_target":{"project_root":"/fixture/project","script_path":"res://target.gd","session_id":"00112233445566778899aabbccddeeff"},
        "resolved_target":{"project_root":"/fixture/project","script_path":"res://target.gd","session_id":"00112233445566778899aabbccddeeff"},
        "interval":{"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000},
        "outcome":outcome,"application":application,"reason":"unavailable_observation","stage":"preflight",
        "progress":progress,"expected":null,
        "before":{
            "document":{"identity":null,
                "validity":{"value":null,"reason":{"code":"document_state_unavailable","action":"Observe again"},"invalidated_evidence":null},
                "open_state":{"value":null,"reason":{"code":"document_state_unavailable","action":"Observe again"},"invalidated_evidence":null}},
            "sources":sources,
            "dirty":{"availability":"unavailable","state":"unknown","reason":{"code":"dirty_state_unavailable","action":"Observe again"},"invalidated_evidence":null},
            "saved_state":null,"saved_state_reason":"unavailable_observation",
            "comparisons":{"disk_resource":"unknown","disk_buffer":"unknown","resource_buffer":"unknown"},
            "agreement":"unknown","consistency":{"atomic":false,"checks":"unavailable","stability":"unknown","detected_changes":[],"recheck_reason":"unavailable"},
            "disk_metadata":null
        },
        "after":null,"persistence":null,"finalization":null,"validation":[],
        "context_recheck":null,"history":"not_participated","diagnostics":[],
        "selection":null,"safe_next_action":"Observe the explicitly selected target again before any new edit"
    }})
}

#[test]
fn unavailable_open_evidence_preserves_refusal_instead_of_invalid_output() {
    let response = complete(
        Operation::Edit,
        &id(),
        open_unavailable("refused", "not_applied"),
        None,
    );
    assert_eq!(response.is_error, Some(true));
    let object = response.structured_content.unwrap();
    assert_eq!(object["error"], Value::Null);
    let outcome = &object["result"]["outcome"];
    assert_eq!(outcome["outcome"], "refused");
    assert_eq!(outcome["reason"], "unavailable_observation");
    assert_eq!(outcome["application"], "not_applied");
    assert_eq!(
        outcome["before"]["document"].get("identity"),
        Some(&Value::Null)
    );
    assert_eq!(
        outcome["before"]["sources"]["B"]["reason"],
        "buffer_unreadable"
    );
    assert!(outcome["before"]["sources"]["B"].get("identity").is_none());
}

#[test]
fn invalidated_open_evidence_retains_effects_and_public_document_identity() {
    let mut result = open_unavailable("applied_unverified", "partly_applied");
    result["outcome"]["stage"] = json!("verifying");
    result["outcome"]["before"]["document"]["identity"] = json!({
        "kind":"standalone","resource_path":"res://target.gd",
        "script_instance_id":"1","editor_instance_id":"2","buffer_instance_id":"3",
        "disk_file_id":{"device":"4","inode":"5"},
    });
    result["outcome"]["after"] = result["outcome"]["before"].take();
    result["outcome"]["progress"]["buffer_application"]["state"] = json!("completed");
    result["outcome"]["after"]["sources"]["B"]["reason"] = json!("source_changed");
    result["outcome"]["after"]["sources"]["B"]["invalidated_evidence"] = json!({
        "source_sha256":"a".repeat(64),"utf8_bytes":51,
        "collection":null,"identity":{"buffer_version":"7"},
        "staleness":{"state":"unknown"},"reason":"source_changed",
    });
    let response = complete(Operation::Edit, &id(), result, None);
    assert_eq!(response.is_error, Some(true));
    let object = response.structured_content.unwrap();
    assert_eq!(object["error"], Value::Null);
    let outcome = &object["result"]["outcome"];
    assert_eq!(outcome["outcome"], "applied_unverified");
    assert_eq!(outcome["application"], "partly_applied");
    assert_eq!(
        outcome["progress"]["buffer_application"]["state"],
        "completed"
    );
    assert!(outcome["after"]["document"]["identity"].is_string());
    let buffer = &outcome["after"]["sources"]["B"];
    assert_eq!(buffer["availability"], "unavailable");
    assert_eq!(buffer["source_sha256"], Value::Null);
    assert_eq!(buffer["invalidated_evidence"]["reason"], "source_changed");
    assert_eq!(
        buffer["invalidated_evidence"]["source_sha256"],
        "a".repeat(64)
    );
    assert!(buffer.get("identity").is_none());
    assert!(buffer["invalidated_evidence"].get("identity").is_none());
}
