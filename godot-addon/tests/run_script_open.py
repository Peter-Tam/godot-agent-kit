#!/usr/bin/env python3
"""Owned real-editor guarded opening caller and retained native-boundary coverage.

Public groups exercise open-gdscript itself with independent disk/editor/history
witnesses, including repeated opening and composed edit durability. The all
campaign runs each public group and the retained native boundary exactly once.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import stat
import subprocess
import tempfile
import signal
import time

import run_observation as observation
from run_script_edit import NativeHarness, STOCK_SHA256, concise_disk, sha
from stock_acceptance import opening_context_fingerprint
from caller_open_acceptance import CallerOpenAcceptanceMixin
from cumulative_open_acceptance import CumulativeOpenAcceptanceMixin
from composed_open_acceptance import ComposedOpenAcceptanceMixin
from open_result_review import OPEN_CODES
from opening_fixture_witness import (BACKGROUND, CURRENT, CURRENT_SOURCE, DECIMAL_FIELDS,
                                     FIXTURE, INVALID_SOURCE, TARGET, TARGET_SOURCE,
                                     method_names, property_projection, source_free_document)

SCENARIOS = ("native-boundary", "new-open", "already-open", "preservation",
             "routing", "interruption", "sequential", "composed", "privacy-export")


def source_free_receipt(receipt):
    keys = ("status", "reason", "request_id", "stage", "next_stage", "mode", "cache_binding",
            "initial_compilation", "document_open", "terminal_discard", "target_parse_code",
            "resource_edited", "selection", "script_id", "editor_id", "buffer_id",
            "target_script_id", "target_editor_id", "target_buffer_id", "protection",
            "script_instance_id", "editor_instance_id", "buffer_instance_id", "entered")
    return {key: receipt[key] for key in keys if key in receipt}


class OpeningHarness(CumulativeOpenAcceptanceMixin, ComposedOpenAcceptanceMixin,
                     CallerOpenAcceptanceMixin, NativeHarness):
    def __init__(self, args, work):
        super().__init__(args, work)
        self.open_processes = []
        self.source_markers.update((b"OPEN_HUMAN_EARLIER", b"OPEN_HUMAN_CONFLICT",
                                    b"OPEN_CURRENT_CHANGED", b"OPEN_RESOURCE_CHANGED",
                                    b"OPEN_CACHE_CHANGED", b"OPEN_LOADER_NOT_CALLED",
                                    b"OPEN_OUTSIDE_PRIVATE_SOURCE", b"OPEN_UNPERMITTED_EFFECT_EXECUTED",
                                    b"OPEN_SAME_BASENAME_PRIVATE", b"OPEN_NEWER_DISK_TEXT",
                                    b"OPEN_TRANSIENT_NATIVE_HISTORY"))
        self.summary.update({"coverage_scope": ("complete_opening_groups" if args.scenario == "all"
                             else "selected_opening_group"),
                             "product_opening_caller_acceptance": args.scenario != "native-boundary",
                             "public_open_gdscript": True,
                             "opener_sha256": observation.digest(args.opener) if args.opener else None,
                             "support_claim": False,
                             "partial_cache_witness": "independent_ordinary_consumer_retains_actual_cached_ref",
                             "driver_sha256": observation.digest(Path(__file__)),
                             "fixture_driver_sha256": observation.digest(FIXTURE / "fixture_driver.gd"),
                             "opening_harness_modules_sha256": {
                                 name: observation.digest(Path(__file__).parent / name)
                                 for name in ("caller_open_acceptance.py", "open_result_review.py",
                                              "opening_fixture_witness.py", "cumulative_open_acceptance.py",
                                              "composed_open_acceptance.py")},
                             "opening_fixture_files": {
                                 str(path.relative_to(FIXTURE)): observation.digest(path)
                                 for path in sorted(FIXTURE.rglob("*")) if path.is_file()}})
        self.summary["acceptance_coverage"] = {
            "new-open": ["US1.1", "US1.2", "US1.4", "US1.5", "focused_native_history_and_durability"],
            "already-open": ["US2.1", "US2.2", "independent_D_R_B_limits"],
            "preservation": ["US2.3", "US2.4", "US2.5", "dirty_current_R_equals_B",
                             "current_R_differs_B", "pending_drag", "stale_compiled_metadata",
                             "post_open_human_current_and_background_selection"],
            "routing": ["US1.3", "US3.1", "US3.2", "US3.3", "US3.4", "US3.5", "actual_shared_slot_busy"],
            "interruption": ["US4.1", "US4.2", "US4.3", "native_entered_stall_loss_cancel_disable",
                             "actual_open_worker_and_parent_owned_helper_failure",
                             "independent_descendants_process_group_and_private_staging_cleanup"],
            "sequential": ["US4.4", "SC-005", "current_identity_and_native_history",
                           "five_genuinely_closed_successes", "five_dirty_recognitions"],
            "composed": ["US1.4", "US4.5", "SC-006", "product_open_before_fresh_observe_edit",
                         "A_E_Save_reopen_cache_reparse_rescan_runtime",
                         "read_only_observation_and_closed_edit_refusal"],
            "privacy-export": ["selected_current_unrelated_other_project_sentinels",
                               "normal_denied_ambiguous_interrupted", "enabled_disabled_hook_only_actual_exports"]}

    def cleanup(self):
        for process in self.open_processes:
            if process.poll() is None:
                process.send_signal(signal.SIGTERM)
            try:
                process.communicate(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=2)
        self.open_processes.clear()
        super().cleanup()

    def safe_log(self, name, payload):
        if name.startswith("editor-") and name.endswith(".log"):
            # Engine parse output may include private current-source snippets.
            # Discard raw bytes; preserve provenance without incidental disclosure.
            payload = json.dumps({"raw_engine_output_discarded": True,
                                  "sha256": hashlib.sha256(payload).hexdigest(),
                                  "utf8_bytes": len(payload)}).encode()
        super().safe_log(name, payload)

    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        if name.startswith("export-"):
            # Keep ordinary gameplay importable; cold gdignore is editor-only.
            return project
        helper = project / "addons/fixture_driver"
        shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "open_fixture_driver.gd")
        (helper / "plugin.gd").write_text(
            '@tool\nextends "res://addons/fixture_driver/open_fixture_driver.gd"\n')
        scripts = project / "scripts/open"
        shutil.copytree(FIXTURE / "scripts", scripts)
        # Global and autoload sources are inert controls, not enabled by default.
        (scripts / "global.gd").rename(scripts / "global.gd.disabled")
        (project / "scripts/subject.gd").write_text(TARGET_SOURCE)
        (project / "scripts/other.gd").write_text(CURRENT_SOURCE)
        # Keep cold sources out of automatic editor/LSP indexing. Current and
        # background documents are acquired explicitly by owned preparation.
        (project / "scripts/.gdignore").touch()
        return project

    def open_action(self, editor, action, *, timeout=35, **arguments):
        identity = secrets.token_hex(16)
        observation.json_file(editor["control"] / "request.json",
                              {"id": identity, "action": action, **arguments})
        def response():
            path = editor["control"] / "response.json"
            if path.is_file():
                result = json.loads(path.read_text())
                if result.get("id") == identity:
                    return result
            observation.require(editor["process"].poll() is None,
                                "owned_open_editor_exited_" + action)
            return None
        result = observation.wait_for(response, "owned_open_action_" + action, timeout)
        if result.get("ok") is not True:
            self.summary["failed_fixture_response"] = {
                key: result.get(key) for key in
                ("id", "action", "ok", "open_installed", "api_revision", "fixture_faults")}
        observation.require(result.get("ok") is True, "owned_open_action_refused_" + action)
        if action == "open_fault":
            observation.require(result["result"].get("status") == "ready" and
                                result["result"].get("request_id") == arguments["request_id"],
                                "actual_fixture_fault_armed_on_same_live_native_attempt")
        # All raw witnesses/context remain in private request-local memory/workdir.
        return {key: value for key, value in result.items() if key not in ("id", "action", "ok")}

    @contextmanager
    def opening_fixture(self, name, *, source=TARGET_SOURCE, current=CURRENT_SOURCE,
                        cached=False, read_only=False, faults=False, no_current=False,
                        config="", setup=None):
        project = self.fixture("open-" + name, controlled=True)
        (project / "scripts/subject.gd").write_text(source)
        (project / "scripts/other.gd").write_text(current)
        if config:
            with (project / "project.godot").open("a") as stream:
                stream.write(config)
        if setup is not None:
            setup(project)
        if faults:
            shutil.copytree(self.args.native_fault_addon, project / "addons/godot_agent_kit/native",
                            dirs_exist_ok=True)
        self.installed_native(project, fixture_only=faults)
        if read_only:
            os.chmod(project / "scripts/subject.gd", 0o444)
        editor = self.start_editor(project)
        try:
            descriptor = observation.wait_for(lambda: next(iter(self.descriptors(project)), None),
                                              "selected_open_editor_session")
            info = self.open_action(editor, "open_info")
            observation.require(info["open_installed"] and info["api_revision"] == 2 and
                                info["build_id"] == self.expected_native_build_id and
                                info["session_id"] == descriptor["session_id"] and descriptor["v"] == 4,
                                "actual_native_owner_selected_v4_revision2")
            if faults:
                observation.require(info["fixture_faults"], "fixture_fault_controls_separate_binary")
                self.native_action(editor, "native_edit_fixture_activate")
                self.open_action(editor, "open_fixture_activate")
            stream, challenge = self.challenge(descriptor)
            stream.close()
            observation.require(challenge["capabilities"]["open_gdscript"] is True and
                                challenge["native_api_revision"] == 2,
                                "actual_authenticated_matched_product_opener")
            if not no_current:
                self.open_action(editor, "open_setup", paths=[BACKGROUND])
                if current == INVALID_SOURCE:
                    self.open_action(editor, "open_setup", exact_paths=[CURRENT])
                else:
                    self.open_action(editor, "open_setup", paths=[CURRENT])
            if cached:
                self.open_action(editor, "open_cache")
            yield project, editor, descriptor
        finally:
            self.close_editor(editor)
            self.editors.remove(editor)

    def open_state(self, editor, project):
        state = self.open_action(editor, "open_witness")["state"]
        disk = {name: observation.disk_witness(project / path) for name, path in
                (("target", "scripts/subject.gd"), ("current", "scripts/other.gd"),
                 ("background", "scripts/open/background.gd"))}
        return state, disk

    def concise_open_state(self, state, disks):
        return {"documents": {key: source_free_document(state[key]) for key in
                              ("target", "current", "background")},
                "disk": {key: concise_disk(value) for key, value in disks.items()},
                "cached_id": state["cached_id"],
                "cached_sha256": sha(state["cached_R"]) if state["cached_R"] is not None else None,
                "cached_edited": state["cached_edited"], "selection": state["selection"],
                "open_paths": state["open_paths"], "loader_calls": state["loader_calls"],
                "pending_node_payload_id": state["pending_node_payload_id"]}

    def assert_preserved(self, before, after, disks, now, name, *, selection=False):
        for key in ("current", "background"):
            observation.require(source_free_document(before[key]) == source_free_document(after[key]) and
                                before[key].get("R") == after[key].get("R") and
                                before[key].get("B") == after[key].get("B"),
                                "independent_human_source_flags_versions_history_" + name + "_" + key)
        observation.require(disks == now and before["pending_node_payload_id"] ==
                            after["pending_node_payload_id"] and before["loader_calls"] ==
                            after["loader_calls"] and (not selection or
                            before["selection"] == after["selection"]),
                            "independent_disk_loader_property_selection_preserved_" + name)

    def inspect(self, editor, descriptor, *, budget_us=9000000, path=TARGET, request_id=None):
        request_id = request_id or secrets.token_hex(16)
        receipt = self.open_action(editor, "open_inspect", request_id=request_id,
                                   session_id=descriptor["session_id"], budget_us=budget_us,
                                   path=path)["result"]
        self.summary["last_native_receipt"] = source_free_receipt(receipt)
        observation.require(receipt.get("request_id") == request_id or
                            (receipt.get("request_id") is None and
                             receipt.get("status") in ("busy", "refused")),
                            "native_or_addon_inspection_same_request_binding")
        return request_id, receipt

    def prepare(self, editor, project, descriptor, *, mutation="", budget_us=9000000):
        before, disks = self.open_state(editor, project)
        request_id, inspected = self.inspect(editor, descriptor, budget_us=budget_us)
        observation.require(inspected.get("status") == "inspected" and
                            inspected.get("target_open") is False,
                            "passive_closed_target_inspection")
        root = project.stat()
        target = disks["target"]
        cache = inspected.get("cache", {})
        observation.require(cache.get("state") in ("absent", "present") and
                            cache.get("script_instance_id", "") == before["cached_id"],
                            "native_cache_independent_passive_identity")
        capture = {"project_device": str(root.st_dev), "project_inode": str(root.st_ino),
                   "target_device": target["device"], "target_inode": target["inode"],
                   "cache_mode": cache["state"], "cached_script_id": before["cached_id"],
                   "sha256": target["sha256"], "utf8_bytes": str(target["size"])}
        prepared = self.open_action(editor, "open_prepare", request_id=request_id,
                                    source=target["text"], capture=capture, mutation=mutation)
        if mutation:
            before = prepared["before"]
        receipt = prepared["result"]
        self.summary["last_native_receipt"] = source_free_receipt(receipt)
        observation.require(receipt.get("request_id") == request_id,
                            "native_preparation_same_request_binding")
        return request_id, receipt, before, disks

    def validate_context(self, editor, project, descriptor, request_id, prepared, *,
                         expected="valid", alter=None, purpose="open_context", supplied=None):
        context = prepared["context"]
        observation.require(context["kind"] == "current_gdscript", "actual_current_context_required")
        before, disks = self.open_state(editor, project)
        doc = before["current"]
        projection = context["projection"]
        observation.require(doc["associated"] and doc["R"] == doc["B"] == context["source"] and
                            context["sha256"] == opening_context_fingerprint(projection) and
                            projection["source_sha256"] == sha(doc["R"]) and
                            projection["source_length"] == str(len(doc["R"].encode())) and
                            projection["request_id"] == request_id and
                            projection["session_id"] == descriptor["session_id"] and
                            projection["project_root"] == str(project.resolve()) and
                            projection["path"] == CURRENT and
                            all(projection[key] == doc[key] for key in
                                ("script_id", "editor_id", "buffer_id", "dirty", "resource_edited",
                                 "has_undo", "has_redo", "tool", "script_base_id")) and
                            projection["version"] == str(doc["version"]) and
                            projection["saved_version"] == str(doc["saved_version"]) and
                            projection["properties"] == property_projection(doc) and
                            projection["methods"] == method_names(doc) and
                            projection["warnings"] == before["effective_context"]["warnings"] and
                            projection["global_classes"] == before["effective_context"]["global_classes"] and
                            projection["autoloads"] == before["effective_context"]["autoloads"] and
                            projection["external_editor"] is before["effective_context"]["external_editor"] and
                            all(isinstance(projection[key], str) and projection[key].isdigit() and
                                str(int(projection[key])) == projection[key] for key in DECIMAL_FIELDS),
                            "independent_current_r_b_identity_dirty_history_typed_guard_fingerprint")
        bindings = self.open_action(editor, "open_class_bindings",
                                    names=[entry["name"] for entry in projection["bindings"]])["bindings"]
        observation.require(bindings == projection["bindings"], "independent_actual_classdb_context_bindings")
        def mutate(request):
            request.update(purpose=purpose, source=supplied)
            if alter is not None:
                alter(request)
        result = self.stock_open_context_validate(
            project, context, "context-" + request_id + "-" + expected, expected, mutate=mutate)
        after, now = self.open_state(editor, project)
        self.assert_preserved(before, after, disks, now, "context_helper", selection=True)
        observation.require(before["target"] == after["target"] and
                            before["cached_id"] == after["cached_id"],
                            "context_helper_no_target_effect")
        binding = result.get("opening_binding")
        if expected == "valid" and purpose == "open_context":
            expected_binding = {key: projection[key] for key in
                                ("request_id", "session_id", "project_root", "project_device",
                                 "project_inode", "path", "script_id", "editor_id", "buffer_id",
                                 "source_sha256", "source_length")}
            expected_binding["guard_sha256"] = context["sha256"]
            roots = [item for item in result["sources"] if item["path"] == CURRENT]
            observation.require(binding == expected_binding and len(roots) == 1 and
                                roots[0]["sha256"] == sha(doc["R"]) and
                                roots[0]["utf8_bytes"] == len(doc["R"].encode()) and
                                str(roots[0]["device"]) == disks["current"]["device"] and
                                str(roots[0]["inode"]) == disks["current"]["inode"] and
                                roots[0]["diagnostics_completed"] is True and
                                roots[0]["symbols_completed"] is True and not result["diagnostics"],
                                "actual_rust_completed_current_source_identity_warning_guard_binding")
        else:
            observation.require(binding is None, "nonvalid_context_never_supplies_native_authorization")
        return result

    def authorization(self, editor, project, descriptor, request_id, prepared):
        context = prepared["context"]
        if context["kind"] == "no_source_editor":
            observation.require(context["projection"] is None and context["source"] is None and
                                context["sha256"] is None and
                                not self.open_state(editor, project)[0]["current"]["associated"],
                                "proven_no_source_no_validation_hashes")
            return "", ""
        result = self.validate_context(editor, project, descriptor, request_id, prepared)
        binding = result["opening_binding"]
        return binding["source_sha256"], binding["guard_sha256"]

    def step(self, editor, request_id, stage, hashes, *, expected="ready", callback=""):
        response = self.open_action(editor, "open_advance", request_id=request_id,
                                    stage=stage, source_hash=hashes[0], context_hash=hashes[1],
                                    callback=callback)
        receipt = response["result"]
        self.summary["last_native_receipt"] = source_free_receipt(receipt)
        observation.require((receipt.get("request_id") == request_id or
                             (receipt.get("request_id") is None and
                              receipt.get("status") in ("refused", "partial"))) and
                            receipt.get("status") in ((expected,) if isinstance(expected, str)
                                                      else expected),
                            "actual_open_stage_" + stage + "_" + str(expected))
        return receipt, response.get("callback", {})

    def assert_terminal(self, editor, request_id, hashes, prior, project, name):
        receipts = []
        for stage in ("bind", "compile", "open"):
            receipt, _ = self.step(editor, request_id, stage, hashes,
                                   expected=("refused", "partial"))
            receipts.append(source_free_receipt(receipt))
        after, now = self.open_state(editor, project)
        observation.require(self.concise_open_state(prior[0], prior[1]) ==
                            self.concise_open_state(after, now),
                            "terminal_discard_never_has_later_effect_" + name)
        return receipts

    def verify_open(self, editor, project, request_id, source, before, disks):
        verified = self.open_action(editor, "open_verify", request_id=request_id,
                                    purpose="post_open")["result"]
        collector = self.open_action(editor, "open_collect", request_id=request_id)
        rechecked = self.open_action(editor, "open_recheck", request_id=request_id,
                                     purpose="post_open")["result"]
        self.summary["last_native_verification"] = [
            source_free_receipt(verified), source_free_receipt(rechecked)]
        after, now = self.open_state(editor, project)
        doc = after["target"]
        sample = collector["sample"]
        observation.require(doc["associated"] and doc["R"] == doc["B"] == source ==
                            now["target"]["text"] and not doc["dirty"] and
                            doc["version"] == doc["saved_version"] and
                            doc["resource_edited"] is False and
                            after["cached_id"] == doc["script_id"] and
                            after["cached_R"] == source and after["cached_edited"] is False and
                            sample["R"]["availability"] == sample["B"]["availability"] == "observed" and
                            sample["R"]["text"] == sample["B"]["text"] == source and
                            sample["dirty"]["state"] == "clean" and
                            sample["document"]["identity"]["script_instance_id"] == doc["script_id"] and
                            sample["document"]["identity"]["editor_instance_id"] == doc["editor_id"] and
                            sample["document"]["identity"]["buffer_instance_id"] == doc["buffer_id"] and
                            collector["recheck"]["checks"] == "performed" and
                            collector["recheck"]["detected_changes"] == [] and
                            verified.get("status") == "observed" and
                            rechecked.get("status") == "unchanged" and
                            verified.get("protection") == rechecked.get("protection") == "unchanged",
                            "independent_disk_script_codeedit_clean_resource_and_collector_identity")
        self.assert_preserved(before, after, disks, now, "new_open")
        return after, now, [source_free_receipt(verified), source_free_receipt(rechecked)]

    def positive(self, name, *, source=TARGET_SOURCE, current=CURRENT_SOURCE, cached=False,
                 read_only=False, no_current=False, human=False, compose=False):
        with self.opening_fixture(name, source=source, current=current, cached=cached,
                                  read_only=read_only, no_current=no_current) as (project, editor, descriptor):
            if human:
                self.open_action(editor, "open_setup", mutation="dirty_equal", idle=True)
                self.open_action(editor, "open_setup", paths=[BACKGROUND], mutation="dirty_equal",
                                 path=BACKGROUND, idle=True)
                self.open_action(editor, "open_setup", paths=[CURRENT])
            request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
            observation.require(prepared.get("status") == "prepared" and
                                prepared.get("mode") == ("cached" if cached else "cold"),
                                "actual_cold_cached_preparation_" + name)
            if human:
                observation.require(all(before[key]["dirty"] and before[key]["has_undo"] and
                                        before[key]["R"] == before[key]["B"] != disks[key]["text"]
                                        for key in ("current", "background")),
                                    "real_unsaved_current_and_background_r_equals_b_not_disk")
            hashes = self.authorization(editor, project, descriptor, request_id, prepared)
            receipts = []
            stages = ("bind", "open") if cached else ("bind", "compile", "open")
            for stage in stages:
                receipt, _ = self.step(editor, request_id, stage, hashes)
                receipts.append(source_free_receipt(receipt))
                if stage == "compile":
                    observation.require(receipt.get("target_parse_code") == (43 if source == INVALID_SOURCE else 0),
                                        "actual_initial_compilation_valid_or_parse_error_" + name)
            after, now, verification = self.verify_open(editor, project, request_id, source, before, disks)
            if cached:
                observation.require(after["target"]["script_id"] == before["cached_id"],
                                    "cached_script_retained_without_reload_or_assignment")
            finish = self.open_action(editor, "open_finish", request_id=request_id)["result"]
            observation.require(not self.open_state(editor, project)[0]["slot_busy"],
                                "terminal_native_finish_releases_actual_shared_slot")
            history = []
            if human:
                for key, path, original in (("current", CURRENT, CURRENT_SOURCE),
                                            ("background", BACKGROUND,
                                             (FIXTURE / "scripts/background.gd").read_text())):
                    other = "background" if key == "current" else "current"
                    preserved = self.open_state(editor, project)[0][other]
                    for operation, expected in (("undo", original), ("redo", before[key]["B"])):
                        self.open_action(editor, "open_mutate", mutation=operation, path=path)
                        transitioned = self.open_action(editor, "open_setup", path=path, idle=True)["state"]
                        observation.require(transitioned[key]["R"] == transitioned[key]["B"] == expected and
                                            transitioned["target"]["R"] == source and
                                            transitioned["target"]["B"] == source and
                                            source_free_document(transitioned[other]) ==
                                            source_free_document(preserved),
                                            "preserved_real_earlier_human_" + key + "_" + operation)
                        history.append({"operation": operation, "document": key,
                                        "state": source_free_document(transitioned[key])})
            capture = self.screenshot(editor, name + ".png")
            self.case(name, mapping="Feature003/T001 native integration §§2–7",
                      before=self.concise_open_state(before, disks), after=self.concise_open_state(after, now),
                      native_receipts=receipts + verification + [source_free_receipt(finish)],
                      preserved_earlier_history=history, capture=capture,
                      source_surfaces="independent_disk_script_codeedit_resource_edited_history")
            if compose:
                basis = self.edit_basis(project, descriptor, name + "-fresh-observe")
                desired = "extends RefCounted\nfunc value() -> int:\n\treturn 53\n"
                self.edit(project, basis, desired, name + "-fresh-edit", "verified_changed", 0,
                          session=descriptor["session_id"], application="applied")
                result, final_disk = self.open_state(editor, project)
                observation.require(result["target"]["R"] == result["target"]["B"] ==
                                    final_disk["target"]["text"] == desired and
                                    not result["target"]["dirty"] and
                                    result["target"]["resource_edited"] is False,
                                    "exact_native_methods_preserve_fresh_guarded_edit_eligibility")

    def preparation_refusal(self, name, *, source=TARGET_SOURCE, current=CURRENT_SOURCE,
                            mutation="", cached=False, config="", setup=None, faults=False):
        with self.opening_fixture(name, source=source, current=current, cached=cached,
                                  config=config, setup=setup, faults=faults) as (project, editor, descriptor):
            if mutation == "dirty_different":
                self.open_action(editor, "open_mutate", mutation=mutation)
                mutation = ""
            request_id, receipt, before, disks = self.prepare(editor, project, descriptor,
                                                             mutation=mutation)
            observation.require(receipt.get("status") == "refused" and receipt.get("reason"),
                                "unsafe_context_or_source_refused_before_authorization_" + name)
            if mutation == "stale_metadata":
                observation.require(before["current"]["R"] == before["current"]["B"] and
                                    receipt["reason"] in (
                                        "compiled_tool_context", "compiled_script_base",
                                        "unsafe_compiled_property", "unsafe_compiled_method",
                                        "compiled_metadata_unavailable_or_limit",
                                        "compiled_property_unavailable", "compiled_method_unavailable"),
                                    "actual_stale_compiled_witness_not_source_conflict_or_blanket_refusal")
            after, now = self.open_state(editor, project)
            self.assert_preserved(before, after, disks, now, name, selection=True)
            observation.require(after["target"] == before["target"] and
                                after["cached_id"] == before["cached_id"],
                                "preparation_refusal_no_cache_or_document_effect_" + name)
            terminal = self.assert_terminal(editor, request_id, ("", ""), (after, now), project, name)
            self.case(name, native_receipt=source_free_receipt(receipt),
                      before=self.concise_open_state(before, disks), after=self.concise_open_state(after, now),
                      later_stages=terminal, source_surfaces="real_source_compiled_context_and_no_effect")

    def validation_refusals(self):
        with self.opening_fixture("validation-owned-helper-failures") as (project, editor, descriptor):
            request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
            self.stock_open_context_regressions(project, prepared["context"])
            after, now = self.open_state(editor, project)
            self.assert_preserved(before, after, disks, now, "owned_helper_failures", selection=True)
            observation.require(after["cached_id"] == "" and not after["target"]["associated"],
                                "helper_loss_deadline_and_wrong_binary_never_authorize_native_effect")
            self.open_action(editor, "open_cancel", request_id=request_id)
        modifications = (("wrong_request", lambda r: r.update(request_id=secrets.token_hex(16))),
                         ("wrong_session", lambda r: r.update(session_id=secrets.token_hex(16))),
                         ("wrong_identity", lambda r: r["open_context"]["projection"].update(buffer_id="0")),
                         ("wrong_source", lambda r: r["open_context"].update(source=CURRENT_SOURCE + "# replaced\n")),
                         ("wrong_fingerprint", lambda r: r["open_context"].update(sha256="0" * 64)),
                         ("wrong_warning", lambda r: r["warnings"].update(enable=not r["warnings"]["enable"])),
                         ("missing_context", lambda r: r.pop("open_context")),
                         ("replaced_path", lambda r: r.update(script=TARGET)))
        for name, alter in modifications:
            with self.opening_fixture("validation-" + name, faults=True) as (project, editor, descriptor):
                request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
                observation.require(prepared["status"] == "prepared", "context_refusal_prepared")
                result = self.validate_context(editor, project, descriptor, request_id, prepared,
                                               expected="unavailable", alter=alter)
                self.open_action(editor, "open_fault", request_id=request_id, fault="callback_bind")
                denied, entered = self.step(editor, request_id, "bind", ("", ""), expected="refused")
                observation.require(entered == {}, "invalid_validation_never_reaches_native_entered_callback")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, name, selection=True)
                observation.require(after["cached_id"] == before["cached_id"] == "" and
                                    not after["target"]["associated"], "invalid_receipt_no_native_entry")
                self.case("validation_" + name, validation_status=result["status"],
                          validation_reason=result.get("reason"), native_receipt=source_free_receipt(denied),
                          source_surfaces="actual_rust_context_binding_and_native_no_entry")
        for name, current, expected, purpose, supplied in (
                ("invalid_current", INVALID_SOURCE, "invalid", "open_context", None),
                ("wrong_purpose", CURRENT_SOURCE, "unavailable", "preflight", None),
                ("supplied_proposal", CURRENT_SOURCE, "unavailable", "open_context", CURRENT_SOURCE)):
            # An invalid current is created by exact source methods on a real
            # Script; no fake diagnostic or completed-valid result is supplied.
            with self.opening_fixture("validation-" + name, current=current, faults=True) as (project, editor, descriptor):
                request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
                observation.require(prepared["status"] == "prepared", "validator_not_native_target_parser")
                result = self.validate_context(editor, project, descriptor, request_id, prepared,
                                               expected=expected, purpose=purpose, supplied=supplied)
                self.open_action(editor, "open_fault", request_id=request_id, fault="callback_bind")
                denied, entered = self.step(editor, request_id, "bind", ("", ""), expected="refused")
                observation.require(entered == {}, "nonvalid_current_context_never_reaches_entered_callback")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, name, selection=True)
                self.case("validation_" + name, validation_status=result["status"],
                          native_receipt=source_free_receipt(denied),
                          source_surfaces="real_source_only_validation_and_before_entry_refusal")

    def current_literal_refusals(self):
        for form in ("absolute", "project_escape", "symlink", "escaped_absolute", "triple_absolute"):
            def setup(project):
                outside = project.parent / ("outside-" + form + ".gd")
                outside.write_text("extends RefCounted\n# OPEN_OUTSIDE_PRIVATE_SOURCE\n")
                if form == "project_escape":
                    value = '\"res://../' + outside.name + '\"'
                elif form == "symlink":
                    (project / "scripts/open/link.gd").symlink_to(outside)
                    value = '\"res://scripts/open/link.gd\"'
                elif form == "escaped_absolute":
                    value = '"\\u002f' + str(outside)[1:] + '"'
                elif form == "triple_absolute":
                    value = '\"\"\"' + str(outside) + '\"\"\"'
                else:
                    value = json.dumps(str(outside))
                (project / "scripts/other.gd").write_text(
                    "extends RefCounted\nfunc value():\n\treturn " + value + "\n")
            with self.opening_fixture("literal-" + form, setup=setup) as (project, editor, descriptor):
                request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
                observation.require(prepared["status"] == "prepared",
                                    "real_current_literal_native_context_before_stock_confinement")
                result = self.validate_context(editor, project, descriptor, request_id, prepared,
                                               expected="unavailable")
                receipt, _ = self.step(editor, request_id, "bind", ("", ""), expected="refused")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, "literal_" + form, selection=True)
                observation.require(not after["target"]["associated"] and after["cached_id"] == "",
                                    "literal_confinement_refusal_has_no_native_entry")
                self.case("current_literal_" + form, validation_status=result["status"],
                          validation_reason=result.get("reason"), native_receipt=source_free_receipt(receipt),
                          source_surfaces="real_current_r_b_and_actual_stock_no_follow_literal_confinement")

    def stage_refusals(self):
        for name, stage, change in (
                ("missing_authorization", "bind", lambda hashes: ("", "")),
                ("wrong_source_hash", "bind", lambda hashes: ("0" * 64, hashes[1])),
                ("wrong_context_hash", "bind", lambda hashes: (hashes[0], "0" * 64)),
                ("compile_before_bind", "compile", lambda hashes: hashes),
                ("open_before_bind", "open", lambda hashes: hashes),
                ("unknown_stage", "arbitrary", lambda hashes: hashes)):
            with self.opening_fixture("stage-" + name, faults=True) as (project, editor, descriptor):
                request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
                hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                probe_stage = stage if stage in ("bind", "compile", "open") else "bind"
                self.open_action(editor, "open_fault", request_id=request_id, fault="callback_" + probe_stage)
                receipt, entered = self.step(editor, request_id, stage, change(hashes), expected="refused")
                observation.require(entered == {}, "wrong_or_unbound_stage_has_no_actual_native_entry")
                observation.require(receipt.get("reason") == "wrong_stage_or_context_validation_binding",
                                    "wrong_binding_refusal_is_not_vacuous_deadline_or_unavailable_native")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, name, selection=True)
                observation.require(not after["target"]["associated"] and after["cached_id"] == "",
                                    "wrong_or_unbound_stage_never_enters_effect")
                later = self.assert_terminal(editor, request_id, hashes, (after, now), project, name)
                self.case("stage_" + name, native_receipt=source_free_receipt(receipt), later_stages=later,
                          source_surfaces="bound_valid_context_and_monotonic_native_stage")
        for completed in ("bind", "compile", "open"):
            with self.opening_fixture("duplicate-" + completed) as (project, editor, descriptor):
                request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                for stage in ("bind", "compile", "open"):
                    self.step(editor, request_id, stage, hashes)
                    if stage == completed:
                        break
                before = self.open_state(editor, project)
                duplicate, _ = self.step(editor, request_id, completed, hashes,
                                          expected=("partial", "refused"))
                later = self.assert_terminal(editor, request_id, hashes, before, project, completed)
                self.case("duplicate_" + completed, native_receipt=source_free_receipt(duplicate),
                          later_stages=later, source_surfaces="one_shot_stage_and_retained_partial_effects")

    def races(self):
        changes = ("file_content", "file_replace", "file_delete", "parent_replace",
                   "project_replace", "target_cache", "target_human_open", "current_source",
                   "current_resource", "current_close", "current_replace", "warning",
                   "autoload", "external_editor", "session", "target_cache_source",
                   "target_cache_replace", "global_context")
        for change in changes:
            cached = change.startswith("target_cache_")
            with self.opening_fixture("race-" + change, cached=cached) as (project, editor, descriptor):
                request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                target = project / "scripts/subject.gd"
                if change == "file_content":
                    target.write_text(TARGET_SOURCE + "# OPEN_FILE_CHANGED\n")
                elif change == "file_replace":
                    replacement = target.with_suffix(".replacement")
                    replacement.write_text(TARGET_SOURCE)
                    replacement.replace(target)
                elif change == "file_delete":
                    target.unlink()
                elif change == "parent_replace":
                    original = project / "scripts"
                    held = project / "scripts-held"
                    original.rename(held)
                    shutil.copytree(held, original)
                elif change == "project_replace":
                    held = project.with_name(project.name + "-held")
                    project.rename(held)
                    shutil.copytree(held, project)
                elif change == "global_context":
                    self.open_action(editor, "open_global_change")
                else:
                    self.open_action(editor, "open_mutate", mutation=change)
                # The human/file transition is the new survivor baseline, never
                # compared as if it were a kit mutation or rolled back.
                before = self.open_action(editor, "open_witness")["state"]
                disk_before = {key: observation.disk_witness(project / path) for key, path in
                               (("current", "scripts/other.gd"), ("background", "scripts/open/background.gd"))}
                target_before = observation.disk_witness(target) if target.exists() else None
                receipt, _ = self.step(editor, request_id, "bind", hashes,
                                      expected=("refused", "partial"))
                after = self.open_action(editor, "open_witness")["state"]
                observation.require(source_free_document(before["target"]) == source_free_document(after["target"]) and
                                    source_free_document(before["current"]) == source_free_document(after["current"]) and
                                    source_free_document(before["background"]) == source_free_document(after["background"]) and
                                    before["cached_id"] == after["cached_id"] and
                                    before["selection"] == after["selection"] and
                                    target_before == (observation.disk_witness(target) if target.exists() else None) and
                                    all(observation.disk_witness(project / ("scripts/other.gd" if key == "current"
                                                                          else "scripts/open/background.gd")) == value
                                        for key, value in disk_before.items()),
                                    "observed_change_defeats_stale_authorization_no_repair_" + change)
                self.case("race_" + change, native_receipt=source_free_receipt(receipt),
                          before={key: source_free_document(before[key]) for key in ("target", "current", "background")},
                          after={key: source_free_document(after[key]) for key in ("target", "current", "background")},
                          source_surfaces="real_file_cache_document_session_effective_context_transition")

    def partial_and_terminal(self):
        for fault in ("fail_after_bind", "expire_bind", "expire_compile", "expire_open"):
            with self.opening_fixture(fault, faults=True) as (project, editor, descriptor):
                request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                stage = "bind" if fault == "fail_after_bind" else fault.removeprefix("expire_")
                for previous in ("bind", "compile"):
                    if previous == stage:
                        break
                    self.step(editor, request_id, previous, hashes)
                self.open_action(editor, "open_fault", request_id=request_id, fault=fault)
                receipt, _ = self.step(editor, request_id, stage, hashes,
                                      expected="refused" if stage == "bind" and fault != "fail_after_bind" else "partial")
                survivor = self.open_state(editor, project)
                state = survivor[0]
                published = stage != "bind" or fault == "fail_after_bind"
                observation.require(bool(state["cached_id"]) is published and
                                    not state["target"]["associated"] and
                                    (not published or (state["cached_R"] == TARGET_SOURCE and
                                                       state["cached_edited"] is False)),
                                    "known_cache_publication_without_document_is_partial_not_rollback")
                later = self.assert_terminal(editor, request_id, hashes, survivor, project, fault)
                self.case(fault, native_receipt=source_free_receipt(receipt), later_stages=later,
                          survivor=self.concise_open_state(*survivor),
                          source_surfaces="actual_partial_cache_publication_and_terminal_expiry")
        for completed in ("prepared", "bind", "compile", "open"):
            for terminal in ("open_cancel", "open_finish"):
                with self.opening_fixture(completed + "-" + terminal) as (project, editor, descriptor):
                    request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                    hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                    if completed != "prepared":
                        for stage in ("bind", "compile", "open"):
                            self.step(editor, request_id, stage, hashes)
                            if stage == completed:
                                break
                    receipt = self.open_action(editor, terminal, request_id=request_id)["result"]
                    survivor = self.open_state(editor, project)
                    observation.require(not survivor[0]["slot_busy"], "terminal_discard_releases_idle_owner")
                    later = self.assert_terminal(editor, request_id, hashes, survivor, project, terminal)
                    self.case(completed + "_" + terminal, native_receipt=source_free_receipt(receipt),
                              later_stages=later, survivor=self.concise_open_state(*survivor),
                              source_surfaces="terminal_discard_at_each_lifecycle_boundary_no_later_effect")

    def property_negative(self):
        with self.opening_fixture("property-source-negative", faults=True) as (project, editor, descriptor):
            request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
            hashes = self.authorization(editor, project, descriptor, request_id, prepared)
            self.open_action(editor, "open_fault", request_id=request_id, fault="property_source")
            receipt, _ = self.step(editor, request_id, "bind", hashes, expected=("partial", "refused"))
            after, now = self.open_state(editor, project)
            observation.require(after["cached_id"] and after["cached_R"] == TARGET_SOURCE and
                                after["cached_edited"] is True and not after["target"]["associated"],
                                "property_assignment_resource_dirty_despite_source_agreement")
            self.assert_preserved(before, after, disks, now, "property_negative", selection=True)
            self.open_action(editor, "open_finish", request_id=request_id)
            # Explicit negative-control preparation opens the published dirty R,
            # never used as evidence that the kit opened it successfully.
            self.open_action(editor, "open_setup", paths=[TARGET])
            opened, open_disks = self.open_state(editor, project)
            observation.require(opened["target"]["R"] == opened["target"]["B"] ==
                                open_disks["target"]["text"] and not opened["target"]["dirty"] and
                                opened["target"]["resource_edited"] is True,
                                "real_property_negative_clean_buffer_but_dirty_resource")
            basis = self.edit_basis(project, descriptor, "property-negative-fresh-observe")
            result = self.edit(project, basis, TARGET_SOURCE + "# composed replacement\n",
                               "property-negative-fresh-edit", "refused", 3,
                               session=descriptor["session_id"], reason="dirty_conflict",
                               application="not_applied")
            final, final_disk = self.open_state(editor, project)
            observation.require(opened["target"] == final["target"] and open_disks == final_disk,
                                "real_existing_edit_refuses_property_negative_without_repair")
            self.case("property_source_negative", native_receipt=source_free_receipt(receipt),
                      after=self.concise_open_state(final, final_disk), edit_reason=result["reason"],
                      capture=self.screenshot(editor, "property-negative.png"),
                      source_surfaces="real_object_property_negative_and_fresh_observe_edit")

    def between_stage_busy(self):
        with self.opening_fixture("between-stage-busy") as (project, editor, descriptor):
            current_basis = self.observe(project, "complete_observation", 0,
                                         session=descriptor["session_id"], script=CURRENT,
                                         name="between-stage-fresh-current-basis")
            request_id, prepared, before, disks = self.prepare(editor, project, descriptor)
            hashes = self.authorization(editor, project, descriptor, request_id, prepared)
            self.step(editor, request_id, "bind", hashes)
            held = self.open_state(editor, project)
            observation.require(held[0]["slot_busy"] and held[0]["owner_matches"] and
                                held[0]["cached_id"] and not held[0]["target"]["associated"],
                                "actual_between_stage_owner_hold_editor_remains_responsive")
            from discovery_scope_acceptance import assert_discovery_busy
            assert_discovery_busy(self, descriptor)
            self.case("scope_refused_during_real_open_owner", reason="busy", context_observed=False)
            stream, peer_id = self.authenticated_peer(descriptor)
            try:
                stream.sendall(observation.packet([4, "observe", peer_id, descriptor["session_id"],
                                                    descriptor["project_root"], TARGET]))
                refusal, raw = observation.receive(stream)
                observation.require(refusal.get("kind") == "failure" and
                                    refusal.get("code") == "unsupported_observation" and
                                    refusal.get("request_id") == peer_id and b"source_code" not in raw and
                                    observation.peer_closed_without_data(stream),
                                    "actual_authenticated_observer_terminal_busy_no_source")
            finally:
                stream.close()
            busy_edit = self.edit(project, current_basis, CURRENT_SOURCE + "# busy proposal\n",
                                  "between-stage-actual-edit-busy", "refused", 3,
                                  session=descriptor["session_id"], script=CURRENT,
                                  reason="busy", application="not_applied")
            observation.require(busy_edit["history"] == "not_participated" and
                                busy_edit["before"] is None and busy_edit["after"] is None,
                                "actual_edit_terminal_busy_without_entry_or_queued_work")
            competitor_id, competing = self.inspect(editor, descriptor)
            observation.require(competing.get("status") in ("busy", "refused") and
                                competing.get("reason") == "slot_busy", "actual_private_owner_terminal_busy")
            self.step(editor, request_id, "compile", hashes)
            self.step(editor, request_id, "open", hashes)
            after, now, _ = self.verify_open(editor, project, request_id, TARGET_SOURCE, before, disks)
            self.open_action(editor, "open_finish", request_id=request_id)
            later, _ = self.step(editor, competitor_id, "bind", hashes, expected="refused")
            final, final_disk = self.open_state(editor, project)
            observation.require(after["target"] == final["target"] and now == final_disk,
                                "busy_competitor_never_queued_or_runs_after_release")
            self.case("between_stage_busy", competitor=source_free_receipt(competing),
                      later=source_free_receipt(later), held=self.concise_open_state(*held),
                      source_surfaces="real_bridge_slot_responsive_busy_and_no_late_request")

    def entered_ownership(self):
        for stage in ("bind", "compile", "open"):
            for mode in ("cancel", "finish", "stop", "disable", "compete", "stall"):
                with self.opening_fixture("entered-" + stage + "-" + mode, faults=True) as (project, editor, descriptor):
                    request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                    hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                    for previous in ("bind", "compile"):
                        if previous == stage:
                            break
                        self.step(editor, request_id, previous, hashes)
                    fault = ("stall_" if mode == "stall" else "callback_") + stage
                    self.open_action(editor, "open_fault", request_id=request_id, fault=fault)
                    if mode == "stall":
                        identity = secrets.token_hex(16)
                        event = editor["control"] / "open-entered.json"
                        event.unlink(missing_ok=True)
                        observation.json_file(editor["control"] / "request.json", {
                            "id": identity, "action": "open_advance", "request_id": request_id,
                            "stage": stage, "source_hash": hashes[0], "context_hash": hashes[1]})
                        observation.wait_for(lambda: event.is_file() and json.loads(event.read_text()) ==
                                             {"request_id": request_id, "stage": stage},
                                             "real_synchronous_native_entry_before_stall", timeout=3)
                        started = time.monotonic()
                        competing = subprocess.run(self.observation_command(project, descriptor["session_id"], TARGET),
                                                   stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=5.5)
                        elapsed = time.monotonic() - started
                        outcome = json.loads(competing.stdout)
                        observation.require(outcome.get("outcome") == "timeout" and elapsed <= 5.1,
                                            "entered_native_stall_not_misrepresented_as_responsive_busy")
                        def returned():
                            path = editor["control"] / "response.json"
                            if path.is_file():
                                value = json.loads(path.read_text())
                                return value if value.get("id") == identity else None
                            return None
                        response = observation.wait_for(returned, "entered_native_stall_returned", timeout=15)
                        receipt, callback = response["result"], response["callback"]
                    else:
                        receipt, callback = self.step(editor, request_id, stage, hashes,
                                                      expected=("ready", "partial", "refused", "discarded"), callback=mode)
                    observation.require(callback.get("stage") == stage and
                                        callback["before"]["slot_busy"] and callback["before"]["owner_matches"] and
                                        callback["retained_slot_busy"] and callback["retained_owner_matches"] and
                                        callback["retained_before"]["entered"] is True and
                                        callback["retained_after"]["entered"] is True and
                                        callback["retained_before"]["shared_owner"] == "open" and
                                        callback["retained_after"]["shared_owner"] == "open" and
                                        callback["current_reference_alive"] is True and
                                        all(callback["retained_before"]["current_" + field] ==
                                            callback["before"]["current"][field] and
                                            callback["retained_after"]["current_" + field] ==
                                            callback["before"]["current"][field]
                                            for field in ("script_id", "editor_id", "buffer_id")),
                                        "real_entered_call_retains_native_refs_and_actual_addon_slot")
                    if mode == "compete":
                        observation.require(callback["competitor"].get("reason") == "slot_busy",
                                            "reentrant_competitor_cannot_replace_entered_owner")
                        self.open_action(editor, "open_cancel", request_id=request_id)
                    elif mode not in ("stop", "disable"):
                        self.open_action(editor, "open_finish", request_id=request_id)
                    survivor = self.open_state(editor, project)
                    observation.require(not survivor[0]["slot_busy"],
                                        "entered_native_owner_released_only_after_return")
                    if mode == "disable":
                        self.action(editor, "enable")
                        fresh = observation.wait_for(
                            lambda: next((value for value in self.descriptors(project)
                                          if value["session_id"] != descriptor["session_id"]), None),
                            "actual_new_session_after_entered_disable")
                        observation.require(fresh["session_id"] != descriptor["session_id"],
                                            "replacement_session_cannot_inherit_entered_attempt")
                    later = self.assert_terminal(editor, request_id, hashes, survivor, project, stage + mode)
                    self.case("entered_" + stage + "_" + mode, native_receipt=source_free_receipt(receipt),
                              retained_before=callback["retained_before"], retained_after=callback["retained_after"],
                              later_stages=later, survivor=self.concise_open_state(*survivor),
                              source_surfaces="actual_entered_slot_refs_terminal_deferral_and_synchronous_stall")

    def pending_drag(self):
        for kind, source in (("primitive", "extends Node\n@export var payload: int = 0\n\n"),
                             ("object", "extends Node\nvar payload: Resource\n\n"),
                             ("exported", "extends Node\n@export var payload: Resource\n\n")):
            for undo in (False, True):
                name = "pending_drag_" + kind + ("_undo" if undo else "")
                with self.opening_fixture(name, current=source) as (project, editor, descriptor):
                    drag = self.open_action(editor, "open_pending_drag", undo=undo)["drag"]
                    observation.require(drag["dragged"]["B"] != drag["before"]["B"] and
                                        drag["dragged"]["has_undo"] and
                                        (not undo or drag["after"]["B"] == drag["before"]["B"]),
                                        "genuine_native_export_drop_and_optional_undo")
                    request_id, receipt, before, disks = self.prepare(editor, project, descriptor)
                    observation.require(receipt["status"] == "refused", "pending_export_context_refused")
                    after, now = self.open_state(editor, project)
                    self.assert_preserved(before, after, disks, now, name, selection=True)
                    observation.require(not after["target"]["associated"] and after["cached_id"] == "" and
                                        before["pending_node_payload_id"] == after["pending_node_payload_id"],
                                        "opening_refusal_no_dragged_object_property_assignment")
                    self.case(name, native_receipt=source_free_receipt(receipt),
                              drag_before=source_free_document(drag["before"]),
                              dragged=source_free_document(drag["dragged"]),
                              after=source_free_document(drag["after"]),
                              payload_id=drag["payload_id"],
                              capture=self.screenshot(editor, name + ".png"),
                              source_surfaces="actual_drag_forwarding_native_history_and_object_property_witness")

    def profiles(self):
        profiles = (("target_tool", "@tool\nextends RefCounted\n"),
                    ("target_export", "extends RefCounted\n@export var value: int\n"),
                    ("target_class_name", "class_name OpeningTargetClass\nextends RefCounted\n"),
                    ("target_static", "extends RefCounted\nstatic func value():\n\tpass\n"),
                    ("target_const", "extends RefCounted\nconst VALUE = 1\n"),
                    ("target_load", 'extends RefCounted\nvar dependency = load("res://scripts/open/current.gd")\n'),
                    ("target_preload", 'extends RefCounted\nvar dependency = preload("res://scripts/open/current.gd")\n'),
                    ("target_script_base", 'extends "res://scripts/open/current.gd"\n'),
                    ("target_dynamic_get", "extends RefCounted\nfunc _get(_key):\n\treturn null\n"),
                    ("target_global", "extends OpenFixtureGlobal\n"),
                    ("target_autoload", "extends RefCounted\nfunc value():\n\treturn OpenFixtureAutoload\n"),
                    ("source_limit_over", "# " + "x" * (524289 - 3) + "\n"),
                    ("identifier_limit_over", "extends RefCounted\n" + "\n".join(
                        f"var unique_identifier_{i}\n" for i in range(257))),
                    ("source_cr", TARGET_SOURCE.replace("\n", "\r\n")),
                    ("source_bom", "\ufeff" + TARGET_SOURCE),
                    ("source_nul", TARGET_SOURCE + "\x00"),
                    ("source_control", TARGET_SOURCE + "\x01"),
                    ("source_unsupported_quote", 'extends RefCounted\nvar text = "unterminated\n'))
        for name, source in profiles:
            setup = None
            config = ""
            if name == "target_global":
                def setup(project):
                    (project / "scripts/open/global.gd.disabled").rename(project / "opening_global.gd")
            if name == "target_autoload":
                config = '\n[autoload]\nOpenFixtureAutoload="*res://scripts/open/autoload.gd"\n'
            self.preparation_refusal(name, source=source, config=config, setup=setup)
        self.preparation_refusal("target_extension", source="extends GodotAgentKitOpeningFixture\n",
                                 faults=True)
        stale = (("stale_tool", "@tool\nextends RefCounted\n"),
                 ("stale_base", 'extends "res://scripts/open/current.gd"\n'),
                 ("stale_property_primitive_export", "extends RefCounted\n@export var payload: int\n"),
                 ("stale_property_object", "extends RefCounted\nvar payload: Resource\n"),
                 ("stale_property_typed_hint", "extends Node\n@export var payload: Node\n"),
                 ("stale_method", "extends RefCounted\nfunc _get(_key):\n\treturn null\n"))
        for name, current in stale:
            self.preparation_refusal(name, current=current, mutation="stale_metadata")
        self.preparation_refusal("dirty_current_r_not_b", mutation="dirty_different")
        self.preparation_refusal("effective_external_editor", mutation="external_editor")
        self.preparation_refusal("compiled_property_limit", current="extends RefCounted\n" +
                                 "".join(f"var property_{i}: int = 0\n" for i in range(64)))
        self.preparation_refusal("compiled_method_limit", current="extends RefCounted\n" +
                                 "".join(f"func method_{i}():\n\tpass\n" for i in range(65)))
        self.preparation_refusal("compiled_property_name_limit",
                                 current="extends RefCounted\nvar " + "p" * 257 + ": int = 0\n",
                                 mutation="stale_metadata")
        self.preparation_refusal("compiled_method_name_limit",
                                 current="extends RefCounted\nfunc " + "m" * 257 + "():\n\tpass\n",
                                 mutation="stale_metadata")
        self.preparation_refusal("compiled_property_hint_limit", current="extends RefCounted\n" +
                                 '@export_enum("' + "x" * 2049 + '") var payload: int\n',
                                 mutation="stale_metadata")
        config = "\n[autoload]\n" + "".join(
            f'OpeningAutoload{i}="*res://scripts/open/autoload.gd"\n' for i in range(65))
        self.preparation_refusal("effective_autoload_limit", config=config)
        def globals_setup(project):
            for i in range(65):
                (project / f"opening_global_{i}.gd").write_text(
                    f"class_name OpeningGlobal{i}\nextends RefCounted\n")
        self.preparation_refusal("effective_global_limit", setup=globals_setup)

    def passive_and_cached_refusals(self):
        for name, arguments in (
                ("wrong_selected_session", {"session_id": "0" * 32}),
                ("expired_deadline", {"budget_us": -1}),
                ("unsupported_extension", {"path": "res://scripts/note.txt"}),
                ("outside_path", {"path": "res://../outside.gd"}),
                ("embedded_path", {"path": "res://container.tscn::GDScript_x"})):
            with self.opening_fixture(name) as (project, editor, descriptor):
                before, disks = self.open_state(editor, project)
                request_id = secrets.token_hex(16)
                receipt = self.open_action(
                    editor, "open_inspect", request_id=request_id,
                    **({"session_id": descriptor["session_id"], "budget_us": 9000000,
                        "path": TARGET} | arguments))["result"]
                observation.require(receipt.get("status") == "refused" and receipt.get("reason"),
                                    "passive_identity_and_profile_refusal_" + name)
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, name, selection=True)
                observation.require(before["target"] == after["target"] and
                                    before["cached_id"] == after["cached_id"] == "",
                                    "inspection_refusal_no_target_source_or_cache_effect")
                self.case(name, native_receipt=source_free_receipt(receipt),
                          source_surfaces="actual_session_deadline_path_admission_without_loading")
        for mutation in ("target_cache_source", "target_cache_edited", "target_cache_wrong_type"):
            with self.opening_fixture(mutation, cached=mutation != "target_cache_wrong_type") as (project, editor, descriptor):
                self.open_action(editor, "open_mutate", mutation=mutation)
                before, disks = self.open_state(editor, project)
                if mutation == "target_cache_wrong_type":
                    request_id, receipt = self.inspect(editor, descriptor)
                else:
                    request_id, receipt, before, disks = self.prepare(editor, project, descriptor)
                observation.require(receipt.get("status") == "refused" and receipt.get("reason"),
                                    "actual_wrong_stale_or_edited_cached_resource_refused")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, mutation, selection=True)
                observation.require(before["cached_id"] == after["cached_id"] and
                                    before["cached_R"] == after["cached_R"] and
                                    before["cached_edited"] == after["cached_edited"] and
                                    not after["target"]["associated"],
                                    "cached_refusal_never_reassigns_reloads_or_repairs_resource")
                self.open_action(editor, "open_cancel", request_id=request_id)
                self.case(mutation, native_receipt=source_free_receipt(receipt),
                          before=self.concise_open_state(before, disks),
                          after=self.concise_open_state(after, now),
                          source_surfaces="independent_cached_script_identity_source_edited_state")

    def already_open_recognition(self):
        for dirty in (False, True):
            name = "already_open_" + ("dirty_divergent" if dirty else "clean")
            with self.opening_fixture(name) as (project, editor, descriptor):
                self.open_action(editor, "open_setup", paths=[TARGET, CURRENT])
                if dirty:
                    self.open_action(editor, "open_mutate", mutation="dirty_different", path=TARGET)
                before, disks = self.open_state(editor, project)
                observation.require(before["target"]["associated"] and
                                    (not dirty or (before["target"]["dirty"] and
                                                   before["target"]["R"] != before["target"]["B"])),
                                    "actual_nonselected_dirty_recognition_control")
                request_id, receipt = self.inspect(editor, descriptor)
                observation.require(receipt.get("status") == "inspected" and
                                    receipt.get("target_open") is True,
                                    "already_open_passive_recognition_without_preparation")
                verified = self.open_action(editor, "open_verify", request_id=request_id,
                                            purpose="recognition")["result"]
                collector = self.open_action(editor, "open_collect", request_id=request_id)
                checked = self.open_action(editor, "open_recheck", request_id=request_id,
                                           purpose="recognition")["result"]
                observation.require(verified.get("status") == "observed" and
                                    checked.get("status") == "unchanged" and
                                    collector["sample"]["R"]["text"] == before["target"]["R"] and
                                    collector["sample"]["B"]["text"] == before["target"]["B"] and
                                    collector["recheck"]["checks"] == "performed" and
                                    collector["recheck"]["detected_changes"] == [],
                                    "recognition_independently_retains_actual_divergent_sources")
                self.open_action(editor, "open_finish", request_id=request_id)
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, name, selection=True)
                observation.require(source_free_document(before["target"]) ==
                                    source_free_document(after["target"]),
                                    "recognition_never_selects_reloads_validates_or_resets_history")
                self.case(name, native_receipts=[source_free_receipt(value) for value in
                                                (receipt, verified, checked)],
                          before=self.concise_open_state(before, disks),
                          after=self.concise_open_state(after, now),
                          source_surfaces="actual_already_open_identity_divergence_preserved_without_entry")

    def published_races(self):
        for completed, change in (("bind", "file_content"), ("compile", "file_replace"),
                                  ("bind", "warning"), ("compile", "current_source"),
                                  ("bind", "target_cache_replace"), ("compile", "target_human_open")):
            name = "published_" + completed + "_" + change
            with self.opening_fixture(name) as (project, editor, descriptor):
                request_id, prepared, _, _ = self.prepare(editor, project, descriptor)
                hashes = self.authorization(editor, project, descriptor, request_id, prepared)
                bound, _ = self.step(editor, request_id, "bind", hashes)
                observation.require(bound["cache_binding"] == "new_resource_published",
                                    "actual_publication_before_human_race")
                if completed == "compile":
                    self.step(editor, request_id, "compile", hashes)
                target = project / "scripts/subject.gd"
                if change == "file_content":
                    target.write_text(TARGET_SOURCE + "# published human file change\n")
                elif change == "file_replace":
                    replacement = target.with_suffix(".replacement")
                    replacement.write_text(TARGET_SOURCE)
                    replacement.replace(target)
                else:
                    self.open_action(editor, "open_mutate", mutation=change)
                survivor = self.open_state(editor, project)
                receipt, _ = self.step(editor, request_id,
                                       "compile" if completed == "bind" else "open",
                                       hashes, expected="partial")
                observation.require(receipt.get("cache_binding") == "new_resource_published",
                                    "post_publication_failure_cannot_be_reduced_to_not_applied")
                later = self.assert_terminal(editor, request_id, hashes, survivor, project, name)
                self.case(name, native_receipt=source_free_receipt(receipt), later_stages=later,
                          survivor=self.concise_open_state(*survivor),
                          source_surfaces="actual_post_publication_invalidation_preserves_human_survivor")

    def native_boundary(self):
        self.compile_window_probe()
        self.group("positive", lambda: (
            self.positive("cold_exact_methods_fresh_observe_edit", compose=True),
            self.positive("cached_retained_resource", cached=True),
            self.positive("empty_exact_source", source=""),
            self.positive("read_only_exact_source", read_only=True),
            self.positive("syntax_invalid_target", source=INVALID_SOURCE),
            self.positive("no_source_current", no_current=True),
            self.positive("dirty_current_human_history", human=True),
            self.positive("target_lf_whitespace_preserved",
                          source="extends RefCounted\nfunc value():\n\treturn 7  \n\n"),
            self.positive("current_lf_whitespace_preserved",
                          current="extends RefCounted\nfunc value():\n\treturn 7  \n\n"),
            self.positive("source_limit_exact", source="# " + "x" * (524288 - 3) + "\n"),
            self.positive("compiled_property_limit_exact", current="extends RefCounted\n" +
                          "".join(f"var property_{i}: int = 0\n" for i in range(63))),
            self.positive("compiled_method_limit_exact", current="extends RefCounted\n" +
                          "".join(f"func method_{i}():\n\tpass\n" for i in range(64))),
            self.positive("compiled_property_name_limit_exact",
                          current="extends RefCounted\nvar " + "p" * 256 + ": int = 0\n"),
            self.positive("compiled_method_name_limit_exact",
                          current="extends RefCounted\nfunc " + "m" * 256 + "():\n\tpass\n"),
            self.positive("source_identifier_lookup_limit_exact", source="extends RefCounted\n" +
                          "".join(f"var field_{i}: int = 0\n" for i in range(252))),
            self.positive("excluded_words_in_comments_strings", source=
                          'extends RefCounted\n# @tool class_name load preload static const\n'
                          'func value():\n\treturn "load preload @export _get class_name"\n')))
        self.group("property-negative", self.property_negative)
        self.group("passive-cached-refusals", self.passive_and_cached_refusals)
        self.group("already-open-recognition", self.already_open_recognition)
        self.group("profile-refusals", self.profiles)
        self.group("context-validation", self.validation_refusals)
        self.group("current-literal-confinement", self.current_literal_refusals)
        self.group("stage-refusals", self.stage_refusals)
        self.group("identity-races", self.races)
        self.group("published-races", self.published_races)
        self.group("terminal-partial", self.partial_and_terminal)
        self.group("between-stage-busy", self.between_stage_busy)
        self.group("entered-ownership", self.entered_ownership)
        self.group("pending-drag-history", self.pending_drag)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", required=True, type=Path)
    parser.add_argument("--observer", required=True, type=Path)
    parser.add_argument("--editor", required=True, type=Path, help="existing edit-gdscript caller")
    parser.add_argument("--opener", type=Path, help="actual public open-gdscript caller; required for public groups")
    parser.add_argument("--stock-validator", required=True, type=Path,
                        help="actual test-only stock_validation_fixture example")
    parser.add_argument("--native-fault-addon", required=True, type=Path,
                        help="separate GAK_FIXTURE editor_integration artifact directory")
    parser.add_argument("--scenario", required=True, choices=(*SCENARIOS, "all"))
    parser.add_argument("--artifacts", required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    observation.require(args.scenario == "native-boundary" or args.opener is not None,
                        "public_opening_group_requires_real_opener_binary")
    for path in (args.godot, args.observer, args.editor, args.stock_validator,
                 *((args.opener,) if args.opener else ())):
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
                            "exact_absolute_executable_inputs_required")
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and
                        not args.artifacts.is_symlink(), "absolute_existing_artifact_directory")
    metadata = args.artifacts.stat()
    observation.require(metadata.st_uid == os.geteuid() and stat.S_IMODE(metadata.st_mode) == 0o700 and
                        not list(args.artifacts.iterdir()), "empty_owned_private_artifact_directory")
    observation.require(observation.digest(args.godot) == STOCK_SHA256,
                        "exact_official_stock_binary_hash")
    version = observation.run([args.godot, "--version"])
    observation.require(version.returncode == 0 and version.stdout.decode().strip() == observation.VERSION,
                        "exact_official_stock_version")
    args.candidate_version = observation.VERSION
    args.candidate_engine_hash = observation.ENGINE_HASH
    native = args.native_fault_addon
    library = native / "libeditor_integration.macos.arm64.dylib"
    manifest = native / "build-manifest.json"
    observation.require(native.is_absolute() and native.is_dir() and not native.is_symlink() and
                        library.is_file() and manifest.is_file() and
                        (native / "editor_integration.gdextension").is_file(),
                        "exact_absolute_fixture_native_artifact_directory")
    receipt = json.loads(manifest.read_text())
    observation.require(receipt.get("fixture_only") is True and
                        receipt.get("engine_sha256") == STOCK_SHA256 and
                        receipt.get("native_library_sha256") == observation.digest(library),
                        "fixture_controls_never_installed_from_product_binary")
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-script-open-", dir=Path.home()) as temporary:
        harness = OpeningHarness(args, Path(temporary))
        status = 0
        try:
            harness.initialize()
            selected = SCENARIOS if args.scenario == "all" else (args.scenario,)
            for scenario in selected:
                harness.group(scenario, getattr(harness, scenario.replace("-", "_")))
            observation.require(len({(case.get("artifact_directory"), case["case"])
                                     for case in harness.cases}) == len(harness.cases),
                                "opening_campaign_unique_real_case_records")
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError, EOFError,
                subprocess.SubprocessError) as error:
            status = 1
            harness.summary.update({"status": "failed", "stage": str(error) if
                                    isinstance(error, observation.Failure) else type(error).__name__})
        finally:
            try:
                harness.cleanup()
                harness.verify_incidental_redaction()
            except (observation.Failure, OSError) as error:
                status = 1
                harness.summary.update({"status": "failed", "cleanup_stage": str(error) if
                                        isinstance(error, observation.Failure) else type(error).__name__})
            harness.summary["artifacts_sha256"] = {
                str(path.relative_to(args.artifacts)): observation.digest(path)
                for path in sorted(args.artifacts.rglob("*")) if path.is_file()}
            harness.summary["passed_case_count"] = len(harness.cases) if status == 0 else 0
            opening_calls = [case for case in harness.cases if case.get("operation") == "open_gdscript"]
            harness.summary["public_opening_call_count"] = len(opening_calls)
            harness.summary["public_outcome_counts"] = {
                outcome: sum(case["outcome"] == outcome for case in opening_calls) for outcome in OPEN_CODES}
            harness.summary["maximum_public_call_seconds"] = max(
                (case["elapsed_seconds"] for case in opening_calls), default=None)
            harness.summary["cumulative_acceptance"] = {
                scenario: "passed" if status == 0 and scenario in harness.summary["groups"] else
                "failed" if args.scenario in (scenario, "all") else "not_run"
                for scenario in ("sequential", "composed")}
            observation.json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps({"status": harness.summary["status"], "stage": harness.summary.get("stage"),
                          "passed_cases": harness.summary["passed_case_count"],
                          "coverage_scope": harness.summary["coverage_scope"],
                          "summary": str(args.artifacts / "summary.json")}))
        return status


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except observation.Failure as error:
        print("Opening acceptance prerequisite failed: " + str(error))
        raise SystemExit(1)
