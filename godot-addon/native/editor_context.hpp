#pragma once

#include "document_guard.hpp"
#include <map>
#include <vector>

namespace gak {
struct NativeBinding { std::string name; uint64_t api_type{}; };
struct CompiledProperty {
    std::string name, hint_string, class_name;
    uint64_t type{}, hint{}, usage{};
};
struct OpeningEffective {
    bool external_editor{}, warnings_enable{};
    std::map<std::string, uint64_t> warning_levels, directory_rules;
    std::vector<std::string> global_classes, autoloads;
    std::string fingerprint;
    size_t record_bytes{};
};
struct OpeningContext {
    enum Kind { NoSource, Current, Unavailable } kind = Unavailable;
    Binding document;
    FileBinding file;
    std::string path, source, source_hash, guard_hash;
    uint64_t version{}, saved_version{};
    bool dirty{}, edited{}, undo{}, redo{}, tool{};
    std::vector<CompiledProperty> properties;
    std::vector<std::string> methods;
    std::vector<NativeBinding> bindings;
    Value projection;
    size_t record_bytes{};
};
const char *opening_effective(OpeningEffective &out);
const char *opening_path_guard(const Session &session, const FileBinding &file);
const char *opening_source_profile(std::string_view source, const OpeningEffective &effective,
        std::vector<NativeBinding> &bindings);
const char *editor_context(const Session &session, const std::string &request,
        const OpeningEffective &effective, const Binding &document, OpeningContext &out,
        const OpeningContext *retained = nullptr, size_t metadata_budget = 256 * 1024,
        bool emit_projection = true);
bool opening_bindings_equal(const std::vector<NativeBinding> &left, const std::vector<NativeBinding> &right);
Value opening_context_reply(const OpeningContext &context);
} // namespace gak
