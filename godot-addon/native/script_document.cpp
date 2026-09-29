#include "native.hpp"

#include <sys/acl.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <algorithm>
#include <cerrno>
#include <charconv>
#include <climits>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

namespace gak {
namespace {
constexpr size_t LIMIT = 512 * 1024;
struct Fd {
    int value = -1;
    explicit Fd(int n = -1) : value(n) {}
    ~Fd() { if (value >= 0) { ::close(value); } }
    Fd(const Fd &) = delete;
    Fd &operator=(const Fd &) = delete;
    Fd(Fd &&other) noexcept : value(other.value) { other.value = -1; }
    Fd &operator=(Fd &&other) noexcept {
        if (this != &other) {
            if (value >= 0) { ::close(value); }
            value = other.value;
            other.value = -1;
        }
        return *this;
    }
};

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
// A retained parent chain detects a same-path parent replacement even if the leaf is relinked.
struct Parent { Fd fd; std::string segment; dev_t device{}; ino_t inode{}; };
struct FileBinding {
    std::vector<Parent> parents;
    Fd leaf;
    std::string name;
    dev_t device{}; ino_t inode{};
    timespec original_mtime{};
};
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
const char *open_file(const Session &session, const std::string &path, FileBinding &out) {
    Fd cursor(dup(session.project_fd));
    if (cursor.value < 0) { return "source_unavailable"; }
    if (!attached_project(session)) { return "namespace_or_descriptor_changed"; }
    for (size_t start = 6; start < path.size();) {
        const size_t end = path.find('/', start);
        const std::string part = path.substr(start, end == path.npos ? end : end - start);
        if (end == path.npos) {
            out.name = part;
            out.leaf = Fd(openat(cursor.value, part.c_str(), O_RDWR | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC));
            if (out.leaf.value < 0) { return open_failure(); }
            struct stat st{};
            if (fstat(out.leaf.value, &st)) { return "source_unavailable"; }
            if (!secure(out.leaf.value, false) || st.st_uid != geteuid()) { return "denied_access"; }
            if (st.st_size < 0 || st.st_size > static_cast<off_t>(LIMIT)) { return "evidence_limit"; }
            out.device = st.st_dev; out.inode = st.st_ino; out.original_mtime = st.st_mtimespec;
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
    return true;
}
struct Profile {
    bool trim_space{}, trim_newlines{}, convert{}, spaces{};
    int64_t indent{};
    bool operator==(const Profile &o) const {
        return trim_space == o.trim_space && trim_newlines == o.trim_newlines && convert == o.convert &&
                spaces == o.spaces && indent == o.indent;
    }
};
bool compatible(const Profile &profile, std::string_view s) {
    if (profile.trim_newlines && (s == "\n" ||
            (s.size() >= 2 && s.substr(s.size() - 2) == "\n\n"))) { return false; }
    bool beginning = true;
    int64_t leading = 0;
    for (size_t i = 0; i < s.size(); ++i) {
        const char c = s[i];
        if (c == '\n') { beginning = true; leading = 0; continue; }
        if (profile.trim_space && (c == ' ' || c == '\t') && (i + 1 == s.size() || s[i + 1] == '\n')) { return false; }
        if (beginning && (c == ' ' || c == '\t')) {
            if (profile.convert && (profile.spaces ? c == '\t' : c == ' ')) {
                if (c == '\t' || ++leading >= profile.indent) { return false; }
            } else if (c == '\t') { leading = 0; }
        } else { beginning = false; leading = 0; }
    }
    return true;
}
bool profile(void *buffer, Profile &out) {
    Name name("EditorInterface");
    void *interface = api.singleton(name.ptr());
    if (!interface) { return false; }
    Value settings = call(interface, "EditorInterface", "get_editor_settings", GAK_HASH_EDITOR_SETTINGS);
    void *instance = object_ptr(settings);
    if (!instance) { return false; }
    const char *keys[] = {
        "text_editor/behavior/files/trim_trailing_whitespace_on_save",
        "text_editor/behavior/files/trim_final_newlines_on_save",
        "text_editor/behavior/files/convert_indent_on_save"
    };
    bool *targets[] = {&out.trim_space, &out.trim_newlines, &out.convert};
    for (size_t i = 0; i < 3; ++i) {
        Value key = string(keys[i]);
        Value setting = call(instance, "EditorSettings", "get_setting", GAK_HASH_GET_SETTING, {&key});
        if (setting.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return false; }
        *targets[i] = truth(setting);
    }
    Value spaces = call(buffer, "CodeEdit", "is_indent_using_spaces", GAK_HASH_INDENT_SPACES);
    Value indent = call(buffer, "CodeEdit", "get_indent_size", GAK_HASH_INDENT_SIZE);
    if (spaces.type() != GDEXTENSION_VARIANT_TYPE_BOOL || !number(indent, out.indent) ||
            out.indent < 1 || out.indent > 1024) { return false; }
    out.spaces = truth(spaces);
    return true;
}
std::string profile_digest(const Profile &p) {
    char serialized[32] = {p.trim_space ? '1' : '0', ':', p.trim_newlines ? '1' : '0', ':',
            p.convert ? '1' : '0', ':', p.spaces ? '1' : '0', ':'};
    const auto result = std::to_chars(serialized + 8, serialized + sizeof(serialized), p.indent);
    return result.ec == std::errc{} ? sha256(std::string_view(serialized, result.ptr - serialized)) : std::string{};
}
struct Binding {
    Value script, editor, buffer;
    void *script_ptr = nullptr, *editor_ptr = nullptr, *buffer_ptr = nullptr;
};
// Never dereference an old editor/CodeEdit pointer; obtain live parallel topology afresh.
bool resolve(const std::string &path, uint64_t rid, uint64_t eid, uint64_t bid, Binding &out) {
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
    if (!number(invoke(scripts, "size"), sc) || !number(invoke(editors, "size"), ec) || sc < 1 || sc != ec || sc > 256) { return false; }
    int matches = 0;
    for (int64_t i = 0; i < sc; ++i) {
        auto *a = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](scripts.ptr()), i);
        auto *b = api.array_index_const(api.internal[GDEXTENSION_VARIANT_TYPE_ARRAY](editors.ptr()), i);
        if (!a || !b) { return false; }
        Value script(a), editor(b);
        void *sp = object_ptr(script), *ep = object_ptr(editor);
        if (!sp || !ep) { return false; }
        Value location = call(sp, "Resource", "get_path", GAK_HASH_RESOURCE_PATH);
        if (location.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return false; }
        const std::string found = bytes(location, 2048);
        if (found != path) { continue; }
        if (!api.cast_to(sp, api.class_tag(gd_name.ptr()))) { return false; }
        ++matches;
        if (id(script) != rid || id(editor) != eid) { return false; }
        Value buffer = call(ep, "ScriptEditorBase", "get_base_editor", GAK_HASH_BASE_EDITOR);
        void *bp = object_ptr(buffer);
        if (!bp || !api.cast_to(bp, api.class_tag(code_name.ptr())) || id(buffer) != bid) { return false; }
        out.script = script; out.editor = editor; out.buffer = buffer;
        out.script_ptr = sp; out.editor_ptr = ep; out.buffer_ptr = bp;
    }
    return matches == 1 && out.script_ptr && out.buffer_ptr;
}
bool unsaved(const std::string &path) {
    Name name("EditorInterface");
    void *interface = api.singleton(name.ptr());
    if (!interface) { return true; }
    Value editor = call(interface, "EditorInterface", "get_script_editor", GAK_HASH_SCRIPT_EDITOR);
    if (!object_ptr(editor)) { return true; }
    Value files = call(object_ptr(editor), "ScriptEditor", "get_unsaved_files", GAK_HASH_UNSAVED);
    if (files.type() != GDEXTENSION_VARIANT_TYPE_PACKED_STRING_ARRAY) { return true; }
    Value target = string(path);
    Value exists = invoke(files, "has", {&target});
    return exists.type() != GDEXTENSION_VARIANT_TYPE_BOOL || truth(exists);
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
} // namespace

struct EditAttempt {
    explicit EditAttempt(Value held) : script(std::move(held)) {}
    std::string request, session, path, before, after, before_hash, after_hash, fault;
    uint64_t rid{}, eid{}, bid{}, expires{};
    dev_t project_device{};
    ino_t project_inode{};
    int64_t prepared_current{}, prepared_saved{}, frozen_current{};
    Profile save_profile;
    FileBinding file;
    Value script;
    enum Stage { Prepared, Buffer, Resource, Content, Metadata, Edited, Tagged } stage = Prepared;
    bool buffer_changed{}, resource_changed{}, write_started{}, truncated{}, synced{}, content_readback{},
            futimens_called{}, metadata_after_observed{}, restored{}, edited_clear_attempted{}, edited_cleared{}, tag_attempted{}, tagged{};
    int write_calls{}, write_errno{}, truncate_errno{}, fsync_errno{}, pread_errno{}, futimens_errno{};
    size_t written{};
    timespec before_restore{}, after_restore{};
};
namespace {
const char *stage_name(EditAttempt::Stage stage) {
    switch (stage) {
        case EditAttempt::Prepared: return "prepared";
        case EditAttempt::Buffer: return "buffer_applied";
        case EditAttempt::Resource: return "resource_applied";
        case EditAttempt::Content: return "content_persisted";
        case EditAttempt::Metadata: return "mtime_restored";
        case EditAttempt::Edited: return "edited_cleared";
        case EditAttempt::Tagged: return "saved_tagged";
    }
    return "unknown";
}
void time_fact(Value &v, const char *field, timespec t) {
    Value stamp = dict();
    put_string(stamp, "seconds", std::to_string(t.tv_sec));
    put_string(stamp, "nanoseconds", std::to_string(t.tv_nsec));
    put(v, field, stamp);
}
Value reply(const EditAttempt *attempt, const char *status, const char *reason) {
    Value out = dict();
    put_string(out, "status", status);
    put_string(out, "reason", reason);
    if (!attempt) { return out; }
    put_string(out, "stage", stage_name(attempt->stage));
    Value facts = dict();
    put_string(facts, "expected_sha256", attempt->before_hash);
    put_string(facts, "desired_sha256", attempt->after_hash);
    put_string(facts, "project_device", std::to_string(attempt->project_device));
    put_string(facts, "project_inode", std::to_string(attempt->project_inode));
    put_string(facts, "target_device", std::to_string(attempt->file.device));
    put_string(facts, "target_inode", std::to_string(attempt->file.inode));
    put_string(facts, "prepared_current", std::to_string(attempt->prepared_current));
    put_string(facts, "prepared_saved", std::to_string(attempt->prepared_saved));
    if (attempt->stage >= EditAttempt::Buffer) {
        put_string(facts, "frozen_current", std::to_string(attempt->frozen_current));
    }
    put(facts, "buffer_changed", boolean(attempt->buffer_changed));
    put(facts, "resource_changed", boolean(attempt->resource_changed));
    put(facts, "write_started", boolean(attempt->write_started));
    put_string(facts, "write_calls", std::to_string(attempt->write_calls));
    put_string(facts, "written_bytes", std::to_string(attempt->written));
    put_string(facts, "write_errno", std::to_string(attempt->write_errno));
    put(facts, "truncate_done", boolean(attempt->truncated));
    put_string(facts, "truncate_errno", std::to_string(attempt->truncate_errno));
    put(facts, "fsync_done", boolean(attempt->synced));
    put_string(facts, "fsync_errno", std::to_string(attempt->fsync_errno));
    put(facts, "pread_done", boolean(attempt->content_readback));
    put_string(facts, "pread_errno", std::to_string(attempt->pread_errno));
    put(facts, "futimens_called", boolean(attempt->futimens_called));
    put_string(facts, "futimens_errno", std::to_string(attempt->futimens_errno));
    put(facts, "mtime_restored", boolean(attempt->restored));
    put(facts, "edited_clear_attempted", boolean(attempt->edited_clear_attempted));
    put(facts, "edited_cleared", boolean(attempt->edited_cleared));
    put(facts, "tag_attempted", boolean(attempt->tag_attempted));
    put(facts, "tagged", boolean(attempt->tagged));
#if GAK_FIXTURE
    if (!attempt->fault.empty()) { put_string(facts, "injected_fault", attempt->fault); }
#endif
    time_fact(facts, "t0", attempt->file.original_mtime);
    if (attempt->futimens_called) { time_fact(facts, "before_restore", attempt->before_restore); }
    if (attempt->metadata_after_observed) { time_fact(facts, "after_restore", attempt->after_restore); }
    put(out, "facts", facts);
    return out;
}
const char *failure_status(const EditAttempt &attempt) {
    return attempt.buffer_changed || attempt.resource_changed || attempt.write_started || attempt.edited_clear_attempted || attempt.tag_attempted
            ? "partial" : "refused";
}
const char *guard(Session &session, EditAttempt &a, Binding &binding, bool needs_content, bool needs_metadata) {
    if (session.call_state != Session::CallState::Idle || session.session_id != a.session || ticks() >= a.expires) {
        return "session_or_expiry_changed";
    }
    if (!file_attached(session, a.file)) { return "namespace_or_descriptor_changed"; }
    if (!resolve(a.path, a.rid, a.eid, a.bid, binding) || id(a.script) != id(binding.script)) {
        return "document_association_changed";
    }
    Profile current;
    if (!profile(binding.buffer_ptr, current) || !(current == a.save_profile) ||
            !compatible(current, a.before) || !compatible(current, a.after)) { return "save_profile_changed"; }
    int64_t version = -1, saved = -1;
    if (!number(call(binding.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), version) ||
            !number(call(binding.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved)) {
        return "version_unavailable";
    }
    if (version != (a.stage == EditAttempt::Prepared ? a.prepared_current : a.frozen_current) ||
            saved != a.prepared_saved) { return "version_changed"; }
    Value b = call(binding.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT);
    Value r = call(binding.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE);
    if (b.type() != GDEXTENSION_VARIANT_TYPE_STRING || r.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            utf8_size(b) < 0 || utf8_size(r) < 0 || utf8_size(b) > static_cast<GDExtensionInt>(LIMIT) ||
            utf8_size(r) > static_cast<GDExtensionInt>(LIMIT)) { return "source_unavailable"; }
    if (bytes(b) != (a.stage == EditAttempt::Prepared ? a.before : a.after) ||
            bytes(r) != (a.stage <= EditAttempt::Buffer ? a.before : a.after)) { return "source_changed"; }
    if (a.stage >= EditAttempt::Edited) {
        Name name("EditorInterface");
        void *interface = api.singleton(name.ptr());
        if (!interface) { return "edited_state_unavailable"; }
        Value edited = call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&binding.script});
        if (edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return "edited_state_unavailable"; }
        if (truth(edited)) { return "edited_state_changed"; }
    }
    if (needs_content) {
        if (!a.write_started || !a.truncated || !a.synced || !a.content_readback) { return "incomplete_content_receipt"; }
        if (!content(a.file.leaf.value, a.after)) { return "disk_content_changed"; }
    } else if (!content(a.file.leaf.value, a.before)) { return "disk_content_changed"; }
    if (needs_metadata) {
        struct stat st{};
        if (!a.restored || fstat(a.file.leaf.value, &st) || !same_time(st.st_mtimespec, a.file.original_mtime)) {
            return "mtime_changed";
        }
    }
    return nullptr;
}
// TextEdit emits lines_edited_from *inside* removal, before it records the
// remove operation. A callback can edit or retag the buffer while this call
// is on the stack. Check the exact intermediate state before inserting.
const char *after_remove(Session &session, EditAttempt &a, Binding &original) {
    if (session.call_state != Session::CallState::Running || session.session_id != a.session ||
            ticks() >= a.expires) { return "cancelled_or_expired_during_edit"; }
    if (!file_attached(session, a.file) || !content(a.file.leaf.value, a.before)) {
        return "disk_or_namespace_changed";
    }
    Binding live;
    if (!resolve(a.path, a.rid, a.eid, a.bid, live) || id(a.script) != id(live.script) ||
            live.buffer_ptr != original.buffer_ptr) { return "document_association_changed"; }
    Profile current;
    Name editor_name("EditorInterface");
    void *interface = api.singleton(editor_name.ptr());
    Value edited;
    if (interface) { edited = call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&live.script}); }
    if (!interface || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL || truth(edited)) {
        return "edited_state_changed";
    }
    if (!profile(live.buffer_ptr, current) || !(current == a.save_profile)) { return "save_profile_changed"; }
    int64_t version = -1, saved = -1;
    Value text = call(live.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT);
    Value resource = call(live.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE);
    if (!number(call(live.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), version) ||
            !number(call(live.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved) ||
            version != a.prepared_current + 1 || saved != a.prepared_saved ||
            text.type() != GDEXTENSION_VARIANT_TYPE_STRING || !bytes(text).empty() ||
            resource.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(resource) != a.before) {
        return "intermediate_source_or_version_changed";
    }
    return nullptr;
}
// A dependency referenced by the script can be compiled/initialized by the
// queued ordinary editor validation/export update, unlike the isolated
// source-only helper. Admit built-in bases and sources without live dependency
// loads, exported defaults, tool initialization or global-class registration;
// never infer purity from a helper's parser result.
bool editor_effects_inert(std::string_view source) {
    for (size_t i = 0; i < source.size();) {
        if (source[i] == '#') {
            i = source.find('\n', i);
            if (i == source.npos) { break; }
        } else if (source[i] == '\'' || source[i] == '"') {
            const char quote = source[i++];
            const bool triple = i + 1 < source.size() && source[i] == quote && source[i + 1] == quote;
            if (triple) { i += 2; }
            bool closed = false;
            while (i < source.size()) {
                if (source[i] == '\\') { i += std::min<size_t>(2, source.size() - i); continue; }
                if (source[i] == quote &&
                        (!triple || (i + 2 < source.size() && source[i + 1] == quote && source[i + 2] == quote))) {
                    i += triple ? 3 : 1;
                    closed = true;
                    break;
                }
                ++i;
            }
            if (!closed) { return false; }
        } else if ((source[i] >= 'A' && source[i] <= 'Z') ||
                (source[i] >= 'a' && source[i] <= 'z') || source[i] == '_') {
            const size_t begin = i++;
            while (i < source.size() && ((source[i] >= 'A' && source[i] <= 'Z') ||
                    (source[i] >= 'a' && source[i] <= 'z') || (source[i] >= '0' && source[i] <= '9') ||
                    source[i] == '_')) { ++i; }
            const std::string_view token = source.substr(begin, i - begin);
            if (token == "tool" && begin && source[begin - 1] == '@') { return false; }
            if (token.size() >= 6 && token.substr(0, 6) == "export" && begin && source[begin - 1] == '@') { return false; }
            if (token == "class_name") { return false; }
            if (token == "extends") {
                size_t next = i;
                while (next < source.size() && (source[next] == ' ' || source[next] == '\t')) { ++next; }
                if (next >= source.size() || !((source[next] >= 'A' && source[next] <= 'Z') ||
                        (source[next] >= 'a' && source[next] <= 'z') || source[next] == '_')) { return false; }
                const size_t start = next++;
                while (next < source.size() && ((source[next] >= 'A' && source[next] <= 'Z') ||
                        (source[next] >= 'a' && source[next] <= 'z') ||
                        (source[next] >= '0' && source[next] <= '9') || source[next] == '_')) { ++next; }
                Name base(std::string(source.substr(start, next - start)).c_str());
                if (!api.class_tag(base.ptr()) || (next < source.size() && source[next] == '.')) { return false; }
            }
            // A following call may be separated by comments or continued
            // lines. References to these loaders are outside this profile.
            if (token == "preload" || token == "load") { return false; }
        } else {
            ++i;
        }
    }
    return true;
}
} // namespace

void edit_cleanup(Session &session) {
    delete session.attempt;
    session.attempt = nullptr;
}
Value edit_inspect(Session &session, const Value &path_value, const Value &original_value,
        const Value &desired_value, const Value &document) {
    Value out = reply(nullptr, "refused", "inspection_unavailable");
    if (session.project_fd < 0 || std::this_thread::get_id() != session.main_thread ||
            session.call_state != Session::CallState::Idle ||
            path_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            original_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            desired_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            utf8_size(original_value) > static_cast<GDExtensionInt>(LIMIT) ||
            utf8_size(desired_value) > static_cast<GDExtensionInt>(LIMIT)) { return out; }
    const std::string path = bytes(path_value, 2048);
    uint64_t rid = 0, eid = 0, bid = 0;
    if (!path_ok(path) || !decimal(get(document, "resource_instance_id"), rid) ||
            !decimal(get(document, "editor_instance_id"), eid) ||
            !decimal(get(document, "buffer_instance_id"), bid) || !attached_project(session)) { return out; }
    Binding bound;
    if (!resolve(path, rid, eid, bid, bound)) { return out; }
    Profile settings;
    int64_t current = -1, saved = -1;
    Name editor_name("EditorInterface");
    void *interface = api.singleton(editor_name.ptr());
    Value edited = interface ? call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED,
            {&bound.script}) : Value{};
    if (!profile(bound.buffer_ptr, settings) ||
            !number(call(bound.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), current) ||
            !number(call(bound.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved) ||
            current < 0 || saved < 0 || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) { return out; }
    out = reply(nullptr, "observed", "");
    Value facts = dict();
    put_string(facts, "current_version", std::to_string(current));
    put_string(facts, "saved_version", std::to_string(saved));
    put(facts, "resource_edited", boolean(truth(edited)));
    put_string(facts, "save_profile", profile_digest(settings));
    put(facts, "original_preserved", boolean(compatible(settings, bytes(original_value))));
    put(facts, "desired_preserved", boolean(compatible(settings, bytes(desired_value))));
    put(out, "facts", facts);
    return out;
}
Value edit_prepare(Session &session, const Value &path_value, const Value &original_value,
        const Value &desired_value, const Value &correlation) {
    if (session.attempt || session.call_state != Session::CallState::Idle) { return reply(nullptr, "busy", "slot_busy"); }
    if (session.project_fd < 0 || std::this_thread::get_id() != session.main_thread ||
            path_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            original_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            desired_value.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            correlation.type() != GDEXTENSION_VARIANT_TYPE_DICTIONARY ||
            utf8_size(path_value) > 2048 || utf8_size(original_value) > static_cast<GDExtensionInt>(LIMIT) ||
            utf8_size(desired_value) > static_cast<GDExtensionInt>(LIMIT)) { return reply(nullptr, "refused", "invalid_arguments"); }
    const std::string path = bytes(path_value, 2048);
    std::string original = bytes(original_value), desired = bytes(desired_value);
    if (!path_ok(path) || original == desired || original.find('\0') != original.npos ||
            desired.find('\0') != desired.npos || original.find('\r') != original.npos ||
            desired.find('\r') != desired.npos || original.find("\xEF\xBB\xBF") != original.npos ||
            desired.find("\xEF\xBB\xBF") != desired.npos || !valid_utf8(original) || !valid_utf8(desired)) {
        return reply(nullptr, "refused", "unsupported_source");
    }
    Value request = get(correlation, "request_id"), sid = get(correlation, "session_id");
    Value document = get(correlation, "document"), expiry = get(correlation, "expiry_tick_us");
    uint64_t rid = 0, eid = 0, bid = 0, expires = 0;
    uint64_t project_device = 0, project_inode = 0, file_device = 0, file_inode = 0, expected_version = 0;
    uint64_t expected_length = 0;
    const uint64_t now = ticks();
    Value expected_hash = get(correlation, "expected_sha256");
    if (request.type() != GDEXTENSION_VARIANT_TYPE_STRING || !valid_request(bytes(request, 64)) ||
            sid.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(sid, 32) != session.session_id ||
            !decimal(expiry, expires) || now == UINT64_MAX || expires <= now || expires - now > 9000000 ||
            !decimal(get(document, "resource_instance_id"), rid) ||
            !decimal(get(document, "editor_instance_id"), eid) ||
            !decimal(get(document, "buffer_instance_id"), bid) ||
            !decimal(get(correlation, "project_device"), project_device) ||
            !decimal(get(correlation, "project_inode"), project_inode) ||
            !decimal(get(correlation, "target_device"), file_device) ||
            !decimal(get(correlation, "target_inode"), file_inode) ||
            !decimal(get(correlation, "expected_version"), expected_version) ||
            !decimal(get(correlation, "expected_length"), expected_length) ||
            expected_hash.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            bytes(expected_hash, 64) != sha256(original) || expected_length != original.size() ||
            project_device != static_cast<uint64_t>(session.device) ||
            project_inode != static_cast<uint64_t>(session.inode)) {
        return reply(nullptr, "refused", "invalid_binding");
    }
    Binding binding;
    if (!resolve(path, rid, eid, bid, binding)) { return reply(nullptr, "refused", "document_mismatch"); }
    if (!editor_effects_inert(original) || !editor_effects_inert(desired)) {
        return reply(nullptr, "refused", "unsafe_editor_effects");
    }
    auto attempt = std::make_unique<EditAttempt>(binding.script);
    attempt->request = bytes(request, 64); attempt->session = session.session_id; attempt->path = path;
    attempt->before_hash = sha256(original);
    attempt->after_hash = sha256(desired);
    attempt->before = std::move(original); attempt->after = std::move(desired);
    attempt->rid = rid; attempt->eid = eid; attempt->bid = bid; attempt->expires = expires;
    attempt->project_device = session.device; attempt->project_inode = session.inode;
    Name editor_name("EditorInterface");
    void *interface = api.singleton(editor_name.ptr());
    Value edited;
    if (interface) { edited = call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&binding.script}); }
    if (!interface || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL) {
        return reply(nullptr, "refused", "edited_state_unavailable");
    }
    if (truth(edited) || unsaved(path)) { return reply(nullptr, "refused", "dirty"); }
    if (const char *reason = open_file(session, path, attempt->file)) {
        return reply(nullptr, "refused", reason);
    }
    if (static_cast<uint64_t>(attempt->file.device) != file_device ||
            static_cast<uint64_t>(attempt->file.inode) != file_inode) {
        return reply(nullptr, "refused", "namespace_or_descriptor_changed");
    }
    if (!content(attempt->file.leaf.value, attempt->before)) {
        return reply(nullptr, "refused", "disk_content_changed");
    }
    if (!profile(binding.buffer_ptr, attempt->save_profile)) {
        return reply(nullptr, "refused", "source_unavailable");
    }
    if (!compatible(attempt->save_profile, attempt->before) ||
            !compatible(attempt->save_profile, attempt->after)) {
        return reply(nullptr, "refused", "save_would_reformat");
    }
    int64_t version = -1, saved = -1;
    Value b = call(binding.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT);
    Value r = call(binding.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE);
    if (b.type() != GDEXTENSION_VARIANT_TYPE_STRING || r.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            bytes(b) != attempt->before || bytes(r) != attempt->before ||
            !number(call(binding.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), version) ||
            !number(call(binding.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved) ||
            version < 0 || version != saved || static_cast<uint64_t>(version) != expected_version) {
        return reply(nullptr, "refused", "dirty_or_changed");
    }
    attempt->prepared_current = version; attempt->prepared_saved = saved;
    Binding again;
    if (const char *reason = guard(session, *attempt, again, false, false)) {
        return reply(nullptr, "refused", reason);
    }
    session.attempt = attempt.release();
    return reply(session.attempt, "prepared", "");
}

Value edit_advance(Session &session, const Value &request, const Value &stage) {
    if (session.call_state != Session::CallState::Idle) { return reply(nullptr, "busy", "slot_busy"); }
    EditAttempt *a = session.attempt;
    if (!a || request.type() != GDEXTENSION_VARIANT_TYPE_STRING || stage.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            bytes(request, 64) != a->request || bytes(stage, 32) != stage_name(a->stage)) {
        return reply(nullptr, "refused", "wrong_attempt_or_stage");
    }
    auto finish = [&](const char *status, const char *reason) {
        const bool interrupted = session.call_state == Session::CallState::Closing;
        const char *effective = interrupted ? failure_status(*a) : status;
        Value result = reply(a, effective, interrupted ? "cancelled_during_stage" : reason);
        if (std::strcmp(effective, "ready") != 0) { edit_cleanup(session); }
        return result;
    };
    Binding binding;
    const bool has_content = a->stage >= EditAttempt::Content;
    const bool has_metadata = a->stage >= EditAttempt::Metadata;
    if (const char *reason = guard(session, *a, binding, has_content, has_metadata)) {
        return finish(failure_status(*a), reason);
    }
    session.call_state = Session::CallState::Running;
    struct StageCall {
        Session &session;
        ~StageCall() {
            const bool closing = session.call_state == Session::CallState::Closing;
            session.call_state = Session::CallState::Idle;
            if (closing) { close(session); }
        }
    } stage_call{session};
    switch (a->stage) {
        case EditAttempt::Prepared: {
            // Removal and insertion are one native history entry, never TextEdit.set_text.
            int64_t lines = 0;
            if (!number(call(binding.buffer_ptr, "TextEdit", "get_line_count", GAK_HASH_LINE_COUNT), lines) ||
                    lines <= 0 || lines > static_cast<int64_t>(LIMIT + 1)) { return finish("refused", "invalid_buffer"); }
            Value line = integer(lines - 1);
            Value last = call(binding.buffer_ptr, "TextEdit", "get_line", GAK_HASH_LINE, {&line});
            if (last.type() != GDEXTENSION_VARIANT_TYPE_STRING) { return finish("refused", "invalid_buffer"); }
            // Godot columns count Unicode scalar code points, not UTF-8 bytes.
            Value length = invoke(last, "length");
            int64_t column = 0;
            if (!number(length, column)) { return finish("refused", "invalid_buffer"); }
            Value zero = integer(0), column_value = integer(column), desired = string(a->after);
            a->buffer_changed = true; // Potentially applied before entering the native complex operation.
            call(binding.buffer_ptr, "TextEdit", "begin_complex_operation", GAK_HASH_COMPLEX_BEGIN);
            if (api.from_id(a->bid) != binding.buffer_ptr) { return finish("partial", "document_closed_during_edit"); }
            if (session.call_state == Session::CallState::Closing) {
                call(binding.buffer_ptr, "TextEdit", "end_complex_operation", GAK_HASH_COMPLEX_END);
                return finish("partial", "cancelled_during_edit");
            }
            call(binding.buffer_ptr, "TextEdit", "remove_text", GAK_HASH_REMOVE_TEXT, {&zero, &zero, &line, &column_value});
            if (api.from_id(a->bid) != binding.buffer_ptr) { return finish("partial", "document_closed_during_edit"); }
            if (const char *reason = after_remove(session, *a, binding)) {
                // Close our own history bracket even if a callback changed the
                // buffer. Never insert the candidate into its newer source.
                call(binding.buffer_ptr, "TextEdit", "end_complex_operation", GAK_HASH_COMPLEX_END);
                return finish("partial", reason);
            }
            call(binding.buffer_ptr, "TextEdit", "insert_text", GAK_HASH_INSERT_TEXT, {&desired, &zero, &zero});
            if (api.from_id(a->bid) != binding.buffer_ptr) { return finish("partial", "document_closed_during_edit"); }
            call(binding.buffer_ptr, "TextEdit", "end_complex_operation", GAK_HASH_COMPLEX_END);
            a->stage = EditAttempt::Buffer;
            Binding after;
            if (!resolve(a->path, a->rid, a->eid, a->bid, after) || id(a->script) != id(after.script)) {
                return finish("partial", "document_association_changed");
            }
            Value text = call(after.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT);
            int64_t saved = -1;
            if (!number(call(after.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), a->frozen_current) ||
                    !number(call(after.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved) ||
                    a->frozen_current <= a->prepared_current || saved != a->prepared_saved ||
                    text.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(text) != a->after) {
                return finish("partial", "buffer_readback_failed");
            }
            break;
        }
        case EditAttempt::Buffer: {
            Value desired = string(a->after);
            a->resource_changed = true;
            call(binding.script_ptr, "Script", "set_source_code", GAK_HASH_SCRIPT_SET_SOURCE, {&desired});
            a->stage = EditAttempt::Resource;
            Binding after;
            if (!resolve(a->path, a->rid, a->eid, a->bid, after) || id(after.script) != id(a->script)) {
                return finish("partial", "document_association_changed");
            }
            Value source = call(after.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE);
            if (source.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(source) != a->after) {
                return finish("partial", "resource_readback_failed");
            }
            break;
        }
        case EditAttempt::Resource: {
            a->write_started = true;
            for (size_t offset = 0; offset < a->after.size();) {
                ssize_t n;
#if GAK_FIXTURE
                if (a->fault == "fail_pwrite" || a->fault == "read_only") { errno = a->fault == "read_only" ? EROFS : EIO; n = -1; }
                else if (a->fault == "short_write" && offset == 0) {
                    n = pwrite(a->file.leaf.value, a->after.data(), 1, 0);
                } else
#endif
                { n = pwrite(a->file.leaf.value, a->after.data() + offset, a->after.size() - offset, static_cast<off_t>(offset)); }
                ++a->write_calls;
                if (n < 0 && errno == EINTR) { continue; }
                if (n <= 0) { a->write_errno = n < 0 ? errno : EIO; return finish("partial", "write_failed"); }
                offset += static_cast<size_t>(n); a->written += static_cast<size_t>(n);
            }
#if GAK_FIXTURE
            if (a->fault == "fail_truncate") { errno = EIO; a->truncate_errno = errno; return finish("partial", "truncate_failed"); }
#endif
            if (ftruncate(a->file.leaf.value, static_cast<off_t>(a->after.size()))) {
                a->truncate_errno = errno; return finish("partial", "truncate_failed");
            }
            a->truncated = true;
#if GAK_FIXTURE
            if (a->fault == "fail_fsync") { errno = EIO; a->fsync_errno = errno; return finish("partial", "fsync_failed"); }
#endif
            if (fsync(a->file.leaf.value)) { a->fsync_errno = errno; return finish("partial", "fsync_failed"); }
            a->synced = true;
#if GAK_FIXTURE
            if (a->fault == "fail_pread") { a->pread_errno = EIO; return finish("partial", "pread_failed"); }
#endif
            if (!content(a->file.leaf.value, a->after) || !file_attached(session, a->file)) {
                a->pread_errno = errno ? errno : EIO; return finish("partial", "pread_or_namespace_changed");
            }
            a->content_readback = true; a->stage = EditAttempt::Content;
            break;
        }
        case EditAttempt::Content: {
            struct stat before{};
            if (fstat(a->file.leaf.value, &before)) { return finish("partial", "metadata_preflight_failed"); }
            a->before_restore = before.st_mtimespec;
#if GAK_FIXTURE
            if (a->fault == "close_fd_before_restore") {
                ::close(a->file.leaf.value); a->file.leaf.value = -1;
            }
#endif
            timespec times[2] = {{0, UTIME_OMIT}, a->file.original_mtime};
#if GAK_FIXTURE
            if (a->fault == "mismatch_mtime") {
                if (++times[1].tv_nsec == 1000000000) { times[1].tv_nsec = 0; ++times[1].tv_sec; }
            }
#endif
            a->futimens_called = true;
#if GAK_FIXTURE
            if (a->fault == "fail_futimens") { errno = EIO; a->futimens_errno = errno; return finish("partial", "mtime_restore_failed"); }
#endif
            if (futimens(a->file.leaf.value, times)) {
                a->futimens_errno = errno; return finish("partial", "mtime_restore_failed");
            }
            struct stat after{};
            if (!fstat(a->file.leaf.value, &after)) {
                a->after_restore = after.st_mtimespec;
                a->metadata_after_observed = true;
            }
            if (!a->metadata_after_observed || !same_time(after.st_mtimespec, a->file.original_mtime) ||
                    !same(after, before) || !content(a->file.leaf.value, a->after) || !file_attached(session, a->file)) {
                return finish("partial", "mtime_readback_failed");
            }
            a->restored = true; a->stage = EditAttempt::Metadata;
            break;
        }
        case EditAttempt::Metadata: {
            Name name("EditorInterface");
            void *interface = api.singleton(name.ptr());
            if (!interface) { return finish("partial", "editor_unavailable"); }
            Value no = boolean(false);
            a->edited_clear_attempted = true;
            call(interface, "EditorInterface", "set_object_edited", GAK_HASH_SET_EDITED, {&binding.script, &no});
            a->stage = EditAttempt::Edited;
            Value edited = call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&binding.script});
            if (edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL || truth(edited)) {
                return finish("partial", "edited_readback_failed");
            }
            a->edited_cleared = true;
            break;
        }
        case EditAttempt::Edited: {
            a->tag_attempted = true;
            call(binding.buffer_ptr, "TextEdit", "tag_saved_version", GAK_HASH_TAG_SAVED);
            a->stage = EditAttempt::Tagged;
            Binding after;
            if (!resolve(a->path, a->rid, a->eid, a->bid, after) || id(after.script) != id(a->script)) {
                return finish("partial", "post_tag_document_changed");
            }
            int64_t current = -1, saved = -1;
            Value b = call(after.buffer_ptr, "TextEdit", "get_text", GAK_HASH_TEXT);
            Value r = call(after.script_ptr, "Script", "get_source_code", GAK_HASH_SCRIPT_SOURCE);
            struct stat st{};
            Name name("EditorInterface");
            void *interface = api.singleton(name.ptr());
            Value edited;
            if (interface) { edited = call(interface, "EditorInterface", "is_object_edited", GAK_HASH_IS_EDITED, {&after.script}); }
            if (!number(call(after.buffer_ptr, "TextEdit", "get_version", GAK_HASH_CURRENT_VERSION), current) ||
                    !number(call(after.buffer_ptr, "TextEdit", "get_saved_version", GAK_HASH_SAVED_VERSION), saved) ||
                    current != a->frozen_current || saved != current ||
                    b.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(b) != a->after ||
                    r.type() != GDEXTENSION_VARIANT_TYPE_STRING || bytes(r) != a->after ||
                    !interface || edited.type() != GDEXTENSION_VARIANT_TYPE_BOOL || truth(edited) ||
                    fstat(a->file.leaf.value, &st) || !same_time(st.st_mtimespec, a->file.original_mtime) ||
                    !content(a->file.leaf.value, a->after) || !file_attached(session, a->file) || unsaved(a->path)) {
                return finish("partial", "post_tag_unverified");
            }
            a->tagged = true;
            return finish("complete", "");
        }
        case EditAttempt::Tagged: return finish("complete", "");
    }
    return finish("ready", "");
}
Value edit_cancel(Session &session, const Value &request) {
    if (!session.attempt || request.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            bytes(request, 64) != session.attempt->request) { return reply(nullptr, "refused", "wrong_attempt"); }
    if (session.call_state != Session::CallState::Idle) {
        session.call_state = Session::CallState::Closing;
        return reply(session.attempt, "partial", "closing_after_active_stage");
    }
    Value result = reply(session.attempt, failure_status(*session.attempt), "cancelled");
    edit_cleanup(session);
    return result;
}
bool edit_expire(Session &session) {
    if (!session.attempt || ticks() < session.attempt->expires) { return false; }
    if (session.call_state == Session::CallState::Idle) { edit_cleanup(session); }
    else { session.call_state = Session::CallState::Closing; }
    return true;
}
#if GAK_FIXTURE
Value edit_fixture_fault(Session &session, const Value &request, const Value &fault) {
    if (session.call_state != Session::CallState::Idle) { return reply(nullptr, "busy", "slot_busy"); }
    static constexpr const char *allowed[] = {"read_only", "short_write", "fail_pwrite", "fail_truncate",
        "fail_fsync", "fail_pread", "fail_futimens", "close_fd_before_restore", "mismatch_mtime"};
    if (!session.attempt || request.type() != GDEXTENSION_VARIANT_TYPE_STRING ||
            bytes(request, 64) != session.attempt->request || fault.type() != GDEXTENSION_VARIANT_TYPE_STRING) {
        return reply(nullptr, "refused", "wrong_attempt");
    }
    std::string selected = bytes(fault, 32);
    if (std::find_if(std::begin(allowed), std::end(allowed), [&](const char *candidate) {
        return selected == candidate;
    }) == std::end(allowed)) { return reply(session.attempt, "refused", "invalid_fault"); }
    session.attempt->fault = std::move(selected);
    return reply(session.attempt, "ready", "");
}
#endif
} // namespace gak
