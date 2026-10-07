"""Public exact-edit complete-source bounds and native-documentation safety."""
from __future__ import annotations

import run_observation as observation
from close_native_acceptance import documents
from closed_script_acceptance import SOURCE_LIMIT
from mcp_interruption_acceptance import DESIRED, PROFILES
from mcp_peer import McpPeer, selectors
from run_script_close import SAFE, TARGET

VALIDATION_GROUPS = {
    "validation-exact-source-boundaries": "mcp_validation_exact_source_boundaries",
    "validation-exact-max-near-match": "mcp_validation_exact_max_near_match",
}


class McpValidationMixin:
    def _validation_exact_success(self, peer, project, editor, descriptor, basis,
                                  old, new, desired, label):
        before, disks = self.state(editor, project)
        root = peer.call("edit_script", dict(selectors(project, descriptor),
                         revision=basis["revision"], old_string=old, new_string=new), label)
        self.mcp_review_edit(root, label, "verified_changed")
        if TARGET not in before["open_paths"]:
            self.assert_closed_success(label, project, editor, before, disks, desired, changed=True)
        else:
            after, now = self.state(editor, project)
            observation.require(documents(after)[TARGET]["B"] == documents(after)[TARGET]["R"] ==
                                now[TARGET]["text"] == desired and
                                documents(after)[TARGET]["dirty"] is False,
                                "exact_complete_source_converges_" + label)
            target, original = documents(after)[TARGET], documents(before)[TARGET]
            observation.require(all(target[key] == original[key] for key in
                                    ("script_id", "editor_id", "buffer_id")) and
                                target["version"] == target["saved_version"] and
                                target["resource_edited"] is False,
                                "exact_validation_preserves_identity_and_observes_saved_state_" + label)
            observation.require(after["open_paths"] == before["open_paths"] and
                                after["selection"] == before["selection"] and
                                all(documents(after)[path] == doc for path, doc in documents(before).items()
                                    if path != TARGET),
                                "exact_validation_preserves_lifecycle_and_unrelated_history_" + label)
            self.record_witness(label, before, disks, after, now, editor)
        fresh = self.mcp_read(peer, project, editor, descriptor, label + "_fresh", source=desired)
        observation.require(fresh["revision"] != basis["revision"],
                            "exact_validation_success_has_new_revision_" + label)
        return fresh

    def mcp_validation_exact_source_boundaries(self):
        # Fragments need not be standalone programs; the derived complete source
        # is the sole validation target. Each success has an independent fixture.
        positives = (("fragment", SAFE, "47", "83", SAFE.replace("47", "83")),
                     ("delete_all", SAFE, SAFE, "", ""),
                     ("whitespace", SAFE, SAFE, " \t\n", " \t\n"),
                     ("insert_empty", "", "", SAFE, SAFE))
        for profile in ("open", "cached", "absent"):
            fixture = self.close_fixture if profile == "open" else self.closed_fixture
            kwargs = {} if profile == "open" else {"cached": profile == "cached"}
            for name, source, old, new, desired in positives:
                label = "mcp_exact_validation_" + profile + "_" + name
                with fixture(label, source=source, **kwargs) as (project, editor, descriptor), McpPeer(self, label) as peer:
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=source)
                    fresh = self._validation_exact_success(peer, project, editor, descriptor, basis,
                                                           old, new, desired, label)
                    if name == "whitespace":
                        refusal = self._preservation_refusal(
                            peer, project, editor, descriptor, fresh["revision"], label + "_empty_old",
                            replacement=SAFE, old_string="")
                        self._exact_reason(refusal, "empty_old_string", label + "_empty_old")
            for source_name, source in (("nonempty", SAFE), ("empty", "")):
                label = "mcp_exact_validation_refusal_" + profile + "_" + source_name
                with fixture(label, source=source, **kwargs) as (project, editor, descriptor), McpPeer(self, label) as peer:
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=source)
                    cases = (("parse", source, "extends RefCounted\nfunc value(:\n", False),
                             ("over_bound", source, "#" * (SOURCE_LIMIT + 1), True),
                             ("utf8_over_bound", source, "#" + "é" * (SOURCE_LIMIT // 2), True))
                    if source:
                        cases += (("valid_fragment_invalid_complete", "47", "extends RefCounted", False),
                                  ("derived_overflow", "47", "#" * SOURCE_LIMIT, False),
                                  ("derived_utf8_overflow", "47", "é" * (SOURCE_LIMIT // 2), False))
                    for name, old, new, input_error in cases:
                        case = label + "_" + name
                        root = self._preservation_refusal(
                            peer, project, editor, descriptor, basis["revision"], case,
                            replacement=new, old_string=old, input_error=input_error)
                        if not input_error:
                            if name.startswith("derived_"):
                                self._exact_reason(root, "invalid_source", case)
                            else:
                                observation.require(root["result"]["outcome"]["reason"] == "parse_error" and
                                                    root["result"]["outcome"]["stage"] != "matching",
                                                    "complete_source_parser_not_fragment_match_" + case)

    def mcp_validation_exact_max_near_match(self):
        line = "# " + "x" * 126 + "\n"
        lines, remainder = divmod(SOURCE_LIMIT - len(SAFE.encode()), len(line))
        run = line * lines + "#" + "x" * (remainder - 2) + "\n"
        source = SAFE + run
        observation.require(len(source.encode()) == SOURCE_LIMIT, "maximum_repetitive_source_byte_bound")
        for profile in ("open", "cached", "absent"):
            label = "mcp_exact_max_near_match_" + profile
            fixture = self.close_fixture if profile == "open" else self.closed_fixture
            kwargs = {} if profile == "open" else {"cached": profile == "cached"}
            with fixture(label, source=source, **kwargs) as (project, editor, descriptor), McpPeer(self, label) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=source)
                for name, old, reason in (("near_end", run[:-1] + "y", "no_match"),
                                          ("near_start", "y" + run[1:], "no_match"),
                                          ("overlapping", line * (lines - 1), "ambiguous_match")):
                    case = label + "_" + name
                    # McpPeer retains the existing public 10s operation clock;
                    # no performance-specific extension or private execution.
                    root = self._preservation_refusal(
                        peer, project, editor, descriptor, basis["revision"], case,
                        replacement="# intentionally distinct", old_string=old)
                    outcome = root["result"]["outcome"]
                    if outcome["reason"] in ("validation_unavailable", "timeout"):
                        # Deadline/validation failure wins on every lifecycle path;
                        # only a verified unchanged result may expose the match error.
                        observation.require(outcome["stage"] != "matching" and
                                            outcome["application"] == "not_applied",
                                            "maximum_match_retains_pre_effect_failure_" + case)
                        if root["result"]["mode"] == "open":
                            observation.require(outcome["history"] == "not_participated" and
                                                all(outcome["progress"][step]["state"] == "not_started" for step in
                                                    ("buffer_application", "resource_sync", "persistence", "finalization")),
                                                "maximum_match_has_no_open_effects_" + case)
                            if outcome["reason"] == "validation_unavailable":
                                observation.require(any(receipt["purpose"] == "unchanged" and
                                                        receipt["status"] == "unavailable" and receipt["cleanup_confirmed"]
                                                        for receipt in outcome["validation"]),
                                                    "maximum_match_retains_unavailable_validation_" + case)
                        else:
                            observation.require(not any(outcome["effects"].values()),
                                                "maximum_match_has_no_closed_effects_" + case)
                        self.cases[-1]["matching_diagnosis"] = "withheld_" + outcome["reason"]
                    else:
                        self._exact_reason(root, reason, case)

    def mcp_validation_preparation(self):
        self.summary.update(coverage_scope='prepared_native_documentation_effect_safety',
                            mcp_acceptance=True, real_client_acceptance=False,
                            mcp_server_sha256=observation.digest(self.args.mcp_server))
        for profile in PROFILES:
            for boundary in ('before', 'after'):
                label = 'native-docs-' + profile + '-' + boundary
                scratch = self.work / (label + '-scratch')
                scratch.mkdir(mode=0o700)
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label, temporary_root=scratch) as peer:
                        namespaces = list(scratch.iterdir())
                        observation.require(len(namespaces) == 1, 'one_prepared_owned_namespace_' + label)
                        snapshot = namespaces[0] / 'home/Library/Caches/Godot/editor_doc_cache-4.7.res'
                        observation.require([p for p in namespaces[0].rglob('*') if p.is_file()] == [snapshot],
                                            'only_native_snapshot_survives_preparation_' + label)
                        if boundary == 'after':
                            call = self._interruption_start(peer, project, editor, descriptor, profile,
                                                            'verify:post_change', label)
                            before, disks = self.state(editor, project)
                            observation.require(disks[TARGET]['text'] == DESIRED,
                                                'actual_effect_precedes_artifact_loss_' + label)
                            snapshot.unlink()
                            self._interruption_release(editor, profile)
                        else:
                            basis = self.mcp_read(peer, project, editor, descriptor, label + '-basis', source=SAFE)
                            before, disks = self.state(editor, project)
                            # Same size, different bytes: size/shape alone must not admit a snapshot.
                            with snapshot.open('r+b') as stream:
                                first = stream.read(1)
                                stream.seek(0)
                                stream.write(bytes([first[0] ^ 1]))
                            call = peer.start('edit_script', dict(selectors(project, descriptor),
                                               revision=basis['revision'], old_string=basis['source'], new_string=DESIRED))
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label,
                                                     'refused' if boundary == 'before' else 'applied_unverified')
                        observation.require(result['outcome']['reason'] == 'validation_unavailable',
                                            'artifact_loss_is_validation_unavailable_' + label)
                        after, now = self._interruption_survivor(project, editor, before, disks, label,
                                                                 no_effect=True)
                        expected = SAFE if boundary == 'before' else DESIRED
                        observation.require(now[TARGET]['text'] == expected,
                                            'artifact_failure_never_changes_or_rolls_back_D_' + label)
                        if profile == 'open':
                            observation.require(after['target']['B'] == after['target']['R'] == expected,
                                                'artifact_failure_retains_actual_open_authorities_' + label)
                        else:
                            observation.require(TARGET not in after['open_paths'] and
                                                after['cached_R'] == (expected if after['cached_id'] else None),
                                                'artifact_failure_preserves_closed_applicability_' + label)
                    observation.require(not list(scratch.iterdir()), 'prepared_namespace_removed_on_EOF_' + label)
                scratch.rmdir()
