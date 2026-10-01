@tool
extends "res://addons/godot_agent_kit/script_discovery.gd"

# Negative getter outcomes only. Supported contexts and signal registration always
# use the real editor; these controls are never installed in the product addon.
func _mode() -> String:
	var drivers := get_tree().get_nodes_in_group("observation_fixture_driver")
	return drivers[0].scope_mode if drivers.size() == 1 else ""

func available() -> bool:
	return _mode() != "disabled_capability" and super.available()

func _settings_directory() -> Variant:
	if _mode() == "unavailable":
		return ""
	if _mode() == "unsupported":
		return "res://unsupported_scope/editor"
	return super._settings_directory()

func _filesystem_facts() -> Dictionary:
	if _mode() == "live_scan":
		# Owned preparation at the sampling boundary, not a synthetic positive.
		EditorInterface.get_resource_filesystem().scan()
	if _mode() == "live_import":
		var filesystem := EditorInterface.get_resource_filesystem()
		var witnessed := {"scanning": false, "importing": false}
		var sample := func(_resources: PackedStringArray):
			witnessed.scanning = filesystem.is_scanning()
			witnessed.importing = filesystem.is_importing()
		filesystem.resources_reimporting.connect(sample)
		# Sample the actual synchronous importer inside this collection interval.
		filesystem.reimport_files(PackedStringArray(["res://scope_import.svg"]))
		filesystem.resources_reimporting.disconnect(sample)
		return witnessed
	if _mode() == "scanning":
		return {"scanning": true, "importing": false}
	if _mode() == "importing":
		return {"scanning": false, "importing": true}
	return super._filesystem_facts()
