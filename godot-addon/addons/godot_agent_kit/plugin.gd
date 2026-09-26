@tool
extends EditorPlugin

const BridgeScript = preload("res://addons/godot_agent_kit/bridge.gd")
const ExportGuardScript = preload("res://addons/godot_agent_kit/export_guard.gd")

var _bridge: Node
var _export_guard: EditorExportPlugin


func _enter_tree() -> void:
	_export_guard = ExportGuardScript.new()
	add_export_plugin(_export_guard)
	if not OS.has_environment("GODOT_AGENT_KIT_REGISTRY"):
		return
	var registry := OS.get_environment("GODOT_AGENT_KIT_REGISTRY")
	if registry.is_empty():
		return
	_bridge = BridgeScript.new()
	_bridge.name = "GodotAgentKitBridge"
	_bridge.add_to_group("godot_agent_kit_session_bridge")
	add_child(_bridge)
	if not _bridge.start(registry):
		_bridge.free()
		_bridge = null


func _exit_tree() -> void:
	if _bridge != null:
		_bridge.stop()
		_bridge.free()
		_bridge = null
	if _export_guard != null:
		remove_export_plugin(_export_guard)
		_export_guard = null
