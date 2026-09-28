#include "native.hpp"

#include <CommonCrypto/CommonDigest.h>
#include <sys/acl.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <algorithm>
#include <cerrno>
#include <charconv>
#include <climits>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>
#include <thread>
#include <unordered_map>
#include <utility>

namespace gak {
namespace {
constexpr size_t MAX_SOURCE = 512 * 1024;
constexpr size_t MAX_DEPENDENCIES = 32;
constexpr size_t MAX_DEPENDENCY_BYTES = 4 * 1024 * 1024;

struct Fd {
    int fd = -1;
    explicit Fd(int n = -1) : fd(n) {}
    ~Fd() { if (fd >= 0) { ::close(fd); } }
    Fd(const Fd &) = delete;
    Fd &operator=(const Fd &) = delete;
};

enum class Access { Ancestor, Project, Directory, Source };

bool permitted(int fd, Access access) {
    struct stat st{};
    const bool directory = access != Access::Source;
    if (fstat(fd, &st) || (directory ? !S_ISDIR(st.st_mode) : !S_ISREG(st.st_mode)) ||
            (st.st_uid != geteuid() && (!directory || st.st_uid != 0)) ||
            (access == Access::Project && st.st_uid != geteuid())) { return false; }
    const bool sticky_ancestor = access == Access::Ancestor && st.st_uid == 0 && (st.st_mode & S_ISVTX);
    if ((st.st_mode & (S_IWGRP | S_IWOTH)) && !sticky_ancestor) { return false; }
    acl_t acl = acl_get_fd_np(fd, ACL_TYPE_EXTENDED);
    if (!acl) { return errno == ENOENT; }
    int cursor = ACL_FIRST_ENTRY;
    while (true) {
        acl_entry_t entry = nullptr;
        const int result = acl_get_entry(acl, cursor, &entry);
        if (result == 0 && entry) {
            acl_tag_t tag;
            if (acl_get_tag_type(entry, &tag) || tag != ACL_EXTENDED_DENY) {
                acl_free(acl);
                return false;
            }
            cursor = ACL_NEXT_ENTRY;
        } else {
            // Darwin returns 0 for an entry, and EINVAL after the last entry.
            const bool exhausted = result == -1 && cursor == ACL_NEXT_ENTRY && errno == EINVAL;
            acl_free(acl);
            return exhausted;
        }
    }
}

bool valid_segment(std::string_view part) {
    return !part.empty() && part != "." && part != ".." && part.find('\0') == part.npos &&
            part.find('\\') == part.npos && part.find(':') == part.npos && part.find('\n') == part.npos &&
            part.find('\r') == part.npos && part.size() <= 255;
}

bool utf8_text(std::string_view source) {
    size_t i = 0;
    while (i < source.size()) {
        const auto c = static_cast<unsigned char>(source[i]);
        if (c == 0) { return false; }
        if (c < 0x80) { ++i; continue; }
        const size_t width = c >= 0xC2 && c <= 0xDF ? 2 :
                c >= 0xE0 && c <= 0xEF ? 3 :
                c >= 0xF0 && c <= 0xF4 ? 4 : 0;
        if (!width || width > source.size() - i) { return false; }
        for (size_t j = 1; j < width; ++j) {
            if ((static_cast<unsigned char>(source[i + j]) & 0xC0) != 0x80) { return false; }
        }
        const auto next = static_cast<unsigned char>(source[i + 1]);
        if ((c == 0xE0 && next < 0xA0) || (c == 0xED && next >= 0xA0) ||
                (c == 0xF0 && next < 0x90) || (c == 0xF4 && next >= 0x90)) { return false; }
        i += width;
    }
    return true;
}

// The canonical path is derived only from ProjectSettings, never from a validation request.
// Open every component with no-follow semantics and check identity at the end.
int open_absolute(const std::string &path, dev_t &device, ino_t &inode) {
    if (path.empty() || path.front() != '/' || path.size() > 1024 || path == "/") { return -1; }
    Fd current(open("/", O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
    if (current.fd < 0 || !permitted(current.fd, Access::Ancestor)) { return -1; }
    size_t pos = 1;
    while (pos < path.size()) {
        const size_t end = path.find('/', pos);
        const std::string_view part(path.data() + pos, (end == path.npos ? path.size() : end) - pos);
        if (!valid_segment(part)) { return -1; }
        const std::string component(part);
        Fd next(openat(current.fd, component.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        if (next.fd < 0 || !permitted(next.fd, end == path.npos ? Access::Project : Access::Ancestor)) { return -1; }
        ::close(current.fd);
        current.fd = next.fd;
        next.fd = -1;
        if (end == path.npos) { break; }
        pos = end + 1;
    }
    struct stat st{};
    if (fstat(current.fd, &st)) { return -1; }
    device = st.st_dev;
    inode = st.st_ino;
    int result = current.fd;
    current.fd = -1;
    return result;
}

bool valid_resource(const std::string &path, bool &remap) {
    remap = path.size() > 9 && path.compare(path.size() - 9, 9, ".gd.remap") == 0;
    if (path.size() < 9 || path.size() > (remap ? 2054 : 2048) || path.compare(0, 6, "res://") != 0 ||
            path.find('\0') != path.npos || (remap ? false : path.compare(path.size() - 3, 3, ".gd") != 0)) { return false; }
    size_t pos = 6;
    while (pos < path.size()) {
        const size_t end = path.find('/', pos);
        const std::string_view part(path.data() + pos, (end == path.npos ? path.size() : end) - pos);
        if (!valid_segment(part)) { return false; }
        if (end == path.npos) { break; }
        pos = end + 1;
    }
    return true;
}

bool path_attached(int project_fd, const std::string &path, dev_t device, ino_t inode) {
    Fd directory(dup(project_fd));
    if (directory.fd < 0) { return false; }
    size_t pos = 6;
    while (true) {
        const size_t end = path.find('/', pos);
        const std::string component(path.substr(pos, (end == path.npos ? path.size() : end) - pos));
        if (end == path.npos) {
            Fd attached(openat(directory.fd, component.c_str(), O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC));
            struct stat st{};
            return attached.fd >= 0 && permitted(attached.fd, Access::Source) &&
                    !fstat(attached.fd, &st) && st.st_dev == device && st.st_ino == inode;
        }
        Fd next(openat(directory.fd, component.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        if (next.fd < 0 || !permitted(next.fd, Access::Directory)) { return false; }
        ::close(directory.fd);
        directory.fd = next.fd;
        next.fd = -1;
        pos = end + 1;
    }
}

struct Snapshot {
    std::string status = "unavailable";
    std::string reason = "access_denied";
    std::string source;
    std::string hash;
    dev_t device = 0;
    ino_t inode = 0;
};

Snapshot read_at(int project_fd, const std::string &path) {
    Snapshot result;
    bool remap = false;
    if (!valid_resource(path, remap)) { result.reason = "invalid_path"; return result; }
    Fd directory(dup(project_fd));
    if (directory.fd < 0) { result.reason = "read_failed"; return result; }
    size_t pos = 6;
    while (true) {
        size_t end = path.find('/', pos);
        std::string component(path.substr(pos, (end == path.npos ? path.size() : end) - pos));
        if (end == path.npos) {
            Fd file(openat(directory.fd, component.c_str(), O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC));
            if (file.fd < 0) {
                if (errno == ENOENT) { result.status = "missing"; result.reason = "not_found"; }
                return result;
            }
            if (!permitted(file.fd, Access::Source)) { return result; }
            struct stat before{}, after{}, attached{};
            if (fstat(file.fd, &before) || before.st_size < 0 || before.st_nlink == 0) {
                result.reason = "invalidated"; return result;
            }
            if (before.st_size > static_cast<off_t>(MAX_SOURCE)) {
                result.reason = "source_limit"; return result;
            }
            if (remap) { result.reason = "unsupported_remap"; return result; }
            result.source.resize(static_cast<size_t>(before.st_size));
            size_t offset = 0;
            while (offset < result.source.size()) {
                ssize_t n = pread(file.fd, result.source.data() + offset, result.source.size() - offset, static_cast<off_t>(offset));
                if (n <= 0) { result.reason = "read_failed"; result.source.clear(); return result; }
                offset += static_cast<size_t>(n);
            }
            if (fstat(file.fd, &after) || fstatat(directory.fd, component.c_str(), &attached, AT_SYMLINK_NOFOLLOW) ||
                    before.st_dev != after.st_dev || before.st_ino != after.st_ino || before.st_size != after.st_size ||
                    before.st_mtimespec.tv_sec != after.st_mtimespec.tv_sec || before.st_mtimespec.tv_nsec != after.st_mtimespec.tv_nsec ||
                    before.st_ctimespec.tv_sec != after.st_ctimespec.tv_sec || before.st_ctimespec.tv_nsec != after.st_ctimespec.tv_nsec ||
                    after.st_dev != attached.st_dev || after.st_ino != attached.st_ino || !S_ISREG(attached.st_mode) ||
                    !permitted(file.fd, Access::Source)) { result.reason = "invalidated"; result.source.clear(); return result; }
            // Compare a second descriptor read without duplicating the source buffer.
            char comparison[8192];
            offset = 0;
            while (offset < result.source.size()) {
                const size_t capacity = std::min(sizeof(comparison), result.source.size() - offset);
                ssize_t n = pread(file.fd, comparison, capacity, static_cast<off_t>(offset));
                if (n <= 0 || std::memcmp(comparison, result.source.data() + offset, static_cast<size_t>(n))) {
                    result.reason = n < 0 ? "read_failed" : "invalidated";
                    result.source.clear();
                    return result;
                }
                offset += static_cast<size_t>(n);
            }
            if (fstat(file.fd, &after) || fstatat(directory.fd, component.c_str(), &attached, AT_SYMLINK_NOFOLLOW) ||
                    before.st_size != after.st_size ||
                    after.st_dev != attached.st_dev || after.st_ino != attached.st_ino || !S_ISREG(attached.st_mode) ||
                    before.st_mtimespec.tv_sec != after.st_mtimespec.tv_sec || before.st_mtimespec.tv_nsec != after.st_mtimespec.tv_nsec ||
                    before.st_ctimespec.tv_sec != after.st_ctimespec.tv_sec || before.st_ctimespec.tv_nsec != after.st_ctimespec.tv_nsec) {
                result.reason = "invalidated"; result.source.clear(); return result;
            }
            if (!path_attached(project_fd, path, before.st_dev, before.st_ino)) {
                result.reason = "invalidated";
                result.source.clear();
                return result;
            }
            if (!utf8_text(result.source)) { result.reason = "invalid_encoding"; result.source.clear(); return result; }
            result.hash = sha256(result.source);
            result.device = before.st_dev;
            result.inode = before.st_ino;
            result.status = "ok";
            result.reason.clear();
            return result;
        }
        Fd next(openat(directory.fd, component.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        if (next.fd < 0) {
            if (errno == ENOENT) { result.status = "missing"; result.reason = "not_found"; }
            return result;
        }
        if (!permitted(next.fd, Access::Directory)) { return result; }
        ::close(directory.fd);
        directory.fd = next.fd;
        next.fd = -1;
        pos = end + 1;
    }
}

bool valid_session_id(const std::string &id) {
    if (id.size() != 32) { return false; }
    for (char c : id) { if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) { return false; } }
    return true;
}

} // namespace

std::string sha256(std::string_view source) {
    unsigned char hash[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256(source.data(), static_cast<CC_LONG>(source.size()), hash);
    constexpr char hex[] = "0123456789abcdef";
    std::string result(CC_SHA256_DIGEST_LENGTH * 2, '0');
    for (size_t i = 0; i < sizeof(hash); ++i) {
        result[2 * i] = hex[hash[i] >> 4];
        result[2 * i + 1] = hex[hash[i] & 15];
    }
    return result;
}

bool engine_binary_matches() {
    Name os_name("OS");
    void *os = api.singleton(os_name.ptr());
    if (!os) { return false; }
    Value executable = call(os, "OS", "get_executable_path", 201670096ULL);
    if (executable.type() != GDEXTENSION_VARIANT_TYPE_STRING || utf8_size(executable) > PATH_MAX) { return false; }
    const std::string reported = bytes(executable, PATH_MAX);
    if (reported.empty() || reported[0] != '/' || reported.find('\0') != reported.npos) { return false; }
    char canonical[PATH_MAX];
    if (!realpath(reported.c_str(), canonical)) { return false; }
    Fd binary(open(canonical, O_RDONLY | O_NOFOLLOW | O_CLOEXEC));
    if (binary.fd < 0) { return false; }
    struct stat before{}, after{}, attached{};
    if (fstat(binary.fd, &before) || !S_ISREG(before.st_mode)) { return false; }
    CC_SHA256_CTX context{};
    if (CC_SHA256_Init(&context) != 1) { return false; }
    char block[65536];
    while (true) {
        const ssize_t count = ::read(binary.fd, block, sizeof(block));
        if (count < 0 && errno == EINTR) { continue; }
        if (count < 0) { return false; }
        if (count == 0) { break; }
        if (CC_SHA256_Update(&context, block, static_cast<CC_LONG>(count)) != 1) { return false; }
    }
    unsigned char digest[CC_SHA256_DIGEST_LENGTH];
    if (CC_SHA256_Final(digest, &context) != 1 || fstat(binary.fd, &after) ||
            stat(canonical, &attached) || before.st_dev != after.st_dev || before.st_ino != after.st_ino ||
            before.st_size != after.st_size || before.st_dev != attached.st_dev || before.st_ino != attached.st_ino ||
            before.st_mtimespec.tv_sec != after.st_mtimespec.tv_sec ||
            before.st_mtimespec.tv_nsec != after.st_mtimespec.tv_nsec) { return false; }
    constexpr char hex[] = "0123456789abcdef";
    char actual[CC_SHA256_DIGEST_LENGTH * 2 + 1];
    for (size_t i = 0; i < sizeof(digest); ++i) {
        actual[i * 2] = hex[digest[i] >> 4];
        actual[i * 2 + 1] = hex[digest[i] & 15];
    }
    actual[CC_SHA256_DIGEST_LENGTH * 2] = '\0';
    return std::strcmp(actual, GAK_ENGINE_BINARY_SHA256) == 0;
}


struct ReadContext {
    ReadContext(Session &selected, std::string path) : session(selected), root_path(std::move(path)) {}
    Session &session;
    std::string root_path;
    Snapshot root;
    struct Witness {
        std::string status, reason, hash;
        dev_t device;
        ino_t inode;
        size_t bytes;
    };
    std::unordered_map<std::string, Witness> records;
    size_t total_bytes = 0;
    size_t dependency_count = 0;
    bool active = true;
    std::string failure;

    bool project_current() const {
        dev_t dev = 0; ino_t ino = 0;
        Fd current(open_absolute(session.project_path, dev, ino));
        struct stat st{};
        return current.fd >= 0 && dev == session.device && ino == session.inode &&
                !fstat(session.project_fd, &st) && st.st_dev == dev && st.st_ino == ino;
    }
    Snapshot read(const std::string &path, bool recheck) {
        Snapshot unavailable;
        unavailable.reason = "invalidated";
        if (!active || (session.call_state == Session::CallState::Closing) || std::this_thread::get_id() != session.main_thread || !project_current()) {
            failure = "invalidated";
            return unavailable;
        }
        if (path == root_path) {
            failure = "root_overlay_only";
            unavailable.reason = failure;
            return unavailable;
        }
        auto previous = records.find(path);
        if (recheck && previous == records.end()) {
            failure = "unattributed_dependency";
            unavailable.reason = failure;
            return unavailable;
        }
        if (!recheck && previous != records.end()) { recheck = true; }
        Snapshot current = read_at(session.project_fd, path);
        if (recheck) {
            const Witness &old = previous->second;
            if (old.status != current.status || old.reason != current.reason || old.hash != current.hash ||
                    old.device != current.device || old.inode != current.inode || old.bytes != current.source.size()) {
                failure = "dependency_changed";
                current.status = "unavailable";
                current.reason = failure;
                current.source.clear();
            }
        } else {
            bool remap = false;
            valid_resource(path, remap);
            if (!remap && (dependency_count >= MAX_DEPENDENCIES ||
                    current.source.size() > MAX_DEPENDENCY_BYTES - total_bytes)) {
                failure = "dependency_limit";
                current.status = "unavailable";
                current.reason = failure;
                current.source.clear();
            } else {
                if (!remap) { ++dependency_count; total_bytes += current.source.size(); }
                records.emplace(path, Witness{current.status, current.reason, current.hash,
                        current.device, current.inode, current.source.size()});
            }
        }
        if (current.status == "unavailable") { failure = current.reason; }
        return current;
    }
};

namespace {
Value snapshot_result(const Snapshot &snapshot) {
    Value result = dict();
    put_string(result, "status", snapshot.status);
    if (snapshot.reason.empty()) { put(result, "reason", Value()); }
    else { put_string(result, "reason", snapshot.reason); }
    if (snapshot.status == "ok") {
        put_string(result, "source", snapshot.source);
        put_string(result, "source_sha256", snapshot.hash);
        put(result, "utf8_bytes", integer(static_cast<int64_t>(snapshot.source.size())));
        Value identity = dict();
        put_string(identity, "device", std::to_string(snapshot.device));
        put_string(identity, "inode", std::to_string(snapshot.inode));
        put(result, "identity", identity);
    }
    return result;
}

void reader_callback(void *userdata, const GDExtensionConstVariantPtr *args, GDExtensionInt count, GDExtensionVariantPtr result, GDExtensionCallError *error) {
    auto &ctx = **static_cast<std::shared_ptr<ReadContext> *>(userdata);
    error->error = GDEXTENSION_CALL_OK;
    Snapshot snapshot;
    snapshot.reason = "invalid_reader_input";
    if (ctx.active && count == 2 && api.type(args[0]) == GDEXTENSION_VARIANT_TYPE_STRING &&
            api.type(args[1]) == GDEXTENSION_VARIANT_TYPE_BOOL) {
        Value path(args[0]), recheck(args[1]);
        snapshot = ctx.read(bytes(path, 2054), truth(recheck));
    } else { ctx.failure = "invalid_reader_input"; }
    Value returned = snapshot_result(snapshot);
    copy_into(result, returned);
}
GDExtensionBool reader_valid(void *userdata) {
    return (**static_cast<std::shared_ptr<ReadContext> *>(userdata)).active;
}
void reader_free(void *userdata) { delete static_cast<std::shared_ptr<ReadContext> *>(userdata); }

Value reader_callable(const std::shared_ptr<ReadContext> &ctx) {
    Storage<GAK_SIZE_CALLABLE> storage;
    GDExtensionCallableCustomInfo2 info{};
    info.callable_userdata = new std::shared_ptr<ReadContext>(ctx);
    info.token = library;
    info.call_func = reader_callback;
    info.is_valid_func = reader_valid;
    info.free_func = reader_free;
    api.new_callable(storage.ptr(), &info);
    Value result = wrap(GDEXTENSION_VARIANT_TYPE_CALLABLE, storage.ptr());
    api.ptr_destructor(GDEXTENSION_VARIANT_TYPE_CALLABLE)(storage.ptr());
    return result;
}

bool decimal_string(const Value &value) {
    if (value.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return false; }
    std::string text = bytes(value);
    if (text.empty() || text.size() > 20 || (text.size() > 1 && text[0] == '0')) { return false; }
    for (char c : text) { if (c < '0' || c > '9') { return false; } }
    return true;
}
bool request_id(const std::string &text) {
    if (text.empty() || text.size() > 64) { return false; }
    for (char c : text) { if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') ||
            (c >= 'A' && c <= 'Z') || c == '-' || c == '_')) { return false; } }
    return true;
}
bool hex_digest(const std::string &text) {
    if (text.size() != 64) { return false; }
    for (char c : text) { if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) { return false; } }
    return true;
}

std::string tick() {
    Name time_name("Time");
    void *time = api.singleton(time_name.ptr());
    if (!time) { return {}; }
    Value now = call(time, "Time", "get_ticks_usec", 3905245786ULL);
    if (now.type() != GDEXTENSION_VARIANT_TYPE_INT) { return {}; }
    int64_t count = 0;
    api.to[GDEXTENSION_VARIANT_TYPE_INT](&count, now.ptr());
    return count >= 0 ? std::to_string(count) : std::string();
}

void invalidate(Value &result, const char *reason) {
    put_string(result, "status", "unavailable");
    put_string(result, "reason", reason);
    put(result, "context_current", boolean(false));
    put(result, "dependencies_current", boolean(false));
    put(result, "diagnostics_complete", boolean(false));
}

// Refusals preserve only validated, non-sensitive correlation; no parser completion is inferred.
Value refusal(const char *reason, const Value *correlation, const std::string &path,
        const std::string &hash, size_t length) {
    const std::string start_tick = tick();
    Value result = dict();
    put_string(result, "status", "unavailable");
    put_string(result, "reason", reason);
    if (correlation && correlation->type() == GDEXTENSION_VARIANT_TYPE_DICTIONARY) {
        for (auto *key : {"request_id", "session_id", "purpose"}) {
            Value value = get(*correlation, key);
            if (value.type() == GDEXTENSION_VARIANT_TYPE_STRING) { put(result, key, value); }
        }
        Value source_document = get(*correlation, "document");
        if (source_document.type() == GDEXTENSION_VARIANT_TYPE_DICTIONARY) {
            Value bound_document = dict();
            for (auto *key : {"resource_instance_id", "editor_instance_id", "buffer_instance_id"}) {
                Value value = get(source_document, key);
                if (decimal_string(value)) { put(bound_document, key, value); }
            }
            put(result, "document", bound_document);
        }
    }
    if (!path.empty()) { put_string(result, "source_path", path); }
    Value input = dict();
    if (!hash.empty()) { put_string(input, "source_sha256", hash); put(input, "utf8_bytes", integer(static_cast<int64_t>(length))); }
    put(result, "input", input);
    put(result, "dependencies", array());
    put(result, "diagnostics", array());
    put(result, "diagnostics_complete", boolean(false));
    put(result, "context_current", boolean(false));
    put(result, "dependencies_current", boolean(false));
    const std::string end_tick = tick();
    if (!start_tick.empty() && !end_tick.empty()) {
        Value collection = dict();
        put_string(collection, "clock_id", "editor");
        put_string(collection, "started_tick", start_tick);
        put_string(collection, "finished_tick", end_tick);
        put(result, "collection", collection);
    } else { put(result, "collection", Value()); }
    put(result, "context", Value());
    return result;
}

bool association(const Value &document, const std::string &source_path) {
    const auto read_id = [&document](const char *field, uint64_t &value) {
        const Value supplied = get(document, field);
        if (supplied.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return false; }
        const std::string text = bytes(supplied);
        if (text.empty() || (text.size() > 1 && text.front() == '0')) { return false; }
        const auto parsed = std::from_chars(text.data(), text.data() + text.size(), value);
        return parsed.ec == std::errc{} && parsed.ptr == text.data() + text.size();
    };
    uint64_t rid = 0, eid = 0, bid = 0;
    if (!read_id("resource_instance_id", rid) || !read_id("editor_instance_id", eid) ||
            !read_id("buffer_instance_id", bid)) { return false; }
    Name editor_interface("EditorInterface"), codeedit_name("CodeEdit"), gdscript_name("GDScript");
    void *interface = api.singleton(editor_interface.ptr());
    void *codeedit_tag = api.class_tag(codeedit_name.ptr());
    void *gdscript_tag = api.class_tag(gdscript_name.ptr());
    if (!interface || !codeedit_tag || !gdscript_tag) { return false; }
    Value se = call(interface, "EditorInterface", "get_script_editor", 90868003ULL);
    void *script_editor = object_ptr(se);
    if (!script_editor) { return false; }
    Value open = call(script_editor, "ScriptEditor", "get_open_script_editors", 3995934104ULL);
    if (open.type() != GDEXTENSION_VARIANT_TYPE_ARRAY) { return false; }
    Value size = invoke(open, "size");
    if (size.type() != GDEXTENSION_VARIANT_TYPE_INT) { return false; }
    int64_t count = 0;
    api.to[GDEXTENSION_VARIANT_TYPE_INT](&count, size.ptr());
    if (count <= 0 || count > 256) { return false; }
    int matches = 0;
    for (int64_t i = 0; i < count; ++i) {
        auto *entry = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](open.ptr()), i);
        if (!entry) { return false; }
        Value editor(entry);
        void *editor_ptr = object_ptr(editor);
        if (!editor_ptr || id(editor) != eid) { continue; }
        Value script = call(editor_ptr, "ScriptEditorBase", "get_edited_resource", GAK_HASH_SCRIPT_EDITOR_EDITED);
        void *script_ptr = object_ptr(script);
        Value buffer = call(editor_ptr, "ScriptEditorBase", "get_base_editor", 2783021301ULL);
        void *buffer_ptr = object_ptr(buffer);
        if (!script_ptr || !buffer_ptr || !api.cast_to(script_ptr, gdscript_tag) ||
                !api.cast_to(buffer_ptr, codeedit_tag) ||
                id(script) != rid || id(buffer) != bid) { return false; }
        Value path = call(script_ptr, "Resource", "get_path", 201670096ULL);
        if (path.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(path) != source_path) { return false; }
        ++matches;
    }
    return matches == 1;
}
} // namespace

void close(Session &session) {
    if (session.call_state != Session::CallState::Idle) {
        session.call_state = Session::CallState::Closing;
        return;
    }
    if (session.project_fd >= 0) { ::close(session.project_fd); }
    session.project_fd = -1;
    session.project_path.clear();
    session.session_id.clear();
}

bool configure(Session &session, const std::string &session_id) {
    if (session.call_state != Session::CallState::Idle || !valid_session_id(session_id)) { return false; }
    close(session);
    Name settings("ProjectSettings");
    void *project_settings = api.singleton(settings.ptr());
    if (!project_settings) { return false; }
    Value resource = string("res://");
    Value absolute = call(project_settings, "ProjectSettings", "globalize_path", 3135753539ULL, {&resource});
    if (absolute.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return false; }
    std::string path = bytes(absolute);
    if (path.empty() || path.size() > 1024 || path.find('\0') != path.npos) { return false; }
    char resolved[PATH_MAX];
    if (!realpath(path.c_str(), resolved)) { return false; }
    path.assign(resolved);
    if (path.size() > 1024) { return false; }
    if (path.size() > 1 && path.back() == '/') { path.pop_back(); }
    dev_t device = 0; ino_t inode = 0;
    int fd = open_absolute(path, device, inode);
    if (fd < 0) { return false; }
    session.project_fd = fd;
    session.project_path = std::move(path);
    session.device = device;
    session.inode = inode;
    session.session_id = session_id;
    return true;
}

Value validation(Session &session, std::initializer_list<const Value *> args) {
    if (args.size() != 3) { return refusal("invalid_arguments", nullptr, {}, {}, 0); }
    if (std::this_thread::get_id() != session.main_thread) { return refusal("wrong_thread", nullptr, {}, {}, 0); }
    const Value &source = *args.begin()[0], &path = *args.begin()[1], &correlation = *args.begin()[2];
    if (source.type() != GDEXTENSION_VARIANT_TYPE_STRING || path.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            correlation.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY) {
        return refusal("invalid_arguments", nullptr, {}, {}, 0);
    }
    if (session.project_fd < 0) { return refusal("session_unavailable", nullptr, {}, {}, 0); }
    if (session.call_state != Session::CallState::Idle) { return refusal("reentrant", nullptr, {}, {}, 0); }
    if (utf8_size(path) < 0 || utf8_size(path) > 2048) { return refusal("invalid_path", nullptr, {}, {}, 0); }
    std::string root = bytes(path, 2048);
    bool remap = false;
    if (!valid_resource(root, remap) || remap) { return refusal("invalid_path", nullptr, {}, {}, 0); }
    if (utf8_size(source) < 0 || utf8_size(source) > static_cast<GDExtensionInt>(MAX_SOURCE)) {
        return refusal("root_source_limit", nullptr, root, {}, 0);
    }
    std::string text = bytes(source, MAX_SOURCE);
    if (!utf8_text(text)) { return refusal("invalid_encoding", nullptr, root, {}, 0); }
    std::string digest = sha256(text);
    Value request = get(correlation, "request_id"), identity = get(correlation, "session_id"), purpose = get(correlation, "purpose");
    Value document = get(correlation, "document"), expected = get(correlation, "expected_source_sha256");
    if (request.type() != GDEXTENSION_VARIANT_TYPE_STRING || !request_id(bytes(request)) ||
            identity.type() != GDEXTENSION_VARIANT_TYPE_STRING || !valid_session_id(bytes(identity)) ||
            purpose.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            (bytes(purpose) != "preflight" && bytes(purpose) != "post_change" && bytes(purpose) != "unchanged") ||
            document.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            !decimal_string(get(document, "resource_instance_id")) ||
            !decimal_string(get(document, "editor_instance_id")) ||
            !decimal_string(get(document, "buffer_instance_id")) ||
            expected.type() != GDEXTENSION_VARIANT_TYPE_STRING || !hex_digest(bytes(expected))) {
        return refusal("invalid_correlation", nullptr, {}, {}, 0);
    }
    if (bytes(identity) != session.session_id) { return refusal("wrong_session", nullptr, {}, {}, 0); }
    session.call_state = Session::CallState::Running;
    struct ActiveCall {
        Session &session;
        ~ActiveCall() {
            const bool closing = session.call_state == Session::CallState::Closing;
            session.call_state = Session::CallState::Idle;
            if (closing) { close(session); }
        }
    } active_call{session};
    auto ctx = std::make_shared<ReadContext>(session, root);
    if (!ctx->project_current()) { return refusal("project_changed", &correlation, {}, {}, 0); }
    if (!association(document, root)) { return refusal("document_mismatch", &correlation, {}, {}, 0); }
    if (bytes(expected) != digest) { return refusal("source_mismatch", &correlation, root, digest, text.size()); }
    ctx->root = read_at(session.project_fd, root);
    if (ctx->root.status != "ok") {
        const bool missing = ctx->root.status == "missing";
        return refusal(missing ? "missing_root" : ctx->root.reason.c_str(), &correlation,
                missing ? root : std::string(), missing ? digest : std::string(), missing ? text.size() : 0);
    }
    Value revision = call(nullptr, "GDScript", "gdscript_validation_api_revision", GAK_HASH_GDSCRIPT_REVISION);
    int64_t revision_number = 0;
    if (revision.type() == GDEXTENSION_VARIANT_TYPE_INT) { api.to[GDEXTENSION_VARIANT_TYPE_INT](&revision_number, revision.ptr()); }
    if (revision_number != 1) { return refusal("engine_api_mismatch", &correlation, root, digest, text.size()); }
    Value reader = reader_callable(ctx);
    Value engine = call(nullptr, "GDScript", "validate_gdscript_source", GAK_HASH_GDSCRIPT_VALIDATE,
            {&source, &path, &reader, &correlation});
    if (engine.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY) {
        ctx->active = false;
        return refusal("engine_api_unavailable", &correlation, root, digest, text.size());
    }
    for (const auto &entry : ctx->records) { ctx->read(entry.first, true); }
    ctx->active = false;
    Snapshot after = read_at(session.project_fd, root);
    if (!ctx->project_current() || (session.call_state == Session::CallState::Closing) || after.status != "ok" || after.device != ctx->root.device ||
            after.inode != ctx->root.inode || after.hash != ctx->root.hash || !ctx->failure.empty()) {
        invalidate(engine, ctx->failure.empty() ? "root_changed" : ctx->failure.c_str());
        return engine;
    }
    if (!association(document, root)) { invalidate(engine, "document_changed"); return engine; }
    Value status = get(engine, "status"), actual_path = get(engine, "source_path"), input = get(engine, "input");
    Value actual_hash = get(input, "source_sha256"), actual_bytes = get(input, "utf8_bytes");
    Value engine_request = get(engine, "request_id"), engine_session = get(engine, "session_id");
    Value engine_purpose = get(engine, "purpose"), engine_document = get(engine, "document");
    Value dependencies = get(engine, "dependencies"), diagnostics = get(engine, "diagnostics");
    Value interval = get(engine, "collection"), context = get(engine, "context");
    Value context_current = get(engine, "context_current"), dependencies_current = get(engine, "dependencies_current");
    Value complete = get(engine, "diagnostics_complete");
    int64_t length = -1;
    if (actual_bytes.type() == GDEXTENSION_VARIANT_TYPE_INT) { api.to[GDEXTENSION_VARIANT_TYPE_INT](&length, actual_bytes.ptr()); }
    const std::string result_status = bytes(status, 16);
    if (status.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            (result_status != "valid" && result_status != "invalid" && result_status != "unavailable") ||
            bytes(actual_path, 2048) != root || bytes(actual_hash, 64) != digest ||
            length != static_cast<int64_t>(text.size()) ||
            bytes(engine_request, 64) != bytes(request, 64) ||
            bytes(engine_session, 32) != session.session_id || bytes(engine_purpose, 16) != bytes(purpose, 16) ||
            engine_document.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            bytes(get(engine_document, "resource_instance_id"), 20) != bytes(get(document, "resource_instance_id"), 20) ||
            bytes(get(engine_document, "editor_instance_id"), 20) != bytes(get(document, "editor_instance_id"), 20) ||
            bytes(get(engine_document, "buffer_instance_id"), 20) != bytes(get(document, "buffer_instance_id"), 20) ||
            dependencies.type() != GDEXTENSION_VARIANT_TYPE_ARRAY ||
            diagnostics.type() != GDEXTENSION_VARIANT_TYPE_ARRAY ||
            interval.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            bytes(get(interval, "clock_id"), 16) != "editor" ||
            !decimal_string(get(interval, "started_tick")) ||
            !decimal_string(get(interval, "finished_tick")) ||
            context_current.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            dependencies_current.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            complete.type() != GDEXTENSION_VARIANT_TYPE_BOOL ||
            (result_status != "unavailable" && (context.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
                    !hex_digest(bytes(context, 64))))) {
        return refusal("engine_result_mismatch", &correlation, root, digest, text.size());
    }
    if (result_status != "unavailable" && (!truth(context_current) || !truth(dependencies_current) || !truth(complete))) {
        invalidate(engine, "incomplete_validation");
    }
    return engine;
}
} // namespace gak
