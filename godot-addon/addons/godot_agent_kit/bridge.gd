@tool
extends Node

const VERSION := "4.7.2.stable.official.ed1daf0bf"
const ENGINE_HASH := "ed1daf0bf001b61586d9930840f2f1394092c079"
const DOMAIN := "godot-agent-kit/observation-bridge/v1"
const MAX_FRAME := 4096
const MAX_PEERS := 32
const PEER_EXPIRY_USEC := 4500000
const FRAME_BUDGET_USEC := 1000
const FRAME_BUDGET_BYTES := 65536
const CAPABILITIES := {
	"observe_gdscript": false,
	"open_enumeration": false,
	"buffer_attribution": false,
	"unsaved_paths": false,
	"cached_resource_lookup": false,
}

var _crypto := Crypto.new()
var _filesystem := DirAccess.open("/")
var _server: TCPServer
var _peers: Array[Dictionary] = []
var _peer_cursor := 0
var _registry := ""
var _project := ""
var _session := ""
var _secret := PackedByteArray()
var _descriptor := ""
var _descriptor_identity := ""
var _registry_identity := ""
var _uid := ""
var _project_identity := ""


func _valid_metadata_path(path: String) -> bool:
	return path.begins_with("/") and not path.contains("\n") and not path.contains("\r") and not path.to_utf8_buffer().has(0)


# Only a fixed native metadata tool sees validated absolute paths. An absent
# ACL and a deny-only ACL are safe; any access grant or unreadable ACL refuses.
func _acl_is_restrictive(path: String, directory: bool) -> bool:
	if not _valid_metadata_path(path):
		return false
	var output: Array = []
	if OS.execute("/bin/ls", ["-lde", "--", path], output) != 0 or output.size() != 1:
		return false
	var text: String = str(output[0])
	if text.length() > 8192:
		return false
	var lines := text.strip_edges().split("\n")
	if lines.is_empty() or not lines[0].ends_with(" " + path):
		return false
	var mode := lines[0].split(" ", false)[0]
	if mode.length() != 10 and mode.length() != 11:
		return false
	if mode[0] != ("d" if directory else "-") or (mode.length() == 11 and mode[10] not in ["+", "@"]):
		return false
	if mode.ends_with("+") and lines.size() == 1:
		return false
	for index in range(1, lines.size()):
		var entry := lines[index].strip_edges()
		var separator := entry.find(":")
		if separator < 1 or not entry.substr(0, separator).is_valid_int() or entry.substr(0, separator).to_int() != index - 1:
			return false
		var action := entry.find(" deny ", separator + 1)
		if action < 0 or entry.find(" allow ", separator + 1) >= 0 or action == separator + 1:
			return false
		var permissions := entry.substr(action + 6).split(",", false)
		if permissions.is_empty():
			return false
		for permission in permissions:
			if permission.is_empty():
				return false
			for character in permission:
				if character not in "abcdefghijklmnopqrstuvwxyz_":
					return false
	return true


func _directory_identity(path: String) -> String:
	var info := _metadata(path)
	if info.size() != 5 or info[2] != "Directory":
		return ""
	return info[3] + ":" + info[4]


# The public Godot filesystem APIs do not report the effective UID, owner, inode,
# object type or ACL. Three fixed macOS tools inspect only validated metadata;
# no shell, peer input, token or script is ever passed to them.
func _owner_uid() -> String:
	var output: Array = []
	if OS.execute("/usr/bin/id", ["-u"], output) != 0 or output.size() != 1:
		return ""
	var value: String = str(output[0]).strip_edges()
	return value if value.is_valid_int() and value == str(value.to_int()) else ""


func _metadata(path: String) -> PackedStringArray:
	if not _valid_metadata_path(path):
		return PackedStringArray()
	var output: Array = []
	if OS.execute("/usr/bin/stat", ["-f", "%u:%p:%HT:%d:%i", "--", path], output) != 0 or output.size() != 1:
		return PackedStringArray()
	var fields := str(output[0]).strip_edges().split(":")
	if fields.size() != 5 or not fields[0].is_valid_int() or not fields[3].is_valid_int() or not fields[4].is_valid_int():
		return PackedStringArray()
	if fields[1].length() < 4 or not fields[1].is_valid_int():
		return PackedStringArray()
	return fields


func _registry_is_private(path: String) -> bool:
	if OS.get_name() != "macOS" or not _valid_metadata_path(path) or path == "/" or path.contains("//") or path.ends_with("/"):
		return false
	if _project_identity.is_empty() or _directory_identity(_project) != _project_identity:
		return false
	_uid = _owner_uid()
	if _uid.is_empty():
		return false
	var root := _metadata("/")
	if root.size() != 5 or root[0] != "0" or root[2] != "Directory" or not _acl_is_restrictive("/", true):
		return false
	var root_mode: String = root[1]
	if root_mode[root_mode.length() - 2] in ["2", "3", "6", "7"] or root_mode[root_mode.length() - 1] in ["2", "3", "6", "7"]:
		return false
	var parts := path.split("/", false)
	var prefix := ""
	var leaf := PackedStringArray()
	for part in parts:
		if part in ["", ".", ".."] or part.contains(":") or part.contains("\n") or part.contains("\r") or part.to_utf8_buffer().has(0):
			return false
		prefix += "/" + part
		var info := _metadata(prefix)
		if info.size() != 5 or info[2] != "Directory" or _filesystem == null or _filesystem.is_link(prefix) \
			or not _acl_is_restrictive(prefix, true):
			return false
		# Compare device/inode, not spelling: case, symlinked project roots and
		# alternate mounts must not put a live token inside the project.
		if info[3] + ":" + info[4] == _project_identity:
			return false
		if info[0] != _uid and info[0] != "0":
			return false
		var octal_mode: String = info[1]
		if prefix == path:
			if info[0] != _uid or not octal_mode.ends_with("0700"):
				return false
			leaf = info
		else:
			# Even when the leaf is private, a substitutable ancestor is unsafe.
			var group_write := octal_mode[octal_mode.length() - 2] in ["2", "3", "6", "7"]
			var other_write := octal_mode[octal_mode.length() - 1] in ["2", "3", "6", "7"]
			if group_write or other_write:
				return false
	return leaf.size() == 5 and _metadata(path) == leaf


func _registry_unchanged() -> bool:
	return not _registry.is_empty() and _registry_is_private(_registry) and ":".join(_metadata(_registry)) == _registry_identity


func start(registry: String) -> bool:
	stop()
	if not Engine.is_editor_hint():
		return false
	var version := Engine.get_version_info()
	if version.get("major") != 4 or version.get("minor") != 7 or version.get("patch") != 2 \
		or version.get("status") != "stable" or version.get("build") != "official" or version.get("hash") != ENGINE_HASH:
		push_error("Godot Agent Kit: unsupported exact editor version")
		return false
	_project = ProjectSettings.globalize_path("res://").trim_suffix("/")
	if not _valid_metadata_path(_project) or _project.to_utf8_buffer().size() > 1024:
		stop()
		return false
	_project_identity = _directory_identity(_project)
	if _project_identity.is_empty() or not _registry_is_private(registry):
		push_error("Godot Agent Kit: unsafe registry or project metadata")
		stop()
		return false
	_registry_identity = ":".join(_metadata(registry))
	_registry = registry
	var sid := _crypto.generate_random_bytes(16)
	_secret = _crypto.generate_random_bytes(32)
	if sid.size() != 16 or _secret.size() != 32:
		stop()
		return false
	_session = sid.hex_encode()
	_server = TCPServer.new()
	var port := -1
	for attempt in 16:
		var random_port := _crypto.generate_random_bytes(2)
		if random_port.size() != 2:
			break
		var candidate := 49152 + ((int(random_port[0]) << 8 | int(random_port[1])) % 16384)
		if _server.listen(candidate, "127.0.0.1") == OK:
			port = candidate
			break
	if port < 0 or not _registry_unchanged() or not _publish(port):
		stop()
		return false
	set_process(true)
	return true


func _publish(port: int) -> bool:
	var final_path := _registry.path_join(_session + ".json")
	var temp_path := _registry.path_join(_session + ".tmp")
	if _filesystem == null or _filesystem.is_link(final_path) or DirAccess.dir_exists_absolute(final_path) or FileAccess.file_exists(final_path) \
		or _filesystem.is_link(temp_path) or DirAccess.dir_exists_absolute(temp_path) or FileAccess.file_exists(temp_path):
		return false
	var descriptor := {
		"v": 1, "session_id": _session, "project_root": _project,
		"godot_version": VERSION, "engine_hash": ENGINE_HASH,
		"host": "127.0.0.1", "port": port, "token": _secret.hex_encode(),
	}
	var bytes := JSON.stringify(descriptor).to_utf8_buffer()
	if bytes.size() > MAX_FRAME:
		return false
	var file := FileAccess.open(temp_path, FileAccess.WRITE)
	if file == null:
		return false
	if not file.store_buffer(bytes):
		file.close()
		_remove_owned_temp(temp_path)
		return false
	file.flush()
	var flush_ok := file.get_error() == OK
	file.close()
	# Editor safe-save may not install temp_path until close. The verified private
	# directory protects these bytes; fix the new file's mode before advertisement.
	if not flush_ok or FileAccess.set_unix_permissions(temp_path, 384) != OK:
		_remove_owned_temp(temp_path)
		return false
	var info := _metadata(temp_path)
	if not _private_regular(info, temp_path) or _filesystem.is_link(temp_path) or not _registry_unchanged() \
		or not _metadata(final_path).is_empty() or DirAccess.rename_absolute(temp_path, final_path) != OK:
		_remove_owned_temp(temp_path)
		return false
	var temp_identity := ":".join(info)
	info = _metadata(final_path)
	if not _private_regular(info, final_path) or not _registry_unchanged() or ":".join(info) != temp_identity:
		if _owned_regular(info) and ":".join(info) == temp_identity and _registry_unchanged() \
			and _filesystem != null and not _filesystem.is_link(final_path):
			DirAccess.remove_absolute(final_path)
		return false
	_descriptor = final_path
	_descriptor_identity = temp_identity
	return true


func _owned_regular(info: PackedStringArray) -> bool:
	return info.size() == 5 and info[0] == _uid and info[2] == "Regular File" and info[1].ends_with("0600")


func _private_regular(info: PackedStringArray, path: String) -> bool:
	return _owned_regular(info) and _acl_is_restrictive(path, false)


func _remove_owned_temp(path: String) -> void:
	if _registry_unchanged() and _owned_regular(_metadata(path)) and _filesystem != null and not _filesystem.is_link(path):
		DirAccess.remove_absolute(path)


func stop() -> void:
	set_process(false)
	for peer in _peers:
		peer.socket.disconnect_from_host()
	_peers.clear()
	_peer_cursor = 0
	if _server != null:
		_server.stop()
		_server = null
	if not _descriptor.is_empty() and _registry_unchanged() and _filesystem != null and not _filesystem.is_link(_descriptor):
		var descriptor_info := _metadata(_descriptor)
		if _owned_regular(descriptor_info) and ":".join(descriptor_info) == _descriptor_identity:
			DirAccess.remove_absolute(_descriptor)
	_registry = ""
	_project = ""
	_session = ""
	_secret.clear()
	_descriptor = ""
	_descriptor_identity = ""
	_registry_identity = ""
	_project_identity = ""


func _exit_tree() -> void:
	stop()


func _process(_delta: float) -> void:
	if _server == null:
		return
	# Work on all connections is shared under one frame-wide time/byte budget.
	var frame_start := Time.get_ticks_usec()
	var budget := FRAME_BUDGET_BYTES
	var schedule := _peers.duplicate()
	if not schedule.is_empty():
		_peer_cursor = (_peer_cursor + 1) % schedule.size()
	for offset in schedule.size():
		var peer: Dictionary = schedule[(offset + _peer_cursor) % schedule.size()]
		if Time.get_ticks_usec() - frame_start >= FRAME_BUDGET_USEC or budget <= 0:
			break
		var socket: StreamPeerTCP = peer.socket
		socket.poll()
		if socket.get_status() != StreamPeerTCP.STATUS_CONNECTED or (peer.state != "authenticated" and Time.get_ticks_usec() - int(peer.since) >= PEER_EXPIRY_USEC):
			_close_peer(peer)
			continue
		if not peer.output.is_empty():
			var remaining: int = peer.output.size() - peer.sent
			var chunk: PackedByteArray = peer.output.slice(peer.sent, peer.sent + mini(remaining, budget))
			var result: Array = socket.put_partial_data(chunk)
			if result.size() != 2 or result[0] != OK or result[1] < 0 or result[1] > chunk.size():
				_close_peer(peer)
				continue
			peer.sent += result[1]
			budget -= result[1]
			if peer.sent == peer.output.size():
				peer.output = PackedByteArray()
				peer.sent = 0
				if peer.state == "closing":
					_close_peer(peer)
			continue
		if peer.state == "closing":
			_close_peer(peer)
			continue
		while budget > 0 and Time.get_ticks_usec() - frame_start < FRAME_BUDGET_USEC and socket.get_available_bytes() > 0:
			var required: int = (4 if peer.needed < 0 else peer.needed + 4) - peer.input.size()
			if required <= 0:
				_close_peer(peer)
				break
			var result: Array = socket.get_partial_data(mini(required, mini(budget, socket.get_available_bytes())))
			if result.size() != 2 or result[0] != OK or result[1].is_empty():
				_close_peer(peer)
				break
			var received: PackedByteArray = result[1]
			peer.input.append_array(received)
			budget -= received.size()
			if peer.needed < 0 and peer.input.size() == 4:
				peer.needed = (int(peer.input[0]) << 24) | (int(peer.input[1]) << 16) | (int(peer.input[2]) << 8) | int(peer.input[3])
				if peer.needed < 1 or peer.needed > MAX_FRAME:
					_close_peer(peer)
					break
			if peer.needed >= 0 and peer.input.size() == peer.needed + 4:
				var message: PackedByteArray = peer.input.slice(4)
				peer.input = PackedByteArray()
				peer.needed = -1
				_handle_frame(peer, message)
				break
	while Time.get_ticks_usec() - frame_start < FRAME_BUDGET_USEC and _server.is_connection_available():
		var incoming := _server.take_connection()
		if incoming == null:
			break
		if _peers.size() >= MAX_PEERS:
			incoming.disconnect_from_host()
			if Time.get_ticks_usec() - frame_start >= FRAME_BUDGET_USEC:
				break
			continue
		# A completed accept belongs to this lifetime even if its syscall crossed
		# the frame budget. Retain it within the cap; service it on the next frame.
		incoming.set_no_delay(true)
		_peers.append({"socket": incoming, "since": Time.get_ticks_usec(), "state": "initial",
			"input": PackedByteArray(), "needed": -1, "output": PackedByteArray(), "sent": 0})


func _close_peer(peer: Dictionary) -> void:
	peer.socket.disconnect_from_host()
	_peers.erase(peer)


func _queue(peer: Dictionary, payload: Dictionary, closing: bool = false) -> void:
	var body := JSON.stringify(payload).to_utf8_buffer()
	if body.size() > MAX_FRAME:
		_close_peer(peer)
		return
	var size := body.size()
	var frame := PackedByteArray([size >> 24 & 255, size >> 16 & 255, size >> 8 & 255, size & 255])
	frame.append_array(body)
	peer.output = frame
	peer.sent = 0
	if closing:
		peer.state = "closing"


func _valid_hex(value: Variant, count: int) -> bool:
	if typeof(value) != TYPE_STRING or value.length() != count:
		return false
	for character in value:
		if character not in "0123456789abcdef":
			return false
	return true


func _valid_id(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING or value.is_empty() or value.length() > 64:
		return false
	for character in value:
		if character not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_":
			return false
	return true


func _fixed_tuple(value: Variant, count: int, operation: String) -> bool:
	return typeof(value) == TYPE_ARRAY and value.size() == count and typeof(value[0]) in [TYPE_INT, TYPE_FLOAT] and value[0] == 1 \
		and typeof(value[1]) == TYPE_STRING and value[1] == operation and _valid_id(value[2]) \
		and typeof(value[3]) == TYPE_STRING and value[3] == _session \
		and typeof(value[4]) == TYPE_STRING and value[4] == _project and value[4].to_utf8_buffer().size() <= 1024


func _handle_frame(peer: Dictionary, bytes: PackedByteArray) -> void:
	var text := bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes:
		_close_peer(peer)
		return
	var parser := JSON.new()
	if parser.parse(text) != OK:
		_close_peer(peer)
		return
	var value: Variant = parser.data
	if peer.state == "initial":
		if not _fixed_tuple(value, 6, "hello") or not _valid_hex(value[5], 64):
			_close_peer(peer)
			return
		var server_nonce := _crypto.generate_random_bytes(32)
		if server_nonce.size() != 32:
			_close_peer(peer)
			return
		var transcript := transcript_bytes(value[2], _session, _project, VERSION, ENGINE_HASH, CAPABILITIES, value[5], server_nonce.hex_encode())
		if transcript.is_empty():
			_close_peer(peer)
			return
		peer.request_id = value[2]
		peer.client_nonce = value[5]
		peer.server_nonce = server_nonce.hex_encode()
		peer.transcript = transcript
		peer.state = "challenged"
		var challenge := _reply_fields(peer, "challenge")
		var proof := role_proof(_secret, "server", transcript)
		if proof.size() != 32:
			_close_peer(peer)
			return
		challenge.server_proof = proof.hex_encode()
		_queue(peer, challenge)
	elif peer.state == "challenged":
		if not _fixed_tuple(value, 8, "authenticate") or value[2] != peer.request_id \
			or not _valid_hex(value[5], 64) or value[5] != peer.client_nonce \
			or not _valid_hex(value[6], 64) or value[6] != peer.server_nonce \
			or not _valid_hex(value[7], 64):
			_close_peer(peer)
			return
		var expected := role_proof(_secret, "client", peer.transcript)
		if expected.size() != 32 or not _crypto.constant_time_compare(expected, (value[7] as String).hex_decode()):
			_close_peer(peer)
			return
		var finish := role_proof(_secret, "finish", peer.transcript)
		if finish.size() != 32:
			_close_peer(peer)
			return
		peer.state = "authenticated"
		var hello := _reply_fields(peer, "hello")
		hello.finish_proof = finish.hex_encode()
		_queue(peer, hello)
	else:
		# No observation handler exists in T002. A completed handshake cannot
		# authorize source acquisition until an actual collector is installed.
		_close_peer(peer)


func _reply_fields(peer: Dictionary, kind: String) -> Dictionary:
	return {"v": 1, "kind": kind, "request_id": peer.request_id,
		"session_id": _session, "project_root": _project,
		"godot_version": VERSION, "engine_hash": ENGINE_HASH,
		"capabilities": CAPABILITIES, "client_nonce": peer.client_nonce,
		"server_nonce": peer.server_nonce}


static func _field(data: PackedByteArray) -> PackedByteArray:
	var length := data.size()
	var out := PackedByteArray([length >> 24 & 255, length >> 16 & 255, length >> 8 & 255, length & 255])
	out.append_array(data)
	return out


static func transcript_bytes(request_id: String, session_id: String, project_root: String,
		godot_version: String, engine_hash: String, capabilities: Dictionary,
		client_nonce: String, server_nonce: String) -> PackedByteArray:
	if session_id.length() != 32 or client_nonce.length() != 64 or server_nonce.length() != 64:
		return PackedByteArray()
	var out := PackedByteArray()
	for part in [DOMAIN, request_id]:
		out.append_array(_field(part.to_utf8_buffer()))
	out.append_array(_field(session_id.hex_decode()))
	for part in [project_root, godot_version, engine_hash]:
		out.append_array(_field(part.to_utf8_buffer()))
	for name in ["observe_gdscript", "open_enumeration", "buffer_attribution", "unsaved_paths", "cached_resource_lookup"]:
		if not capabilities.has(name) or typeof(capabilities[name]) != TYPE_BOOL:
			return PackedByteArray()
		out.append_array(_field(PackedByteArray([1 if capabilities[name] else 0])))
	out.append_array(_field(client_nonce.hex_decode()))
	out.append_array(_field(server_nonce.hex_decode()))
	return out


static func role_proof(secret: PackedByteArray, role: String, transcript: PackedByteArray) -> PackedByteArray:
	if secret.size() != 32 or role not in ["server", "client", "finish"] or transcript.is_empty():
		return PackedByteArray()
	var message := _field(role.to_ascii_buffer())
	message.append_array(transcript)
	return Crypto.new().hmac_digest(HashingContext.HASH_SHA256, secret, message)
