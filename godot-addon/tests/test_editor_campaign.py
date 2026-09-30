"""Exercise serial campaign checkpoints with real isolated stand-in children.

These tests establish orchestration and evidence reuse, not Godot acceptance.
"""
import contextlib
import importlib.util
import hashlib
import io
import json
import os
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest import mock


TESTS = Path(__file__).resolve().parent
with mock.patch.object(sys, "path", [str(TESTS), *sys.path]):
    spec = importlib.util.spec_from_file_location(
        "editor_campaign", TESTS / "run_editor_campaign.py")
    campaign = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(campaign)

EXPECTED_SCENARIOS = {
    "open": ("native-boundary", "new-open", "already-open", "preservation",
             "routing", "interruption", "sequential", "composed", "privacy-export"),
    "edit": ("native-primitives", "clean-open", "conflicts", "routing", "interruption",
             "validation", "history", "durability", "sequential", "privacy-export"),
    "observation": ("all",),
}


@contextlib.contextmanager
def fixed_tool_probes(home):
    """No host compiler, SDK, rustup, or auto-install process is invoked."""
    run = campaign.subprocess.run

    def isolated(command, *args, **kwargs):
        if command[0] in ("rustc", "swiftc", "xcrun", "rustup"):
            return campaign.subprocess.CompletedProcess(
                command, 1, stdout=b"fixed unavailable tool", stderr=b"")
        return run(command, *args, **kwargs)

    with (mock.patch.object(campaign.Path, "home", return_value=home),
          mock.patch.object(campaign.shutil, "which", return_value=None),
          mock.patch.object(campaign.subprocess, "run", side_effect=isolated)):
        yield


# Outcome plans and traces live outside the campaign and runner source. Each child
# observes the checkpoint that existed when it started, before writing evidence.
CHILD = r'''
import argparse
import json
import os
from pathlib import Path
import sys

parser = argparse.ArgumentParser()
for name in ("godot", "observer", "editor", "opener", "stock-validator", "native-fault-addon"):
    parser.add_argument("--" + name)
parser.add_argument("--scenario", required=True)
parser.add_argument("--artifacts", required=True, type=Path)
args = parser.parse_args()
suite = {"run_script_open.py": "open", "run_script_edit.py": "edit",
         "run_observation.py": "observation"}[Path(sys.argv[0]).name]
key = suite + ":" + args.scenario
plan = json.loads(Path(os.environ["CAMPAIGN_PLAN"]).read_text()).get(key, {})
manifest = json.loads(Path(os.environ["CAMPAIGN_MANIFEST"]).read_text())
step = next(step for step in manifest["steps"]
            if (step["suite"], step["scenario"]) == (suite, args.scenario))
record = {"suite": suite, "scenario": args.scenario, "attempt": str(args.artifacts),
          "empty": not any(args.artifacts.iterdir()),
          "mode": args.artifacts.stat().st_mode & 0o777,
          "checkpoint": step}
with open(os.environ["CAMPAIGN_TRACE"], "a") as trace:
    trace.write(json.dumps(record) + "\n")
summary = {"status": plan.get("status", "passed"),
           "private_source": os.environ["CAMPAIGN_SECRET"]}
(args.artifacts / "summary.json").write_text(json.dumps(summary))
(args.artifacts / "retained.txt").write_text(key)
if plan.get("mutate"):
    Path(plan["mutate"]).write_text("changed during execution")
print(os.environ["CAMPAIGN_SECRET"])
if plan.get("interrupt_parent"):
    import signal
    signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})
    signal.alarm(5)
    os.kill(os.getppid(), signal.SIGTERM)
    signal.sigwait({signal.SIGINT})
    (args.artifacts / "cleaned-up.txt").write_text("runner finished cleanup")
if plan.get("signal"):
    os.kill(os.getpid(), plan["signal"])
sys.exit(plan.get("exit", 0))
'''


class CampaignTests(unittest.TestCase):
    def setUp(self):
        temporary = TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / "repo"
        tests = self.repo / "godot-addon/tests"
        tests.mkdir(parents=True)
        for filename in campaign.RUNNERS.values():
            (tests / filename).write_text(CHILD)
        self.directory = self.root / "campaign"
        self.plan = self.root / "plan.json"
        self.plan.write_text("{}")
        self.trace = self.root / "trace.jsonl"
        self.secret = "PRIVATE_SOURCE_AND_CREDENTIAL_9c77"
        self.inputs = {}
        for name in ("godot", "observer", "editor", "opener", "stock_validator"):
            path = self.root / name
            path.write_text(self.secret + name)
            path.chmod(0o700)
            self.inputs[name] = path
        fault = self.root / "fault"
        fault.mkdir()
        (fault / "library.dylib").write_text(self.secret)
        (fault / "build-manifest.json").write_text('{"fixture_only": true}')
        self.inputs["native_fault_addon"] = fault
        self.addCleanup(mock.patch.stopall)
        mock.patch.object(campaign, "REPO", self.repo).start()
        mock.patch.dict(os.environ, {
            "CAMPAIGN_PLAN": str(self.plan), "CAMPAIGN_TRACE": str(self.trace),
            "CAMPAIGN_MANIFEST": str(self.directory / "manifest.json"),
            "CAMPAIGN_SECRET": self.secret,
        }).start()

    def invoke(self, suite="all", *flags, real_fingerprint=False):
        argv = ["--suite", suite, "--campaign-dir", str(self.directory)]
        for name, path in self.inputs.items():
            argv.extend(["--" + name.replace("_", "-"), str(path)])
        argv.extend(flags)
        output = io.StringIO()
        with contextlib.ExitStack() as stack:
            stack.enter_context(contextlib.redirect_stdout(output))
            stack.enter_context(contextlib.redirect_stderr(output))
            if not real_fingerprint:
                stack.enter_context(mock.patch.object(campaign, "fingerprint", return_value="a" * 64))
            else:
                stack.enter_context(fixed_tool_probes(self.root))
            result = campaign.main(argv)
        return result, output.getvalue()

    def records(self):
        return [json.loads(line) for line in self.trace.read_text().splitlines()] if self.trace.exists() else []

    def manifest(self):
        return json.loads((self.directory / "manifest.json").read_text())

    def save_manifest(self, value):
        (self.directory / "manifest.json").write_text(json.dumps(value))

    def expected(self, suite="all"):
        suites = ("open", "edit", "observation") if suite == "all" else (suite,)
        return [(selected, scenario) for selected in suites for scenario in EXPECTED_SCENARIOS[selected]]

    def outcome(self, suite, scenario, **values):
        self.plan.write_text(json.dumps({suite + ":" + scenario: values}))

    def test_all_runs_in_order_with_private_empty_checkpointed_attempts(self):
        result, output = self.invoke()
        self.assertEqual(result, 0)
        records = self.records()
        self.assertEqual([(r["suite"], r["scenario"]) for r in records], self.expected())
        self.assertEqual([(r["suite"], r["scenario"]) for r in records if r["suite"] == "observation"],
                         [("observation", "all")])
        manifest = self.manifest()
        self.assertEqual(manifest["schema_version"], 1)
        self.assertEqual([(s["suite"], s["scenario"]) for s in manifest["steps"]], self.expected())
        for record, step in zip(records, manifest["steps"]):
            with self.subTest(step=(step["suite"], step["scenario"])):
                self.assertTrue(record["empty"])
                self.assertEqual(record["mode"], 0o700)
                self.assertEqual(record["checkpoint"]["status"], "running")
                self.assertEqual(step["status"], "passed")
                self.assertEqual(step["exit_code"], 0)
                self.assertEqual(step["execution"], "executed")
                self.assertEqual(step["fingerprint"], "a" * 64)
                self.assertRegex(step["summary_sha256"], r"^[0-9a-f]{64}$")
                self.assertTrue(step["command_identity"])
                self.assertFalse(Path(step["attempt_dir"]).is_absolute())
                self.assertEqual(self.directory / step["attempt_dir"], Path(record["attempt"]))
                self.assertEqual(self.directory / step["summary_path"], Path(record["attempt"]) / "summary.json")
                self.assertIn("PASS", output)
        self.assertNotIn(self.secret, (self.directory / "manifest.json").read_text())

    def test_fail_fast_records_uncompleted_steps_and_nonzero(self):
        first = self.expected()[0]
        self.outcome(*first, exit=7)
        result, output = self.invoke()
        self.assertNotEqual(result, 0)
        self.assertEqual(len(self.records()), 1)
        steps = self.manifest()["steps"]
        self.assertEqual(steps[0]["status"], "failed")
        self.assertEqual(steps[0]["exit_code"], 7)
        self.assertTrue(all(step["status"] == "not_run" and step["execution"] == "not_run"
                            for step in steps[1:]))
        self.assertIn("FAIL", output)
        for suite, scenario in self.expected():
            self.assertIn(scenario, output)

    def test_keep_going_reports_multiple_failures_without_losing_later_passes(self):
        failures = {("open", "routing"), ("edit", "history")}
        self.plan.write_text(json.dumps({suite + ":" + scenario: {"exit": 8}
                                       for suite, scenario in failures}))
        result, output = self.invoke("all", "--keep-going")
        self.assertNotEqual(result, 0)
        self.assertEqual([(r["suite"], r["scenario"]) for r in self.records()], self.expected())
        for step in self.manifest()["steps"]:
            key = (step["suite"], step["scenario"])
            failed = key in failures
            self.assertEqual(step["status"], "failed" if failed else "passed")
            self.assertIn(("FAIL " if failed else "PASS ") + "/".join(key), output)

    def test_fail_fast_resume_preserves_later_passed_checkpoints(self):
        self.outcome(*self.expected()[0], exit=7)
        self.assertNotEqual(self.invoke("all", "--keep-going")[0], 0)
        before = len(self.records())
        self.assertNotEqual(self.invoke("all", "--resume")[0], 0)
        self.assertEqual(len(self.records()), before + 1)
        self.assertTrue(all(step["execution"] == "reused"
                            for step in self.manifest()["steps"][1:]))
        self.plan.write_text("{}")
        self.assertEqual(self.invoke("all", "--resume")[0], 0)
        self.assertEqual(len(self.records()), before + 2)

    def test_parent_interruption_waits_for_runner_cleanup_and_never_accepts_it(self):
        self.outcome(*self.expected()[0], interrupt_parent=True)
        self.assertNotEqual(self.invoke("all", "--keep-going")[0], 0)
        self.assertEqual(len(self.records()), 1)
        first = self.manifest()["steps"][0]
        self.assertEqual(first["status"], "interrupted")
        self.assertEqual(first["exit_code"], 0)
        self.assertEqual((self.directory / first["attempt_dir"] / "cleaned-up.txt").read_text(),
                         "runner finished cleanup")

    def test_child_interruption_stops_even_keep_going(self):
        self.outcome(*self.expected()[0], signal=15)
        self.assertNotEqual(self.invoke("all", "--keep-going")[0], 0)
        self.assertEqual(len(self.records()), 1)
        steps = self.manifest()["steps"]
        self.assertEqual(steps[0]["status"], "interrupted")
        self.assertTrue(all(step["status"] == "not_run" for step in steps[1:]))

    def test_nonzero_child_is_never_reused_despite_passed_summary(self):
        self.outcome("observation", "all", exit=3)
        self.assertNotEqual(self.invoke("observation")[0], 0)
        failed = self.manifest()["steps"][0]
        old_attempt = self.directory / failed["attempt_dir"]
        old_summary = (old_attempt / "summary.json").read_bytes()
        self.plan.write_text("{}")
        self.assertEqual(self.invoke("observation", "--resume")[0], 0)
        new = self.manifest()["steps"][0]
        self.assertNotEqual(new["attempt_dir"], failed["attempt_dir"])
        self.assertEqual((old_attempt / "summary.json").read_bytes(), old_summary)
        self.assertEqual((old_attempt / "retained.txt").read_text(), "observation:all")
        self.assertEqual(len(self.records()), 2)

    def test_matching_passed_steps_resume_without_spawning(self):
        self.assertEqual(self.invoke()[0], 0)
        before = self.records()
        with mock.patch.object(campaign.subprocess, "Popen", side_effect=AssertionError("spawned reused child")):
            result, output = self.invoke("all", "--resume")
        self.assertEqual(result, 0)
        self.assertEqual(self.records(), before)
        self.assertTrue(all(step["execution"] == "reused" for step in self.manifest()["steps"]))
        self.assertIn("REUSED", output)

    def test_untrustworthy_checkpoint_evidence_reruns(self):
        for damage in ("missing", "corrupt", "summary_status", "non_object", "hash",
                       "fingerprint", "identity", "running", "interrupted", "exit", "path"):
            with self.subTest(damage=damage):
                self.directory = self.root / damage
                os.environ["CAMPAIGN_MANIFEST"] = str(self.directory / "manifest.json")
                self.assertEqual(self.invoke("observation")[0], 0)
                checkpoint = self.manifest()
                step = checkpoint["steps"][0]
                old_attempt = step["attempt_dir"]
                summary = self.directory / step["summary_path"]
                if damage == "missing":
                    summary.unlink()
                elif damage == "corrupt":
                    summary.write_text("{broken")
                    step["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
                elif damage in ("summary_status", "non_object"):
                    summary.write_text('{"status": "failed"}' if damage == "summary_status" else "[]")
                    step["summary_sha256"] = hashlib.sha256(summary.read_bytes()).hexdigest()
                elif damage == "hash":
                    summary.write_text('{"status": "passed", "extra": "changed"}')
                elif damage == "fingerprint":
                    step["fingerprint"] = "b" * 64
                elif damage == "identity":
                    step["command_identity"] = "untrusted command"
                elif damage in ("running", "interrupted"):
                    step["status"] = damage
                elif damage == "exit":
                    step["exit_code"] = 9
                else:
                    step["summary_path"] = "../plan.json"
                self.save_manifest(checkpoint)
                count = len(self.records())
                self.assertEqual(self.invoke("observation", "--resume")[0], 0)
                self.assertEqual(len(self.records()), count + 1)
                self.assertNotEqual(self.manifest()["steps"][0]["attempt_dir"], old_attempt)

    def test_zero_exit_with_failed_summary_is_failure(self):
        self.outcome("observation", "all", status="failed")
        self.assertNotEqual(self.invoke("observation")[0], 0)
        self.assertEqual(self.manifest()["steps"][0]["status"], "failed")

    def test_inputs_changed_during_child_execution_cannot_pass(self):
        self.outcome("observation", "all", mutate=str(self.inputs["observer"]))
        self.assertNotEqual(self.invoke("observation", real_fingerprint=True)[0], 0)
        self.assertEqual(self.manifest()["steps"][0]["status"], "failed")

    def test_malformed_manifest_cannot_reuse_or_execute_embedded_commands(self):
        self.assertEqual(self.invoke("observation")[0], 0)
        marker = self.root / "command-was-executed"
        self.save_manifest({"schema_version": 1, "steps": [{"suite": "observation", "scenario": "all",
                            "command": [sys.executable, "-c", "open(%r, 'w').close()" % str(marker)]}]})
        before = len(self.records())
        self.assertEqual(self.invoke("observation", "--resume")[0], 0)
        self.assertEqual(len(self.records()), before + 1)
        self.assertFalse(marker.exists())

    def test_atomic_replace_observes_complete_json_and_preserves_previous_on_interrupt(self):
        self.assertEqual(self.invoke("observation")[0], 0)
        manifest_path = self.directory / "manifest.json"
        previous = manifest_path.read_bytes()
        replace = campaign.os.replace
        observed = []

        def interrupt(source, destination):
            if Path(destination) == manifest_path:
                observed.append(json.loads(Path(source).read_text()))
                self.assertEqual(manifest_path.read_bytes(), previous)
                raise KeyboardInterrupt
            return replace(source, destination)

        with mock.patch.object(campaign.os, "replace", side_effect=interrupt):
            try:
                result, _ = self.invoke("observation", "--resume")
            except KeyboardInterrupt:
                pass
            else:
                self.assertNotEqual(result, 0)
        self.assertTrue(observed)
        self.assertEqual(manifest_path.read_bytes(), previous)
        replacements = []

        def observe(source, destination):
            if Path(destination) == manifest_path:
                replacements.append(json.loads(Path(source).read_text()))
                json.loads(manifest_path.read_text())
            return replace(source, destination)

        with mock.patch.object(campaign.os, "replace", side_effect=observe):
            self.assertEqual(self.invoke("observation", "--resume")[0], 0)
        self.assertTrue(replacements)
        self.assertEqual(self.manifest()["steps"][0]["execution"], "reused")

    def test_cli_cannot_choose_arbitrary_suite_scenario_or_runner(self):
        valid = ["--suite", "observation", "--campaign-dir", str(self.directory),
                 "--godot", str(self.inputs["godot"]), "--observer", str(self.inputs["observer"])]
        for arguments in (["--suite", "../plan.json"], ["--scenario", "../../evil.py"],
                          ["--runner", str(self.plan)]):
            with self.subTest(arguments=arguments), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as rejected:
                    campaign.main([*valid, *arguments])
                self.assertEqual(rejected.exception.code, 2)
        self.assertEqual(self.records(), [])


class FingerprintTests(unittest.TestCase):
    def test_execution_inputs_change_fingerprint_but_documentation_does_not(self):
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = root / "repo"
            relative_inputs = (
                "godot-addon/tests/run_script_open.py",
                "godot-addon/tests/run_script_edit.py",
                "godot-addon/tests/run_observation.py",
                "godot-addon/tests/caller_open_acceptance.py",
                "godot-addon/tests/fixture_bridge.gd",
                "godot-addon/tests/fixtures/script_open/project.godot",
                "godot-addon/tests/fixtures/script_edit/scripts/subject.gd",
                "godot-addon/tests/fixtures/observation/project.godot",
                "godot-addon/addons/godot_agent_kit/plugin.gd",
                "godot-addon/addons/godot_agent_kit/native/editor_integration.gdextension",
                "godot-addon/addons/godot_agent_kit/native/libeditor_integration.macos.arm64.dylib",
                "godot-addon/addons/godot_agent_kit/native/build-manifest.json",
                "godot-addon/native/build.py",
                "godot-addon/native/editor_integration.cpp",
                "mcp-server/Cargo.lock",
                "mcp-server/src/lib.rs",
                "mcp-server/rust-toolchain.toml",
            )
            paths = []
            for relative in relative_inputs:
                path = repo / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("original input")
                paths.append(path)
            values = {}
            for name in ("godot", "observer", "editor", "opener", "stock_validator"):
                path = root / name
                path.write_text("binary input " + name)
                path.chmod(0o700)
                values[name] = path
                paths.append(path)
            for relative in ("libgodot_agent_kit.rlib", "deps/libserde_json-test.rlib",
                             "Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip"):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("runtime dependency")
                paths.append(path)
            fault = root / "fault"
            fault.mkdir()
            for name in ("editor_integration.gdextension", "libeditor_integration.macos.arm64.dylib", "build-manifest.json"):
                path = fault / name
                path.write_text("fault " + name)
                paths.append(path)
            args = SimpleNamespace(**values, native_fault_addon=fault, suite="all",
                                   campaign_dir=root / "campaign", keep_going=False, resume=False)
            with mock.patch.object(campaign, "REPO", repo), fixed_tool_probes(root):
                baseline = {suite: campaign.fingerprint(args, suite, scenario)
                            for suite, scenario in (("open", "new-open"), ("edit", "clean-open"))}
                self.assertRegex(baseline["open"], r"^[0-9a-f]{64}$")
                self.assertEqual(campaign.fingerprint(args, "open", "new-open"), baseline["open"])
                for path in paths:
                    with self.subTest(input=path.relative_to(root)):
                        suite, scenario = (("edit", "clean-open") if "script_edit" in path.parts
                                           else ("open", "new-open"))
                        original = path.read_bytes()
                        path.write_bytes(original + b" mutation")
                        self.assertNotEqual(campaign.fingerprint(args, suite, scenario), baseline[suite])
                        path.write_bytes(original)
                for relative in ("README.md", "docs/maintenance.md", "godot-addon/tests/README.md",
                                 "godot-addon/addons/godot_agent_kit/README.md",
                                 "godot-addon/tests/fixtures/script_open/README.md"):
                    path = repo / relative
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("docs do not affect editor execution")
                self.assertEqual(campaign.fingerprint(args, "open", "new-open"), baseline["open"])


if __name__ == "__main__":
    unittest.main()
