"""Interpret only the public close result, before consulting fixture evidence."""
import re
import run_observation as observation

CLOSE_CODES = {"verified_newly_closed": 0, "already_closed_unchanged": 0,
               "refused": 3, "applied_unverified": 4, "effects_unknown": 4}
KEYS = {"schema_version", "operation", "request_id", "requested_target", "resolved_target",
        "interval", "outcome", "reason", "stage", "application", "expected", "progress",
        "before", "observation", "resource_state", "protection", "selection", "history",
        "diagnostics", "safe_next_action"}
REASONS = set("complete already_closed invalid_request missing_basis invalid_basis revision_changed identity_changed ambiguous_session session_ended session_changed editor_unavailable disconnected denied_access outside_project missing_script invalid_document_kind unsupported_representation unsupported_capability open_state_unknown dirty_conflict resource_edited_conflict source_divergence stale_resource stale_buffer evidence_unavailable unsafe_editor_context context_limit context_parse_invalid context_validation_unavailable target_changed source_changed context_changed busy protocol_error native_failure native_revalidation_unavailable verification_incomplete verification_failed timeout cancelled".split())
STAGES = set("accepted selected inspected prepared validated authorized closing settling verifying terminal".split())


def review_close_result(result):
    require = observation.require
    require(isinstance(result, dict) and set(result) == KEYS, "close_review_exact_required_nullable_keys")
    require(type(result["schema_version"]) is int and result["schema_version"] == 1 and
            result["operation"] == "close_gdscript", "close_review_public_contract")
    outcome = result["outcome"]
    require(outcome in CLOSE_CODES and result["reason"] in REASONS and result["stage"] in STAGES,
            "close_review_closed_vocabularies")
    applications = {"verified_newly_closed": {"applied"}, "already_closed_unchanged": {"not_applied"},
                    "refused": {"not_applied"}, "applied_unverified": {"applied", "partly_applied"},
                    "effects_unknown": {"unknown"}}
    require(result["application"] in applications[outcome], "close_review_effect_knowledge")
    progress = result["progress"]
    require(set(progress) == {"closing", "native_revalidation", "verification"}, "close_review_progress_keys")
    for item in progress.values():
        require(set(item) == {"state", "reason", "collection"} and item["state"] in
                {"not_started", "not_applicable", "entered", "completed", "failed", "unknown"},
                "close_review_progress_meaning")
    resource = result["resource_state"]
    require(set(resource) == {"state", "resource_edited", "reason", "collection"} and
            resource["state"] in {"retained", "unloaded", "unavailable", "invalidated", "not_collected"} and
            (resource["resource_edited"] is None or type(resource["resource_edited"]) is bool),
            "close_review_resource_not_inferred_from_buffer")
    protection = result["protection"]
    require(set(protection) == {"status", "revalidation", "required_count", "completed_count", "reason"} and
            protection["status"] in {"not_applicable", "preserved", "unavailable", "invalidated"} and
            protection["revalidation"] in {"not_applicable", "pending", "completed", "unavailable", "invalidated"},
            "close_review_private_protection_summary")
    for key in ("required_count", "completed_count"):
        require(protection[key] is None or type(protection[key]) is int and 0 <= protection[key] <= 7,
                "close_review_bounded_obligation_count")
    history = result["history"]
    require(set(history) == {"participation", "target_buffer", "unrelated", "reason"} and
            history["participation"] == "not_participated" and history["target_buffer"] in
            {"retained", "disposed", "unavailable", "not_applicable"} and history["unrelated"] in
            {"observed", "unavailable", "invalidated", "not_applicable"}, "close_review_native_history_not_undoable_close")
    selection = result["selection"]
    if selection is not None:
        require(set(selection) == {"before", "after", "request_effect"} and
                all(selection[key] in {"target", "other", "no_source_editor", "unknown"}
                    for key in ("before", "after")) and
                selection["request_effect"] in {"none", "native_fallback", "unknown"}, "close_review_selection_relations")
    expected = result["expected"]
    if expected is not None:
        require(expected["use"] in {"matched", "mismatched", "not_applied_to_closed_state", "unavailable"},
                "close_review_basis_use_not_authority")
    snapshot = result["observation"]
    if snapshot is not None:
        require(set(snapshot) == {"purpose", "interval", "snapshot"} and snapshot["purpose"] in
                {"preparation", "recognition", "verification", "survivor"}, "close_review_observation_purpose")
    require(isinstance(result["diagnostics"], list) and len(result["diagnostics"]) <= 64 and
            all(set(item) == {"stage", "reason", "code"} for item in result["diagnostics"]),
            "close_review_source_free_bounded_diagnostics")
    def source_free(value):
        if isinstance(value, dict):
            require("text" not in value and "source_code" not in value and "receipt_bindings" not in value,
                    "close_review_no_source_or_receipt_bodies")
            for item in value.values(): source_free(item)
        elif isinstance(value, list):
            for item in value: source_free(item)
    source_free(result)
    if outcome == "verified_newly_closed":
        require(result["reason"] == "complete" and result["stage"] in {"verifying", "terminal"} and
                snapshot is not None and snapshot["purpose"] == "verification",
                "close_review_success_requires_fresh_verification")
        _review_verified_closure(result)
    elif outcome == "already_closed_unchanged":
        require(result["reason"] == "already_closed" and snapshot is not None and snapshot["purpose"] == "recognition" and
                progress["closing"]["state"] in {"not_started", "not_applicable"} and
                progress["native_revalidation"]["state"] in {"not_started", "not_applicable"} and
                (expected is None or expected["use"] == "not_applied_to_closed_state"), "close_review_recognition_is_no_effect")
    if outcome in {"applied_unverified", "effects_unknown"}:
        require(isinstance(result["safe_next_action"], str) and "observ" in result["safe_next_action"].lower() and
                not any(word in result["safe_next_action"].lower() for word in ("retry", "rollback")),
                "close_review_possible_effects_require_fresh_observation")
    return {"classification": outcome, "application": result["application"], "basis_use": expected["use"] if expected else None,
            "stage": result["stage"], "progress": {key: value["state"] for key, value in progress.items()},
            "resource": resource["state"], "protection": protection["status"], "history": history["target_buffer"],
            "observation_purpose": snapshot["purpose"] if snapshot else None}


def _review_verified_closure(result):
    require = observation.require
    latest, before = result["observation"], result["before"]
    target, interval = result["resolved_target"], result["interval"]
    require(isinstance(target, dict) and isinstance(before, dict) and
            isinstance(latest.get("snapshot"), dict), "close_review_success_has_acquired_evidence")
    snapshot = latest["snapshot"]
    acquisition = latest["interval"]
    require(snapshot.get("target") == target and
            all(type(interval.get(key)) is int and type(acquisition.get(key)) is int
                for key in ("started_unix_ms", "finished_unix_ms", "elapsed_us")) and
            interval["started_unix_ms"] <= acquisition["started_unix_ms"] <=
            acquisition["finished_unix_ms"] <= interval["finished_unix_ms"] and
            0 <= acquisition["elapsed_us"] <= interval["elapsed_us"] <= 10_000_000,
            "close_review_fresh_acquisition_bound_to_original_target")

    def stamp(value, editor=False):
        require(isinstance(value, dict) and value.get("clock_id") ==
                ("editor:" + target["session_id"] if editor else "caller") and
                all(isinstance(value.get(key), str) and re.fullmatch(r"0|[1-9][0-9]*", value[key])
                    for key in ("started_tick_us", "finished_tick_us")) and
                int(value["started_tick_us"]) <= int(value["finished_tick_us"]) and
                type(value.get("received_elapsed_us")) is int and
                0 <= value["received_elapsed_us"] <= interval["elapsed_us"],
                "close_review_attributed_collection")
        return value["received_elapsed_us"]

    closing = result["progress"]["closing"]
    require(closing["state"] == "completed" and closing["reason"] is None,
            "close_review_actual_completed_close")
    closed_at = stamp(closing["collection"], editor=True)
    for name in ("verification", "native_revalidation"):
        step = result["progress"][name]
        require(step["state"] == "completed" or
                name == "native_revalidation" and step["state"] == "not_applicable",
                "close_review_completed_continuation_and_verification")
        require(step["reason"] is None, "close_review_no_failed_success_stage")
        if name == "native_revalidation" and step["state"] == "completed":
            require(stamp(step["collection"], editor=True) >= closed_at,
                    "close_review_post_close_completed_stage")

    document = snapshot.get("document", {})
    identity = document.get("identity", {})
    for name, value in (("open_state", "not_open"), ("validity", "valid")):
        fact = document.get(name, {})
        require(fact.get("value") == value and fact.get("invalidated_evidence") is None and
                fact.get("reason") is None, "close_review_valid_fresh_closed_document")
        editor_fact = name == "open_state" or fact.get("collection", {}).get("clock_id") == "editor:" + target["session_id"]
        require(stamp(fact.get("collection"), editor=editor_fact) >= closed_at,
                "close_review_document_acquired_after_close")
    require(identity.get("resource_path") == target["script_path"] and
            identity.get("kind") == "external_gdscript" and
            identity.get("editor_instance_id") is None and identity.get("buffer_instance_id") is None,
            "close_review_no_surviving_document_identity")
    consistency = snapshot.get("consistency", {})
    require(consistency.get("checks") == "performed" and consistency.get("detected_changes") == [] and
            consistency.get("recheck_reason") is None and consistency.get("stability") == "unknown" and
            consistency.get("atomic") is False, "close_review_all_checks_without_invalidation")
    sources = snapshot.get("sources", {})
    require(set(sources) == {"D", "R", "B"}, "close_review_independent_authorities_required")

    def observed(authority, source, fresh):
        digest, witness = source.get("digest"), source.get("witness")
        require(source.get("authority") == authority and source.get("availability") == "observed" and
                isinstance(digest, dict) and isinstance(digest.get("sha256"), str) and
                re.fullmatch(r"[0-9a-f]{64}", digest["sha256"]) is not None and
                type(digest.get("utf8_bytes")) is int and digest["utf8_bytes"] >= 0 and
                source.get("invalidated_evidence") is None and source.get("reason") is None and
                isinstance(witness, dict) and witness.get("resource_path") == target["script_path"] and
                isinstance(source.get("staleness"), dict) and source["staleness"].get("state") == "unknown",
                "close_review_real_attributed_source_" + authority)
        received = stamp(source.get("collection"), editor=authority != "D")
        if fresh:
            require(received >= closed_at, "close_review_source_acquired_after_close_" + authority)
        return digest, witness

    disk, disk_witness = observed("D", sources["D"], True)
    prior_disk, prior_disk_witness = observed("D", before.get("sources", {}).get("D", {}), False)
    require(disk == prior_disk and isinstance(disk_witness.get("disk_file_id"), dict) and
            disk_witness["disk_file_id"] == prior_disk_witness.get("disk_file_id"),
            "close_review_unchanged_independently_acquired_disk_and_file")
    if document["validity"]["collection"]["clock_id"] == "caller":
        require(document["validity"]["collection"] == sources["D"]["collection"],
                "close_review_validity_bound_to_independent_disk")
    buffer = sources["B"]
    dirty = snapshot.get("dirty", {})
    require(buffer.get("authority") == "B" and buffer.get("availability") == "not_applicable" and
            buffer.get("digest") is None and buffer.get("invalidated_evidence") is None and
            dirty.get("availability") == "not_applicable" and dirty.get("state") == "not_applicable" and
            dirty.get("invalidated_evidence") is None, "close_review_real_absent_buffer_not_unreadable")
    resource = result["resource_state"]
    require(resource["state"] in {"retained", "unloaded"} and resource["reason"] is None and
            stamp(resource["collection"], editor=True) >= closed_at,
            "close_review_independently_acquired_resource_state")
    if resource["state"] == "retained":
        current, witness = observed("R", sources["R"], True)
        original, original_witness = observed("R", before.get("sources", {}).get("R", {}), False)
        require(current == original == disk and witness.get("script_instance_id") is not None and
                witness["script_instance_id"] == original_witness.get("script_instance_id") ==
                before.get("document", {}).get("identity", {}).get("script_instance_id") ==
                identity.get("script_instance_id") and resource["resource_edited"] is False and
                snapshot.get("comparisons", {}).get("disk_resource") == "equal",
                "close_review_original_retained_resource_unchanged")
    else:
        resource_source = sources["R"]
        require(resource_source.get("authority") == "R" and resource_source.get("availability") == "unavailable" and
                resource_source.get("reason", {}).get("code") == "resource_not_loaded" and
                resource_source.get("digest") is None and resource_source.get("invalidated_evidence") is None and
                identity.get("script_instance_id") is None and resource["resource_edited"] is None,
                "close_review_genuinely_unloaded_resource")
    protection = result["protection"]
    require(protection["status"] == "preserved" and protection["reason"] is None and
            type(protection["required_count"]) is int and
            protection["completed_count"] == protection["required_count"] and
            protection["revalidation"] in {"completed", "not_applicable"} and
            (protection["revalidation"] != "not_applicable" or protection["required_count"] == 0),
            "close_review_complete_native_protection")
    action = result["safe_next_action"]
    require(isinstance(action, str) and all(word in action.lower() for word in ("fresh", "observ", "explicit")) and
            not any(word in action.lower() for word in ("retry", "rollback")),
            "close_review_safe_explicit_next_action")
