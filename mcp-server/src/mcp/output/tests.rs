use super::*;

fn id() -> RequestId {
    RequestId::new("result-boundary").unwrap()
}

fn assert_unavailable_result(
    response: CallToolResult,
    operation: Operation,
    code: &str,
    stage: &str,
    application: &str,
) {
    let wire = serde_json::to_value(response).unwrap();
    assert_eq!(wire["isError"], true);
    let next_action = match operation {
        Operation::Discover => "check_setup",
        Operation::Read | Operation::Edit => "fresh_read",
    };
    assert_eq!(
        wire["structuredContent"],
        json!({
            "schema_version":2,"operation":operation.name(),"request_id":id().as_str(),
            "result":null,
            "error":{
                "category":"host","code":code,"stage":stage,"application":application,
                "requested_target":null,"resolved_target":null,
                "next_action":{"kind":next_action},
            },
        })
    );
}

#[test]
fn discovery_host_failure_requires_setup_not_script_recovery() {
    assert_unavailable_result(
        failure(Operation::Discover, &id(), Failure::HostAfterDispatch),
        Operation::Discover,
        "host_failure",
        "execute",
        "not_applied",
    );
}

#[test]
fn read_host_failure_requires_reading_again_without_edit_effects() {
    assert_unavailable_result(
        failure(Operation::Read, &id(), Failure::HostAfterDispatch),
        Operation::Read,
        "host_failure",
        "execute",
        "not_applied",
    );
}

#[test]
fn edit_host_failure_requires_original_target_read_and_retains_uncertainty() {
    assert_unavailable_result(
        failure(Operation::Edit, &id(), Failure::HostAfterDispatch),
        Operation::Edit,
        "host_failure",
        "execute",
        "unknown",
    );
}

#[test]
fn discovery_invalid_output_requires_setup_not_script_recovery() {
    let mut input = limited_discovery();
    input["interval"] = Value::Null;
    assert_unavailable_result(
        complete(Operation::Discover, &id(), input, None),
        Operation::Discover,
        "invalid_output",
        "deliver",
        "not_applied",
    );
}

#[test]
fn read_invalid_output_requires_reading_again_without_edit_effects() {
    let mut input = limited_read();
    input["state"]["interval"] = Value::Null;
    assert_unavailable_result(
        complete(Operation::Read, &id(), input, None),
        Operation::Read,
        "invalid_output",
        "deliver",
        "not_applied",
    );
}

#[test]
fn predispatch_failures_keep_distinct_categories_stages_and_recovery() {
    for operation in [Operation::Discover, Operation::Read, Operation::Edit] {
        for (reason, category, code, stage, next_action) in [
            (
                Failure::InvalidArguments,
                "input",
                "invalid_arguments",
                "validate_request",
                "correct_request",
            ),
            (
                Failure::Busy,
                "admission",
                "server_busy",
                "admission",
                "none",
            ),
            (
                Failure::HostBeforeDispatch,
                "host",
                "host_failure",
                "admission",
                "check_setup",
            ),
        ] {
            let wire = serde_json::to_value(failure(operation, &id(), reason)).unwrap();
            assert_eq!(wire["isError"], true);
            assert_eq!(wire["structuredContent"]["schema_version"], 2);
            assert_eq!(wire["structuredContent"]["result"], Value::Null);
            assert_eq!(
                wire["structuredContent"]["error"],
                json!({
                    "category":category,"code":code,"stage":stage,"application":"not_applied",
                    "requested_target":null,"resolved_target":null,
                    "next_action":{"kind":next_action},
                })
            );
        }
    }
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
        assert_unavailable_result(
            result,
            Operation::Edit,
            "invalid_output",
            "deliver",
            application,
        );
    }
}

#[test]
fn oversized_edit_results_preserve_effect_certainty_and_source_free_recovery() {
    for (outcome, application) in [
        ("verified_changed", "applied"),
        ("applied_unverified", "partly_applied"),
        ("application_unknown", "unknown"),
    ] {
        let mut result = closed(outcome, application);
        result["outcome"]["effects"]["authorized"] = json!(true);
        result["outcome"]["effects"]["disk_entered"] = json!(true);
        if application != "unknown" {
            result["outcome"]["effects"]["written_bytes"] = json!(1);
        }
        if application != "applied" {
            result["outcome"]["next_action"] = json!({"kind":"fresh_read"});
        }
        result["outcome"]["limitations"] =
            json!(["private-output-sentinel".repeat(16 * 1024 * 1024 / 23 + 1)]);
        // Establish that this reaches the size boundary, not an earlier schema
        // rejection. Use the production projection and validator unchanged.
        let mut projected = result.clone();
        edit::project(&mut projected, None, id().as_str()).unwrap();
        edit::validate(&projected, id().as_str()).unwrap();
        let response = complete(Operation::Edit, &id(), result, None);
        let wire = serde_json::to_value(&response).unwrap();
        assert!(!wire.to_string().contains("private-output-sentinel"));
        assert_unavailable_result(
            response,
            Operation::Edit,
            "invalid_output",
            "deliver",
            application,
        );
    }
}

#[test]
fn malformed_effect_receipts_retain_observed_changes_and_authorization_uncertainty() {
    for (field, value, application) in [
        ("resource_changed", json!(true), "partly_applied"),
        ("written_bytes", json!(1), "partly_applied"),
        ("truncated", json!(true), "partly_applied"),
        ("resource_entered", json!(true), "unknown"),
        ("disk_entered", json!(true), "unknown"),
        ("authorized", json!(true), "unknown"),
    ] {
        let mut result = closed("refused", "not_applied");
        result["outcome"]["effects"][field] = value;
        result["outcome"]["private_source"] = json!("effect-receipt-private-source");
        let response = complete(Operation::Edit, &id(), result, None);
        let wire = serde_json::to_value(&response).unwrap();
        assert!(!wire.to_string().contains("effect-receipt-private-source"));
        assert_unavailable_result(
            response,
            Operation::Edit,
            "invalid_output",
            "deliver",
            application,
        );
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

fn limited_discovery() -> Value {
    json!({
        "schema_version":1,"request_id":"result-boundary","outcome":"limited_listing",
        "interval":{"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000},
        "requested_target":{"project_root":"/fixture/project","session_id":null},
        "resolved_target":null,"diagnostics":[],"selection":null,
        "inventory":{
            "scope":{"policy":"project","project_data_directory":".godot","exclusions":[]},
            "entries":["res://target.gd"],
            "collection":{"clock_id":"caller","started_tick_us":"0","finished_tick_us":"1000","received_elapsed_us":1000},
            "coverage":"partial","validity":"observed",
            "consistency":{"recheck":"completed","stability":"unchanged","atomic":false},
            "visited_entries":1,"visited_directories":1
        }
    })
}

fn limited_read() -> Value {
    let mut sources = Map::new();
    for (name, availability, reason) in [
        ("disk", "observed", None),
        (
            "loaded_resource",
            "unavailable",
            Some("resource_unreadable"),
        ),
        ("editor_buffer", "not_applicable", Some("not_open")),
    ] {
        sources.insert(
            name.into(),
            json!({
                "availability":availability,"current":name == "disk",
                "equals_source":if name == "disk" { Some(true) } else { None },
                "text":null,"text_origin":null,"reason":reason,"invalidated":null,"staleness":null
            }),
        );
    }
    json!({
        "source":"extends Node\n","revision":null,
        "state":{
            "status":"limited_observation","target":null,
            "requested_target":{"project_root":"/fixture/project","script_path":"res://target.gd","session_id":null},
            "document":{"lifecycle":"closed","identity":null,"validity":"valid",
                "validity_reason":null,"invalidated_validity":null,"open_state_reason":null,"invalidated_lifecycle":null},
            "source_origin":"disk","sources":sources,"dirty":null,"consistency":null,
            "interval":{"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000},
            "diagnostics":[],"limitations":["resource_unreadable"],
            "revision_unavailable_reason":"unavailable_observation",
            "next_action":"fresh_read","selection":null
        }
    })
}

#[test]
fn wall_clock_rollback_preserves_observations_and_edit_outcomes() {
    let mut open = open_unavailable("verified_changed", "applied");
    open["outcome"]["after"] = open["outcome"]["before"].take();
    open["outcome"]["after"]["document"]["open_state"]["value"] = json!("open");
    open["outcome"]["after"]["document"]["open_state"]["reason"] = Value::Null;
    open["outcome"]["after"]["agreement"] = json!("agree");
    let rollback = json!({"started_unix_ms":2000,"finished_unix_ms":1000,"elapsed_us":1234});
    for (operation, mut input, interval_path, is_error) in [
        (Operation::Discover, limited_discovery(), "/interval", false),
        (Operation::Read, limited_read(), "/state/interval", false),
        (Operation::Edit, open, "/outcome/interval", false),
        (
            Operation::Edit,
            closed("verified_changed", "applied"),
            "/outcome/interval",
            false,
        ),
        (
            Operation::Edit,
            closed("verified_unchanged", "not_applied"),
            "/outcome/interval",
            false,
        ),
        (
            Operation::Edit,
            open_unavailable("refused", "not_applied"),
            "/outcome/interval",
            true,
        ),
    ] {
        *input.pointer_mut(interval_path).unwrap() = rollback.clone();
        let original = input.clone();
        let response = complete(operation, &id(), input, None);
        assert_eq!(response.is_error, Some(is_error));
        let object = response.structured_content.unwrap();
        assert_eq!(object["error"], Value::Null);
        assert_eq!(object["result"].pointer(interval_path), Some(&rollback));
        if operation != Operation::Edit {
            assert_eq!(object["result"], original);
        } else {
            assert_eq!(
                object["result"]["outcome"]["outcome"],
                original["outcome"]["outcome"]
            );
            assert_eq!(
                object["result"]["outcome"]["application"],
                original["outcome"]["application"]
            );
        }
    }
}

#[test]
fn malformed_intervals_still_fail_without_losing_known_effects() {
    let mut intervals = vec![
        json!({"started_unix_ms":1,"finished_unix_ms":2}),
        json!({"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000,"clock":"wall"}),
    ];
    for field in ["started_unix_ms", "finished_unix_ms", "elapsed_us"] {
        for value in [json!(-1), json!(1.5), json!("1000"), Value::Null] {
            let mut interval = json!({"started_unix_ms":1,"finished_unix_ms":2,"elapsed_us":1000});
            interval[field] = value;
            intervals.push(interval);
        }
    }
    for interval in intervals {
        for (operation, mut input, interval_path, application) in [
            (
                Operation::Discover,
                limited_discovery(),
                "/interval",
                "not_applied",
            ),
            (
                Operation::Read,
                limited_read(),
                "/state/interval",
                "not_applied",
            ),
            (
                Operation::Edit,
                closed("verified_changed", "applied"),
                "/outcome/interval",
                "applied",
            ),
        ] {
            *input.pointer_mut(interval_path).unwrap() = interval.clone();
            let response = complete(operation, &id(), input, None);
            assert_eq!(response.is_error, Some(true));
            let object = response.structured_content.unwrap();
            assert_eq!(object["result"], Value::Null);
            assert_eq!(object["error"]["code"], "invalid_output");
            assert_eq!(object["error"]["application"], application);
        }
    }
}

#[test]
fn confirmed_closed_buffer_matches_public_contract_for_both_resource_profiles() {
    for (outcome, application) in [
        ("verified_changed", "applied"),
        ("verified_unchanged", "not_applied"),
    ] {
        for state in ["absent", "present"] {
            let mut input = closed(outcome, application);
            if state == "present" {
                let resource = json!({"state":"present","edited":false,"sha256":"a".repeat(64),"utf8_bytes":0});
                input["outcome"]["evidence"]["before"]["resource"] = resource.clone();
                input["outcome"]["evidence"]["after"]["resource"] = resource;
                input["outcome"]["evidence"]["resource"] =
                    json!({"admitted":"present","availability":"observed","state":"present"});
                input["outcome"]["limitations"] = json!(["loaded_class_not_reloaded"]);
            }
            let response = complete(Operation::Edit, &id(), input, None);
            assert_eq!(response.is_error, Some(false));
            let object = response.structured_content.unwrap();
            assert_eq!(object["error"], Value::Null);
            let outcome = &object["result"]["outcome"];
            assert_eq!(outcome["history"], "not_applicable_closed");
            assert_eq!(outcome["evidence"]["buffer"], "not_applicable_closed");
            assert_eq!(outcome["evidence"]["resource"]["state"], state);
        }
    }
    let schema = Value::Object(schema(Operation::Edit));
    assert_eq!(
        schema["$defs"]["ClosedBuffer"]["enum"],
        json!(["not_applicable_closed", "unavailable"])
    );
}

#[test]
fn unconfirmed_closed_buffer_remains_unavailable_and_rejects_invented_absence() {
    for final_state in ["open", "unknown"] {
        for buffer in ["unavailable", "not_applicable", "invented"] {
            let mut input = closed("application_unknown", "unknown");
            input["outcome"]["lifecycle"]["final_state"] = json!(final_state);
            input["outcome"]["evidence"]["buffer"] = json!(buffer);
            input["outcome"]["evidence"]["after"] = Value::Null;
            input["outcome"]["evidence"]["independently_verified"] = json!(false);
            input["outcome"]["next_action"] = json!({"kind":"fresh_read"});
            let response = complete(Operation::Edit, &id(), input, None);
            assert_eq!(response.is_error, Some(true));
            let object = response.structured_content.unwrap();
            if buffer == "unavailable" {
                assert_eq!(object["error"], Value::Null);
                assert_eq!(
                    object["result"]["outcome"]["evidence"]["buffer"],
                    "unavailable"
                );
                assert_eq!(object["result"]["outcome"]["history"], "unknown");
                assert_eq!(object["result"]["outcome"]["application"], "unknown");
            } else {
                assert_eq!(object["result"], Value::Null);
                assert_eq!(object["error"]["code"], "invalid_output");
                assert_eq!(object["error"]["application"], "unknown");
            }
        }
    }
}

fn open_matching(reason: &str) -> Value {
    let mut input = open_unavailable("refused", "not_applied");
    let outcome = &mut input["outcome"];
    outcome["reason"] = json!(reason);
    outcome["stage"] = json!("matching");
    outcome["safe_next_action"] = json!("fresh_read");
    outcome["expected"] = json!({"private_basis":"PRIVATE_BASIS_SENTINEL"});
    let before = &mut outcome["before"];
    before["document"]["identity"] = json!({
        "kind":"standalone","resource_path":"res://target.gd",
        "script_instance_id":"1","editor_instance_id":"2","buffer_instance_id":"3",
        "disk_file_id":{"device":"4","inode":"5"}
    });
    before["document"]["validity"] =
        json!({"value":"valid","reason":null,"invalidated_evidence":null});
    before["document"]["open_state"] =
        json!({"value":"open","reason":null,"invalidated_evidence":null});
    for authority in ["D", "R", "B"] {
        before["sources"][authority]["availability"] = json!("observed");
        before["sources"][authority]["source_sha256"] = json!("a".repeat(64));
        before["sources"][authority]["utf8_bytes"] = json!(10);
        before["sources"][authority]["reason"] = Value::Null;
    }
    before["dirty"] = json!({
        "availability":"observed","state":"clean","reason":null,"invalidated_evidence":null
    });
    before["agreement"] = json!("agree");
    before["saved_state"] = json!({
        "current_version":"7","saved_version":"7","resource_edited":false,
        "save_profile":"exact","original_preserved":true,"desired_preserved":true
    });
    let after = before.clone();
    outcome["after"] = after;
    outcome["progress"]["validation"]["state"] = json!("completed");
    outcome["progress"]["verification"]["state"] = json!("completed");
    input
}

#[test]
fn serialized_matching_refusals_preserve_evidence_history_and_fresh_read() {
    let revision = format!("sr1:{}", "c".repeat(64));
    for reason in [
        "no_match",
        "ambiguous_match",
        "empty_old_string",
        "invalid_source",
    ] {
        for mode in ["open", "closed"] {
            let input = if mode == "open" {
                open_matching(reason)
            } else {
                let mut input = closed("refused", "not_applied");
                input["outcome"]["reason"] = json!(reason);
                input["outcome"]["stage"] = json!("matching");
                input["outcome"]["next_action"] = json!({"kind":"fresh_read"});
                input
            };
            let response = complete(Operation::Edit, &id(), input, Some(&revision));
            let wire = serde_json::to_value(response).unwrap();
            assert_eq!(wire["isError"], true);
            let envelope = &wire["structuredContent"];
            assert_eq!(envelope["schema_version"], 2);
            assert_eq!(envelope["error"], Value::Null);
            assert_eq!(envelope["result"]["mode"], mode);
            let outcome = &envelope["result"]["outcome"];
            assert_eq!(outcome["outcome"], "refused");
            assert_eq!(outcome["application"], "not_applied");
            assert_eq!(outcome["reason"], reason);
            assert_eq!(outcome["stage"], "matching");
            assert_eq!(outcome["revision"], revision);
            if mode == "open" {
                assert_eq!(outcome["safe_next_action"], "fresh_read");
                assert_eq!(outcome["history"], "not_participated");
                for step in [
                    "buffer_application",
                    "resource_sync",
                    "persistence",
                    "finalization",
                ] {
                    assert_eq!(outcome["progress"][step]["state"], "not_started");
                }
                assert_eq!(outcome["persistence"], Value::Null);
                assert_eq!(outcome["finalization"], Value::Null);
                assert_eq!(outcome["before"], outcome["after"]);
                assert_eq!(outcome["after"]["document"]["open_state"]["value"], "open");
                assert_eq!(outcome["after"]["saved_state"]["resource_edited"], false);
                assert!(outcome["after"]["document"]["identity"].is_string());
                assert!(outcome.get("expected").is_none());
            } else {
                assert_eq!(outcome["next_action"]["kind"], "fresh_read");
                assert_eq!(outcome["history"], "not_applicable_closed");
                assert_eq!(outcome["lifecycle"]["observed_final"], "closed");
                assert_eq!(outcome["evidence"]["before"], outcome["evidence"]["after"]);
                assert_eq!(outcome["effects"]["authorized"], false);
                assert_eq!(outcome["effects"]["written_bytes"], 0);
            }
            assert_eq!(wire["content"].as_array().unwrap().len(), 1);
            let summary = wire["content"][0]["text"].as_str().unwrap();
            assert!(!summary.contains(&revision));
            assert!(!wire.to_string().contains("PRIVATE_BASIS_SENTINEL"));
        }
    }
}

fn assert_invalid_open_matching(input: Value) {
    let mut projected = input.clone();
    edit::project(&mut projected, None, id().as_str()).unwrap();
    // These contradictions must reach semantic validation, not fail decoding.
    edit::OpenOutcome::deserialize(&projected["outcome"]).unwrap();
    assert_eq!(edit::validate(&projected, id().as_str()), Err(()));
    let wire = serde_json::to_value(complete(Operation::Edit, &id(), input, None)).unwrap();
    assert_eq!(wire["isError"], true);
    assert_eq!(wire["structuredContent"]["result"], Value::Null);
    assert_eq!(wire["structuredContent"]["error"]["code"], "invalid_output");
    assert_eq!(
        wire["structuredContent"]["error"]["next_action"]["kind"],
        "fresh_read"
    );
}

#[test]
fn open_matching_refusal_rejects_inconsistent_terminal_facts_and_guidance() {
    for (outcome, application) in [
        ("verified_unchanged", "not_applied"),
        ("verified_changed", "applied"),
        ("applied_unverified", "partly_applied"),
        ("application_unknown", "unknown"),
        ("refused", "applied"),
        ("refused", "partly_applied"),
        ("refused", "unknown"),
    ] {
        let mut input = open_matching("no_match");
        input["outcome"]["outcome"] = json!(outcome);
        input["outcome"]["application"] = json!(application);
        assert_invalid_open_matching(input);
    }
    for (field, value) in [
        ("history", "native_complex_edit"),
        ("history", "unknown"),
        ("history", "not_applicable_closed"),
        ("safe_next_action", "none"),
    ] {
        let mut input = open_matching("no_match");
        input["outcome"][field] = json!(value);
        assert_invalid_open_matching(input);
    }
}

#[test]
fn open_matching_refusal_rejects_started_or_uncertain_effect_steps() {
    for step in [
        "buffer_application",
        "resource_sync",
        "persistence",
        "finalization",
    ] {
        for state in ["entered", "completed", "failed", "unknown"] {
            let mut input = open_matching("no_match");
            input["outcome"]["progress"][step]["state"] = json!(state);
            assert_invalid_open_matching(input);
        }
    }
}

#[test]
fn open_matching_refusal_rejects_receipts_even_without_claimed_effects() {
    for (field, receipt) in [
        (
            "persistence",
            json!({
                "intended":{"sha256":"a".repeat(64),"utf8_bytes":10},
                "write_started":false,"bytes_written":0,"truncated":false,
                "flushed":false,"readback_matches":false,"attached":false,
                "descriptor_open":false,"interference":false,
                "original_mtime":null,"mtime":null,"restore_attempted":false,
                "restore_errno":null,"restored":false,"reason":null
            }),
        ),
        (
            "finalization",
            json!({
                "status":"not_started","reason":null,
                "before_source":null,"after_source":null,
                "before_current":null,"after_current":null,
                "before_saved":null,"after_saved":null,
                "before_resource_edited":null,"after_resource_edited":null,
                "steps":{"resource_edited":false,"saved_version":false}
            }),
        ),
    ] {
        let mut input = open_matching("no_match");
        input["outcome"][field] = receipt;
        assert_invalid_open_matching(input);
    }
}

#[test]
fn matching_projection_cannot_restore_denied_revision_or_private_basis() {
    let revision = format!("sr1:{}", "d".repeat(64));
    for mode in ["open", "closed"] {
        let mut input = if mode == "open" {
            open_matching("no_match")
        } else {
            let mut input = closed("refused", "not_applied");
            input["outcome"]["reason"] = json!("no_match");
            input["outcome"]["stage"] = json!("matching");
            input["outcome"]["next_action"] = json!({"kind":"fresh_read"});
            input
        };
        if mode == "open" {
            input["outcome"]["expected"] = Value::Null;
            input["outcome"]["before"] = Value::Null;
            input["outcome"]["after"] = Value::Null;
            input["outcome"]["resolved_target"] = Value::Null;
        } else {
            input["outcome"]["evidence"] = Value::Null;
            input["outcome"]["target"] = Value::Null;
        }
        let wire =
            serde_json::to_value(complete(Operation::Edit, &id(), input, Some(&revision))).unwrap();
        assert_eq!(wire["isError"], true);
        assert_eq!(wire["structuredContent"]["error"], Value::Null);
        assert_eq!(
            wire["structuredContent"]["result"]["outcome"]["revision"],
            Value::Null
        );
        assert!(!wire.to_string().contains(&revision));
    }
}

#[test]
fn matching_refusal_cannot_hide_effect_uncertainty_or_publish_unchanged_success() {
    for (outcome, application) in [
        ("verified_unchanged", "not_applied"),
        ("application_unknown", "unknown"),
        ("applied_unverified", "partly_applied"),
    ] {
        let mut input = closed(outcome, application);
        input["outcome"]["reason"] = json!("ambiguous_match");
        input["outcome"]["stage"] = json!("matching");
        input["outcome"]["next_action"] = json!({"kind":"fresh_read"});
        let wire = serde_json::to_value(complete(Operation::Edit, &id(), input, None)).unwrap();
        assert_eq!(wire["isError"], true);
        assert_eq!(wire["structuredContent"]["result"], Value::Null);
        assert_eq!(wire["structuredContent"]["error"]["code"], "invalid_output");
        assert_eq!(
            wire["structuredContent"]["error"]["application"],
            application
        );
    }
    let mut input = closed("refused", "not_applied");
    input["outcome"]["reason"] = json!("no_match");
    input["outcome"]["stage"] = json!("matching");
    input["outcome"]["next_action"] = json!({"kind":"fresh_read"});
    input["outcome"]["effects"]["disk_entered"] = json!(true);
    let wire = serde_json::to_value(complete(Operation::Edit, &id(), input, None)).unwrap();
    assert_eq!(wire["structuredContent"]["error"]["application"], "unknown");
}

#[test]
fn all_public_schemas_and_results_use_two_without_reversioning_local_records() {
    for (operation, input) in [
        (Operation::Discover, limited_discovery()),
        (Operation::Read, limited_read()),
        (Operation::Edit, closed("verified_unchanged", "not_applied")),
    ] {
        let advertised = Value::Object(schema(operation));
        assert_eq!(advertised["properties"]["schema_version"]["const"], 2);
        assert_eq!(
            advertised["properties"]["operation"]["const"],
            operation.name()
        );
        let wire = serde_json::to_value(complete(operation, &id(), input, None)).unwrap();
        assert_eq!(wire["structuredContent"]["schema_version"], 2);
        assert_eq!(wire["structuredContent"]["operation"], operation.name());
        assert_eq!(wire["structuredContent"]["error"], Value::Null);
        assert!(wire["structuredContent"]["result"].is_object());
        assert_eq!(wire["content"].as_array().unwrap().len(), 1);
        if operation == Operation::Discover {
            assert_eq!(wire["structuredContent"]["result"]["schema_version"], 1);
        }
    }
}
