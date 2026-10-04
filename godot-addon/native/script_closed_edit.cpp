#include "editor_context.hpp"

#include <fcntl.h>
#include <unistd.h>
#include <algorithm>
#include <cerrno>
#include <limits>

namespace gak {
struct ClosedAttempt {
    enum Phase { Prepared, Resource, Content, Metadata } phase = Prepared;
    std::string request, session, path, before, after, profile, context_hash, save_hash, reason;
    uint64_t expires{}, epoch{}, rid{};
    FileBinding file, settings_file;
    std::string settings_source;
    struct stat original{}, verified_metadata{};
    Value script;
    OpeningEffective effective;
    bool present{}, terminal{}, resource_entered{}, resource_changed{}, disk_entered{}, truncated{}, synced{}, readback{}, restore_entered{}, restored{};
    size_t written{};
#if GAK_FIXTURE
    std::string fault;
#endif
};
namespace {
void *singleton(const char *s) { Name n(s); return api.singleton(n.ptr()); }
#if GAK_FIXTURE
std::string pending_fault;
#endif
struct Running {
    Session &s;
    explicit Running(Session &session) : s(session) { s.call_state = Session::CallState::Running; }
    ~Running() { bool closing = s.call_state == Session::CallState::Closing; s.call_state = Session::CallState::Idle; if (closing) { close(s); } }
};
void epoch_advance(Session &s) {
    if (s.closed_epoch == UINT64_MAX) { s.closed_epoch_valid = false; }
    else { ++s.closed_epoch; }
}
void epoch_callback(void *userdata, const GDExtensionConstVariantPtr *, GDExtensionInt,
        GDExtensionVariantPtr, GDExtensionCallError *error) {
    error->error = GDEXTENSION_CALL_OK;
    auto &s = *static_cast<Session *>(userdata);
    epoch_advance(s);
}
bool equal(Value a, const Value &b) {
    if (a.type() != b.type()) { return false; }
    if (a.type() == GDEXTENSION_VARIANT_TYPE_STRING) { return bytes(a) == bytes(b); }
    if (a.type() == GDEXTENSION_VARIANT_TYPE_NIL) { return true; }
    if (a.type() == GDEXTENSION_VARIANT_TYPE_BOOL) { return truth(a) == truth(b); }
    if (a.type() == GDEXTENSION_VARIANT_TYPE_INT) { int64_t x = 0, y = 0; return number(a, x) && number(b, y) && x == y; }
    Value result;
    return a.type() == GDEXTENSION_VARIANT_TYPE_DICTIONARY && checked_invoke(result, a, "recursive_equal", {&b}) && truth(result);
}
bool same_revision(const struct stat &a, const struct stat &b) {
    return same(a, b) && a.st_size == b.st_size && a.st_mode == b.st_mode && a.st_uid == b.st_uid &&
            same_time(a.st_mtimespec, b.st_mtimespec) && same_time(a.st_ctimespec, b.st_ctimespec);
}
Value stamp(timespec t) { Value v = dict(); put_string(v, "seconds", std::to_string(t.tv_sec)); put(v, "nanoseconds", integer(t.tv_nsec)); return v; }
Value revision(const FileBinding &file, const struct stat &st, const std::string &source) {
    Value v = dict(); put_string(v, "device", std::to_string(file.device)); put_string(v, "inode", std::to_string(file.inode));
    put_string(v, "utf8_bytes", std::to_string(source.size())); put_string(v, "sha256", sha256(source));
    put(v, "mtime", stamp(st.st_mtimespec)); put(v, "ctime", stamp(st.st_ctimespec)); return v;
}
const char *read_file(int fd, std::string &source, struct stat &st) {
    if (fstat(fd, &st) || st.st_size < 0 || static_cast<uint64_t>(st.st_size) > SOURCE_LIMIT) { return "file_metadata_unavailable"; }
    source.resize(static_cast<size_t>(st.st_size)); size_t offset = 0;
    while (offset < source.size()) { ssize_t n = pread(fd, source.data() + offset, source.size() - offset, offset); if (n < 0 && errno == EINTR) { continue; } if (n <= 0) { return "file_read_unavailable"; } offset += n; }
    struct stat after{}; if (fstat(fd, &after) || !same_revision(st, after)) { return "file_revision_changed"; }
    if (!valid_utf8(source)) { return "file_utf8_unavailable"; } return nullptr;
}
const char *settings_guard(Session &s, const ClosedAttempt &a) {
    return file_attached(s, a.settings_file) && content(a.settings_file.leaf.value, a.settings_source)
            ? nullptr : "validator_settings_changed";
}
const char *capture_settings(Session &s, ClosedAttempt &a) {
    if (const char *r = pin_file(s, "res://project.godot", a.settings_file, O_RDONLY, true)) { return r; }
    struct stat st{};
    if (const char *r = read_file(a.settings_file.leaf.value, a.settings_source, st)) { return r; }
    return settings_guard(s, a);
}
const char *cache(const std::string &path, Value &script, bool &present) {
    Value target = string(path), has, found;
    if (!checked_call(has, singleton("ResourceLoader"), "ResourceLoader", "has_cached", GAK_HASH_HAS_CACHED, {&target}) || has.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            !checked_call(found, singleton("ResourceLoader"), "ResourceLoader", "get_cached_ref", GAK_HASH_CACHED_REF, {&target})) { return "cache_unavailable"; }
    present = truth(has);
    if (!present) { return found.type() == GDEXTENSION_VARIANT_TYPE_NIL || (found.type() == GDEXTENSION_VARIANT_TYPE_OBJECT && !object_ptr(found)) ? nullptr : "cache_disagreement"; }
    Name gd("GDScript"); if (!object_ptr(found) || !api.class_tag(gd.ptr()) || !api.cast_to(object_ptr(found), api.class_tag(gd.ptr()))) { return "unsupported_resource"; }
    Value actual; std::string actual_path;
    if (!checked_call(actual, object_ptr(found), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) || !checked_bytes(actual, actual_path, 2048) || actual_path != path) { return "resource_path_changed"; }
    script = found; return nullptr;
}
const char *absence(const std::string &path) {
    Binding binding; bool found = false; if (!find_document(path, binding, found)) { return "roster_unavailable"; } if (found) { return "target_open"; }
    Value editor, files, dirty, target = string(path);
    if (!checked_call(editor, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
            !checked_call(files, object_ptr(editor), "ScriptEditor", "get_unsaved_files", GAK_HASH_UNSAVED) || files.type() != GDEXTENSION_VARIANT_TYPE_PACKED_STRING_ARRAY ||
            !checked_invoke(dirty, files, "has", {&target}) || dirty.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "unsaved_roster_unavailable"; }
    return truth(dirty) ? "target_unsaved" : nullptr;
}
// Closed Save compatibility uses settings directly, never invents a CodeEdit.
const char *save_profile(std::string_view source, std::string &hash) {
    Value settings; if (!checked_call(settings, singleton("EditorInterface"), "EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS)) { return "save_profile_unavailable"; }
    const char *keys[] = {"text_editor/behavior/files/trim_trailing_whitespace_on_save", "text_editor/behavior/files/trim_final_newlines_on_save", "text_editor/behavior/files/convert_indent_on_save", "text_editor/behavior/indent/type", "text_editor/behavior/indent/size"};
    bool flags[3]{}; int64_t kind = -1, size = -1; std::string encoded;
    for (size_t i = 0; i < std::size(keys); ++i) {
        Value key = string(keys[i]), value;
        if (!checked_call(value, object_ptr(settings), "EditorSettings", "get_setting", GAK_HASH_GET_SETTING, {&key})) { return "save_profile_unavailable"; }
        if (i < 3) { if (value.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "save_profile_unavailable"; } flags[i] = truth(value); encoded += flags[i] ? "1:" : "0:"; }
        else { int64_t n = -1; if (!number(value, n)) { return "save_profile_unavailable"; } if (i == 3) { kind = n; } else { size = n; } encoded += std::to_string(n) + ":"; }
    }
    if ((kind != 0 && kind != 1) || size < 1 || size > 1024) { return "save_profile_unavailable"; }
    if (flags[1] && (source == "\n" || (source.size() >= 2 && source.substr(source.size() - 2) == "\n\n"))) { return "save_would_change_source"; }
    bool beginning = true; int64_t spaces = 0;
    for (size_t i = 0; i < source.size(); ++i) {
        char c = source[i]; if (c == '\n') { beginning = true; spaces = 0; continue; }
        if (flags[0] && (c == ' ' || c == '\t') && (i + 1 == source.size() || source[i + 1] == '\n')) { return "save_would_change_source"; }
        if (beginning && (c == ' ' || c == '\t')) {
            if (flags[2] && ((kind == 1 && c == '\t') || (kind == 0 && c == ' ' && ++spaces >= size))) { return "save_would_change_source"; }
        } else { beginning = false; }
    }
    hash = sha256(encoded); return nullptr;
}
struct Sample { Value state, context, script; bool present{}; std::string source, profile, save; struct stat st{}; };
const char *sample(Session &s, const std::string &request, const std::string &path, const FileBinding &file, Sample &out) {
    const uint64_t epoch = s.closed_epoch;
    if (!s.closed_epoch_valid || !s.closed_signal_callable || !api.from_id(s.closed_signal_owner)) { return "close_epoch_unavailable"; }
    Value signal = name("script_close"), connected;
    if (!checked_call(connected, api.from_id(s.closed_signal_owner), "Object", "is_connected", GAK_HASH_IS_CONNECTED, {&signal, s.closed_signal_callable}) || !truth(connected)) { return "close_epoch_unavailable"; }
    if (!file_attached(s, file)) { return "namespace_changed"; }
    if (const char *r = absence(path)) { return r; }
    if (const char *r = read_file(file.leaf.value, out.source, out.st)) { return r; }
    if (const char *r = opening_path_guard(s, file)) { return r; }
    if (const char *r = cache(path, out.script, out.present)) { return r; }
    Value resource = dict(); put_string(resource, "state", out.present ? "present" : "absent");
    for (const char *key : {"instance_id", "path", "source", "edited", "profile_sha256"}) { put(resource, key, Value()); }
    OpeningEffective effective; if (const char *r = opening_effective(effective)) { return r; }
    if (const char *r = save_profile(out.source, out.save)) { return r; }
    std::string actual = out.source;
    if (out.present) {
        Value text, edited;
        if (!checked_call(text, object_ptr(out.script), "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) || !checked_bytes(text, actual, SOURCE_LIMIT) ||
                !checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&out.script}) || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "resource_unavailable"; }
        put_string(resource, "instance_id", std::to_string(id(out.script))); put_string(resource, "path", path); put_string(resource, "source", actual); put(resource, "edited", edited);
    }
    OpeningContext context;
    if (const char *r = closed_context(s, request, path, actual, effective, out.script, context, out.profile)) { return r; }
    out.profile = sha256(out.profile + out.save);
    if (out.present) { put_string(resource, "profile_sha256", out.profile); }
    out.context = dict(); put(out.context, "projection", context.projection); put_string(out.context, "source", actual); put_string(out.context, "sha256", context.guard_hash);
    out.state = dict(); put_string(out.state, "project_device", std::to_string(s.device)); put_string(out.state, "project_inode", std::to_string(s.inode));
    put(out.state, "file_revision", revision(file, out.st, out.source)); put_string(out.state, "close_epoch", std::to_string(s.closed_epoch)); put_string(out.state, "lifecycle", "closed"); put(out.state, "resource", resource);
    struct stat final_stat{};
    if (fstat(file.leaf.value, &final_stat) || !same_revision(out.st, final_stat) ||
            !content(file.leaf.value, out.source)) { return "file_revision_changed"; }
    Value final_script; bool final_present = false;
    if (const char *r = cache(path, final_script, final_present)) { return r; }
    if (final_present != out.present || (final_present && id(final_script) != id(out.script))) { return "cache_branch_or_identity_changed"; }
    if (const char *r = absence(path)) { return r; }
    if (!file_attached(s, file)) { return "namespace_changed"; }
    if (!s.closed_epoch_valid || s.closed_epoch != epoch) { return "close_epoch_changed"; }
    return nullptr;
}
Value expected_resource(const Sample &sample) {
    Value out = get(sample.state, "resource");
    // The checked intent binds a digest/length instead of duplicating R text.
    Value v = dict(); for (const char *k : {"state", "instance_id", "path", "edited", "profile_sha256"}) { put(v, k, get(out, k)); }
    put(v, "source_sha256", sample.present ? string(sha256(bytes(get(out, "source")))) : Value());
    put(v, "utf8_bytes", sample.present ? string(std::to_string(utf8_size(get(out, "source")))) : Value()); return v;
}
Value receipt(const ClosedAttempt &a) {
    Value v = dict(); put_string(v, "request_id", a.request); put_string(v, "session_id", a.session); put_string(v, "script_path", a.path); put_string(v, "native_build_id", GAK_BUILD_ID); put(v, "native_api_revision", integer(4));
    const char *phases[] = {"prepared", "resource_applied", "content_persisted", "mtime_restored"}; put_string(v, "phase", phases[a.phase]);
    put(v, "resource_entered", boolean(a.resource_entered)); put(v, "resource_changed", boolean(a.resource_changed)); put(v, "disk_entered", boolean(a.disk_entered)); put_string(v, "written_bytes", std::to_string(a.written));
    put(v, "truncate_done", boolean(a.truncated)); put(v, "fsync_done", boolean(a.synced)); put(v, "pread_done", boolean(a.readback)); put(v, "futimens_called", boolean(a.restore_entered)); put(v, "mtime_restored", boolean(a.restored)); put(v, "terminal_discard", boolean(a.terminal)); put(v, "reason", a.reason.empty() ? Value() : string(a.reason)); return v;
}
Value reply(const ClosedAttempt *a, const char *status, const char *reason, const Sample *observed = nullptr) {
    Value out = dict(); put_string(out, "status", status); put(out, "reason", reason && *reason ? string(reason) : Value());
    put(out, "expiry_tick_us", a ? string(std::to_string(a->expires)) : Value()); put(out, "native", a ? receipt(*a) : Value()); put(out, "state", observed ? observed->state : Value()); put(out, "context", observed ? observed->context : Value()); return out;
}
Value fail(ClosedAttempt &a, const char *reason) { a.terminal = true; if (a.reason.empty()) { a.reason = reason; } return reply(&a, a.resource_entered || a.disk_entered ? "partial" : "refused", a.reason.c_str()); }
bool owned(ClosedAttempt *a, const Value &request) { std::string r; return a && checked_bytes(request, r, 64) && r == a->request; }
const char *owner_guard(Session &s, const ClosedAttempt &a) {
    if (a.terminal) { return "terminal_discard"; }
    if (s.call_state == Session::CallState::Closing || s.session_id != a.session || ticks() >= a.expires) { return "session_or_expiry_changed"; }
    if (!s.closed_epoch_valid || s.closed_epoch != a.epoch) { return "close_epoch_changed"; } return nullptr;
}
#if GAK_FIXTURE
void fixture(ClosedAttempt &a, const char *stage) {
    Value key = name("godot_agent_kit_closed_fixture_callback"), fallback, callback;
    if (checked_call(callback, singleton("Engine"), "Object", "get_meta", GAK_HASH_GET_META, {&key, &fallback}) && callback.type() == GDEXTENSION_VARIANT_TYPE_CALLABLE) {
        Value request = string(a.request), reached = string(stage), result; checked_invoke(result, callback, "call", {&request, &reached});
    }
}
#endif
const char *guard(Session &s, ClosedAttempt &a, bool disk_after, bool resource_after, Sample &current, bool metadata = true) {
    if (const char *r = owner_guard(s, a)) { return r; }
    if (const char *r = settings_guard(s, a)) { return r; }
#if GAK_FIXTURE
    if (a.fault == "missing_cache_getters") { return "cache_unavailable"; } if (a.fault == "missing_roster") { return "roster_unavailable"; }
    if (a.fault == "missing_context") { return "closed_context_unavailable"; }
#endif
    if (const char *r = sample(s, a.request, a.path, a.file, current)) { return r; }
    if (current.present != a.present || (current.present && id(current.script) != a.rid)) { return "cache_branch_or_identity_changed"; }
    if (current.present && truth(get(get(current.state, "resource"), "edited"))) { return "resource_edited"; }
    if (current.profile != a.profile || current.save != a.save_hash) { return "effective_or_compiled_profile_changed"; }
    if (current.source != (disk_after ? a.after : a.before)) { return "disk_source_changed"; }
    if (!disk_after && !same_revision(a.original, current.st)) { return "file_revision_changed"; }
    if (current.present && bytes(get(get(current.state, "resource"), "source")) != (resource_after ? a.after : a.before)) { return "resource_source_changed"; }
    if (disk_after && metadata && (!a.restored || !same_time(current.st.st_mtimespec, a.original.st_mtimespec))) { return "mtime_changed"; }
    if (disk_after && metadata && !same_revision(a.verified_metadata, current.st)) { return "file_revision_changed"; }
    if (const char *r = settings_guard(s, a)) { return r; }
    return owner_guard(s, a);
}
bool correlation(Session &s, const Value &c, std::string &request, uint64_t &expiry) {
    std::string session; uint64_t now = ticks(); return checked_bytes(get(c, "request_id"), request, 64) && valid_request(request) && checked_bytes(get(c, "session_id"), session, 32) && session == s.session_id && decimal(get(c, "expiry_tick_us"), expiry) && expiry > now && expiry - now <= 9000000;
}
}

bool closed_configure(Session &s) {
    epoch_advance(s); if (s.closed_epoch == UINT64_MAX) { s.closed_epoch_valid = false; return true; }
    Value editor, signal = name("script_close"), has;
    if (!checked_call(editor, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) || !id(editor) || !checked_call(has, object_ptr(editor), "Object", "has_signal", GAK_HASH_HAS_SIGNAL, {&signal}) || !truth(has)) { return false; }
    Storage<GAK_SIZE_CALLABLE> storage; GDExtensionCallableCustomInfo2 info{}; info.callable_userdata = &s; info.token = library; info.call_func = epoch_callback;
    api.new_callable(storage.ptr(), &info); Value callable = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr()); api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    Value flags = integer(0), result; int64_t code = -1;
    if (!checked_call(result, object_ptr(editor), "Object", "connect", GAK_HASH_CONNECT, {&signal, &callable, &flags}) || !number(result, code) || code != 0) { return false; }
    s.closed_signal_owner = id(editor); s.closed_signal_callable = new Value(callable); s.closed_epoch_valid = true; return true;
}
void closed_disconnect(Session &s) {
    s.closed_epoch_valid = false;
    if (!s.closed_signal_callable) { return; }
    if (void *owner = api.from_id(s.closed_signal_owner)) { Value signal = name("script_close"), result; checked_call(result, owner, "Object", "disconnect", GAK_HASH_DISCONNECT, {&signal, s.closed_signal_callable}); }
    delete s.closed_signal_callable; s.closed_signal_callable = nullptr; s.closed_signal_owner = 0;
}
void closed_cleanup(Session &s) { if (s.call_state != Session::CallState::Idle) { return; } if (auto *a = closed_attempt(s)) { s.owner = std::monostate{}; delete a; } }
void closed_closing(Session &s) { if (auto *a = closed_attempt(s)) { a->terminal = true; if (a->reason.empty()) { a->reason = "session_closed"; } } }
Value closed_inspect(Session &s, const Value &path_value, const Value &c) {
    std::string path, request; uint64_t expiry = 0;
    if (occupied(s) || s.call_state != Session::CallState::Idle) { return reply(nullptr, "unavailable", "slot_busy"); }
    if (!checked_bytes(path_value, path, 2048) || !path_ok(path) || !correlation(s, c, request, expiry)) { return reply(nullptr, "refused", "invalid_binding"); }
    Running running(s); FileBinding file; Sample observed;
#if GAK_FIXTURE
    std::string fault;
    if (pending_fault == "missing_cache_getters" || pending_fault == "missing_roster" ||
            pending_fault == "missing_context" || pending_fault == "epoch_reset" || pending_fault == "epoch_overflow") {
        fault = std::move(pending_fault); pending_fault.clear();
    }
    if (fault == "epoch_overflow") { s.closed_epoch = UINT64_MAX; s.closed_epoch_valid = false; }
    if (fault == "epoch_reset") { epoch_advance(s); }
    if (fault == "missing_context") { return reply(nullptr, "unavailable", "closed_context_unavailable"); }
    if (fault == "missing_cache_getters" || fault == "missing_roster") { return reply(nullptr, "unavailable", fault == "missing_roster" ? "roster_unavailable" : "cache_unavailable"); }
#endif
    if (const char *r = pin_file(s, path, file, O_RDONLY, true)) { return reply(nullptr, "refused", r); }
    if (const char *r = sample(s, request, path, file, observed)) { return reply(nullptr, "unavailable", r); }
    if (s.call_state == Session::CallState::Closing || ticks() >= expiry) { return reply(nullptr, "unavailable", "session_or_expiry_changed"); }
    return reply(nullptr, "inspected", "", &observed);
}
Value closed_prepare(Session &s, const Value &path_value, const Value &expected, const Value &desired, const Value &c) {
    std::string path, request, source; uint64_t expiry = 0;
    if (occupied(s) || s.call_state != Session::CallState::Idle) { return reply(nullptr, "refused", "slot_busy"); }
    if (!checked_bytes(path_value, path, 2048) || !path_ok(path) || !checked_bytes(desired, source, SOURCE_LIMIT) || !correlation(s, c, request, expiry)) { return reply(nullptr, "refused", "invalid_binding"); }
    auto *a = new ClosedAttempt; a->path = path; a->request = request; a->session = s.session_id; a->expires = expiry; a->after = std::move(source); s.owner = a;
#if GAK_FIXTURE
    a->fault = std::move(pending_fault); pending_fault.clear(); if (a->fault == "epoch_overflow") { s.closed_epoch = UINT64_MAX; s.closed_epoch_valid = false; }
    if (a->fault == "epoch_reset") { epoch_advance(s); }
#endif
    Running running(s); Sample observed;
    // Retain the exact confined settings file before preparing the validator context.
    if (const char *r = capture_settings(s, *a)) { return fail(*a, r); }
    // Validate target profile before requesting write access to the existing inode.
    FileBinding readonly; if (const char *r = pin_file(s, path, readonly, O_RDONLY, true)) { return fail(*a, r); }
    if (const char *r = sample(s, request, path, readonly, observed)) { return fail(*a, r); }
    if (observed.present && truth(get(get(observed.state, "resource"), "edited"))) { return fail(*a, "resource_edited"); }
    if (observed.present && bytes(get(get(observed.state, "resource"), "source")) != observed.source) { return fail(*a, "resource_disk_divergent"); }
    for (const char *key : {"project_device", "project_inode", "file_revision", "close_epoch"}) { if (!equal(get(expected, key), get(observed.state, key))) { return fail(*a, "expected_revision_changed"); } }
    if (!equal(get(expected, "resource"), expected_resource(observed))) { return fail(*a, "expected_resource_changed"); }
    if (const char *r = opening_effective(a->effective)) { return fail(*a, r); }
    std::vector<NativeBinding> bindings; if (const char *r = opening_source_profile(a->after, a->effective, bindings)) { return fail(*a, r); }
    std::string save; if (const char *r = save_profile(a->after, save)) { return fail(*a, r); }
    if (save != observed.save) { return fail(*a, "save_profile_changed"); }
    if (const char *r = pin_file(s, path, a->file, O_RDWR, true)) { return fail(*a, r); }
    if (a->file.device != readonly.device || a->file.inode != readonly.inode) { return fail(*a, "namespace_changed"); }
    a->before = observed.source; a->original = observed.st; a->epoch = s.closed_epoch; a->present = observed.present; a->script = observed.script; a->rid = id(a->script); a->profile = observed.profile; a->save_hash = observed.save; a->context_hash = bytes(get(observed.context, "sha256"));
    Sample fresh; if (const char *r = guard(s, *a, false, false, fresh)) { return fail(*a, r); }
    return reply(a, "prepared", "", &fresh);
}
Value closed_apply(Session &s, const Value &request, const Value &source_hash, const Value &context_hash) {
    auto *a = closed_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    Running running(s); if (a->phase != ClosedAttempt::Prepared || a->terminal) { return fail(*a, "terminal_or_duplicate_apply"); }
    if (bytes(source_hash, 64) != sha256(a->after) || bytes(context_hash, 64) != a->context_hash) { return fail(*a, "validation_binding_changed"); }
#if GAK_FIXTURE
    fixture(*a, "before_apply"); if (a->fault == "expire_before_apply") { a->expires = 0; }
#endif
    Sample current; if (const char *r = guard(s, *a, false, false, current)) { return fail(*a, r); }
    if (a->before == a->after) { return fail(*a, "unchanged_effect_entry_forbidden"); }
    if (a->present) {
        a->resource_entered = true; Value value = string(a->after), result;
        bool called = checked_call(result, object_ptr(a->script), "Script", "set_source_code", GAK_HASH_SCRIPT_SET_SOURCE, {&value});
        Value actual; std::string actual_source;
        if (checked_call(actual, object_ptr(a->script), "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) && checked_bytes(actual, actual_source, SOURCE_LIMIT)) { a->resource_changed = actual_source != a->before; }
        a->phase = ClosedAttempt::Resource;
        if (!called) { return fail(*a, "resource_setter_unavailable"); }
    }
#if GAK_FIXTURE
    fixture(*a, "after_resource"); if (a->fault == "expire_after_resource") { a->expires = 0; }
#endif
    if (const char *r = guard(s, *a, false, true, current)) { return fail(*a, r); }
#if GAK_FIXTURE
    fixture(*a, "before_write");
#endif
    if (const char *r = guard(s, *a, false, true, current)) { return fail(*a, r); }
    a->disk_entered = true;
#if GAK_FIXTURE
    if (a->fault == "lost_write") { return fail(*a, "write_failed"); }
#endif
    while (a->written < a->after.size()) {
        size_t count = a->after.size() - a->written;
#if GAK_FIXTURE
        if (a->fault == "partial_write") { if (a->written) { return fail(*a, "write_failed"); } count = std::max<size_t>(1, count / 2); }
#endif
        ssize_t n = pwrite(a->file.leaf.value, a->after.data() + a->written, count, a->written);
        if (n < 0 && errno == EINTR) { continue; } if (n <= 0) { return fail(*a, "write_failed"); } a->written += n;
    }
#if GAK_FIXTURE
        if (a->fault == "partial_write") { return fail(*a, "write_failed"); }
#endif
    if (ftruncate(a->file.leaf.value, a->after.size())) { return fail(*a, "truncate_failed"); } a->truncated = true;
    if (fsync(a->file.leaf.value)) { return fail(*a, "flush_failed"); } a->synced = true;
    if (!content(a->file.leaf.value, a->after) || !file_attached(s, a->file)) { return fail(*a, "persisted_readback_failed"); } a->readback = true; a->phase = ClosedAttempt::Content;
    struct stat persisted{};
    if (fstat(a->file.leaf.value, &persisted)) { return fail(*a, "metadata_unavailable"); }
#if GAK_FIXTURE
    fixture(*a, "after_write"); if (a->fault == "expire_after_write") { a->expires = 0; }
#endif
    // Do not alter metadata after newer work, lifecycle or cancellation became visible.
    if (const char *r = guard(s, *a, true, true, current, false)) { return fail(*a, r); }
    if (!same_revision(persisted, current.st)) { return fail(*a, "newer_disk_work"); }
    struct stat now{};
    timespec times[2] = {{0, UTIME_OMIT}, a->original.st_mtimespec}; a->restore_entered = true;
#if GAK_FIXTURE
    if (a->fault == "mtime_failure") { return fail(*a, "mtime_restore_failed"); }
#endif
    if (futimens(a->file.leaf.value, times)) { return fail(*a, "mtime_restore_failed"); }
    if (fstat(a->file.leaf.value, &now) || !same_time(now.st_mtimespec, a->original.st_mtimespec)) { return fail(*a, "mtime_readback_failed"); }
    a->verified_metadata = now; a->restored = true; a->phase = ClosedAttempt::Metadata;
#if GAK_FIXTURE
    fixture(*a, "after_mtime");
#endif
    if (const char *r = guard(s, *a, true, true, current)) { return fail(*a, r); }
    return reply(a, "applied", "", &current);
}
namespace {
Value verify(Session &s, const Value &request, const Value &purpose_value, bool recheck) {
    auto *a = closed_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    std::string purpose; if (!checked_bytes(purpose_value, purpose, 32) || (purpose != "preflight" && purpose != "unchanged" && purpose != "post_change")) { return fail(*a, "invalid_purpose"); }
    Running running(s);
#if GAK_FIXTURE
    fixture(*a, "before_verify");
#endif
    bool after = purpose == "post_change";
    if (a->terminal && after) {
        Sample survivor;
        if (s.session_id != a->session || ticks() >= a->expires) { return reply(a, "partial", a->reason.c_str()); }
        if (const char *r = settings_guard(s, *a)) { return reply(a, "partial", r); }
        if (const char *r = sample(s, a->request, a->path, a->file, survivor)) { return reply(a, "partial", r); }
        if (const char *r = settings_guard(s, *a)) { return reply(a, "partial", r); }
        return reply(a, "partial", a->reason.c_str(), &survivor);
    }
    if ((after && a->phase != ClosedAttempt::Metadata) || (!after && a->phase != ClosedAttempt::Prepared) || (purpose == "unchanged" && a->before != a->after)) { return fail(*a, "invalid_stage"); }
    Sample observed; if (const char *r = guard(s, *a, after, after, observed)) { return fail(*a, r); }
    // Original preflight must still bind the same exact validation context.
    if (!after && bytes(get(observed.context, "sha256")) != a->context_hash) { return fail(*a, "validation_context_changed"); }
    return reply(a, recheck ? "rechecked" : "verified", "", &observed);
}
}
Value closed_verify(Session &s, const Value &r, const Value &p) { return verify(s, r, p, false); }
Value closed_recheck(Session &s, const Value &r, const Value &p) { return verify(s, r, p, true); }
Value closed_finish(Session &s, const Value &request) {
    auto *a = closed_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    a->terminal = true; Value result = reply(a, "finished", a->reason.c_str()); closed_cleanup(s); return result;
}
Value closed_abort(Session &s, const Value &request) {
    auto *a = closed_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    Value result = fail(*a, "cancelled"); if (s.call_state == Session::CallState::Idle) { closed_cleanup(s); } return result;
}
Value closed_expire(Session &s) {
    auto *a = closed_attempt(s); if (!a || ticks() < a->expires) { return Value(); }
    Value result = fail(*a, "expired"); if (s.call_state == Session::CallState::Idle) { closed_cleanup(s); } return result;
}
#if GAK_FIXTURE
Value closed_fixture_fault(Session &s, const Value &request, const Value &fault_value) {
    std::string r, fault; if (!checked_bytes(request, r, 64) || !checked_bytes(fault_value, fault, 64)) { return reply(nullptr, "refused", "invalid_fault"); }
    const char *allowed[] = {"missing_cache_getters", "missing_roster", "missing_context", "epoch_reset", "epoch_overflow", "partial_write", "lost_write", "mtime_failure", "expire_before_apply", "expire_after_resource", "expire_after_write"};
    if (std::find_if(std::begin(allowed), std::end(allowed), [&](const char *f) { return fault == f; }) == std::end(allowed)) { return reply(nullptr, "refused", "invalid_fault"); }
    if (r.empty() && !occupied(s)) { pending_fault = fault; return reply(nullptr, "ready", ""); }
    auto *a = closed_attempt(s);
    if (!owned(a, request) || a->terminal || a->phase != ClosedAttempt::Prepared) { return reply(nullptr, "refused", "wrong_attempt"); }
    a->fault = fault;
    if (fault == "epoch_reset") { epoch_advance(s); }
    if (fault == "epoch_overflow") { s.closed_epoch = UINT64_MAX; s.closed_epoch_valid = false; }
    return reply(a, "ready", "");
}
Value closed_fixture_state(Session &s, const Value &request) { auto *a = closed_attempt(s); return owned(a, request) ? reply(a, "state", "") : reply(nullptr, "refused", "wrong_attempt"); }
#endif
} // namespace gak
