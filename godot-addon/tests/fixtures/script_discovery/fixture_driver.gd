@tool
extends "res://addons/fixture_driver/open_fixture_driver.gd"

# Private controls and independent native witnesses, never product opcodes.
var discovery_hold := ""
var discovery_fault := ""

func _native_paths(directory: EditorFileSystemDirectory, paths: Array) -> void:
	for index in directory.get_file_count():
		var path := directory.get_file_path(index)
		if path.get_extension().to_lower() == "gd": paths.append(path)
	for index in directory.get_subdir_count():
		_native_paths(directory.get_subdir(index), paths)

func _dispatch_open_request(request: Dictionary) -> void:
	# Dispatch the parent's single parsed snapshot. Reopening request.json here
	# could route a newer discovery command through a lower-level dispatcher.
	if not String(request.get("action", "")).begins_with("discovery_"):
		super._dispatch_open_request(request)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id: return
	_last_id = id
	var response := {"id": id, "action": request.action, "ok": true}
	var bridge := _open_bridge()
	match String(request.action):
		"discovery_oracle":
			var paths := []
			var filesystem := EditorInterface.get_resource_filesystem()
			_native_paths(filesystem.get_filesystem(), paths)
			paths.sort()
			response.paths = paths
			response.scanning = filesystem.is_scanning()
			response.importing = filesystem.is_importing()
			response.settings_directory = EditorInterface.get_editor_paths().get_project_settings_dir()
		"discovery_hold":
			discovery_hold = request.get("stage", "begin")
			discovery_fault = request.get("fault", "")
			DirAccess.remove_absolute(_control.path_join("discovery-event.json"))
		"discovery_release":
			discovery_hold = ""
			if bridge != null: bridge.discovery_release()
		"discovery_scope_change":
			scope_mode = "changed_data_directory"
		"discovery_expire":
			var active: Dictionary = bridge.get("_active") if bridge != null else {}
			response.ok = active.has("expiry_tick_us")
			if response.ok:
				active.expiry_tick_us = Time.get_ticks_usec()
				response.expiry_tick_us = str(active.expiry_tick_us)
		"discovery_history_step":
			# Deliberate human history activity without changing tab selection.
			var doc := _document("res://scripts/subject.gd")
			var operation: String = request.get("operation", "")
			response.ok = doc.get("associated", false) and operation in ["undo", "redo"]
			if response.ok:
				var buffer := EditorInterface.get_script_editor().get_open_script_editors()[doc.index].get_base_editor() as CodeEdit
				response.ok = buffer.has_undo() if operation == "undo" else buffer.has_redo()
				if response.ok:
					if operation == "undo": buffer.undo()
					else: buffer.redo()
					response.document = _document("res://scripts/subject.gd")
		"discovery_unloaded":
			response.cached = ResourceLoader.has_cached(request.get("path", ""))
			response.loader_calls = _open_loader.calls
			response.open_paths = _witness().open_paths
		_: response.ok = false
	var output := FileAccess.open(_control.path_join("response.json"), FileAccess.WRITE)
	if output != null:
		output.store_string(JSON.stringify(response))
		output.close()

func discovery_barrier(_peer: Dictionary, payload: Dictionary) -> bool:
	var stage := "begin" if payload.get("kind") == "discover_state" else "recheck"
	if stage != discovery_hold: return false
	if payload.get("expiry_tick_us") == null: return false
	var event := FileAccess.open(_control.path_join("discovery-event.json"), FileAccess.WRITE)
	if event != null:
		event.store_string(JSON.stringify({"stage": stage, "request_id": payload.request_id}))
		event.close()
	return true
