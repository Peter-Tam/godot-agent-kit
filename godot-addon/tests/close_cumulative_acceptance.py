"""Stateful public-close requests and the full composed editor workflow."""
from __future__ import annotations

import json
import signal
import subprocess
import time

import run_observation as observation
from caller_close_acceptance import TARGET, CURRENT, SAFE, _close_witness
from close_native_acceptance import documents
from run_script_edit import sha


class CumulativeCloseAcceptanceMixin:
    def cumulative_close(self, project, editor, descriptor, basis, name,
                         expected="verified_newly_closed", *, reason=None, script=TARGET):
        self._close_case_editor = editor
        before, disks = self.state(editor, project)
        result = self.public_close(project, descriptor, basis, name, expected,
                                   reason=reason, script=script)
        self.close_evidence(name, before, disks, *self.state(editor, project), result,
                            no_effect=expected in ("refused", "already_closed_unchanged"))
        return result

    def cumulative_coherent(self, project, editor, source, name):
        state, disks = self.state(editor, project)
        doc = state["target"]
        observation.require(doc["associated"] and doc["R"] == doc["B"] == disks[TARGET]["text"] == source and
                            not doc["dirty"] and not doc["resource_edited"] and
                            doc["version"] == doc["saved_version"],
                            "cumulative_independent_clean_D_R_B_" + name)
        observation.json_file(self.artifacts / (name + "-coherence.json"), _close_witness(state, disks))
        return state, disks

    def cumulative_human_history(self, editor):
        self.close_action(editor, "close_human", path=CURRENT, mutation="select")
        self.close_action(editor, "close_human", path=CURRENT, mutation="dirty_equal")
        # Wait for the ordinary editor validation event; do not copy B into R.
        self.open_action(editor, "open_setup", paths=[], path=CURRENT, idle=True)
        doc = self.close_action(editor, "close_document", path=CURRENT)["document"]
        observation.require(doc["dirty"] and doc["R"] == doc["B"] and doc["has_undo"],
                            "cumulative_real_protected_human_history")
        self.close_action(editor, "close_human", path=TARGET, mutation="select")

    def cumulative_history_witness(self, project, editor, name):
        before, disks = self.state(editor, project)
        history = self.close_history(editor, CURRENT)
        self.open_action(editor, "open_setup", paths=[], path=CURRENT, idle=True)
        after, now = self.state(editor, project)
        original, final = documents(before)[CURRENT], documents(after)[CURRENT]
        observation.require(original["B"] == final["B"] and original["script_id"] == final["script_id"] and
                            original["editor_id"] == final["editor_id"] and
                            original["buffer_id"] == final["buffer_id"] and disks == now,
                            "cumulative_unrelated_real_history_and_identity_" + name)
        evidence = name + "-history.json"
        observation.json_file(self.artifacts / evidence, {"before": _close_witness(before, disks),
                              "after": _close_witness(after, now), "reachability": history})
        next(case for case in reversed(self.cases) if case["case"] == name)["unrelated_history_evidence"] = evidence
        if after["target"]["associated"]:
            self.close_action(editor, "close_human", path=TARGET, mutation="select")

    def sequential(self):
        self.compile_window_probe()
        start = len(self.cases)
        with self.close_fixture("cumulative-sequence", paths=[TARGET, CURRENT]) as (project, editor, descriptor):
            self._close_case_editor = editor
            self.cumulative_human_history(editor)
            revisions = []
            for index in range(5):
                prefix = f"close_sequence_{index + 1:02d}"
                basis = self.close_basis(project, descriptor, prefix + "_before_human")
                mutation = "dirty_disk_equal" if index % 2 == 0 else "dirty_equal"
                self.close_action(editor, "close_human", path=TARGET, mutation=mutation)
                human, human_disk = self.state(editor, project)
                observation.require(human["target"]["dirty"] and human["target"]["has_undo"] and
                                    human["target"]["version"] != human["target"]["saved_version"] and
                                    ((human["target"]["B"] == human_disk[TARGET]["text"]) == (index % 2 == 0)),
                                    "sequence_actual_different_or_equal_text_dirty_" + prefix)
                dirty_name = prefix + "_dirty_refused"
                self.cumulative_close(project, editor, descriptor, basis, dirty_name, "refused", reason="dirty_conflict")
                self.cumulative_history_witness(project, editor, dirty_name)
                saved = self.native_action(editor, "native_edit_save")
                source = human["target"]["B"]
                observation.require(saved["focused"] and saved["disk_matches"], "sequence_ordinary_human_Save")
                self.cumulative_coherent(project, editor, source, prefix + "_saved")
                revisions.append(sha(source))
                # Old source equality does not confer authority on a newer version.
                stale_name = prefix + "_stale_refused"
                self.cumulative_close(project, editor, descriptor, basis, stale_name, "refused")
                self.cumulative_history_witness(project, editor, stale_name)
                fresh = self.close_basis(project, descriptor, prefix + "_fresh_close")
                prior, _ = self.state(editor, project)
                close_name = prefix + "_newly_closed"
                self.cumulative_close(project, editor, descriptor, fresh, close_name)
                self.cumulative_history_witness(project, editor, close_name)
                recognition_name = prefix + "_already_closed"
                self.cumulative_close(project, editor, descriptor, None, recognition_name, "already_closed_unchanged")
                self.cumulative_history_witness(project, editor, recognition_name)
                self.public_open(project, descriptor, prefix + "_separate_reopen")
                reopened, _ = self.cumulative_coherent(project, editor, source, prefix + "_reopened")
                observation.require((reopened["target"]["editor_id"], reopened["target"]["buffer_id"]) !=
                                    (prior["target"]["editor_id"], prior["target"]["buffer_id"]),
                                    "sequence_new_buffer_same_or_changed_persisted_source")
                missing_name = prefix + "_unsafe_null_basis"
                self.cumulative_close(project, editor, descriptor, None, missing_name, "refused", reason="missing_basis")
                self.cumulative_history_witness(project, editor, missing_name)
            # Invalidate a held pre-entry request with an actual same-text human
            # close/reopen. Release only after its immutable terminal result.
            basis = self.close_basis(project, descriptor, "sequence_late_basis")
            original, original_disk = self.state(editor, project)
            name = "close_sequence_cancel_newer_buffer"
            process, started, _ = self.held_public_close(project, editor, descriptor, basis, name, "prepare")
            self.close_action(editor, "close_human", path=TARGET, mutation="reopen")
            newer, disks = self.state(editor, project)
            observation.require(newer["target"]["associated"] and disks == original_disk and
                                newer["target"]["B"] == original["target"]["B"] and
                                (newer["target"]["editor_id"], newer["target"]["buffer_id"]) !=
                                (original["target"]["editor_id"], original["target"]["buffer_id"]),
                                "sequence_actual_same_text_newer_buffer_at_barrier")
            process.send_signal(signal.SIGTERM)
            result = self.complete_public_close(process, started, name, "refused")
            self.close_action(editor, "close_release")
            self.close_action(editor, "close_idle", frames=8)
            self.close_evidence(name, newer, disks, *self.state(editor, project), result, no_effect=True)
            self.cumulative_history_witness(project, editor, name)
            basis = self.close_basis(project, descriptor, "sequence_unsafe_context_basis")
            self.close_action(editor, "close_human", path=CURRENT, mutation="resource_source")
            self.cumulative_close(project, editor, descriptor, basis, "close_sequence_unsafe_R_B_refused", "refused")
            self.cumulative_history_witness(project, editor, "close_sequence_unsafe_R_B_refused")
        requests = [case for case in self.cases[start:] if case.get("operation") == "close_gdscript"]
        counts = {outcome: sum(case["outcome"] == outcome for case in requests)
                  for outcome in ("verified_newly_closed", "already_closed_unchanged", "refused")}
        dirty = [case for case in requests if case["reason"] == "dirty_conflict"]
        observation.require(len(requests) >= 20 and counts["verified_newly_closed"] >= 5 and
                            counts["already_closed_unchanged"] >= 5 and len(dirty) >= 5,
                            "real_public_close_sequence_minima")
        self.summary["close_sequence"] = {"requests": len(requests), "outcomes": counts,
                                          "dirty_refusals": len(dirty), "persisted_revision_sha256": revisions}

    def composed_locator(self, project, descriptor, name):
        started = time.monotonic()
        discovery = subprocess.run([str(self.args.discoverer), "--registry", str(self.registry),
                                    "--project", str(project), "--session", descriptor["session_id"]],
                                   capture_output=True, timeout=5.2)
        found = json.loads(discovery.stdout)
        entries = found.get("inventory", {}).get("entries", [])
        observation.require(time.monotonic() - started <= 5 and discovery.returncode == 0 and
                            found["outcome"] == "complete_listing" and entries.count(TARGET) == 1 and
                            all(secret not in discovery.stdout + discovery.stderr for secret in self.secrets),
                            "composed_public_discovery_supplies_locator_" + name)
        observation.json_file(self.artifacts / (name + "-discovery.json"), found)
        self.safe_log(name + "-discovery.stderr", discovery.stderr)
        return next(path for path in entries if path == TARGET)

    def composed(self):
        self.compile_window_probe()
        def discoverable(project):
            (project / "scripts/.gdignore").unlink()
        for profile in ("clean", "dirty_different", "dirty_equal", "history"):
            prefix = "close_composed_" + profile
            with self.close_fixture(prefix, paths=[CURRENT], selected=CURRENT,
                                    setup=discoverable) as (project, editor, descriptor):
                self._close_case_editor = editor
                locator = self.composed_locator(project, descriptor, prefix)
                self.public_open(project, descriptor, prefix + "_product_open", script=locator)
                self.cumulative_human_history(editor)
                basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                     script=locator, name=prefix + "_observe")
                value = 83 if profile == "clean" else 97
                desired = self.revision(value)
                self.edit(project, basis, desired, prefix + "_product_edit", "verified_changed", 0,
                          session=descriptor["session_id"], script=locator)
                self.cumulative_coherent(project, editor, desired, prefix + "_A")
                fresh = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                     script=locator, name=prefix + "_fresh_after_edit")
                if profile.startswith("dirty_"):
                    self.close_action(editor, "close_human", path=locator,
                                      mutation="dirty_disk_equal" if profile == "dirty_equal" else "dirty_equal")
                    human, disks = self.state(editor, project)
                    observation.require(human["target"]["dirty"] and human["target"]["has_undo"] and
                                        ((human["target"]["B"] == desired) == (profile == "dirty_equal")),
                                        "composed_B_actual_dirty_work")
                    self.edit(project, fresh, self.revision(109), prefix + "_B_edit_refused", "refused", 3,
                              session=descriptor["session_id"], application="not_applied", script=locator)
                    after, now = self.state(editor, project)
                    observation.require(documents(human) == documents(after) and disks == now,
                                        "composed_B_edit_keeps_work_and_history")
                    name = prefix + "_B_close_refused"
                    self.cumulative_close(project, editor, descriptor, fresh, name, "refused", reason="dirty_conflict", script=locator)
                    self.cumulative_history_witness(project, editor, name)
                    desired = human["target"]["B"]
                    saved = self.native_action(editor, "native_edit_save")
                    observation.require(saved["focused"] and saved["disk_matches"], "composed_B_deliberate_human_Save")
                    self.cumulative_coherent(project, editor, desired, prefix + "_human_saved")
                elif profile == "history":
                    for operation, source in (("undo", SAFE), ("redo", desired)):
                        doc = self.close_action(editor, "close_human", path=locator, mutation=operation)["state"]["target"]
                        observation.require(doc["B"] == source and doc["dirty"] and
                                            doc["version"] != doc["saved_version"], "composed_C_real_" + operation)
                        saved = self.native_action(editor, "native_edit_save")
                        observation.require(saved["focused"] and saved["disk_matches"], "composed_C_ordinary_Save_" + operation)
                        self.cumulative_coherent(project, editor, source, prefix + "_" + operation + "_saved")
                fresh = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                     script=locator, name=prefix + "_fresh_close_basis")
                prior, prior_disk = self.state(editor, project)
                name = prefix + "_product_close"
                self.cumulative_close(project, editor, descriptor, fresh, name, script=locator)
                self.cumulative_history_witness(project, editor, name)
                self.edit(project, fresh, desired, prefix + "_closed_edit_refuses", "refused", 3,
                          session=descriptor["session_id"], reason="closed_target", application="not_applied", script=locator)
                self.public_open(project, descriptor, prefix + "_separate_product_reopen", script=locator)
                reopened, disk = self.cumulative_coherent(project, editor, desired, prefix + "_D")
                observation.require(disk[TARGET] == prior_disk[TARGET] and
                                    (reopened["target"]["editor_id"], reopened["target"]["buffer_id"]) !=
                                    (prior["target"]["editor_id"], prior["target"]["buffer_id"]),
                                    "composed_D_exact_file_new_buffer_persisted_source")
                self.native_action(editor, "native_edit_save_clean")
                self.native_action(editor, "native_edit_reparse")
                self.native_action(editor, "native_edit_scan")
                self.cumulative_coherent(project, editor, desired, prefix + "_ordinary_durability")
                self.native_runtime(project, value)
        # The established twenty fresh-basis edit stress includes real Save,
        # conflicts and history transitions. It is nested evidence, not a tenth
        # close group, and runs only once when all dispatched clean-close first.
        if not self.summary.get("existing_twenty_edit_stress_executed"):
            self.state = self.snapshot
            try:
                self.sequential_edit()
            finally:
                del self.state
            self.summary["existing_twenty_edit_stress_executed"] = True
        self.summary["composed_matrix"] = {"profiles": ["clean", "dirty_different", "dirty_equal", "history"],
                                           "twenty_edit_stress": "existing_twenty_edit_stress_executed",
                                           "close_minima": "separate_sequential_group"}
