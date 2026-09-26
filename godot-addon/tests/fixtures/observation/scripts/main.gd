extends Node

var _frames := 0


func _ready() -> void:
	print("T002_FIXTURE_RUNTIME_READY")


func _process(_delta: float) -> void:
	if get_parent() != get_tree().root or get_tree().root.get_child_count() != 1 or get_child_count() != 0:
		get_tree().quit(2)
		return
	_frames += 1
	if _frames >= 180:
		get_tree().quit()
