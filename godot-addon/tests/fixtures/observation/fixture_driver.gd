@tool
extends EditorPlugin

const Bridge = preload("res://addons/godot_agent_kit/bridge.gd")
const PRODUCT := "godot_agent_kit"
const MAX_CONTROL := 4096

var _control := ""
var _last_id := ""
var _cached_subject: GDScript
var restriction := ""
var transition := ""
var _transition_record := {}
var _unpathed_script: GDScript
var _duplicate_script: GDScript


func _enter_tree() -> void:
	_control = OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if _control.begins_with("/") and DirAccess.dir_exists_absolute(_control):
		set_process(true)
	add_to_group("observation_fixture_driver")


func _process(_delta: float) -> void:
	var request_file := _control.path_join("request.json")
	if not FileAccess.file_exists(request_file):
		return
	var input := FileAccess.open(request_file, FileAccess.READ)
	if input == null:
		return
	if input.get_length() > MAX_CONTROL:
		input.close()
		return
	var raw := input.get_buffer(input.get_length())
	input.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw:
		return
	var parser := JSON.new()
	if parser.parse(text) != OK or typeof(parser.data) != TYPE_DICTIONARY:
		return
	var request: Dictionary = parser.data
	if request.size() != 2 or typeof(request.get("id")) != TYPE_STRING or typeof(request.get("action")) != TYPE_STRING:
		return
	var id: String = request.id
	var action: String = request.action
	if id.is_empty() or id.length() > 64 or id == _last_id:
		return
	_last_id = id
	var response := {"id": id, "action": action, "ok": true}
	match action:
		"witness":
			response.merge(_witness())
		"present":
			# Owned GUI preparation only; never reachable through the product.
			var window := EditorInterface.get_base_control().get_window()
			window.mode = Window.MODE_WINDOWED
			window.show()
			window.grab_focus()
			await get_tree().process_frame
		"cache_subject":
			# Explicit owned preparation; do not depend on import-time cache races.
			_cached_subject = load("res://scripts/subject.gd") as GDScript
			response.ok = _cached_subject != null
		"prepare_subject":
			response.merge(_prepare("res://scripts/subject.gd"))
			response.ok = response.get("ready", false)
		"prepare_empty":
			response.merge(_prepare("res://scripts/empty.gd"))
			response.ok = response.get("ready", false)
		"seed_history":
			var history := _document("res://scripts/subject.gd")
			if not history.get("associated", false):
				response.ok = false
			else:
				var buffer := EditorInterface.get_script_editor().get_open_script_editors()[history.index].get_base_editor() as CodeEdit
				buffer.set_caret_line(buffer.get_line_count() - 1)
				buffer.set_caret_column(buffer.get_line(buffer.get_caret_line()).length())
				buffer.insert_text_at_caret("# FIXTURE_UNDO_HISTORY")
				buffer.undo()
				response.merge(_document("res://scripts/subject.gd"))
		"append_subject":
			var before := _document("res://scripts/subject.gd")
			if before.get("associated", false):
				var editors := EditorInterface.get_script_editor().get_open_script_editors()
				var buffer := editors[before.index].get_base_editor() as CodeEdit
				buffer.set_caret_line(buffer.get_line_count() - 1)
				buffer.set_caret_column(buffer.get_line(buffer.get_caret_line()).length())
				buffer.insert_text_at_caret("\n# FIXTURE_RECHECK_CHANGE")
				response.merge(_document("res://scripts/subject.gd"))
			else:
				response.ok = false
		"cap_resource_exact", "cap_resource_over", "cap_buffer_exact", "cap_buffer_over":
			var doc := _document("res://scripts/subject.gd")
			if not doc.get("associated", false):
				response.ok = false
			else:
				var script := EditorInterface.get_script_editor().get_open_scripts()[doc.index] as GDScript
				var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
				var amount := 524289 if action.ends_with("_over") else 524288
				# The pinned editor copies valid B into R during idle validation.
				# A syntax-invalid synthetic B keeps the independent authorities
				# stable without disabling native processing or replacing getters.
				var invalid_prefix := "var =\n# "
				if action.begins_with("cap_resource"):
					buffer.text = "var =\n"
					script.source_code = "# " + "r".repeat(amount - 2)
				else:
					buffer.text = invalid_prefix + "b".repeat(amount - invalid_prefix.length())
				response.merge(_document("res://scripts/subject.gd"))
		"prepare_other":
			response.merge(_prepare("res://scripts/other.gd"))
			response.ok = response.get("ready", false)
		"dirty_subject", "dirty_other", "equal_dirty_subject":
			var path := "res://scripts/other.gd" if action == "dirty_other" else "res://scripts/subject.gd"
			var doc := _document(path)
			if not doc.get("associated", false):
				response.ok = false
			else:
				var code := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
				if action == "equal_dirty_subject":
					var unchanged := code.text
					code.text = "var =\n# TEMPORARY_DIRTY_EDIT\n"
					code.text = unchanged
				else:
					# Invalid syntax prevents the editor from automatically copying
					# B into R during idle parsing on the pinned native editor.
					code.text = "var =\n# FIXTURE_UNSAVED_CHANGE\n"
				response.merge(_document(path))
		"resource_subject":
			var doc := _document("res://scripts/subject.gd")
			if not doc.get("associated", false):
				response.ok = false
			else:
				var script := EditorInterface.get_script_editor().get_open_scripts()[doc.index] as GDScript
				script.source_code = "extends RefCounted\n# RESOURCE_DIFFERENT\n"
				response.merge(_document("res://scripts/subject.gd"))
		"restrict_dirty", "restrict_global", "restrict_association", "restore_dirty":
			restriction = "withhold_dirty" if action == "restrict_dirty" else (
				"unattributed_global" if action == "restrict_global" else (
				"withhold_association" if action == "restrict_association" else ""))
			response.restriction = restriction
		"transition_buffer", "transition_dirty", "transition_resource", "transition_disk", "transition_rename", "transition_remove", "transition_close", "transition_close_resource", "transition_replace":
			transition = action.trim_prefix("transition_")
			_transition_record = {}
			response.transition = transition
		"transition_witness":
			response.merge({"transition": transition, "record": _transition_record,
				"document": _document("res://scripts/subject.gd")})
		"mixed_tabs":
			# Open documentation, then activate the selected plain-text file
			# through the owned FileSystemDock's real item-activation signal.
			var script_editor := EditorInterface.get_script_editor()
			script_editor.goto_help("class_name:Node")
			await get_tree().process_frame
			response.documentation_selected = script_editor.get_current_script() == null
			var text_path := "res://scripts/note.txt"
			var dock := EditorInterface.get_file_system_dock()
			dock.navigate_to_path(text_path)
			await get_tree().process_frame
			var activated := false
			if EditorInterface.get_selected_paths().has(text_path):
				for tree in dock.find_children("*", "Tree", true, false):
					var file_tree := tree as Tree
					if file_tree != null and file_tree.get_selected() != null \
						and file_tree.get_selected().get_text(0) == "note.txt":
						file_tree.item_activated.emit()
						activated = true
						break
				if not activated:
					for list in dock.find_children("*", "ItemList", true, false):
						var file_list := list as ItemList
						if file_list != null:
							for item in file_list.get_selected_items():
								if file_list.get_item_text(item) == "note.txt":
									file_list.item_activated.emit(item)
									activated = true
									break
			await get_tree().process_frame
			response.merge(_witness())
			response.text_selected = EditorInterface.get_selected_paths().has(text_path)
			response.ok = response.documentation_selected and response.text_selected \
				and activated and response.editor_count > response.script_count
		"unpathed_script":
			_unpathed_script = GDScript.new()
			_unpathed_script.source_code = "extends RefCounted\n"
			EditorInterface.edit_script(_unpathed_script)
			response.merge(_witness())
			response.ok = response.open_paths.has("")
		"duplicate_script":
			_duplicate_script = GDScript.new()
			_duplicate_script.source_code = "extends RefCounted\n# DISTINCT_UNIQUE_OBJECT\n"
			# Open a distinct native tab first: the editor deduplicates paths
			# at opening. Then give the two actual resources a nonunique path.
			_duplicate_script.set_path_cache("res://scripts/other.gd")
			EditorInterface.edit_script(_duplicate_script)
			_duplicate_script.set_path_cache("res://scripts/subject.gd")
			response.merge(_witness())
			response.ok = response.open_paths.count("res://scripts/subject.gd") > 1
		"disable", "enable":
			var enabled := action == "enable"
			EditorInterface.set_plugin_enabled(PRODUCT, enabled)
			response.plugin_enabled = EditorInterface.is_plugin_enabled(PRODUCT)
			response.ok = response.plugin_enabled == enabled
		"proof_vectors":
			response.merge(_proof_vectors())
			response.ok = response.transcript_bytes == 270 and response.server_matches and response.client_matches and response.finish_matches
		"quit":
			pass
		_:
			response.ok = false
	if not _respond(response):
		return
	if action == "quit":
		get_tree().call_deferred("quit")


func _respond(response: Dictionary) -> bool:
	var temporary := _control.path_join("response.tmp")
	var destination := _control.path_join("response.json")
	var file := FileAccess.open(temporary, FileAccess.WRITE)
	if file == null:
		return false
	var written := file.store_string(JSON.stringify(response))
	file.flush()
	file.close()
	if not written or FileAccess.set_unix_permissions(temporary, 384) != OK:
		return false
	return DirAccess.rename_absolute(temporary, destination) == OK


func _witness() -> Dictionary:
	var version := Engine.get_version_info()
	var exact_version := "%s.%s.%s.%s.%s.%s" % [version.major, version.minor, version.patch, version.status, version.build, version.hash.left(9)]
	var nodes: Array[Dictionary] = []
	for node in get_tree().get_nodes_in_group("godot_agent_kit_session_bridge"):
		nodes.append({"name": node.name, "type": node.get_class()})
	var editor := EditorInterface.get_script_editor()
	var scripts := editor.get_open_scripts()
	var editors := editor.get_open_script_editors()
	var paths := PackedStringArray()
	var script_ids := PackedStringArray()
	var script_types := PackedStringArray()
	var editor_types := PackedStringArray()
	var editor_ids := PackedStringArray()
	for script in scripts:
		paths.append(script.resource_path if script != null else "")
		script_ids.append(String.num_uint64(script.get_instance_id()) if script != null else "")
		script_types.append(script.get_class() if script != null else "")
	for base in editors:
		editor_types.append(base.get_class() if base != null else "")
		editor_ids.append(String.num_uint64(base.get_instance_id()) if base != null else "")
	var selected := editor.get_current_script()
	var cached := ResourceLoader.get_cached_ref("res://scripts/subject.gd") as GDScript
	return {"version": exact_version, "engine_hash": version.get("hash", ""),
		"editor_hint": Engine.is_editor_hint(), "pid": OS.get_process_id(),
		"plugin_enabled": EditorInterface.is_plugin_enabled(PRODUCT),
		"product_nodes": nodes, "open_paths": paths,
		"script_count": scripts.size(), "editor_count": editors.size(),
		"script_ids": script_ids, "script_types": script_types,
		"editor_types": editor_types, "editor_ids": editor_ids,
		"unsaved_paths": editor.get_unsaved_files(),
		"current_script": selected.resource_path if selected != null else "",
		"subject_cached_id": String.num_uint64(cached.get_instance_id()) if cached != null else "",
		"subject": _document("res://scripts/subject.gd"),
		"empty": _document("res://scripts/empty.gd"),
		"other": _document("res://scripts/other.gd")}


func apply_transition() -> void:
	if transition.is_empty():
		return
	var mode := transition
	transition = ""
	var before := _document("res://scripts/subject.gd")
	_transition_record = {"mode": mode, "before": before}
	var scripts := EditorInterface.get_script_editor().get_open_scripts()
	if not before.get("associated", false):
		_transition_record.ok = false
		return
	var script := scripts[before.index] as GDScript
	var buffer := EditorInterface.get_script_editor().get_open_script_editors()[before.index].get_base_editor() as CodeEdit
	var path := "res://scripts/subject.gd"
	match mode:
		"buffer", "dirty":
			buffer.text = "var =\n# TRANSITION_BUFFER\n"
		"resource":
			script.source_code = "extends RefCounted\n# TRANSITION_RESOURCE\n"
		"disk":
			var file := FileAccess.open(path, FileAccess.WRITE)
			if file != null:
				file.store_string("extends RefCounted\n# TRANSITION_DISK\n")
				file.flush()
				file.close()
			_transition_record.ok = file != null
		"rename":
			_transition_record.ok = DirAccess.rename_absolute(
				ProjectSettings.globalize_path(path),
				ProjectSettings.globalize_path("res://scripts/renamed.gd")) == OK
			if _transition_record.ok:
				script.resource_path = "res://scripts/renamed.gd"
		"remove":
			_transition_record.ok = DirAccess.remove_absolute(ProjectSettings.globalize_path(path)) == OK
		"close", "close_resource":
			_transition_record.ok = EditorInterface.get_script_editor().close_file(path) == OK
			if mode == "close_resource":
				script.source_code = "extends RefCounted\n# TRANSITION_RESOURCE\n"
		"replace":
			if EditorInterface.get_script_editor().close_file(path) == OK:
				var replacement := GDScript.new()
				replacement.source_code = before.R
				replacement.take_over_path(path)
				EditorInterface.edit_script(replacement)
				_transition_record.replacement_id = String.num_uint64(replacement.get_instance_id())
			else:
				_transition_record.ok = false
	_transition_record["after"] = _document(path)
	_transition_record.original_resource = script.source_code
	_transition_record.witness_after = _witness()
	if not _transition_record.has("ok"):
		_transition_record.ok = true


func _prepare(path: String) -> Dictionary:
	# Preparation alone may load/select. Neither this action nor this helper is
	# reachable through the product's authenticated observe/recheck operations.
	var script := load(path) as GDScript
	if script == null:
		return {"ready": false}
	EditorInterface.edit_script(script)
	EditorInterface.set_main_screen_editor("Script")
	var state := _document(path)
	return {"ready": true, "document": state}


func _document(path: String) -> Dictionary:
	var editor := EditorInterface.get_script_editor()
	var scripts := editor.get_open_scripts()
	var editors := editor.get_open_script_editors()
	var matches := 0
	var index := -1
	for i in scripts.size():
		if scripts[i] != null and scripts[i].resource_path == path:
			matches += 1
			index = i
	var result := {"path": path, "script_count": scripts.size(), "editor_count": editors.size(),
		"matches": matches, "associated": false}
	if matches != 1 or scripts.size() != editors.size() or not (scripts[index] is GDScript):
		return result
	var base: ScriptEditorBase = editors[index]
	if base == null:
		return result
	var buffer := base.get_base_editor() as CodeEdit
	if buffer == null:
		return result
	var script := scripts[index] as GDScript
	var unsaved: PackedStringArray = editor.get_unsaved_files()
	result.merge({"associated": true, "index": index, "script_id": String.num_uint64(script.get_instance_id()),
		"editor_id": String.num_uint64(base.get_instance_id()), "buffer_id": String.num_uint64(buffer.get_instance_id()),
		"script_type": script.get_class(), "editor_type": base.get_class(),
		"buffer_type": buffer.get_class(), "R": script.source_code, "B": buffer.text,
		"dirty": path in unsaved, "version": buffer.get_version(),
		"saved_version": buffer.get_saved_version(),
		"caret_line": buffer.get_caret_line(), "caret_column": buffer.get_caret_column(),
		"has_selection": buffer.has_selection(), "selection_from_line": buffer.get_selection_from_line() if buffer.has_selection() else null,
		"selection_from_column": buffer.get_selection_from_column() if buffer.has_selection() else null,
		"selection_to_line": buffer.get_selection_to_line() if buffer.has_selection() else null,
		"selection_to_column": buffer.get_selection_to_column() if buffer.has_selection() else null,
		"has_undo": buffer.has_undo(), "has_redo": buffer.has_redo()}, true)
	return result


func _proof_vectors() -> Dictionary:
	# Synthetic public contract constants. Never inspect a live descriptor or expose
	# transcript, nonce, token, or proof bytes in the control response.
	var token := "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".hex_decode()
	var caps := {"observe_gdscript": true, "open_enumeration": true,
		"buffer_attribution": true, "unsaved_paths": true, "cached_resource_lookup": true}
	var transcript := Bridge.transcript_bytes("example-1", "00112233445566778899aabbccddeeff",
		"/fixture/project", "4.7.2.stable.official.ed1daf0bf",
		"ed1daf0bf001b61586d9930840f2f1394092c079", caps,
		"202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
		"404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f")
	return {"transcript_bytes": transcript.size(),
		"server_matches": Bridge.role_proof(token, "server", transcript).hex_encode() == "31cd95cdc00afe815371791207ddedda98cb79e1d6bf181bd4fb8ac3cc8d2ed1",
		"client_matches": Bridge.role_proof(token, "client", transcript).hex_encode() == "8516bc474f397be9deae2a70c4ae5577041eb6753cb1dde328278978c2f8b5e8",
		"finish_matches": Bridge.role_proof(token, "finish", transcript).hex_encode() == "ff0f3f3507b79628be82e0cd67539765a174d9d0323aff470d62bbecbf12dd32"}
