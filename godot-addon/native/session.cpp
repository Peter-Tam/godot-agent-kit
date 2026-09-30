#include "native.hpp"

#include <CommonCrypto/CommonDigest.h>
#include <sys/acl.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <cerrno>
#include <climits>
#include <cstdlib>
#include <cstring>
#include <string>

namespace gak {
namespace {

struct Fd {
    int fd = -1;
    explicit Fd(int n = -1) : fd(n) {}
    ~Fd() { if (fd >= 0) { ::close(fd); } }
    Fd(const Fd &) = delete;
    Fd &operator=(const Fd &) = delete;
};

enum class Access { Ancestor, Project };

bool permitted(int fd, Access access) {
    struct stat st{};
    if (fstat(fd, &st) || !S_ISDIR(st.st_mode) ||
            (st.st_uid != geteuid() && st.st_uid != 0) ||
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

// The canonical path is derived only from ProjectSettings, never from a request.
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


bool valid_session_id(const std::string &id) {
    if (id.size() != 32) { return false; }
    for (char c : id) { if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) { return false; } }
    return true;
}

} // namespace
bool valid_utf8(std::string_view source) { return utf8_text(source); }
bool same_time(timespec left, timespec right) {
    return left.tv_sec == right.tv_sec && left.tv_nsec == right.tv_nsec;
}

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

void close(Session &session) {
    if (session.call_state != Session::CallState::Idle) {
        open_closing(session);
        session.call_state = Session::CallState::Closing;
        return;
    }
    edit_cleanup(session);
    open_cleanup(session);
    if (session.project_fd >= 0) { ::close(session.project_fd); }
    session.project_fd = -1;
    session.project_path.clear();
    session.session_id.clear();
}

bool configure(Session &session, const std::string &session_id) {
    if (occupied(session) || session.call_state != Session::CallState::Idle || !valid_session_id(session_id)) { return false; }
    close(session);
    Name settings("ProjectSettings");
    void *project_settings = api.singleton(settings.ptr());
    if (!project_settings) { return false; }
    Value resource = string("res://");
    Value absolute = call(project_settings, "ProjectSettings", "globalize_path", GAK_HASH_GLOBALIZE_PATH, {&resource});
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

} // namespace gak
