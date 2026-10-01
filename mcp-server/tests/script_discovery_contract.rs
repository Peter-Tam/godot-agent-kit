//! Public terminal boundary checks. Reducer and codec internals are tested privately.
use godot_agent_kit::observation::{ObservationInterval, RequestId};
use godot_agent_kit::script_discovery::DiscoveryOutcome;

#[test]
fn invalid_request_never_discloses_a_target_or_implies_an_empty_project() {
    let outcome = DiscoveryOutcome::invalid_request(
        RequestId::new("safe-correlation").unwrap(),
        ObservationInterval::new(1000, 1001, 100),
    );
    assert_eq!(outcome.exit_code(), 3);
    let bytes = outcome.to_json_line().unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_eq!(bytes.iter().filter(|b| **b == b'\n').count(), 1);
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["outcome"], "refused");
    for key in [
        "requested_target",
        "resolved_target",
        "inventory",
        "selection",
    ] {
        assert!(
            value.as_object().unwrap().contains_key(key),
            "missing required nullable {key}"
        );
        assert!(value[key].is_null(), "invalid request disclosed {key}");
    }
    assert_eq!(value["diagnostics"][0]["code"], "invalid_request");
    assert!(value["diagnostics"][0]["scope"].is_null());
    assert!(value["diagnostics"][0]["omitted_count"].is_null());
}

#[test]
fn host_boundary_failure_is_not_a_successful_empty_inventory() {
    let outcome = DiscoveryOutcome::host_failure(
        RequestId::new("launcher-failure").unwrap(),
        ObservationInterval::new(0, 1, 10),
    );
    let value: serde_json::Value =
        serde_json::from_slice(&outcome.to_json_line().unwrap()).unwrap();
    assert_eq!(value["outcome"], "interrupted");
    assert!(value["inventory"].is_null());
    assert!(value["requested_target"].is_null());
    assert!(value["resolved_target"].is_null());
    assert_eq!(value["diagnostics"][0]["code"], "protocol_error");
    assert!(value["diagnostics"][0]["scope"].is_null());
}
