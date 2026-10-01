#!/usr/bin/env python3
"""Eight individually runnable public discovery acceptance groups (Feature 004 T003)."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import shutil
import signal
import stat
import subprocess
import tempfile
import time

import run_observation as observation
from run_script_open import OpeningHarness
from run_script_edit import STOCK_SHA256
from discovery_live_cases import DiscoveryLiveCases

SCENARIOS = ("inventory", "routing", "coverage", "interruption", "readonly",
             "sequential", "composed", "privacy-export")
FIXTURE = Path(__file__).parent / "fixtures/script_discovery"
EXITS = {"complete_listing": 0, "limited_listing": 2, "refused": 3, "interrupted": 4}
EXCLUSIONS = ["dot_names", "project_data_directory", "gdignore_subtrees", "nested_projects",
              "non_gdscript_files", "non_regular_files", "embedded_or_unsaved_documents"]


class DiscoveryHarness(DiscoveryLiveCases, OpeningHarness):
    def __init__(self, args, work):
        super().__init__(args, work)
        self.discovery_processes = []
        self.discovery_requests = set()
        self.expected = {}
        self.source_markers.add(b"DISCOVERY_PRIVATE_SOURCE_BODY")
        self.summary.update({"coverage_scope": ("complete_discovery_groups" if args.scenario == "all"
                                               else "selected_discovery_group"),
                             "public_discover_gdscripts": True, "support_claim": False,
                             "discoverer_sha256": observation.digest(args.discoverer),
                             "driver_sha256": observation.digest(Path(__file__)),
                             "discovery_fixture_files": {str(path.relative_to(FIXTURE)): observation.digest(path)
                                                         for path in sorted(FIXTURE.rglob("*")) if path.is_file()},
                             "discovery_cases_sha256": observation.digest(Path(__file__).with_name("discovery_live_cases.py"))})
        self.summary["product_opening_caller_acceptance"] = args.scenario in ("readonly", "interruption", "composed", "all")
        self.summary["acceptance_coverage"] = {
            "inventory": ["exact_100_scripts_10_folders", "native_visibility", "both_effective_data_modes",
                          "empty_and_hidden_only_empty", "source_independent_eligibility"],
            "routing": ["unique_ambiguous_exact_ended_sessions", "authentication_denial",
                        "root_ancestor_mode_acl", "outside_symlink", "root_replacement_before_enumeration"],
            "coverage": ["fresh_create_rename_remove", "idle_cache_negative_control", "unreadable_entries",
                         "unsupported_names_markers", "fixed_bounds", "zero_limited", "diagnostic_overflow",
                         "epoch_scope_scan_import_invalidations"],
            "interruption": ["begin_recheck_barriers", "sigint_sigterm_eof_disable_expiry_timeout",
                             "stalled_owned_worker_and_editor", "malformed_identity_oversized_frames",
                             "late_release_no_replay", "shared_slot_both_directions"],
            "readonly": ["clean_dirty_equal_dirty_divergent_nonselected_unloaded", "native_undo_redo",
                         "first_use_clean_fresh_observe_open_edit_and_dirty_refusal",
                         "existing_focused_history_save_reopen_reparse_rescan_runtime",
                         "ignored_deleted_known_document_observation"],
            "sequential": ["twenty_fresh_complete_requests", "intentional_create_rename_remove_human_history",
                           "independent_D_R_B_identity_versions_selection_unloaded",
                           "controlled_access_limitations", "actual_prior_undo_redo", "delayed_no_effects"],
            "composed": ["discovered_closed_locator_before_product_open",
                         "complete_existing_A_E_dirty_different_equal_history_durability",
                         "Save_close_reopen_reparse_rescan_runtime", "changed_removed_ended_known_documents"],
            "privacy-export": ["selected_unselected_source_credentials", "authorized_ambiguous_denied_interrupted",
                               "result_only_review", "enabled_disabled_hook_only_actual_exports"]}

    def initialize(self):
        super().initialize()
        for flag in ("--help", "--version"):
            response = observation.run([self.args.discoverer, flag])
            observation.require(response.returncode == 0 and not response.stderr and response.stdout,
                                "discovery_bootstrap_" + flag)
            self.safe_log("discoverer-" + flag[2:] + ".log", response.stdout)
        self.case("public_discovery_help_version_no_project_access")

    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        helper = project / "addons/fixture_driver"
        if name.startswith("export-"):
            # Export preparation disables private drivers; retaining their real
            # sources lets pack inspection prove their exclusion as well.
            shutil.copy2(Path(__file__).parent / "fixtures/script_open/fixture_driver.gd",
                         helper / "open_fixture_driver.gd")
        shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "discovery_fixture_driver.gd")
        shutil.copy2(FIXTURE / "scope_fixture.gd", helper / "discovery_scope_fixture.gd")
        if name.startswith("export-"):
            return project
        (helper / "plugin.gd").write_text('@tool\nextends "res://addons/fixture_driver/discovery_fixture_driver.gd"\n')
        (project / "scripts/.gdignore").unlink()
        bridge = project / "addons/godot_agent_kit/bridge.gd"
        text = bridge.read_text()
        old = 'preload("res://addons/godot_agent_kit/script_discovery.gd")'
        observation.require(text.count(old) == 1, "owned_scope_getter_seam")
        bridge.write_text(text.replace(old, 'preload("res://addons/fixture_driver/discovery_scope_fixture.gd")'))
        # Installation manifest, independently prepared before any public call.
        # Include all visible production and fixture scripts, not just catalog.
        self.expected[project] = {"res://" + str(path.relative_to(project))
                                  for path in project.rglob("*.gd")
                                  if "scripts/native/cold/" not in str(path.relative_to(project))}
        return project

    def add_script(self, project, relative, source="extends RefCounted\n# DISCOVERY_PRIVATE_SOURCE_BODY\n", *, included=True):
        path = project / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)
        if included:
            self.expected[project].add("res://" + relative)
        return path

    @contextmanager
    def live(self, name, *, setup=None, visible=False):
        project = self.fixture("discovery-" + name, controlled=True)
        if visible:
            config = project / "project.godot"
            config.write_text(config.read_text().replace("[application]", "[application]\nconfig/use_hidden_project_data_directory=false"))
        if setup:
            setup(project)
        editor = self.start_editor(project)
        try:
            descriptor = observation.wait_for(lambda: next(iter(self.descriptors(project)), None), "discovery_live_descriptor")
            self.settled(editor)
            yield project, editor, descriptor
        finally:
            self.close_editor(editor)
            self.editors.remove(editor)

    def settled(self, editor):
        def idle():
            oracle = self.open_action(editor, "discovery_oracle")
            return oracle if not oracle["scanning"] and not oracle["importing"] else None
        return observation.wait_for(idle, "native_inventory_settled", timeout=60)

    def start_discovery(self, project, descriptor=None, *, registry=None, extra=()):
        command = [str(self.args.discoverer), "--registry", str(registry or self.registry), "--project", str(project)]
        if descriptor is not None:
            command += ["--session", descriptor["session_id"]]
        command.extend(str(value) for value in extra)
        started = time.monotonic()
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.discovery_processes.append(process)
        process.discovery_binding = (str(project), descriptor["session_id"] if descriptor else None)
        return process, started

    def finish_discovery(self, process, started, name, *, outcome=None, reason=None, expected=None):
        stdout, stderr = process.communicate(timeout=max(0.1, 5.2 - (time.monotonic() - started)))
        elapsed = time.monotonic() - started
        observation.require(elapsed <= 5, "discovery_five_second_deadline_" + name)
        observation.require(len(stdout) <= 6 * 1024 * 1024 + 1 and stdout.endswith(b"\n") and stdout.count(b"\n") == 1,
                            "bounded_one_json_result_" + name)
        observation.require(b"res://" not in stderr and str(self.registry).encode() not in stderr,
                            "incidental_discovery_stderr_has_no_paths_" + name)
        self.safe_log(name + ".stderr", stderr)
        self.safe_log(name + ".json", stdout)
        result = json.loads(stdout)
        self.review_result(result, process.returncode, name)
        requested_project, requested_session = process.discovery_binding
        if result["requested_target"] is not None:
            observation.require(result["requested_target"] ==
                                {"project_root": requested_project, "session_id": requested_session},
                                "independent_requested_locator_" + name)
        target = result["resolved_target"]
        if target is not None:
            requested_project, requested_session = process.discovery_binding
            observation.require(target["project_root"] == requested_project and
                                (requested_session is None or target["session_id"] == requested_session),
                                "independent_requested_target_binding_" + name)
        if outcome:
            observation.require(result["outcome"] == outcome, "discovery_outcome_" + name)
        if reason:
            observation.require(reason in {item["code"] for item in result["diagnostics"]}, "discovery_reason_" + name)
        if expected is not None:
            observation.require(result["inventory"] is not None and result["inventory"]["entries"] == sorted(expected, key=lambda path: path.encode()),
                                "independent_exact_inventory_" + name)
        self.case(name, operation="discover_gdscripts", outcome=result["outcome"], elapsed_seconds=elapsed,
                  entry_count=len(result["inventory"]["entries"]) if result["inventory"] else None,
                  reasons=[item["code"] for item in result["diagnostics"]], result_only_review=True)
        return result

    def discover(self, project, descriptor, name, **checks):
        return self.finish_discovery(*self.start_discovery(project, descriptor), name, **checks)

    def review_result(self, result, code, name):
        observation.require(set(result) == {"schema_version", "request_id", "outcome", "interval", "requested_target", "resolved_target", "inventory", "diagnostics", "selection"}, "exact_discovery_envelope_" + name)
        request = result["request_id"]
        observation.require(result["schema_version"] == 1 and isinstance(request, str) and len(request) == 32 and
                            all(char in "0123456789abcdef" for char in request) and request not in self.discovery_requests,
                            "fresh_discovery_request_" + name)
        self.discovery_requests.add(request)
        observation.require(result["outcome"] in EXITS and code == EXITS[result["outcome"]], "discovery_exit_meaning_" + name)
        interval = result["interval"]
        observation.require(set(interval) == {"started_unix_ms", "finished_unix_ms", "elapsed_us"} and
                            all(type(value) is int and value >= 0 for value in interval.values()) and
                            interval["elapsed_us"] <= 5_000_000, "bounded_descriptive_interval_" + name)
        selection = result["selection"]
        if selection is not None:
            observation.require(set(selection) == {"candidate_sessions", "missing_selector"} and
                                selection["missing_selector"] == "session_id" and
                                all(isinstance(session, str) and len(session) == 32 and
                                    all(char in "0123456789abcdef" for char in session)
                                    for session in selection["candidate_sessions"]),
                                "safe_session_only_result_feedback_" + name)
        observation.require(len(result["diagnostics"]) <= 64, "bounded_diagnostics_" + name)
        for diagnostic in result["diagnostics"]:
            observation.require(set(diagnostic) == {"code", "stage", "scope", "message", "action", "omitted_count"} and
                                diagnostic["message"] and diagnostic["action"] and diagnostic["stage"] in
                                {"validate_request", "resolve_target", "authenticate", "begin_scope", "enumerate", "recheck", "finalize"},
                                "actionable_result_only_diagnostic_" + name)
            scope = diagnostic["scope"]
            observation.require(scope is None or (scope.startswith("res://") and
                                not any(char in scope for char in ("?", "#", "\\", "\n", "\r"))),
                                "safe_diagnostic_scope_" + name)
        inventory = result["inventory"]
        if result["outcome"] == "refused":
            observation.require(inventory is None and result["diagnostics"], "refusal_without_inventory_" + name)
        if inventory is None:
            return
        target = result["resolved_target"]
        observation.require(target is not None and target["request_id"] == request and target["session_id"] and
                            set(inventory) == {"scope", "entries", "collection", "coverage", "validity", "consistency", "visited_entries", "visited_directories"}, "attributed_inventory_" + name)
        observation.require(set(target) == {"request_id", "project_root", "project_file_id", "session_id", "godot_version"} and
                            set(target["project_file_id"]) == {"device", "inode"} and
                            target["godot_version"] == {"version": self.version, "hash": self.engine_hash},
                            "exact_authenticated_target_schema_" + name)
        stamp = inventory["collection"]
        observation.require(set(stamp) == {"clock_id", "started_tick_us", "finished_tick_us", "received_elapsed_us"} and
                            stamp["clock_id"] == "caller" and
                            int(stamp["started_tick_us"]) <= int(stamp["finished_tick_us"]) and
                            0 <= stamp["received_elapsed_us"] <= 5_000_000,
                            "separate_caller_collection_clock_" + name)
        scope = inventory["scope"]
        observation.require(scope == {"policy": "godot_project_files_v1", "project_data_directory": scope["project_data_directory"], "exclusions": EXCLUSIONS} and
                            scope["project_data_directory"] in ("res://.godot", "res://godot"), "result_only_scope_" + name)
        entries = inventory["entries"]
        observation.require(len(entries) <= 1024 and entries == sorted(set(entries), key=lambda path: path.encode()) and
                            all(path.startswith("res://") and len(path.encode()) <= 2048 for path in entries) and
                            inventory["visited_entries"] <= 16384 and inventory["visited_directories"] <= 1024,
                            "inventory_bounds_identity_" + name)
        observation.require(inventory["consistency"]["atomic"] is False and inventory["consistency"]["stability"] in ("unknown", "changed"), "no_atomic_claim_" + name)
        if result["outcome"] == "complete_listing":
            observation.require(inventory["coverage"] == "complete" and inventory["validity"] == "observed" and
                                inventory["consistency"] == {"recheck": "completed", "stability": "unknown", "atomic": False} and not result["diagnostics"], "complete_evidence_" + name)
        else:
            observation.require(inventory["coverage"] != "complete" and result["diagnostics"], "no_false_complete_" + name)
        if result["outcome"] == "interrupted":
            observation.require(inventory["validity"] == "earlier_observation" and inventory["consistency"]["recheck"] == "unavailable", "interrupted_earlier_evidence_" + name)

    def gate(self, editor, stage, *, fault=""):
        self.open_action(editor, "discovery_hold", stage=stage, fault=fault)

    def wait_gate(self, editor, stage):
        path = editor["control"] / "discovery-event.json"
        def reached():
            if path.is_file():
                event = json.loads(path.read_text())
                return event if event["stage"] == stage else None
            return None
        return observation.wait_for(reached, "actual_discovery_" + stage + "_barrier", timeout=3)

    def release(self, editor):
        self.open_action(editor, "discovery_release")

    def cleanup(self):
        for process in self.discovery_processes:
            if process.poll() is None:
                process.send_signal(signal.SIGTERM)
            try:
                process.communicate(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=2)
        super().cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ("godot", "discoverer", "observer", "opener", "editor", "stock-validator"):
        parser.add_argument("--" + flag, type=Path, required=True)
    parser.add_argument("--native-fault-addon", type=Path)
    parser.add_argument("--scenario", choices=("all",) + SCENARIOS, required=True)
    parser.add_argument("--artifacts", type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    for path in (args.godot, args.discoverer, args.observer, args.opener, args.editor, args.stock_validator):
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK), "absolute_executable_inputs")
    observation.require(observation.digest(args.godot) == STOCK_SHA256, "exact_stock_godot_hash")
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and not args.artifacts.is_symlink() and
                        args.artifacts.stat().st_uid == os.geteuid() and stat.S_IMODE(args.artifacts.stat().st_mode) == 0o700 and
                        not list(args.artifacts.iterdir()), "fresh_private_artifact_directory")
    args.candidate_version, args.candidate_engine_hash = observation.VERSION, observation.ENGINE_HASH
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-discovery-", dir=Path.home()) as temporary:
        harness = DiscoveryHarness(args, Path(temporary))
        status = 0
        try:
            harness.initialize()
            for scenario in SCENARIOS if args.scenario == "all" else (args.scenario,):
                harness.group(scenario, getattr(harness, scenario.replace("-", "_")))
            harness.verify_incidental_redaction()
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError, EOFError, subprocess.SubprocessError) as error:
            status = 1
            harness.summary.update({"status": "failed", "stage": str(error) if isinstance(error, observation.Failure) else type(error).__name__})
        finally:
            try:
                harness.cleanup()
                harness.verify_incidental_redaction()
            except (observation.Failure, OSError) as error:
                status = 1
                harness.summary.update({"status": "failed", "cleanup_stage": str(error) if isinstance(error, observation.Failure) else type(error).__name__})
            calls = [case for case in harness.cases if case.get("operation") == "discover_gdscripts"]
            harness.summary.update({"passed_case_count": len(harness.cases) if status == 0 else 0,
                                    "public_discovery_call_count": len(calls),
                                    "maximum_public_call_seconds": max((case["elapsed_seconds"] for case in calls), default=None),
                                    "public_outcome_counts": {outcome: sum(case["outcome"] == outcome for case in calls) for outcome in EXITS}})
            observation.json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps({"status": harness.summary["status"], "stage": harness.summary.get("stage"),
                          "passed_cases": harness.summary["passed_case_count"], "summary": str(args.artifacts / "summary.json")}))
        return status


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except observation.Failure as error:
        print("Discovery acceptance prerequisite failed: " + str(error))
        raise SystemExit(1)
