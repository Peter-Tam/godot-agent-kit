// Owned acceptance-only GDExtension. Never installed in the product addon or export.
#include "native.hpp"

#include <CommonCrypto/CommonDigest.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

#include <cerrno>
#include <chrono>
#include <string>
#include <thread>

namespace gak { Api api; void *library = nullptr; }
namespace {
using namespace gak;
struct Context {
    const Value *source;
    const Value *path;
    const Value *correlation;
    const Value *reader;
    int root_fd;
    bool nested_called = false;
    int dependency_calls = 0;
    std::string control_path;
    bool barrier_reached = false;
    Value nested;
};

std::string hex(const unsigned char *bytes, size_t n) {
    static constexpr char digits[] = "0123456789abcdef";
    std::string result(n * 2, '\0');
    for (size_t i = 0; i < n; ++i) {
        result[2 * i] = digits[bytes[i] >> 4];
        result[2 * i + 1] = digits[bytes[i] & 15];
    }
    return result;
}

Value read_source(Context &ctx, const Value &path) {
    Value response = dict();
    put_string(response, "status", "unavailable");
    put_string(response, "reason", "fixture_confined_read_failed");
    const std::string requested = bytes(path, 2054);
    const bool remap = requested == "res://scripts/native/level_one.gd.remap";
    if ((!remap && requested != "res://scripts/native/level_one.gd") || ctx.root_fd < 0) {
        return response;
    }
    int scripts = openat(ctx.root_fd, "scripts", O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    if (scripts < 0) { return response; }
    int native = openat(scripts, "native", O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    close(scripts);
    if (native < 0) { return response; }
    int file = openat(native, remap ? "level_one.gd.remap" : "level_one.gd",
            O_RDONLY | O_NOFOLLOW | O_CLOEXEC);
    const int open_error = errno;
    close(native);
    if (file < 0) {
        if (open_error == ENOENT) {
            put_string(response, "status", "missing");
            put_string(response, "reason", "not_found");
        }
        return response;
    }
    if (remap) {
        close(file);
        put_string(response, "reason", "unsupported_remap");
        return response;
    }
    struct stat before{}, after{};
    bool valid = fstat(file, &before) == 0 && S_ISREG(before.st_mode) &&
            before.st_size >= 0 && before.st_size <= 512 * 1024;
    std::string source;
    if (valid) {
        source.resize(static_cast<size_t>(before.st_size));
        size_t offset = 0;
        while (offset < source.size()) {
            ssize_t n = ::read(file, source.data() + offset, source.size() - offset);
            if (n <= 0) { valid = false; break; }
            offset += static_cast<size_t>(n);
        }
        char extra{};
        valid = valid && ::read(file, &extra, 1) == 0 && fstat(file, &after) == 0 &&
                before.st_dev == after.st_dev && before.st_ino == after.st_ino &&
                before.st_size == after.st_size && before.st_mtimespec.tv_sec == after.st_mtimespec.tv_sec &&
                before.st_mtimespec.tv_nsec == after.st_mtimespec.tv_nsec;
    }
    close(file);
    if (!valid) { return response; }
    unsigned char digest[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256(source.data(), static_cast<CC_LONG>(source.size()), digest);
    put_string(response, "status", "ok");
    put(response, "reason", Value());
    put_string(response, "source", source);
    put_string(response, "source_sha256", hex(digest, sizeof(digest)));
    put(response, "utf8_bytes", integer(static_cast<int64_t>(source.size())));
    Value identity = dict();
    put_string(identity, "device", std::to_string(before.st_dev));
    put_string(identity, "inode", std::to_string(before.st_ino));
    put(response, "identity", identity);
    return response;
}

bool hold_first_read(Context &ctx) {
    if (ctx.control_path.empty() || ctx.barrier_reached) { return true; }
    ctx.barrier_reached = true;
    const int directory = open(ctx.control_path.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    if (directory < 0) { return false; }
    const int event = openat(directory, "fixture_dependency_read",
            O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
    if (event < 0) { close(directory); return false; }
    close(event);
    const auto until = std::chrono::steady_clock::now() + std::chrono::seconds(8);
    bool released = false;
    while (std::chrono::steady_clock::now() < until) {
        struct stat current{};
        if (fstatat(directory, "fixture_dependency_release", &current, AT_SYMLINK_NOFOLLOW) == 0 &&
                S_ISREG(current.st_mode)) {
            released = true;
            break;
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    close(directory);
    return released;
}

void reader_callback(void *userdata, const GDExtensionConstVariantPtr *arguments, GDExtensionInt count,
        GDExtensionVariantPtr destination, GDExtensionCallError *error) {
    error->error = GDEXTENSION_CALL_OK;
    auto &ctx = *static_cast<Context *>(userdata);
    ++ctx.dependency_calls;
    if (!ctx.nested_called) {
        ctx.nested_called = true;
        Value attempted = call(nullptr, "GDScript", "validate_gdscript_source", GAK_HASH_GDSCRIPT_VALIDATE,
                {ctx.source, ctx.path, ctx.reader, ctx.correlation});
        ctx.nested = attempted;
    }
    Value response = count == 2 ? read_source(ctx, Value(arguments[0])) : dict();
    if (bytes(get(response, "status"), 16) == "ok" && !hold_first_read(ctx)) {
        put_string(response, "status", "unavailable");
        put_string(response, "reason", "fixture_dependency_barrier_timeout");
    }
    copy_into(destination, response);
}

void invoke_outer(void *, const GDExtensionConstVariantPtr *arguments, GDExtensionInt count,
        GDExtensionVariantPtr destination, GDExtensionCallError *error) {
    error->error = GDEXTENSION_CALL_OK;
    Value response = dict();
    if ((count != 4 && count != 5) || api.type(arguments[0]) != GDEXTENSION_VARIANT_TYPE_STRING ||
            api.type(arguments[1]) != GDEXTENSION_VARIANT_TYPE_STRING ||
            api.type(arguments[2]) != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            api.type(arguments[3]) != GDEXTENSION_VARIANT_TYPE_STRING ||
            (count == 5 && api.type(arguments[4]) != GDEXTENSION_VARIANT_TYPE_STRING)) {
        copy_into(destination, response);
        return;
    }
    Value source(arguments[0]), path(arguments[1]), correlation(arguments[2]), root(arguments[3]);
    std::string project = bytes(root, 4096);
    int root_fd = project.empty() ? -1 : open(project.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    Storage<GAK_SIZE_CALLABLE> storage;
    Context ctx{};
    ctx.source = &source;
    ctx.path = &path;
    ctx.correlation = &correlation;
    ctx.root_fd = root_fd;
    if (count == 5) { ctx.control_path = bytes(Value(arguments[4]), 4096); }
    GDExtensionCallableCustomInfo2 info{};
    info.callable_userdata = &ctx;
    info.token = library;
    info.call_func = reader_callback;
    api.new_callable(storage.ptr(), &info);
    Value reader = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr());
    api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    ctx.reader = &reader;
    Value outer = root_fd < 0 ? Value() :
            call(nullptr, "GDScript", "validate_gdscript_source", GAK_HASH_GDSCRIPT_VALIDATE,
                    {&source, &path, &reader, &correlation});
    if (root_fd >= 0) { close(root_fd); }
    put(response, "outer", outer);
    put(response, "nested", ctx.nested);
    put(response, "reader_calls", integer(ctx.dependency_calls));
    copy_into(destination, response);
}

void initialize(void *, GDExtensionInitializationLevel level) {
    if (level != GDEXTENSION_INITIALIZATION_EDITOR) { return; }
    Name engine_name("Engine");
    void *engine = api.singleton(engine_name.ptr());
    if (!engine) { return; }
    Value key = name("godot_agent_kit_fixture_reentry");
    Value existing = call(engine, "Object", "has_meta", 2619796661ULL, {&key});
    if (truth(existing)) { return; }
    Storage<GAK_SIZE_CALLABLE> storage;
    GDExtensionCallableCustomInfo2 info{};
    info.token = library;
    info.call_func = invoke_outer;
    api.new_callable(storage.ptr(), &info);
    Value callback = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr());
    api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    Value ignored = call(engine, "Object", "set_meta", 3776071444ULL, {&key, &callback});
    (void)ignored;
}

void deinitialize(void *, GDExtensionInitializationLevel level) {
    if (level != GDEXTENSION_INITIALIZATION_EDITOR) { return; }
    Name engine_name("Engine");
    void *engine = api.singleton(engine_name.ptr());
    if (engine) {
        Value key = name("godot_agent_kit_fixture_reentry");
        Value ignored = call(engine, "Object", "remove_meta", 3304788590ULL, {&key});
        (void)ignored;
    }
}

template <typename T> bool load(T &destination, GDExtensionInterfaceGetProcAddress proc, const char *name) {
    destination = reinterpret_cast<T>(proc(name));
    return destination != nullptr;
}
} // namespace

extern "C" __attribute__((visibility("default"))) GDExtensionBool fixture_reentry_init(
        GDExtensionInterfaceGetProcAddress proc, GDExtensionClassLibraryPtr extension,
        GDExtensionInitialization *initialization) {
    using namespace gak;
    library = extension;
    api = Api{};
    if (!load(api.new_nil, proc, "variant_new_nil") ||
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
            !load(api.new_callable, proc, "callable_custom_create2") ||
            !load(api.singleton, proc, "global_get_singleton") ||
            !load(api.bind, proc, "classdb_get_method_bind") ||
            !load(api.bound_call, proc, "object_method_bind_call")) { return 0; }
    for (auto kind : {GDEXTENSION_VARIANT_TYPE_BOOL, GDEXTENSION_VARIANT_TYPE_INT,
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
    initialization->initialize = initialize;
    initialization->deinitialize = deinitialize;
    return 1;
}
