@tool
extends "res://addons/fixture_driver/open_fixture_driver.gd"

# Private owned controls, never a product close transport or success reducer.
const CLOSE_TARGET := "res://scripts/subject.gd"
const CLOSE_CURRENT := "res://scripts/other.gd"
const CLOSE_BACKGROUND := "res://scripts/close/background.gd"
const CLOSE_CONTROL_LIMIT := 4 * 1024 * 1024
var _close_target_ref: GDScript
var _close_original_script_id := 0
var _close_refs: Array[Resource] = []
var _close_callback_mode := ""
var _close_callback_result := {}
var _close_callback_count := 0
var _close_events: Array[Dictionary] = []
var _close_connections: Array[Dictionary] = []
var _close_files := {}
var _close_settings := {}

class CloseWaitWitness extends RefCounted:
	signal completed(result: Dictionary)
	var done := false
	var result := {}

func _close_wait_start(owner: Node, request_id: String, witness: CloseWaitWitness) -> void:
	witness.result = await owner.wait_for_completion(request_id)
	witness.done = true
	witness.completed.emit(witness.result)

func _enter_tree() -> void:
	super._enter_tree()
	Engine.set_meta("godot_agent_kit_close_fixture_callback", _close_entered)

func _exit_tree() -> void:
	Engine.remove_meta("godot_agent_kit_close_fixture_callback")
	_close_disconnect_observers()
	for key in _close_settings:
		EditorInterface.get_editor_settings().set_setting(key, _close_settings[key])
	_close_target_ref = null
	_close_refs.clear()
	super._exit_tree()

func _close_owner() -> Node:
	var bridge := _open_bridge()
	return bridge.get_parent().get_node_or_null("GodotAgentKitScriptClose") if bridge != null else null

func _close_path(path: String) -> bool:
	return path in [CLOSE_TARGET, CLOSE_CURRENT] or (path.begins_with("res://scripts/close/") and path.ends_with(".gd") and not path.contains(".."))

func _close_document(path: String) -> Dictionary:
	return _open_document(path)

func _close_state() -> Dictionary:
	var bridge := _open_bridge()
	var editor := EditorInterface.get_script_editor()
	var cached := ResourceLoader.get_cached_ref(CLOSE_TARGET)
	var current := editor.get_current_script()
	var documents := []
	var paths := []
	for script in editor.get_open_scripts():
		var path: String = script.resource_path if script != null else ""
		paths.append(path)
		documents.append(_close_document(path))
	var context := _open_effective_context()
	context.idle_parse_delay = EditorInterface.get_editor_settings().get_setting("text_editor/completion/idle_parse_delay")
	context.idle_parse_error_delay = EditorInterface.get_editor_settings().get_setting("text_editor/completion/idle_parse_delay_with_errors_found")
	context.sort_scripts = EditorInterface.get_editor_settings().get_setting("text_editor/script_list/sort_scripts_by")
	var active: Dictionary = bridge.get("_active") if bridge != null else {}
	return {"target": _close_document(CLOSE_TARGET), "current": _close_document(CLOSE_CURRENT),
		"background": _close_document(CLOSE_BACKGROUND), "documents": documents, "open_paths": paths,
		"selection": current.resource_path if current != null else "",
		"cached_id": String.num_uint64(cached.get_instance_id()) if cached != null else "",
		"cached_R": cached.get_source_code() if cached is GDScript else null,
		"cached_edited": EditorInterface.is_object_edited(cached) if cached != null else null,
		"original_target_alive": is_instance_id_valid(_close_original_script_id),
		"target_retained": _close_target_ref != null, "slot_busy": not active.is_empty(),
		"owner_matches": active.get("operation_owner") == _close_owner(),
		"session_id": bridge.get("_session") if bridge != null else "",
		"loader_calls": _open_loader.calls, "effective_context": context,
		"pending_node_payload_id": _open_payload_id(),
		"events": _close_events.duplicate(true)}

func _close_disconnect_observers() -> void:
	for item in _close_connections:
		var object: Object = instance_from_id(item.id.to_int())
		if is_instance_valid(object) and object.is_connected(item.signal, item.callback):
			object.disconnect(item.signal, item.callback)
	_close_connections.clear()

func _close_observe() -> void:
	_close_disconnect_observers()
	_close_events.clear()
	var editor := EditorInterface.get_script_editor()
	var visit := _close_visit
	if editor.has_signal("editor_script_changed"):
		editor.connect("editor_script_changed", visit)
		_close_connections.append({"id": String.num_uint64(editor.get_instance_id()), "signal": "editor_script_changed", "callback": visit})
	for base in editor.get_open_script_editors():
		if base.has_signal("edited_script_changed"):
			var callback := _close_completion.bind(String.num_uint64(base.get_instance_id()))
			base.connect("edited_script_changed", callback)
			_close_connections.append({"id": String.num_uint64(base.get_instance_id()), "signal": "edited_script_changed", "callback": callback})

func _close_visit(script: Script) -> void:
	if _close_events.size() < 256:
		_close_events.append({"event": "visit", "script_id": String.num_uint64(script.get_instance_id()) if script != null else "0", "tick_us": str(Time.get_ticks_usec())})

func _close_completion(editor_id: String) -> void:
	if _close_events.size() < 256:
		_close_events.append({"event": "completion", "editor_id": editor_id, "tick_us": str(Time.get_ticks_usec())})

func _close_release_target_refs() -> void:
	_close_target_ref = null
	_cached_subject = null
	_prepared_subject_buffer = null
	for index in range(_open_refs.size() - 1, -1, -1):
		if _open_refs[index].resource_path == CLOSE_TARGET: _open_refs.remove_at(index)
	for index in range(_close_refs.size() - 1, -1, -1):
		if _close_refs[index].resource_path == CLOSE_TARGET: _close_refs.remove_at(index)

func _close_human(path: String, mutation: String) -> bool:
	if not _close_path(path): return false
	var doc := _close_document(path)
	if mutation == "open": return _prepare(path).get("ready", false)
	if mutation == "reopen" and doc.get("matches", -1) == 0:
		return _prepare(path).get("ready", false)
	if not doc.get("associated", false): return false
	var editor := EditorInterface.get_script_editor()
	var script: GDScript = editor.get_open_scripts()[doc.index]
	var buffer := editor.get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
	match mutation:
		"dirty_equal":
			buffer.begin_complex_operation()
			buffer.insert_text("# CLOSE_HUMAN_EARLIER\n", 0, 0)
			buffer.end_complex_operation()
		"dirty_different":
			buffer.begin_complex_operation()
			buffer.insert_text("var = # CLOSE_HUMAN_CONFLICT\n", 1, 0)
			buffer.end_complex_operation()
		"dirty_disk_equal", "same_text":
			buffer.begin_complex_operation()
			buffer.insert_text("# CLOSE_TRANSIENT_HISTORY\n", 0, 0)
			buffer.end_complex_operation()
			buffer.begin_complex_operation()
			buffer.remove_text(0, 0, 1, 0)
			buffer.end_complex_operation()
		"undo": buffer.undo()
		"redo": buffer.redo()
		"resource_source": script.set_source_code("extends RefCounted\n# CLOSE_RESOURCE_CONFLICT\n")
		"resource_edited":
			var source := script.get_source_code()
			script.source_code = source + "# CLOSE_RESOURCE_EDITED\n"
			script.source_code = source
		"select": EditorInterface.edit_script(script)
		"close": return editor.close_file(path) == OK
		"reopen":
			if editor.close_file(path) != OK: return false
			return _prepare(path).get("ready", false)
		"replace":
			var replacement := GDScript.new()
			replacement.set_source_code(doc.R)
			replacement.reload(false)
			if editor.close_file(path) != OK: return false
			replacement.take_over_path(path)
			_close_refs.append(replacement)
			EditorInterface.edit_script(replacement)
		_: return false
	return true

func _close_config(setting: String, value: Variant) -> bool:
	match setting:
		"warning":
			if typeof(value) not in [TYPE_INT, TYPE_FLOAT] or not is_finite(float(value)) or value != floor(float(value)) or value < 0 or value > 2: return false
			ProjectSettings.set_setting("debug/gdscript/warnings/unused_variable", int(value))
		"autoload":
			if typeof(value) != TYPE_BOOL: return false
			ProjectSettings.set_setting("autoload/CloseFixtureAutoload", "*res://scripts/close/autoload.gd" if value == true else null)
		"external_editor", "idle_parse_delay", "idle_parse_error_delay", "sort_scripts":
			if setting == "external_editor" and typeof(value) != TYPE_BOOL: return false
			if setting == "sort_scripts":
				if typeof(value) not in [TYPE_INT, TYPE_FLOAT] or not is_finite(float(value)) or value != floor(float(value)) or value < 0 or value > 2: return false
				value = int(value)
			if setting in ["idle_parse_delay", "idle_parse_error_delay"] and (typeof(value) not in [TYPE_INT, TYPE_FLOAT] or not is_finite(float(value)) or value < 0 or value > 60): return false
			var keys := {"external_editor": "text_editor/external/use_external_editor", "idle_parse_delay": "text_editor/completion/idle_parse_delay",
				"idle_parse_error_delay": "text_editor/completion/idle_parse_delay_with_errors_found", "sort_scripts": "text_editor/script_list/sort_scripts_by"}
			var key: String = keys[setting]
			var settings := EditorInterface.get_editor_settings()
			if not _close_settings.has(key): _close_settings[key] = settings.get_setting(key)
			settings.set_setting(key, value)
		_: return false
	return true

func _close_file(path: String, mutation: String) -> bool:
	if not _close_path(path): return false
	if not _close_files.has(path):
		var original := FileAccess.open(path, FileAccess.READ)
		if original == null: return false
		_close_files[path] = original.get_buffer(original.get_length())
		original.close()
	if mutation == "remove": return DirAccess.remove_absolute(ProjectSettings.globalize_path(path)) == OK
	if mutation == "replace_same":
		if DirAccess.rename_absolute(ProjectSettings.globalize_path(path), ProjectSettings.globalize_path(path + ".original")) != OK: return false
	elif mutation not in ["source", "restore"]: return false
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null: return false
	if mutation == "source": file.store_string("extends RefCounted\n# CLOSE_DISK_CHANGED\n")
	else: file.store_buffer(_close_files[path])
	file.close()
	return true

func _close_entered(request_id: String, stage: String) -> void:
	var owner := _close_owner()
	var bridge := _open_bridge()
	var current := _close_document(CLOSE_CURRENT)
	var current_script: Script
	var current_editor: ScriptEditorBase
	var current_buffer: CodeEdit
	if current.get("associated", false):
		current_script = EditorInterface.get_script_editor().get_open_scripts()[current.index]
		current_editor = EditorInterface.get_script_editor().get_open_script_editors()[current.index]
		current_buffer = current_editor.get_base_editor() as CodeEdit
	_close_callback_count += 1
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	_close_callback_result = {"request_id": request_id, "stage": stage, "count": _close_callback_count,
		"before": _close_state(), "owner_id": String.num_uint64(owner.get_instance_id()) if owner != null else "0"}
	if api.has("close_fixture_state"): _close_callback_result.retained_before = api.close_fixture_state.call(request_id)
	match _close_callback_mode:
		"cancel": _close_callback_result.result = owner.cancel(request_id)
		"finish": _close_callback_result.result = owner.finish(request_id)
		"stop": bridge.stop()
		"disable": EditorInterface.set_plugin_enabled(PRODUCT, false)
		"compete":
			_close_callback_result.result = owner.inspect(CLOSE_TARGET, {"request_id": "close-entered-competitor", "session_id": bridge.get("_session"), "expiry_tick_us": str(Time.get_ticks_usec() + 9000000)})
		"target_typing": _close_callback_result.mutated = _close_human(CLOSE_TARGET, "dirty_different")
		"current_typing": _close_callback_result.mutated = _close_human(CLOSE_CURRENT, "dirty_equal")
	_close_callback_result.after = _close_state()
	_close_callback_result.current_reference_alive = is_instance_valid(current_script) and \
		is_instance_valid(current_editor) and is_instance_valid(current_buffer) and \
		String.num_uint64(current_script.get_instance_id()) == current.script_id and \
		String.num_uint64(current_editor.get_instance_id()) == current.editor_id and \
		String.num_uint64(current_buffer.get_instance_id()) == current.buffer_id
	var active: Dictionary = bridge.get("_active") if is_instance_valid(bridge) else {}
	_close_callback_result.retained_slot_busy = not active.is_empty()
	_close_callback_result.retained_owner_matches = active.get("operation_owner") == owner
	_close_callback_result.owner_reference_alive = is_instance_valid(owner)
	if api.has("close_fixture_state"): _close_callback_result.retained_after = api.close_fixture_state.call(request_id)

func _process(_delta: float) -> void:
	var path := _control.path_join("request.json")
	if not FileAccess.file_exists(path): return
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null: return
	if file.get_length() > CLOSE_CONTROL_LIMIT:
		file.close()
		return
	var raw := file.get_buffer(file.get_length())
	file.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw: return
	var request: Variant = JSON.parse_string(text)
	if request is Dictionary: _dispatch_close_request(request)

func _dispatch_close_request(request: Dictionary) -> void:
	if not String(request.get("action", "")).begins_with("close_"):
		super._dispatch_open_request(request)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id: return
	_last_id = id
	var response := {"id": id, "action": request.action, "ok": true}
	var owner := _close_owner()
	var bridge := _open_bridge()
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	var request_id: String = request.get("request_id", "")
	match String(request.action):
		"close_info":
			response.close_installed = owner != null and api.has_all(["close_inspect", "close_prepare", "close_advance", "close_status", "close_recheck", "close_verify", "close_finish", "close_abort", "close_expire"])
			response.api_revision = api.api_revision.call() if api.has("api_revision") else 0
			response.build_id = api.build_id.call() if api.has("build_id") else ""
			response.fixture_faults = api.has("close_fixture_fault")
			response.session_id = bridge.get("_session") if bridge != null else ""
			response.editor_tick_us = str(Time.get_ticks_usec())
		"close_fixture_activate":
			var prefix := "res://addons/godot_agent_kit/native/"
			var manifest: Variant = JSON.parse_string(FileAccess.get_file_as_string(prefix + "build-manifest.json"))
			var library := prefix + "libeditor_integration.macos.arm64.dylib"
			response.ok = manifest is Dictionary and manifest.get("fixture_only") == true and \
				manifest.get("native_api_revision") == 3 and manifest.get("native_family") == "editor_integration" and \
				manifest.get("native_library_sha256") == FileAccess.get_sha256(library) and \
				owner != null and bridge != null and api.has_all(["close_fixture_fault", "close_fixture_state", "api_revision", "build_id"]) and \
				api.api_revision.call() == 3 and api.build_id.call() == manifest.get("native_build_id")
		"close_setup":
			_close_release_target_refs()
			for path in request.get("paths", [CLOSE_TARGET, CLOSE_CURRENT, CLOSE_BACKGROUND]):
				if typeof(path) != TYPE_STRING or not _close_path(path):
					response.ok = false
					break
				if path == CLOSE_TARGET and request.get("replacement_target", false):
					# Keep the compiler's ordinary cached Resource distinct from the
					# newly bound document. Its last editor-owned reference can then
					# expire naturally; no cache is cleared after the guarded close.
					var compiler_cached := load(path) as GDScript
					if compiler_cached == null:
						response.ok = false
						break
					var source := GDScript.new()
					source.set_source_code(FileAccess.get_file_as_string(path))
					source.take_over_path(path)
					source.reload(false)
					EditorInterface.edit_script(source)
					response.ok = response.ok and _document(path).get("associated", false)
				else:
					response.ok = response.ok and _prepare(path).get("ready", false)
			var selected: String = request.get("selected", CLOSE_TARGET)
			if not selected.is_empty(): response.ok = response.ok and _close_human(selected, "select")
			EditorInterface.set_main_screen_editor("Script")
			if request.get("retain_target", false): _close_target_ref = ResourceLoader.get_cached_ref(CLOSE_TARGET) as GDScript
			else: _close_release_target_refs()
			var target := _close_document(CLOSE_TARGET)
			_close_original_script_id = EditorInterface.get_script_editor().get_open_scripts()[target.index].get_instance_id() if target.get("associated", false) else 0
			_close_observe()
			response.state = _close_state()
		"close_state": response.state = _close_state()
		"close_document": response.document = _close_document(request.get("path", CLOSE_TARGET))
		"close_human":
			response.before = _close_state()
			response.ok = _close_human(request.get("path", CLOSE_CURRENT), request.get("mutation", ""))
			response.state = _close_state()
		"close_config":
			response.ok = _close_config(request.get("setting", ""), request.get("value"))
			response.state = _close_state()
		"close_file":
			response.ok = _close_file(request.get("path", CLOSE_TARGET), request.get("mutation", ""))
			response.state = _close_state()
		"close_refs":
			_close_release_target_refs()
			if request.get("retain_target", false): _close_target_ref = ResourceLoader.get_cached_ref(CLOSE_TARGET) as GDScript
			response.state = _close_state()
		"close_idle":
			var frames: int = request.get("frames", 8)
			response.ok = frames >= 0 and frames <= 600
			if response.ok:
				for frame in frames: await get_tree().process_frame
			response.state = _close_state()
		"close_arm":
			_close_callback_mode = request.get("callback", "")
			response.ok = _close_callback_mode in ["", "cancel", "finish", "stop", "disable", "compete", "target_typing", "current_typing"]
			_close_callback_result = {}
			_close_callback_count = 0
		"close_callback_witness": response.callback = _close_callback_result
		"close_collect":
			var collector := OpenObservation.new()
			response.sample = collector.collect(bridge.get("_session") if bridge != null else "", ProjectSettings.globalize_path("res://").trim_suffix("/"), request.get("path", CLOSE_TARGET))
			response.recheck = collector.recheck()
			collector.clear()
		"close_inspect":
			response.ok = owner != null
			if response.ok: response.result = owner.inspect(request.get("path", CLOSE_TARGET), request.get("correlation", {}))
		"close_prepare":
			response.ok = owner != null
			if response.ok: response.result = owner.prepare(request_id, request.get("captured_source", ""), request.get("capture_and_basis", {}))
		"close_advance":
			response.ok = owner != null
			if response.ok: response.result = owner.advance(request_id, request.get("guard_sha256", ""), request.get("receipt_bindings", []))
			response.callback = _close_callback_result
		"close_status":
			response.ok = owner != null
			if response.ok: response.result = owner.status(request_id)
		"close_recheck":
			response.ok = owner != null
			if response.ok: response.result = owner.recheck(request_id, request.get("purpose", ""))
		"close_verify":
			response.ok = owner != null
			if response.ok: response.result = owner.verify(request_id, request.get("purpose", ""))
		"close_wait":
			response.ok = owner != null
			if response.ok: response.result = await owner.wait_for_completion(request_id)
		"close_wait_duplicate":
			response.ok = owner != null
			if response.ok:
				var witness := CloseWaitWitness.new()
				_close_wait_start(owner, request_id, witness)
				await get_tree().process_frame
				var second: Dictionary = await owner.wait_for_completion(request_id)
				var first: Dictionary = witness.result
				if not witness.done: first = await witness.completed
				response.result = {"first": first, "second": second}
		"close_finish":
			response.ok = owner != null
			if response.ok: response.result = owner.finish(request_id)
		"close_abort":
			response.ok = owner != null
			if response.ok: response.result = owner.cancel(request_id)
		"close_cancel_owned":
			response.ok = owner != null
			if response.ok: owner.cancel_owned()
		"close_lifetime":
			response.before = _close_state()
			match String(request.get("control", "")):
				"stop", "session":
					response.ok = bridge != null
					if response.ok: bridge.stop()
				"disable": EditorInterface.set_plugin_enabled(PRODUCT, false)
				"enable": EditorInterface.set_plugin_enabled(PRODUCT, true)
				"cancel_owned":
					response.ok = owner != null
					if response.ok: owner.cancel_owned()
				_: response.ok = false
			response.state = _close_state()
		"close_renew_visit":
			var editor := EditorInterface.get_script_editor()
			var script := editor.get_current_script()
			var status: Dictionary = owner.status(request_id) if owner != null else {}
			var native: Dictionary = status.get("native", {})
			response.ok = api.has("close_fixture_fault") and owner != null and \
				native.get("entered", false) and native.get("continuation", {}).get("state") == "completed" and \
				script != null and script.resource_path == CLOSE_CURRENT and \
				_close_document(CLOSE_CURRENT).get("associated", false)
			if response.ok:
				editor.editor_script_changed.emit(script)
			response.state = _close_state()
		"close_fault":
			var fault: String = request.get("fault", "")
			response.ok = api.has("close_fixture_fault") and fault in ["expire_before_entry", "cancel_at_entry", "disable_at_entry", "fail_after_return", "drop_completion", "premature_completion", "wrong_completion", "callback_entry", "callback_completion", "missing_signal"]
			if response.ok: response.result = api.close_fixture_fault.call(request_id, fault)
		"close_native_witness":
			response.ok = api.has("close_fixture_state")
			if response.ok: response.fixture_state = api.close_fixture_state.call(request_id)
		_: response.ok = false
	_respond(response)
