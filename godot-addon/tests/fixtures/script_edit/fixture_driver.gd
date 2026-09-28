@tool
extends "res://fixture_driver.gd"

# All controls below belong only to the owned fixture. The product addon never
# receives an action, source string, test session, or synthetic validation result.
const NATIVE_META := "godot_agent_kit_native_validation"
const NATIVE_CONTROL_LIMIT := 540000

var _effect_loader: ResourceFormatLoader
var _effect_object: RefCounted

func _enter_tree() -> void:
	super._enter_tree()
	_effect_loader = preload("res://addons/fixture_driver/native_effect_loader.gd").new()

func _exit_tree() -> void:
	if _effect_object != null:
		GDScript.gdscript_validation_test_clear_object()
		_effect_object = null
	_effect_loader = null
	super._exit_tree()

func _process(delta: float) -> void:
	var request_file := _control.path_join("request.json")
	if not FileAccess.file_exists(request_file):
		return
	var input := FileAccess.open(request_file, FileAccess.READ)
	if input == null:
		return
	var size := input.get_length()
	if size > NATIVE_CONTROL_LIMIT:
		input.close()
		return
	var raw := input.get_buffer(size)
	input.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw:
		return
	var parser := JSON.new()
	if parser.parse(text) != OK or typeof(parser.data) != TYPE_DICTIONARY:
		return
	var request: Dictionary = parser.data
	if not String(request.get("action", "")).begins_with("native_"):
		super._process(delta)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id:
		return
	_last_id = id
	var action: String = request.action
	var response := {"id": id, "action": action, "ok": true}
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	if action == "native_info":
		response.api_installed = api.has_all(["api_revision", "build_id", "configure", "close", "validate"])
		if response.api_installed:
			response.api_revision = api.api_revision.call()
			response.build_id = api.build_id.call()
		response.tool_cached = ResourceLoader.has_cached("res://scripts/native/cold/tool_initializer.gd")
	elif not api.has_all(["api_revision", "build_id", "configure", "close", "validate"]):
		response.ok = false
	elif action == "native_idle":
		# History preparation queues ordinary editor validation. Observe its
		# actual Resource convergence before measuring B's non-interference.
		var deadline := Time.get_ticks_usec() + 5000000
		while true:
			var document := _document("res://scripts/subject.gd")
			if document.get("associated", false) and document.R == document.B:
				break
			if Time.get_ticks_usec() >= deadline:
				response.ok = false
				break
			await get_tree().process_frame
	elif action == "native_close":
		api.close.call()
	elif action == "native_configure":
		response.configured = api.configure.call(request.get("session_id", ""))
	elif action == "native_context":
		var mapping: String = request.get("mapping", "")
		if mapping not in ["context_node", "context_other", "context_invalid"]:
			response.ok = false
		else:
			ProjectSettings.set_setting("autoload/ValidationDependency",
				"*res://scripts/native/" + mapping + ".gd")
			response.mapping = mapping
	elif action == "native_effect_install":
		_effect_object = preload("res://addons/fixture_driver/native_effect_object.gd").new()
		response.installed = GDScript.gdscript_validation_test_set_object(_effect_object)
	elif action == "native_effect_remove":
		response.removed = GDScript.gdscript_validation_test_clear_object()
		_effect_object = null
	elif action in ["native_validate", "native_reentry", "native_dependency_barrier", "native_cycle_validation"]:
		var path: String = request.get("source_path", "res://scripts/subject.gd")
		var doc := _document("res://scripts/subject.gd")
		if not doc.get("associated", false):
			response.ok = false
		else:
			var source: String = request.get("source", "")
			if request.get("generate", "") == "exact":
				source = "extends RefCounted\n# " + "é".repeat(262133) + "x"
			elif request.get("generate", "") == "over":
				source = "extends RefCounted\n# " + "x".repeat(524268)
			var document := {"resource_instance_id": doc.script_id,
				"editor_instance_id": doc.editor_id, "buffer_instance_id": doc.buffer_id}
			if request.get("wrong_document", false):
				document.buffer_instance_id = "0"
			var correlation := {"request_id": request.get("request_id", ""),
				"session_id": request.get("session_id", ""),
				"purpose": request.get("purpose", "preflight"),
				"document": document,
				"expected_source_sha256": request.get("expected_source_sha256", source.sha256_text())}
			if request.get("wrong_hash", false):
				correlation.expected_source_sha256 = "0".repeat(64)
			ResourceLoader.add_resource_format_loader(_effect_loader, true)
			_effect_loader.set("armed", true)
			if action in ["native_reentry", "native_dependency_barrier"]:
				if not Engine.has_meta("godot_agent_kit_fixture_reentry"):
					response.ok = false
				else:
					var fixture_reader: Callable = Engine.get_meta("godot_agent_kit_fixture_reentry")
					if action == "native_dependency_barrier":
						response.result = fixture_reader.call(source, path, correlation,
							ProjectSettings.globalize_path("res://"), _control)
					else:
						response.result = fixture_reader.call(source, path, correlation,
							ProjectSettings.globalize_path("res://"))
			elif action == "native_cycle_validation":
				response.statuses = []
				response.object_count_before = int(Performance.get_monitor(Performance.OBJECT_COUNT))
				for index in 32:
					correlation.request_id = request.get("request_id", "") + "_" + str(index)
					var result: Dictionary = api.validate.call(source, path, correlation)
					response.statuses.append(result.get("status", "missing"))
				response.object_count_after = int(Performance.get_monitor(Performance.OBJECT_COUNT))
			elif request.get("wrong_thread", false):
				var thread := Thread.new()
				var error := thread.start(func() -> Dictionary:
					return api.validate.call(source, path, correlation))
				response.ok = error == OK
				if response.ok:
					response.result = thread.wait_to_finish()
			else:
				response.result = api.validate.call(source, path, correlation)
			_effect_loader.set("armed", false)
			ResourceLoader.remove_resource_format_loader(_effect_loader)
			response.actual_input_sha256 = source.sha256_text()
			response.actual_input_utf8_bytes = source.to_utf8_buffer().size()
	else:
		response.ok = false
	_respond(response)
