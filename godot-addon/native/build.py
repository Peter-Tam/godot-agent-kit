#!/usr/bin/env python3
"""Build the stock editor finalizer or the separately retained patched validation oracle."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

BASE = "ed1daf0bf001b61586d9930840f2f1394092c079"
STOCK_SHA256 = "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf"
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
    parser.add_argument("--godot", required=True, type=Path, help="absolute matching stock or patched-oracle editor executable")
    parser.add_argument("--fixture-faults", action="store_true", dest="fixture",
                        help="enable fault hooks only in a separate disposable fixture build")
    parser.add_argument("--output-addon", type=Path, help="output native directory for fixture-only builds")
    args = parser.parse_args()
    addon = args.output_addon.resolve() if args.output_addon else ADDON
    if args.fixture and (args.output_addon is None or addon == ADDON):
        parser.error("--fixture-faults requires --output-addon outside the product addon")
    if args.output_addon and not args.fixture:
        parser.error("--output-addon is reserved for isolated fixture builds")
    if not args.godot.is_absolute() or not args.godot.is_file():
        parser.error("--godot must be an absolute existing engine executable")
    engine = args.godot.resolve(strict=True)
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        parser.error("the current native reader supports only macOS arm64")
    engine_sha = digest(engine)
    stock = engine_sha == STOCK_SHA256
    source = next((p for p in engine.parents if (p / "main/main.cpp").exists() and (p / ".git").exists()), None)
    if not stock:
        if source is None or run(["git", "-C", str(source), "rev-parse", "HEAD"]).stdout.strip() != BASE:
            parser.error("unrecognized binary: neither the pinned official stock build nor a pinned patched checkout")
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
    if stock and (api["header"]["version_build"] != "official" or not version.startswith("4.7.2.stable.official.")):
        parser.error("official binary version/build does not match the pinned stock artifact")
    methods = {
        "SCRIPT_EDITOR": method(api, "EditorInterface", "get_script_editor"),
        "OPEN_SCRIPTS": method(api, "ScriptEditor", "get_open_scripts"),
        "OPEN_EDITORS": method(api, "ScriptEditor", "get_open_script_editors"),
        "BASE_EDITOR": method(api, "ScriptEditorBase", "get_base_editor"),
        "RESOURCE_PATH": method(api, "Resource", "get_path"),
        "SCRIPT_SOURCE": method(api, "Script", "get_source_code"),
        "SCRIPT_SET_SOURCE": method(api, "Script", "set_source_code"),
        "TEXT": method(api, "TextEdit", "get_text"),
        "LINE_COUNT": method(api, "TextEdit", "get_line_count"),
        "LINE": method(api, "TextEdit", "get_line"),
        "CURRENT_VERSION": method(api, "TextEdit", "get_version"),
        "SAVED_VERSION": method(api, "TextEdit", "get_saved_version"),
        "COMPLEX_BEGIN": method(api, "TextEdit", "begin_complex_operation"),
        "COMPLEX_END": method(api, "TextEdit", "end_complex_operation"),
        "REMOVE_TEXT": method(api, "TextEdit", "remove_text"),
        "INSERT_TEXT": method(api, "TextEdit", "insert_text"),
        "TAG_SAVED": method(api, "TextEdit", "tag_saved_version"),
        "UNSAVED": method(api, "ScriptEditor", "get_unsaved_files"),
        "SET_EDITED": method(api, "EditorInterface", "set_object_edited"),
        "IS_EDITED": method(api, "EditorInterface", "is_object_edited"),
        "EDITOR_SETTINGS": method(api, "EditorInterface", "get_editor_settings"),
        "GET_SETTING": method(api, "EditorSettings", "get_setting"),
        "INDENT_SPACES": method(api, "CodeEdit", "is_indent_using_spaces"),
        "INDENT_SIZE": method(api, "CodeEdit", "get_indent_size"),
    }
    if not stock:
        methods.update({
            "GDSCRIPT_REVISION": method(api, "GDScript", "gdscript_validation_api_revision"),
            "GDSCRIPT_VALIDATE": method(api, "GDScript", "validate_gdscript_source"),
            "SCRIPT_EDITOR_EDITED": method(api, "ScriptEditorBase", "get_edited_resource"),
        })
    inputs = {p.name: digest(p) for p in (header, abi_json, api_json, HERE / "extension.cpp", HERE / "validation.cpp", HERE / "script_document.cpp", HERE / "native.hpp", HERE / "build.py")}
    if not stock:
        inputs["engine-api.patch"] = digest(HERE / "engine-api.patch")
    compiler = os.environ.get("CXX", "clang++")
    compiler_version = run([compiler, "--version"]).stdout.splitlines()[0]
    sdk_version = run(["xcrun", "--show-sdk-version"]).stdout.strip()
    flags = ["-std=c++17", "-O2", "-fPIC", "-fvisibility=hidden", "-dynamiclib", "-Wall", "-Wextra", "-Werror", "-I", str(BUILD), "-I", str(HERE)]
    flags += [f"-DGAK_STOCK={int(stock)}", f"-DGAK_FIXTURE={int(args.fixture)}"]
    build_id = hashlib.sha256(json.dumps({
        "engine_sha256": engine_sha, "inputs": inputs, "configuration": "float_64/macos-arm64",
        "mode": "stock" if stock else "oracle", "fixture": args.fixture,
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
    output = addon / "libscript_edit.macos.arm64.dylib"
    addon.mkdir(parents=True, exist_ok=True)
    run([compiler, *flags, str(HERE / "extension.cpp"), str(HERE / "validation.cpp"),
         str(HERE / "script_document.cpp"), "-o", str(output)])
    manifest = {
        "base_commit": BASE, "engine_version": version, "engine_binary": str(engine), "engine_sha256": engine_sha,
        "mode": "stock" if stock else "oracle", "fixture_only": args.fixture,
        "abi_sha256": digest(abi_json), "api_sha256": digest(api_json), "header_sha256": digest(header),
        "native_build_id": build_id, "native_library_sha256": digest(output),
        "source_sha256": {key: value for key, value in inputs.items() if key.endswith((".cpp", ".hpp", ".py"))},
        "compiler": compiler_version,
        "sdk": sdk_version, "build_flags": flags,
        "platform": "macos-arm64", "generated_from_exact_binary": True,
    }
    if not stock:
        manifest["patch_sha256"] = inputs["engine-api.patch"]
    (addon / "build-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"native_build_id": build_id, "manifest": str(addon / "build-manifest.json")}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        sys.exit(f"native build failed: {exc}")
