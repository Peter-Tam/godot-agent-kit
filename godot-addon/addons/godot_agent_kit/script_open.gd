@tool
extends Node

# Native code owns admission, lifecycle ordering and effect facts. This node only
# holds the actual bridge slot and cannot outlive entered native cleanup.
enum Lifecycle { IDLE, OWNED, ENTERED, CLOSING }
var _state := Lifecycle.IDLE
var _native: Dictionary = {}
var _bridge: Node
var _request := ""
var _session := ""
var _expiry := 0
var _product_attempt := false


func configure(native_api: Dictionary, bridge: Node) -> void:
	if _state != Lifecycle.IDLE:
		return
	_native = native_api
	_bridge = bridge
	set_process(not _native.is_empty())


func is_active_stage() -> bool:
	return _state in [Lifecycle.ENTERED, Lifecycle.CLOSING]


func _release_attempt() -> void:
	_request = ""
	_session = ""
	_product_attempt = false
	_expiry = 0
	_state = Lifecycle.IDLE
	if is_instance_valid(_bridge):
		_bridge.call("_release_operation", self)
	if not is_inside_tree():
		_native = {}
		_bridge = null
		call_deferred("free")


func inspect(path: String, correlation: Dictionary) -> Dictionary:
	if _native.is_empty() or not is_instance_valid(_bridge):
		return {"status": "refused", "reason": "native_unavailable"}
	if _state != Lifecycle.IDLE:
		return {"status": "busy", "reason": "slot_busy"}
	var request_id: Variant = correlation.get("request_id")
	var session_id: Variant = correlation.get("session_id")
	var expiry: Variant = correlation.get("expiry_tick_us")
	var now := Time.get_ticks_usec()
	if not _bridge.call("_valid_id", request_id) or not (session_id is String) \
			or session_id != _bridge.get("_session"):
		return {"status": "refused", "reason": "wrong_attempt"}
	if not _bridge.call("_valid_decimal", expiry) or expiry.length() > 19 \
			or not expiry.is_valid_int() or expiry.to_int() <= now or expiry.to_int() - now > 9000000:
		return {"status": "refused", "reason": "invalid_expiry"}
	var active: Dictionary = _bridge.get("_active")
	var admitted: bool = active.get("operation_owner") == self \
		and active.get("request_id") == request_id and active.get("open_path") == path \
		and active.get("expiry_tick_us") == expiry.to_int() and active.get("pending") == null
	if not admitted and not _bridge.call("_claim_operation", self):
		return {"status": "busy", "reason": "slot_busy"}
	active = _bridge.get("_active")
	_product_attempt = admitted
	_request = request_id
	_session = session_id
	_expiry = expiry.to_int()
	_state = Lifecycle.OWNED
	active.request_id = _request
	active.expiry_tick_us = _expiry
	var result := _call_owned(_request, "open_inspect", [path, correlation])
	if _state == Lifecycle.OWNED and result.get("status") != "inspected":
		cancel(_request)
	return result


func _call_owned(request_id: String, operation: String, arguments: Array) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state != Lifecycle.OWNED:
		return {"status": "busy", "reason": "active_native_stage"}
	var active: Dictionary = _bridge.get("_active") if is_instance_valid(_bridge) else {}
	if active.get("operation_owner") != self or active.get("request_id") != _request \
			or _bridge.get("_session") != _session:
		return cancel(request_id)
	if Time.get_ticks_usec() >= _expiry:
		return cancel(request_id)
	_state = Lifecycle.ENTERED
	var result: Variant = _native[operation].callv(arguments)
	if _state == Lifecycle.CLOSING:
		# The reentrant abort marked discard but retained native references. Only
		# now can the boundary finish cleanup and release this entered-call slot.
		var terminal: Variant = _native["open_abort"].call(request_id)
		_release_attempt()
		if terminal is Dictionary and terminal.get("terminal_discard") == true:
			return terminal
		# Session close may have already performed deferred cleanup. Preserve
		# native stage facts instead of a wrong-attempt cleanup reply.
		return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}
	_state = Lifecycle.OWNED
	if not (result is Dictionary):
		cancel(request_id)
		return {"status": "partial", "reason": "native_unavailable"}
	return result


func prepare(request_id: String, source: String, capture: Dictionary) -> Dictionary:
	return _call_owned(request_id, "open_prepare", [request_id, source, capture])


func advance(request_id: String, stage: String, source_hash: String, context_hash: String) -> Dictionary:
	if _product_attempt:
		return {"status": "refused", "reason": "wrong_attempt_or_stage"}
	return _call_owned(request_id, "open_advance", [request_id, stage, source_hash, context_hash])


func advance_bound(request_id: String, stage: String, source_hash: String, context_hash: String) -> Dictionary:
	var active: Dictionary = _bridge.get("_active") if is_instance_valid(_bridge) else {}
	if not _product_attempt or active.get("operation_owner") != self \
			or active.get("request_id") != request_id or active.get("open_native_stage_authorized") != stage:
		return {"status": "refused", "reason": "wrong_attempt_or_stage"}
	active.erase("open_native_stage_authorized")
	return _call_owned(request_id, "open_advance", [request_id, stage, source_hash, context_hash])


func verify(request_id: String, purpose: String) -> Dictionary:
	return _call_owned(request_id, "open_verify", [request_id, purpose])


func recheck(request_id: String, purpose: String) -> Dictionary:
	return _call_owned(request_id, "open_recheck", [request_id, purpose])


func finish(request_id: String) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state != Lifecycle.OWNED:
		return {"status": "busy", "reason": "active_native_stage"}
	var result := _call_owned(request_id, "open_finish", [request_id])
	if _state == Lifecycle.OWNED:
		_release_attempt()
	return result


func cancel(request_id: String) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state == Lifecycle.CLOSING:
		return {"status": "busy", "reason": "closing_after_active_stage"}
	var entered := _state == Lifecycle.ENTERED
	_state = Lifecycle.CLOSING
	var result: Variant = _native["open_abort"].call(request_id)
	if not entered:
		_release_attempt()
	return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}


func cancel_owned() -> void:
	if _state != Lifecycle.IDLE:
		cancel(_request)


func _process(_delta: float) -> void:
	if _state != Lifecycle.IDLE and Time.get_ticks_usec() >= _expiry:
		cancel(_request)
	if not _native.is_empty():
		_native["open_expire"].call()


func _exit_tree() -> void:
	set_process(false)
	cancel_owned()
	if _state == Lifecycle.IDLE:
		_native = {}
		_bridge = null
