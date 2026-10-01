@tool
extends Node

# Private read-only owner: editor facts and the existing bridge slot only.
# It never receives a script locator or enters a native/document operation.
var _bridge: Node
var _paths: Object
var _filesystem: Object
var _peer: Dictionary = {}
var _epoch := 0


func configure(bridge: Node) -> void:
	cancel_owned()
	_bridge = bridge


func available() -> bool:
	if not Engine.is_editor_hint() or not EditorInterface.has_method("get_editor_paths") \
			or not EditorInterface.has_method("get_resource_filesystem"):
		return false
	var paths: Object = EditorInterface.get_editor_paths()
	var filesystem: Object = EditorInterface.get_resource_filesystem()
	return is_instance_valid(paths) and paths.has_method("get_project_settings_dir") \
		and is_instance_valid(filesystem) and filesystem.has_method("is_scanning") \
		and filesystem.has_method("is_importing") and filesystem.has_signal("filesystem_changed")


func _register_context() -> bool:
	var paths: Object = EditorInterface.get_editor_paths()
	var filesystem: Object = EditorInterface.get_resource_filesystem()
	if _filesystem != filesystem and is_instance_valid(_filesystem) \
			and _filesystem.is_connected("filesystem_changed", _filesystem_changed):
		_filesystem.disconnect("filesystem_changed", _filesystem_changed)
	_paths = paths
	_filesystem = filesystem
	if not is_instance_valid(_paths) or not _paths.has_method("get_project_settings_dir") \
			or not is_instance_valid(_filesystem) or not _filesystem.has_method("is_scanning") \
			or not _filesystem.has_method("is_importing") or not _filesystem.has_signal("filesystem_changed"):
		return false
	if not _filesystem.is_connected("filesystem_changed", _filesystem_changed):
		_filesystem.connect("filesystem_changed", _filesystem_changed)
	return true


func _filesystem_changed() -> void:
	_epoch += 1


func _settings_directory() -> Variant:
	return _paths.call("get_project_settings_dir")


func _filesystem_facts() -> Dictionary:
	return {"scanning": _filesystem.call("is_scanning"), "importing": _filesystem.call("is_importing")}


func _envelope(peer: Dictionary, kind: String, started: int) -> Dictionary:
	return {"v": 4, "kind": kind, "request_id": peer.request_id,
		"session_id": _bridge.get("_session"), "project_root": _bridge.get("_project"),
		"collection": _bridge.call("_editor_stamp", started)}


func _state(peer: Dictionary, kind: String, started: int, status: String,
		reason: Variant = null, context: Variant = null) -> Dictionary:
	var reply := _envelope(peer, kind, started)
	reply.merge({"status": status, "reason": reason, "context": context,
		"expiry_tick_us": str(peer.expiry_tick_us) if _peer == peer else null})
	return reply


func _observe(peer: Dictionary, kind: String, started: int) -> Dictionary:
	if not _register_context():
		return _state(peer, kind, started, "unavailable", "unavailable_editor_context")
	var settings_directory: Variant = _settings_directory()
	if typeof(settings_directory) != TYPE_STRING or settings_directory.is_empty():
		return _state(peer, kind, started, "unavailable", "unavailable_editor_context")
	if settings_directory not in ["res://.godot/editor", "res://godot/editor"]:
		return _state(peer, kind, started, "refused", "unsupported_visibility_policy")
	var facts := _filesystem_facts()
	var scanning: Variant = facts.get("scanning")
	var importing: Variant = facts.get("importing")
	if typeof(scanning) != TYPE_BOOL or typeof(importing) != TYPE_BOOL:
		return _state(peer, kind, started, "unavailable", "unavailable_editor_context")
	return _state(peer, kind, started, "observed", null,
		{"policy": "godot_project_files_v1", "project_data_directory": settings_directory.get_base_dir(),
		"filesystem_epoch": str(_epoch), "scanning": scanning, "importing": importing})


func handle(peer: Dictionary, value: Array, size: int) -> void:
	var operation: String = value[1]
	var arity := 6 if operation == "discover_begin" else 5
	if operation not in ["discover_begin", "discover_recheck", "discover_finish", "discover_abort"] \
			or size > 4096 or not _bridge.call("_fixed_tuple", value, arity, operation) \
			or value[2] != peer.request_id:
		_bridge.call("_close_peer", peer)
		return
	var started := Time.get_ticks_usec()
	if operation == "discover_begin":
		if peer.has("discovery_started") or not _bridge.call("_integral_control", value[5], 1, 4500):
			_bridge.call("_close_peer", peer)
			return
		peer.discovery_started = true
		if not peer.auth_capabilities.discover_gdscripts or not available():
			_bridge.call("_queue", peer, _state(peer, "discover_state", started,
				"refused", "unsupported_discovery"), true)
			return
		if not _bridge.call("_claim_operation", self, peer):
			_bridge.call("_queue", peer, _state(peer, "discover_state", started, "refused", "busy"), true)
			return
		_peer = peer
		peer.expiry_tick_us = started + int(value[5]) * 1000
		var state := _observe(peer, "discover_state", started)
		if Time.get_ticks_usec() >= int(peer.expiry_tick_us):
			_bridge.call("_close_peer", peer)
			return
		_bridge.call("_queue", peer, state)
		return
	if _peer != peer or _bridge.get("_active") != peer or not peer.has("expiry_tick_us") \
			or started >= int(peer.expiry_tick_us):
		_bridge.call("_close_peer", peer)
		return
	if operation == "discover_recheck":
		var rechecked := _observe(peer, "discover_rechecked", started)
		if Time.get_ticks_usec() >= int(peer.expiry_tick_us):
			_bridge.call("_close_peer", peer)
			return
		_bridge.call("_queue", peer, rechecked)
		return
	var reply := _envelope(peer,
		"discover_finished" if operation == "discover_finish" else "discover_aborted", started)
	reply.terminal_discard = operation == "discover_abort"
	cancel_owned()
	_bridge.call("_queue", peer, reply, true)


func cancel_owned() -> void:
	if is_instance_valid(_filesystem) and _filesystem.is_connected("filesystem_changed", _filesystem_changed):
		_filesystem.disconnect("filesystem_changed", _filesystem_changed)
	_paths = null
	_filesystem = null
	_peer = {}
	if is_instance_valid(_bridge):
		_bridge.call("_release_operation", self)


func is_active_stage() -> bool:
	# Read-only scopes have no entered-native retention stage.
	return false


func _exit_tree() -> void:
	cancel_owned()
	_bridge = null
