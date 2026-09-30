"""Real edit-gdscript GUI acceptance; all mutation goes through the public CLI."""
import json
import os
import secrets
import shutil
import signal
import stat
import subprocess
import time

import run_observation as observation
from native_finalization_acceptance import DESIRED, ROOT
from caller_privacy_acceptance import CallerPrivacyAcceptanceMixin
from edit_result_review import review_edit_result


class CallerEditAcceptanceMixin(CallerPrivacyAcceptanceMixin):
    def caller_fixture(self, name, *, prior=False, faults=False, baseline=None):
        project = self.fixture("caller-" + name, controlled=True)
        if baseline is not None:
            (project / "scripts/subject.gd").write_text(baseline)
        if faults:
            destination = project / "addons/godot_agent_kit/native"
            shutil.copytree(self.args.native_fault_addon, destination, dirs_exist_ok=True)
        self.installed_native(project, fixture_only=faults)
        editor = self.start_editor(project)
        descriptor = observation.wait_for(lambda: next(iter(self.descriptors(project)), None),
                                          "edit_native_advertisement")
        observation.require(descriptor["v"] == 3, "edit_private_v3_descriptor")
        info = self.native_action(editor, "native_info")
        observation.require(info["edit_installed"] and info["open_installed"] and
                            info["api_revision"] == 2 and
                            info["build_id"] == self.expected_native_build_id, "real_native_shared_build")
        if faults:
            self.native_action(editor, "native_edit_fixture_activate")
        self.prepare_caller_subject(project, editor, descriptor, name)
        if prior:
            seeded = self.native_action(editor, "native_edit_human", mode="seed_prior")
            observation.require(seeded["document"]["dirty"] and
                                seeded["document"]["has_undo"], "real_prior_history")
            self.native_action(editor, "native_edit_save")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.native_action(editor, "native_idle")
        before, disks = self.snapshot(editor, project)
        observation.require(not before["subject"]["dirty"] and
                            before["subject"]["R"] == before["subject"]["B"] ==
                            disks["subject"]["text"] and before["other"]["dirty"] and
                            before["current_script"] == "res://scripts/other.gd", "edit_independent_initial_state")
        return project, editor, descriptor, before, disks

    def prepare_caller_subject(self, project, editor, descriptor, name):
        self.action(editor, "prepare_subject")

    def edit_basis(self, project, descriptor, name):
        return self.observe(project, "complete_observation", 0,
                            session=descriptor["session_id"], name=name)

    def edit_command(self, project, session=None, script=ROOT):
        command = [str(self.args.editor), "--registry", str(self.registry), "--project",
                   str(project), "--script", script]
        if session is not None:
            command.extend(("--session", session))
        return command

    def edit_payload(self, basis, desired, request_id=None):
        return {"schema_version": 1, "request_id": request_id or secrets.token_hex(16),
                "basis": basis, "replacement_source": desired}

    def validated_source(self, result, desired, purpose, status, *, diagnostic_path=None):
        attributed = [value for value in result["validation"]
                      if value["purpose"] == purpose and value["status"] == status and
                      value["source_path"] == ROOT and
                      value["input"] == {"sha256": self.stock_sha(desired),
                                         "utf8_bytes": len(desired.encode("utf-8"))}]
        observation.require(len(attributed) == 1, "caller_exact_fresh_validation_" + purpose)
        validation = attributed[0]
        observation.require(validation["cleanup_confirmed"] is True and
                            validation["context_current"] is True and
                            validation["dependencies_current"] is True and
                            all(source["diagnostics_completed"] and source["symbols_completed"]
                                for source in validation["sources"]) and
                            any(source["path"] == ROOT and
                                source["source"] == validation["input"] for source in validation["sources"]),
                            "independent_helper_root_and_closure_fences_" + purpose)
        if diagnostic_path is not None:
            observation.require(any(diagnostic["path"] == diagnostic_path and
                                    diagnostic["source"] is not None
                                    for diagnostic in validation["diagnostics"]),
                                "real_source_attributed_diagnostic_" + purpose)
        return validation

    def edit_process(self, project, payload, *, session=None, script=ROOT):
        return subprocess.Popen(self.edit_command(project, session, script),
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE)

    def complete_edit(self, process, payload, name, expected, code, *, reason=None,
                      application=None, started=None, correlated=True):
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8") if process.stdin is not None else None
        started = started or time.monotonic()
        try:
            stdout, stderr = process.communicate(
                data, timeout=max(0.1, 10.4 - (time.monotonic() - started)))
        except subprocess.TimeoutExpired as error:
            process.kill()
            process.communicate(timeout=2)
            raise observation.Failure("edit_supervised_deadline_exceeded_" + name) from error
        elapsed = time.monotonic() - started
        result = json.loads(stdout)
        observation.require(result.get("safe_next_action") and
                            all(secret not in stdout for secret in self.secrets),
                            "edit_next_action_and_no_credentials_" + name)
        self.safe_log(name + ".stderr", stderr)
        observation.json_file(self.artifacts / (name + ".json"), result)
        observation.require(elapsed <= 10.0 and
                            process.returncode in ((code,) if isinstance(code, int) else code) and
                            stdout.endswith(b"\n") and stdout.count(b"\n") == 1,
                            f"edit_cli_bounded_single_result_{name}_exit_{process.returncode}_elapsed_{elapsed:.3f}")
        request_id = result.get("request_id")
        valid_id = isinstance(request_id, str) and 1 <= len(request_id) <= 64 and all(
            character in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"
            for character in request_id)
        observation.require(result.get("schema_version") == 1 and
                            valid_id and (not correlated or request_id == payload["request_id"]) and
                            result.get("outcome") in ((expected,) if isinstance(expected, str) else expected) and
                            (reason is None or result.get("reason") == reason) and
                            (application is None or result.get("application") == application),
                            "edit_public_outcome_" + name)
        result_review = review_edit_result(result)
        self.case(name, source_surfaces="actual_caller_and_independent_native_disk_witness",
                  outcome=result["outcome"], reason=result.get("reason"),
                  application=result.get("application"), history=result.get("history"),
                  elapsed_seconds=elapsed, evidence=name + ".json", result_review=result_review)
        return result

    def edit(self, project, basis, desired, name, expected, code, *, session=None,
             reason=None, application=None, script=ROOT, payload=None, correlated=True):
        payload = payload or self.edit_payload(basis, desired)
        started = time.monotonic()
        process = self.edit_process(project, payload, session=session, script=script)
        return self.complete_edit(process, payload, name, expected, code,
                                  reason=reason, application=application, correlated=correlated,
                                  started=started)

    def state(self, editor, project):
        before, disks = self.snapshot(editor, project)
        return before, disks

    def unchanged_state(self, editor, project, prior, disks, name):
        current, now = self.state(editor, project)
        observation.require(prior == current and disks == now,
                            "refusal_preserves_real_d_r_b_dirty_version_history_" + name)
        return current

    def changed_state(self, editor, project, prior, disks, desired, name):
        after, now = self.state(editor, project)
        subject = after["subject"]
        observation.require(subject["B"] == subject["R"] == now["subject"]["text"] == desired and
                            not subject["dirty"] and subject["version"] == subject["saved_version"] and
                            subject["has_undo"] and now["subject"]["mtime_ns"] ==
                            disks["subject"]["mtime_ns"] and
                            (subject["script_id"], subject["editor_id"], subject["buffer_id"]) ==
                            (prior["subject"]["script_id"], prior["subject"]["editor_id"],
                             prior["subject"]["buffer_id"]) and
                            after["other"] == prior["other"] and now["other"] == disks["other"] and
                            after["current_script"] == prior["current_script"],
                            "caller_independent_exact_d_r_b_saved_t0_isolation_" + name)
        observation.require(self.native_action(editor, "native_edit_inspect")["resource_edited"] is False,
                            "public_edited_flag_after_caller_" + name)
        return after, now

    def held_edit(self, project, editor, basis, desired, stage, name, *, session, script=ROOT):
        event = editor["control"] / "event.json"
        event.unlink(missing_ok=True)
        self.native_action(editor, "native_edit_hold", stage=stage)
        payload = self.edit_payload(basis, desired)
        started = time.monotonic()
        process = self.edit_process(project, payload, session=session, script=script)
        process.stdin.write(json.dumps(payload, ensure_ascii=False).encode("utf-8"))
        process.stdin.close()
        process.stdin = None

        def reached():
            observation.require(process.poll() is None and editor["process"].poll() is None,
                                "edit_barrier_process_live_" + name)
            if event.exists():
                value = json.loads(event.read_text())
                return value if value == {"stage": "edit:" + stage} else None
            return None

        observation.wait_for(reached, "real_edit_barrier_" + stage, timeout=6)
        return process, payload, started

    def release_edit(self, editor, process, payload, started, name, expected, code, *, reason=None):
        self.native_action(editor, "native_edit_release")
        return self.complete_edit(process, payload, name, expected, code,
                                  reason=reason, started=started)

    def clean_open_edit(self):
        self.compile_window_probe()
        project, editor, descriptor, before, disks = self.caller_fixture("clean")
        self.screenshot(editor, "caller-clean-before.png")
        basis = self.edit_basis(project, descriptor, "caller-clean-basis")
        changed = self.edit(project, basis, DESIRED, "caller_us1_clean_nonselected_dirty_other",
                            "verified_changed", 0, session=descriptor["session_id"])
        observation.require(changed["application"] == "applied" and
                            changed["history"] == "native_complex_edit", "real_caller_native_history")
        self.validated_source(changed, DESIRED, "preflight", "valid")
        self.validated_source(changed, DESIRED, "post_change", "valid")
        after, now = self.changed_state(editor, project, before, disks, DESIRED, "clean")
        self.screenshot(editor, "caller-clean-after.png")
        fresh = self.edit_basis(project, descriptor, "caller-clean-fresh")
        equal = self.edit(project, fresh, DESIRED, "caller_us1_verified_unchanged",
                          "verified_unchanged", 0, session=descriptor["session_id"])
        self.validated_source(equal, DESIRED, "unchanged", "valid")
        self.unchanged_state(editor, project, after, now, "equal_no_history_or_write")
        self.edit(project, basis, DESIRED, "caller_stale_equal_intent_is_not_unchanged",
                  "refused", 3, session=descriptor["session_id"], application="not_applied")
        self.unchanged_state(editor, project, after, now, "stale_equal_intent")
        later = self.native_action(editor, "native_edit_human", mode="later_value")["document"]
        observation.require(later["dirty"] and later["B"] != DESIRED,
                            "later_human_edit_after_agent_clean_tag")
        ordinary = self.native_action(editor, "native_edit_save")
        human, human_disk = self.state(editor, project)
        observation.require(ordinary["focused"] and ordinary["disk_matches"] and
                            human["subject"]["B"] == human["subject"]["R"] ==
                            human_disk["subject"]["text"] == later["B"] and
                            not human["subject"]["dirty"],
                            "later_ordinary_save_no_false_prompt_or_reconciliation")
        # Ordinary Godot Save may refresh other cached Script sources. It must
        # still preserve their unsaved buffer/history and never save their disk.
        observation.require(
            human["other"]["dirty"] and human_disk["other"] == disks["other"] and
            all(human["other"][key] == before["other"][key]
                for key in ("B", "version", "saved_version", "has_undo", "has_redo")),
            "later_human_save_preserves_unrelated_unsaved_work")
        reopened = self.native_action(editor, "native_edit_human", mode="close_reopen")["document"]
        observation.require(reopened["B"] == reopened["R"] == human_disk["subject"]["text"] and
                            not reopened["dirty"], "caller_edit_survives_real_close_reopen")
        fresh = self.edit_basis(project, descriptor, "basis-clean-followup")
        current, disk = self.state(editor, project)
        self.edit(project, fresh, DESIRED, "caller_fresh_repeat_after_human_save",
                  "verified_changed", 0, session=descriptor["session_id"])
        self.changed_state(editor, project, current, disk, DESIRED, "fresh_repeat")
        self.case("caller_later_human_save_reopen_and_repeat", screenshot=
                  self.screenshot(editor, "caller-later-save-reopen.png"),
                  source_surfaces="actual_later_human_save_close_reopen_fresh_basis")
        # Cross the source-free control ceiling through the actual authenticated
        # edit exchange, not just native mutation or a synthetic codec peer.
        fresh = self.edit_basis(project, descriptor, "caller-large-frame-basis")
        current, disk = self.state(editor, project)
        large = DESIRED + "# " + "large_frame_" * 1024 + "\n"
        changed = self.edit(project, fresh, large, "caller_v3_large_selected_frame",
                            "verified_changed", 0, session=descriptor["session_id"])
        self.validated_source(changed, large, "preflight", "valid")
        self.validated_source(changed, large, "post_change", "valid")
        self.changed_state(editor, project, current, disk, large, "large_selected_frame")

    def conflict_edit(self):
        self.compile_window_probe()
        for name, action, desired in (("dirty_different", "dirty_subject", DESIRED),
                                      ("dirty_equal", "equal_dirty_subject", DESIRED),
                                      ("same_text_changed_version", "native_same_text", DESIRED),
                                      ("stale_resource", "resource_subject", DESIRED)):
            project, editor, descriptor, _, _ = self.caller_fixture(name)
            capture = self.screenshot(editor, "caller-conflict-before.png") if name == "dirty_different" else None
            basis = self.edit_basis(project, descriptor, "basis-" + name)
            (self.native_action(editor, "native_edit_human", mode="same_text")
             if action == "native_same_text" else self.action(editor, action))
            before, disks = self.state(editor, project)
            self.edit(project, basis, desired, "caller_conflict_" + name,
                      "refused", 3, session=descriptor["session_id"], application="not_applied")
            if capture is not None:
                self.cases[-1]["screenshot"] = capture
            self.unchanged_state(editor, project, before, disks, name)
        project, editor, descriptor, before, disks = self.caller_fixture("missing-basis")
        basis = self.edit_basis(project, descriptor, "basis-missing")
        for name, faulty in (("missing_snapshot", {**basis, "snapshot": None}),
                             ("wrong_session", {**basis, "resolved_target":
                              {**basis["resolved_target"], "session_id": secrets.token_hex(16)}})):
            self.edit(project, faulty, DESIRED, "caller_conflict_" + name, "refused", 3,
                      session=descriptor["session_id"], application="not_applied")
            self.unchanged_state(editor, project, before, disks, name)
        for name, desired in (("carriage_return", "extends RefCounted\r\n"),
                              ("nul", "extends RefCounted\n\x00"),
                              ("bom", "\ufeffextends RefCounted\n"),
                              ("over_limit", "# " + "a" * 524288)):
            self.edit(project, basis, desired, "caller_conflict_" + name,
                      "refused", 3, session=descriptor["session_id"], application="not_applied")
            self.unchanged_state(editor, project, before, disks, name)
        process, payload, started = self.held_edit(project, editor, basis, DESIRED,
                                                   "apply", "new-human-before-boundary",
                                                   session=descriptor["session_id"])
        self.native_action(editor, "native_edit_human", mode="dirty")
        human, human_disk = self.state(editor, project)
        self.release_edit(editor, process, payload, started, "caller_human_before_apply_refused",
                          "refused", 3)
        self.unchanged_state(editor, project, human, human_disk, "human_before_apply")
        project, editor, descriptor, _, _ = self.caller_fixture("preflight-replaced-document")
        basis = self.edit_basis(project, descriptor, "basis-preflight-replaced-document")
        process, payload, started = self.held_edit(
            project, editor, basis, DESIRED, "verify:preflight", "guard-replaced-document",
            session=descriptor["session_id"])
        self.native_action(editor, "native_edit_human", mode="close_reopen")
        human, human_disk = self.state(editor, project)
        result = self.release_edit(
            editor, process, payload, started, "caller_replaced_guard_never_authorized",
            "refused", 3, reason="identity_changed")
        observation.require(result["application"] == "not_applied" and
                            result["history"] == "not_participated",
                            "fresh_guard_failure_has_no_released_authority")
        self.unchanged_state(editor, project, human, human_disk, "replaced_guard")
        self.conflict_basis_cases()

    def conflict_basis_cases(self):
        for name, transition, surface in (("known_stale_resource", "resource", "R"),
                                          ("known_stale_buffer", "buffer", "B")):
            project, editor, descriptor, _, _ = self.caller_fixture(name)
            if surface == "B":
                self.action(editor, "dirty_subject")
            self.action(editor, "transition_" + transition)
            basis = self.observe(project, "limited_observation", 2,
                                 session=descriptor["session_id"], name="basis-" + name)
            observation.require(basis["snapshot"]["sources"][surface]["availability"] ==
                                "unavailable" and
                                basis["snapshot"]["sources"][surface]["invalidated_evidence"] is not None,
                                "actual_known_stale_basis_" + name)
            before, disks = self.state(editor, project)
            self.edit(project, basis, DESIRED, "caller_conflict_" + name, "refused", 3,
                      session=descriptor["session_id"], application="not_applied")
            self.unchanged_state(editor, project, before, disks, name)
        for name, preparation, expected in (
                ("dirty_equal_basis", "equal_dirty_subject", "complete_observation"),
                ("divergent_basis", "resource_subject", "complete_observation"),
                ("missing_dirty_fact", "restrict_dirty", "limited_observation"),
                ("oversized_buffer_basis", "cap_buffer_over", "limited_observation")):
            project, editor, descriptor, _, _ = self.caller_fixture(name)
            self.action(editor, preparation)
            basis = self.observe(project, expected, 0 if expected == "complete_observation" else 2,
                                 session=descriptor["session_id"], name="basis-" + name)
            before, disks = self.state(editor, project)
            self.edit(project, basis, DESIRED, "caller_conflict_" + name, "refused", 3,
                      session=descriptor["session_id"], application="not_applied")
            self.unchanged_state(editor, project, before, disks, name)
        project, editor, descriptor, _, _ = self.caller_fixture("replaced-document")
        basis = self.edit_basis(project, descriptor, "basis-replaced-document")
        self.native_action(editor, "native_edit_human", mode="close_reopen")
        current, disk = self.state(editor, project)
        observation.require(basis["snapshot"]["document"]["identity"]["buffer_instance_id"] !=
                            current["subject"]["buffer_id"], "real_replaced_buffer_identity")
        self.edit(project, basis, DESIRED, "caller_conflict_replaced_document", "refused", 3,
                  session=descriptor["session_id"], application="not_applied")
        self.unchanged_state(editor, project, current, disk, "replaced_document")
        self.malformed_input_cases(project, editor, descriptor)

    def malformed_input_cases(self, project, editor, descriptor):
        basis = self.edit_basis(project, descriptor, "basis-malformed-input")
        good = self.edit_payload(basis, DESIRED)
        plain = json.dumps(good, ensure_ascii=False).encode("utf-8")
        variants = (
            ("duplicate_schema", plain.replace(b'"schema_version": 1,',
                                               b'"schema_version": 1, "schema_version": 1,', 1)),
            ("unknown_field", plain[:-1] + b', "unrecognized_control": true}'),
            ("trailing_document", plain + b' {}'),
            ("over_input_bound", plain + b' ' * (12 * 1024 * 1024 + 1)))
        before, disks = self.state(editor, project)
        for name, wire in variants:
            started = time.monotonic()
            process = subprocess.run(self.edit_command(project, descriptor["session_id"]),
                                     input=wire, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                     timeout=10.4)
            observation.require(process.returncode == 3 and
                                time.monotonic() - started <= 10.4 and
                                DESIRED.encode() not in process.stdout,
                                "strict_bounded_input_before_authorization_" + name)
            self.safe_log("malformed-" + name + ".stderr", process.stderr)
            result = json.loads(process.stdout)
            observation.require(result["outcome"] == "refused" and
                                result["reason"] == "invalid_request" and
                                result["application"] == "not_applied" and
                                result["history"] == "not_participated" and
                                result["before"] is None and result["after"] is None,
                                "malformed_no_source_or_history_authorization_" + name)
            self.unchanged_state(editor, project, before, disks, "malformed_" + name)
            self.case("caller_strict_malformed_" + name, exit_code=process.returncode,
                      source_surfaces="real_stdin_codec_no_native_boundary")



    def routing_edit(self):
        self.compile_window_probe()
        project, editor, descriptor, before, disks = self.caller_fixture("routing")
        basis = self.edit_basis(project, descriptor, "basis-routing")
        capture = self.screenshot(editor, "caller-routing-before.png")
        for name, session, script in (("wrong_session", secrets.token_hex(16), ROOT),
                                      ("unknown_script", descriptor["session_id"],
                                       "res://scripts/absent.gd"),
                                      ("built_in", descriptor["session_id"],
                                       "res://container.tscn::GDScript_x")):
            self.edit(project, basis, DESIRED, "caller_route_" + name,
                      "refused", 3, session=session, script=script, application="not_applied")
            if name == "wrong_session":
                self.cases[-1]["screenshot"] = capture
            self.unchanged_state(editor, project, before, disks, name)
        second = self.start_editor(project)
        second_descriptor = observation.wait_for(lambda: next((item for item in self.descriptors(project)
                                if item["session_id"] != descriptor["session_id"]), None),
                                                 "second_edit_session")
        self.edit(project, basis, DESIRED, "caller_route_ambiguous_no_session",
                  "refused", 3, application="not_applied")
        self.unchanged_state(editor, project, before, disks, "ambiguous")
        observation.require(second_descriptor["session_id"] != descriptor["session_id"],
                            "two_distinct_owned_sessions")
        self.close_editor(second)
        self.editors.remove(second)
        process, payload, started = self.held_edit(project, editor, basis, DESIRED,
                                                   "apply", "same-session-overlap",
                                                   session=descriptor["session_id"])
        other = "extends RefCounted\nfunc value() -> int:\n\treturn 31\n"
        rejected = self.edit(project, basis, other, "caller_same_session_busy_no_boundary",
                             "refused", 3, session=descriptor["session_id"],
                             reason="busy", application="not_applied")
        observation.require(rejected["expected"] is None and rejected["before"] is None and
                            rejected["after"] is None and rejected["history"] == "not_participated",
                            "busy_refusal_source_free_without_history_entry")
        self.unchanged_state(editor, project, before, disks, "overlap_busy")
        self.release_edit(editor, process, payload, started, "caller_overlap_first_changed",
                          "verified_changed", 0)
        after, now = self.changed_state(editor, project, before, disks, DESIRED, "overlap")
        self.edit(project, basis, DESIRED, "caller_overlap_old_basis_equal_stale",
                  "refused", 3, session=descriptor["session_id"], application="not_applied")
        self.unchanged_state(editor, project, after, now, "no_late_busy_apply")
        fresh = self.edit_basis(project, descriptor, "basis-after-overlap")
        self.edit(project, fresh, other, "caller_overlap_fresh_basis_changes",
                  "verified_changed", 0, session=descriptor["session_id"])
        second_before, second_disk = self.changed_state(editor, project, after, now, other, "fresh_after_busy")
        latest = self.edit_basis(project, descriptor, "basis-entered-overlap")
        self.native_action(editor, "native_edit_probe_entered")
        process, payload, started = self.held_edit(project, editor, latest, DESIRED,
                                                   "buffer_applied", "entered-native-overlap",
                                                   session=descriptor["session_id"])
        entered = self.native_action(editor, "native_edit_probe_result")["callback"]
        observation.require(entered.get("called") is True and
                            entered.get("slot_before") is True and
                            entered.get("slot_after") is True,
                            "actual_synchronous_native_buffer_entry_owns_shared_slot")
        middle, middle_disk = self.state(editor, project)
        observation.require(middle["subject"]["B"] == DESIRED and
                            middle_disk["subject"]["text"] == second_disk["subject"]["text"],
                            "native_buffer_entered_before_persistence")
        rejected = self.edit(project, latest, other, "caller_entered_native_stage_overlap_busy",
                             "refused", 3, session=descriptor["session_id"],
                             reason="busy", application="not_applied")
        observation.require(rejected["expected"] is None and rejected["before"] is None and
                            rejected["after"] is None,
                            "entered_stage_busy_refusal_has_no_source_evidence")
        self.unchanged_state(editor, project, middle, middle_disk, "entered_overlap_busy")
        self.release_edit(editor, process, payload, started,
                          "caller_entered_native_stage_first_completes", "verified_changed", 0)
        self.changed_state(editor, project, second_before, second_disk, DESIRED, "entered_overlap")
        self.routing_boundaries()

    def routing_boundaries(self):
        project, editor, descriptor, original, disks = self.caller_fixture("route-closed")
        basis = self.edit_basis(project, descriptor, "basis-route-closed")
        self.native_action(editor, "native_edit_human", mode="close")
        before = self.action(editor, "witness")
        observation.require(not before["subject"]["associated"] and
                            disks["subject"] == observation.disk_witness(project / "scripts/subject.gd"),
                            "closed_not_reopened_for_edit")
        closed = self.edit(project, basis, DESIRED, "caller_route_closed_not_force_open",
                           "refused", 3, session=descriptor["session_id"],
                           reason="closed_target", application="not_applied")
        observation.require(closed["stage"] == "selected" and
                            closed["resolved_target"]["session_id"] == descriptor["session_id"],
                            "closed_refusal_retains_authenticated_target_stage")
        observation.require(self.action(editor, "witness") == before and
                            disks["subject"] == observation.disk_witness(project / "scripts/subject.gd"),
                            "closed_refusal_no_document_or_disk_change")
        project, editor, descriptor, original, disks = self.caller_fixture("route-unavailable")
        basis = self.edit_basis(project, descriptor, "basis-route-unavailable")
        self.action(editor, "duplicate_script")
        before = self.action(editor, "witness")
        self.edit(project, basis, DESIRED, "caller_route_unknown_open_association",
                  "refused", 3, session=descriptor["session_id"],
                  reason="unavailable_observation", application="not_applied")
        observation.require(self.action(editor, "witness") == before and
                            disks["subject"] == observation.disk_witness(project / "scripts/subject.gd"),
                            "unknown_open_no_guessed_tab")
        project, editor, descriptor, original, disks = self.caller_fixture("route-denied")
        basis = self.edit_basis(project, descriptor, "basis-route-denied")
        path = project / "scripts/subject.gd"
        mode = stat.S_IMODE(path.stat().st_mode)
        path.chmod(0)
        try:
            result = self.edit(project, basis, DESIRED, "caller_route_denied_no_source",
                               "refused", 3, session=descriptor["session_id"],
                               reason="denied_access", application="not_applied")
            observation.require(result.get("expected") is None and
                                result.get("before") is None and result.get("after") is None,
                                "denied_no_source_derived_edit_evidence")
        finally:
            path.chmod(mode)
        observation.require(self.action(editor, "witness") == original and
                            observation.disk_witness(path)["text"] == disks["subject"]["text"],
                            "denied_request_no_source_history_change")
        self.routing_access_precedence()
        project, editor, descriptor, original, disks = self.caller_fixture("route-ended")
        basis = self.edit_basis(project, descriptor, "basis-route-ended")
        self.close_editor(editor)
        self.editors.remove(editor)
        self.edit(project, basis, DESIRED, "caller_route_ended_session_no_retarget",
                  "refused", 3, session=descriptor["session_id"],
                  reason="unavailable_observation", application="not_applied")
        observation.require(observation.disk_witness(project / "scripts/subject.gd") == disks["subject"],
                            "ended_session_no_hidden_write")
        self.restarted_basis_no_acquisition(project, basis, descriptor)
        project, editor, descriptor, original, disks = self.caller_fixture("route-unresponsive")
        basis = self.edit_basis(project, descriptor, "basis-route-unresponsive")
        os.kill(editor["process"].pid, signal.SIGSTOP)
        editor["suspended"] = True
        try:
            self.edit(project, basis, DESIRED, "caller_route_unresponsive_editor_deadline",
                      ("refused", "application_unknown"), (3, 4),
                      session=descriptor["session_id"], application="not_applied")
        finally:
            os.kill(editor["process"].pid, signal.SIGCONT)
            editor["suspended"] = False
        self.unchanged_state(editor, project, original, disks, "unresponsive_no_late_edit")
        project, editor, descriptor, original, disks = self.caller_fixture("route-wrong-project")
        basis = self.edit_basis(project, descriptor, "basis-route-wrong-project")
        other_project, other_editor, other_descriptor, other_before, other_disks = self.caller_fixture(
            "route-wrong-project-secondary")
        self.edit(other_project, basis, DESIRED, "caller_route_explicit_wrong_project",
                  "refused", 3, session=other_descriptor["session_id"], application="not_applied")
        self.unchanged_state(editor, project, original, disks, "wrong_project_original")
        self.unchanged_state(other_editor, other_project, other_before, other_disks,
                             "wrong_project_other")
        self.edit(project, basis, DESIRED, "caller_route_outside_project",
                  "refused", 3, session=descriptor["session_id"], script="res://../outside.gd",
                  reason="invalid_request", application="not_applied", correlated=False)
        self.unchanged_state(editor, project, original, disks, "outside_project")
        unsupported = self.fixture("caller-unsupported-native", controlled=True)
        shutil.rmtree(unsupported / "addons/godot_agent_kit/native")
        unsupported_editor = self.start_editor(unsupported)
        unsupported_descriptor = observation.wait_for(
            lambda: next(iter(self.descriptors(unsupported)), None), "unsupported_native_session")
        self.action(unsupported_editor, "prepare_subject")
        clean = self.edit_basis(unsupported, unsupported_descriptor, "basis-unsupported-native")
        challenge_socket, challenge = self.challenge(unsupported_descriptor)
        challenge_socket.close()
        observation.require(challenge["capabilities"]["edit_open_gdscript"] is False and
                            challenge["capabilities"]["open_gdscript"] is False and
                            challenge["native_api_revision"] == 0 and challenge["native_build_id"] == "",
                            "unsupported_native_never_advertised")
        before = self.action(unsupported_editor, "witness")
        disk = observation.disk_witness(unsupported / "scripts/subject.gd")
        self.edit(unsupported, clean, DESIRED, "caller_route_unsupported_engine",
                  "refused", 3, session=unsupported_descriptor["session_id"],
                  application="not_applied")
        observation.require(self.action(unsupported_editor, "witness") == before and
                            disk == observation.disk_witness(unsupported / "scripts/subject.gd"),
                            "unsupported_native_no_fake_mutation")

    def routing_access_precedence(self):
        for transition in ("dirty", "same_text", "close"):
            project, editor, descriptor, _, disks = self.caller_fixture(
                "denied-" + transition)
            basis = self.edit_basis(project, descriptor, "basis-denied-" + transition)
            self.native_action(editor, "native_edit_human", mode=transition)
            before = self.action(editor, "witness")
            path = project / "scripts/subject.gd"
            mode = stat.S_IMODE(path.stat().st_mode)
            path.chmod(0)
            try:
                result = self.edit(
                    project, basis, DESIRED, "caller_denial_precedes_" + transition,
                    "refused", 3, session=descriptor["session_id"],
                    reason="denied_access", application="not_applied")
                observation.require(
                    result["expected"] is None and result["before"] is None and
                    result["after"] is None and result["history"] == "not_participated",
                    "combined_denial_suppresses_source_without_effect_" + transition)
            finally:
                path.chmod(mode)
            disk = observation.disk_witness(path)
            observation.require(
                self.action(editor, "witness") == before and
                all(disk[key] == disks["subject"][key]
                    for key in ("text", "inode", "mtime_ns")),
                "combined_denial_preserves_human_and_disk_" + transition)

    def restarted_basis_no_acquisition(self, project, basis, prior_descriptor):
        editor = self.start_editor(project)
        descriptor = observation.wait_for(
            lambda: next((item for item in self.descriptors(project)
                          if item["session_id"] != prior_descriptor["session_id"]), None),
            "restarted_editor_lifetime")
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        before, disks = self.state(editor, project)
        event = editor["control"] / "event.json"
        event.unlink(missing_ok=True)
        self.native_action(editor, "native_edit_hold", stage="prepare")
        try:
            self.edit(project, basis, DESIRED, "caller_old_basis_new_implicit_lifetime",
                      "refused", 3, reason="session_changed", application="not_applied")
            observation.require(not event.exists(),
                                "replacement_lifetime_never_receives_source_preparation")
        finally:
            self.native_action(editor, "native_edit_release")
        observation.require(descriptor["session_id"] != prior_descriptor["session_id"],
                            "restarted_lifetime_is_real")
        self.unchanged_state(editor, project, before, disks, "replacement_lifetime")


    def validation_edit(self):
        self.compile_window_probe()
        invalid = "extends RefCounted\nvar = # ACTUAL_PROPOSAL_PARSE_FAILURE\n"
        for name, desired in (("syntax", invalid),
                              ("analyzer", 'extends RefCounted\nvar broken: int = "wrong"\n'),
                              ("dependency", 'extends "./native/missing.gd"\n'),
                              ("tool_effect", "@tool\nextends RefCounted\n"),
                              ("export_effect", "extends RefCounted\n@export var visible: int = 2\n"),
                              ("autoload_effect", 'extends "./native/cold/tool_initializer.gd"\n')):
            project, editor, descriptor, before, disks = self.caller_fixture("validation-" + name)
            basis = self.edit_basis(project, descriptor, "basis-validation-" + name)
            capture = self.screenshot(editor, "caller-validation-before.png") if name == "syntax" else None
            result = self.edit(project, basis, desired, "caller_preflight_" + name,
                               "refused", 3, session=descriptor["session_id"],
                               application="not_applied")
            if capture is not None:
                self.cases[-1]["screenshot"] = capture
            observation.require(result["history"] == "not_participated", "invalid_no_history_" + name)
            if name in ("syntax", "analyzer"):
                self.validated_source(result, desired, "preflight", "invalid",
                                      diagnostic_path=ROOT)
            self.unchanged_state(editor, project, before, disks, name)
            self.close_editor(editor)
        for name, replacement in (("invalid_dependency", "extends RefCounted\nvar =\n"),
                                  ("unconfined_dependency", None)):
            project, editor, descriptor, before, disks = self.caller_fixture("validation-" + name)
            dependency = project / "scripts/native/level_one.gd"
            if replacement is None:
                external = self.work / ("outside-" + name + ".gd")
                external.write_text("extends RefCounted\n")
                dependency.unlink()
                dependency.symlink_to(external)
            else:
                dependency.write_text(replacement)
            basis = self.edit_basis(project, descriptor, "basis-validation-" + name)
            desired = 'extends "./native/level_one.gd"\nconst NATIVE_DIRECT := 1\n'
            self.edit(project, basis, desired, "caller_preflight_" + name,
                      "refused", 3, session=descriptor["session_id"], application="not_applied")
            self.unchanged_state(editor, project, before, disks, name)
            self.close_editor(editor)
        project, editor, descriptor, before, disks = self.caller_fixture("validation-warning")
        self.native_action(editor, "native_promote_unused_warning")
        basis = self.edit_basis(project, descriptor, "basis-validation-warning")
        self.edit(project, basis,
                  "extends RefCounted\nfunc value() -> int:\n\tvar unused := 1\n\treturn 2\n",
                  "caller_preflight_analyzer_warning_as_error", "refused", 3,
                  session=descriptor["session_id"], application="not_applied")
        self.unchanged_state(editor, project, before, disks, "promoted_actual_analyzer_warning")
        self.close_editor(editor)
        for name, source, initial in (("empty", "", None),
                                      ("unicode", "extends RefCounted\n# Café λ 漢字\n", None),
                                      ("invalid_repair", DESIRED,
                                       'extends RefCounted\nvar broken: int = "wrong"\n'),
                                      ("json_controls", "extends RefCounted\n# controls " +
                                       "".join(chr(code) for code in range(1, 32)
                                               if code not in (10, 13)) +
                                       " END\n" + r"# literals \v \u0001 \\v END" + "\n", None),
                                      ("max_bytes", "extends RefCounted\n# " +
                                       "é" * 262133 + "x", None)):
            project, editor, descriptor, before, disks = self.caller_fixture(
                "validation-" + name, baseline=initial)
            basis = self.edit_basis(project, descriptor, "basis-validation-" + name)
            result = self.edit(project, basis, source, "caller_validation_" + name,
                               "verified_changed", 0, session=descriptor["session_id"])
            self.validated_source(result, source, "preflight", "valid")
            self.validated_source(result, source, "post_change", "valid")
            self.changed_state(editor, project, before, disks, source, name)
            self.close_editor(editor)
        observation.require(len(("extends RefCounted\n# " + "é" * 262133 + "x").encode()) ==
                            524288, "inclusive_maximal_utf8_boundary")

    def history_edit(self):
        self.compile_window_probe()
        original = self.revision(17)
        project, editor, descriptor, before, disks = self.caller_fixture(
            "history", prior=True, baseline=original)
        first = before["subject"]["B"]
        basis = self.edit_basis(project, descriptor, "basis-history")
        capture = self.screenshot(editor, "caller-history-before.png")
        self.edit(project, basis, DESIRED, "caller_history_one_complex_edit",
                  "verified_changed", 0, session=descriptor["session_id"])
        self.cases[-1]["screenshot"] = capture
        changed, changed_disks = self.changed_state(editor, project, before, disks, DESIRED, "history")
        self.edit(project, basis, DESIRED, "caller_history_stale_refused",
                  "refused", 3, session=descriptor["session_id"])
        fresh = self.edit_basis(project, descriptor, "basis-history-unchanged")
        self.edit(project, fresh, DESIRED, "caller_history_unchanged_no_entry",
                  "verified_unchanged", 0, session=descriptor["session_id"])
        self.unchanged_state(editor, project, changed, changed_disks, "refused_unchanged_history")
        for name, source in (("undo", first), ("redo", DESIRED)):
            step = self.native_action(editor, "native_edit_human", mode=name)["document"]
            observation.require(step["B"] == source and step["dirty"],
                                "caller_real_native_" + name + "_before_save")
            saved = self.native_action(editor, "native_edit_save")
            observed, actual_disk = self.state(editor, project)
            observation.require(saved["focused"] and saved["disk_matches"] and
                                observed["subject"]["B"] == observed["subject"]["R"] ==
                                actual_disk["subject"]["text"] == source and
                                not observed["subject"]["dirty"],
                                "caller_real_native_" + name + "_ordinary_save")
            observation.require(
                observed["other"]["dirty"] and actual_disk["other"] == disks["other"] and
                all(observed["other"][key] == before["other"][key]
                    for key in ("B", "version", "saved_version", "has_undo", "has_redo")),
                "caller_" + name + "_save_preserves_unrelated_unsaved_work")
            self.case("caller_history_" + name + "_ordinary_save",
                      source_surfaces="actual_undo_or_redo_then_native_save_d_r_b")
        previous = self.native_action(editor, "native_edit_human", mode="undo")["document"]
        older = self.native_action(editor, "native_edit_human", mode="undo")["document"]
        observation.require(previous["B"] == first and older["B"] == original and
                            older["has_redo"], "caller_earlier_history_order_reachable")
        self.case("caller_earlier_saved_human_history_reachable",
                  source_surfaces="actual_two_native_undo_steps")

    def interruption_edit(self):
        self.compile_window_probe()
        for stage in ("prepare", "apply", "buffer_applied", "resource_applied",
                      "content_persisted", "mtime_restored", "edited_cleared",
                      "verify:post_change", "recheck:post_change"):
            project, editor, descriptor, before, disks = self.caller_fixture("timeout-" + stage.replace(":", "-"))
            basis = self.edit_basis(project, descriptor, "basis-timeout-" + stage.replace(":", "-"))
            capture = self.screenshot(editor, "caller-interruption-before.png") if stage == "prepare" else None
            process, payload, started = self.held_edit(project, editor, basis, DESIRED, stage,
                                                       "timeout-" + stage,
                                                       session=descriptor["session_id"])
            # Wait for the actual supervised deadline, not a sleep or a synthetic reply.
            name = "caller_timeout_" + stage.replace(":", "-")
            result = self.complete_edit(
                process, payload, name,
                "refused" if stage == "prepare" else
                ("application_unknown", "applied_unverified"),
                4 if stage == "prepare" else (2, 4), started=started)
            if capture is not None:
                self.cases[-1]["screenshot"] = capture
            observation.require(result["outcome"] not in ("verified_changed", "verified_unchanged") and
                                (stage == "prepare" or result["application"] != "not_applied"),
                                "no_false_timeout_success_or_not_applied_" + name)
            if stage == "edited_cleared":
                finalization = result["finalization"]
                observation.require(
                    finalization is not None and
                    finalization["status"] == "partial_or_unknown" and
                    finalization["steps"]["resource_edited"] is True and
                    finalization["steps"]["saved_version"] is False and
                    finalization["before_current"] is not None and
                    finalization["after_current"] == finalization["before_current"],
                    "confirmed_edited_clear_survives_unknown_saved_tag")
            self.native_action(editor, "native_edit_release")
            after, now = self.state(editor, project)
            if stage in ("prepare", "apply"):
                self.unchanged_state(editor, project, before, disks, "no_late_boundary_" + name)
            else:
                observation.require(after["other"] == before["other"] and now["other"] == disks["other"] and
                                    after["subject"]["has_undo"] and
                                    (after["subject"]["B"] != before["subject"]["B"] or
                                     now["subject"]["text"] != disks["subject"]["text"]) and
                                    (stage not in ("verify:post_change", "recheck:post_change") or
                                     now["subject"]["text"] == DESIRED),
                                    "retained_actual_partial_history_and_human_work_" + name)
            self.case("caller_deadline_stage_" + stage.replace(":", "-"),
                      source_surfaces="real_stage_barrier_deadline_and_independent_after_state")
        project, editor, descriptor, before, disks = self.caller_fixture("stdin-no-eof")
        started = time.monotonic()
        process = self.edit_process(project, None, session=descriptor["session_id"])
        process.stdin.write(b'{"schema_version":1,"request_id":"incomplete-before-authorization",')
        process.stdin.flush()
        try:
            process.wait(timeout=10.4)
            stdout, stderr = process.communicate(timeout=1)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=2)
        elapsed = time.monotonic() - started
        result = json.loads(stdout)
        observation.require(elapsed <= 10.0 and process.returncode == 4 and
                            stdout.endswith(b"\n") and stdout.count(b"\n") == 1 and
                            result["application"] == "not_applied" and
                            all(secret not in stdout for secret in self.secrets),
                            "stdin_no_eof_supervised_before_authorization")
        result_review = review_edit_result(result)
        observation.json_file(self.artifacts / "caller-stdin-no-eof.json", result)
        self.safe_log("caller-stdin-no-eof.stderr", stderr)
        self.unchanged_state(editor, project, before, disks, "stdin_no_eof")
        self.case("caller_stalled_stdin_no_authorize_no_late_apply",
                  source_surfaces="actual_supervised_input_deadline_and_disk_history",
                  elapsed_seconds=elapsed, evidence="caller-stdin-no-eof.json", result_review=result_review)
        project, editor, descriptor, before, disks = self.caller_fixture("disconnect")
        basis = self.edit_basis(project, descriptor, "basis-disconnect")
        process, payload, started = self.held_edit(project, editor, basis, DESIRED,
                                                   "apply", "disconnect_after_authorize",
                                                   session=descriptor["session_id"])
        self.action(editor, "disable")
        result = self.complete_edit(process, payload, "caller_disconnect_after_authorize",
                                    "application_unknown", 4, started=started)
        observation.require(result["application"] == "unknown", "lost_authorization_ack_is_unknown")
        disconnected, disconnected_disk = self.state(editor, project)
        observation.require(
            disconnected["subject"] == before["subject"] and
            disconnected["other"] == before["other"] and disconnected_disk == disks and
            disconnected["current_script"] == before["current_script"] and
            disconnected["open_paths"] == before["open_paths"],
            "disconnect_preserves_documents_history_and_disk_without_late_apply")
        self.partial_persistence_cases()

    def partial_persistence_cases(self):
        for fault in ("fail_pwrite", "fail_futimens", "mismatch_mtime", "close_fd_before_restore"):
            project, editor, descriptor, before, disks = self.caller_fixture("fault-" + fault, faults=True)
            basis = self.edit_basis(project, descriptor, "basis-fault-" + fault)
            stage = "resource_applied" if fault == "fail_pwrite" else "content_persisted"
            process, payload, started = self.held_edit(project, editor, basis, DESIRED, stage,
                                                       "fault-" + fault, session=descriptor["session_id"])
            injected = self.native_action(editor, "native_edit_fault", request_id=payload["request_id"],
                                          fault=fault)["result"]
            observation.require(injected["status"] == "ready", "actual_native_fixture_fault_" + fault)
            result = self.release_edit(editor, process, payload, started, "caller_partial_" + fault,
                                       "applied_unverified", 2)
            observation.require(result["application"] in ("applied", "partly_applied") and
                                result["progress"]["finalization"]["state"] != "completed",
                                "fault_retains_known_application_without_finalization_" + fault)
            if fault in ("fail_futimens", "mismatch_mtime", "close_fd_before_restore"):
                persistence = result["persistence"]
                observation.require(persistence is not None and
                                    persistence["restore_attempted"] is True and
                                    persistence["restored"] is False and
                                    persistence["original_mtime"] is not None,
                                    "real_content_success_metadata_failure_not_saved_" + fault)
            after, now = self.state(editor, project)
            observation.require(after["subject"]["has_undo"] and
                                after["other"] == before["other"] and now["other"] == disks["other"] and
                                (fault == "fail_pwrite" or now["subject"]["text"] == DESIRED) and
                                (fault not in ("fail_futimens", "mismatch_mtime", "close_fd_before_restore") or
                                 after["subject"]["dirty"]),
                                "known_partial_content_no_false_saved_tag_" + fault)

    def post_change_validation_edit(self):
        for name, stage, transition in (("newer_human_before_tag", "edited_cleared", "dirty"),
                                        ("newer_human_after_persist", "content_persisted", "dirty"),
                                        ("invalid_newer_human", "verify:post_change", "invalid"),
                                        ("invalid_selected_warning", "verify:post_change", "warning"),
                                        ("unavailable_selected_context", "verify:post_change", "context")):
            project, editor, descriptor, before, disks = self.caller_fixture("post-" + name)
            desired = ("extends RefCounted\nfunc value() -> int:\n"
                       "\tvar unused := 1\n\treturn 23\n") if transition == "warning" else DESIRED
            basis = self.edit_basis(project, descriptor, "basis-post-" + name)
            process, payload, started = self.held_edit(project, editor, basis, desired, stage, name,
                                                       session=descriptor["session_id"])
            if transition == "warning":
                promoted = self.native_action(editor, "native_promote_unused_warning")
                observation.require(promoted["level"] == 2, "actual_selected_warning_promoted")
            elif transition == "context":
                context = self.native_action(editor, "native_edit_unsupported_warning_context")
                observation.require(context["type"] == 4, "actual_selected_unsupported_context")
            else:
                self.native_action(editor, "native_edit_human", mode=transition)
            human, _ = self.state(editor, project)
            result = self.release_edit(editor, process, payload, started,
                                       "caller_post_change_" + name, "applied_unverified", 2)
            after, now = self.state(editor, project)
            observation.require(result["history"] != "not_participated" and
                                after["other"] == before["other"] and now["other"] == disks["other"],
                                "post_change_never_false_refusal_or_unrelated_edit_" + name)
            if transition in ("dirty", "invalid"):
                observation.require(after["subject"]["B"] == human["subject"]["B"] and
                                    (transition != "invalid" or "var =" in after["subject"]["B"]),
                                    "newer_human_text_never_overwritten_" + name)
            else:
                observation.require(after["subject"]["B"] == desired and
                                    now["subject"]["text"] == desired,
                                    "invalid_or_unavailable_context_keeps_applied_source_" + name)
            if transition == "warning":
                # Changed warning context can disqualify the verdict, but must
                # not erase the actual post-change, source-attributed error.
                observation.require(any(
                    item["purpose"] == "post_change" and
                    item["status"] in ("invalid", "unavailable") and
                    item["input"]["sha256"] == self.stock_sha(desired) and
                    item["cleanup_confirmed"] and
                    all(source["diagnostics_completed"] and source["symbols_completed"]
                        for source in item["sources"]) and
                    any(diagnostic["origin"] == "root" and diagnostic["path"] == ROOT and
                        diagnostic["source"] is not None and
                        diagnostic["source"]["sha256"] == self.stock_sha(desired)
                        for diagnostic in item["diagnostics"])
                    for item in result["validation"]),
                    "fresh_post_change_warning_error_retained_with_context_qualification")
            elif transition == "context":
                observation.require(not any(item["purpose"] == "post_change" and
                                            item["status"] == "valid" for item in result["validation"]),
                                    "actual_unavailable_context_cannot_claim_validation")
                observation.require(result["reason"] == "unavailable_observation" and
                                    result["after"] is not None and
                                    result["after"]["sources"]["B"]["source_sha256"] ==
                                    self.stock_sha(desired),
                                    "unavailable_post_context_retains_independent_after_state")
