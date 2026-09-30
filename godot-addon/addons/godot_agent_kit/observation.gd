@tool
extends RefCounted

# Request-local, getter-only editor evidence. No editor selection, resource load, or
# filesystem source read is permitted here; the caller owns independent disk D.
const SOURCE_LIMIT := 512 * 1024

var _session := ""
var _project := ""
var _path := ""
var _original := {}


func _stamp(started: int, finished: int) -> Dictionary:
	return {"clock_id": "editor:" + _session, "started_tick_us": str(started),
		"finished_tick_us": str(finished), "received_elapsed_us": 0}


func _reason(code: String, action: String) -> Dictionary:
	return {"code": code, "action": action}


func _fact(value: String, stamp: Dictionary) -> Dictionary:
	return {"value": value, "collection": stamp}


func _unknown_fact(code: String = "unavailable") -> Dictionary:
	return {"reason": _reason(code, "Establish the requested document identity and open state")}


func _source_unavailable(authority: String, code: String) -> Dictionary:
	var action := "Establish document-specific live editor evidence"
	match code:
		"too_large": action = "Inspect this source using a separately authorized bounded workflow"
		"resource_not_loaded", "resource_unreadable": action = "Inspect already-loaded resource state without force-loading"
		"identity_changed", "source_changed", "document_closed": action = "Start a new observation of the intended editor and script"
		"document_not_open": action = "Observe an open document to obtain buffer text"
	return {"authority": authority, "availability": "unavailable", "reason": _reason(code, action)}


func _dirty_unavailable(code: String = "dirty_attribution_unavailable") -> Dictionary:
	return {"availability": "unavailable", "state": "unknown", "reason": _reason(code,
		"Establish document-specific editor unsaved-work evidence")}


# RefCounted ObjectIDs use the high bit; GDScript int is signed. Keep all uint64
# bits when crossing the decimal-string wire boundary.
func _witness(script: GDScript, editor: ScriptEditorBase, buffer: CodeEdit) -> Dictionary:
	var witness := {"resource_path": _path}
	if script != null:
		witness.script_instance_id = String.num_uint64(script.get_instance_id())
	if editor != null:
		witness.editor_instance_id = String.num_uint64(editor.get_instance_id())
	if buffer != null:
		witness.buffer_instance_id = String.num_uint64(buffer.get_instance_id())
		witness.source_version = str(buffer.get_version())
	return witness


func _read_source(authority: String, source: String, start: int, witness: Dictionary) -> Dictionary:
	# Check scalar length before allocating UTF-8 for oversized source text.
	if source.length() > SOURCE_LIMIT or source.to_utf8_buffer().size() > SOURCE_LIMIT:
		return _source_unavailable(authority, "too_large")
	return {"authority": authority, "availability": "observed", "text": source,
		"collection": _stamp(start, Time.get_ticks_usec()), "witness": witness,
		"staleness": {"state": "unknown"}}


func _scan(script_editor: ScriptEditor) -> Dictionary:
	var scripts: Array = script_editor.get_open_scripts()
	var editors: Array = script_editor.get_open_script_editors()
	var paths: Array[String] = []
	var script_ids: Array[int] = []
	var editor_ids: Array[int] = []
	var unique := true
	var target_index := -1
	var target_script: GDScript
	for i in scripts.size():
		var script: Variant = scripts[i]
		if not (script is GDScript) or not is_instance_valid(script):
			unique = false
			paths.append("")
			script_ids.append(0)
			continue
		var path: String = script.resource_path
		if path.is_empty() or paths.has(path):
			unique = false
		paths.append(path)
		script_ids.append(script.get_instance_id())
		if path == _path:
			if target_index != -1:
				unique = false
			target_index = i
			target_script = script
	for editor in editors:
		if not (editor is ScriptEditorBase) or not is_instance_valid(editor):
			unique = false
			editor_ids.append(0)
		else:
			editor_ids.append(editor.get_instance_id())
	return {"paths": paths, "script_ids": script_ids, "editor_ids": editor_ids,
		"unique": unique, "matched": scripts.size() == editors.size(),
		"index": target_index, "script": target_script, "editors": editors}


func _same_target_editor(before: Dictionary, after: Dictionary) -> bool:
	# Other tabs may move without changing the target's script/editor association.
	return before.unique and before.matched and after.unique and after.matched \
		and before.index >= 0 and after.index >= 0 \
		and before.script_ids[before.index] == after.script_ids[after.index] \
		and before.editor_ids[before.index] != 0 \
		and before.editor_ids[before.index] == after.editor_ids[after.index]


func _unsaved(script_editor: ScriptEditor, paths: Array[String]) -> Dictionary:
	var started := Time.get_ticks_usec()
	var files: PackedStringArray = script_editor.get_unsaved_files()
	var finished := Time.get_ticks_usec()
	var attributable := true
	var seen: Dictionary = {}
	for path in files:
		# An unassignable global indication cannot establish that this document is clean.
		if path.is_empty() or seen.has(path) or not paths.has(path):
			attributable = false
		seen[path] = true
	return {"stamp": _stamp(started, finished), "known": attributable,
		"dirty": files.has(_path), "paths": files}

func _invalid_target(started: int) -> Dictionary:
	var stamp := _stamp(started, Time.get_ticks_usec())
	_original = {"open": false, "closed": false}
	return {"v": 3, "kind": "sample", "request_id": "", "session_id": _session,
		"project_root": _project, "script_path": _path, "collection": stamp,
		"document": {"identity": null, "validity": _fact("invalid", stamp),
			"open_state": _unknown_fact("open_state_unknown")},
		"R": _source_unavailable("R", "resource_not_loaded"),
		"B": _source_unavailable("B", "buffer_attribution_unavailable"),
		"dirty": _dirty_unavailable(), "diagnostics": []}


func collect(session: String, project: String, path: String) -> Dictionary:
	_session = session
	_project = project
	_path = path
	var started := Time.get_ticks_usec()
	var builtin := _path.contains("::")
	if not builtin and not _path.ends_with(".gd"):
		return _invalid_target(started)
	var script_editor := EditorInterface.get_script_editor()
	var before := _scan(script_editor)
	var initial_stamp := _stamp(started, Time.get_ticks_usec())
	var index: int = before.index
	var script: GDScript = before.script
	var target_unique: bool = index >= 0 and before.paths.count(_path) == 1 and script != null
	var resource := _source_unavailable("R",
		"resource_unreadable" if index >= 0 and not target_unique else "resource_not_loaded")
	var buffer := _source_unavailable("B", "buffer_attribution_unavailable")
	var dirty := _dirty_unavailable()
	var identity: Variant = null
	var editor: ScriptEditorBase
	var code: CodeEdit
	var version := -1
	var resource_text := ""
	var buffer_text := ""
	var unsaved := {}
	var buffer_witness := {}
	var initial_changes: Array[Dictionary] = []
	var initial_check_available := true
	if target_unique and script.resource_path == _path:
		var resource_start := Time.get_ticks_usec()
		resource_text = script.source_code
		resource = _read_source("R", resource_text, resource_start, _witness(script, null, null))
		if script.resource_path == _path and before.unique and before.matched:
			var possible: Variant = before.editors[index]
			if possible is ScriptEditorBase and is_instance_valid(possible):
				editor = possible
				var base: Variant = editor.get_base_editor()
				if base is CodeEdit and is_instance_valid(base):
					code = base
					var buffer_start := Time.get_ticks_usec()
					version = code.get_version()
					buffer_text = code.text
					buffer_witness = _witness(script, editor, code)
					buffer = _read_source("B", buffer_text, buffer_start, buffer_witness)
					unsaved = _unsaved(script_editor, before.paths)
	var after := _scan(script_editor)
	var stable_script: bool = target_unique and after.paths.count(_path) == 1 \
		and after.script != null and after.script.get_instance_id() == script.get_instance_id() \
		and script.resource_path == _path
	if target_unique:
		# The first enumeration and its actual objects are the original identity.
		# Preserve their evidence for the reducer to invalidate if they change
		# during collection, rather than silently replacing it with another tab.
		identity = {"kind": "builtin_gdscript" if builtin else "external_gdscript", "resource_path": _path,
			"script_instance_id": String.num_uint64(before.script_ids[index])}
		if not buffer_witness.is_empty():
			identity.editor_instance_id = buffer_witness.editor_instance_id
			identity.buffer_instance_id = buffer_witness.buffer_instance_id
			if not unsaved.is_empty() and unsaved.known:
				dirty = {"availability": "observed", "state": "dirty" if unsaved.dirty else "clean",
					"collection": unsaved.stamp, "witness": buffer_witness}
		if not stable_script:
			if script.resource_path != _path or after.index >= 0:
				initial_changes.append({"surface": "document", "code": "identity_changed"})
			elif after.unique:
				initial_changes.append({"surface": "document", "code": "document_closed"})
			else:
				initial_check_available = false
		elif code != null and not _same_target_editor(before, after):
			if after.unique and after.matched:
				initial_changes.append({"surface": "document", "code": "identity_changed"})
			else:
				initial_check_available = false
		elif resource.availability == "observed" and script.source_code != resource_text:
			initial_changes.append({"surface": "R", "code": "source_changed"})
		if code != null and (initial_changes.is_empty() or initial_changes[0].surface == "R") \
				and is_instance_valid(code) and is_instance_valid(editor) \
				and editor.get_base_editor() == code and (code.get_version() != version \
				or (buffer.availability == "observed" and code.text != buffer_text)):
			initial_changes.append({"surface": "B", "code": "source_changed"})
	var valid_stamp := _stamp(started, Time.get_ticks_usec())
	var open_fact := _unknown_fact("open_state_unknown")
	var valid_fact := _unknown_fact()
	if target_unique:
		open_fact = _fact("open", initial_stamp)
		valid_fact = _fact("valid", initial_stamp)
	elif index == -1 and after.index == -1 and before.unique and after.unique \
			and before.matched and after.matched:
		# Absence from a stable, complete enumeration proves closed, not loaded.
		open_fact = _fact("not_open", valid_stamp)
		buffer = {"authority": "B", "availability": "not_applicable", "reason": _reason(
			"document_not_open", "Observe an open document to obtain buffer text")}
		dirty = {"availability": "not_applicable", "state": "not_applicable", "reason": _reason(
			"document_not_open", "Observe an open document to obtain buffer dirty state")}
	if index == -1 and after.index == -1:
		# Cache evidence is independent of open-state attribution. A mixed or
		# unassignable tab list must not erase an already-present Resource.
		var cached_start := Time.get_ticks_usec()
		var cached: Variant = ResourceLoader.get_cached_ref(_path)
		if cached is GDScript and cached.resource_path == _path:
			resource_text = cached.source_code
			resource = _read_source("R", resource_text, cached_start, _witness(cached, null, null))
			identity = {"kind": "builtin_gdscript" if builtin else "external_gdscript", "resource_path": _path,
				"script_instance_id": String.num_uint64(cached.get_instance_id())}
			valid_fact = _fact("valid", _stamp(cached_start, Time.get_ticks_usec()))
			script = cached
	if builtin and identity == null:
		# Resolving a missing built-in identity would require loading/parsing its
		# container. Refuse that operation rather than manufacture observability.
		_original.clear()
		return {"v": 3, "kind": "failure", "request_id": "", "session_id": _session,
			"project_root": _project, "script_path": _path,
			"code": "unsupported_observation", "stage": "read_editor"}
	var finished := Time.get_ticks_usec()
	var result := {"v": 3, "kind": "sample", "request_id": "", "session_id": _session,
		"project_root": _project, "script_path": _path, "collection": _stamp(started, finished),
		"document": {"identity": identity, "validity": valid_fact, "open_state": open_fact},
		"R": resource, "B": buffer, "dirty": dirty, "diagnostics": []}
	_original = {"script": script, "editor": editor, "code": code,
		"before": before, "changes": initial_changes, "check_available": initial_check_available,
		"resource": resource_text if resource.availability == "observed" else null,
		"buffer": buffer_text if buffer.availability == "observed" else null,
		"version": version, "dirty": unsaved.get("dirty") if dirty.availability == "observed" else null,
		"closed": open_fact.get("value") == "not_open", "open": open_fact.get("value") == "open"}
	return result


func recheck() -> Dictionary:
	var started := Time.get_ticks_usec()
	var script_editor := EditorInterface.get_script_editor()
	var original := _original
	var current := _scan(script_editor)
	var changes: Array[Dictionary] = []
	if original.has("changes"):
		changes = original.changes
	var performed: bool = original.get("check_available", true)
	var old_script: GDScript = original.get("script")
	# A closed tab leaves freed Node references in the request-local dictionary.
	# Keep them untyped until validity is checked; a typed assignment itself fails.
	var old_editor: Variant = original.get("editor")
	var old_code: Variant = original.get("code")
	# The held Resource remains independently observable after its tab closes.
	# A document transition must not hide a separate change to that same R.
	if old_script != null and is_instance_valid(old_script) \
			and old_script.resource_path == _path and original.get("resource") != null \
			and old_script.source_code != original.resource \
			and not changes.has({"surface": "R", "code": "source_changed"}):
		changes.append({"surface": "R", "code": "source_changed"})
	if original.get("open", false):
		if not changes.is_empty() and changes[0].surface == "document":
			pass # Collection already established closure or identity replacement.
		elif old_script != null and is_instance_valid(old_script) \
				and old_script.resource_path != _path:
			changes.append({"surface": "document", "code": "identity_changed"})
		elif current.index == -1:
			if current.unique and current.matched:
				changes.append({"surface": "document", "code": "document_closed"})
			else:
				performed = false
		elif not current.unique or not current.matched:
			performed = false
		elif old_script == null or not is_instance_valid(old_script) \
				or current.script != old_script or current.paths.count(_path) != 1:
			changes.append({"surface": "document", "code": "identity_changed"})
		elif old_code != null and (not _same_target_editor(original.before, current) \
				or not is_instance_valid(old_editor) or not is_instance_valid(old_code) \
				or old_editor.get_base_editor() != old_code):
			changes.append({"surface": "document", "code": "identity_changed"})
		else:
			if old_code != null:
				if ((original.buffer != null and old_code.text != original.buffer) \
						or old_code.get_version() != original.version) \
						and not changes.has({"surface": "B", "code": "source_changed"}):
					changes.append({"surface": "B", "code": "source_changed"})
				if original.dirty != null:
					var indication := _unsaved(script_editor, current.paths)
					if indication.known:
						if indication.dirty != original.dirty:
							changes.append({"surface": "dirty", "code": "source_changed"})
					else:
						performed = false
			elif original.buffer != null or original.dirty != null:
				performed = false
	elif original.get("closed", false):
		if current.index >= 0:
			if current.unique:
				changes.append({"surface": "document", "code": "identity_changed"})
			else:
				performed = false
		elif not current.unique or not current.matched:
			performed = false
		if old_script != null and is_instance_valid(old_script):
			if old_script.resource_path != _path:
				changes.append({"surface": "document", "code": "identity_changed"})
	else:
		performed = false
	var finished := Time.get_ticks_usec()
	_original.clear()
	var result := {"v": 3, "kind": "recheck", "request_id": "", "session_id": _session,
		"project_root": _project, "script_path": _path, "collection": _stamp(started, finished),
		"checks": "performed" if performed else "unavailable", "detected_changes": changes}
	if not performed:
		result.reason = "unavailable"
	return result


func clear() -> void:
	_original.clear()
	_session = ""
	_project = ""
	_path = ""
