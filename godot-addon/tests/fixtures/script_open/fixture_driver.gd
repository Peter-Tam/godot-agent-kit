@tool
extends "res://addons/fixture_driver/native_fixture_driver.gd"

# Owned preparation and independent witnesses only. All lifecycle entry goes
# through the installed addon owner and its real bridge operation slot.
const OpenObservation = preload("res://addons/godot_agent_kit/observation.gd")
const OPEN_CONTROL_LIMIT := 2200000
const OPEN_TARGET := "res://scripts/subject.gd"
const OPEN_CURRENT := "res://scripts/other.gd"
const OPEN_BACKGROUND := "res://scripts/open/background.gd"

var _open_refs: Array[Resource] = []
var _open_loader: WitnessLoader
var _open_callback_mode := ""
var _open_callback_result := {}
var _open_original_settings := {}
var _open_pending_node: Node
var _open_pending_resource: Resource
var _open_hold_stage := ""
var _open_hold_response := ""
var _open_hold_announced := false
var _open_fault_stage := ""
var _open_fault_name := ""
var _open_held_peer := {}

func _open_publish_event(stage: String, request_id: String) -> bool:
	var event := FileAccess.open(_control.path_join("open-event.json"), FileAccess.WRITE)
	if event == null: return false
	event.store_string(JSON.stringify({"stage": stage, "request_id": request_id}))
	event.close()
	return true

func open_barrier(stage: String, peer: Dictionary) -> bool:
	_open_held_peer = peer
	if stage == _open_fault_stage and not _open_fault_name.is_empty():
		var api: Dictionary = Engine.get_meta(NATIVE_META, {})
		if api.has("open_fixture_fault"):
			api.open_fixture_fault.call(peer.request_id, _open_fault_name)
		_open_fault_stage = ""
		_open_fault_name = ""
	if stage != _open_hold_stage: return true
	if not _open_hold_announced:
		_open_hold_announced = _open_publish_event(stage, peer.request_id)
	return false

func open_response_barrier(kind: String, peer: Dictionary, _reply: Dictionary) -> bool:
	_open_retain_cached()
	_open_held_peer = peer
	if kind != _open_hold_response: return true
	if not _open_hold_announced:
		_open_hold_announced = _open_publish_event("response:" + kind, peer.request_id)
	return false


class WitnessLoader extends ResourceFormatLoader:
	var calls := 0
	func _get_recognized_extensions() -> PackedStringArray:
		return PackedStringArray(["gd"])
	func _handles_type(type: StringName) -> bool:
		return type in [&"Script", &"GDScript"]
	func _get_resource_type(path: String) -> String:
		return "GDScript" if path.ends_with(".gd") else ""
	func _recognize_path(path: String, _type: StringName) -> bool:
		return path.ends_with(".gd")
	func _load(path: String, _original_path: String, _use_sub_threads: bool, _cache_mode: int) -> Variant:
		calls += 1
		# Instrument, then delegate to the actual stock loader for owned fixture
		# preparation. Any lifecycle use is independently visible in calls.
		ResourceLoader.remove_resource_format_loader(self)
		var resource := ResourceLoader.load(path)
		ResourceLoader.add_resource_format_loader(self, true)
		return resource

func _enter_tree() -> void:
	super._enter_tree()
	_open_loader = WitnessLoader.new()
	ResourceLoader.add_resource_format_loader(_open_loader, true)
	Engine.set_meta("godot_agent_kit_open_fixture_callback", _open_entered)

func _exit_tree() -> void:
	Engine.remove_meta("godot_agent_kit_open_fixture_callback")
	if _open_loader != null:
		ResourceLoader.remove_resource_format_loader(_open_loader)
	for key in _open_original_settings:
		EditorInterface.get_editor_settings().set_setting(key, _open_original_settings[key])
	_open_refs.clear()
	super._exit_tree()

func _open_bridge() -> Node:
	return get_tree().get_first_node_in_group("godot_agent_kit_session_bridge")

func _open_owner() -> Node:
	var bridge := _open_bridge()
	return bridge.get_parent().get_node_or_null("GodotAgentKitScriptOpen") if bridge != null else null

func _open_retain_cached() -> void:
	# An independent ordinary cache consumer holds an actually published R so
	# native ownership cleanup is not mistaken for rollback/cache non-publication.
	var cached := ResourceLoader.get_cached_ref(OPEN_TARGET)
	if cached != null and not _open_refs.has(cached): _open_refs.append(cached)

func _open_document(path: String) -> Dictionary:
	var doc := _document(path)
	if doc.get("associated", false):
		var script: GDScript = EditorInterface.get_script_editor().get_open_scripts()[doc.index]
		doc.resource_edited = EditorInterface.is_object_edited(script)
		doc.tool = script.is_tool()
		var base := script.get_base_script()
		doc.script_base_id = String.num_uint64(base.get_instance_id()) if base != null else "0"
		doc.properties = script.get_script_property_list()
		doc.methods = script.get_script_method_list()
	return doc

func _open_effective_context() -> Dictionary:
	var prefix := "debug/gdscript/warnings/"
	var levels := {}
	for property in ProjectSettings.get_property_list():
		var key: String = property.get("name", "")
		if not key.begins_with(prefix) or key.trim_prefix(prefix).contains("."): continue
		var warning_name := key.trim_prefix(prefix)
		if warning_name in ["enable", "directory_rules", "exclude_addons",
			"property_used_as_function", "constant_used_as_function", "function_used_as_property"]: continue
		var value: Variant = ProjectSettings.get_setting_with_override(key)
		if typeof(value) == TYPE_INT: levels[warning_name] = value
	var globals := []
	for entry in ProjectSettings.get_global_class_list():
		if entry.get("language") == "GDScript": globals.append(String(entry["class"]))
	globals.sort()
	var autoloads := []
	for property in ProjectSettings.get_property_list():
		var key: String = property.get("name", "")
		if key.begins_with("autoload/"): autoloads.append(key.trim_prefix("autoload/"))
	autoloads.sort()
	return {"warnings": {"enable": ProjectSettings.get_setting_with_override(prefix + "enable"),
		"levels": levels, "directory_rules": ProjectSettings.get_setting_with_override(prefix + "directory_rules")},
		"global_classes": globals, "autoloads": autoloads,
		"external_editor": EditorInterface.get_editor_settings().get_setting("text_editor/external/use_external_editor")}

func _open_state() -> Dictionary:
	var bridge := _open_bridge()
	var cached := ResourceLoader.get_cached_ref(OPEN_TARGET)
	var current := EditorInterface.get_script_editor().get_current_script()
	var paths := []
	for script in EditorInterface.get_script_editor().get_open_scripts():
		paths.append(script.resource_path if script != null else "")
	return {"target": _open_document(OPEN_TARGET), "current": _open_document(OPEN_CURRENT),
		"background": _open_document(OPEN_BACKGROUND), "open_paths": paths,
		"selection": current.resource_path if current != null else "",
		"cached_id": String.num_uint64(cached.get_instance_id()) if cached != null else "",
		"cached_R": cached.get_source_code() if cached is GDScript else null,
		"cached_edited": EditorInterface.is_object_edited(cached) if cached != null else null,
		"slot_busy": bridge != null and not (bridge.get("_active") as Dictionary).is_empty(),
		"owner_matches": bridge != null and (bridge.get("_active") as Dictionary).get("operation_owner") == _open_owner(),
		"session_id": bridge.get("_session") if bridge != null else "",
		"loader_calls": _open_loader.calls,
		"effective_context": _open_effective_context(),
		"pending_node_payload_id": _open_payload_id()}

func _open_payload_id() -> String:
	if not is_instance_valid(_open_pending_node):
		return ""
	var value: Variant = _open_pending_node.get("payload")
	return String.num_uint64(value.get_instance_id()) if value is Object else str(value)

func _open_entered(request_id: String, stage: String) -> void:
	# Entry precedes deliberate endpoint loss/disable. The event is not a
	# completion receipt; survivor witnesses are acquired after the native return.
	var event := FileAccess.open(_control.path_join("open-entered.json"), FileAccess.WRITE)
	if event != null:
		event.store_string(JSON.stringify({"request_id": request_id, "stage": stage}))
		event.close()
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	var owner := _open_owner()
	var bridge := _open_bridge()
	var current := _open_document(OPEN_CURRENT)
	var current_script: Script
	var current_editor: ScriptEditorBase
	var current_buffer: CodeEdit
	if current.get("associated", false):
		current_script = EditorInterface.get_script_editor().get_open_scripts()[current.index]
		current_editor = EditorInterface.get_script_editor().get_open_script_editors()[current.index]
		current_buffer = current_editor.get_base_editor() as CodeEdit
	_open_callback_result = {"stage": stage, "before": _open_state(),
		"retained_before": api.open_fixture_state.call(request_id)}
	_open_retain_cached()
	if _open_callback_mode == "cancel":
		_open_callback_result.terminal = owner.cancel(request_id)
	elif _open_callback_mode == "finish":
		_open_callback_result.terminal = owner.finish(request_id)
	elif _open_callback_mode == "stop":
		bridge.stop()
	elif _open_callback_mode == "disable":
		EditorInterface.set_plugin_enabled(PRODUCT, false)
	elif _open_callback_mode == "compete":
		_open_callback_result.competitor = owner.inspect(OPEN_TARGET,
			{"request_id": "entered-competitor", "session_id": bridge.get("_session"),
			"expiry_tick_us": str(Time.get_ticks_usec() + 9000000)})
	elif _open_callback_mode == "target_typing":
		_open_callback_result.mutated = _open_mutate("dirty_different", OPEN_TARGET)
	elif _open_callback_mode == "current_typing":
		_open_callback_result.mutated = _open_mutate("current_source")
	_open_callback_result.after = _open_state()
	_open_callback_result.retained_after = api.open_fixture_state.call(request_id)
	_open_callback_result.retained_slot_busy = is_instance_valid(bridge) and \
		not (bridge.get("_active") as Dictionary).is_empty()
	_open_callback_result.retained_owner_matches = is_instance_valid(bridge) and \
		(bridge.get("_active") as Dictionary).get("operation_owner") == owner
	_open_callback_result.current_reference_alive = is_instance_valid(current_script) and \
		is_instance_valid(current_editor) and is_instance_valid(current_buffer) and \
		String.num_uint64(current_script.get_instance_id()) == current.script_id and \
		String.num_uint64(current_editor.get_instance_id()) == current.editor_id and \
		String.num_uint64(current_buffer.get_instance_id()) == current.buffer_id

func _open_mutate(mode: String, source_path: String = OPEN_CURRENT) -> bool:
	var doc := _open_document(source_path)
	var script: GDScript = null
	var buffer: CodeEdit = null
	if doc.get("associated", false):
		script = EditorInterface.get_script_editor().get_open_scripts()[doc.index]
		buffer = EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
	match mode:
		"", "none": return true
		"dirty_equal":
			if buffer == null: return false
			buffer.begin_complex_operation()
			buffer.insert_text("# OPEN_HUMAN_EARLIER\n", 0, 0)
			buffer.end_complex_operation()
			# No assignment to R: ordinary editor idle validation must converge it.
		"dirty_different":
			if buffer == null: return false
			buffer.begin_complex_operation()
			buffer.insert_text("var = # OPEN_HUMAN_CONFLICT\n", 1, 0)
			buffer.end_complex_operation()
		"dirty_disk_equal":
			if buffer == null: return false
			buffer.begin_complex_operation()
			buffer.insert_text("# OPEN_TRANSIENT_NATIVE_HISTORY\n", 0, 0)
			buffer.end_complex_operation()
			buffer.begin_complex_operation()
			buffer.remove_text(0, 0, 1, 0)
			buffer.end_complex_operation()
		"target_reopen":
			if EditorInterface.get_script_editor().close_file(OPEN_TARGET) != OK: return false
			return _prepare(OPEN_TARGET).get("ready", false)
		"target_close":
			return EditorInterface.get_script_editor().close_file(OPEN_TARGET) == OK
		"undo":
			if buffer == null: return false
			buffer.undo()
		"redo":
			if buffer == null: return false
			buffer.redo()
		"stale_metadata":
			if buffer == null: return false
			var safe := "extends RefCounted\nfunc value() -> int:\n\treturn 7\n"
			script.set_source_code(safe)
			buffer.text = safe
		"current_source":
			if buffer == null: return false
			buffer.insert_text("# OPEN_CURRENT_CHANGED\n", 0, 0)
		"current_resource":
			if script == null: return false
			script.set_source_code("extends RefCounted\n# OPEN_RESOURCE_CHANGED\n")
		"current_close":
			return EditorInterface.get_script_editor().close_file(OPEN_CURRENT) == OK
		"current_replace":
			if script == null: return false
			var replacement := GDScript.new()
			replacement.set_source_code(doc.R)
			replacement.reload(false)
			if EditorInterface.get_script_editor().close_file(OPEN_CURRENT) != OK: return false
			replacement.take_over_path(OPEN_CURRENT)
			_open_refs.append(replacement)
			EditorInterface.edit_script(replacement)
		"target_cache":
			var added := load(OPEN_TARGET) as GDScript
			if added == null: return false
			_open_refs.append(added)
		"target_cache_source":
			var cached := ResourceLoader.get_cached_ref(OPEN_TARGET) as GDScript
			if cached == null: return false
			cached.set_source_code("extends RefCounted\n# OPEN_CACHE_CHANGED\n")
		"target_cache_edited":
			var cached := ResourceLoader.get_cached_ref(OPEN_TARGET) as GDScript
			if cached == null: return false
			var original := cached.get_source_code()
			cached.source_code = original + "# OPEN_CACHE_PROPERTY_NEGATIVE\n"
			cached.source_code = original
		"target_cache_wrong_type":
			var occupant := Resource.new()
			occupant.set_path(OPEN_TARGET)
			_open_refs.append(occupant)
		"target_cache_replace":
			var replacement := GDScript.new()
			replacement.set_source_code(FileAccess.get_file_as_string(OPEN_TARGET))
			replacement.reload(false)
			replacement.take_over_path(OPEN_TARGET)
			_open_refs.append(replacement)
		"target_human_open":
			return _prepare(OPEN_TARGET).get("ready", false)
		"warning":
			var key := "debug/gdscript/warnings/unused_variable"
			ProjectSettings.set_setting(key, (int(ProjectSettings.get_setting_with_override(key)) + 1) % 3)
		"autoload":
			ProjectSettings.set_setting("autoload/OpenFixtureAutoload", "*res://scripts/open/autoload.gd")
		"external_editor":
			var settings := EditorInterface.get_editor_settings()
			var key := "text_editor/external/use_external_editor"
			if not _open_original_settings.has(key): _open_original_settings[key] = settings.get_setting(key)
			settings.set_setting(key, not bool(settings.get_setting(key)))
		"session":
			var api: Dictionary = Engine.get_meta(NATIVE_META, {})
			api.close.call()
		_: return false
	return true

func _open_pending_drag(undo_drop: bool) -> Dictionary:
	EditorInterface.open_scene_from_path("res://scripts/open/pending.tscn")
	for frame in 4:
		await get_tree().process_frame
	_open_pending_node = EditorInterface.get_edited_scene_root()
	var prepared := _prepare(OPEN_CURRENT)
	if _open_pending_node == null or not prepared.get("ready", false): return {"ok": false}
	EditorInterface.edit_node(_open_pending_node)
	var doc := _open_document(OPEN_CURRENT)
	var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
	_open_pending_resource = load("res://scripts/open/payload.tres")
	if _open_pending_resource == null: return {"ok": false}
	var alt := InputEventKey.new()
	alt.keycode = KEY_ALT
	alt.pressed = true
	Input.parse_input_event(alt)
	await get_tree().process_frame
	buffer.set_caret_line(buffer.get_line_count() - 1)
	buffer.set_caret_column(0)
	var position := buffer.get_rect().get_center()
	var motion := InputEventMouseMotion.new()
	motion.position = buffer.global_position + position
	motion.global_position = motion.position
	Input.parse_input_event(motion)
	buffer.force_drag({"type": "resource", "resource": _open_pending_resource}, Label.new())
	var release := InputEventMouseButton.new()
	release.button_index = MOUSE_BUTTON_LEFT
	release.position = motion.position
	release.global_position = motion.position
	release.pressed = false
	Input.parse_input_event(release)
	await get_tree().process_frame
	alt.pressed = false
	Input.parse_input_event(alt)
	var dragged := _open_document(OPEN_CURRENT)
	if undo_drop: buffer.undo()
	return {"ok": dragged.B != doc.B and dragged.B.contains("@export var payload"),
		"before": doc, "dragged": dragged, "after": _open_document(OPEN_CURRENT),
		"payload_id": _open_payload_id(), "drag_resource_id": String.num_uint64(_open_pending_resource.get_instance_id())}

func _process(delta: float) -> void:
	var path := _control.path_join("request.json")
	if not FileAccess.file_exists(path): return
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null: return
	if file.get_length() > OPEN_CONTROL_LIMIT:
		file.close()
		return
	var raw := file.get_buffer(file.get_length())
	file.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw: return
	var request: Variant = JSON.parse_string(text)
	if typeof(request) != TYPE_DICTIONARY: return
	if not String(request.get("action", "")).begins_with("open_"):
		super._dispatch_native_request(request)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id: return
	_last_id = id
	var response := {"id": id, "action": request.action, "ok": true}
	var owner := _open_owner()
	var bridge := _open_bridge()
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	var request_id: String = request.get("request_id", "")
	match String(request.action):
		"open_info":
			response.open_installed = owner != null and api.has_all(["open_inspect", "open_prepare", "open_advance", "open_verify", "open_recheck", "open_finish", "open_abort", "open_expire"])
			response.api_revision = api.api_revision.call() if api.has("api_revision") else 0
			response.build_id = api.build_id.call() if api.has("build_id") else ""
			response.fixture_faults = api.has_all(["open_fixture_fault", "open_fixture_state"])
			response.session_id = bridge.get("_session") if bridge != null else ""
		"open_witness": response.state = _open_state()
		"open_document":
			response.document = _open_document(request.get("path", OPEN_TARGET))
		"open_select":
			var selected_path: String = request.get("path", "")
			var selected := _open_document(selected_path)
			response.ok = selected_path in [OPEN_CURRENT, OPEN_BACKGROUND] and selected.get("associated", false)
			if response.ok:
				# Existing-document native navigation, not load/reopen or text assignment.
				var selected_script: Script = EditorInterface.get_script_editor().get_open_scripts()[selected.index]
				EditorInterface.edit_script(selected_script)
				EditorInterface.set_main_screen_editor("Script")
				await get_tree().process_frame
				response.state = _open_state()
				response.ok = response.state.selection == selected_path
		"open_callback_witness":
			response.callback = _open_callback_result
			response.plugin_enabled = EditorInterface.is_plugin_enabled(PRODUCT)
		"open_arm":
			_open_hold_stage = request.get("stage", "")
			_open_hold_response = request.get("response_kind", "")
			_open_fault_stage = request.get("native_stage", "")
			_open_fault_name = request.get("native_fault", "")
			_open_callback_mode = request.get("callback", "")
			_open_hold_announced = false
			_open_held_peer = {}
			response.ok = _open_hold_stage.is_empty() or _open_hold_stage in [
				"begin", "prepare", "bind", "compile", "open", "verify:recognition",
				"verify:post_open", "recheck:recognition", "recheck:post_open", "finish", "abort"]
			response.ok = response.ok and (_open_hold_response.is_empty() or _open_hold_response in [
				"open_state", "open_prepared", "open_progress", "open_sample", "open_rechecked"])
		"open_release":
			_open_hold_stage = ""
			_open_hold_response = ""
			_open_hold_announced = false
			_open_callback_mode = ""
		"open_idle":
			# Let released peers, late socket EOF and ordinary editor frames settle.
			# Unlike edit's convergence helper this also observes truly closed targets.
			for frame in 8:
				await get_tree().process_frame
		"open_disconnect":
			response.ok = _open_held_peer.has("socket")
			if response.ok: _open_held_peer.socket.disconnect_from_host()
		"open_fixture_activate":
			var manifest_file := FileAccess.open("res://addons/godot_agent_kit/native/build-manifest.json", FileAccess.READ)
			var manifest: Variant = JSON.parse_string(manifest_file.get_as_text()) if manifest_file != null else null
			var library := "res://addons/godot_agent_kit/native/libeditor_integration.macos.arm64.dylib"
			response.ok = manifest is Dictionary and manifest.get("fixture_only") == true and \
				manifest.get("native_api_revision") == 2 and manifest.get("native_family") == "editor_integration" and \
				manifest.get("native_library_sha256") == FileAccess.get_sha256(library) and \
				owner != null and bridge != null and api.has("open_fixture_fault")
			if response.ok:
				bridge.attach_open(owner, 2, api.build_id.call())
				response.ok = bridge.get("_open_owner") == owner
		"open_setup":
			for source_path in request.get("paths", []):
				response.ok = response.ok and _prepare(source_path).get("ready", false)
			for source_path in request.get("exact_paths", []):
				var source := GDScript.new()
				source.set_source_code(FileAccess.get_file_as_string(source_path))
				source.set_path(source_path)
				source.reload(false)
				_open_refs.append(source)
				EditorInterface.edit_script(source)
			response.ok = response.ok and _open_mutate(request.get("mutation", ""), request.get("path", OPEN_CURRENT))
			if request.get("idle", false):
				var deadline := Time.get_ticks_usec() + 5000000
				while Time.get_ticks_usec() < deadline:
					var current := _open_document(request.get("path", OPEN_CURRENT))
					if current.get("associated", false) and current.R == current.B: break
					await get_tree().process_frame
			response.state = _open_state()
		"open_cache":
			var cached := load(OPEN_TARGET) as GDScript
			response.ok = cached != null
			if cached != null: _open_refs.append(cached)
			response.state = _open_state()
		"open_inspect":
			response.ok = owner != null
			if response.ok:
				response.result = owner.inspect(request.get("path", OPEN_TARGET), {"request_id": request_id,
					"session_id": request.get("session_id", ""), "expiry_tick_us": str(Time.get_ticks_usec() + int(request.get("budget_us", 9000000)))})
		"open_prepare":
			response.ok = owner != null and _open_mutate(request.get("mutation", ""))
			response.before = _open_state()
			if response.ok: response.result = owner.prepare(request_id, request.get("source", ""), request.get("capture", {}))
			response.after = _open_state()
		"open_advance":
			response.ok = owner != null
			if response.ok:
				_open_callback_mode = request.get("callback", "")
				_open_callback_result = {}
				response.result = owner.advance(request_id, request.get("stage", ""), request.get("source_hash", ""), request.get("context_hash", ""))
				_open_retain_cached()
				response.callback = _open_callback_result
				_open_callback_mode = ""
		"open_verify", "open_recheck":
			response.ok = owner != null
			if response.ok:
				response.result = owner.verify(request_id, request.get("purpose", "post_open")) if request.action == "open_verify" else owner.recheck(request_id, request.get("purpose", "post_open"))
		"open_collect":
			response.ok = owner != null and bridge != null and (bridge.get("_active") as Dictionary).get("operation_owner") == owner
			if response.ok:
				var session: String = bridge.get("_session")
				var collector := OpenObservation.new()
				response.sample = collector.collect(session, ProjectSettings.globalize_path("res://").trim_suffix("/"), OPEN_TARGET)
				response.recheck = collector.recheck()
				response.ok = session == bridge.get("_session") and (bridge.get("_active") as Dictionary).get("operation_owner") == owner
				collector.clear()
		"open_finish", "open_cancel":
			response.ok = owner != null
			if response.ok: response.result = owner.finish(request_id) if request.action == "open_finish" else owner.cancel(request_id)
		"open_fault":
			response.ok = api.has("open_fixture_fault")
			if response.ok: response.result = api.open_fixture_fault.call(request_id, request.get("fault", ""))
		"open_mutate":
			response.ok = _open_mutate(request.get("mutation", ""), request.get("path", OPEN_CURRENT))
			response.state = _open_state()
		"open_class_bindings":
			var names: Array = request.get("names", [])
			response.ok = names.size() <= 256
			response.bindings = []
			for binding_name in names:
				if typeof(binding_name) != TYPE_STRING or binding_name.to_utf8_buffer().size() > 256 \
						or not ClassDB.class_exists(binding_name):
					response.ok = false
					break
				response.bindings.append({"name": binding_name, "api_type": ClassDB.class_get_api_type(binding_name)})
		"open_global_change":
			var previous := ProjectSettings.get_global_class_list()
			var changed := FileAccess.open("res://opening_changed_global.gd", FileAccess.WRITE)
			response.ok = changed != null
			if response.ok:
				changed.store_string("class_name OpeningChangedGlobal\nextends RefCounted\n")
				changed.close()
				var filesystem := EditorInterface.get_resource_filesystem()
				filesystem.scan()
				var deadline := Time.get_ticks_usec() + 6000000
				var found := false
				while Time.get_ticks_usec() < deadline:
					for entry in ProjectSettings.get_global_class_list():
						if entry.get("class") == "OpeningChangedGlobal": found = true
					if found and not filesystem.is_scanning(): break
					await get_tree().process_frame
				response.ok = found and previous != ProjectSettings.get_global_class_list()
			response.state = _open_state()
		"open_pending_drag":
			response.drag = await _open_pending_drag(request.get("undo", false))
			response.ok = response.drag.ok
			response.state = _open_state()
		_:
			response.ok = false
	_respond(response)
