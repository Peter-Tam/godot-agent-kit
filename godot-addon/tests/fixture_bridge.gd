@tool
extends "res://addons/godot_agent_kit/bridge.gd"

const FixtureCollector = preload("res://addons/fixture_driver/fixture_collector.gd")


func _collect_pending() -> void:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	if drivers.size() != 1 or _active.is_empty() or not _active.has("pending"):
		super._collect_pending()
		return
	var driver: Variant = drivers[0]
	if _active.pending == "observe":
		var collector = FixtureCollector.new()
		collector.restriction = driver.restriction
		_active.collector = collector
	elif _active.pending == "recheck":
		# Executed only after the caller has received the original sample and read D.
		# The real recheck immediately follows this controlled editor-side barrier.
		driver.apply_transition()
	super._collect_pending()
