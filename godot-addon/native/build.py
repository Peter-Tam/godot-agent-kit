#!/usr/bin/env python3
"""Build editor-only validation against the ABI emitted by one patched Godot binary."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

BASE = "ed1daf0bf001b61586d9930840f2f1394092c079"
HERE = Path(__file__).resolve().parent
ADDON = HERE.parent / "addons" / "godot_agent_kit" / "native"
BUILD = HERE / "build"


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def run(args, **kwargs):
    return subprocess.run(args, check=True, text=True, capture_output=True, **kwargs)


def method(api, cls, name):
    entry = next(c for c in api["classes"] if c["name"] == cls)
    methods = [m for m in entry.get("methods", []) if m["name"] == name]
    if len(methods) != 1:
        raise ValueError(f"Required method missing or ambiguous: {cls}.{name}")
    return methods[0]["hash"]


def single_precision_sizes(api):
    if api["header"].get("precision") != "single":
        raise ValueError("the native integration requires the engine's single-precision ABI")
    raw_sizes = next((x["sizes"] for x in api["builtin_class_sizes"] if x["build_configuration"] == "float_64"), None)
    if raw_sizes is None:
        raise ValueError("missing float_64 Godot builtin sizes")
    return {entry["name"]: entry["size"] for entry in raw_sizes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", required=True, type=Path, help="absolute patched editor executable")
    args = parser.parse_args()
    engine = args.godot.resolve(strict=True)
    if not args.godot.is_absolute() or not engine.is_file():
        parser.error("--godot must be an absolute existing engine executable")
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        parser.error("the current native reader supports only macOS arm64")
    source = next((p for p in engine.parents if (p / "main/main.cpp").exists() and (p / ".git").exists()), None)
    if source is None:
        parser.error("engine must reside inside its pinned, patched source checkout")
    if run(["git", "-C", str(source), "rev-parse", "HEAD"]).stdout.strip() != BASE:
        parser.error("engine checkout does not match pinned Godot base")
    patch = HERE / "engine-api.patch"
    if not patch.is_file():
        parser.error("missing reviewed native engine-api.patch")
    try:
        run(["git", "-C", str(source), "apply", "--reverse", "--check", str(patch)])
    except subprocess.CalledProcessError as exc:
        parser.error("engine checkout does not match the native patch: " + exc.stderr.strip())
    version = run([str(engine), "--version"]).stdout.strip()
    if not version.startswith("4.7.2.stable.") or BASE[:9] not in version:
        parser.error("engine binary does not report pinned 4.7.2 base")
    BUILD.mkdir(parents=True, exist_ok=True)
    run([str(engine), "--headless", "--dump-gdextension-interface", "--dump-gdextension-interface-json", "--dump-extension-api"], cwd=BUILD)
    header, abi_json, api_json = (BUILD / n for n in ("gdextension_interface.h", "gdextension_interface.json", "extension_api.json"))
    api = json.loads(api_json.read_text())
    sizes = single_precision_sizes(api)
    if (api["header"]["version_major"] != 4 or api["header"]["version_minor"] != 7
            or api["header"]["version_patch"] != 2 or api["header"]["version_status"] != "stable"
            or not api["header"]["version_build"]
            or f'.{api["header"]["version_build"]}.' not in version):
        parser.error("generated API version/build does not match the pinned binary")
    required = {"Variant", "String", "StringName", "Dictionary", "Array", "Callable"}
    if not required <= sizes.keys():
        parser.error("generated ABI is missing native value sizes")
    methods = {
        "GDSCRIPT_REVISION": method(api, "GDScript", "gdscript_validation_api_revision"),
        "GDSCRIPT_VALIDATE": method(api, "GDScript", "validate_gdscript_source"),
        "SCRIPT_EDITOR_EDITED": method(api, "ScriptEditorBase", "get_edited_resource"),
    }
    engine_sha = digest(engine)
    inputs = {p.name: digest(p) for p in (patch, header, abi_json, api_json, HERE / "extension.cpp", HERE / "validation.cpp", HERE / "native.hpp", HERE / "build.py")}
    compiler = os.environ.get("CXX", "clang++")
    compiler_version = run([compiler, "--version"]).stdout.splitlines()[0]
    sdk_version = run(["xcrun", "--show-sdk-version"]).stdout.strip()
    flags = ["-std=c++17", "-O2", "-fPIC", "-fvisibility=hidden", "-dynamiclib", "-Wall", "-Wextra", "-Werror", "-I", str(BUILD), "-I", str(HERE)]
    build_id = hashlib.sha256(json.dumps({
        "engine_sha256": engine_sha, "inputs": inputs, "configuration": "float_64/macos-arm64",
        "compiler": compiler_version, "sdk": sdk_version, "flags": flags,
    }, sort_keys=True).encode()).hexdigest()
    macros = ["// Generated by build.py from the exact --godot binary; never hand-edit.", "#pragma once"]
    macros += [f"#define GAK_SIZE_{k.upper()} {sizes[k]}" for k in sorted(required)]
    macros += [f"#define GAK_HASH_{k} {v}ULL" for k, v in methods.items()]
    macros += [f'#define GAK_ENGINE_HASH "{BASE}"',
               f'#define GAK_ENGINE_BUILD "{api["header"]["version_build"]}"',
               f'#define GAK_ENGINE_BINARY_SHA256 "{engine_sha}"',
               f'#define GAK_BUILD_ID "{build_id}"']
    (BUILD / "native_abi_sizes.h").write_text("\n".join(macros) + "\n")
    output = ADDON / "libscript_edit.macos.arm64.dylib"
    ADDON.mkdir(parents=True, exist_ok=True)
    run([compiler, *flags, str(HERE / "extension.cpp"), str(HERE / "validation.cpp"), "-o", str(output)])
    manifest = {
        "base_commit": BASE, "engine_version": version, "engine_binary": str(engine), "engine_sha256": engine_sha,
        "abi_sha256": digest(abi_json), "api_sha256": digest(api_json), "header_sha256": digest(header),
        "patch_sha256": digest(patch), "native_build_id": build_id, "native_library_sha256": digest(output),
        "source_sha256": {key: value for key, value in inputs.items() if key.endswith((".cpp", ".hpp", ".py"))},
        "compiler": compiler_version,
        "sdk": sdk_version, "build_flags": flags,
        "platform": "macos-arm64", "generated_from_exact_binary": True,
    }
    (ADDON / "build-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"native_build_id": build_id, "manifest": str(ADDON / "build-manifest.json")}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        sys.exit(f"native build failed: {exc}")
