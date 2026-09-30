"""Product opening preparation consumed by existing edit A–E acceptance."""

import run_observation as observation
from native_finalization_acceptance import DESIRED
from opening_fixture_witness import TARGET


class ComposedOpenAcceptanceMixin:
    def _composed_closed_observation(self, editor, project, descriptor, name):
        before, disks = self.open_state(editor, project)
        observation.require(not before["target"]["associated"] and
                            TARGET not in before["open_paths"] and not before["slot_busy"],
                            "composed_independently_closed_before_observation_" + name)
        result = self.observe(project, "not_open", 0, session=descriptor["session_id"], name=name)
        after, now = self.open_state(editor, project)
        snapshot = result["snapshot"]
        sources = snapshot["sources"]
        observation.require(before == after and disks == now and
                            result["resolved_target"]["project_root"] == str(project.resolve()) and
                            result["resolved_target"]["session_id"] == descriptor["session_id"] and
                            result["resolved_target"]["script_path"] == TARGET and
                            snapshot["document"]["open_state"]["value"] == "not_open" and
                            sources["D"]["availability"] == "observed" and
                            sources["D"]["text"] == disks["target"]["text"] and
                            sources["D"]["witness"]["disk_file_id"] ==
                            {"device": disks["target"]["device"], "inode": disks["target"]["inode"]} and
                            sources["B"]["availability"] == "not_applicable" and
                            snapshot["dirty"]["availability"] == "not_applicable" and
                            snapshot["consistency"]["checks"] == "performed" and
                            not snapshot["consistency"]["detected_changes"],
                            "composed_closed_observation_read_only_exact_source_" + name)
        if before["cached_id"]:
            observation.require(sources["R"]["availability"] == "observed" and
                                sources["R"]["text"] == before["cached_R"] and
                                sources["R"]["witness"]["script_instance_id"] == before["cached_id"],
                                "composed_closed_observation_actual_cached_resource_" + name)
        else:
            observation.require(sources["R"]["availability"] == "unavailable" and
                                sources["R"]["reason"]["code"] == "resource_not_loaded",
                                "composed_closed_observation_does_not_load_resource_" + name)
        witness = self.artifacts / (name + "-witness.json")
        observation.json_file(witness, {"before": self.concise_open_state(before, disks),
                                        "after": self.concise_open_state(after, now)})
        self.cases[-1].update(witness=str(witness.resolve()),
                              observation_evidence=str((self.artifacts / (name + ".json")).resolve()))
        return before, disks

    def prepare_caller_subject(self, project, editor, descriptor, name):
        # This replaces only caller_fixture's first target preparation. Prior
        # human history and dirty-background setup still run in that fixture.
        before, disks = self._composed_closed_observation(
            editor, project, descriptor, "composed_closed_observation_" + name)
        opened_name = "composed_product_open_" + name
        result = self.public_open(project, descriptor, opened_name)
        after, now = self.open_state(editor, project)
        document = after["target"]
        observation.require(document["associated"] and document["matches"] == 1 and
                            document["R"] == document["B"] == now["target"]["text"] ==
                            disks["target"]["text"] and not document["dirty"] and
                            document["resource_edited"] is False and
                            document["version"] == document["saved_version"] and
                            after["cached_id"] == document["script_id"] and
                            after["cached_R"] == document["R"] and
                            after["cached_edited"] is False and
                            after["open_paths"].count(TARGET) == 1 and not after["slot_busy"] and
                            after["selection"] == TARGET and
                            (not before["cached_id"] or document["script_id"] == before["cached_id"]),
                            "composed_product_open_independent_clean_exact_d_r_b_identity_" + name)
        self.assert_preserved(before, after, disks, now, opened_name)
        self.assert_public_before(result, before, disks)
        self.assert_public_sources(result, after, now, descriptor)
        self.assert_public_selection(result, before, after, effect="selected_target")
        self.opening_evidence(opened_name, before, disks, after, now,
                              capture=self.screenshot(editor, opened_name + ".png"))
        self.cases[-1].update(
            opening_evidence=str((self.artifacts / (opened_name + ".json")).resolve()),
            opening_witness=str((self.artifacts / (opened_name + "-witness.json")).resolve()))

        fresh_name = "composed_fresh_observation_" + name
        fresh = self.edit_basis(project, descriptor, fresh_name)
        observed, observed_disks = self.open_state(editor, project)
        observation.require(after == observed and now == observed_disks and
                            fresh["request_id"] != result["request_id"],
                            "composed_separate_fresh_observation_is_read_only_" + name)
        self.compare_snapshot(fresh, descriptor, {"subject": observed["target"]},
                              observed_disks["target"], project / "scripts/subject.gd")
        self.cases[-1].update(
            after=self.concise_open_state(observed, observed_disks),
            observation_evidence=str((self.artifacts / (fresh_name + ".json")).resolve()),
            source_surfaces="separate_actual_observer_and_independent_post_open_d_r_b_dirty_edited_identity")

    def durable_reopen(self, editor, project, source, name):
        # Keep the ordinary existing close/reopen proof and additionally exercise
        # the opening fixture's real ordinary cached Resource consumer.
        prior, prior_disk = self.open_state(editor, project)
        after, now = super().durable_reopen(editor, project, source, name)
        reopened, reopened_disk = self.open_state(editor, project)
        consumer = self.open_action(editor, "open_cache")["state"]
        consumed, consumed_disk = self.open_state(editor, project)
        observation.require(prior["target"]["script_id"] == reopened["target"]["script_id"] ==
                            consumed["target"]["script_id"] == consumer["cached_id"] and
                            consumer["cached_R"] == source and consumer["cached_edited"] is False and
                            consumer == consumed == reopened and
                            consumed_disk == reopened_disk == prior_disk,
                            "composed_real_reopen_and_ordinary_cached_consumer_keep_identity_source_" + name)
        witness = self.artifacts / (name + "-cache-consumer-witness.json")
        observation.json_file(witness, {"before_close": self.concise_open_state(prior, prior_disk),
                                        "after_reopen": self.concise_open_state(reopened, reopened_disk),
                                        "after_consumer": self.concise_open_state(consumed, consumed_disk)})
        self.case(name + "_ordinary_cached_consumer", witness=str(witness.resolve()),
                  before=self.concise_open_state(prior, prior_disk),
                  after=self.concise_open_state(consumed, consumed_disk),
                  source_surfaces="actual_resource_loader_cache_consumer_after_real_editor_close_reopen")
        return after, now

    def composed_closed_edit(self):
        self.compile_window_probe()
        project, editor, descriptor, _, _ = self.caller_fixture("closed-edit")
        basis = self.edit_basis(project, descriptor, "composed_closed_edit_prior_open_basis")
        self.native_action(editor, "native_edit_human", mode="close")
        before, disks = self._composed_closed_observation(
            editor, project, descriptor, "composed_closed_after_product_open_observation")
        name = "composed_closed_edit_never_implicitly_opens"
        refused = self.edit(project, basis, DESIRED, name, "refused", 3,
                            session=descriptor["session_id"], reason="closed_target",
                            application="not_applied")
        after, now = self.open_state(editor, project)
        observation.require(refused["stage"] == "selected" and
                            refused["resolved_target"]["session_id"] == descriptor["session_id"] and
                            refused["resolved_target"]["script_path"] == TARGET and
                            refused["history"] == "not_participated" and
                            before == after and disks == now and
                            not after["target"]["associated"] and TARGET not in after["open_paths"] and
                            not after["slot_busy"],
                            "composed_closed_edit_refusal_preserves_target_source_human_work_and_cache")
        witness = self.artifacts / (name + "-witness.json")
        observation.json_file(witness, {"before": self.concise_open_state(before, disks),
                                        "after": self.concise_open_state(after, now)})
        self.cases[-1].update(witness=str(witness.resolve()),
                              edit_evidence=str((self.artifacts / (name + ".json")).resolve()))
        self._composed_closed_observation(
            editor, project, descriptor, "composed_closed_after_refusal_observation")

    def composed(self):
        groups = (("composed-clean-edit", self.clean_open_edit),
                  ("composed-conflict-edit", self.conflict_edit),
                  ("composed-history-edit", self.history_edit),
                  ("composed-durability-edit", self.durability_edit),
                  ("composed-sequential-edit", self.sequential_edit),
                  ("composed-closed-contracts", self.composed_closed_edit))
        for name, operation in groups:
            self.group(name, operation)
        self.summary["composed_acceptance"] = {
            "A": "composed-clean-edit", "B": "composed-conflict-edit",
            "C": "composed-history-edit", "D": "composed-durability-edit",
            "E": "composed-sequential-edit",
            "closed_observation_and_edit": "composed-closed-contracts",
            "initial_target_preparation": "actual_public_open_gdscript_then_separate_fresh_observation"}
