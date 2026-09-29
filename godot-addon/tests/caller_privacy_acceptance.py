"""Whole-caller privacy and installed-native production-export acceptance."""
import json
import stat

import run_observation as observation


class CallerPrivacyAcceptanceMixin:
    def privacy_export_edit(self):
        self.compile_window_probe()
        selected_marker = "CALLER_SELECTED_PRIVATE_SOURCE"
        proposal_marker = "CALLER_REQUESTED_PRIVATE_SOURCE"
        unselected_marker = "CALLER_UNSELECTED_PRIVATE_SOURCE"
        self.source_markers.update(marker.encode() for marker in
                                   (selected_marker, proposal_marker, unselected_marker))
        baseline = "extends RefCounted\n# " + selected_marker + "\nfunc value() -> int:\n\treturn 17\n"
        desired = "extends RefCounted\n# " + proposal_marker + "\nfunc value() -> int:\n\treturn 43\n"
        other_source = "extends RefCounted\n# " + unselected_marker + "\nfunc value() -> int:\n\treturn 97\n"
        project, editor, descriptor, before, disks = self.caller_fixture("privacy", baseline=baseline)
        other_project, other_editor, _, other_before, other_disks = self.caller_fixture(
            "privacy-unselected", baseline=other_source)
        basis = self.edit_basis(project, descriptor, "privacy-authorized-basis")
        observation.require(selected_marker in json.dumps(basis) and unselected_marker not in json.dumps(basis),
                            "privacy_selected_observation_contains_requested_source")
        authorized = self.edit(project, basis, desired, "privacy_authorized_edit",
                               "verified_changed", 0, session=descriptor["session_id"])
        self.validated_source(authorized, desired, "post_change", "valid")
        after, now = self.changed_state(editor, project, before, disks, desired, "privacy-authorized")
        capture = self.screenshot(editor, "privacy-authorized-visible.png")
        fresh = self.edit_basis(project, descriptor, "privacy-current-basis")

        second = self.start_editor(project)
        observation.wait_for(lambda: len(self.descriptors(project)) == 2, "privacy_two_live_sessions")
        self.action(second, "prepare_subject")
        self.action(second, "prepare_other")
        self.action(second, "dirty_other")
        self.native_action(second, "native_idle")
        second_before, second_disk = self.state(second, project)
        ambiguous = self.edit(project, fresh, baseline, "privacy_ambiguous_edit",
                              "refused", 3, application="not_applied")
        observation.require(ambiguous["resolved_target"] is None and
                            all(ambiguous[key] is None for key in ("expected", "before", "after")),
                            "privacy_ambiguous_has_no_source_evidence")
        self.unchanged_state(editor, project, after, now, "privacy-ambiguous")
        self.unchanged_state(second, project, second_before, second_disk, "privacy-second-session")
        self.close_editor(second)
        self.editors.remove(second)

        after, now = self.state(editor, project)
        path = project / "scripts/subject.gd"
        mode = stat.S_IMODE(path.stat().st_mode)
        path.chmod(0)
        try:
            denied = self.edit(project, fresh, baseline, "privacy_denied_edit", "refused", 3,
                               session=descriptor["session_id"], reason="denied_access",
                               application="not_applied")
            observation.require(all(denied[key] is None for key in ("expected", "before", "after")),
                                "privacy_denied_has_no_source_evidence")
        finally:
            path.chmod(mode)
        current, disk = self.state(editor, project)
        observation.json_file(self.artifacts / "privacy-denied-witness.json", {
            "before": self.source_free_state(after, now),
            "after": self.source_free_state(current, disk)})
        observation.require(current == after and all(disk["subject"][key] == now["subject"][key]
                            for key in ("text", "inode", "mtime_ns")) and disk["other"] == now["other"],
                            "privacy_denied_preserves_source_and_human_history")

        process, payload, started = self.held_edit(
            project, editor, fresh, baseline, "buffer_applied", "privacy-interruption",
            session=descriptor["session_id"])
        interrupted = self.complete_edit(process, payload, "privacy_interrupted_edit",
                                         ("applied_unverified", "application_unknown"), (2, 4), started=started)
        self.native_action(editor, "native_edit_release")
        survivor, survivor_disk = self.state(editor, project)
        observation.require(survivor["subject"]["B"] == baseline and survivor["subject"]["dirty"] and
                            survivor["other"] == after["other"] and
                            survivor_disk["other"] == now["other"], "privacy_interrupted_actual_survivor")
        self.unchanged_state(other_editor, other_project, other_before, other_disks, "privacy-unselected")
        unselected_hash = self.stock_sha(other_source)
        for label, result in (("authorized", authorized), ("ambiguous", ambiguous),
                              ("denied", denied), ("interrupted", interrupted)):
            encoded = json.dumps(result)
            observation.require(unselected_marker not in encoded and unselected_hash not in encoded and
                                str(other_project) not in encoded, "privacy_no_other_target_evidence_" + label)
            if label in ("ambiguous", "denied"):
                observation.require(selected_marker not in encoded and proposal_marker not in encoded and
                                    self.stock_sha(baseline) not in encoded and
                                    self.stock_sha(desired) not in encoded,
                                    "privacy_no_denied_source_derived_evidence_" + label)
        # Re-read metadata while all relevant source sentinels and live owners are present.
        self.descriptors()
        observation.json_file(self.artifacts / "privacy-survivor.json",
                              self.source_free_state(survivor, survivor_disk))
        self.case("caller_privacy_target_isolation_and_interrupted_survivor",
                  source_surfaces="independent_selected_and_unselected_d_r_b_history",
                  screenshot=capture, evidence="privacy-survivor.json")
        for owned in (other_editor, editor):
            self.close_editor(owned)
            self.editors.remove(owned)
        self.native_export()
