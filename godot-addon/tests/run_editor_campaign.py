#!/usr/bin/env python3
"""Run fixed real-editor acceptance groups serially with resumable evidence."""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile

from run_script_open import SCENARIOS as OPEN_SCENARIOS
from run_script_edit import SCENARIOS as EDIT_SCENARIOS
from run_script_discovery import SCENARIOS as DISCOVERY_SCENARIOS
from run_script_close import SCENARIOS as CLOSE_SCENARIOS

REPO = Path(__file__).resolve().parents[2]
RUNNERS = {"open": "run_script_open.py", "edit": "run_script_edit.py",
           "observation": "run_observation.py", "discovery": "run_script_discovery.py",
           "close": "run_script_close.py"}
SCENARIOS = {"open": OPEN_SCENARIOS, "edit": EDIT_SCENARIOS, "observation": ("all",),
             "discovery": DISCOVERY_SCENARIOS, "close": CLOSE_SCENARIOS}
ATTEMPT = re.compile(r"attempt-([0-9]{3,})\Z")


class _CheckpointInterrupted(KeyboardInterrupt):
    pass


def _digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def _options(args, suite):
    names = ["godot", "observer"]
    if suite in ("open", "edit", "discovery", "close"):
        names += ["editor", "stock_validator", "native_fault_addon"]
    if suite in ("open", "discovery", "close"):
        names.append("opener")
    if suite in ("discovery", "close"):
        names.append("discoverer")
    if suite == "close":
        names.append("closer")
    return [(name, getattr(args, name)) for name in names]


def fingerprint(args, suite, scenario):
    """Hash execution inputs, never repository history or documentation.

    Trees are deliberately conservative within the runner's loaded sources. Tool
    probes cover the compiler behind rustup/xcrun shims, not just shim bytes.
    Environment values enter only this digest and are never recorded as evidence.
    """
    h = hashlib.sha256()

    def item(name, value):
        for part in (str(name).encode(), str(value).encode()):
            h.update(len(part).to_bytes(8, "big"))
            h.update(part)

    def file(path):
        path = Path(path)
        item("path", path.absolute())
        try:
            item("resolved", path.resolve())
            item("mode", stat.S_IMODE(path.stat().st_mode))
            item("bytes", _digest(path))
        except (OSError, ValueError):
            item("unavailable", True)

    def tree(path, suffixes=None):
        path = Path(path)
        item("tree", path.absolute())
        if not path.is_dir():
            item("missing_tree", True)
            return
        for child in sorted(path.rglob("*")):
            if ("__pycache__" in child.parts or child.name == ".DS_Store"
                    or child.suffix.lower() in (".md", ".markdown", ".rst")):
                continue
            if child.is_file() and (suffixes is None or child.suffix in suffixes):
                file(child)

    item("campaign_schema", 1)
    item("suite", suite)
    item("scenario", scenario)
    item("host", (platform.system(), platform.release(), platform.machine()))
    item("environment", json.dumps(dict(os.environ), sort_keys=True))
    file(Path(__file__))
    file(Path(sys.executable))
    tests = REPO / "godot-addon" / "tests"
    # All these helpers are Python imports or fixture GDScript loaded by runners.
    for child in sorted(tests.glob("*")):
        if (child.suffix in (".py", ".gd") and child.is_file()
                and not child.name.startswith("test_")):
            file(child)
    tree(tests / "fixtures" / "observation")
    if suite in ("open", "edit", "discovery", "close"):
        tree(tests / "fixtures" / ("script_" + suite))
        if suite in ("open", "discovery", "close"):
            tree(tests / "fixtures" / "script_edit")
        if suite in ("discovery", "close"):
            tree(tests / "fixtures" / "script_open")
        if suite == "close":
            tree(tests / "fixtures" / "script_discovery")
    tree(REPO / "godot-addon" / "addons" / "godot_agent_kit")
    tree(REPO / "godot-addon" / "native", {".py", ".cpp", ".hpp", ".h", ".json"})
    tree(REPO / "mcp-server" / "src", {".rs"})
    tree(REPO / "mcp-server" / "examples", {".rs"})
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "build.rs"):
        file(REPO / "mcp-server" / name)
    for name, path in _options(args, suite):
        item("option", name)
        if name == "native_fault_addon":
            tree(path)
        else:
            file(path)
    debug = args.observer.parent
    file(debug / "libgodot_agent_kit.rlib")
    item("rlib_directory", debug / "deps")
    for child in sorted((debug / "deps").glob("*.rlib")):
        file(child)
    file(Path.home() / "Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip")
    for command in (("rustc", "+1.98.1", "--version", "--verbose"),
                    ("swiftc", "--version"), ("xcrun", "--show-sdk-path"),
                    ("xcrun", "--find", "swiftc"),
                    ("rustup", "which", "--toolchain", "1.98.1", "rustc")):
        item("tool_command", command)
        executable = shutil.which(command[0])
        if executable:
            file(Path(executable))
        try:
            result = subprocess.run(command, shell=False, capture_output=True, timeout=30,
                                    env={**os.environ, "RUSTUP_AUTO_INSTALL": "0"})
            item("tool_exit", result.returncode)
            item("tool_output", hashlib.sha256(result.stdout + result.stderr).hexdigest())
            if command == ("xcrun", "--show-sdk-path") and result.returncode == 0:
                sdk = Path(os.fsdecode(result.stdout).strip())
                file(sdk / "SDKSettings.json")
                file(sdk / "SDKSettings.plist")
            if command[0] == "rustup" or command[:2] == ("xcrun", "--find"):
                if result.returncode == 0:
                    file(Path(os.fsdecode(result.stdout).strip()))
        except (OSError, subprocess.SubprocessError):
            item("tool_unavailable", True)
    return h.hexdigest()


def _command(args, suite, scenario, attempt):
    command = [sys.executable, str(REPO / "godot-addon" / "tests" / RUNNERS[suite])]
    for name, value in _options(args, suite):
        command.extend(["--" + name.replace("_", "-"), str(value)])
    return command + ["--scenario", scenario, "--artifacts", str(attempt)]


def _identity(command):
    return hashlib.sha256(json.dumps(command, separators=(",", ":")).encode()).hexdigest()


def _checkpoint(root, manifest):
    fd, temporary = tempfile.mkstemp(prefix=".manifest-", dir=root)
    try:
        with os.fdopen(fd, "w") as stream:
            json.dump(manifest, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, root / "manifest.json")
    except KeyboardInterrupt as error:
        raise _CheckpointInterrupted from error
    finally:
        Path(temporary).unlink(missing_ok=True)


def _private(path):
    metadata = path.stat()
    return (not path.is_symlink() and path.is_dir() and metadata.st_uid == os.geteuid()
            and stat.S_IMODE(metadata.st_mode) == 0o700)


def _evidence(root, step, args):
    """Validate only expected owned paths; manifest content never becomes argv."""
    try:
        name = step["attempt_dir"]
        if not isinstance(name, str):
            return False
        parts = Path(name).parts
        if (len(parts) != 3 or parts[:2] != (step["suite"], step["scenario"])
                or not ATTEMPT.fullmatch(parts[2])):
            return False
        attempt = root / name
        summary = attempt / "summary.json"
        if (not all(_private(path) for path in (attempt.parent.parent, attempt.parent, attempt))
                or summary.is_symlink() or not summary.is_file()
                or step["summary_path"] != name + "/summary.json"
                or step["command_identity"] != _identity(_command(
                    args, step["suite"], step["scenario"], attempt))
                or step["summary_sha256"] != _digest(summary)):
            return False
        return json.loads(summary.read_text()).get("status") == "passed"
    except (OSError, ValueError, TypeError, KeyError, AttributeError):
        return False


def _fresh_attempt(root, suite, scenario):
    for component in (suite, scenario):
        root = root / component
        root.mkdir(mode=0o700, exist_ok=True)
        if not _private(root):
            raise ValueError("campaign step directory must be private")
    numbers = [int(match.group(1)) for path in root.iterdir()
               if (match := ATTEMPT.fullmatch(path.name))]
    number = max(numbers, default=0) + 1
    while True:
        attempt = root / f"attempt-{number:03d}"
        try:
            attempt.mkdir(mode=0o700)
            return attempt
        except FileExistsError:
            number += 1


def _load(root):
    path = root / "manifest.json"
    if path.is_symlink():
        return []
    try:
        payload = json.loads(path.read_text())
        if (isinstance(payload, dict) and payload.get("schema_version") == 1
                and isinstance(payload.get("steps"), list)):
            return payload["steps"]
    except (OSError, ValueError, TypeError):
        pass
    return []


def _execute(command):
    # Isolate terminal signals so the wrapper can wait for the runner's own
    # cleanup before returning or allowing another campaign to take the lock.
    with subprocess.Popen(command, shell=False, start_new_session=True) as child:
        try:
            return child.wait(), False
        except KeyboardInterrupt:
            if child.poll() is None:
                child.send_signal(signal.SIGINT)
            return child.wait(), True


def _run(args, root):
    suites = RUNNERS if args.suite == "all" else (args.suite,)
    selected = [(suite, scenario) for suite in suites for scenario in SCENARIOS[suite]]
    previous = _load(root) if args.resume else []
    steps = [{"suite": suite, "scenario": scenario, "status": "not_run", "exit_code": None,
              "fingerprint": None, "attempt_dir": None, "summary_path": None,
              "summary_sha256": None, "command_identity": None, "execution": "not_run"}
             for suite, scenario in selected]
    manifest = {"schema_version": 1, "steps": steps}
    stopped = False
    checkpoint_interrupted = False
    try:
        # Validate all previous passes before executing: fail-fast must not erase
        # a later checkpoint merely because an earlier failed attempt fails again.
        for step in steps:
            suite, scenario = step["suite"], step["scenario"]
            old = [entry for entry in previous if isinstance(entry, dict)
                   and (entry.get("suite"), entry.get("scenario")) == (suite, scenario)]
            if (len(old) == 1 and old[0].get("status") == "passed"
                    and type(old[0].get("exit_code")) is int and old[0]["exit_code"] == 0
                    and old[0].get("fingerprint") == fingerprint(args, suite, scenario)
                    and _evidence(root, old[0], args)):
                step.update({key: old[0].get(key) for key in step if key not in ("suite", "scenario")})
                step["execution"] = "reused"
        _checkpoint(root, manifest)
        for step in steps:
            if step["execution"] == "reused":
                continue
            suite, scenario = step["suite"], step["scenario"]
            current = fingerprint(args, suite, scenario)
            attempt = _fresh_attempt(root, suite, scenario)
            relative = str(attempt.relative_to(root))
            command = _command(args, suite, scenario, attempt)
            step.update(status="running", fingerprint=current, attempt_dir=relative,
                        summary_path=relative + "/summary.json",
                        command_identity=_identity(command), execution="executed")
            _checkpoint(root, manifest)
            try:
                # Child output stays with the child/terminal, never in the manifest.
                exit_code, interrupted = _execute(command)
                step["exit_code"] = exit_code
                summary = attempt / "summary.json"
                if summary.is_file() and not summary.is_symlink():
                    step["summary_sha256"] = _digest(summary)
                step["status"] = ("passed" if exit_code == 0 and not interrupted
                                  and fingerprint(args, suite, scenario) == current
                                  and _evidence(root, step, args) else "failed")
                if interrupted or exit_code in (-signal.SIGINT, -signal.SIGTERM):
                    step["status"] = "interrupted"
            except OSError:
                step["status"] = "failed"
            _checkpoint(root, manifest)
            if (step["status"] == "interrupted"
                    or (step["status"] != "passed" and not args.keep_going)):
                break
    except _CheckpointInterrupted:
        stopped = True
        checkpoint_interrupted = True
    except KeyboardInterrupt:
        stopped = True
        for step in steps:
            if step["status"] == "running":
                step["status"] = "interrupted"
    # An earlier pass is not current evidence after an input changes mid-campaign.
    try:
        for step in steps:
            if step["status"] == "passed" and (
                    fingerprint(args, step["suite"], step["scenario"]) != step["fingerprint"]
                    or not _evidence(root, step, args)):
                step["status"] = "failed"
        if not checkpoint_interrupted:
            _checkpoint(root, manifest)
    except KeyboardInterrupt:
        stopped = True
    for step in steps:
        label = ("REUSED" if step["status"] == "passed" and step["execution"] == "reused"
                 else "PASS" if step["status"] == "passed" else "FAIL")
        print(f"{label} {step['suite']}/{step['scenario']} ({step['status']})")
    return 0 if not stopped and all(step["status"] == "passed" for step in steps) else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", required=True, choices=(*RUNNERS, "all"))
    parser.add_argument("--campaign-dir", required=True, type=Path)
    for name in ("godot", "observer", "editor", "opener", "discoverer", "closer",
                 "stock-validator", "native-fault-addon"):
        parser.add_argument("--" + name, type=Path, required=name in ("godot", "observer"))
    parser.add_argument("--keep-going", action="store_true")
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args(argv)
    suites = RUNNERS if args.suite == "all" else (args.suite,)
    for suite in suites:
        for name, path in _options(args, suite):
            if path is None or not path.is_absolute():
                parser.error("--" + name.replace("_", "-") + " requires an absolute path")
            if name == "native_fault_addon":
                valid = path.is_dir() and not path.is_symlink()
            else:
                valid = path.is_file() and os.access(path, os.X_OK)
            if not valid:
                parser.error("required execution input is unavailable: --" + name.replace("_", "-"))
        if suite != "observation":
            for native in (args.native_fault_addon,
                           REPO / "godot-addon/addons/godot_agent_kit/native"):
                if not all((native / name).is_file() for name in (
                        "editor_integration.gdextension",
                        "libeditor_integration.macos.arm64.dylib", "build-manifest.json")):
                    parser.error("required native execution artifacts are unavailable")
    root = args.campaign_dir
    if not root.is_absolute():
        parser.error("--campaign-dir requires an absolute path")
    try:
        root.mkdir(mode=0o700, exist_ok=True)
        if not _private(root):
            parser.error("campaign directory must be owned, nonsymlink, and mode 0700")
        if not args.resume and any(root.iterdir()):
            parser.error("nonempty campaign directory requires --resume")
        # Prevent simultaneous invocations from sharing a checkpoint or GUI campaign.
        with (root / ".campaign-lock").open("a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            old_handler = signal.getsignal(signal.SIGTERM)
            def interrupted(signum, frame):
                raise KeyboardInterrupt
            signal.signal(signal.SIGTERM, interrupted)
            try:
                return _run(args, root)
            finally:
                signal.signal(signal.SIGTERM, old_handler)
    except (OSError, ValueError):
        print("FAIL campaign (checkpoint or execution input unavailable)")
        return 1
    except KeyboardInterrupt:
        print("FAIL campaign (interrupted)")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
