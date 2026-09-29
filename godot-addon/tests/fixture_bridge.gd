@tool
extends "res://addons/godot_agent_kit/bridge.gd"

const FixtureCollector = preload("res://addons/fixture_driver/fixture_collector.gd")


# Fixture-only gate: the product keeps processing other peers while a selected
# attempt owns the slot at a named gap. No native call or source is forged.
func _edit_ready(peer: Dictionary, stage: String) -> bool:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() == 1 and drivers[0].has_method("edit_barrier") \
			and not drivers[0].edit_barrier(stage, peer.get("request_id", "")):
		return false
	return super._edit_ready(peer, stage)


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
