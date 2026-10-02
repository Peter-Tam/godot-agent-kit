"""Real T001 close controls and independent assertions, with no public verdict."""
from __future__ import annotations

import copy
import json
import time

import run_observation as observation
from opening_fixture_witness import method_names, property_projection
from run_script_edit import sha

TARGET = "res://scripts/subject.gd"
CURRENT = "res://scripts/other.gd"
BACKGROUND = "res://scripts/close/background.gd"
SAFE = "extends RefCounted\nfunc value() -> int:\n\treturn 47\n"
DOC_KEYS = ("path", "script_id", "editor_id", "buffer_id", "R", "B", "version",
            "saved_version", "dirty", "resource_edited", "has_undo", "has_redo")


def facts(result):
    return result.get("native") or result


def documents(state):
    return {d["path"]: {key: d[key] for key in DOC_KEYS} for d in state["documents"]}


def typed_metadata_size(value):
    """Size E without materializing a second large encoded projection."""
    if value is None:
        return 1
    if type(value) is bool:
        return 2
    if type(value) is str:
        return 5 + len(value.encode("utf-8"))
    if type(value) is int:
        observation.require(0 <= value <= 0xffffffffffffffff, "metadata_unsigned_integer")
        return 9
    if type(value) is list:
        return 5 + sum(typed_metadata_size(item) for item in value)
    observation.require(type(value) is dict, "closed_metadata_encoding_type")
    return 5 + sum(4 + len(key.encode("utf-8")) + typed_metadata_size(item)
                   for key, item in value.items())


def close_metadata_size(context):
    return typed_metadata_size(context["projection"]) + sum(
        typed_metadata_size(record["projection"]) for record in context["documents"])


class CloseNativeAcceptanceMixin:
    def preserved_no_entry(self, attempt, result, *, baseline=None):
        before, old_disks = baseline or (attempt["before"], attempt["disks"])
        after, disks = self.state(attempt["editor"], attempt["project"])
        native = facts(result)
        observation.require(native.get("entered") is not True and
                            documents(before) == documents(after) and old_disks == disks and
                            before["selection"] == after["selection"],
                            "refusal_preserves_actual_roster_D_R_B_flags_versions_history_selection")
        observation.require(result.get("status") in ("refused", "unavailable", "expired", "busy"),
                            "bounded_explicit_no_entry_non_success")
        observation.require(result.get("reason") is not None, "distinct_refusal_has_reason")

    def completed_effect(self, attempt):
        result = self.advance_close(attempt)
        native = facts(result)
        observation.require(result["status"] == "returned" and native["entered"] is True and
                            native["close_error"] == 0 and native["old_document_removed"] is True,
                            "one_actual_native_close_return_and_removal_not_product_success")
        waited = self.settle_close(attempt)
        observation.require(waited["status"] == "completed", "actual_attributable_native_continuation")
        ledger = facts(waited)["continuation"]
        required = set(ledger["required_editor_ids"])
        completed = set(ledger["completed_editor_ids"])
        state = self.close_action(attempt["editor"], "close_state")["state"]
        observed = state["events"]
        surviving = documents(state)
        associations = {d["script_id"]: d["editor_id"] for d in attempt["before"]["documents"]
                        if d["path"] != TARGET and surviving.get(d["path"], {}).get("editor_id") == d["editor_id"] and
                        surviving[d["path"]]["script_id"] == d["script_id"]}
        entry_tick = int(native["entry_collection"]["started_tick_us"])
        latest_visits = {}
        for event in observed:
            if event["event"] == "visit" and int(event["tick_us"]) >= entry_tick:
                editor_id = associations.get(event["script_id"])
                if editor_id is not None:
                    latest_visits[editor_id] = max(latest_visits.get(editor_id, 0), int(event["tick_us"]))
        observation.require(required == completed == set(latest_visits),
                            "exact_independently_visited_surviving_protected_editor_obligations")
        collections = {entry["editor_id"]: entry for entry in ledger["collections"]}
        observation.require(required.issubset(collections), "every_required_editor_has_native_collection")
        for editor_id, latest_visit in latest_visits.items():
            visit, completion = collections[editor_id]["visit"], collections[editor_id]["completion"]
            observation.require(visit is not None and completion is not None and
                                int(completion["started_tick_us"]) >= latest_visit >= entry_tick and
                                int(completion["started_tick_us"]) >= int(visit["started_tick_us"]) >= entry_tick and
                                completion["clock_id"] == "editor:" + attempt["descriptor"]["session_id"] and
                                any(event["event"] == "completion" and event["editor_id"] == editor_id and
                                    int(event["tick_us"]) >= latest_visit for event in observed),
                                "latest_native_completion_follows_latest_independent_relevant_visit")
        verification = self.close_action(attempt["editor"], "close_verify",
                                         request_id=attempt["request_id"], purpose="post_close")["result"]
        observation.require(verification["status"] == "observed", "separate_native_and_ordinary_verification")
        return result, waited

    def positive_close(self, name, **options):
        expected_resource = options.pop("expected_resource", None)
        with self.close_fixture(name, **options) as (project, editor, descriptor):
            before_capture = self.screenshot(editor, name + "-before.png")
            attempt = self.prepare_close(project, editor, descriptor)
            observation.require(attempt["before"]["original_target_alive"],
                                "independent_original_script_liveness_before_close")
            self.authorize_close(attempt)
            result, waited = self.completed_effect(attempt)
            independent = self.independent_close(attempt)
            if expected_resource:
                observation.require(independent["resource"] == expected_resource,
                                    "independent_natural_resource_" + expected_resource)
            self.terminate_close(attempt)
            after_capture = self.screenshot(editor, name + "-after.png")
            self.case(name, operation="private_native_close", public_success_claim=False,
                      native_error=facts(result)["close_error"], continuation=waited["status"],
                      protected_count=len(attempt["authorization"]["results"]),
                      captures=[before_capture, after_capture], **independent)

    def recognition_close(self):
        with self.close_fixture("recognition-empty-editor", paths=[], selected="") as (project, editor, descriptor):
            before, disks = self.state(editor, project)
            request_id, inspected = self.inspect_close(editor, descriptor)
            observation.require(inspected["status"] == "inspected" and
                                inspected["sample"]["document"]["open_state"]["value"] == "not_open",
                                "fresh_ordinary_closed_recognition_no_effect_profile")
            capture = self.rust_close("close_capture", project, descriptor, request_id)
            recheck = self.close_action(editor, "close_recheck", request_id=request_id,
                                        purpose="recognition")["result"]
            verification = self.close_action(editor, "close_verify", request_id=request_id,
                                             purpose="recognition")["result"]
            disk = self.rust_close("close_recheck", project, descriptor, request_id, capture=capture)
            after, now = self.state(editor, project)
            observation.require(recheck["status"] == "rechecked" and verification["status"] == "observed" and
                                disk["unchanged"] and documents(before) == documents(after) and disks == now and
                                not facts(verification)["entered"], "recognition_has_no_validation_or_entry")
            finished = self.close_action(editor, "close_finish", request_id=request_id)["result"]
            observation.require(facts(verification)["protection"]["status"] == "not_applicable" and
                                facts(finished)["protection"]["status"] == "not_applicable",
                                "recognition_finish_never_manufactures_protection")
            self.case("already_closed_empty_editor", operation="private_recognition", validator_children=0)
        with self.close_fixture("inspected-only-finish") as (project, editor, descriptor):
            before, disks = self.state(editor, project)
            request_id, inspected = self.inspect_close(editor, descriptor)
            finished = self.close_action(editor, "close_finish", request_id=request_id)["result"]
            after, now = self.state(editor, project)
            observation.require(facts(inspected)["protection"]["status"] == "not_applicable" and
                                facts(finished)["protection"]["status"] == "not_applicable" and
                                not facts(finished)["entered"] and documents(before) == documents(after) and
                                disks == now and before["selection"] == after["selection"],
                                "inspected_only_finish_preserves_no_admitted_protection")
            self.case("inspected_only_finish_not_applicable", validator_children=0)

    def dirty_target_refusals(self):
        for mutation in ("dirty_different", "dirty_disk_equal", "resource_edited"):
            with self.close_fixture("target-" + mutation) as (project, editor, descriptor):
                self.close_action(editor, "close_human", path=TARGET, mutation=mutation)
                state, _ = self.state(editor, project)
                target = state["target"]
                if mutation == "dirty_different":
                    observation.require(target["dirty"] and target["R"] != target["B"], "real_dirty_target")
                elif mutation == "dirty_disk_equal":
                    observation.require(target["dirty"] and target["B"] ==
                                        (project / "scripts/subject.gd").read_text(),
                                        "equal_disk_text_is_not_clean_evidence")
                else:
                    observation.require(target["resource_edited"], "independent_resource_edited_flag")
                attempt = self.prepare_close(project, editor, descriptor)
                self.preserved_no_entry(attempt, attempt["prepared"])
                verified = self.survivor_close(attempt, attempt["prepared"])
                observation.require(verified["resource_state"]["state"] == "retained" and
                                    verified["resource_edited"] == target["resource_edited"] and
                                    facts(verified)["protection"]["status"] == "not_applicable",
                                    "failed_pre_entry_survivor_fresh_resource_without_admitted_protection")
                self.terminate_close(attempt, "close_abort")
                self.case("target_" + mutation + "_refuses", reason=attempt["prepared"]["reason"],
                          capture=self.screenshot(editor, mutation + "-preserved.png"))

    def basis_refusals(self):
        alterations = {"stale_version": lambda b: b.update(current_version=str(int(b["current_version"]) + 1)),
                       "missing_basis": lambda b: b.clear(),
                       "stale_editor_identity": lambda b: b.update(editor_instance_id="0")}
        for name, alter in alterations.items():
            with self.close_fixture(name) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor, alter_basis=alter)
                self.preserved_no_entry(attempt, attempt["prepared"])
                self.terminate_close(attempt, "close_abort")
                self.case(name + "_refuses", reason=attempt["prepared"]["reason"])

    def profile_refusals(self):
        for source in ("unsafe_tool", "unsafe_export", "unsafe_static", "unsafe_load"):
            def setup(project, source=source):
                (project / "scripts/subject.gd").write_text((project / ("scripts/close/" + source + ".gd")).read_text())
            with self.close_fixture("profile-" + source, setup=setup) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.preserved_no_entry(attempt, attempt["prepared"])
                self.terminate_close(attempt, "close_abort")
                self.case(source + "_effect_profile_refuses", reason=attempt["prepared"]["reason"])
        with self.close_fixture("external-editor") as (project, editor, descriptor):
            self.close_action(editor, "close_config", setting="external_editor", value=True)
            attempt = self.prepare_close(project, editor, descriptor)
            self.preserved_no_entry(attempt, attempt["prepared"])
            self.terminate_close(attempt, "close_abort")
            self.case("unsupported_external_editor_refuses", reason=attempt["prepared"]["reason"])

    def protection_dirty(self):
        for path, name in ((CURRENT, "dirty_current"), (BACKGROUND, "dirty_background")):
            with self.close_fixture(name) as (project, editor, descriptor):
                self.close_action(editor, "close_human", path=path, mutation="select")
                self.close_action(editor, "close_human", path=path, mutation="dirty_equal")
                # This is actual human idle parsing, before any attempt. It is not
                # a preparation retry, a resource setter, or an event substitute.
                self.close_action(editor, "open_setup", paths=[], path=path, idle=True)
                earlier = self.close_action(editor, "close_document", path=path)["document"]
                self.close_action(editor, "close_human", path=path, mutation="undo")
                undone = self.close_action(editor, "close_document", path=path)["document"]
                observation.require(undone["B"] != earlier["B"] and undone["has_redo"],
                                    "real_earlier_human_undo_transition")
                self.close_action(editor, "close_human", path=path, mutation="redo")
                redone = self.close_action(editor, "close_document", path=path)["document"]
                observation.require(redone["B"] == earlier["B"] and redone["has_undo"],
                                    "real_earlier_human_redo_transition")
                self.close_action(editor, "open_setup", paths=[], path=path, idle=True)
                state, disks = self.state(editor, project)
                doc = next(d for d in state["documents"] if d["path"] == path)
                ready = {"dirty": doc["dirty"], "R_equals_B": doc["R"] == doc["B"],
                         "unsaved_source": doc["R"] != disks[path]["text"], "history": doc["has_undo"]}
                if not all(ready.values()):
                    self.summary["failed_protection_setup"] = {
                        **ready, "idle_parse_delay": state["effective_context"]["idle_parse_delay"],
                        "idle_parse_error_delay": state["effective_context"]["idle_parse_error_delay"]}
                observation.require(all(ready.values()), "actual_prior_undo_redo_dirty_protected_R_equals_B")
                if path != CURRENT:
                    self.close_action(editor, "close_human", path=TARGET, mutation="select")
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                self.completed_effect(attempt)
                independent = self.independent_close(attempt)
                self.terminate_close(attempt)
                self.close_action(editor, "close_human", path=path, mutation="select")
                self.close_action(editor, "close_human", path=path, mutation="undo")
                after_undo = self.close_action(editor, "close_document", path=path)["document"]
                observation.require(after_undo["B"] == undone["B"] and after_undo["has_redo"],
                                    "preserved_prior_human_undo_after_real_close")
                self.close_action(editor, "close_human", path=path, mutation="redo")
                after_redo = self.close_action(editor, "close_document", path=path)["document"]
                observation.require(after_redo["B"] == redone["B"] and
                                    after_redo["dirty"] and after_redo["has_undo"],
                                    "preserved_prior_human_redo_after_real_close")
                self.case(name + "_R_equals_B_preserved", prior_undo_redo=True,
                          preserved_undo_redo=True, capture=self.screenshot(editor, name + "-history.png"),
                          **independent)
            with self.close_fixture(name + "-mismatch") as (project, editor, descriptor):
                self.close_action(editor, "close_config", setting="idle_parse_delay", value=8.0)
                self.close_action(editor, "close_human", path=path, mutation="dirty_different")
                state, _ = self.state(editor, project)
                doc = next(d for d in state["documents"] if d["path"] == path)
                observation.require(doc["dirty"] and doc["R"] != doc["B"], "demonstrated_live_R_not_B")
                attempt = self.prepare_close(project, editor, descriptor)
                self.preserved_no_entry(attempt, attempt["prepared"])
                self.terminate_close(attempt, "close_abort")
                self.case(name + "_R_not_B_refuses_before_entry", reason=attempt["prepared"]["reason"],
                          capture=self.screenshot(editor, name + "-mismatch-preserved.png"))

    def bounded_contexts(self):
        for count in (8, 9):
            paths = [TARGET, CURRENT] + ["res://scripts/close/extra_%d.gd" % i for i in range(1, count - 1)]
            with self.close_fixture("roster-limit-" + str(count), paths=paths) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                if count == 9 or attempt["prepared"]["status"] != "prepared":
                    self.preserved_no_entry(attempt, attempt["prepared"])
                    self.terminate_close(attempt, "close_abort")
                else:
                    validation = self.authorize_close(attempt, allow_unavailable=True)
                    if validation["status"] == "valid":
                        self.completed_effect(attempt)
                        self.independent_close(attempt)
                        self.terminate_close(attempt)
                    else:
                        released = self.terminate_close(attempt, "close_abort")
                        after, disks = self.state(editor, project)
                        observation.require(not facts(released).get("entered") and
                                            documents(after) == documents(attempt["before"]) and disks == attempt["disks"],
                                            "bounded_maximum_validation_non_success_has_no_close_effect")
                self.case("exact_roster_bound" if count == 8 else "one_over_roster_bound",
                          document_count=count, preparation=attempt["prepared"]["status"])
        for size in (524288, 524289):
            sizes = (size // 2, size - size // 2)
            def setup(project, sizes=sizes):
                for path, length in zip(("scripts/other.gd", "scripts/close/background.gd"), sizes):
                    text = SAFE + "#" + "x" * (length - len(SAFE.encode()) - 2) + "\n"
                    (project / path).write_text(text)
            with self.close_fixture("source-aggregate-" + str(size), paths=[TARGET, CURRENT, BACKGROUND],
                                    setup=setup) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                protected = [d for d in attempt["before"]["documents"] if d["path"] != TARGET]
                observation.require(len(protected) == 2 and
                                    sum(len(d["R"].encode("utf-8")) for d in protected) == size and
                                    all(d["R"] == d["B"] and len(d["R"].encode("utf-8")) <= 524288
                                        for d in protected), "independent_multi_document_remaining_source_aggregate")
                if size > 524288:
                    self.preserved_no_entry(attempt, attempt["prepared"])
                    observation.require(attempt["prepared"]["reason"] == "protected_source_aggregate_limit",
                                        "aggregate_specific_one_over_refusal_before_entry")
                else:
                    observation.require(attempt["prepared"]["status"] == "prepared",
                                        "exact_multi_document_remaining_source_bound_prepares")
                # No validator/close is needed to prove the preparation bound.
                self.terminate_close(attempt, "close_abort")
                self.case("exact_remaining_source_bound" if size == 524288 else "one_over_remaining_source_bound",
                          source_bytes=size, protected_source_bytes=list(sizes),
                          preparation=attempt["prepared"]["status"])
        for setting in ("idle_parse_delay", "idle_parse_error_delay"):
            with self.close_fixture(setting + "-exceeds-lease") as (project, editor, descriptor):
                self.close_action(editor, "close_config", setting=setting, value=20.0)
                attempt = self.prepare_close(project, editor, descriptor)
                self.preserved_no_entry(attempt, attempt["prepared"])
                self.terminate_close(attempt, "close_abort")
                self.case(setting + "_exceeds_original_lease", reason=attempt["prepared"]["reason"])

    def pre_entry_races(self):
        controls = (
            ("file_same_text_identity", "close_file", {"mutation": "replace_same"}),
            ("protected_file_same_text_identity", "close_file", {"path": CURRENT, "mutation": "replace_same"}),
            ("file_source", "close_file", {"mutation": "source"}),
            ("missing_file", "close_file", {"mutation": "remove"}),
            ("target_same_text_version", "close_human", {"path": TARGET, "mutation": "same_text"}),
            ("target_same_text_replacement", "close_human", {"path": TARGET, "mutation": "replace"}),
            ("protected_same_text_version", "close_human", {"path": CURRENT, "mutation": "same_text"}),
            ("roster_new_document", "close_human", {"path": "res://scripts/close/extra_1.gd", "mutation": "open"}),
            ("roster_removed_document", "close_human", {"path": BACKGROUND, "mutation": "close"}),
            ("roster_same_text_replacement", "close_human", {"path": CURRENT, "mutation": "replace"}),
            ("selected_identity", "close_human", {"path": CURRENT, "mutation": "select"}),
            ("warning_configuration", "close_config", {"setting": "warning", "value": 2}),
            ("idle_delay_configuration", "close_config", {"setting": "idle_parse_delay", "value": 0.8}),
            ("idle_error_delay_configuration", "close_config", {"setting": "idle_parse_error_delay", "value": 0.8}),
            ("sort_configuration", "close_config", {"setting": "sort_scripts", "value": 1}),
            ("autoload_configuration", "close_config", {"setting": "autoload", "value": True}),
            ("external_editor_configuration", "close_config", {"setting": "external_editor", "value": True}),
        )
        for name, action, args in controls:
            with self.close_fixture("race-" + name) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                args = dict(args)
                config = attempt["before"]["effective_context"]
                if action == "close_config":
                    if args["setting"] == "sort_scripts":
                        args["value"] = (int(config["sort_scripts"]) + 1) % 3
                    elif args["setting"] == "warning":
                        args["value"] = (config["warnings"]["levels"]["unused_variable"] + 1) % 3
                    elif args["setting"] in ("idle_parse_delay", "idle_parse_error_delay"):
                        args["value"] = float(config[args["setting"]]) + 0.1
                self.close_action(editor, action, **args)
                baseline = self.state(editor, project)
                result = self.advance_close(attempt)
                self.preserved_no_entry(attempt, result, baseline=baseline)
                self.terminate_close(attempt, "close_abort")
                self.close_action(editor, "close_idle", frames=60)
                after, disks = self.state(editor, project)
                observation.require(after["target"]["associated"] and
                                    after["target"]["buffer_id"] == baseline[0]["target"]["buffer_id"] and
                                    set(after["open_paths"]) == set(baseline[0]["open_paths"]) and disks == baseline[1],
                                    "invalidated_attempt_never_enters_late")
                self.case(name + "_invalidates_before_entry", reason=result["reason"])

    def receipt_boundaries(self):
        for name in ("missing", "duplicate", "wrong_request", "wrong_session", "wrong_target",
                     "wrong_document", "wrong_script", "wrong_editor", "wrong_buffer", "wrong_source",
                     "wrong_length", "wrong_document_guard", "wrong_context", "wrong_guard",
                     "boolean_authority"):
            with self.close_fixture("receipt-" + name, paths=[TARGET, CURRENT]) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                bindings = copy.deepcopy(attempt["authorization"]["receipt_bindings"])
                guard = None
                if name == "missing":
                    bindings.clear()
                elif name == "duplicate":
                    bindings.append(copy.deepcopy(bindings[0]))
                elif name == "wrong_request":
                    bindings[0]["request_id"] = "another-close"
                elif name == "wrong_session":
                    bindings[0]["session_id"] = "0" * 32
                elif name == "wrong_target":
                    bindings[0]["target_path"] = CURRENT
                elif name == "wrong_document":
                    bindings[0]["document_path"] = BACKGROUND
                elif name in ("wrong_script", "wrong_editor", "wrong_buffer"):
                    bindings[0][name.removeprefix("wrong_") + "_id"] = "0"
                elif name == "wrong_source":
                    bindings[0]["source_sha256"] = "0" * 64
                elif name == "wrong_length":
                    bindings[0]["utf8_bytes"] += 1
                elif name == "wrong_document_guard":
                    bindings[0]["guard_sha256"] = "0" * 64
                elif name == "boolean_authority":
                    bindings[0]["valid"] = True
                elif name == "wrong_context":
                    bindings[0]["context_sha256"] = "0" * 64
                else:
                    guard = "0" * 64
                result = self.advance_close(attempt, bindings=bindings, guard=guard)
                self.preserved_no_entry(attempt, result)
                self.terminate_close(attempt, "close_abort")
                self.case("receipt_" + name + "_cannot_authorize", reason=result["reason"])

    def duplicates_overlap_lease(self):
        with self.close_fixture("duplicates-overlap") as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            _, overlap = self.inspect_close(editor, descriptor)
            observation.require(overlap["status"] in ("busy", "refused"), "second_close_cannot_claim_owned_slot")
            result, waited = self.completed_effect(attempt)
            before, disks = self.state(editor, project)
            duplicate = self.advance_close(attempt)
            observation.require(duplicate["status"] == "refused" and facts(duplicate)["entered"] is True,
                                "duplicate_advance_preserves_first_effect_never_replays")
            second_wait = self.settle_close(attempt)
            observation.require(second_wait["status"] in ("refused", "unavailable"), "duplicate_wait_not_second_waiter")
            after, now = self.state(editor, project)
            observation.require(documents(before) == documents(after) and disks == now,
                                "duplicates_no_new_effect")
            self.independent_close(attempt)
            self.terminate_close(attempt)
            self.case("duplicates_and_overlap_exactly_one_close", original_native_error=facts(result)["close_error"])
        with self.close_fixture("original-lease") as (project, editor, descriptor):
            self.close_action(editor, "close_config", setting="idle_parse_delay", value=0.1)
            attempt = self.prepare_close(project, editor, descriptor, budget_us=800000)
            original = attempt["prepared"].get("expiry_tick_us")
            observation.require(attempt["prepared"]["status"] == "prepared" and original is not None,
                                "actual_prepared_attempt_owns_original_finite_expiry")
            self.close_action(editor, "close_idle", frames=90)
            expired = self.close_action(editor, "close_status", request_id=attempt["request_id"])["result"]
            observation.require(expired.get("expiry_tick_us") in (None, original) and
                                not facts(expired).get("entered"), "status_never_renews_original_lease")
            # Real validation is unnecessary after expiry; use no receipts and prove
            # the stale request cannot apply even after a human changes the target.
            self.close_action(editor, "close_human", path=TARGET, mutation="dirty_different")
            baseline = self.state(editor, project)
            result = self.close_action(editor, "close_advance", request_id=attempt["request_id"],
                                       guard_sha256=attempt["prepared"].get("guard_sha256") or "0" * 64,
                                       receipt_bindings=[])["result"]
            self.preserved_no_entry(attempt, result, baseline=baseline)
            self.case("original_expiry_no_renewal_no_late_close", expired=True)
        with self.close_fixture("wrong-session") as (project, editor, descriptor):
            before, disks = self.state(editor, project)
            _, result = self.inspect_close(editor, descriptor, session_id="0" * 32)
            after, now = self.state(editor, project)
            observation.require(result["status"] == "refused" and documents(before) == documents(after) and disks == now,
                                "wrong_session_no_owner_or_entry")
            self.case("session_identity_race_refuses", reason=result["reason"])

    def post_native_independent_failure(self):
        for name, action, args in (
            ("D_identity", "close_file", {"mutation": "replace_same"}),
            ("D_source", "close_file", {"mutation": "source"}),
            ("protected_human_source", "close_human", {"path": CURRENT, "mutation": "dirty_different"}),
            ("reopened_target", "close_human", {"path": TARGET, "mutation": "reopen"}),
            ("protected_replacement", "close_human", {"path": CURRENT, "mutation": "replace"}),
        ):
            with self.close_fixture("post-native-" + name) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                result, waited = self.completed_effect(attempt)
                self.close_action(editor, action, **args)
                self.independent_close(attempt, expect=False)
                before_finish = self.state(editor, project)
                self.terminate_close(attempt, "close_abort")
                after_finish = self.state(editor, project)
                observation.require(documents(before_finish[0]) == documents(after_finish[0]) and
                                    before_finish[1] == after_finish[1], "abort_does_not_compensate_or_restore")
                self.case("native_OK_completed_but_" + name + "_unverified", native_error=facts(result)["close_error"],
                          continuation=waited["status"], independent_verified=False)

        with self.close_fixture("post-native-reopened-new-identity") as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            returned, _ = self.completed_effect(attempt)
            self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
            self.close_action(editor, "close_human", path=TARGET, mutation="replace")
            baseline, disks = self.state(editor, project)
            target = baseline["target"]
            observation.require(target["associated"] and
                                all(target[key] != attempt["before"]["target"][key]
                                    for key in ("script_id", "editor_id", "buffer_id")),
                                "independent_new_reopened_script_editor_buffer_identity")
            verified = self.survivor_close(attempt, returned)
            observation.require(verified["resource_state"]["state"] == "invalidated" and
                                facts(verified)["invalidated"] and
                                facts(verified)["protection"]["status"] == "invalidated",
                                "fresh_replacement_resource_is_not_original_retained_resource")
            self.survivor_close(attempt, verified)
            self.terminate_close(attempt, "close_abort")
            after, now = self.state(editor, project)
            observation.require(documents(baseline) == documents(after) and disks == now,
                                "survivor_and_abort_preserve_new_identity_sources_and_history")
            self.case("reopened_new_identity_survivor_observation_not_authorization", entered=True)

    def human_history_sort(self):
        for sort in (0, 1):
            with self.close_fixture("history-sort-" + str(sort)) as (project, editor, descriptor):
                self.close_action(editor, "close_config", setting="sort_scripts", value=sort)
                for path in (BACKGROUND, CURRENT, TARGET, BACKGROUND, TARGET):
                    self.close_action(editor, "close_human", path=path, mutation="select")
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                returned, waited = self.completed_effect(attempt)
                self.independent_close(attempt)
                observer = self.close_action(editor, "close_state")["state"]["events"]
                script_paths = {d["script_id"]: d["path"] for d in attempt["before"]["documents"]}
                entry_tick = int(facts(returned)["entry_collection"]["started_tick_us"])
                history = [script_paths.get(event["script_id"]) for event in observer
                           if event["event"] == "visit" and int(event["tick_us"]) < entry_tick]
                destinations = {script_paths.get(event["script_id"]) for event in observer
                                if event["event"] == "visit" and int(event["tick_us"]) >= entry_tick}
                observation.require(history == [BACKGROUND, CURRENT, TARGET, BACKGROUND, TARGET] and
                                    destinations and destinations.issubset({CURRENT, BACKGROUND}),
                                    "actual_ordered_human_history_and_protected_post_close_sort_destinations")
                self.terminate_close(attempt)
                self.case("multiple_history_sort_destinations_" + str(sort), sort=sort,
                          continuation=waited["status"], history=history,
                          observed_destinations=sorted(destinations))

    def event_and_lifetime_cases(self):
        # Faults belong exclusively to the separate native fixture artifact.
        # Missing/wrong/premature witnesses must not be replaced by a sleep or
        # by the ordinary observer's independently collected event count.
        for fault in ("drop_completion", "premature_completion", "wrong_completion"):
            with self.close_fixture("event-" + fault, paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
                self.close_action(editor, "close_config", setting="idle_parse_delay", value=0.8)
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                armed = self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault=fault)["result"]
                observation.require(armed["status"] == "ready", "same_request_native_fault_armed")
                result = self.advance_close(attempt)
                observation.require(facts(result)["entered"] is True and facts(result)["close_error"] == 0,
                                    "real_native_effect_for_event_boundary")
                ledger = facts(result)["continuation"]
                observation.require(set(ledger["completed_editor_ids"]).issubset(set(ledger["required_editor_ids"])),
                                    "completion_never_covers_unknown_editor")
                waited = self.settle_close(attempt)
                if fault in ("drop_completion", "wrong_completion"):
                    pending = (set(facts(waited).get("continuation", {}).get("required_editor_ids", [])) -
                               set(facts(waited).get("continuation", {}).get("completed_editor_ids", [])))
                    incomplete = waited["status"] in ("expired", "unavailable", "invalidated")
                    if not incomplete or not pending:
                        self.summary["failed_native_event"] = {
                            "fault": fault, "wait_status": waited["status"], "reason": waited.get("reason"),
                            "native_reason": facts(waited).get("reason"),
                            "phase": facts(waited).get("phase"),
                            "returned_required_count": len(ledger["required_editor_ids"]),
                            "pending_count": len(pending),
                            "continuation": facts(waited).get("continuation", {}).get("state")}
                    observation.require(incomplete and pending,
                                        "missing_misbound_or_renewed_completion_keeps_original_obligation_pending")
                    events = self.close_action(editor, "close_state")["state"]["events"]
                    current = attempt["before"]["current"]
                    observation.require(set(facts(waited)["continuation"]["required_editor_ids"]) ==
                                        {current["editor_id"]} and
                                        not facts(waited)["continuation"]["completed_editor_ids"],
                                        "original_captured_editor_obligation_not_unknown_callback_remains_pending")
                    entry_tick = int(facts(result)["entry_collection"]["started_tick_us"])
                    visits = [int(event["tick_us"]) for event in events if event["event"] == "visit" and
                              event["script_id"] == current["script_id"] and int(event["tick_us"]) >= entry_tick]
                    observation.require(visits, "pending_obligation_has_independent_real_script_visit")
                else:
                    observation.require(waited["status"] == "completed", "only_actual_matching_completion_settles")
                    self.independent_close(attempt)
                self.terminate_close(attempt, "close_abort")
                self.case("native_event_" + fault, native_error=facts(result)["close_error"],
                          continuation=waited["status"], required_count=len(ledger["required_editor_ids"]),
                          public_success_claim=False)
        with self.close_fixture("event-renew-visit", paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            returned, completed = self.completed_effect(attempt)
            baseline, disks = self.state(editor, project)
            current = attempt["before"]["current"]
            renewed = self.close_action(editor, "close_renew_visit", request_id=attempt["request_id"])
            status = self.close_action(editor, "close_status", request_id=attempt["request_id"])["result"]
            verification = self.close_action(editor, "close_verify", request_id=attempt["request_id"],
                                             purpose="post_close")["result"]
            ledger = facts(status)["continuation"]
            events = renewed["state"]["events"]
            latest_visit = max(int(event["tick_us"]) for event in events if event["event"] == "visit" and
                               event["script_id"] == current["script_id"])
            observation.require(completed["status"] == "completed" and ledger["state"] == "pending" and
                                set(ledger["required_editor_ids"]) == {current["editor_id"]} and
                                not ledger["completed_editor_ids"] and verification["status"] != "observed" and
                                not facts(status)["invalidated"] and
                                any(event["event"] == "completion" and event["editor_id"] == current["editor_id"] and
                                    int(facts(returned)["entry_collection"]["started_tick_us"]) <=
                                    int(event["tick_us"]) < latest_visit for event in events),
                                "actual_completion_then_independent_normal_visit_requires_new_matching_completion")
            self.terminate_close(attempt, "close_abort")
            after, now = self.state(editor, project)
            observation.require(documents(baseline) == documents(after) and disks == now,
                                "fixed_renew_visit_has_no_source_selection_or_document_operation")
            self.case("native_event_renew_visit", continuation=ledger["state"], public_success_claim=False)
        with self.close_fixture("replacement-event", paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
            self.close_action(editor, "close_config", setting="idle_parse_delay", value=1.0)
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault="drop_completion")
            result = self.advance_close(attempt)
            self.close_action(editor, "close_human", path=CURRENT, mutation="replace")
            replaced = self.state(editor, project)
            waited = self.settle_close(attempt)
            observation.require(waited["status"] in ("invalidated", "unavailable", "expired"),
                                "replacement_editor_event_never_completes_original_obligation")
            self.independent_close(attempt, expect=False)
            self.terminate_close(attempt, "close_abort")
            observation.require(documents(replaced[0]) == documents(self.state(editor, project)[0]),
                                "replacement_document_preserved_after_terminal_cleanup")
            self.case("replacement_completion_attribution_refuses", continuation=waited["status"])
        for fault in ("missing_signal", "expire_before_entry", "cancel_at_entry", "disable_at_entry", "fail_after_return"):
            with self.close_fixture("fault-" + fault, paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault=fault)
                result = self.advance_close(attempt)
                if fault in ("missing_signal", "expire_before_entry"):
                    self.preserved_no_entry(attempt, result)
                else:
                    observation.require(facts(result)["entered"] is True,
                                        "entered_attempt_retains_effect_facts_through_fault")
                before = self.state(editor, project)
                self.close_action(editor, "close_idle", frames=90)
                after = self.state(editor, project)
                observation.require(documents(before[0]) == documents(after[0]) and before[1] == after[1],
                                    "terminal_fault_no_late_or_repeated_close")
                self.case("native_lifetime_" + fault, entered=facts(result).get("entered"),
                          native_status=result["status"], public_success_claim=False)
        for phase, fault in (("entered", "callback_entry"), ("callback", "callback_completion")):
            callbacks = ("cancel", "finish", "stop", "disable", "compete", "current_typing")
            if phase == "entered":
                callbacks += ("target_typing",)
            for callback in callbacks:
                with self.close_fixture("owned-" + phase + "-" + callback, paths=[TARGET, CURRENT],
                                        faults=True) as (project, editor, descriptor):
                    attempt = self.prepare_close(project, editor, descriptor)
                    self.authorize_close(attempt)
                    self.close_action(editor, "close_arm", callback=callback)
                    self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault=fault)
                    result = self.advance_close(attempt)
                    if phase == "callback":
                        self.settle_close(attempt)
                    witness = self.close_action(editor, "close_callback_witness")["callback"]
                    observation.require(witness.get("count", 0) > 0 and witness["stage"] == phase and
                                        witness["request_id"] == attempt["request_id"] and
                                        witness["owner_reference_alive"] and witness["current_reference_alive"],
                                        "actual_reentrant_callback_retains_live_owner_and_current_document")
                    if callback == "compete":
                        observation.require(witness["result"]["status"] in ("busy", "refused"),
                                            "reentrant_competing_owner_cannot_enter")
                    entered = phase == "callback" or callback == "compete"
                    observation.require(facts(result).get("entered") is entered,
                                        "reentrant_pre_entry_guard_or_actual_post_entry_facts")
                    after, disks = self.state(editor, project)
                    observation.require(after["target"]["associated"] is not entered,
                                        "reentrant_teardown_preserves_exact_close_application_state")
                    if callback == "target_typing":
                        observation.require(after["target"]["dirty"] and
                                            after["target"]["B"] != attempt["before"]["target"]["B"],
                                            "reentrant_target_typing_preserved_and_defeats_entry")
                    if callback == "current_typing":
                        observation.require(after["current"]["dirty"] and
                                            after["current"]["B"] != attempt["before"]["current"]["B"],
                                            "human_callback_change_not_rolled_back")
                    self.close_action(editor, "close_idle", frames=60)
                    later, now = self.state(editor, project)
                    observation.require({p: (d["buffer_id"], d["B"]) for p, d in documents(after).items()} ==
                                        {p: (d["buffer_id"], d["B"]) for p, d in documents(later).items()} and disks == now,
                                        "callback_owner_lifetime_safe_no_late_entry")
                    self.case("reentrant_" + phase + "_" + callback, callback_observed=True,
                              entered=entered, public_success_claim=False)
        for control in ("cancel_owned", "stop", "disable"):
            with self.close_fixture("preauthorization-" + control) as (project, editor, descriptor):
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                self.close_action(editor, "close_lifetime", control=control)
                baseline = self.state(editor, project)
                self.close_action(editor, "close_idle", frames=90)
                after, disks = self.state(editor, project)
                observation.require(documents(baseline[0]) == documents(after) and baseline[1] == disks and
                                    after["target"]["associated"], "lost_disabled_cancelled_attempt_never_closes_late")
                self.case("pre_entry_" + control + "_no_late_close", entered=False)
        for control in ("cancel_owned", "stop", "disable"):
            with self.close_fixture("pending-" + control, paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
                self.close_action(editor, "close_config", setting="idle_parse_delay", value=1.0)
                attempt = self.prepare_close(project, editor, descriptor)
                self.authorize_close(attempt)
                self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault="drop_completion")
                result = self.advance_close(attempt)
                observation.require(facts(result)["entered"] and facts(result)["close_error"] == 0,
                                    "pending_lifetime_loss_after_actual_effect")
                self.close_action(editor, "close_lifetime", control=control)
                self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
                self.close_action(editor, "close_human", path=TARGET, mutation="dirty_different")
                baseline = self.state(editor, project)
                self.close_action(editor, "close_idle", frames=120)
                after, disks = self.state(editor, project)
                observation.require(after["target"]["associated"] and after["target"]["B"] ==
                                    baseline[0]["target"]["B"] and after["target"]["buffer_id"] ==
                                    baseline[0]["target"]["buffer_id"] and baseline[1] == disks,
                                    "old_pending_authority_cannot_close_newer_reopened_buffer")
                self.case("pending_" + control + "_applied_unverified_no_late_close",
                          native_error=0, public_success_claim=False)

    def close_privacy_export(self):
        with self.close_fixture("privacy", source=SAFE + "# CLOSE_PRIVATE_TARGET\n",
                                retained=True) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            self.completed_effect(attempt)
            self.independent_close(attempt)
            self.terminate_close(attempt)
            incidental = json.dumps(self.summary, ensure_ascii=False).encode()
            observation.require(all(marker not in incidental for marker in self.source_markers) and
                                all(secret not in incidental for secret in self.secrets),
                                "close_sources_and_credentials_never_in_summary")
            for descriptor_path in self.registry.rglob("*.json"):
                raw = descriptor_path.read_bytes()
                observation.require(all(marker not in raw for marker in self.source_markers),
                                    "close_private_sources_never_in_discovery_metadata")
            self.case("private_close_sources_not_incidental", protected_sources_logged=False)
        # Existing facility actually produces and runs enabled, disabled, and
        # hook-only exports and inspects both archive and application packs.
        self.native_export()

    def owner_boundaries(self):
        with self.close_fixture("wrong-request-owner", paths=[TARGET, CURRENT]) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            wrong = "wrong" + attempt["request_id"]
            for opcode in ("edit_prepare", "open_begin"):
                stream, peer_id = self.authenticated_peer(descriptor)
                try:
                    request = [5, opcode, peer_id, descriptor["session_id"], descriptor["project_root"], TARGET]
                    if opcode == "open_begin":
                        request.append(9000)
                    else:
                        capture = attempt["capture"]
                        target = attempt["before"]["target"]
                        request.extend([9000, attempt["request_id"], capture["project_device"],
                                        capture["project_inode"], capture["target_device"], capture["target_inode"],
                                        target["script_id"], target["editor_id"], target["buffer_id"],
                                        str(target["version"]), capture["source_sha256"],
                                        capture["source_length"], capture["source"]])
                    stream.sendall(observation.packet(request))
                    reply, _ = observation.receive(stream, 12 * 1024 * 1024)
                    observation.require(reply["status"] in ("busy", "refused") and
                                        reply["reason"] in ("busy", "slot_busy") and
                                        reply.get("sample") is None and reply.get("native") is None,
                                        "actual_" + opcode + "_excluded_by_close_owned_shared_slot")
                finally:
                    stream.close()
                state = self.close_action(editor, "close_state")["state"]
                observation.require(state["slot_busy"] and state["owner_matches"],
                                    "other_operation_cannot_retire_native_close_owner")
            capture = {key: attempt["capture"][key] for key in
                       ("project_device", "project_inode", "target_device", "target_inode", "source_sha256")}
            capture.update(utf8_bytes=int(attempt["capture"]["source_length"]), basis={})
            for action, args in (
                ("close_prepare", dict(captured_source=attempt["capture"]["source"], capture_and_basis=capture)),
                ("close_advance", dict(guard_sha256=attempt["authorization"]["guard_sha256"],
                                       receipt_bindings=attempt["authorization"]["receipt_bindings"])),
                ("close_verify", dict(purpose="post_close")),
                ("close_recheck", dict(purpose="pre_close")),
            ):
                result = self.close_action(editor, action, request_id=wrong, **args)["result"]
                observation.require(result["status"] in ("refused", "unavailable") and
                                    result.get("sample") is None and result.get("context") is None,
                                    "wrong_request_has_no_active_owner_sample_or_protection")
                state = self.close_action(editor, "close_state")["state"]
                observation.require(state["slot_busy"] and state["owner_matches"],
                                    "wrong_request_cannot_retire_actual_owned_attempt")
            self.completed_effect(attempt)
            self.independent_close(attempt)
            self.terminate_close(attempt)
            self.case("wrong_request_all_stages_preserve_real_owner_authority", tested_stages=4)
        with self.close_fixture("preentry-wait", paths=[TARGET, CURRENT]) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            waited = self.settle_close(attempt)
            observation.require(waited["status"] == "unavailable" and not facts(waited).get("entered"),
                                "wait_before_entry_unavailable_and_irrevocably_retires_attempt")
            result = self.advance_close(attempt)
            self.preserved_no_entry(attempt, result)
            state = self.close_action(editor, "close_state")["state"]
            observation.require(not state["slot_busy"], "preentry_wait_releases_only_owned_slot")
            self.case("preentry_wait_retires_no_late_entry", native_entered=False)
        with self.close_fixture("pending-duplicate-wait", paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            self.authorize_close(attempt)
            self.close_action(editor, "close_fault", request_id=attempt["request_id"], fault="drop_completion")
            result = self.advance_close(attempt)
            observation.require(facts(result)["entered"], "pending_duplicate_wait_actual_entered_close")
            expiry = attempt["prepared"]["expiry_tick_us"]
            remaining = (int(expiry) - int(self.close_action(editor, "close_info")["editor_tick_us"])) / 1_000_000
            started = time.monotonic()
            pair = self.close_action(editor, "close_wait_duplicate", request_id=attempt["request_id"])["result"]
            elapsed = time.monotonic() - started
            observation.json_file(self.artifacts / "pending-waiter-witness.json", {
                "first_pending": pair["first_pending"], "original_expiry": expiry,
                "remaining_original_seconds": remaining, "elapsed_seconds": elapsed,
                "first": {key: pair["first"].get(key) for key in ("status", "reason", "expiry_tick_us")},
                "second": {key: pair["second"].get(key) for key in ("status", "reason", "expiry_tick_us")}})
            observation.require(pair["first_pending"] and pair["second"]["reason"] == "duplicate_wait" and
                                pair["first"]["status"] != "completed" and
                                pair["first"].get("continuation", {}).get("state") != "completed" and
                                elapsed <= max(0, remaining) + 0.5 and
                                pair["second"].get("expiry_tick_us") in (None, expiry) and
                                pair["first"].get("expiry_tick_us") in (None, expiry),
                                "pending_duplicate_cannot_attach_waiter_or_extend_original_expiry")
            self.case("one_pending_waiter_original_expiry", first=pair["first"]["status"],
                      duplicate_reason=pair["second"]["reason"], first_pending=True,
                      remaining_original_seconds=remaining, elapsed_seconds=elapsed,
                      native_reason=pair["first"].get("reason"), witness="pending-waiter-witness.json")

    def target_capture_bounds(self):
        for size in (524288, 524289):
            source = SAFE + "#" + "x" * (size - len(SAFE.encode()) - 2) + "\n"
            with self.close_fixture("target-bound-" + str(size), source=source, paths=[TARGET]) as (project, editor, descriptor):
                before, disks = self.state(editor, project)
                capture = self.rust_close("close_capture", project, descriptor, "targetbound")
                if size > 524288:
                    observation.require(capture == {"status": "unavailable", "reason": "close_disk_unavailable"},
                                        "independent_Rust_D_one_over_bound_no_source_or_fallback")
                else:
                    observation.require(int(capture["source_length"]) == size and capture["source"] == source,
                                        "independent_Rust_D_exact_source_bound")
                    attempt = self.prepare_close(project, editor, descriptor)
                    if attempt["prepared"]["status"] == "prepared":
                        self.terminate_close(attempt, "close_abort")
                    else:
                        self.preserved_no_entry(attempt, attempt["prepared"])
                after, now = self.state(editor, project)
                observation.require(documents(before) == documents(after) and disks == now,
                                    "bounded_capture_and_aborted_admission_have_no_effect")
                self.case("exact_target_source_bound" if size == 524288 else "one_over_target_source_bound",
                          source_bytes=size, native_entered=False)
        with self.close_fixture("missing-D-surviving-buffer") as (project, editor, descriptor):
            self.close_action(editor, "close_file", mutation="remove")
            before, disks = self.state(editor, project)
            capture = self.rust_close("close_capture", project, descriptor, "missing")
            observation.require(capture == {"status": "unavailable", "reason": "close_disk_missing"} and
                                before["target"]["associated"], "missing_D_is_not_already_closed_recognition")
            after, now = self.state(editor, project)
            observation.require(documents(before) == documents(after) and disks == now,
                                "missing_disk_capture_no_effect_or_source_fallback")
            self.case("missing_disk_with_live_buffer_refuses", native_entered=False)

    def compiled_metadata_bounds(self, public_callback=None):
        # Calibrate against an actual admitted typed record, then grow ordinary
        # inert source identifiers. A one-byte name change is one byte of E;
        # source hashes retain their fixed width. No forged context authorizes
        # any close, and the exact-bound control checks the actual returned size.
        lengths = [128] * (7 * 120)
        paths = [TARGET] + ["res://scripts/close/extra_%d.gd" % index for index in range(1, 8)]

        def setup(project):
            for index in range(1, 8):
                offset = (index - 1) * 120
                lines = ["extends RefCounted"]
                for number in range(60):
                    name = "p_%02d_" % number + "x" * (lengths[offset + number] - 5)
                    lines.append("var %s: int = %d" % (name, number))
                for number in range(60):
                    name = "m_%02d_" % number + "y" * (lengths[offset + 60 + number] - 5)
                    lines.extend(("func %s() -> int:" % name, "\treturn %d" % number))
                (project / ("scripts/close/extra_%d.gd" % index)).write_text("\n".join(lines) + "\n")

        with self.close_fixture("metadata-bound-0", paths=paths, setup=setup) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            observation.require(attempt["prepared"]["status"] == "prepared", "real_metadata_calibration_admitted")
            delta = 262144 - close_metadata_size(attempt["prepared"]["context"])
            observation.require(0 <= delta < len(lengths) * 128, "reachable_exact_metadata_bound")
            self.terminate_close(attempt, "close_abort")
        for index in range(len(lengths)):
            grow = min(delta, 256 - lengths[index])
            lengths[index] += grow
            delta -= grow
        observation.require(delta == 0 and lengths[-1] < 256, "inert_identifier_growth_within_individual_limits")
        with self.close_fixture("metadata-bound-1", paths=paths, setup=setup) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            observation.require(attempt["prepared"]["status"] == "prepared", "exact_metadata_bound_admitted")
            exact = copy.deepcopy(attempt["prepared"]["context"])
            observation.require(close_metadata_size(exact) == 262144, "actual_exact_256KiB_typed_metadata")
            self.terminate_close(attempt, "close_abort")
            self.case("exact_aggregate_metadata_bound", metadata_bytes=262144, native_entered=False)
            if public_callback is not None:
                public_callback(project, editor, descriptor, 262144)
        lengths[-1] += 1
        with self.close_fixture("metadata-bound-2", paths=paths, setup=setup) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            before = attempt["before"]
            live = {doc["path"]: doc for doc in before["documents"]}
            observation.require(sum(len(doc["R"].encode()) for path, doc in live.items() if path != TARGET) <= 524288,
                                "metadata_limit_not_substituted_by_source_limit")
            # Rebuild just the size facts independently from this editor's real
            # compiled getters and identities; these are never receipts.
            root = project.stat()
            p = exact["projection"]
            p.update(request_id=attempt["request_id"], session_id=descriptor["session_id"],
                     project_root=str(project), project_device=str(root.st_dev), project_inode=str(root.st_ino))
            for record in exact["documents"]:
                d = live[record["projection"]["path"]]
                record["projection"].update(request_id=attempt["request_id"], session_id=descriptor["session_id"],
                    project_root=str(project), project_device=str(root.st_dev), project_inode=str(root.st_ino),
                    source_sha256=sha(d["R"]), source_length=str(len(d["R"].encode())),
                    version=str(d["version"]), saved_version=str(d["saved_version"]),
                    properties=property_projection(d), methods=method_names(d),
                    **{key: d[key] for key in ("script_id", "editor_id", "buffer_id", "dirty", "resource_edited",
                                              "has_undo", "has_redo", "tool", "script_base_id")})
            for entries in (p["roster"], p["protected"]):
                for entry in entries:
                    entry.update({key: live[entry["path"]][key] for key in ("script_id", "editor_id", "buffer_id")})
            target = live[TARGET]
            p["target"].update(target_device=attempt["capture"]["target_device"],
                target_inode=attempt["capture"]["target_inode"], version=str(target["version"]),
                saved_version=str(target["saved_version"]),
                **{key: target[key] for key in ("script_id", "editor_id", "buffer_id", "dirty", "resource_edited")})
            p.update(selected_script_id=target["script_id"], selected_editor_id=target["editor_id"],
                     selected_buffer_id=target["buffer_id"])
            observation.require(close_metadata_size(exact) == 262145, "independent_real_one_over_metadata_size")
            self.preserved_no_entry(attempt, attempt["prepared"])
            observation.require(attempt["prepared"]["reason"] == "close_context_metadata_limit",
                                "one_over_metadata_not_unrelated_per_document_refusal")
            self.terminate_close(attempt, "close_abort")
            self.case("one_over_aggregate_metadata_bound", metadata_bytes=262145, native_entered=False)
            if public_callback is not None:
                public_callback(project, editor, descriptor, 262145)

    def pending_export_and_stale_compiled(self):
        profiles = (
            ("primitive", "extends Node\n@export var payload: int = 0\n\n"),
            ("object", "extends Node\nvar payload: Resource\n\n"),
            ("exported", "extends Node\n@export var payload: Resource\n\n"),
        )
        for name, source in profiles:
            for undo in (False, True):
                def setup(project, source=source):
                    (project / "scripts/other.gd").write_text(source)
                with self.close_fixture("pending-" + name + "-" + str(undo), setup=setup) as (project, editor, descriptor):
                    self.close_action(editor, "close_config", setting="idle_parse_delay", value=8.0)
                    drag = self.open_action(editor, "open_pending_drag", undo=undo)["drag"]
                    observation.require(drag["dragged"]["B"] != drag["before"]["B"] and
                                        drag["dragged"]["has_undo"] and
                                        (not undo or drag["after"]["B"] == drag["before"]["B"]),
                                        "genuine_native_pending_export_drop_and_earlier_undo")
                    attempt = self.prepare_close(project, editor, descriptor)
                    self.preserved_no_entry(attempt, attempt["prepared"])
                    after, _ = self.state(editor, project)
                    observation.require(after["pending_node_payload_id"] ==
                                        attempt["before"]["pending_node_payload_id"],
                                        "native_refusal_does_not_apply_pending_object_property")
                    self.terminate_close(attempt, "close_abort")
                    self.case("pending_export_" + name + ("_undo" if undo else "") + "_refuses",
                              reason=attempt["prepared"]["reason"], native_entered=False)
        def setup(project):
            (project / "scripts/other.gd").write_text("extends Node\n@export var payload: Resource\n\n")
        with self.close_fixture("stale-compiled-export", setup=setup) as (project, editor, descriptor):
            self.close_action(editor, "close_config", setting="idle_parse_delay", value=8.0)
            self.open_action(editor, "open_pending_drag", undo=True)
            self.open_action(editor, "open_mutate", mutation="stale_metadata")
            state, _ = self.state(editor, project)
            observation.require(state["current"]["R"] == state["current"]["B"] and
                                "@export" not in state["current"]["R"] and
                                any(item["name"] == "payload" for item in state["current"]["properties"]),
                                "actual_stale_compiled_export_despite_safe_equal_current_source")
            attempt = self.prepare_close(project, editor, descriptor)
            self.preserved_no_entry(attempt, attempt["prepared"])
            observation.require(attempt["prepared"]["reason"] in
                                ("unsafe_compiled_property", "compiled_property_unavailable",
                                 "compiled_metadata_unavailable_or_limit"),
                                "stale_compiled_effect_refusal_not_source_mismatch")
            after, _ = self.state(editor, project)
            observation.require(after["pending_node_payload_id"] == state["pending_node_payload_id"],
                                "stale_compiled_refusal_preserves_actual_pending_property")
            self.terminate_close(attempt, "close_abort")
            self.case("stale_compiled_pending_export_refuses", reason=attempt["prepared"]["reason"])

    def invalid_remaining_validation(self):
        def setup(project):
            (project / "scripts/other.gd").write_text("extends RefCounted\nfunc value(:\n")
        with self.close_fixture("invalid-protected-source", paths=[TARGET, CURRENT],
                                setup=setup) as (project, editor, descriptor):
            attempt = self.prepare_close(project, editor, descriptor)
            observation.require(attempt["prepared"]["status"] == "prepared",
                                "effect_safe_protected_syntax_error_reaches_real_validator")
            context = attempt["prepared"]["context"]
            result = self.rust_close("close_validate", project, descriptor, attempt["request_id"],
                                     **self.close_validation_fields(attempt), context=context,
                                     guard_sha256=attempt["prepared"]["guard_sha256"],
                                     remaining_budget_ms=min(9000, int((attempt["cutoff"] - time.monotonic()) * 1000)))
            observation.require(result["status"] == "invalid" and not result["receipt_bindings"] and
                                len(result["results"]) == 1 and result["results"][0]["status"] == "invalid" and
                                result["results"][0]["cleanup_confirmed"] and
                                result["results"][0]["child_reaped"] and
                                result["results"][0]["sources"][0]["path"] == CURRENT and
                                result["results"][0]["sources"][0]["diagnostics_completed"] and
                                result["results"][0]["sources"][0]["symbols_completed"],
                                "actual_invalid_close_context_child_fences_cleanup_no_authorization")
            attempt["authorization"] = result
            refused = self.advance_close(attempt, guard=attempt["prepared"]["guard_sha256"], bindings=[])
            self.preserved_no_entry(attempt, refused)
            self.terminate_close(attempt, "close_abort")
            self.case("invalid_actual_protected_validator_refuses_native_entry",
                      actual_child_status="invalid", private_diagnostics_logged=False)

    def native_boundary(self):
        self.compile_window_probe()
        self.group("positives", lambda: (
            self.positive_close("selected", retained=True, expected_resource="retained_original"),
            self.positive_close("nonselected", selected=CURRENT),
            self.positive_close("last_tab", paths=[TARGET], replacement_target=True, expected_resource="unloaded"),
            self.positive_close("stable_two_document", paths=[TARGET, CURRENT]),
            self.positive_close("empty_target", source="", paths=[TARGET, CURRENT]),
            self.positive_close("syntax_invalid_target", source="extends RefCounted\nfunc value(:\n", paths=[TARGET, CURRENT]),
            self.positive_close("read_only_target", read_only=True),
            self.recognition_close()))
        for name, method in (
            ("target-refusals", self.dirty_target_refusals),
            ("basis-refusals", self.basis_refusals),
            ("effect-profile", self.profile_refusals),
            ("pending-export-and-stale-compiled", self.pending_export_and_stale_compiled),
            ("dirty-protected-history", self.protection_dirty),
            ("bounds", self.bounded_contexts),
            ("target-capture-bounds", self.target_capture_bounds),
            ("compiled-metadata-bound", self.compiled_metadata_bounds),
            ("request-owner-boundaries", self.owner_boundaries),
            ("pre-entry-races", self.pre_entry_races),
            ("real-receipt-boundaries", self.receipt_boundaries),
            ("invalid-remaining-validation", self.invalid_remaining_validation),
            ("duplicates-overlap-original-lease", self.duplicates_overlap_lease),
            ("history-sort", self.human_history_sort),
            ("events-callback-lifetime", self.event_and_lifetime_cases),
            ("independent-verification-invalidated", self.post_native_independent_failure),
            ("source-privacy-export", self.close_privacy_export),
        ):
            self.group(name, method)
