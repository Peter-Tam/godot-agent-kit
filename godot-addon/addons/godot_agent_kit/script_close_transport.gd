@tool
extends RefCounted

# Transport only: the shared owner supplies facts, the caller reduces outcomes.
const NATIVE_FIELDS := ["request_id", "session_id", "script_path", "native_build_id",
	"native_api_revision", "phase", "script_instance_id", "editor_instance_id",
	"buffer_instance_id", "entry_collection", "return_collection", "entered", "close_error",
	"old_document_removed", "selection", "target_buffer", "protection", "continuation",
	"invalidated", "reason", "terminal_discard"]
const CAPTURE_FIELDS := ["project_device", "project_inode", "target_device", "target_inode",
	"source_sha256", "utf8_bytes", "basis"]
const BASIS_FIELDS := ["script_instance_id", "editor_instance_id", "buffer_instance_id", "current_version"]
const RECEIPT_FIELDS := ["request_id", "session_id", "target_path", "document_path", "script_id",
	"editor_id", "buffer_id", "source_sha256", "utf8_bytes", "guard_sha256", "context_sha256"]
var _bridge: Node


func configure(bridge: Node) -> void:
	_bridge = bridge



# Godot's JSON parser replaces duplicate object keys. Check the validated raw
# tuple first so an overwritten capture/receipt cannot reach the native ABI.
func unique_keys(bytes: PackedByteArray) -> bool:
	var stack := []
	var quoted := false
	var escaped := false
	var token := 0
	for index in bytes.size():
		var byte := bytes[index]
		if quoted:
			if escaped:
				escaped = false
			elif byte == 92:
				escaped = true
			elif byte == 34:
				quoted = false
				if not stack.is_empty() and stack[-1].object and stack[-1].key:
					var key: Variant = JSON.parse_string(bytes.slice(token, index + 1).get_string_from_utf8())
					if not (key is String) or stack[-1].keys.has(key):
						return false
					stack[-1].keys[key] = true
					stack[-1].key = false
		elif byte == 34:
			quoted = true
			token = index
		elif byte in [123, 91]:
			stack.append({"object": byte == 123, "key": byte == 123, "keys": {}})
		elif byte in [125, 93]:
			if stack.is_empty():
				return false
			stack.pop_back()
		elif byte == 44 and not stack.is_empty() and stack[-1].object:
			stack[-1].key = true
	return stack.is_empty() and not quoted

func _exact(value: Variant, fields: Array) -> bool:
	return value is Dictionary and value.size() == fields.size() and value.has_all(fields)


func _capture(value: Variant, source: Variant) -> bool:
	if not _exact(value, CAPTURE_FIELDS) or not _exact(value.basis, BASIS_FIELDS) \
			or not (source is String) or source.length() > 512 * 1024 \
			or source.to_utf8_buffer().size() > 512 * 1024 \
			or not _bridge.call("_valid_hex", value.source_sha256, 64) \
			or not _bridge.call("_integral_control", value.utf8_bytes, 0, 512 * 1024) \
			or int(value.utf8_bytes) != source.to_utf8_buffer().size() \
			or value.source_sha256 != source.sha256_text():
		return false
	for key in ["project_device", "project_inode", "target_device", "target_inode"]:
		if not _bridge.call("_valid_decimal", value[key]):
			return false
	for key in BASIS_FIELDS:
		if not _bridge.call("_valid_decimal", value.basis[key]) \
				or (key != "current_version" and value.basis[key] == "0"):
			return false
	return true


func _receipts(peer: Dictionary, value: Variant) -> bool:
	if not (value is Array) or value.size() > 7:
		return false
	var previous := ""
	for receipt in value:
		if not _exact(receipt, RECEIPT_FIELDS) or receipt.request_id != peer.request_id \
				or receipt.session_id != peer.close_session or receipt.target_path != peer.close_path \
				or not (receipt.document_path is String) or receipt.document_path <= previous \
				or receipt.document_path == peer.close_path or not receipt.document_path.ends_with(".gd") \
				or receipt.document_path.contains("::") or not _bridge.call("_safe_locator", receipt.document_path) \
				or not _bridge.call("_integral_control", receipt.utf8_bytes, 0, 512 * 1024):
			return false
		for key in ["script_id", "editor_id", "buffer_id"]:
			if not _bridge.call("_valid_decimal", receipt[key]) or receipt[key] == "0":
				return false
		for key in ["source_sha256", "guard_sha256", "context_sha256"]:
			if not _bridge.call("_valid_hex", receipt[key], 64):
				return false
		previous = receipt.document_path
	return true


func _envelope(peer: Dictionary, kind: String, started: int, result: Dictionary) -> Dictionary:
	var facts: Variant = result.get("native")
	var native: Variant = null
	if facts is Dictionary:
		native = {}
		for key in NATIVE_FIELDS:
			native[key] = facts.get(key)
	return {"v": 5, "kind": kind, "request_id": peer.request_id,
		"session_id": peer.get("close_session", _bridge.get("_session")),
		"project_root": peer.get("close_project", _bridge.get("_project")),
		"script_path": peer.get("close_path", ""),
		"collection": _bridge.call("_editor_stamp", started), "native": native,
		"status": result.get("status", "refused"), "reason": result.get("reason"),
		"expiry_tick_us": result.get("expiry_tick_us")}


func _send(peer: Dictionary, reply: Dictionary, closing: bool = false) -> void:
	if not _bridge.get("_peers").has(peer):
		return
	peer.close_reply = reply
	peer.close_reply_closing = closing
	flush(peer)


func flush(peer: Dictionary) -> void:
	if not peer.has("close_reply") or not peer.output.is_empty():
		return
	var reply: Dictionary = peer.close_reply
	if not _bridge.call("_close_response_ready", peer, reply.kind, reply):
		return
	var closing: bool = peer.close_reply_closing
	peer.erase("close_reply")
	peer.erase("close_reply_closing")
	_bridge.call("_queue", peer, reply, closing, true)


func _refuse(peer: Dictionary, path: String, reason: String) -> void:
	var reply := _envelope(peer, "close_state", Time.get_ticks_usec(), {"status": "refused", "reason": reason})
	reply.script_path = path
	reply.merge({"sample": null, "resource_edited": null, "resource_state": null, "selection": null})
	_send(peer, reply, true)


func handle(peer: Dictionary, value: Array, size: int) -> void:
	var operation: String = value[1]
	var arity := 8 if operation in ["close_prepare", "close_advance"] else 7 if operation in ["close_begin", "close_verify", "close_recheck"] else 6
	if operation not in ["close_begin", "close_prepare", "close_recheck", "close_advance", "close_wait", "close_verify", "close_finish", "close_abort"] \
			or size > (_bridge.MAX_SELECTED_REQUEST if operation == "close_prepare" else _bridge.MAX_FRAME) \
			or not _bridge.call("_fixed_tuple", value, arity, operation) or value[2] != peer.request_id \
			or not (value[5] is String) or value[5].to_utf8_buffer().size() > 2048:
		_bridge.call("_close_peer", peer)
		return
	var path: String = value[5]
	if operation == "close_begin":
		if not _bridge.call("_integral_control", value[6], 1, 9000) \
				or peer.has("operation_owner") or peer.has("collector") or peer.has("close_path"):
			_bridge.call("_close_peer", peer)
			return
		if not _bridge.get("_active").is_empty():
			_refuse(peer, path, "slot_busy")
			return
		var owner: Variant = _bridge.get("_close_owner")
		if not peer.auth_capabilities.close_gdscript or peer.auth_native_revision != 3 \
				or peer.auth_native_build_id != _bridge.get("_native_build_id") or not is_instance_valid(owner):
			_refuse(peer, path, "unsupported_capability")
			return
		if not path.ends_with(".gd") or path.contains("::") or not _bridge.call("_safe_locator", path):
			_refuse(peer, path, "out_of_project")
			return
		if not _bridge.call("_claim_operation", owner, peer):
			_refuse(peer, path, "slot_busy")
			return
		peer.close_path = path
		peer.close_session = _bridge.get("_session")
		peer.close_project = _bridge.get("_project")
		peer.expiry_tick_us = Time.get_ticks_usec() + int(value[6]) * 1000
		peer.pending = operation
		return
	if _bridge.get("_active") != peer or peer.get("operation_owner") != _bridge.get("_close_owner") \
			or path != peer.get("close_path") or peer.has("close_reply") or peer.get("close_terminal", false) \
			or Time.get_ticks_usec() >= int(peer.expiry_tick_us) \
			or (peer.has("pending") and operation not in ["close_finish", "close_abort"]) \
			or (peer.get("close_wait_pending", false) and operation not in ["close_finish", "close_abort"]):
		_bridge.call("_close_peer", peer)
		return
	var legal := true
	if operation == "close_prepare":
		legal = peer.get("close_stage") == "inspected_open" and _capture(value[7], value[6])
	elif operation == "close_advance":
		legal = peer.get("close_stage") == "pre_close" and not peer.get("close_advanced", false) \
			and _bridge.call("_valid_hex", value[6], 64) and _receipts(peer, value[7])
	elif operation == "close_wait":
		legal = peer.get("close_advanced", false) and not peer.get("close_wait_used", false)
	elif operation == "close_verify":
		legal = value[6] is String and ((value[6] == "recognition" and peer.get("close_stage") == "inspected_closed") \
			or (value[6] == "post_close" and peer.get("close_stage") == "waited") \
			or (value[6] == "survivor" and peer.has("close_stage") and peer.get("close_verified") != "survivor"))
	elif operation == "close_recheck":
		legal = value[6] is String and ((value[6] == "pre_close" and peer.get("close_stage") == "prepared") \
			or (value[6] in ["recognition", "post_close"] and peer.get("close_verified") == value[6]))
	if not legal:
		_bridge.call("_close_peer", peer)
		return
	if operation in ["close_finish", "close_abort"]:
		peer.close_terminal = true
	peer.close_tuple = value
	peer.pending = operation


func _recheck(peer: Dictionary, result: Dictionary, started: int) -> Dictionary:
	var observed: Variant = result.get("recheck")
	if observed is Dictionary:
		return observed
	return {"v": 5, "kind": "recheck", "request_id": peer.request_id,
		"session_id": peer.close_session, "project_root": peer.close_project, "script_path": peer.close_path,
		"collection": _bridge.call("_editor_stamp", started), "checks": "unavailable",
		"detected_changes": [], "reason": "unavailable"}


func collect(peer: Dictionary) -> void:
	var operation: String = peer.pending
	var stage := operation.trim_prefix("close_")
	var tuple: Array = peer.get("close_tuple", [])
	if operation in ["close_verify", "close_recheck"]:
		stage += ":" + tuple[6]
	if not peer.output.is_empty() or peer.has("close_reply") or not _bridge.call("_close_ready", peer, stage):
		return
	var started := Time.get_ticks_usec()
	peer.erase("pending")
	peer.erase("close_tuple")
	var owner: Node = peer.operation_owner
	var result: Dictionary = {}
	var kind := ""
	if operation == "close_begin":
		result = owner.call("inspect", peer.close_path, {"request_id": peer.request_id,
			"session_id": peer.close_session, "expiry_tick_us": str(peer.expiry_tick_us)})
		kind = "close_state"
		var facts: Variant = result.get("native")
		peer.close_stage = "inspected_open" if facts is Dictionary and facts.get("editor_instance_id") != null else "inspected_closed"
	elif operation == "close_prepare":
		result = owner.call("prepare", peer.request_id, tuple[6], tuple[7])
		kind = "close_prepared"
		peer.close_stage = "prepared" if result.get("status") == "prepared" else "failed"
	elif operation == "close_advance":
		peer.close_advanced = true
		result = owner.call("advance", peer.request_id, tuple[6], tuple[7])
		kind = "close_progress"
		peer.close_stage = "returned" if result.get("status") == "returned" else "failed"
	elif operation == "close_wait":
		peer.close_wait_used = true
		peer.close_wait_pending = true
		result = await owner.wait_for_completion(peer.request_id)
		if peer.get("close_terminal", false) or not _bridge.get("_peers").has(peer):
			return
		peer.close_wait_pending = false
		kind = "close_waited"
		peer.close_stage = "waited" if result.get("status") == "completed" else "failed"
	elif operation == "close_verify":
		result = owner.call("verify", peer.request_id, tuple[6])
		kind = "close_sample"
		peer.close_verified = tuple[6]
		peer.close_stage = "verified" if result.get("status") == "observed" else "failed"
	elif operation == "close_recheck":
		result = owner.call("recheck", peer.request_id, tuple[6])
		kind = "close_rechecked"
		peer.close_verified = ""
		peer.close_stage = tuple[6] if result.get("status") == "rechecked" else "failed"
	else:
		kind = "close_finished" if operation == "close_finish" else "close_aborted"
		if not peer.has("close_stage"):
			_bridge.call("_release_operation", owner)
			result = {"status": "released", "reason": null, "terminal_discard": true}
		else:
			result = owner.call("finish" if operation == "close_finish" else "cancel", peer.request_id)
	var reply := _envelope(peer, kind, started, result)
	if kind == "close_state":
		for key in ["sample", "resource_edited", "resource_state", "selection"]:
			reply[key] = result.get(key)
	elif kind == "close_prepared":
		for key in ["sample", "target_guard", "context", "guard_sha256", "validation"]:
			reply[key] = result.get(key)
	elif kind == "close_sample":
		reply.purpose = tuple[6]
		for key in ["sample", "resource_edited", "resource_state", "protection", "selection"]:
			reply[key] = result.get(key)
	elif kind == "close_rechecked":
		reply.purpose = tuple[6]
		reply.recheck = _recheck(peer, result, started)
		for key in ["protection", "selection", "resource_state"]:
			reply[key] = result.get(key)
	elif kind == "close_waited":
		reply.continuation = result.get("continuation")
		reply.protection = result.get("protection")
	elif kind in ["close_finished", "close_aborted"]:
		reply.terminal_discard = result.get("terminal_discard", false)
	_send(peer, reply, kind in ["close_finished", "close_aborted"] or (kind == "close_state" and reply.status != "inspected"))
