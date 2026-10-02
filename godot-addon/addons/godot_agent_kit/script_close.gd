@tool
extends Node

# The single private/native owner is shared by fixture and product consumers.
# No stage here supplies a public success verdict.
const ObservationScript = preload("res://addons/godot_agent_kit/observation.gd")
signal completion_ready(result: Dictionary)

enum Lifecycle { IDLE, OWNED, ENTERED, CLOSING }
enum WaitState { UNUSED, PENDING, ANSWERED }
var _state := Lifecycle.IDLE
var _wait_state := WaitState.UNUSED
var _native: Dictionary = {}
var _bridge: Node
var _request := ""
var _session := ""
var _path := ""
var _expiry := 0
var _collector: RefCounted
var _retirement: Dictionary = {}


func configure(native_api: Dictionary, bridge: Node) -> void:
	if _state != Lifecycle.IDLE:
		return
	_native = native_api
	_bridge = bridge
	set_process(not _native.is_empty())


func is_active_stage() -> bool:
	if _state in [Lifecycle.ENTERED, Lifecycle.CLOSING]:
		return true
	if _state == Lifecycle.OWNED and not _native.is_empty():
		var result: Variant = _native["close_status"].call(_request)
		return result is Dictionary and result.get("reason") == "slot_busy"
	return false


func _clear_collector() -> void:
	if _collector != null:
		_collector.clear()
		_collector = null


func _release_attempt() -> void:
	_clear_collector()
	_retirement = {}
	_request = ""
	_session = ""
	_path = ""
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
		return {"status": "refused", "reason": "busy"}
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
	var reserved: bool = active.get("operation_owner") == self and active.get("request_id") == request_id
	if not reserved and not _bridge.call("_claim_operation", self):
		return {"status": "refused", "reason": "busy"}
	_request = request_id
	_session = session_id
	_path = path
	_expiry = expiry.to_int()
	_wait_state = WaitState.UNUSED
	_state = Lifecycle.OWNED
	active = _bridge.get("_active")
	active.request_id = _request
	active.expiry_tick_us = _expiry
	var result := _call_owned(_request, "close_inspect", [path, correlation])
	result.sample = _sample(true)
	if _state == Lifecycle.OWNED and result.get("status") != "inspected":
		cancel(_request)
	return result


func _call_owned(request_id: String, operation: String, arguments: Array) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state != Lifecycle.OWNED:
		return {"status": "refused", "reason": "active_native_stage"}
	var active: Dictionary = _bridge.get("_active") if is_instance_valid(_bridge) else {}
	if active.get("operation_owner") != self or active.get("request_id") != _request \
			or _bridge.get("_session") != _session or Time.get_ticks_usec() >= _expiry:
		return cancel(request_id)
	_state = Lifecycle.ENTERED
	var result: Variant = _native[operation].callv(arguments)
	if _state == Lifecycle.CLOSING:
		var terminal: Variant = _native["close_abort"].call(request_id)
		var retained: Dictionary = terminal if terminal is Dictionary and terminal.get("native") != null \
			else (result if result is Dictionary else {"status": "unavailable", "reason": "native_unavailable"})
		_resolve_wait(retained, "unavailable")
		_release_attempt()
		return retained
	_state = Lifecycle.OWNED
	if not (result is Dictionary):
		var terminal := cancel(request_id)
		terminal.status = "unavailable"
		return terminal
	return result


func _sample(retain: bool) -> Variant:
	if _state != Lifecycle.OWNED or not is_instance_valid(_bridge) or Time.get_ticks_usec() >= _expiry:
		return null
	var scope: PackedStringArray = _bridge.call("_scope_witness", _path)
	if scope.is_empty():
		return null
	var collector := ObservationScript.new()
	var sample: Dictionary = collector.collect(_session, _bridge.get("_project"), _path)
	sample.request_id = _request
	if _bridge.call("_scope_witness", _path) != scope or sample.get("kind") != "sample":
		collector.clear()
		return null
	if retain:
		_clear_collector()
		_collector = collector
	else:
		collector.clear()
	return sample


func _native_length(record: Dictionary) -> Dictionary:
	# JSON numbers arrive as floats. Do not narrow a fractional, Boolean or
	# out-of-range value into an integer accepted by the native ABI.
	var length: Variant = record.get("utf8_bytes")
	if typeof(length) == TYPE_FLOAT and is_instance_valid(_bridge) \
			and _bridge.call("_integral_control", length, 0, ObservationScript.SOURCE_LIMIT):
		var converted := record.duplicate()
		converted.utf8_bytes = int(length)
		return converted
	return record


func prepare(request_id: String, source: String, capture: Dictionary) -> Dictionary:
	if _state != Lifecycle.OWNED or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	var result := _call_owned(request_id, "close_prepare", [request_id, source, _native_length(capture)])
	result.sample = _sample(true)
	result.validation = _validation(result.get("context"))
	return result


func _validation(context: Variant) -> Variant:
	if _state != Lifecycle.OWNED or not is_instance_valid(_bridge) or not (context is Dictionary):
		return null
	var documents: Variant = context.get("documents")
	if not (documents is Array) or documents.size() > 7:
		return null
	var inputs := []
	for document in documents:
		if not (document is Dictionary) or document.get("kind") != "current_gdscript":
			return null
		var projection: Variant = document.get("projection")
		if not (projection is Dictionary) or not (projection.get("warnings") is Dictionary) \
				or not (projection.get("global_classes") is Array):
			return null
		var warnings: Dictionary = projection.warnings.duplicate()
		warnings.provenance = {"source": "editor_project_settings",
			"project_root": _bridge.get("_project"), "session_id": _session}
		inputs.append({"document_path": projection.path, "script_id": projection.script_id,
			"editor_id": projection.editor_id, "buffer_id": projection.buffer_id,
			"executable": OS.get_executable_path(), "warnings": warnings,
			"global_classes": projection.global_classes})
	return inputs


func advance(request_id: String, guard_sha256: String, receipt_bindings: Array) -> Dictionary:
	if _state != Lifecycle.OWNED or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	# No observation-held target Resource may manufacture post-close retention.
	_clear_collector()
	var native_receipts := receipt_bindings
	if receipt_bindings.size() <= 7:
		for index in receipt_bindings.size():
			var record: Variant = receipt_bindings[index]
			if record is Dictionary:
				var converted := _native_length(record)
				if not is_same(converted, record):
					if is_same(native_receipts, receipt_bindings):
						native_receipts = receipt_bindings.duplicate()
					native_receipts[index] = converted
	return _call_owned(request_id, "close_advance", [request_id, guard_sha256, native_receipts])


func status(request_id: String) -> Dictionary:
	return _call_owned(request_id, "close_status", [request_id])


func verify(request_id: String, purpose: String) -> Dictionary:
	if _state != Lifecycle.OWNED or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	var result := _call_owned(request_id, "close_verify", [request_id, purpose])
	result.sample = _sample(true)
	return result


func recheck(request_id: String, purpose: String) -> Dictionary:
	if _state != Lifecycle.OWNED or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	var observed: Variant = null
	if _collector != null and is_instance_valid(_bridge) and Time.get_ticks_usec() < _expiry:
		var scope: PackedStringArray = _bridge.call("_scope_witness", _path)
		if not scope.is_empty():
			observed = _collector.recheck()
			observed.request_id = request_id
			if _bridge.call("_scope_witness", _path) != scope:
				observed.checks = "unavailable"
				observed.reason = "unavailable"
	_clear_collector()
	var result := _call_owned(request_id, "close_recheck", [request_id, purpose])
	result.recheck = observed
	return result


func _continuation(result: Dictionary) -> Dictionary:
	var facts: Variant = result.get("native")
	var continuation: Variant = facts.get("continuation") if facts is Dictionary else result.get("continuation")
	return continuation if continuation is Dictionary else {}


func _wait_reply(result: Dictionary, outcome: String) -> Dictionary:
	return {"status": outcome, "reason": result.get("reason"), "native": result.get("native"),
		"expiry_tick_us": result.get("expiry_tick_us"), "continuation": _continuation(result),
		"protection": result.native.get("protection") if result.get("native") is Dictionary else result.get("protection")}


func _resolve_wait(result: Dictionary, outcome: String) -> void:
	if _wait_state == WaitState.PENDING:
		_wait_state = WaitState.ANSWERED
		completion_ready.emit(_wait_reply(result, outcome))


func wait_for_completion(request_id: String) -> Dictionary:
	if _state != Lifecycle.OWNED or _request != request_id:
		return {"status": "unavailable", "reason": "wrong_attempt"}
	if _wait_state != WaitState.UNUSED:
		return {"status": "unavailable", "reason": "duplicate_wait"}
	_wait_state = WaitState.ANSWERED
	var result := status(request_id)
	var native: Variant = result.get("native")
	if not (native is Dictionary) or native.get("entered") != true \
			or native.get("phase") not in ["returned", "settling", "settled", "terminal"]:
		if _state == Lifecycle.OWNED:
			cancel(request_id)
		return _wait_reply(result, "unavailable")
	var state: Variant = _continuation(result).get("state")
	if state in ["completed", "not_applicable"]:
		return _wait_reply(result, "completed")
	if state == "invalidated":
		return _wait_reply(result, "invalidated")
	if state != "pending" or _state != Lifecycle.OWNED:
		return _wait_reply(result, "unavailable")
	_wait_state = WaitState.PENDING
	return await completion_ready


func finish(request_id: String) -> Dictionary:
	return _retire(request_id, "close_finish")


func cancel(request_id: String) -> Dictionary:
	return _retire(request_id, "close_abort")


func _retire(request_id: String, operation: String) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state == Lifecycle.CLOSING:
		return {"status": "refused", "reason": "closing_after_active_stage"}
	var called := _state == Lifecycle.ENTERED
	var entered := is_active_stage()
	var expired := Time.get_ticks_usec() >= _expiry
	_state = Lifecycle.CLOSING
	var result: Variant = _native[operation].call(request_id)
	var facts: Dictionary = result if result is Dictionary else {"status": "unavailable", "reason": "native_unavailable"}
	_retirement = facts
	if not entered:
		_resolve_wait(facts, "expired" if expired else "unavailable")
		_release_attempt()
	elif not called:
		# An independent native signal callback still owns the slot. Finish
		# retirement after its stack returns, even when this node is detached.
		call_deferred("_finish_retirement")
	return facts


func _finish_retirement() -> void:
	if _state != Lifecycle.CLOSING:
		return
	var status_reply: Variant = _native["close_status"].call(_request)
	if status_reply is Dictionary and status_reply.get("reason") == "slot_busy":
		call_deferred("_finish_retirement")
		return
	var terminal: Variant = _native["close_abort"].call(_request)
	if terminal is Dictionary and terminal.get("native") != null:
		_retirement = terminal
	_resolve_wait(_retirement, "expired" if Time.get_ticks_usec() >= _expiry else "unavailable")
	_retirement = {}
	_release_attempt()


func cancel_owned() -> void:
	if _state != Lifecycle.IDLE:
		cancel(_request)


func _process(_delta: float) -> void:
	if _state == Lifecycle.OWNED and Time.get_ticks_usec() >= _expiry:
		cancel(_request)
		return
	if _state == Lifecycle.OWNED and _wait_state == WaitState.PENDING:
		var result := status(_request)
		var state: Variant = _continuation(result).get("state")
		if state in ["completed", "not_applicable"]:
			_resolve_wait(result, "completed")
		elif state == "invalidated":
			_resolve_wait(result, "invalidated")
		elif state != "pending":
			_resolve_wait(result, "unavailable")


func _exit_tree() -> void:
	set_process(false)
	cancel_owned()
	if _state == Lifecycle.IDLE:
		_native = {}
		_bridge = null
