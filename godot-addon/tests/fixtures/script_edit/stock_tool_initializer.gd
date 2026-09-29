@tool
extends RefCounted

static var initialized := _mark_initializer()

static func _mark_initializer() -> int:
	var output := FileAccess.open("res://runtime-marker-stock-initializer.txt", FileAccess.WRITE)
	if output != null:
		output.store_string("STOCK_INITIALIZER_EXECUTED")
		output.close()
	return 1

func _init() -> void:
	var output := FileAccess.open("res://runtime-marker-stock-constructor.txt", FileAccess.WRITE)
	if output != null:
		output.store_string("STOCK_CONSTRUCTOR_EXECUTED")
		output.close()
