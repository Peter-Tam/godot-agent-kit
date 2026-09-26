@tool
extends EditorPlugin

const Bridge = preload("res://addons/godot_agent_kit/bridge.gd")
const PRODUCT := "godot_agent_kit"
const MAX_CONTROL := 4096

var _control := ""
var _last_id := ""


func _enter_tree() -> void:
	_control = OS.get_environment("GODOT_AGENT_KIT_FIXTURE_CONTROL")
	if _control.begins_with("/") and DirAccess.dir_exists_absolute(_control):
		set_process(true)


func _process(_delta: float) -> void:
	var request_file := _control.path_join("request.json")
	if not FileAccess.file_exists(request_file):
		return
	var input := FileAccess.open(request_file, FileAccess.READ)
	if input == null:
		return
	if input.get_length() > MAX_CONTROL:
		input.close()
		return
	var raw := input.get_buffer(input.get_length())
	input.close()
	var text := raw.get_string_from_utf8()
	if text.to_utf8_buffer() != raw:
		return
	var parser := JSON.new()
	if parser.parse(text) != OK or typeof(parser.data) != TYPE_DICTIONARY:
		return
	var request: Dictionary = parser.data
	if request.size() != 2 or typeof(request.get("id")) != TYPE_STRING or typeof(request.get("action")) != TYPE_STRING:
		return
	var id: String = request.id
	var action: String = request.action
	if id.is_empty() or id.length() > 64 or id == _last_id:
		return
	_last_id = id
	var response := {"id": id, "action": action, "ok": true}
	match action:
		"witness":
			response.merge(_witness())
		"disable", "enable":
			var enabled := action == "enable"
			EditorInterface.set_plugin_enabled(PRODUCT, enabled)
			response.plugin_enabled = EditorInterface.is_plugin_enabled(PRODUCT)
			response.ok = response.plugin_enabled == enabled
		"proof_vectors":
			response.merge(_proof_vectors())
			response.ok = response.transcript_bytes == 270 and response.server_matches and response.client_matches and response.finish_matches
		"quit":
			pass
		_:
			response.ok = false
	if not _respond(response):
		return
	if action == "quit":
		get_tree().call_deferred("quit")


func _respond(response: Dictionary) -> bool:
	var temporary := _control.path_join("response.tmp")
	var destination := _control.path_join("response.json")
	var file := FileAccess.open(temporary, FileAccess.WRITE)
	if file == null:
		return false
	var written := file.store_string(JSON.stringify(response))
	file.flush()
	file.close()
	if not written or FileAccess.set_unix_permissions(temporary, 384) != OK:
		return false
	return DirAccess.rename_absolute(temporary, destination) == OK


func _witness() -> Dictionary:
	var version := Engine.get_version_info()
	var exact_version := "%s.%s.%s.%s.%s.%s" % [version.major, version.minor, version.patch, version.status, version.build, version.hash.left(9)]
	var nodes: Array[Dictionary] = []
	for node in get_tree().get_nodes_in_group("godot_agent_kit_session_bridge"):
		nodes.append({"name": node.name, "type": node.get_class()})
	var editor := EditorInterface.get_script_editor()
	var paths := PackedStringArray()
	for script in editor.get_open_scripts():
		if script != null:
			paths.append(script.resource_path)
	var selected := editor.get_current_script()
	return {"version": exact_version, "engine_hash": version.get("hash", ""),
		"editor_hint": Engine.is_editor_hint(), "pid": OS.get_process_id(),
		"plugin_enabled": EditorInterface.is_plugin_enabled(PRODUCT),
		"product_nodes": nodes, "open_paths": paths,
		"current_script": selected.resource_path if selected != null else ""}


func _proof_vectors() -> Dictionary:
	# Synthetic public contract constants. Never inspect a live descriptor or expose
	# transcript, nonce, token, or proof bytes in the control response.
	var token := "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".hex_decode()
	var caps := {"observe_gdscript": true, "open_enumeration": true,
		"buffer_attribution": true, "unsaved_paths": true, "cached_resource_lookup": true}
	var transcript := Bridge.transcript_bytes("example-1", "00112233445566778899aabbccddeeff",
		"/fixture/project", "4.7.2.stable.official.ed1daf0bf",
		"ed1daf0bf001b61586d9930840f2f1394092c079", caps,
		"202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
		"404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f")
	return {"transcript_bytes": transcript.size(),
		"server_matches": Bridge.role_proof(token, "server", transcript).hex_encode() == "31cd95cdc00afe815371791207ddedda98cb79e1d6bf181bd4fb8ac3cc8d2ed1",
		"client_matches": Bridge.role_proof(token, "client", transcript).hex_encode() == "8516bc474f397be9deae2a70c4ae5577041eb6753cb1dde328278978c2f8b5e8",
		"finish_matches": Bridge.role_proof(token, "finish", transcript).hex_encode() == "ff0f3f3507b79628be82e0cd67539765a174d9d0323aff470d62bbecbf12dd32"}
