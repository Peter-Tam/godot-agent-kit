#!/usr/bin/env python3
"""Build the native editor integration against the pinned official Godot binary."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import stat

BASE = "ed1daf0bf001b61586d9930840f2f1394092c079"
STOCK_SHA256 = "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf"
HERE = Path(__file__).resolve().parent
ADDON = HERE.parent / "addons" / "godot_agent_kit" / "native"
BUILD = HERE / "build"
SOURCES = ("extension.cpp", "session.cpp", "document_guard.cpp", "script_document.cpp",
           "editor_context.cpp", "open_context.cpp", "script_open.cpp", "script_close.cpp", "script_closed_edit.cpp")
HEADERS = ("native.hpp", "document_guard.hpp", "editor_context.hpp", "open_context.hpp")
DESCRIPTOR = """[configuration]
entry_symbol = "editor_integration_library_init"
compatibility_minimum = "4.7"
reloadable = false

[libraries]
macos.arm64 = "res://addons/godot_agent_kit/native/libeditor_integration.macos.arm64.dylib"
"""
OLD_DESCRIPTOR = DESCRIPTOR.replace("editor_integration", "script_edit")


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


def close_method(api):
    entry = next((c for c in api["classes"] if c["name"] == "ScriptEditor"), None)
    if entry is None:
        raise ValueError("Required class missing: ScriptEditor")
    candidates = [m for m in entry.get("methods", []) if m["name"] == "close_file"]
    if len(candidates) != 1:
        raise ValueError("Required method missing or ambiguous: ScriptEditor.close_file")
    value = candidates[0]
    arguments = value.get("arguments", [])
    if (value.get("is_static", False) or value.get("is_vararg", False)
            or value.get("is_virtual", False) or value.get("is_const", False)
            or len(arguments) != 1 or arguments[0].get("type") != "String"
            or "default_value" in arguments[0]
            or value.get("return_value", {}).get("type") != "enum::Error"
            or type(value.get("hash")) is not int or not 0 < value["hash"] <= 0xFFFFFFFF):
        raise ValueError("Unsupported ScriptEditor.close_file signature")
    return value["hash"]


def enum_constant(api, name):
    values = [value["value"] for enum in api["global_enums"]
              for value in enum["values"] if value["name"] == name]
    if len(values) != 1:
        raise ValueError(f"Required enum constant missing or ambiguous: {name}")
    return values[0]


def remove_obsolete_artifacts(addon):
    """Remove only old kit artifacts whose exact installed provenance is known."""
    descriptor = addon / "script_edit.gdextension"
    library = addon / "libscript_edit.macos.arm64.dylib"
    pending = []
    if descriptor.exists() or descriptor.is_symlink():
        info = descriptor.lstat()
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or descriptor.read_text() != OLD_DESCRIPTOR):
            raise ValueError("obsolete descriptor is not the kit-owned installed artifact")
        pending.append(descriptor)
    if library.exists() or library.is_symlink():
        info = library.lstat()
        manifest_path = addon / "build-manifest.json"
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or not manifest_path.is_file() or manifest_path.is_symlink()):
            raise ValueError("obsolete library has no safe kit-owned installation witness")
        manifest = json.loads(manifest_path.read_text())
        if (manifest.get("base_commit") != BASE or manifest.get("engine_sha256") != STOCK_SHA256
                or manifest.get("native_library", library.name) != library.name
                or manifest.get("native_library_sha256") != digest(library)):
            raise ValueError("obsolete library does not match the kit-owned installed manifest")
        pending.append(library)
    # Validate all candidates before removing either one; preserve unrelated files.
    for path in pending:
        path.unlink()



def single_precision_sizes(api):
    if api["header"].get("precision") != "single":
        raise ValueError("the native integration requires the engine's single-precision ABI")
    raw_sizes = next((x["sizes"] for x in api["builtin_class_sizes"] if x["build_configuration"] == "float_64"), None)
    if raw_sizes is None:
        raise ValueError("missing float_64 Godot builtin sizes")
    return {entry["name"]: entry["size"] for entry in raw_sizes}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", required=True, type=Path, help="absolute pinned official editor executable")
    parser.add_argument("--fixture-faults", action="store_true", dest="fixture",
                        help="enable fault hooks only in a separate disposable fixture build")
    parser.add_argument("--output-addon", type=Path, help="output native directory for fixture-only builds")
    args = parser.parse_args(argv)
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
    if engine_sha != STOCK_SHA256:
        parser.error("unrecognized binary: expected the pinned official stock build")
    version = run([str(engine), "--version"]).stdout.strip()
    if version != "4.7.2.stable.official." + BASE[:9]:
        parser.error("engine binary does not report pinned 4.7.2.stable.official build")
    BUILD.mkdir(parents=True, exist_ok=True)
    run([str(engine), "--headless", "--dump-gdextension-interface", "--dump-gdextension-interface-json", "--dump-extension-api"], cwd=BUILD)
    header, abi_json, api_json = (BUILD / n for n in ("gdextension_interface.h", "gdextension_interface.json", "extension_api.json"))
    api = json.loads(api_json.read_text())
    sizes = single_precision_sizes(api)
    if (api["header"]["version_major"] != 4 or api["header"]["version_minor"] != 7
            or api["header"]["version_patch"] != 2 or api["header"]["version_status"] != "stable"
            or api["header"]["version_build"] != "official"):
        parser.error("generated API version/build does not match the pinned official binary")
    required = {"Variant", "String", "StringName", "Dictionary", "Array", "Callable"}
    if not required <= sizes.keys():
        parser.error("generated ABI is missing native value sizes")
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
        "RESOURCE_SET_PATH": method(api, "Resource", "set_path"),
        "SCRIPT_RELOAD": method(api, "Script", "reload"),
        "SCRIPT_TOOL": method(api, "Script", "is_tool"),
        "SCRIPT_BASE": method(api, "Script", "get_base_script"),
        "SCRIPT_PROPERTIES": method(api, "Script", "get_script_property_list"),
        "SCRIPT_METHODS": method(api, "Script", "get_script_method_list"),
        "CURRENT_EDITOR": method(api, "ScriptEditor", "get_current_editor"),
        "CURRENT_SCRIPT": method(api, "ScriptEditor", "get_current_script"),
        "HAS_UNDO": method(api, "TextEdit", "has_undo"),
        "HAS_REDO": method(api, "TextEdit", "has_redo"),
        "HAS_CACHED": method(api, "ResourceLoader", "has_cached"),
        "CACHED_REF": method(api, "ResourceLoader", "get_cached_ref"),
        "EDIT_SCRIPT": method(api, "EditorInterface", "edit_script"),
        "INSTANTIATE": method(api, "ClassDB", "instantiate"),
        "CLASS_EXISTS": method(api, "ClassDB", "class_exists"),
        "CLASS_API": method(api, "ClassDB", "class_get_api_type"),
        "PROJECT_OVERRIDE": method(api, "ProjectSettings", "get_setting_with_override"),
        "GLOBALIZE_PATH": method(api, "ProjectSettings", "globalize_path"),
        "GLOBAL_CLASSES": method(api, "ProjectSettings", "get_global_class_list"),
        "OBJECT_PROPERTIES": method(api, "Object", "get_property_list"),
        "CLOSE_FILE": close_method(api),
        "HAS_SIGNAL": method(api, "Object", "has_signal"),
        "CONNECT": method(api, "Object", "connect"),
        "DISCONNECT": method(api, "Object", "disconnect"),
        "IS_CONNECTED": method(api, "Object", "is_connected"),
        "GET_META": method(api, "Object", "get_meta"),
        "OBJECT_SET": method(api, "Object", "set"),
    }
    inputs = {p.name: digest(p) for p in (header, abi_json, api_json,
              *(HERE / name for name in (*SOURCES, *HEADERS, "build.py")))}
    compiler = os.environ.get("CXX", "clang++")
    compiler_version = run([compiler, "--version"]).stdout.splitlines()[0]
    sdk_version = run(["xcrun", "--show-sdk-version"]).stdout.strip()
    flags = ["-std=c++17", "-O2", "-fPIC", "-fvisibility=hidden", "-dynamiclib", "-Wall", "-Wextra", "-Werror", "-I", str(BUILD), "-I", str(HERE)]
    flags += [f"-DGAK_FIXTURE={int(args.fixture)}"]
    build_id = hashlib.sha256(json.dumps({
        "engine_sha256": engine_sha, "inputs": inputs, "configuration": "float_64/macos-arm64",
        "fixture": args.fixture,
        "compiler": compiler_version, "sdk": sdk_version, "flags": flags,
    }, sort_keys=True).encode()).hexdigest()
    macros = ["// Generated by build.py from the exact --godot binary; never hand-edit.", "#pragma once"]
    macros += [f"#define GAK_SIZE_{k.upper()} {sizes[k]}" for k in sorted(required)]
    macros += [f"#define GAK_HASH_{k} {v}ULL" for k, v in methods.items()]
    macros += [f"#define GAK_ERR_PARSE_ERROR {enum_constant(api, 'ERR_PARSE_ERROR')}"]
    macros += [f'#define GAK_ENGINE_HASH "{BASE}"',
               f'#define GAK_ENGINE_BUILD "{api["header"]["version_build"]}"',
               f'#define GAK_ENGINE_BINARY_SHA256 "{engine_sha}"',
               f'#define GAK_BUILD_ID "{build_id}"']
    (BUILD / "native_abi_sizes.h").write_text("\n".join(macros) + "\n")
    output = addon / "libeditor_integration.macos.arm64.dylib"
    addon.mkdir(parents=True, exist_ok=True)
    run([compiler, *flags, *(str(HERE / name) for name in SOURCES), "-o", str(output)])
    installed_descriptor = addon / "editor_integration.gdextension"
    if installed_descriptor.exists() or installed_descriptor.is_symlink():
        info = installed_descriptor.lstat()
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or installed_descriptor.read_text() != DESCRIPTOR):
            raise ValueError("current descriptor is not the expected kit-owned artifact")
    remove_obsolete_artifacts(addon)
    installed_descriptor.write_text(DESCRIPTOR)
    manifest = {
        "base_commit": BASE, "engine_version": version, "engine_binary": str(engine), "engine_sha256": engine_sha,
        "fixture_only": args.fixture,
        "native_api_revision": 4, "native_family": "editor_integration",
        "native_library": output.name, "entry_symbol": "editor_integration_library_init",
        "abi_sha256": digest(abi_json), "api_sha256": digest(api_json), "header_sha256": digest(header),
        "native_build_id": build_id, "native_library_sha256": digest(output),
        "source_sha256": {key: value for key, value in inputs.items() if key.endswith((".cpp", ".hpp", ".py"))},
        "compiler": compiler_version,
        "sdk": sdk_version, "build_flags": flags,
        "platform": "macos-arm64", "generated_from_exact_binary": True,
    }
    (addon / "build-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"native_build_id": build_id, "manifest": str(addon / "build-manifest.json")}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        sys.exit(f"native build failed: {exc}")
