@tool
extends EditorPlugin

const BridgeScript = preload("res://addons/godot_agent_kit/bridge.gd")
const ExportGuardScript = preload("res://addons/godot_agent_kit/export_guard.gd")
const ScriptEditScript = preload("res://addons/godot_agent_kit/script_edit.gd")
const ScriptOpenScript = preload("res://addons/godot_agent_kit/script_open.gd")
const NATIVE_EXTENSION := "res://addons/godot_agent_kit/native/editor_integration.gdextension"
const NATIVE_LIBRARY := "res://addons/godot_agent_kit/native/libeditor_integration.macos.arm64.dylib"
const NATIVE_MANIFEST := "res://addons/godot_agent_kit/native/build-manifest.json"
const ENGINE_SHA256 := "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf"

var _bridge: Node
var _export_guard: EditorExportPlugin
var _native: Dictionary = {}
var _edit: Node
var _open: Node


func _load_native() -> void:
	# .gdignore prevents automatic discovery/export-registry references.
	# An unbuilt checkout must retain observation without missing-library errors.
	if not Engine.is_editor_hint() or not OS.has_feature("macos") or not OS.has_feature("arm64"):
		return
	if FileAccess.file_exists(NATIVE_EXTENSION) and FileAccess.file_exists(NATIVE_LIBRARY) \
			and not GDExtensionManager.is_extension_loaded(NATIVE_EXTENSION):
		GDExtensionManager.load_extension(NATIVE_EXTENSION)


func _native_api() -> Dictionary:
	if not Engine.has_meta("godot_agent_kit_native"):
		return {}
	var candidate: Variant = Engine.get_meta("godot_agent_kit_native")
	if not (candidate is Dictionary):
		return {}
	for operation in ["configure", "close", "api_revision", "build_id", "edit_inspect",
			"edit_prepare", "edit_advance", "edit_cancel", "edit_expire", "open_inspect",
			"open_prepare", "open_advance", "open_verify", "open_recheck", "open_finish",
			"open_abort", "open_expire"]:
		if not candidate.has(operation) or not (candidate[operation] is Callable) \
				or not candidate[operation].is_custom() or not candidate[operation].is_valid():
			return {}
	if candidate["api_revision"].call() != 2:
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
	var family := _native_api()
	if not family.is_empty():
		_edit = ScriptEditScript.new()
		_edit.name = "GodotAgentKitScriptEdit"
		add_child(_edit)
		_edit.configure(family, null)
		_open = ScriptOpenScript.new()
		_open.name = "GodotAgentKitScriptOpen"
		add_child(_open)
		_open.configure(family, null)
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
		if _edit != null:
			_edit.configure(_native, _bridge)
		if _open != null:
			_open.configure(_native, _bridge)
		var build_id := _matched_build_id(_native)
		_bridge.attach_edit(_edit if not build_id.is_empty() else null, 2 if not build_id.is_empty() else 0, build_id)


func _matched_build_id(candidate: Dictionary) -> String:
	if candidate.is_empty() or not FileAccess.file_exists(NATIVE_MANIFEST):
		return ""
	var manifest_file := FileAccess.open(NATIVE_MANIFEST, FileAccess.READ)
	if manifest_file == null or manifest_file.get_length() > 4096:
		return ""
	var parser := JSON.new()
	if parser.parse(manifest_file.get_as_text()) != OK or not (parser.data is Dictionary):
		return ""
	var manifest: Dictionary = parser.data
	var build_id: Variant = candidate["build_id"].call()
	if manifest.get("fixture_only") != false or manifest.get("generated_from_exact_binary") != true \
			or manifest.get("native_api_revision") != 2 \
			or manifest.get("native_family") != "editor_integration" \
			or manifest.get("native_library") != "libeditor_integration.macos.arm64.dylib" \
			or manifest.get("entry_symbol") != "editor_integration_library_init" \
			or manifest.get("engine_version") != BridgeScript.VERSION \
			or manifest.get("base_commit") != BridgeScript.ENGINE_HASH \
			or manifest.get("engine_sha256") != ENGINE_SHA256 \
			or manifest.get("native_build_id") != build_id \
			or not FileAccess.file_exists(NATIVE_LIBRARY) \
			or FileAccess.get_sha256(NATIVE_LIBRARY) != manifest.get("native_library_sha256") \
			or FileAccess.get_sha256(OS.get_executable_path()) != ENGINE_SHA256:
		return ""
	return build_id


func _exit_tree() -> void:
	if _bridge != null:
		_bridge.stop()
	if _edit != null:
		var entered: bool = _edit.is_active_stage()
		remove_child(_edit)
		if not entered:
			_edit.free()
		_edit = null
	if _open != null:
		var entered: bool = _open.is_active_stage()
		remove_child(_open)
		if not entered:
			_open.free()
		_open = null
	var api := _native if not _native.is_empty() else _native_api()
	if not api.is_empty():
		api["close"].call()
	_native = {}
	if _bridge != null:
		var active: Dictionary = _bridge.get("_active")
		var owner: Variant = active.get("operation_owner")
		if is_instance_valid(owner) and owner.call("is_active_stage"):
			# Its returning owner releases and frees the detached bridge only
			# after native entered-call cleanup. No replacement slot is admitted.
			remove_child(_bridge)
		else:
			_bridge.free()
		_bridge = null
	if _export_guard != null:
		remove_export_plugin(_export_guard)
		_export_guard = null
