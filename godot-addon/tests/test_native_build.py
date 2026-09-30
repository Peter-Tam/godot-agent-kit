"""Reject unsupported Godot binaries and unsafe native ABI layouts."""
import contextlib
import importlib.util
import io
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest import mock


spec = importlib.util.spec_from_file_location(
    "native_build", Path(__file__).resolve().parents[1] / "native" / "build.py")
native_build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native_build)


class AbiRefusalTests(unittest.TestCase):
    def test_double_precision_cannot_use_present_float_layout(self):
        api = {"header": {"precision": "double"}, "builtin_class_sizes": [
            {"build_configuration": "float_64", "sizes": [{"name": "Variant", "size": 24}]},
            {"build_configuration": "double_64", "sizes": [{"name": "Variant", "size": 40}]},
        ]}
        with self.assertRaises(ValueError):
            native_build.single_precision_sizes(api)

    def test_missing_precision_cannot_default_to_float(self):
        api = {"header": {}, "builtin_class_sizes": [
            {"build_configuration": "float_64", "sizes": [{"name": "Variant", "size": 24}]},
        ]}
        with self.assertRaises(ValueError):
            native_build.single_precision_sizes(api)

    def test_32_bit_layout_cannot_supply_64_bit_storage(self):
        api = {"header": {"precision": "single"}, "builtin_class_sizes": [
            {"build_configuration": "float_32", "sizes": [{"name": "String", "size": 4}]},
        ]}
        with self.assertRaises(ValueError):
            native_build.single_precision_sizes(api)


class StockBinaryRefusalTests(unittest.TestCase):
    def test_unknown_executable_is_refused_without_invoking_it(self):
        with TemporaryDirectory() as directory:
            binary = Path(directory) / "unknown-godot"
            binary.write_bytes(b"not the official Godot editor")
            stderr = io.StringIO()
            with (mock.patch.object(native_build.platform, "system", return_value="Darwin"),
                  mock.patch.object(native_build.platform, "machine", return_value="arm64"),
                  mock.patch.object(native_build, "run", side_effect=AssertionError("unknown executable invoked")),
                  contextlib.redirect_stderr(stderr),
                  self.assertRaises(SystemExit) as refused):
                native_build.main(["--godot", str(binary)])
            self.assertEqual(refused.exception.code, 2)

    def test_verified_binary_requires_exact_official_version(self):
        with TemporaryDirectory() as directory:
            binary = Path(directory) / "godot"
            binary.write_bytes(b"test binary")
            stderr = io.StringIO()
            with (mock.patch.object(native_build.platform, "system", return_value="Darwin"),
                  mock.patch.object(native_build.platform, "machine", return_value="arm64"),
                  mock.patch.object(native_build, "digest", return_value=native_build.STOCK_SHA256),
                  mock.patch.object(native_build, "run",
                                    return_value=SimpleNamespace(stdout="4.7.2.stable.custom.ed1daf0bf")) as run,
                  contextlib.redirect_stderr(stderr),
                  self.assertRaises(SystemExit) as refused):
                native_build.main(["--godot", str(binary)])
            run.assert_called_once_with([str(binary.resolve()), "--version"])
            self.assertEqual(refused.exception.code, 2)


class InstalledCutoverTests(unittest.TestCase):
    def install_old(self, directory, library_bytes=b"old kit library"):
        addon = Path(directory)
        (addon / "script_edit.gdextension").write_text(native_build.OLD_DESCRIPTOR)
        library = addon / "libscript_edit.macos.arm64.dylib"
        library.write_bytes(library_bytes)
        (addon / "build-manifest.json").write_text(json.dumps({
            "base_commit": native_build.BASE,
            "engine_sha256": native_build.STOCK_SHA256,
            "native_library_sha256": native_build.digest(library),
        }))
        return addon, library

    def test_matched_obsolete_artifacts_removed_without_touching_neighbors(self):
        with TemporaryDirectory() as directory:
            addon, library = self.install_old(directory)
            neighbor = addon / "unrelated.gdextension"
            neighbor.write_text("unrelated plugin")
            native_build.remove_obsolete_artifacts(addon)
            self.assertFalse(library.exists())
            self.assertFalse((addon / "script_edit.gdextension").exists())
            self.assertEqual(neighbor.read_text(), "unrelated plugin")

    def test_replaced_library_refuses_entire_removal_without_deleting_descriptor(self):
        with TemporaryDirectory() as directory:
            addon, library = self.install_old(directory)
            library.write_bytes(b"user replacement")
            with self.assertRaises(ValueError):
                native_build.remove_obsolete_artifacts(addon)
            self.assertEqual(library.read_bytes(), b"user replacement")
            self.assertEqual((addon / "script_edit.gdextension").read_text(), native_build.OLD_DESCRIPTOR)

    def test_unrelated_old_descriptor_is_preserved_with_matching_library(self):
        with TemporaryDirectory() as directory:
            addon, library = self.install_old(directory)
            descriptor = addon / "script_edit.gdextension"
            descriptor.write_text("user-owned different extension")
            with self.assertRaises(ValueError):
                native_build.remove_obsolete_artifacts(addon)
            self.assertEqual(descriptor.read_text(), "user-owned different extension")
            self.assertEqual(library.read_bytes(), b"old kit library")

    def test_symlink_library_cannot_delete_external_file_or_descriptor(self):
        with TemporaryDirectory() as directory:
            addon, library = self.install_old(directory)
            external = addon / "external-library"
            external.write_bytes(b"external")
            library.unlink()
            library.symlink_to(external)
            with self.assertRaises(ValueError):
                native_build.remove_obsolete_artifacts(addon)
            self.assertTrue(library.is_symlink())
            self.assertEqual(external.read_bytes(), b"external")
            self.assertTrue((addon / "script_edit.gdextension").exists())




if __name__ == "__main__":
    unittest.main()
