"""T003 public MCP preservation, witnessed independently in isolated editors.

Setup controls perturb real state; only the executable MCP peer performs the
operation under test. Raw permitted responses are retained by McpPeer, never
converted to the private workflow driver's result shape.
"""
from __future__ import annotations

import json
import os
import secrets

import run_observation as observation
from close_native_acceptance import documents
from closed_script_acceptance import CHANGED, SOURCE_LIMIT, source_free
from mcp_peer import McpPeer, McpAdversarialMixin, selectors
from run_script_close import TARGET, CURRENT, BACKGROUND, SAFE
from run_script_edit import sha

PRESERVATION_GROUPS = {
    "preservation-open-history": "mcp_preservation_open_history",
    "preservation-prior-native-history": "mcp_preservation_prior_native_history",
    "preservation-revisions": "mcp_preservation_revisions",
    "preservation-identities": "mcp_preservation_identities",
    "preservation-cache-absence": "mcp_preservation_cache_absence",
    "preservation-resource-safety": "mcp_preservation_resource_safety",
    "preservation-session-replacement": "mcp_preservation_session_replacement",
    "preservation-source-context": "mcp_preservation_source_context",
    "preservation-equality-races": "mcp_preservation_equality_races",
    "preservation-active-reconfigure": "mcp_preservation_active_reconfigure",
    "preservation-late-resource-dirty": "mcp_preservation_late_resource_dirty",
    "preservation-native-entry": "mcp_preservation_native_entry",
    "preservation-selection-privacy": "mcp_preservation_selection_privacy",
    "preservation-shared-slot": "mcp_preservation_shared_slot",
}


class McpPreservationAcceptanceMixin(McpAdversarialMixin):
    def mcp_preservation(self):
        for name, method in PRESERVATION_GROUPS.items():
            self.group(name, getattr(self, method))

    def _preservation_refusal(self, peer, project, editor, descriptor, revision,
                              label, replacement=CHANGED, arguments=None, input_error=False):
        before, disks = self.state(editor, project)
        args = selectors(project, descriptor)
        args.update(revision=revision, replacement_source=replacement)
        if arguments is not None:
            args = arguments
        obj = peer.call("edit_script", args, label)
        if input_error:
            observation.require(obj["result"] is None and obj["error"]["category"] == "input" and
                                obj["error"]["application"] == "not_applied",
                                "invalid_arguments_refuse_before_dispatch_" + label)
        else:
            self.mcp_review_edit(obj, label, "refused")
        self.assert_no_effect(label, project, editor, before, disks)
        # A real editor event fence, not a delay, catches queued/late work.
        self.close_action(editor, "close_idle", frames=16)
        self.assert_no_effect(label + "_terminal", project, editor, before, disks)
        return obj

    def _preservation_dirty_unrelated(self, editor):
        self.close_action(editor, "close_human", path=CURRENT, mutation="dirty_equal")
        self.close_action(editor, "open_setup", paths=[], path=CURRENT, idle=True)
        state = self.close_action(editor, "closed_witness")["state"]
        doc = documents(state)[CURRENT]
        observation.require(doc["dirty"] and doc["has_undo"],
                            "actual_unrelated_unsaved_native_history")

    def mcp_preservation_open_history(self):
        for mutation in ("dirty_equal", "dirty_different", "dirty_disk_equal", "same_text"):
            label = "mcp_open_" + mutation
            with self.close_fixture(label, retained=True) as (project, editor, descriptor), McpPeer(self, label) as peer:
                self._preservation_dirty_unrelated(editor)
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_clean", source=SAFE)
                self.close_action(editor, "close_human", path=TARGET, mutation=mutation)
                # Settle the actual editor's Resource update without saving.
                self.close_action(editor, "open_setup", paths=[], path=TARGET, idle=True)
                human, disks = self.state(editor, project)
                doc = documents(human)[TARGET]
                observation.require(doc["dirty"] and doc["has_undo"], "real_dirty_history_" + label)
                if mutation in ("dirty_disk_equal", "same_text"):
                    observation.require(doc["B"] == disks[TARGET]["text"] == SAFE,
                                        "same_text_still_has_new_version_and_dirty_history_" + label)
                self.mcp_read(peer, project, editor, descriptor, label + "_dirty", revision=False)
                self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label)
                # Probe real surviving history, rather than only an undoable flag.
                original = doc["B"]
                self.close_action(editor, "close_human", path=TARGET, mutation="undo")
                undone, _ = self.state(editor, project)
                observation.require(documents(undone)[TARGET]["B"] != original,
                                    "refusal_preserves_usable_prior_undo_" + label)
                self.close_action(editor, "close_human", path=TARGET, mutation="redo")
                redone, now = self.state(editor, project)
                observation.require(documents(redone)[TARGET]["B"] == original and now == disks and
                                    documents(redone)[CURRENT] == documents(human)[CURRENT],
                                    "refusal_preserves_prior_redo_and_unrelated_history_" + label)
                self.record_witness(label + "_history", human, disks, redone, now, editor)

    def mcp_preservation_revisions(self):
        for opened in (False, True):
            fixture = self.close_fixture if opened else self.closed_fixture
            label = "mcp_revision_" + ("open" if opened else "closed")
            with fixture(label) as (project, editor, descriptor), McpPeer(self, label) as peer:
                self._preservation_dirty_unrelated(editor)
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                wrong = peer.call("read_script", selectors(project, descriptor, script_path=BACKGROUND), label + "_other")["result"]
                observation.require(wrong["revision"] is not None and wrong["revision"] != basis["revision"],
                                    "revision_is_target_bound_" + label)
                for name, token in (("malformed", "sr1:bogus"), ("fabricated", "sr1:" + sha(SAFE)),
                                    ("wrong_target", wrong["revision"])):
                    self._preservation_refusal(peer, project, editor, descriptor, token, label + "_" + name,
                                              input_error=name == "malformed")
                args = selectors(project, descriptor)
                args["replacement_source"] = CHANGED
                self._preservation_refusal(peer, project, editor, descriptor, None, label + "_missing",
                                          arguments=args, input_error=True)
                before, disks = self.state(editor, project)
                self.mcp_edit(peer, project, descriptor, basis["revision"], CHANGED, label + "_first", "verified_changed")
                if not opened:
                    self.assert_closed_success(label + "_first", project, editor, before, disks, CHANGED, changed=True)
                else:
                    after, now = self.state(editor, project)
                    observation.require(documents(after)[TARGET]["B"] == documents(after)[TARGET]["R"] == now[TARGET]["text"] == CHANGED and
                                        documents(after)[CURRENT] == documents(before)[CURRENT],
                                        "open_effect_converges_without_losing_unrelated_history")
                    self.record_witness(label + "_first", before, disks, after, now, editor)
                self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label + "_stale")
                self.mcp_survivor_read(peer, project, editor, descriptor, label + "_survivor")

    def _preservation_transition(self, project, editor, mutation):
        if mutation == "parent_namespace":
            parent = project / "scripts"
            old = parent.stat().st_ino
            displaced = project / "scripts.displaced"
            os.rename(parent, displaced)
            parent.mkdir()
            for child in displaced.iterdir():
                os.rename(child, parent / child.name)
            observation.require(parent.stat().st_ino != old, "actual_parent_namespace_identity_replacement")
        elif mutation.startswith("open_"):
            self.close_action(editor, "close_human", path=TARGET, mutation=mutation.removeprefix("open_"))
            if mutation == "open_close":
                self.close_action(editor, "close_refs")
        else:
            self.closed_transition(project, editor, mutation)

    def mcp_preservation_identities(self):
        closed = ("same_text", "namespace", "parent_namespace", "cache_appear", "cache_replace",
                  "target_open", "aba", "unrelated_epoch", "reconfigure")
        for mutation in closed + ("open_same_text", "open_reopen", "open_replace", "open_close"):
            opened = mutation.startswith("open_")
            label = "mcp_identity_" + mutation
            fixture = self.close_fixture if opened else self.closed_fixture
            kwargs = {} if opened else {"cached": mutation == "cache_replace"}
            with fixture(label, **kwargs) as (project, editor, descriptor), McpPeer(self, label) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                self._preservation_transition(project, editor, mutation)
                self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label)
                self.mcp_survivor_read(peer, project, editor, descriptor, label + "_survivor")

    def mcp_preservation_cache_absence(self):
        with self.closed_fixture("mcp_cache_detach", cached=True) as (project, editor, descriptor), McpPeer(self, "cache_detach") as peer:
            basis = self.mcp_read(peer, project, editor, descriptor, "cache_detach_basis", source=SAFE)
            # A real actor removes the cached Resource's path association.
            # The old present-R revision must not silently switch to absent R.
            self.close_action(editor, "closed_resource", mutation="detach")
            actual, _ = self.state(editor, project)
            observation.require(not actual["cache_has"] and not actual["cached_id"] and
                                actual["cache_type"] is None, "actual_canonical_cache_absence_after_detach")
            self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], "cache_detach_refused")
            self.mcp_survivor_read(peer, project, editor, descriptor, "cache_detach_survivor")

    def mcp_preservation_resource_safety(self):
        for mutation in ("divergent", "dirty", "equal_dirty"):
            label = "mcp_R_" + mutation
            with self.closed_fixture(label, cached=True) as (project, editor, descriptor), McpPeer(self, label) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_clean", source=SAFE)
                state = self.close_action(editor, "closed_resource", mutation=mutation)["state"]
                observation.require(state["cached_R"] != SAFE if mutation == "divergent" else state["cached_edited"] is True,
                                    "actual_unsafe_loaded_resource_" + label)
                if mutation == "equal_dirty":
                    observation.require(state["cached_R"] == SAFE, "actual_matching_text_dirty_R")
                self.mcp_read(peer, project, editor, descriptor, label + "_read", revision=False, source=SAFE)
                for name, token in (("prior", basis["revision"]), ("digest", "sr1:" + sha(SAFE))):
                    self._preservation_refusal(peer, project, editor, descriptor, token, label + "_" + name)
        for fault in ("missing_cache_getters", "missing_roster", "missing_context", "epoch_overflow", "epoch_reset"):
            label = "mcp_unknown_" + fault
            with self.closed_fixture(label, faults=True) as (project, editor, descriptor), McpPeer(self, label) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                self.close_action(editor, "closed_fault", fault=fault)
                fresh = self.mcp_read(peer, project, editor, descriptor, label + "_read", revision=fault == "epoch_reset")
                if fault == "epoch_reset":
                    observation.require(fresh["revision"] != basis["revision"], "epoch_identity_never_reused")
                self.close_action(editor, "closed_fault", fault=fault)
                self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label)

    def mcp_preservation_source_context(self):
        replacements = {"over_bound": "#" * (SOURCE_LIMIT + 1), "nul": SAFE + "\0",
                        "cr": SAFE.replace("\n", "\r\n"), "bom": "\ufeff" + SAFE,
                        "parse": "extends RefCounted\nfunc value(:\n", "tool": "@tool\n" + SAFE,
                        "global": "class_name McpUnapprovedGlobal\n" + SAFE,
                        "preload": SAFE + 'const BAD = preload("res://scripts/other.gd")\n'}
        for opened in (False, True):
            fixture = self.close_fixture if opened else self.closed_fixture
            label = "mcp_source_" + ("open" if opened else "closed")
            with fixture(label) as (project, editor, descriptor), McpPeer(self, label) as peer:
                self._preservation_dirty_unrelated(editor)
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                for name, replacement in replacements.items():
                    self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label + "_" + name,
                                              replacement, input_error=name in ("over_bound", "nul", "cr", "bom"))
                if opened:
                    context = self.native_action(editor, "native_edit_unsupported_warning_context")
                    observation.require(context["type"] == 4, "actual_unreproducible_open_warning_context")
                else:
                    self.close_action(editor, "close_config", setting="external_editor", value=True)
                self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], label + "_context")
                self._preservation_refusal(peer, project, editor, descriptor, "sr1:" + sha(SAFE), label + "_context_digest")
        for name, source in (("original_tool", "@tool\n" + SAFE),
                             ("original_preload", SAFE + 'const BAD = preload("res://scripts/other.gd")\n'),
                             ("original_invalid", "extends RefCounted\nfunc value(:\n")):
            with self.closed_fixture("mcp_" + name, source=source) as (project, editor, descriptor), McpPeer(self, name) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, name + "_read", revision=name == "original_invalid", source=source)
                self._preservation_refusal(peer, project, editor, descriptor,
                                          basis["revision"] or "sr1:" + sha(source), name)
        with self.closed_fixture("mcp_profile", cached=True) as (project, editor, descriptor), McpPeer(self, "profile") as peer:
            basis = self.mcp_read(peer, project, editor, descriptor, "profile_basis", source=SAFE)
            self.close_action(editor, "closed_save_profile", key="trim_trailing_whitespace_on_save", value=True)
            self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], "profile_transition",
                                      replacement=CHANGED + "# trailing   \n")
            fresh = self.mcp_read(peer, project, editor, descriptor, "profile_fresh", source=SAFE)
            self._preservation_refusal(peer, project, editor, descriptor, fresh["revision"], "profile_current_guard",
                                      replacement=CHANGED + "# trailing   \n")

    def mcp_preservation_equality_races(self):
        mutations = ("same_text", "namespace", "parent_namespace", "cache_appear", "cache_replace",
                     "target_open", "aba", "unrelated_epoch", "reconfigure", "context")
        for stage in ("prepare", "apply"):
            for mutation in mutations:
                if stage == "apply" and mutation == "reconfigure":
                    continue  # Active reconfiguration is refused, tested separately.
                label = "mcp_race_" + stage + "_" + mutation
                with self.closed_fixture(label, cached=mutation == "cache_replace") as (project, editor, descriptor), McpPeer(self, label) as peer:
                    self._preservation_dirty_unrelated(editor)
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                    self.close_action(editor, "closed_arm", stage=stage)
                    call = peer.start("edit_script", dict(selectors(project, descriptor), revision=basis["revision"], replacement_source=CHANGED))
                    self.wait_mcp_barrier(editor, stage, peer, call)
                    self._preservation_transition(project, editor, mutation)
                    before, disks = self.state(editor, project)
                    self.close_action(editor, "closed_release")
                    self.mcp_review_edit(peer.finish(call, label), label, "refused")
                    self.assert_no_effect(label, project, editor, before, disks)
                    self.close_action(editor, "close_idle", frames=16)
                    self.assert_no_effect(label + "_terminal", project, editor, before, disks)
                    self.mcp_survivor_read(peer, project, editor, descriptor, label + "_survivor")
        for stage in ("prepare", "apply"):
            for mutation in ("same_text", "dirty_equal", "reopen", "close", "resource_edited"):
                if stage == "apply" and mutation == "resource_edited":
                    continue  # The focused native dirty-state regression owns this boundary.
                label = "mcp_open_race_" + stage + "_" + mutation
                with self.close_fixture(label) as (project, editor, descriptor), McpPeer(self, label) as peer:
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                    self.native_action(editor, "native_edit_hold", stage=stage)
                    call = peer.start("edit_script", dict(selectors(project, descriptor), revision=basis["revision"], replacement_source=CHANGED))
                    self.wait_mcp_barrier(editor, "edit:" + stage, peer, call)
                    self.close_action(editor, "close_human", path=TARGET, mutation=mutation)
                    before, disks = self.state(editor, project)
                    self.native_action(editor, "native_edit_release")
                    self.mcp_review_edit(peer.finish(call, label), label, "refused")
                    self.assert_no_effect(label, project, editor, before, disks)
                    self.close_action(editor, "close_idle", frames=16)
                    self.assert_no_effect(label + "_terminal", project, editor, before, disks)

    def mcp_preservation_active_reconfigure(self):
        with self.closed_fixture("mcp_active_reconfigure", cached=True) as (project, editor, descriptor), McpPeer(self, "active_reconfigure") as peer:
            basis = self.mcp_read(peer, project, editor, descriptor, "active_reconfigure_basis", source=SAFE)
            self.close_action(editor, "closed_arm", stage="apply")
            call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                 revision=basis["revision"], replacement_source=CHANGED))
            self.wait_mcp_barrier(editor, "apply", peer, call)
            before, disks = self.state(editor, project)
            attempted = self.close_action(editor, "closed_try_reconfigure")
            observation.require(attempted["accepted"] is False, "native_owner_refuses_active_reconfiguration")
            self.close_action(editor, "closed_release")
            self.mcp_review_edit(peer.finish(call, "active_reconfigure_owner"), "active_reconfigure_owner", "verified_changed")
            after, now = self.assert_closed_success("active_reconfigure_owner", project, editor,
                                                    before, disks, CHANGED, changed=True)
            self.close_action(editor, "close_idle", frames=16)
            self.assert_no_effect("active_reconfigure_terminal", project, editor, after, now)

    def mcp_preservation_late_resource_dirty(self):
        for stage in ("apply", "buffer_applied", "resource_applied", "content_persisted", "mtime_restored", "edited_cleared"):
            label = "mcp_late_R_dirty_" + stage
            with self.close_fixture(label) as (project, editor, descriptor), McpPeer(self, label) as peer:
                self._preservation_dirty_unrelated(editor)
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                self.native_action(editor, "native_edit_hold", stage=stage)
                call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                     revision=basis["revision"], replacement_source=CHANGED))
                self.wait_mcp_barrier(editor, "edit:" + stage, peer, call)
                prior, _ = self.state(editor, project)
                observation.require(prior["cached_edited"] is False, "native_owned_prefix_has_clean_R_" + stage)
                self.close_action(editor, "close_human", path=TARGET, mutation="resource_edited")
                before, disks = self.state(editor, project)
                observation.require(before["cached_edited"] is True and
                                    before["cached_R"] == prior["cached_R"], "actual_late_equal_text_dirty_R_" + stage)
                self.native_action(editor, "native_edit_release")
                root = peer.finish(call, label)
                after, now = self.state(editor, project)
                self.record_witness(label, before, disks, after, now, editor)
                self.mcp_review_edit(root, label, "refused" if stage == "apply" else "applied_unverified")
                observation.require(source_free(before, disks) == source_free(after, now),
                                    "late_dirty_R_preserves_all_authorities_and_history_" + stage)
                self.close_action(editor, "close_idle", frames=16)
                self.assert_no_effect(label + "_terminal", project, editor, before, disks)
                self.mcp_survivor_read(peer, project, editor, descriptor, label + "_fresh")

    def mcp_preservation_native_entry(self):
        for action in ("open", "aba", "newer_resource", "cache_appear", "cache_replace", "equal_dirty", "profile", "context"):
            label = "mcp_native_entry_" + action
            with self.closed_fixture(label, cached=action not in ("cache_appear", "open", "aba"), faults=True) as (project, editor, descriptor), McpPeer(self, label) as peer:
                basis = self.mcp_read(peer, project, editor, descriptor, label + "_basis", source=SAFE)
                before, disks = self.state(editor, project)
                self.close_action(editor, "closed_arm", callback_stage="before_apply", callback_action=action)
                self.mcp_edit(peer, project, descriptor, basis["revision"], CHANGED, label, "refused")
                witness = self.close_action(editor, "closed_witness")
                callbacks = [event for event in witness["events"] if event["stage"] == "callback:before_apply"]
                observation.require(len(callbacks) == 1 and callbacks[0]["action"] == action and
                                    callbacks[0]["request_id"] == self.last_workflow_request,
                                    "actual_native_callback_domain_binding_" + label)
                entered = [event["stage"] for event in witness["events"]]
                observation.require("before_apply" in entered and not any(stage in entered for stage in
                                    ("after_resource", "after_write", "after_mtime")),
                                    "native_entry_refusal_has_no_effect_prefix_" + label)
                after, now = self.state(editor, project)
                observation.require(source_free(callbacks[0]["state"], disks) == source_free(after, now),
                                    "native_entry_preserves_callback_survivor_" + label)
                observation.json_file(self.artifacts / (label + "-callbacks.json"),
                                      [{key: event[key] for key in ("request_id", "stage", "action")} for event in callbacks])
                self.cases[-1]["callback_evidence"] = label + "-callbacks.json"
                self.record_witness(label, before, disks, after, now, editor)
                self.close_action(editor, "close_idle", frames=16)
                self.assert_no_effect(label + "_terminal", project, editor, after, now)

    def _preservation_private(self, obj, witnesses, descriptors, label):
        raw = json.dumps(obj)
        result = obj["result"]
        if isinstance(result, dict) and "inventory" in result:
            observation.require(result["inventory"] is None, "no_unauthorized_candidate_inventory_" + label)
        if isinstance(result, dict) and "outcome" in result:
            self.mcp_review_edit(obj, label, "refused")
            observation.require(result["outcome"]["application"] == "not_applied",
                                "unresolved_or_denied_target_has_no_application_" + label)
        for _, disks in witnesses:
            for disk in disks.values():
                observation.require(disk["text"] not in raw and disk["sha256"] not in raw,
                                    "no_unauthorized_source_or_digest_" + label)
        for descriptor in descriptors:
            for key in ("endpoint", "token"):
                value = descriptor.get(key)
                if isinstance(value, str):
                    observation.require(value not in raw, "no_private_routing_disclosure_" + label)

    def mcp_preservation_selection_privacy(self):
        source = SAFE + "# MCP_PRESERVATION_PRIVATE_TARGET\n"
        self.source_markers.add(b"MCP_PRESERVATION_PRIVATE_TARGET")
        with self.closed_fixture("mcp_privacy", cached=True, source=source) as (project, editor, descriptor), McpPeer(self, "privacy") as peer:
            basis = self.mcp_read(peer, project, editor, descriptor, "privacy_permitted", source=source)
            second = self.start_editor(project)
            try:
                pair = observation.wait_for(lambda: self.descriptors(project) if len(self.descriptors(project)) == 2 else None,
                                            "mcp_two_real_authenticated_sessions")
                self.close_action(second, "close_setup", paths=[CURRENT, BACKGROUND], selected=CURRENT)
                witnesses = [self.state(owner, project) for owner in (editor, second)]
                for tool in ("read_script", "discover_scripts", "edit_script"):
                    args = selectors(project, descriptor, select_session=False)
                    if tool == "discover_scripts": args.pop("script_path")
                    if tool == "edit_script": args.update(revision=basis["revision"], replacement_source=CHANGED)
                    obj = peer.call(tool, args, "ambiguous_" + tool)
                    observation.require(obj["error"] is not None or
                                        (obj["result"].get("source") is None and obj["result"].get("revision") is None),
                                        "ambiguous_session_never_guesses_" + tool)
                    self._preservation_private(obj, witnesses, pair, "ambiguous_" + tool)
                    for owner, (before, disks) in zip((editor, second), witnesses):
                        self.assert_no_effect("ambiguous_" + tool, project, owner, before, disks)
                ended = next(item for item in pair if item["session_id"] != descriptor["session_id"])
            finally:
                self.close_editor(second)
                self.editors.remove(second)
            observation.wait_for(lambda: all(item["session_id"] != ended["session_id"] for item in self.descriptors(project)),
                                 "explicit_second_session_really_ended")
            for tool in ("read_script", "discover_scripts", "edit_script"):
                before, disks = self.state(editor, project)
                args = selectors(project, ended)
                if tool == "discover_scripts": args.pop("script_path")
                if tool == "edit_script": args.update(revision=basis["revision"], replacement_source=CHANGED)
                obj = peer.call(tool, args, "ended_" + tool)
                self._preservation_private(obj, [(before, disks)], [descriptor], "ended_" + tool)
                observation.require(obj["result"] is None or (obj["result"].get("source") is None and obj["result"].get("revision") is None),
                                    "ended_selector_never_falls_back_" + tool)
                self.assert_no_effect("ended_" + tool, project, editor, before, disks)
            outside = project.parent / ("mcp-outside-" + secrets.token_hex(8) + ".gd")
            outside_source = SAFE + "# MCP_PRESERVATION_OUTSIDE_SECRET\n"
            self.source_markers.add(b"MCP_PRESERVATION_OUTSIDE_SECRET")
            outside.write_text(outside_source)
            escape = project / "scripts/mcp_escape.gd"
            escape.symlink_to(outside)
            try:
                for path in ("res://../" + outside.name, str(outside), "res://scripts/mcp_escape.gd"):
                    for tool in ("read_script", "edit_script"):
                        before, disks = self.state(editor, project)
                        args = selectors(project, descriptor, script_path=path)
                        if tool == "edit_script": args.update(revision=basis["revision"], replacement_source=CHANGED)
                        obj = peer.call(tool, args, "confinement_" + tool + "_" + secrets.token_hex(4))
                        self._preservation_private(obj, [(before, disks)], [descriptor], "confinement")
                        observation.require(outside_source not in json.dumps(obj) and sha(outside_source) not in json.dumps(obj) and
                                            (obj["result"] is None or obj["result"].get("source") is None),
                                            "confinement_has_no_outside_source_or_digest")
                        observation.require(outside.read_text() == outside_source, "outside_real_file_untouched")
                        self.assert_no_effect("confinement_" + tool, project, editor, before, disks)
            finally:
                escape.unlink()
                outside.unlink()
            target = project / "scripts/subject.gd"
            before, disks = self.state(editor, project)
            mode = target.stat().st_mode & 0o777
            # An independently owned read descriptor remains usable after denial;
            # never relax permissions while the product executes its checks.
            with target.open("rb") as witness:
                target.chmod(0)
                denied_stat = os.fstat(witness.fileno())
                try:
                    denied = peer.call("read_script", selectors(project, descriptor), "denied_read")
                    observation.require(denied["result"]["source"] is None and denied["result"]["revision"] is None and
                                        denied["result"]["state"]["status"] == "denied_access" and
                                        denied["result"]["state"]["sources"] is None,
                                        "denied_read_suppresses_all_authorities")
                    self._preservation_private(denied, [(before, disks)], [descriptor], "denied_read")
                    discovery_args = selectors(project, descriptor)
                    discovery_args.pop("script_path")
                    discovery = peer.call("discover_scripts", discovery_args, "denied_discovery")
                    inventory = discovery["result"]["inventory"] if discovery["result"] is not None else None
                    observation.require(inventory is None or TARGET not in inventory["entries"],
                                        "denied_script_never_published_as_authorized_inventory")
                    observation.require(source not in json.dumps(discovery) and sha(source) not in json.dumps(discovery),
                                        "denied_discovery_has_no_source_hash_or_diagnostic_disclosure")
                    edit = peer.call("edit_script", dict(selectors(project, descriptor),
                                     revision=basis["revision"], replacement_source=CHANGED), "denied_edit")
                    result = self.mcp_review_edit(edit, "denied_edit", "refused")
                    observation.require(result["outcome"]["reason"] == "denied_access" and
                                        result["outcome"]["application"] == "not_applied",
                                        "denied_edit_never_authorizes_from_matching_revision")
                    self._preservation_private(edit, [(before, disks)], [descriptor], "denied_edit")
                    self.close_action(editor, "close_idle", frames=16)
                    after = self.close_action(editor, "closed_witness")["state"]
                    now_stat = os.fstat(witness.fileno())
                    observation.require(witness.read().decode("utf-8") == disks[TARGET]["text"] and
                                        all(getattr(now_stat, key) == getattr(denied_stat, key) for key in
                                            ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns", "st_mode")) and
                                        source_free(before, {}) == source_free(after, {}),
                                        "denial_preserves_real_disk_descriptor_resource_roster_history")
                finally:
                    target.chmod(mode)
            final, now = self.state(editor, project)
            observation.require(all(now[path] == disk for path, disk in disks.items() if path != TARGET),
                                "denial_preserves_unrelated_disk_history")
            self.record_witness("denied_edit", before, disks, final, now, editor)
            self.mcp_survivor_read(peer, project, editor, descriptor, "privacy_survivor")

    def mcp_preservation_shared_slot(self):
        with self.closed_fixture("mcp_slot") as (project, editor, descriptor), McpPeer(self, "slot_owner") as owner, McpPeer(self, "slot_contender") as contender:
            self._preservation_dirty_unrelated(editor)
            basis = self.mcp_read(owner, project, editor, descriptor, "slot_basis", source=SAFE)
            self.close_action(editor, "closed_arm", stage="apply")
            call = owner.start("edit_script", dict(selectors(project, descriptor), revision=basis["revision"], replacement_source=CHANGED))
            self.wait_mcp_barrier(editor, "apply", owner, call)
            before, disks = self.state(editor, project)
            rejected = contender.start("edit_script", dict(selectors(project, descriptor), revision=basis["revision"], replacement_source=SAFE + "# NEVER_QUEUED\n"))
            obj = contender.finish(rejected, "slot_independent_connection_refused")
            self.mcp_review_edit(obj, "slot_independent_connection_refused", "refused")
            observation.require(obj["result"]["outcome"]["reason"] in ("busy", "slot_busy", "editor_busy"),
                                "independent_server_reaches_actual_editor_slot_not_adapter_capacity")
            self.assert_no_effect("slot_contender", project, editor, before, disks)
            # Cancellation of the rejected connection's ID (and the other
            # connection's matching ID) must not release/cancel the real owner.
            contender.cancel(rejected)
            contender.send({"jsonrpc": "2.0", "method": "notifications/cancelled", "params": {"requestId": call["id"]}})
            local = self.start_workflow(project, descriptor, "edit", revision=basis["revision"], replacement_source=SAFE)
            result = self.complete_workflow(local, "slot_local_workflow_refused", "edit")
            self.review_workflow_edit(result, "slot_local_workflow_refused", "refused")
            self.assert_no_effect("slot_local_workflow", project, editor, before, disks)
            for operation in ("open_begin", "close_begin"):
                stream, request_id = self.authenticated_peer(descriptor)
                with stream:
                    stream.sendall(observation.packet([6, operation, request_id, descriptor["session_id"], descriptor["project_root"], TARGET, 9000]))
                    reply, _ = observation.receive(stream)
                    observation.require(reply.get("reason") == "slot_busy", "shared_native_slot_excludes_local_" + operation)
            self.close_action(editor, "closed_release")
            self.mcp_review_edit(owner.finish(call, "slot_owner_completed"), "slot_owner_completed", "verified_changed")
            after, now = self.assert_closed_success("slot_owner_completed", project, editor, before, disks, CHANGED, changed=True)
            self.close_action(editor, "close_idle", frames=16)
            self.assert_no_effect("slot_no_queue_replay_cross_cancel", project, editor, after, now)
            self.mcp_survivor_read(contender, project, editor, descriptor, "slot_survivor")

    def mcp_preservation_session_replacement(self):
        with self.closed_fixture("mcp_session_replacement", cached=True) as (project, editor, descriptor), McpPeer(self, "session_replacement") as peer:
            self._preservation_dirty_unrelated(editor)
            basis = self.mcp_read(peer, project, editor, descriptor, "session_old_basis", source=SAFE)
            before, disks = self.state(editor, project)
            # Deliberate adversarial session replacement is a perturbation, never
            # a repair path invoked by an edit or by a refusal handler.
            self.close_action(editor, "close_lifetime", control="disable")
            self.close_action(editor, "close_lifetime", control="enable")
            replacement = observation.wait_for(lambda: next((item for item in self.descriptors(project)
                if item["session_id"] != descriptor["session_id"]), None), "actual_replaced_editor_session")
            self._preservation_refusal(peer, project, editor, descriptor, basis["revision"], "old_explicit_session_no_fallback")
            self._preservation_refusal(peer, project, editor, replacement, basis["revision"], "old_revision_new_session_refused")
            self.assert_no_effect("session_replacement_preserves_human_history", project, editor, before, disks)
            fresh = self.mcp_read(peer, project, editor, replacement, "session_replacement_fresh", source=SAFE)
            observation.require(fresh["revision"] != basis["revision"], "same_source_new_session_has_distinct_revision")

    def mcp_preservation_prior_native_history(self):
        with self.close_fixture("mcp_prior_native_history") as (project, editor, descriptor), McpPeer(self, "prior_native_history") as peer:
            self._preservation_dirty_unrelated(editor)
            basis = self.mcp_read(peer, project, editor, descriptor, "prior_native_basis", source=SAFE)
            result = self.mcp_edit(peer, project, descriptor, basis["revision"], CHANGED, "prior_native_applied", "verified_changed")
            observation.require(result["outcome"]["history"] == "native_complex_edit",
                                "prior_actual_native_transaction_claims_history")
            after, disks = self.state(editor, project)
            doc = documents(after)[TARGET]
            observation.require(doc["B"] == doc["R"] == disks[TARGET]["text"] == CHANGED and doc["has_undo"],
                                "prior_native_effect_independently_converged")
            fresh = self.mcp_read(peer, project, editor, descriptor, "prior_native_fresh", source=CHANGED)
            self._preservation_refusal(peer, project, editor, descriptor, fresh["revision"], "prior_native_invalid_desired",
                                      replacement="extends RefCounted\nfunc value(:\n")
            self.close_action(editor, "close_human", path=TARGET, mutation="undo")
            undone, undone_disks = self.state(editor, project)
            observation.require(documents(undone)[TARGET]["B"] == SAFE and undone_disks == disks and
                                documents(undone)[CURRENT] == documents(after)[CURRENT],
                                "refusal_preserves_prior_native_transaction_real_undo")
            self.close_action(editor, "close_human", path=TARGET, mutation="redo")
            redone, now = self.state(editor, project)
            observation.require(documents(redone)[TARGET]["B"] == CHANGED and now == disks and
                                documents(redone)[CURRENT] == documents(after)[CURRENT],
                                "refusal_preserves_prior_native_transaction_real_redo")
            self.record_witness("prior_native_history", after, disks, redone, now, editor)
