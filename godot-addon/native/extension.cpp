#include "native.hpp"

#include <cstring>
#include <cstdlib>
#include <thread>

namespace gak {
Api api;
void *library = nullptr;
namespace {
Session session;
bool registered = false;
#if GAK_FIXTURE
GDExtensionInterfaceClassdbRegisterExtensionClass6 register_fixture_class{};
GDExtensionInterfaceClassdbUnregisterExtensionClass unregister_fixture_class{};
bool fixture_class_registered = false;
#endif

template <typename T> bool load(T &destination, GDExtensionInterfaceGetProcAddress proc, const char *name) {
    destination = reinterpret_cast<T>(proc(name));
    return destination != nullptr;
}

bool supported_engine() {
    GDExtensionGodotVersion2 v{};
    api.version(&v);
    return v.major == 4 && v.minor == 7 && v.patch == 2 && v.hash &&
            std::strcmp(v.hash, GAK_ENGINE_HASH) == 0 && v.status && std::strcmp(v.status, "stable") == 0 &&
            v.build && std::strcmp(v.build, GAK_ENGINE_BUILD) == 0;
}

bool engine_revision() {
    Name editor("EditorInterface"), script("Script"), text("TextEdit"), base("ScriptEditorBase");
    Name setter("set_object_edited"), source("set_source_code"), saved("tag_saved_version"), buffer("get_base_editor");
    if (!api.bind(editor.ptr(), setter.ptr(), GAK_HASH_SET_EDITED) ||
            !api.bind(script.ptr(), source.ptr(), GAK_HASH_SCRIPT_SET_SOURCE) ||
            !api.bind(text.ptr(), saved.ptr(), GAK_HASH_TAG_SAVED) ||
            !api.bind(base.ptr(), buffer.ptr(), GAK_HASH_BASE_EDITOR)) { return false; }
    struct Required { const char *cls; const char *method; uint64_t hash; };
    const Required methods[] = {
        {"ClassDB", "instantiate", GAK_HASH_INSTANTIATE},
        {"ClassDB", "class_exists", GAK_HASH_CLASS_EXISTS},
        {"ClassDB", "class_get_api_type", GAK_HASH_CLASS_API},
        {"Resource", "set_path", GAK_HASH_RESOURCE_SET_PATH},
        {"Resource", "get_path", GAK_HASH_RESOURCE_PATH},
        {"Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE},
        {"Script", "reload", GAK_HASH_SCRIPT_RELOAD},
        {"Script", "is_tool", GAK_HASH_SCRIPT_TOOL},
        {"Script", "get_base_script", GAK_HASH_SCRIPT_BASE},
        {"Script", "get_script_property_list", GAK_HASH_SCRIPT_PROPERTIES},
        {"Script", "get_script_method_list", GAK_HASH_SCRIPT_METHODS},
        {"ResourceLoader", "has_cached", GAK_HASH_HAS_CACHED},
        {"ResourceLoader", "get_cached_ref", GAK_HASH_CACHED_REF},
        {"EditorInterface", "edit_script", GAK_HASH_EDIT_SCRIPT},
        {"EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS},
        {"EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED},
        {"EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR},
        {"ScriptEditor", "get_current_editor", GAK_HASH_CURRENT_EDITOR},
        {"ScriptEditor", "get_current_script", GAK_HASH_CURRENT_SCRIPT},
        {"ScriptEditor", "get_open_scripts", GAK_HASH_OPEN_SCRIPTS},
        {"ScriptEditor", "get_open_script_editors", GAK_HASH_OPEN_EDITORS},
        {"ScriptEditor", "get_unsaved_files", GAK_HASH_UNSAVED},
        {"TextEdit", "get_text", GAK_HASH_TEXT},
        {"TextEdit", "get_version", GAK_HASH_CURRENT_VERSION},
        {"TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION},
        {"TextEdit", "has_undo", GAK_HASH_HAS_UNDO},
        {"TextEdit", "has_redo", GAK_HASH_HAS_REDO},
        {"EditorSettings", "get_setting", GAK_HASH_GET_SETTING},
        {"ProjectSettings", "get_setting_with_override", GAK_HASH_PROJECT_OVERRIDE},
        {"ProjectSettings", "globalize_path", GAK_HASH_GLOBALIZE_PATH},
        {"ProjectSettings", "get_global_class_list", GAK_HASH_GLOBAL_CLASSES},
        {"Object", "get_property_list", GAK_HASH_OBJECT_PROPERTIES},
    };
    for (const auto &required : methods) {
        Name cls(required.cls), method(required.method);
        if (!api.bind(cls.ptr(), method.ptr(), required.hash)) { return false; }
    }
    return true;
}

enum class Action { Configure, Close, Revision, BuildId, Inspect, Prepare, Advance, Cancel, Expire,
    OpenInspect, OpenPrepare, OpenAdvance, OpenVerify, OpenRecheck, OpenFinish, OpenAbort, OpenExpire,
#if GAK_FIXTURE
    FixtureFault, OpenFixtureFault, OpenFixtureState,
#endif
};

void native_callback(void *userdata, const GDExtensionConstVariantPtr *arguments, GDExtensionInt count,
        GDExtensionVariantPtr destination, GDExtensionCallError *error) {
    error->error = GDEXTENSION_CALL_OK;
    const Action action = *static_cast<Action *>(userdata);
    if (action == Action::BuildId) {
        if (count == 0) { copy_into(destination, string(GAK_BUILD_ID)); }
        return;
    }
    if (action == Action::Revision) {
        if (count == 0 && std::this_thread::get_id() == session.main_thread && engine_revision()) {
            copy_into(destination, integer(2));
        } else { copy_into(destination, integer(0)); }
        return;
    }
    if (action == Action::Close) {
        if (count == 0 && std::this_thread::get_id() == session.main_thread) { close(session); }
        return;
    }
    if (action == Action::Configure) {
        bool ok = false;
        if (count == 1 && api.type(arguments[0]) == GDEXTENSION_VARIANT_TYPE_STRING &&
                std::this_thread::get_id() == session.main_thread && engine_revision()) {
            Value id(arguments[0]);
            ok = configure(session, bytes(id));
        }
        copy_into(destination, boolean(ok));
        return;
    }
    if (action == Action::Inspect) {
        Value result;
        if (count == 4 && std::this_thread::get_id() == session.main_thread) {
            Value path(arguments[0]), original(arguments[1]), desired(arguments[2]), document(arguments[3]);
            result = edit_inspect(session, path, original, desired, document);
        }
        copy_into(destination, result);
        return;
    }
    if (action == Action::Prepare) {
        Value result;
        if (count == 4 && std::this_thread::get_id() == session.main_thread) {
            Value path(arguments[0]), expected(arguments[1]), desired(arguments[2]), correlation(arguments[3]);
            result = edit_prepare(session, path, expected, desired, correlation);
        }
        copy_into(destination, result);
        return;
    }
    if (action == Action::Advance || action == Action::Cancel) {
        Value result;
        if (std::this_thread::get_id() == session.main_thread &&
                count == (action == Action::Advance ? 2 : 1)) {
            Value request(arguments[0]);
            if (action == Action::Advance) {
                Value stage(arguments[1]);
                result = edit_advance(session, request, stage);
            } else { result = edit_cancel(session, request); }
        }
        copy_into(destination, result);
        return;
    }
    if (action == Action::Expire) {
        if (count == 0 && std::this_thread::get_id() == session.main_thread) { edit_expire(session); }
        return;
    }
    if (action == Action::OpenExpire) {
        if (count == 0 && std::this_thread::get_id() == session.main_thread) { open_expire(session); }
        return;
    }
    if (action == Action::OpenInspect || action == Action::OpenPrepare || action == Action::OpenAdvance ||
            action == Action::OpenVerify || action == Action::OpenRecheck || action == Action::OpenFinish ||
            action == Action::OpenAbort) {
        Value result;
        if (std::this_thread::get_id() == session.main_thread) {
            if (action == Action::OpenInspect && count == 2) {
                Value path(arguments[0]), correlation(arguments[1]); result = open_inspect(session, path, correlation);
            } else if (action == Action::OpenPrepare && count == 3) {
                Value request(arguments[0]), source(arguments[1]), capture(arguments[2]); result = open_prepare(session, request, source, capture);
            } else if (action == Action::OpenAdvance && count == 4) {
                Value request(arguments[0]), stage(arguments[1]), source(arguments[2]), context(arguments[3]);
                result = open_advance(session, request, stage, source, context);
            } else if ((action == Action::OpenVerify || action == Action::OpenRecheck) && count == 2) {
                Value request(arguments[0]), purpose(arguments[1]);
                result = action == Action::OpenVerify ? open_verify(session, request, purpose) : open_recheck(session, request, purpose);
            } else if ((action == Action::OpenFinish || action == Action::OpenAbort) && count == 1) {
                Value request(arguments[0]);
                result = action == Action::OpenFinish ? open_finish(session, request) : open_abort(session, request);
            }
        }
        copy_into(destination, result); return;
    }
#if GAK_FIXTURE
    if (action == Action::FixtureFault) {
        Value result;
        if (count == 2 && std::this_thread::get_id() == session.main_thread) {
            Value request(arguments[0]), fault(arguments[1]);
            result = edit_fixture_fault(session, request, fault);
        }
        copy_into(destination, result);
    }
    if (action == Action::OpenFixtureFault || action == Action::OpenFixtureState) {
        Value result;
        if (std::this_thread::get_id() == session.main_thread) {
            if (action == Action::OpenFixtureFault && count == 2) {
                Value request(arguments[0]), fault(arguments[1]); result = open_fixture_fault(session, request, fault);
            } else if (action == Action::OpenFixtureState && count == 1) {
                Value request(arguments[0]); result = open_fixture_state(session, request);
            }
        }
        copy_into(destination, result);
    }
#endif
}

Action configure_action = Action::Configure, close_action = Action::Close;
Action revision_action = Action::Revision, build_action = Action::BuildId;
Action inspect_action = Action::Inspect, prepare_action = Action::Prepare, advance_action = Action::Advance, cancel_action = Action::Cancel;
Action expire_action = Action::Expire;
Action open_inspect_action = Action::OpenInspect, open_prepare_action = Action::OpenPrepare;
Action open_advance_action = Action::OpenAdvance, open_verify_action = Action::OpenVerify, open_recheck_action = Action::OpenRecheck;
Action open_finish_action = Action::OpenFinish, open_abort_action = Action::OpenAbort, open_expire_action = Action::OpenExpire;
#if GAK_FIXTURE
Action fault_action = Action::FixtureFault;
Action open_fault_action = Action::OpenFixtureFault, open_state_action = Action::OpenFixtureState;
#endif

Value callable(Action &action) {
    Storage<GAK_SIZE_CALLABLE> storage;
    GDExtensionCallableCustomInfo2 info{};
    info.callable_userdata = &action;
    info.token = library;
    info.call_func = native_callback;
    api.new_callable(storage.ptr(), &info);
    Value value = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr());
    api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    return value;
}

void *engine_singleton() {
    Name name("Engine");
    return api.singleton(name.ptr());
}

void editor_initialize(void *, GDExtensionInitializationLevel level) {
    if (level != GDEXTENSION_INITIALIZATION_EDITOR || !supported_engine() || !engine_binary_matches()) { return; }
    session.main_thread = std::this_thread::get_id();
#if GAK_FIXTURE
    Name fixture_class("GodotAgentKitOpeningFixture"), parent("RefCounted");
    GDExtensionClassCreationInfo6 info{};
    info.is_abstract = true; info.is_exposed = true;
    info.free_instance_func = [](void *, GDExtensionClassInstancePtr instance) { std::free(instance); };
    register_fixture_class(library, fixture_class.ptr(), parent.ptr(), &info);
    fixture_class_registered = true;
#endif
    void *engine = engine_singleton();
    if (!engine) { return; }
    Value key = name("godot_agent_kit_native");
    Value found = call(engine, "Object", "has_meta", 2619796661ULL, {&key});
    if (found.type() != GDEXTENSION_VARIANT_TYPE_BOOL || truth(found)) { return; }
    Value metadata = dict();
    put(metadata, "configure", callable(configure_action));
    put(metadata, "close", callable(close_action));
    put(metadata, "api_revision", callable(revision_action));
    put(metadata, "build_id", callable(build_action));
    put(metadata, "edit_inspect", callable(inspect_action));
    put(metadata, "edit_prepare", callable(prepare_action));
    put(metadata, "edit_advance", callable(advance_action));
    put(metadata, "edit_cancel", callable(cancel_action));
    put(metadata, "edit_expire", callable(expire_action));
    put(metadata, "open_inspect", callable(open_inspect_action));
    put(metadata, "open_prepare", callable(open_prepare_action));
    put(metadata, "open_advance", callable(open_advance_action));
    put(metadata, "open_verify", callable(open_verify_action));
    put(metadata, "open_recheck", callable(open_recheck_action));
    put(metadata, "open_finish", callable(open_finish_action));
    put(metadata, "open_abort", callable(open_abort_action));
    put(metadata, "open_expire", callable(open_expire_action));
#if GAK_FIXTURE
    put(metadata, "edit_fixture_fault", callable(fault_action));
    put(metadata, "open_fixture_fault", callable(open_fault_action));
    put(metadata, "open_fixture_state", callable(open_state_action));
#endif
    Value done = call(engine, "Object", "set_meta", 3776071444ULL, {&key, &metadata});
    (void)done;
    registered = true;
}

void editor_deinitialize(void *, GDExtensionInitializationLevel level) {
    if (level != GDEXTENSION_INITIALIZATION_EDITOR) { return; }
    close(session);
    if (registered) {
        void *engine = engine_singleton();
        if (engine) {
            Value key = name("godot_agent_kit_native");
            Value removed = call(engine, "Object", "remove_meta", 3304788590ULL, {&key});
            (void)removed;
        }
        registered = false;
    }
#if GAK_FIXTURE
    if (fixture_class_registered) {
        Name fixture_class("GodotAgentKitOpeningFixture");
        unregister_fixture_class(library, fixture_class.ptr());
        fixture_class_registered = false;
    }
#endif
}
} // namespace
} // namespace gak

extern "C" __attribute__((visibility("default"))) GDExtensionBool editor_integration_library_init(
        GDExtensionInterfaceGetProcAddress proc, GDExtensionClassLibraryPtr library,
        GDExtensionInitialization *initialization) {
    using namespace gak;
    api = Api{};
    gak::library = library;
    if (!load(api.version, proc, "get_godot_version2") ||
            !load(api.new_nil, proc, "variant_new_nil") ||
            !load(api.new_copy, proc, "variant_new_copy") ||
            !load(api.destroy, proc, "variant_destroy") ||
            !load(api.type, proc, "variant_get_type") ||
            !load(api.variant_call, proc, "variant_call") ||
            !load(api.construct, proc, "variant_construct") ||
            !load(api.get_from, proc, "get_variant_from_type_constructor") ||
            !load(api.get_to, proc, "get_variant_to_type_constructor") ||
            !load(api.get_internal, proc, "variant_get_ptr_internal_getter") ||
            !load(api.ptr_destructor, proc, "variant_get_ptr_destructor") ||
            !load(api.new_string, proc, "string_new_with_utf8_chars_and_len") ||
            !load(api.string_utf8, proc, "string_to_utf8_chars") ||
            !load(api.new_name, proc, "string_name_new_with_utf8_chars") ||
            !load(api.dictionary_index, proc, "dictionary_operator_index") ||
            !load(api.dictionary_index_const, proc, "dictionary_operator_index_const") ||
            !load(api.has_key, proc, "variant_has_key") ||
            !load(api.array_index_const, proc, "array_operator_index_const") ||
            !load(api.new_callable, proc, "callable_custom_create2") ||
            !load(api.singleton, proc, "global_get_singleton") ||
            !load(api.bind, proc, "classdb_get_method_bind") ||
            !load(api.bound_call, proc, "object_method_bind_call") ||
            !load(api.class_tag, proc, "classdb_get_class_tag") ||
            !load(api.cast_to, proc, "object_cast_to") ||
            !load(api.instance_id, proc, "object_get_instance_id") ||
            !load(api.from_id, proc, "object_get_instance_from_id")) { return 0; }
#if GAK_FIXTURE
    if (!load(register_fixture_class, proc, "classdb_register_extension_class6") ||
            !load(unregister_fixture_class, proc, "classdb_unregister_extension_class")) { return 0; }
#endif
    for (const auto kind : {GDEXTENSION_VARIANT_TYPE_BOOL, GDEXTENSION_VARIANT_TYPE_INT,
            GDEXTENSION_VARIANT_TYPE_STRING, GDEXTENSION_VARIANT_TYPE_STRING_NAME,
            GDEXTENSION_VARIANT_TYPE_OBJECT, GDEXTENSION_VARIANT_TYPE_CALLABLE,
            GDEXTENSION_VARIANT_TYPE_DICTIONARY, GDEXTENSION_VARIANT_TYPE_ARRAY}) {
        api.from[kind] = api.get_from(kind);
        api.to[kind] = api.get_to(kind);
        api.internal[kind] = api.get_internal(kind);
        if (!api.from[kind] || !api.to[kind] || !api.internal[kind]) { return 0; }
    }
    for (auto kind : {GDEXTENSION_VARIANT_TYPE_STRING, GDEXTENSION_VARIANT_TYPE_STRING_NAME,
            GDEXTENSION_VARIANT_TYPE_CALLABLE, GDEXTENSION_VARIANT_TYPE_DICTIONARY,
            GDEXTENSION_VARIANT_TYPE_ARRAY}) {
        if (!api.ptr_destructor(kind)) { return 0; }
    }
    initialization->minimum_initialization_level = GDEXTENSION_INITIALIZATION_EDITOR;
    initialization->userdata = nullptr;
    initialization->initialize = editor_initialize;
    initialization->deinitialize = editor_deinitialize;
    return 1;
}
