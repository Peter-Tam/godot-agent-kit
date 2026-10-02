"""Run the trusted GUI job's shell stages against isolated candidate metadata.

Command stand-ins simulate a provisioned runner, not Godot acceptance. The tests
prove rejection/ordering; they do not certify the real binaries or GUI platform.
"""

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import yaml


WORKFLOWS = Path(__file__).resolve().parents[1] / "workflows"
REVISION = "a" * 40
CANDIDATE = {
    "head": REVISION,
    "macos": "26.6.2",
    "arch": "arm64",
    "godot": "4.7.2.stable.official.ed1daf0bf",
    "godot_hash": "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf",
    "template_hash": "88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792",
    "rust": "rustc 1.98.1 (48a229cea 2026-09-01)",
    "host": "aarch64-apple-darwin",
    "cargo": "1.98.1",
    "python": [3, 10, 0],
}
TEMPLATE = "Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip"

# Each stand-in accepts only the fixed validation/build commands. Actual shell
# failures must prevent later stages, including the simulated GUI invocation.
COMMAND = r'''
import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
args = sys.argv[1:]
metadata = json.loads(Path(os.environ["CANDIDATE_METADATA"]).read_text())
with open(os.environ["COMMAND_TRACE"], "a") as trace:
    trace.write(json.dumps({"command": name, "args": args, "cwd": os.getcwd(),
                            "auto_install": os.environ.get("RUSTUP_AUTO_INSTALL")}) + "\n")
if name == "git" and args == ["rev-parse", "HEAD"]:
    print(metadata["head"])
elif name == "sw_vers" and args == ["-productVersion"]:
    print(metadata["macos"])
elif name == "uname" and args == ["-m"]:
    print(metadata["arch"])
elif name == "godot" and args == ["--version"]:
    print(metadata["godot"])
elif name == "shasum" and args[:2] == ["-a", "256"] and len(args) == 3:
    target = Path(args[2])
    if not target.is_file():
        sys.exit(1)
    if target == Path(os.environ["HOME"]) / metadata["template_path"]:
        digest = metadata["template_hash"]
    elif target == Path(sys.argv[0]).with_name("godot"):
        digest = metadata["godot_hash"]
    else:
        raise RuntimeError("unexpected checksum target")
    print(digest + "  " + str(target))
elif name == "rustc" and args == ["--version"]:
    print(metadata["rust"])
elif name == "rustc" and args == ["--print", "host-tuple"]:
    print(metadata["host"])
elif name == "cargo" and args == ["--version"]:
    print("cargo " + metadata["cargo"] + " (fixture)")
elif name == "cargo" and args[0] in ("fmt", "clippy", "test", "doc", "build"):
    if args[0] == metadata.get("fail_baseline"):
        sys.exit(1)
    if args[0] == "build":
        if args != ["build", "--locked", "--lib", "--bin", "observe-gdscript",
                    "--bin", "edit-gdscript", "--example", "stock_validation_fixture"]:
            sys.exit("missing complete caller/observer/validator build")
        targets = Path(os.environ["GITHUB_WORKSPACE"]) / "mcp-server/target/debug"
        for binary in ("observe-gdscript", "edit-gdscript",
                       "examples/stock_validation_fixture"):
            path = targets / binary
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic binary")
            path.chmod(0o700)
elif name == "python3" and len(args) == 2 and args[0] == "-c":
    sys.version_info = tuple(metadata["python"])
    exec(args[1])
elif name == "python3" and args[0] == "godot-addon/native/build.py":
    product = (Path(os.environ["GITHUB_WORKSPACE"]) /
               "godot-addon/addons/godot_agent_kit/native")
    fixture = Path(os.environ["RUNNER_TEMP"]) / "script-edit-fault-native"
    base = ["godot-addon/native/build.py", "--godot",
            str(Path(sys.argv[0]).with_name("godot"))]
    is_fixture = args == [*base, "--fixture-faults", "--output-addon", str(fixture)]
    if not is_fixture and args != base:
        sys.exit("wrong native build contract")
    if metadata.get("fail_native_build") == ("fixture" if is_fixture else "production"):
        sys.exit(1)
    if is_fixture and not (product / "build-manifest.json").is_file():
        sys.exit("fixture build preceded product")
    target = fixture if is_fixture else product
    target.mkdir(parents=True, exist_ok=True)
    (target / "libeditor_integration.macos.arm64.dylib").write_text("fixture" if is_fixture else "product")
    (target / "build-manifest.json").write_text(json.dumps({
        "fixture_only": is_fixture, "native_api_revision": 3, "native_family": "editor_integration",
        "native_library": "libeditor_integration.macos.arm64.dylib",
        "entry_symbol": "editor_integration_library_init",
        "native_build_id": "f" * 64 if is_fixture else "e" * 64}))
elif name == "python3" and args[0] == "godot-addon/tests/run_script_edit.py":
    workspace = Path(os.environ["GITHUB_WORKSPACE"])
    temp = Path(os.environ["RUNNER_TEMP"])
    expected = ["godot-addon/tests/run_script_edit.py",
                "--godot", str(Path(sys.argv[0]).with_name("godot")),
                "--observer", str(workspace / "mcp-server/target/debug/observe-gdscript"),
                "--editor", str(workspace / "mcp-server/target/debug/edit-gdscript"),
                "--stock-validator", str(workspace / "mcp-server/target/debug/examples/stock_validation_fixture"),
                "--native-fault-addon", str(temp / "script-edit-fault-native"),
                "--scenario", "all", "--artifacts", str(temp / "script-edit-evidence/results")]
    if args != expected or not all(Path(path).is_file() for path in (
            expected[4], expected[6], expected[8],
            temp / "script-edit-evidence/production-build-manifest.json",
            temp / "script-edit-evidence/fixture-build-manifest.json")):
        sys.exit("incomplete script-edit acceptance inputs")
    prod = json.loads((temp / "script-edit-evidence/production-build-manifest.json").read_text())
    fault = json.loads((temp / "script-edit-evidence/fixture-build-manifest.json").read_text())
    if prod["fixture_only"] or not fault["fixture_only"] or prod["native_build_id"] == fault["native_build_id"]:
        sys.exit("mixed normal and fixture native artifacts")
    artifacts = Path(expected[-1])
    if not artifacts.is_dir() or any(artifacts.iterdir()) or artifacts.stat().st_mode & 0o777 != 0o700:
        sys.exit("script-edit evidence directory must start private and empty")
    failed = metadata.get("fail_gui") == "script-edit"
    (artifacts / "summary.json").write_text(json.dumps({"status": "failed" if failed else "passed"}))
    if failed:
        sys.exit(1)
elif name == "python3" and args[0] == "godot-addon/tests/run_observation.py":
    temp = Path(os.environ["RUNNER_TEMP"])
    artifacts = temp / "observation-evidence"
    expected = ["godot-addon/tests/run_observation.py",
                "--godot", str(Path(sys.argv[0]).with_name("godot")),
                "--observer", str(Path(os.environ["GITHUB_WORKSPACE"]) /
                                  "mcp-server/target/debug/observe-gdscript"),
                "--scenario", "all", "--artifacts", str(artifacts)]
    if args != expected or not (temp / "script-edit-evidence/results/summary.json").is_file():
        sys.exit("observation ran without completed script-edit suite")
    if not artifacts.is_dir() or any(artifacts.iterdir()) or artifacts.stat().st_mode & 0o777 != 0o700:
        sys.exit("observation evidence directory must start private and empty")
    (artifacts / "summary.json").write_text(json.dumps({"status": "passed"}))
    if metadata.get("fail_gui") == "observation":
        sys.exit(1)
else:
    raise RuntimeError("unexpected command: " + repr([name, *args]))
'''


class ExecutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.live = yaml.load((WORKFLOWS / "live-editor.yml").read_text(), Loader=yaml.BaseLoader)
        cls.ci = yaml.load((WORKFLOWS / "ci.yml").read_text(), Loader=yaml.BaseLoader)
        cls.gui = cls.live["jobs"]["live-editor"]

    def execute(self, changes=None, missing=()):
        with tempfile.TemporaryDirectory(prefix="gui-candidate-") as directory:
            root = Path(directory)
            workspace, home, tools, runner_temp = (root / name for name in ("workspace", "home", "bin", "temp"))
            for path in (workspace / "mcp-server", home, tools, runner_temp):
                path.mkdir(parents=True)
            template = home / TEMPLATE
            template.parent.mkdir(parents=True)
            if "template" not in missing:
                template.write_bytes(b"synthetic template; checksum metadata is simulated")
            metadata = root / "candidate.json"
            metadata.write_text(json.dumps({**CANDIDATE, "template_path": TEMPLATE, **(changes or {})}))
            trace = root / "trace.jsonl"
            trace.touch()
            for name in ("git", "sw_vers", "uname", "godot", "shasum", "rustc", "cargo", "python3"):
                if name not in missing:
                    command = tools / name
                    command.write_text("#!" + sys.executable + "\n" + COMMAND)
                    command.chmod(0o700)
            for name in ("cut", "mkdir", "cp"):
                (tools / name).symlink_to(shutil.which(name))
            environment = {
                "PATH": str(tools), "HOME": str(home), "GITHUB_WORKSPACE": str(workspace),
                "RUNNER_TEMP": str(runner_temp), "CANDIDATE_METADATA": str(metadata),
                "COMMAND_TRACE": str(trace), **self.gui.get("env", {}),
            }
            for step in self.gui["steps"]:
                if "run" not in step:
                    continue
                step_env = {
                    key: value.replace("${{ needs.trust-gate.outputs.revision }}", REVISION)
                    for key, value in step.get("env", {}).items()
                }
                result = subprocess.run(
                    ["/bin/bash", "-e", "-c", step["run"]],
                    cwd=workspace / step.get("working-directory", "."),
                    env={**environment, **step_env}, capture_output=True, text=True, timeout=20,
                )
                if result.returncode:
                    break
            calls = [json.loads(line) for line in trace.read_text().splitlines()]
            return result, calls

    @staticmethod
    def native_calls(calls):
        return [call for call in calls if call["command"] == "cargo" and call["args"] != ["--version"]]

    @staticmethod
    def native_build_calls(calls):
        return [call for call in calls if call["command"] == "python3" and
                call["args"][0] == "godot-addon/native/build.py"]

    @staticmethod
    def gui_calls(calls):
        return [call for call in calls if call["command"] == "python3" and
                call["args"][0] in ("godot-addon/tests/run_script_edit.py",
                                   "godot-addon/tests/run_observation.py")]

    def test_exact_candidate_reaches_complete_acceptance(self):
        for python in ([3, 10, 0], [3, 14, 0]):
            with self.subTest(python=python):
                result, calls = self.execute({"python": python})
                self.assertEqual(result.returncode, 0, result.stderr)
                gui = self.gui_calls(calls)
                self.assertEqual([call["args"][0] for call in gui], [
                    "godot-addon/tests/run_script_edit.py",
                    "godot-addon/tests/run_observation.py"])
                self.assertEqual(len(self.native_build_calls(calls)), 2)
                self.assertTrue(all(call["args"][call["args"].index("--scenario") + 1] == "all"
                                    for call in gui))
                for call in self.native_calls(calls):
                    self.assertEqual(Path(call["cwd"]).name, "mcp-server")
                    self.assertEqual(call["auto_install"], "0")

    def test_mismatched_candidate_never_reaches_native_or_gui(self):
        mismatches = {
            "macos": "26.6.1", "arch": "x86_64", "godot": "4.7.1.stable.official.other",
            "godot_hash": "0" * 64, "template_hash": "0" * 64,
            "rust": "rustc 1.98.0 (other)", "host": "x86_64-apple-darwin",
            "cargo": "1.98.0", "python": [3, 9, 99],
        }
        for key, value in mismatches.items():
            with self.subTest(field=key):
                result, calls = self.execute({key: value})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.native_calls(calls), [])
                self.assertEqual(self.native_build_calls(calls), [])
                self.assertEqual(self.gui_calls(calls), [])

    def test_missing_candidate_artifact_or_tool_fails_closed(self):
        for missing in ("godot", "template", "rustc", "cargo", "python3", "shasum"):
            with self.subTest(missing=missing):
                result, calls = self.execute(missing=(missing,))
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.native_calls(calls), [])
                self.assertEqual(self.native_build_calls(calls), [])
                self.assertEqual(self.gui_calls(calls), [])

    def test_each_failed_native_gate_prevents_gui_acceptance(self):
        for command in ("fmt", "clippy", "test", "doc", "build"):
            with self.subTest(command=command):
                result, calls = self.execute({"fail_baseline": command})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.native_calls(calls)[-1]["args"][0], command)
                self.assertEqual(self.native_build_calls(calls), [])
                self.assertEqual(self.gui_calls(calls), [])

    def test_native_artifact_failure_never_reaches_editor(self):
        for stage, count in (("production", 1), ("fixture", 2)):
            with self.subTest(stage=stage):
                result, calls = self.execute({"fail_native_build": stage})
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(len(self.native_build_calls(calls)), count)
                self.assertEqual(self.gui_calls(calls), [])

    def test_failed_edit_suite_cannot_start_observation(self):
        result, calls = self.execute({"fail_gui": "script-edit"})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call["args"][0] for call in self.gui_calls(calls)],
                         ["godot-addon/tests/run_script_edit.py"])

    def test_wrong_checkout_fails_before_candidate_execution(self):
        result, calls = self.execute({"head": "b" * 40})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call["command"] for call in calls], ["git"])

    def test_manual_dispatch_keeps_trusted_gui_boundary(self):
        self.assertEqual(set(self.live["on"]), {"workflow_dispatch"})
        self.assertEqual(set(self.live["on"]["workflow_dispatch"]["inputs"]), {"reviewed_sha"})
        self.assertEqual(self.gui["needs"], "trust-gate")
        self.assertEqual(self.gui["environment"], "live-editor")
        self.assertEqual(self.gui["runs-on"], ["self-hosted", "macOS", "ARM64", "godot-live-editor-ephemeral"])
        checkout = self.gui["steps"][0]
        self.assertEqual(checkout["with"]["ref"], "${{ needs.trust-gate.outputs.revision }}")
        self.assertEqual(checkout["with"]["persist-credentials"], "false")
        for job in self.ci["jobs"].values():
            self.assertNotIn("self-hosted", job["runs-on"])



if __name__ == "__main__":
    unittest.main()
