@tool
extends "res://addons/fixture_driver/close_fixture_driver.gd"

# Owned T001 controls: setup/races and independent public editor/cache witnesses.
# None of these actions reduces a product outcome or supplies observed source to it.
var _closed_hold_stage := ""
var _closed_hold_kind := ""
var _closed_announced := false
var _closed_peer := {}
var _closed_callback_stage := ""
var _closed_callback_action := ""
var _closed_effect_seen := false
var _closed_events: Array[Dictionary] = []

func _enter_tree() -> void:
	super._enter_tree()
	Engine.set_meta("godot_agent_kit_closed_fixture_callback", _closed_entered)

func _exit_tree() -> void:
	Engine.remove_meta("godot_agent_kit_closed_fixture_callback")
	super._exit_tree()

func _closed_publish(stage: String, request_id: String) -> void:
	var file := FileAccess.open(_control.path_join("closed-event.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify({"stage": stage, "request_id": request_id}))
		file.close()

func closed_barrier(stage: String, peer: Dictionary) -> bool:
	_closed_peer = peer
	if stage != _closed_hold_stage: return true
	if not _closed_announced:
		_closed_publish(stage, peer.get("request_id", ""))
		_closed_announced = true
	return false

func closed_response_barrier(kind: String, peer: Dictionary, _reply: Dictionary) -> bool:
	_closed_peer = peer
	if kind != _closed_hold_kind: return true
	if not _closed_announced:
		_closed_publish("response:" + kind, peer.get("request_id", ""))
		_closed_announced = true
	return false

func _closed_entered(request_id: String, stage: String) -> void:
	if _closed_events.size() < 32:
		_closed_events.append({"request_id": request_id, "stage": stage, "state": _close_state()})
	if stage in ["after_write", "after_mtime"]: _closed_effect_seen = true
	if stage == "before_verify" and not _closed_effect_seen: return
	if stage != _closed_callback_stage: return
	_closed_callback_stage = ""
	match _closed_callback_action:
		"open": _prepare(CLOSE_TARGET)
		"aba":
			_prepare(CLOSE_TARGET)
			EditorInterface.get_script_editor().close_file(CLOSE_TARGET)
			_close_release_target_refs()
		"newer_resource":
			var cached := ResourceLoader.get_cached_ref(CLOSE_TARGET) as GDScript
			if cached != null: cached.set_source_code("extends RefCounted\n# CLOSED_NEWER_WORK\n")
		"newer_disk":
			var file := FileAccess.open(CLOSE_TARGET, FileAccess.WRITE)
			if file != null:
				file.store_string("extends RefCounted\n# CLOSED_NEWER_WORK\n")
				file.close()
		"cache_appear": _close_target_ref = load(CLOSE_TARGET) as GDScript
		"settings":
			var file := FileAccess.open("res://project.godot", FileAccess.READ_WRITE)
			if file != null:
				file.seek_end()
				file.store_string("\n[gdextension]\nfixture_unsupported=true\n")
				file.close()
		"stop": _open_bridge().stop()
		"disable": EditorInterface.set_plugin_enabled(PRODUCT, false)
	if _closed_events.size() < 32:
		_closed_events.append({"request_id": request_id, "stage": "callback:" + stage,
			"action": _closed_callback_action, "state": _close_state()})

func _dispatch_close_request(request: Dictionary) -> void:
	if not String(request.get("action", "")).begins_with("closed_"):
		super._dispatch_close_request(request)
		return
	var id: Variant = request.get("id")
	if typeof(id) != TYPE_STRING or id.is_empty() or id.length() > 64 or id == _last_id: return
	_last_id = id
	var response := {"id": id, "action": request.action, "ok": true}
	var api: Dictionary = Engine.get_meta(NATIVE_META, {})
	match String(request.action):
		"closed_arm":
			_closed_hold_stage = request.get("stage", "")
			_closed_hold_kind = request.get("kind", "")
			_closed_callback_stage = request.get("callback_stage", "")
			_closed_callback_action = request.get("callback_action", "")
			_closed_announced = false
			_closed_events.clear()
			_closed_effect_seen = false
		"closed_info":
			response.complete_family = api.has_all(["closed_inspect", "closed_prepare", "closed_apply", "closed_verify",
				"closed_recheck", "closed_finish", "closed_abort", "closed_expire"])
			response.fixture_faults = api.has("closed_fixture_fault") and api.has("closed_fixture_state")
		"closed_release":
			_closed_hold_stage = ""
			_closed_hold_kind = ""
			_closed_announced = false
		"closed_save_profile":
			var key: String = request.get("key", "")
			response.ok = key in ["trim_trailing_whitespace_on_save", "trim_final_newlines_on_save", "convert_indent_on_save"] \
				and typeof(request.get("value")) == TYPE_BOOL
			if response.ok:
				var setting := "text_editor/behavior/files/" + key
				var settings := EditorInterface.get_editor_settings()
				if not _close_settings.has(setting): _close_settings[setting] = settings.get_setting(setting)
				settings.set_setting(setting, request.value)
		"closed_fault":
			response.ok = api.has("closed_fixture_fault")
			if response.ok: response.result = api.closed_fixture_fault.call(request.get("request_id", ""), request.get("fault", ""))
		"closed_disconnect":
			response.ok = _closed_peer.has("socket")
			if response.ok: _closed_peer.socket.disconnect_from_host()
		"closed_fixture_activate":
			var bridge := _open_bridge()
			var owner := bridge.get_parent().get_node_or_null("GodotAgentKitScriptClosedEdit") if bridge != null else null
			var prefix := "res://addons/godot_agent_kit/native/"
			var manifest: Variant = JSON.parse_string(FileAccess.get_file_as_string(prefix + "build-manifest.json"))
			response.ok = owner != null and manifest is Dictionary and manifest.get("fixture_only") == true \
				and manifest.get("native_api_revision") == 4 and api.has_all(["closed_fixture_fault", "closed_fixture_state"]) \
				and api.api_revision.call() == 4 and api.build_id.call() == manifest.get("native_build_id") \
				and manifest.get("native_library_sha256") == FileAccess.get_sha256(prefix + "libeditor_integration.macos.arm64.dylib")
			if response.ok: bridge.attach_closed(owner, 4, api.build_id.call())
		"closed_reconfigure":
			response.ok = api.has("configure") and api.configure.call(_open_bridge().get("_session"))
		"closed_native_state":
			response.ok = api.has("closed_fixture_state")
			if response.ok: response.result = api.closed_fixture_state.call(request.get("request_id", ""))
		"closed_witness":
			response.state = _close_state()
			response.state.cache_has = ResourceLoader.has_cached(CLOSE_TARGET)
			var cached := ResourceLoader.get_cached_ref(CLOSE_TARGET)
			response.state.cache_type = cached.get_class() if cached != null else null
			response.events = _closed_events.duplicate(true)
		"closed_resource":
			var cached := ResourceLoader.get_cached_ref(CLOSE_TARGET) as GDScript
			var mutation: String = request.get("mutation", "")
			if mutation == "appear":
				_close_target_ref = load(CLOSE_TARGET) as GDScript
				cached = _close_target_ref
			response.ok = cached != null
			if response.ok:
				match mutation:
					"divergent": cached.set_source_code("extends RefCounted\n# CLOSED_RESOURCE_DIVERGENCE\n")
					"dirty": cached.source_code = cached.get_source_code() + "# CLOSED_RESOURCE_DIRTY\n"
					"equal_dirty":
						var source := cached.get_source_code()
						cached.source_code = source + "# CLOSED_TEMPORARY_DIRTY\n"
						cached.source_code = source
					"replace":
						var replacement := GDScript.new()
						replacement.set_source_code(cached.get_source_code())
						replacement.reload(false)
						replacement.take_over_path(CLOSE_TARGET)
						_close_target_ref = replacement
					"appear": pass
					_: response.ok = false
			response.state = _close_state()
		_: response.ok = false
	_respond(response)
