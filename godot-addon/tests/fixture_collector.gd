@tool
extends "res://addons/godot_agent_kit/observation.gd"

var restriction := ""

func _scan(script_editor: ScriptEditor) -> Dictionary:
	var actual := super._scan(script_editor)
	if restriction == "withhold_association":
		actual.unique = false
	return actual



func _unsaved(script_editor: ScriptEditor, paths: Array[String]) -> Dictionary:
	var actual := super._unsaved(script_editor, paths)
	if restriction == "withhold_dirty":
		# Erase attribution only; never assert a made-up dirty or clean state.
		actual.known = false
	elif restriction == "unattributed_global" and not actual.paths.is_empty():
		# Retain the actual Godot indication, but remove the association map.
		# This cannot create a positive dirty observation for the target.
		actual = super._unsaved(script_editor, [])
	return actual
