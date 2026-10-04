@tool
extends Node

# The private native attempt and authenticated bridge share exactly one slot.
# An entered synchronous native call retains that slot until it returns.
enum Lifecycle { IDLE, PREPARING, PREPARED, FINISHING, CLOSING }
var _state := Lifecycle.IDLE
var _native: Dictionary = {}
var _bridge: Node
var _request := ""
var _expiry := 0
var _entered_stage := false
var _product_attempt := false


func configure(native_api: Dictionary, bridge: Node) -> void:
	if _state != Lifecycle.IDLE:
		return
	_native = native_api
	_bridge = bridge
	set_process(not _native.is_empty())


func is_active_stage() -> bool:
	return _entered_stage


func inspect(path: String, original: String, desired: String, document: Dictionary) -> Dictionary:
	if _native.is_empty() or not _native.has("edit_inspect"):
		return {}
	var result: Variant = _native["edit_inspect"].call(path, original, desired, document)
	return result if result is Dictionary else {}


func _release_attempt() -> void:
	_request = ""
	_product_attempt = false
	_expiry = 0
	_state = Lifecycle.IDLE
	if is_instance_valid(_bridge):
		_bridge.call("_release_operation", self)
	if not is_inside_tree():
		_native = {}
		_bridge = null
		call_deferred("free")


func begin(path: String, expected: String, desired: String, correlation: Dictionary) -> Dictionary:
	if _native.is_empty() or not is_instance_valid(_bridge) or _state != Lifecycle.IDLE:
		return {"status": "busy", "reason": "slot_busy"}
	# The selected v6 peer claims on admission. Its exact immutable tuple must
	# still own this preparation; a private fixture may claim only an empty slot.
	var active: Dictionary = _bridge.get("_active")
	var admitted: bool = active.get("operation_owner") == self and active.get("request_id") == correlation.get("request_id") \
		and active.get("edit_path") == path and active.has("prepare_tuple")
	if admitted:
		var tuple: Array = active.prepare_tuple
		var document: Variant = correlation.get("document")
		if not (document is Dictionary) or expected.sha256_text() != tuple[16] \
				or str(expected.to_utf8_buffer().size()) != tuple[17] or desired != tuple[18] \
				or document.get("resource_instance_id") != tuple[12] \
				or document.get("editor_instance_id") != tuple[13] \
				or document.get("buffer_instance_id") != tuple[14] \
				or correlation.get("expected_version") != tuple[15]:
			return {"status": "refused", "reason": "wrong_attempt"}
	elif _bridge.call("_claim_operation", self) != true:
		return {"status": "busy", "reason": "slot_busy"}
	_product_attempt = admitted
	_state = Lifecycle.PREPARING
	_request = correlation.get("request_id", "")
	var expiry: Variant = correlation.get("expiry_tick_us")
	if not (expiry is String) or not expiry.is_valid_int() or expiry.to_int() <= Time.get_ticks_usec():
		_release_attempt()
		return {"status": "refused", "reason": "invalid_expiry"}
	_expiry = expiry.to_int()
	_entered_stage = true
	var result: Variant = _native["edit_prepare"].call(path, expected, desired, correlation)
	_entered_stage = false
	if _state == Lifecycle.CLOSING:
		if result is Dictionary and result.get("status") == "prepared":
			_native["edit_cancel"].call(_request)
		_release_attempt()
		return {"status": "refused", "reason": "cancelled_during_prepare"}
	if not (result is Dictionary):
		_release_attempt()
		return {"status": "refused", "reason": "native_unavailable"}
	if result.get("status") == "prepared":
		_state = Lifecycle.PREPARED
	else:
		_release_attempt()
	return result


# Only one entered native stage per bridge scheduling pass. Failed stages keep
# the slot for read-only survivor collection and explicit terminal cleanup.
func advance_bound(request_id: String, stage: String) -> Dictionary:
	var active: Dictionary = _bridge.get("_active") if is_instance_valid(_bridge) else {}
	if not _product_attempt or active.get("operation_owner") != self \
			or active.get("request_id") != request_id or not active.get("apply_started", false) \
			or active.get("native_stage_authorized") != stage:
		return {"status": "refused", "reason": "wrong_attempt_or_stage"}
	active.erase("native_stage_authorized")
	return _advance_stage(request_id, stage)


func _advance_stage(request_id: String, stage: String) -> Dictionary:
	if _state != Lifecycle.PREPARED or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt_or_stage"}
	if Time.get_ticks_usec() >= _expiry:
		return cancel(request_id)
	_state = Lifecycle.FINISHING
	_entered_stage = true
	var result: Variant = _native["edit_advance"].call(request_id, stage)
	_entered_stage = false
	if _state == Lifecycle.CLOSING:
		var closing := result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}
		_release_attempt()
		return closing
	_state = Lifecycle.PREPARED
	return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}


# The private fixture exercises the same six guarded stages.
func finish(request_id: String) -> Dictionary:
	if _product_attempt:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state != Lifecycle.PREPARED:
		return {"status": "busy", "reason": "slot_busy"}
	if _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	var stage := "prepared"
	for _step in 6:
		var result := _advance_stage(request_id, stage)
		if _state == Lifecycle.IDLE:
			return result
		if result.get("status") != "ready":
			cancel(request_id)
			return result
		stage = str(result.get("stage", ""))
	var failed := cancel(request_id)
	return {"status": "partial", "reason": "incomplete_native_sequence", "native": failed}


func cancel(request_id: String) -> Dictionary:
	if _state == Lifecycle.IDLE or _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if _state == Lifecycle.CLOSING:
		return {"status": "busy", "reason": "closing_after_active_stage"}
	if _state == Lifecycle.PREPARING:
		_state = Lifecycle.CLOSING
		return {"status": "refused", "reason": "closing_after_active_stage"}
	if _state == Lifecycle.FINISHING:
		_state = Lifecycle.CLOSING
		var deferred: Variant = _native["edit_cancel"].call(request_id)
		return deferred if deferred is Dictionary else {"status": "partial", "reason": "native_unavailable"}
	_entered_stage = true
	var result: Variant = _native["edit_cancel"].call(request_id)
	_entered_stage = false
	_release_attempt()
	return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}


func finish_bound(request_id: String) -> Dictionary:
	return cancel(request_id)


func cancel_owned() -> void:
	if _state != Lifecycle.IDLE:
		cancel(_request)


func _process(_delta: float) -> void:
	if _state != Lifecycle.IDLE and Time.get_ticks_usec() >= _expiry:
		cancel(_request)
	if not _native.is_empty():
		_native["edit_expire"].call()


func _exit_tree() -> void:
	set_process(false)
	cancel_owned()
	if _state == Lifecycle.IDLE:
		_native = {}
		_bridge = null
