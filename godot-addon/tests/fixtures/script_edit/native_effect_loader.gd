@tool
extends ResourceFormatLoader

var armed := false

# Fixture-owned independent effect witness; this must never be entered by B.
func _mark_effect() -> void:
	if not armed or not Thread.is_main_thread():
		return
	var control := OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if control.begins_with("/"):
		var file := FileAccess.open(control.path_join("excluded_loader_was_called"), FileAccess.WRITE)
		if file != null:
			file.store_string("loader callback invoked")
			file.close()

func _get_recognized_extensions() -> PackedStringArray:
	_mark_effect()
	return PackedStringArray(["native_effect"])

func _get_resource_type(path: String) -> String:
	_mark_effect()
	return "Resource" if path.ends_with(".native_effect") else ""

func _exists(path: String) -> bool:
	_mark_effect()
	return path.ends_with(".native_effect")

func _load(path: String, _original_path: String, _use_sub_threads: bool, _cache_mode: int) -> Variant:
	_mark_effect()
	return Resource.new() if path.ends_with(".native_effect") else null
