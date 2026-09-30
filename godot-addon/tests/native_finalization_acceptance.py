"""Target-bound stock GDExtension saved-state behavior in disposable GUI fixtures."""
import errno
import secrets
import shutil
import subprocess
from pathlib import Path

import run_observation as observation

ROOT = "res://scripts/subject.gd"
DESIRED = 'extends RefCounted\nfunc value() -> int:\n\treturn 23\n'
STAGES = ("prepared", "buffer_applied", "resource_applied", "content_persisted",
          "mtime_restored", "edited_cleared", "saved_tagged")
CONSUMER = Path(__file__).parent / "fixtures/script_edit/scripts/native/consumer.gd"


class NativeFinalizationMixin:
    def edit_fixture(self, name, *, prior=False, faults=False, baseline_source=None):
        project = self.fixture("finalization-" + name)
        if baseline_source is not None:
            (project / "scripts/subject.gd").write_text(baseline_source)
        if faults:
            observation.require(self.args.native_fault_addon is not None,
                                "separate_fault_instrumented_artifact_required")
            destination = project / "addons/godot_agent_kit/native"
            shutil.copytree(self.args.native_fault_addon, destination, dirs_exist_ok=True)
        self.installed_native(project, fixture_only=faults)
        editor = self.start_editor(project, configured=False)
        info = self.native_action(editor, "native_info")
        observation.require(info.get("edit_installed") is True and
                            info.get("open_installed") is True and
                            info.get("api_revision") == 2 and
                            info.get("build_id") == self.expected_native_build_id,
                            "actual_stock_native_edit_api_" + name)
        session = secrets.token_hex(16)
        observation.require(self.native_action(editor, "native_configure",
                                               session_id=session).get("configured") is True,
                            "actual_native_attempt_session_" + name)
        self.stock_sessions[editor["control"]] = session
        self.action(editor, "prepare_subject")
        if prior:
            seeded = self.native_action(editor, "native_edit_human", mode="seed_prior")
            observation.require(seeded["document"]["dirty"] and
                                seeded["document"]["has_undo"],
                                "real_earlier_native_history_" + name)
            saved = self.native_action(editor, "native_edit_save")
            observation.require(saved["disk_matches"] and
                                saved["before_disk_changed"] and
                                not saved["document"]["dirty"] and
                                saved["document"]["has_undo"],
                                "actually_delivered_ordinary_prior_save_" + name)
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.native_action(editor, "native_idle")
        before, disks = self.snapshot(editor, project)
        observation.require(not before["subject"]["dirty"] and
                            before["subject"]["B"] == before["subject"]["R"] ==
                            disks["subject"]["text"] and before["other"]["dirty"] and
                            before["current_script"] == "res://scripts/other.gd",
                            "native_clean_nonselected_target_other_dirty_" + name)
        return editor, project, session, before, disks

    def edit_prepare(self, editor, project, session, source=DESIRED, **kwargs):
        before, disks = self.snapshot(editor, project)
        request_id = secrets.token_hex(16)
        result = self.native_action(editor, "native_edit_prepare", request_id=request_id,
                                    session_id=session, expected=disks["subject"]["text"],
                                    desired=source, **kwargs)["result"]
        return request_id, result, before, disks

    def edit_step(self, editor, request_id, stage, expected):
        result = self.native_action(editor, "native_edit_advance", request_id=request_id,
                                    stage=stage)["result"]
        observation.require(isinstance(result, dict) and result.get("status") == expected,
                            "native_real_step_" + stage + "_" + expected)
        return result

    def assert_native_isolation(self, name, editor, project, other, other_disk):
        after, disks = self.snapshot(editor, project)
        observation.require(after["other"] == other and disks["other"] == other_disk and
                            after["current_script"] == "res://scripts/other.gd",
                            "native_did_not_change_unrelated_history_focus_disk_" + name)
        return after, disks

    def native_success(self, name="native_t0_clean_persisted", *, history=False, fault=None,
                       desired=DESIRED, baseline_source=None):
        editor, project, session, before, disks_before = self.edit_fixture(
            name, prior=history, faults=fault is not None, baseline_source=baseline_source)
        capture = self.screenshot(editor, name + "-before.png")
        self.stock_validate(editor, project, name + "_preflight_exact_proposed", desired, "valid")
        before, disks_before = self.snapshot(editor, project)
        t0 = disks_before["subject"]["mtime_ns"]
        before_edited = self.native_action(editor, "native_edit_inspect")["resource_edited"]
        observation.require(before_edited is False,
                            "public_resource_edited_flag_clean_before_native_effect")
        request_id, prepared, _, _ = self.edit_prepare(editor, project, session, source=desired)
        observation.require(prepared.get("status") == "prepared" and
                            prepared.get("stage") == "prepared" and
                            prepared.get("facts", {}).get("tagged") is not True,
                            "native_prepared_without_effect_" + name)
        for stage, next_stage in zip(STAGES, STAGES[1:]):
            if stage == "resource_applied" and fault:
                injected = self.native_action(editor, "native_edit_fault", request_id=request_id,
                                              fault=fault).get("result")
                observation.require(isinstance(injected, dict) and
                                    injected.get("status") == "ready",
                                    "native_short_write_fault_armed")
            result = self.edit_step(editor, request_id, stage,
                                    "complete" if next_stage == "saved_tagged" else "ready")
            observation.require(result.get("stage") == next_stage,
                                "native_actual_stage_order_" + next_stage)
            if next_stage == "content_persisted":
                middle, middle_disks = self.snapshot(editor, project)
                observation.require(middle_disks["subject"]["text"] == desired and
                                    middle_disks["subject"]["mtime_ns"] != t0 and
                                    middle["subject"]["B"] == middle["subject"]["R"] == desired and
                                    middle["subject"]["dirty"],
                                    "real_content_before_distinct_t0_restoration_" + name)
            elif next_stage == "mtime_restored":
                restored, restored_disks = self.snapshot(editor, project)
                observation.require(restored_disks["subject"]["mtime_ns"] == t0 and
                                    restored["subject"]["dirty"],
                                    "actual_t0_restored_before_tag_" + name)
        facts = result.get("facts", {})
        observation.require(facts.get("write_started") is True and
                            int(facts["written_bytes"]) == len(desired.encode("utf-8")) and
                            int(facts["write_calls"]) >= (1 if desired else 0) and
                            facts.get("truncate_done") is True and
                            facts.get("fsync_done") is True and
                            facts.get("pread_done") is True and
                            facts.get("futimens_called") is True and
                            facts.get("mtime_restored") is True and
                            facts.get("edited_cleared") is True and
                            facts.get("tagged") is True,
                            "complete_separate_content_metadata_and_native_saved_receipts_" + name)
        def as_ns(stamp):
            return int(stamp["seconds"]) * 1_000_000_000 + int(stamp["nanoseconds"])
        observation.require(as_ns(facts["t0"]) == t0 and
                            as_ns(facts["after_restore"]) == t0 and
                            as_ns(facts["before_restore"]) != t0,
                            "independent_exact_original_t0_and_same_attempt_metadata_readback")
        if fault == "short_write":
            observation.require(int(facts["write_calls"]) >= 2,
                                "real_two_pwrite_calls_after_short_write")
        after, disks_after = self.assert_native_isolation(name, editor, project,
                                                           before["other"], disks_before["other"])
        subject = after["subject"]
        observation.require(disks_after["subject"]["text"] == desired and
                            disks_after["subject"]["sha256"] == self.stock_sha(desired) and
                            disks_after["subject"]["mtime_ns"] == t0 and
                            disks_after["subject"]["device"] == disks_before["subject"]["device"] and
                            disks_after["subject"]["inode"] == disks_before["subject"]["inode"] and
                            subject["B"] == subject["R"] == desired and
                            not subject["dirty"] and
                            subject["version"] == subject["saved_version"] and
                            (subject["script_id"], subject["editor_id"], subject["buffer_id"]) ==
                            (before["subject"]["script_id"], before["subject"]["editor_id"],
                             before["subject"]["buffer_id"]) and
                            subject["has_undo"],
                            "independent_exact_saved_d_r_b_identity_versions_and_history_" + name)
        after_edited = self.native_action(editor, "native_edit_inspect")["resource_edited"]
        observation.require(after_edited is False,
                            "independent_public_resource_edited_false_after_t0_and_tag")
        self.stock_validate(editor, project, name + "_post_change_fresh_actual", None,
                            "valid", purpose="post_change")
        self.case(name, mapping="T003/native-integration §§2–3; research §18 T1",
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after), facts=facts,
                  screenshot=capture,
                  source_surfaces="actual_independent_d_r_b_dirty_versions_and_t0")
        self.native_action(editor, "native_edit_cancel", request_id=request_id)
        if history:
            self.native_history(editor, project, before["subject"]["B"], desired)
        elif fault is None and desired == DESIRED:
            self.native_continuation(editor, project)
        return editor, project

    def native_post_change_failure(self):
        desired = ('extends RefCounted\nfunc value() -> int:\n'
                   '\tvar unused_value := 1\n\treturn 23\n')
        editor, project = self.native_success("native_post_change_known_application",
                                               desired=desired)
        self.native_action(editor, "native_promote_unused_warning")
        invalid = self.stock_validate(
            editor, project, "native_post_change_warning_invalid", None, "invalid",
            purpose="post_change", warnings={"enable": True, "levels": {"unused_variable": 2}})
        unavailable = self.stock_validate(
            editor, project, "native_post_change_deadline_unavailable", None, "unavailable",
            purpose="post_change", interruption="deadline")
        current, disks = self.snapshot(editor, project)
        observation.require(invalid["context_sha256"] == unavailable["context_sha256"] and
                            current["subject"]["B"] == current["subject"]["R"] ==
                            disks["subject"]["text"] == desired and
                            current["subject"]["has_undo"] and current["other"]["dirty"],
                            "post_change_failures_preserve_observed_native_application_and_human_work")
        self.case("native_known_application_survives_failed_post_validation",
                  validation_statuses=[invalid["status"], unavailable["status"]],
                  after=self.source_free_state(current, disks),
                  source_surfaces="actual_native_application_then_independent_invalid_unavailable_no_rollback")

    def native_history(self, editor, project, prior, desired):
        for index, (operation, source) in enumerate((("undo", prior), ("redo", desired))):
            operation_witness = self.native_action(editor, "native_edit_human", mode=operation)
            observation.require(operation_witness["document"]["B"] == source and
                                operation_witness["document"]["dirty"],
                                "real_native_" + operation + "_history")
            ordinary = self.native_action(editor, "native_edit_save")
            witness, disks = self.snapshot(editor, project)
            observation.require(ordinary["focused"] and ordinary["before_disk_changed"] and
                                ordinary["disk_matches"] and
                                disks["subject"]["text"] == source and
                                witness["subject"]["B"] == witness["subject"]["R"] == source and
                                not witness["subject"]["dirty"] and
                                witness["subject"]["version"] == witness["subject"]["saved_version"],
                                "native_" + operation + "_ordinary_save_actual_d_r_b")
            self.case("native_" + operation + "_ordinary_save", operation=operation,
                      document=self.source_free_state(witness, disks),
                      source_surfaces="real_undo_redo_then_delivered_human_save")
        back_to_prior = self.native_action(editor, "native_edit_human", mode="undo")["document"]
        older = self.native_action(editor, "native_edit_human", mode="undo")["document"]
        observation.require(back_to_prior["B"] == prior and older["B"] != prior and
                            older["has_redo"], "native_earlier_history_not_destroyed")
        self.case("native_earlier_history_reachable", prior_sha256=self.stock_sha(prior),
                  earlier_sha256=self.stock_sha(older["B"]),
                  source_surfaces="actual_two_native_undo_operations")

    def native_runtime(self, project, value):
        consumer = project / "scripts/native/consumer.gd"
        shutil.copy2(CONSUMER, consumer)
        try:
            child = subprocess.run([str(self.args.godot), "--headless", "--path", str(project),
                                    "--script", "res://scripts/native/consumer.gd"],
                                   stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                   timeout=12, check=False)
        except subprocess.TimeoutExpired as error:
            raise observation.Failure("fresh_stock_runtime_deadline") from error
        marker = ("NATIVE_RUNTIME_VALUE=" + str(value)).encode()
        observation.require(child.returncode == 0 and child.stdout.count(marker) == 1 and
                            child.stdout.count(b"NATIVE_RUNTIME_VALUE=") == 1,
                            "fresh_official_stock_consumer_loaded_persisted_value_" + str(value))
        self.case("native_fresh_runtime_value_" + str(value),
                  value=value, consumer_sha256=observation.digest(consumer),
                  source_surfaces="actual_fresh_runtime_target_and_preloaded_consumer")

    def native_continuation(self, editor, project):
        scan = self.native_action(editor, "native_edit_scan")
        witness, disks = self.snapshot(editor, project)
        observation.require(scan["settled"] and scan["events"] > scan["before_events"] and
                            witness["subject"]["R"] == witness["subject"]["B"] == DESIRED and
                            disks["subject"]["text"] == DESIRED and
                            not witness["subject"]["dirty"],
                            "explicit_editor_filesystem_rescan_kept_bound_source")
        self.case("native_explicit_filesystem_rescan", event_count=scan["events"],
                  after=self.source_free_state(witness, disks),
                  source_surfaces="actual_editor_filesystem_signal_and_independent_d_r_b")
        old = witness["subject"]
        reopened = self.native_action(editor, "native_edit_human", mode="close_reopen")["document"]
        observation.require(reopened["associated"] and reopened["B"] == reopened["R"] == DESIRED and
                            not reopened["dirty"] and reopened["version"] == reopened["saved_version"] and
                            (reopened["editor_id"], reopened["buffer_id"]) !=
                            (old["editor_id"], old["buffer_id"]),
                            "real_clean_close_reopen_new_document_identity")
        self.native_runtime(project, 23)
        human = self.native_action(editor, "native_edit_human", mode="later_value")["document"]
        observation.require(human["dirty"] and human["version"] != human["saved_version"],
                            "later_human_edit_remains_dirty_after_tag")
        saved = self.native_action(editor, "native_edit_save")
        after, after_disk = self.snapshot(editor, project)
        expected = 'extends RefCounted\nfunc value() -> int:\n\treturn 29\n'
        observation.require(saved["focused"] and saved["before_disk_changed"] and
                            saved["disk_matches"] and after_disk["subject"]["text"] == expected and
                            after["subject"]["B"] == after["subject"]["R"] == expected and
                            not after["subject"]["dirty"] and
                            after["subject"]["version"] == after["subject"]["saved_version"],
                            "later_delivered_human_save_no_false_external_dialog")
        self.case("native_later_human_save_no_false_dialog",
                  saved=self.source_free_state(after, after_disk),
                  source_surfaces="actual_gui_save_input_disk_change_and_clean_d_r_b")
        self.native_runtime(project, 29)
        self.close_editor(editor)
        fresh = self.start_editor(project, configured=False)
        self.action(fresh, "prepare_other")
        self.action(fresh, "prepare_subject")
        current, disk = self.snapshot(fresh, project)
        observation.require(current["subject"]["B"] == current["subject"]["R"] == expected and
                            disk["subject"]["text"] == expected and
                            current["subject"]["version"] == current["subject"]["saved_version"] and
                            not current["subject"]["dirty"],
                            "fresh_editor_process_reparses_persisted_revision")
        capture = self.screenshot(fresh, "fresh-editor-reparsed.png")
        self.case("native_fresh_editor_reparse", window=capture,
                  fresh=self.source_free_state(current, disk),
                  source_surfaces="actual_separate_editor_reparse_and_owned_window")

    def native_refusal(self, name, *, at="prepared", mutation=None, fault=None,
                       expect_applied=False, prior=False):
        editor, project, session, baseline, original_disks = self.edit_fixture(
            name, prior=prior, faults=fault is not None)
        request_id, prepared, _, _ = self.edit_prepare(editor, project, session)
        observation.require(prepared.get("status") == "prepared", "native_case_prepared_" + name)
        stage = "prepared"
        while stage != at:
            result = self.edit_step(editor, request_id, stage, "ready")
            stage = result["stage"]
        before, disks_before = self.snapshot(editor, project)
        observation.require(before["other"] == baseline["other"] and
                            disks_before["other"] == original_disks["other"],
                            "native_prefix_preserves_unrelated_work_" + name)
        displaced = None
        replacement = None
        displaced_witness = None
        if mutation in ("dirty", "same_text", "saved_version", "close_reopen", "profile"):
            self.native_action(editor, "native_edit_human", mode=mutation)
        elif mutation in ("leaf", "parent"):
            target = project / "scripts/subject.gd"
            if mutation == "leaf":
                displaced = project / "scripts/held-subject.gd"
                target.rename(displaced)
                shutil.copy2(displaced, target)
                replacement = observation.disk_witness(target)
            else:
                displaced = project / "old-scripts"
                (project / "scripts").rename(displaced)
                (project / "scripts").mkdir()
                shutil.copy2(displaced / "subject.gd", target)
                shutil.copy2(displaced / "other.gd", project / "scripts/other.gd")
                replacement = observation.disk_witness(target)
            displaced_witness = observation.disk_witness(
                displaced if mutation == "leaf" else displaced / "subject.gd")
        if fault:
            injected = self.native_action(editor, "native_edit_fault", request_id=request_id,
                                          fault=fault).get("result")
            observation.require(isinstance(injected, dict) and
                                injected.get("status") == "ready",
                                "real_fixture_fault_armed_" + name)
        guard_before, guard_disks = self.snapshot(editor, project)
        result = self.native_action(editor, "native_edit_advance", request_id=request_id,
                                    stage=stage)["result"]
        facts = result.get("facts", {})
        observation.require(result.get("status") in ("refused", "partial") and
                            facts.get("tagged") is not True and
                            (stage in ("mtime_restored", "edited_cleared") or
                             facts.get("mtime_restored") is not True),
                            "no_new_saved_tag_or_t0_restoration_on_failure_" + name)
        if expect_applied:
            observation.require(result["status"] == "partial" and
                                (facts.get("write_started") is True or stage in
                                 ("buffer_applied", "resource_applied", "content_persisted",
                                  "mtime_restored", "edited_cleared")),
                                "known_partial_never_reclassified_not_applied_" + name)
        else:
            observation.require(int(facts.get("written_bytes", "0")) == 0,
                                "prewrite_refusal_did_not_persist_" + name)
        if fault == "close_fd_before_restore":
            observation.require(facts.get("futimens_called") is True and
                                int(facts["futimens_errno"]) == errno.EBADF,
                                "real_futimens_ebadf_distinct_from_injected_error")
        elif fault == "fail_futimens":
            observation.require(facts.get("futimens_called") is True and
                                int(facts["futimens_errno"]) == errno.EIO,
                                "injected_metadata_error_distinct_from_actual_ebadf")
        elif fault == "mismatch_mtime":
            observation.require(facts.get("futimens_called") is True and
                                int(facts["futimens_errno"]) == 0 and
                                facts.get("after_restore") != facts.get("t0") and
                                facts.get("mtime_restored") is False,
                                "successful_real_futimens_return_cannot_hide_wrong_t0_readback")
        if fault:
            observation.require(facts.get("injected_fault") == fault,
                                "fixture_only_fault_provenance_" + name)
        after, disks_after = self.snapshot(editor, project)
        if replacement is not None:
            observation.require(disks_after["subject"] == replacement and
                                observation.disk_witness(
                                    displaced if mutation == "leaf" else
                                    displaced / "subject.gd") == displaced_witness and
                                displaced_witness["text"] == disks_before["subject"]["text"],
                                "no_redirected_write_or_retimestamp_" + name)
        else:
            current = disks_after["subject"]["text"]
            written = int(facts.get("written_bytes", "0"))
            if written:
                observation.require(current.startswith(DESIRED[:written]),
                                    "actual_partial_write_prefix_" + name)
            elif at in ("content_persisted", "mtime_restored", "edited_cleared"):
                observation.require(current == DESIRED,
                                    "already_persisted_actual_partial_disk_" + name)
            else:
                observation.require(current == original_disks["subject"]["text"],
                                    "no_content_write_on_guard_refusal_" + name)
        observation.require(after["other"] == guard_before["other"] and
                            disks_after["other"] == guard_disks["other"] and
                            after["current_script"] == guard_before["current_script"],
                            "failed_native_attempt_preserves_unrelated_work_" + name)
        if at == "content_persisted" and mutation is None:
            observation.require(after["subject"].get("associated") and
                                after["subject"]["R"] == after["subject"]["B"] == DESIRED and
                                after["subject"]["dirty"] and
                                after["subject"]["version"] !=
                                after["subject"]["saved_version"],
                                "known_content_applied_without_false_saved_tag_" + name)
        if mutation == "close_reopen":
            original_ids = (before["subject"]["editor_id"], before["subject"]["buffer_id"])
            replacement_ids = (after["subject"]["editor_id"], after["subject"]["buffer_id"])
            observation.require(after["subject"]["associated"] and
                                replacement_ids != original_ids and
                                after["subject"]["B"] == after["subject"]["R"] ==
                                disks_after["subject"]["text"] and
                                result.get("reason") == "document_association_changed" and
                                facts.get("tag_attempted") is False and
                                facts.get("tagged") is False,
                                "closed_original_identity_not_tagged_new_editor_has_own_saved_version")
        elif mutation != "saved_version":
            observation.require(not (after["subject"].get("associated", False) and
                                     (after["subject"]["editor_id"], after["subject"]["buffer_id"]) ==
                                     (before["subject"]["editor_id"], before["subject"]["buffer_id"]) and
                                     after["subject"]["B"] == DESIRED and
                                     after["subject"]["saved_version"] ==
                                     after["subject"]["version"]),
                                "original_candidate_never_claimed_clean_tag_on_failure_" + name)
        self.case(name, mapping="T003/native-integration §§2–3 failure boundaries",
                  stage=stage, native_status=result["status"], reason=result.get("reason"),
                  facts=facts, before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="independent_partial_d_r_b_dirty_current_saved_and_namespace")
        self.native_action(editor, "native_edit_cancel", request_id=request_id)

    def native_representation_refusal(self):
        for name, source, trim, reason in (
                ("native_crlf_refused_before_history", DESIRED.replace("\n", "\r\n"),
                 False, "unsupported_source"),
                ("native_bom_refused_before_history", "\ufeff" + DESIRED,
                 False, "unsupported_source"),
                ("native_embedded_bom_refused_before_history", DESIRED + "# \ufeff\n",
                 False, "unsupported_source"),
                ("native_newline_only_trim_refused_before_history", "\n",
                 True, "save_would_reformat")):
            editor, project, session, _, _ = self.edit_fixture(name)
            if trim:
                observation.require(self.native_action(
                    editor, "native_edit_trim_final_newlines")["enabled"] is True,
                    "real_editor_trim_final_newlines_enabled")
            before, disks_before = self.snapshot(editor, project)
            request_id, result, _, _ = self.edit_prepare(editor, project, session, source=source)
            after, disks_after = self.snapshot(editor, project)
            observation.require(result.get("status") == "refused" and
                                result.get("reason") == reason and
                                after == before and disks_after == disks_before and
                                not result.get("facts", {}).get("tag_attempted", False),
                                "unsupported_exact_source_refused_without_history_" + name)
            self.case(name, reason=reason,
                      before=self.source_free_state(before, disks_before),
                      after=self.source_free_state(after, disks_after),
                      source_surfaces="real_unchanged_buffer_versions_history_resource_and_disk")
            self.native_action(editor, "native_edit_cancel", request_id=request_id)

    def native_editor_effect_admission(self):
        name = "native_cold_tool_editor_effect_refused"
        editor, project, session, _, _ = self.edit_fixture(name)
        observation.require(self.native_action(editor, "native_info")["tool_cached"] is False,
                            "native_cold_dependency_absent_before_live_admission")
        before, disks_before = self.snapshot(editor, project)
        risky = 'extends "./native/cold/tool_initializer.gd"\nfunc value() -> int:\n\treturn 23\n'
        request_id, result, _, _ = self.edit_prepare(editor, project, session, source=risky)
        settled = self.native_action(editor, "native_edit_settle")
        after, disks_after = self.snapshot(editor, project)
        observation.require(result.get("status") == "refused" and
                            result.get("reason") == "unsafe_editor_effects" and
                            not result.get("facts", {}).get("tag_attempted", False) and
                            not settled["initializer_called"] and not settled["constructor_called"] and
                            not settled["tool_cached"] and before == after and disks_before == disks_after,
                            "cold_tool_source_refused_before_live_editor_continuation_or_history")
        self.case(name, reason=result["reason"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="real_cold_dependency_editor_and_initializer_sentinel")
        self.native_action(editor, "native_edit_cancel", request_id=request_id)
        for label, separator in (("newline", "\n"), ("comment", " # continued\n")):
            source = ('extends RefCounted\nconst REF = (preload' + separator +
                      '("res://scripts/native/level_one.gd"))\n')
            _, refused, _, _ = self.edit_prepare(editor, project, session, source=source)
            current, current_disks = self.snapshot(editor, project)
            observation.require(refused.get("status") == "refused" and
                                refused.get("reason") == "unsafe_editor_effects" and
                                current == before and current_disks == disks_before,
                                "continued_loader_refused_without_history_" + label)
            self.case("native_continued_loader_refused_" + label,
                      reason=refused["reason"],
                      source_surfaces="independent_unchanged_d_r_b_versions_history")

        name = "native_ordinary_editor_cold_initializer_hazard"
        editor, project, _session, _, _ = self.edit_fixture(name)
        observation.require(self.native_action(editor, "native_info")["tool_cached"] is False,
                            "actual_cold_dependency_before_ordinary_human_change")
        observation.require(not (editor["control"] / "excluded_initializer_was_called").exists(),
                            "cold_initializer_not_preexisting_before_human_change")
        human = self.native_action(editor, "native_edit_human", mode="cold_inheritance")
        observation.require(human["document"]["B"] == risky,
                            "actual_code_edit_changed_to_cold_inheritance")
        continuation = self.native_action(
            editor, "native_edit_settle", wait_for_initializer=True)
        observation.require(continuation["initializer_called"] and
                            (editor["control"] / "excluded_initializer_was_called").is_file(),
                            "ordinary_editor_queued_export_update_executed_cold_tool_initializer")
        self.case(name, marker="excluded_initializer_was_called",
                  source_surfaces="actual_text_changed_ordinary_validation_queued_tool_effect")

    def native_sync_remove_race(self):
        name = "native_synchronous_human_change_during_removal"
        editor, project, session, before, disks_before = self.edit_fixture(name)
        request_id, prepared, _, _ = self.edit_prepare(editor, project, session)
        observation.require(prepared.get("status") == "prepared",
                            "native_synchronous_callback_attempt_prepared")
        event = self.native_action(editor, "native_edit_advance", request_id=request_id,
                                   stage="prepared", mode="change")
        result, callback = event["result"], event["callback"]
        immediate = self.native_action(editor, "native_edit_inspect")["document"]
        after, disks_after = self.snapshot(editor, project)
        observation.require(callback.get("called") is True and
                            callback.get("human_text", "").startswith("# HUMAN_CALLBACK_NEWER\n") and
                            result.get("status") == "partial" and
                            result.get("reason") == "intermediate_source_or_version_changed" and
                            result.get("facts", {}).get("buffer_changed") is True and
                            result["facts"].get("tag_attempted") is False and
                            immediate["B"].startswith("# HUMAN_CALLBACK_NEWER\n") and
                            DESIRED not in immediate["B"] and
                            immediate["version"] != immediate["saved_version"] and
                            disks_after["subject"] == disks_before["subject"] and
                            after["other"] == before["other"],
                            "sync_newer_human_buffer_preserved_no_candidate_insert_disk_or_tag")
        self.case(name, reason=result["reason"], facts=result["facts"],
                  immediate_buffer_sha256=self.stock_sha(immediate["B"]),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="actual_lines_edited_callback_newer_buffer_and_partial_history")

    def native_private_reentrant_finish(self):
        for mode in ("cancel", "stop", "nested_finish"):
            name = "native_private_sync_" + mode + "_holds_bridge_slot"
            project = self.fixture(name, controlled=True)
            self.installed_native(project)
            editor = self.start_editor(project, configured=True)
            descriptor = observation.wait_for(
                lambda: self.descriptors(project), "native_private_effectful_owner_advertised")[0]
            self.action(editor, "prepare_subject")
            self.action(editor, "prepare_other")
            self.action(editor, "dirty_other")
            self.native_action(editor, "native_idle")
            before, disks_before = self.snapshot(editor, project)
            request_id = secrets.token_hex(16)
            begin = self.native_action(
                editor, "native_edit_private_begin", request_id=request_id,
                session_id=descriptor["session_id"], expected=disks_before["subject"]["text"],
                desired=DESIRED)["result"]
            observation.require(begin.get("status") == "prepared",
                                "private_effectful_callback_attempt_prepared_" + mode)
            response = self.native_action(
                editor, "native_edit_private_finish", request_id=request_id, mode=mode)
            callback, result = response["callback"], response["result"]
            observation.require(response["slot_busy_before"] and
                                callback.get("called") is True and
                                callback["slot_before"] and callback["slot_after"] and
                                response["slot_busy_after"] is False,
                                "shared_slot_owned_until_outer_effectful_stage_returns_" + mode)
            after, disks_after = self.snapshot(editor, project)
            if mode in ("cancel", "stop"):
                cancelled = (callback.get("stopped") is True if mode == "stop" else
                             callback["cancel"]["status"] == "partial" and
                             callback["cancel"]["reason"] == "closing_after_active_stage")
                observation.require(cancelled and result["status"] == "partial" and
                                    result["reason"] == "cancelled_during_stage" and
                                    result["facts"]["buffer_changed"] and
                                    not result["facts"]["tag_attempted"] and
                                    response["document"]["B"] == "" and
                                    response["document"]["version"] !=
                                    response["document"]["saved_version"] and
                                    disks_after["subject"] == disks_before["subject"],
                                    "callback_cancel_partial_after_remove_before_slot_release")
            else:
                observation.require(callback["nested"]["status"] == "busy" and
                                    callback["nested"]["reason"] == "slot_busy" and
                                    result["status"] == "complete" and
                                    result["facts"]["tagged"] and
                                    after["subject"]["B"] == after["subject"]["R"] == DESIRED and
                                    disks_after["subject"]["text"] == DESIRED and
                                    after["subject"]["version"] ==
                                    after["subject"]["saved_version"],
                                    "nested_finish_refused_outer_owner_completed_one_effect_sequence")
            observation.require(after["other"] == before["other"] and
                                disks_after["other"] == disks_before["other"],
                                "private_callback_preserved_other_human_work_" + mode)
            self.case(name, native_status=result["status"], facts=result["facts"],
                      before=self.source_free_state(before, disks_before),
                      after=self.source_free_state(after, disks_after),
                      source_surfaces="actual_synchronous_callback_bridge_slot_and_bound_native_stage")

    def native_read_only_prepare(self):
        name = "native_actual_read_only_preparation_refusal"
        editor, project, session, before, disks_before = self.edit_fixture(name)
        target = project / "scripts/subject.gd"
        target.chmod(0o400)
        try:
            protected, protected_disk = self.snapshot(editor, project)
            request_id, result, _, _ = self.edit_prepare(editor, project, session)
            after, after_disk = self.snapshot(editor, project)
            observation.require(result.get("status") == "refused" and
                                result.get("reason") == "denied_access" and
                                after_disk == protected_disk and after == protected,
                                "real_odwr_read_only_permission_refusal_no_history_or_write")
            self.case(name, reason=result["reason"],
                      before=self.source_free_state(protected, protected_disk),
                      after=self.source_free_state(after, after_disk),
                      source_surfaces="real_permission_denied_no_history_no_tag")
            self.native_action(editor, "native_edit_cancel", request_id=request_id)
        finally:
            target.chmod(0o600)

    def native_prepare_boundaries(self):
        for name, options, mutation in (
                ("native_wrong_document_binding", {"wrong_document": True}, None),
                ("native_wrong_session_binding", {"wrong_session": True}, None),
                ("native_excess_local_expiry_refused", {"expiry_budget_us": 9500000}, None),
                ("native_wrong_selected_path_binding",
                 {"source_path": "res://scripts/other.gd"}, None),
                ("native_target_dirty_before_prepare", {}, "dirty"),
                ("native_same_text_newer_before_prepare", {}, "same_text")):
            editor, project, session, _, _ = self.edit_fixture(name)
            if mutation:
                self.native_action(editor, "native_edit_human", mode=mutation)
            before, disks_before = self.snapshot(editor, project)
            arguments = {"session_id": secrets.token_hex(16) if
                         options.get("wrong_session") else session,
                         "request_id": secrets.token_hex(16),
                         "expected": disks_before["subject"]["text"],
                         "desired": DESIRED,
                         "wrong_document": options.get("wrong_document", False),
                         "expiry_budget_us": options.get("expiry_budget_us", 9000000),
                         "source_path": options.get("source_path", ROOT)}
            result = self.native_action(editor, "native_edit_prepare", **arguments)["result"]
            after, disks_after = self.snapshot(editor, project)
            observation.require(result.get("status") == "refused" and
                                not result.get("facts", {}).get("tagged", False) and
                                after == before and disks_after == disks_before,
                                "native_prepare_refused_before_effect_" + name)
            self.case(name, native_status=result["status"], reason=result.get("reason"),
                      before=self.source_free_state(before, disks_before),
                      after=self.source_free_state(after, disks_after),
                      source_surfaces="real_bound_target_source_versions_unchanged")
        editor, project, session, _, _ = self.edit_fixture("native_overlap_busy")
        first, prepared, before, disks_before = self.edit_prepare(editor, project, session)
        observation.require(prepared.get("status") == "prepared",
                            "first_native_attempt_admitted")
        second, overlap, _, _ = self.edit_prepare(editor, project, session)
        after, disks_after = self.snapshot(editor, project)
        observation.require(second != first and overlap.get("status") == "busy" and
                            overlap.get("reason") == "slot_busy" and
                            after == before and disks_after == disks_before,
                            "busy_native_attempt_not_queued_or_applied")
        self.native_action(editor, "native_edit_cancel", request_id=first)
        self.case("native_overlap_busy", native_status=overlap["status"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="genuine_native_attempt_slot_no_history_effect")

    def native_bridge_slot(self):
        name = "native_shared_bridge_collection_slot"
        project = self.fixture(name, controlled=True)
        self.installed_native(project)
        editor = self.start_editor(project, configured=True)
        descriptor = observation.wait_for(lambda: self.descriptors(project),
                                          "native_authenticated_observation_advertised")[0]
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        before, disks_before = self.snapshot(editor, project)
        request = secrets.token_hex(16)
        owner = self.native_action(editor, "native_edit_private_begin",
                                   request_id=request, session_id=descriptor["session_id"],
                                   expected=disks_before["subject"]["text"],
                                   desired=DESIRED)["result"]
        observation.require(owner.get("status") == "prepared" and
                            self.native_action(editor, "native_edit_private_info")["slot_busy"],
                            "private_addon_owner_claimed_existing_bridge_slot")
        stream, peer_id = self.authenticated_peer(descriptor)
        try:
            stream.sendall(observation.packet([3, "observe", peer_id,
                                                descriptor["session_id"],
                                                descriptor["project_root"], ROOT]))
            refusal, raw = observation.receive(stream)
            observation.require(refusal == {"v": 3, "kind": "failure",
                                            "request_id": peer_id,
                                            "session_id": descriptor["session_id"],
                                            "project_root": descriptor["project_root"],
                                            "script_path": ROOT, "code": "unsupported_observation",
                                            "stage": "read_editor"} and
                                DESIRED.encode() not in raw,
                                "authenticated_observation_refused_without_source_during_edit")
        finally:
            stream.close()
        after, disks_after = self.snapshot(editor, project)
        observation.require(after == before and disks_after == disks_before,
                            "busy_observer_and_prepared_edit_had_zero_history_source_effects")
        cancelled = self.native_action(editor, "native_edit_private_cancel",
                                       request_id=request)["result"]
        observation.require(cancelled.get("reason") == "cancelled" and
                            not self.native_action(editor, "native_edit_private_info")["slot_busy"],
                            "private_attempt_cancel_released_slot")
        stream, peer_id = self.authenticated_peer(descriptor)
        try:
            sample = self.peer_operation(stream, descriptor, peer_id, "observe",
                                         project / "scripts/subject.gd")
            observation.require(sample["R"]["text"] == before["subject"]["R"] and
                                sample["B"]["text"] == before["subject"]["B"] and
                                sample["dirty"]["state"] == "clean",
                                "authenticated_observation_resumes_after_edit_cleanup")
        finally:
            stream.close()
        observation.wait_for(lambda: not self.native_action(
            editor, "native_edit_private_info")["slot_busy"],
            "observation_slot_released_after_peer_close")
        self.action(editor, "hold_observe")
        stream, peer_id = self.authenticated_peer(descriptor)
        try:
            stream.sendall(observation.packet([3, "observe", peer_id,
                                                descriptor["session_id"],
                                                descriptor["project_root"], ROOT]))
            observation.wait_for(lambda: (editor["control"] / "event.json").is_file(),
                                 "actual_authenticated_collection_at_owned_barrier")
            busy = self.native_action(editor, "native_edit_private_begin",
                                      request_id=secrets.token_hex(16),
                                      session_id=descriptor["session_id"],
                                      expected=disks_before["subject"]["text"],
                                      desired=DESIRED)["result"]
            observation.require(busy.get("status") == "busy" and
                                busy.get("reason") == "slot_busy",
                                "live_collection_refuses_private_edit_before_native_effects")
            self.action(editor, "release_hold")
            result, _raw = observation.receive(stream, 12 * 1024 * 1024)
            observation.require(result.get("kind") == "sample",
                                "held_real_collection_completed_after_release")
        finally:
            stream.close()
            self.action(editor, "release_hold")
        end, end_disk = self.snapshot(editor, project)
        observation.require(end == before and end_disk == disks_before,
                            "mutually_exclusive_native_and_observation_slot_has_zero_effects")
        self.case(name, mapping="T003/native-integration §1 private slot; T004 caller pending",
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(end, end_disk),
                  source_surfaces="authenticated_observe_busy_and_actual_native_zero_effects")

    def native_finalization(self):
        self.compile_window_probe()
        self.group("bridge-slot", self.native_bridge_slot)
        self.group("reentrant-finish", self.native_private_reentrant_finish)
        self.group("editor-effects", self.native_editor_effect_admission)
        self.group("representation", self.native_representation_refusal)
        self.group("synchronous-human-change", self.native_sync_remove_race)
        self.group("clean-persistence", lambda: self.native_success("native_t0_clean_persisted"))
        self.group("same-size-persistence", lambda: self.native_success(
            "native_same_size_t0_persistence",
            baseline_source='extends RefCounted\nfunc value() -> int:\n\treturn 18\n'))
        self.group("empty-source", lambda: self.native_success("native_empty_saved_state", desired=""))
        self.group("invalid-source-repair", lambda: self.native_success(
            "native_invalid_source_repaired",
            baseline_source='extends RefCounted\nvar broken: int = "wrong"\n'))
        self.group("post-change-failure", self.native_post_change_failure)
        large = "extends RefCounted\n# " + "é" * 262133 + "x"
        observation.require(len(large.encode("utf-8")) == 524288,
                            "native_maximal_unicode_utf8_source_bytes")
        self.group("unicode-limit", lambda: self.native_success(
            "native_exact_unicode_source_limit", desired=large))
        self.group("short-write", lambda: self.native_success(
            "native_eligible_short_write", fault="short_write"))
        self.group("read-only", self.native_read_only_prepare)
        self.group("preparation", self.native_prepare_boundaries)
        self.group("native-history", lambda: self.native_success(
            "native_undo_save_redo_save_prior_history", history=True))
        for name, at, mutation in (
                ("native_dirty_before_write", "resource_applied", "dirty"),
                ("native_profile_changed_before_write", "resource_applied", "profile"),
                ("native_same_text_version_before_restore", "content_persisted", "same_text"),
                ("native_saved_version_before_write", "resource_applied", "saved_version"),
                ("native_close_reopen_before_write", "prepared", "close_reopen"),
                ("native_close_reopen_before_restoration", "content_persisted", "close_reopen"),
                ("native_leaf_replaced_before_write", "resource_applied", "leaf"),
                ("native_parent_replaced_before_write", "resource_applied", "parent"),
                ("native_profile_changed_before_restore", "content_persisted", "profile"),
                ("native_leaf_replaced_before_restore", "content_persisted", "leaf"),
                ("native_parent_replaced_before_restore", "content_persisted", "parent"),
                ("native_human_after_persist", "content_persisted", "dirty"),
                ("native_human_before_tag", "edited_cleared", "dirty")):
            self.group(name, lambda: self.native_refusal(
                name, at=at, mutation=mutation, expect_applied=at != "prepared"))
        for name, at, fault in (
                ("native_pwrite_error", "resource_applied", "fail_pwrite"),
                ("native_injected_erofs", "resource_applied", "read_only"),
                ("native_truncate_error", "resource_applied", "fail_truncate"),
                ("native_fsync_error", "resource_applied", "fail_fsync"),
                ("native_pread_error", "resource_applied", "fail_pread"),
                ("native_futimens_error", "content_persisted", "fail_futimens"),
                ("native_mismatched_actual_t0_readback", "content_persisted", "mismatch_mtime"),
                ("native_real_ebadf_after_write", "content_persisted", "close_fd_before_restore")):
            self.group(name, lambda: self.native_refusal(
                name, at=at, fault=fault, expect_applied=True))
