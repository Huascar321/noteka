// Diagnóstico macOS externo a latencia. Bloques por archivo, no uso exclusivo APFS.
#include <copyfile.h>
#include <fcntl.h>
#include <sys/clonefile.h>
#include <sys/stat.h>
#include <unistd.h>
#include <cerrno>
#include <cstdarg>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <mutex>
#include <set>
#include <string>
#include <utility>

namespace {
using Key = std::pair<dev_t, ino_t>;
struct File {
    std::set<std::string> names;
    std::set<int> fds;
    unsigned long long allocated = 0, size = 0, peak = 0;
    bool workspace = false, shm = false;
};
struct State {
    std::mutex mutex;
    std::map<Key, File> files;
    std::map<int, Key> fds;
    std::string root, workspace, report;
    unsigned long long peak = 0, with_shm = 0, copies = 0, clones = 0;
};
State* state = nullptr;

bool below(const std::string& path, const std::string& root) {
    return !root.empty() && path.compare(0, root.size() + 1, root + "/") == 0;
}
std::string canonical(const char* path) {
    char buffer[PATH_MAX];
    return path && realpath(path, buffer) ? buffer : "";
}
void sample(File& f, const struct stat& s) {
    f.allocated = static_cast<unsigned long long>(s.st_blocks) * 512;
    f.size = static_cast<unsigned long long>(s.st_size);
    f.peak = std::max(f.peak, f.allocated);
}
void refresh() {
    unsigned long long scratch = 0, shm = 0;
    for (auto& item : state->files) {
        auto& f = item.second;
        bool live = false;
        struct stat s;
        for (int fd : f.fds) {
            if (fstat(fd, &s) == 0 && Key(s.st_dev, s.st_ino) == item.first) {
                sample(f, s); live = true; break;
            }
        }
        for (auto it = f.names.begin(); it != f.names.end();) {
            if (stat(it->c_str(), &s) != 0 || Key(s.st_dev, s.st_ino) != item.first) {
                it = f.names.erase(it);
            } else {
                sample(f, s); live = true; ++it;
            }
        }
        if (!live) { f.allocated = 0; f.size = 0; }
        if (!f.workspace) scratch += f.allocated;
        if (f.shm) shm += f.allocated;
    }
    state->peak = std::max(state->peak, scratch);
    state->with_shm = std::max(state->with_shm, scratch + shm);
}
void observe_path(const std::string& path, int fd = -1) {
    struct stat s;
    if (path.empty() || (fd >= 0 ? fstat(fd, &s) : stat(path.c_str(), &s)) != 0 || !S_ISREG(s.st_mode)) return;
    bool workspace = path == state->workspace + "/noteka.sqlite" || path == state->workspace + "/noteka.sqlite-wal" || path == state->workspace + "/noteka.sqlite-shm";
    if (!workspace && !below(path, state->root)) return;
    Key key(s.st_dev, s.st_ino);
    auto& f = state->files[key];
    f.workspace = workspace; f.shm = workspace && path == state->workspace + "/noteka.sqlite-shm";
    f.names.insert(path);
    if (fd >= 0) { f.fds.insert(fd); state->fds[fd] = key; }
    sample(f, s);
}
template<class F> void event(F action) {
    int saved = errno;
    if (state) { std::lock_guard<std::mutex> guard(state->mutex); action(); refresh(); }
    errno = saved;
}
void opened(int fd) {
    if (fd < 0) return;
    event([&] { char path[PATH_MAX]; if (fcntl(fd, F_GETPATH, path) == 0) observe_path(canonical(path), fd); });
}
void changed() { event([] {}); }
std::string quote(const std::string& value) {
    std::string out = "\"";
    for (unsigned char c : value) {
        if (c == '\\' || c == '"') { out += '\\'; out += c; }
        else if (c < 32) { char buf[8]; snprintf(buf, sizeof(buf), "\\u%04x", c); out += buf; }
        else out += c;
    }
    return out + "\"";
}
__attribute__((constructor)) void start() {
    const char* root = getenv("NOTEKA_SCRATCH_ROOT");
    const char* report = getenv("NOTEKA_SCRATCH_REPORT");
    if (!root || !report) return;
    auto* s = new State;
    s->root = canonical(root); s->report = report;
    s->workspace = canonical(getenv("NOTEKA_SCRATCH_WORKSPACE"));
    if (!s->root.empty()) state = s;
}
__attribute__((destructor)) void finish() {
    if (!state) return;
    State* s = state;
    std::lock_guard<std::mutex> guard(s->mutex);
    refresh(); state = nullptr; // No instrumentar el propio reporte.
    FILE* out = fopen(s->report.c_str(), "w");
    if (!out) return;
    size_t scratch_files = 0;
    for (const auto& item : s->files) if (!item.second.workspace) ++scratch_files;
    fprintf(out, "{\"files\":%zu,\"copies\":%llu,\"clones\":%llu,\"peak_allocated_file_bytes\":%llu,\"peak_temp_including_workspace_shm_allocated_bytes\":%llu,\"file_peaks\":[", scratch_files, s->copies, s->clones, s->peak, s->with_shm);
    for (int group = 0; group < 2; ++group) {
        bool first = true;
        for (const auto& item : s->files) {
            const auto& f = item.second;
            if (f.workspace != (group == 1)) continue;
            if (!first) fputc(',', out);
            first = false;
            fprintf(out, "{\"device\":%llu,\"inode\":%llu,\"open_refs\":%zu,\"named\":%s,\"peak_allocated_bytes\":%llu,\"remaining_names\":[", static_cast<unsigned long long>(item.first.first), static_cast<unsigned long long>(item.first.second), f.fds.size(), f.names.empty() ? "false" : "true", f.peak);
            bool first_name = true;
            for (const auto& name : f.names) { if (!first_name) fputc(',', out); first_name = false; fputs(quote(name).c_str(), out); }
            fputs("]}", out);
        }
        fputs(group == 0 ? "],\"workspace_files\":[" : "]}\n", out);
    }
    fclose(out);
}
} // namespace

#define INTERPOSE(replacement, original) \
    __attribute__((used)) static struct { const void* replacement; const void* original; } \
    interpose_##original __attribute__((section("__DATA,__interpose"))) = { \
        reinterpret_cast<const void*>(replacement), reinterpret_cast<const void*>(original) };

extern "C" int probe_open(const char* path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); mode = va_arg(ap, int); va_end(ap); }
    int fd = open(path, flags, mode); opened(fd); return fd;
}
INTERPOSE(probe_open, open)
extern "C" int probe_openat(int base, const char* path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) { va_list ap; va_start(ap, flags); mode = va_arg(ap, int); va_end(ap); }
    int fd = openat(base, path, flags, mode); opened(fd); return fd;
}
INTERPOSE(probe_openat, openat)
extern "C" ssize_t probe_write(int fd, const void* data, size_t count) { auto rc = write(fd, data, count); changed(); return rc; }
INTERPOSE(probe_write, write)
extern "C" ssize_t probe_pwrite(int fd, const void* data, size_t count, off_t offset) { auto rc = pwrite(fd, data, count, offset); changed(); return rc; }
INTERPOSE(probe_pwrite, pwrite)
extern "C" int probe_ftruncate(int fd, off_t size) { int rc = ftruncate(fd, size); changed(); return rc; }
INTERPOSE(probe_ftruncate, ftruncate)
extern "C" int probe_close(int fd) {
    changed(); int rc = close(fd);
    if (rc == 0) event([&] { auto it = state->fds.find(fd); if (it != state->fds.end()) { state->files[it->second].fds.erase(fd); state->fds.erase(it); } });
    return rc;
}
INTERPOSE(probe_close, close)
extern "C" int probe_unlink(const char* path) { changed(); int rc = unlink(path); changed(); return rc; }
INTERPOSE(probe_unlink, unlink)
extern "C" int probe_unlinkat(int base, const char* path, int flags) { changed(); int rc = unlinkat(base, path, flags); changed(); return rc; }
INTERPOSE(probe_unlinkat, unlinkat)
extern "C" int probe_copyfile(const char* from, const char* to, copyfile_state_t s, copyfile_flags_t flags) {
    int rc = copyfile(from, to, s, flags);
    if (rc == 0) event([&] { ++state->copies; observe_path(canonical(to)); });
    return rc;
}
INTERPOSE(probe_copyfile, copyfile)
extern "C" int probe_fcopyfile(int from, int to, copyfile_state_t s, copyfile_flags_t flags) {
    int rc = fcopyfile(from, to, s, flags); if (rc == 0) { opened(to); event([] { ++state->copies; }); } return rc;
}
INTERPOSE(probe_fcopyfile, fcopyfile)
extern "C" int probe_clonefile(const char* from, const char* to, int flags) {
    int rc = clonefile(from, to, flags); if (rc == 0) event([&] { ++state->clones; observe_path(canonical(to)); }); return rc;
}
INTERPOSE(probe_clonefile, clonefile)
extern "C" int probe_clonefileat(int from_base, const char* from, int to_base, const char* to, int flags) {
    int rc = clonefileat(from_base, from, to_base, to, flags);
    if (rc == 0) { int fd = openat(to_base, to, O_RDONLY); if (fd >= 0) { opened(fd); probe_close(fd); } event([] { ++state->clones; }); } return rc;
}
INTERPOSE(probe_clonefileat, clonefileat)
extern "C" int probe_fclonefileat(int from, int to_base, const char* to, int flags) {
    int rc = fclonefileat(from, to_base, to, flags);
    if (rc == 0) { int fd = openat(to_base, to, O_RDONLY); if (fd >= 0) { opened(fd); probe_close(fd); } event([] { ++state->clones; }); } return rc;
}
INTERPOSE(probe_fclonefileat, fclonefileat)
