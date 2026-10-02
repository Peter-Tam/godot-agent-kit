"""Host orchestration contracts, not virtualization or Godot acceptance."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest import mock

import run_in_vm as vm


class SourceAndEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "VM test")
        self.git("config", "user.email", "vm-test@example.invalid")
        (self.repo / "input.txt").write_text("committed input\n")
        self.git("add", "input.txt")
        self.git("commit", "-qm", "fixture")
        self.commit = self.git("rev-parse", "HEAD").strip()

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.repo), *args], check=True,
                              capture_output=True, text=True).stdout

    def archive(self, entries):
        archive = self.root / "evidence.tar"
        with tarfile.open(archive, "w") as stream:
            for name, value, kind in entries:
                member = tarfile.TarInfo(name)
                member.type = kind
                if kind == tarfile.REGTYPE:
                    member.size = len(value)
                    stream.addfile(member, io.BytesIO(value))
                else:
                    member.linkname = value.decode()
                    stream.addfile(member)
        return archive

    def test_dirty_default_refuses_but_explicit_revision_is_committed_tree(self):
        self.assertEqual(vm._revision(self.repo, None), self.commit)
        (self.repo / "input.txt").write_text("not the committed input\n")
        (self.repo / "private.txt").write_text("not tracked\n")
        with self.assertRaisesRegex(vm.VMError, "Uncommitted source"):
            vm._revision(self.repo, None)
        selected = vm._revision(self.repo, "HEAD")
        archive = self.root / "source.tar"
        vm._git(self.repo, "archive", "--format=tar", "--output=" + str(archive), selected)
        with tarfile.open(archive) as source:
            self.assertEqual(source.getnames(), ["input.txt"])
            self.assertEqual(source.extractfile("input.txt").read(), b"committed input\n")

    def test_unknown_or_option_revision_cannot_select_source(self):
        for value in ("does-not-exist", "--all", "HEAD; touch injected"):
            with self.subTest(value=value), self.assertRaises(vm.VMError):
                vm._revision(self.repo, value)
        self.assertFalse((self.repo / "injected").exists())

    def test_worker_upload_pins_bytes_when_local_helper_changes(self):
        source = self.root / "vm_guest.py"
        original = b"original guest control version\n"
        source.write_bytes(original)
        copied = {}
        def transfer(snapshot, destination):
            source.write_bytes(b"next guest control version\n")
            copied[destination] = Path(snapshot).read_bytes()
        tart = mock.Mock()
        tart.upload.side_effect = transfer
        with mock.patch.object(vm, "__file__", str(self.root / "run_in_vm.py")):
            destination, identity = vm._install_worker(tart)
        self.assertEqual(identity, hashlib.sha256(original).hexdigest())
        self.assertEqual(Path(destination).name, identity + ".py")
        self.assertEqual(copied[destination], original)

    def test_evidence_mapping_is_private_and_never_overwrites_prior_fetch(self):
        data = b'{"status":"failed"}\n'
        archive = self.archive([("artifacts/summary.json", data, tarfile.REGTYPE)])
        destination = self.root / "vm-evidence"
        vm._extract(archive, destination)
        summary = destination / "artifacts/summary.json"
        self.assertEqual(summary.read_bytes(), data)
        self.assertEqual(summary.stat().st_mode & 0o777, 0o600)
        self.assertEqual(destination.stat().st_mode & 0o777, 0o700)
        with self.assertRaises(FileExistsError):
            vm._extract(archive, destination)
        self.assertEqual(summary.read_bytes(), data)

    def test_guest_archive_cannot_escape_or_install_symlinks(self):
        for name, kind in (("../escaped", tarfile.REGTYPE),
                           ("/tmp/escaped", tarfile.REGTYPE),
                           ("summary.json", tarfile.SYMTYPE),
                           ("summary.json", tarfile.LNKTYPE)):
            with self.subTest(name=name, kind=kind):
                archive = self.archive([(name, b"../../escaped", kind)])
                with self.assertRaises(vm.VMError):
                    vm._extract(archive, self.root / ("fetch-" + str(len(list(self.root.iterdir())))))
        self.assertFalse((self.root / "escaped").exists())

    def test_duplicate_archive_entry_refuses_instead_of_replacing_evidence(self):
        archive = self.archive([("summary.json", b"failure", tarfile.REGTYPE),
                                ("summary.json", b"pass", tarfile.REGTYPE)])
        with self.assertRaises(vm.VMError):
            vm._extract(archive, self.root / "duplicate")
        self.assertEqual((self.root / "duplicate/summary.json").read_bytes(), b"failure")

    def test_missing_vm_is_error_without_source_or_host_runner_execution(self):
        with (mock.patch.object(vm, "Tart", side_effect=vm.VMError("VM is stopped")),
              mock.patch.object(vm, "_sync") as sync,
              contextlib.redirect_stderr(io.StringIO()) as errors):
            status = vm.main(["--state", str(self.root / "state"), "run", "close",
                              "--scenario", "preservation"])
        self.assertEqual(status, 1)
        self.assertIn("No host Godot fallback", errors.getvalue())
        sync.assert_not_called()

    def test_reachable_vm_without_completed_readiness_cannot_run_acceptance(self):
        tart = mock.Mock()
        tart.status.return_value = {"running": True, "startup_readiness": None}
        args = vm._parser().parse_args(["run", "close", "--scenario", "clean-close"])
        with mock.patch.object(vm, "_install_worker") as install:
            with self.assertRaises(vm.VMError):
                vm._execute(tart, self.root / "state", args, repo=self.repo)
        install.assert_not_called()
        self.assertFalse((self.root / "state/artifacts").exists())

    def test_failed_runner_status_survives_artifact_transfer_failure(self):
        args = vm._parser().parse_args(["run", "edit", "--scenario", "clean-open",
                                       "--run-id", "retained-failure", "--revision", "HEAD"])
        state = self.root / "state"
        tart = mock.Mock()
        tart.status.return_value = {"state": "running", "running": True,
                                    "startup_readiness": {"load_average_1m": 1.0, "cpu_count": 4}}
        with (mock.patch.object(vm, "_sync"),
              mock.patch.object(vm, "_worker", return_value=subprocess.CompletedProcess([], 37, b"", b"")),
              mock.patch.object(vm, "_fetch", side_effect=subprocess.CalledProcessError(70, ["tart", "exec"])),
              mock.patch.object(vm, "_host_godot_processes", return_value=[]),
              contextlib.redirect_stderr(io.StringIO()), contextlib.redirect_stdout(io.StringIO())):
            status = vm._execute(tart, state, args, repo=self.repo)
        self.assertEqual(status, 37)
        records = list((state / "artifacts/retained-failure").glob("*.host.json"))
        self.assertEqual(len(records), 1)
        record = json.loads(records[0].read_text())
        self.assertEqual(record["runner_exit_code"], 37)
        self.assertEqual(record["revision"], self.commit)
        self.assertEqual(record["run_id"], "retained-failure")

    def test_command_surface_refuses_shell_commands_and_ambiguous_resume(self):
        for arguments in (["run", "/bin/sh", "-c", "true"],
                          ["run", "close", "--scenario", "preservation", "--godot", "/host/Godot"],
                          ["campaign", "open", "--resume"],
                          ["fetch-artifacts", "../other"], ["reset"]):
            with self.subTest(arguments=arguments), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as failure:
                    vm.main(arguments)
                self.assertEqual(failure.exception.code, 2)

    def test_process_sampling_uses_executable_names_not_backend_or_mcp_titles(self):
        output = "101 tart\n102 Godot\n103 node\n"
        with mock.patch.object(vm.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, output, "")):
            matches = vm._host_godot_processes()
        self.assertEqual([item.split()[0] for item in matches], ["102"])


if __name__ == "__main__":
    unittest.main()
