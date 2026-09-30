#include "document_guard.hpp"

#include <sys/acl.h>
#include <fcntl.h>
#include <unistd.h>
#include <algorithm>
#include <cerrno>
#include <charconv>
#include <cstring>
#include <climits>
#include <cstdlib>

namespace gak {
constexpr size_t LIMIT = SOURCE_LIMIT;
Fd::~Fd() { if (value >= 0) { ::close(value); } }
Fd::Fd(Fd &&other) noexcept : value(other.value) { other.value = -1; }
Fd &Fd::operator=(Fd &&other) noexcept {
    if (this != &other) {
        if (value >= 0) { ::close(value); }
        value = other.value;
        other.value = -1;
    }
    return *this;
}

bool number(const Value &v, int64_t &n) {
    if (v.type() != GDEXTENSION_VARIANT_TYPE_INT) { return false; }
    api.to[GDEXTENSION_VARIANT_TYPE_INT](&n, const_cast<void *>(v.ptr()));
    return true;
}
bool decimal(const Value &v, uint64_t &n) {
    if (v.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return false; }
    const std::string s = bytes(v, 20);
    if (s.empty() || (s.size() > 1 && s[0] == '0')) { return false; }
    const auto p = std::from_chars(s.data(), s.data() + s.size(), n);
    return p.ec == std::errc{} && p.ptr == s.data() + s.size();
}
bool component(std::string_view s) {
    return !s.empty() && s != "." && s != ".." && s.size() <= 255 &&
            s.find_first_of("\\:\r\n\0", 0, 5) == s.npos;
}
bool path_ok(const std::string &p) {
    if (p.size() < 10 || p.size() > 2048 || p.compare(0, 6, "res://") ||
            p.compare(p.size() - 3, 3, ".gd") || p.find("::") != p.npos) { return false; }
    for (size_t start = 6; start < p.size();) {
        const size_t end = p.find('/', start);
        if (!component(std::string_view(p).substr(start, end == p.npos ? end : end - start))) { return false; }
        if (end == p.npos) { break; }
        start = end + 1;
    }
    return true;
}
bool secure(int fd, bool directory, bool project = false) {
    struct stat st{};
    if (fstat(fd, &st) || st.st_nlink == 0 || (directory ? !S_ISDIR(st.st_mode) : !S_ISREG(st.st_mode)) ||
            (st.st_uid != geteuid() && (!directory || project || st.st_uid != 0)) ||
            (st.st_mode & (S_IWGRP | S_IWOTH))) { return false; }
    acl_t acl = acl_get_fd_np(fd, ACL_TYPE_EXTENDED);
    if (!acl) { return errno == ENOENT; }
    int cursor = ACL_FIRST_ENTRY;
    while (true) {
        acl_entry_t entry = nullptr;
        const int result = acl_get_entry(acl, cursor, &entry);
        if (result == 0 && entry) {
            acl_tag_t tag{};
            if (acl_get_tag_type(entry, &tag) || tag != ACL_EXTENDED_DENY) { acl_free(acl); return false; }
            cursor = ACL_NEXT_ENTRY;
        } else {
            const bool done = result == -1 && cursor == ACL_NEXT_ENTRY && errno == EINVAL;
            acl_free(acl);
            return done;
        }
    }
}
bool same(const struct stat &a, const struct stat &b) { return a.st_dev == b.st_dev && a.st_ino == b.st_ino; }
bool attached_project(const Session &session) {
    Name settings_name("ProjectSettings");
    Value resource_root = string("res://"), current_root;
    std::string reported;
    char canonical[PATH_MAX];
    if (!checked_call(current_root, api.singleton(settings_name.ptr()), "ProjectSettings", "globalize_path",
            GAK_HASH_GLOBALIZE_PATH, {&resource_root}) || !checked_bytes(current_root, reported, 1025) ||
            reported.empty() || reported.front() != '/' || reported.find('\0') != reported.npos ||
            !realpath(reported.c_str(), canonical) || session.project_path != canonical) { return false; }
    Fd dir(open("/", O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
    if (dir.value < 0) { return false; }
    const std::string &path = session.project_path;
    for (size_t start = 1; start < path.size();) {
        const size_t end = path.find('/', start);
        const std::string part = path.substr(start, end == path.npos ? end : end - start);
        if (!component(part)) { return false; }
        Fd next(openat(dir.value, part.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        if (next.value < 0) { return false; }
        dir = std::move(next);
        if (end == path.npos) { break; }
        start = end + 1;
    }
    struct stat a{}, b{};
    return !fstat(dir.value, &a) && !fstat(session.project_fd, &b) && same(a, b) &&
            a.st_dev == session.device && a.st_ino == session.inode && secure(session.project_fd, true, true);
}
bool file_attached(const Session &session, const FileBinding &file) {
    if (!attached_project(session)) { return false; }
    Fd cursor(dup(session.project_fd));
    if (cursor.value < 0) { return false; }
    for (const Parent &parent : file.parents) {
        Fd next(openat(cursor.value, parent.segment.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        struct stat held{}, found{};
        if (next.value < 0 || fstat(parent.fd.value, &held) || fstat(next.value, &found) ||
                !same(held, found) || held.st_dev != parent.device || held.st_ino != parent.inode ||
                !secure(next.value, true)) { return false; }
        cursor = std::move(next);
    }
    struct stat found{}, held{};
    return !fstatat(cursor.value, file.name.c_str(), &found, AT_SYMLINK_NOFOLLOW) &&
            !fstat(file.leaf.value, &held) && S_ISREG(found.st_mode) && same(found, held) &&
            held.st_dev == file.device && held.st_ino == file.inode && secure(file.leaf.value, false);
}
const char *open_failure() {
    switch (errno) {
        case EACCES: case EPERM: case ELOOP: return "denied_access";
        case ENOENT: case ENOTDIR: return "namespace_or_descriptor_changed";
        default: return "source_unavailable";
    }
}
const char *pin_file(const Session &session, const std::string &path, FileBinding &out, int access) {
    Fd cursor(dup(session.project_fd));
    if (cursor.value < 0) { return "source_unavailable"; }
    if (!attached_project(session)) { return "namespace_or_descriptor_changed"; }
    for (size_t start = 6; start < path.size();) {
        const size_t end = path.find('/', start);
        const std::string part = path.substr(start, end == path.npos ? end : end - start);
        if (end == path.npos) {
            out.name = part;
            out.leaf = Fd(openat(cursor.value, part.c_str(), access | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC));
            if (out.leaf.value < 0) { return open_failure(); }
            struct stat st{};
            if (fstat(out.leaf.value, &st)) { return "source_unavailable"; }
            if (!secure(out.leaf.value, false) || st.st_uid != geteuid()) { return "denied_access"; }
            if (st.st_size < 0 || st.st_size > static_cast<off_t>(LIMIT)) { return "evidence_limit"; }
            out.device = st.st_dev; out.inode = st.st_ino;
            return file_attached(session, out) ? nullptr : "namespace_or_descriptor_changed";
        }
        Fd next(openat(cursor.value, part.c_str(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC));
        if (next.value < 0) { return open_failure(); }
        struct stat st{};
        if (fstat(next.value, &st)) { return "source_unavailable"; }
        if (!secure(next.value, true)) { return "denied_access"; }
        out.parents.push_back(Parent{Fd(dup(next.value)), part, st.st_dev, st.st_ino});
        if (out.parents.back().fd.value < 0) { return "source_unavailable"; }
        cursor = std::move(next);
        start = end + 1;
    }
    return "denied_access";
}
bool content(int fd, std::string_view expected) {
    struct stat st{};
    if (fstat(fd, &st) || st.st_size != static_cast<off_t>(expected.size())) { return false; }
    char block[8192];
    for (size_t i = 0; i < expected.size();) {
        const size_t size = std::min(sizeof(block), expected.size() - i);
        ssize_t n = pread(fd, block, size, static_cast<off_t>(i));
        if (n < 0 && errno == EINTR) { continue; }
        if (n <= 0 || std::memcmp(block, expected.data() + i, static_cast<size_t>(n))) { return false; }
        i += static_cast<size_t>(n);
    }
    struct stat after{};
    return !fstat(fd, &after) && same(st, after) && st.st_size == after.st_size &&
            same_time(st.st_mtimespec, after.st_mtimespec) && same_time(st.st_ctimespec, after.st_ctimespec);
}
// Never dereference an old editor/CodeEdit pointer; obtain live parallel topology afresh.
bool find_document(const std::string &path, Binding &out, bool &found) {
    found = false;
    Name editor_name("EditorInterface"), gd_name("GDScript"), code_name("CodeEdit");
    void *interface = api.singleton(editor_name.ptr());
    if (!interface || !api.class_tag(gd_name.ptr()) || !api.class_tag(code_name.ptr())) { return false; }
    Value owner = call(interface, "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR);
    void *se = object_ptr(owner);
    if (!se) { return false; }
    Value scripts = call(se, "ScriptEditor", "get_open_scripts", GAK_HASH_OPEN_SCRIPTS);
    Value editors = call(se, "ScriptEditor", "get_open_script_editors", GAK_HASH_OPEN_EDITORS);
    if (scripts.type() != GDEXTENSION_VARIANT_TYPE_ARRAY || editors.type() != GDEXTENSION_VARIANT_TYPE_ARRAY) { return false; }
    int64_t sc = 0, ec = 0;
    if (!number(invoke(scripts, "size"), sc) || !number(invoke(editors, "size"), ec) || sc < 0 || sc != ec || sc > 256) { return false; }
    int matches = 0;
    for (int64_t i = 0; i < sc; ++i) {
        auto *a = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](scripts.ptr()), i);
        auto *b = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](editors.ptr()), i);
        if (!a || !b) { return false; }
        Value script(a), editor(b);
        void *sp = object_ptr(script), *ep = object_ptr(editor);
        if (!sp || !ep) { return false; }
        Value location = call(sp, "Resource", "get_path", GAK_HASH_RESOURCE_PATH);
        std::string found_path;
        if (!checked_bytes(location, found_path, 2048)) { return false; }
        if (found_path != path) { continue; }
        if (!api.cast_to(sp, api.class_tag(gd_name.ptr()))) { return false; }
        ++matches;
        Value buffer = call(ep, "ScriptEditorBase", "get_base_editor", GAK_HASH_BASE_EDITOR);
        void *bp = object_ptr(buffer);
        if (!bp || !api.cast_to(bp, api.class_tag(code_name.ptr()))) { return false; }
        out.script = script; out.editor = editor; out.buffer = buffer;
        out.script_ptr = sp; out.editor_ptr = ep; out.buffer_ptr = bp;
        out.script_id = id(script); out.editor_id = id(editor); out.buffer_id = id(buffer);
    }
    found = matches == 1;
    return matches <= 1;
}
bool resolve(const std::string &path, uint64_t rid, uint64_t eid, uint64_t bid, Binding &out) {
    bool found = false;
    return find_document(path, out, found) && found &&
            out.script_id == rid && out.editor_id == eid && out.buffer_id == bid;
}
uint64_t ticks() {
    Name name("Time");
    void *time = api.singleton(name.ptr());
    int64_t current = -1;
    return time && number(call(time, "Time", "get_ticks_usec", 3905245786ULL), current) && current >= 0
            ? static_cast<uint64_t>(current) : UINT64_MAX;
}
bool valid_request(const std::string &id) {
    if (id.empty() || id.size() > 64) { return false; }
    for (char c : id) { if (!((c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') ||
            (c >= 'a' && c <= 'z') || c == '_' || c == '-')) { return false; } }
    return true;
}
} // namespace gak
