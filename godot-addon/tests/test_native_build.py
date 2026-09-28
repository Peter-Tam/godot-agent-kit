"""Reject ABI descriptions that would undersize native Variant storage."""
import importlib.util
from pathlib import Path
import unittest


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


if __name__ == "__main__":
    unittest.main()
