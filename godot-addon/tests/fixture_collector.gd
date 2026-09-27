@tool
extends "res://addons/godot_agent_kit/observation.gd"

var restriction := ""

func _scan(script_editor: ScriptEditor) -> Dictionary:
	var actual := super._scan(script_editor)
	if restriction == "withhold_association":
		actual.unique = false
	elif restriction == "withhold_open":
		# Hide only the target association and completeness of enumeration.
		# Neither an open tab nor absence from tabs can then prove its state.
		actual.unique = false
		actual.index = -1
		actual.script = null
	return actual


func _read_source(authority: String, source: String, start: int, witness: Dictionary) -> Dictionary:
	if (authority == "R" and restriction == "withhold_resource") \
			or (authority == "B" and restriction == "withhold_buffer"):
		return _source_unavailable(authority,
			"resource_unreadable" if authority == "R" else "buffer_unreadable")
	return super._read_source(authority, source, start, witness)



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
