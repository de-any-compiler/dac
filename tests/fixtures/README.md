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
