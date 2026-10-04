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
