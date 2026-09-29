@tool
extends Node

# This owner is private to the editor plugin. T004 will add authenticated routing;
# no edit command is offered by the observation bridge in T003.
enum Lifecycle { IDLE, PREPARING, PREPARED, FINISHING, CLOSING }
var _state := Lifecycle.IDLE
var _native: Dictionary = {}
var _bridge: Node
var _request := ""
var _expiry := 0


func configure(native_api: Dictionary, bridge: Node) -> void:
	if _state != Lifecycle.IDLE:
		return
	_native = native_api
	_bridge = bridge
	set_process(not _native.is_empty())


func _release_attempt() -> void:
	_request = ""
	_expiry = 0
	_state = Lifecycle.IDLE
	if is_instance_valid(_bridge):
		_bridge.call("_release_edit", self)
	if not is_inside_tree():
		_native = {}
		_bridge = null


func begin(path: String, expected: String, desired: String, correlation: Dictionary) -> Dictionary:
	if _native.is_empty() or not is_instance_valid(_bridge) or _state != Lifecycle.IDLE:
		return {"status": "busy", "reason": "slot_busy"}
	if _bridge.call("_claim_edit", self) != true:
		return {"status": "busy", "reason": "slot_busy"}
	_state = Lifecycle.PREPARING
	_request = correlation.get("request_id", "")
	var expiry: Variant = correlation.get("expiry_tick_us")
	if not (expiry is String) or not expiry.is_valid_int() or expiry.to_int() <= Time.get_ticks_usec():
		_release_attempt()
		return {"status": "refused", "reason": "invalid_expiry"}
	_expiry = expiry.to_int()
	var result: Variant = _native["edit_prepare"].call(path, expected, desired, correlation)
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


# No yield inside a stage: its synchronous callbacks may request cancellation,
# but they must not release the shared bridge slot until that call returns.
func finish(request_id: String) -> Dictionary:
	if _state != Lifecycle.PREPARED:
		return {"status": "busy", "reason": "slot_busy"}
	if _request != request_id:
		return {"status": "refused", "reason": "wrong_attempt"}
	if Time.get_ticks_usec() >= _expiry:
		return cancel(request_id)
	_state = Lifecycle.FINISHING
	var stage := "prepared"
	for _step in 6:
		var result: Variant = _native["edit_advance"].call(request_id, stage)
		if _state == Lifecycle.CLOSING:
			_release_attempt()
			return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}
		if not (result is Dictionary):
			_native["edit_cancel"].call(request_id)
			_release_attempt()
			return {"status": "partial", "reason": "native_unavailable"}
		if result.get("status") != "ready":
			_release_attempt()
			return result
		stage = str(result.get("stage", ""))
	_native["edit_cancel"].call(request_id)
	_release_attempt()
	return {"status": "partial", "reason": "incomplete_native_sequence"}


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
	var result: Variant = _native["edit_cancel"].call(request_id)
	_release_attempt()
	return result if result is Dictionary else {"status": "partial", "reason": "native_unavailable"}


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
