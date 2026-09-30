@tool
extends RefCounted

# Authenticated tuple/framing integration only. Native owns guards and stages;
# the caller owns admission, authorization and terminal outcome reduction.
const ObservationScript = preload("res://addons/godot_agent_kit/observation.gd")
const NATIVE_FIELDS := ["request_id", "stage", "next_stage", "cache_binding",
	"initial_compilation", "document_open", "target_parse_code", "script_instance_id",
	"editor_instance_id", "buffer_instance_id", "target_open", "terminal_discard",
	"entered", "mode", "protection", "selection"]
var _bridge: Node


func configure(bridge: Node) -> void:
	_bridge = bridge


func _envelope(peer: Dictionary, kind: String, started: int, native: Variant = null) -> Dictionary:
	var facts: Variant = null
	if native is Dictionary:
		facts = {}
		for key in NATIVE_FIELDS:
			facts[key] = native.get(key)
	return {"v": 3, "kind": kind, "request_id": peer.request_id,
		"session_id": _bridge.get("_session"), "project_root": _bridge.get("_project"),
		"script_path": peer.get("open_path", ""),
		"collection": _bridge.call("_editor_stamp", started), "native": facts,
		"status": native.get("status", "refused") if native is Dictionary else "refused",
		"reason": native.get("reason", "native_unavailable") if native is Dictionary else "native_unavailable",
		"resource_edited": native.get("resource_edited") if native is Dictionary else null,
		"expiry_tick_us": native.get("expiry_tick_us") if native is Dictionary else null}


func _cache(native: Dictionary) -> Variant:
	var cache: Variant = native.get("cache")
	if not (cache is Dictionary):
		return null
	return {"state": cache.get("state"), "script_instance_id": cache.get("script_instance_id") if cache.get("script_instance_id", "") != "" else null,
		"sha256": cache.get("sha256") if cache.get("sha256", "") != "" else null,
		"utf8_bytes": cache.get("utf8_bytes") if cache.get("utf8_bytes", "") != "" else null,
		"reason": cache.get("reason")}


func _send(peer: Dictionary, reply: Dictionary, closing: bool = false) -> void:
	if not _bridge.get("_peers").has(peer):
		return
	peer.open_reply = reply
	peer.open_reply_closing = closing
	flush(peer)


func flush(peer: Dictionary) -> void:
	if not peer.has("open_reply") or not peer.output.is_empty():
		return
	var reply: Dictionary = peer.open_reply
	if not _bridge.call("_open_response_ready", peer, reply.kind, reply):
		return
	var closing: bool = peer.open_reply_closing
	peer.erase("open_reply")
	peer.erase("open_reply_closing")
	_bridge.call("_queue", peer, reply, closing, true)


func _refuse_begin(peer: Dictionary, path: String, reason: String) -> void:
	var reply := _envelope(peer, "open_state", Time.get_ticks_usec())
	reply.script_path = path
	reply.reason = reason
	reply.merge({"sample": null, "cache": null})
	_send(peer, reply, true)


func _source_fits(value: Variant) -> bool:
	return value is String and value.length() <= 512 * 1024 and value.to_utf8_buffer().size() <= 512 * 1024


func handle(peer: Dictionary, value: Array, size: int) -> void:
	var operation: String = value[1]
	var arity := 15 if operation == "open_prepare" else 9 if operation == "open_advance" else 7 if operation in ["open_begin", "open_verify", "open_recheck"] else 6
	if operation not in ["open_begin", "open_prepare", "open_advance", "open_verify", "open_recheck", "open_finish", "open_abort"] \
			or size > (_bridge.MAX_SELECTED_REQUEST if operation == "open_prepare" else _bridge.MAX_FRAME) \
			or not _bridge.call("_fixed_tuple", value, arity, operation) or value[2] != peer.request_id \
			or not (value[5] is String) or value[5].to_utf8_buffer().size() > 2048:
		_bridge.call("_close_peer", peer)
		return
	var path: String = value[5]
	if operation == "open_begin":
		if not _bridge.call("_integral_control", value[6], 1, 9000):
			_bridge.call("_close_peer", peer)
			return
		if peer.has("operation_owner") or peer.has("collector") or peer.has("open_path"):
			_bridge.call("_close_peer", peer)
			return
		if not _bridge.get("_active").is_empty():
			_refuse_begin(peer, path, "slot_busy")
			return
		var owner: Variant = _bridge.get("_open_owner")
		if not peer.auth_capabilities.open_gdscript or peer.auth_native_revision != 2 \
				or peer.auth_native_build_id != _bridge.get("_native_build_id") or not is_instance_valid(owner):
			_refuse_begin(peer, path, "unsupported_capability")
			return
		if not path.ends_with(".gd") or path.contains("::") or not _bridge.call("_safe_locator", path):
			_refuse_begin(peer, path, "out_of_project")
			return
		if not _bridge.call("_claim_operation", owner, peer):
			_refuse_begin(peer, path, "slot_busy")
			return
		peer.open_path = path
		peer.expiry_tick_us = Time.get_ticks_usec() + int(value[6]) * 1000
		peer.pending = operation
		return
	if _bridge.get("_active") != peer or peer.get("operation_owner") != _bridge.get("_open_owner") \
			or path != peer.get("open_path") or peer.has("open_reply") \
			or (peer.has("pending") and (operation not in ["open_finish", "open_abort"] \
			or peer.pending in ["open_finish", "open_abort"])):
		_bridge.call("_close_peer", peer)
		return
	if operation == "open_prepare":
		if peer.get("open_next_stage") != "prepare" or peer.get("open_branch") != "closed" \
				or value[10] not in ["absent", "present"] or not (value[11] is String) \
				or (value[10] == "absent" and value[11] != "") \
				or (value[10] == "present" and (not _bridge.call("_valid_decimal", value[11]) or value[11] == "0")) \
				or not _bridge.call("_valid_hex", value[12], 64) \
				or not _bridge.call("_valid_decimal", value[13]) or not _source_fits(value[14]):
			_bridge.call("_close_peer", peer)
			return
		for index in range(6, 10):
			if not _bridge.call("_valid_decimal", value[index]):
				_bridge.call("_close_peer", peer)
				return
		peer.open_prepare_tuple = value
	elif operation == "open_advance":
		if not (value[6] is String) or value[6] not in ["bind", "compile", "open"] \
				or value[6] != peer.get("open_next_stage") \
				or not (value[7] is String) or not (value[8] is String) \
				or (value[7] != "" and not _bridge.call("_valid_hex", value[7], 64)) \
				or (value[8] != "" and not _bridge.call("_valid_hex", value[8], 64)) \
				or (value[7] == "") != (value[8] == "") or peer.has("open_verified_purpose"):
			_bridge.call("_close_peer", peer)
			return
		peer.open_advance_tuple = value
	elif operation in ["open_verify", "open_recheck"]:
		if not (value[6] is String) or value[6] not in ["recognition", "post_open"] \
				or (value[6] == "recognition" and peer.get("open_branch") != "recognition") \
				or (value[6] == "post_open" and peer.get("open_reached_stage") != "open") \
				or (operation == "open_verify" and peer.has("open_verified_purpose")) \
				or (operation == "open_recheck" and peer.get("open_verified_purpose") != value[6]):
			_bridge.call("_close_peer", peer)
			return
		peer.open_purpose = value[6]
	peer.pending = operation


func _sample(peer: Dictionary, retain: bool = false) -> Variant:
	var scope: PackedStringArray = _bridge.call("_scope_witness", peer.open_path)
	if scope.is_empty():
		return null
	var collector := ObservationScript.new()
	var sample: Dictionary = collector.collect(_bridge.get("_session"), _bridge.get("_project"), peer.open_path)
	sample.request_id = peer.request_id
	if _bridge.call("_scope_witness", peer.open_path) != scope:
		collector.clear()
		return null
	if retain and sample.get("kind") == "sample":
		if peer.has("open_collector"):
			peer.open_collector.clear()
		peer.open_collector = collector
	else:
		collector.clear()
	return sample if sample.get("kind") == "sample" else null


func _recheck(peer: Dictionary) -> Dictionary:
	var result: Variant = null
	if peer.has("open_collector"):
		var scope: PackedStringArray = _bridge.call("_scope_witness", peer.open_path)
		if not scope.is_empty():
			result = peer.open_collector.recheck()
			result.request_id = peer.request_id
			if _bridge.call("_scope_witness", peer.open_path) != scope:
				result.checks = "unavailable"
				result.reason = "unavailable"
		peer.open_collector.clear()
		peer.erase("open_collector")
	if result == null:
		result = {"v": 3, "kind": "recheck", "request_id": peer.request_id,
			"session_id": _bridge.get("_session"), "project_root": _bridge.get("_project"),
			"script_path": peer.open_path, "collection": _bridge.call("_editor_stamp", Time.get_ticks_usec()),
			"checks": "unavailable", "detected_changes": [], "reason": "unavailable"}
	return result


func _validation(context: Variant) -> Variant:
	if not (context is Dictionary) or context.get("kind") != "current_gdscript":
		return null
	var projection: Variant = context.get("projection")
	if not (projection is Dictionary) or not (projection.get("warnings") is Dictionary) \
			or not (projection.get("global_classes") is Array):
		return null
	var warnings: Dictionary = projection.warnings.duplicate()
	warnings.provenance = {"source": "editor_project_settings", "project_root": _bridge.get("_project"),
		"session_id": _bridge.get("_session")}
	return {"executable": OS.get_executable_path(), "warnings": warnings, "global_classes": projection.global_classes}


func collect(peer: Dictionary) -> void:
	var operation: String = peer.pending
	var stage := operation.trim_prefix("open_")
	if operation == "open_advance":
		stage = peer.open_advance_tuple[6]
	elif operation in ["open_verify", "open_recheck"]:
		stage += ":" + peer.open_purpose
	if not peer.output.is_empty() or peer.has("open_reply") or not _bridge.call("_open_ready", peer, stage):
		return
	var started := Time.get_ticks_usec()
	peer.erase("pending")
	var owner: Node = peer.operation_owner
	var native: Dictionary
	var sample: Variant = null
	var scope: PackedStringArray = _bridge.call("_scope_witness", peer.open_path)
	if operation == "open_begin":
		native = owner.call("inspect", peer.open_path, {"request_id": peer.request_id,
			"session_id": _bridge.get("_session"), "expiry_tick_us": str(peer.expiry_tick_us)})
		sample = _sample(peer)
		var reply := _envelope(peer, "open_state", started, native)
		reply.merge({"sample": sample, "cache": _cache(native)})
		if _bridge.call("_scope_witness", peer.open_path) != scope:
			reply.sample = null
		peer.open_branch = "recognition" if native.get("target_open") == true else "closed"
		peer.open_next_stage = native.get("next_stage", "")
		peer.open_reached_stage = native.get("stage", "")
		var closing: bool = native.get("status") != "inspected"
		if closing:
			owner.call("cancel_owned")
		_send(peer, reply, closing)
		return
	if operation == "open_prepare":
		var tuple: Array = peer.open_prepare_tuple
		peer.erase("open_prepare_tuple")
		native = owner.call("prepare", peer.request_id, tuple[14], {"project_device": tuple[6],
			"project_inode": tuple[7], "target_device": tuple[8], "target_inode": tuple[9],
			"cache_mode": tuple[10], "cached_script_id": tuple[11], "sha256": tuple[12], "utf8_bytes": tuple[13]})
		sample = _sample(peer)
		var reply := _envelope(peer, "open_prepared", started, native)
		reply.merge({"mode": native.get("mode"), "sample": sample, "context": native.get("context"),
			"validation": _validation(native.get("context"))})
		for key in ["project_device", "project_inode", "target_device", "target_inode"]:
			reply[key] = native.get(key)
		if _bridge.call("_scope_witness", peer.open_path) != scope:
			reply.sample = null
			reply.context = null
			reply.validation = null
		peer.open_next_stage = native.get("next_stage", "")
		peer.open_reached_stage = native.get("stage", "")
		var closing: bool = native.get("status") != "prepared"
		if closing:
			owner.call("cancel_owned")
		_send(peer, reply, closing)
		return
	if operation == "open_advance":
		var tuple: Array = peer.open_advance_tuple
		peer.erase("open_advance_tuple")
		peer.open_native_stage_authorized = tuple[6]
		native = owner.call("advance_bound", peer.request_id, tuple[6], tuple[7], tuple[8])
		peer.erase("open_native_stage_authorized")
		peer.open_next_stage = native.get("next_stage", "")
		peer.open_reached_stage = native.get("stage", "")
		_send(peer, _envelope(peer, "open_progress", started, native))
		return
	if operation == "open_verify":
		native = owner.call("verify", peer.request_id, peer.open_purpose)
		sample = _sample(peer, true)
		peer.open_verified_purpose = peer.open_purpose
		var reply := _envelope(peer, "open_sample", started, native)
		reply.merge({"purpose": peer.open_purpose, "sample": sample,
			"protection": native.get("protection"), "selection": native.get("selection", "unknown")})
		if native.get("status") != "observed":
			reply.resource_edited = null
		if _bridge.call("_scope_witness", peer.open_path) != scope:
			reply.sample = null
		_send(peer, reply)
		return
	if operation == "open_recheck":
		var recheck := _recheck(peer)
		native = owner.call("recheck", peer.request_id, peer.open_purpose)
		peer.open_verified_purpose = ""
		var reply := _envelope(peer, "open_rechecked", started, native)
		reply.merge({"purpose": peer.open_purpose, "recheck": recheck,
			"protection": native.get("protection"), "selection": native.get("selection", "unknown")})
		if native.get("status") != "unchanged":
			reply.resource_edited = null
		_send(peer, reply)
		return
	if operation in ["open_finish", "open_abort"]:
		# A terminal control can discard a held begin before native inspection.
		# Release that exact claimed slot; no native attempt ever existed.
		var unentered: bool = not peer.has("open_branch")
		if unentered:
			_bridge.call("_release_operation", owner)
		else:
			native = owner.call("finish" if operation == "open_finish" else "cancel", peer.request_id)
		if peer.has("open_collector"):
			peer.open_collector.clear()
			peer.erase("open_collector")
		peer.erase("open_prepare_tuple")
		peer.erase("open_advance_tuple")
		var reply := _envelope(peer, "open_finished" if operation == "open_finish" else "open_aborted", started, native)
		reply.terminal_discard = unentered or native.get("terminal_discard") == true
		if unentered:
			reply.native = null
			reply.status = "discarded"
			reply.reason = "finished" if operation == "open_finish" else "cancelled"
		_send(peer, reply, true)
