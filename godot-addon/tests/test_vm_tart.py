"""Deterministic control-plane safety tests, not VM/product acceptance."""
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import vm_tart as vm


class TartTests(unittest.TestCase):
    def setUp(self):
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.tart = vm.Tart(Path(self.directory.name).resolve() / "private-state")
        self.rows = {}

    def own(self, **changes):
        receipt = {"vm": vm.VM_NAME, "home": str(self.tart.home),
                   "image": vm.DEFAULT_IMAGE, "base": False, "configured": True}
        receipt.update(changes)
        self.tart._write_json(self.tart.receipt, receipt)

    def stopped(self, name=vm.VM_NAME):
        self.rows[name] = {"Name": name, "State": "stopped", "Running": False}

    def running(self):
        self.rows[vm.VM_NAME] = {"Name": vm.VM_NAME, "State": "running", "Running": True}

    def launcher(self, bootstrap=False):
        self.tart._write_json(self.tart.launcher, {
            "pid": 4242, "identity": "Fri Oct 2 17:00:00 2026 owned-tart",
            "argv": self.tart._launch_argv(bootstrap),
            "network": "bootstrap-nat" if bootstrap else "host-only"})

    def inventory(self):
        return mock.patch.object(self.tart, "_inventory", side_effect=lambda: dict(self.rows))

    def test_missing_backend_gives_install_guidance_and_never_searches_path(self):
        with mock.patch("vm_tart.subprocess.run") as run:
            with self.assertRaisesRegex(vm.VMError, "LOCAL_VM.md"):
                self.tart.status()
        run.assert_not_called()

    def test_wrong_backend_version_rejected(self):
        self.tart.backend.parent.mkdir(parents=True)
        self.tart.backend.write_text("backend")
        self.tart.backend.chmod(0o700)
        with mock.patch("vm_tart.subprocess.run", return_value=subprocess.CompletedProcess([], 0, b"2.39.0\n", b"")):
            with self.assertRaisesRegex(vm.VMError, "Expected Tart 2.40.1"):
                self.tart.status()

    def test_state_symlink_does_not_touch_unrelated_directory(self):
        external = Path(self.directory.name) / "external"
        external.mkdir()
        marker = external / "keep"
        marker.write_text("untouched")
        alias = Path(self.directory.name) / "alias"
        alias.symlink_to(external, target_is_directory=True)
        with self.assertRaisesRegex(vm.VMError, "symlink"):
            vm.Tart(alias)
        self.assertEqual(marker.read_text(), "untouched")
        self.assertFalse((external / "tart").exists())

    def test_stopped_exec_never_invokes_host_or_backend_command(self):
        self.own()
        self.stopped()
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "no host fallback"):
                self.tart.exec(["/usr/bin/true"])
        run.assert_not_called()

    def test_missing_exec_never_invokes_command(self):
        self.own()
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaises(vm.VMError):
                self.tart.exec(["/usr/bin/true"])
        run.assert_not_called()

    def test_external_running_vm_is_not_stopped_or_executed(self):
        self.own()
        self.running()
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            for operation in (self.tart.stop, lambda: self.tart.exec(["/usr/bin/true"])):
                with self.subTest(operation=operation), self.assertRaises(vm.VMError):
                    operation()
        run.assert_not_called()

    def test_pid_reuse_or_different_vm_lock_rejects_running_vm(self):
        self.own()
        self.running()
        self.launcher()
        for identity, pid in (("different birth", 4242), ("Fri Oct 2 17:00:00 2026 owned-tart", 4343)):
            with self.subTest(identity=identity, pid=pid), self.inventory(), mock.patch.object(self.tart, "_process_identity", return_value=identity), mock.patch.object(self.tart, "_lock_pid", return_value=pid):
                with self.assertRaisesRegex(vm.VMError, "PID/identity"):
                    self.tart.status()

    def test_unsafe_launcher_options_rejected_even_with_live_pid(self):
        self.own()
        self.running()
        self.launcher()
        receipt = self.tart._json(self.tart.launcher)
        receipt["argv"] += ["--dir", "/private/host"]
        self.tart._write_json(self.tart.launcher, receipt)
        with self.inventory(), self.assertRaisesRegex(vm.VMError, "Unsafe"):
            self.tart.status()

    def test_unmarked_stopped_profile_not_adopted_or_overwritten(self):
        self.stopped()
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "unmarked"):
                self.tart.setup()
        run.assert_not_called()

    def test_interrupted_owned_clone_configures_without_recloning(self):
        self.own(configured=False)
        self.stopped()
        changes = []
        def configure(args, **kwargs):
            changes.append(args[0])
            return subprocess.CompletedProcess(args, 0, b"", b"")
        with self.inventory(), mock.patch.object(self.tart, "_run", side_effect=configure):
            self.tart.setup()
            self.tart.setup()
        self.assertEqual(changes, ["set"])
        self.assertTrue(self.tart._json(self.tart.receipt)["configured"])

    def test_clone_failure_preserves_recoverable_ownership(self):
        with self.inventory(), mock.patch.object(self.tart, "_run", return_value=subprocess.CompletedProcess([], 1, b"", b"download failed")):
            with self.assertRaisesRegex(vm.VMError, "download failed"):
                self.tart.setup()
        self.assertFalse(self.tart._owned()["configured"])

    def test_reset_never_deletes_running_vm_or_unowned_base(self):
        self.own(base=True)
        self.running()
        self.stopped(vm.BASE_NAME)
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "must be stopped"):
                self.tart.reset()
        run.assert_not_called()
        self.own(base=False)
        self.stopped()
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "No owned base"):
                self.tart.reset()
        run.assert_not_called()

    def test_save_base_does_not_replace_existing_base(self):
        self.own(base=True)
        self.stopped()
        self.stopped(vm.BASE_NAME)
        with self.inventory(), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "already exists"):
                self.tart.save_base()
        run.assert_not_called()

    def test_reset_replaces_only_fixed_stopped_owned_profile(self):
        self.own(base=True)
        self.stopped()
        self.stopped(vm.BASE_NAME)
        unrelated = "personal-vm"
        self.stopped(unrelated)
        def backend(args, **kwargs):
            if args[0] == "delete":
                del self.rows[args[1]]
            elif args[0] == "clone":
                self.stopped(args[2])
            return subprocess.CompletedProcess(args, 0, b"", b"")
        with self.inventory(), mock.patch.object(self.tart, "_run", side_effect=backend):
            self.tart.reset()
        self.assertEqual(set(self.rows), {vm.VM_NAME, vm.BASE_NAME, unrelated})

    def test_network_mode_change_requires_explicit_stop(self):
        self.own()
        self.running()
        with self.inventory(), mock.patch.object(self.tart, "_backend"), mock.patch.object(self.tart, "_verify_launcher", return_value={"network": "bootstrap-nat"}), mock.patch.object(self.tart, "_run") as run:
            with self.assertRaisesRegex(vm.VMError, "stop before"):
                self.tart.start()
        run.assert_not_called()

    def test_detached_launch_is_isolated_and_readiness_failure_is_clear(self):
        self.own()
        self.stopped()
        process = mock.Mock(pid=4242)
        process.poll.return_value = None
        with mock.patch.object(self.tart, "_backend"), mock.patch.object(self.tart, "_inventory", side_effect=[dict(self.rows), {vm.VM_NAME: {"Running": True}}]), mock.patch.object(self.tart, "_process_identity", return_value="identity"), mock.patch.object(self.tart, "_verify_launcher"), mock.patch.object(self.tart, "_run", return_value=subprocess.CompletedProcess([], 1, b"", b"no guest agent")), mock.patch("vm_tart.subprocess.Popen", return_value=process) as launch, mock.patch("vm_tart.time.monotonic", side_effect=[0, 0, 0, 181]), mock.patch("vm_tart.time.sleep"):
            with self.assertRaisesRegex(vm.VMError, "Guest Agent/vsock unavailable"):
                self.tart.start()
        args = launch.call_args.args[0]
        self.assertIn("--net-host", args)
        for flag in ("--no-graphics", "--no-audio", "--no-clipboard", "--no-usb-accessories"):
            self.assertIn(flag, args)
        for flag in ("--dir", "--vnc", "--graphics", "--disk"):
            self.assertNotIn(flag, args)
        self.assertTrue(launch.call_args.kwargs["start_new_session"])
        self.assertEqual(launch.call_args.kwargs["stdin"], subprocess.DEVNULL)
        self.assertEqual(self.tart.log.stat().st_mode & 0o777, 0o600)
        self.assertEqual(self.tart._json(self.tart.launcher)["network"], "host-only")
        self.assertNotIn("--net-host", self.tart._launch_argv(True))

    def test_running_boot_waits_through_control_timeout_without_relaunch(self):
        self.own()
        self.running()
        responses = [vm.VMError("connection timeout"), subprocess.CompletedProcess([], 0, b"", b"")]
        with (self.inventory(), mock.patch.object(self.tart, "_backend"),
              mock.patch.object(self.tart, "_verify_launcher", return_value={"network": "host-only"}),
              mock.patch.object(self.tart, "_run", side_effect=responses),
              mock.patch("vm_tart.subprocess.Popen") as launch,
              mock.patch("vm_tart.time.monotonic", side_effect=[0, 0, 0, 1, 1]),
              mock.patch("vm_tart.time.sleep")):
            self.tart.start()
        launch.assert_not_called()

    def test_unresponsive_guest_stops_but_cannot_claim_durable_snapshot(self):
        self.own()
        self.running()
        def backend(args, **kwargs):
            if args[0] == "exec":
                return subprocess.CompletedProcess(args, 1, b"", b"guest unavailable")
            self.stopped()
            return subprocess.CompletedProcess(args, 0, b"", b"")
        with (self.inventory(), mock.patch.object(self.tart, "_verify_launcher"),
              mock.patch.object(self.tart, "_run", side_effect=backend)):
            with self.assertRaises(vm.VMError):
                self.tart.stop()
        self.assertEqual(self.rows[vm.VM_NAME]["State"], "stopped")
        self.assertFalse(self.tart._owned()["base"])


    def test_transport_timeout_is_infrastructure_error(self):
        self.tart._version_checked = True
        with mock.patch("vm_tart.subprocess.run", side_effect=subprocess.TimeoutExpired("tart", 1)):
            with self.assertRaisesRegex(vm.VMError, "transport failed"):
                self.tart._run(["exec", vm.VM_NAME, "/usr/bin/true"], timeout=1)

    def test_upload_fixed_receiver_transfers_binary_and_rejects_mutated_input(self):
        source = Path(self.directory.name) / "source"
        payload = b"\x00\xff\n" * 20000
        source.write_bytes(payload)
        target = Path(self.directory.name) / "guest" / "name with spaces"
        def execute(argv, **kwargs):
            return subprocess.run([sys.executable, *argv[1:]], capture_output=True, check=True, **kwargs)
        with mock.patch.object(self.tart, "exec", side_effect=execute):
            self.tart.upload(source, str(target))
        self.assertEqual(target.read_bytes(), payload)
        def corrupt(argv, **kwargs):
            return subprocess.run([sys.executable, *argv[1:]], input=b"truncated", capture_output=True, check=True)
        with mock.patch.object(self.tart, "exec", side_effect=corrupt):
            with self.assertRaisesRegex(vm.VMError, "upload failed"):
                self.tart.upload(source, str(target))
        self.assertEqual(target.read_bytes(), payload)
        self.assertEqual(list(target.parent.iterdir()), [target])

    def test_upload_rejects_wrong_receipt(self):
        source = Path(self.directory.name) / "source"
        source.write_bytes(b"content")
        with mock.patch.object(self.tart, "exec", return_value=subprocess.CompletedProcess([], 0, b'{"bytes": 0, "sha256": "wrong"}', b"")):
            with self.assertRaisesRegex(vm.VMError, "receipt"):
                self.tart.upload(source, "/Users/admin/file")

    def test_upload_rejects_guest_traversal_before_transport(self):
        with mock.patch.object(self.tart, "exec") as execute:
            for destination in ("relative", "/Users/admin/../escape", "/", "/bad\0path"):
                with self.subTest(destination=destination), self.assertRaises(vm.VMError):
                    self.tart.upload(Path("unused"), destination)
        execute.assert_not_called()

    def test_telemetry_and_external_tart_configuration_are_not_inherited(self):
        with mock.patch.dict("os.environ", {"TRACEPARENT": "secret", "TRACESTATE": "secret", "OTEL_EXPORTER_OTLP_ENDPOINT": "external", "CIRRUS_TOKEN": "secret", "TART_HOME": "/external", "TART_NO_AUTO_PRUNE": "0"}):
            tart = vm.Tart(Path(self.directory.name).resolve() / "other-private-state")
        self.assertEqual(tart.env["TART_HOME"], str(tart.home))
        self.assertEqual(tart.env["TART_NO_AUTO_PRUNE"], "1")
        self.assertFalse(any(key.startswith(("TRACE", "OTEL", "CIRRUS")) for key in tart.env))


if __name__ == "__main__":
    unittest.main()
