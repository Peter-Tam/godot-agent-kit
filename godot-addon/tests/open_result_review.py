"""Public opening outcome interpretation, independent of fixture intent or state."""
import run_observation as observation

OPEN_FIELDS = {"schema_version", "operation", "request_id", "requested_target", "resolved_target",
               "interval", "outcome", "reason", "stage", "application", "progress", "before",
               "observation", "resource_edited", "target_parse", "protection", "history",
               "selection", "diagnostics", "safe_next_action"}
OPEN_STAGES = ("accepted", "selected", "prepared", "context_validating", "authorized", "binding",
               "compiling", "opening", "verifying", "verified")
OPEN_CODES = {"verified_newly_opened": 0, "already_open_unchanged": 0, "refused": 3,
              "applied_unverified": 4, "effects_unknown": 4}


def review_open_result(result):
    """Interpret only public evidence, before consulting any fixture witness."""
    observation.require(set(result) == OPEN_FIELDS and result["schema_version"] == 1 and
                        result["operation"] == "open_gdscript" and
                        result["outcome"] in OPEN_CODES and result["stage"] in OPEN_STAGES and
                        isinstance(result["request_id"], str) and len(result["request_id"]) == 32 and
                        all(character in "0123456789abcdef" for character in result["request_id"]),
                        "opening_v1_exact_result_and_generated_correlation")
    outcome, application = result["outcome"], result["application"]
    observation.require(isinstance(result["safe_next_action"], str) and
                        result["history"]["participation"] == "not_participated",
                        "opening_result_alone_preservation_and_safe_next_action")
    selection = result["selection"]
    if selection is not None:
        observation.require(set(selection) == {"before", "after", "request_effect"} and
                            selection["before"] in ("target", "other", "no_source_editor", "unknown") and
                            selection["after"] in ("target", "other", "no_source_editor", "unknown") and
                            selection["request_effect"] in ("none", "selected_target", "unknown"),
                            "opening_result_alone_selection_relation_and_request_effect")
    progress = result["progress"]
    observation.require(set(progress) == {"resource_binding", "initial_compilation", "document_open",
                                         "verification"}, "opening_exact_progress_facts")
    for name, fact in progress.items():
        fields = {"state", "reason", "collection", "script_instance_id", "editor_instance_id",
                  "buffer_instance_id"} | ({"mode"} if name == "resource_binding" else set())
        observation.require(set(fact) == fields and fact["state"] in
                            ("not_started", "not_applicable", "entered", "completed", "failed", "unknown"),
                            "opening_bounded_progress_" + name)
    latest = result["observation"]
    snapshot = latest["snapshot"] if latest is not None else None
    interval = result["interval"]
    observation.require(set(interval) == {"started_unix_ms", "finished_unix_ms", "elapsed_us"} and
                        all(type(value) is int and value >= 0 for value in interval.values()) and
                        interval["started_unix_ms"] <= interval["finished_unix_ms"] and
                        interval["elapsed_us"] <= 10_000_000, "opening_result_operation_interval")
    if snapshot is not None:
        observation.require(latest["purpose"] in ("preparation", "recognition", "verification") and
                            snapshot["target"] == result["resolved_target"],
                            "opening_single_latest_exact_target_snapshot")
        acquisition = snapshot["interval"]
        observation.require(set(acquisition) == set(interval) and
                            all(type(value) is int and value >= 0 for value in acquisition.values()) and
                            interval["started_unix_ms"] <= acquisition["started_unix_ms"] <=
                            acquisition["finished_unix_ms"] <= interval["finished_unix_ms"] and
                            acquisition["elapsed_us"] <= interval["elapsed_us"],
                            "opening_result_separate_attributed_acquisition_interval")
        for authority, source in snapshot["sources"].items():
            observation.require(source["authority"] == authority, "opening_independent_authority")
            if source["availability"] == "observed":
                text = source["text"]
                observation.require(isinstance(text, str) and source["collection"] is not None and
                                    source["witness"] is not None,
                                    "opening_actual_empty_or_nonempty_attributed_source")
    if outcome == "verified_newly_opened":
        observation.require(application == "applied" and result["reason"] == "complete" and
                            result["stage"] == "verified" and latest is not None and
                            latest["purpose"] == "verification" and
                            all(source["availability"] == "observed"
                                for source in snapshot["sources"].values()) and
                            len({source["text"] for source in snapshot["sources"].values()}) == 1 and
                            snapshot["document"]["open_state"]["value"] == "open" and
                            snapshot["dirty"]["state"] == "clean" and
                            result["resource_edited"]["availability"] == "observed" and
                            result["resource_edited"]["value"] is False and
                            snapshot["consistency"]["checks"] == "performed" and
                            snapshot["consistency"]["detected_changes"] == [] and
                            result["protection"]["status"] in ("preserved", "not_applicable") and
                            progress["document_open"]["state"] == progress["verification"]["state"] == "completed",
                            "opening_result_alone_new_open_independent_verified_postconditions")
    elif outcome == "already_open_unchanged":
        observation.require(application == "not_applied" and result["reason"] == "already_open" and
                            latest is not None and latest["purpose"] == "recognition" and
                            snapshot["document"]["open_state"]["value"] == "open" and
                            result["stage"] not in ("context_validating", "authorized", "binding",
                                                    "compiling", "opening") and
                            result["target_parse"]["state"] == "not_collected" and
                            result["target_parse"]["origin"] is None and
                            selection is not None and selection["request_effect"] == "none" and
                            all(progress[name]["state"] in ("not_started", "not_applicable")
                                for name in ("resource_binding", "initial_compilation", "document_open")),
                            "opening_result_alone_no_effect_recognition_not_clean_inference")
    elif outcome == "refused":
        observation.require(application == "not_applied" and
                            progress["document_open"]["state"] not in ("entered", "completed") and
                            not (progress["resource_binding"]["mode"] == "created" and
                                 progress["resource_binding"]["state"] == "completed"),
                            "opening_result_alone_irrevocable_no_effect_refusal")
    elif outcome == "applied_unverified":
        observation.require(application in ("applied", "partly_applied") and
                            (progress["resource_binding"]["mode"] == "created" or
                             progress["document_open"]["state"] in ("entered", "completed", "failed", "unknown")),
                            "opening_result_alone_retained_known_lifecycle_effect")
    else:
        observation.require(application == "unknown", "opening_result_alone_authorization_uncertainty")
    return {"target": result["resolved_target"] or result["requested_target"], "outcome": outcome,
            "application": application, "furthest_stage": result["stage"],
            "source_availability": {key: value["availability"] for key, value in
                                    snapshot["sources"].items()} if snapshot else None,
            "buffer_dirty": snapshot["dirty"]["state"] if snapshot else None,
            "target_parse": result["target_parse"]["state"],
            "known_progress": {key: value["state"] for key, value in progress.items()},
            "selection": selection,
            "operation_interval": interval,
            "observation_interval": snapshot["interval"] if snapshot else None,
            "safe_next_action": result["safe_next_action"]}
