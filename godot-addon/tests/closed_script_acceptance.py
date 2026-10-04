"""Independent real-editor T001 proof through the supervised workflow consumer.

The driver is not MCP. Setup controls never stand in for product execution, and
postconditions come from confined disk reads plus public cache/editor witnesses.
"""
from __future__ import annotations

from contextlib import contextmanager
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import struct
import subprocess
import time

import run_observation as observation
from close_native_acceptance import documents
from run_script_close import TARGET, CURRENT, BACKGROUND, SAFE
from run_script_edit import sha

FIXTURE = Path(__file__).parent / "fixtures/mcp"
CHANGED = "extends RefCounted\nfunc value() -> int:\n\treturn 83\n"
SOURCE_LIMIT = 512 * 1024


def source_free(state, disks):
    """Persist hashes, never independent private source witnesses."""
    return {
        "documents": {path: {key: sha(value) if key in ("R", "B") and isinstance(value, str) else value
                              for key, value in doc.items()} for path, doc in documents(state).items()},
        "disk": {path: {key: value for key, value in disk.items() if key != "text"}
                 for path, disk in disks.items()},
        "cached_id": state["cached_id"],
        "cached_source_sha256": sha(state["cached_R"]) if isinstance(state["cached_R"], str) else None,
        "cached_edited": state["cached_edited"],
        "cache_has": state["cache_has"], "cache_type": state["cache_type"],
        "selection": state["selection"], "open_paths": state["open_paths"],
        "loader_calls": state["loader_calls"],
    }


class ClosedScriptAcceptanceMixin:
    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        if not name.startswith("export-"):
            helper = project / "addons/fixture_driver"
            shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "closed_fixture_driver.gd")
            (helper / "plugin.gd").write_text(
                '@tool\nextends "res://addons/fixture_driver/closed_fixture_driver.gd"\n')
        return project

    def state(self, editor, project):
        state = self.close_action(editor, "closed_witness")["state"]
        observation.require(state["cache_has"] == bool(state["cached_id"]) and
                            (state["cache_type"] == "GDScript" if state["cache_has"] else state["cache_type"] is None),
                            "independent_agreeing_public_cache_getters")
        paths = [path for path in state["open_paths"] if path.startswith("res://")]
        if TARGET not in paths:
            paths.append(TARGET)
        disks = {path: observation.disk_witness(project / path.removeprefix("res://"))
                 for path in paths if (project / path.removeprefix("res://")).is_file()}
        return state, disks

    @contextmanager
    def closed_fixture(self, name, *, cached=False, source=SAFE, faults=False, setup=None):
        paths = [TARGET, CURRENT, BACKGROUND] if cached else [CURRENT, BACKGROUND]
        with self.close_fixture(name, source=source, paths=paths, selected=CURRENT,
                                retained=cached, faults=faults, setup=setup) as fixture:
            project, editor, descriptor = fixture
            if faults:
                self.close_action(editor, "closed_fixture_activate")
            info = self.close_action(editor, "closed_info")
            observation.require(info["complete_family"] and info["fixture_faults"] is faults,
                                "separate_closed_fixture_fault_exports_and_complete_production_family")
            if cached:
                self.close_action(editor, "close_human", path=TARGET, mutation="close")
            state, disks = self.state(editor, project)
            observation.require(TARGET not in state["open_paths"] and
                                bool(state["cached_id"]) == cached and
                                (not cached or state["cached_R"] == source and state["cached_edited"] is False),
                                "independent_admitted_closed_cache_branch_" + name)
            stream, challenge = self.challenge(descriptor)
            stream.close()
            observation.require(challenge["capabilities"]["edit_closed_gdscript"] is True,
                                "authenticated_complete_closed_family")
            yield project, editor, descriptor

    def start_workflow(self, project, descriptor, operation, *, select_session=True, **fields):
        started = time.monotonic()
        request_id = secrets.token_hex(16)
        command = [str(self.args.workflow), "--registry", str(self.registry), "--project", str(project),
                   "--script", TARGET]
        if select_session:
            command.extend(["--session", descriptor["session_id"]])
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE)
        self.open_processes.append(process)
        payload = dict(operation=operation, request_id=request_id, **fields)
        try:
            process.stdin.write(json.dumps(payload, ensure_ascii=False).encode())
            process.stdin.close()
        except BrokenPipeError:
            pass
        process.stdin = None
        return process, started, request_id

    def complete_workflow(self, pending, name, operation):
        process, started, request_id = pending
        limit = 5 if operation == "read" else 10
        try:
            stdout, stderr = process.communicate(timeout=max(.01, limit + .2 - (time.monotonic() - started)))
        except subprocess.TimeoutExpired as error:
            process.kill()
            process.communicate(timeout=2)
            raise observation.Failure("supervised_workflow_deadline_" + name) from error
        finally:
            if process.poll() is not None and process in self.open_processes:
                self.open_processes.remove(process)
        elapsed = time.monotonic() - started
        observation.require(elapsed <= limit and len(stdout) <= 12 * 1024 * 1024 and
                            stdout.endswith(b"\n") and stdout.count(b"\n") == 1,
                            "bounded_single_supervised_result_" + name)
        observation.require(not stderr and all(secret not in stdout for secret in self.secrets),
                            "supervised_consumer_no_incidental_source_or_credentials_" + name)
        result = json.loads(stdout)
        observation.require(isinstance(result, dict), "typed_workflow_result_" + name)
        if operation == "edit":
            observation.require(all(marker not in stdout for marker in self.source_markers) and
                                CURRENT.encode() not in stdout and BACKGROUND.encode() not in stdout,
                                "edit_source_free_private_context_" + name)
        observation.json_file(self.artifacts / (name + ".json"), result)
        self.case(name, operation="trusted_script_" + operation, elapsed_seconds=elapsed,
                  exit_code=process.returncode, evidence=name + ".json", request_id=request_id,
                  mcp_acceptance=False, real_client_acceptance=False)
        self.last_workflow_request = request_id
        return result

    def workflow_read(self, project, editor, descriptor, name, *, revision=True, source=None):
        before, disks = self.state(editor, project)
        result = self.complete_workflow(self.start_workflow(project, descriptor, "read"), name, "read")
        observation.require(set(result) == {"source", "revision", "state"} and
                            isinstance(result["state"], dict), "lean_read_projection_" + name)
        token = result["revision"]
        observation.require((isinstance(token, str) and re.fullmatch(r"sr1:[0-9a-f]{64}", token))
                            if revision else token is None, "read_revision_eligibility_" + name)
        if source is not None:
            observation.require(result["source"] == source, "read_exact_observed_source_" + name)
        after, now = self.state(editor, project)
        observation.require(source_free(before, disks) == source_free(after, now),
                            "read_does_not_load_open_write_or_change_history_" + name)
        # The state projection must retain the distinctions needed to reason about
        # lifecycle and applicability, rather than return the old observation envelope.
        state = result["state"]
        observation.require(all(key in state for key in ("document", "sources", "dirty", "consistency",
                                                       "limitations", "revision_unavailable_reason")),
                            "useful_authority_state_projection_" + name)
        observation.require(state["document"]["lifecycle"] == ("open" if TARGET in after["open_paths"] else "closed") and
                            state.get("source_origin") == ("editor_buffer" if TARGET in after["open_paths"] else "disk") and
                            set(state["sources"]) == {"disk", "loaded_resource", "editor_buffer"} and
                            (state["revision_unavailable_reason"] is None if revision else
                             state["revision_unavailable_reason"] is not None),
                            "read_lifecycle_provenance_and_null_reason_" + name)
        observation.require(all(authority.get("text") is None for authority in state["sources"].values()
                                if authority.get("equals_source") is True),
                            "agreeing_source_not_duplicated_in_projection_" + name)
        if TARGET not in after["open_paths"]:
            resource_dirty = state["dirty"]["loaded_resource"]
            observation.require("non_atomic_observation" in state["limitations"],
                                "closed_read_no_atomic_claim_" + name)
            # Dirty/ineligible observations are still observations. A null token
            # must not make this independent witness disappear.
            if revision or name.startswith("resource_") or name.startswith("privacy_"):
                observation.require(resource_dirty["availability"] ==
                                    ("observed" if after["cached_id"] else "not_applicable") and
                                    resource_dirty["state"] ==
                                    (("dirty" if after["cached_edited"] else "clean") if after["cached_id"] else None),
                                    "closed_read_actual_R_dirty_even_without_revision_" + name)
                resource = state["sources"]["loaded_resource"]
                if after["cached_id"]:
                    observation.require(resource["availability"] == "observed" and resource["current"] and
                                        resource["equals_source"] == (after["cached_R"] == result["source"]) and
                                        (resource["text"] == after["cached_R"] if after["cached_R"] != result["source"]
                                         else resource["text"] is None),
                                        "closed_read_exact_independent_R_source_" + name)
                    observation.require(state["consistency"]["comparisons"]["disk_resource"] ==
                                        ("equal" if after["cached_R"] == now[TARGET]["text"] else "different"),
                                        "closed_read_independent_D_R_comparison_" + name)
                else:
                    observation.require(not resource["current"] and resource["text"] is None and
                                        resource["equals_source"] is None,
                                        "absent_R_never_invents_current_source_" + name)
            if name == "missing_cache_getters_read":
                observation.require(resource_dirty["availability"] == "unavailable" and resource_dirty["state"] is None,
                                    "missing_getters_never_fabricate_R_dirty_facts")
        self.record_witness(name, before, disks, after, now, editor)
        return result

    def workflow_edit(self, project, descriptor, revision, replacement, name, expected):
        result = self.complete_workflow(self.start_workflow(project, descriptor, "edit", revision=revision,
                                                            replacement_source=replacement), name, "edit")
        return self.review_workflow_edit(result, name, expected)

    def review_workflow_edit(self, result, name, expected):
        observation.require(set(result) == {"mode", "outcome"} and
                            result["mode"] in ("open", "closed", "undetermined") and
                            isinstance(result["outcome"], dict), "typed_edit_projection_" + name)
        outcome = result["outcome"]
        allowed = {expected} if isinstance(expected, str) else set(expected)
        observation.require(outcome["outcome"] in allowed and outcome["application"] in
                            ("not_applied", "applied", "partly_applied", "unknown"),
                            "truthful_expected_effect_outcome_" + name)
        relations = {"verified_changed": "applied", "verified_unchanged": "not_applied",
                     "refused": "not_applied"}
        if outcome["outcome"] in relations:
            observation.require(outcome["application"] == relations[outcome["outcome"]],
                                "exact_outcome_application_relation_" + name)
        effects = outcome.get("effects")
        if effects and any(effects.get(key) for key in ("resource_changed", "written_bytes", "truncated")):
            observation.require(outcome["application"] in ("applied", "partly_applied") and
                                outcome["outcome"] not in ("refused", "verified_unchanged", "application_unknown"),
                                "known_effects_never_unknown_or_not_applied_" + name)
        if outcome["application"] == "unknown":
            observation.require(outcome["outcome"] == "application_unknown" and
                                not (effects and any(effects.get(key) for key in
                                                     ("resource_changed", "written_bytes", "truncated"))),
                                "unknown_only_without_known_source_effect_facts_" + name)
        if outcome["outcome"] == "refused":
            observation.require(outcome.get("reason") is not None, "refusal_retains_cause_" + name)
            if result["mode"] == "undetermined":
                observation.require(outcome["request_id"] == self.last_workflow_request and
                                    isinstance(outcome["diagnostics"], list) and
                                    outcome["stage"] == (outcome["diagnostics"][-1]["stage"]
                                                         if outcome["diagnostics"] else "preflight") and
                                    set(outcome["interval"]) == {"started_unix_ms", "finished_unix_ms", "elapsed_us"} and
                                    0 <= outcome["interval"]["elapsed_us"] <= 10_000_000 and
                                    outcome["interval"]["started_unix_ms"] <= outcome["interval"]["finished_unix_ms"] and
                                    "requested_target" in outcome and "target" in outcome and "selection" in outcome and
                                    isinstance(outcome["next_action"]["kind"], str) and
                                    outcome.get("evidence") is None,
                                    "source_free_refusal_retains_correlation_stage_interval_and_guidance_" + name)
        if outcome["outcome"] in ("verified_changed", "verified_unchanged") and result["mode"] == "closed":
            observation.require(outcome["history"] == "not_applicable_closed",
                                "closed_history_truthfully_not_applicable_" + name)
            observation.require(outcome["lifecycle"] == {"admitted": "closed", "final_state": "closed"} and
                                outcome["evidence"]["independently_verified"] is True and
                                outcome["evidence"]["atomic"] is False and
                                outcome["evidence"]["buffer"] == "not_applicable",
                                "closed_success_complete_applicability_and_interval_limits_" + name)
        self.last_workflow_edit = result
        self.cases[-1].update(mode=result["mode"], outcome=outcome["outcome"],
                              application=outcome["application"], reason=outcome.get("reason"))
        return result

    def record_witness(self, name, before, disks, after, now, editor):
        observation.json_file(self.artifacts / (name + "-witness.json"),
                              dict(before=source_free(before, disks), after=source_free(after, now)))
        self.cases[-1].update(independent_evidence=name + "-witness.json",
                              screenshot=self.screenshot(editor, name + ".png"),
                              source_surfaces="independent_disk_public_cache_complete_editor_roster")

    def assert_closed_success(self, name, project, editor, before, disks, desired, *, changed):
        after, now = self.state(editor, project)
        observation.require(TARGET not in after["open_paths"] and
                            documents(before) == documents(after) and before["selection"] == after["selection"] and
                            before["loader_calls"] == after["loader_calls"] and
                            all(now[path] == disk for path, disk in disks.items() if path != TARGET),
                            "closed_success_preserves_B_absence_unrelated_history_and_loader_" + name)
        observation.require(now[TARGET]["text"] == desired and
                            all(now[TARGET][key] == disks[TARGET][key] for key in
                                ("device", "inode", "mtime_ns", "mode")) and
                            before["cached_id"] == after["cached_id"] and
                            (after["cached_R"] == desired and after["cached_edited"] is False
                             if before["cached_id"] else after["cached_R"] is None and after["cached_edited"] is None),
                            "independent_D_R_applicability_identity_mtime_" + name)
        outcome = self.last_workflow_edit["outcome"]
        effects = outcome["effects"]
        observation.require(outcome["evidence"]["resource"] == {
                                "admitted": "present" if before["cached_id"] else "absent",
                                "availability": "observed" if after["cached_id"] else "not_applicable",
                                "state": "present" if after["cached_id"] else "absent"} and
                            ("loaded_class_not_reloaded" in outcome["limitations"] if before["cached_id"] else
                             "loaded_class_not_reloaded" not in outcome["limitations"]),
                            "success_reports_actual_R_branch_and_no_hot_reload_claim_" + name)
        if changed:
            observation.require(effects["resource_entered"] == bool(before["cached_id"]) and
                                effects["disk_entered"] and effects["written_bytes"] == len(desired.encode()) and
                                effects["truncated"] and effects["flushed"] and effects["readback"] and
                                effects["mtime_restored"], "complete_persistence_effect_facts_" + name)
        else:
            observation.require(not any(effects.values()), "unchanged_has_no_setter_write_metadata_authorization_" + name)
        if not changed:
            observation.require(disks == now and source_free(before, disks) == source_free(after, now),
                                "unchanged_has_zero_timestamp_source_history_effects_" + name)
        else:
            observation.require(now[TARGET]["sha256"] != disks[TARGET]["sha256"] and
                                now[TARGET]["ctime_ns"] != disks[TARGET]["ctime_ns"],
                                "changed_exact_source_and_new_ctime_" + name)
        self.record_witness(name, before, disks, after, now, editor)
        return after, now

    def assert_no_effect(self, name, project, editor, before, disks):
        after, now = self.state(editor, project)
        observation.require(source_free(before, disks) == source_free(after, now),
                            "independent_no_effect_preserves_newer_work_" + name)
        self.record_witness(name, before, disks, after, now, editor)

    def closed_positives(self):
        profiles = (("cached", True, SAFE, CHANGED), ("absent", False, SAFE, CHANGED),
                    ("unicode", False, SAFE, CHANGED + "# café 雪 🐳\n"),
                    ("empty_desired", False, SAFE, ""), ("empty_original", False, "", SAFE),
                    ("exact_source_bound", False, SAFE,
                     SAFE + "# " + "x" * (SOURCE_LIMIT - len(SAFE.encode()) - 3) + "\n"))
        for name, cached, original, desired in profiles:
            with self.closed_fixture(name, cached=cached, source=original) as (project, editor, descriptor):
                first = self.workflow_read(project, editor, descriptor, name + "_read", source=original)
                second = self.workflow_read(project, editor, descriptor, name + "_stable_read", source=original)
                observation.require(first["revision"] == second["revision"],
                                    "revision_excludes_request_ids_and_intervals_" + name)
                before, disks = self.state(editor, project)
                self.workflow_edit(project, descriptor, first["revision"], original, name + "_unchanged", "verified_unchanged")
                self.assert_closed_success(name + "_unchanged", project, editor, before, disks, original, changed=False)
                self.workflow_edit(project, descriptor, first["revision"], desired, name + "_changed", "verified_changed")
                self.assert_closed_success(name + "_changed", project, editor, before, disks, desired, changed=True)
                fresh = self.workflow_read(project, editor, descriptor, name + "_fresh_read", source=desired)
                observation.require(fresh["revision"] != first["revision"], "changed_read_invalidates_revision_" + name)

    def closed_source_refusals(self):
        replacements = {"over_bound": "#" * (SOURCE_LIMIT + 1), "nul": SAFE + "\0",
                        "cr": SAFE.replace("\n", "\r\n"), "bom": "\ufeff" + SAFE,
                        "parse": "extends RefCounted\nfunc value(:\n", "tool": "@tool\n" + SAFE,
                        "global": "class_name ClosedUnapprovedGlobal\n" + SAFE,
                        "preload": SAFE + 'const BAD = preload("res://scripts/other.gd")\n'}
        with self.closed_fixture("source-refusals", cached=True) as (project, editor, descriptor):
            capture = self.workflow_read(project, editor, descriptor, "source_refusal_basis", source=SAFE)
            for name, replacement in replacements.items():
                before, disks = self.state(editor, project)
                if name in ("over_bound", "nul", "cr", "bom"):
                    result = self.complete_workflow(self.start_workflow(
                        project, descriptor, "edit", revision=capture["revision"], replacement_source=replacement),
                        "source_refusal_" + name, "edit")
                    observation.require(result == {"error": {"code": "invalid_request", "application": "not_applied"}},
                                        "checked_source_input_refused_before_execution_" + name)
                else:
                    self.workflow_edit(project, descriptor, capture["revision"], replacement, "source_refusal_" + name, "refused")
                self.assert_no_effect("source_refusal_" + name, project, editor, before, disks)
        for profile, source in (("original_tool", "@tool\n" + SAFE),
                                ("original_preload", SAFE + 'const BAD = preload("res://scripts/other.gd")\n')):
            with self.closed_fixture(profile, source=source) as (project, editor, descriptor):
                self.workflow_read(project, editor, descriptor, profile + "_read", revision=False, source=source)
                before, disks = self.state(editor, project)
                self.workflow_edit(project, descriptor, "sr1:" + "0" * 64, CHANGED, profile + "_refused", "refused")
                self.assert_no_effect(profile + "_refused", project, editor, before, disks)
        invalid = "extends RefCounted\nfunc value(:\n"
        with self.closed_fixture("original-invalid", source=invalid) as (project, editor, descriptor):
            # A revision binds observed state; it is not a parse verdict or mutation authority.
            capture = self.workflow_read(project, editor, descriptor, "original_invalid_read", source=invalid)
            before, disks = self.state(editor, project)
            result = self.workflow_edit(project, descriptor, capture["revision"], CHANGED,
                                        "original_invalid_refused", "refused")
            outcome = result["outcome"]
            observation.require(outcome["reason"] == "parse_error" and
                                outcome["evidence"]["validation"] ==
                                {"original": False, "desired": True, "actual": False} and
                                not outcome["effects"]["authorized"],
                                "valid_desired_never_bypasses_invalid_original")
            self.assert_no_effect("original_invalid_refused", project, editor, before, disks)
        for mutation in ("divergent", "dirty", "equal_dirty"):
            with self.closed_fixture("resource-" + mutation, cached=True) as (project, editor, descriptor):
                clean = self.workflow_read(project, editor, descriptor, "resource_" + mutation + "_clean", source=SAFE)
                changed = self.close_action(editor, "closed_resource", mutation=mutation)["state"]
                observation.require(changed["cached_R"] != SAFE if mutation == "divergent" else
                                    changed["cached_edited"] is True, "real_unsafe_loaded_resource_" + mutation)
                if mutation == "equal_dirty":
                    observation.require(changed["cached_R"] == SAFE, "real_equal_text_dirty_R")
                read = self.workflow_read(project, editor, descriptor, "resource_" + mutation + "_read", revision=False, source=SAFE)
                observation.require(read["state"]["revision_unavailable_reason"] ==
                                    ("dirty_resource" if changed["cached_edited"] else "divergent_resource"),
                                    "resource_revision_null_preserves_specific_cause_" + mutation)
                before, disks = self.state(editor, project)
                self.workflow_edit(project, descriptor, clean["revision"], CHANGED, "resource_" + mutation + "_refused", "refused")
                self.assert_no_effect("resource_" + mutation + "_refused", project, editor, before, disks)
        for fault in ("missing_cache_getters", "missing_roster", "missing_context", "epoch_overflow"):
            with self.closed_fixture(fault, faults=True) as (project, editor, descriptor):
                clean = self.workflow_read(project, editor, descriptor, fault + "_clean", source=SAFE)
                self.close_action(editor, "closed_fault", fault=fault)
                self.workflow_read(project, editor, descriptor, fault + "_read", revision=False)
                self.close_action(editor, "closed_fault", fault=fault)
                before, disks = self.state(editor, project)
                self.workflow_edit(project, descriptor, clean["revision"], CHANGED, fault + "_refused", "refused")
                self.assert_no_effect(fault + "_refused", project, editor, before, disks)
        with self.closed_fixture("epoch-reset", faults=True) as (project, editor, descriptor):
            clean = self.workflow_read(project, editor, descriptor, "epoch_reset_clean", source=SAFE)
            self.close_action(editor, "closed_fault", fault="epoch_reset")
            fresh = self.workflow_read(project, editor, descriptor, "epoch_reset_fresh", source=SAFE)
            observation.require(clean["revision"] != fresh["revision"], "epoch_reset_never_reuses_old_basis")
            before, disks = self.state(editor, project)
            self.workflow_edit(project, descriptor, clean["revision"], CHANGED, "epoch_reset_refused", "refused")
            self.assert_no_effect("epoch_reset_refused", project, editor, before, disks)

    def wait_closed_barrier(self, editor, stage, pending):
        process = pending[0]
        def reached():
            observation.require(process.poll() is None, "workflow_exited_before_barrier_" + stage)
            path = editor["control"] / "closed-event.json"
            if path.is_file():
                event = json.loads(path.read_text())
                if event.get("request_id") == pending[2] and event.get("stage") == stage:
                    return event
            return None
        return observation.wait_for(reached, "actual_closed_barrier_" + stage, timeout=8)

    def closed_revision_races(self):
        for mutation in ("same_text", "namespace", "cache_appear", "cache_replace", "target_open", "aba", "unrelated_epoch", "reconfigure", "context"):
            cached = mutation == "cache_replace"
            with self.closed_fixture("revision-" + mutation, cached=cached) as (project, editor, descriptor):
                first = self.workflow_read(project, editor, descriptor, mutation + "_basis", source=SAFE)
                self.closed_transition(project, editor, mutation)
                changed, changed_disks = self.state(editor, project)
                self.workflow_edit(project, descriptor, first["revision"], CHANGED, mutation + "_stale", "refused")
                self.assert_no_effect(mutation + "_stale", project, editor, changed, changed_disks)
                fresh = self.workflow_read(project, editor, descriptor, mutation + "_fresh", revision=mutation != "context", source=SAFE)
                if mutation != "context":
                    observation.require(fresh["revision"] != first["revision"], "stable_fact_revision_changed_" + mutation)
        # Races after comparison/prepare use product stage barriers, not stale token
        # rejection alone. Hold the original clock while a real actor changes state.
        for stage in ("prepare", "apply", "verify:post_change", "recheck:post_change"):
            for mutation in ("same_text", "namespace", "cache_appear", "target_open", "aba", "unrelated_epoch"):
                name = stage.replace(":", "_") + "_" + mutation
                with self.closed_fixture(name) as (project, editor, descriptor):
                    basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                    self.close_action(editor, "closed_arm", stage=stage)
                    pending = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=CHANGED)
                    self.wait_closed_barrier(editor, stage, pending)
                    self.closed_transition(project, editor, mutation)
                    newer, disks = self.state(editor, project)
                    self.close_action(editor, "closed_release")
                    result = self.complete_workflow(pending, name, "edit")
                    self.review_workflow_edit(result, name, "refused" if stage in ("prepare", "apply") else
                                              ("applied_unverified", "application_unknown"))
                    self.assert_no_effect(name, project, editor, newer, disks)

    def closed_transition(self, project, editor, mutation):
        path = project / "scripts/subject.gd"
        if mutation == "same_text":
            old = path.stat()
            raw = path.read_bytes()
            with path.open("r+b") as stream:
                stream.write(raw)
                stream.flush()
                os.fsync(stream.fileno())
            os.utime(path, ns=(old.st_atime_ns, old.st_mtime_ns))
            observation.require(path.stat().st_ctime_ns != old.st_ctime_ns, "same_text_write_changed_file_revision")
        elif mutation == "namespace":
            sibling = path.with_name("subject.replacement")
            sibling.write_bytes(path.read_bytes())
            old_inode = path.stat().st_ino
            os.replace(sibling, path)
            observation.require(path.stat().st_ino != old_inode, "actual_namespace_replacement")
        elif mutation in ("cache_appear", "cache_replace"):
            self.close_action(editor, "closed_resource", mutation="appear" if mutation == "cache_appear" else "replace")
        elif mutation in ("target_open", "aba"):
            self.close_action(editor, "close_human", path=TARGET, mutation="open")
            if mutation == "aba":
                self.close_action(editor, "close_human", path=TARGET, mutation="close")
                self.close_action(editor, "close_refs")
        elif mutation == "unrelated_epoch":
            self.close_action(editor, "close_human", path=BACKGROUND, mutation="close")
        elif mutation == "reconfigure":
            self.close_action(editor, "closed_reconfigure")
        elif mutation == "context":
            self.close_action(editor, "close_config", setting="autoload", value=True)

    def closed_effect_faults(self):
        for cached in (False, True):
            for fault in ("partial_write", "lost_write", "mtime_failure", "expire_before_apply", "expire_after_write") + \
                    (("expire_after_resource",) if cached else ()):
                name = ("cached_" if cached else "absent_") + fault
                fault_desired = "# CLOSED_DISTINCT_WRITE_PREFIX\n" + CHANGED
                with self.closed_fixture(name, cached=cached, faults=True) as (project, editor, descriptor):
                    basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                    self.close_action(editor, "closed_arm", stage="apply")
                    pending = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=fault_desired)
                    self.wait_closed_barrier(editor, "apply", pending)
                    before, disks = self.state(editor, project)
                    self.close_action(editor, "closed_fault", request_id=pending[2], fault=fault)
                    self.close_action(editor, "closed_release")
                    result = self.complete_workflow(pending, name, "edit")
                    self.review_workflow_edit(result, name, "refused" if fault == "expire_before_apply" else
                                              ("applied_unverified", "application_unknown"))
                    after, now = self.state(editor, project)
                    if fault == "expire_before_apply":
                        self.assert_no_effect(name, project, editor, before, disks)
                    else:
                        observation.require(TARGET not in after["open_paths"] and documents(before) == documents(after),
                                            "partial_effect_never_opens_or_alters_unrelated_history_" + name)
                        observation.require(result["outcome"]["application"] != "not_applied",
                                            "known_native_entry_stays_effect_sensitive_" + name)
                        if fault == "partial_write":
                            observation.require(now[TARGET]["text"] != SAFE and now[TARGET]["text"] != fault_desired,
                                                "independent_partial_persistence_" + name)
                            observation.require(0 < result["outcome"]["effects"]["written_bytes"] < len(fault_desired.encode()),
                                                "partial_write_receipt_matches_independent_partial_disk_" + name)
                        elif fault in ("mtime_failure", "expire_after_write"):
                            observation.require(now[TARGET]["text"] == fault_desired, "known_persisted_desired_after_failure_" + name)
                            observation.require(result["outcome"]["effects"]["mtime_restored"] is False and
                                                now[TARGET]["mtime_ns"] != disks[TARGET]["mtime_ns"],
                                                "failed_metadata_is_not_verified_success_" + name)
                        elif fault == "expire_after_resource":
                            observation.require(after["cached_R"] == fault_desired and now[TARGET] == disks[TARGET],
                                                "resource_only_partial_application")
                        elif fault == "lost_write":
                            observation.require(now[TARGET] == disks[TARGET] and
                                                result["outcome"]["effects"]["disk_entered"] and
                                                result["outcome"]["effects"]["written_bytes"] == 0 and
                                                (after["cached_R"] == fault_desired if cached else not after["cached_id"]),
                                                "lost_write_keeps_known_R_or_unknown_disk_entry_truthful_" + name)
                        if fault in ("partial_write", "lost_write", "mtime_failure"):
                            outcome = result["outcome"]
                            observation.require(outcome["application"] ==
                                                ("applied" if fault == "mtime_failure" else
                                                 "unknown" if fault == "lost_write" and not cached else "partly_applied"),
                                                "exact_independently_witnessed_partial_effect_classification_" + name)
                            evidence = outcome["evidence"]
                            observation.require(outcome["reason"] ==
                                                ("mtime_restore_failed" if fault == "mtime_failure" else "write_failed") and
                                                outcome["lifecycle"]["final_state"] == "closed" and
                                                evidence["buffer"] == "not_applicable" and
                                                evidence["independently_verified"] is False and
                                                evidence["after"] == {
                                                    "disk": {"sha256": now[TARGET]["sha256"],
                                                             "utf8_bytes": len(now[TARGET]["text"].encode())},
                                                    "resource": {"state": "present" if cached else "absent",
                                                                 "edited": after["cached_edited"] if cached else None,
                                                                 "sha256": sha(after["cached_R"]) if cached else None,
                                                                 "utf8_bytes": len(after["cached_R"].encode()) if cached else None}},
                                                "permitted_survivor_evidence_independently_matches_actual_partial_D_R_" + name)
                        self.record_witness(name, before, disks, after, now, editor)

    def closed_save_and_slot_guards(self):
        with self.closed_fixture("save-format", cached=True) as (project, editor, descriptor):
            self.close_action(editor, "closed_save_profile", key="trim_trailing_whitespace_on_save", value=True)
            basis = self.workflow_read(project, editor, descriptor, "save_profile_basis", source=SAFE)
            before, disks = self.state(editor, project)
            self.workflow_edit(project, descriptor, basis["revision"], CHANGED + "# trailing   \n",
                               "save_would_reformat_refused", "refused")
            self.assert_no_effect("save_would_reformat_refused", project, editor, before, disks)
        with self.closed_fixture("settings-before-apply", cached=True, faults=True) as (project, editor, descriptor):
            basis = self.workflow_read(project, editor, descriptor, "settings_guard_basis", source=SAFE)
            before, disks = self.state(editor, project)
            settings_before = (project / "project.godot").read_bytes()
            self.close_action(editor, "closed_arm", callback_stage="before_apply", callback_action="settings")
            result = self.workflow_edit(project, descriptor, basis["revision"], CHANGED,
                                        "settings_guard_before_apply", "refused")
            events = self.close_action(editor, "closed_witness")["events"]
            observation.require(result["outcome"]["reason"] == "validator_settings_changed" and
                                not any(value for key, value in result["outcome"]["effects"].items()
                                        if key not in ("authorized", "entry_refused")) and
                                (project / "project.godot").read_bytes() != settings_before and
                                any(event["stage"] == "callback:before_apply" and
                                    event["request_id"] == self.last_workflow_request and event["action"] == "settings"
                                    for event in events) and
                                not any(event["stage"] in ("after_resource", "after_write", "after_mtime") for event in events),
                                "unsupported_immediate_settings_change_refuses_before_any_source_effect")
            self.assert_no_effect("settings_guard_before_apply", project, editor, before, disks)
            self.close_action(editor, "close_idle", frames=16)
            self.assert_no_effect("settings_guard_terminal", project, editor, before, disks)
        with self.closed_fixture("shared-slot") as (project, editor, descriptor):
            basis = self.workflow_read(project, editor, descriptor, "slot_basis", source=SAFE)
            self.close_action(editor, "closed_arm", stage="apply")
            pending = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=CHANGED)
            self.wait_closed_barrier(editor, "apply", pending)
            before, disks = self.state(editor, project)
            for operation in ("open_begin", "close_begin"):
                stream, request_id = self.authenticated_peer(descriptor)
                with stream:
                    stream.sendall(observation.packet([6, operation, request_id, descriptor["session_id"],
                                                       descriptor["project_root"], TARGET, 9000]))
                    result, raw = observation.receive(stream)
                    observation.require(result["status"] in ("busy", "refused", "unavailable") and
                                        result.get("reason") == "slot_busy" and CHANGED.encode() not in raw,
                                        "real_shared_slot_excludes_legacy_" + operation)
            self.case("closed_shared_slot_exclusion", operation="private_v6_native4")
            self.assert_no_effect("closed_shared_slot_exclusion", project, editor, before, disks)
            self.close_action(editor, "closed_release")
            result = self.complete_workflow(pending, "slot_owner_completed", "edit")
            self.review_workflow_edit(result, "slot_owner_completed", "verified_changed")
            self.assert_closed_success("slot_owner_completed", project, editor, before, disks, CHANGED, changed=True)

    def closed_acquisition_boundaries(self):
        for mutation in ("same_text", "namespace", "cache_appear", "cache_replace", "target_open", "unrelated_epoch"):
            name = "supplement_" + mutation
            with self.closed_fixture(name, cached=mutation == "cache_replace") as (project, editor, descriptor):
                self.close_action(editor, "closed_arm", stage="inspect")
                pending = self.start_workflow(project, descriptor, "read")
                self.wait_closed_barrier(editor, "inspect", pending)
                self.closed_transition(project, editor, mutation)
                before, disks = self.state(editor, project)
                self.close_action(editor, "closed_release")
                result = self.complete_workflow(pending, name, "read")
                state = result["state"]
                observation.require(result["revision"] is None and state["revision_unavailable_reason"] is not None,
                                    "changed_supplement_never_refreshes_edit_basis_" + name)
                if mutation in ("same_text", "namespace"):
                    authority = state["sources"]["disk"]
                    observation.require(not authority["current"] and authority["invalidated"] is not None and
                                        authority["invalidated"]["current"] is False and state["source_origin"] != "disk",
                                        "changed_D_not_published_as_current_with_extended_interval_" + name)
                if mutation in ("cache_appear", "cache_replace"):
                    authority = state["sources"]["loaded_resource"]
                    observation.require(not authority["current"] and authority["availability"] != "observed",
                                        "changed_R_identity_invalidates_old_resource_source_" + name)
                if mutation in ("target_open", "unrelated_epoch"):
                    observation.require(state["document"]["lifecycle"] == "unknown" and
                                        state["document"]["invalidated_lifecycle"] is not None,
                                        "changed_lifecycle_invalidates_old_closed_fact_" + name)
                self.assert_no_effect(name, project, editor, before, disks)
        for boundary, reason in (("missing", "missing_target"), ("absent_session", "editor_unavailable"),
                                 ("silent_editor", "timeout"), ("disconnected", "disconnected_editor")):
            name = "core_refusal_" + boundary
            with self.closed_fixture(name, cached=True) as (project, editor, descriptor):
                before, disks = self.state(editor, project)
                selected = descriptor
                target = project / "scripts/subject.gd"
                if boundary == "missing":
                    target.rename(target.with_suffix(".held"))
                    before, disks = self.state(editor, project)
                elif boundary == "absent_session":
                    selected = dict(descriptor, session_id="f" * 32)
                elif boundary == "silent_editor":
                    editor["process"].send_signal(signal.SIGSTOP)
                else:
                    self.action(editor, "hold_observe")
                try:
                    pending = self.start_workflow(project, selected, "edit", revision="sr1:" + "0" * 64,
                                                  replacement_source=CHANGED)
                    if boundary == "disconnected":
                        self.wait_barrier(editor, "observe", pending[0])
                        self.close_action(editor, "close_lifetime", control="disable")
                    result = self.complete_workflow(pending, name, "edit")
                    self.review_workflow_edit(result, name, "refused")
                    observation.require(result["outcome"]["reason"] == reason,
                                        "distinct_source_free_core_refusal_cause_" + boundary)
                    if boundary == "missing":
                        self.assert_no_effect(name, project, editor, before, disks)
                finally:
                    if boundary == "missing":
                        target.with_suffix(".held").rename(target)
                    elif boundary == "silent_editor":
                        editor["process"].send_signal(signal.SIGCONT)
                    elif boundary == "disconnected":
                        self.action(editor, "release_hold")
                if boundary != "missing":
                    self.assert_no_effect(name, project, editor, before, disks)
        with self.closed_fixture("safe-selection") as (project, editor, descriptor):
            second = self.start_editor(project)
            try:
                pair = observation.wait_for(lambda: self.descriptors(project) if len(self.descriptors(project)) == 2 else None,
                                            "workflow_two_authenticated_sessions")
                self.close_action(second, "close_setup", paths=[CURRENT, BACKGROUND], selected=CURRENT)
                witnesses = [self.state(owner, project) for owner in (editor, second)]
                for operation in ("read", "edit"):
                    pending = self.start_workflow(project, descriptor, operation, select_session=False,
                                                  **({} if operation == "read" else
                                                     {"revision": "sr1:" + "0" * 64, "replacement_source": CHANGED}))
                    result = self.complete_workflow(pending, "selection_" + operation, operation)
                    state = result["state"] if operation == "read" else result["outcome"]
                    if operation == "edit":
                        self.review_workflow_edit(result, "selection_edit", "refused")
                    observation.require(state["selection"]["missing_selector"] == "session_id" and
                                        set(state["selection"]["candidate_sessions"]) == {item["session_id"] for item in pair} and
                                        (state["next_action"] == "specify_session" if operation == "read" else
                                         state["next_action"]["kind"] == "specify_session"),
                                        "safe_actionable_session_selection_" + operation)
                    observation.require(all(sha(disk["text"]) not in json.dumps(result)
                                            for _, disks in witnesses for disk in disks.values()) and
                                        all(item["endpoint"] not in json.dumps(result)
                                            for item in pair if isinstance(item.get("endpoint"), str)),
                                        "selection_no_candidate_source_digest_or_endpoint_" + operation)
                    for owner, (before, disks) in zip((editor, second), witnesses):
                        self.assert_no_effect("selection_" + operation, project, owner, before, disks)
            finally:
                self.close_editor(second)
                self.editors.remove(second)

    def closed_exchange(self, stream, descriptor, request_id, operation, *tail):
        stream.sendall(observation.packet([6, operation, request_id, descriptor["session_id"],
                                           descriptor["project_root"], TARGET, *tail]))
        reply, _ = observation.receive(stream)
        observation.require(all(reply[key] == value for key, value in
                                {"v": 6, "request_id": request_id, "session_id": descriptor["session_id"],
                                 "project_root": descriptor["project_root"], "script_path": TARGET}.items()),
                            "authenticated_closed_reply_binding_" + operation)
        return reply

    def closed_wire_boundaries(self):
        with self.closed_fixture("closed-wire", cached=True) as (project, editor, descriptor):
            stream, request_id = self.authenticated_peer(descriptor)
            with stream:
                inspected = self.closed_exchange(stream, descriptor, request_id, "closed_inspect", 9000)
            observation.require(inspected["status"] == "inspected", "real_closed_inspect_for_wire_basis")
            state = inspected["state"]
            expected = {key: state[key] for key in ("project_device", "project_inode", "file_revision", "close_epoch")}
            resource = state["resource"]
            expected["resource"] = {key: resource[key] for key in ("state", "instance_id", "path", "edited", "profile_sha256")}
            expected["resource"].update(source_sha256=sha(resource["source"]),
                                        utf8_bytes=str(len(resource["source"].encode())))
            before, disks = self.state(editor, project)
            basis = self.workflow_read(project, editor, descriptor, "busy_before_prepare_basis", source=SAFE)
            owner, owner_id = self.authenticated_peer(descriptor)
            with owner:
                prepared = self.closed_exchange(owner, descriptor, owner_id, "closed_prepare", expected, CHANGED, 9000)
                observation.require(prepared["status"] == "prepared", "actual_competing_closed_owner")
                peer, peer_id = self.authenticated_peer(descriptor)
                with peer:
                    busy = self.closed_exchange(peer, descriptor, peer_id, "closed_prepare", expected, CHANGED, 9000)
                    observation.require(busy["kind"] == "closed_prepared" and busy["status"] == "refused" and
                                        busy["reason"] == "slot_busy" and busy["state"] is None and busy["native"] is None,
                                        "prepare_busy_correct_kind_reason_no_effect")
                self.case("closed_prepare_busy", operation="authenticated_private_v6")
                self.assert_no_effect("closed_prepare_busy", project, editor, before, disks)
                refusal = self.workflow_edit(project, descriptor, basis["revision"], CHANGED,
                                             "workflow_busy_after_read", "refused")
                observation.require(refusal["outcome"]["reason"] == "slot_busy",
                                    "workflow_busy_retains_actual_acquisition_cause")
                self.assert_no_effect("workflow_busy_after_read", project, editor, before, disks)
                self.closed_exchange(owner, descriptor, owner_id, "closed_abort")
            for name in ("arity", "unknown_field", "duplicate_field", "escaped_duplicate_field", "request_binding", "session_binding",
                         "unowned_apply", "unowned_verify", "unowned_finish", "unowned_recheck", "unowned_abort",
                         "out_of_order_apply", "out_of_order_recheck", "duplicate_verify", "duplicate_prepare",
                         "stale_apply", "stale_verify", "stale_finish"):
                stream, request_id = self.authenticated_peer(descriptor)
                with stream:
                    prefix = [6, "closed_prepare", request_id, descriptor["session_id"], descriptor["project_root"], TARGET]
                    if name.startswith("unowned_"):
                        operation = "closed_" + name.removeprefix("unowned_")
                        tail = ["0" * 64, "0" * 64] if operation == "closed_apply" else \
                               ["preflight"] if operation in ("closed_verify", "closed_recheck") else []
                        value = [6, operation, *prefix[2:], *tail]
                    else:
                        value = [*prefix, expected, CHANGED, 9000]
                    if name in ("out_of_order_apply", "out_of_order_recheck", "duplicate_verify", "duplicate_prepare",
                                "stale_apply", "stale_verify", "stale_finish"):
                        prepared = self.closed_exchange(stream, descriptor, request_id, "closed_prepare", expected, CHANGED, 9000)
                        observation.require(prepared["kind"] == "closed_prepared" and prepared["status"] == "prepared",
                                            "actual_wire_owner_prepared_" + name)
                        if name == "duplicate_verify":
                            verified = self.closed_exchange(stream, descriptor, request_id, "closed_verify", "preflight")
                            observation.require(verified["status"] == "verified", "actual_first_verify")
                            value = [6, "closed_verify", *prefix[2:], "preflight"]
                        elif name.startswith("stale_"):
                            self.closed_exchange(stream, descriptor, request_id, "closed_abort")
                            operation = "closed_" + name.removeprefix("stale_")
                            tail = ["0" * 64, "0" * 64] if operation == "closed_apply" else \
                                   ["post_change"] if operation == "closed_verify" else []
                            value = [6, operation, *prefix[2:], *tail]
                        elif name == "out_of_order_apply":
                            value = [6, "closed_apply", *prefix[2:], "0" * 64, "0" * 64]
                        elif name == "out_of_order_recheck":
                            value = [6, "closed_recheck", *prefix[2:], "preflight"]
                    if name == "arity":
                        value.pop()
                    elif name == "unknown_field":
                        value[6] = dict(expected, unapproved=True)
                    elif name == "request_binding":
                        value[2] = secrets.token_hex(16)
                    elif name == "session_binding":
                        value[3] = secrets.token_hex(16)
                    payload = observation.packet(value)
                    if name in ("duplicate_field", "escaped_duplicate_field"):
                        key = "close_epoch" if name == "duplicate_field" else r"close_\u0065poch"
                        raw = json.dumps(value, separators=(",", ":")).replace(
                            '"close_epoch":', '"' + key + '":"0","close_epoch":', 1).encode()
                        payload = struct.pack(">I", len(raw)) + raw
                    stream.sendall(payload)
                    observation.require(observation.peer_closed_without_data(stream),
                                        "closed_negative_actual_rejection_" + name)
                observation.wait_for(lambda: not self.action(editor, "scope_slot")["busy"],
                                     "closed_negative_owner_cleanup_" + name)
                self.case("closed_wire_" + name, operation="authenticated_private_v6_closed_negative")
                self.assert_no_effect("closed_wire_" + name, project, editor, before, disks)

    def closed_native(self):
        self.compile_window_probe()
        self.group("closed-positives", self.closed_positives)
        self.group("closed-refusals", self.closed_source_refusals)
        self.group("closed-revisions-and-boundary-races", self.closed_revision_races)
        self.group("closed-effect-faults", self.closed_effect_faults)
        self.group("closed-acquisition-invalidation-and-selection", self.closed_acquisition_boundaries)
        self.group("closed-authenticated-wire-boundaries", self.closed_wire_boundaries)
        self.group("closed-save-profile-and-shared-slot", self.closed_save_and_slot_guards)

    def closed_cancellation(self):
        for stage in ("prepare", "apply", "verify:post_change"):
            for control in ("signal", "disconnect", "disable"):
                name = stage.replace(":", "_") + "_" + control
                with self.closed_fixture(name, cached=True) as (project, editor, descriptor):
                    basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                    self.close_action(editor, "closed_arm", stage=stage)
                    pending = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=CHANGED)
                    self.wait_closed_barrier(editor, stage, pending)
                    newer, disks = self.state(editor, project)
                    if control == "signal":
                        pending[0].send_signal(signal.SIGTERM)
                    elif control == "disconnect":
                        # The fixture closes only the owned operation channel.
                        self.close_action(editor, "closed_disconnect")
                    else:
                        self.close_action(editor, "close_lifetime", control="disable")
                    if control == "signal":
                        result = self.complete_workflow(pending, name, "edit")
                        self.close_action(editor, "closed_release")
                    else:
                        self.close_action(editor, "closed_release")
                        result = self.complete_workflow(pending, name, "edit")
                    expected = ("refused" if stage == "prepare" else
                                ("refused", "application_unknown") if stage == "apply" else
                                ("applied_unverified", "application_unknown"))
                    self.review_workflow_edit(result, name, expected)
                    self.assert_no_effect(name, project, editor, newer, disks)
                    # Event barrier: ordinary frames are not an arbitrary timed sleep.
                    self.close_action(editor, "close_idle", frames=16)
                    self.assert_no_effect(name + "_terminal", project, editor, newer, disks)
        for stage in ("before_apply", "after_resource", "after_write", "before_verify"):
            for action in ("open", "aba", "newer_resource", "newer_disk", "cache_appear", "stop", "disable"):
                name = "native_" + stage + "_" + action
                with self.closed_fixture(name, cached=action != "cache_appear", faults=True) as (project, editor, descriptor):
                    basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                    before, disks = self.state(editor, project)
                    self.close_action(editor, "closed_arm", callback_stage=stage, callback_action=action)
                    result = self.workflow_edit(project, descriptor, basis["revision"], CHANGED, name,
                                                ("refused", "applied_unverified", "application_unknown"))
                    after, now = self.state(editor, project)
                    events = self.close_action(editor, "closed_witness")["events"]
                    callbacks = [event for event in events if event["stage"] == "callback:" + stage]
                    observation.require(len(callbacks) == 1 and callbacks[0]["action"] == action and
                                        callbacks[0]["request_id"] == self.last_workflow_request,
                                        "independent_callback_request_stage_action_" + name)
                    entered = [event["stage"] for event in events
                               if event["request_id"] == self.last_workflow_request and
                               not event["stage"].startswith("callback:")]
                    observation.require(stage in entered, "actual_native_stage_entered_" + name)
                    if stage == "before_apply":
                        observation.require("after_resource" not in entered and "after_write" not in entered and
                                            "after_mtime" not in entered,
                                            "before_apply_has_no_late_effect_callback_" + name)
                        if action in ("stop", "disable"):
                            observation.require(source_free(before, disks) == source_free(after, now),
                                                "before_apply_lifetime_preserves_original_D_R_documents_" + name)
                    if stage == "after_resource" and action == "newer_resource":
                        observation.require(now == disks and documents(before) == documents(after) and
                                            before["selection"] == after["selection"] and
                                            "after_write" not in entered and "after_mtime" not in entered,
                                            "newer_R_after_resource_forbids_later_D_metadata_effects_" + name)
                    if action not in ("stop", "disable"):
                        effects = result["outcome"].get("effects")
                        observation.require(effects is not None and
                                            effects["resource_entered"] == (stage != "before_apply" and bool(before["cached_id"])) and
                                            effects["disk_entered"] == (stage in ("after_write", "before_verify")),
                                            "receipt_matches_independent_stage_effect_prefix_" + name)
                    observation.json_file(self.artifacts / (name + "-callbacks.json"),
                                          [{"request_id": event["request_id"], "stage": event["stage"],
                                            "action": event.get("action")} for event in events])
                    self.cases[-1]["callback_evidence"] = name + "-callbacks.json"
                    if action == "newer_resource":
                        observation.require(after["cached_R"] == "extends RefCounted\n# CLOSED_NEWER_WORK\n",
                                            "newer_R_not_rolled_back_or_reasserted_" + name)
                    if action == "newer_disk":
                        observation.require(now[TARGET]["text"] == "extends RefCounted\n# CLOSED_NEWER_WORK\n",
                                            "newer_D_not_rolled_back_or_reasserted_" + name)
                    if action == "cache_appear":
                        observation.require(bool(after["cached_id"]), "newly_appeared_cache_not_evicted_" + name)
                    if action == "open":
                        observation.require(TARGET in after["open_paths"], "newly_opened_target_never_compensated_" + name)
                    if stage != "before_apply" and not (action == "cache_appear" and stage == "after_resource"):
                        observation.require(result["outcome"]["application"] != "not_applied",
                                            "post_entry_cancellation_preserves_effect_" + name)
                    self.record_witness(name, before, disks, after, now, editor)
                    self.close_action(editor, "close_idle", frames=16)
                    self.assert_no_effect(name + "_terminal", project, editor, after, now)

    def closed_durability(self):
        for cached in (False, True):
            name = "durability_" + ("cached" if cached else "absent")
            with self.closed_fixture(name, cached=cached) as (project, editor, descriptor):
                self.close_action(editor, "close_human", path=CURRENT, mutation="dirty_equal")
                human, disks = self.state(editor, project)
                basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                result = self.workflow_edit(project, descriptor, basis["revision"], CHANGED, name, "verified_changed")
                after, now = self.assert_closed_success(name, project, editor, human, disks, CHANGED, changed=True)
                self.close_history(editor, CURRENT)
                history_after, _ = self.state(editor, project)
                # Opening is later durability evidence, never a means of admission.
                self.public_open(project, descriptor, name + "_ordinary_open")
                opened, opened_disks = self.state(editor, project)
                observation.require(opened["target"]["B"] == opened["target"]["R"] == CHANGED ==
                                    opened_disks[TARGET]["text"] and not opened["target"]["dirty"],
                                    "later_real_visible_B_converges_without_reconciliation_" + name)
                self.native_action(editor, "native_edit_save_clean")
                parsed = self.native_action(editor, "native_edit_reparse")
                observation.require(parsed["parse_completed"] and parsed["parse_error"] == 0,
                                    "later_actual_reparse_completed_" + name)
                scanned = self.native_action(editor, "native_edit_scan")
                observation.require(scanned["settled"] and scanned["events"] > scanned["before_events"],
                                    "later_actual_rescan_signal_" + name)
                self.native_runtime(project, 83)
                final, final_disks = self.state(editor, project)
                observation.require(final["target"]["B"] == final["target"]["R"] == CHANGED ==
                                    final_disks[TARGET]["text"] and
                                    all(final["current"][key] == history_after["current"][key] for key in
                                        ("B", "version", "saved_version", "dirty", "has_undo", "has_redo")) and
                                    final_disks[CURRENT] == disks[CURRENT],
                                    "ordinary_Save_reparse_rescan_runtime_preserve_target_and_human_history_" + name)
                self.record_witness(name + "_durable", human, disks, final, final_disks, editor)

    def closed_legacy_preservation(self):
        # Only representative reachable v6/native4 paths: no historical campaign.
        with self.closed_fixture("legacy-boundary", cached=True) as (project, editor, descriptor):
            vectors = self.action(editor, "proof_vectors")
            observation.require(vectors == {"server_matches": True, "client_matches": True, "finish_matches": True},
                                "matched_cross_language_v6_native4_vectors")
            stream, challenge = self.challenge(descriptor)
            tampered = dict(challenge, capabilities=dict(challenge["capabilities"]))
            tampered["capabilities"]["edit_closed_gdscript"] = False
            self.refused(stream, observation.packet(self.authenticate(
                descriptor, challenge, observation.proof(descriptor, tampered, "client"))))
            self.refused(self.connect(descriptor), observation.packet(
                [5, "hello", "old-peer", descriptor["session_id"], descriptor["project_root"], "a" * 64]))
            self.case("v6_native4_authentication_capability_binding", **vectors)
            self.close_action(editor, "close_human", path=TARGET, mutation="open")
            prior_open_basis = self.observe(project, "complete_observation", 0,
                                            session=descriptor["session_id"], name="legacy_prior_open_basis")
            self.close_action(editor, "close_human", path=TARGET, mutation="close")
            before, disks = self.state(editor, project)
            self.observe(project, "not_open", 0, session=descriptor["session_id"], name="legacy_closed_observation")
            self.edit(project, prior_open_basis, CHANGED, "legacy_edit_still_closed_refused", "refused", 3,
                      session=descriptor["session_id"], reason="closed_target", application="not_applied")
            self.assert_no_effect("legacy_edit_still_closed_refused", project, editor, before, disks)
            self.public_open(project, descriptor, "legacy_open")
            workflow_basis = self.workflow_read(project, editor, descriptor, "workflow_open_read", source=SAFE)
            self.workflow_edit(project, descriptor, workflow_basis["revision"], SAFE, "workflow_open_unchanged", "verified_unchanged")
            open_desired = "# WORKFLOW_OPEN_EDIT\n" + SAFE
            self.workflow_edit(project, descriptor, workflow_basis["revision"], open_desired,
                               "workflow_open_changed", "verified_changed")
            workflow_state, workflow_disks = self.state(editor, project)
            observation.require(workflow_state["target"]["R"] == workflow_state["target"]["B"] == open_desired ==
                                workflow_disks[TARGET]["text"], "actual_workflow_open_independent_D_R_B")
            basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"], name="legacy_open_observation")
            self.edit(project, basis, CHANGED, "legacy_open_edit", "verified_changed", 0, session=descriptor["session_id"])
            edited, edited_disks = self.state(editor, project)
            observation.require(edited["target"]["R"] == edited["target"]["B"] == CHANGED == edited_disks[TARGET]["text"],
                                "legacy_open_edit_independent_D_R_B")
            # Actual legacy close retains its original checked observation basis.
            basis = self.close_basis(project, descriptor, "legacy_close")
            pending = self.start_public_close(project, basis, session=descriptor["session_id"])
            self.complete_public_close(*pending, "legacy_close", "verified_newly_closed")
            closed, closed_disks = self.state(editor, project)
            observation.require(TARGET not in closed["open_paths"] and closed_disks[TARGET] == edited_disks[TARGET],
                                "legacy_close_preserves_persisted_source")
        # Discovery reaches the changed authentication and shared owner boundary.
        # Use one unignored inert source and independently check its real locator.
        with self.closed_fixture("legacy-discovery", setup=lambda project:
                                 (project / "catalog.gd").write_text(SAFE)) as (project, editor, descriptor):
            before, disks = self.state(editor, project)
            started = time.monotonic()
            result = subprocess.run([str(self.args.discoverer), "--registry", str(self.registry),
                                     "--project", str(project), "--session", descriptor["session_id"]],
                                    stdin=subprocess.DEVNULL, capture_output=True, timeout=5.2)
            observation.require(result.returncode == 0 and not result.stderr and time.monotonic() - started <= 5,
                                "legacy_discovery_supervised_bounded")
            discovered = json.loads(result.stdout)
            observation.require(discovered["outcome"] == "complete_listing" and
                                "res://catalog.gd" in discovered["inventory"]["entries"] and
                                discovered["inventory"]["coverage"] == "complete" and
                                discovered["resolved_target"]["session_id"] == descriptor["session_id"],
                                "legacy_discovery_actual_confined_inventory")
            observation.json_file(self.artifacts / "legacy-discovery.json", discovered)
            self.case("legacy_discovery", operation="discover_gdscripts", evidence="legacy-discovery.json")
            self.assert_no_effect("legacy_discovery", project, editor, before, disks)

    def closed_privacy_export(self):
        with self.closed_fixture("privacy", cached=True,
                                 source=SAFE + "# CLOSED_PRIVATE_TARGET\n") as (project, editor, descriptor):
            self.source_markers.add(b"CLOSED_PRIVATE_TARGET")
            self.source_markers.add(b"CLOSED_NEWER_WORK")
            basis = self.workflow_read(project, editor, descriptor, "privacy_permitted_read",
                                       source=SAFE + "# CLOSED_PRIVATE_TARGET\n")
            target = project / "scripts/subject.gd"
            target.chmod(0)
            try:
                denied = self.complete_workflow(self.start_workflow(project, descriptor, "read"),
                                                "privacy_denied_read", "read")
                observation.require(denied.get("source") is None and denied.get("revision") is None and
                                    denied["state"]["status"] == "denied_access" and
                                    denied["state"]["sources"] is None and
                                    sha(SAFE + "# CLOSED_PRIVATE_TARGET\n") not in json.dumps(denied) and
                                    b"CLOSED_PRIVATE_TARGET" not in json.dumps(denied).encode(),
                                    "denied_read_suppresses_all_source_authorities")
                denied_edit = self.workflow_edit(project, descriptor, basis["revision"], CHANGED,
                                                 "privacy_denied_edit", "refused")
                observation.require(denied_edit["outcome"]["reason"] == "denied_access" and
                                    denied_edit["outcome"]["target"] is None and
                                    denied_edit["outcome"]["requested_target"] is None and
                                    sha(SAFE + "# CLOSED_PRIVATE_TARGET\n") not in json.dumps(denied_edit) and
                                    b"CLOSED_PRIVATE_TARGET" not in json.dumps(denied_edit).encode(),
                                    "denied_edit_source_summaries_suppressed")
            finally:
                target.chmod(0o600)
            basis = self.workflow_read(project, editor, descriptor, "privacy_fresh_after_permission",
                                       source=SAFE + "# CLOSED_PRIVATE_TARGET\n")
            before, disks = self.state(editor, project)
            self.workflow_edit(project, descriptor, basis["revision"], CHANGED, "privacy_changed", "verified_changed")
            self.assert_closed_success("privacy_changed", project, editor, before, disks, CHANGED, changed=True)
        with self.closed_fixture("privacy-after-effects", cached=True,
                                 source=SAFE + "# CLOSED_PRIVATE_TARGET\n") as (project, editor, descriptor):
            basis = self.workflow_read(project, editor, descriptor, "privacy_effect_basis",
                                       source=SAFE + "# CLOSED_PRIVATE_TARGET\n")
            self.close_action(editor, "closed_arm", stage="verify:post_change")
            pending = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=CHANGED)
            self.wait_closed_barrier(editor, "verify:post_change", pending)
            target = project / "scripts/subject.gd"
            target.chmod(0)
            try:
                self.close_action(editor, "closed_release")
                denied = self.complete_workflow(pending, "privacy_denial_after_effect", "edit")
                self.review_workflow_edit(denied, "privacy_denial_after_effect",
                                          ("applied_unverified", "application_unknown"))
                observation.require(denied["outcome"]["application"] != "not_applied" and
                                    denied["outcome"]["evidence"] is None and
                                    b"CLOSED_PRIVATE_TARGET" not in json.dumps(denied).encode(),
                                    "later_denial_keeps_effect_facts_and_suppresses_evidence")
            finally:
                target.chmod(0o600)
            after, disks = self.state(editor, project)
            observation.require(disks[TARGET]["text"] == after["cached_R"] == CHANGED and
                                TARGET not in after["open_paths"], "denial_never_rolls_back_known_effects")
        for fault in ("partial_write", "lost_write", "mtime_failure"):
            for denial in ("chmod", "scope"):
                name = "privacy_" + fault + "_then_" + denial
                with self.closed_fixture(name, cached=True, faults=True) as (project, editor, descriptor):
                    basis = self.workflow_read(project, editor, descriptor, name + "_basis", source=SAFE)
                    before, disks = self.state(editor, project)
                    self.close_action(editor, "closed_arm", stage="apply")
                    pending = self.start_workflow(project, descriptor, "edit",
                                                  revision=basis["revision"], replacement_source=CHANGED)
                    self.wait_closed_barrier(editor, "apply", pending)
                    self.close_action(editor, "closed_fault", request_id=pending[2], fault=fault)
                    self.close_action(editor, "closed_arm", stage="verify:post_change")
                    self.wait_closed_barrier(editor, "verify:post_change", pending)
                    survivor, survivor_disks = self.state(editor, project)
                    target = project / "scripts/subject.gd"
                    if denial == "chmod":
                        target.chmod(0)
                    else:
                        target.rename(target.with_suffix(".private"))
                        target.symlink_to(project / CURRENT.removeprefix("res://"))
                    try:
                        self.close_action(editor, "closed_release")
                        result = self.complete_workflow(pending, name, "edit")
                        self.review_workflow_edit(result, name, "applied_unverified")
                        outcome = result["outcome"]
                        observation.require(outcome["reason"] == ("mtime_restore_failed" if fault == "mtime_failure" else "write_failed") and
                                            outcome["application"] in ("applied", "partly_applied") and
                                            outcome["effects"]["resource_entered"] and outcome["effects"]["disk_entered"] and
                                            outcome["evidence"] is None,
                                            "earlier_failure_known_effects_sticky_later_disclosure_denial_" + name)
                        encoded = json.dumps(result)
                        observation.require(all(sha(text) not in encoded and text not in encoded
                                                for text in (SAFE, CHANGED, survivor_disks[TARGET]["text"])) and
                                            '"utf8_bytes"' not in encoded and '"sha256"' not in encoded,
                                            "partial_denial_suppresses_source_hash_length_and_text_" + name)
                    finally:
                        if denial == "chmod":
                            target.chmod(0o600)
                        else:
                            target.unlink()
                            target.with_suffix(".private").rename(target)
                    after, now = self.state(editor, project)
                    observation.require(after["cached_R"] == survivor["cached_R"] == CHANGED and
                                        now[TARGET]["text"] == survivor_disks[TARGET]["text"] and
                                        documents(before) == documents(after) and TARGET not in after["open_paths"],
                                        "partial_denial_never_compensates_or_resumes_" + name)
                    self.record_witness(name, before, disks, after, now, editor)
        self.native_export()

    def closed_lifecycle(self):
        self.compile_window_probe()
        self.group("closed-cancel-and-newer-work", self.closed_cancellation)
        self.group("closed-later-durability-and-history", self.closed_durability)
        self.group("matched-v6-native4-legacy-preservation", self.closed_legacy_preservation)
        self.group("closed-privacy-export", self.closed_privacy_export)
