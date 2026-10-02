#pragma once

#include <sys/stat.h>
#include <sys/types.h>
#include "gdextension_interface.h"
#include "native_abi_sizes.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <initializer_list>
#include <string>
#include <string_view>
#include <thread>
#include <variant>

namespace gak {

struct Api {
    GDExtensionInterfaceGetGodotVersion2 version{};
    GDExtensionInterfaceVariantNewNil new_nil{};
    GDExtensionInterfaceVariantNewCopy new_copy{};
    GDExtensionInterfaceVariantDestroy destroy{};
    GDExtensionInterfaceVariantGetType type{};
    GDExtensionInterfaceVariantCall variant_call{};
    GDExtensionInterfaceVariantConstruct construct{};
    GDExtensionInterfaceGetVariantFromTypeConstructor get_from{};
    GDExtensionInterfaceGetVariantToTypeConstructor get_to{};
    GDExtensionInterfaceGetVariantGetInternalPtrFunc get_internal{};
    GDExtensionVariantFromTypeConstructorFunc from[GDEXTENSION_VARIANT_TYPE_VARIANT_MAX]{};
    GDExtensionTypeFromVariantConstructorFunc to[GDEXTENSION_VARIANT_TYPE_VARIANT_MAX]{};
    GDExtensionVariantGetInternalPtrFunc internal[GDEXTENSION_VARIANT_TYPE_VARIANT_MAX]{};
    GDExtensionInterfaceVariantGetPtrDestructor ptr_destructor{};
    GDExtensionInterfaceStringNewWithUtf8CharsAndLen new_string{};
    GDExtensionInterfaceStringToUtf8Chars string_utf8{};
    GDExtensionInterfaceStringNameNewWithUtf8Chars new_name{};
    GDExtensionInterfaceDictionaryOperatorIndex dictionary_index{};
    GDExtensionInterfaceDictionaryOperatorIndexConst dictionary_index_const{};
    GDExtensionInterfaceVariantHasKey has_key{};
    GDExtensionInterfaceArrayOperatorIndexConst array_index_const{};
    GDExtensionInterfaceCallableCustomCreate2 new_callable{};
    GDExtensionInterfaceGlobalGetSingleton singleton{};
    GDExtensionInterfaceClassdbGetMethodBind bind{};
    GDExtensionInterfaceObjectMethodBindCall bound_call{};
    GDExtensionInterfaceObjectGetInstanceId instance_id{};
    GDExtensionInterfaceObjectGetInstanceFromId from_id{};
    GDExtensionInterfaceClassdbGetClassTag class_tag{};
    GDExtensionInterfaceObjectCastTo cast_to{};
};
extern Api api;
extern void *library;

template <size_t N> struct Storage {
    alignas(8) std::byte bytes[N]{};
    void *ptr() { return bytes; }
    const void *ptr() const { return bytes; }
};

struct Value {
    Storage<GAK_SIZE_VARIANT> v;
    Value() { api.new_nil(v.ptr()); }
    explicit Value(GDExtensionConstVariantPtr src) { api.new_copy(v.ptr(), src); }
    Value(const Value &other) { api.new_copy(v.ptr(), other.ptr()); }
    Value &operator=(const Value &other) {
        if (this != &other) { api.destroy(ptr()); api.new_copy(ptr(), other.ptr()); }
        return *this;
    }
    ~Value() { api.destroy(ptr()); }
    void *ptr() { return v.ptr(); }
    const void *ptr() const { return v.ptr(); }
    GDExtensionVariantType type() const { return api.type(ptr()); }
};

struct Name {
    Storage<GAK_SIZE_STRINGNAME> n;
    explicit Name(const char *s) { api.new_name(n.ptr(), s); }
    ~Name() { api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_STRING_NAME)(n.ptr()); }
    const void *ptr() const { return n.ptr(); }
    Name(const Name &) = delete;
};

struct Text {
    Storage<GAK_SIZE_STRING> s;
    explicit Text(std::string_view bytes) { api.new_string(s.ptr(), bytes.data(), static_cast<GDExtensionInt>(bytes.size())); }
    ~Text() { api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_STRING)(s.ptr()); }
    const void *ptr() const { return s.ptr(); }
    Text(const Text &) = delete;
};

inline Value wrap(GDExtensionVariantType type, const void *p) {
    Value v;
    api.destroy(v.ptr());
    api.from[type](v.ptr(), const_cast<void *>(p));
    return v;
}
inline Value string(std::string_view s) { Text t(s); return wrap(GDEXTENSION_VARIANT_TYPE_STRING, t.ptr()); }
inline Value name(const char *s) { Name n(s); return wrap(GDEXTENSION_VARIANT_TYPE_STRING_NAME, n.ptr()); }
inline Value integer(int64_t n) { return wrap(GDEXTENSION_VARIANT_TYPE_INT, &n); }
inline Value boolean(bool b) { GDExtensionBool x = b; return wrap(GDEXTENSION_VARIANT_TYPE_BOOL, &x); }

inline GDExtensionInt utf8_size(const Value &v) {
    return v.type() == GDEXTENSION_VARIANT_TYPE_STRING
            ? api.string_utf8(api.internal[GDEXTENSION_VARIANT_TYPE_STRING](const_cast<void *>(v.ptr())), nullptr, 0)
            : -1;
}
inline std::string bytes(const Value &v, size_t limit = 512 * 1024) {
    const auto count = utf8_size(v);
    if (count < 0 || static_cast<uint64_t>(count) > limit) { return {}; }
    std::string result(static_cast<size_t>(count), '\0');
    if (count) {
        api.string_utf8(api.internal[GDEXTENSION_VARIANT_TYPE_STRING](const_cast<void *>(v.ptr())), result.data(), count);
    }
    return result;
}
inline bool checked_bytes(const Value &v, std::string &out, size_t limit = 512 * 1024) {
    const auto count = utf8_size(v);
    if (count < 0 || static_cast<uint64_t>(count) > limit) { return false; }
    out.resize(static_cast<size_t>(count));
    return count == 0 || api.string_utf8(api.internal[GDEXTENSION_VARIANT_TYPE_STRING](
            const_cast<void *>(v.ptr())), out.data(), count) == count;
}
inline uint64_t id(const Value &v) {
    if (v.type() != GDEXTENSION_VARIANT_TYPE_OBJECT) { return 0; }
    void *ptr = nullptr;
    api.to[GDEXTENSION_VARIANT_TYPE_OBJECT](&ptr, const_cast<void *>(v.ptr()));
    return ptr ? api.instance_id(ptr) : 0;
}
inline void *object_ptr(const Value &v) {
    if (v.type() != GDEXTENSION_VARIANT_TYPE_OBJECT) { return nullptr; }
    void *ptr = nullptr;
    api.to[GDEXTENSION_VARIANT_TYPE_OBJECT](&ptr, const_cast<void *>(v.ptr()));
    return ptr;
}
inline bool truth(const Value &v) {
    if (v.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return false; }
    GDExtensionBool yes = 0;
    api.to[GDEXTENSION_VARIANT_TYPE_BOOL](&yes, const_cast<void *>(v.ptr()));
    return yes;
}
inline Value collection(GDExtensionVariantType kind) {
    Value v;
    api.destroy(v.ptr());
    GDExtensionCallError error{};
    api.construct(kind, v.ptr(), nullptr, 0, &error);
    return v;
}
inline Value dict() { return collection(GDEXTENSION_VARIANT_TYPE_DICTIONARY); }
inline void put(Value &dict, const char *key, const Value &value) {
    Value k = string(key);
    auto *dst = api.dictionary_index(api.internal[GDEXTENSION_VARIANT_TYPE_DICTIONARY](dict.ptr()), k.ptr());
    api.destroy(dst);
    api.new_copy(dst, value.ptr());
}
inline Value get(const Value &dict, const char *key) {
    if (dict.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY) { return Value(); }
    Value k = string(key);
    GDExtensionBool valid = 0;
    if (!api.has_key(dict.ptr(), k.ptr(), &valid) || !valid) { return Value(); }
    auto *src = api.dictionary_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_DICTIONARY](const_cast<void *>(dict.ptr())), k.ptr());
    return src ? Value(src) : Value();
}
inline void put_string(Value &dict, const char *key, std::string_view value) { put(dict, key, string(value)); }
inline bool checked_call(Value &result, void *instance, const char *cls, const char *method,
        uint64_t hash, std::initializer_list<const Value *> args = {}) {
    Name class_name(cls), method_name(method);
    auto binding = api.bind(class_name.ptr(), method_name.ptr(), hash);
    if (!binding || !instance || args.size() > 4) { return false; }
    std::array<GDExtensionConstVariantPtr, 4> raw{};
    size_t i = 0;
    for (auto *v : args) { raw[i++] = v->ptr(); }
    api.destroy(result.ptr());
    GDExtensionCallError error{};
    api.bound_call(binding, instance, raw.data(), static_cast<GDExtensionInt>(args.size()), result.ptr(), &error);
    if (error.error == GDEXTENSION_CALL_OK) { return true; }
    api.destroy(result.ptr()); api.new_nil(result.ptr());
    return false;
}
inline Value call(void *instance, const char *cls, const char *method, uint64_t hash,
        std::initializer_list<const Value *> args = {}) {
    Value result;
    checked_call(result, instance, cls, method, hash, args);
    return result;
}
inline bool checked_invoke(Value &result, Value &self, const char *method,
        std::initializer_list<const Value *> args = {}) {
    Name key(method);
    if (args.size() > 4) { return false; }
    std::array<GDExtensionConstVariantPtr, 4> raw{};
    size_t i = 0;
    for (auto *v : args) { raw[i++] = v->ptr(); }
    api.destroy(result.ptr());
    GDExtensionCallError error{};
    api.variant_call(self.ptr(), key.ptr(), raw.data(), static_cast<GDExtensionInt>(args.size()), result.ptr(), &error);
    if (error.error == GDEXTENSION_CALL_OK) { return true; }
    api.destroy(result.ptr()); api.new_nil(result.ptr());
    return false;
}
inline Value invoke(Value &self, const char *method, std::initializer_list<const Value *> args = {}) {
    Value result;
    checked_invoke(result, self, method, args);
    return result;
}
inline void copy_into(void *destination, const Value &v) { api.destroy(destination); api.new_copy(destination, v.ptr()); }

std::string sha256(std::string_view bytes);
struct EditAttempt;
struct OpenAttempt;
struct CloseAttempt;
bool valid_utf8(std::string_view source);
bool same_time(timespec left, timespec right);
bool engine_binary_matches();
struct Session {
    int project_fd = -1;
    dev_t device = 0;
    ino_t inode = 0;
    std::string project_path;
    std::string session_id;
    std::thread::id main_thread;
    uint64_t close_generation{};
    enum class CallState { Idle, Running, Closing };
    std::variant<std::monostate, EditAttempt *, OpenAttempt *, CloseAttempt *> owner;
    CallState call_state = CallState::Idle;
};
inline EditAttempt *edit_attempt(Session &session) {
    auto *held = std::get_if<EditAttempt *>(&session.owner);
    return held ? *held : nullptr;
}
inline OpenAttempt *open_attempt(Session &session) {
    auto *held = std::get_if<OpenAttempt *>(&session.owner);
    return held ? *held : nullptr;
}
inline CloseAttempt *close_attempt(Session &session) {
    auto *held = std::get_if<CloseAttempt *>(&session.owner);
    return held ? *held : nullptr;
}
inline bool occupied(const Session &session) {
    return !std::holds_alternative<std::monostate>(session.owner);
}
bool configure(Session &session, const std::string &session_id);
void close(Session &session);
void edit_cleanup(Session &session);
void open_cleanup(Session &session);
void open_closing(Session &session);
Value open_inspect(Session &session, const Value &path, const Value &correlation);
Value open_prepare(Session &session, const Value &request, const Value &source, const Value &capture);
Value open_advance(Session &session, const Value &request, const Value &stage,
        const Value &source_hash, const Value &context_hash);
Value open_verify(Session &session, const Value &request, const Value &purpose);
Value open_recheck(Session &session, const Value &request, const Value &purpose);
Value open_finish(Session &session, const Value &request);
Value open_abort(Session &session, const Value &request);
bool open_expire(Session &session);
void close_cleanup(Session &session);
void close_closing(Session &session);
Value close_inspect(Session &session, const Value &path, const Value &correlation);
Value close_prepare(Session &session, const Value &request, const Value &source, const Value &capture);
Value close_advance(Session &session, const Value &request, const Value &guard, const Value &receipts);
Value close_status(Session &session, const Value &request);
Value close_verify(Session &session, const Value &request, const Value &purpose);
Value close_recheck(Session &session, const Value &request, const Value &purpose);
Value close_finish(Session &session, const Value &request);
Value close_abort(Session &session, const Value &request);
Value close_expire(Session &session);
Value edit_inspect(Session &session, const Value &path, const Value &original,
        const Value &desired, const Value &document);
Value edit_prepare(Session &session, const Value &path, const Value &expected,
        const Value &desired, const Value &correlation);
Value edit_advance(Session &session, const Value &request, const Value &stage);
Value edit_cancel(Session &session, const Value &request);
bool edit_expire(Session &session);
#if GAK_FIXTURE
Value edit_fixture_fault(Session &session, const Value &request, const Value &fault);
Value open_fixture_fault(Session &session, const Value &request, const Value &fault);
Value open_fixture_state(Session &session, const Value &request);
Value close_fixture_fault(Session &session, const Value &request, const Value &fault);
Value close_fixture_state(Session &session, const Value &request);
#endif
} // namespace gak
