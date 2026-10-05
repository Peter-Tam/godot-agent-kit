"""T003 real-editor MCP interruption evidence; fixture controls are not consumers.

Every recovery addresses the original explicit target.  Missing transport output
is recorded as unavailable delivery, never converted into an operation outcome.
"""
from __future__ import annotations

from contextlib import contextmanager
import json
import os
import signal
import subprocess
import time

import run_observation as observation
from close_native_acceptance import documents
from closed_script_acceptance import CHANGED, source_free
from mcp_peer import McpPeer, McpAdversarialMixin, selectors
from run_script_close import TARGET, CURRENT, SAFE
from run_script_edit import sha

PROFILES = ("open", "cached-closed", "absent-closed")
NEWER = "extends RefCounted\n# CLOSED_NEWER_WORK\n"
DESIRED = "# MCP_INTERRUPTION_PRIVATE_SOURCE_SENTINEL\n" + CHANGED

INTERRUPTION_GROUPS = {
    "interruption-cancellation": "mcp_interruption_cancellation",
    "interruption-original-deadline": "mcp_interruption_deadline",
    "interruption-unresponsive-editor": "mcp_interruption_unresponsive",
    "interruption-disconnect-and-signals": "mcp_interruption_shutdown",
    "interruption-editor-channels": "mcp_interruption_channels",
    "interruption-known-effects": "mcp_interruption_effects",
    "interruption-newer-work": "mcp_interruption_newer_work",
    "interruption-sticky-denial": "mcp_interruption_denial",
    "interruption-lost-output": "mcp_interruption_output",
    "interruption-worker-loss": "mcp_interruption_worker_loss",
    "interruption-blocked-output": "mcp_interruption_blocked_output",
    "interruption-malformed-private-reply": "mcp_interruption_malformed",
}


class McpInterruptionMixin(McpAdversarialMixin):
    @contextmanager
    def _interruption_fixture(self, label, profile, *, faults=False):
        self.source_markers.add(b"MCP_INTERRUPTION_PRIVATE_SOURCE_SENTINEL")
        with self.closed_fixture(label, cached=profile != "absent-closed", faults=faults) as fixture:
            project, editor, descriptor = fixture
            if profile == "open":
                self.close_action(editor, "close_human", path=TARGET, mutation="open")
                if faults:
                    self.native_action(editor, "native_edit_fixture_activate")
            # A genuine unrelated human history is part of every survivor witness.
            self.close_action(editor, "close_human", path=CURRENT, mutation="dirty_equal")
            self.close_action(editor, "open_setup", paths=[], path=CURRENT, idle=True)
            state, _ = self.state(editor, project)
            observation.require(state["current"]["dirty"] and state["current"]["has_undo"] and
                                (TARGET in state["open_paths"]) == (profile == "open"),
                                "interruption_admitted_profile_and_human_history_" + label)
            yield fixture

    @staticmethod
    def _interruption_stages(profile):
        return (("prepare", "apply", "buffer_applied", "resource_applied", "content_persisted",
                 "mtime_restored", "edited_cleared", "verify:post_change", "recheck:post_change")
                if profile == "open" else ("prepare", "apply", "verify:post_change"))

    def _interruption_arm(self, editor, profile, stage):
        self.close_action(editor, "closed_arm")
        if profile == "open":
            self.native_action(editor, "native_edit_hold", stage=stage)
        else:
            self.close_action(editor, "closed_arm", stage=stage)

    def _interruption_release(self, editor, profile):
        self.native_action(editor, "native_edit_release") if profile == "open" else \
            self.close_action(editor, "closed_release")

    def _interruption_start(self, peer, project, editor, descriptor, profile, stage, label):
        basis = self.mcp_read(peer, project, editor, descriptor, label + "-basis", source=SAFE)
        self._interruption_arm(editor, profile, stage)
        call = peer.start("edit_script", dict(selectors(project, descriptor),
                                             revision=basis["revision"], replacement_source=DESIRED))
        self.wait_mcp_barrier(editor, "edit:" + stage if profile == "open" else stage, peer, call)
        return call

    def _interruption_review(self, root, call, stage, label):
        expected = ("refused" if stage == "prepare" else
                    ("refused", "application_unknown") if stage == "apply" else
                    ("applied_unverified", "application_unknown"))
        result = self.mcp_review_edit(root, label, expected)
        outcome = result["outcome"]
        observation.require(root["request_id"] == call["domain_request_id"] and
                            root["request_id"] != call["id"] and
                            outcome.get("reason") is not None and
                            outcome.get("stage") is not None and
                            (outcome.get("safe_next_action") if result["mode"] == "open" else
                             outcome.get("next_action")) is not None,
                            "interruption_correlation_cause_stage_recovery_" + label)
        if stage not in ("prepare", "apply"):
            observation.require(outcome["application"] != "not_applied",
                                "interruption_real_effect_never_refused_" + label)
        return result

    def _interruption_survivor(self, project, editor, before, disks, label, *, no_effect=False,
                              human_selection=False):
        observation.require(editor["process"].poll() is None, "interruption_never_kills_editor_" + label)
        after, now = self.state(editor, project)
        prior_docs, later_docs = documents(before), documents(after)
        observation.require(all(later_docs.get(path) == doc for path, doc in prior_docs.items() if path != TARGET) and
                            all(now.get(path) == disk for path, disk in disks.items() if path != TARGET) and
                            (human_selection or before["selection"] == after["selection"]),
                            "interruption_unrelated_human_history_and_selection_" + label)
        if no_effect:
            observation.require(source_free(before, disks) == source_free(after, now),
                                "interruption_independent_no_effect_" + label)
        self.record_witness(label, before, disks, after, now, editor)
        self.close_action(editor, "close_idle", frames=16)
        self.assert_no_effect(label + "-terminal", project, editor, after, now)
        return after, now

    def _interruption_recover(self, project, editor, descriptor, label):
        with McpPeer(self, label + "-recovery") as recovery:
            self.mcp_survivor_read(recovery, project, editor, descriptor, label + "-fresh-original-target")

    def _interruption_matrix(self, controls, *, stages=None):
        for profile in PROFILES:
            for stage in stages or self._interruption_stages(profile):
                for control in controls:
                    label = "mcp-" + profile + "-" + stage.replace(":", "-") + "-" + control
                    with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                        with McpPeer(self, label) as peer:
                            call = self._interruption_start(peer, project, editor, descriptor, profile, stage, label)
                            before, disks = self.state(editor, project)
                            if control == "cancel":
                                peer.cancel(call)
                                # Cancellation may suppress output while stdio stays
                                # open. EOF bounds cleanup; no timeout is delivery.
                                peer.eof()
                            elif control == "eof":
                                peer.eof()
                            elif control in ("sigint", "sigterm"):
                                peer.process.send_signal(signal.SIGINT if control == "sigint" else signal.SIGTERM)
                            elif control == "disable":
                                self.close_action(editor, "close_lifetime", control="disable")
                            elif control == "channel-loss":
                                if profile == "open":
                                    self.action(editor, "disable")
                                else:
                                    self.close_action(editor, "closed_disconnect")
                            elif control == "native-loss":
                                self.close_action(editor, "closed_reconfigure")
                                # Drop the old native attempt, not the editor.
                                # Release only after revocation has returned.
                                self._interruption_release(editor, profile)
                            root = peer.finish(call, label, delivery_optional=control in
                                               ("cancel", "eof", "sigint", "sigterm"))
                            if root is not None:
                                self._interruption_review(root, call, stage, label)
                            if control in ("cancel", "eof", "sigint", "sigterm"):
                                peer.wait_exit(bound=max(0.01, call["started"] + 11 - time.monotonic()))
                            self._interruption_release(editor, profile)
                            self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                        self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_cancellation(self):
        self._interruption_matrix(("cancel",))

    def mcp_interruption_deadline(self):
        # finish consumes output against the original call clock while the real
        # editor barrier stays closed; release occurs only after terminal refusal.
        self._interruption_matrix(("deadline",))
        for profile in PROFILES:
            label = "mcp-original-acquisition-clock-" + profile
            with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                with McpPeer(self, label) as peer:
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "-basis", source=SAFE)
                    before, disks = self.state(editor, project)
                    if profile == "open":
                        self.action(editor, "hold_observe")
                    else:
                        self.close_action(editor, "closed_arm", stage="inspect")
                    call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                         revision=basis["revision"], replacement_source=DESIRED))
                    if profile == "open":
                        self.wait_barrier(editor, "observe", peer.process)
                    else:
                        self.wait_mcp_barrier(editor, "inspect", peer, call, acquisition=True)
                    root = peer.finish(call, label)
                    result = self.mcp_review_edit(root, label, "refused")
                    observation.require(result["outcome"]["reason"] == "timeout" and
                                        result["outcome"]["application"] == "not_applied",
                                        "acquisition_consumes_original_edit_clock_" + label)
                    if profile == "open":
                        self.action(editor, "release_hold")
                    else:
                        self._interruption_release(editor, profile)
                    self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_shutdown(self):
        self._interruption_matrix(("eof", "sigint", "sigterm"), stages=("prepare", "apply", "verify:post_change"))

    def mcp_interruption_channels(self):
        self._interruption_matrix(("disable", "channel-loss"), stages=("prepare", "apply", "verify:post_change"))
        self._interruption_matrix(("native-loss",), stages=("apply", "verify:post_change"))

    def mcp_interruption_unresponsive(self):
        for profile in PROFILES:
            label = "mcp-unresponsive-" + profile
            with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                with McpPeer(self, label) as peer:
                    basis = self.mcp_read(peer, project, editor, descriptor, label + "-basis", source=SAFE)
                    before, disks = self.state(editor, project)
                    editor["process"].send_signal(signal.SIGSTOP)
                    try:
                        read = peer.call("read_script", selectors(project, descriptor), label + "-read")
                        observation.require(read["error"] is None and read["result"]["source"] is None and
                                            read["result"]["revision"] is None and
                                            read["result"]["state"]["status"] == "timeout",
                                            "unresponsive_read_five_second_terminal_" + profile)
                        scope = selectors(project, descriptor)
                        scope.pop("script_path")
                        discovery = peer.call("discover_scripts", scope, label + "-discover")
                        observation.require(discovery["error"] is None and
                                            discovery["result"]["outcome"] == "interrupted" and
                                            any(d["code"] == "timeout" for d in discovery["result"]["diagnostics"]),
                                            "unresponsive_discovery_five_second_terminal_" + profile)
                        call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                             revision=basis["revision"], replacement_source=DESIRED))
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label, "refused")
                        observation.require(result["outcome"]["application"] == "not_applied" and
                                            result["outcome"]["reason"] == "timeout",
                                            "unresponsive_editor_original_deadline_refusal_" + profile)
                    finally:
                        editor["process"].send_signal(signal.SIGCONT)
                    self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_effects(self):
        for fault in ("fail_pwrite", "fail_futimens", "mismatch_mtime", "close_fd_before_restore"):
            label = "mcp-effects-open-" + fault
            stage = "resource_applied" if fault == "fail_pwrite" else "content_persisted"
            with self._interruption_fixture(label, "open", faults=True) as (project, editor, descriptor):
                with McpPeer(self, label) as peer:
                    call = self._interruption_start(peer, project, editor, descriptor, "open", stage, label)
                    before, disks = self.state(editor, project)
                    injected = self.native_action(editor, "native_edit_fault",
                                                  request_id=call["domain_request_id"], fault=fault)["result"]
                    observation.require(injected["status"] == "ready", "real_open_fault_installed_" + label)
                    self._interruption_release(editor, "open")
                    root = peer.finish(call, label)
                    result = self.mcp_review_edit(root, label, "applied_unverified")
                    after, now = self._interruption_survivor(project, editor, before, disks, label)
                    outcome = result["outcome"]
                    observation.require(outcome["application"] in ("applied", "partly_applied") and
                                        after["target"]["has_undo"] and
                                        (fault == "fail_pwrite" or now[TARGET]["text"] == DESIRED),
                                        "open_partial_effect_keeps_real_history_" + label)
                    if fault != "fail_pwrite":
                        observation.require(outcome["persistence"]["restore_attempted"] and
                                            not outcome["persistence"]["restored"] and after["target"]["dirty"],
                                            "open_metadata_failure_never_false_saved_" + label)
                self._interruption_recover(project, editor, descriptor, label)
        for profile in ("cached-closed", "absent-closed"):
            faults = ("partial_write", "lost_write", "mtime_failure", "expire_before_apply", "expire_after_write")
            if profile == "cached-closed":
                faults += ("expire_after_resource",)
            for fault in faults:
                label = "mcp-effects-" + profile + "-" + fault
                with self._interruption_fixture(label, profile, faults=True) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        call = self._interruption_start(peer, project, editor, descriptor, profile, "apply", label)
                        before, disks = self.state(editor, project)
                        injected = self.close_action(editor, "closed_fault", request_id=call["domain_request_id"], fault=fault)
                        observation.require(injected["ok"], "real_native_fault_installed_" + label)
                        self._interruption_release(editor, profile)
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label, "refused" if fault == "expire_before_apply" else
                                                      ("applied_unverified", "application_unknown"))
                        after, now = self._interruption_survivor(project, editor, before, disks, label,
                                                               no_effect=fault == "expire_before_apply")
                        effects = result["outcome"].get("effects")
                        if fault == "partial_write":
                            observation.require(now[TARGET]["text"] not in (SAFE, DESIRED) and
                                                0 < effects["written_bytes"] < len(DESIRED.encode()) and
                                                result["outcome"]["application"] == "partly_applied",
                                                "actual_partial_disk_bytes_not_success_" + label)
                        elif fault in ("mtime_failure", "expire_after_write"):
                            observation.require(now[TARGET]["text"] == DESIRED and not effects["mtime_restored"] and
                                                now[TARGET]["mtime_ns"] != disks[TARGET]["mtime_ns"],
                                                "actual_disk_effect_failed_metadata_" + label)
                        elif fault == "expire_after_resource":
                            observation.require(after["cached_R"] == DESIRED and now[TARGET] == disks[TARGET] and
                                                result["outcome"]["application"] != "not_applied",
                                                "actual_R_only_effect_" + label)
                        elif fault == "lost_write":
                            observation.require(now[TARGET] == disks[TARGET] and effects["disk_entered"] and
                                                effects["written_bytes"] == 0 and
                                                result["outcome"]["application"] ==
                                                ("partly_applied" if profile == "cached-closed" else "unknown"),
                                                "actual_lost_disk_ack_retains_known_R_or_unknown_" + label)
                    self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_newer_work(self):
        self.source_markers.add(b"HUMAN_CALLBACK_NEWER")
        for mode in ("change", "cancel", "stop", "disable", "change_stop", "change_disable"):
            label = "mcp-synchronous-open-entry-" + mode
            with self._interruption_fixture(label, "open") as (project, editor, descriptor):
                with McpPeer(self, label) as peer:
                    call = self._interruption_start(peer, project, editor, descriptor, "open", "apply", label)
                    before, disks = self.state(editor, project)
                    self.close_action(editor, "closed_open_callback", mode=mode,
                                      request_id=call["domain_request_id"])
                    self._interruption_release(editor, "open")
                    root = peer.finish(call, label)
                    result = self.mcp_review_edit(root, label, ("applied_unverified", "application_unknown"))
                    callback = self.native_action(editor, "native_edit_probe_result")["callback"]
                    after, now = self._interruption_survivor(project, editor, before, disks, label)
                    observation.require(callback.get("called") is True and
                                        callback["request_id"] == root["request_id"] == call["domain_request_id"] and
                                        callback["stage"] == "lines_edited_from" and callback["mode"] == mode and
                                        callback["slot_before"] is True and
                                        result["outcome"]["application"] != "not_applied" and
                                        now[TARGET] == disks[TARGET] and
                                        DESIRED not in after["target"]["B"],
                                        "actual_synchronous_native_entry_no_candidate_reassertion_" + label)
                    if mode.startswith("change"):
                        observation.require(after["target"]["B"] == callback["human_text"] and
                                            "# HUMAN_CALLBACK_NEWER" in after["target"]["B"] and
                                            after["target"]["dirty"] and after["target"]["has_undo"],
                                            "actual_native_entry_newer_human_work_survives_" + label)
                    observation.json_file(self.artifacts / (label + "-callback.json"),
                                          {key: callback[key] for key in
                                           ("request_id", "stage", "mode", "called", "slot_before", "slot_after")})
                    self.cases[-1]["callback_evidence"] = label + "-callback.json"
                self._interruption_recover(project, editor, descriptor, label)
        for stage in ("buffer_applied", "resource_applied", "content_persisted", "mtime_restored"):
            for control in ("cancel", "disable", "sigterm"):
                label = "mcp-newer-open-" + stage + "-" + control
                with self._interruption_fixture(label, "open") as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        call = self._interruption_start(peer, project, editor, descriptor, "open", stage, label)
                        self.close_action(editor, "close_human", path=TARGET, mutation="dirty_equal")
                        before, disks = self.state(editor, project)
                        observation.require(before["target"]["dirty"] and before["target"]["has_undo"] and
                                            "# CLOSE_HUMAN_EARLIER" in before["target"]["B"],
                                            "real_newer_typing_after_open_effect_" + label)
                        if control == "cancel":
                            peer.cancel(call)
                            peer.eof()
                        elif control == "sigterm":
                            peer.process.send_signal(signal.SIGTERM)
                        else:
                            self.close_action(editor, "close_lifetime", control="disable")
                        root = peer.finish(call, label, delivery_optional=control != "disable")
                        if root is not None:
                            self._interruption_review(root, call, stage, label)
                        self._interruption_release(editor, "open")
                        self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                    self._interruption_recover(project, editor, descriptor, label)
        for profile in ("cached-closed", "absent-closed"):
            stages = ("after_resource", "after_write", "after_mtime") if profile == "cached-closed" else \
                ("after_write", "after_mtime")
            for stage in stages:
                for action in ("newer_resource", "newer_disk", "open"):
                    if profile == "absent-closed" and action == "newer_resource":
                        continue
                    for interruption in ("stop", "disable"):
                        label = "mcp-newer-" + profile + "-" + stage + "-" + action + "-" + interruption
                        with self._interruption_fixture(label, profile, faults=True) as (project, editor, descriptor):
                            with McpPeer(self, label) as peer:
                                basis = self.mcp_read(peer, project, editor, descriptor, label + "-basis", source=SAFE)
                                before, disks = self.state(editor, project)
                                combined = action + "_" + interruption
                                self.close_action(editor, "closed_arm", callback_stage=stage, callback_action=combined)
                                call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                                     revision=basis["revision"], replacement_source=DESIRED))
                                root = peer.finish(call, label)
                                result = self.mcp_review_edit(root, label, ("applied_unverified", "application_unknown"))
                                after, now = self._interruption_survivor(project, editor, before, disks, label,
                                                                       human_selection=action == "open")
                                events = self.close_action(editor, "closed_witness")["events"]
                                callbacks = [event for event in events if event["stage"] == "callback:" + stage]
                                observation.require(len(callbacks) == 1 and callbacks[0]["action"] == combined and
                                                    callbacks[0]["request_id"] == root["request_id"] and
                                                    any(event["stage"] == stage and event["request_id"] == root["request_id"]
                                                        for event in events) and
                                                    result["outcome"]["application"] != "not_applied",
                                                    "actual_effect_callback_correlation_" + label)
                                if action == "newer_resource":
                                    observation.require(after["cached_R"] == NEWER, "newer_R_not_reasserted_" + label)
                                elif action == "newer_disk":
                                    observation.require(now[TARGET]["text"] == NEWER, "newer_D_not_compensated_" + label)
                                else:
                                    observation.require(TARGET in after["open_paths"], "newly_opened_document_not_closed_" + label)
                                observation.json_file(self.artifacts / (label + "-callbacks.json"),
                                                      [{key: event[key] for key in ("request_id", "stage", "action") if key in event}
                                                       for event in events])
                                self.cases[-1]["callback_evidence"] = label + "-callbacks.json"
                            self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_denial(self):
        label = "mcp-sticky-denial-open-newer-partial"
        self.source_markers.add(b"HUMAN_CALLBACK_NEWER")
        with self._interruption_fixture(label, "open") as (project, editor, descriptor):
            with McpPeer(self, label) as peer:
                call = self._interruption_start(peer, project, editor, descriptor, "open", "apply", label)
                before, disks = self.state(editor, project)
                target = project / TARGET.removeprefix("res://")
                mode = target.stat().st_mode
                with target.open("rb") as retained:
                    self.close_action(editor, "closed_open_callback", mode="change_deny",
                                      request_id=call["domain_request_id"])
                    self._interruption_release(editor, "open")
                    try:
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label, ("applied_unverified", "application_unknown"))
                        callback = self.native_action(editor, "native_edit_probe_result")["callback"]
                        after = self.close_action(editor, "closed_witness")["state"]
                        stat_before = os.fstat(retained.fileno())
                        raw = retained.read()
                        stat_after = os.fstat(retained.fileno())
                        encoded = json.dumps(root)
                        denied_disk = dict(disks[TARGET], text=raw.decode("utf-8"), sha256=sha(raw.decode("utf-8")),
                                           size=len(raw), mode=stat_after.st_mode & 0o777,
                                           ctime_ns=stat_after.st_ctime_ns)
                        observation.require(callback["denial_code"] == 0 and
                                            callback["request_id"] == root["request_id"] == call["domain_request_id"] and
                                            callback["mode"] == "change_deny" and callback["called"] and
                                            all(getattr(stat_before, key) == getattr(stat_after, key) for key in
                                                ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")) and
                                            denied_disk["mode"] == 0 and
                                            raw.decode("utf-8") == disks[TARGET]["text"] and
                                            str(stat_after.st_ino) == disks[TARGET]["inode"] and
                                            str(stat_after.st_dev) == disks[TARGET]["device"] and
                                            stat_after.st_mtime_ns == disks[TARGET]["mtime_ns"] and
                                            after["target"]["B"] == callback["human_text"] and
                                            after["target"]["dirty"] and after["target"]["has_undo"] and
                                            result["outcome"]["application"] in ("partly_applied", "unknown") and
                                            all(result["outcome"].get(key) is None for key in ("expected", "before", "after")) and
                                            all(value not in encoded for value in
                                                (DESIRED, "# HUMAN_CALLBACK_NEWER", disks[TARGET]["sha256"],
                                                 sha(DESIRED), sha(callback["human_text"]))) and
                                            all(documents(after).get(path) == doc for path, doc in documents(before).items()
                                                if path != TARGET) and editor["process"].poll() is None,
                                            "open_partial_causal_failure_then_sticky_actual_denial_" + label)
                        self.close_action(editor, "close_idle", frames=16)
                        later = self.close_action(editor, "closed_witness")["state"]
                        observation.require(documents(after) == documents(later) and
                                            after["selection"] == later["selection"],
                                            "open_denied_partial_never_compensates_newer_work_" + label)
                        observed_disk = dict(disks, **{TARGET: denied_disk})
                        observation.json_file(self.artifacts / (label + "-witness.json"),
                                              dict(before=source_free(before, disks),
                                                   after=source_free(after, observed_disk),
                                                   retained_fd_identity=True, observed_permission=0,
                                                   callback_request_id=callback["request_id"],
                                                   callback_stage=callback["stage"]))
                        self.cases[-1]["independent_evidence"] = label + "-witness.json"
                    finally:
                        target.chmod(mode)
            self._interruption_recover(project, editor, descriptor, label)
        for profile in ("cached-closed", "absent-closed"):
            label = "mcp-sticky-denial-" + profile
            with self._interruption_fixture(label, profile, faults=True) as (project, editor, descriptor):
                with McpPeer(self, label) as peer:
                    call = self._interruption_start(peer, project, editor, descriptor, profile, "apply", label)
                    before, disks = self.state(editor, project)
                    target = project / TARGET.removeprefix("res://")
                    mode = target.stat().st_mode
                    self.close_action(editor, "closed_fault", request_id=call["domain_request_id"], fault="mtime_failure")
                    self.close_action(editor, "closed_arm", callback_stage="before_verify", callback_action="deny_disk")
                    try:
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label, "applied_unverified")
                        outcome = result["outcome"]
                        encoded = json.dumps(root)
                        events = self.close_action(editor, "closed_witness")["events"]
                        observation.require(outcome["reason"] == "mtime_restore_failed" and
                                            outcome["application"] == "applied" and
                                            outcome["evidence"] is None and
                                            disks[TARGET]["sha256"] not in encoded and
                                            DESIRED not in encoded and
                                            any(event["stage"] == "callback:before_verify" and
                                                event["action"] == "deny_disk" and
                                                event["request_id"] == root["request_id"] for event in events) and
                                            target.stat().st_mode & 0o777 == 0,
                                            "actual_denial_sticky_after_earlier_metadata_failure_" + label)
                    finally:
                        target.chmod(mode)
                    after, now = self._interruption_survivor(project, editor, before, disks, label)
                    observation.require(now[TARGET]["text"] == DESIRED and
                                        now[TARGET]["mtime_ns"] != disks[TARGET]["mtime_ns"],
                                        "denial_does_not_rollback_known_disk_effect_" + label)
                self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_output(self):
        for profile in PROFILES:
            for stage in self._interruption_stages(profile):
                label = "mcp-lost-output-" + profile + "-" + stage.replace(":", "-")
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        call = self._interruption_start(peer, project, editor, descriptor, profile, stage, label)
                        before, disks = self.state(editor, project)
                        # Close the actual receiving pipe, not the editor channel.
                        peer.process.stdout.close()
                        peer.wait_exit(bound=max(0.01, call["started"] + 11 - time.monotonic()))
                        self._interruption_release(editor, profile)
                        self.case(label, operation="edit_script", delivery="unavailable", mcp_request_id=call["id"],
                                  domain_request_id=call["domain_request_id"],
                                  elapsed_seconds=time.monotonic() - call["started"],
                                  performance_claim="none_output_not_consumed")
                        self._interruption_survivor(project, editor, before, disks, label,
                                                   no_effect=stage == "prepare")
                    self._interruption_recover(project, editor, descriptor, label)
        self._interruption_partial_delivery(blocked=False)

    def mcp_interruption_worker_loss(self):
        for profile in PROFILES:
            for stage in ("prepare", "apply", "verify:post_change"):
                label = "mcp-worker-loss-" + profile + "-" + stage.replace(":", "-")
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        call = self._interruption_start(peer, project, editor, descriptor, profile, stage, label)
                        before, disks = self.state(editor, project)
                        # Enumerate only descendants of this owned server and
                        # require the exact existing worker executable argument.
                        listing = subprocess.run(["ps", "-axo", "pid=,ppid=,command="], check=True,
                                                 capture_output=True, text=True).stdout
                        rows = [line.strip().split(None, 2) for line in listing.splitlines()]
                        rows = [(int(pid), int(ppid), command) for pid, ppid, command in rows]
                        descendants = {peer.process.pid}
                        while True:
                            expanded = descendants | {pid for pid, ppid, _ in rows if ppid in descendants}
                            if expanded == descendants:
                                break
                            descendants = expanded
                        flag = "--internal-edit-worker" if profile == "open" else "--internal-closed-edit-worker"
                        workers = [pid for pid, _, command in rows
                                   if pid in descendants and flag in command.split()]
                        observation.require(len(workers) == 1 and editor["process"].pid not in workers,
                                            "identified_owned_actual_worker_only_" + label)
                        os.kill(workers[0], signal.SIGKILL)
                        root = peer.finish(call, label)
                        self._interruption_review(root, call, stage, label)
                        self._interruption_release(editor, profile)
                        self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                    self._interruption_recover(project, editor, descriptor, label)

    def _interruption_output_capacity(self, peer, label):
        # Probe an owned anonymous pipe created by the same OS primitive as
        # Popen stdout; never touch the editor or drain the product output.
        read_fd, write_fd = os.pipe()
        capacity = 0
        try:
            os.set_blocking(write_fd, False)
            chunk = b"x" * (1024 * 1024)
            while True:
                try:
                    capacity += os.write(write_fd, chunk)
                except BlockingIOError:
                    break
        finally:
            os.close(write_fd)
            os.close(read_fd)
        peer.send({"jsonrpc": "2.0", "id": "capacity-catalog", "method": "tools/list", "params": {}})
        catalog = peer.receive(timeout=5)
        observation.require(catalog.get("id") == "capacity-catalog" and
                            isinstance(catalog.get("result", {}).get("tools"), list),
                            "actual_consumed_catalog_for_output_capacity_" + label)
        size = len(json.dumps(catalog, separators=(",", ":"), ensure_ascii=False).encode()) - 64
        encoded = json.dumps(catalog).encode()
        observation.require(all(secret not in encoded for secret in self.secrets) and
                            all(marker not in encoded for marker in self.source_markers),
                            "catalog_backpressure_evidence_is_source_and_credential_free_" + label)
        count = capacity // size + 1 if size > 0 else 8
        observation.require(0 < count <= 7, "bounded_catalog_replies_exceed_measured_pipe_" + label)
        return {"pipe_capacity_bytes": capacity, "catalog_response_min_bytes": size,
                "catalog_reply_count": count}

    @staticmethod
    def _interruption_block_output(peer, capacity):
        for index in range(capacity["catalog_reply_count"]):
            peer.send({"jsonrpc": "2.0", "id": "blocked-" + str(index),
                       "method": "tools/list", "params": {}})

    def mcp_interruption_blocked_output(self):
        for profile in PROFILES:
            for stage in self._interruption_stages(profile):
                label = "mcp-blocked-output-" + profile + "-" + stage.replace(":", "-")
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        capacity = self._interruption_output_capacity(peer, label)
                        call = self._interruption_start(peer, project, editor, descriptor, profile, stage, label)
                        before, disks = self.state(editor, project)
                        # Complete valid control frames, <=7 outstanding IDs;
                        # the measured genuine catalog replies exceed the pipe.
                        self._interruption_block_output(peer, capacity)
                        peer.wait_exit(bound=max(0.01, call["started"] + 11 - time.monotonic()))
                        self._interruption_release(editor, profile)
                        self.case(label, operation="edit_script", delivery="unavailable",
                                  mcp_request_id=call["id"], domain_request_id=call["domain_request_id"],
                                  performance_claim="none_output_not_consumed",
                                  fault="actual_unread_stdout_pipe", **capacity)
                        self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                    self._interruption_recover(project, editor, descriptor, label)
        self._interruption_partial_delivery(blocked=True)

    def _interruption_partial_delivery(self, *, blocked):
        for profile in ("cached-closed", "absent-closed"):
            for fault in ("partial_write", "lost_write", "mtime_failure"):
                label = "mcp-" + ("blocked" if blocked else "lost") + "-partial-output-" + profile + "-" + fault
                with self._interruption_fixture(label, profile, faults=True) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        capacity = self._interruption_output_capacity(peer, label) if blocked else None
                        call = self._interruption_start(peer, project, editor, descriptor, profile, "apply", label)
                        _, original_disk = self.state(editor, project)
                        self.close_action(editor, "closed_fault", request_id=call["domain_request_id"], fault=fault)
                        self.close_action(editor, "closed_arm", stage="verify:post_change")
                        self.wait_mcp_barrier(editor, "verify:post_change", peer, call)
                        before, disks = self.state(editor, project)
                        if fault == "partial_write":
                            observation.require(disks[TARGET]["text"] not in (SAFE, DESIRED),
                                                "actual_partial_disk_before_output_failure_" + label)
                        elif fault == "mtime_failure":
                            observation.require(disks[TARGET]["text"] == DESIRED and
                                                disks[TARGET]["mtime_ns"] != original_disk[TARGET]["mtime_ns"],
                                                "actual_metadata_failure_before_output_failure_" + label)
                        else:
                            observation.require(disks[TARGET] == original_disk[TARGET] and
                                                (before["cached_R"] == DESIRED if profile == "cached-closed" else
                                                 not before["cached_id"]),
                                                "actual_lost_write_survivor_before_output_failure_" + label)
                        if blocked:
                            self._interruption_block_output(peer, capacity)
                        else:
                            peer.process.stdout.close()
                        peer.wait_exit(bound=max(0.01, call["started"] + 11 - time.monotonic()))
                        self._interruption_release(editor, profile)
                        self.case(label, operation="edit_script", delivery="unavailable",
                                  mcp_request_id=call["id"], domain_request_id=call["domain_request_id"],
                                  performance_claim="none_output_not_consumed", native_fault=fault,
                                  **(capacity or {}))
                        self._interruption_survivor(project, editor, before, disks, label, no_effect=True)
                    self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption_malformed(self):
        for profile in PROFILES:
            kinds = (("edit_prepared", "prepare"), ("edit_applied", "verify:post_change")) if profile == "open" else \
                (("closed_prepared", "prepare"), ("closed_applied", "verify:post_change"),
                 ("closed_verified", "verify:post_change"))
            for kind, effect_stage in kinds:
                label = "mcp-malformed-" + profile + "-" + kind
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label) as peer:
                        basis = self.mcp_read(peer, project, editor, descriptor, label + "-basis", source=SAFE)
                        before, disks = self.state(editor, project)
                        if kind == "closed_verified":
                            # The same reply kind also serves preflight; arm the
                            # corruption only after an independently entered effect.
                            self._interruption_arm(editor, profile, "verify:post_change")
                            call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                                 revision=basis["revision"], replacement_source=DESIRED))
                            self.wait_mcp_barrier(editor, "verify:post_change", peer, call)
                            self.close_action(editor, "closed_arm", reply_fault="malformed", reply_fault_kind=kind)
                        else:
                            self.close_action(editor, "closed_arm", reply_fault="malformed", reply_fault_kind=kind)
                            call = peer.start("edit_script", dict(selectors(project, descriptor),
                                                                 revision=basis["revision"], replacement_source=DESIRED))
                        root = peer.finish(call, label)
                        if root["error"] is not None:
                            error = root["error"]
                            observation.require(error["category"] == "host" and
                                                error["application"] == ("not_applied" if effect_stage == "prepare" else "unknown") and
                                                error["next_action"]["kind"] == "fresh_read",
                                                "private_decoder_not_invalid_client_or_false_rollback_" + label)
                        else:
                            result = self.mcp_review_edit(root, label, "refused" if effect_stage == "prepare" else
                                                          ("applied_unverified", "application_unknown"))
                            observation.require(result["outcome"]["reason"] != "invalid_request" and
                                                (effect_stage == "prepare" or result["outcome"]["application"] != "not_applied"),
                                                "private_decoder_retains_effect_uncertainty_" + label)
                        after, now = self._interruption_survivor(project, editor, before, disks, label,
                                                               no_effect=effect_stage == "prepare")
                        if effect_stage != "prepare":
                            observation.require(now[TARGET]["text"] == DESIRED,
                                                "real_persisted_effect_before_malformed_reply_" + label)
                    self._interruption_recover(project, editor, descriptor, label)

    def mcp_interruption(self):
        for name, method in INTERRUPTION_GROUPS.items():
            self.group(name, getattr(self, method))
