"""Actual public close caller groups, reusing the owned native/editor witnesses."""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import secrets
import signal
import stat
import subprocess
import tempfile
import time

import run_observation as observation
from discovery_scope_acceptance import exchange as scope_exchange
from close_native_acceptance import documents
from close_result_review import CLOSE_CODES, review_close_result
from run_script_edit import sha

TARGET = "res://scripts/subject.gd"
CURRENT = "res://scripts/other.gd"
BACKGROUND = "res://scripts/close/background.gd"
SAFE = "extends RefCounted\nfunc value() -> int:\n\treturn 47\n"


def _close_witness(state, disk):
    return {"documents": {path: {key: sha(value) if key in ("R", "B") and isinstance(value, str) else value
                                  for key, value in doc.items()} for path, doc in documents(state).items()},
            "disk": {path: {key: value for key, value in witness.items() if key != "text"}
                     for path, witness in disk.items()}, "selection": state["selection"],
            "cached_id": state["cached_id"], "original_target_alive": state["original_target_alive"],
            "events": state["events"], "effective_context": state["effective_context"],
            "trace": state.get("close_trace", [])}


class CallerCloseAcceptanceMixin:
    def close_command(self, project, session=None, script=TARGET):
        command = [str(self.args.closer), "--registry", str(self.registry), "--project", str(project),
                   "--script", script]
        if session is not None:
            command += ["--session", session]
        return command

    def close_basis(self, project, descriptor, name):
        return self.observe(project, "complete_observation", 0, session=descriptor["session_id"], name=name + "-basis")

    def start_public_close(self, project, basis, *, session=None, script=TARGET, data=None, eof=True):
        started = time.monotonic()
        process = subprocess.Popen(self.close_command(project, session, script), stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.open_processes.append(process)
        payload = {"schema_version": 1, "request_id": secrets.token_hex(16), "basis": basis}
        try:
            process.stdin.write(data if data is not None else json.dumps(payload, ensure_ascii=False).encode())
            process.stdin.flush()
        except BrokenPipeError:
            # Early bounded-input rejection is reviewed from the actual result.
            pass
        if eof:
            try:
                process.stdin.close()
            except BrokenPipeError:
                pass
            process.stdin = None
        return process, started

    def complete_public_close(self, process, started, name, expected, *, reason=None):
        try:
            stdout, stderr = process.communicate(timeout=max(.01, 10.2 - (time.monotonic() - started)))
        except subprocess.TimeoutExpired as error:
            process.kill()
            process.communicate(timeout=2)
            raise observation.Failure("public_close_deadline_exceeded_" + name) from error
        finally:
            if process.poll() is not None and process in self.open_processes:
                self.open_processes.remove(process)
        elapsed = time.monotonic() - started
        observation.require(elapsed <= 10 and stdout.endswith(b"\n") and stdout.count(b"\n") == 1 and
                            len(stdout) <= 12 * 1024 * 1024, "public_close_single_bounded_ten_second_result_" + name)
        result = json.loads(stdout)
        for raw in (stdout, stderr):
            observation.require(all(secret not in raw for secret in self.secrets) and
                                all(marker not in raw for marker in self.source_markers), "public_close_source_and_secret_free_" + name)
            observation.require(CURRENT.encode() not in raw and BACKGROUND.encode() not in raw,
                                "public_close_no_protected_paths_" + name)
        observation.json_file(self.artifacts / (name + ".json"), result)
        interpretation = review_close_result(result)
        allowed = {expected} if isinstance(expected, str) else set(expected)
        codes = 2 if result["reason"] == "invalid_request" else CLOSE_CODES[result["outcome"]]
        matches = result["outcome"] in allowed and process.returncode == codes and result["interval"]["elapsed_us"] <= 10000000
        matches = matches and (reason is None or result["reason"] in ({reason} if isinstance(reason, str) else set(reason)))
        if not matches:
            self.summary["failed_public_result"] = {"case": name, "outcome": result["outcome"], "reason": result["reason"],
                                                    "application": result["application"], "stage": result["stage"],
                                                    "exit_code": process.returncode, "elapsed_seconds": elapsed}
        observation.require(matches, "public_close_expected_outcome_" + name)
        self.safe_log(name + ".stderr", stderr)
        self.case(name, operation="close_gdscript", request_id=result["request_id"], outcome=result["outcome"],
                  reason=result["reason"], application=result["application"], elapsed_seconds=elapsed,
                  result_only_review=interpretation, evidence=name + ".json")
        return result

    def public_close(self, project, descriptor, basis, name, expected="verified_newly_closed", *,
                     reason=None, session=True, script=TARGET, data=None):
        process, started = self.start_public_close(project, basis,
                                                  session=descriptor["session_id"] if session is True else session,
                                                  script=script, data=data)
        return self.complete_public_close(process, started, name, expected, reason=reason)

    def close_evidence(self, name, before, disks, after, now, result, *, no_effect=False):
        encoded = json.dumps(result)
        target_sources = {value for value in (before["target"].get("R"), before["target"].get("B"),
                                              after["target"].get("R"), after["target"].get("B"),
                                              disks.get(TARGET, {}).get("text"), now.get(TARGET, {}).get("text"))
                          if isinstance(value, str)}
        for path, document in documents(before).items():
            if path == TARGET: continue
            observation.require(path not in encoded, "public_close_no_protected_locator_" + name)
            for authority in ("R", "B"):
                source = document[authority]
                if source not in target_sources:
                    observation.require(sha(source) not in encoded, "public_close_no_protected_digest_" + name)
        if result["outcome"] in ("verified_newly_closed", "already_closed_unchanged"):
            snapshot = result["observation"]["snapshot"]
            identity = snapshot["document"]["identity"]
            observation.require(result["resolved_target"]["script_path"] == TARGET and
                                result["resolved_target"]["session_id"] == after["session_id"] and
                                snapshot["document"]["open_state"]["value"] == "not_open" and
                                not after["target"]["associated"], "public_close_fresh_independent_exact_closed_target_" + name)
            sources = {"D": now.get(TARGET, {}).get("text"), "R": after["cached_R"], "B": None}
            for authority, surface in snapshot["sources"].items():
                if surface["availability"] == "observed":
                    source = sources[authority]
                    observation.require(isinstance(source, str) and surface["digest"] ==
                                        {"sha256": sha(source), "utf8_bytes": len(source.encode())} and
                                        surface["witness"] is not None and surface["collection"] is not None,
                                        "public_close_independent_fresh_authority_digest_" + authority + "_" + name)
                else:
                    observation.require(surface["digest"] is None, "public_close_unavailable_is_not_empty_" + name)
            if after["cached_id"]:
                observation.require(identity["script_instance_id"] == after["cached_id"],
                                    "public_close_fresh_resource_identity_" + name)
        if result["outcome"] == "already_closed_unchanged":
            observation.require(not before["target"]["associated"] and not after["target"]["associated"],
                                "recognition_never_certifies_an_open_target")
        if no_effect:
            observation.require(documents(before) == documents(after) and disks == now and
                                before["selection"] == after["selection"] and
                                before["cached_id"] == after["cached_id"] and
                                before["cached_R"] == after["cached_R"] and
                                before["loader_calls"] == after["loader_calls"],
                                "public_close_independent_no_effect_" + name)
        elif result["outcome"] == "verified_newly_closed":
            protected = {path: doc for path, doc in documents(before).items() if path != TARGET}
            observation.require(documents(after) == protected and not after["target"]["associated"] and
                                disks == now, "public_close_exact_roster_and_independent_D_" + name)
            observation.require(before["selection"] == after["selection"] if before["selection"] != TARGET else
                                after["selection"] in protected if protected else after["selection"] == "",
                                "public_close_native_selection_only_" + name)
            if after["cached_id"]:
                observation.require(after["cached_id"] == before["target"]["script_id"] and
                                    after["cached_R"] == before["target"]["R"] and
                                    after["cached_edited"] == before["target"]["resource_edited"] and
                                    result["resource_state"]["state"] == "retained", "public_close_original_retained_R_" + name)
            else:
                observation.require(not after["original_target_alive"] and result["resource_state"]["state"] == "unloaded",
                                    "public_close_genuinely_unloaded_R_" + name)
            snapshot = result["observation"]["snapshot"]
            observation.require(snapshot["document"]["open_state"]["value"] == "not_open" and
                                snapshot["sources"]["B"]["availability"] == "not_applicable",
                                "public_close_fresh_buffer_absence_not_empty_" + name)
            required = result["protection"]["required_count"]
            observation.require(type(required) is int and result["protection"]["completed_count"] == required,
                                "public_close_all_continuation_obligations_" + name)
            # Fixture records actual native callbacks independently of the caller's ledger.
            events = after["events"][len(before["events"]):]
            visits = {event["script_id"] for event in events if event["event"] == "visit"}
            observation.require(required == len({doc["editor_id"] for doc in protected.values()
                                                  if doc["script_id"] in visits}),
                                "public_close_exact_independent_continuation_obligation_count_" + name)
            for doc in protected.values():
                if doc["script_id"] in visits:
                    last_visit = max(int(event["tick_us"]) for event in events
                                     if event["event"] == "visit" and event["script_id"] == doc["script_id"])
                    observation.require(any(event["event"] == "completion" and event["editor_id"] == doc["editor_id"] and
                                            int(event["tick_us"]) >= last_visit for event in events),
                                        "public_close_independent_native_completion_" + name)
        observation.json_file(self.artifacts / (name + "-witness.json"),
                              {"before": _close_witness(before, disks), "after": _close_witness(after, now)})
        case = next(case for case in reversed(self.cases) if case["case"] == name)
        case.update(independent_evidence=name + "-witness.json", screenshot=self.screenshot(self._close_case_editor, name + ".png"))

    def close_history(self, editor, path):
        original = self.close_action(editor, "close_document", path=path)["document"]
        observation.require(original["has_undo"], "public_close_real_unrelated_prior_undo")
        self.close_action(editor, "close_human", path=path, mutation="select")
        undone = self.close_action(editor, "close_human", path=path, mutation="undo")["state"]
        undo = next(doc for doc in undone["documents"] if doc["path"] == path)
        redone = self.close_action(editor, "close_human", path=path, mutation="redo")["state"]
        redo = next(doc for doc in redone["documents"] if doc["path"] == path)
        observation.require(undo["B"] != original["B"] and undo["has_redo"] and redo["B"] == original["B"] and
                            redo["has_undo"], "public_close_actual_unrelated_undo_redo_reachability")
        return {"undo_sha256": sha(undo["B"]), "redo_sha256": sha(redo["B"])}

    def clean_close(self):
        self.compile_window_probe()
        profiles = (("selected_retained", {"retained": True}), ("nonselected", {"selected": CURRENT}),
                    ("last", {"paths": [TARGET]}), ("stable_two", {"paths": [TARGET, CURRENT]}),
                    ("empty", {"source": "", "paths": [TARGET]}),
                    ("readonly", {"read_only": True, "paths": [TARGET]}),
                    ("safe_invalid", {"source": "extends RefCounted\nfunc value(:\n", "paths": [TARGET]}),
                    ("unloaded", {"paths": [TARGET], "replacement_target": True}))
        profiles = profiles[1:2]
        self.summary["private_case_filter"] = "public_close_nonselected"
        for profile, options in profiles:
            name = "public_close_" + profile
            with self.close_fixture(name, **options) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                observation.json_file(self.artifacts / (name + "-before.json"), _close_witness(before, disks))
                try:
                    result = self.public_close(project, descriptor, basis, name)
                finally:
                    after, now = self.state(editor, project)
                    observation.json_file(self.artifacts / (name + "-after.json"), _close_witness(after, now))
                self.close_evidence(name, before, disks, after, now, result)
                self.observe(project, "not_open", 0, session=descriptor["session_id"], name=name + "-closed-observation")
                self.public_open(project, descriptor, name + "-intentional-reopen")
                reopened, reopened_disk = self.state(editor, project)
                observation.require(reopened["target"]["associated"] and reopened["target"]["B"] == disks[TARGET]["text"] and
                                    reopened_disk == disks, "public_close_separate_reopen_persisted_revision_" + profile)
        return
        self.first_use_composition()
        # Reuse established A-E regressions without advertising a new close
        # matrix. These consumers use the existing subject/other oracle.
        self.state = self.snapshot
        try:
            self.group("existing-history-and-durability", self.history_edit)
            self.group("existing-twenty-edit-stress", self.sequential_edit)
        finally:
            del self.state

    def first_use_composition(self):
        def discoverable(project):
            (project / "scripts/.gdignore").unlink()
        with self.close_fixture("first-public-composition", paths=[CURRENT], selected=CURRENT,
                                setup=discoverable) as (project, editor, descriptor):
            self._close_case_editor = editor
            discovery = subprocess.run([str(self.args.discoverer), "--registry", str(self.registry), "--project", str(project),
                                        "--session", descriptor["session_id"]], capture_output=True, timeout=5.2)
            found = json.loads(discovery.stdout)
            entries = found.get("inventory", {}).get("entries", [])
            observation.require(discovery.returncode == 0 and found["outcome"] == "complete_listing" and
                                entries.count(TARGET) == 1 and
                                all(secret not in discovery.stdout + discovery.stderr for secret in self.secrets),
                                "first_public_discovery_exact_locator")
            locator = next(path for path in entries if path == TARGET)
            self.public_open(project, descriptor, "first_public_open", script=locator)
            basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                 name="first_public_observe-basis", script=locator)
            desired = SAFE.replace("47", "83")
            self.edit(project, basis, desired, "first_public_edit", "verified_changed", 0,
                      session=descriptor["session_id"], script=locator)
            fresh = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                 name="first_public_fresh_observe-basis", script=locator)
            before, disks = self.state(editor, project)
            observation.require(before["target"]["R"] == before["target"]["B"] == disks[TARGET]["text"] == desired,
                                "first_public_A_independent_edit_convergence")
            result = self.public_close(project, descriptor, fresh, "first_public_close", script=locator)
            after, now = self.state(editor, project)
            self.close_evidence("first_public_close", before, disks, after, now, result)
            self.edit(project, fresh, desired, "first_public_closed_edit_refuses", "refused", 3,
                      session=descriptor["session_id"], reason="closed_target", application="not_applied", script=locator)
            self.public_open(project, descriptor, "first_public_D_reopen", script=locator)
            reopened, disk = self.state(editor, project)
            observation.require(reopened["target"]["R"] == reopened["target"]["B"] == disk[TARGET]["text"] == desired,
                                "first_public_D_persisted_exact_reopen")
            reopened_basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                          name="first_public_reopened_clean_basis", script=locator)
            self.close_action(editor, "close_human", path=TARGET, mutation="dirty_disk_equal")
            dirty, dirty_disk = self.state(editor, project)
            observation.require(dirty["target"]["dirty"] and
                                dirty["target"]["B"] == dirty["target"]["R"] == dirty_disk[TARGET]["text"] == desired,
                                "first_public_equal_text_is_attributably_dirty")
            refused = self.public_close(project, descriptor, reopened_basis, "first_public_B_equal_dirty_refuses", "refused",
                                        script=locator, reason="dirty_conflict")
            self.close_evidence("first_public_B_equal_dirty_refuses", dirty, dirty_disk,
                                *self.state(editor, project), refused, no_effect=True)
            self.native_action(editor, "native_edit_save")
            self.native_action(editor, "native_edit_reparse")
            self.native_action(editor, "native_edit_scan")
            coherent, persisted = self.state(editor, project)
            observation.require(coherent["target"]["R"] == coherent["target"]["B"] == persisted[TARGET]["text"] == desired,
                                "first_public_ordinary_Save_reparse_rescan_keeps_revision")
            self.native_runtime(project, 83)

    def already_closed(self):
        self.compile_window_probe()
        profiles = (("unloaded", {"paths": [], "selected": ""}),
                    ("cached", {"paths": [], "selected": ""}),
                    ("cached_divergent", {"paths": [], "selected": ""}),
                    ("source_limited", {"paths": [], "selected": "", "source": SAFE + "#" + "x" * 524288}))
        for profile, options in profiles:
            with self.close_fixture("recognition-" + profile, **options) as (project, editor, descriptor):
                self._close_case_editor = editor
                if profile.startswith("cached"): self.open_action(editor, "open_cache")
                if profile == "cached_divergent":
                    self.source_markers.add(b"CLOSE_PRIVATE_CACHED_SOURCE")
                    self.close_action(editor, "close_cached_source")
                before, disks = self.state(editor, project)
                name = "public_already_closed_" + profile
                result = self.public_close(project, descriptor, None, name, "already_closed_unchanged")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
                observation.require(not after["slot_busy"], "recognition_releases_slot_without_close_context")
        with self.close_fixture("recognition-old-basis", retained=True) as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "recognition-old-open")
            prior, prior_disk = self.state(editor, project)
            newly_closed = self.public_close(project, descriptor, basis, "recognition-product-first-close")
            self.close_evidence("recognition-product-first-close", prior, prior_disk,
                                *self.state(editor, project), newly_closed)
            before, disks = self.state(editor, project)
            result = self.public_close(project, descriptor, basis, "public_old_basis_not_applied", "already_closed_unchanged")
            after, now = self.state(editor, project)
            observation.require(result["expected"]["use"] == "not_applied_to_closed_state", "old_basis_not_retroactive_certification")
            self.close_evidence("public_old_basis_not_applied", before, disks, after, now, result, no_effect=True)
            self.close_action(editor, "close_file", mutation="replace_same")
            baseline, disk = self.state(editor, project)
            result = self.public_close(project, descriptor, basis, "public_closed_replacement_file_refuses", "refused")
            after, now = self.state(editor, project)
            self.close_evidence("public_closed_replacement_file_refuses", baseline, disk, after, now, result, no_effect=True)
        for count in (2, 9):
            paths = [CURRENT, BACKGROUND] + ["res://scripts/close/extra_%d.gd" % number for number in range(1, count - 1)]
            name = "public_recognition_protected_history_" + str(count)
            with self.close_fixture(name, paths=paths, selected=CURRENT) as (project, editor, descriptor):
                self._close_case_editor = editor
                self.close_action(editor, "close_human", path=CURRENT, mutation="dirty_equal")
                self.open_action(editor, "open_setup", paths=[], path=CURRENT, idle=True)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, None, name, "already_closed_unchanged")
                self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=True)
                self.cases[-1]["native_unrelated_history"] = self.close_history(editor, CURRENT)
        with self.close_fixture("recognition-replaced-session") as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "recognition-original-session")
            before, disks = self.state(editor, project)
            closed = self.public_close(project, descriptor, basis, "recognition-before-session-replacement")
            self.close_evidence("recognition-before-session-replacement", before, disks, *self.state(editor, project), closed)
            self.close_action(editor, "close_lifetime", control="disable")
            self.close_action(editor, "close_lifetime", control="enable")
            replacement = observation.wait_for(lambda: next((value for value in self.descriptors(project)
                if value["session_id"] != descriptor["session_id"]), None), "fresh_close_session_lifetime")
            before, disks = self.state(editor, project)
            refused = self.public_close(project, replacement, basis, "public_closed_wrong_session_basis", "refused", reason="session_changed")
            self.close_evidence("public_closed_wrong_session_basis", before, disks, *self.state(editor, project), refused, no_effect=True)

    def held_public_close(self, project, editor, descriptor, basis, name, stage="advance", *, response_kind="",
                          native_stage="", native_fault="", callback="", response_purpose=""):
        event = editor["control"] / "close-event.json"
        event.unlink(missing_ok=True)
        self.close_action(editor, "close_arm", stage=stage, response_kind=response_kind, native_stage=native_stage,
                          native_fault=native_fault, callback=callback,
                          response_purpose=response_purpose or ("post_close" if response_kind == "close_rechecked" else ""))
        process, started = self.start_public_close(project, basis, session=descriptor["session_id"])
        expected = "response:" + response_kind if response_kind else stage
        def reached():
            if event.is_file():
                value = json.loads(event.read_text())
                if value.get("stage") == expected: return value
            observation.require(process.poll() is None and editor["process"].poll() is None,
                                "public_close_live_barrier_" + name)
            return None
        observed = observation.wait_for(reached, "public_close_actual_barrier_" + name, timeout=7)
        return process, started, observed

    def preservation(self):
        self.compile_window_probe()
        for mutation in ("dirty_different", "dirty_disk_equal", "resource_edited", "resource_source", "same_text", "reopen", "replace"):
            name = "public_target_" + mutation + "_refuses"
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.close_action(editor, "close_human", path=TARGET, mutation=mutation)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
                if before["target"]["has_undo"]: self.close_history(editor, TARGET)
        for basis_kind in ("null", "closed", "dirty", "wrong_session"):
            with self.close_fixture("basis-" + basis_kind) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = None if basis_kind == "null" else self.close_basis(project, descriptor, "basis-" + basis_kind)
                if basis_kind == "closed":
                    prior, prior_disk = self.state(editor, project)
                    closed = self.public_close(project, descriptor, basis, "basis-close-preparation")
                    self.close_evidence("basis-close-preparation", prior, prior_disk, *self.state(editor, project), closed)
                    basis = self.observe(project, "not_open", 0, session=descriptor["session_id"], name="closed-ineligible-basis")
                    self.public_open(project, descriptor, "basis-deliberate-reopen")
                elif basis_kind == "dirty":
                    self.close_action(editor, "close_human", path=TARGET, mutation="dirty_disk_equal")
                    basis = self.close_basis(project, descriptor, "dirty-ineligible-basis")
                elif basis_kind == "wrong_session":
                    basis = copy.deepcopy(basis)
                    basis["resolved_target"]["session_id"] = "0" * 32
                before, disks = self.state(editor, project)
                name = "public_" + basis_kind + "_basis_refuses"
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
        for path in (CURRENT, BACKGROUND):
            name = "public_protected_dirty_equal_" + ("current" if path == CURRENT else "background")
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                self.close_action(editor, "close_human", path=path, mutation="select")
                self.close_action(editor, "close_human", path=path, mutation="dirty_equal")
                self.open_action(editor, "open_setup", paths=[], path=path, idle=True)
                self.close_history(editor, path)
                self.open_action(editor, "open_setup", paths=[], path=path, idle=True)
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                doc = next(doc for doc in before["documents"] if doc["path"] == path)
                observation.require(doc["dirty"] and doc["R"] == doc["B"] and doc["has_undo"], "real_dirty_current_R_equals_B")
                result = self.public_close(project, descriptor, basis, name)
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result)
                self.cases[-1]["native_history_after_close"] = self.close_history(editor, path)
        self.close_races()
        self.close_context_profiles()
        self.close_public_bounds()
        self.close_unavailable_surfaces()
        self.close_pending_metadata()
        self.close_representation_refusals()
        self.close_history_destinations()

    def close_history_destinations(self):
        for sorting in (0, 1, 2):
            name = "public_close_history_sort_" + str(sorting)
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                self.close_action(editor, "close_config", setting="sort_scripts", value=sorting)
                for path in (CURRENT, BACKGROUND):
                    self.close_action(editor, "close_human", path=path, mutation="dirty_equal")
                    self.open_action(editor, "open_setup", paths=[], path=path, idle=True)
                for path in (BACKGROUND, CURRENT, TARGET, BACKGROUND, TARGET):
                    self.close_action(editor, "close_human", path=path, mutation="select")
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name)
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result)
                self.cases[-1]["native_unrelated_history"] = [
                    self.close_history(editor, path) for path in (CURRENT, BACKGROUND)]

    def close_unavailable_surfaces(self):
        for restriction in ("resource", "buffer", "dirty", "association", "open"):
            name = "public_unavailable_" + restriction
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.action(editor, "restrict_" + restriction)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
        for operation in ("missing_D", "external_D_stales_R_B"):
            name = "public_source_" + operation
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.close_action(editor, "close_file", mutation="remove" if operation == "missing_D" else "source")
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)

    def close_representation_refusals(self):
        for label, source in (("CR", SAFE.replace("\n", "\r\n")), ("BOM", "\ufeff" + SAFE),
                              ("NUL", SAFE + "\x00"), ("control", SAFE + "\x01")):
            name = "public_unsupported_source_" + label
            with self.close_fixture(name, source=source, paths=[TARGET]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
        for setting in ("idle_parse_delay", "idle_parse_error_delay", "external_editor", "autoload"):
            name = "public_unsafe_setting_" + setting
            def setup(project):
                if setting == "autoload":
                    with (project / "project.godot").open("a") as config:
                        config.write('\n[autoload]\nCloseFixtureAutoload="*res://scripts/close/autoload.gd"\n')
                    (project / "scripts/other.gd").write_text(
                        "extends RefCounted\nfunc value():\n\treturn CloseFixtureAutoload\n")
            with self.close_fixture(name, setup=setup) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.close_action(editor, "close_config", setting=setting,
                                  value=True if setting in ("autoload", "external_editor") else 20.0)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)

    def close_pending_metadata(self):
        profiles = (("primitive", "extends Node\n@export var payload: int = 0\n\n"),
                    ("object", "extends Node\nvar payload: Resource\n\n"),
                    ("exported", "extends Node\n@export var payload: Resource\n\n"))
        for label, source in profiles:
            for undo in (False, True):
                def setup(project, source=source):
                    (project / "scripts/other.gd").write_text(source)
                name = "public_pending_" + label + ("_undo" if undo else "")
                with self.close_fixture(name, paths=[TARGET, CURRENT], setup=setup) as (project, editor, descriptor):
                    self._close_case_editor = editor
                    basis = self.close_basis(project, descriptor, name)
                    self.close_action(editor, "close_config", setting="idle_parse_delay", value=8)
                    drag = self.open_action(editor, "open_pending_drag", undo=undo)["drag"]
                    observation.require(drag["dragged"]["B"] != drag["before"]["B"] and drag["dragged"]["has_undo"] and
                                        (not undo or drag["after"]["B"] == drag["before"]["B"]),
                                        "actual_native_pending_drop_and_history")
                    before, disks = self.state(editor, project)
                    result = self.public_close(project, descriptor, basis, name, "refused")
                    after, now = self.state(editor, project)
                    observation.require(after["pending_node_payload_id"] == before["pending_node_payload_id"],
                                        "public_refusal_preserves_pending_inspector_property")
                    self.close_evidence(name, before, disks, after, now, result, no_effect=True)
        def setup(project):
            (project / "scripts/other.gd").write_text("extends Node\n@export var payload: Resource\n\n")
        with self.close_fixture("public-stale-compiled", paths=[TARGET, CURRENT], setup=setup) as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "public-stale-compiled")
            self.close_action(editor, "close_config", setting="idle_parse_delay", value=8)
            self.open_action(editor, "open_pending_drag", undo=True)
            self.open_action(editor, "open_mutate", mutation="stale_metadata")
            before, disks = self.state(editor, project)
            observation.require(before["current"]["R"] == before["current"]["B"] and
                                "@export" not in before["current"]["R"] and
                                any(item["name"] == "payload" for item in before["current"]["properties"]),
                                "real_stale_compiled_property_despite_equal_sources")
            result = self.public_close(project, descriptor, basis, "public_stale_compiled_refuses", "refused")
            self.close_evidence("public_stale_compiled_refuses", before, disks, *self.state(editor, project), result, no_effect=True)

    def close_races(self):
        controls = (("target_version", "close_human", {"path": TARGET, "mutation": "same_text"}),
                    ("target_reopen", "close_human", {"path": TARGET, "mutation": "reopen"}),
                    ("target_typing", "close_human", {"path": TARGET, "mutation": "dirty_different"}),
                    ("protected_version", "close_human", {"path": CURRENT, "mutation": "same_text"}),
                    ("ordinary_Save", "human_Save", {}),
                    ("roster_add", "close_human", {"path": "res://scripts/close/extra_1.gd", "mutation": "open"}),
                    ("roster_remove", "close_human", {"path": BACKGROUND, "mutation": "close"}),
                    ("roster_replace", "close_human", {"path": CURRENT, "mutation": "replace"}),
                    ("selection", "close_human", {"path": CURRENT, "mutation": "select"}),
                    ("file_identity", "close_file", {"mutation": "replace_same"}),
                    ("protected_file_identity", "close_file", {"path": CURRENT, "mutation": "replace_same"}),
                    ("file_source", "close_file", {"mutation": "source"}),
                    ("missing_D", "close_file", {"mutation": "remove"}),
                    ("warnings", "close_config", {"setting": "warning", "value": 2}),
                    ("idle_delay", "close_config", {"setting": "idle_parse_delay", "value": .8}),
                    ("error_delay", "close_config", {"setting": "idle_parse_error_delay", "value": .8}),
                    ("sorting", "close_config", {"setting": "sort_scripts", "value": 2}),
                    ("autoload", "close_config", {"setting": "autoload", "value": True}),
                    ("external_editor", "close_config", {"setting": "external_editor", "value": True}))
        for label, action, arguments in controls:
            name = "public_preentry_race_" + label
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, "recheck:pre_close")
                if action == "human_Save":
                    self.close_action(editor, "close_human", path=TARGET, mutation="same_text")
                    self.native_action(editor, "native_edit_save")
                else:
                    self.close_action(editor, action, **arguments)
                baseline, disks = self.state(editor, project)
                self.close_action(editor, "close_release")
                result = self.complete_public_close(process, started, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, baseline, disks, after, now, result, no_effect=True)

    def close_context_profiles(self):
        profiles = ("unsafe_tool", "unsafe_static", "unsafe_export", "unsafe_load", "unsafe_preload", "invalid")
        for profile in profiles:
            path = "res://scripts/close/" + profile + ".gd"
            name = "public_context_" + profile
            with self.close_fixture(name, paths=[TARGET, path]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
        for path in (CURRENT, BACKGROUND):
            name = "public_protected_R_not_B_" + ("current" if path == CURRENT else "background")
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.close_action(editor, "close_config", setting="idle_parse_delay", value=8)
                self.close_action(editor, "close_human", path=path, mutation="dirty_different")
                before, disks = self.state(editor, project)
                doc = next(doc for doc in before["documents"] if doc["path"] == path)
                observation.require(doc["R"] != doc["B"], "real_protected_pending_source_application_negative")
                result = self.public_close(project, descriptor, basis, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)

    def close_public_bounds(self):
        for size in (524288, 524289):
            source = SAFE + "#" + "x" * (size - len(SAFE.encode()) - 2) + "\n"
            name = "public_target_source_bound_" + str(size)
            with self.close_fixture(name, source=source, paths=[TARGET]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = (self.close_basis(project, descriptor, name) if size == 524288 else
                         self.observe(project, "limited_observation", 2, session=descriptor["session_id"], name=name + "-limited-basis"))
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name,
                                           "verified_newly_closed" if size == 524288 else "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=size > 524288)
        for count in (8, 9):
            paths = [TARGET, CURRENT, BACKGROUND] + ["res://scripts/close/extra_%d.gd" % n for n in range(1, count - 2)]
            name = "public_roster_bound_" + str(count)
            with self.close_fixture(name, paths=paths) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                # Maximum contexts may exhaust validation budget, but must not apply if they do.
                result = self.public_close(project, descriptor, basis, name,
                                           ("verified_newly_closed", "refused") if count == 8 else "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=result["outcome"] == "refused")
        for size in (524288, 524289):
            def setup(project, size=size):
                for relative, length in zip(("scripts/other.gd", "scripts/close/background.gd"),
                                            (size // 2, size - size // 2)):
                    (project / relative).write_text(SAFE + "#" + "x" * (length - len(SAFE.encode()) - 2) + "\n")
            name = "public_remaining_source_bound_" + str(size)
            with self.close_fixture(name, setup=setup) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                observation.require(sum(len(doc["R"].encode()) for doc in before["documents"] if doc["path"] != TARGET) == size,
                                    "public_independent_exact_remaining_source_aggregate")
                result = self.public_close(project, descriptor, basis, name,
                                           ("verified_newly_closed", "refused") if size == 524288 else "refused",
                                           reason=("complete", "context_validation_unavailable", "timeout") if size == 524288 else "context_limit")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=result["outcome"] == "refused")
        def public_metadata(project, editor, descriptor, size):
            self._close_case_editor = editor
            name = "public_metadata_bound_" + str(size)
            basis = self.close_basis(project, descriptor, name)
            before, disks = self.state(editor, project)
            result = self.public_close(project, descriptor, basis, name,
                                       ("verified_newly_closed", "refused") if size == 262144 else "refused",
                                       reason=("complete", "context_validation_unavailable", "timeout") if size == 262144 else "context_limit")
            self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=result["outcome"] == "refused")
        self.compiled_metadata_bounds(public_callback=public_metadata)

    def routing(self):
        self.compile_window_probe()
        with self.close_fixture("routing-exact") as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "routing-exact")
            for label, script in (("missing", "res://scripts/missing.gd"), ("outside", "res://../outside.gd"),
                                  ("embedded", "res://main.tscn::GDScript_dead"), ("non_gdscript", "res://main.tscn")):
                before, disks = self.state(editor, project)
                name = "public_routing_" + label
                result = self.public_close(project, descriptor, None, name, "refused", script=script)
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)
            before, disks = self.state(editor, project)
            result = self.public_close(project, descriptor, basis, "public_ended_session", "refused", session="0" * 32)
            after, now = self.state(editor, project)
            self.close_evidence("public_ended_session", before, disks, after, now, result, no_effect=True)
            second = self.start_editor(project)
            try:
                observation.wait_for(lambda: len(self.descriptors(project)) == 2, "close_real_live_ambiguous_session")
                # Registry availability precedes the editor's restored-tab work.
                self.close_action(second, "close_setup", paths=[TARGET, CURRENT, BACKGROUND], selected=TARGET)
                second_before, second_disk = self.state(second, project)
                result = self.public_close(project, descriptor, basis, "public_ambiguous_basis_does_not_select", "refused",
                                           session=None, reason="ambiguous_session")
                after, now = self.state(editor, project)
                self.close_evidence("public_ambiguous_basis_does_not_select", before, disks, after, now, result, no_effect=True)
                second_after, second_now = self.state(second, project)
                # Ordinary startup can still emit visits/load unrelated editor
                # resources. Compare document/source/history effects, not that activity.
                observation.require(documents(second_before) == documents(second_after) and
                                    second_disk == second_now and all(
                                        second_before[key] == second_after[key] for key in
                                        ("selection", "cached_id", "cached_R", "cached_edited",
                                         "effective_context", "pending_node_payload_id")) and
                                    not second_after["slot_busy"], "ambiguous_other_session_untouched")
            finally:
                self.close_editor(second)
                self.editors.remove(second)
            target = project / "scripts/subject.gd"
            target.chmod(0)
            try:
                denied = self.public_close(project, descriptor, basis, "public_denied_file", "refused", reason="denied_access")
            finally:
                target.chmod(0o600)
            after, now = self.state(editor, project)
            observation.require(documents(before) == documents(after) and
                                all(now[path]["sha256"] == witness["sha256"] and
                                    now[path]["inode"] == witness["inode"] for path, witness in disks.items()),
                                "denied_close_no_source_or_effect")
            self.close_evidence("public_denied_file", before, disks, after, now, denied)
            metadata = next(path for path in self.registry.glob("*.json") if
                            json.loads(path.read_text())["session_id"] == descriptor["session_id"])
            original = metadata.read_bytes()
            replacement = json.loads(original)
            replacement["token"] = secrets.token_hex(32)
            self.secrets.add(replacement["token"].encode())
            observation.json_file(metadata, replacement)
            try:
                prior, prior_disk = self.state(editor, project)
                denied = self.public_close(project, descriptor, basis, "public_denied_authentication", "refused", reason="denied_access")
                observation.require(denied["before"] is None and denied["observation"] is None,
                                    "unauthenticated_close_has_no_candidate_source")
                self.close_evidence("public_denied_authentication", prior, prior_disk, *self.state(editor, project), denied, no_effect=True)
            finally:
                metadata.write_bytes(original)
                metadata.chmod(0o600)
        self.close_namespace_races()
        self.close_operation_overlap()
        self.close_unsupported_contexts()
        self.close_input_boundaries()

    def close_unsupported_contexts(self):
        for action in ("mixed_tabs", "unpathed_script", "duplicate_script"):
            name = "public_unsupported_context_" + action
            def setup(project):
                if action == "mixed_tabs":
                    (project / "scripts/.gdignore").unlink()
            with self.close_fixture(name, setup=setup) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                self.action(editor, "prepare_subject")
                self.action(editor, action)
                before = self.action(editor, "witness")
                disk = observation.disk_witness(project / "scripts/subject.gd")
                result = self.public_close(project, descriptor, basis, name, "refused")
                after = self.action(editor, "witness")
                now = observation.disk_witness(project / "scripts/subject.gd")
                held_buffer = after["prepared_subject_buffer"]
                observation.require(before == after and disk == now and held_buffer is not None,
                                    "unsupported_association_preserves_real_held_buffer_and_roster")
                held = {key: value for key, value in held_buffer.items() if key != "text"}
                held["sha256"] = sha(held_buffer["text"])
                observation.json_file(self.artifacts / (name + "-witness.json"), {
                    "held_target": held, "disk": {key: value for key, value in disk.items() if key != "text"},
                    "script_ids": after["script_ids"], "editor_ids": after["editor_ids"],
                    "script_types": after["script_types"], "editor_types": after["editor_types"]})
                self.cases[-1].update(independent_evidence=name + "-witness.json",
                                     screenshot=self.screenshot(editor, name + ".png"))
        for label, source in (("tool", "@tool\nextends RefCounted\n"),
                              ("export", "extends RefCounted\n@export var value: int\n"),
                              ("static", "extends RefCounted\nstatic var value: int = 3\n"),
                              ("base", 'extends "res://scripts/close/safe.gd"\n'),
                              ("global", "class_name CloseUnsafeGlobal\nextends RefCounted\n")):
            name = "public_target_profile_" + label
            with self.close_fixture(name, source=source, paths=[TARGET]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, name, "refused")
                self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=True)

    def close_input_boundaries(self):
        with self.close_fixture("public-input-boundaries") as (project, editor, descriptor):
            self._close_case_editor = editor
            good = b'{"schema_version":1,"request_id":"strict-close","basis":null}'
            variants = (("unknown", good[:-1] + b',"force":true}'),
                        ("duplicate", good.replace(b'"schema_version":1', b'"schema_version":1,"schema_version":1')),
                        ("trailing", good + b' {}'), ("invalid_utf8", good + b'\xff'),
                        ("deep", b'{"schema_version":1,"request_id":"strict-close","basis":' + b'[' * 33 + b'0' + b']' * 33 + b'}'),
                        ("truncated", good[:-1]))
            for label, wire in variants:
                before, disks = self.state(editor, project)
                name = "public_invalid_input_" + label
                result = self.public_close(project, descriptor, None, name, "refused", data=wire, reason="invalid_request")
                self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=True)
            for size in (12 * 1024 * 1024, 12 * 1024 * 1024 + 1):
                wire = good + b" " * (size - len(good))
                name = "public_stdin_bound_" + str(size)
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, None, name, "refused", data=wire,
                                           reason="missing_basis" if size == 12 * 1024 * 1024 else "invalid_request")
                self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=True)

    def close_namespace_races(self):
        for label in ("leaf", "parent", "symlink"):
            with self.close_fixture("namespace-" + label) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, "namespace-" + label)
                process, started, _ = self.held_public_close(project, editor, descriptor, basis, label, "recheck:pre_close")
                target = project / "scripts/subject.gd"
                if label == "leaf":
                    target.rename(target.with_suffix(".original"))
                    target.write_text(SAFE)
                elif label == "parent":
                    directory = project / "scripts"
                    directory.rename(project / "original-scripts")
                    directory.mkdir()
                    target.write_text(SAFE)
                else:
                    outside = self.work / "outside-close.gd"
                    outside.write_text(SAFE + "# CLOSE_OTHER_PROJECT\n")
                    self.source_markers.add(b"CLOSE_OTHER_PROJECT")
                    target.rename(target.with_suffix(".original"))
                    target.symlink_to(outside)
                before, disks = self.state(editor, project)
                self.close_action(editor, "close_release")
                name = "public_namespace_" + label
                result = self.complete_public_close(process, started, name, "refused")
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=True)

    def close_operation_overlap(self):
        with self.close_fixture("operation-overlap") as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "overlap")
            process, started, _ = self.held_public_close(project, editor, descriptor, basis, "close-owner", "prepare")
            before, disks = self.state(editor, project)
            result = self.public_close(project, descriptor, basis, "public_close_overlapping_close", "refused", reason="busy")
            self.close_evidence("public_close_overlapping_close", before, disks, *self.state(editor, project), result, no_effect=True)
            self.public_open(project, descriptor, "close_owner_refuses_open", "refused", reason="busy")
            self.edit(project, basis, SAFE, "close_owner_refuses_edit", "refused", 3, session=descriptor["session_id"], reason="busy")
            self.observe(project, "unsupported_observation", 2, session=descriptor["session_id"], name="close_owner_refuses_observe")
            discovery = subprocess.run([str(self.args.discoverer), "--registry", str(self.registry), "--project", str(project),
                                        "--session", descriptor["session_id"]], capture_output=True, timeout=5.2)
            observation.require(discovery.returncode == 3 and "busy" in discovery.stdout.decode(), "close_owner_refuses_discovery")
            process.send_signal(signal.SIGTERM)
            cancelled = self.complete_public_close(process, started, "public_close_overlap_owner_cancel", "refused")
            self.close_action(editor, "close_release")
            self.close_evidence("public_close_overlap_owner_cancel", before, disks,
                                *self.state(editor, project), cancelled, no_effect=True)
        for operation in ("open", "edit", "observe", "discovery"):
            with self.close_fixture("other-owner-" + operation, paths=[TARGET, CURRENT]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, "other-owner-" + operation)
                if operation == "open":
                    owner, started, _ = self.held_open(project, editor, descriptor, "begin", "open-owner")
                elif operation == "edit":
                    owner, payload, started = self.held_edit(project, editor, basis, SAFE.replace("47", "83"), "prepare",
                                                            "edit-owner", session=descriptor["session_id"])
                else:
                    owner, owner_id = self.authenticated_peer(descriptor)
                    if operation == "observe":
                        self.peer_operation(owner, descriptor, owner_id, "observe", project / "scripts/subject.gd")
                    else:
                        admitted = scope_exchange(self, owner, descriptor, owner_id, "discover_begin", 4500)
                        observation.require(admitted["status"] == "observed", "real_discovery_owner_holds_shared_slot")
                before, disks = self.state(editor, project)
                name = "public_close_busy_" + operation + "_owner"
                result = self.public_close(project, descriptor, basis, name, "refused", reason="busy")
                self.close_evidence(name, before, disks, *self.state(editor, project), result, no_effect=True)
                if operation == "open":
                    owner.send_signal(signal.SIGTERM)
                    self.complete_open(owner, started, "overlap-open-owner-cancel", "refused")
                    self.open_action(editor, "open_release")
                elif operation == "edit":
                    owner.send_signal(signal.SIGTERM)
                    self.release_edit(editor, owner, payload, started, "overlap-edit-owner-cancel", "refused", 4)
                else:
                    if operation == "observe":
                        self.peer_operation(owner, descriptor, owner_id, "recheck", project / "scripts/subject.gd")
                    else:
                        scope_exchange(self, owner, descriptor, owner_id, "discover_finish")
                    owner.close()

    def interruption(self):
        self.compile_window_probe()
        for stage, expected in (("prepare", "refused"), ("advance", "effects_unknown"),
                                ("wait", "applied_unverified"), ("verify:post_close", "applied_unverified")):
            for mode in ("timeout", "SIGINT", "SIGTERM", "EOF", "malformed", "disable", "worker_loss"):
                name = "public_interruption_" + stage.replace(":", "_") + "_" + mode
                with self.close_fixture(name, faults=True) as (project, editor, descriptor):
                    self._close_case_editor = editor
                    basis = self.close_basis(project, descriptor, name)
                    before, disks = self.state(editor, project)
                    process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, stage)
                    worker = None
                    if mode == "worker_loss":
                        worker = observation.wait_for(lambda: self._owned_close_worker(process),
                                                      "actual_close_worker_at_gap", timeout=1)
                        os.kill(worker["pid"], signal.SIGKILL)
                    if mode.startswith("SIG"): process.send_signal(getattr(signal, mode))
                    elif mode == "EOF": self.close_action(editor, "close_disconnect")
                    elif mode == "malformed": self.close_action(editor, "close_malformed")
                    elif mode == "disable": self.close_action(editor, "close_lifetime", control="disable")
                    result = self.complete_public_close(process, started, name,
                        ("refused", "effects_unknown") if stage == "advance" and mode != "worker_loss" else expected)
                    if mode == "disable": self.close_action(editor, "close_lifetime", control="enable")
                    baseline, disk = self.state(editor, project)
                    self.close_action(editor, "close_release")
                    self.close_action(editor, "close_idle", frames=8)
                    final, now = self.state(editor, project)
                    observation.require(documents(baseline) == documents(final) and disk == now and baseline["selection"] == final["selection"],
                                        "public_close_terminal_no_late_entry_or_upgrade_" + name)
                    preentry = stage in ("prepare", "advance")
                    self.close_evidence(name, before if preentry else baseline,
                                        disks if preentry else disk, final, now, result, no_effect=preentry)
                    if worker is not None:
                        observation.require(self._owned_process_info(worker["pid"]) is None and
                                            not self._owned_children(process.pid), "close_worker_loss_owned_reap")
        for response, expected in (("close_prepared", "refused"), ("close_progress", "effects_unknown"),
                                   ("close_waited", "applied_unverified"), ("close_sample", "applied_unverified"),
                                   ("close_rechecked", "applied_unverified")):
            for mode in ("timeout", "EOF", "malformed"):
                name = "public_lost_delivery_" + response + "_" + mode
                with self.close_fixture(name) as (project, editor, descriptor):
                    self._close_case_editor = editor
                    basis = self.close_basis(project, descriptor, name)
                    before, disks = self.state(editor, project)
                    process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, "",
                                                                response_kind=response)
                    if mode == "EOF": self.close_action(editor, "close_disconnect")
                    elif mode == "malformed": self.close_action(editor, "close_malformed")
                    result = self.complete_public_close(process, started, name, expected)
                    baseline, disk = self.state(editor, project)
                    self.close_action(editor, "close_release")
                    self.close_action(editor, "close_idle", frames=8)
                    after, now = self.state(editor, project)
                    observation.require(documents(baseline) == documents(after) and disk == now,
                                        "missing_or_malformed_delivery_no_late_close")
                    self.close_evidence(name, before, disks, after, now, result, no_effect=expected == "refused")
        for fault in ("drop_completion", "wrong_completion", "premature_completion", "fail_after_return", "missing_signal",
                      "cancel_at_entry", "disable_at_entry", "expire_before_entry"):
            name = "public_native_boundary_" + fault
            with self.close_fixture(name, faults=True) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                self.close_action(editor, "close_arm", native_stage="advance", native_fault=fault)
                expected = ("refused" if fault in ("missing_signal", "expire_before_entry") else
                            "verified_newly_closed" if fault == "premature_completion" else "applied_unverified")
                result = self.public_close(project, descriptor, basis, name, expected)
                after, now = self.state(editor, project)
                self.close_evidence(name, before, disks, after, now, result, no_effect=result["outcome"] == "refused")
        for change in ("reopen", "file_source", "protected_source", "protected_identity"):
            name = "public_post_native_OK_" + change
            with self.close_fixture(name) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, "verify:post_close")
                if change == "reopen":
                    self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
                    self.close_action(editor, "close_human", path=TARGET, mutation="dirty_different")
                elif change == "file_source": self.close_action(editor, "close_file", mutation="source")
                else: self.close_action(editor, "close_human", path=CURRENT,
                                        mutation="dirty_different" if change == "protected_source" else "replace")
                survivor, disks = self.state(editor, project)
                self.close_action(editor, "close_release")
                result = self.complete_public_close(process, started, name, "applied_unverified")
                after, now = self.state(editor, project)
                self.close_evidence(name, survivor, disks, after, now, result, no_effect=True)
        with self.close_fixture("public-renewed-visit", paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "public-renewed-visit")
            process, started, event = self.held_public_close(project, editor, descriptor, basis,
                                                            "public-renewed-visit", "verify:post_close")
            self.close_action(editor, "close_renew_visit", request_id=event["request_id"])
            baseline, disks = self.state(editor, project)
            self.close_action(editor, "close_release")
            result = self.complete_public_close(process, started, "public_renewed_visit_not_old_completion", "applied_unverified")
            self.close_evidence("public_renewed_visit_not_old_completion", baseline, disks,
                                *self.state(editor, project), result, no_effect=True)
        with self.close_fixture("public-newer-during-settling", paths=[TARGET, CURRENT], faults=True) as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "public-newer-during-settling")
            process, started, _ = self.held_public_close(project, editor, descriptor, basis,
                "public-newer-during-settling", "wait", native_stage="advance", native_fault="drop_completion")
            self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
            self.close_action(editor, "close_human", path=TARGET, mutation="dirty_different")
            baseline, disks = self.state(editor, project)
            self.close_action(editor, "close_release")
            result = self.complete_public_close(process, started, "public_settling_preserves_newer_buffer", "applied_unverified")
            self.close_evidence("public_settling_preserves_newer_buffer", baseline, disks,
                                *self.state(editor, project), result, no_effect=True)
        with self.close_fixture("blocked-stdin", paths=[TARGET]) as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "blocked-stdin")
            before, disks = self.state(editor, project)
            process, started = self.start_public_close(project, basis, session=descriptor["session_id"], data=b'{', eof=False)
            # Keep the writer open without communicate closing it; stdout is drained independently.
            stdin = process.stdin
            process.stdin = None
            try:
                result = self.complete_public_close(process, started, "public_clock_before_blocked_stdin", "refused", reason="timeout")
            finally:
                stdin.close()
            self.close_evidence("public_clock_before_blocked_stdin", before, disks, *self.state(editor, project), result, no_effect=True)
        with self.close_fixture("unresponsive-editor") as (project, editor, descriptor):
            self._close_case_editor = editor
            basis = self.close_basis(project, descriptor, "unresponsive-editor")
            before, disks = self.state(editor, project)
            os.kill(editor["process"].pid, signal.SIGSTOP)
            try:
                stalled = self.public_close(project, descriptor, basis, "public_unresponsive_editor", "refused", reason="timeout")
            finally:
                os.kill(editor["process"].pid, signal.SIGCONT)
            self.close_action(editor, "close_idle", frames=8)
            after, now = self.state(editor, project)
            observation.require(documents(before) == documents(after) and disks == now, "unresponsive_editor_no_late_close")
            self.close_evidence("public_unresponsive_editor", before, disks, after, now, stalled, no_effect=True)
        self.close_entered_stalls()
        self.close_validator_failures()

    def _owned_close_worker(self, process):
        matches = []
        for pid in self._owned_children(process.pid):
            child = self._owned_process_info(pid)
            if child is not None and child["command"].endswith(" --internal-close-worker"):
                matches.append(child)
        observation.require(len(matches) <= 1, "one_actual_owned_close_worker")
        return matches[0] if matches else None

    def close_entered_stalls(self):
        for phase, fault in (("entered", "callback_entry"), ("callback", "callback_completion")):
            for mode in ("timeout", "cancel", "loss", "disable", "worker_loss"):
                name = "public_entered_" + phase + "_" + mode
                with self.close_fixture(name, faults=True) as (project, editor, descriptor):
                    self._close_case_editor = editor
                    basis = self.close_basis(project, descriptor, name)
                    before, disks = self.state(editor, project)
                    event_path = editor["control"] / "close-entered.json"
                    release_path = editor["control"] / "close-native-release.json"
                    event_path.unlink(missing_ok=True)
                    release_path.unlink(missing_ok=True)
                    self.close_action(editor, "close_arm", native_stage="advance", native_fault=fault,
                                      callback={"loss": "hold_stop", "disable": "hold_disable"}.get(mode, "hold"))
                    process, started = self.start_public_close(project, basis, session=descriptor["session_id"])
                    def entered():
                        if event_path.is_file():
                            event = json.loads(event_path.read_text())
                            return event if event.get("stage") == phase else None
                        observation.require(process.poll() is None, "actual_close_native_entered_caller_live")
                        return None
                    event = observation.wait_for(entered, "actual_close_entered_callback_" + name, timeout=7)
                    if mode == "cancel": process.send_signal(signal.SIGTERM)
                    worker = None
                    if mode == "worker_loss":
                        worker = observation.wait_for(lambda: self._owned_close_worker(process),
                                                      "real_close_worker_inside_native_call", timeout=1)
                        os.kill(worker["pid"], signal.SIGKILL)
                    try:
                        result = self.complete_public_close(process, started, name,
                                                           "effects_unknown" if phase == "entered" else "applied_unverified")
                    finally:
                        # This is a controlled lease-expiry barrier, not a
                        # continuation-completion witness or extra caller budget.
                        observation.wait_for(lambda: time.monotonic() - started >= 9.6,
                                             "original_native_lease_expired_before_release", timeout=10)
                        observation.json_file(release_path, {"request_id": event["request_id"]})
                    # Native work returns only after the caller has terminalized.
                    callback = self.close_action(editor, "close_callback_witness")["callback"]
                    after, now = self.state(editor, project)
                    observation.require(disks == now and documents(after) ==
                                        ({path: doc for path, doc in documents(before).items() if path != TARGET}
                                         if phase == "callback" else documents(before)),
                                        "actual_entered_call_no_late_old_authority_close")
                    if mode == "disable":
                        observation.require(callback["retained_slot_busy"] and callback["retained_owner_matches"] and
                                            callback["current_reference_alive"] and callback["owner_reference_alive"],
                                            "disable_retains_entered_owner_and_protected_references")
                    self.close_action(editor, "close_release")
                    self.close_action(editor, "close_idle", frames=8)
                    final, final_disk = self.state(editor, project)
                    observation.require(documents(final) == documents(after) and final_disk == now,
                                        "entered_late_return_never_upgrades_or_replays")
                    self.close_evidence(name, after, now, final, final_disk, result, no_effect=True)
                    if worker is not None:
                        observation.require(self._owned_process_info(worker["pid"]) is None,
                                            "entered_close_worker_loss_reaped")

    def close_validator_failures(self):
        for mode in ("timeout", "cancel", "worker_loss", "helper_loss", "engine_loss"):
            name = "public_close_context_" + mode
            with self.close_fixture(name, paths=[TARGET, CURRENT]) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, name)
                before, disks = self.state(editor, project)
                owned = {}
                try:
                    process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, "prepare")
                    owned.update(process=process, started=started)
                    owned["worker"] = observation.wait_for(lambda: self._owned_close_worker(process),
                                                          "actual_public_close_worker_before_helper", timeout=1)
                    self.close_action(editor, "close_release")
                    def helper_and_engine():
                        observation.require(process.poll() is None, "close_caller_live_during_real_validator")
                        for pid in self._owned_children(process.pid):
                            helper = self._owned_process_info(pid)
                            if helper is None or not helper["command"].endswith(" --internal-stock-validation-worker"):
                                continue
                            for child_pid in self._owned_children(pid):
                                engine = self._owned_process_info(child_pid)
                                if engine is not None:
                                    executable = observation.run(["/bin/ps", "-p", child_pid, "-o", "comm="], timeout=1)
                                    if Path(executable.stdout.decode().strip()).name == self.args.godot.name:
                                        os.kill(pid, signal.SIGSTOP)
                                        os.kill(child_pid, signal.SIGSTOP)
                                        owned.update(helper=helper, engine=engine)
                                        return helper, engine
                        return None
                    helper, engine = observation.wait_for(helper_and_engine, "real_close_context_helper_and_engine", timeout=4)
                    observation.require(helper["parent"] == process.pid and engine["parent"] == helper["pid"] and
                                        helper["group"] == engine["group"] == helper["pid"] and
                                        helper["group"] not in (os.getpgrp(), process.pid, editor["process"].pid),
                                        "close_context_parent_owned_sequential_child_group")
                    working = observation.run(["/usr/sbin/lsof", "-a", "-p", engine["pid"], "-d", "cwd", "-Fn"], timeout=1)
                    directories = [Path(line[1:]) for line in working.stdout.decode().splitlines() if line.startswith("n")]
                    observation.require(len(directories) == 1 and directories[0].name == "project",
                                        "close_context_actual_private_staging")
                    private = directories[0].parent
                    metadata = private.lstat()
                    observation.require(private.name.startswith("godot-validation-") and
                                        private.parent.resolve() == Path(tempfile.gettempdir()).resolve() and
                                        stat.S_ISDIR(metadata.st_mode) and metadata.st_uid == os.geteuid() and
                                        stat.S_IMODE(metadata.st_mode) == 0o700 and not private.is_symlink(),
                                        "close_context_owned_private_staging")
                    owned.update(private=private, private_identity=(metadata.st_dev, metadata.st_ino))
                    staged = observation.disk_witness(directories[0] / CURRENT.removeprefix("res://"))
                    observation.require(staged["text"] == before["current"]["R"] == before["current"]["B"],
                                        "actual_close_context_stages_only_admitted_protected_source")
                    members = self._owned_group_members(helper["group"])
                    observation.require({helper["pid"], engine["pid"]} <= {row["pid"] for row in members},
                                        "close_context_observed_owned_cleanup_group")
                    owned["group_before"] = members
                    if mode == "cancel": process.send_signal(signal.SIGTERM)
                    elif mode.endswith("_loss"):
                        victim = {"worker_loss": owned["worker"], "helper_loss": helper, "engine_loss": engine}[mode]
                        os.kill(victim["pid"], signal.SIGKILL)
                        if mode == "engine_loss": os.kill(helper["pid"], signal.SIGCONT)
                    result = self.complete_public_close(process, started, name, "refused")
                    cleanup = self._assert_validation_cleanup(owned, name)
                    after, now = self.state(editor, project)
                    self.close_evidence(name, before, disks, after, now, result, no_effect=True)
                    self.cases[-1]["owned_close_context_cleanup"] = cleanup
                finally:
                    self._cleanup_validation_control(owned)

    def privacy_export(self):
        self.compile_window_probe()
        source = SAFE + "# CLOSE_PRIVATE_TARGET\n"
        with self.close_fixture("privacy-other-project", source=SAFE + "# CLOSE_OTHER_PROJECT\n") as (other, other_editor, other_descriptor):
            self.source_markers.add(b"CLOSE_OTHER_PROJECT")
            other_before, other_disk = self.state(other_editor, other)
            with self.close_fixture("privacy-selected", source=source) as (project, editor, descriptor):
                self._close_case_editor = editor
                basis = self.close_basis(project, descriptor, "privacy")
                before, disks = self.state(editor, project)
                result = self.public_close(project, descriptor, basis, "public_privacy_authorized")
                self.close_evidence("public_privacy_authorized", before, disks, *self.state(editor, project), result)
                second = self.start_editor(project)
                try:
                    observation.wait_for(lambda: len(self.descriptors(project)) == 2, "privacy_close_real_ambiguity")
                    prior, prior_disk = self.state(editor, project)
                    ambiguous = self.public_close(project, descriptor, None, "public_privacy_ambiguous", "refused", session=None,
                                                  reason="ambiguous_session")
                    observation.require(ambiguous["before"] is None and ambiguous["observation"] is None and
                                        sha(source) not in json.dumps(ambiguous), "close_ambiguous_no_candidate_digest")
                    self.close_evidence("public_privacy_ambiguous", prior, prior_disk,
                                        *self.state(editor, project), ambiguous, no_effect=True)
                finally:
                    self.close_editor(second)
                    self.editors.remove(second)
                target = project / "scripts/subject.gd"
                prior, prior_disk = self.state(editor, project)
                target.chmod(0)
                try:
                    denied = self.public_close(project, descriptor, None, "public_privacy_denied", "refused", reason="denied_access")
                    observation.require(denied["before"] is None and denied["observation"] is None and
                                        sha(source) not in json.dumps(denied), "close_denied_no_candidate_digest")
                finally:
                    target.chmod(0o600)
                self.close_evidence("public_privacy_denied", prior, prior_disk,
                                    *self.state(editor, project), denied)
                self.public_open(project, descriptor, "privacy-intentional-reopen")
                fresh = self.close_basis(project, descriptor, "privacy-interrupted")
                process, started, _ = self.held_public_close(project, editor, descriptor, fresh, "privacy-interrupted", "verify:post_close")
                self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
                self.close_action(editor, "close_human", path=TARGET, mutation="dirty_different")
                survivor, disk = self.state(editor, project)
                process.send_signal(signal.SIGTERM)
                result = self.complete_public_close(process, started, "public_privacy_interrupted", "applied_unverified")
                self.close_action(editor, "close_release")
                self.close_evidence("public_privacy_interrupted", survivor, disk, *self.state(editor, project), result, no_effect=True)
                for case in self.cases:
                    if case.get("operation") != "close_gdscript": continue
                    encoded = (self.artifacts / case["evidence"]).read_bytes()
                    observation.require(str(other).encode() not in encoded and
                                        all(sha(doc["R"]).encode() not in encoded for doc in other_before["documents"]),
                                        "close_privacy_no_other_project_path_or_hash")
            observation.require(documents(other_before) == documents(self.state(other_editor, other)[0]) and
                                other_disk == self.state(other_editor, other)[1], "close_privacy_other_project_untouched")
        self.native_export()
