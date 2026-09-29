@tool
extends "res://fixture_driver.gd"

# All controls below belong only to the owned fixture. The product addon never
# receives an action, source string, test session, or synthetic validation result.
const NATIVE_META := "godot_agent_kit_native"
const NATIVE_CONTROL_LIMIT := 540000

var _native_filesystem_events := 0
var _native_session := ""
var _native_remove_mode := ""
var _native_remove_request := ""
var _native_remove_owner: Node
var _native_remove_bridge: Node
var _native_remove_buffer: CodeEdit
var _native_remove_witness := {}


func _native_removed(_from_line: int, _to_line: int) -> void:
	if _native_remove_mode.is_empty():
		return
	var mode := _native_remove_mode
	_native_remove_mode = "" # The human edit below emits the same signal.
	_native_remove_witness = {"called": true, "slot_before":
		is_instance_valid(_native_remove_bridge) and
		not (_native_remove_bridge.get("_active") as Dictionary).is_empty()}
	if mode == "change":
		_native_remove_buffer.insert_text("# HUMAN_CALLBACK_NEWER\n", 0, 0)
		_native_remove_witness.human_text = _native_remove_buffer.text
	elif mode == "cancel":
		_native_remove_witness.cancel = _native_remove_owner.cancel(_native_remove_request)
	elif mode == "stop":
		_native_remove_bridge.stop()
		_native_remove_witness.stopped = true
	elif mode == "nested_finish":
		_native_remove_witness.nested = _native_remove_owner.finish(_native_remove_request)
	_native_remove_witness.slot_after = is_instance_valid(_native_remove_bridge) and \
		not (_native_remove_bridge.get("_active") as Dictionary).is_empty()

func _native_filesystem_changed() -> void:
	_native_filesystem_events += 1

func _enter_tree() -> void:
	super._enter_tree()
	EditorInterface.get_resource_filesystem().filesystem_changed.connect(_native_filesystem_changed)

func _process(_delta: float) -> void:
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
		super._dispatch_request(request)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id:
		return
	_last_id = id
	var action: String = request.action
	var response := {"id": id, "action": action, "ok": true}
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	if action == "native_info":
		response.api_installed = api.has_all(["api_revision", "build_id", "configure", "close"])
		response.edit_installed = api.has_all(["edit_prepare", "edit_advance", "edit_cancel"])
		if response.api_installed:
			response.api_revision = api.api_revision.call()
			response.build_id = api.build_id.call()
		response.tool_cached = ResourceLoader.has_cached("res://scripts/native/cold/tool_initializer.gd")
		response.filesystem_events = _native_filesystem_events
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
	elif action == "native_warning_info":
		var prefix := "debug/gdscript/warnings/"
		var levels := {}
		for property in ProjectSettings.get_property_list():
			var key: String = property.get("name", "")
			if not key.begins_with(prefix) or key.trim_prefix(prefix).contains("."):
				continue
			var warning_name := key.trim_prefix(prefix)
			if warning_name in ["enable", "directory_rules", "exclude_addons",
					"property_used_as_function", "constant_used_as_function",
					"function_used_as_property"]:
				continue
			var value: Variant = ProjectSettings.get_setting_with_override(key)
			if typeof(value) == TYPE_INT:
				levels[warning_name] = value
		response.warning_profile = {"enable": ProjectSettings.get_setting_with_override(
				prefix + "enable"), "levels": levels, "directory_rules":
				ProjectSettings.get_setting_with_override(prefix + "directory_rules")}
		var global_classes := []
		for entry in ProjectSettings.get_global_class_list():
			if entry.get("language") == "GDScript" and typeof(entry.get("class")) in [TYPE_STRING, TYPE_STRING_NAME]:
				global_classes.append(String(entry["class"]))
				if global_classes.size() > 256:
					break
		response.global_classes = global_classes
		response.session_id = _native_session
		response.project_root = ProjectSettings.globalize_path("res://").trim_suffix("/")
	elif action == "native_promote_unused_warning":
		var key := "debug/gdscript/warnings/unused_variable"
		ProjectSettings.set_setting("debug/gdscript/warnings/enable", true)
		ProjectSettings.set_setting(key, 2)
		response.level = ProjectSettings.get_setting_with_override(key)
	elif not api.has_all(["api_revision", "build_id", "configure", "close"]):
		response.ok = false
	elif action == "native_close":
		api.close.call()
		_native_session = ""
	elif action == "native_configure":
		var session_id: String = request.get("session_id", "")
		response.configured = api.configure.call(session_id)
		if response.configured:
			_native_session = session_id
	elif action == "native_edit_inspect":
		var doc := _document("res://scripts/subject.gd")
		response.ok = doc.get("associated", false)
		if response.ok:
			var script := EditorInterface.get_script_editor().get_open_scripts()[doc.index]
			response.resource_edited = EditorInterface.is_object_edited(script)
			response.document = doc
	elif action == "native_edit_private_info":
		var bridge: Node = get_tree().get_first_node_in_group("godot_agent_kit_session_bridge")
		response.ok = bridge != null
		if response.ok:
			response.slot_busy = not (bridge.get("_active") as Dictionary).is_empty()
	elif action in ["native_edit_private_begin", "native_edit_private_cancel",
			"native_edit_private_finish"]:
		var bridge: Node = get_tree().get_first_node_in_group("godot_agent_kit_session_bridge")
		var edit_owner: Node = bridge.get_parent().get_node_or_null("GodotAgentKitScriptEdit") \
			if bridge != null else null
		response.ok = edit_owner != null
		if response.ok and action == "native_edit_private_cancel":
			response.result = edit_owner.cancel(request.get("request_id", ""))
		elif response.ok and action == "native_edit_private_finish":
			var doc := _document("res://scripts/subject.gd")
			response.ok = doc.get("associated", false)
			if response.ok:
				var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
				var mode: String = request.get("mode", "")
				response.slot_busy_before = not (bridge.get("_active") as Dictionary).is_empty()
				if mode in ["change", "cancel", "stop", "nested_finish"]:
					_native_remove_mode = mode
					_native_remove_request = request.get("request_id", "")
					_native_remove_owner = edit_owner
					_native_remove_bridge = bridge
					_native_remove_buffer = buffer
					_native_remove_witness = {}
					buffer.lines_edited_from.connect(_native_removed)
				response.result = edit_owner.finish(request.get("request_id", ""))
				if mode in ["change", "cancel", "stop", "nested_finish"]:
					buffer.lines_edited_from.disconnect(_native_removed)
					_native_remove_mode = ""
					response.callback = _native_remove_witness
					response.slot_busy_after = not (bridge.get("_active") as Dictionary).is_empty()
				response.document = _document("res://scripts/subject.gd")
		elif response.ok:
			var doc := _document("res://scripts/subject.gd")
			response.ok = doc.get("associated", false)
			if response.ok:
				response.result = edit_owner.begin("res://scripts/subject.gd",
					request.get("expected", doc.R), request.get("desired", ""),
					{"request_id": request.get("request_id", ""),
					"session_id": request.get("session_id", ""),
					"document": {"resource_instance_id": doc.script_id,
						"editor_instance_id": doc.editor_id, "buffer_instance_id": doc.buffer_id},
					"expiry_tick_us": str(Time.get_ticks_usec() + 9000000)})
	elif action == "native_edit_prepare":
		var doc := _document("res://scripts/subject.gd")
		if not doc.get("associated", false) or not api.has("edit_prepare"):
			response.ok = false
		else:
			var binding := {"resource_instance_id": doc.script_id,
				"editor_instance_id": doc.editor_id, "buffer_instance_id": doc.buffer_id}
			if request.get("wrong_document", false):
				binding.buffer_instance_id = "0"
			response.result = api.edit_prepare.call(
				request.get("source_path", "res://scripts/subject.gd"),
				request.get("expected", doc.R), request.get("desired", ""),
				{"request_id": request.get("request_id", ""),
				"session_id": request.get("session_id", ""),
				"document": binding,
				"expiry_tick_us": str(Time.get_ticks_usec() + int(request.get("expiry_budget_us", 9000000)))})
	elif action == "native_edit_advance":
		response.ok = api.has("edit_advance")
		if response.ok:
			var mode: String = request.get("mode", "")
			var doc := _document("res://scripts/subject.gd")
			var buffer: CodeEdit = null
			if mode == "change" and doc.get("associated", false):
				buffer = EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
				_native_remove_mode = mode
				_native_remove_bridge = get_tree().get_first_node_in_group("godot_agent_kit_session_bridge")
				_native_remove_buffer = buffer
				_native_remove_witness = {}
				buffer.lines_edited_from.connect(_native_removed)
			response.result = api.edit_advance.call(
				request.get("request_id", ""), request.get("stage", ""))
			if buffer != null:
				buffer.lines_edited_from.disconnect(_native_removed)
				_native_remove_mode = ""
				response.callback = _native_remove_witness
	elif action == "native_edit_settle":
		var deadline := Time.get_ticks_usec() + 5000000
		for _frame in 4:
			await get_tree().process_frame
		while request.get("wait_for_initializer", false) and \
				not FileAccess.file_exists(_control.path_join("excluded_initializer_was_called")) and \
				Time.get_ticks_usec() < deadline:
			await get_tree().process_frame
		response.tool_cached = ResourceLoader.has_cached("res://scripts/native/cold/tool_initializer.gd")
		response.initializer_called = FileAccess.file_exists(_control.path_join("excluded_initializer_was_called"))
		response.constructor_called = FileAccess.file_exists(_control.path_join("excluded_constructor_was_called"))
		response.document = _document("res://scripts/subject.gd")
	elif action == "native_edit_trim_final_newlines":
		var settings := EditorInterface.get_editor_settings()
		var key := "text_editor/behavior/files/trim_final_newlines_on_save"
		settings.set_setting(key, true)
		response.enabled = settings.get_setting(key)
	elif action == "native_edit_cancel":
		response.ok = api.has("edit_cancel")
		if response.ok:
			response.result = api.edit_cancel.call(request.get("request_id", ""))
	elif action == "native_edit_fault":
		response.ok = api.has("edit_fixture_fault")
		if response.ok:
			response.result = api.edit_fixture_fault.call(
				request.get("request_id", ""), request.get("fault", ""))
	elif action == "native_edit_human":
		var doc := _document("res://scripts/subject.gd")
		response.ok = doc.get("associated", false)
		if response.ok:
			var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
			var mode: String = request.get("mode", "")
			match mode:
				"seed_prior":
					buffer.set_caret_line(buffer.get_line_count() - 1)
					buffer.set_caret_column(buffer.get_line(buffer.get_caret_line()).length())
					buffer.begin_complex_operation()
					buffer.insert_text_at_caret("\n# EARLIER_NATIVE_HISTORY\n")
					buffer.end_complex_operation()
				"dirty":
					buffer.text = "extends RefCounted\n# HUMAN_NEWER_TEXT\n"
				"later_value":
					buffer.text = "extends RefCounted\nfunc value() -> int:\n\treturn 29\n"
				"cold_inheritance":
					buffer.text = "extends \"./native/cold/tool_initializer.gd\"\nfunc value() -> int:\n\treturn 23\n"
				"same_text":
					var original := buffer.text
					buffer.text = "extends RefCounted\n# HUMAN_TEMPORARY_TEXT\n"
					buffer.text = original
				"profile":
					buffer.set_indent_size(8 if buffer.get_indent_size() != 8 else 4)
				"saved_version":
					buffer.tag_saved_version()
				"undo":
					buffer.undo()
				"redo":
					buffer.redo()
				"close_reopen":
					response.ok = EditorInterface.get_script_editor().close_file(
						"res://scripts/subject.gd") == OK
					if response.ok:
						response.merge(_prepare("res://scripts/subject.gd"))
				_:
					response.ok = false
			response.document = _document("res://scripts/subject.gd")
	elif action == "native_edit_save":
		var doc := _document("res://scripts/subject.gd")
		response.ok = doc.get("associated", false) and doc.dirty
		if response.ok:
			var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
			var script := EditorInterface.get_script_editor().get_open_scripts()[doc.index] as GDScript
			EditorInterface.edit_script(script)
			buffer.grab_focus()
			await get_tree().process_frame
			response.focused = buffer.has_focus()
			var before_disk := FileAccess.get_file_as_string("res://scripts/subject.gd")
			var press := InputEventKey.new()
			press.keycode = KEY_S
			press.meta_pressed = true
			press.alt_pressed = true
			press.pressed = true
			Input.parse_input_event(press)
			var release := press.duplicate() as InputEventKey
			release.pressed = false
			Input.parse_input_event(release)
			var deadline := Time.get_ticks_usec() + 4000000
			while Time.get_ticks_usec() < deadline:
				if FileAccess.get_file_as_string("res://scripts/subject.gd") == doc.B \
						and not _document("res://scripts/subject.gd").dirty:
					break
				await get_tree().process_frame
			var saved := _document("res://scripts/subject.gd")
			response.before_disk_changed = before_disk != saved.B
			response.disk_matches = FileAccess.get_file_as_string("res://scripts/subject.gd") == saved.B
			response.document = saved
			response.ok = response.focused and response.before_disk_changed \
				and response.disk_matches and not saved.dirty \
				and saved.version == saved.saved_version
	elif action == "native_edit_scan":
		var filesystem := EditorInterface.get_resource_filesystem()
		var deadline := Time.get_ticks_usec() + 6000000
		while filesystem.is_scanning() and Time.get_ticks_usec() < deadline:
			await get_tree().process_frame
		# Let an already-finished scan drain its worker before requesting ours.
		await get_tree().process_frame
		var before := _native_filesystem_events
		filesystem.scan()
		while (filesystem.is_scanning() or _native_filesystem_events <= before) \
				and Time.get_ticks_usec() < deadline:
			await get_tree().process_frame
		response.before_events = before
		response.events = _native_filesystem_events
		response.settled = not filesystem.is_scanning()
		response.ok = response.settled and response.events > before
	else:
		response.ok = false
	_respond(response)
