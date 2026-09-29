@tool
extends Node

const VERSION := "4.7.2.stable.official.ed1daf0bf"
const ENGINE_HASH := "ed1daf0bf001b61586d9930840f2f1394092c079"
const DOMAIN := "godot-agent-kit/editor-bridge/v2"
const ObservationScript = preload("res://addons/godot_agent_kit/observation.gd")
const MAX_FRAME := 4096
const MAX_SELECTED_REQUEST := 4 * 1024 * 1024
const MAX_RESPONSE := 12 * 1024 * 1024
const MAX_PEERS := 32
const PEER_EXPIRY_USEC := 4500000
# Expire handshakes early enough for frame-budgeted polling to meet the 4.5 s
# unauthenticated lifetime bound; source collection keeps its separate lease.
const HANDSHAKE_EXPIRY_USEC := 4000000
const FRAME_BUDGET_USEC := 1000
const FRAME_BUDGET_BYTES := 65536
const CAPABILITIES := {
	"observe_gdscript": true,
	"open_enumeration": true,
	"buffer_attribution": true,
	"unsaved_paths": true,
	"cached_resource_lookup": true,
	"edit_open_gdscript": false,
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

var _edit_owner: Node
var _native_revision := 0
var _native_build_id := ""
var _active: Dictionary = {}


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
		"v": 2, "session_id": _session, "project_root": _project,
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


# The private native owner and the authenticated peer use the same slot.
# Only the owner that claimed it can release it.
func _claim_edit(owner: Node, peer: Dictionary = {}) -> bool:
	if not is_instance_valid(owner) or _server == null or _session.is_empty() or not _active.is_empty():
		return false
	_active = peer if not peer.is_empty() else {"edit_owner": owner}
	_active.edit_owner = owner
	return true


func _release_edit(owner: Node) -> void:
	if _active.get("edit_owner") == owner:
		_active = {}


func attach_edit(owner: Node, revision: int, build_id: String) -> void:
	var valid: bool = is_instance_valid(owner) and owner.get("_bridge") == self \
		and revision == 1 and _valid_hex(build_id, 64)
	if valid:
		var installed: Variant = Engine.get_meta("godot_agent_kit_native", {})
		var family: Variant = owner.get("_native")
		valid = installed is Dictionary and family is Dictionary \
			and family.has_all(["api_revision", "build_id", "edit_inspect", "edit_prepare",
				"edit_advance", "edit_cancel", "edit_expire"]) \
			and family["api_revision"].call() == 1 and family["build_id"].call() == build_id \
			and installed.get("build_id") == family["build_id"]
	_edit_owner = owner if valid else null
	_native_revision = 1 if valid else 0
	_native_build_id = build_id if valid else ""


func _capabilities() -> Dictionary:
	var capabilities := CAPABILITIES.duplicate()
	capabilities.edit_open_gdscript = is_instance_valid(_edit_owner) and _native_revision == 1
	return capabilities


func stop() -> void:
	set_process(false)
	var edit_owner: Variant = _active.get("edit_owner")
	if is_instance_valid(edit_owner):
		edit_owner.call("cancel_owned")
	for peer in _peers:
		peer.socket.disconnect_from_host()
		if peer.has("collector"):
			peer.collector.clear()
		if peer.has("edit_collector"):
			peer.edit_collector.clear()
	# Reentrant shutdown may interrupt an already-entered native call. Its owner
	# releases this marker only after the call and its history cleanup return.
	if not _active.has("edit_owner") or not is_instance_valid(edit_owner) or not edit_owner.call("is_active_stage"):
		_active.clear()
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
	_edit_owner = null
	_native_revision = 0
	_native_build_id = ""


func _exit_tree() -> void:
	stop()


func _process(_delta: float) -> void:
	if _server == null:
		return
	# Work on all connections is shared under one frame-wide time/byte budget.
	var frame_start := Time.get_ticks_usec()
	if _active.has("observation_since") and Time.get_ticks_usec() - int(_active.observation_since) >= PEER_EXPIRY_USEC:
		_close_peer(_active)
	if _active.has("edit_owner") and _active.has("expiry_tick_us") \
			and Time.get_ticks_usec() >= int(_active.expiry_tick_us):
		_close_peer(_active)
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
		if socket.get_status() != StreamPeerTCP.STATUS_CONNECTED or (peer.state in ["initial", "challenged"] and Time.get_ticks_usec() - int(peer.since) >= HANDSHAKE_EXPIRY_USEC):
			_close_peer(peer)
			continue
		if peer.has("frame_pending"):
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
			if peer.needed > MAX_FRAME and not peer.get("large_prefix", false):
				required = mini(required, 516 - peer.input.size())
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
				var limit := MAX_SELECTED_REQUEST if peer.state == "authenticated" else MAX_FRAME
				if peer.needed < 1 or peer.needed > limit:
					_close_peer(peer)
					break
			if peer.needed > MAX_FRAME and not peer.get("large_prefix", false) \
					and peer.input.size() == 516:
				var prefix: String = peer.input.slice(4).get_string_from_utf8()
				for whitespace in [" ", "\t", "\r", "\n"]:
					prefix = prefix.replace(whitespace, "")
				if not prefix.begins_with('[2,"edit_prepare",'):
					_close_peer(peer)
					break
				peer.large_prefix = true
			if peer.needed >= 0 and peer.input.size() == peer.needed + 4:
				if peer.state == "authenticated":
					peer.frame_pending = peer.input
				else:
					_handle_frame(peer, peer.input.slice(4))
				peer.input = PackedByteArray()
				peer.needed = -1
				peer.erase("large_prefix")
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
	# Decode the at-most-one completed selected frame outside the shared network
	# pass; its source-bearing JSON and native/getter work are separate stages.
	for peer in schedule:
		if peer.has("frame_pending") and _peers.has(peer):
			var message: PackedByteArray = peer.frame_pending
			peer.erase("frame_pending")
			_handle_frame(peer, message.slice(4))
			break
	if not _active.is_empty() and _active.has("pending"):
		if _active.has("edit_owner"):
			_collect_edit(_active)
		else:
			_collect_pending()


func _close_peer(peer: Dictionary) -> void:
	if _active == peer:
		if peer.has("edit_owner"):
			var owner: Node = peer.edit_owner
			if is_instance_valid(owner):
				owner.call("cancel_owned")
			if _active == peer and (not is_instance_valid(owner) or not owner.call("is_active_stage")):
				_active = {}
		else:
			_active = {}
	peer.socket.disconnect_from_host()
	if peer.has("collector"):
		peer.collector.clear()
		peer.erase("collector")
	if peer.has("edit_collector"):
		peer.edit_collector.clear()
		peer.erase("edit_collector")
	_peers.erase(peer)


# Godot's json_escape emits \v and leaves other C0 bytes raw. Repair only
# those escapes in the already-serialized JSON: a literal "\\v" stays intact.
# Ordinary payloads retain their original UTF-8 buffer without another copy.
static func _json_wire(value: Variant) -> PackedByteArray:
	var json := JSON.stringify(value)
	var bytes := json.to_utf8_buffer()
	var needs_repair := json.contains("\\v")
	if not needs_repair:
		for control in range(32):
			if bytes.has(control):
				needs_repair = true
				break
	if not needs_repair:
		return bytes
	var repairs := PackedInt32Array()
	var extra := 0
	var quoted := false
	var escaped := false
	for index in bytes.size():
		var byte := bytes[index]
		if not quoted:
			if byte == 34:
				quoted = true
			continue
		if escaped:
			escaped = false
			if byte == 118: # Godot's non-JSON vertical-tab escape.
				repairs.append(1 - index) # Negative offset of its backslash.
				extra += 4
			continue
		if byte == 92:
			escaped = true
		elif byte == 34:
			quoted = false
		elif byte < 32:
			repairs.append(index)
			extra += 5
	if repairs.is_empty():
		return bytes
	var result := PackedByteArray()
	result.resize(bytes.size() + extra)
	var source := 0
	var target := 0
	for repair in repairs:
		var offset := -repair if repair < 0 else repair
		while source < offset:
			result[target] = bytes[source]
			source += 1
			target += 1
		result[target] = 92
		result[target + 1] = 117
		result[target + 2] = 48
		result[target + 3] = 48
		if repair < 0:
			result[target + 4] = 48
			result[target + 5] = 98
			source += 2
		else:
			var byte := bytes[source]
			result[target + 4] = "0123456789abcdef".unicode_at(byte >> 4)
			result[target + 5] = "0123456789abcdef".unicode_at(byte & 15)
			source += 1
		target += 6
	while source < bytes.size():
		result[target] = bytes[source]
		source += 1
		target += 1
	return result


func _queue(peer: Dictionary, payload: Dictionary, closing: bool = false, large: bool = false) -> void:
	if not _peers.has(peer):
		return
	var body := _json_wire(payload)
	if body.size() > (MAX_RESPONSE if large else MAX_FRAME):
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


func _integral_control(value: Variant, minimum: int, maximum: int) -> bool:
	if typeof(value) not in [TYPE_INT, TYPE_FLOAT] or value < minimum or value > maximum:
		return false
	return value == int(value)


func _fixed_tuple(value: Variant, count: int, operation: String) -> bool:
	return typeof(value) == TYPE_ARRAY and value.size() == count and _integral_control(value[0], 2, 2) \
		and typeof(value[1]) == TYPE_STRING and value[1] == operation and _valid_id(value[2]) \
		and typeof(value[3]) == TYPE_STRING and value[3] == _session \
		and typeof(value[4]) == TYPE_STRING and value[4] == _project and value[4].to_utf8_buffer().size() <= 1024


func _json_depth_ok(bytes: PackedByteArray) -> bool:
	var depth := 0
	var quoted := false
	var escaped := false
	var finished := false
	var last := 0
	for byte in bytes:
		if quoted:
			if escaped:
				escaped = false
			elif byte == 92:
				escaped = true
			elif byte == 34:
				quoted = false
			continue
		if byte == 32 or byte == 9 or byte == 10 or byte == 13:
			continue
		if finished:
			return false
		if byte == 34:
			quoted = true
		elif byte == 91 or byte == 123:
			if depth == 0 and byte != 91:
				return false
			depth += 1
			if depth > 32:
				return false
		elif byte == 93 or byte == 125:
			if last == 44:
				return false
			depth -= 1
			if depth < 0:
				return false
			if depth == 0:
				finished = true
		elif depth == 0 or byte == 47:
			return false
		last = byte
	return finished and not quoted


func _handle_frame(peer: Dictionary, bytes: PackedByteArray) -> void:
	var text := bytes.get_string_from_utf8()
	if text.to_utf8_buffer() != bytes or not _json_depth_ok(bytes):
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
		peer.auth_capabilities = _capabilities()
		peer.auth_native_revision = _native_revision
		peer.auth_native_build_id = _native_build_id
		var transcript := transcript_bytes(value[2], _session, _project, VERSION, ENGINE_HASH,
			peer.auth_capabilities, peer.auth_native_revision, peer.auth_native_build_id,
			value[5], server_nonce.hex_encode())
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
		if peer.auth_capabilities != _capabilities() or peer.auth_native_revision != _native_revision \
				or peer.auth_native_build_id != _native_build_id:
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
	elif peer.state == "authenticated":
		if typeof(value) == TYPE_ARRAY and value.size() >= 2 and typeof(value[1]) == TYPE_STRING \
				and (value[1] as String).begins_with("edit_"):
			_handle_edit_tuple(peer, value, bytes.size())
			return
		if bytes.size() > MAX_FRAME:
			_close_peer(peer)
			return
		if not _fixed_tuple(value, 6, "observe") and not _fixed_tuple(value, 6, "recheck"):
			_close_peer(peer)
			return
		if value[2] != peer.request_id or typeof(value[5]) != TYPE_STRING:
			_close_peer(peer)
			return
		var operation: String = value[1]
		var path: String = value[5]
		if operation == "observe":
			if peer.has("collector"):
				_close_peer(peer)
				return
			if not _active.is_empty():
				_queue(peer, _failure(peer, path, "read_editor", "unsupported_observation"), true)
				return
			peer.collector = ObservationScript.new()
			peer.observation_since = Time.get_ticks_usec()
			peer.requested_path = path
			peer.pending = "observe"
			_active = peer
		else:
			if _active != peer or not peer.has("collector") or peer.has("pending"):
				_close_peer(peer)
				return
			if path != peer.observation_path:
				_queue(peer, _failure(peer, path, "recheck"), true)
				peer.collector.clear()
				_active = {}
				return
			peer.pending = "recheck"
	else:
		_close_peer(peer)


func _valid_decimal(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING or value.is_empty() or value.length() > 20 \
			or (value.length() > 1 and value[0] == "0"):
		return false
	for character in value:
		if character not in "0123456789":
			return false
	return value.length() < 20 or value <= "18446744073709551615"


func _valid_replacement(value: Variant) -> bool:
	if typeof(value) != TYPE_STRING or value.length() > 512 * 1024:
		return false
	var source: String = value
	var bytes := source.to_utf8_buffer()
	return bytes.size() <= 512 * 1024 and not bytes.has(0) \
		and not source.contains("\r") and not source.contains(char(0xfeff))


func _editor_stamp(started: int) -> Dictionary:
	return {"clock_id": "editor:" + _session, "started_tick_us": str(started),
		"finished_tick_us": str(Time.get_ticks_usec()), "received_elapsed_us": 0}


func _edit_envelope(peer: Dictionary, kind: String) -> Dictionary:
	return {"v": 2, "kind": kind, "request_id": peer.request_id,
		"session_id": _session, "project_root": _project,
		"script_path": peer.get("edit_path", ""), "collection": _editor_stamp(Time.get_ticks_usec())}


func _edit_busy(peer: Dictionary, path: String) -> void:
	var reply := _edit_envelope(peer, "edit_prepared")
	reply.script_path = path
	reply.merge({"status": "busy", "reason": "busy", "intent": null,
		"native": null, "sample": null, "saved_state": null, "context": null,
		"expiry_tick_us": null})
	_queue(peer, reply, true)


func _handle_edit_tuple(peer: Dictionary, value: Array, size: int) -> void:
	var operation: String = value[1]
	var arity := 19 if operation == "edit_prepare" else 7 if operation in ["edit_verify", "edit_recheck"] else 6
	if operation not in ["edit_prepare", "edit_apply", "edit_verify", "edit_recheck",
			"edit_finish", "edit_abort"] or size > (MAX_SELECTED_REQUEST if operation == "edit_prepare" else MAX_FRAME) \
			or not _fixed_tuple(value, arity, operation) or value[2] != peer.request_id \
			or typeof(value[5]) != TYPE_STRING or value[5].to_utf8_buffer().size() > 2048:
		_close_peer(peer)
		return
	var path: String = value[5]
	if operation == "edit_prepare":
		if peer.has("edit_owner") or peer.has("collector"):
			_close_peer(peer)
			return
		if not _active.is_empty():
			_edit_busy(peer, path)
			return
		if not peer.auth_capabilities.edit_open_gdscript or peer.auth_native_revision != 1 \
				or peer.auth_native_build_id != _native_build_id \
				or _edit_owner == null or not is_instance_valid(_edit_owner):
			_queue(peer, _failure(peer, path, "read_editor", "unsupported_capability"), true)
			return
		if not _integral_control(value[6], 1, 9000) \
				or not _valid_id(value[7]) or value[7] == peer.request_id \
				or not _valid_hex(value[16], 64) or not _valid_decimal(value[17]) \
				or not _valid_replacement(value[18]):
			_close_peer(peer)
			return
		for index in range(8, 16):
			if not _valid_decimal(value[index]):
				_close_peer(peer)
				return
		if not path.ends_with(".gd") or path.contains("::") or not _safe_locator(path):
			_queue(peer, _failure(peer, path, "read_editor"), true)
			return
		if not _claim_edit(_edit_owner, peer):
			_edit_busy(peer, path)
			return
		peer.edit_path = path
		peer.expiry_tick_us = Time.get_ticks_usec() + int(value[6]) * 1000
		peer.prepare_tuple = value
		peer.pending = "edit_prepare"
		return
	if _active != peer or not peer.has("edit_owner") or path != peer.edit_path \
			or (peer.has("pending") and (operation not in ["edit_abort", "edit_finish"] \
			or peer.pending in ["edit_abort", "edit_finish"])):
		_close_peer(peer)
		return
	if operation == "edit_apply":
		if peer.get("edit_intent") != "changed" or peer.get("apply_started", false) \
				or peer.has("verified_purpose"):
			_close_peer(peer)
			return
		peer.apply_started = true
		peer.apply_stage = "prepared"
		peer.pending = "edit_apply"
	elif operation in ["edit_verify", "edit_recheck"]:
		if typeof(value[6]) != TYPE_STRING or value[6] not in ["preflight", "post_change", "unchanged"] \
				or (value[6] == "unchanged") != (peer.get("edit_intent") == "unchanged") \
				or (value[6] == "post_change" and not peer.get("apply_done", false)) \
				or (value[6] == "preflight" and (peer.get("apply_started", false) \
				or peer.get("edit_intent") != "changed")):
			_close_peer(peer)
			return
		if operation == "edit_recheck" and peer.get("verified_purpose") != value[6]:
			_close_peer(peer)
			return
		if operation == "edit_verify" and peer.has("verified_purpose"):
			_close_peer(peer)
			return
		peer.purpose = value[6]
		peer.pending = operation
	else:
		peer.pending = operation


func _failure(peer: Dictionary, path: String, stage: String, code: String = "out_of_project") -> Dictionary:
	return {"v": 2, "kind": "failure", "request_id": peer.request_id,
		"session_id": _session, "project_root": _project, "script_path": path,
		"code": code, "stage": stage}


func _safe_locator(path: String) -> bool:
	if _filesystem == null or _filesystem.is_link(_project) or _directory_identity(_project) != _project_identity \
		or not path.begins_with("res://") or path.to_utf8_buffer().size() > 2048:
		return false
	var locator := path.substr(6).split("::", true)
	if locator.size() > 2:
		return false
	var container: String = locator[0]
	if locator.size() == 2:
		var subresource: String = locator[1]
		if not subresource.begins_with("GDScript_") or subresource.length() <= 9 \
			or container.get_extension() not in ["tscn", "tres", "scn", "res"]:
			return false
		for c in subresource:
			var scalar := c.unicode_at(0)
			if not (scalar >= 65 and scalar <= 90 or scalar >= 97 and scalar <= 122 \
				or scalar >= 48 and scalar <= 57 or scalar == 95):
				return false
	var parts := container.split("/", true)
	var current := _project
	for i in parts.size():
		var part: String = parts[i]
		if part.is_empty() or part in [".", ".."] or part.contains("\\") or part.contains(":") \
			or part.contains("?") or part.contains("#"):
			return false
		for c in part:
			var scalar := c.unicode_at(0)
			if scalar < 32 or scalar >= 127 and scalar <= 159:
				return false
		current = current.path_join(part)
		if _filesystem == null or _filesystem.is_link(current):
			return false
		# A missing leaf can belong to an already-open document, but an
		# uninspectable ancestor cannot authorize any Resource text.
		if i < parts.size() - 1 and not DirAccess.dir_exists_absolute(current):
			return false
		if i == parts.size() - 1 and DirAccess.dir_exists_absolute(current):
			return false
	return _directory_identity(_project) == _project_identity

func _scope_witness(path: String) -> PackedStringArray:
	if not _safe_locator(path):
		return PackedStringArray()
	var witness := PackedStringArray([_project_identity])
	var current := _project
	# A built-in's scope belongs to its container; never treat its subresource
	# identifier as a filename or read container bytes as standalone script D.
	var parts := path.get_slice("::", 0).substr(6).split("/", true)
	for i in parts.size():
		current = current.path_join(parts[i])
		var info := _metadata(current)
		if i < parts.size() - 1:
			if info.size() != 5 or info[2] != "Directory":
				return PackedStringArray()
		elif not info.is_empty() and (info.size() != 5 or info[2] != "Regular File"):
			return PackedStringArray()
		witness.append("missing" if info.is_empty() else info[2] + ":" + info[3] + ":" + info[4])
	return witness


func _collect_pending() -> void:
	var peer := _active
	var operation: String = peer.pending
	peer.erase("pending")
	var path: String = peer.get("observation_path", "")
	if operation == "observe":
		path = peer.get("requested_path", "")
	if Time.get_ticks_usec() - int(peer.observation_since) >= PEER_EXPIRY_USEC:
		_close_peer(peer)
		return
	# Validate scope immediately before and after every getter pass. In the
	# event of a file/parent replacement, never serialize collected text.
	var scope_before := _scope_witness(path)
	if scope_before.is_empty():
		_queue(peer, _failure(peer, path, "read_editor" if operation == "observe" else "recheck"), true)
		peer.collector.clear()
		_active = {}
		return
	var reply: Dictionary
	if operation == "observe":
		reply = peer.collector.collect(_session, _project, path)
	else:
		reply = peer.collector.recheck()
	if Time.get_ticks_usec() - int(peer.observation_since) >= PEER_EXPIRY_USEC:
		_close_peer(peer)
		return
	if _scope_witness(path) != scope_before:
		_queue(peer, _failure(peer, path, "read_editor" if operation == "observe" else "recheck"), true)
		peer.collector.clear()
		_active = {}
		return
	reply.request_id = peer.request_id
	var failed: bool = reply.get("kind") == "failure"
	_queue(peer, reply, operation == "recheck" or failed, true)
	if operation == "observe" and not failed:
		peer.observation_path = path
	else:
		peer.collector.clear()
		peer.erase("collector")
		_active = {}


# Fixture bridge subclasses may hold a queued stage at this nonblocking gap.
# Product never yields inside an already-entered native call.
func _edit_ready(_peer: Dictionary, _stage: String) -> bool:
	return true


func _edit_context() -> Variant:
	var started := Time.get_ticks_usec()
	var prefix := "debug/gdscript/warnings/"
	var enabled: Variant = ProjectSettings.get_setting_with_override(prefix + "enable")
	var directories: Variant = ProjectSettings.get_setting_with_override(prefix + "directory_rules")
	var executable := OS.get_executable_path()
	if typeof(enabled) != TYPE_BOOL or typeof(directories) != TYPE_DICTIONARY \
			or executable.is_empty() or not executable.begins_with("/") or directories.size() > 32:
		return null
	var rules := {}
	for key in directories:
		var decision: Variant = directories[key]
		if typeof(key) != TYPE_STRING or typeof(decision) != TYPE_INT or decision not in [0, 1]:
			return null
		rules[key] = decision
	var levels := {}
	for property in ProjectSettings.get_property_list():
		var full_name: String = property.get("name", "")
		if not full_name.begins_with(prefix):
			continue
		var key := full_name.trim_prefix(prefix)
		if key in ["enable", "directory_rules", "exclude_addons", "renamed_in_godot_4_hint",
				"property_used_as_function", "constant_used_as_function", "function_used_as_property"] \
				or key.contains("."):
			continue
		var level: Variant = ProjectSettings.get_setting_with_override(full_name)
		if typeof(level) != TYPE_INT or level < 0 or level > 2 or levels.has(key) or levels.size() >= 64:
			return null
		levels[key] = level
	var classes := []
	for entry in ProjectSettings.get_global_class_list():
		if entry.get("language") != "GDScript":
			continue
		var registered_class: Variant = entry.get("class")
		if typeof(registered_class) not in [TYPE_STRING, TYPE_STRING_NAME] or classes.size() >= 256:
			return null
		classes.append(String(registered_class))
	classes.sort()
	return {"executable_path": executable, "warning_profile": {
		"enable": enabled, "levels": levels, "directory_rules": rules},
		"global_classes": classes, "collection": _editor_stamp(started)}


func _edit_probe(peer: Dictionary, retain: bool = false) -> Dictionary:
	var path: String = peer.edit_path
	var scope := _scope_witness(path)
	if scope.is_empty():
		return {"sample": null, "saved_state": null, "context": null}
	var collector := ObservationScript.new()
	var sample: Dictionary = collector.collect(_session, _project, path)
	sample.request_id = peer.request_id
	var saved: Variant = null
	if sample.get("kind") == "sample":
		var identity: Variant = sample.document.get("identity")
		if identity is Dictionary and identity.get("kind") == "external_gdscript" \
				and identity.has_all(["script_instance_id", "editor_instance_id", "buffer_instance_id"]):
			var document := {"resource_instance_id": identity.script_instance_id,
				"editor_instance_id": identity.editor_instance_id,
				"buffer_instance_id": identity.buffer_instance_id}
			var started := Time.get_ticks_usec()
			var original: String = peer.get("baseline", sample.R.get("text", ""))
			var desired: String = peer.get("desired", original)
			var inspection: Dictionary = _edit_owner.call("inspect", path, original, desired, document)
			var facts: Variant = inspection.get("facts")
			if inspection.get("status") == "observed" and facts is Dictionary \
					and facts.has_all(["current_version", "saved_version", "resource_edited",
					"save_profile", "original_preserved", "desired_preserved"]):
				saved = {"document": identity, "collection": _editor_stamp(started),
					"current_version": facts.current_version, "saved_version": facts.saved_version,
					"resource_edited": facts.resource_edited, "save_profile": facts.save_profile,
					"original_preserved": facts.original_preserved,
					"desired_preserved": facts.desired_preserved}
	var context: Variant = _edit_context()
	if _scope_witness(path) != scope:
		collector.clear()
		return {"sample": null, "saved_state": null, "context": null}
	if retain and sample.get("kind") == "sample":
		if peer.has("edit_collector"):
			peer.edit_collector.clear()
		peer.edit_collector = collector
	else:
		collector.clear()
	return {"sample": sample if sample.get("kind") == "sample" else null,
		"saved_state": saved, "context": context}


func _edit_preparation_reason(peer: Dictionary, sample: Variant, saved: Variant, context: Variant) -> String:
	if not (sample is Dictionary):
		return "unavailable_observation"
	var open_state: Variant = sample.document.get("open_state")
	if open_state is Dictionary and open_state.get("value") == "not_open":
		return "closed_target"
	if not (open_state is Dictionary) or open_state.get("value") != "open" \
			or sample.document.get("validity", {}).get("value") != "valid":
		return "unavailable_observation"
	var identity: Variant = sample.document.get("identity")
	if not (identity is Dictionary) or not identity.has_all(
			["kind", "resource_path", "script_instance_id", "editor_instance_id", "buffer_instance_id"]):
		return "unavailable_observation"
	var tuple: Array = peer.prepare_tuple
	if identity.kind != "external_gdscript" or identity.resource_path != peer.edit_path \
			or identity.script_instance_id != tuple[12] \
			or identity.editor_instance_id != tuple[13] or identity.buffer_instance_id != tuple[14]:
		return "identity_changed"
	if sample.R.get("availability") != "observed" or sample.B.get("availability") != "observed" \
			or sample.dirty.get("availability") != "observed":
		return "unavailable_observation"
	if sample.dirty.get("state") == "dirty":
		return "dirty"
	if sample.dirty.get("state") != "clean":
		return "unavailable_observation"
	if sample.R.text != sample.B.text:
		return "divergence"
	var original: String = sample.R.text
	if original.contains("\r") or original.to_utf8_buffer().has(0) or original.contains(char(0xfeff)):
		return "unsupported_representation"
	if not (saved is Dictionary) or not (context is Dictionary):
		return "unavailable_observation"
	var saved_identity: Variant = saved.get("document")
	if not (saved_identity is Dictionary):
		return "unavailable_observation"
	if saved_identity != identity:
		return "identity_changed"
	if saved.resource_edited or saved.current_version != saved.saved_version:
		return "dirty"
	var witness: Variant = sample.B.get("witness")
	if not (witness is Dictionary) or not witness.has_all(["script_instance_id",
			"editor_instance_id", "buffer_instance_id", "source_version"]):
		return "unavailable_observation"
	if witness.script_instance_id != tuple[12] or witness.editor_instance_id != tuple[13] \
			or witness.buffer_instance_id != tuple[14]:
		return "identity_changed"
	if witness.source_version != tuple[15] or saved.current_version != tuple[15] \
			or original.sha256_text() != tuple[16] or str(original.to_utf8_buffer().size()) != tuple[17]:
		return "revision_mismatch"
	var project_info := _metadata(_project)
	var file_info := _metadata(_project.path_join(peer.edit_path.substr(6)))
	if project_info.size() != 5 or file_info.size() != 5:
		return "unavailable_observation"
	if project_info[3] != tuple[8] or project_info[4] != tuple[9] \
			or file_info[3] != tuple[10] or file_info[4] != tuple[11]:
		return "identity_changed"
	if not saved.original_preserved or not saved.desired_preserved:
		return "save_would_reformat"
	return ""


func _collect_edit(peer: Dictionary) -> void:
	var operation: String = peer.pending
	var stage: String = operation
	if operation == "edit_apply":
		stage = "apply" if peer.apply_stage == "prepared" else peer.apply_stage
	elif operation in ["edit_verify", "edit_recheck"]:
		stage = operation.trim_prefix("edit_") + ":" + peer.purpose
	elif operation == "edit_prepare":
		stage = "prepare"
	if not peer.output.is_empty() or not _edit_ready(peer, stage):
		return
	var operation_started := Time.get_ticks_usec()
	peer.erase("pending")
	if operation == "edit_prepare":
		var tuple: Array = peer.prepare_tuple
		peer.desired = tuple[18]
		var inspected := _edit_probe(peer, true)
		var reply := _edit_envelope(peer, "edit_prepared")
		reply.merge({"status": "refused", "reason": _edit_preparation_reason(peer,
			inspected.sample, inspected.saved_state, inspected.context),
			"intent": null, "native": null, "sample": inspected.sample,
			"saved_state": inspected.saved_state, "context": inspected.context,
			"expiry_tick_us": str(peer.expiry_tick_us)})
		if reply.reason == "":
			peer.baseline = inspected.sample.R.text
			peer.edit_intent = "unchanged" if peer.desired == peer.baseline else "changed"
			peer.verified_purpose = "unchanged" if peer.edit_intent == "unchanged" else "preflight"
			if peer.edit_intent == "unchanged":
				reply.status = "prepared"
				reply.reason = ""
				reply.intent = "unchanged"
			else:
				var binding := {"resource_instance_id": tuple[12], "editor_instance_id": tuple[13],
					"buffer_instance_id": tuple[14]}
				var correlation := {"request_id": peer.request_id, "session_id": _session,
					"document": binding, "expiry_tick_us": str(peer.expiry_tick_us),
					"project_device": tuple[8], "project_inode": tuple[9],
					"target_device": tuple[10], "target_inode": tuple[11],
					"expected_version": tuple[15], "expected_sha256": tuple[16],
					"expected_length": tuple[17]}
				var native: Dictionary = _edit_owner.call("begin", peer.edit_path, peer.baseline, peer.desired, correlation)
				reply.native = native if native.has("facts") and native.has("stage") else null
				reply.status = "prepared" if native.get("status") == "prepared" else "refused"
				reply.reason = native.get("reason", "")
				reply.intent = "changed" if reply.status == "prepared" else null
		peer.erase("prepare_tuple")
		reply.collection = _editor_stamp(operation_started)
		if reply.status != "prepared":
			_queue(peer, reply, true, true)
			if _active == peer:
				_edit_owner.call("cancel_owned")
				_active = {}
		else:
			_queue(peer, reply, false, true)
		return
	if operation == "edit_apply":
		var before := {"sample": null, "saved_state": null}
		if peer.apply_stage in ["mtime_restored", "edited_cleared"]:
			before = _edit_probe(peer)
		if _active != peer:
			return
		peer.native_stage_authorized = peer.apply_stage
		var native: Dictionary = _edit_owner.call("advance_bound", peer.request_id, peer.apply_stage)
		peer.erase("native_stage_authorized")
		if _active != peer:
			return
		var after := _edit_probe(peer)
		var stamp := _editor_stamp(operation_started)
		var event := {"stage": native.get("stage", peer.apply_stage), "native": native,
			"collection": stamp, "before_sample": before.sample,
			"after_sample": after.sample, "before_saved_state": before.saved_state,
			"after_saved_state": after.saved_state}
		# Progress carries each full source snapshot exactly once.
		if native.get("status") in ["ready", "complete"]:
			if native.get("status") == "complete":
				peer.apply_done = true
				peer.last_native_status = "complete"
				peer.final_native = native
				if not peer.has("pending"):
					peer.pending = "edit_applied"
			else:
				peer.apply_stage = native.get("stage", "")
				if not peer.has("pending"):
					peer.pending = "edit_apply"
			var progress := _edit_envelope(peer, "edit_progress")
			progress.collection = stamp
			progress.event = event
			_queue(peer, progress, false, true)
			return
		peer.apply_done = true
		peer.last_native_status = native.get("status", "partial")
		var reply := _edit_envelope(peer, "edit_applied")
		reply.merge({"status": native.get("status", "partial"), "reason": native.get("reason", ""),
			"native": native, "events": [event]})
		reply.collection = stamp
		_queue(peer, reply, false, true)
		return
	if operation == "edit_applied":
		var native: Dictionary = peer.final_native
		peer.erase("final_native")
		var reply := _edit_envelope(peer, "edit_applied")
		reply.merge({"status": native.get("status", "complete"), "reason": native.get("reason", ""),
			"native": native, "events": []})
		_queue(peer, reply, false, true)
		return
	if operation == "edit_verify":
		var inspected := _edit_probe(peer, true)
		peer.verified_purpose = peer.purpose
		var reply := _edit_envelope(peer, "edit_sample")
		reply.merge({"purpose": peer.purpose, "sample": inspected.sample,
			"saved_state": inspected.saved_state, "context": inspected.context})
		reply.collection = _editor_stamp(operation_started)
		_queue(peer, reply, false, true)
		return
	if operation == "edit_recheck":
		var recheck: Variant = null
		if peer.has("edit_collector"):
			var scope := _scope_witness(peer.edit_path)
			if not scope.is_empty():
				recheck = peer.edit_collector.recheck()
				recheck.request_id = peer.request_id
				if _scope_witness(peer.edit_path) != scope:
					recheck = null
			peer.edit_collector.clear()
			peer.erase("edit_collector")
		if recheck == null:
			recheck = _edit_envelope(peer, "recheck")
			recheck.merge({"checks": "unavailable", "detected_changes": [],
				"reason": "unavailable"})
		peer.erase("verified_purpose")
		var inspected := _edit_probe(peer)
		var reply := _edit_envelope(peer, "edit_rechecked")
		reply.merge({"purpose": peer.purpose, "recheck": recheck,
			"saved_state": inspected.saved_state, "context": inspected.context})
		reply.collection = _editor_stamp(operation_started)
		_queue(peer, reply, false, true)
		return
	if operation in ["edit_finish", "edit_abort"]:
		var native: Variant = null
		if peer.get("edit_intent") == "changed":
			native = _edit_owner.call("finish_bound", peer.request_id)
		var never_entered: bool = not peer.get("apply_started", false) \
			or (peer.get("apply_stage") == "prepared" and native is Dictionary \
			and native.get("status") == "refused" and native.get("stage") == "prepared") \
			or (peer.get("apply_done", false) and peer.get("apply_stage") == "prepared" \
			and peer.get("last_native_status") == "refused")
		# The native context is automatically consumed after a terminal stage;
		# wrong_attempt here is cleanup feedback, not a new application refusal.
		if peer.get("apply_done", false) and native is Dictionary \
				and native.get("reason") == "wrong_attempt":
			native = null
		var reply := _edit_envelope(peer, "edit_finished" if operation == "edit_finish" else "edit_aborted")
		reply.merge({"status": "terminal", "reason": "", "terminal_before_boundary": never_entered,
			"native": native})
		reply.collection = _editor_stamp(operation_started)
		if _active == peer:
			_active = {}
		_queue(peer, reply, true)


func _reply_fields(peer: Dictionary, kind: String) -> Dictionary:
	return {"v": 2, "kind": kind, "request_id": peer.request_id,
		"session_id": _session, "project_root": _project,
		"godot_version": VERSION, "engine_hash": ENGINE_HASH,
		"capabilities": peer.auth_capabilities, "native_api_revision": peer.auth_native_revision,
		"native_build_id": peer.auth_native_build_id, "client_nonce": peer.client_nonce,
		"server_nonce": peer.server_nonce}


static func _field(data: PackedByteArray) -> PackedByteArray:
	var length := data.size()
	var out := PackedByteArray([length >> 24 & 255, length >> 16 & 255, length >> 8 & 255, length & 255])
	out.append_array(data)
	return out


static func transcript_bytes(request_id: String, session_id: String, project_root: String,
		godot_version: String, engine_hash: String, capabilities: Dictionary,
		native_api_revision: int, native_build_id: String,
		client_nonce: String, server_nonce: String) -> PackedByteArray:
	if session_id.length() != 32 or client_nonce.length() != 64 or server_nonce.length() != 64 \
			or native_api_revision not in [0, 1] or (native_api_revision == 0 and not native_build_id.is_empty()) \
			or (native_api_revision == 1 and (native_build_id.length() != 64 \
			or not native_build_id.is_valid_hex_number() or native_build_id != native_build_id.to_lower())):
		return PackedByteArray()
	var out := PackedByteArray()
	for part in [DOMAIN, request_id]:
		out.append_array(_field(part.to_utf8_buffer()))
	out.append_array(_field(session_id.hex_decode()))
	for part in [project_root, godot_version, engine_hash]:
		out.append_array(_field(part.to_utf8_buffer()))
	for name in ["observe_gdscript", "open_enumeration", "buffer_attribution", "unsaved_paths",
			"cached_resource_lookup", "edit_open_gdscript"]:
		if not capabilities.has(name) or typeof(capabilities[name]) != TYPE_BOOL:
			return PackedByteArray()
		out.append_array(_field(PackedByteArray([1 if capabilities[name] else 0])))
	out.append_array(_field(PackedByteArray([0, 0, 0, native_api_revision])))
	out.append_array(_field(native_build_id.to_utf8_buffer()))
	out.append_array(_field(client_nonce.hex_decode()))
	out.append_array(_field(server_nonce.hex_decode()))
	return out


static func role_proof(secret: PackedByteArray, role: String, transcript: PackedByteArray) -> PackedByteArray:
	if secret.size() != 32 or role not in ["server", "client", "finish"] or transcript.is_empty():
		return PackedByteArray()
	var message := _field(role.to_ascii_buffer())
	message.append_array(transcript)
	return Crypto.new().hmac_digest(HashingContext.HASH_SHA256, secret, message)
