"""Cumulative product opening with real human, identity and native-history transitions."""

import run_observation as observation
from opening_fixture_witness import BACKGROUND, CURRENT, TARGET, source_free_document


class CumulativeOpenAcceptanceMixin:
    def _sequence_open(self, project, editor, descriptor, name, expected, *, reason=None,
                       retired=None):
        before, disks = self.open_state(editor, project)
        observation.require(len(before["open_paths"]) == len(set(before["open_paths"])),
                            "sequence_no_preexisting_duplicate_documents_" + name)
        if expected == "verified_newly_opened":
            observation.require(not before["target"]["associated"] and
                                before["target"]["matches"] == 0 and
                                TARGET not in before["open_paths"] and
                                (not before["cached_id"] or
                                 (before["cached_R"] == disks["target"]["text"] and
                                  before["cached_edited"] is False)),
                                "sequence_genuinely_closed_supported_target_" + name)
        elif expected == "already_open_unchanged":
            observation.require(before["target"]["associated"] and
                                before["target"]["matches"] == 1,
                                "sequence_actual_existing_target_identity_" + name)
        result = self.public_open(project, descriptor, name, expected, reason=reason)
        if expected == "verified_newly_opened":
            after, now = self.open_state(editor, project)
            document = after["target"]
            observation.require(document["associated"] and document["matches"] == 1 and
                                document["R"] == document["B"] == now["target"]["text"] ==
                                disks["target"]["text"] and not document["dirty"] and
                                document["version"] == document["saved_version"] and
                                document["resource_edited"] is False and
                                not document["has_undo"] and not document["has_redo"] and
                                after["selection"] == TARGET and not after["slot_busy"],
                                "sequence_independent_clean_open_without_source_history_action_" + name)
            self.assert_preserved(before, after, disks, now, name)
            observation.require(result["progress"]["resource_binding"]["mode"] ==
                                ("reused" if before["cached_id"] else "created") and
                                (not before["cached_id"] or
                                 document["script_id"] == before["cached_id"]),
                                "sequence_existing_resource_never_reloaded_or_replaced_" + name)
            if retired is not None:
                observation.require((document["editor_id"], document["buffer_id"]) != retired,
                                    "sequence_new_open_has_current_not_retired_document_identity_" + name)
        else:
            after, now = self.assert_no_effect(editor, project, before, disks, name)
        observation.require(len(after["open_paths"]) == len(set(after["open_paths"])) and
                            after["open_paths"].count(TARGET) == int(after["target"]["associated"]) and
                            result["history"]["participation"] == "not_participated",
                            "sequence_no_duplicate_or_opening_history_participation_" + name)
        if result["before"] is not None:
            self.assert_public_before(result, before, disks)
        if expected != "refused":
            self.assert_public_sources(result, after, now, descriptor)
        if result["selection"] is not None:
            self.assert_public_selection(result, before, after, effect=(
                "selected_target" if expected == "verified_newly_opened" else "none"))
        capture = self.screenshot(editor, name + ".png")
        self.opening_evidence(name, before, disks, after, now,
                              capture=str((self.artifacts / capture).resolve()))
        case = self.cases[-1]
        case.update(evidence=str((self.artifacts / (name + ".json")).resolve()),
                    witness=str((self.artifacts / (name + "-witness.json")).resolve()),
                    genuinely_closed_before=not before["target"]["associated"],
                    dirty_before=before["target"].get("dirty"),
                    native_history_before=source_free_document(before["target"]),
                    native_history_after=source_free_document(after["target"]))
        if expected == "refused":
            # A frame-bounded actual editor action lets terminal EOF/cleanup and
            # any illicit deferred lifecycle work run before the later witness.
            self.open_action(editor, "open_idle")
            late, late_disks = self.open_state(editor, project)
            observation.require(late == after and late_disks == now and
                                not late["slot_busy"] and not late["target"]["associated"],
                                "sequence_proven_refusal_never_reopens_later_" + name)
            late_path = self.artifacts / (name + "-late-witness.json")
            observation.json_file(late_path, {
                "terminal": self.concise_open_state(after, now),
                "after_native_frames": self.concise_open_state(late, late_disks)})
            case["late_no_effect_witness"] = str(late_path.resolve())
        return after, now

    def _sequence_mutate(self, editor, project, transitions, name, mode, *, path=TARGET,
                         idle=False, expected=None):
        before, disks = self.open_state(editor, project)
        key = {TARGET: "target", CURRENT: "current", BACKGROUND: "background"}[path]
        previous = before[key]
        self.open_action(editor, "open_setup" if idle else "open_mutate",
                         mutation=mode, path=path, **({"idle": True} if idle else {}))
        after, now = self.open_state(editor, project)
        document = after[key]
        observation.require(disks == now and document["associated"] and
                            all(document[field] == previous[field] for field in
                                ("script_id", "editor_id", "buffer_id", "saved_version")) and
                            document["B"] != previous["B"] and
                            document["version"] != previous["version"] and
                            (expected is None or document["B"] == expected) and
                            (not idle or document["R"] == document["B"]),
                            "sequence_actual_native_typing_or_history_transition_" + name)
        for other in ("target", "current", "background"):
            if other != key:
                observation.require(
                    source_free_document(before[other]) == source_free_document(after[other]) and
                    before[other].get("R") == after[other].get("R") and
                    before[other].get("B") == after[other].get("B"),
                    "sequence_native_human_action_does_not_change_another_document_" + name + "_" + other)
        if mode == "undo":
            observation.require(document["has_redo"], "sequence_real_undo_retains_redo_" + name)
        elif mode == "redo":
            observation.require(document["has_undo"], "sequence_real_redo_retains_undo_" + name)
        transitions.append({"transition": name, "native_action": mode, "document": key,
                            "before": self.concise_open_state(before, disks),
                            "after": self.concise_open_state(after, now)})
        return after, now

    def _sequence_save(self, editor, project, transitions, name):
        before, disks = self.open_state(editor, project)
        source = before["target"]["B"]
        observation.require(before["target"]["dirty"] and source != disks["target"]["text"] and
                            before["target"]["R"] == source,
                            "sequence_human_save_chooses_actual_unsaved_source_" + name)
        saved = self.native_action(editor, "native_edit_save")
        after, now = self.open_state(editor, project)
        document = after["target"]
        observation.require(saved["focused"] and saved["before_disk_changed"] and saved["disk_matches"] and
                            document["R"] == document["B"] == now["target"]["text"] == source and
                            not document["dirty"] and document["resource_edited"] is False and
                            document["version"] == document["saved_version"] and document["has_undo"],
                            "sequence_ordinary_human_save_preserves_source_and_history_" + name)
        for key in ("current", "background"):
            observation.require(disks[key] == now[key] and
                                source_free_document(before[key]) == source_free_document(after[key]) and
                                before[key]["R"] == after[key]["R"] and before[key]["B"] == after[key]["B"],
                                "sequence_human_save_preserves_unrelated_unsaved_work_" + key)
        transitions.append({"transition": name, "native_action": "ordinary_save_shortcut",
                            "before": self.concise_open_state(before, disks),
                            "after": self.concise_open_state(after, now)})
        return after, now

    def _sequence_close(self, editor, project, transitions, name, *, reopen=False):
        before, disks = self.open_state(editor, project)
        document = before["target"]
        observation.require(document["associated"] and not document["dirty"] and
                            document["R"] == document["B"] == disks["target"]["text"],
                            "sequence_human_close_never_discards_unsaved_work_" + name)
        self.open_action(editor, "open_mutate", mutation="target_reopen" if reopen else "target_close")
        self.open_action(editor, "open_idle")
        after, now = self.open_state(editor, project)
        self.assert_preserved(before, after, disks, now, name)
        retired = (document["editor_id"], document["buffer_id"])
        observation.require(after["cached_id"] == document["script_id"] and
                            after["cached_R"] == document["R"] and after["cached_edited"] is False,
                            "sequence_human_close_retains_exact_clean_source_resource_" + name)
        if reopen:
            current = after["target"]
            observation.require(current["associated"] and not current["dirty"] and
                                current["R"] == current["B"] == document["B"] and
                                (current["editor_id"], current["buffer_id"]) != retired,
                                "sequence_deliberate_human_reopen_has_new_current_identity_" + name)
        else:
            observation.require(not after["target"]["associated"] and after["target"]["matches"] == 0 and
                                TARGET not in after["open_paths"],
                                "sequence_independent_human_closed_document_absence_" + name)
        transitions.append({"transition": name, "native_action": "human_close_reopen" if reopen else
                            "human_close", "before": self.concise_open_state(before, disks),
                            "after": self.concise_open_state(after, now)})
        return retired

    def sequential(self):
        self.compile_window_probe()
        first_case = len(self.cases)
        transitions = []
        with self.opening_fixture("sequential-evolving") as (project, editor, descriptor):
            # These are pre-existing human actions, not an opener-created history.
            originals, original_disks = self.open_state(editor, project)
            for key, path in (("background", BACKGROUND), ("current", CURRENT)):
                typed, _ = self._sequence_mutate(editor, project, transitions, "prior_" + key + "_typing",
                                                  "dirty_equal", path=path, idle=True)
                source = typed[key]["B"]
                self._sequence_mutate(editor, project, transitions, "prior_" + key + "_undo", "undo",
                                      path=path, idle=True, expected=originals[key]["B"])
                self._sequence_mutate(editor, project, transitions, "prior_" + key + "_redo", "redo",
                                      path=path, idle=True, expected=source)
            retired = None
            for index in range(1, 6):
                prefix = f"sequential_{index:02d}"
                opened, disk = self._sequence_open(project, editor, descriptor, prefix + "_closed_open",
                                                   "verified_newly_opened", retired=retired)
                base = opened["target"]["B"]
                # The human selects an existing tab; recognition may not steal it.
                self.open_action(editor, "open_select", path=CURRENT)
                self._sequence_open(project, editor, descriptor, prefix + "_clean_nonselected",
                                    "already_open_unchanged")
                if index == 2:
                    before, _ = self.open_state(editor, project)
                    self.open_action(editor, "open_setup", mutation="dirty_disk_equal", path=TARGET,
                                     idle=True)
                    equal, equal_disk = self.open_state(editor, project)
                    observation.require(equal["target"]["B"] == equal["target"]["R"] ==
                                        equal_disk["target"]["text"] == base and equal["target"]["dirty"] and
                                        equal["target"]["version"] != before["target"]["version"] and
                                        equal["target"]["version"] != equal["target"]["saved_version"],
                                        "sequence_equal_source_genuinely_new_dirty_native_history")
                    transitions.append({"transition": prefix + "_same_text_new_version",
                                        "native_action": "type_then_remove_in_separate_history_actions",
                                        "before": self.concise_open_state(before, disk),
                                        "after": self.concise_open_state(equal, equal_disk)})
                    self._sequence_open(project, editor, descriptor, prefix + "_dirty_disk_equal",
                                        "already_open_unchanged")
                    transient = "# OPEN_TRANSIENT_NATIVE_HISTORY\n" + base
                    self._sequence_mutate(editor, project, transitions, prefix + "_undo_removal", "undo",
                                          idle=True, expected=transient)
                    self._sequence_open(project, editor, descriptor, prefix + "_dirty_undo_with_redo",
                                        "already_open_unchanged")
                    self._sequence_mutate(editor, project, transitions, prefix + "_redo_removal", "redo",
                                          idle=True, expected=base)
                    self._sequence_mutate(editor, project, transitions, prefix + "_new_human_typing",
                                          "dirty_equal", idle=True)
                elif index == 3:
                    conflict, _ = self._sequence_mutate(editor, project, transitions,
                                                        prefix + "_invalid_typing", "dirty_different")
                    invalid = conflict["target"]["B"]
                    observation.require(conflict["target"]["dirty"] and
                                        conflict["target"]["R"] == base != invalid,
                                        "sequence_dirty_different_actual_resource_buffer_divergence")
                    self._sequence_open(project, editor, descriptor, prefix + "_dirty_different",
                                        "already_open_unchanged")
                    self._sequence_mutate(editor, project, transitions, prefix + "_newer_invalid_typing",
                                          "dirty_equal")
                    self._sequence_open(project, editor, descriptor, prefix + "_newer_dirty_version",
                                        "already_open_unchanged")
                    self._sequence_mutate(editor, project, transitions, prefix + "_undo_newer_typing",
                                          "undo", expected=invalid)
                    self._sequence_mutate(editor, project, transitions, prefix + "_undo_invalid_typing",
                                          "undo", idle=True, expected=base)
                    self._sequence_mutate(editor, project, transitions, prefix + "_redo_invalid_typing",
                                          "redo", expected=invalid)
                    self._sequence_mutate(editor, project, transitions, prefix + "_human_undo_invalid",
                                          "undo", idle=True, expected=base)
                    self._sequence_mutate(editor, project, transitions, prefix + "_human_valid_typing",
                                          "dirty_equal", idle=True)
                else:
                    typed, _ = self._sequence_mutate(editor, project, transitions, prefix + "_human_typing",
                                                    "dirty_equal", idle=True)
                    human_source = typed["target"]["B"]
                    observation.require(typed["target"]["dirty"] and typed["target"]["has_undo"] and
                                        typed["target"]["R"] == human_source != base,
                                        "sequence_dirty_current_source_equality_not_cleanliness_" + prefix)
                    self._sequence_open(project, editor, descriptor, prefix + "_dirty_equal",
                                        "already_open_unchanged")
                    if index == 1:
                        self._sequence_mutate(editor, project, transitions, prefix + "_undo_earlier", "undo",
                                              idle=True, expected=base)
                        self._sequence_open(project, editor, descriptor, prefix + "_clean_undo_with_redo",
                                            "already_open_unchanged")
                        self._sequence_mutate(editor, project, transitions, prefix + "_redo_earlier", "redo",
                                              idle=True, expected=human_source)
                    elif index == 4:
                        newer, _ = self._sequence_mutate(editor, project, transitions,
                                                        prefix + "_second_human_typing", "dirty_equal", idle=True)
                        self._sequence_open(project, editor, descriptor, prefix + "_newer_dirty_equal",
                                            "already_open_unchanged")
                        self._sequence_mutate(editor, project, transitions, prefix + "_undo_second", "undo",
                                              idle=True, expected=human_source)
                        self._sequence_open(project, editor, descriptor, prefix + "_dirty_undo_with_redo",
                                            "already_open_unchanged")
                        self._sequence_mutate(editor, project, transitions, prefix + "_redo_second", "redo",
                                              idle=True, expected=newer["target"]["B"])
                if index == 5:
                    # Reach the exact original prior actions after all five opens.
                    for key, path in (("current", CURRENT), ("background", BACKGROUND)):
                        retained, _ = self.open_state(editor, project)
                        self._sequence_mutate(editor, project, transitions, "surviving_" + key + "_undo",
                                              "undo", path=path, idle=True,
                                              expected=original_disks[key]["text"])
                        self._sequence_mutate(editor, project, transitions, "surviving_" + key + "_redo",
                                              "redo", path=path, idle=True, expected=retained[key]["B"])
                    before_resource, before_resource_disk = self._sequence_mutate(
                        editor, project, transitions, prefix + "_divergent_human_buffer", "dirty_different")
                    self.action(editor, "resource_subject")
                    divergent, divergent_disk = self.open_state(editor, project)
                    observation.require(divergent["target"]["dirty"] and
                                        len({divergent["target"]["R"], divergent["target"]["B"],
                                             divergent_disk["target"]["text"]}) == 3,
                                        "sequence_independently_three_way_divergent_human_state")
                    observation.require(divergent_disk == before_resource_disk and
                                        divergent["target"]["B"] == before_resource["target"]["B"] and
                                        all(divergent["target"][key] == before_resource["target"][key]
                                            for key in ("script_id", "editor_id", "buffer_id", "version",
                                                        "saved_version", "has_undo", "has_redo")),
                                        "sequence_independent_resource_change_preserves_real_buffer_history")
                    transitions.append({
                        "transition": prefix + "_independent_resource_divergence",
                        "native_action": "fixture_resource_source_conflict",
                        "before": self.concise_open_state(before_resource, before_resource_disk),
                        "after": self.concise_open_state(divergent, divergent_disk)})
                    self._sequence_open(project, editor, descriptor, prefix + "_three_way_divergent",
                                        "already_open_unchanged")
                    # Deliberately leave this work unresolved: no Save, reload,
                    # source assignment or close is used to repair recognition.
                    break
                self._sequence_save(editor, project, transitions, prefix + "_human_save")
                if index == 1:
                    self._sequence_close(editor, project, transitions, prefix + "_human_reopen", reopen=True)
                    self._sequence_open(project, editor, descriptor, prefix + "_human_reopened_current_identity",
                                        "already_open_unchanged")
                retired = self._sequence_close(editor, project, transitions, prefix + "_human_close")
        with self.opening_fixture("sequential-unsafe-survivor") as (project, editor, descriptor):
            unsafe_original, _ = self.open_state(editor, project)
            unsafe_typed, _ = self._sequence_mutate(
                editor, project, transitions, "unsafe_current_human_typing", "dirty_different", path=CURRENT)
            self._sequence_open(project, editor, descriptor, "sequential_unsafe_current_conflict", "refused",
                                reason=("dirty_conflict", "unsafe_editor_context"))
            unsafe_newer, _ = self._sequence_mutate(
                editor, project, transitions, "unsafe_current_newer_human_typing", "dirty_equal", path=CURRENT)
            self._sequence_open(project, editor, descriptor, "sequential_unsafe_newer_current", "refused",
                                reason=("dirty_conflict", "unsafe_editor_context"))
            refused_cases = self.cases[-2:]
            self._sequence_mutate(editor, project, transitions, "unsafe_human_undo_newer", "undo",
                                  path=CURRENT, expected=unsafe_typed["current"]["B"])
            recovered, recovered_disk = self._sequence_mutate(
                editor, project, transitions, "unsafe_human_undo_conflict", "undo", path=CURRENT,
                idle=True, expected=unsafe_original["current"]["B"])
            observation.require(not recovered["current"]["dirty"] and
                                not recovered["target"]["associated"] and not recovered["cached_id"] and
                                recovered["current"]["has_redo"],
                                "sequence_human_undo_makes_context_eligible_without_discarding_redo")
            self.open_action(editor, "open_idle")
            eligible_late, eligible_disk = self.open_state(editor, project)
            observation.require(eligible_late == recovered and eligible_disk == recovered_disk and
                                not eligible_late["slot_busy"],
                                "sequence_refused_attempts_cannot_reopen_after_context_becomes_safe")
            eligible_path = self.artifacts / "sequential-refusals-after-human-context-recovery.json"
            observation.json_file(eligible_path, {
                "after_human_undo": self.concise_open_state(recovered, recovered_disk),
                "after_native_frames": self.concise_open_state(eligible_late, eligible_disk)})
            for case in refused_cases:
                case["late_no_reopen_after_context_recovery"] = str(eligible_path.resolve())
            # Restore the actual unsaved work through its retained native history.
            # There is no successful product call against the temporary recovery.
            self._sequence_mutate(editor, project, transitions, "unsafe_human_redo_conflict", "redo",
                                  path=CURRENT, expected=unsafe_typed["current"]["B"])
            self._sequence_mutate(editor, project, transitions, "unsafe_human_redo_newer", "redo",
                                  path=CURRENT, expected=unsafe_newer["current"]["B"])
            before_cache, before_cache_disk = self.open_state(editor, project)
            self.open_action(editor, "open_mutate", mutation="target_cache")
            self.open_action(editor, "open_mutate", mutation="target_cache_source")
            conflicting_cache, conflicting_disk = self.open_state(editor, project)
            observation.require(not before_cache["cached_id"] and conflicting_cache["cached_id"] and
                                not conflicting_cache["target"]["associated"] and
                                conflicting_cache["cached_R"] != conflicting_disk["target"]["text"] and
                                before_cache_disk == conflicting_disk,
                                "sequence_actual_retained_conflicting_resource_not_fabricated_absence")
            transitions.append({"transition": "unsafe_retained_cache_source_change",
                                "native_action": "fixture_cache_source_conflict",
                                "before": self.concise_open_state(before_cache, before_cache_disk),
                                "after": self.concise_open_state(conflicting_cache, conflicting_disk)})
            self._sequence_open(project, editor, descriptor, "sequential_unsafe_conflicting_retained_cache",
                                "refused", reason="cached_source_conflict")
        calls = [case for case in self.cases[first_case:] if case.get("operation") == "open_gdscript"]
        newly_opened = [case for case in calls if case["outcome"] == "verified_newly_opened"]
        clean = [case for case in calls if case["outcome"] == "already_open_unchanged" and
                 case["dirty_before"] is False]
        dirty = [case for case in calls if case["outcome"] == "already_open_unchanged" and case["dirty_before"]]
        refusals = [case for case in calls if case["outcome"] == "refused"]
        observation.require(len(calls) == 25 and len(newly_opened) == 5 and len(clean) == 7 and
                            len(dirty) == 10 and len(refusals) == 3 and
                            all(case["genuinely_closed_before"] for case in newly_opened) and
                            len({case["request_id"] for case in calls}) == len(calls),
                            "sequence_25_distinct_real_requests_five_closed_ten_dirty_three_refusals")
        trace = self.artifacts / "sequential-human-transitions.json"
        observation.json_file(trace, {"native_human_transitions": transitions})
        evidence = {"opening_requests": len(calls), "genuinely_closed_successes": len(newly_opened),
                    "clean_recognitions": len(clean), "dirty_recognitions": len(dirty),
                    "unsafe_refusals": len(refusals), "evolving_owned_fixtures": 2,
                    "human_transition_witness": str(trace.resolve()),
                    "opening_result_paths": [case["evidence"] for case in calls],
                    "native_history": "exact_prior_sources_reachable_by_actual_undo_redo_before_and_after_five_opens",
                    "source_surfaces": "actual_product_results_then_independent_disk_resource_buffer_history_witnesses"}
        self.summary["cumulative_opening"] = evidence
        self.case("sequential_cumulative_acceptance", **evidence)
