#!/usr/bin/env python3
"""Owned GUI acceptance of the read-only, exact-source native GDScript validator.

No edit caller, writer, finalizer, bridge edit operation, or synthetic parser is used.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import stat
import subprocess
import tempfile

import run_observation as observation

FIXTURE = Path(__file__).parent / "fixtures" / "script_edit"
ROOT = "res://scripts/subject.gd"
DIRECT = 'extends "./native/level_one.gd"\nconst NATIVE_DIRECT := 1\n'
NESTED = 'extends "./native/deep/level_two.gd"\nconst NATIVE_TRANSITIVE := 2\n'
LOADER = 'extends RefCounted\nconst NATIVE_LOADER = preload("res://scripts/native/excluded.native_effect")\n'
CONTEXT = 'extends RefCounted\nfunc get_context() -> ValidationDependency:\n\treturn null\n'
MARKERS = (b"NATIVE_DIRECT", b"NATIVE_TRANSITIVE", b"NATIVE_LOADER", b"NATIVE_SIBLING_VALUE",
           b"NATIVE_DEPENDENCY_VALUE", b"NATIVE_VALID_ROOT", b"NATIVE_INVALID_ROOT",
           b"NATIVE_CONTEXT_ID", b"not an int", b"x" * 64, "é".encode() * 64)
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
SAFE_CATEGORY = re.compile(r"[A-Za-z][A-Za-z0-9_]*\Z")


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def concise_document(doc):
    if not doc.get("associated"):
        return {"associated": False, "matches": doc.get("matches")}
    return {key: doc[key] for key in ("path", "script_id", "editor_id", "buffer_id", "version",
                                       "saved_version", "dirty", "has_undo", "has_redo",
                                       "caret_line", "caret_column", "has_selection")} | {
        "resource_sha256": sha(doc["R"]), "buffer_sha256": sha(doc["B"])}


def concise_disk(witness):
    return {key: witness[key] for key in ("sha256", "size", "device", "inode", "mtime_ns",
                                          "ctime_ns", "mode")}


def dependency_witness(project, relative):
    # Component-relative I/O independently observes paths beyond macOS PATH_MAX.
    fd = os.open(project, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        parts = relative.split("/")
        for index, part in enumerate(parts):
            flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK
            if index < len(parts) - 1:
                flags |= os.O_DIRECTORY
            child = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = child
        before = os.fstat(fd)
        observation.require(stat.S_ISREG(before.st_mode), "independent_dependency_regular_file")
        raw = bytearray()
        while True:
            block = os.read(fd, 65536)
            if not block:
                break
            raw.extend(block)
            observation.require(len(raw) <= 524288, "independent_dependency_source_bound")
        after = os.fstat(fd)
        observation.require((before.st_dev, before.st_ino, before.st_size,
                             before.st_mtime_ns, before.st_ctime_ns) ==
                            (after.st_dev, after.st_ino, after.st_size,
                             after.st_mtime_ns, after.st_ctime_ns) and len(raw) == before.st_size,
                            "independent_dependency_stable_read")
        return {"sha256": hashlib.sha256(raw).hexdigest(), "size": len(raw),
                "device": str(before.st_dev), "inode": str(before.st_ino)}
    finally:
        os.close(fd)


class NativeHarness(observation.Harness):
    def __init__(self, args, work):
        super().__init__(args, work)
        self.source_markers.update(MARKERS)
        self.summary["coverage_scope"] = "t002_native_validation_only"
        self.summary["source_observation"] = True
        self.summary["fixture_driver_sha256"] = observation.digest(FIXTURE / "fixture_driver.gd")
        self.summary["native_fixture_files"] = {str(p.relative_to(FIXTURE)): observation.digest(p)
                                               for p in sorted(FIXTURE.rglob("*")) if p.is_file()}

    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        if not name.startswith("export-"):
            shutil.copytree(FIXTURE / "scripts", project / "scripts", dirs_exist_ok=True,
                            ignore=shutil.ignore_patterns("tool_initializer.gd"))
            cold = project / "scripts" / "native" / "cold"
            cold.mkdir()
            (cold / ".gdignore").touch()
            shutil.copy2(FIXTURE / "scripts" / "native" / "tool_initializer.gd", cold)
        helper = project / "addons" / "fixture_driver"
        shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "native_fixture_driver.gd")
        shutil.copy2(FIXTURE / "native_effect_loader.gd", helper / "native_effect_loader.gd")
        shutil.copy2(FIXTURE / "native_effect_object.gd", helper / "native_effect_object.gd")
        (helper / "plugin.gd").write_text('@tool\nextends "res://addons/fixture_driver/native_fixture_driver.gd"\n')
        return project

    def native_action(self, editor, action, **arguments):
        request_id = secrets.token_hex(8)
        observation.json_file(editor["control"] / "request.json",
                              {"id": request_id, "action": action, **arguments})

        def response():
            path = editor["control"] / "response.json"
            if path.exists():
                result = json.loads(path.read_text())
                if result.get("id") == request_id:
                    return result
            observation.require(editor["process"].poll() is None, "native_editor_exited_" + action)
            return None

        result = observation.wait_for(response, "native_fixture_action_" + action, timeout=35)
        observation.require(result.get("ok") is True, "native_fixture_action_failed_" + action)
        return {key: value for key, value in result.items() if key not in ("id", "action", "ok")}

    def installed_native(self, project):
        addon = project / "addons" / "godot_agent_kit"
        manifest = addon / "native" / "script_edit.gdextension"
        binaries = list(addon.rglob("*.dylib"))
        build_manifest = addon / "native" / "build-manifest.json"
        observation.require(manifest.is_file() and build_manifest.is_file() and len(binaries) == 1,
                            "native_manifest_build_receipt_and_exactly_one_installed_library_required")
        provenance = json.loads(build_manifest.read_text())
        observation.require(provenance.get("base_commit") == observation.ENGINE_HASH and
                            provenance.get("engine_version") == self.version and
                            provenance.get("engine_sha256") == observation.digest(self.args.godot) and
                            provenance.get("native_library_sha256") == observation.digest(binaries[0]) and
                            HEX64.fullmatch(str(provenance.get("native_build_id", ""))),
                            "native_binary_generated_abi_and_pinned_engine_provenance")
        self.summary["native_manifest_sha256"] = observation.digest(manifest)
        self.summary["native_build_manifest_sha256"] = observation.digest(build_manifest)
        self.summary["engine_patch_sha256"] = provenance.get("patch_sha256")
        self.summary["gdextension_abi_sha256"] = provenance.get("abi_sha256")
        self.summary["gdextension_api_sha256"] = provenance.get("api_sha256")
        self.summary["native_toolchain"] = {"compiler": provenance.get("compiler"),
                                            "sdk": provenance.get("sdk")}
        self.summary["native_library_sha256"] = observation.digest(binaries[0])
        self.summary["native_library_relative_path"] = str(binaries[0].relative_to(project))
        self.expected_native_build_id = provenance["native_build_id"]

    def install_reentry_fixture(self, project):
        native = observation.REPO / "godot-addon" / "native"
        headers = native / "build"
        observation.require(all((headers / name).is_file() for name in
                                ("gdextension_interface.h", "native_abi_sizes.h")),
                            "exact_native_abi_headers_required_for_fixture")
        destination = project / "addons" / "fixture_driver"
        binary = destination / "libreentry.macos.arm64.dylib"
        command = ["clang++", "-std=c++17", "-O2", "-fPIC", "-fvisibility=hidden",
                   "-dynamiclib", "-Wall", "-Wextra", "-Werror", "-I", str(headers),
                   "-I", str(native), str(FIXTURE / "reentry_reader.cpp"), "-o", str(binary)]
        result = observation.run(command, timeout=90)
        self.safe_log("fixture-native-build.stderr", result.stderr)
        observation.require(result.returncode == 0 and binary.is_file(),
                            "fixture_native_custom_reader_build")
        (destination / "reentry.gdextension").write_text(
            '[configuration]\nentry_symbol = "fixture_reentry_init"\n'
            'compatibility_minimum = "4.7"\nreloadable = false\n\n'
            '[libraries]\nmacos.arm64 = '
            '"res://addons/fixture_driver/libreentry.macos.arm64.dylib"\n')
        self.summary["fixture_reader_sha256"] = observation.digest(binary)
        self.summary["fixture_reader_manifest_sha256"] = observation.digest(
            destination / "reentry.gdextension")

    def snapshot(self, editor, project):
        witness = self.action(editor, "witness")
        disk = {name: observation.disk_witness(project / "scripts" / (name + ".gd"))
                for name in ("subject", "other")}
        observation.require(witness["subject"].get("associated") and
                            witness["other"].get("associated"), "independent_two_open_documents")
        return witness, disk

    def source_free_state(self, witness, disk):
        return {"subject": concise_document(witness["subject"]),
                "other": concise_document(witness["other"]),
                "disk": {name: concise_disk(value) for name, value in disk.items()},
                "current_script": witness["current_script"],
                "open_paths": witness["open_paths"],
                "unsaved_paths": witness["unsaved_paths"]}

    def check_result(self, project, result, source, path, request_id, session, status, *,
                     origin=None, dependency=None, generated=False):
        observation.require(isinstance(result, dict) and result.get("status") == status,
                            "real_native_status_" + status)
        observation.require(result.get("request_id") == request_id and
                            result.get("session_id") == session and
                            result.get("purpose") == "preflight", "real_native_correlation")
        observation.require(result.get("source_path") == path, "native_exact_source_path")
        actual = result.get("input", {})
        observation.require(actual.get("source_sha256") == sha(source) and
                            actual.get("utf8_bytes") == len(source.encode("utf-8")),
                            "native_actual_supplied_utf8")
        clock = result.get("collection", {})
        observation.require(clock.get("clock_id") == "editor" and
                            all(isinstance(clock.get(key), str) and clock[key].isdigit()
                                for key in ("started_tick", "finished_tick")) and
                            int(clock["finished_tick"]) >= int(clock["started_tick"]),
                            "native_editor_clock_interval")
        observation.require(result.get("document") == self.target_document,
                            "native_bound_actual_document")
        diagnostics = result.get("diagnostics", [])
        observation.require(isinstance(diagnostics, list) and len(diagnostics) <= 64 and
                            all(isinstance(d, dict) and d.get("origin") in ("root", "dependency")
                                for d in diagnostics), "native_source_attributed_diagnostics")
        observation.require(result.get("reason") is None or
                            (isinstance(result["reason"], str) and
                             SAFE_CATEGORY.fullmatch(result["reason"])),
                            "source_free_validation_reason_category")
        dependencies = result.get("dependencies", [])
        observation.require(isinstance(dependencies, list) and len(dependencies) <= 32 and
                            all(isinstance(d, dict) and
                                isinstance(d.get("path"), str) and
                                d["path"].startswith("res://") and
                                len(d["path"].encode()) <= 2048 and
                                HEX64.fullmatch(str(d.get("source_sha256", ""))) and
                                isinstance(d.get("utf8_bytes"), int) and
                                0 <= d["utf8_bytes"] <= 524288 and
                                isinstance(d.get("identity"), dict) and
                                all(str(d["identity"].get(key, "")).isdigit()
                                    for key in ("device", "inode")) and
                                "source" not in d for d in dependencies),
                            "source_free_confined_bounded_dependency_witnesses")
        if status in ("valid", "invalid"):
            for entry in dependencies:
                relative = entry["path"].removeprefix("res://")
                observation.require(relative and ".." not in relative.split("/") and
                                    not relative.startswith("/"), "confined_dependency_path_components")
                independent = dependency_witness(project, relative)
                identity = entry.get("identity", {})
                observation.require(entry["source_sha256"] == independent["sha256"] and
                                    entry["utf8_bytes"] == independent["size"] and
                                    identity.get("device") == independent["device"] and
                                    identity.get("inode") == independent["inode"],
                                    "independent_dependency_content_and_inode")
        observation.require(all((d.get("path") is None or
                                 (isinstance(d["path"], str) and
                                  d["path"].startswith("res://") and
                                  len(d["path"].encode()) <= 2048)) and
                                (d.get("source_sha256") is None or
                                 HEX64.fullmatch(str(d["source_sha256"]))) and
                                len(str(d.get("message", "")).encode()) <= 2048
                                for d in diagnostics), "redacted_bounded_source_diagnostics")
        observation.require(all(isinstance(d.get("category"), str) and
                                SAFE_CATEGORY.fullmatch(d["category"])
                                for d in diagnostics), "source_free_diagnostic_categories")
        if status == "valid":
            observation.require(result.get("diagnostics_complete") is True and not diagnostics and
                                result.get("context_current") is True and
                                result.get("dependencies_current") is True and
                                isinstance(result.get("context"), str) and
                                HEX64.fullmatch(result["context"]), "completed_real_parser_analyzer")
        elif status == "invalid":
            observation.require(result.get("diagnostics_complete") is True and bool(diagnostics) and
                                all(isinstance(d.get("message"), str) and d["message"] for d in diagnostics),
                                "actual_invalid_diagnostics")
        else:
            observation.require(isinstance(result.get("reason"), str) and result["reason"] and
                                result.get("diagnostics_complete") is False,
                                "unavailable_requires_explicit_incomplete_reason")
        observation.require(all(d.get("origin") != "root" or
                                d.get("source_sha256") == sha(source)
                                for d in diagnostics), "root_diagnostic_actual_supplied_source")
        if origin is not None:
            observation.require(any(d.get("origin") == origin and
                                    d.get("path") == (path if origin == "root" else dependency)
                                    for d in diagnostics), "diagnostic_provenance_" + origin)
        if dependency is not None and status == "valid":
            observation.require(any(d.get("path") == dependency and HEX64.fullmatch(
                str(d.get("source_sha256", ""))) for d in result.get("dependencies", [])),
                "independently_read_source_dependency")
        return {"status": status, "reason": result.get("reason"),
                "request_id": request_id, "session_id": session, "purpose": result["purpose"],
                "source_path": result["source_path"], "input": actual,
                "collection": clock, "context": result.get("context"),
                "context_current": result.get("context_current"),
                "dependencies_current": result.get("dependencies_current"),
                "dependencies": [{"path": d["path"],
                                  "identity": {key: d["identity"][key] for key in ("device", "inode")},
                                  "source_sha256": d["source_sha256"],
                                  "utf8_bytes": d["utf8_bytes"]}
                                 for d in dependencies],
                "diagnostics_complete": result.get("diagnostics_complete"),
                "diagnostics": [{"origin": d.get("origin"), "path": d.get("path"),
                                 "source_sha256": d.get("source_sha256"), "line": d.get("line"),
                                 "column": d.get("column"), "category": d.get("category"),
                                 "message_sha256": sha(str(d.get("message", "")))}
                                for d in diagnostics], "generated_in_fixture": generated}

    def validate(self, editor, project, session, name, source, expected, *, path=ROOT,
                 origin=None, dependency=None, generate=None, wrong_session=False,
                 wrong_hash=False, wrong_document=False, wrong_thread=False,
                 closed=False, wrong_path=False, reason=None):
        before, disks_before = self.snapshot(editor, project)
        request_id = secrets.token_hex(16)
        selected_session = secrets.token_hex(16) if wrong_session else session
        argument = {"session_id": selected_session, "request_id": request_id,
                    "source_path": path, "source": "" if generate else source,
                    "expected_source_sha256": sha(source), "wrong_hash": wrong_hash,
                    "wrong_document": wrong_document, "wrong_thread": wrong_thread}
        if generate:
            argument["generate"] = generate
        self.target_document = {"resource_instance_id": before["subject"]["script_id"],
                                "editor_instance_id": before["subject"]["editor_id"],
                                "buffer_instance_id": before["subject"]["buffer_id"]}
        result = self.native_action(editor, "native_validate", **argument)
        after, disks_after = self.snapshot(editor, project)
        observation.require(self.source_free_state(before, disks_before) ==
                            self.source_free_state(after, disks_after) and
                            before["subject"]["R"] == after["subject"]["R"] and
                            before["subject"]["B"] == after["subject"]["B"] and
                            before["other"]["R"] == after["other"]["R"] and
                            before["other"]["B"] == after["other"]["B"] and
                            disks_before == disks_after,
                            "native_validation_changed_independent_source_dirty_history_or_selection_" + name)
        observation.require(result.get("actual_input_sha256") == sha(source) and
                            result.get("actual_input_utf8_bytes") == len(source.encode("utf-8")),
                            "fixture_input_bytes_independently_match_" + name)
        boundary = (wrong_hash or wrong_session or wrong_document or wrong_thread or
                    closed or wrong_path or generate == "over")
        if expected == "unavailable" and boundary:
            native = result.get("result", {})
            reason = ("wrong_thread" if wrong_thread else
                      "session_unavailable" if closed else
                      "wrong_session" if wrong_session else
                      "root_source_limit" if generate == "over" else
                      "document_mismatch" if wrong_document or wrong_path else
                      "source_mismatch")
            observation.require(isinstance(native, dict) and native.get("status") == "unavailable" and
                                native.get("reason") == reason and
                                native.get("diagnostics_complete") is False and
                                native.get("diagnostics") == [] and
                                native.get("dependencies") == [],
                                "actual_native_boundary_refusal_" + name)
            returned_document = native.get("document")
            requested_document = dict(self.target_document)
            if wrong_document:
                requested_document["buffer_instance_id"] = "0"
            observation.require(native.get("request_id") in (None, request_id) and
                                native.get("session_id") in (None, selected_session) and
                                native.get("purpose") in (None, "preflight") and
                                returned_document in (None, requested_document) and
                                native.get("source_path") in (None, path),
                                "boundary_correlation_only_when_actually_returned_" + name)
            returned_input = native.get("input")
            observation.require(isinstance(returned_input, dict) and
                                returned_input.get("source_sha256") in (None, sha(source)) and
                                returned_input.get("utf8_bytes") in
                                (None, len(source.encode("utf-8"))),
                                "no_invented_native_source_input_" + name)
            clock = native.get("collection", {})
            observation.require(isinstance(clock, dict) and clock.get("clock_id") == "editor" and
                                all(isinstance(clock.get(key), str) and clock[key].isdigit()
                                    for key in ("started_tick", "finished_tick")),
                                "boundary_actual_editor_clock_" + name)
            proof = {"status": native["status"], "reason": native["reason"],
                     "returned_request_id": native.get("request_id"),
                     "returned_session_id": native.get("session_id"),
                     "returned_document": returned_document,
                     "returned_source_path": native.get("source_path"),
                     "returned_input": returned_input, "collection": clock,
                     "requested_request_id": request_id,
                     "requested_input_sha256": sha(source),
                     "requested_input_utf8_bytes": len(source.encode("utf-8"))}
        else:
            proof = self.check_result(project, result.get("result"), source, path,
                                      request_id, session, expected, origin=origin,
                                      dependency=dependency, generated=bool(generate))
        if reason is not None:
            observation.require(result["result"].get("reason") == reason,
                                "native_specific_reason_" + name)
        self.case(name, mapping="T002/native-integration §4; quickstart §4.B",
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after), validation=proof,
                  source_surfaces="independent_d_r_b_dirty_versions_history",
                  source_in_artifact=False)
        return result["result"]

    def reentrant_reader(self, editor, project, session):
        before, disks_before = self.snapshot(editor, project)
        request_id = secrets.token_hex(16)
        self.target_document = {"resource_instance_id": before["subject"]["script_id"],
                                "editor_instance_id": before["subject"]["editor_id"],
                                "buffer_instance_id": before["subject"]["buffer_id"]}
        response = self.native_action(
            editor, "native_reentry", session_id=session, request_id=request_id,
            source=DIRECT, source_path=ROOT, expected_source_sha256=sha(DIRECT))
        after, disks_after = self.snapshot(editor, project)
        observation.require(self.source_free_state(before, disks_before) ==
                            self.source_free_state(after, disks_after) and
                            before["subject"]["R"] == after["subject"]["R"] and
                            before["subject"]["B"] == after["subject"]["B"] and
                            disks_before == disks_after, "native_reentry_read_only")
        value = response.get("result", {})
        observation.require(isinstance(value, dict) and value.get("reader_calls", 0) >= 1,
                            "real_native_reader_called_during_outer_analysis")
        nested = value.get("nested", {})
        observation.require(isinstance(nested, dict) and
                            nested.get("status") == "unavailable" and
                            nested.get("reason") == "reentrant" and
                            nested.get("diagnostics_complete") is False and
                            nested.get("source_path") is None and
                            nested.get("request_id") == request_id and
                            nested.get("input", {}).get("source_sha256") == sha(DIRECT),
                            "real_nested_engine_entry_refused_before_reader")
        outer = self.check_result(project, value.get("outer"), DIRECT, ROOT, request_id, session,
                                  "valid", dependency="res://scripts/native/level_one.gd")
        self.case("native_nested_entry_real_custom_reader",
                  mapping="T002/native-integration §4 effect and reentrancy",
                  outer=outer, nested_status=nested["status"], nested_reason=nested["reason"],
                  reader_calls=value["reader_calls"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="independent_d_r_b_dirty_versions_history")

    def dependency_read_invalidation(self, editor, project, session):
        dependency = project / "scripts" / "native" / "level_one.gd"
        original = dependency.read_bytes()
        before, disks_before = self.snapshot(editor, project)
        request_id = secrets.token_hex(16)
        self.target_document = {"resource_instance_id": before["subject"]["script_id"],
                                "editor_instance_id": before["subject"]["editor_id"],
                                "buffer_instance_id": before["subject"]["buffer_id"]}
        reached = editor["control"] / "fixture_dependency_read"
        release = editor["control"] / "fixture_dependency_release"
        observation.require(not reached.exists() and not release.exists(),
                            "new_dependency_barrier_has_no_old_event")
        with ThreadPoolExecutor(max_workers=1) as executor:
            pending = executor.submit(self.native_action, editor, "native_dependency_barrier",
                                      session_id=session, request_id=request_id, source=DIRECT,
                                      source_path=ROOT, expected_source_sha256=sha(DIRECT))
            try:
                observation.wait_for(lambda: reached.is_file(), "actual_native_reader_initial_read", timeout=6)
                dependency.write_text('extends RefCounted\nvar = # FIXTURE_MIDCALL_DEPENDENCY_CHANGE\n')
                release.touch(mode=0o600, exist_ok=False)
                response = pending.result(timeout=15)
            finally:
                if not release.exists():
                    release.touch(mode=0o600)
                dependency.write_bytes(original)
        reached.unlink()
        release.unlink()
        after, disks_after = self.snapshot(editor, project)
        before_state = self.source_free_state(before, disks_before)
        after_state = self.source_free_state(after, disks_after)
        unchanged = (before_state == after_state and
                     before["subject"]["R"] == after["subject"]["R"] and
                     before["subject"]["B"] == after["subject"]["B"] and
                     dependency.read_bytes() == original and disks_before == disks_after)
        if not unchanged:
            self.summary["failed_state_change"] = {"before": before_state, "after": after_state}
        observation.require(unchanged, "dependency_invalidation_root_and_other_work_unchanged")
        value = response.get("result", {})
        observation.require(isinstance(value, dict) and value.get("reader_calls", 0) >= 2,
                            "native_reader_rechecked_after_dependency_write")
        outer = value.get("outer")
        proof = self.check_result(project, outer, DIRECT, ROOT, request_id, session, "unavailable")
        observation.require(outer.get("reason") == "context_invalidated" and
                            outer.get("dependencies_current") is False,
                            "actual_midcall_dependency_invalidation_unavailable")
        self.case("native_midcall_dependency_change_invalidates",
                  mapping="T002/native-integration §4 dependency recheck",
                  result=proof, native_reader_calls=value["reader_calls"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="independent_d_r_b_dirty_versions_history")

    def cyclic_parser_lifetime(self, editor, project, session):
        before, disks_before = self.snapshot(editor, project)
        source = 'extends "./native/cycle.gd"\n'
        result = self.native_action(
            editor, "native_cycle_validation", source=source, source_path=ROOT,
            request_id=secrets.token_hex(16), session_id=session,
            expected_source_sha256=sha(source))
        after, disks_after = self.snapshot(editor, project)
        observation.require(result.get("statuses") == ["invalid"] * 32,
                            "cyclic_root_overlay_rejected_on_every_attempt")
        observation.require(result["object_count_after"] == result["object_count_before"],
                            "call_local_cyclic_parser_objects_released")
        observation.require(self.source_free_state(before, disks_before) ==
                            self.source_free_state(after, disks_after),
                            "cyclic_validation_preserves_document_and_history")
        self.case("native_cyclic_parser_lifetime", attempts=32,
                  before_objects=result["object_count_before"],
                  after_objects=result["object_count_after"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="independent_d_r_b_dirty_versions_history")

    def path_and_message_limits(self, editor, project, session):
        prefix = "res://scripts/native/cold/"
        directories = ["p" + str(index) + "x" * 238 for index in range(8)]
        remaining = 2048 - len(prefix) - sum(len(part) + 1 for part in directories) - len("target.gd") - 1
        directories.append("z" * remaining)
        exact_path = prefix + "/".join(directories) + "/target.gd"
        over_path = exact_path.removesuffix("target.gd") + "targetx.gd"
        observation.require(len(exact_path.encode()) == 2048 and len(over_path.encode()) == 2049,
                            "fixture_exact_dependency_path_bounds")
        fd = os.open(project / "scripts/native/cold", os.O_RDONLY | os.O_DIRECTORY)
        try:
            for part in directories:
                os.mkdir(part, mode=0o700, dir_fd=fd)
                child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                os.close(fd)
                fd = child
            for leaf in ("target.gd", "targetx.gd"):
                with os.fdopen(os.open(leaf, os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                       0o600, dir_fd=fd), "w") as output:
                    output.write("extends RefCounted\n")
        finally:
            os.close(fd)
        self.validate(editor, project, session, "native_exact_dependency_path",
                      f'extends "{exact_path}"\n', "valid", dependency=exact_path)
        self.validate(editor, project, session, "native_over_dependency_path",
                      f'extends "{over_path}"\n', "unavailable", reason="path_limit")

        identifier = "missing_length_probe"
        def source_for(name):
            return f"extends RefCounted\nfunc check():\n\t{name}()\n"
        sample = self.validate(editor, project, session, "native_diagnostic_length_profile",
                               source_for(identifier), "invalid", origin="root")
        observation.require(len(sample["diagnostics"]) == 1, "single_native_method_diagnostic")
        overhead = len(sample["diagnostics"][0]["message"].encode()) - len(identifier)
        exact = self.validate(editor, project, session, "native_exact_diagnostic_message_bytes",
                              source_for("m" * (2048 - overhead)), "invalid", origin="root")
        observation.require(len(exact["diagnostics"][0]["message"].encode()) == 2048,
                            "exactly_two_kibibytes_of_actual_diagnostic")
        self.validate(editor, project, session, "native_over_diagnostic_message_bytes",
                      source_for("m" * (2049 - overhead)), "unavailable", reason="diagnostic_limit")

    def native_validation(self):
        self.compile_window_probe()
        project = self.fixture("native-parser")
        self.installed_native(project)
        self.install_reentry_fixture(project)
        editor = self.start_editor(project, configured=False)
        info = self.native_action(editor, "native_info")
        observation.require(info.get("api_installed") is True and info.get("api_revision") == 1
                            and isinstance(info.get("build_id"), str) and
                            HEX64.fullmatch(info["build_id"]) and
                            info["build_id"] == self.expected_native_build_id,
                            "patched_engine_native_api_build_identity")
        self.summary["native_api_revision"] = info["api_revision"]
        self.summary["native_build_id"] = info["build_id"]
        observation.require(not self.descriptors(project), "patched_editor_has_no_v1_bridge_advertisement")
        session = secrets.token_hex(16)
        observation.require(self.native_action(editor, "native_configure", session_id=session)
                            .get("configured") is True, "fixture_owned_native_session_configured")
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.action(editor, "seed_sequence_history")
        self.native_action(editor, "native_idle")
        before, disks = self.snapshot(editor, project)
        observation.require(before["subject"]["has_redo"] and before["other"]["dirty"] and
                            before["current_script"] == "res://scripts/other.gd",
                            "fixture_real_prior_history_unrelated_dirty_nonselected_target")
        before_capture = self.screenshot(editor, "native-validation-before.png")
        self.validate(editor, project, session, "native_valid_unicode_root",
                      'extends RefCounted\n# NATIVE_VALID_ROOT café\n', "valid")
        self.validate(editor, project, session, "native_empty_source", "", "valid")
        self.validate(editor, project, session, "native_invalid_root_parser",
                      'extends RefCounted\nvar = # NATIVE_INVALID_ROOT\n', "invalid", origin="root")
        self.validate(editor, project, session, "native_invalid_root_analyzer",
                      'extends RefCounted\nconst BAD: int = "not an integer"\n', "invalid", origin="root")
        self.validate(editor, project, session, "native_direct_relative_dependency", DIRECT,
                      "valid", dependency="res://scripts/native/level_one.gd")
        self.reentrant_reader(editor, project, session)
        nested = self.validate(editor, project, session, "native_transitive_nested_relative", NESTED,
                               "valid", dependency="res://scripts/native/deep/level_two.gd")
        paths = {d.get("path") for d in nested.get("dependencies", [])}
        observation.require({"res://scripts/native/level_one.gd",
                             "res://scripts/native/sibling.gd"} <= paths,
                            "native_nested_relative_and_transitive_path_witnesses")
        self.dependency_read_invalidation(editor, project, session)
        dependency = project / "scripts" / "native" / "level_one.gd"
        initial = dependency.read_bytes()
        try:
            dependency.write_text('extends RefCounted\nvar = # NATIVE_INVALID_DEPENDENCY\n')
            self.validate(editor, project, session, "native_changed_dependency_parser", DIRECT,
                          "invalid", origin="dependency", dependency="res://scripts/native/level_one.gd")
        finally:
            dependency.write_bytes(initial)
        self.validate(editor, project, session, "native_stale_cache_revalidated", DIRECT,
                      "valid", dependency="res://scripts/native/level_one.gd")
        self.validate(editor, project, session, "native_missing_confined_dependency",
                      'extends "./native/absent.gd"\n', "invalid", origin="dependency",
                      dependency="res://scripts/native/absent.gd")
        self.validate(editor, project, session, "native_unconfined_dependency",
                      'extends "../../../outside.gd"\n', "unavailable")
        outside = project.parent / "outside-owned-source.gd"
        outside.write_text('extends RefCounted\n# OUTSIDE_SOURCE_SENTINEL\n')
        link = project / "scripts" / "native" / "redirect.gd"
        link.symlink_to(outside)
        self.source_markers.add(b"OUTSIDE_SOURCE_SENTINEL")
        self.validate(editor, project, session, "native_symlink_dependency_refused",
                      'extends "./native/redirect.gd"\n', "unavailable")
        observation.require(outside.read_text() == 'extends RefCounted\n# OUTSIDE_SOURCE_SENTINEL\n',
                            "outside_source_never_written")
        dependency.chmod(0)
        try:
            self.validate(editor, project, session, "native_unreadable_dependency_refused",
                          DIRECT, "unavailable")
        finally:
            dependency.chmod(0o600)
        acl_rule = "everyone allow read"
        acl_add = observation.run(["chmod", "+a", acl_rule, dependency])
        observation.require(acl_add.returncode == 0, "fixture_allow_acl_installed")
        try:
            self.validate(editor, project, session, "native_allow_acl_dependency_refused",
                          DIRECT, "unavailable")
        finally:
            acl_remove = observation.run(["chmod", "-a", acl_rule, dependency])
            observation.require(acl_remove.returncode == 0, "fixture_allow_acl_removed")
        remap = dependency.with_suffix(".gd.remap")
        remap.write_text('[remap]\npath="res://scripts/native/cold/not-read.gd"\n')
        try:
            self.validate(editor, project, session, "native_remapped_dependency_refused",
                          DIRECT, "unavailable", reason="unsupported_remap")
        finally:
            remap.unlink()
        (project / "scripts/native/cold/bytecode.gdc").write_bytes(b"not GDScript bytecode")
        self.validate(editor, project, session, "native_binary_dependency_refused",
                      'extends RefCounted\nconst BINARY = preload("./native/cold/bytecode.gdc")\n',
                      "unavailable", reason="unsupported_effect")
        effect_file = project / "scripts" / "native" / "excluded.native_effect"
        effect_file.write_text("fixture excluded loader payload\n")
        effect = editor["control"] / "excluded_loader_was_called"
        observation.require(not effect.exists(), "loader_sentinel_clean_before_validation")
        self.validate(editor, project, session, "native_excluded_loader_before_hook", LOADER,
                      "unavailable", reason="unsupported_effect")
        observation.require(not effect.exists(), "loader_exists_type_load_callbacks_not_invoked")
        tool_effects = ("excluded_initializer_was_called", "excluded_constructor_was_called")
        observation.require(all(not (editor["control"] / marker).exists() for marker in tool_effects),
                            "tool_effects_not_preexisting")
        observation.require(self.native_action(editor, "native_info").get("tool_cached") is False,
                            "tool_effect_script_not_loaded_or_scanned")
        self.validate(editor, project, session, "native_tool_script_shallow_without_initialization",
                      'extends RefCounted\nconst TOOL = preload("./native/cold/tool_initializer.gd")\n',
                      "valid", dependency="res://scripts/native/cold/tool_initializer.gd")
        observation.require(self.native_action(editor, "native_info").get("tool_cached") is False,
                            "native_validation_does_not_publish_loaded_tool_script")
        observation.require(all(not (editor["control"] / name).exists() for name in tool_effects),
                            "tool_static_initializer_and_constructor_not_executed")
        getter_marker = editor["control"] / "excluded_getter_was_called"
        observation.require(not getter_marker.exists(), "getter_sentinel_not_preexisting")
        observation.require(self.native_action(editor, "native_effect_install")
                            .get("installed") is True, "real_dynamic_getter_fixture_installed")
        try:
            self.validate(editor, project, session, "native_dynamic_getter_refused_before_effect",
                          'extends RefCounted\nconst G = ValidationEffectSentinel.SENTINEL\n',
                          "unavailable")
            observation.require(not getter_marker.exists(), "dynamic_project_getter_not_executed")
            stringify_marker = editor["control"] / "excluded_stringify_was_called"
            observation.require(not stringify_marker.exists(), "stringify_sentinel_not_preexisting")
            self.validate(editor, project, session, "native_diagnostic_stringification_refused",
                          'extends RefCounted\nconst D = {ValidationEffectSentinel: 1, ValidationEffectSentinel: 2}\n',
                          "unavailable")
            observation.require(not stringify_marker.exists(),
                                "diagnostic_formatting_does_not_execute_project_to_string")
        finally:
            observation.require(self.native_action(editor, "native_effect_remove")
                                .get("removed") is True, "dynamic_getter_fixture_removed")
        self.validate(editor, project, session, "native_wrong_source_path", DIRECT,
                      "unavailable", path="res://scripts/other.gd", wrong_path=True)
        exact = 'extends RefCounted\n# ' + 'é' * 262133 + 'x'
        over = 'extends RefCounted\n# ' + 'x' * 524268
        observation.require(len(exact.encode()) == 524288 and len(over.encode()) == 524289,
                            "fixture_real_utf8_source_limits")
        self.validate(editor, project, session, "native_exact_root_limit", exact, "valid",
                      generate="exact")
        self.validate(editor, project, session, "native_over_root_limit", over, "unavailable",
                      generate="over")
        self.validate(editor, project, session, "native_wrong_hash", DIRECT, "unavailable",
                      wrong_hash=True)
        self.validate(editor, project, session, "native_wrong_document", DIRECT, "unavailable",
                      wrong_document=True)
        self.validate(editor, project, session, "native_wrong_session", DIRECT, "unavailable",
                      wrong_session=True)
        self.validate(editor, project, session, "native_wrong_thread", DIRECT, "unavailable",
                      wrong_thread=True)
        limit_dir = project / "scripts" / "native" / "limits"
        limit_dir.mkdir()
        for index in range(33):
            (limit_dir / f"dep_{index:02d}.gd").write_text(
                f"extends RefCounted\nconst FIXTURE_VALUE := {index}\n")
        def dependency_source(count):
            return "extends RefCounted\n" + "".join(
                f'const DEP_{index:02d} = preload("./native/limits/dep_{index:02d}.gd")\n'
                for index in range(count))
        at_count = self.validate(editor, project, session, "native_exact_dependency_count",
                                 dependency_source(32), "valid")
        observation.require(len(at_count["dependencies"]) == 32,
                            "exactly_thirty_two_distinct_read_dependencies")
        self.validate(editor, project, session, "native_over_dependency_count",
                      dependency_source(33), "unavailable")
        oversized = project / "scripts" / "native" / "large_dependency.gd"
        prefix = "extends RefCounted\n# "
        oversized.write_text(prefix + "x" * (524288 - len(prefix.encode())))
        large_root = 'extends RefCounted\nconst LARGE = preload("./native/large_dependency.gd")\n'
        self.validate(editor, project, session, "native_exact_dependency_source_bytes",
                      large_root, "valid", dependency="res://scripts/native/large_dependency.gd")
        oversized.write_text(prefix + "x" * (524289 - len(prefix.encode())))
        self.validate(editor, project, session, "native_over_dependency_source_bytes",
                      large_root, "unavailable", reason="source_limit")
        for index in range(8):
            (limit_dir / f"bytes_{index:02d}.gd").write_text(
                prefix + "x" * (524288 - len(prefix.encode())))
        (limit_dir / "bytes_08.gd").write_text("\n")
        def byte_total_source(count):
            return "extends RefCounted\n" + "".join(
                f'const BYTE_{index:02d} = preload("./native/limits/bytes_{index:02d}.gd")\n'
                for index in range(count))
        total = self.validate(editor, project, session, "native_exact_total_dependency_bytes",
                              byte_total_source(8), "valid")
        observation.require(sum(value["utf8_bytes"] for value in total["dependencies"])
                            == 4 * 1024 * 1024, "exactly_four_megabytes_of_actual_dependencies")
        self.validate(editor, project, session, "native_over_total_dependency_bytes",
                      byte_total_source(9), "unavailable")
        def errors(count):
            return "extends RefCounted\n" + "".join(
                f'func check_{index:02d}():\n\tmissing_{index:02d}()\n' for index in range(count))
        diagnostic_limit = self.validate(editor, project, session,
                                         "native_exact_diagnostic_limit", errors(64),
                                         "invalid", origin="root")
        observation.require(len(diagnostic_limit["diagnostics"]) == 64,
                            "exactly_sixty_four_real_analyzer_diagnostics")
        self.validate(editor, project, session, "native_over_diagnostic_limit",
                      errors(65), "unavailable")
        self.path_and_message_limits(editor, project, session)
        self.native_action(editor, "native_context", mapping="context_node")
        first = self.validate(editor, project, session, "native_autoload_context_initial", CONTEXT,
                              "valid")
        self.native_action(editor, "native_context", mapping="context_other")
        second = self.validate(editor, project, session, "native_autoload_context_changed", CONTEXT,
                               "valid")
        observation.require(first["context"] != second["context"],
                            "actual_autoload_context_generation_changed")
        self.native_action(editor, "native_context", mapping="context_invalid")
        self.validate(editor, project, session, "native_invalid_autoload_cannot_certify_valid",
                      "extends Node\nfunc example():\n\treturn ValidationDependency\n",
                      "invalid", origin="dependency",
                      dependency="res://scripts/native/context_invalid.gd")
        self.native_action(editor, "native_context", mapping="context_node")
        self.cyclic_parser_lifetime(editor, project, session)
        replay = self.action(editor, "replay_sequence_history")
        observation.require(replay["after"]["has_redo"], "real_prior_undo_redo_reachable_after_native_B")
        after_capture = self.screenshot(editor, "native-validation-after.png")
        self.case("native_prior_history_replayed_after_validation",
                  source_surfaces="actual_codeedit_undo_redo", action_count=len(replay["steps"]),
                  before_capture={"path": before_capture,
                                  "sha256": observation.digest(self.artifacts / before_capture)},
                  after_capture={"path": after_capture,
                                 "sha256": observation.digest(self.artifacts / after_capture)})
        disabled = self.action(editor, "disable")
        observation.require(disabled.get("plugin_enabled") is False, "product_addon_disabled")
        self.validate(editor, project, session, "native_disable_closes_active_session", DIRECT,
                      "unavailable", closed=True)
        observation.require(self.native_action(editor, "native_configure", session_id=session)
                            .get("configured") is True, "fixture_owned_session_without_product_plugin")
        self.validate(editor, project, session, "native_present_addon_disabled_validation",
                      DIRECT, "valid", dependency="res://scripts/native/level_one.gd")
        self.native_action(editor, "native_close")
        self.validate(editor, project, session, "native_closed_session_refusal", DIRECT,
                      "unavailable", closed=True)
        enabled = self.action(editor, "enable")
        observation.require(enabled.get("plugin_enabled") is True, "product_addon_reenabled")
        self.validate(editor, project, session, "native_reenable_without_bridge_no_session",
                      DIRECT, "unavailable", closed=True)

    def native_export(self):
        project = self.fixture("native-export-installed")
        self.installed_native(project)
        super().export_boundary()
        for label in ("enabled", "disabled", "hook-only"):
            out = self.work / ("export-" + label)
            self.installed_native(out)
            app = self.work / ("export-output-" + label) / "fixture.app"
            observation.require(not any(
                "godot_agent_kit" in str(path.relative_to(app)) or
                "fixture_driver" in str(path.relative_to(app)) or
                path.name.startswith(("libscript_edit", "libreentry")) or
                path.suffix == ".gdextension"
                for path in app.rglob("*") if path.is_file()),
                "native_extension_and_tooling_not_bundled_in_game_" + label)

    def inspect_pack(self, entries, stage):
        super().inspect_pack(entries, stage)
        observation.require(not any(name.endswith((".gdextension", ".dylib", ".so", ".dll"))
                                    for name in entries), "native_tooling_absent_from_pack_" + stage)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", required=True, type=Path)
    parser.add_argument("--observer", required=True, type=Path)
    parser.add_argument("--scenario", required=True, choices=("native-validation",))
    parser.add_argument("--artifacts", required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    for path in (args.godot, args.observer):
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
                            "absolute_executable_required")
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and
                        not args.artifacts.is_symlink(), "absolute_existing_artifact_directory")
    metadata = args.artifacts.stat()
    observation.require(metadata.st_uid == os.geteuid() and stat.S_IMODE(metadata.st_mode) == 0o700
                        and not list(args.artifacts.iterdir()), "empty_private_artifact_directory")
    version = observation.run([args.godot, "--version"])
    observation.require(version.returncode == 0 and re.fullmatch(
        r"4\.7\.2\.stable\.[^\s]+\.ed1daf0bf", version.stdout.decode().strip()),
        "pinned_4_7_2_engine_version_required")
    args.candidate_version = version.stdout.decode().strip()
    args.candidate_engine_hash = observation.ENGINE_HASH
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-native-validation-", dir=Path.home()) as temp:
        harness = NativeHarness(args, Path(temp))
        status = 0
        try:
            harness.initialize()
            harness.group("native-validation", harness.native_validation)
            harness.group("native-export", harness.native_export)
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError,
                EOFError, subprocess.SubprocessError) as error:
            status = 1
            harness.summary["status"] = "failed"
            harness.summary["stage"] = str(error) if isinstance(error, observation.Failure) else type(error).__name__
        finally:
            try:
                harness.cleanup()
            except (observation.Failure, OSError) as error:
                status = 1
                harness.summary["status"] = "failed"
                harness.summary["cleanup_stage"] = (str(error) if isinstance(error, observation.Failure)
                                                    else type(error).__name__)
            try:
                harness.verify_incidental_redaction()
            except (observation.Failure, OSError) as error:
                status = 1
                harness.summary["status"] = "failed"
                harness.summary["redaction_stage"] = (str(error) if isinstance(error, observation.Failure)
                                                      else type(error).__name__)
            harness.summary["artifacts_sha256"] = {
                str(path.relative_to(args.artifacts)): observation.digest(path)
                for path in sorted(args.artifacts.rglob("*")) if path.is_file()
            }
            harness.summary["passed_case_count"] = len(harness.cases) if status == 0 else 0
            observation.json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps({"status": harness.summary["status"], "stage": harness.summary.get("stage"),
                          "passed_cases": harness.summary["passed_case_count"],
                          "summary": str(args.artifacts / "summary.json")}))
        return status


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except observation.Failure as error:
        print("Acceptance prerequisite failed: " + str(error))
        raise SystemExit(1)
