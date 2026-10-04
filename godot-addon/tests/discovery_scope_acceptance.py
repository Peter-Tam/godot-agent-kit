"""Private v6 read-only scope probes, owned by existing boundary groups.

No inventory, public runner, or product control surface is introduced here.
Raw context/source/credentials remain request-local; evidence records are safe facts.
"""
from __future__ import annotations

import json
import shutil
import struct
import time

import run_observation as observation

STATE_KEYS = {"v", "kind", "request_id", "session_id", "project_root", "collection",
              "status", "reason", "context", "expiry_tick_us"}
CONTEXT_KEYS = {"policy", "project_data_directory", "filesystem_epoch", "scanning", "importing"}
TERMINAL_KEYS = {"v", "kind", "request_id", "session_id", "project_root", "collection", "terminal_discard"}


def tuple_for(descriptor, request, opcode, *tail):
    return [6, opcode, request, descriptor["session_id"], descriptor["project_root"], *tail]


def exchange(harness, stream, descriptor, request, opcode, *tail):
    wire = observation.packet(tuple_for(descriptor, request, opcode, *tail))
    stream.sendall(wire)
    reply, raw = observation.receive(stream)
    kind = {"discover_begin": "discover_state", "discover_recheck": "discover_rechecked",
            "discover_finish": "discover_finished", "discover_abort": "discover_aborted"}[opcode]
    observation.require(set(reply) == (STATE_KEYS if opcode in ("discover_begin", "discover_recheck")
                                      else TERMINAL_KEYS), "scope_exact_reply_shape")
    observation.require(all(reply[key] == value for key, value in {
        "v": 6, "kind": kind, "request_id": request, "session_id": descriptor["session_id"],
        "project_root": descriptor["project_root"]}.items()), "scope_authenticated_binding")
    stamp = reply["collection"]
    observation.require(set(stamp) == {"clock_id", "started_tick_us", "finished_tick_us", "received_elapsed_us"}
                        and stamp["clock_id"] == "editor:" + descriptor["session_id"]
                        and stamp["received_elapsed_us"] == 0
                        and canonical(stamp["started_tick_us"]) and canonical(stamp["finished_tick_us"])
                        and int(stamp["finished_tick_us"]) >= int(stamp["started_tick_us"]),
                        "scope_actual_editor_collection_stamp")
    for sentinel in (*observation.SOURCE_SENTINELS, *harness.source_markers, *harness.secrets):
        observation.require(sentinel not in raw, "scope_source_credential_privacy")
    observation.require(b"res://scripts" not in raw and b"source_code" not in raw,
                        "scope_no_selected_or_unrelated_source_path")
    if opcode in ("discover_begin", "discover_recheck"):
        observation.require(reply["status"] in ("observed", "unavailable", "refused"), "scope_closed_status")
        if reply["context"] is not None:
            context = reply["context"]
            observation.require(set(context) == CONTEXT_KEYS and context["policy"] == "godot_project_files_v1"
                                and context["project_data_directory"] in ("res://.godot", "res://godot")
                                and canonical(context["filesystem_epoch"])
                                and type(context["scanning"]) is bool and type(context["importing"]) is bool,
                                "scope_exact_typed_context")
        observation.require(reply["expiry_tick_us"] is None or canonical(reply["expiry_tick_us"]),
                            "scope_canonical_expiry")
    else:
        observation.require(type(reply["terminal_discard"]) is bool, "scope_typed_terminal")
    return reply


def canonical(value):
    return isinstance(value, str) and value.isascii() and value.isdecimal() and str(int(value)) == value


def slot_free(harness, editor):
    observation.wait_for(lambda: not harness.action(editor, "scope_slot")["busy"], "scope_slot_cleanup")


def assert_discovery_busy(harness, descriptor):
    """Called while a real existing observation/edit/open owns the common slot."""
    stream, request = harness.authenticated_peer(descriptor)
    with stream:
        reply = exchange(harness, stream, descriptor, request, "discover_begin", 4000)
    observation.require(reply["status"] == "refused" and reply["reason"] == "busy"
                        and reply["context"] is None and reply["expiry_tick_us"] is None,
                        "scope_same_slot_busy_no_owned_context")


def session_scope_cases(harness, descriptor, editor):
    # Valid begin/finish, and malformed authenticated controls cannot leave owners.
    stream, request = harness.authenticated_peer(descriptor)
    with stream:
        reply = exchange(harness, stream, descriptor, request, "discover_begin", 4500)
        observation.require(reply["status"] == "observed", "scope_admitted_budget_endpoint")
        exchange(harness, stream, descriptor, request, "discover_finish")
    slot_free(harness, editor)
    harness.case("scope_budget_endpoint_4500", budget_ms=4500)
    for name, change in (
            ("missing_budget", lambda value: value[:-1]),
            ("extra_field", lambda value: [*value, None]),
            ("boolean_budget", lambda value: [*value[:-1], True]),
            ("fractional_budget", lambda value: [*value[:-1], 1.5]),
            ("zero_budget", lambda value: [*value[:-1], 0]),
            ("negative_budget", lambda value: [*value[:-1], -1]),
            ("over_budget", lambda value: [*value[:-1], 4501]),
            ("string_budget", lambda value: [*value[:-1], "4000"]),
            ("wrong_request", lambda value: [*value[:2], "wrong-owner", *value[3:]]),
            ("wrong_session", lambda value: [*value[:3], "f" * 32, *value[4:]]),
            ("wrong_project", lambda value: [*value[:4], "/not-selected", value[5]]),
            ("nested_budget", lambda value: [*value[:-1], [[0]]])):
        stream, request = harness.authenticated_peer(descriptor)
        with stream:
            stream.sendall(observation.packet(change(tuple_for(descriptor, request, "discover_begin", 4000))))
            observation.require(observation.peer_closed_without_data(stream), "scope_reject_" + name)
        slot_free(harness, editor)
        harness.case("scope_reject_" + name)
    for opcode in ("discover_recheck", "discover_finish", "discover_abort"):
        stream, request = harness.authenticated_peer(descriptor)
        with stream:
            stream.sendall(observation.packet(tuple_for(descriptor, request, opcode)))
            observation.require(observation.peer_closed_without_data(stream), "scope_stage_before_begin")
        harness.case("scope_reject_unowned_" + opcode)
    for name, payload in (("oversized_selected_prefix", struct.pack(">I", 4 * 1024 * 1024 + 1)),
                          ("depth33", None)):
        stream, request = harness.authenticated_peer(descriptor)
        with stream:
            if payload is None:
                body = ('[6,"discover_begin",' + json.dumps(request) + ',' +
                        json.dumps(descriptor["session_id"]) + ',' + json.dumps(descriptor["project_root"]) +
                        ',' + '[' * 33 + '0' + ']' * 33 + ']').encode()
                payload = struct.pack(">I", len(body)) + body
            stream.sendall(payload)
            observation.require(observation.peer_closed_without_data(stream), "scope_preallocation_depth_rejection")
        harness.case("scope_reject_" + name, body_sent=name == "depth33")
    # Header under selected-source ceiling followed by discovery opcode proves the
    # operation-specific 4 KiB bound, not merely the general selected-source limit.
    stream, request = harness.authenticated_peer(descriptor)
    with stream:
        body = json.dumps(tuple_for(descriptor, request, "discover_begin", 4000)).encode()
        body += b" " * (4097 - len(body))
        stream.sendall(struct.pack(">I", len(body)) + body)
        observation.require(observation.peer_closed_without_data(stream), "scope_4kib_control_bound")
    harness.case("scope_reject_4097_byte_control")
    slot_free(harness, editor)
    stream, request = harness.authenticated_peer(descriptor)
    with stream:
        body = json.dumps(tuple_for(descriptor, request, "discover_begin", 4500)).encode()
        body += b" " * (4096 - len(body))
        stream.sendall(struct.pack(">I", len(body)) + body)
        accepted, _ = observation.receive(stream)
        observation.require(accepted["status"] == "observed" and accepted["request_id"] == request,
                            "scope_exact_4096_byte_control_admitted")
        exchange(harness, stream, descriptor, request, "discover_finish")
    harness.case("scope_admit_4096_byte_control")
    for opcode in ("discover_recheck", "discover_finish", "discover_abort"):
        for fault in ("extra", "missing", "typed_binding"):
            stream, request = harness.authenticated_peer(descriptor)
            with stream:
                exchange(harness, stream, descriptor, request, "discover_begin", 4500)
                value = tuple_for(descriptor, request, opcode)
                if fault == "extra":
                    value.append(None)
                elif fault == "missing":
                    value.pop()
                else:
                    value[3] = True
                stream.sendall(observation.packet(value))
                observation.require(observation.peer_closed_without_data(stream),
                                    "scope_strict_owned_stage_" + opcode + "_" + fault)
            slot_free(harness, editor)
            harness.case("scope_reject_owned_" + opcode + "_" + fault)


def executor_scope_cases(harness):
    for visible in (False, True):
        project = harness.fixture("scope-visible" if visible else "scope-hidden", controlled=True)
        config = project / "project.godot"
        (project / "scope_import.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="red"/></svg>')
        if visible:
            text = config.read_text()
            # This setting must precede launch; the effective path is editor-owned.
            config.write_text(text.replace("[application]", "[application]\nconfig/use_hidden_project_data_directory=false"))
        bridge = project / "addons/godot_agent_kit/bridge.gd"
        text = bridge.read_text()
        old = 'preload("res://addons/godot_agent_kit/script_discovery.gd")'
        observation.require(text.count(old) == 1, "scope_owned_fixture_preload_seam")
        bridge.write_text(text.replace(old, 'preload("res://addons/fixture_driver/fixture_discovery_scope.gd")'))
        if visible:
            # This owned project proves scope availability with no installed
            # native mutation family, not merely a synthetic capability bit.
            shutil.rmtree(project / "addons/godot_agent_kit/native")
        editor = harness.start_editor(project)
        try:
            descriptor = observation.wait_for(lambda: next(iter(harness.descriptors(project)), None), "scope_descriptor")
            capability_peer, capability = harness.challenge(descriptor)
            capability_peer.close()
            observation.require(capability["capabilities"]["discover_gdscripts"] is True,
                                "scope_public_getters_advertise_capability")
            if visible:
                observation.require(capability["native_api_revision"] == 0 and
                                    capability["capabilities"]["edit_open_gdscript"] is False and
                                    capability["capabilities"]["open_gdscript"] is False,
                                    "scope_available_without_native_mutation_family")
            # A valid 1ms budget need not outlive the next editor/TCP frame.
            # Independently witness the real owner's queued admission, not silence.
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                peer.sendall(observation.packet(tuple_for(descriptor, peer_id, "discover_begin", 1)))
                try:
                    short, _ = observation.receive(peer)
                    observation.require(short["status"] == "observed", "scope_short_budget_if_delivered")
                except (EOFError, ConnectionResetError):
                    pass
            admitted = harness.action(editor, "scope_admission")
            observation.require(admitted["request_id"] == peer_id and admitted["status"] == "observed"
                                and int(admitted["expiry_tick_us"]) - int(admitted["started_tick_us"]) == 1000,
                                "scope_one_ms_real_admission_expiry")
            slot_free(harness, editor)
            harness.case("scope_budget_endpoint_1_visible" if visible else "scope_budget_endpoint_1_hidden",
                         budget_ms=1, queued_admission_witness=True, delivery_required=False)
            harness.action(editor, "prepare_subject")
            harness.action(editor, "dirty_subject")
            history = harness.action(editor, "seed_sequence_history")
            harness.action(editor, "scope_selection")
            profile = "visible" if visible else "hidden"
            before_shot = harness.screenshot(editor, "scope-" + profile + "-before.png")
            before = harness.action(editor, "witness")
            subject = project / "scripts/subject.gd"
            disk = (subject.read_bytes(), subject.stat().st_mtime_ns)
            expected = "res://godot" if visible else "res://.godot"
            effective = harness.action(editor, "scope_effective")
            observation.require(effective["settings_directory"] == expected + "/editor",
                                "scope_real_effective_directory")
            stream, request = harness.authenticated_peer(descriptor)
            with stream:
                begin = exchange(harness, stream, descriptor, request, "discover_begin", 4500)
                observation.require(begin["context"]["project_data_directory"] == expected, "scope_effective_not_setting")
                changed = harness.action(editor, "scope_unsaved_hidden" if visible else "scope_unsaved_visible")
                observation.require(changed["mutable_hidden_setting"] == visible and
                                    changed["settings_directory"] == expected + "/editor", "scope_unsaved_setting_not_effective")
                recheck = exchange(harness, stream, descriptor, request, "discover_recheck")
                observation.require(recheck["context"]["project_data_directory"] == expected and
                                    int(recheck["collection"]["started_tick_us"]) > int(begin["collection"]["finished_tick_us"]),
                                    "scope_fresh_real_recheck")
                harness.action(editor, "scope_epoch_change")
                epoch = exchange(harness, stream, descriptor, request, "discover_recheck")
                observation.require(int(epoch["context"]["filesystem_epoch"]) > int(recheck["context"]["filesystem_epoch"]),
                                    "scope_public_signal_epoch_invalidation")
                assert_discovery_busy(harness, descriptor)
                for operation, tail in (("observe", ["res://scripts/subject.gd"]),
                                        ("open_begin", ["res://scripts/subject.gd", 4000]),
                                        ("edit_prepare", ["res://scripts/subject.gd", 4000, "base-id", *(["0"] * 8), "a" * 64, "0", ""])):
                    peer, peer_id = harness.authenticated_peer(descriptor)
                    with peer:
                        peer.sendall(observation.packet(tuple_for(descriptor, peer_id, operation, *tail)))
                        refusal, raw = observation.receive(peer)
                        observation.require(refusal.get("reason") in ("busy", "slot_busy") or refusal.get("code") == "unsupported_observation",
                                            "scope_blocks_existing_" + operation)
                        observation.require(b"source_code" not in raw and observation.SOURCE_SENTINEL not in raw,
                                            "scope_busy_source_free")
                exchange(harness, stream, descriptor, request, "discover_abort")
            slot_free(harness, editor)
            # Observation owner really retains its original sample until recheck.
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                harness.peer_operation(peer, descriptor, peer_id, "observe", subject)
                assert_discovery_busy(harness, descriptor)
                harness.peer_operation(peer, descriptor, peer_id, "recheck", subject)
            slot_free(harness, editor)
            for mode, status, reason in (("unavailable", "unavailable", "unavailable_editor_context"),
                                         ("unsupported", "refused", "unsupported_visibility_policy"),
                                         ("scanning", "observed", None), ("importing", "observed", None)):
                harness.action(editor, "scope_" + mode)
                peer, peer_id = harness.authenticated_peer(descriptor)
                with peer:
                    facts = exchange(harness, peer, descriptor, peer_id, "discover_begin", 4000)
                    observation.require(facts["status"] == status and facts["reason"] == reason, "scope_negative_context")
                    if mode in ("scanning", "importing"):
                        observation.require(facts["context"][mode] is True, "scope_independent_busy_fact")
                    else:
                        observation.require(facts["context"] is None, "scope_no_guessed_context")
                    exchange(harness, peer, descriptor, peer_id, "discover_finish")
                harness.case("scope_" + mode + ("_visible" if visible else "_hidden"),
                             getter_fault_fixture=True, context_status=status)
            harness.action(editor, "scope_normal")
            harness.action(editor, "scope_live_scan")
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                live_scan = exchange(harness, peer, descriptor, peer_id, "discover_begin", 4500)
                observation.require(live_scan["context"]["scanning"] is True, "scope_real_active_scan_fact")
                harness.action(editor, "scope_normal")
                exchange(harness, peer, descriptor, peer_id, "discover_finish")
            harness.case("scope_live_scanning_visible" if visible else "scope_live_scanning_hidden",
                         getter_fault_fixture=False, real_public_scan=True)
            harness.action(editor, "scope_live_import")
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                live_import = exchange(harness, peer, descriptor, peer_id, "discover_begin", 4500)
                observation.require(live_import["context"]["importing"] is True, "scope_real_active_import_fact")
                harness.action(editor, "scope_normal")
                exchange(harness, peer, descriptor, peer_id, "discover_finish")
            harness.case("scope_live_importing_visible" if visible else "scope_live_importing_hidden",
                         getter_fault_fixture=False, real_public_import_signal_sample=True)
            harness.action(editor, "scope_disabled_capability")
            challenge_peer, challenge = harness.challenge(descriptor)
            challenge_peer.close()
            observation.require(challenge["capabilities"]["discover_gdscripts"] is False,
                                "scope_false_capability_authenticated")
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                unsupported = exchange(harness, peer, descriptor, peer_id, "discover_begin", 4500)
                observation.require(unsupported["reason"] == "unsupported_discovery" and
                                    unsupported["context"] is None and unsupported["expiry_tick_us"] is None,
                                    "scope_false_capability_never_claims")
            peer, peer_id = harness.authenticated_peer(descriptor)
            with peer:
                harness.peer_operation(peer, descriptor, peer_id, "observe", subject)
                harness.peer_operation(peer, descriptor, peer_id, "recheck", subject)
            harness.action(editor, "scope_normal")
            harness.case("scope_false_capability_preserves_observation_visible" if visible else
                         "scope_false_capability_preserves_observation_hidden",
                         discovery_capability=False, real_observation_recheck=True)
            # Wrong owner stages and duplicate begin cannot release somebody else's scope.
            owner, owner_id = harness.authenticated_peer(descriptor)
            with owner:
                exchange(harness, owner, descriptor, owner_id, "discover_begin", 4500)
                wrong, wrong_id = harness.authenticated_peer(descriptor)
                with wrong:
                    wrong.sendall(observation.packet(tuple_for(descriptor, wrong_id, "discover_abort")))
                    observation.require(observation.peer_closed_without_data(wrong), "scope_wrong_owner_closed")
                exchange(harness, owner, descriptor, owner_id, "discover_recheck")
                owner.sendall(observation.packet(tuple_for(descriptor, owner_id, "discover_begin", 4000)))
                observation.require(observation.peer_closed_without_data(owner), "scope_duplicate_begin_closed")
            slot_free(harness, editor)
            for cleanup in ("disconnect", "expiry", "disable"):
                peer, peer_id = harness.authenticated_peer(descriptor)
                exchange(harness, peer, descriptor, peer_id, "discover_begin", 100 if cleanup == "expiry" else 4500)
                if cleanup == "disconnect":
                    peer.close()
                elif cleanup == "expiry":
                    time.sleep(.15)
                    try:
                        peer.sendall(observation.packet(tuple_for(descriptor, peer_id, "discover_recheck")))
                        observation.require(observation.peer_closed_without_data(peer), "scope_late_expired_stage_closed")
                    except (BrokenPipeError, ConnectionResetError):
                        pass
                    peer.close()
                else:
                    harness.action(editor, "disable")
                    observation.require(observation.peer_closed_without_data(peer), "scope_disable_closes_owner")
                    peer.close()
                    harness.action(editor, "enable")
                    old_session = descriptor["session_id"]
                    descriptor = observation.wait_for(lambda: next((d for d in harness.descriptors(project)
                                                                    if d["session_id"] != old_session), None), "scope_fresh_session")
                slot_free(harness, editor)
                harness.case("scope_cleanup_" + cleanup + ("_visible" if visible else "_hidden"))
            after = harness.action(editor, "witness")
            keys = ("open_paths", "current_script", "unsaved_paths", "script_ids", "editor_ids",
                    "subject", "prepared_subject_buffer", "subject_cached_id", "subject_cached_R",
                    "cold_cached_id", "cold_cached_R", "other", "empty")
            observation.require(all(before[key] == after[key] for key in keys) and
                                disk == (subject.read_bytes(), subject.stat().st_mtime_ns),
                                "scope_independent_source_selection_dirty_history_noninterference")
            after_shot = harness.screenshot(editor, "scope-" + profile + "-after.png")
            replay = harness.action(editor, "replay_sequence_history")
            observation.require([step["text"] for step in replay["steps"]] ==
                                [history["history"][key] for key in ("second", "first", "initial", "first")],
                                "scope_real_undo_redo_preserved")
            harness.case("scope_effective_visible" if visible else "scope_effective_hidden",
                         effective_data_directory=expected, independent_disk_preserved=True,
                         before_screenshot=before_shot, after_screenshot=after_shot,
                         real_history_transitions=4, source_selection_dirty_preserved=True,
                         same_slot_operations=["observation", "edit", "open", "discovery"],
                         scope_is_not_inventory_verification=True)
        finally:
            harness.close_editor(editor)
