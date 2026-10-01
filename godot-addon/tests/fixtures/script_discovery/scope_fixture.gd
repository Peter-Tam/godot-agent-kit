@tool
extends "res://addons/fixture_driver/fixture_discovery_scope.gd"

# A negative scope-invalidation control only. Normal acquisition uses the actual
# EditorPaths getter inherited from production; no positive scope is fabricated.
func _settings_directory() -> Variant:
	if _mode() == "changed_data_directory":
		var actual: String = EditorInterface.get_editor_paths().get_project_settings_dir()
		return "res://godot/editor" if actual == "res://.godot/editor" else "res://.godot/editor"
	return super._settings_directory()
