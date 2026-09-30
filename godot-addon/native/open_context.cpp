#include "open_context.hpp"

#include <algorithm>
#include <fcntl.h>
#include <set>
#include <iterator>
#include <cerrno>
#include <CommonCrypto/CommonDigest.h>
#include <charconv>

namespace gak {
namespace {
void *singleton(const char *name) { Name key(name); return api.singleton(key.ptr()); }
bool text(const Value &value, std::string &out, size_t limit) {
    if (value.type() == GDEXTENSION_VARIANT_TYPE_STRING) { return checked_bytes(value, out, limit); }
    if (value.type() != GDEXTENSION_VARIANT_TYPE_STRING_NAME) { return false; }
    Value converted;
    api.destroy(converted.ptr());
    GDExtensionConstVariantPtr args[] = {value.ptr()};
    GDExtensionCallError error{};
    api.construct(GDEXTENSION_VARIANT_TYPE_STRING, converted.ptr(), args, 1, &error);
    return error.error == GDEXTENSION_CALL_OK && checked_bytes(converted, out, limit);
}
bool count(Value &array, int64_t &size, int64_t limit) {
    Value result;
    return array.type() == GDEXTENSION_VARIANT_TYPE_ARRAY &&
            checked_invoke(result, array, "size") && number(result, size) && size >= 0 && size <= limit;
}
Value at(Value &array, int64_t index) {
    const auto *value = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](array.ptr()), index);
    return value ? Value(value) : Value();
}
void append(Value &array, const Value &value) { Value result; checked_invoke(result, array, "append", {&value}); }
bool setting(void *owner, const char *cls, const char *method, uint64_t hash,
        const std::string &key, Value &out) {
    Value name = string(key);
    return checked_call(out, owner, cls, method, hash, {&name});
}
bool natural(const Value &v, uint64_t &out) {
    int64_t n = -1;
    if (!number(v, n) || n < 0) { return false; }
    out = static_cast<uint64_t>(n);
    return true;
}
bool ordered_names(std::vector<std::string> &names) {
    std::sort(names.begin(), names.end());
    return std::adjacent_find(names.begin(), names.end()) == names.end();
}
// Stream only the closed opening guard record. Rechecks do not allocate an
// encoded record or rebuild the outgoing Godot projection.
struct GuardDigest {
    CC_SHA256_CTX context{};
    size_t bytes{};
    bool good = CC_SHA256_Init(&context) == 1;
    void raw(std::string_view value) {
        bytes += value.size();
        good = CC_SHA256_Update(&context, value.data(), static_cast<CC_LONG>(value.size())) == 1 && good;
    }
    void frame(std::string_view value) {
        char length[4];
        const uint32_t n = static_cast<uint32_t>(value.size());
        for (size_t i = 0; i < 4; ++i) { length[i] = static_cast<char>(n >> (24 - i * 8)); }
        raw(std::string_view(length, sizeof(length))); raw(value);
    }
    void text(std::string_view value) { raw("s"); frame(value); }
    void flag(bool value) { const char encoded[] = {'b', static_cast<char>(value)}; raw(std::string_view(encoded, 2)); }
    void natural(uint64_t value) {
        char encoded[9] = {'u'};
        for (size_t i = 0; i < 8; ++i) { encoded[i + 1] = static_cast<char>(value >> (56 - i * 8)); }
        raw(std::string_view(encoded, sizeof(encoded)));
    }
    void decimal(uint64_t value) {
        char digits[20];
        const auto result = std::to_chars(digits, digits + sizeof(digits), value);
        if (result.ec != std::errc{}) { good = false; return; }
        text(std::string_view(digits, result.ptr - digits));
    }
    void collection(char kind, size_t count) {
        char encoded[5] = {kind};
        for (size_t i = 0; i < 4; ++i) { encoded[i + 1] = static_cast<char>(count >> (24 - i * 8)); }
        raw(std::string_view(encoded, sizeof(encoded)));
    }
    std::string finish() {
        unsigned char digest[CC_SHA256_DIGEST_LENGTH];
        if (CC_SHA256_Final(digest, &context) != 1 || !good) { return {}; }
        constexpr char hex[] = "0123456789abcdef";
        std::string out(sizeof(digest) * 2, '0');
        for (size_t i = 0; i < sizeof(digest); ++i) { out[2 * i] = hex[digest[i] >> 4]; out[2 * i + 1] = hex[digest[i] & 15]; }
        return out;
    }
};
void digest_names(GuardDigest &digest, const std::vector<std::string> &names) {
    digest.collection('a', names.size());
    for (const auto &name : names) { digest.text(name); }
}
void digest_levels(GuardDigest &digest, const std::map<std::string, uint64_t> &levels) {
    digest.collection('o', levels.size());
    for (const auto &entry : levels) { digest.frame(entry.first); digest.natural(entry.second); }
}
void digest_warnings(GuardDigest &digest, const OpeningEffective &effective) {
    digest.collection('o', 3);
    digest.frame("directory_rules"); digest_levels(digest, effective.directory_rules);
    digest.frame("enable"); digest.flag(effective.warnings_enable);
    digest.frame("levels"); digest_levels(digest, effective.warning_levels);
}
Value names_value(const std::vector<std::string> &names) {
    Value out = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (const auto &name : names) { append(out, string(name)); }
    return out;
}
Value levels_value(const std::map<std::string, uint64_t> &levels) {
    Value out = dict();
    for (const auto &entry : levels) { put(out, entry.first.c_str(), integer(static_cast<int64_t>(entry.second))); }
    return out;
}
Value warning_value(const OpeningEffective &e) {
    Value out = dict();
    put(out, "enable", boolean(e.warnings_enable)); put(out, "levels", levels_value(e.warning_levels));
    put(out, "directory_rules", levels_value(e.directory_rules)); return out;
}
const char *compiled(OpeningContext &c) {
    Value value;
    if (!checked_call(value, c.document.script_ptr, "Script", "is_tool", GAK_HASH_SCRIPT_TOOL) ||
            value.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "compiled_tool_unavailable"; }
    c.tool = truth(value);
    if (c.tool) { return "compiled_tool_context"; }
    if (!checked_call(value, c.document.script_ptr, "Script", "get_base_script", GAK_HASH_SCRIPT_BASE)) {
        return "compiled_base_unavailable";
    }
    if (value.type() != GDEXTENSION_VARIANT_TYPE_NIL &&
            (value.type() != GDEXTENSION_VARIANT_TYPE_OBJECT || object_ptr(value))) { return "compiled_script_base"; }
    Value properties, methods;
    int64_t pc = 0, mc = 0;
    if (!checked_call(properties, c.document.script_ptr, "Script", "get_script_property_list", GAK_HASH_SCRIPT_PROPERTIES) ||
            !count(properties, pc, 64) ||
            !checked_call(methods, c.document.script_ptr, "Script", "get_script_method_list", GAK_HASH_SCRIPT_METHODS) ||
            !count(methods, mc, 64)) { return "compiled_metadata_unavailable_or_limit"; }
    for (int64_t i = 0; i < pc; ++i) {
        Value p = at(properties, i);
        CompiledProperty property;
        if (!text(get(p, "name"), property.name, 256) || property.name.empty() ||
                !text(get(p, "hint_string"), property.hint_string, 2048) ||
                !text(get(p, "class_name"), property.class_name, 256) ||
                !natural(get(p, "type"), property.type) || property.type >= GDEXTENSION_VARIANT_TYPE_VARIANT_MAX ||
                !natural(get(p, "hint"), property.hint) || !natural(get(p, "usage"), property.usage)) {
            return "compiled_property_unavailable";
        }
        const bool display = (property.usage & (64 | 128 | 256)) != 0 && !(property.usage & 4096);
        if (!display && (property.type == GDEXTENSION_VARIANT_TYPE_OBJECT || property.hint != 0 ||
                !property.hint_string.empty() || !property.class_name.empty() ||
                ((property.usage & 4096) && (property.usage & 4)))) { return "unsafe_compiled_property"; }
        c.properties.push_back(std::move(property));
    }
    std::sort(c.properties.begin(), c.properties.end(), [](const auto &a, const auto &b) { return a.name < b.name; });
    for (size_t i = 1; i < c.properties.size(); ++i) {
        if (c.properties[i - 1].name == c.properties[i].name) { return "duplicate_compiled_property"; }
    }
    for (int64_t i = 0; i < mc; ++i) {
        Value m = at(methods, i);
        std::string name;
        if (!text(get(m, "name"), name, 256) || name.empty()) { return "compiled_method_unavailable"; }
        if (name == "_get" || name == "_set" || name == "_get_property_list") { return "unsafe_compiled_method"; }
        c.methods.push_back(std::move(name));
    }
    return ordered_names(c.methods) ? nullptr : "duplicate_compiled_method";
}
bool project(const Session &s, const std::string &request, const OpeningEffective &e, OpeningContext &c, bool emit) {
    GuardDigest digest;
    digest.frame("godot-agent-kit/open-context/v1");
    const size_t domain_bytes = digest.bytes;
    digest.collection('o', 26);
    digest.frame("autoloads"); digest_names(digest, e.autoloads);
    digest.frame("bindings"); digest.collection('a', c.bindings.size());
    for (const auto &binding : c.bindings) {
        digest.collection('o', 2);
        digest.frame("api_type"); digest.natural(binding.api_type);
        digest.frame("name"); digest.text(binding.name);
    }
    const uint64_t script_id = c.document.script_id, editor_id = c.document.editor_id, buffer_id = c.document.buffer_id;
    digest.frame("buffer_id"); digest.decimal(buffer_id);
    digest.frame("dirty"); digest.flag(c.dirty);
    digest.frame("editor_id"); digest.decimal(editor_id);
    digest.frame("external_editor"); digest.flag(e.external_editor);
    digest.frame("global_classes"); digest_names(digest, e.global_classes);
    digest.frame("has_redo"); digest.flag(c.redo);
    digest.frame("has_undo"); digest.flag(c.undo);
    digest.frame("methods"); digest_names(digest, c.methods);
    digest.frame("path"); digest.text(c.path);
    digest.frame("project_device"); digest.decimal(s.device);
    digest.frame("project_inode"); digest.decimal(s.inode);
    digest.frame("project_root"); digest.text(s.project_path);
    digest.frame("properties"); digest.collection('a', c.properties.size());
    for (const auto &property : c.properties) {
        digest.collection('o', 6);
        digest.frame("class_name"); digest.text(property.class_name);
        digest.frame("hint"); digest.natural(property.hint);
        digest.frame("hint_string"); digest.text(property.hint_string);
        digest.frame("name"); digest.text(property.name);
        digest.frame("type"); digest.natural(property.type);
        digest.frame("usage"); digest.natural(property.usage);
    }
    digest.frame("request_id"); digest.text(request);
    digest.frame("resource_edited"); digest.flag(c.edited);
    digest.frame("saved_version"); digest.decimal(c.saved_version);
    digest.frame("script_base_id"); digest.text("0");
    digest.frame("script_id"); digest.decimal(script_id);
    digest.frame("session_id"); digest.text(s.session_id);
    digest.frame("source_length"); digest.decimal(c.source.size());
    digest.frame("source_sha256"); digest.text(c.source_hash);
    digest.frame("tool"); digest.flag(c.tool);
    digest.frame("version"); digest.decimal(c.version);
    digest.frame("warnings"); digest_warnings(digest, e);
    c.record_bytes = digest.bytes - domain_bytes;
    if (c.record_bytes > 256 * 1024) { return false; }
    c.guard_hash = digest.finish();
    if (c.guard_hash.empty()) { return false; }
    if (!emit) { return true; }
    Value p = dict();
    auto str = [&](const char *key, const std::string &value) { put_string(p, key, value); };
    auto flag = [&](const char *key, bool value) { put(p, key, boolean(value)); };
    str("request_id", request); str("session_id", s.session_id); str("project_root", s.project_path);
    str("project_device", std::to_string(s.device)); str("project_inode", std::to_string(s.inode)); str("path", c.path);
    str("script_id", std::to_string(script_id)); str("editor_id", std::to_string(editor_id)); str("buffer_id", std::to_string(buffer_id));
    str("source_sha256", c.source_hash); str("source_length", std::to_string(c.source.size()));
    str("version", std::to_string(c.version)); str("saved_version", std::to_string(c.saved_version)); str("script_base_id", "0");
    flag("dirty", c.dirty); flag("resource_edited", c.edited); flag("has_undo", c.undo); flag("has_redo", c.redo);
    flag("tool", c.tool); flag("external_editor", e.external_editor);
    Value props = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (const auto &property : c.properties) {
        Value value = dict();
        put_string(value, "name", property.name); put_string(value, "hint_string", property.hint_string);
        put_string(value, "class_name", property.class_name); put(value, "type", integer(property.type));
        put(value, "hint", integer(property.hint)); put(value, "usage", integer(property.usage)); append(props, value);
    }
    put(p, "properties", props); put(p, "methods", names_value(c.methods)); put(p, "warnings", warning_value(e));
    put(p, "global_classes", names_value(e.global_classes)); put(p, "autoloads", names_value(e.autoloads));
    Value bindings = collection(GDEXTENSION_VARIANT_TYPE_ARRAY);
    for (const auto &binding : c.bindings) {
        Value value = dict(); put_string(value, "name", binding.name); put(value, "api_type", integer(binding.api_type)); append(bindings, value);
    }
    put(p, "bindings", bindings); c.projection = p;
    return true;
}
} // namespace

const char *opening_effective(OpeningEffective &e) {
    void *interface = singleton("EditorInterface"), *ps = singleton("ProjectSettings");
    if (!interface || !ps) { return "effective_context_unavailable"; }
    Value settings;
    if (!checked_call(settings, interface, "EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS)) {
        return "effective_context_unavailable";
    }
    Value external;
    if (!setting(object_ptr(settings), "EditorSettings", "get_setting", GAK_HASH_GET_SETTING,
            "text_editor/external/use_external_editor", external) || external.type() != GDEXTENSION_VARIANT_TYPE_BOOL) {
        return "external_editor_unavailable";
    }
    e.external_editor = truth(external);
    if (e.external_editor) { return "external_editor_enabled"; }
    const std::string prefix = "debug/gdscript/warnings/";
    Value enabled, directories, properties, classes;
    if (!setting(ps, "ProjectSettings", "get_setting_with_override", GAK_HASH_PROJECT_OVERRIDE, prefix + "enable", enabled) ||
            enabled.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            !setting(ps, "ProjectSettings", "get_setting_with_override", GAK_HASH_PROJECT_OVERRIDE, prefix + "directory_rules", directories) ||
            directories.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            !checked_call(properties, ps, "Object", "get_property_list", GAK_HASH_OBJECT_PROPERTIES) ||
            !checked_call(classes, ps, "ProjectSettings", "get_global_class_list", GAK_HASH_GLOBAL_CLASSES)) {
        return "effective_context_unavailable";
    }
    e.warnings_enable = truth(enabled);
    Value keys;
    int64_t count_keys = 0, count_props = 0, count_classes = 0;
    if (!checked_invoke(keys, directories, "keys") || !count(keys, count_keys, 32) ||
            !count(properties, count_props, 8192) || !count(classes, count_classes, 64)) { return "effective_context_limit"; }
    for (int64_t i = 0; i < count_keys; ++i) {
        std::string key;
        Value k = at(keys, i), decision;
        uint64_t level = 0;
        if (!checked_bytes(k, key, 2048) || key.compare(0, 6, "res://") || key.find('\0') != key.npos ||
                key.find("..") != key.npos || key.find_first_of("\"\\\n\r") != key.npos ||
                !checked_invoke(decision, directories, "get", {&k}) || !natural(decision, level) || level > 1 ||
                !e.directory_rules.emplace(key, level).second) { return "warning_rules_unavailable"; }
    }
    std::string property_name;
    for (int64_t i = 0; i < count_props; ++i) {
        Value property = at(properties, i);
        std::string &name = property_name;
        if (!text(get(property, "name"), name, 2048)) { return "effective_property_unavailable"; }
        if (name.compare(0, 9, "autoload/") == 0) {
            std::string autoload = name.substr(9), value;
            Value setting_value;
            if (autoload.empty() || autoload.size() > 256 || e.autoloads.size() >= 64 ||
                    !setting(ps, "ProjectSettings", "get_setting_with_override", GAK_HASH_PROJECT_OVERRIDE, name, setting_value) ||
                    !checked_bytes(setting_value, value, 2048) || value.empty()) { return "autoload_context_unavailable_or_limit"; }
            e.autoloads.push_back(std::move(autoload));
        }
        if (name.compare(0, prefix.size(), prefix)) { continue; }
        const std::string key = name.substr(prefix.size());
        if (key == "enable" || key == "directory_rules" || key == "exclude_addons" || key == "renamed_in_godot_4_hint" ||
                key == "property_used_as_function" || key == "constant_used_as_function" || key == "function_used_as_property" ||
                key.find('.') != key.npos) { continue; }
        Value value; uint64_t level = 0;
        if (key.empty() || key.size() > 128 || e.warning_levels.size() >= 64 ||
                !setting(ps, "ProjectSettings", "get_setting_with_override", GAK_HASH_PROJECT_OVERRIDE, name, value) ||
                !natural(value, level) || level > 2 || !e.warning_levels.emplace(key, level).second) { return "warning_context_unavailable_or_limit"; }
    }
    static constexpr const char *warnings[] = {
        "unassigned_variable", "unassigned_variable_op_assign", "unused_variable", "unused_local_constant",
        "unused_private_class_variable", "unused_parameter", "unused_signal", "shadowed_variable",
        "shadowed_variable_base_class", "shadowed_global_identifier", "unreachable_code", "unreachable_pattern",
        "standalone_expression", "standalone_ternary", "incompatible_ternary", "untyped_declaration",
        "inferred_declaration", "unsafe_property_access", "unsafe_method_access", "unsafe_cast",
        "unsafe_call_argument", "unsafe_void_return", "return_value_discarded", "static_called_on_instance",
        "missing_tool", "redundant_static_unload", "redundant_await", "missing_await", "assert_always_true",
        "assert_always_false", "integer_division", "narrowing_conversion", "int_as_enum_without_cast",
        "int_as_enum_without_match", "enum_variable_without_default", "empty_file", "deprecated_keyword",
        "confusable_identifier", "confusable_local_declaration", "confusable_local_usage",
        "confusable_capture_reassignment", "confusable_temporary_modification", "inference_on_variant",
        "native_method_override", "get_node_default_without_onready", "onready_with_export",
    };
    if (e.warning_levels.size() != std::size(warnings)) { return "warning_context_unavailable"; }
    for (const char *warning : warnings) {
        if (!e.warning_levels.count(warning)) { return "warning_context_unavailable"; }
    }
    for (int64_t i = 0; i < count_classes; ++i) {
        Value entry = at(classes, i);
        std::string name;
        if (!text(get(entry, "class"), name, 256) || name.empty()) { return "global_context_unavailable"; }
        e.global_classes.push_back(std::move(name));
    }
    if (!ordered_names(e.global_classes) || !ordered_names(e.autoloads)) { return "duplicate_effective_name"; }
    GuardDigest digest;
    digest.collection('o', 4);
    digest.frame("autoloads"); digest_names(digest, e.autoloads);
    digest.frame("external_editor"); digest.flag(e.external_editor);
    digest.frame("global_classes"); digest_names(digest, e.global_classes);
    digest.frame("warnings"); digest_warnings(digest, e);
    e.record_bytes = digest.bytes;
    if (e.record_bytes > 256 * 1024) { return "effective_context_limit"; }
    e.fingerprint = digest.finish();
    if (e.fingerprint.empty()) { return "effective_context_unavailable"; }
    return nullptr;
}

const char *opening_source_profile(std::string_view source, const OpeningEffective &e, std::vector<NativeBinding> &bindings) {
    if (source.size() > SOURCE_LIMIT || !valid_utf8(source) || source.find("\xEF\xBB\xBF") != source.npos) { return "unsupported_source"; }
    for (size_t i = 0; i < source.size(); ++i) {
        const auto c = static_cast<unsigned char>(source[i]);
        if ((c < 32 && c != '\n' && c != '\t') || c == 127 ||
                (c == 0xC2 && i + 1 < source.size() && static_cast<unsigned char>(source[i + 1]) >= 0x80 &&
                static_cast<unsigned char>(source[i + 1]) <= 0x9F)) { return "unsupported_source"; }
    }
    std::set<std::string> identifiers;
    std::set<std::string> bases;
    bool needs_base = false;
    auto alpha = [](unsigned char c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'; };
    for (size_t i = 0; i < source.size();) {
        unsigned char c = source[i];
        if (c == '#') { i = source.find('\n', i); if (i == source.npos) { break; } continue; }
        if (c == ' ' || c == '\t') { ++i; continue; }
        if (c == '\n') { if (needs_base) { return "unsupported_inheritance"; } ++i; continue; }
        if (c == '@' || c >= 128) { return "unsafe_source_profile"; }
        if (c == '\'' || c == '"') {
            if (needs_base) { return "unsupported_inheritance"; }
            const char quote = source[i++];
            const bool triple = i + 1 < source.size() && source[i] == quote && source[i + 1] == quote;
            if (triple) { i += 2; }
            bool closed = false;
            while (i < source.size()) {
                if (source[i] == '\\') { if (++i >= source.size()) { return "unsupported_lexical_form"; } ++i; continue; }
                if (!triple && source[i] == '\n') { return "unsupported_lexical_form"; }
                if (source[i] == quote && (!triple || (i + 2 < source.size() && source[i + 1] == quote && source[i + 2] == quote))) {
                    i += triple ? 3 : 1; closed = true; break;
                }
                ++i;
            }
            if (!closed) { return "unsupported_lexical_form"; }
            continue;
        }
        if (!alpha(c)) { if (needs_base) { return "unsupported_inheritance"; } ++i; continue; }
        const size_t start = i++;
        while (i < source.size() && (alpha(source[i]) || (source[i] >= '0' && source[i] <= '9'))) { ++i; }
        const std::string token(source.substr(start, i - start));
        if (token.size() > 256 || (identifiers.emplace(token).second && identifiers.size() > 256)) { return "source_identifier_limit"; }
        if ((token == "r" || token == "R") && i < source.size() && (source[i] == '\'' || source[i] == '"')) {
            return "unsupported_lexical_form";
        }
        if (token == "class_name" || token == "static" || token == "const" || token == "load" || token == "preload" ||
                token == "_get" || token == "_set" || token == "_get_property_list") { return "unsafe_source_profile"; }
        if (std::binary_search(e.global_classes.begin(), e.global_classes.end(), token) ||
                std::binary_search(e.autoloads.begin(), e.autoloads.end(), token)) { return "project_source_dependency"; }
        if (needs_base) {
            bases.emplace(token);
            size_t next = i;
            while (next < source.size() && (source[next] == ' ' || source[next] == '\t')) { ++next; }
            if (next < source.size() && source[next] != '\n' && source[next] != '#') { return "unsupported_inheritance"; }
        }
        needs_base = token == "extends";
    }
    if (needs_base) { return "unsupported_inheritance"; }
    for (const auto &identifier : identifiers) {
        Value key = string(identifier), exists, category;
        void *db = singleton("ClassDB");
        if (!checked_call(exists, db, "ClassDB", "class_exists", GAK_HASH_CLASS_EXISTS, {&key}) ||
                exists.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "class_binding_unavailable"; }
        if (truth(exists)) {
            uint64_t api_type = 0;
            if (!checked_call(category, db, "ClassDB", "class_get_api_type", GAK_HASH_CLASS_API, {&key}) ||
                    !natural(category, api_type)) { return "class_binding_unavailable"; }
            if (api_type != 0) { return "unsupported_class_binding"; }
            bindings.push_back({identifier, api_type});
        } else if (bases.count(identifier)) { return "unsupported_inheritance"; }
    }
    return nullptr;
}

const char *opening_path_guard(const Session &session, const FileBinding &file) {
    const int parent = file.parents.empty() ? session.project_fd : file.parents.back().fd.value;
    struct stat remap{};
    const std::string companion = file.name + ".remap";
    if (!fstatat(parent, companion.c_str(), &remap, AT_SYMLINK_NOFOLLOW)) { return "unsupported_remapped_path"; }
    return errno == ENOENT || errno == ENAMETOOLONG ? nullptr : "remap_state_unavailable";
}
const char *opening_context(const Session &session, const std::string &request, const OpeningEffective &e,
        OpeningContext &out, const OpeningContext *retained, bool after_open) {
    Value owner, current_editor, current_script;
    if (!checked_call(owner, singleton("EditorInterface"), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
            !object_ptr(owner) ||
            !checked_call(current_editor, object_ptr(owner), "ScriptEditor", "get_current_editor", GAK_HASH_CURRENT_EDITOR) ||
            !checked_call(current_script, object_ptr(owner), "ScriptEditor", "get_current_script", GAK_HASH_CURRENT_SCRIPT)) {
        return "current_context_unavailable";
    }
    if (after_open && retained && retained->kind == OpeningContext::NoSource) {
        out.kind = OpeningContext::NoSource; return nullptr;
    }
    Value script = after_open && retained ? retained->document.script : current_script;
    if (!object_ptr(script)) {
        if (object_ptr(current_editor) || (current_editor.type() != GDEXTENSION_VARIANT_TYPE_NIL &&
                current_editor.type() != GDEXTENSION_VARIANT_TYPE_OBJECT) ||
                (current_script.type() != GDEXTENSION_VARIANT_TYPE_NIL &&
                current_script.type() != GDEXTENSION_VARIANT_TYPE_OBJECT)) { return "unsupported_current_editor"; }
        out.kind = OpeningContext::NoSource; return nullptr;
    }
    Name gd("GDScript");
    if (!api.class_tag(gd.ptr()) || !api.cast_to(object_ptr(script), api.class_tag(gd.ptr()))) { return "unsupported_current_script"; }
    Value path;
    if (!checked_call(path, object_ptr(script), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
            !checked_bytes(path, out.path, 2048) || !path_ok(out.path)) { return "unsupported_current_path"; }
    if (after_open && retained) {
        if (!resolve(out.path, id(script), retained->document.editor_id, retained->document.buffer_id, out.document)) {
            return "current_document_association_changed";
        }
    } else {
        bool found = false;
        if (!find_document(out.path, out.document, found) || !found ||
                out.document.script_id != id(script) || out.document.editor_id != id(current_editor)) {
            return "current_document_association_unavailable";
        }
    }
    Value r, b, version, saved, files, dirty, edited, undo, redo;
    Value current_path = string(out.path);
    if (!checked_call(files, object_ptr(owner), "ScriptEditor", "get_unsaved_files", GAK_HASH_UNSAVED) ||
            files.type() != GDEXTENSION_VARIANT_TYPE_PACKED_STRING_ARRAY ||
            !checked_invoke(dirty, files, "has", {&current_path}) || dirty.type() != GDEXTENSION_VARIANT_TYPE_BOOL) {
        return "current_dirty_unavailable";
    }
    std::string resource, visible;
    if (!checked_call(r, out.document.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE) ||
            !checked_bytes(r, resource, SOURCE_LIMIT) ||
            !checked_call(b, out.document.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT) ||
            !checked_bytes(b, visible, SOURCE_LIMIT)) { return "current_source_unavailable"; }
    if (resource != visible) { return "current_source_conflict"; }
    out.source = std::move(resource); out.source_hash = sha256(out.source);
    if (!checked_call(version, out.document.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION) || !natural(version, out.version) ||
            !checked_call(saved, out.document.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION) || !natural(saved, out.saved_version) ||
            !checked_call(edited, singleton("EditorInterface"), "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&out.document.script}) || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            !checked_call(undo, out.document.buffer_ptr, "TextEdit", "has_undo", GAK_HASH_HAS_UNDO) || undo.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            !checked_call(redo, out.document.buffer_ptr, "TextEdit", "has_redo", GAK_HASH_HAS_REDO) || redo.type() != GDEXTENSION_VARIANT_TYPE_BOOL) {
        return "current_protection_unavailable";
    }
    out.dirty = truth(dirty); out.edited = truth(edited); out.undo = truth(undo); out.redo = truth(redo);
    if (retained && retained->kind == OpeningContext::Current) {
        if (!file_attached(session, retained->file)) { return "current_namespace_changed"; }
    } else if (const char *reason = pin_file(session, out.path, out.file, O_RDONLY)) { return reason; }
    if (const char *reason = opening_path_guard(session,
            retained && retained->kind == OpeningContext::Current ? retained->file : out.file)) { return reason; }
    if (const char *reason = opening_source_profile(out.source, e, out.bindings)) { return reason; }
    if (const char *reason = compiled(out)) { return reason; }
    if (!project(session, request, e, out, retained == nullptr)) { return "current_context_limit"; }
    out.kind = OpeningContext::Current;
    return nullptr;
}

bool opening_bindings_equal(const std::vector<NativeBinding> &a, const std::vector<NativeBinding> &b) {
    return a.size() == b.size() && std::equal(a.begin(), a.end(), b.begin(), [](const auto &x, const auto &y) {
        return x.name == y.name && x.api_type == y.api_type;
    });
}
Value opening_context_reply(const OpeningContext &context) {
    Value out = dict();
    put_string(out, "kind", context.kind == OpeningContext::Current ? "current_gdscript" :
            context.kind == OpeningContext::NoSource ? "no_source_editor" : "unavailable");
    put(out, "reason", Value()); put(out, "projection", context.projection);
    put(out, "source", context.kind == OpeningContext::Current ? string(context.source) : Value());
    put(out, "sha256", context.kind == OpeningContext::Current ? string(context.guard_hash) : Value()); return out;
}
} // namespace gak
