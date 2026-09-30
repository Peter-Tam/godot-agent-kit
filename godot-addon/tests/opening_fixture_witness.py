"""Private opening fixture sources and independent editor witness projections."""
import json
from pathlib import Path

from run_script_edit import sha

FIXTURE = Path(__file__).parent / "fixtures" / "script_open"
TARGET = "res://scripts/subject.gd"
CURRENT = "res://scripts/other.gd"
BACKGROUND = "res://scripts/open/background.gd"
TARGET_SOURCE = (FIXTURE / "scripts/target.gd").read_text()
CURRENT_SOURCE = (FIXTURE / "scripts/current.gd").read_text()
INVALID_SOURCE = (FIXTURE / "scripts/invalid.gd").read_text()
DECIMAL_FIELDS = ("project_device", "project_inode", "script_id", "editor_id", "buffer_id",
                  "source_length", "version", "saved_version", "script_base_id")


def property_projection(document):
    keys = ("name", "type", "hint", "hint_string", "usage", "class_name")
    return sorted(({key: item[key] for key in keys} for item in document["properties"]),
                  key=lambda item: item["name"].encode("utf-8"))


def method_names(document):
    return sorted((item["name"] for item in document["methods"]), key=lambda name: name.encode("utf-8"))


def source_free_document(document):
    if not document.get("associated"):
        return {"associated": False, "matches": document.get("matches")}
    keys = ("script_id", "editor_id", "buffer_id", "version", "saved_version", "dirty",
            "resource_edited", "has_undo", "has_redo", "caret_line", "caret_column",
            "has_selection", "tool", "script_base_id")
    return {key: document[key] for key in keys} | {
        "resource_sha256": sha(document["R"]), "buffer_sha256": sha(document["B"]),
        "compiled_properties_sha256": sha(json.dumps(property_projection(document), sort_keys=True)),
        "compiled_methods_sha256": sha(json.dumps(method_names(document)))}
