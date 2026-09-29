"""Cumulative real-editor durability and sequential caller acceptance."""

import run_observation as observation


class CumulativeEditAcceptanceMixin:
    @staticmethod
    def revision(value):
        return f"extends RefCounted\nfunc value() -> int:\n\treturn {value}\n"

    def coherent_revision(self, editor, project, source, name):
        witness, disks = self.snapshot(editor, project)
        subject = witness["subject"]
        observation.require(
            subject["associated"] and subject["B"] == subject["R"] ==
            disks["subject"]["text"] == source and
            disks["subject"]["sha256"] == self.stock_sha(source) and
            not subject["dirty"] and subject["version"] == subject["saved_version"] and
            witness["other"]["dirty"],
            "independent_clean_d_r_b_saved_revision_" + name)
        observation.require(self.native_action(editor, "native_edit_inspect")["resource_edited"] is False,
                            "public_resource_clean_after_revision_" + name)
        return witness, disks

    def ordinary_clean_save(self, editor, project, source, name):
        before, before_disk = self.coherent_revision(editor, project, source, name + "_before_save")
        result = self.native_action(editor, "native_edit_save_clean")
        after, disks = self.coherent_revision(editor, project, source, name + "_after_save")
        observation.json_file(self.artifacts / (name + "-ordinary-save-witness.json"), {
            "before": self.source_free_state(before, before_disk),
            "after": self.source_free_state(after, disks),
            "focused": result["focused"], "disk_matches": result["disk_matches"],
            "before_disk_changed": result["before_disk_changed"]})
        observation.require(result["focused"] and result["disk_matches"] and
                            result["before_disk_changed"] is False and
                            result["document"]["B"] == source and
                            before["subject"]["has_undo"] and
                            after["subject"]["has_undo"] and
                            result["document"]["has_undo"] and
                            disks["other"] == before_disk["other"] and
                            all(after["other"][key] == before["other"][key]
                                for key in ("B", "dirty", "version", "saved_version",
                                            "has_undo", "has_redo")),
                            "ordinary_editor_save_preserves_native_revision_and_human_work_" + name)
        self.case(name + "_ordinary_save", before=self.source_free_state(before, before_disk),
                  after=self.source_free_state(after, disks), shortcut_focused=result["focused"],
                  source_surfaces="delivered_real_editor_save_and_independent_disk_resource_buffer")
        return after, disks

    def assert_other_preserved(self, before, disks, after, now, name):
        # Godot's ordinary Save is permitted to refresh cached R for another
        # script; it may not change that script's unsaved human B/history or D.
        observation.require(now["other"] == disks["other"] and after["other"]["dirty"] and
                            all(after["other"][key] == before["other"][key]
                                for key in ("B", "version", "saved_version",
                                            "has_undo", "has_redo")),
                            "other_unsaved_human_work_survives_" + name)

    def durable_reopen(self, editor, project, source, name):
        prior, disk = self.coherent_revision(editor, project, source, name + "_before_close")
        reopened = self.native_action(editor, "native_edit_human", mode="close_reopen")["document"]
        after, now = self.coherent_revision(editor, project, source, name + "_after_reopen")
        observation.require(reopened["associated"] and reopened["B"] == reopened["R"] == source and
                            (after["subject"]["editor_id"], after["subject"]["buffer_id"]) !=
                            (prior["subject"]["editor_id"], prior["subject"]["buffer_id"]) and
                            now["subject"]["device"] == disk["subject"]["device"] and
                            now["subject"]["inode"] == disk["subject"]["inode"],
                            "real_close_reopen_keeps_saved_revision_" + name)
        self.assert_other_preserved(prior, disk, after, now, name + "_reopen")
        self.case(name + "_close_reopen", before=self.source_free_state(prior, disk),
                  after=self.source_free_state(after, now),
                  screenshot=self.screenshot(editor, name + "-reopened.png"),
                  source_surfaces="actual_closed_and_reopened_code_edit_independent_d_r_b")
        return after, now

    def durability_edit(self):
        self.compile_window_probe()
        initial, desired = self.revision(17), self.revision(23)
        project, editor, descriptor, before, disks = self.caller_fixture(
            "durability", baseline=initial)
        capture = self.screenshot(editor, "durability-before.png")
        basis = self.edit_basis(project, descriptor, "durability-fresh-basis")
        result = self.edit(project, basis, desired, "durability_caller_changed",
                           "verified_changed", 0, session=descriptor["session_id"])
        observation.require(result["application"] == "applied" and
                            result["history"] == "native_complex_edit", "durability_actual_native_history")
        self.validated_source(result, desired, "preflight", "valid")
        self.validated_source(result, desired, "post_change", "valid")
        after, now = self.changed_state(editor, project, before, disks, desired, "durability")
        self.cases[-1]["screenshot"] = capture
        self.case("durability_changed_witness", before=self.source_free_state(before, disks),
                  after=self.source_free_state(after, now),
                  screenshot=self.screenshot(editor, "durability-changed.png"),
                  source_surfaces="actual_cli_then_independent_d_r_b_saved_native_history")
        after, now = self.ordinary_clean_save(editor, project, desired, "durability")
        self.assert_other_preserved(before, disks, after, now, "durability_save")
        self.durable_reopen(editor, project, desired, "durability")
        self.native_action(editor, "native_idle")
        parsed = self.native_action(editor, "native_edit_reparse")
        after, now = self.coherent_revision(editor, project, desired, "durability_reparse")
        observation.require(parsed["parse_completed"] and parsed["parse_error"] == 0 and
                            parsed["document"]["R"] == parsed["document"]["B"] == desired and
                            parsed["script_id"] == after["subject"]["script_id"],
                            "actual_resource_reparse_keeps_edited_behavior")
        self.case("durability_completed_reparse", parse_error=parsed["parse_error"],
                  state=self.source_free_state(after, now),
                  source_surfaces="synchronous_real_script_reload_result_and_d_r_b")
        scan = self.native_action(editor, "native_edit_scan")
        after, now = self.coherent_revision(editor, project, desired, "durability_rescan")
        observation.require(scan["settled"] and scan["events"] > scan["before_events"],
                            "completed_editor_filesystem_rescan_event")
        self.case("durability_completed_rescan", before_events=scan["before_events"],
                  after_events=scan["events"], state=self.source_free_state(after, now),
                  source_surfaces="real_completed_filesystem_changed_signal_and_d_r_b")
        self.native_runtime(project, 23)
        survived, survived_disk = self.coherent_revision(editor, project, desired,
                                                       "durability_runtime_survivor")
        self.assert_other_preserved(after, now, survived, survived_disk, "durability_runtime")
        self.case("durability_runtime_survivor", runtime_marker="NATIVE_RUNTIME_VALUE=23",
                  before=self.source_free_state(after, now),
                  after=self.source_free_state(survived, survived_disk),
                  screenshot=self.screenshot(editor, "durability-runtime-survived.png"),
                  source_surfaces="actual_fresh_stock_runtime_value_and_live_editor_d_r_b")

    def sequential_edit(self):
        self.compile_window_probe()
        project, editor, descriptor, before, disks = self.caller_fixture(
            "sequential", prior=True, baseline=self.revision(17))
        self.screenshot(editor, "sequential-before.png")
        previous = before["subject"]["B"]
        successful, refusals = [], []
        for index in range(20):
            name = f"sequential_{index + 1:02d}"
            desired = self.revision(300 + index)
            basis = self.edit_basis(project, descriptor, name + "_fresh_basis")
            result = self.edit(project, basis, desired, name + "_changed",
                               "verified_changed", 0, session=descriptor["session_id"])
            observation.require(result["application"] == "applied" and
                                result["history"] == "native_complex_edit" and
                                desired != previous, "real_distinct_successful_history_" + name)
            self.validated_source(result, desired, "preflight", "valid")
            self.validated_source(result, desired, "post_change", "valid")
            changed, changed_disk = self.changed_state(editor, project, before, disks, desired, name)
            self.cases[-1]["before"] = self.source_free_state(before, disks)
            self.cases[-1]["after"] = self.source_free_state(changed, changed_disk)
            saved, saved_disk = self.ordinary_clean_save(editor, project, desired, name)
            self.assert_other_preserved(before, disks, saved, saved_disk, name)
            successful.append(desired)
            if index in (4, 10, 16):
                # The old basis is stale even for an intent equal to the current
                # text; no automatic rebase or new native history is permitted.
                stale = self.edit(project, basis, desired, name + "_stale_refused",
                                  "refused", 3, session=descriptor["session_id"],
                                  application="not_applied")
                observation.require(stale["history"] == "not_participated",
                                    "stale_refusal_no_history_" + name)
                unchanged = self.unchanged_state(editor, project, saved, saved_disk,
                                                 name + "_stale")
                self.cases[-1]["before"] = self.source_free_state(saved, saved_disk)
                self.cases[-1]["after"] = self.source_free_state(unchanged, saved_disk)
                refusals.append(stale)
                clean_basis = self.edit_basis(project, descriptor, name + "_pre_human_basis")
                human = self.native_action(editor, "native_edit_human", mode="later_value")["document"]
                observation.require(human["dirty"] and human["B"] == self.revision(29) and
                                    human["version"] != human["saved_version"],
                                    "actual_new_unsaved_human_revision_" + name)
                human_state, human_disk = self.state(editor, project)
                dirty = self.edit(project, clean_basis, desired, name + "_dirty_refused",
                                  "refused", 3, session=descriptor["session_id"],
                                  application="not_applied")
                observation.require(dirty["history"] == "not_participated",
                                    "dirty_refusal_no_native_history_" + name)
                unchanged = self.unchanged_state(editor, project, human_state, human_disk,
                                                 name + "_dirty")
                self.cases[-1]["before"] = self.source_free_state(human_state, human_disk)
                self.cases[-1]["after"] = self.source_free_state(unchanged, human_disk)
                refusals.append(dirty)
                human_save = self.native_action(editor, "native_edit_save")
                before, disks = self.coherent_revision(editor, project, human["B"],
                                                      name + "_human_saved")
                observation.require(human_save["focused"] and human_save["before_disk_changed"] and
                                    human_save["disk_matches"], "ordinary_save_keeps_human_source_" + name)
                self.assert_other_preserved(saved, saved_disk, before, disks, name + "_human")
                self.case(name + "_human_work_preserved", dirty_before=self.source_free_state(
                    human_state, human_disk), saved_after=self.source_free_state(before, disks),
                    source_surfaces="dirty_and_stale_refusals_then_actual_human_save")
                previous = human["B"]
            else:
                before, disks = saved, saved_disk
                previous = desired
        observation.require(len(successful) == 20 and len(set(successful)) == 20 and
                            len(refusals) == 6,
                            "actual_20_fresh_changes_and_six_interleaved_refusals")
        final, penultimate = successful[-1], successful[-2]
        for operation, source in (("undo", penultimate), ("redo", final)):
            step = self.native_action(editor, "native_edit_human", mode=operation)["document"]
            observation.require(step["B"] == source and step["dirty"] and
                                step["version"] != step["saved_version"],
                                "actual_usable_native_history_" + operation)
            dirty_witness, dirty_disk = self.state(editor, project)
            saved = self.native_action(editor, "native_edit_save")
            after, now = self.coherent_revision(editor, project, source,
                                                "sequential_" + operation + "_saved")
            observation.require(saved["focused"] and saved["before_disk_changed"] and
                                saved["disk_matches"] and dirty_disk["subject"]["text"] != source,
                                "actual_history_then_ordinary_save_" + operation)
            self.assert_other_preserved(before, disks, after, now, "sequential_" + operation)
            self.case("sequential_" + operation + "_ordinary_save",
                      dirty_before=self.source_free_state(dirty_witness, dirty_disk),
                      saved_after=self.source_free_state(after, now),
                      source_surfaces="actual_code_edit_history_then_delivered_save_and_d_r_b")
            before, disks = after, now
        self.durable_reopen(editor, project, final, "sequential_final")
        self.case("sequential_cumulative_count", successful_edits=len(successful),
                  dirty_or_stale_refusals=len(refusals), final_sha256=self.stock_sha(final),
                  source_surfaces="each_real_cli_transition_and_independent_save_history_reopen_cases")
