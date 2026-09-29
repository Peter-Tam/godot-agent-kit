#!/usr/bin/env python3
"""Owned stock-helper and native-finalization acceptance.

The stock groups use independent live-editor and descriptor witnesses; neither
group requires a public edit caller (the authenticated caller is T004).
"""
from __future__ import annotations

import argparse
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

FIXTURE = Path(__file__).parent / "fixtures" / "script_edit"
MARKERS = (b"NATIVE_DIRECT", b"NATIVE_TRANSITIVE", b"NATIVE_SIBLING_VALUE",
           b"NATIVE_DEPENDENCY_VALUE", b"not an int")
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


class NativeHarness(NativeFinalizationMixin, StockAcceptanceMixin, observation.Harness):
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
        self.summary["coverage_scope"] = "t003_stock_helper_native_finalization"
        self.summary["source_observation"] = True
        self.summary["fixture_driver_sha256"] = observation.digest(FIXTURE / "fixture_driver.gd")
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

    def installed_native(self, project, *, fixture_only=False):
        addon = project / "addons" / "godot_agent_kit"
        manifest = addon / "native" / "script_edit.gdextension"
        binaries = list(addon.rglob("*.dylib"))
        build_manifest = addon / "native" / "build-manifest.json"
        observation.require(manifest.is_file() and build_manifest.is_file() and len(binaries) == 1,
                            "native_manifest_build_receipt_and_exactly_one_installed_library_required")
        provenance = json.loads(build_manifest.read_text())
        observation.require(
            provenance.get("base_commit") == observation.ENGINE_HASH and
            provenance.get("engine_version") == self.version and
            provenance.get("engine_sha256") == observation.digest(self.args.godot) and
            provenance.get("native_library_sha256") == observation.digest(binaries[0]) and
            provenance.get("fixture_only") is fixture_only and
            HEX64.fullmatch(str(provenance.get("native_build_id", ""))),
            "native_binary_generated_abi_and_pinned_engine_provenance")
        self.summary["native_manifest_sha256"] = observation.digest(manifest)
        self.summary["native_build_manifest_sha256"] = observation.digest(build_manifest)
        self.summary["gdextension_abi_sha256"] = provenance.get("abi_sha256")
        self.summary["gdextension_api_sha256"] = provenance.get("api_sha256")
        self.summary["native_toolchain"] = {"compiler": provenance.get("compiler"),
                                            "sdk": provenance.get("sdk")}
        self.summary["native_library_sha256"] = observation.digest(binaries[0])
        self.summary["native_library_relative_path"] = str(binaries[0].relative_to(project))
        self.expected_native_build_id = provenance["native_build_id"]


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
                path.name.startswith("libscript_edit") or
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
    parser.add_argument("--scenario", required=True, choices=("native-primitives",))
    parser.add_argument("--stock-validator", type=Path,
                        help="built test-only Rust validator example (native-primitives)")
    parser.add_argument("--native-fault-addon", type=Path,
                        help="isolated fixture build output native/ directory (native-primitives)")
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
    observation.require(observation.digest(args.godot) == STOCK_SHA256,
                        "exact_official_stock_binary_hash")
    version = observation.run([args.godot, "--version"])
    observation.require(version.returncode == 0 and
                        version.stdout.decode().strip() == observation.VERSION,
                        "pinned_official_4_7_2_engine_version_required")
    args.candidate_version = version.stdout.decode().strip()
    args.candidate_engine_hash = observation.ENGINE_HASH
    observation.require(args.stock_validator is not None and
                        args.stock_validator.is_absolute() and
                        args.stock_validator.is_file() and
                        os.access(args.stock_validator, os.X_OK),
                        "built_stock_validator_fixture_required")
    observation.require(args.native_fault_addon is not None and
                        args.native_fault_addon.is_absolute() and
                        (args.native_fault_addon / "build-manifest.json").is_file() and
                        (args.native_fault_addon /
                         "libscript_edit.macos.arm64.dylib").is_file(),
                        "separate_native_fixture_fault_artifact_required")
    fault_receipt = json.loads((args.native_fault_addon / "build-manifest.json").read_text())
    observation.require(fault_receipt.get("fixture_only") is True and
                        fault_receipt.get("engine_sha256") == STOCK_SHA256 and
                        fault_receipt.get("native_library_sha256") == observation.digest(
                            args.native_fault_addon / "libscript_edit.macos.arm64.dylib"),
                        "fixture_fault_artifact_never_product_library")
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-native-primitives-", dir=Path.home()) as temp:
        harness = NativeHarness(args, Path(temp))
        status = 0
        try:
            harness.initialize()
            harness.group("stock-validation", harness.stock_validation)
            harness.group("native-finalization", harness.native_finalization)
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
