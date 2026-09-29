"""Reject unsupported Godot binaries and unsafe native ABI layouts."""
import contextlib
import importlib.util
import io
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


if __name__ == "__main__":
    unittest.main()
