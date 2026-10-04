@tool
extends RefCounted

const EXPECTED_FIELDS := ["project_device", "project_inode", "file_revision", "close_epoch", "resource"]
const REVISION_FIELDS := ["device", "inode", "utf8_bytes", "sha256", "mtime", "ctime"]
const RESOURCE_FIELDS := ["state", "instance_id", "path", "source_sha256", "utf8_bytes", "edited", "profile_sha256"]
var _bridge: Node


func configure(bridge: Node) -> void:
	_bridge = bridge


func _exact(value: Variant, fields: Array) -> bool:
	return value is Dictionary and value.size() == fields.size() and value.has_all(fields)


func _decimal(value: Variant) -> bool:
	return _bridge.call("_valid_decimal", value)


func _length(value: Variant) -> bool:
	return _decimal(value) and value.length() <= 6 and value.to_int() <= 512 * 1024


func _time(value: Variant) -> bool:
	if not _exact(value, ["seconds", "nanoseconds"]) or not (value.seconds is String) \
			or not _bridge.call("_integral_control", value.nanoseconds, 0, 999999999):
		return false
	var seconds: String = value.seconds
	if seconds.begins_with("-"):
		var magnitude := seconds.substr(1)
		return seconds != "-0" and _decimal(magnitude) \
			and (magnitude.length() < 19 or (magnitude.length() == 19 and magnitude <= "9223372036854775808"))
	return _decimal(seconds) and (seconds.length() < 19 or (seconds.length() == 19 and seconds <= "9223372036854775807"))


func _expected(value: Variant, path: String) -> bool:
	if not _exact(value, EXPECTED_FIELDS) or not _exact(value.file_revision, REVISION_FIELDS) \
			or not _exact(value.resource, RESOURCE_FIELDS):
		return false
	for key in ["project_device", "project_inode", "close_epoch"]:
		if not _decimal(value[key]):
			return false
	var revision: Dictionary = value.file_revision
	if not _decimal(revision.device) or not _decimal(revision.inode) or not _length(revision.utf8_bytes) \
			or not _bridge.call("_valid_hex", revision.sha256, 64) or not _time(revision.mtime) or not _time(revision.ctime):
		return false
	var resource: Dictionary = value.resource
	if resource.state == "absent":
		for key in RESOURCE_FIELDS:
			if key != "state" and resource[key] != null:
				return false
		return true
	return resource.state == "present" and _decimal(resource.instance_id) and resource.instance_id != "0" \
		and resource.path == path and typeof(resource.edited) == TYPE_BOOL and not resource.edited \
		and _length(resource.utf8_bytes) and _bridge.call("_valid_hex", resource.source_sha256, 64) \
		and _bridge.call("_valid_hex", resource.profile_sha256, 64)


func _source(value: Variant) -> bool:
	return _bridge.call("_valid_replacement", value)


func _envelope(peer: Dictionary, kind: String, started: int, result: Dictionary) -> Dictionary:
	return {"v": 6, "kind": kind, "request_id": peer.request_id,
		"session_id": peer.get("closed_session", _bridge.get("_session")),
		"project_root": peer.get("closed_project", _bridge.get("_project")),
		"script_path": peer.get("closed_path", ""), "collection": _bridge.call("_editor_stamp", started),
		"status": result.get("status", "unavailable"), "reason": result.get("reason"),
		"expiry_tick_us": result.get("expiry_tick_us"), "state": result.get("state"),
		"native": result.get("native"), "context": result.get("context")}


func _send(peer: Dictionary, reply: Dictionary, closing: bool) -> void:
	if not _bridge.get("_peers").has(peer):
		return
	peer.closed_reply = reply
	peer.closed_reply_closing = closing
	flush(peer)


func flush(peer: Dictionary) -> void:
	if not peer.has("closed_reply") or not peer.output.is_empty():
		return
	var reply: Dictionary = peer.closed_reply
	if not _bridge.call("_closed_response_ready", peer, reply.kind, reply):
		return
	var closing: bool = peer.closed_reply_closing
	peer.erase("closed_reply")
	peer.erase("closed_reply_closing")
	_bridge.call("_queue", peer, reply, closing, true)


func _refuse(peer: Dictionary, operation: String, reason: String) -> void:
	var kind := "closed_prepared" if operation == "closed_prepare" else "closed_state"
	_send(peer, _envelope(peer, kind, Time.get_ticks_usec(),
		{"status": "refused", "reason": reason}), true)


func handle(peer: Dictionary, value: Array, size: int) -> void:
	var operation: String = value[1]
	var arity := 9 if operation == "closed_prepare" else 8 if operation == "closed_apply" \
		else 7 if operation in ["closed_inspect", "closed_verify", "closed_recheck"] else 6
	if operation not in ["closed_inspect", "closed_prepare", "closed_apply", "closed_verify", "closed_recheck", "closed_finish", "closed_abort"] \
			or size > (_bridge.MAX_SELECTED_REQUEST if operation == "closed_prepare" else _bridge.MAX_FRAME) \
			or not _bridge.call("_fixed_tuple", value, arity, operation) or value[2] != peer.request_id \
			or not (value[5] is String) or value[5].to_utf8_buffer().size() > 2048:
		_bridge.call("_close_peer", peer)
		return
	var path: String = value[5]
	if operation in ["closed_inspect", "closed_prepare"]:
		var budget: Variant = value[6] if operation == "closed_inspect" else value[8]
		if peer.has("operation_owner") or peer.has("collector") or peer.has("closed_path") \
				or not _bridge.call("_integral_control", budget, 1, 9000) \
				or (operation == "closed_prepare" and (not _expected(value[6], path) or not _source(value[7]))):
			_bridge.call("_close_peer", peer)
			return
		peer.closed_path = path
		peer.closed_session = _bridge.get("_session")
		peer.closed_project = _bridge.get("_project")
		if not _bridge.get("_active").is_empty():
			_refuse(peer, operation, "slot_busy")
			return
		var owner: Variant = _bridge.get("_closed_owner")
		if not peer.auth_capabilities.edit_closed_gdscript or peer.auth_native_revision != 4 \
				or peer.auth_native_build_id != _bridge.get("_native_build_id") or not is_instance_valid(owner):
			_refuse(peer, operation, "unsupported_capability")
			return
		if not path.ends_with(".gd") or path.contains("::") or not _bridge.call("_safe_locator", path):
			_refuse(peer, operation, "out_of_project")
			return
		if operation == "closed_prepare" and not _bridge.call("_claim_operation", owner, peer):
			_refuse(peer, operation, "slot_busy")
			return
		peer.expiry_tick_us = Time.get_ticks_usec() + int(budget) * 1000
		peer.closed_tuple = value
		peer.pending = operation
		return
	if _bridge.get("_active") != peer or peer.get("operation_owner") != _bridge.get("_closed_owner") \
			or path != peer.get("closed_path") or peer.has("pending") or peer.has("closed_reply") \
			or peer.get("closed_terminal", false) or Time.get_ticks_usec() >= int(peer.expiry_tick_us):
		_bridge.call("_close_peer", peer)
		return
	var legal := true
	if operation == "closed_apply":
		legal = peer.get("closed_stage") == "preflight" and not peer.get("closed_apply_started", false) \
			and _bridge.call("_valid_hex", value[6], 64) and _bridge.call("_valid_hex", value[7], 64)
	elif operation in ["closed_verify", "closed_recheck"]:
		legal = value[6] is String and value[6] in ["preflight", "unchanged", "post_change"]
		if legal and operation == "closed_verify":
			legal = not peer.has("closed_verified") and ((value[6] == "preflight" and peer.get("closed_stage") == "prepared") \
				or (value[6] == "unchanged" and peer.get("closed_stage") in ["prepared", "preflight", "unchanged"] and not peer.get("closed_apply_started", false)) \
				or (value[6] == "post_change" and peer.get("closed_apply_started", false)))
		elif legal:
			legal = peer.get("closed_verified") == value[6]
	if not legal:
		_bridge.call("_close_peer", peer)
		return
	if operation in ["closed_finish", "closed_abort"]:
		peer.closed_terminal = true
	peer.closed_tuple = value
	peer.pending = operation


func collect(peer: Dictionary) -> void:
	var operation: String = peer.pending
	var tuple: Array = peer.closed_tuple
	var stage := operation.trim_prefix("closed_")
	if operation in ["closed_verify", "closed_recheck"]:
		stage += ":" + tuple[6]
	if not peer.output.is_empty() or peer.has("closed_reply") or not _bridge.call("_closed_ready", peer, stage):
		return
	if Time.get_ticks_usec() >= int(peer.expiry_tick_us):
		_bridge.call("_close_peer", peer)
		return
	var started := Time.get_ticks_usec()
	peer.erase("pending")
	peer.erase("closed_tuple")
	var owner: Node = _bridge.get("_closed_owner")
	var result: Dictionary
	var kind: String
	var closing := false
	var correlation := {"request_id": peer.request_id, "session_id": peer.closed_session,
		"expiry_tick_us": str(peer.expiry_tick_us)}
	if operation == "closed_inspect":
		result = owner.call("inspect", peer.closed_path, correlation)
		kind = "closed_state"
		closing = true
	elif operation == "closed_prepare":
		result = owner.call("prepare", peer.closed_path, tuple[6], tuple[7], correlation)
		kind = "closed_prepared"
		peer.closed_stage = "prepared" if result.get("status") == "prepared" else "failed"
		closing = result.get("status") != "prepared"
	elif operation == "closed_apply":
		peer.closed_apply_started = true
		result = owner.call("apply", peer.request_id, tuple[6], tuple[7])
		kind = "closed_applied"
		peer.closed_stage = "applied" if result.get("status") == "applied" else "failed"
	elif operation == "closed_verify":
		result = owner.call("verify", peer.request_id, tuple[6])
		kind = "closed_verified"
		peer.closed_verified = tuple[6]
		peer.closed_stage = "verified" if result.get("status") == "verified" else "failed"
	elif operation == "closed_recheck":
		result = owner.call("recheck", peer.request_id, tuple[6])
		kind = "closed_rechecked"
		peer.erase("closed_verified")
		peer.closed_stage = tuple[6] if result.get("status") == "rechecked" else "failed"
	else:
		result = owner.call("finish" if operation == "closed_finish" else "cancel", peer.request_id)
		kind = "closed_finished" if operation == "closed_finish" else "closed_aborted"
		closing = true
	_send(peer, _envelope(peer, kind, started, result), closing)
