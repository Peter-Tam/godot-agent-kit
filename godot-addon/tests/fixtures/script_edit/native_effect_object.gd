@tool
extends RefCounted

func _get(property: StringName) -> Variant:
	var control := OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if control.begins_with("/"):
		var file := FileAccess.open(control.path_join("excluded_getter_was_called"), FileAccess.WRITE)
		if file != null:
			file.store_string(String(property))
			file.close()
	return null

func _to_string() -> String:
	var control := OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if control.begins_with("/"):
		var file := FileAccess.open(control.path_join("excluded_stringify_was_called"), FileAccess.WRITE)
		if file != null:
			file.store_string("diagnostic object conversion invoked")
			file.close()
	return "fixture effect object"
