@tool
extends RefCounted

static var INITIALIZER_WITNESS := _mark_effect()

static func _mark_effect() -> int:
	var control := OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if control.begins_with("/"):
		var file := FileAccess.open(control.path_join("excluded_initializer_was_called"), FileAccess.WRITE)
		if file != null:
			file.store_string("static initializer executed")
			file.close()
	return 1

func _init() -> void:
	var control := OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if control.begins_with("/"):
		var file := FileAccess.open(control.path_join("excluded_constructor_was_called"), FileAccess.WRITE)
		if file != null:
			file.store_string("script constructor executed")
			file.close()
