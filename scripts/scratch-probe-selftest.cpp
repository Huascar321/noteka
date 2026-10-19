// Cinco archivos: abierto sin nombre, cerrado con nombre, copia y clon APFS.
#include <copyfile.h>
#include <fcntl.h>
#include <sys/clonefile.h>
#include <unistd.h>
#include <cassert>
#include <cstdlib>
#include <string>

int main() {
    std::string root = getenv("NOTEKA_SCRATCH_ROOT");
    auto path = [&](const char* name) { return root + "/" + name; };
    std::string bytes(65536, 'x');
    int a = open(path("a").c_str(), O_CREAT | O_RDWR | O_EXCL, 0600);
    assert(a >= 0 && write(a, bytes.data(), bytes.size()) == 65536);
    assert(unlink(path("a").c_str()) == 0);
    assert(pwrite(a, bytes.data(), bytes.size(), 65536) == 65536);
    int b = openat(AT_FDCWD, path("b").c_str(), O_CREAT | O_RDWR | O_EXCL, 0600);
    assert(b >= 0 && write(b, bytes.data(), 32768) == 32768 && close(b) == 0);
    assert(copyfile(path("b").c_str(), path("c").c_str(), nullptr, COPYFILE_ALL) == 0);
    assert(clonefile(path("b").c_str(), path("d").c_str(), 0) == 0);
    int e = open(path("e").c_str(), O_CREAT | O_RDWR | O_EXCL, 0600);
    assert(e >= 0 && write(e, bytes.data(), 32768) == 32768);
    assert(ftruncate(e, 65536) == 0);
    assert(close(e) == 0 && close(a) == 0);
    for (auto name : {"b", "c", "d", "e"}) assert(unlinkat(AT_FDCWD, path(name).c_str(), 0) == 0);
}
