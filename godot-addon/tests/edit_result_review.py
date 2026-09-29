"""Consumer-visible outcome checks, independent of fixture state and expected intent."""
import re

from run_observation import require


_HASH = re.compile(r"[0-9a-f]{64}\Z")
_EFFECTS = ("buffer_application", "resource_sync", "persistence", "finalization")


def review_edit_result(result):
    """Reject contradictory success/certainty evidence; retain facts for human review."""
    outcome, application = result["outcome"], result["application"]
    require(result["operation"] == "edit_open_gdscript" and
            isinstance(result["safe_next_action"], str) and
            bool(result["safe_next_action"].strip()), "result_only_operation_and_next_action")
    require(result["reason"] and result["stage"] and
            result["history"] in ("not_participated", "native_complex_edit", "unknown"),
            "result_only_reason_stage_history")
    require(application in {
        "verified_changed": ("applied",),
        "verified_unchanged": ("not_applied",),
        "refused": ("not_applied",),
        "applied_unverified": ("applied", "partly_applied"),
        "application_unknown": ("unknown",),
    }.get(outcome, ()), "result_only_terminal_application_certainty")
    progress = result["progress"]
    for stage in (*_EFFECTS, "validation", "verification"):
        fact = progress[stage]
        require(fact["state"] in ("not_started", "entered", "completed", "failed", "unknown"),
                "result_only_stage_knowledge_" + stage)
        if fact["state"] == "completed":
            require(fact["collection"] is not None, "result_only_completed_stage_attribution_" + stage)
    evidence = {}
    for phase in ("before", "after"):
        sample = result[phase]
        if sample is None:
            evidence[phase] = None
            continue
        surfaces = {}
        for authority in ("D", "R", "B"):
            surface = sample["sources"][authority]
            require(surface["authority"] == authority, "result_only_source_authority")
            if surface["availability"] == "observed":
                require(_HASH.fullmatch(surface["source_sha256"] or "") is not None and
                        type(surface["utf8_bytes"]) is int and surface["utf8_bytes"] >= 0 and
                        surface["collection"] is not None and surface["identity"] is not None,
                        "result_only_observed_source_attribution")
            else:
                require(surface["source_sha256"] is None and surface["utf8_bytes"] is None and
                        surface["reason"] is not None, "result_only_unavailable_is_not_source")
            surfaces[authority] = {key: surface[key] for key in
                                  ("availability", "reason", "source_sha256", "utf8_bytes")}
        dirty = sample["dirty"]
        require(dirty["availability"] == "observed" or dirty["reason"] is not None,
                "result_only_missing_dirty_reason")
        evidence[phase] = {"sources": surfaces, "dirty": dirty["state"],
                           "dirty_availability": dirty["availability"],
                           "dirty_reason": dirty["reason"], "agreement": sample["agreement"]}
    if outcome in ("verified_changed", "verified_unchanged"):
        target, after = result["resolved_target"], result["after"]
        require(target is not None and result["expected"] is not None and after is not None,
                "result_only_success_target_revision_and_after")
        require(after["document"]["open_state"]["value"] == "open" and
                after["document"]["validity"]["value"] == "valid" and
                after["agreement"] == "agree" and
                all(value == "equal" for value in after["comparisons"].values()) and
                after["dirty"]["availability"] == "observed" and
                after["dirty"]["state"] == "clean" and
                after["consistency"]["checks"] == "performed" and
                not after["consistency"]["detected_changes"],
                "result_only_success_observed_coherence")
        sources = [after["sources"][authority] for authority in ("D", "R", "B")]
        require(all(source["availability"] == "observed" for source in sources) and
                len({(source["source_sha256"], source["utf8_bytes"]) for source in sources}) == 1,
                "result_only_success_exact_sources")
        saved = after["saved_state"]
        require(saved is not None and saved["resource_edited"] is False and
                saved["current_version"] == saved["saved_version"] and
                saved["request_id"] == result["request_id"] and
                saved["session_id"] == target["session_id"], "result_only_success_saved_state")
        purpose = "post_change" if outcome == "verified_changed" else "unchanged"
        validations = [item for item in result["validation"] if item["purpose"] == purpose]
        require(len(validations) == 1, "result_only_success_fresh_validation")
        validation = validations[0]
        require(validation["status"] == "valid" and validation["cleanup_confirmed"] is True and
                validation["context_current"] is True and validation["dependencies_current"] is True and
                validation["request_id"] == result["request_id"] and
                validation["session_id"] == target["session_id"] and
                validation["source_path"] == target["script_path"] and
                validation["input"] == {"sha256": sources[0]["source_sha256"],
                                         "utf8_bytes": sources[0]["utf8_bytes"]} and
                any(source["path"] == target["script_path"] for source in validation["sources"]) and
                all(source["diagnostics_completed"] and source["symbols_completed"]
                    for source in validation["sources"]), "result_only_success_source_validation")
        require(progress["verification"]["state"] == "completed",
                "result_only_success_independent_verification")
        if outcome == "verified_changed":
            require(result["history"] == "native_complex_edit" and
                    all(progress[stage]["state"] == "completed" for stage in _EFFECTS) and
                    result["persistence"] is not None and result["persistence"]["restored"] is True and
                    result["finalization"] is not None and
                    result["finalization"]["status"] == "complete", "result_only_changed_effects")
        else:
            require(result["history"] == "not_participated" and
                    all(progress[stage]["state"] == "not_started" for stage in _EFFECTS) and
                    result["persistence"] is None and result["finalization"] is None,
                    "result_only_unchanged_without_mutation")
    if outcome == "refused":
        require(result["history"] == "not_participated", "result_only_refusal_without_history")
    return {"target": result["resolved_target"] or result["requested_target"],
            "outcome": outcome, "application": application, "reason": result["reason"],
            "stage": result["stage"], "history": result["history"], "evidence": evidence,
            "safe_next_action": result["safe_next_action"]}
