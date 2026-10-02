#include "editor_context.hpp"

#include <algorithm>
#include <array>
#include <charconv>
#include <cmath>
#include <CommonCrypto/CommonDigest.h>
#include <fcntl.h>
#include <memory>
#include <optional>
#include <set>
#include <cerrno>
#include <unistd.h>

namespace gak {
struct CloseAttempt {
    enum class Phase { Inspected, Prepared, Entered, Returned, Settling, Settled, Terminal } phase = Phase::Inspected;
    struct Connection { uint64_t object{}; const char *signal{}; Value callable; };
    std::string request, session, path, source, source_hash, guard_hash, invalidation, validator_context;
    uint64_t generation{}, expires{}, entry{}, returned{}, idle_delay{}, error_delay{};
    std::map<uint64_t, std::pair<uint64_t, uint64_t>> collections;
    int64_t sort_order{};
    uint64_t rid{}, eid{}, bid{}, selected_script{}, selected_editor{}, selected_buffer{}, after_script{};
    bool open{}, entered{}, discarded{}, terminal{}, selected_known{}, after_known{};
    std::optional<uint64_t> recognition_cache_id;
    std::optional<int64_t> error;
    std::optional<bool> removed, edited;
    OpeningEffective effective;
    OpeningContext target;
    FileBinding recognition_file, settings_file;
    std::string settings_source;
    std::vector<std::unique_ptr<OpeningContext>> protected_documents;
    std::vector<std::array<uint64_t, 3>> identities;
    std::vector<std::string> paths;
    std::set<uint64_t> required, completed;
    std::vector<Connection> connections;
    Value aggregate, target_guard;
    std::string resource_state = "not_collected";
#if GAK_FIXTURE
    std::string fault;
#endif
};
namespace {
void *singleton(const char *value) { Name key(value); return api.singleton(key.ptr()); }
bool size(Value &v, int64_t &n, int64_t limit) {
    Value result;
    return v.type() == GDEXTENSION_VARIANT_TYPE_ARRAY && checked_invoke(result, v, "size") &&
            number(result, n) && n >= 0 && n <= limit;
}
Value at(Value &v, int64_t i) {
    auto *p = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](v.ptr()), i);
    return p ? Value(p) : Value();
}
void append(Value &v, const Value &item) { Value ignored; checked_invoke(ignored, v, "append", {&item}); }
bool exact(const Value &value, std::initializer_list<const char *> fields) {
    Value copy = value, keys; int64_t n = 0;
    if (copy.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY || !checked_invoke(keys, copy, "keys") ||
            !size(keys, n, fields.size()) || static_cast<size_t>(n) != fields.size()) { return false; }
    for (int64_t i = 0; i < n; ++i) {
        std::string key;
        if (!checked_bytes(at(keys, i), key, 64) || std::find_if(fields.begin(), fields.end(),
                [&](const char *field) { return key == field; }) == fields.end()) { return false; }
    }
    return true;
}
Value stamp(const std::string &session, uint64_t start, uint64_t end) {
    if (!start) { return Value(); }
    Value out = dict(); put_string(out, "clock_id", "editor:" + session);
    put_string(out, "started_tick_us", std::to_string(start));
    put_string(out, "finished_tick_us", std::to_string(end)); put(out, "received_elapsed_us", integer(0)); return out;
}
const char *phase(CloseAttempt::Phase value) {
    switch (value) {
        case CloseAttempt::Phase::Inspected: return "inspected";
        case CloseAttempt::Phase::Prepared: return "prepared";
        case CloseAttempt::Phase::Entered: return "entered";
        case CloseAttempt::Phase::Returned: return "returned";
        case CloseAttempt::Phase::Settling: return "settling";
        case CloseAttempt::Phase::Settled: return "settled";
        case CloseAttempt::Phase::Terminal: return "terminal";
    }
    return "terminal";
}
const char *continuation(const CloseAttempt &a) {
    if (!a.invalidation.empty()) { return "invalidated"; }
    if (!a.entered || a.required.empty()) { return "not_applicable"; }
    return a.completed == a.required && a.returned ? "completed" : "pending";
}
const char *relation(uint64_t value, uint64_t target, bool known) {
    return !known ? "unknown" : !value ? "no_source_editor" : value == target ? "target" : "other";
}
Value selection(const CloseAttempt &a) {
    Value out = dict();
    put_string(out, "before", relation(a.selected_script, a.rid, a.selected_known));
    put_string(out, "after", relation(a.after_script, a.rid, a.after_known));
    put_string(out, "request_effect", !a.entered ? "none" : !a.after_known ? "unknown" :
            a.selected_script == a.rid && a.after_script != a.rid ? "native_fallback" : "none");
    return out;
}
Value protection(const CloseAttempt &a) {
    Value out = dict();
    const bool applicable = !a.guard_hash.empty();
    put_string(out, "status", !applicable ? "not_applicable" :
            !a.invalidation.empty() ? "invalidated" : "preserved");
    put_string(out, "revalidation", applicable ? continuation(a) : "not_applicable");
    put(out, "required_count", integer(a.required.size())); put(out, "completed_count", integer(a.completed.size()));
    put(out, "reason", a.invalidation.empty() ? Value() : string(a.invalidation)); return out;
}
Value ledger(const CloseAttempt &a) {
    Value out = dict(), required = collection(GDEXTENSION_VARIANT_TYPE_ARRAY), completed = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (auto value : a.required) { append(required, string(std::to_string(value))); }
    for (auto value : a.completed) { append(completed, string(std::to_string(value))); }
    put(out, "required_editor_ids", required); put(out, "completed_editor_ids", completed);
    Value collections = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (const auto &entry : a.collections) {
        Value record = dict(); put_string(record, "editor_id", std::to_string(entry.first));
        put(record, "visit", stamp(a.session, entry.second.first, entry.second.first));
        put(record, "completion", stamp(a.session, entry.second.second, entry.second.second)); append(collections, record);
    }
    put(out, "collections", collections); put_string(out, "state", continuation(a));
    put(out, "reason", a.invalidation.empty() ? Value() : string(a.invalidation)); return out;
}
Value facts(const CloseAttempt &a) {
    Value out = dict();
    put_string(out, "request_id", a.request); put_string(out, "session_id", a.session); put_string(out, "script_path", a.path);
    put_string(out, "native_build_id", GAK_BUILD_ID); put(out, "native_api_revision", integer(3));
    put_string(out, "phase", phase(a.phase));
    put(out, "script_instance_id", a.rid ? string(std::to_string(a.rid)) : Value());
    put(out, "editor_instance_id", a.eid ? string(std::to_string(a.eid)) : Value());
    put(out, "buffer_instance_id", a.bid ? string(std::to_string(a.bid)) : Value());
    put(out, "entry_collection", stamp(a.session, a.entry, a.entry)); put(out, "return_collection", stamp(a.session, a.returned, a.returned));
    put(out, "entered", boolean(a.entered)); put(out, "close_error", a.error ? integer(*a.error) : Value());
    put(out, "old_document_removed", a.removed ? boolean(*a.removed) : Value()); put(out, "selection", selection(a));
    put_string(out, "target_buffer", !a.open ? "not_applicable" : !a.entered ? "retained" :
            a.removed && *a.removed && !api.from_id(a.bid) && !api.from_id(a.eid) ? "disposed" : "unavailable");
    put(out, "protection", protection(a)); put(out, "continuation", ledger(a));
    put(out, "invalidated", boolean(!a.invalidation.empty()));
    put(out, "reason", a.invalidation.empty() ? Value() : string(a.invalidation));
    put(out, "terminal_discard", boolean(a.discarded)); return out;
}
Value reply(const CloseAttempt *a, const char *status, const char *reason) {
    Value out = dict(); put_string(out, "status", status); put_string(out, "reason", reason);
    put(out, "native", a ? facts(*a) : Value());
    put(out, "expiry_tick_us", a ? string(std::to_string(a->expires)) : Value()); return out;
}
void invalidate(CloseAttempt &a, const char *reason) { if (a.invalidation.empty()) { a.invalidation = reason; } }
Value fail(CloseAttempt &a, const char *reason) {
    invalidate(a, reason); a.terminal = true; a.discarded = !a.entered;
    a.phase = CloseAttempt::Phase::Terminal; return reply(&a, "refused", a.invalidation.c_str());
}
void observe_removal(CloseAttempt &a, const Binding &current, bool found) {
    if (a.removed && *a.removed) { return; }
    if (!api.from_id(a.eid) && !api.from_id(a.bid)) { a.removed = true; }
    else if (found && current.editor_id == a.eid && current.buffer_id == a.bid) { a.removed = false; }
}
bool owned(const CloseAttempt *a, const Value &request) {
    std::string text; return a && checked_bytes(request, text, 64) && text == a->request;
}
struct Running {
    Session &session;
    explicit Running(Session &value) : session(value) { session.call_state = Session::CallState::Running; }
    ~Running() {
        const bool closing = session.call_state == Session::CallState::Closing;
        session.call_state = Session::CallState::Idle;
        if (closing) { close(session); }
    }
};
const char *scope_guard(const Session &s, const CloseAttempt &a) {
    if (s.session_id != a.session || s.project_fd < 0 || ticks() >= a.expires ||
            s.call_state == Session::CallState::Closing) { return "session_or_expiry_changed"; }
    return attached_project(s) ? nullptr : "project_namespace_changed";
}
const char *owner_guard(const Session &s, const CloseAttempt &a) {
    if (!a.invalidation.empty()) { return a.invalidation.c_str(); }
    if (a.terminal) { return "terminal_attempt"; }
    return scope_guard(s, a);
}
bool selected(uint64_t &script_id, uint64_t &editor_id, uint64_t &buffer_id) {
    Value owner, script, editor;
    if (!checked_call(owner, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
            !checked_call(script, object_ptr(owner), "ScriptEditor", "get_current_script", GAK_HASH_CURRENT_SCRIPT) ||
            !checked_call(editor, object_ptr(owner), "ScriptEditor", "get_current_editor", GAK_HASH_CURRENT_EDITOR)) { return false; }
    script_id = id(script); editor_id = id(editor); buffer_id = 0;
    if (!script_id) { return !editor_id && (script.type() == GDEXTENSION_VARIANT_TYPE_NIL || script.type() == GDEXTENSION_VARIANT_TYPE_OBJECT); }
    Value path; std::string text; Binding b; bool found = false;
    if (!checked_call(path, object_ptr(script), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
            !checked_bytes(path, text, 2048) || !find_document(text, b, found) || !found ||
            b.script_id != script_id || b.editor_id != editor_id) { return false; }
    buffer_id = b.buffer_id; return true;
}
const char *roster(std::vector<std::string> &paths, std::vector<std::array<uint64_t, 3>> &identities) {
    Value owner, scripts, editors; int64_t n = 0, m = 0;
    if (!checked_call(owner, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
            !checked_call(scripts, object_ptr(owner), "ScriptEditor", "get_open_scripts", GAK_HASH_OPEN_SCRIPTS) || !size(scripts, n, 8) ||
            !checked_call(editors, object_ptr(owner), "ScriptEditor", "get_open_script_editors", GAK_HASH_OPEN_EDITORS) ||
            !size(editors, m, 8) || n != m) { return "roster_unavailable_or_limit"; }
    std::vector<std::pair<std::string, std::array<uint64_t, 3>>> entries;
    std::set<uint64_t> rids, eids, bids;
    for (int64_t i = 0; i < n; ++i) {
        Value script = at(scripts, i), editor = at(editors, i), path_value;
        std::string path; Binding b; bool found = false; Name gd("GDScript");
        if (!object_ptr(script) || !api.cast_to(object_ptr(script), api.class_tag(gd.ptr())) ||
                !checked_call(path_value, object_ptr(script), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
                !checked_bytes(path_value, path, 2048) || !path_ok(path) ||
                !find_document(path, b, found) || !found || b.script_id != id(script) || b.editor_id != id(editor) ||
                !rids.insert(b.script_id).second || !eids.insert(b.editor_id).second || !bids.insert(b.buffer_id).second) {
            return "unsupported_roster_association";
        }
        entries.push_back({std::move(path), {b.script_id, b.editor_id, b.buffer_id}});
    }
    std::sort(entries.begin(), entries.end(), [](const auto &x, const auto &y) { return x.first < y.first; });
    for (auto &entry : entries) {
        if (!paths.empty() && paths.back() == entry.first) { return "duplicate_roster_path"; }
        paths.push_back(std::move(entry.first)); identities.push_back(entry.second);
    }
    return nullptr;
}
const char *idle(uint64_t &delay, uint64_t &error_delay) {
    Value settings;
    if (!checked_call(settings, singleton("EditorInterface"), "EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS)) {
        return "idle_parse_delay_unavailable";
    }
    const char *keys[] = {"text_editor/completion/idle_parse_delay", "text_editor/completion/idle_parse_delay_with_errors_found"};
    uint64_t *delays[] = {&delay, &error_delay};
    for (size_t i = 0; i < 2; ++i) {
        Value value, key = string(keys[i]);
        if (!checked_call(value, object_ptr(settings), "EditorSettings", "get_setting", GAK_HASH_GET_SETTING, {&key})) { return "idle_parse_delay_unavailable"; }
        double seconds = 0;
        if (value.type() == GDEXTENSION_VARIANT_TYPE_FLOAT) { api.to[GDEXTENSION_VARIANT_TYPE_FLOAT](&seconds, value.ptr()); }
        else { int64_t n = 0; if (!number(value, n)) { return "idle_parse_delay_unavailable"; } seconds = n; }
        if (!std::isfinite(seconds) || seconds < 0 || seconds > 9) { return "idle_parse_delay_unsupported"; }
        *delays[i] = static_cast<uint64_t>(std::ceil(seconds * 1000000));
    }
    return nullptr;
}
const char *sort_order(int64_t &order) {
    Value settings, value, key = string("text_editor/script_list/sort_scripts_by");
    if (!checked_call(settings, singleton("EditorInterface"), "EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS) ||
            !checked_call(value, object_ptr(settings), "EditorSettings", "get_setting", GAK_HASH_GET_SETTING, {&key}) ||
            !number(value, order) || order < 0 || order > 2) { return "script_sort_configuration_unavailable"; }
    return nullptr;
}
// The same typed F/E format as individual source-context records, streamed with
// a hard metadata ceiling rather than allocating an encoded aggregate.
struct Digest {
    CC_SHA256_CTX context{}; size_t count{}; bool good = CC_SHA256_Init(&context) == 1;
    void raw(std::string_view s) {
        constexpr size_t limit = 256 * 1024 + 64; // Domain frame is not projection metadata.
        if (s.size() > limit - std::min(count, limit)) { good = false; return; }
        count += s.size(); good = CC_SHA256_Update(&context, s.data(), static_cast<CC_LONG>(s.size())) == 1 && good;
    }
    void length(uint32_t n) { char b[4]; for (size_t i = 0; i < 4; ++i) { b[i] = static_cast<char>(n >> (24 - i * 8)); } raw({b, 4}); }
    void frame(std::string_view s) { length(s.size()); raw(s); }
    bool encode(Value value, unsigned depth = 0) {
        if (depth > 8 || !good) { return false; }
        switch (value.type()) {
            case GDEXTENSION_VARIANT_TYPE_STRING: { std::string s; if (!checked_bytes(value, s, 2048)) { return false; } raw("s"); frame(s); break; }
            case GDEXTENSION_VARIANT_TYPE_BOOL: { char b[2] = {'b', static_cast<char>(truth(value))}; raw({b, 2}); break; }
            case GDEXTENSION_VARIANT_TYPE_ARRAY: {
                int64_t n = 0; if (!size(value, n, 8)) { return false; } raw("a"); length(n);
                for (int64_t i = 0; i < n; ++i) { if (!encode(at(value, i), depth + 1)) { return false; } } break;
            }
            case GDEXTENSION_VARIANT_TYPE_DICTIONARY: {
                Value keys; int64_t n = 0; if (!checked_invoke(keys, value, "keys") || !size(keys, n, 32)) { return false; }
                std::vector<std::string> names;
                for (int64_t i = 0; i < n; ++i) { std::string key; if (!checked_bytes(at(keys, i), key, 64)) { return false; } names.push_back(std::move(key)); }
                std::sort(names.begin(), names.end()); raw("o"); length(n);
                for (const auto &key : names) { frame(key); if (!encode(get(value, key.c_str()), depth + 1)) { return false; } } break;
            }
            default: return false;
        }
        return good;
    }
    std::string finish() {
        unsigned char bytes[32]; if (!good || CC_SHA256_Final(bytes, &context) != 1) { return {}; }
        const char *hex = "0123456789abcdef"; std::string out(64, '0');
        for (size_t i = 0; i < 32; ++i) { out[i * 2] = hex[bytes[i] >> 4]; out[i * 2 + 1] = hex[bytes[i] & 15]; } return out;
    }
};
Value identity(const std::string &path, const std::array<uint64_t, 3> &ids) {
    Value out = dict(); put_string(out, "path", path); put_string(out, "script_id", std::to_string(ids[0]));
    put_string(out, "editor_id", std::to_string(ids[1])); put_string(out, "buffer_id", std::to_string(ids[2])); return out;
}
const char *aggregate(const Session &s, CloseAttempt &a) {
    Value target = dict();
    put_string(target, "target_device", std::to_string(a.target.file.device)); put_string(target, "target_inode", std::to_string(a.target.file.inode));
    put_string(target, "script_id", std::to_string(a.rid)); put_string(target, "editor_id", std::to_string(a.eid)); put_string(target, "buffer_id", std::to_string(a.bid));
    put_string(target, "source_sha256", a.source_hash); put_string(target, "source_length", std::to_string(a.source.size()));
    put_string(target, "version", std::to_string(a.target.version)); put_string(target, "saved_version", std::to_string(a.target.saved_version));
    put(target, "dirty", boolean(a.target.dirty)); put(target, "resource_edited", boolean(a.target.edited)); put_string(target, "guard_sha256", a.target.guard_hash);
    a.target_guard = target;
    Value p = dict(), roster_value = collection(GDEXTENSION_VARIANT_TYPE_ARRAY), protected_value = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    put_string(p, "request_id", a.request); put_string(p, "session_id", a.session); put_string(p, "project_root", s.project_path);
    put_string(p, "project_device", std::to_string(s.device)); put_string(p, "project_inode", std::to_string(s.inode));
    put_string(p, "target_path", a.path); put(p, "target", target);
    put_string(p, "selected_script_id", std::to_string(a.selected_script)); put_string(p, "selected_editor_id", std::to_string(a.selected_editor));
    put_string(p, "selected_buffer_id", std::to_string(a.selected_buffer)); put_string(p, "idle_parse_delay_us", std::to_string(a.idle_delay));
    put_string(p, "idle_parse_error_delay_us", std::to_string(a.error_delay));
    put_string(p, "effective_sha256", a.effective.fingerprint);
    for (size_t i = 0; i < a.paths.size(); ++i) { append(roster_value, identity(a.paths[i], a.identities[i])); }
    size_t bytes = 0;
    Value documents = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (const auto &c : a.protected_documents) {
        Value value = identity(c->path, {c->document.script_id, c->document.editor_id, c->document.buffer_id});
        put_string(value, "guard_sha256", c->guard_hash); append(protected_value, value); append(documents, opening_context_reply(*c)); bytes += c->record_bytes;
    }
    put(p, "roster", roster_value); put(p, "protected", protected_value);
    Digest digest; digest.frame("godot-agent-kit/close-context/v1"); const size_t domain_bytes = digest.count;
    if (!digest.encode(p) || digest.count - domain_bytes > 256 * 1024 ||
            bytes > 256 * 1024 - (digest.count - domain_bytes)) { return "close_context_metadata_limit"; }
    a.guard_hash = digest.finish(); if (a.guard_hash.empty()) { return "close_context_hash_unavailable"; }
    a.aggregate = dict(); put(a.aggregate, "projection", p); put(a.aggregate, "documents", documents); return nullptr;
}
const char *preservation_guard(Session &s, const CloseAttempt &a, bool include_target) {
    OpeningEffective effective;
    if (const char *reason = opening_effective(effective)) { return reason; }
    if (effective.fingerprint != a.effective.fingerprint) { return "effective_context_changed"; }
    uint64_t delay = 0, error_delay = 0;
    if (const char *reason = idle(delay, error_delay)) { return reason; }
    if (delay != a.idle_delay || error_delay != a.error_delay) { return "idle_parse_delay_changed"; }
    int64_t order = -1;
    if (const char *reason = sort_order(order)) { return reason; }
    if (order != a.sort_order) { return "script_sort_configuration_changed"; }
    auto document_guard = [&](const OpeningContext &original) -> const char * {
        OpeningContext fresh;
        if (const char *reason = editor_context(s, a.request, effective, original.document, fresh, &original)) { return reason; }
        return fresh.guard_hash == original.guard_hash ? nullptr : "document_preservation_changed";
    };
    if (include_target) { if (const char *reason = document_guard(a.target)) { return reason; } }
    for (const auto &c : a.protected_documents) { if (const char *reason = document_guard(*c)) { return reason; } }
    return nullptr;
}
const char *guard(Session &s, CloseAttempt &a, bool after) {
    if (const char *reason = owner_guard(s, a)) { return reason; }
    const FileBinding &file = a.open ? a.target.file : a.recognition_file;
    if (!file_attached(s, file)) { return "target_namespace_changed"; }
    if (a.open && !content(file.leaf.value, a.source)) { return "target_disk_changed"; }
    if (a.open && (!file_attached(s, a.settings_file) || !content(a.settings_file.leaf.value, a.settings_source))) {
        return "validator_settings_changed";
    }
    Binding b; bool found = false;
    if (!find_document(a.path, b, found)) { return "target_association_unavailable"; }
    if (!a.open) { return found ? "recognized_target_opened" : nullptr; }
    if (after) {
        observe_removal(a, b, found);
        if (found) { return "target_document_reopened_or_retained"; }
        if (api.from_id(a.eid) || api.from_id(a.bid)) { return "target_disposal_unavailable"; }
    } else if (!found || b.script_id != a.rid || b.editor_id != a.eid || b.buffer_id != a.bid) { return "target_identity_changed"; }
    std::vector<std::string> paths; std::vector<std::array<uint64_t, 3>> ids;
    if (const char *reason = roster(paths, ids)) { return reason; }
    size_t j = 0;
    for (size_t i = 0; i < a.paths.size(); ++i) {
        if (after && a.paths[i] == a.path) { continue; }
        if (j >= paths.size() || paths[j] != a.paths[i] || ids[j] != a.identities[i]) { return "roster_changed"; } ++j;
    }
    if (j != paths.size()) { return "roster_changed"; }
    if (const char *reason = preservation_guard(s, a, !after)) { return reason; }
    uint64_t sr = 0, se = 0, sb = 0;
    if (!selected(sr, se, sb)) { return "selection_unavailable"; }
    if (!after || a.selected_script != a.rid) {
        if (sr != a.selected_script || se != a.selected_editor || sb != a.selected_buffer) { return "selection_changed"; }
    } else if (sr && std::find(ids.begin(), ids.end(), std::array<uint64_t, 3>{sr, se, sb}) == ids.end()) { return "selection_outside_protection"; }
    a.after_script = sr; a.after_known = true;
    if (const char *reason = owner_guard(s, a)) { return reason; }
    const uint64_t now = ticks();
    if (!after && (now >= a.expires || std::max(a.idle_delay, a.error_delay) >= a.expires - now)) { return "idle_parse_delay_exceeds_lease"; }
    return nullptr;
}
// Serialize only the existing validator's closed WarningSettings and name list.
// Field order follows its Rust structs; map order follows their BTreeMap keys.
std::string json_string(std::string_view value) {
    std::string out = "\"";
    constexpr char hex[] = "0123456789abcdef";
    for (unsigned char c : value) {
        if (c == '"' || c == '\\') { out += '\\'; out += static_cast<char>(c); }
        else if (c == '\n') { out += "\\n"; }
        else if (c == '\r') { out += "\\r"; }
        else if (c == '\t') { out += "\\t"; }
        else if (c == '\b') { out += "\\b"; }
        else if (c == '\f') { out += "\\f"; }
        else if (c < 32) { out += "\\u00"; out += hex[c >> 4]; out += hex[c & 15]; }
        else { out += static_cast<char>(c); }
    }
    return out + "\"";
}
std::string json_levels(const std::map<std::string, uint64_t> &values) {
    std::string out = "{"; bool first = true;
    for (const auto &entry : values) {
        if (!first) { out += ','; } first = false;
        out += json_string(entry.first) + ":" + std::to_string(entry.second);
    }
    return out + "}";
}
const char *validator_context(Session &s, CloseAttempt &a) {
    if (const char *reason = pin_file(s, "res://project.godot", a.settings_file, O_RDONLY, true)) { return reason; }
    struct stat st{};
    if (fstat(a.settings_file.leaf.value, &st) || st.st_size < 0 || st.st_size > static_cast<off_t>(SOURCE_LIMIT)) { return "validator_settings_limit"; }
    a.settings_source.resize(static_cast<size_t>(st.st_size));
    size_t offset = 0;
    while (offset < a.settings_source.size()) {
        const ssize_t n = pread(a.settings_file.leaf.value, a.settings_source.data() + offset,
                a.settings_source.size() - offset, static_cast<off_t>(offset));
        if (n < 0 && errno == EINTR) { continue; }
        if (n <= 0) { return "validator_settings_unavailable"; } offset += static_cast<size_t>(n);
    }
    if (!valid_utf8(a.settings_source) || !content(a.settings_file.leaf.value, a.settings_source)) { return "validator_settings_changed"; }
    const auto &e = a.effective;
    std::string warnings = std::string("{\"enable\":") + (e.warnings_enable ? "true" : "false") +
            ",\"levels\":" + json_levels(e.warning_levels) + ",\"directory_rules\":" + json_levels(e.directory_rules) +
            ",\"provenance\":{\"source\":\"editor_project_settings\",\"project_root\":" + json_string(s.project_path) +
            ",\"session_id\":" + json_string(s.session_id) + "}}";
    std::string globals = "["; bool first = true;
    for (const auto &value : e.global_classes) { if (!first) { globals += ','; } first = false; globals += json_string(value); }
    globals += "]";
    a.validator_context = sha256(sha256(a.settings_source) + ":" + warnings + ":" + globals);
    return nullptr;
}
#if GAK_FIXTURE
void fixture_callback(CloseAttempt &a, const char *stage) {
    Value key = name("godot_agent_kit_close_fixture_callback"), fallback, callable;
    if (checked_call(callable, singleton("Engine"), "Object", "get_meta", GAK_HASH_GET_META, {&key, &fallback}) &&
            callable.type() == GDEXTENSION_VARIANT_TYPE_CALLABLE) {
        Value request = string(a.request), stage_value = string(stage), ignored;
        checked_invoke(ignored, callable, "call", {&request, &stage_value});
    }
}
#endif
struct Witness { Session *session; std::string request; uint64_t generation, editor; bool visit; };
void witness(void *data, const GDExtensionConstVariantPtr *arguments, GDExtensionInt count,
        GDExtensionVariantPtr, GDExtensionCallError *error) {
    error->error = GDEXTENSION_CALL_OK;
    auto &w = *static_cast<Witness *>(data); Session &s = *w.session; CloseAttempt *a = close_attempt(s);
    if (!a || a->request != w.request || a->generation != w.generation || a->terminal ||
            !a->entered || !a->invalidation.empty()) { return; }
#if GAK_FIXTURE
    if (!w.visit && a->fault == "drop_completion") { return; }
#endif
    // Hold this exact Callable through the callback return, including reentrant
    // session teardown which may disconnect and release its connection vector.
    Value keep_alive;
    for (const auto &connection : a->connections) {
        if ((w.visit && std::string_view(connection.signal) == "editor_script_changed") ||
                (!w.visit && connection.object == w.editor)) { keep_alive = connection.callable; break; }
    }
    const bool outer = s.call_state == Session::CallState::Idle;
    if (outer) { s.call_state = Session::CallState::Running; }
    const uint64_t now = ticks();
    uint64_t observed_editor = w.editor;
#if GAK_FIXTURE
    // Corrupt only attribution at the fixed fault boundary. The production
    // completion path must ignore it rather than manufacture a verdict.
    if (!w.visit && a->fault == "wrong_completion") { observed_editor = 0; }
#endif
    if (now >= a->expires || s.call_state == Session::CallState::Closing) { invalidate(*a, "session_or_expiry_changed"); }
    else if ((w.visit && count != 1) || (!w.visit && count != 0)) { invalidate(*a, "signal_signature_changed"); }
    else if (w.visit) {
        Value visited(arguments[0]); const uint64_t sr = id(visited);
        if (sr && sr != a->rid) {
            const auto found = std::find_if(a->protected_documents.begin(), a->protected_documents.end(),
                    [&](const auto &c) { return c->document.script_id == sr; });
            Binding live;
            if (found == a->protected_documents.end() || !resolve((*found)->path, sr,
                    (*found)->document.editor_id, (*found)->document.buffer_id, live)) { invalidate(*a, "visit_outside_protection"); }
            else {
                const uint64_t se = live.editor_id;
                a->required.insert(se); a->completed.erase(se); a->collections[se] = {now, 0};
            }
        } else if (visited.type() != GDEXTENSION_VARIANT_TYPE_NIL && visited.type() != GDEXTENSION_VARIANT_TYPE_OBJECT) {
            invalidate(*a, "visit_attribution_unavailable");
        }
    } else if (a->required.count(observed_editor)) {
        const auto found = std::find_if(a->protected_documents.begin(), a->protected_documents.end(),
                [&](const auto &c) { return c->document.editor_id == observed_editor; });
        if (found == a->protected_documents.end()) { invalidate(*a, "completion_attribution_unavailable"); }
        else {
            OpeningContext fresh;
            const char *reason = a->returned ? guard(s, *a, true) :
                    editor_context(s, a->request, a->effective, (*found)->document, fresh, found->get());
            if (reason || (!a->returned && fresh.guard_hash != (*found)->guard_hash)) { invalidate(*a, reason ? reason : "completion_preservation_changed"); }
            else { a->completed.insert(observed_editor); a->collections[observed_editor].second = now; }
        }
    }
#if GAK_FIXTURE
    if (!w.visit && a->fault == "callback_completion") { a->fault.clear(); fixture_callback(*a, "callback"); }
#endif
    if (!a->invalidation.empty()) {
        a->terminal = true; a->discarded = !a->entered;
        if (a->returned) { a->phase = CloseAttempt::Phase::Terminal; }
    }
    if (a->returned && !a->terminal) { a->phase = a->required == a->completed ? CloseAttempt::Phase::Settled : CloseAttempt::Phase::Settling; }
    if (a->terminal && outer) { a->phase = CloseAttempt::Phase::Terminal; }
    if (outer) {
        const bool closing = s.call_state == Session::CallState::Closing; s.call_state = Session::CallState::Idle;
        if (closing) { close(s); }
    }
}
const char *connect(Session &s, CloseAttempt &a, uint64_t object, const char *signal, bool visit) {
    Value sig = name(signal), has, result;
    void *ptr = api.from_id(object);
    if (!checked_call(has, ptr, "Object", "has_signal", GAK_HASH_HAS_SIGNAL, {&sig}) || !truth(has)) { return "required_signal_unavailable"; }
    auto *data = new Witness{&s, a.request, a.generation, visit ? 0 : object, visit};
    Storage<GAK_SIZE_CALLABLE> storage; GDExtensionCallableCustomInfo2 info{};
    info.callable_userdata = data; info.token = library; info.call_func = witness;
    info.free_func = [](void *p) { delete static_cast<Witness *>(p); };
    api.new_callable(storage.ptr(), &info); Value callable = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr());
    api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    Value flags = integer(0); int64_t code = -1;
    if (!checked_call(result, ptr, "Object", "connect", GAK_HASH_CONNECT, {&sig, &callable, &flags}) || !number(result, code) || code != 0) {
        return "signal_connection_failed";
    }
    a.connections.push_back({object, signal, callable}); return nullptr;
}
Value resource(CloseAttempt &a) {
    const uint64_t start = ticks(); Value out = dict(), path = string(a.path), cached, script, edited;
    bool known = checked_call(cached, singleton("ResourceLoader"), "ResourceLoader", "has_cached", GAK_HASH_HAS_CACHED, {&path}) &&
            cached.type() == GDEXTENSION_VARIANT_TYPE_BOOL &&
            checked_call(script, singleton("ResourceLoader"), "ResourceLoader", "get_cached_ref", GAK_HASH_CACHED_REF, {&path});
    a.edited.reset(); a.resource_state = "unavailable";
    if (!a.open && known) {
        const uint64_t current_id = id(script);
        if (a.recognition_cache_id && *a.recognition_cache_id != current_id) { invalidate(a, "recognition_resource_identity_changed"); }
        else { a.recognition_cache_id = current_id; }
    }
    if (a.open && !a.entered && known && (!truth(cached) || id(script) != a.rid)) {
        invalidate(a, "cache_document_identity_mismatch");
    }
    if (known && !truth(cached) && !object_ptr(script) &&
            (script.type() == GDEXTENSION_VARIANT_TYPE_NIL || script.type() == GDEXTENSION_VARIANT_TYPE_OBJECT)) { a.resource_state = "unloaded"; }
    else if (known && truth(cached) && object_ptr(script)) {
        Value source, actual_path; std::string text, actual; Name gd("GDScript");
        if (!api.class_tag(gd.ptr()) || !api.cast_to(object_ptr(script), api.class_tag(gd.ptr())) ||
                !checked_call(actual_path, object_ptr(script), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
                !checked_bytes(actual_path, actual, 2048) || actual != a.path) {
            invalidate(a, "resource_cache_attribution_changed"); a.resource_state = "invalidated";
        } else if (a.entered && id(script) != a.rid) { invalidate(a, "retained_resource_replaced"); a.resource_state = "invalidated"; }
        else if (checked_call(source, object_ptr(script), "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) && checked_bytes(source, text, SOURCE_LIMIT) &&
                checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&script}) && edited.type() == GDEXTENSION_VARIANT_TYPE_BOOL) {
            a.edited = truth(edited); a.resource_state = "retained";
            if (a.entered && (sha256(text) != a.source_hash || *a.edited)) { invalidate(a, "retained_resource_changed"); a.resource_state = "invalidated"; }
        }
    }
    put_string(out, "state", a.resource_state); put(out, "resource_edited", a.edited ? boolean(*a.edited) : Value());
    put(out, "reason", a.resource_state == "unavailable" ? string("resource_unavailable") :
            a.invalidation.empty() ? Value() : string(a.invalidation)); put(out, "collection", stamp(a.session, start, ticks())); return out;
}
} // namespace

void close_cleanup(Session &s) {
    CloseAttempt *a = close_attempt(s); if (!a || s.call_state != Session::CallState::Idle) { return; }
    for (auto &c : a->connections) {
        if (void *object = api.from_id(c.object)) {
            Value signal = name(c.signal), connected;
            if (checked_call(connected, object, "Object", "is_connected", GAK_HASH_IS_CONNECTED, {&signal, &c.callable}) && truth(connected)) {
                Value ignored; checked_call(ignored, object, "Object", "disconnect", GAK_HASH_DISCONNECT, {&signal, &c.callable});
            }
        }
    }
    s.owner = std::monostate{}; delete a;
}
void close_closing(Session &s) {
    if (auto *a = close_attempt(s)) { a->terminal = true; a->discarded = !a->entered; invalidate(*a, "session_closed"); }
}
Value close_inspect(Session &s, const Value &path_value, const Value &correlation) {
    if (occupied(s) || s.call_state != Session::CallState::Idle) { return reply(nullptr, "refused", "slot_busy"); }
    std::string path, request, session; uint64_t expiry = 0, now = ticks();
    if (s.project_fd < 0 || std::this_thread::get_id() != s.main_thread || !checked_bytes(path_value, path, 2048) || !path_ok(path) ||
            !exact(correlation, {"request_id", "session_id", "expiry_tick_us"}) ||
            !checked_bytes(get(correlation, "request_id"), request, 64) || !valid_request(request) ||
            !checked_bytes(get(correlation, "session_id"), session, 32) || session != s.session_id ||
            !decimal(get(correlation, "expiry_tick_us"), expiry) || expiry <= now || expiry - now > 9000000) { return reply(nullptr, "refused", "invalid_binding"); }
    if (s.close_generation == UINT64_MAX) { return reply(nullptr, "refused", "generation_exhausted"); }
    auto *a = new CloseAttempt; a->generation = ++s.close_generation;
    a->path = path; a->request = request; a->session = session; a->expires = expiry; s.owner = a;
    Running running(s); Binding b;
    if (!attached_project(s) || !find_document(path, b, a->open)) { return fail(*a, "document_association_unavailable"); }
    if (const char *reason = pin_file(s, path, a->recognition_file, O_RDONLY, a->open)) { return fail(*a, reason); }
    if (a->open) { a->target.document = b; a->rid = b.script_id; a->eid = b.editor_id; a->bid = b.buffer_id; }
    a->selected_known = selected(a->selected_script, a->selected_editor, a->selected_buffer);
    a->after_known = a->selected_known; a->after_script = a->selected_script;
    Value state = resource(*a);
    if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason); }
    Value out = reply(a, "inspected", ""); put(out, "resource_state", state);
    put(out, "resource_edited", a->edited ? boolean(*a->edited) : Value()); put(out, "selection", selection(*a)); return out;
}
Value close_prepare(Session &s, const Value &request, const Value &source_value, const Value &capture) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    Running running(s);
    if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason); }
    if (!a->open || a->phase != CloseAttempt::Phase::Inspected) { return fail(*a, "wrong_stage"); }
    uint64_t pd = 0, pi = 0, td = 0, ti = 0, rid = 0, eid = 0, bid = 0, version = 0;
    int64_t length = -1; std::string hash; Value basis = get(capture, "basis");
    if (!exact(capture, {"project_device", "project_inode", "target_device", "target_inode", "source_sha256", "utf8_bytes", "basis"}) ||
            !exact(basis, {"script_instance_id", "editor_instance_id", "buffer_instance_id", "current_version"}) ||
            !decimal(get(capture, "project_device"), pd) || !decimal(get(capture, "project_inode"), pi) ||
            !decimal(get(capture, "target_device"), td) || !decimal(get(capture, "target_inode"), ti) ||
            !decimal(get(basis, "script_instance_id"), rid) || !decimal(get(basis, "editor_instance_id"), eid) ||
            !decimal(get(basis, "buffer_instance_id"), bid) || !decimal(get(basis, "current_version"), version) ||
            !number(get(capture, "utf8_bytes"), length) || length < 0 || !checked_bytes(get(capture, "source_sha256"), hash, 64) ||
            !checked_bytes(source_value, a->source, SOURCE_LIMIT) || hash != sha256(a->source) || static_cast<uint64_t>(length) != a->source.size() ||
            pd != static_cast<uint64_t>(s.device) || pi != static_cast<uint64_t>(s.inode) || rid != a->rid || eid != a->eid || bid != a->bid) {
        return fail(*a, "invalid_capture_or_basis");
    }
    a->source_hash = hash;
    if (!file_attached(s, a->recognition_file) || td != static_cast<uint64_t>(a->recognition_file.device) ||
            ti != static_cast<uint64_t>(a->recognition_file.inode)) { return fail(*a, "target_identity_changed"); }
    if (const char *reason = opening_effective(a->effective)) { return fail(*a, reason); }
    if (const char *reason = validator_context(s, *a)) { return fail(*a, reason); }
    if (const char *reason = idle(a->idle_delay, a->error_delay)) { return fail(*a, reason); }
    if (const char *reason = sort_order(a->sort_order)) { return fail(*a, reason); }
    const uint64_t now = ticks();
    if (now >= a->expires || std::max(a->idle_delay, a->error_delay) >= a->expires - now) { return fail(*a, "idle_parse_delay_exceeds_lease"); }
    if (const char *reason = roster(a->paths, a->identities)) { return fail(*a, reason); }
    Binding target = a->target.document;
    if (const char *reason = editor_context(s, a->request, a->effective, target, a->target,
            nullptr, 256 * 1024, false)) { return fail(*a, reason); }
    if (a->target.source != a->source || a->target.version != version || a->target.dirty || a->target.edited ||
            a->target.version != a->target.saved_version || !content(a->target.file.leaf.value, a->source)) { return fail(*a, "target_not_clean_or_basis_changed"); }
    if (td != static_cast<uint64_t>(a->target.file.device) || ti != static_cast<uint64_t>(a->target.file.inode)) { return fail(*a, "target_identity_changed"); }
    std::string().swap(a->target.source); // D/R/B agreed; retain only the independent captured comparison source.
    if (!a->selected_known) { return fail(*a, "selection_unavailable"); }
    size_t sources = 0, metadata = 0;
    for (size_t i = 0; i < a->paths.size(); ++i) {
        if (a->paths[i] == a->path) { continue; }
        Binding document;
        if (!resolve(a->paths[i], a->identities[i][0], a->identities[i][1], a->identities[i][2], document)) { return fail(*a, "roster_identity_changed"); }
        Value source;
        if (!checked_call(source, document.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) ||
                utf8_size(source) < 0 || static_cast<uint64_t>(utf8_size(source)) > SOURCE_LIMIT - sources) { return fail(*a, "protected_source_aggregate_limit"); }
        auto c = std::make_unique<OpeningContext>();
        if (const char *reason = editor_context(s, a->request, a->effective, document, *c,
                nullptr, 256 * 1024 - metadata)) { return fail(*a, reason); }
        sources += c->source.size(); metadata += c->record_bytes;
        if (metadata > 256 * 1024) { return fail(*a, "close_context_metadata_limit"); }
        a->protected_documents.push_back(std::move(c));
    }
    if (const char *reason = aggregate(s, *a)) { return fail(*a, reason); }
    a->phase = CloseAttempt::Phase::Prepared;
    if (const char *reason = guard(s, *a, false)) { return fail(*a, reason); }
    Value out = reply(a, "prepared", ""); put(out, "target_guard", a->target_guard); put(out, "context", a->aggregate);
    put_string(out, "guard_sha256", a->guard_hash); return out;
}
Value close_advance(Session &s, const Value &request, const Value &guard_value, const Value &receipts_value) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    Running running(s); std::string hash; Value receipts = receipts_value; int64_t count = 0;
    if (a->phase != CloseAttempt::Phase::Prepared || a->entered || !checked_bytes(guard_value, hash, 64) || hash != a->guard_hash ||
            !size(receipts, count, 7) || static_cast<size_t>(count) != a->protected_documents.size()) { return fail(*a, "wrong_stage_or_authorization"); }
    for (int64_t i = 0; i < count; ++i) {
        const auto &c = *a->protected_documents[i]; Value receipt = at(receipts, i);
        if (!exact(receipt, {"request_id", "session_id", "target_path", "document_path", "script_id", "editor_id", "buffer_id", "source_sha256", "utf8_bytes", "guard_sha256", "context_sha256"})) {
            return fail(*a, "invalid_receipt_binding");
        }
        const std::pair<const char *, std::string> fields[] = {{"request_id", a->request}, {"session_id", a->session}, {"target_path", a->path},
            {"document_path", c.path}, {"script_id", std::to_string(c.document.script_id)}, {"editor_id", std::to_string(c.document.editor_id)},
            {"buffer_id", std::to_string(c.document.buffer_id)}, {"source_sha256", c.source_hash}, {"guard_sha256", c.guard_hash}, {"context_sha256", a->validator_context}};
        for (const auto &field : fields) { if (bytes(get(receipt, field.first), 2048) != field.second || field.second.empty()) { return fail(*a, "receipt_binding_mismatch"); } }
        int64_t length = -1; if (!number(get(receipt, "utf8_bytes"), length) || length < 0 || static_cast<size_t>(length) != c.source.size()) { return fail(*a, "receipt_source_length_mismatch"); }
    }
    Value owner;
    if (!checked_call(owner, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) || !id(owner)) { return fail(*a, "script_editor_unavailable"); }
#if GAK_FIXTURE
    if (a->fault == "missing_signal") { return fail(*a, "required_signal_unavailable"); }
#endif
    if (const char *reason = connect(s, *a, id(owner), "editor_script_changed", true)) { return fail(*a, reason); }
    for (const auto &c : a->protected_documents) { if (const char *reason = connect(s, *a, c->document.editor_id, "edited_script_changed", false)) { return fail(*a, reason); } }
#if GAK_FIXTURE
    if (a->fault == "expire_before_entry") { return fail(*a, "session_or_expiry_changed"); }
    if (a->fault == "premature_completion" && !a->protected_documents.empty()) {
        Witness pre{&s, a->request, a->generation, a->protected_documents.front()->document.editor_id, false};
        GDExtensionCallError error{}; witness(&pre, nullptr, 0, nullptr, &error);
    }
    if (a->fault == "callback_entry") { fixture_callback(*a, "entered"); }
#endif
    if (const char *reason = guard(s, *a, false)) { return fail(*a, reason); }
    Value path = string(a->path), result;
    a->entered = true; a->phase = CloseAttempt::Phase::Entered; a->entry = ticks();
    const bool returned = checked_call(result, object_ptr(owner), "ScriptEditor", "close_file", GAK_HASH_CLOSE_FILE, {&path});
    a->returned = ticks(); a->phase = CloseAttempt::Phase::Returned;
    int64_t code = -1; if (returned && number(result, code) && code >= 0 && code <= 255) { a->error = code; }
    // No target-only Resource reference survives the call into ordinary observation.
    a->target.document = Binding{}; a->target.source.clear(); a->target.projection = Value();
    Binding immediate; bool found = false;
    if (find_document(a->path, immediate, found)) {
        observe_removal(*a, immediate, found);
    }
    uint64_t selected_editor = 0, selected_buffer = 0;
    a->after_known = selected(a->after_script, selected_editor, selected_buffer);
#if GAK_FIXTURE
    if (a->fault == "fail_after_return") { return fail(*a, "fixture_after_return"); }
    if (a->fault == "cancel_at_entry") { Value id_value = string(a->request); close_abort(s, id_value); }
    if (a->fault == "disable_at_entry") { close(s); }
#endif
    if (!returned || !a->error) { return fail(*a, "close_return_unavailable"); }
    if (*a->error != 0) { return fail(*a, "close_error"); }
    if (const char *reason = guard(s, *a, true)) { return fail(*a, reason); }
    a->phase = a->required == a->completed ? CloseAttempt::Phase::Settled : CloseAttempt::Phase::Settling;
    return reply(a, "returned", "");
}
Value close_status(Session &s, const Value &request) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "unavailable", "slot_busy"); }
    Running running(s);
    if (!a->terminal) {
        if (const char *reason = owner_guard(s, *a)) { return fail(*a, reason); }
        if (a->returned) { if (const char *reason = guard(s, *a, true)) { return fail(*a, reason); } }
    }
    return reply(a, continuation(*a), a->invalidation.c_str());
}
namespace {
Value observe(Session &s, const Value &request, const Value &purpose_value, bool recheck) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (s.call_state != Session::CallState::Idle) { return reply(a, "refused", "slot_busy"); }
    Running running(s); std::string purpose;
    if (!recheck && checked_bytes(purpose_value, purpose, 16) && purpose == "survivor") {
        // Fresh failure evidence is not another successful-close guard. Keep
        // the original effects/invalidation, and irrevocably retire authority.
        if (const char *reason = scope_guard(s, *a)) { return fail(*a, reason); }
        if (!file_attached(s, a->recognition_file)) { return fail(*a, "target_namespace_changed"); }
        a->terminal = true; a->discarded = !a->entered; a->phase = CloseAttempt::Phase::Terminal;
        if (!a->guard_hash.empty()) {
            if (const char *reason = preservation_guard(s, *a, false)) { invalidate(*a, reason); }
        }
        Value state = resource(*a), current_selection = selection(*a);
        uint64_t sr = 0, se = 0, sb = 0;
        const bool known = selected(sr, se, sb);
        put_string(current_selection, "after", relation(sr, a->rid, known));
        Value out = reply(a, "observed", a->invalidation.c_str()); put_string(out, "purpose", purpose);
        put(out, "resource_state", state); put(out, "resource_edited", a->edited ? boolean(*a->edited) : Value());
        put(out, "protection", protection(*a)); put(out, "selection", current_selection); return out;
    }
    if (!checked_bytes(purpose_value, purpose, 16) || !((purpose == "recognition" && !a->open && !a->entered) ||
            (purpose == "pre_close" && recheck && a->phase == CloseAttempt::Phase::Prepared) ||
            (purpose == "post_close" && a->returned))) { return fail(*a, "wrong_verification_purpose"); }
    if (const char *reason = guard(s, *a, a->entered)) { return fail(*a, reason); }
    Value state = resource(*a);
    if (!a->invalidation.empty()) { return fail(*a, a->invalidation.c_str()); }
    if (a->entered && purpose != "survivor" && a->required != a->completed) { return reply(a, "unavailable", "native_validation_pending"); }
    Value out = reply(a, recheck ? "rechecked" : "observed", ""); put_string(out, "purpose", purpose);
    put(out, "resource_state", state); put(out, "resource_edited", a->edited ? boolean(*a->edited) : Value());
    put(out, "protection", protection(*a)); put(out, "selection", selection(*a)); return out;
}
Value terminal(Session &s, const Value &request, bool abort) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    a->terminal = true; a->discarded = !a->entered;
    if (abort) { invalidate(*a, "cancelled"); }
    if (s.call_state == Session::CallState::Idle) { a->phase = CloseAttempt::Phase::Terminal; }
    Value out = reply(a, "released", a->invalidation.c_str()); put(out, "terminal_discard", boolean(a->discarded));
    if (s.call_state == Session::CallState::Idle) { close_cleanup(s); } return out;
}
} // namespace
Value close_verify(Session &s, const Value &request, const Value &purpose) { return observe(s, request, purpose, false); }
Value close_recheck(Session &s, const Value &request, const Value &purpose) { return observe(s, request, purpose, true); }
Value close_finish(Session &s, const Value &request) { return terminal(s, request, false); }
Value close_abort(Session &s, const Value &request) { return terminal(s, request, true); }
Value close_expire(Session &s) {
    auto *a = close_attempt(s); if (!a) { return reply(nullptr, "refused", "no_attempt"); }
    if (ticks() < a->expires) { return reply(a, "unchanged", ""); }
    a->terminal = true; a->discarded = !a->entered; invalidate(*a, "expired");
    if (s.call_state == Session::CallState::Idle) { a->phase = CloseAttempt::Phase::Terminal; }
    Value out = reply(a, "expired", "expired");
    if (s.call_state == Session::CallState::Idle) { close_cleanup(s); } return out;
}
#if GAK_FIXTURE
Value close_fixture_fault(Session &s, const Value &request, const Value &fault_value) {
    auto *a = close_attempt(s); std::string fault;
    if (!owned(a, request) || s.call_state != Session::CallState::Idle || a->terminal ||
            !checked_bytes(fault_value, fault, 32)) { return reply(nullptr, "refused", "wrong_attempt"); }
    const char *allowed[] = {"expire_before_entry", "cancel_at_entry", "disable_at_entry", "fail_after_return", "drop_completion", "premature_completion", "wrong_completion", "callback_entry", "callback_completion", "missing_signal"};
    if (std::find_if(std::begin(allowed), std::end(allowed), [&](const char *value) { return fault == value; }) == std::end(allowed)) { return reply(a, "refused", "invalid_fault"); }
    a->fault = fault; return reply(a, "ready", "");
}
Value close_fixture_state(Session &s, const Value &request) {
    auto *a = close_attempt(s); if (!owned(a, request)) { return reply(nullptr, "refused", "wrong_attempt"); }
    return reply(a, "observed", "");
}
#endif
} // namespace gak
