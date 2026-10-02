#include "open_context.hpp"

namespace gak {
const char *opening_context(const Session &session, const std::string &request, const OpeningEffective &effective,
        OpeningContext &out, const OpeningContext *retained, bool after_open) {
    Name key("EditorInterface");
    Value owner, current_editor, current_script;
    if (!checked_call(owner, api.singleton(key.ptr()), "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR) ||
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
    Value path_value;
    std::string path;
    if (!checked_call(path_value, object_ptr(script), "Resource", "get_path", GAK_HASH_RESOURCE_PATH) ||
            !checked_bytes(path_value, path, 2048) || !path_ok(path)) { return "unsupported_current_path"; }
    Binding document;
    if (after_open && retained) {
        if (!resolve(path, id(script), retained->document.editor_id, retained->document.buffer_id, document)) {
            return "current_document_association_changed";
        }
    } else {
        bool found = false;
        if (!find_document(path, document, found) || !found || document.script_id != id(script) ||
                document.editor_id != id(current_editor)) { return "current_document_association_unavailable"; }
    }
    return editor_context(session, request, effective, document, out, retained);
}
} // namespace gak
