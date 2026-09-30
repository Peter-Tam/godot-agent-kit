@tool
extends RefCounted

static var effect_witness := _mark_effect()

static func _mark_effect() -> int:
	var marker := FileAccess.open("res://opening-unpermitted-effect.txt", FileAccess.WRITE)
	if marker != null:
		marker.store_string("OPEN_UNPERMITTED_EFFECT_EXECUTED")
		marker.close()
	print("OPEN_UNPERMITTED_EFFECT_EXECUTED")
	return 1
