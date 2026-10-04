@tool
extends Node

# Own only the lifetime/slot. Native owns all observations, guards and effects.
enum Lifecycle { IDLE, OWNED, ENTERED, CLOSING }
var _state := Lifecycle.IDLE
var _native: Dictionary = {}
var _bridge: Node
var _request := ""
var _session := ""
var _expiry := 0


func configure(native_api: Dictionary, bridge: Node) -> void:
	if _state != Lifecycle.IDLE:
		return
	_native = native_api
	_bridge = bridge
	set_process(not _native.is_empty())


func is_active_stage() -> bool:
	return _state in [Lifecycle.ENTERED, Lifecycle.CLOSING]


func _release() -> void:
	_request = ""
	_session = ""
	_expiry = 0
	_state = Lifecycle.IDLE
	if is_instance_valid(_bridge):
		_bridge.call("_release_operation", self)
	if not is_inside_tree():
		_native = {}
		_bridge = null
		call_deferred("free")


func _validation(result: Dictionary) -> Dictionary:
	var context: Variant = result.get("context")
	if context is Dictionary and context.get("projection") is Dictionary and is_instance_valid(_bridge):
		var projection: Dictionary = context.projection
		if projection.get("warnings") is Dictionary and projection.get("global_classes") is Array:
			var warnings: Dictionary = projection.warnings.duplicate()
			warnings.provenance = {"source": "editor_project_settings",
				"project_root": _bridge.get("_project"), "session_id": _session}
			context.validation = {"executable": OS.get_executable_path(), "warnings": warnings,
				"global_classes": projection.global_classes}
	return result


func _enter(operation: String, arguments: Array) -> Dictionary:
	_state = Lifecycle.ENTERED
	var result: Variant = _native[operation].callv(arguments)
	if _state == Lifecycle.CLOSING:
		var terminal: Variant = _native["closed_abort"].call(_request)
		if terminal is Dictionary and terminal.get("native") != null:
			result = terminal
		_release()
		return result if result is Dictionary else {"status": "unavailable", "reason": "native_unavailable"}
	_state = Lifecycle.OWNED
	return _validation(result) if result is Dictionary else {"status": "unavailable", "reason": "native_unavailable"}


func _correlation(correlation: Dictionary) -> bool:
	if _native.is_empty() or not is_instance_valid(_bridge) or _state != Lifecycle.IDLE:
		return false
	var expiry: Variant = correlation.get("expiry_tick_us")
	var now := Time.get_ticks_usec()
	return _bridge.call("_valid_id", correlation.get("request_id")) \
		and correlation.get("session_id") == _bridge.get("_session") \
		and _bridge.call("_valid_decimal", expiry) and expiry.length() <= 19 \
		and expiry.is_valid_int() and expiry.to_int() > now and expiry.to_int() - now <= 9000000


func inspect(path: String, correlation: Dictionary) -> Dictionary:
	if not _correlation(correlation) or not _bridge.get("_active").is_empty():
		return {"status": "refused", "reason": "slot_busy"}
	_request = correlation.request_id
	_session = correlation.session_id
	_expiry = correlation.expiry_tick_us.to_int()
	# A transient slot protects synchronous entered-call shutdown, not a read lease.
	if not _bridge.call("_claim_operation", self):
		_release()
		return {"status": "refused", "reason": "slot_busy"}
	var result := _enter("closed_inspect", [path, correlation])
	if _state != Lifecycle.IDLE:
		_release()
	return result


func prepare(path: String, expected: Dictionary, desired: String, correlation: Dictionary) -> Dictionary:
	if not _correlation(correlation):
		return {"status": "refused", "reason": "wrong_attempt"}
	var active: Dictionary = _bridge.get("_active")
	if active.get("operation_owner") != self or active.get("request_id") != correlation.request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	_request = correlation.request_id
	_session = correlation.session_id
	_expiry = correlation.expiry_tick_us.to_int()
	# JSON numeric nanoseconds must not arrive at the integer-only native ABI as floats.
	var converted := expected.duplicate(true)
	converted.file_revision.mtime.nanoseconds = int(converted.file_revision.mtime.nanoseconds)
	converted.file_revision.ctime.nanoseconds = int(converted.file_revision.ctime.nanoseconds)
	var result := _enter("closed_prepare", [path, converted, desired, correlation])
	if _state == Lifecycle.OWNED and result.get("status") != "prepared":
		cancel(_request)
	return result


func _owned(request: String, operation: String, arguments: Array) -> Dictionary:
	if _state != Lifecycle.OWNED or request != _request:
		return {"status": "refused", "reason": "wrong_attempt"}
	var active: Dictionary = _bridge.get("_active") if is_instance_valid(_bridge) else {}
	if active.get("operation_owner") != self or active.get("request_id") != request \
			or _bridge.get("_session") != _session or Time.get_ticks_usec() >= _expiry:
		return cancel(request)
	return _enter(operation, arguments)


func apply(request: String, source_hash: String, context_hash: String) -> Dictionary:
	return _owned(request, "closed_apply", [request, source_hash, context_hash])


func verify(request: String, purpose: String) -> Dictionary:
	return _owned(request, "closed_verify", [request, purpose])


func recheck(request: String, purpose: String) -> Dictionary:
	return _owned(request, "closed_recheck", [request, purpose])


func _terminal(request: String, operation: String) -> Dictionary:
	if _state == Lifecycle.IDLE or request != _request:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state == Lifecycle.CLOSING:
		return {"status": "unavailable", "reason": "active_native_stage"}
	var entered := _state == Lifecycle.ENTERED
	_state = Lifecycle.CLOSING
	var result: Variant = _native[operation].call(request)
	if not entered:
		_release()
	return result if result is Dictionary else {"status": "unavailable", "reason": "native_unavailable"}


func finish(request: String) -> Dictionary:
	return _terminal(request, "closed_finish")


func cancel(request: String) -> Dictionary:
	return _terminal(request, "closed_abort")


func cancel_owned() -> void:
	if _state != Lifecycle.IDLE:
		cancel(_request)


func _process(_delta: float) -> void:
	if _state == Lifecycle.OWNED and Time.get_ticks_usec() >= _expiry:
		cancel_owned()
	elif _state == Lifecycle.IDLE and not _native.is_empty():
		_native["closed_expire"].call()


func _exit_tree() -> void:
	set_process(false)
	cancel_owned()
	if _state == Lifecycle.IDLE:
		_native = {}
		_bridge = null
