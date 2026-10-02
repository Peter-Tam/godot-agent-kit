#include "open_context.hpp"

#include <fcntl.h>
#include <algorithm>
#include <chrono>
#include <memory>
#include <optional>

namespace gak {
struct OpenAttempt {
    enum Stage { Inspected, Prepared, Bound, Compiled, Opened } stage = Inspected;
    std::string request, session, path, source, source_hash, cache_hash, invalidation;
#if GAK_FIXTURE
    std::string fault;
#endif
    uint64_t expires{}, cache_length{};
    bool cached{}, cache_observed{}, document_observed{}, target_open{}, terminal{}, open_entered{};
    Value script;
    Binding target;
    FileBinding file;
    OpeningEffective effective;
    OpeningContext context;
    std::vector<NativeBinding> source_bindings;
    std::string cache_binding = "not_started", compilation = "not_started", document_open = "not_started";
    std::optional<int64_t> parse_code;
    std::optional<bool> edited;
    std::string selection = "unknown";
};
namespace {
void *singleton(const char *name) { Name key(name); return api.singleton(key.ptr()); }
const char *next(const OpenAttempt &a) {
    if (a.terminal || a.target_open || a.stage == OpenAttempt::Opened) { return ""; }
    switch (a.stage) {
        case OpenAttempt::Inspected: return "prepare";
        case OpenAttempt::Prepared: return "bind";
        case OpenAttempt::Bound: return a.cached ? "open" : "compile";
        case OpenAttempt::Compiled: return "open";
        case OpenAttempt::Opened: return "";
    }
    return "";
}
const char *reached(const OpenAttempt &a) {
    switch (a.stage) {
        case OpenAttempt::Inspected: return "inspected";
        case OpenAttempt::Prepared: return "prepared";
        case OpenAttempt::Bound: return "bind";
        case OpenAttempt::Compiled: return "compile";
        case OpenAttempt::Opened: return "open";
    }
    return "unknown";
}
bool effected(const OpenAttempt &a) {
    return a.cache_binding == "new_resource_published" || a.cache_binding == "unknown" || a.open_entered;
}
const char *failure_status(const OpenAttempt &a) { return effected(a) ? "partial" : "refused"; }
void identity(Value &out, const Binding &binding) {
    if (binding.script_id) { put_string(out, "script_instance_id", std::to_string(binding.script_id)); }
    if (binding.editor_id) { put_string(out, "editor_instance_id", std::to_string(binding.editor_id)); }
    if (binding.buffer_id) { put_string(out, "buffer_instance_id", std::to_string(binding.buffer_id)); }
}
Value reply(const OpenAttempt *a, const char *status, const char *reason, const Session *session = nullptr) {
    Value out = dict(); put_string(out, "status", status); put_string(out, "reason", reason);
    if (!a) { return out; }
    put_string(out, "request_id", a->request); put_string(out, "stage", reached(*a)); put_string(out, "next_stage", next(*a));
    put_string(out, "cache_binding", a->cache_binding); put_string(out, "initial_compilation", a->compilation);
    put_string(out, "document_open", a->document_open); put(out, "target_parse_code", a->parse_code ? integer(*a->parse_code) : Value());
    put(out, "resource_edited", a->edited ? boolean(*a->edited) : Value()); put_string(out, "selection", a->selection);
    put(out, "target_open", a->document_observed ? boolean(a->target_open || a->stage == OpenAttempt::Opened) : Value());
    put(out, "terminal_discard", boolean(a->terminal));
    put(out, "entered", boolean(session && session->call_state != Session::CallState::Idle));
    put_string(out, "expiry_tick_us", std::to_string(a->expires));
    put(out, "mode", a->stage >= OpenAttempt::Prepared ? string(a->cached ? "cached" : "cold") : Value());
    if ((a->cached || a->cache_binding == "new_resource_published") && id(a->script)) {
        put_string(out, "script_instance_id", std::to_string(id(a->script)));
    }
    identity(out, a->target);
    if (a->stage == OpenAttempt::Inspected) {
        Value cache_state = dict();
        put_string(cache_state, "state", !a->cache_observed ? "unavailable" : a->cached ? "present" : "absent");
        put_string(cache_state, "script_instance_id", a->cache_observed && a->cached ? std::to_string(id(a->script)) : "");
        put(cache_state, "sha256", !a->cache_observed || (a->cached && a->cache_hash.empty()) ? Value() : string(a->cache_hash));
        put(cache_state, "utf8_bytes", !a->cache_observed || (a->cached && a->cache_hash.empty()) ? Value() :
                string(a->cached ? std::to_string(a->cache_length) : ""));
        if (!a->cache_observed) { put_string(cache_state, "reason", reason); }
        put(out, "cache", cache_state);
    }
    return out;
}
Value fail(OpenAttempt &a, const char *reason, const Session *session = nullptr) {
    a.terminal = true;
    if (a.invalidation.empty()) { a.invalidation = reason; }
    return reply(&a, failure_status(a), a.invalidation.c_str(), session);
}
struct EnteredCall {
    Session &session;
    explicit EnteredCall(Session &s) : session(s) { session.call_state = Session::CallState::Running; }
    ~EnteredCall() {
        const bool closing = session.call_state == Session::CallState::Closing;
        session.call_state = Session::CallState::Idle;
        if (closing) { close(session); }
    }
};
bool owned(const OpenAttempt *a, const Value &request) {
    std::string value;
    return a && checked_bytes(request, value, 64) && value == a->request;
}
const char *owner_guard(const Session &s, const OpenAttempt &a) {
    if (a.terminal) { return a.invalidation.empty() ? "terminal_discard" : a.invalidation.c_str(); }
    if (s.call_state == Session::CallState::Closing || s.project_fd < 0 || s.session_id != a.session ||
            ticks() >= a.expires) { return "session_or_expiry_changed"; }
    return nullptr;
}
const char *cache(const std::string &path, Value &script, bool &present) {
    void *loader = singleton("ResourceLoader");
    Value location = string(path), has, found;
    if (!checked_call(has, loader, "ResourceLoader", "has_cached", GAK_HASH_HAS_CACHED, {&location}) ||
            has.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            !checked_call(found, loader, "ResourceLoader", "get_cached_ref", GAK_HASH_CACHED_REF, {&location})) {
        return "cache_unavailable";
    }
    present = truth(has);
    if (!present) {
        return (found.type() == GDEXTENSION_VARIANT_TYPE_NIL ||
                (found.type() == GDEXTENSION_VARIANT_TYPE_OBJECT && !object_ptr(found))) ? nullptr : "cache_state_changed";
    }
    Name gd("GDScript");
    if (!object_ptr(found)) { return "cache_state_changed"; }
    if (!api.class_tag(gd.ptr()) || !api.cast_to(object_ptr(found), api.class_tag(gd.ptr()))) { return "wrong_type_cache"; }
    Value actual_path;
    std::string actual;
    if (!checked_call(actual_path, object_ptr(found), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
            !checked_bytes(actual_path, actual, 2048) || actual != path) { return "cache_path_mismatch"; }
    script = found; return nullptr;
}
const char *resource(OpenAttempt &a) {
    Value source, edited;
    std::string actual;
    if (!object_ptr(a.script) ||
            !checked_call(source, object_ptr(a.script), "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) ||
            !checked_bytes(source, actual, SOURCE_LIMIT)) { return "resource_source_unavailable"; }
    if (actual != a.source) { return "resource_source_changed"; }
    if (!checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&a.script}) ||
            edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { a.edited.reset(); return "resource_edited_unavailable"; }
    a.edited = truth(edited);
    return *a.edited ? "resource_edited" : nullptr;
}
void selection(OpenAttempt &a) {
    Value editor, script;
    if (!checked_call(editor, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
            !checked_call(script, object_ptr(editor), "ScriptEditor", "get_current_script", GAK_HASH_CURRENT_SCRIPT)) {
        a.selection = "unknown"; return;
    }
    if (script.type() != GDEXTENSION_VARIANT_TYPE_NIL && script.type() != GDEXTENSION_VARIANT_TYPE_OBJECT) {
        a.selection = "unknown"; return;
    }
    a.selection = !object_ptr(script) ? "no_source" : id(script) == id(a.script) ? "requested_target" : "other";
}
const char *target_guard(OpenAttempt &a, bool after_open) {
    Binding document; bool found = false;
    if (!find_document(a.path, document, found)) { return "document_association_unavailable"; }
    if (after_open || a.target_open) {
        if (!found || document.script_id != id(a.script) ||
                (a.target.editor_id && (document.editor_id != a.target.editor_id || document.buffer_id != a.target.buffer_id))) {
            return "document_association_changed";
        }
    } else if (found) { return "target_opened_during_attempt"; }
    Value actual; bool present = false;
    if (const char *reason = cache(a.path, actual, present)) { return reason; }
    const bool needs_cache = a.cached || a.stage >= OpenAttempt::Bound;
    if (present != needs_cache || (present && id(actual) != id(a.script))) { return "cache_identity_changed"; }
    if (present) { if (const char *reason = resource(a)) { return reason; } }
    if (found) { a.target = document; }
    return nullptr;
}
const char *guard(Session &s, OpenAttempt &a, bool after_open = false) {
    if (const char *reason = owner_guard(s, a)) { return reason; }
    if (!file_attached(s, a.file)) { return "namespace_or_descriptor_changed"; }
    if (!content(a.file.leaf.value, a.source)) { return "disk_content_changed"; }
    if (const char *reason = opening_path_guard(s, a.file)) { return reason; }
    if (const char *reason = target_guard(a, after_open)) { return reason; }
    OpeningEffective effective;
    if (const char *reason = opening_effective(effective)) { return reason; }
    if (effective.fingerprint != a.effective.fingerprint) { return "effective_context_changed"; }
    std::vector<NativeBinding> bindings;
    if (const char *reason = opening_source_profile(a.source, effective, bindings)) { return reason; }
    if (!opening_bindings_equal(bindings, a.source_bindings)) { return "target_class_bindings_changed"; }
    OpeningContext context;
    if (const char *reason = opening_context(s, a.request, effective, context, &a.context, after_open)) { return reason; }
    if (context.kind != a.context.kind || context.guard_hash != a.context.guard_hash) {
        return "current_context_changed";
    }
    return owner_guard(s, a);
}
#if GAK_FIXTURE
void fixture_callback(OpenAttempt &a, const std::string &stage) {
    Value key = string("godot_agent_kit_open_fixture_callback"), callback;
    if (checked_call(callback, singleton("Engine"), "Object", "get_meta", GAK_HASH_GET_META, {&key}) &&
            callback.type() == GDEXTENSION_VARIANT_TYPE_CALLABLE) {
        Value request = string(a.request), reached_stage = string(stage), result;
        checked_invoke(result, callback, "call", {&request, &reached_stage});
    }
}
void fixture_entered(OpenAttempt &a, const std::string &stage) {
    if (a.fault == "callback_" + stage || a.fault == "stall_" + stage) { fixture_callback(a, stage); }
    if (a.fault == "stall_" + stage) { std::this_thread::sleep_for(std::chrono::seconds(11)); }
}
#endif
} // namespace

void open_cleanup(Session &session) {
    if (auto *attempt = open_attempt(session)) { session.owner = std::monostate{}; delete attempt; }
}
void open_closing(Session &session) {
    if (auto *attempt = open_attempt(session)) {
        attempt->terminal = true;
        if (attempt->invalidation.empty()) { attempt->invalidation = "session_closed"; }
    }
}
Value open_inspect(Session &s, const Value &path_value, const Value &correlation) {
    if (occupied(s) || s.call_state != Session::CallState::Idle) { return reply(nullptr, "refused", "slot_busy"); }
    std::string path, request, sid;
    uint64_t expires = 0;
    const uint64_t now = ticks();
    if (s.project_fd < 0 || std::this_thread::get_id() != s.main_thread ||
            !checked_bytes(path_value, path, 2048) || !path_ok(path) ||
            !checked_bytes(get(correlation, "request_id"), request, 64) || !valid_request(request) ||
            !checked_bytes(get(correlation, "session_id"), sid, 32) || sid != s.session_id ||
            !decimal(get(correlation, "expiry_tick_us"), expires) || now == UINT64_MAX ||
            expires <= now || expires - now > 9000000) { return reply(nullptr, "refused", "invalid_binding"); }
    auto *a = new OpenAttempt;
    a->request = std::move(request); a->session = s.session_id; a->path = std::move(path); a->expires = expires;
    s.owner = a; // Reserve before the first editor/cache inspection, including callbacks.
    EnteredCall entered(s);
    if (!attached_project(s)) { return fail(*a, "namespace_or_descriptor_changed", &s); }
    if (!find_document(a->path, a->target, a->target_open)) { return fail(*a, "document_association_unavailable", &s); }
    a->document_observed = true;
    const char *cache_reason = cache(a->path, a->script, a->cached);
    if (cache_reason) {
        Value result = fail(*a, cache_reason, &s), state = dict();
        put_string(state, "state", "unavailable"); put_string(state, "reason", cache_reason); put(result, "cache", state);
        return result;
    }
    a->cache_observed = true;
    if (a->target_open && (!a->cached || id(a->script) != id(a->target.script))) {
        return fail(*a, "cache_document_identity_mismatch", &s);
    }
    Value state = dict(); put_string(state, "state", a->cached ? "present" : "absent");
    put_string(state, "script_instance_id", a->cached ? std::to_string(id(a->script)) : "");
    if (a->cached) {
        Value source, edited; std::string actual;
        if (checked_call(source, object_ptr(a->script), "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) &&
                checked_bytes(source, actual, SOURCE_LIMIT)) {
            a->cache_hash = sha256(actual); a->cache_length = actual.size();
            put_string(state, "sha256", a->cache_hash); put_string(state, "utf8_bytes", std::to_string(actual.size()));
        } else { put(state, "sha256", Value()); put(state, "utf8_bytes", Value()); put_string(state, "reason", "resource_source_unavailable"); }
        if (checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&a->script}) &&
                edited.type() == GDEXTENSION_VARIANT_TYPE_BOOL) { a->edited = truth(edited); }
    } else { put_string(state, "sha256", ""); put_string(state, "utf8_bytes", ""); }
    selection(*a);
    if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason, &s); }
    Value result = reply(a, "inspected", "", &s); put(result, "cache", state);
    return result;
}

Value open_prepare(Session &s, const Value &request, const Value &source_value, const Value &capture) {
    OpenAttempt *a = open_attempt(s);
    if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy", &s); }
    auto refuse = [&](const char *reason) {
        Value result = fail(*a, reason, &s), context = opening_context_reply(a->context);
        put_string(context, "reason", reason);
        put(result, "context", context); return result;
    };
    if (a->terminal || a->target_open || a->stage != OpenAttempt::Inspected) { return refuse("wrong_attempt_or_stage"); }
    EnteredCall entered(s);
    if (const char *reason = owner_guard(s, *a)) { return refuse(reason); }
    uint64_t pd = 0, pi = 0, td = 0, ti = 0, length = 0;
    std::string hash, mode, cached_id, source;
    if (!decimal(get(capture, "project_device"), pd) || !decimal(get(capture, "project_inode"), pi) ||
            !decimal(get(capture, "target_device"), td) || !decimal(get(capture, "target_inode"), ti) ||
            !decimal(get(capture, "utf8_bytes"), length) || !checked_bytes(get(capture, "sha256"), hash, 64) ||
            !checked_bytes(get(capture, "cache_mode"), mode, 16) || !checked_bytes(get(capture, "cached_script_id"), cached_id, 20) ||
            !checked_bytes(source_value, source, SOURCE_LIMIT) || pd != static_cast<uint64_t>(s.device) || pi != static_cast<uint64_t>(s.inode) ||
            mode != (a->cached ? "present" : "absent") || cached_id != (a->cached ? std::to_string(id(a->script)) : "") ||
            hash != sha256(source) || length != source.size()) { return refuse("invalid_capture_binding"); }
    a->source = std::move(source); a->source_hash = std::move(hash);
    if (a->cached) {
        if (a->cache_hash.empty()) { return refuse("cached_source_unavailable"); }
        if (a->cache_hash != a->source_hash || a->cache_length != length) { return refuse("cached_source_conflict"); }
        if (!a->edited) { return refuse("resource_edited_unavailable"); }
        if (*a->edited) { return refuse("resource_edited"); }
    }
    if (const char *reason = pin_file(s, a->path, a->file, O_RDONLY, true)) { return refuse(reason); }
    if (td != static_cast<uint64_t>(a->file.device) || ti != static_cast<uint64_t>(a->file.inode)) {
        return refuse("namespace_or_descriptor_changed");
    }
    if (!content(a->file.leaf.value, a->source)) { return refuse("disk_content_changed"); }
    if (const char *reason = opening_path_guard(s, a->file)) { return refuse(reason); }
    if (const char *reason = target_guard(*a, false)) { return refuse(reason); }
    if (const char *reason = opening_effective(a->effective)) { return refuse(reason); }
    if (const char *reason = opening_source_profile(a->source, a->effective, a->source_bindings)) { return refuse(reason); }
    if (const char *reason = opening_context(s, a->request, a->effective, a->context)) { return refuse(reason); }
    size_t admission_bytes = a->context.kind == OpeningContext::Current ? a->context.record_bytes : a->effective.record_bytes;
    for (const auto &binding : a->source_bindings) { admission_bytes += binding.name.size() + 32; }
    if (admission_bytes > 256 * 1024) { return refuse("opening_context_limit"); }
    a->stage = OpenAttempt::Prepared;
    if (a->cached) { a->compilation = "not_applicable"; }
    if (const char *reason = guard(s, *a)) { return refuse(reason); }
    Value result = reply(a, "prepared", "", &s); put(result, "context", opening_context_reply(a->context));
    put_string(result, "project_device", std::to_string(s.device)); put_string(result, "project_inode", std::to_string(s.inode));
    put_string(result, "target_device", std::to_string(a->file.device)); put_string(result, "target_inode", std::to_string(a->file.inode));
    return result;
}

Value open_advance(Session &s, const Value &request, const Value &stage_value, const Value &source_hash, const Value &context_hash) {
    OpenAttempt *a = open_attempt(s);
    if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy", &s); }
    std::string stage, validated_source, validated_context;
    if (a->terminal || !checked_bytes(stage_value, stage, 16) || stage != next(*a) ||
            (stage != "bind" && stage != "compile" && stage != "open") ||
            !checked_bytes(source_hash, validated_source, 64) || !checked_bytes(context_hash, validated_context, 64) ||
            validated_source != a->context.source_hash || validated_context != a->context.guard_hash) {
        return fail(*a, "wrong_stage_or_context_validation_binding");
    }
    EnteredCall entered(s);
#if GAK_FIXTURE
    if (a->fault == "expire_" + stage) { a->expires = ticks(); }
    fixture_entered(*a, stage);
#endif
    if (const char *reason = guard(s, *a)) { return fail(*a, reason, &s); }
    if (stage == "bind") {
        if (a->cached) { a->cache_binding = "reused_existing"; a->stage = OpenAttempt::Bound; }
        else {
            Value cls = string("GDScript"), result, source = string(a->source);
            if (!checked_call(a->script, singleton("ClassDB"), "ClassDB", "instantiate", GAK_HASH_INSTANTIATE, {&cls}) ||
                    !object_ptr(a->script)) { a->cache_binding = "failed_before_publication"; return fail(*a, "construction_failed", &s); }
            if (const char *reason = guard(s, *a)) { return fail(*a, reason, &s); }
#if GAK_FIXTURE
            if (a->fault == "property_source") {
                Value key = string("source_code");
                if (!checked_call(result, object_ptr(a->script), "Object", "set", GAK_HASH_OBJECT_SET, {&key, &source})) {
                    return fail(*a, "source_initialization_failed", &s);
                }
            } else
#endif
            if (!checked_call(result, object_ptr(a->script), "Script", "set_source_code", GAK_HASH_SCRIPT_SET_SOURCE, {&source})) {
                a->cache_binding = "failed_before_publication"; return fail(*a, "source_initialization_failed", &s);
            }
            // The unbound object is never target evidence. Recheck the original
            // cold cache/document/current/file guards before non-takeover publication.
            if (const char *reason = guard(s, *a)) { return fail(*a, reason, &s); }
            Value path = string(a->path);
            a->cache_binding = "unknown"; // Publication may occur once this void setter is entered.
            const bool returned = checked_call(result, object_ptr(a->script), "Resource", "set_path", GAK_HASH_RESOURCE_SET_PATH, {&path});
            Value found; bool present = false;
            const char *cache_reason = cache(a->path, found, present);
            if (!cache_reason && present && id(found) == id(a->script)) { a->cache_binding = "new_resource_published"; a->stage = OpenAttempt::Bound; }
            if (!returned || cache_reason || a->cache_binding != "new_resource_published") {
                return fail(*a, cache_reason ? cache_reason : "path_binding_failed", &s);
            }
            if (const char *reason = resource(*a)) { return fail(*a, reason, &s); }
#if GAK_FIXTURE
            if (a->fault == "fail_after_bind") { return fail(*a, "fixture_after_bind", &s); }
#endif
        }
    } else if (stage == "compile") {
        if (a->cached || a->stage != OpenAttempt::Bound) { return fail(*a, "existing_resource_compilation_forbidden", &s); }
        Value no = boolean(false), result;
        a->compilation = "unknown";
        if (!checked_call(result, object_ptr(a->script), "Script", "reload", GAK_HASH_SCRIPT_RELOAD, {&no})) {
            a->compilation = "unavailable_failed"; return fail(*a, "initial_compilation_unavailable", &s);
        }
        int64_t code = -1;
        if (!number(result, code) || code < 0) { return fail(*a, "initial_compilation_result_unavailable", &s); }
        a->parse_code = code;
        if (code != 0 && code != GAK_ERR_PARSE_ERROR) {
            a->compilation = "unavailable_failed"; return fail(*a, "initial_compilation_failed", &s);
        }
        a->compilation = code == 0 ? "completed_valid" : "completed_invalid"; a->stage = OpenAttempt::Compiled;
    } else {
        Value line = integer(-1), column = integer(0), focus = boolean(true), result;
        a->open_entered = true; a->document_open = "entered";
        const bool returned = checked_call(result, singleton("EditorInterface"), "EditorInterface", "edit_script", GAK_HASH_EDIT_SCRIPT,
                {&a->script, &line, &column, &focus});
        Binding actual; bool found = false;
        const bool known = find_document(a->path, actual, found);
        selection(*a);
        if (known && found && id(actual.script) == id(a->script)) {
            a->target = actual; a->document_open = "association_obtained"; a->stage = OpenAttempt::Opened;
        } else { a->document_open = known ? "failed_unverified" : "unknown"; }
        if (!returned || a->stage != OpenAttempt::Opened) { return fail(*a, "document_open_unverified", &s); }
    }
    if (const char *reason = guard(s, *a, a->stage == OpenAttempt::Opened)) { return fail(*a, reason, &s); }
    selection(*a);
    return reply(a, "ready", "", &s);
}

namespace {
Value observe(Session &s, const Value &request, const Value &purpose_value, bool recheck) {
    OpenAttempt *a = open_attempt(s);
    if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy", &s); }
    std::string purpose;
    if (!checked_bytes(purpose_value, purpose, 16) ||
            !((purpose == "recognition" && a->target_open && a->stage == OpenAttempt::Inspected) ||
            (purpose == "post_open" && !a->target_open && a->stage == OpenAttempt::Opened))) {
        return fail(*a, "wrong_verification_purpose");
    }
    EnteredCall entered(s);
    if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason, &s); }
    if (purpose == "recognition") {
        Binding actual;
        if (!attached_project(s) || !resolve(a->path, id(a->script), a->target.editor_id, a->target.buffer_id, actual)) {
            return fail(*a, "document_association_changed", &s);
        }
        a->target = actual;
        Value edited;
        if (!checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&a->script}) ||
                edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { a->edited.reset(); }
        else { a->edited = truth(edited); }
    } else if (const char *reason = guard(s, *a, true)) { return fail(*a, reason, &s); }
    selection(*a);
    if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason, &s); }
    Value result = reply(a, recheck ? "unchanged" : "observed", "", &s);
    put_string(result, "protection", purpose == "recognition" ? "not_applicable" : "unchanged");
    // No R/B bytes are copied from preparation into verification. The separate
    // ordinary collector must independently establish target source and dirty state.
    return result;
}
Value terminal(Session &s, const Value &request, bool abort) {
    OpenAttempt *a = open_attempt(s);
    if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    a->terminal = true;
    if (a->invalidation.empty()) { a->invalidation = abort ? "cancelled" : "finished"; }
    Value result = reply(a, "discarded", a->invalidation.c_str(), &s);
    if (s.call_state == Session::CallState::Idle) { open_cleanup(s); }
    return result;
}
} // namespace
Value open_verify(Session &s, const Value &request, const Value &purpose) { return observe(s, request, purpose, false); }
Value open_recheck(Session &s, const Value &request, const Value &purpose) { return observe(s, request, purpose, true); }
Value open_finish(Session &s, const Value &request) { return terminal(s, request, false); }
Value open_abort(Session &s, const Value &request) { return terminal(s, request, true); }
bool open_expire(Session &s) {
    OpenAttempt *a = open_attempt(s);
    if (!a || ticks() < a->expires) { return false; }
    a->terminal = true;
    if (a->invalidation.empty()) { a->invalidation = "session_or_expiry_changed"; }
    if (s.call_state == Session::CallState::Idle) { open_cleanup(s); }
    return true;
}
#if GAK_FIXTURE
Value open_fixture_fault(Session &s, const Value &request, const Value &fault_value) {
    OpenAttempt *a = open_attempt(s);
    std::string fault;
    if (!owned(a, request) || a->terminal || s.call_state != Session::CallState::Idle ||
            !checked_bytes(fault_value, fault, 32)) { return reply(nullptr, "refused", "wrong_attempt"); }
    static const char *allowed[] = {"property_source", "fail_after_bind", "expire_bind", "expire_compile", "expire_open",
        "stall_bind", "stall_compile", "stall_open", "callback_bind", "callback_compile", "callback_open"};
    if (std::find_if(std::begin(allowed), std::end(allowed), [&](const char *value) { return fault == value; }) == std::end(allowed)) {
        return reply(a, "refused", "invalid_fault");
    }
    a->fault = std::move(fault); return reply(a, "ready", "");
}
Value open_fixture_state(Session &s, const Value &request) {
    OpenAttempt *a = open_attempt(s);
    if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    Value result = reply(a, "observed", "", &s);
    put_string(result, "shared_owner", "open"); put_string(result, "target_script_id", std::to_string(id(a->script)));
    put_string(result, "current_script_id", std::to_string(a->context.document.script_id));
    put_string(result, "current_editor_id", std::to_string(a->context.document.editor_id));
    put_string(result, "current_buffer_id", std::to_string(a->context.document.buffer_id));
    return result;
}
#endif
} // namespace gak
