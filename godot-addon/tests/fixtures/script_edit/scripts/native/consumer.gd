extends SceneTree

func _initialize() -> void:
	var target := preload("res://scripts/subject.gd").new()
	print("NATIVE_RUNTIME_VALUE=", target.value())
	quit()
