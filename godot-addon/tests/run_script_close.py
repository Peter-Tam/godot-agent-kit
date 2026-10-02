#!/usr/bin/env python3
"""T001 owned native-boundary close evidence; never a public close caller."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import math
import os
from pathlib import Path
import secrets
import shutil
import stat
import subprocess
import tempfile
import time

import run_observation as observation
from run_script_edit import STOCK_SHA256
from run_script_open import OpeningHarness
from close_native_acceptance import CloseNativeAcceptanceMixin, documents, facts

FIXTURE = Path(__file__).parent / "fixtures/script_close"
TARGET = "res://scripts/subject.gd"
CURRENT = "res://scripts/other.gd"
BACKGROUND = "res://scripts/close/background.gd"
SAFE = "extends RefCounted\nfunc value() -> int:\n\treturn 47\n"


class CloseHarness(CloseNativeAcceptanceMixin, OpeningHarness):
    def __init__(self, args, work):
        super().__init__(args, work)
        self.summary.update(coverage_scope="T001_native_boundary_only",
                            product_close_acceptance=False, public_close_gdscript=False,
                            product_opening_caller_acceptance=False,
                            driver_sha256=observation.digest(Path(__file__)),
                            close_acceptance_sha256=observation.digest(
                                Path(__file__).with_name("close_native_acceptance.py")),
                            close_fixture_files={str(p.relative_to(FIXTURE)): observation.digest(p)
                                                 for p in sorted(FIXTURE.rglob("*")) if p.is_file()})
        self.summary["acceptance_coverage"] = {
            "native-boundary": ["guarded_one_shot_close", "independent_target_and_protection_witnesses",
                                "actual_native_visits_and_completion", "real_protected_history",
                                "bounded_source_context_and_receipts", "identity_and_configuration_races",
                                "owned_callback_retirement", "privacy_and_production_exports"]}
        self.source_markers.update((b"CLOSE_PRIVATE_TARGET", b"CLOSE_PRIVATE_CURRENT",
                                    b"CLOSE_PRIVATE_BACKGROUND", b"CLOSE_HUMAN"))

    def case(self, name, **evidence):
        super().case(name, **evidence)
        # Native cases must not inherit an unobserved "not_acquired" assertion.
        if "source_surfaces" not in evidence:
            self.cases[-1].pop("source_surfaces", None)

    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        if name.startswith("export-"):
            return project
        helper = project / "addons/fixture_driver"
        shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "close_fixture_driver.gd")
        (helper / "plugin.gd").write_text('@tool\nextends "res://addons/fixture_driver/close_fixture_driver.gd"\n')
        shutil.copytree(FIXTURE / "scripts", project / "scripts/close")
        return project

    def close_action(self, editor, action, **arguments):
        return self.open_action(editor, action, **arguments)

    @contextmanager
    def close_fixture(self, name, *, source=SAFE, paths=None, selected=TARGET,
                      retained=False, read_only=False, faults=False, setup=None, replacement_target=False):
        project = self.fixture("close-" + name, controlled=True)
        (project / "scripts/subject.gd").write_text(source)
        (project / "scripts/other.gd").write_text(SAFE + "# CLOSE_PRIVATE_CURRENT\n")
        if setup:
            setup(project)
        if faults:
            shutil.copytree(self.args.native_fault_addon, project / "addons/godot_agent_kit/native",
                            dirs_exist_ok=True)
        self.installed_native(project, fixture_only=faults)
        if read_only:
            (project / "scripts/subject.gd").chmod(0o444)
        editor = self.start_editor(project)
        try:
            descriptor = observation.wait_for(lambda: next(iter(self.descriptors(project)), None),
                                              "owned_close_editor_session")
            info = self.close_action(editor, "close_info")
            observation.require(info["api_revision"] == 3 and
                                info["build_id"] == self.expected_native_build_id and
                                info["session_id"] == descriptor["session_id"] and descriptor["v"] == 5,
                                "matched_native_revision3_bridge5")
            stream, challenge = self.challenge(descriptor)
            stream.close()
            observation.require(challenge["capabilities"]["close_gdscript"] is False,
                                "private_fixture_never_advertises_public_close")
            if faults:
                observation.require(info["fixture_faults"], "separate_native_fault_artifact")
                self.close_action(editor, "close_fixture_activate")
            self.close_action(editor, "close_setup", paths=paths if paths is not None else
                              [TARGET, CURRENT, BACKGROUND], selected=selected,
                              retain_target=retained, replacement_target=replacement_target)
            self.present_editor(editor)
            yield project, editor, descriptor
        finally:
            self.close_editor(editor)
            self.editors.remove(editor)

    def state(self, editor, project):
        state = self.close_action(editor, "close_state")["state"]
        disks = {path: observation.disk_witness(project / path.removeprefix("res://"))
                 if (project / path.removeprefix("res://")).is_file() else {"availability": "missing"}
                 for path in state["open_paths"] if path.startswith("res://")}
        if TARGET not in disks and (project / "scripts/subject.gd").is_file():
            disks[TARGET] = observation.disk_witness(project / "scripts/subject.gd")
        return state, disks

    def rust_close(self, operation, project, descriptor, request_id, **fields):
        envelope = dict(operation=operation, project=str(project), script=TARGET,
                        request_id=request_id, session_id=descriptor["session_id"], **fields)
        response = subprocess.run([str(self.args.stock_validator)],
                                  input=json.dumps(envelope).encode(), capture_output=True, timeout=12)
        observation.require(response.returncode == 0 and not response.stderr,
                            "private_rust_close_envelope_no_incidental_output")
        observation.require(len(response.stdout) <= 12 * 1024 * 1024,
                            "private_rust_close_bounded_output")
        return json.loads(response.stdout)

    def inspect_close(self, editor, descriptor, *, request_id=None, budget_us=9000000,
                      session_id=None):
        request_id = request_id or secrets.token_hex(16)
        started = time.monotonic()
        tick = self.close_action(editor, "close_info")["editor_tick_us"]
        budget_us -= int((time.monotonic() - started) * 1_000_000)
        observation.require(budget_us > 0, "original_cutoff_before_owned_inspection")
        result = self.close_action(editor, "close_inspect", path=TARGET,
                                  correlation={"request_id": request_id,
                                               "session_id": session_id or descriptor["session_id"],
                                               "expiry_tick_us": str(int(tick) + budget_us)})["result"]
        return request_id, result

    def prepare_close(self, project, editor, descriptor, *, budget_us=9000000,
                      alter_basis=None, alter_capture=None):
        started = time.monotonic()
        before, disks = self.state(editor, project)
        request_id = secrets.token_hex(16)
        captured = self.rust_close("close_capture", project, descriptor, request_id)
        collected = self.close_action(editor, "close_collect", path=TARGET)
        sample = collected["sample"]
        identity = sample["document"]["identity"]
        doc = before["target"]
        observation.require(identity["script_instance_id"] == doc["script_id"] and
                            identity["editor_instance_id"] == doc["editor_id"] and
                            identity["buffer_instance_id"] == doc["buffer_id"] and
                            (sample["R"]["availability"] != "observed" or
                             sample["R"]["text"] == doc.get("R")) and
                            (sample["B"]["availability"] != "observed" or
                             sample["B"]["text"] == doc.get("B")) and
                            captured["source"] == disks[TARGET]["text"],
                            "independent_rust_D_ordinary_R_B_and_private_identity")
        basis = {key: identity[key] for key in
                 ("script_instance_id", "editor_instance_id", "buffer_instance_id")}
        basis["current_version"] = str(doc["version"])
        if alter_basis:
            alter_basis(basis)
        capture = {key: captured[key] for key in
                   ("project_device", "project_inode", "target_device", "target_inode", "source_sha256")}
        capture.update(utf8_bytes=int(captured["source_length"]), basis=basis)
        if alter_capture:
            alter_capture(capture)
        remaining_us = budget_us - int((time.monotonic() - started) * 1_000_000)
        request_id, inspected = self.inspect_close(editor, descriptor, request_id=request_id, budget_us=remaining_us)
        observation.require(inspected["status"] == "inspected", "native_passive_open_inspection")
        prepared = self.close_action(editor, "close_prepare", request_id=request_id,
                                     captured_source=captured["source"], capture_and_basis=capture)["result"]
        return dict(project=project, editor=editor, descriptor=descriptor, request_id=request_id,
                    before=before, disks=disks, capture=captured, prepared=prepared,
                    cutoff=started + budget_us / 1_000_000)

    def close_validation_fields(self, attempt):
        prepared = attempt["prepared"]
        records = prepared["context"]["documents"]
        entries = prepared["validation"]
        observation.require(len(entries) == len(records), "actual_sorted_validation_request_complete")
        for entry, record in zip(entries, records):
            p = record["projection"]
            observation.require(entry["document_path"] == p["path"] and
                                all(entry[key] == p[key] for key in ("script_id", "editor_id", "buffer_id")) and
                                entry["warnings"]["enable"] == p["warnings"]["enable"] and
                                entry["warnings"]["levels"] == p["warnings"]["levels"] and
                                entry["warnings"]["directory_rules"] == p["warnings"]["directory_rules"] and
                                entry["global_classes"] == p["global_classes"] and
                                entry["warnings"]["provenance"] == {
                                    "source": "editor_project_settings", "project_root": str(attempt["project"]),
                                    "session_id": attempt["descriptor"]["session_id"]} and
                                Path(entry["executable"]).resolve() == self.args.godot.resolve(),
                                "actual_validator_entry_bound_to_each_captured_document")
        if entries:
            first = entries[0]
            observation.require(all(all(entry[key] == first[key] for key in
                                        ("executable", "warnings", "global_classes")) for entry in entries),
                                "one_actual_project_effective_validation_environment")
            return dict(binary=first["executable"], warnings=first["warnings"], global_classes=first["global_classes"])
        context = attempt["before"]["effective_context"]
        return dict(binary=str(self.args.godot), warnings={**context["warnings"], "provenance": {
            "source": "editor_project_settings", "project_root": str(attempt["project"]),
            "session_id": attempt["descriptor"]["session_id"]}}, global_classes=context["global_classes"])

    def authorize_close(self, attempt, *, allow_unavailable=False):
        prepared = attempt["prepared"]
        if prepared.get("status") != "prepared":
            self.summary["failed_close_preparation"] = {
                "status": prepared.get("status"), "reason": prepared.get("reason"),
                "phase": (prepared.get("native") or {}).get("phase")}
        observation.require(prepared["status"] == "prepared", "real_close_preparation_required")
        context = prepared["context"]
        projection = context["projection"]
        state = attempt["before"]
        documents = {doc["path"]: doc for doc in state["documents"]}
        observation.require({entry["path"] for entry in projection["roster"]} == set(state["open_paths"]) and
                            {entry["path"] for entry in projection["protected"]} ==
                            set(state["open_paths"]) - {TARGET}, "complete_independently_observed_roster")
        observation.require(len(projection["roster"]) == len(state["documents"]) == len(documents) and
                            len(projection["protected"]) == len(context["documents"]) == len(documents) - 1 and
                            all(len({d[key] for d in state["documents"]}) == len(documents)
                                for key in ("script_id", "editor_id", "buffer_id")) and
                            projection["idle_parse_delay_us"] ==
                            str(math.ceil(float(state["effective_context"]["idle_parse_delay"]) * 1_000_000)) and
                            projection["idle_parse_error_delay_us"] ==
                            str(math.ceil(float(state["effective_context"]["idle_parse_error_delay"]) * 1_000_000)),
                            "unique_complete_roster_and_actual_idle_configuration")
        for record in context["documents"]:
            p = record["projection"]
            d = documents[p["path"]]
            observation.require(record["source"] == d["R"] == d["B"] and
                                all(p[key] == d[key] for key in
                                    ("script_id", "editor_id", "buffer_id", "dirty", "resource_edited",
                                     "has_undo", "has_redo")) and
                                p["version"] == str(d["version"]) and
                                p["saved_version"] == str(d["saved_version"]),
                                "protected_real_source_identity_flags_history")
        remaining = min(9000, int((attempt["cutoff"] - time.monotonic()) * 1000))
        observation.require(remaining > 0, "original_caller_budget_before_validation")
        result = self.rust_close("close_validate", attempt["project"], attempt["descriptor"],
                                 attempt["request_id"], **self.close_validation_fields(attempt),
                                 remaining_budget_ms=remaining, context=context,
                                 guard_sha256=prepared["guard_sha256"])
        if allow_unavailable and result["status"] == "unavailable":
            observation.require(result["reason"] is not None and not result["receipt_bindings"] and
                                result["guard_sha256"] == prepared["guard_sha256"] and
                                all(child["cleanup_confirmed"] and
                                    (not child["child_spawned"] or child["child_reaped"])
                                    for child in result["results"]),
                                "maximum_bound_explicit_validation_unavailable_owned_cleanup_no_authorization")
            return result
        observation.require(result["status"] == "valid" and
                            result["guard_sha256"] == prepared["guard_sha256"] and
                            len(result["results"]) == len(context["documents"]) and
                            len(result["receipt_bindings"]) == len(context["documents"]),
                            "independent_rust_aggregate_and_actual_sequential_children")
        for child, record in zip(result["results"], context["documents"]):
            p = record["projection"]
            observation.require(child["purpose"] == "close_context" and child["status"] == "valid" and
                                child["cleanup_confirmed"] is True and child["child_spawned"] is True and
                                child["child_reaped"] is True and len(child["sources"]) == 1 and
                                child["sources"][0]["path"] == p["path"] and
                                child["sources"][0]["sha256"] == p["source_sha256"] and
                                child["sources"][0]["utf8_bytes"] == int(p["source_length"]) and
                                child["sources"][0]["diagnostics_completed"] is True and
                                child["sources"][0]["symbols_completed"] is True,
                                "actual_close_context_child_exact_source_fences_owned_cleanup")
        attempt["authorization"] = result
        return result

    def advance_close(self, attempt, *, bindings=None, guard=None):
        auth = attempt["authorization"]
        observation.require(time.monotonic() < attempt["cutoff"], "caller_original_cutoff_before_authorization")
        return self.close_action(attempt["editor"], "close_advance", request_id=attempt["request_id"],
                                 guard_sha256=guard if guard is not None else auth["guard_sha256"],
                                 receipt_bindings=bindings if bindings is not None else auth["receipt_bindings"])["result"]

    def settle_close(self, attempt):
        return self.close_action(attempt["editor"], "close_wait", request_id=attempt["request_id"])["result"]

    def independent_close(self, attempt, *, expect=True):
        fresh = self.rust_close("close_recheck", attempt["project"], attempt["descriptor"],
                                attempt["request_id"], capture=attempt["capture"])
        collected = self.close_action(attempt["editor"], "close_collect", path=TARGET)
        state, disks = self.state(attempt["editor"], attempt["project"])
        before = attempt["before"]
        keys = ("path", "script_id", "editor_id", "buffer_id", "R", "B", "version",
                "saved_version", "dirty", "resource_edited", "has_undo", "has_redo")
        protected_before = {d["path"]: {k: d[k] for k in keys}
                            for d in before["documents"] if d["path"] != TARGET}
        protected_after = {d["path"]: {k: d[k] for k in keys} for d in state["documents"]}
        preserved = protected_before == protected_after and attempt["disks"] == disks
        sample = collected["sample"]
        closed = sample["document"]["open_state"].get("value") == "not_open" and not state["target"]["associated"]
        retained = sample["R"]["availability"] == "observed"
        unloaded = (sample["R"]["availability"] == "unavailable" and
                    sample["R"].get("reason", {}).get("code") == "resource_not_loaded" and
                    state["cached_id"] == "" and state["original_target_alive"] is False)
        resource_ok = unloaded or (retained and sample["R"]["text"] == attempt["capture"]["source"] and
                       sample["document"]["identity"]["script_instance_id"] == before["target"]["script_id"] and
                       state["cached_edited"] == before["target"]["resource_edited"])
        selection_ok = (before["selection"] == state["selection"] if before["selection"] != TARGET else
                        state["selection"] in protected_after if protected_after else state["selection"] == "")
        verified = fresh["unchanged"] and closed and preserved and resource_ok and selection_ok
        observation.require(verified is expect, "independent_final_facts_not_native_OK")
        return dict(verified=verified, resource="retained_original" if retained else "unloaded" if unloaded else "unavailable",
                    disk_unchanged=fresh["unchanged"], protected_preserved=preserved)

    def survivor_close(self, attempt, previous):
        before, disks = self.state(attempt["editor"], attempt["project"])
        tick = int(self.close_action(attempt["editor"], "close_info")["editor_tick_us"])
        result = self.close_action(attempt["editor"], "close_verify",
                                   request_id=attempt["request_id"], purpose="survivor")["result"]
        after, now = self.state(attempt["editor"], attempt["project"])
        native, old = facts(result), facts(previous)
        identity = result["sample"]["document"]["identity"]
        target = before["target"]
        expected_selection = ("no_source_editor" if not before["selection"] else
                              "target" if next(d for d in before["documents"]
                                               if d["path"] == before["selection"])["script_id"] ==
                              attempt["before"]["target"]["script_id"] else "other")
        observation.require(result["status"] == "observed" and
                            native["entered"] == old["entered"] and
                            native["close_error"] == old["close_error"] and
                            native["old_document_removed"] == old["old_document_removed"] and
                            all(native[key] == old[key] for key in
                                ("script_instance_id", "editor_instance_id", "buffer_instance_id")) and
                            (not old["invalidated"] or
                             native["invalidated"] and native["reason"] == old["reason"]) and
                            result["selection"]["after"] == expected_selection and
                            int(result["resource_state"]["collection"]["started_tick_us"]) >= tick and
                            identity["script_instance_id"] == target["script_id"] and
                            identity["editor_instance_id"] == target["editor_id"] and
                            identity["buffer_instance_id"] == target["buffer_id"] and
                            result["sample"]["R"]["text"] == target["R"] and
                            result["sample"]["B"]["text"] == target["B"] and
                            documents(before) == documents(after) and disks == now and
                            before["selection"] == after["selection"],
                            "fresh_scoped_survivor_facts_preserve_sticky_effects_and_actual_new_document")
        return result

    def terminate_close(self, attempt, action="close_finish"):
        return self.close_action(attempt["editor"], action, request_id=attempt["request_id"])["result"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("godot", "observer", "editor", "stock-validator", "native-fault-addon", "artifacts"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--scenario", required=True, choices=("native-boundary",))
    args = parser.parse_args()
    args.opener = None
    os.umask(0o077)
    for path in (args.godot, args.observer, args.editor, args.stock_validator):
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
                            "absolute_actual_executable_inputs")
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and
                        not args.artifacts.is_symlink() and not list(args.artifacts.iterdir()) and
                        args.artifacts.stat().st_uid == os.geteuid() and
                        stat.S_IMODE(args.artifacts.stat().st_mode) == 0o700,
                        "owned_empty_private_artifact_directory")
    observation.require(observation.digest(args.godot) == STOCK_SHA256, "pinned_official_stock_binary")
    args.candidate_version = observation.VERSION
    args.candidate_engine_hash = observation.ENGINE_HASH
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-script-close-", dir=Path.home()) as directory:
        harness = CloseHarness(args, Path(directory))
        code = 0
        try:
            harness.initialize()
            harness.group("native-boundary", harness.native_boundary)
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError, EOFError,
                subprocess.SubprocessError) as error:
            code = 1
            harness.summary.update(status="failed", stage=str(error) if isinstance(error, observation.Failure)
                                   else type(error).__name__)
        finally:
            try:
                harness.cleanup()
                harness.verify_incidental_redaction()
            except (observation.Failure, OSError) as error:
                code = 1
                harness.summary.update(status="failed", cleanup_stage=str(error))
            harness.summary["passed_case_count"] = len(harness.cases) if not code else 0
            observation.json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps(dict(status=harness.summary["status"], stage=harness.summary.get("stage"),
                              passed_cases=harness.summary["passed_case_count"],
                              coverage_scope=harness.summary["coverage_scope"],
                              summary=str(args.artifacts / "summary.json"))))
        return code


if __name__ == "__main__":
    raise SystemExit(main())
