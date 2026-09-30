#pragma once

#include "native.hpp"
#include <string>
#include <vector>

namespace gak {
constexpr size_t SOURCE_LIMIT = 512 * 1024;
struct Fd {
    int value = -1;
    explicit Fd(int n = -1) : value(n) {}
    ~Fd();
    Fd(const Fd &) = delete;
    Fd &operator=(const Fd &) = delete;
    Fd(Fd &&other) noexcept;
    Fd &operator=(Fd &&other) noexcept;
};
struct Parent { Fd fd; std::string segment; dev_t device{}; ino_t inode{}; };
struct FileBinding {
    std::vector<Parent> parents;
    Fd leaf;
    std::string name;
    dev_t device{};
    ino_t inode{};
};
bool number(const Value &v, int64_t &n);
bool decimal(const Value &v, uint64_t &n);
bool path_ok(const std::string &path);
bool same(const struct stat &a, const struct stat &b);
bool attached_project(const Session &session);
bool file_attached(const Session &session, const FileBinding &file);
const char *pin_file(const Session &session, const std::string &path, FileBinding &out, int access);
bool content(int fd, std::string_view expected);
struct Binding {
    Value script, editor, buffer;
    void *script_ptr = nullptr, *editor_ptr = nullptr, *buffer_ptr = nullptr;
    uint64_t script_id{}, editor_id{}, buffer_id{};
};
bool find_document(const std::string &path, Binding &out, bool &found);
// Reacquire the public parallel topology; never dereference retained editor pointers.
bool resolve(const std::string &path, uint64_t rid, uint64_t eid, uint64_t bid, Binding &out);
uint64_t ticks();
bool valid_request(const std::string &request);
} // namespace gak
