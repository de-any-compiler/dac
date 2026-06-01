# Test fixtures

Small binaries used by `dac-binfmt` and `dac-cli` integration tests.
Kept here at workspace root so multiple crates can share them via
`CARGO_MANIFEST_DIR/../../tests/fixtures/<name>`.

## ELF (x86-64, Linux)

| File                       | What it is                                  | How to rebuild                                                   |
| -------------------------- | ------------------------------------------- | ---------------------------------------------------------------- |
| `hello-x86_64`             | PIE executable with full symbol table       | `gcc -Os hello.c -o hello-x86_64`                                |
| `hello-x86_64-stripped`    | Same, with `-s` (no `.symtab` / `.strtab`)  | `strip -s hello-x86_64 -o hello-x86_64-stripped`                 |
| `libsample.so`             | Shared library with three exports + a relo  | `gcc -Os -shared -fPIC sample.c -o libsample.so`                 |

The C sources are intentionally minimal so the binaries stay <20 KiB each
and the round-trip tests stay focused on parser invariants, not on
glibc-version drift. Reference sources:

```c
/* hello.c */
#include <unistd.h>
int main(void) { write(1, "hello\n", 6); return 42; }

/* sample.c */
int sample_value = 42;
int sample_add(int a, int b) { return a + b; }
const char* sample_greeting(void) { return "hello from libsample"; }
```

Both were built with `gcc 13.x` on x86-64 Linux with `glibc 2.39`. The
parser tests do not assume any specific glibc symbol set — they only assert
properties that any conformant build would satisfy (entry point present,
some `FUNC` symbols, some `OBJECT` symbols in `libsample.so`, `libc.so.6`
in needed libraries, etc.).

## PE (x86-64, Windows)

| File                          | What it is                                       | How to rebuild                                                                                         |
| ----------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| `hello-x86_64.exe`            | PE32+ console exe with the COFF symbol table     | `x86_64-w64-mingw32-gcc -Os -ffunction-sections -fdata-sections -Wl,--gc-sections hello_pe.c -o tmp.exe` then `x86_64-w64-mingw32-strip --strip-debug tmp.exe -o hello-x86_64.exe` |
| `hello-x86_64-stripped.exe`   | Same, with full `--strip-all` (no symtab)        | `x86_64-w64-mingw32-strip --strip-all tmp.exe -o hello-x86_64-stripped.exe`                            |
| `sample.dll`                  | PE32+ DLL with three `__declspec(dllexport)`s    | `x86_64-w64-mingw32-gcc -Os -shared -ffunction-sections -fdata-sections -Wl,--gc-sections sample_dll.c -o tmp.dll` then `x86_64-w64-mingw32-strip --strip-debug tmp.dll -o sample.dll` |

The unstripped variant keeps the COFF symbol table (so `main` round-trips
through the parser) but drops debug sections, keeping the file ~40 KiB.
The fully-stripped variant is ~16 KiB; the DLL is ~30 KiB. Reference
sources:

```c
/* hello_pe.c */
#include <windows.h>
int main(void) {
    HANDLE h = GetStdHandle(STD_OUTPUT_HANDLE);
    const char msg[] = "hello from dac PE\n";
    DWORD written = 0;
    WriteFile(h, msg, (DWORD)(sizeof(msg) - 1), &written, NULL);
    return 42;
}

/* sample_dll.c */
#include <windows.h>
__declspec(dllexport) int sample_value = 42;
__declspec(dllexport) int sample_add(int a, int b) { return a + b; }
__declspec(dllexport) const char* sample_greeting(void) {
    return "hello from sample DLL";
}
BOOL WINAPI DllMain(HINSTANCE h, DWORD r, LPVOID p) { (void)h;(void)r;(void)p; return TRUE; }
```

Built with `mingw-w64 16.x` (`x86_64-w64-mingw32-gcc`). The parser tests
do not assume specific `api-ms-win-crt-*` import names — they only assert
that `KERNEL32.dll` is among the needed libraries, that `.text` is
executable, that the exports include `sample_add` / `sample_greeting` /
`sample_value`, and that the embedded string lands in the extracted
`StringRef` set.
