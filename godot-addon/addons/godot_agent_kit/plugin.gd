@tool
extends EditorPlugin

const BridgeScript = preload("res://addons/godot_agent_kit/bridge.gd")
const ExportGuardScript = preload("res://addons/godot_agent_kit/export_guard.gd")
const NATIVE_EXTENSION := "res://addons/godot_agent_kit/native/script_edit.gdextension"
const NATIVE_LIBRARY := "res://addons/godot_agent_kit/native/libscript_edit.macos.arm64.dylib"

var _bridge: Node
var _export_guard: EditorExportPlugin
var _native: Dictionary = {}


func _load_native() -> void:
	# .gdignore prevents automatic discovery/export-registry references.
	# An unbuilt checkout must retain observation without missing-library errors.
	if not Engine.is_editor_hint() or not OS.has_feature("macos") or not OS.has_feature("arm64"):
		return
	if FileAccess.file_exists(NATIVE_EXTENSION) and FileAccess.file_exists(NATIVE_LIBRARY) \
			and not GDExtensionManager.is_extension_loaded(NATIVE_EXTENSION):
		GDExtensionManager.load_extension(NATIVE_EXTENSION)


func _native_api() -> Dictionary:
	if not Engine.has_meta("godot_agent_kit_native_validation"):
		return {}
	var candidate: Variant = Engine.get_meta("godot_agent_kit_native_validation")
	if not (candidate is Dictionary):
		return {}
	for operation in ["configure", "close", "validate", "api_revision", "build_id"]:
		if not candidate.has(operation) or not (candidate[operation] is Callable) \
				or not candidate[operation].is_custom() or not candidate[operation].is_valid():
			return {}
	if candidate["api_revision"].call() != 1:
		return {}
	var build_id: Variant = candidate["build_id"].call()
	if not (build_id is String) or build_id.length() != 64 or not build_id.is_valid_hex_number() \
			or build_id != build_id.to_lower():
		return {}
	return candidate


func _enter_tree() -> void:
	_export_guard = ExportGuardScript.new()
	add_export_plugin(_export_guard)
	_load_native()
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
	else:
		_native = _native_api()
		if not _native.is_empty() and not _native["configure"].call(_bridge.get("_session")):
			_native = {}


func _exit_tree() -> void:
	var api := _native if not _native.is_empty() else _native_api()
	if not api.is_empty():
		api["close"].call()
	_native = {}
	if _bridge != null:
		_bridge.stop()
		_bridge.free()
		_bridge = null
	if _export_guard != null:
		remove_export_plugin(_export_guard)
		_export_guard = null
