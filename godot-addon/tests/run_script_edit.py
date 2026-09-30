#!/usr/bin/env python3
"""Real edit-gdscript GUI caller acceptance and retained native primitives.

Each selected edit group uses the bounded public stdin CLI and independent
editor, disk, history and owned-window witnesses.
"""
from __future__ import annotations

import argparse
import configparser
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
from stock_acceptance import StockAcceptanceMixin
from native_finalization_acceptance import NativeFinalizationMixin
from caller_edit_acceptance import CallerEditAcceptanceMixin
from cumulative_edit_acceptance import CumulativeEditAcceptanceMixin

FIXTURE = Path(__file__).parent / "fixtures" / "script_edit"
SCENARIOS = ("native-primitives", "clean-open", "conflicts", "routing", "interruption",
             "validation", "history", "durability", "sequential", "privacy-export")
MARKERS = (b"NATIVE_DIRECT", b"NATIVE_TRANSITIVE", b"NATIVE_SIBLING_VALUE",
           b"NATIVE_DEPENDENCY_VALUE", b"not an int", b"func value() -> int:",
           b"ACTUAL_PROPOSAL_PARSE_FAILURE", b"HUMAN_NEWER_TEXT",
           b"HUMAN_POST_CHANGE_INVALID", b"unsupported_fixture_rules_shape")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
STOCK_SHA256 = "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf"


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


class NativeHarness(CumulativeEditAcceptanceMixin, CallerEditAcceptanceMixin,
                    NativeFinalizationMixin, StockAcceptanceMixin, observation.Harness):
    stock_sha = staticmethod(sha)
    stock_dependency = staticmethod(dependency_witness)

    def __init__(self, args, work):
        super().__init__(args, work)
        self.source_markers.update(MARKERS)
        self.source_markers.update((b"STOCK_PRIVATE_PARSER_SOURCE",
                                    b"STOCK_PRIVATE_WARNING_SOURCE",
                                    b"STATIC_INITIALIZER_EXECUTED",
                                    b"OUTSIDE_STOCK_SECRET"))
        self.stock_sessions = {}
        self.summary["coverage_scope"] = (
            "t005_complete_edit_and_native_groups" if args.scenario == "all" else
            "t003_stock_helper_native_finalization" if args.scenario == "native-primitives" else
            "t005_selected_caller_story_group" if args.scenario in
            ("durability", "sequential", "privacy-export") else
            "t004_selected_caller_story_group")
        self.summary["source_observation"] = True
        self.summary["fixture_driver_sha256"] = observation.digest(FIXTURE / "fixture_driver.gd")
        self.summary["caller_acceptance_sha256"] = observation.digest(
            Path(__file__).parent / "caller_edit_acceptance.py")
        self.summary["cumulative_acceptance_sha256"] = observation.digest(
            Path(__file__).parent / "cumulative_edit_acceptance.py")
        self.summary["caller_privacy_acceptance_sha256"] = observation.digest(
            Path(__file__).parent / "caller_privacy_acceptance.py")
        self.summary["edit_result_review_sha256"] = observation.digest(
            Path(__file__).parent / "edit_result_review.py")
        self.summary["edit_caller_sha256"] = (
            observation.digest(args.editor) if args.editor and args.editor.is_file() else None)
        self.summary["stock_validator_sha256"] = (
            observation.digest(args.stock_validator) if args.stock_validator and
            args.stock_validator.is_file() else None)
        self.summary["native_fixture_files"] = {str(p.relative_to(FIXTURE)): observation.digest(p)
                                               for p in sorted(FIXTURE.rglob("*")) if p.is_file()}

    def fixture(self, name, *, controlled=False):
        project = super().fixture(name, controlled=controlled)
        if not name.startswith("export-"):
            native = project / "scripts" / "native"
            (native / "deep").mkdir(parents=True)
            for relative in ("level_one.gd", "sibling.gd", "deep/level_two.gd"):
                origin = FIXTURE / "scripts" / "native" / relative
                shutil.copy2(origin, native / relative)
            cold = native / "cold"
            cold.mkdir()
            (cold / ".gdignore").touch()
            shutil.copy2(FIXTURE / "scripts/native/tool_initializer.gd",
                         cold / "tool_initializer.gd")
        helper = project / "addons" / "fixture_driver"
        shutil.copy2(FIXTURE / "fixture_driver.gd", helper / "native_fixture_driver.gd")
        (helper / "plugin.gd").write_text('@tool\nextends "res://addons/fixture_driver/native_fixture_driver.gd"\n')
        return project

    def native_action(self, editor, action, **arguments):
        if action in ("native_edit_save", "native_edit_save_clean"):
            # Control.has_focus alone does not establish a foreground OS window.
            # Deliver the ordinary shortcut once, after the owned surface is ready.
            self.present_editor(editor)
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
        if result.get("ok") is not True:
            observation.require(all(secret not in json.dumps(result).encode() for secret in self.secrets),
                                "failed_fixture_action_credential_redaction")
            observation.json_file(self.artifacts / ("failed-" + action + ".json"), result)
        observation.require(result.get("ok") is True, "native_fixture_action_failed_" + action)
        return {key: value for key, value in result.items() if key not in ("id", "action", "ok")}

    def installed_native(self, project, *, fixture_only=False):
        addon = project / "addons" / "godot_agent_kit"
        manifest = addon / "native" / "editor_integration.gdextension"
        binaries = list(addon.rglob("*.dylib"))
        build_manifest = addon / "native" / "build-manifest.json"
        observation.require(manifest.is_file() and build_manifest.is_file() and len(binaries) == 1,
                            "native_manifest_build_receipt_and_exactly_one_installed_library_required")
        descriptor = configparser.ConfigParser(interpolation=None)
        descriptor.read(manifest)
        observation.require(
            json.loads(descriptor["configuration"]["entry_symbol"]) == "editor_integration_library_init" and
            json.loads(descriptor["libraries"]["macos.arm64"]) ==
            "res://addons/godot_agent_kit/native/libeditor_integration.macos.arm64.dylib" and
            binaries[0] == addon / "native" / "libeditor_integration.macos.arm64.dylib",
            "installed_shared_native_descriptor_and_library_identity")
        provenance = json.loads(build_manifest.read_text())
        observation.require(
            provenance.get("base_commit") == observation.ENGINE_HASH and
            provenance.get("engine_version") == self.version and
            provenance.get("engine_sha256") == observation.digest(self.args.godot) and
            provenance.get("native_library_sha256") == observation.digest(binaries[0]) and
            provenance.get("fixture_only") is fixture_only and
            provenance.get("native_api_revision") == 2 and
            provenance.get("native_family") == "editor_integration" and
            provenance.get("native_library") == binaries[0].name and
            provenance.get("entry_symbol") == "editor_integration_library_init" and
            HEX64.fullmatch(str(provenance.get("native_build_id", ""))),
            "native_binary_generated_abi_and_pinned_engine_provenance")
        if not fixture_only:
            # Keep compatibility fields tied to the production build even when
            # a later fixture-only fault run installs a different binary.
            self.summary["native_manifest_sha256"] = observation.digest(manifest)
            self.summary["native_build_manifest_sha256"] = observation.digest(build_manifest)
            self.summary["gdextension_abi_sha256"] = provenance.get("abi_sha256")
            self.summary["gdextension_api_sha256"] = provenance.get("api_sha256")
            self.summary["native_toolchain"] = {"compiler": provenance.get("compiler"),
                                                "sdk": provenance.get("sdk")}
            self.summary["native_library_sha256"] = observation.digest(binaries[0])
            self.summary["native_library_relative_path"] = str(binaries[0].relative_to(project))
        self.expected_native_build_id = provenance["native_build_id"]
        self.summary.setdefault("native_artifacts", {})[
            "fixture" if fixture_only else "production"] = {
                "manifest_sha256": observation.digest(manifest),
                "build_manifest_sha256": observation.digest(build_manifest),
                "library_sha256": observation.digest(binaries[0]),
                "native_build_id": provenance["native_build_id"],
                "abi_sha256": provenance.get("abi_sha256"),
                "api_sha256": provenance.get("api_sha256"),
                "toolchain": {"compiler": provenance.get("compiler"), "sdk": provenance.get("sdk")},
                "library_relative_path": str(binaries[0].relative_to(project))}


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
                path.name.startswith("libeditor_integration") or
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
    parser.add_argument("--scenario", required=True, choices=(*SCENARIOS, "all"))
    parser.add_argument("--editor", type=Path, help="built edit-gdscript stdin caller")
    parser.add_argument("--stock-validator", type=Path,
                        help="built test-only Rust validator example (native-primitives)")
    parser.add_argument("--native-fault-addon", type=Path,
                        help="separate fixture-only native build for interruption and native-primitives")
    parser.add_argument("--artifacts", required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    for path in (args.godot, args.observer):
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
                            "absolute_executable_required")
    if args.scenario != "native-primitives":
        observation.require(args.editor is not None and args.editor.is_absolute() and
                            args.editor.is_file() and os.access(args.editor, os.X_OK),
                            "absolute_edit_caller_executable_required")
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and
                        not args.artifacts.is_symlink(), "absolute_existing_artifact_directory")
    metadata = args.artifacts.stat()
    observation.require(metadata.st_uid == os.geteuid() and stat.S_IMODE(metadata.st_mode) == 0o700
                        and not list(args.artifacts.iterdir()), "empty_private_artifact_directory")
    observation.require(observation.digest(args.godot) == STOCK_SHA256,
                        "exact_official_stock_binary_hash")
    version = observation.run([args.godot, "--version"])
    observation.require(version.returncode == 0 and
                        version.stdout.decode().strip() == observation.VERSION,
                        "pinned_official_4_7_2_engine_version_required")
    args.candidate_version = version.stdout.decode().strip()
    args.candidate_engine_hash = observation.ENGINE_HASH
    if args.scenario in ("native-primitives", "all"):
        observation.require(args.stock_validator is not None and
                            args.stock_validator.is_absolute() and
                            args.stock_validator.is_file() and
                            os.access(args.stock_validator, os.X_OK),
                            "built_stock_validator_fixture_required")
    if args.scenario in ("native-primitives", "interruption", "all"):
        observation.require(args.native_fault_addon is not None and
                            args.native_fault_addon.is_absolute() and
                            (args.native_fault_addon / "build-manifest.json").is_file() and
                            (args.native_fault_addon /
                             "libeditor_integration.macos.arm64.dylib").is_file(),
                            "separate_native_fixture_fault_artifact_required")
        fault_receipt = json.loads((args.native_fault_addon / "build-manifest.json").read_text())
        observation.require(fault_receipt.get("fixture_only") is True and
                            fault_receipt.get("engine_sha256") == STOCK_SHA256 and
                            fault_receipt.get("native_api_revision") == 2 and
                            fault_receipt.get("native_family") == "editor_integration" and
                            fault_receipt.get("native_library") == "libeditor_integration.macos.arm64.dylib" and
                            fault_receipt.get("entry_symbol") == "editor_integration_library_init" and
                            fault_receipt.get("native_library_sha256") == observation.digest(
                                args.native_fault_addon / "libeditor_integration.macos.arm64.dylib"),
                            "fixture_fault_artifact_never_product_library")
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-edit-acceptance-", dir=Path.home()) as temp:
        harness = NativeHarness(args, Path(temp))
        status = 0
        try:
            harness.initialize()
            for scenario in SCENARIOS:
                if args.scenario not in (scenario, "all"):
                    continue
                if scenario == "native-primitives":
                    harness.group("stock-validation", harness.stock_validation)
                    harness.group("native-finalization", harness.native_finalization)
                    if args.scenario == "native-primitives":
                        harness.group("native-export", harness.native_export)
                else:
                    actions = {
                        "clean-open": harness.clean_open_edit,
                        "conflicts": harness.conflict_edit,
                        "routing": harness.routing_edit,
                        "interruption": harness.interruption_edit,
                        "validation": lambda: (harness.validation_edit(),
                                               harness.post_change_validation_edit()),
                        "history": harness.history_edit,
                        "durability": harness.durability_edit,
                        "sequential": harness.sequential_edit,
                        "privacy-export": harness.privacy_export_edit,
                    }
                    harness.group(scenario, actions[scenario])
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
