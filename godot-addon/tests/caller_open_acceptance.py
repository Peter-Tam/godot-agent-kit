"""Actual open-gdscript campaigns with independent owned-editor witnesses."""
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import stat
import subprocess
import time
import tempfile

import run_observation as observation
from opening_fixture_witness import (BACKGROUND, CURRENT, CURRENT_SOURCE, FIXTURE,
                                     INVALID_SOURCE, TARGET, TARGET_SOURCE, source_free_document)
from open_result_review import OPEN_CODES, review_open_result
from run_script_edit import sha


class CallerOpenAcceptanceMixin:
    def open_command(self, project, session=None, script=TARGET):
        command = [str(self.args.opener), "--registry", str(self.registry), "--project",
                   str(project), "--script", script]
        if session is not None:
            command.extend(("--session", session))
        return command

    def start_open(self, project, *, session=None, script=TARGET):
        started = time.monotonic()
        process = subprocess.Popen(self.open_command(project, session, script),
                                   stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE)
        self.open_processes.append(process)
        return process, started

    def complete_open(self, process, started, name, expected, *, reason=None):
        try:
            stdout, stderr = process.communicate(timeout=max(0.01, 10.2 - (time.monotonic() - started)))
        except subprocess.TimeoutExpired as error:
            process.kill()
            process.communicate(timeout=2)
            raise observation.Failure("public_open_deadline_exceeded_" + name) from error
        finally:
            if process.poll() is not None and process in self.open_processes:
                self.open_processes.remove(process)
        elapsed = time.monotonic() - started
        observation.require(elapsed <= 10 and stdout.endswith(b"\n") and stdout.count(b"\n") == 1 and
                            len(stdout) <= 12 * 1024 * 1024,
                            "public_open_ten_second_single_bounded_result_" + name)
        result = json.loads(stdout)
        interpretation = review_open_result(result)
        allowed = (expected,) if isinstance(expected, str) else expected
        expected_code = (2 if result["outcome"] == "refused" and result["reason"] == "invalid_request"
                         else OPEN_CODES[result["outcome"]])
        matches_expected = (result["outcome"] in allowed and
                            process.returncode == expected_code and
                            result["interval"]["elapsed_us"] <= 10_000_000 and
                            (reason is None or result["reason"] in
                             ((reason,) if isinstance(reason, str) else reason)))
        if not matches_expected:
            self.summary["failed_public_result"] = {
                "case": name, "outcome": result["outcome"], "reason": result["reason"],
                "stage": result["stage"], "application": result["application"],
                "exit_code": process.returncode, "elapsed_seconds": elapsed}
        observation.require(matches_expected, "public_open_expected_outcome_" + name)
        observation.require(all(secret not in stdout for secret in self.secrets),
                            "public_open_no_authentication_material_" + name)
        observation.require(all(path.encode() not in stderr for path in (CURRENT, BACKGROUND)),
                            "public_open_stderr_no_private_context_path_" + name)
        self.safe_log(name + ".stderr", stderr)
        observation.json_file(self.artifacts / (name + ".json"), result)
        source_free = {key: value for key, value in result.items() if key not in
                       ("before", "observation", "requested_target", "resolved_target")}
        observation.require(all(marker not in json.dumps(source_free).encode()
                                for marker in self.source_markers),
                            "public_open_source_free_diagnostics_and_private_protection_" + name)
        observation.require(all(path not in json.dumps(source_free) for path in (CURRENT, BACKGROUND)),
                            "public_open_no_private_context_path_projection_" + name)
        observation.require(all(case.get("request_id") != result["request_id"] for case in self.cases),
                            "public_open_fresh_request_id_" + name)
        self.case(name, operation="open_gdscript", request_id=result["request_id"],
                  outcome=result["outcome"], reason=result["reason"], application=result["application"],
                  elapsed_seconds=elapsed, result_only_review=interpretation, evidence=name + ".json",
                  source_surfaces="actual_public_opener_before_independent_witness_comparison")
        return result

    def public_open(self, project, descriptor, name, expected="verified_newly_opened", *,
                    session=True, script=TARGET, reason=None):
        process, started = self.start_open(
            project, session=descriptor["session_id"] if session is True else session, script=script)
        return self.complete_open(process, started, name, expected, reason=reason)

    def opening_evidence(self, name, before, disks, after, now, *, capture=None):
        evidence = {"before": self.concise_open_state(before, disks),
                    "after": self.concise_open_state(after, now)}
        observation.json_file(self.artifacts / (name + "-witness.json"), evidence)
        case = next(value for value in reversed(self.cases) if value["case"] == name)
        case.update(witness=name + "-witness.json", screenshot=capture)

    def assert_public_before(self, result, state, disks):
        summary = result["before"]
        document = state["target"]
        observation.require(summary is not None and set(summary) ==
                            {"mode", "document", "sources", "dirty", "resource_edited"} and
                            summary["mode"] == ("open" if document["associated"] else
                                                "closed_cached" if state["cached_id"] else "closed_uncached"),
                            "public_before_independent_open_and_cache_mode")
        actual = {"D": disks["target"]["text"],
                  "R": document.get("R") if document["associated"] else state["cached_R"],
                  "B": document.get("B") if document["associated"] else None}
        for authority, surface in summary["sources"].items():
            observation.require("text" not in surface and surface["authority"] == authority,
                                "public_before_summary_never_duplicates_source")
            if surface["availability"] == "observed":
                source = actual[authority]
                observation.require(source is not None and surface["digest"] ==
                                    {"sha256": sha(source), "utf8_bytes": len(source.encode())} and
                                    surface["collection"] is not None and surface["witness"] is not None,
                                    "public_before_independent_actual_source_hash_length_" + authority)
            else:
                observation.require(surface["digest"] is None, "public_before_no_invented_source_" + authority)
        if document["associated"]:
            identity = summary["document"]["identity"]
            observation.require(identity["script_instance_id"] == document["script_id"] and
                                identity["editor_instance_id"] == document["editor_id"] and
                                identity["buffer_instance_id"] == document["buffer_id"] and
                                summary["dirty"]["state"] == ("dirty" if document["dirty"] else "clean"),
                                "public_before_independent_identity_and_dirty_not_source_equality")
        edited = summary["resource_edited"]
        if edited["availability"] == "observed":
            observation.require(edited["value"] == (document["resource_edited"] if document["associated"]
                                                    else state["cached_edited"]),
                                "public_before_resource_edited_separate_from_buffer_dirty")

    def assert_public_sources(self, result, state, disks, descriptor, *, key="target", path=TARGET):
        snapshot = result["observation"]["snapshot"]
        identity = snapshot["document"]["identity"]
        document = state[key]
        observation.require(result["resolved_target"]["script_path"] == path and
                            result["resolved_target"]["session_id"] == descriptor["session_id"] and
                            identity["resource_path"] == path and
                            identity["script_instance_id"] == document["script_id"] and
                            identity["editor_instance_id"] == document["editor_id"] and
                            identity["buffer_instance_id"] == document["buffer_id"] and
                            snapshot["dirty"]["state"] == ("dirty" if document["dirty"] else "clean"),
                            "public_result_independent_actual_document_identity")
        for authority, text in (("D", disks[key]["text"]), ("R", document["R"]), ("B", document["B"])):
            source = snapshot["sources"][authority]
            if source["availability"] == "observed":
                observation.require(source["text"] == text,
                                    "public_result_independent_source_" + authority)

    def assert_public_selection(self, result, before, after, *, effect):
        def relation(state):
            selected = state["selection"]
            return "target" if selected == TARGET else "other" if selected else "no_source_editor"
        observation.require(result["selection"] == {
            "before": relation(before), "after": relation(after), "request_effect": effect},
            "public_selection_before_after_and_request_effect_match_independent_active_tab")

    def assert_no_effect(self, editor, project, before, disks, name):
        after, now = self.open_state(editor, project)
        self.assert_preserved(before, after, disks, now, name, selection=True)
        observation.require(before["target"] == after["target"] and
                            before["cached_id"] == after["cached_id"] and
                            before["cached_R"] == after["cached_R"] and
                            before["cached_edited"] == after["cached_edited"] and
                            before["open_paths"] == after["open_paths"] and not after["slot_busy"],
                            "public_refusal_or_recognition_no_target_lifecycle_effect_" + name)
        self.opening_evidence(name, before, disks, after, now)
        return after, now

    def public_positive(self, name, *, source=TARGET_SOURCE, cached=False, read_only=False,
                        no_current=False, compose=False):
        with self.opening_fixture(name, source=source, cached=cached, read_only=read_only,
                                  no_current=no_current) as (project, editor, descriptor):
            before, disks = self.open_state(editor, project)
            observation.require(not before["target"]["associated"] and
                                bool(before["cached_id"]) == cached, "genuinely_closed_cold_or_cached_" + name)
            result = self.public_open(project, descriptor, name)
            after, now = self.open_state(editor, project)
            document = after["target"]
            observation.require(document["associated"] and document["R"] == document["B"] ==
                                now["target"]["text"] == source and not document["dirty"] and
                                document["resource_edited"] is False and
                                after["open_paths"].count(TARGET) == 1 and not after["slot_busy"],
                                "actual_public_closed_to_open_clean_d_r_b_" + name)
            observation.require(after["selection"] == TARGET,
                                "new_open_actual_native_navigation_selects_exact_target_" + name)
            self.assert_preserved(before, after, disks, now, name)
            self.assert_public_sources(result, after, now, descriptor)
            self.assert_public_before(result, before, disks)
            self.assert_public_selection(result, before, after, effect="selected_target")
            observation.require(result["progress"]["resource_binding"]["mode"] ==
                                ("reused" if cached else "created") and
                                (not cached or document["script_id"] == before["cached_id"]) and
                                result["target_parse"]["state"] ==
                                ("not_collected" if cached else "invalid" if source == INVALID_SOURCE else "valid"),
                                "public_cold_cached_parse_evidence_distinct_" + name)
            self.opening_evidence(name, before, disks, after, now,
                                  capture=self.screenshot(editor, name + ".png"))
            if compose:
                self.focused_open_edit(editor, project, descriptor, source)

    def new_open(self):
        self.compile_window_probe()
        for name, options in (
                ("public_cold_fresh_observe_edit", {"compose": True}),
                ("public_cached_exact_resource", {"cached": True}),
                ("public_empty_source", {"source": ""}),
                ("public_read_only_source", {"read_only": True}),
                ("public_syntax_invalid_target", {"source": INVALID_SOURCE}),
                ("public_no_source_current", {"no_current": True}),
                ("public_exact_source_limit", {"source": "# " + "x" * (524288 - 3) + "\n"})):
            self.public_positive(name, **options)

    def focused_open_edit(self, editor, project, descriptor, original):
        # No fixture prepare/open action is used for the target transition.
        self.open_action(editor, "open_setup", paths=[CURRENT], mutation="dirty_equal", idle=True)
        seeded = self.native_action(editor, "native_edit_human", mode="seed_prior")["document"]
        prior = seeded["B"]
        self.native_action(editor, "native_edit_save")
        basis = self.edit_basis(project, descriptor, "public_open_separate_fresh_observation")
        desired = self.revision(53)
        changed = self.edit(project, basis, desired, "public_open_existing_eligible_edit",
                            "verified_changed", 0, session=descriptor["session_id"], application="applied")
        self.validated_source(changed, desired, "post_change", "valid")
        self.coherent_revision(editor, project, desired, "public_open_edited")
        for operation, source in (("undo", prior), ("redo", desired)):
            document = self.native_action(editor, "native_edit_human", mode=operation)["document"]
            observation.require(document["B"] == source and document["dirty"],
                                "public_open_edit_actual_history_" + operation)
            saved = self.native_action(editor, "native_edit_save")
            witness, disks = self.coherent_revision(editor, project, source, "public_open_" + operation)
            observation.require(saved["focused"] and saved["disk_matches"] and
                                saved["before_disk_changed"],
                                "public_open_edit_actual_history_ordinary_save_" + operation)
            self.case("public_open_edit_" + operation + "_save",
                      state=self.source_free_state(witness, disks),
                      source_surfaces="actual_native_history_and_delivered_save_d_r_b")
        self.native_action(editor, "native_edit_human", mode="undo")
        older = self.native_action(editor, "native_edit_human", mode="undo")["document"]
        observation.require(older["B"] == original and older["has_redo"],
                            "public_open_edit_prior_human_native_history_reachable")
        self.native_action(editor, "native_edit_human", mode="redo")
        self.native_action(editor, "native_edit_human", mode="redo")
        self.native_action(editor, "native_idle")
        self.ordinary_clean_save(editor, project, desired, "public_open_edit_durability")
        self.durable_reopen(editor, project, desired, "public_open_edit_durability")
        parsed = self.native_action(editor, "native_edit_reparse")
        scan = self.native_action(editor, "native_edit_scan")
        observation.require(parsed["parse_error"] == 0 and scan["settled"] and
                            scan["events"] > scan["before_events"], "public_open_edit_reparse_rescan")
        self.native_runtime(project, 53)
        witness, disks = self.coherent_revision(editor, project, desired, "public_open_runtime_survivor")
        clean_basis = self.edit_basis(project, descriptor, "public_open_edit_before_dirty")
        self.native_action(editor, "native_edit_human", mode="invalid")
        dirty, dirty_disks = self.open_state(editor, project)
        self.edit(project, clean_basis, self.revision(71), "public_open_existing_dirty_edit_refusal",
                  "refused", 3, session=descriptor["session_id"], application="not_applied")
        after, now = self.open_state(editor, project)
        observation.require(after == dirty and now == dirty_disks,
                            "public_open_edit_preserves_newer_dirty_human_work")
        self.case("public_open_edit_focused_history_durability_runtime",
                  state=self.source_free_state(witness, disks), prior_history_reached=True,
                  screenshot=self.screenshot(editor, "public-open-edit-dirty-preserved.png"),
                  source_surfaces="open_fresh_observe_edit_undo_save_redo_save_reopen_reparse_rescan_runtime")


    def already_open(self):
        self.compile_window_probe()
        for mode in ("clean_selected", "clean_nonselected", "dirty_different", "dirty_disk_equal",
                     "dirty_equal", "resource_divergent", "resource_limited", "buffer_limited",
                     "disk_limited", "unsafe_current", "unsupported_target_profile"):
            name = "public_recognition_" + mode
            with self.opening_fixture(name, source=(
                    "@tool\nextends RefCounted\n" if mode == "unsupported_target_profile" else TARGET_SOURCE)) as (
                    project, editor, descriptor):
                self.open_action(editor, "open_setup", paths=[TARGET])
                # Navigate before constructing divergent/limited target state:
                # native tab departure may apply B into R even for invalid B.
                if mode != "clean_selected":
                    self.open_action(editor, "open_setup", paths=[CURRENT])
                if mode in ("dirty_different", "dirty_disk_equal", "dirty_equal"):
                    self.open_action(editor, "open_setup", mutation=mode, path=TARGET,
                                     idle=mode != "dirty_different")
                elif mode == "resource_divergent":
                    self.open_action(editor, "open_mutate", mutation="dirty_different", path=TARGET)
                    self.action(editor, "resource_subject")
                elif mode in ("resource_limited", "buffer_limited"):
                    self.action(editor, "cap_" + mode.split("_")[0] + "_over")
                elif mode == "disk_limited":
                    (project / "scripts/subject.gd").write_text("# " + "d" * 524288)
                if mode == "unsafe_current":
                    self.open_action(editor, "open_mutate", mutation="dirty_different")
                before, disks = self.open_state(editor, project)
                if mode == "dirty_disk_equal":
                    observation.require(before["target"]["dirty"] and before["target"]["B"] ==
                                        disks["target"]["text"], "actual_equal_but_dirty_not_clean")
                if mode == "resource_divergent":
                    observation.require(before["target"]["R"] != before["target"]["B"],
                                        "actual_resource_divergence_before_recognition")
                if mode.endswith("_limited"):
                    authority = {"resource_limited": "R", "buffer_limited": "B", "disk_limited": "D"}[mode]
                    source = disks["target"]["text"] if authority == "D" else before["target"][authority]
                    observation.require(len(source.encode("utf-8")) > 524288,
                                        "actual_independent_source_limit_prepared_" + authority)
                result = self.public_open(project, descriptor, name, "already_open_unchanged")
                after, now = self.assert_no_effect(editor, project, before, disks, name)
                self.assert_public_sources(result, after, now, descriptor)
                self.assert_public_before(result, before, disks)
                self.assert_public_selection(result, before, after, effect="none")
                if mode.endswith("_limited"):
                    authority = {"resource_limited": "R", "buffer_limited": "B", "disk_limited": "D"}[mode]
                    observation.require(result["observation"]["snapshot"]["sources"][authority]["availability"]
                                        != "observed", "recognition_retains_independent_source_limit_" + authority)
                if before["target"]["has_undo"]:
                    old = before["target"]["B"]
                    undo = self.open_action(editor, "open_mutate", mutation="undo", path=TARGET)["state"]
                    redo = self.open_action(editor, "open_mutate", mutation="redo", path=TARGET)["state"]
                    observation.require(undo["target"]["B"] != old and redo["target"]["B"] == old,
                                        "recognition_actual_native_history_still_usable_" + mode)
                case = next(value for value in self.cases if value["case"] == name)
                if before["target"]["has_undo"]:
                    case["native_history_after_recognition"] = {
                        "undo": source_free_document(undo["target"]),
                        "redo": source_free_document(redo["target"])}
                case["screenshot"] = self.screenshot(editor, name + ".png")

    def held_open(self, project, editor, descriptor, stage, name, *, response_kind="",
                  native_stage="", native_fault="", callback=""):
        for filename in ("open-event.json", "open-entered.json"):
            (editor["control"] / filename).unlink(missing_ok=True)
        self.open_action(editor, "open_arm", stage=stage, response_kind=response_kind,
                         native_stage=native_stage, native_fault=native_fault, callback=callback)
        process, started = self.start_open(project, session=descriptor["session_id"])
        event_file = editor["control"] / ("open-entered.json" if native_fault else "open-event.json")
        expected = native_stage if native_fault else "response:" + response_kind if response_kind else stage
        def reached():
            if event_file.is_file():
                event = json.loads(event_file.read_text())
                if event.get("stage") == expected:
                    return event
            # Entered loss/disable may finish the caller before this poll; its
            # already-written native entry event remains the actual witness.
            observation.require(process.poll() is None and editor["process"].poll() is None,
                                "real_public_open_barrier_process_live_" + name)
            return None
        event = observation.wait_for(reached, "actual_public_open_event_" + name, timeout=7)
        observation.require(len(event["request_id"]) == 32, "public_attempt_barrier_real_request_id")
        return process, started, event

    def finish_held_open(self, editor, process, started, name, expected, *, reason=None):
        self.open_action(editor, "open_release")
        return self.complete_open(process, started, name, expected, reason=reason)

    def preservation(self):
        self.compile_window_probe()
        name = "public_open_dirty_current_equal_and_background_history"
        with self.opening_fixture(name) as (project, editor, descriptor):
            for path in (BACKGROUND, CURRENT):
                self.open_action(editor, "open_setup", paths=[path], mutation="dirty_equal", path=path, idle=True)
            before, disks = self.open_state(editor, project)
            observation.require(all(before[key]["dirty"] and before[key]["has_undo"] and
                                    before[key]["R"] == before[key]["B"] != disks[key]["text"]
                                    for key in ("current", "background")),
                                "actual_dirty_equal_current_and_background_prior_history")
            result = self.public_open(project, descriptor, name)
            after, now = self.open_state(editor, project)
            self.assert_preserved(before, after, disks, now, name)
            self.assert_public_sources(result, after, now, descriptor)
            self.assert_public_before(result, before, disks)
            self.assert_public_selection(result, before, after, effect="selected_target")
            history = []
            for key, path in (("current", CURRENT), ("background", BACKGROUND)):
                for operation, expected in (("undo", disks[key]["text"]), ("redo", before[key]["B"])):
                    transitioned = self.open_action(editor, "open_setup", mutation=operation,
                                                    path=path, idle=True)["state"]
                    observation.require(transitioned[key]["R"] == transitioned[key]["B"] == expected and
                                        transitioned["target"] == after["target"],
                                        "public_open_preserved_actual_prior_" + key + "_" + operation)
                    history.append({"document": key, "operation": operation,
                                    "sha256": sha(transitioned[key]["B"])})
            self.opening_evidence(name, before, disks, after, now,
                                  capture=self.screenshot(editor, name + ".png"))
            self.cases[-1]["native_prior_history"] = history
        for mode, current in (("dirty_different", CURRENT_SOURCE), ("stale_metadata",
                             '@tool\nextends RefCounted\n@export var payload: Resource\n'),
                              ("invalid_current", INVALID_SOURCE)):
            name = "public_current_refusal_" + mode
            with self.opening_fixture(name, current=current) as (project, editor, descriptor):
                if mode != "invalid_current":
                    self.open_action(editor, "open_mutate", mutation=mode)
                before, disks = self.open_state(editor, project)
                result = self.public_open(project, descriptor, name, "refused", reason=(
                    ("dirty_conflict", "unsafe_editor_context") if mode == "dirty_different" else
                    ("unsafe_editor_context", "context_changed") if mode == "stale_metadata" else
                    ("unsafe_editor_context", "current_parse_invalid")))
                observation.require(result["target_parse"]["state"] not in ("valid", "invalid"),
                                    "private_current_validation_failure_not_target_parse_evidence")
                self.assert_no_effect(editor, project, before, disks, name)
        name = "public_pending_export_drag_refusal"
        current = "extends Node\n@export var payload: Resource\nfunc value() -> int:\n\treturn 7\n"
        with self.opening_fixture(name, current=current) as (project, editor, descriptor):
            drag = self.open_action(editor, "open_pending_drag", undo=True)["drag"]
            observation.require(drag["ok"], "real_editor_drag_history_not_synthetic_pending_flag")
            before, disks = self.open_state(editor, project)
            self.public_open(project, descriptor, name, "refused", reason=(
                "unsafe_editor_context", "dirty_conflict", "context_changed"))
            self.assert_no_effect(editor, project, before, disks, name)
        for mode, stage in (("target_human_open", "bind"), ("target_reopen", "recheck:recognition"),
                            ("dirty_different", "verify:post_open"),
                            ("target_reopen", "recheck:post_open")):
            name = "public_boundary_human_" + mode + "_" + stage.replace(":", "_")
            with self.opening_fixture(name) as (project, editor, descriptor):
                if stage == "recheck:recognition":
                    self.open_action(editor, "open_setup", paths=[TARGET, CURRENT])
                process, started, _ = self.held_open(project, editor, descriptor, stage, name)
                self.open_action(editor, "open_mutate", mutation=mode, path=TARGET)
                survivor, disks = self.open_state(editor, project)
                expected = "refused" if stage == "recheck:recognition" else (
                    ("refused", "already_open_unchanged") if stage == "bind" else "applied_unverified")
                self.finish_held_open(editor, process, started, name, expected)
                after, now = self.open_state(editor, project)
                observation.require(survivor["target"] == after["target"] and
                                    survivor["open_paths"] == after["open_paths"] and disks == now and
                                    survivor["selection"] == after["selection"],
                                    "public_boundary_race_preserves_exact_newer_human_document")
                self.opening_evidence(name, survivor, disks, after, now,
                                      capture=self.screenshot(editor, name + ".png"))
        for path, key in ((CURRENT, "current"), (BACKGROUND, "background")):
            for stage in ("verify:post_open", "recheck:post_open"):
                name = "public_post_open_human_selection_" + key + "_" + stage.replace(":", "_")
                with self.opening_fixture(name) as (project, editor, descriptor):
                    before, disks = self.open_state(editor, project)
                    process, started, event = self.held_open(project, editor, descriptor, stage, name)
                    opened, opened_disk = self.open_state(editor, project)
                    observation.require(opened["target"]["associated"] and opened["selection"] == TARGET,
                                        "selection_race_starts_after_actual_target_open_and_selection")
                    selected = self.open_action(editor, "open_select", path=path)["state"]
                    observation.require(selected["selection"] == path and
                                        selected["target"] == opened["target"] and
                                        selected["open_paths"] == opened["open_paths"],
                                        "human_navigates_existing_document_without_target_reopen_or_typing")
                    result = self.finish_held_open(editor, process, started, name, "verified_newly_opened")
                    after, now = self.open_state(editor, project)
                    self.assert_preserved(before, after, disks, now, name)
                    self.assert_public_before(result, before, disks)
                    self.assert_public_sources(result, after, now, descriptor)
                    self.assert_public_selection(result, before, after, effect="selected_target")
                    observation.require(after["selection"] == path and
                                        after["target"] == selected["target"] and now == opened_disk and
                                        after["open_paths"] == selected["open_paths"] and not after["slot_busy"],
                                        "verification_observes_human_selection_without_restoring_target")
                    self.open_action(editor, "open_idle")
                    late, late_disk = self.open_state(editor, project)
                    observation.require(late == after and late_disk == now,
                                        "post_open_human_selection_survives_terminal_release_and_idle")
                    self.opening_evidence(name, before, disks, after, now,
                                          capture=self.screenshot(editor, name + ".png"))
                    self.cases[-1].update(
                        boundary_event=event, human_selected_document=key,
                        after_human_navigation=self.concise_open_state(selected, opened_disk))

    def wait_restored_documents(self, editor, paths):
        # A fixture plugin can answer before ScriptEditor restores its saved
        # layout. Observe that real startup transition before freezing witnesses.
        def restored():
            state = self.open_action(editor, "open_witness")["state"]
            return state if set(state["open_paths"]) == set(paths) else None
        state = observation.wait_for(restored, "owned_editor_saved_documents_restored")
        self.open_action(editor, "open_idle")
        return state

    def routing(self):
        self.compile_window_probe()
        with self.opening_fixture("routing-exact") as (project, editor, descriptor):
            sibling = project / "scripts/same/subject.gd"
            sibling.parent.mkdir()
            sibling.write_text("extends RefCounted\n# OPEN_SAME_BASENAME_PRIVATE\n")
            sibling_before = observation.disk_witness(sibling)
            before, disks = self.open_state(editor, project)
            result = self.public_open(project, descriptor, "public_exact_path_same_basename")
            after, now = self.open_state(editor, project)
            self.assert_public_sources(result, after, now, descriptor)
            self.assert_public_before(result, before, disks)
            self.assert_public_selection(result, before, after, effect="selected_target")
            observation.require("res://scripts/same/subject.gd" not in after["open_paths"] and
                                observation.disk_witness(sibling) == sibling_before,
                                "public_exact_same_basename_never_selects_sibling")
            self.opening_evidence("public_exact_path_same_basename", before, disks, after, now,
                                  capture=self.screenshot(editor, "routing-exact-path.png"))
            second = self.start_editor(project)
            try:
                descriptors = observation.wait_for(lambda: self.descriptors(project)
                                                  if len(self.descriptors(project)) == 2 else None,
                                                  "two_actual_same_project_editor_lifetimes")
                second_descriptor = next(value for value in descriptors if
                                         value["session_id"] != descriptor["session_id"])
                self.wait_restored_documents(second, after["open_paths"])
                self.open_action(second, "open_setup", paths=[CURRENT])
                self.open_action(second, "open_mutate", mutation="target_close")
                second_before, second_disk = self.open_state(second, project)
                first_before, first_disk = self.open_state(editor, project)
                ambiguous = self.public_open(project, descriptor, "public_ambiguous_live_sessions",
                                             "refused", session=None, reason="ambiguous_session")
                observation.require(ambiguous["resolved_target"] is None and
                                    ambiguous["before"] is None and ambiguous["observation"] is None,
                                    "public_ambiguity_no_candidate_source")
                self.assert_no_effect(editor, project, first_before, first_disk,
                                      "public_ambiguous_live_sessions")
                second_after, second_now = self.open_state(second, project)
                observation.json_file(self.artifacts / "ambiguous-second-session-witness.json", {
                    "before": self.concise_open_state(second_before, second_disk),
                    "after": self.concise_open_state(second_after, second_now)})
                self.cases[-1]["other_session_witness"] = "ambiguous-second-session-witness.json"
                observation.require((second_after, second_now) == (second_before, second_disk),
                                    "public_ambiguity_both_sessions_untouched")
                result = self.public_open(project, second_descriptor, "public_exact_second_live_session")
                second_after, second_now = self.open_state(second, project)
                self.assert_public_sources(result, second_after, second_now, second_descriptor)
                self.assert_public_before(result, second_before, second_disk)
                self.assert_public_selection(result, second_before, second_after, effect="selected_target")
                observation.require(self.open_state(editor, project) == (first_before, first_disk),
                                    "public_exact_session_not_editor_focus")
                self.opening_evidence("public_exact_second_live_session", second_before, second_disk,
                                      second_after, second_now, capture=
                                      self.screenshot(second, "routing-exact-second-session.png"))
            finally:
                self.close_editor(second)
                self.editors.remove(second)
            ended = self.public_open(project, descriptor, "public_ended_session_no_substitution", "refused",
                                     session=second_descriptor["session_id"], reason=(
                                         "session_ended", "editor_unavailable"))
            observation.require(ended["observation"] is None, "public_ended_session_source_free")
            self.assert_no_effect(editor, project, first_before, first_disk,
                                  "public_ended_session_no_substitution")
            bad_metadata = next(path for path in self.registry.glob("*.json") if
                                json.loads(path.read_text())["session_id"] == descriptor["session_id"])
            original = bad_metadata.read_bytes()
            denied_descriptor = json.loads(original)
            denied_descriptor["token"] = secrets.token_hex(32)
            observation.json_file(bad_metadata, denied_descriptor)
            try:
                denied = self.public_open(project, descriptor, "public_denied_authentication", "refused",
                                          reason="denied_access")
                observation.require(denied["before"] is None and denied["observation"] is None,
                                    "public_denied_authentication_no_source_or_hash")
            finally:
                bad_metadata.write_bytes(original)
                bad_metadata.chmod(0o600)
        replacement = self.start_editor(project)
        try:
            replacement_descriptor = observation.wait_for(
                lambda: next((value for value in self.descriptors(project) if
                              value["session_id"] != descriptor["session_id"]), None),
                "actual_restarted_editor_lifetime")
            self.wait_restored_documents(replacement, first_before["open_paths"])
            self.open_action(replacement, "open_setup", paths=[CURRENT])
            before, disks = self.open_state(replacement, project)
            self.public_open(project, replacement_descriptor, "public_replaced_editor_lifetime_refusal",
                             "refused", session=descriptor["session_id"],
                             reason=("session_ended", "session_changed", "editor_unavailable"))
            self.assert_no_effect(replacement, project, before, disks,
                                  "public_replaced_editor_lifetime_refusal")
        finally:
            self.close_editor(replacement)
            self.editors.remove(replacement)
        for path, reason in (("res://scripts/not-here.gd", "missing_script"),
                             ("res://scripts/note.txt", "invalid_document_kind"),
                             ("res://container.tscn::GDScript_x", "invalid_document_kind"),
                             ("../outside.gd", ("outside_project", "invalid_request")),
                             ("res://scripts/outside-link.gd", ("outside_project", "denied_access")),
                             ("res://scripts/subject.gd", "denied_access")):
            name = "public_target_refusal_" + ("denied_source" if path == TARGET else
                                               path.replace("/", "_").replace(":", "_").replace(".", "_"))
            with self.opening_fixture(name) as (project, editor, descriptor):
                outside = self.work / "outside.gd"
                outside.write_text("extends RefCounted\n# OPEN_OUTSIDE_PRIVATE_SOURCE\n")
                (project / "scripts/outside-link.gd").symlink_to(outside)
                outside_before = observation.disk_witness(outside)
                before, disks = self.open_state(editor, project)
                target = project / "scripts/subject.gd"
                if path == TARGET:
                    target.chmod(0)
                try:
                    result = self.public_open(project, descriptor, name, "refused", script=path, reason=reason)
                finally:
                    if path == TARGET:
                        target.chmod(0o600)
                after, now = self.open_state(editor, project)
                observation.require(before == after and all(now[key]["text"] == disks[key]["text"] and
                                    now[key]["inode"] == disks[key]["inode"] and
                                    now[key]["mtime_ns"] == disks[key]["mtime_ns"] for key in disks),
                                    "public_document_refusal_no_source_lifecycle_change")
                observation.require(result["observation"] is None and result["before"] is None,
                                    "public_denied_invalid_or_missing_target_no_candidate_source")
                self.opening_evidence(name, before, disks, after, now)
                observation.require(observation.disk_witness(outside) == outside_before,
                                    "public_denied_outside_authority_untouched")
        for mode, reason in (("target_cache_source", "cached_source_conflict"),
                             ("target_cache_edited", "dirty_conflict"),
                             ("target_cache_wrong_type", "invalid_document_kind")):
            name = "public_retained_cache_refusal_" + mode
            with self.opening_fixture(name, cached=mode != "target_cache_wrong_type") as (
                    project, editor, descriptor):
                self.open_action(editor, "open_mutate", mutation=mode)
                before, disks = self.open_state(editor, project)
                self.public_open(project, descriptor, name, "refused", reason=reason)
                self.assert_no_effect(editor, project, before, disks, name)
        for name, source in (
                ("tool", '@tool\nextends RefCounted\n'),
                ("static_initializer", (FIXTURE / "scripts/unsafe_effect.gd").read_text()),
                ("dependency", 'extends "res://scripts/open/autoload.gd"\n'),
                ("global_class", "class_name PublicOpeningUnsafeGlobal\nextends RefCounted\n"),
                ("source_over_limit", "# " + "x" * 524288)):
            with self.opening_fixture("profile-" + name, source=source) as (project, editor, descriptor):
                before, disks = self.open_state(editor, project)
                case = "public_unsupported_effect_profile_" + name
                self.public_open(project, descriptor, case, "refused", reason=(
                    "unsupported_representation", "unsafe_editor_context", "evidence_unavailable",
                    "denied_access"))
                self.assert_no_effect(editor, project, before, disks, case)
                observation.require(not (project / "opening-unpermitted-effect.txt").exists(),
                                    "public_unsupported_load_never_executes_actual_static_initializer")
        for action in ("unpathed_script", "duplicate_script"):
            name = "public_unavailable_native_context_" + action
            with self.opening_fixture(name) as (project, editor, descriptor):
                if action == "duplicate_script":
                    self.open_action(editor, "open_setup", paths=[TARGET, CURRENT])
                self.action(editor, action)
                before, disks = self.open_state(editor, project)
                self.public_open(project, descriptor, name, "refused", reason=(
                    "unsafe_editor_context", "open_state_unknown", "evidence_unavailable",
                    "unsupported_representation"))
                self.assert_no_effect(editor, project, before, disks, name)
        for change in ("file_replace", "file_content", "parent_replace", "target_cache_replace",
                       "current_replace", "warning", "external_editor", "session"):
            name = "public_preentry_race_" + ("native_session_closed" if change == "session" else change)
            with self.opening_fixture(name) as (project, editor, descriptor):
                process, started, _ = self.held_open(project, editor, descriptor, "bind", name)
                target = project / "scripts/subject.gd"
                if change == "file_replace":
                    replacement = target.with_suffix(".replacement")
                    replacement.write_text(TARGET_SOURCE)
                    replacement.replace(target)
                elif change == "file_content":
                    target.write_text(TARGET_SOURCE + "# OPEN_NEWER_DISK_TEXT\n")
                elif change == "parent_replace":
                    old = project / "scripts-old"
                    (project / "scripts").rename(old)
                    shutil.copytree(old, project / "scripts")
                elif change == "target_cache_replace":
                    self.open_action(editor, "open_mutate", mutation="target_cache")
                else:
                    self.open_action(editor, "open_mutate", mutation=change)
                survivor, disks = self.open_state(editor, project)
                # The bind control already crossed parent authorization. Direct
                # native-session destruction loses its attributable discard
                # receipt; observed absence afterward cannot imply rollback.
                self.finish_held_open(editor, process, started, name,
                                      "effects_unknown" if change == "session" else "refused",
                                      reason="protocol_error" if change == "session" else None)
                after, now = self.open_state(editor, project)
                observation.require(after["target"] == survivor["target"] and
                                    after["cached_id"] == survivor["cached_id"] and
                                    after["cached_R"] == survivor["cached_R"] and
                                    after["open_paths"] == survivor["open_paths"] and now == disks,
                                    "public_preentry_identity_race_no_stale_retarget_or_late_entry")
                self.opening_evidence(name, survivor, disks, after, now)
        self.public_busy()

    def public_busy(self):
        with self.opening_fixture("public-real-busy") as (project, editor, descriptor):
            basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                 script=CURRENT, name="public_busy_current_fresh_basis")
            process, payload, started = self.held_edit(
                project, editor, basis, CURRENT_SOURCE + "# BUSY_HUMAN_CURRENT\n",
                "prepare", "public_busy_edit_owner", session=descriptor["session_id"], script=CURRENT)
            try:
                before, disks = self.open_state(editor, project)
                observation.require(before["slot_busy"], "actual_existing_edit_owns_shared_slot")
                self.public_open(project, descriptor, "public_busy_refused_never_queued",
                                 "refused", reason="busy")
                after, now = self.open_state(editor, project)
                observation.require(before == after and disks == now, "public_busy_no_slot_or_source_effect")
                self.opening_evidence("public_busy_refused_never_queued", before, disks, after, now)
            finally:
                process.send_signal(signal.SIGTERM)
                self.complete_edit(process, payload, "public_busy_owner_cancelled", "refused", 4,
                                   started=started, reason="cancellation", application="not_applied")
                self.native_action(editor, "native_edit_release")
            survivor, disk = self.open_state(editor, project)
            self.open_action(editor, "open_idle")
            later, later_disk = self.open_state(editor, project)
            observation.require(not later["target"]["associated"] and not later["cached_id"] and
                                not later["slot_busy"] and survivor == later and disk == later_disk,
                                "public_busy_refusal_irrevocable_after_other_owner_release")

    def _owned_process_info(self, pid):
        inspected = observation.run(
            ["/bin/ps", "-ww", "-p", pid, "-o", "pid=,ppid=,pgid=,command="], timeout=1)
        observation.require(inspected.returncode in (0, 1), "owned_process_inspection")
        rows = inspected.stdout.decode().splitlines()
        if not rows:
            return None
        observation.require(len(rows) == 1, "unique_owned_process")
        values = rows[0].split(None, 3)
        observation.require(len(values) == 4 and int(values[0]) == pid,
                            "attributable_owned_process_identity")
        return {"pid": pid, "parent": int(values[1]), "group": int(values[2]),
                "command": values[3]}

    def _owned_children(self, parent):
        inspected = observation.run(["/usr/bin/pgrep", "-P", parent], timeout=1)
        observation.require(inspected.returncode in (0, 1), "owned_descendant_inspection")
        return [int(value) for value in inspected.stdout.split()]

    def _owned_opening_worker(self, process):
        matches = []
        for pid in self._owned_children(process.pid):
            child = self._owned_process_info(pid)
            if child is not None and child["command"].endswith(" --internal-open-worker"):
                matches.append(child)
        observation.require(len(matches) <= 1, "one_actual_owned_opening_worker")
        return matches[0] if matches else None

    def _owned_group_members(self, group):
        inspected = observation.run(["/bin/ps", "-ax", "-o", "pid=,ppid=,pgid=,stat="], timeout=1)
        observation.require(inspected.returncode == 0, "owned_validation_process_group_inspection")
        members = []
        for line in inspected.stdout.decode().splitlines():
            values = line.split()
            if len(values) == 4 and int(values[2]) == group:
                members.append({"pid": int(values[0]), "parent": int(values[1]),
                                "group": group, "state": values[3]})
        return sorted(members, key=lambda value: value["pid"])

    def _hold_public_open_context(self, project, editor, descriptor, name, owned):
        process, started, event = self.held_open(project, editor, descriptor, "prepare", name)
        owned.update(process=process, started=started, event=event)
        owned["worker"] = observation.wait_for(
            lambda: self._owned_opening_worker(process), "actual_public_opening_worker", timeout=1)
        self.open_action(editor, "open_release")

        def helper_and_engine():
            observation.require(process.poll() is None, "public_caller_live_during_open_context_helper")
            for pid in self._owned_children(process.pid):
                helper = self._owned_process_info(pid)
                if helper is None or not helper["command"].endswith(" --internal-stock-validation-worker"):
                    continue
                owned["helper"] = helper
                for child in self._owned_children(pid):
                    engine = self._owned_process_info(child)
                    if engine is not None:
                        executable = observation.run(["/bin/ps", "-p", child, "-o", "comm="], timeout=1)
                        if Path(executable.stdout.decode().strip()).name == self.args.godot.name:
                            # Stop the real helper only once its actual stock child exists,
                            # before it can deliver a completed-validation receipt.
                            os.kill(pid, signal.SIGSTOP)
                            os.kill(child, signal.SIGSTOP)
                            owned["engine"] = engine
                            return helper, engine
            return None

        helper, engine = observation.wait_for(
            helper_and_engine, "actual_parent_owned_open_context_helper_and_stock_descendant", timeout=4)
        observation.require(helper["parent"] == process.pid and
                            engine["parent"] == helper["pid"] and
                            helper["group"] == engine["group"] == helper["pid"] and
                            helper["group"] not in (os.getpgrp(), process.pid, editor["process"].pid) and
                            self._owned_process_info(owned["worker"]["pid"]) is not None,
                            "helper_is_parent_owned_sibling_not_owned_by_killable_opening_worker")
        working = observation.run(
            ["/usr/sbin/lsof", "-a", "-p", engine["pid"], "-d", "cwd", "-Fn"], timeout=1)
        directories = [Path(line[1:]) for line in working.stdout.decode().splitlines()
                       if line.startswith("n")]
        observation.require(len(directories) == 1, "actual_stock_descendant_private_working_directory")
        staged = directories[0]
        private = staged.parent
        metadata = private.lstat()
        observation.require(staged.name == "project" and
                            private.name.startswith("godot-validation-") and
                            private.parent.resolve() == Path(tempfile.gettempdir()).resolve() and
                            stat.S_ISDIR(metadata.st_mode) and metadata.st_uid == os.geteuid() and
                            stat.S_IMODE(metadata.st_mode) == 0o700 and not private.is_symlink(),
                            "independently_observed_owned_private_validation_staging")
        owned.update(private=private, private_identity=(metadata.st_dev, metadata.st_ino))
        current = observation.disk_witness(staged / CURRENT.removeprefix("res://"))
        state, disks = self.open_state(editor, project)
        observation.require(state["slot_busy"] and state["owner_matches"] and
                            state["current"]["R"] == state["current"]["B"] == current["text"] and
                            not state["target"]["associated"] and not state["cached_id"],
                            "real_open_context_uses_admitted_private_current_before_any_effect")
        members = self._owned_group_members(helper["group"])
        observation.require({helper["pid"], engine["pid"]} <= {row["pid"] for row in members},
                            "actual_helper_and_stock_descendant_belong_to_owned_cleanup_group")
        owned["group_before"] = members
        return state, disks

    def _assert_validation_cleanup(self, owned, name):
        group = owned["helper"]["group"]
        observation.wait_for(
            lambda: all(self._owned_process_info(owned[key]["pid"]) is None
                        for key in ("worker", "helper", "engine")) and
                    not self._owned_group_members(group),
            "caller_worker_helper_descendants_and_group_gone_" + name, timeout=2)
        observation.wait_for(lambda: not owned["private"].exists(),
                             "validation_private_staging_removed_" + name, timeout=2)
        return {"public_parent_pid": owned["process"].pid,
                "worker_pid": owned["worker"]["pid"],
                "helper_pid": owned["helper"]["pid"], "stock_descendant_pid": owned["engine"]["pid"],
                "helper_process_group": group, "group_before": owned["group_before"],
                "group_after": self._owned_group_members(group),
                "private_root": str(owned["private"]), "private_root_identity": owned["private_identity"],
                "worker_gone": True, "helper_gone": True, "stock_descendant_gone": True,
                "private_staging_removed": True,
                "witness": "independent_ps_pgrep_lsof_and_filesystem_after_public_result"}

    def _cleanup_validation_control(self, owned):
        # Failure-only fallback runs after the acceptance assertions, never creates
        # their proof. Restrict every signal/removal to captured owned identities.
        process = owned.get("process")
        if process is not None:
            if process.poll() is None:
                process.send_signal(signal.SIGTERM)
            try:
                process.communicate(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=2)
            if process in self.open_processes:
                self.open_processes.remove(process)
        helper = owned.get("helper")
        if helper is not None:
            captured = [value for key in ("helper", "engine") if (value := owned.get(key)) is not None]
            if any((live := self._owned_process_info(value["pid"])) is not None and
                   live["group"] == helper["group"] and live["command"] == value["command"]
                   for value in captured):
                try:
                    os.killpg(helper["group"], signal.SIGKILL)
                except ProcessLookupError:
                    pass
        private = owned.get("private")
        if private is not None and private.exists():
            metadata = private.lstat()
            observation.require((metadata.st_dev, metadata.st_ino) == owned["private_identity"] and
                                stat.S_ISDIR(metadata.st_mode) and not private.is_symlink(),
                                "failure_cleanup_only_captured_private_staging_identity")
            shutil.rmtree(private)

    def public_open_context_failures(self):
        reasons = {"opening_worker_killed": "disconnected",
                   "helper_interrupted": "current_validation_unavailable",
                   "helper_killed": "current_validation_unavailable",
                   "stock_descendant_killed": "current_validation_unavailable",
                   "caller_cancelled": "cancelled",
                   "helper_deadline": ("timeout", "current_validation_unavailable")}
        for kind, reason in reasons.items():
            name = "public_open_context_" + kind
            with self.opening_fixture(name) as (project, editor, descriptor):
                before, disks = self.open_state(editor, project)
                owned = {}
                try:
                    held, held_disk = self._hold_public_open_context(
                        project, editor, descriptor, name, owned)
                    self.assert_preserved(before, held, disks, held_disk, name, selection=True)
                    if kind == "opening_worker_killed":
                        os.kill(owned["worker"]["pid"], signal.SIGKILL)
                    elif kind == "helper_interrupted":
                        os.kill(owned["helper"]["pid"], signal.SIGTERM)
                        os.kill(owned["helper"]["pid"], signal.SIGCONT)
                    elif kind == "helper_killed":
                        os.kill(owned["helper"]["pid"], signal.SIGKILL)
                    elif kind == "stock_descendant_killed":
                        os.kill(owned["engine"]["pid"], signal.SIGKILL)
                        os.kill(owned["helper"]["pid"], signal.SIGCONT)
                    elif kind == "caller_cancelled":
                        owned["process"].send_signal(signal.SIGTERM)
                    else:
                        # The parent/helper deadline must kill even a stopped stock child.
                        os.kill(owned["helper"]["pid"], signal.SIGCONT)
                    result = self.complete_open(
                        owned["process"], owned["started"], name, "refused", reason=reason)
                    observation.require(result["stage"] == "context_validating" and
                                        result["selection"]["request_effect"] == "none" and
                                        all(result["progress"][stage]["state"] in ("not_started", "not_applicable")
                                            for stage in ("resource_binding", "initial_compilation", "document_open")),
                                        "public_helper_failure_cannot_release_lifecycle_authorization")
                    cleanup = self._assert_validation_cleanup(owned, name)
                    after, now = self.assert_no_effect(editor, project, before, disks, name)
                    self.open_action(editor, "open_idle")
                    late, late_disk = self.open_state(editor, project)
                    observation.require(late == after and late_disk == now and
                                        not late["slot_busy"] and not late["cached_id"] and
                                        not late["target"]["associated"],
                                        "real_helper_failure_irrevocable_no_effect_no_late_native_entry")
                    self.opening_evidence(name, before, disks, after, now,
                                          capture=self.screenshot(editor, name + ".png"))
                    self.cases[-1].update(process_cleanup=cleanup, boundary_event=owned["event"],
                                         at_helper_interruption=self.concise_open_state(held, held_disk))
                finally:
                    self._cleanup_validation_control(owned)

    def interruption(self):
        self.compile_window_probe()
        self.public_open_context_failures()
        checkpoints = (
            ("before_authorization", "prepare", "", "refused"),
            ("lost_preparation_receipt", "", "open_prepared", "refused"),
            ("after_authorization", "bind", "", ("refused", "effects_unknown")),
            ("after_cache_publication", "compile", "", "applied_unverified"),
            ("after_compilation", "open", "", "applied_unverified"),
            ("after_document_open", "verify:post_open", "", "applied_unverified"),
            ("during_verification", "recheck:post_open", "", "applied_unverified"),
            ("lost_publication_receipt", "", "open_progress", "effects_unknown"),
            ("lost_verification_receipt", "", "open_sample", "applied_unverified"))
        for kind in ("stall", "loss", "cancel", "disable", "worker_loss"):
            for checkpoint, stage, response_kind, expected in checkpoints:
                name = "public_" + kind + "_" + checkpoint
                with self.opening_fixture(name) as (project, editor, descriptor):
                    before, disks = self.open_state(editor, project)
                    process, started, event = self.held_open(
                        project, editor, descriptor, stage, name, response_kind=response_kind)
                    at_boundary, boundary_disk = self.open_state(editor, project)
                    worker = observation.wait_for(
                        lambda: self._owned_opening_worker(process), "actual_owned_opening_worker_at_gap",
                        timeout=1) if kind == "worker_loss" else None
                    if kind == "loss":
                        self.open_action(editor, "open_disconnect")
                    elif kind == "cancel":
                        process.send_signal(signal.SIGTERM)
                    elif kind == "disable":
                        disabled = self.action(editor, "disable")
                        observation.require(disabled["plugin_enabled"] is False,
                                            "actual_plugin_disabled_during_peer_owned_opening_attempt")
                    elif kind == "worker_loss":
                        os.kill(worker["pid"], signal.SIGKILL)
                    # Loss of owned IPC is not proof that the editor disconnected.
                    # Its exact transport reason is incidental; enforce the public
                    # effect certainty, bounded delivery and actual cleanup below.
                    result = self.complete_open(process, started, name, expected,
                                                reason=None if kind == "worker_loss" else (
                                                    "timeout", "disconnected", "cancelled",
                                                    "verification_incomplete", "evidence_unavailable"))
                    self.open_action(editor, "open_release")
                    survivor, now = self.open_state(editor, project)
                    self.assert_preserved(before, survivor, disks, now, name,
                                          selection=checkpoint in ("before_authorization", "after_authorization"))
                    observation.require(now == boundary_disk and
                                        survivor["target"] == at_boundary["target"] and
                                        survivor["cached_id"] == at_boundary["cached_id"] and
                                        survivor["cached_R"] == at_boundary["cached_R"],
                                        "public_interruption_independent_survivor_no_rollback")
                    self.open_action(editor, "open_idle")
                    late, late_disk = self.open_state(editor, project)
                    observation.require(late == survivor and late_disk == now and not late["slot_busy"],
                                        "public_terminal_result_no_retry_or_late_false_refusal_effect")
                    if result["outcome"] == "refused":
                        observation.require(not late["target"]["associated"] and not late["cached_id"],
                                            "public_refused_attempt_can_never_open_later")
                    self.opening_evidence(name, before, disks, survivor, now,
                                          capture=self.screenshot(editor, name + ".png"))
                    self.cases[-1]["boundary_event"] = event
                    self.cases[-1]["at_interruption"] = self.concise_open_state(at_boundary, boundary_disk)
                    if kind == "worker_loss":
                        observation.require(self._owned_process_info(worker["pid"]) is None and
                                            not self._owned_children(process.pid),
                                            "actual_killed_opening_worker_reaped_and_no_parent_descendant")
                        self.cases[-1]["killed_opening_worker_pid"] = worker["pid"]
                    if kind == "disable":
                        observation.require(not self.descriptors(project),
                                            "plugin_disable_removes_owned_peer_routing_descriptor")
                        self.cases[-1]["plugin_disabled"] = True
                    if survivor["target"]["associated"] and kind != "disable":
                        fresh = self.observe(project, "complete_observation", 0,
                                             session=descriptor["session_id"], name=name + "_fresh_observe")
                        observation.require(fresh["snapshot"]["sources"]["B"]["text"] ==
                                            survivor["target"]["B"], "public_interrupted_fresh_original_target")
                        repeated = name + "_intentional_recognition"
                        self.public_open(project, descriptor, repeated, "already_open_unchanged")
                        self.assert_no_effect(editor, project, survivor, now, repeated)
        for kind in ("stall", "loss", "cancel", "disable", "worker_loss"):
            for stage, expected in (("bind", "effects_unknown"), ("compile", "applied_unverified"),
                                    ("open", "applied_unverified")):
                name = "public_native_entered_" + kind + "_" + stage
                with self.opening_fixture(name, faults=True) as (project, editor, descriptor):
                    before, disks = self.open_state(editor, project)
                    process, started, event = self.held_open(
                        project, editor, descriptor, "", name, native_stage=stage,
                        native_fault="stall_" + stage,
                        callback={"loss": "stop", "disable": "disable"}.get(kind, ""))
                    if kind == "cancel":
                        process.send_signal(signal.SIGTERM)
                    elif kind == "worker_loss":
                        worker = observation.wait_for(
                            lambda: self._owned_opening_worker(process),
                            "actual_owned_opening_worker_during_entered_native_call", timeout=1)
                        os.kill(worker["pid"], signal.SIGKILL)
                    self.complete_open(process, started, name, expected,
                                       reason={"stall": "timeout", "loss": "disconnected",
                                               "disable": "disconnected", "worker_loss": None,
                                               "cancel": "cancelled"}[kind])
                    # Synchronous native work cannot be cancelled by the caller.
                    # Read only after its actual return, then prove the late guard.
                    self.open_action(editor, "open_release", timeout=20)
                    callback = self.open_action(editor, "open_callback_witness")
                    if kind == "disable":
                        retained = callback["callback"]
                        observation.require(callback["plugin_enabled"] is False and
                                            retained["retained_slot_busy"] and retained["retained_owner_matches"] and
                                            retained["current_reference_alive"],
                                            "public_disable_retains_entered_owner_and_actual_departing_references")
                    after, now = self.open_state(editor, project)
                    self.assert_preserved(before, after, disks, now, name, selection=True)
                    observation.require(not after["target"]["associated"] and not after["slot_busy"] and
                                        bool(after["cached_id"]) == (stage != "bind"),
                                        "native_entered_interruption_survivor_no_late_open")
                    self.open_action(editor, "open_idle")
                    late, late_disk = self.open_state(editor, project)
                    observation.require(after == late and now == late_disk,
                                        "native_entered_late_return_never_amends_delivered_outcome")
                    self.opening_evidence(name, before, disks, after, now,
                                          capture=self.screenshot(editor, name + ".png"))
                    self.cases[-1]["native_entered_event"] = event
                    if kind == "disable":
                        retained = callback["callback"]
                        self.cases[-1]["disable_entered_ownership_witness"] = {
                            key: retained[key] for key in ("stage", "retained_before", "retained_after",
                                                          "retained_slot_busy", "retained_owner_matches",
                                                          "current_reference_alive")}
                    if kind == "worker_loss":
                        observation.require(self._owned_process_info(worker["pid"]) is None,
                                            "actual_entered_opening_worker_kill_reaped")
                        self.cases[-1]["killed_opening_worker_pid"] = worker["pid"]

    def privacy_export(self):
        self.compile_window_probe()
        markers = {"selected": "OPEN_SELECTED_PRIVATE_SOURCE", "current": "OPEN_CURRENT_PRIVATE_SOURCE",
                   "background": "OPEN_UNRELATED_PRIVATE_SOURCE", "other": "OPEN_OTHER_PROJECT_PRIVATE_SOURCE"}
        self.source_markers.update(value.encode() for value in markers.values())
        source = self.revision(41).replace("extends RefCounted", "extends RefCounted\n# " + markers["selected"])
        current = CURRENT_SOURCE.replace("extends RefCounted", "extends RefCounted\n# " + markers["current"])
        background = self.revision(11).replace("extends RefCounted", "extends RefCounted\n# " + markers["background"])
        other_source = self.revision(97).replace("extends RefCounted", "extends RefCounted\n# " + markers["other"])
        def setup(project):
            (project / "scripts/open/background.gd").write_text(background)
        with self.opening_fixture("privacy-other-project", source=other_source) as (
                other_project, other_editor, other_descriptor):
            other_before, other_disk = self.open_state(other_editor, other_project)
            with self.opening_fixture("privacy-selected", source=source, current=current, setup=setup) as (
                    project, editor, descriptor):
                before, disks = self.open_state(editor, project)
                authorized = self.public_open(project, descriptor, "public_privacy_authorized")
                after, now = self.open_state(editor, project)
                self.assert_preserved(before, after, disks, now, "privacy-authorized")
                self.assert_public_selection(authorized, before, after, effect="selected_target")
                self.opening_evidence("public_privacy_authorized", before, disks, after, now,
                                      capture=self.screenshot(editor, "public-privacy-authorized.png"))
                second = self.start_editor(project)
                try:
                    observation.wait_for(lambda: len(self.descriptors(project)) == 2,
                                         "privacy_actual_live_ambiguity")
                    self.wait_restored_documents(second, after["open_paths"])
                    second_before, second_disk = self.open_state(second, project)
                    ambiguous = self.public_open(project, descriptor, "public_privacy_ambiguous", "refused",
                                                 session=None, reason="ambiguous_session")
                    self.assert_no_effect(editor, project, after, now, "public_privacy_ambiguous")
                    observation.require(self.open_state(second, project) == (second_before, second_disk),
                                        "privacy_other_same_project_session_untouched")
                finally:
                    self.close_editor(second)
                    self.editors.remove(second)
                self.open_action(editor, "open_mutate", mutation="target_close")
                closed, closed_disk = self.open_state(editor, project)
                target = project / "scripts/subject.gd"
                target.chmod(0)
                try:
                    denied = self.public_open(project, descriptor, "public_privacy_denied", "refused",
                                              reason="denied_access")
                finally:
                    target.chmod(0o600)
                denied_after, denied_disk = self.open_state(editor, project)
                observation.require(closed == denied_after and
                                    denied_disk["target"]["text"] == closed_disk["target"]["text"],
                                    "privacy_denied_survivor_unchanged")
                self.opening_evidence("public_privacy_denied", closed, closed_disk, denied_after, denied_disk)
                process, started, _ = self.held_open(project, editor, descriptor, "verify:post_open",
                                                     "public_privacy_interrupted")
                self.open_action(editor, "open_mutate", mutation="dirty_different", path=TARGET)
                survivor, survivor_disk = self.open_state(editor, project)
                process.send_signal(signal.SIGTERM)
                interrupted = self.complete_open(process, started, "public_privacy_interrupted",
                                                  "applied_unverified", reason="cancelled")
                self.open_action(editor, "open_release")
                final, final_disk = self.open_state(editor, project)
                observation.require(final["target"] == survivor["target"] and final_disk == survivor_disk,
                                    "privacy_interrupted_newer_human_work_survives")
                self.opening_evidence("public_privacy_interrupted", survivor, survivor_disk, final, final_disk,
                                      capture=self.screenshot(editor, "public-privacy-human-survivor.png"))
                for label, result in (("authorized", authorized), ("ambiguous", ambiguous),
                                      ("denied", denied), ("interrupted", interrupted)):
                    encoded = json.dumps(result)
                    for key, private_source in (("current", current), ("background", background),
                                                ("other", other_source)):
                        observation.require(markers[key] not in encoded and sha(private_source) not in encoded,
                                            "public_privacy_no_private_source_or_hash_" + label + "_" + key)
                    observation.require(CURRENT not in encoded and BACKGROUND not in encoded and
                                        str(other_project) not in encoded,
                                        "public_privacy_no_private_context_path_" + label)
                    if label in ("ambiguous", "denied"):
                        observation.require(markers["selected"] not in encoded and sha(source) not in encoded and
                                            result["before"] is None and result["observation"] is None,
                                            "public_privacy_denied_candidate_suppressed_" + label)
                    elif label == "authorized":
                        observation.require(markers["selected"] in encoded,
                                            "public_privacy_only_selected_target_intentionally_exposed")
                self.descriptors()
            observation.require(self.open_state(other_editor, other_project) == (other_before, other_disk),
                                "privacy_other_project_independent_survivor_untouched")
        self.native_export()
