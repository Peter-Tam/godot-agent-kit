@tool
extends "res://addons/godot_agent_kit/bridge.gd"

const FixtureCollector = preload("res://addons/fixture_driver/fixture_collector.gd")

var _discovery_held: Array[Dictionary] = []

func discovery_release() -> void:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	for held in _discovery_held:
		if not _peers.has(held.peer): continue
		var payload: Dictionary = held.payload
		var fault: String = drivers[0].discovery_fault if drivers.size() == 1 else ""
		if fault == "identity":
			payload = payload.duplicate(true)
			payload.request_id = "wrong-request"
		super._queue(held.peer, payload, held.closing, held.large)
		if fault == "malformed":
			held.peer.output = PackedByteArray([0, 0, 0, 3, 123, 0, 125])
			held.peer.sent = 0
		elif fault == "oversized":
			held.peer.output = PackedByteArray([0, 0, 16, 1])
			held.peer.sent = 0
	_discovery_held.clear()



# Fixture-only gate: the product keeps processing other peers while a selected
# attempt owns the slot at a named gap. No native call or source is forged.
func _edit_ready(peer: Dictionary, stage: String) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("edit_barrier") \
			and not drivers[0].edit_barrier(stage, peer.get("request_id", "")):
		return false
	return super._edit_ready(peer, stage)

func _open_ready(peer: Dictionary, stage: String) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("open_barrier") \
			and not drivers[0].open_barrier(stage, peer):
		return false
	return super._open_ready(peer, stage)


func _open_response_ready(peer: Dictionary, kind: String, reply: Dictionary) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("open_response_barrier") \
			and not drivers[0].open_response_barrier(kind, peer, reply):
		return false
	return super._open_response_ready(peer, kind, reply)
func _close_ready(peer: Dictionary, stage: String) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("close_barrier") \
			and not drivers[0].close_barrier(stage, peer):
		return false
	return super._close_ready(peer, stage)

func _close_response_ready(peer: Dictionary, kind: String, reply: Dictionary) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("close_response_barrier") \
			and not drivers[0].close_response_barrier(kind, peer, reply):
		return false
	return super._close_response_ready(peer, kind, reply)

func _closed_ready(peer: Dictionary, stage: String) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("closed_barrier") \
			and not drivers[0].closed_barrier(stage, peer):
		return false
	return super._closed_ready(peer, stage)

func _closed_response_ready(peer: Dictionary, kind: String, reply: Dictionary) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("closed_response_barrier") \
			and not drivers[0].closed_response_barrier(kind, peer, reply):
		return false
	return super._closed_response_ready(peer, kind, reply)




func _collect_pending() -> void:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() != 1 or _active.is_empty() or not _active.has("pending"):
		super._collect_pending()
		return
	var driver: Variant = drivers[0]
	if driver.hold_at(_active.pending):
		# A real authenticated operation is paused, never replaced with synthetic
		# R/B evidence. The caller's own deadline still applies.
		return
	if _active.pending == "observe":
		var collector = FixtureCollector.new()
		collector.restriction = driver.restriction
		_active.collector = collector
	elif _active.pending == "recheck":
		# Executed only after the caller has received the original sample and read D.
		# The real recheck immediately follows this controlled editor-side barrier.
		driver.apply_transition()
	super._collect_pending()


func _queue(peer: Dictionary, payload: Dictionary, closing: bool = false, large: bool = false) -> void:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("consume_closed_reply_fault") \
			and drivers[0].consume_closed_reply_fault(String(payload.get("kind", ""))) == "malformed":
		# Negative fixture only: corrupt a genuine reply after product execution.
		# Keep complete framing; never synthesize positive source/effect evidence.
		super._queue(peer, payload, closing, large)
		peer.output = PackedByteArray([0, 0, 0, 3, 123, 0, 125])
		peer.sent = 0
		return
	if drivers.size() == 1 and payload.get("kind") == "discover_state":
		# Witness real admission even when a 1ms scope expires before TCP delivery.
		drivers[0].scope_admission = {"request_id": payload.request_id,
			"status": payload.status, "expiry_tick_us": payload.expiry_tick_us,
			"started_tick_us": payload.collection.started_tick_us}
	if drivers.size() == 1 and payload.get("kind") in ["discover_state", "discover_rechecked"] \
			and drivers[0].has_method("discovery_barrier") and drivers[0].discovery_barrier(peer, payload):
		_discovery_held.append({"peer": peer, "payload": payload,
			"closing": closing, "large": large})
		return
	if drivers.size() != 1 or payload.get("kind") not in ["sample", "recheck"]:
		super._queue(peer, payload, closing, large)
		return
	var restriction: String = drivers[0].restriction
	if (payload.kind == "sample" and restriction == "reject_sample_identity") \
		or (payload.kind == "recheck" and restriction == "reject_recheck_identity"):
		# Corrupt only the request identity of a genuine native reply.
		var mismatched := payload.duplicate()
		mismatched.request_id = "wrong-request"
		super._queue(peer, mismatched, closing, large)
		return
	if payload.kind == "recheck" and restriction in ["reject_recheck_malformed", "reject_recheck_oversized"]:
		# Neither refusal synthesizes positive evidence or includes source bytes.
		super._queue(peer, payload, closing, large)
		peer.output = PackedByteArray([0, 0, 0, 3, 123, 0, 125]) \
			if restriction == "reject_recheck_malformed" else PackedByteArray([0, 192, 0, 1])
		peer.sent = 0
		return
	super._queue(peer, payload, closing, large)
