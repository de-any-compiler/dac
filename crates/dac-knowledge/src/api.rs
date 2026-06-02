//! API signature catalogue (B2.6, FR-14, FR-16).
//!
//! A curated table of standard-library and platform-API function
//! signatures. The type-propagation pass in [`dac_recovery`] uses
//! this table to seed types at call sites: a call to `strlen` with
//! one argument binds the argument's [`Type`] to
//! `Ptr(Int(8, Unsigned))` and the return value to `Int(64, Unsigned)`
//! (on a 64-bit target).
//!
//! ## Scope at B2.6
//!
//! - **libc minimal set**: string ops, mem ops, the allocator, basic
//!   I/O (`malloc`, `free`, `memcpy`, `memset`, `strlen`, `strcmp`,
//!   `strcpy`, `strncpy`, `printf`, `puts`, `fopen`, `fclose`,
//!   `fread`, `fwrite`, `read`, `write`, `open`, `close`, `exit`,
//!   `abort`).
//! - **Win32 minimal set**: handle lifecycle, file I/O, the heap,
//!   process exit (`CreateFileA`, `CloseHandle`, `ReadFile`,
//!   `WriteFile`, `GetLastError`, `HeapAlloc`, `HeapFree`,
//!   `ExitProcess`).
//!
//! ## What this module does not do
//!
//! - **Variadic argument typing.** `printf`'s format string is
//!   parsed by a later batch (B3.3 idiom catalogue) — here it is
//!   modelled as a single `const char *` formal followed by a
//!   variadic tail that types propagation cannot constrain.
//! - **Wide-character or platform-conditional variants.** Only the
//!   common-case ASCII / 8-bit signatures land here; `wcslen`,
//!   `_open_s`, etc. arrive when the surface is needed.
//! - **Callback signatures.** Function-pointer parameters carry
//!   `Ptr(Unknown)` until the lifter has function-type support.
//!
//! ## Determinism
//!
//! All entries are `&'static` and constructed at compile time, so
//! iteration order is stable across runs. The lookup helpers
//! ([`lookup_api_signature`], [`api_signatures`]) walk the table in
//! the order entries appear here.

use dac_ir::Type;

/// One API function signature.
///
/// `parameters` lists positional parameters; `is_variadic` covers the
/// trailing `...` of C variadic functions like `printf`. The variadic
/// tail does not get its own [`Type`] — variadic arguments are typed
/// (when they are typed at all) by format-string analysis in B3.3.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiSignature {
    /// Canonical function name as it appears in the imports table or
    /// symbol table. Case-sensitive: `strlen` is libc, `Strlen` is
    /// not a thing here.
    pub name: &'static str,
    /// Library or component this signature belongs to. Used for
    /// diagnostics and so the propagation pass can prefer the
    /// component that matches the binary's target OS when two
    /// libraries publish the same name.
    pub library: ApiLibrary,
    /// Return type. Use [`Type::Unknown`] for genuinely-unknown
    /// returns (`void` is not yet modelled as a distinct type — the
    /// propagation pass treats `Unknown` returns as "no info" and
    /// keeps the call's result type at the lattice bottom).
    pub return_ty: Type,
    /// Positional parameters in source order.
    pub parameters: &'static [ApiParameter],
    /// True when the function accepts a trailing `...`.
    pub is_variadic: bool,
}

/// Which library / component an [`ApiSignature`] comes from.
///
/// Today only `Libc` and `Win32` ship; richer partitioning (POSIX
/// vs. GNU extensions, MSVCRT vs. UCRT) is a follow-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ApiLibrary {
    /// Standard C library — `libc.so`, `msvcrt.dll`/`ucrtbase.dll`'s C
    /// surface.
    Libc,
    /// Windows Win32 API — `kernel32.dll` and friends.
    Win32,
}

impl ApiLibrary {
    /// Stable lowercase identifier for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ApiLibrary::Libc => "libc",
            ApiLibrary::Win32 => "win32",
        }
    }
}

/// One positional parameter of an [`ApiSignature`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiParameter {
    /// Parameter name, for diagnostics and the `--debug` "why this
    /// type?" trail.
    pub name: &'static str,
    /// Parameter type.
    pub ty: Type,
}

/// Look up an API signature by name across every library this batch
/// ships.
///
/// Returns the first match in [`API_SIGNATURES`]. Iteration is
/// deterministic and library precedence follows the table order
/// (`Libc` before `Win32`) — the propagation pass can re-rank by
/// target OS once the binary model exposes a target-OS field
/// (M3-ish).
#[must_use]
pub fn lookup_api_signature(name: &str) -> Option<&'static ApiSignature> {
    API_SIGNATURES.iter().find(|s| s.name == name)
}

/// Look up an API signature, restricting the search to one library.
#[must_use]
pub fn lookup_api_signature_in(name: &str, library: ApiLibrary) -> Option<&'static ApiSignature> {
    API_SIGNATURES
        .iter()
        .find(|s| s.library == library && s.name == name)
}

/// All API signatures shipped with this build, in declaration order.
#[must_use]
pub fn api_signatures() -> &'static [ApiSignature] {
    API_SIGNATURES.as_slice()
}

// --- Type-builder helpers used by the const tables ------------------
//
// `Type::signed_int` / `Type::unsigned_int` / `Type::int_of_width` are
// all `const`; `Type::ptr_to` is not because it boxes the pointee.
// Pointer types in this table therefore use the raw `Type::Ptr`
// constructor with a const-evaluable `Box` (the value is constructed
// at the use site, not in `const` context).

const VOID_PTR_PARAM: ApiParameter = ApiParameter {
    name: "p",
    ty: Type::Unknown,
};

const SIZE_T_PARAM: ApiParameter = ApiParameter {
    name: "n",
    ty: Type::Int(dac_ir::IntType {
        width_bits: 64,
        signedness: dac_ir::Signedness::Unsigned,
    }),
};

const INT_PARAM: ApiParameter = ApiParameter {
    name: "x",
    ty: Type::Int(dac_ir::IntType {
        width_bits: 32,
        signedness: dac_ir::Signedness::Signed,
    }),
};

const SIZE_T: Type = Type::Int(dac_ir::IntType {
    width_bits: 64,
    signedness: dac_ir::Signedness::Unsigned,
});

const INT32_S: Type = Type::Int(dac_ir::IntType {
    width_bits: 32,
    signedness: dac_ir::Signedness::Signed,
});

const INT32_U: Type = Type::Int(dac_ir::IntType {
    width_bits: 32,
    signedness: dac_ir::Signedness::Unsigned,
});

const SSIZE_T: Type = Type::Int(dac_ir::IntType {
    width_bits: 64,
    signedness: dac_ir::Signedness::Signed,
});

// `Ptr<T>` is built ad-hoc per entry because `Box::new` is not
// `const` for arbitrary `T` in stable Rust. Each helper returns an
// owned `ApiSignature`-style construction the table can wrap in a
// `const` array via `ApiSignature` field initialization — but the
// pointer fields themselves require a non-const construction, so
// the table lives behind a `static OnceLock<Vec<ApiSignature>>`-free
// alternative: we build the table once at compile time using a
// lazy-init macro? No — we can avoid both by storing parameter
// lists as `&'static [ApiParameter]` slices computed in a sibling
// helper module that uses `const fn` where possible. Where a
// `Ptr` is needed, we use a private `const ANY_PTR: Type` that
// wraps `Ptr(Unknown)` via a different mechanism.
//
// In practice: `Type::Ptr(Box<Type>)` is not const-constructible at
// declaration site. The signature table is therefore built behind a
// `static` accessor that runs once via `LazyLock`.

use std::sync::LazyLock;

/// Master signature table — all libc and Win32 entries in declaration
/// order. Behind a [`LazyLock`] because pointer types involve
/// `Box::new` which is not yet `const`-callable.
static API_SIGNATURES: LazyLock<Vec<ApiSignature>> = LazyLock::new(build_api_signatures);

fn build_api_signatures() -> Vec<ApiSignature> {
    let mut out = Vec::new();
    out.extend(libc_signatures());
    out.extend(win32_signatures());
    out
}

fn char_ptr() -> Type {
    Type::ptr_to(Type::unsigned_int(8))
}

fn const_char_ptr() -> Type {
    // The lattice does not distinguish const-ness; const is a C-AST
    // attribute restored by the backend.
    char_ptr()
}

fn void_ptr() -> Type {
    // `void *` lowers to a generic pointer in the lattice.
    Type::ptr_to(Type::Unknown)
}

fn file_ptr() -> Type {
    Type::ptr_to(Type::Unknown)
}

fn handle_ty() -> Type {
    // Windows HANDLE is an opaque pointer-sized value; model as a
    // generic pointer.
    Type::ptr_to(Type::Unknown)
}

fn libc_signatures() -> Vec<ApiSignature> {
    vec![
        // size_t strlen(const char *s);
        ApiSignature {
            name: "strlen",
            library: ApiLibrary::Libc,
            return_ty: SIZE_T,
            parameters: leak_params(vec![ApiParameter {
                name: "s",
                ty: const_char_ptr(),
            }]),
            is_variadic: false,
        },
        // int strcmp(const char *a, const char *b);
        ApiSignature {
            name: "strcmp",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "a",
                    ty: const_char_ptr(),
                },
                ApiParameter {
                    name: "b",
                    ty: const_char_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // char *strcpy(char *dst, const char *src);
        ApiSignature {
            name: "strcpy",
            library: ApiLibrary::Libc,
            return_ty: char_ptr(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "dst",
                    ty: char_ptr(),
                },
                ApiParameter {
                    name: "src",
                    ty: const_char_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // char *strncpy(char *dst, const char *src, size_t n);
        ApiSignature {
            name: "strncpy",
            library: ApiLibrary::Libc,
            return_ty: char_ptr(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "dst",
                    ty: char_ptr(),
                },
                ApiParameter {
                    name: "src",
                    ty: const_char_ptr(),
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // void *memcpy(void *dst, const void *src, size_t n);
        ApiSignature {
            name: "memcpy",
            library: ApiLibrary::Libc,
            return_ty: void_ptr(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "dst",
                    ty: void_ptr(),
                },
                ApiParameter {
                    name: "src",
                    ty: void_ptr(),
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // void *memset(void *p, int c, size_t n);
        ApiSignature {
            name: "memset",
            library: ApiLibrary::Libc,
            return_ty: void_ptr(),
            parameters: leak_params(vec![
                VOID_PTR_PARAM,
                ApiParameter {
                    name: "c",
                    ty: INT32_S,
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // int memcmp(const void *a, const void *b, size_t n);
        ApiSignature {
            name: "memcmp",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "a",
                    ty: void_ptr(),
                },
                ApiParameter {
                    name: "b",
                    ty: void_ptr(),
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // void *malloc(size_t n);
        ApiSignature {
            name: "malloc",
            library: ApiLibrary::Libc,
            return_ty: void_ptr(),
            parameters: leak_params(vec![SIZE_T_PARAM]),
            is_variadic: false,
        },
        // void *calloc(size_t n, size_t size);
        ApiSignature {
            name: "calloc",
            library: ApiLibrary::Libc,
            return_ty: void_ptr(),
            parameters: leak_params(vec![
                SIZE_T_PARAM,
                ApiParameter {
                    name: "size",
                    ty: SIZE_T,
                },
            ]),
            is_variadic: false,
        },
        // void *realloc(void *p, size_t n);
        ApiSignature {
            name: "realloc",
            library: ApiLibrary::Libc,
            return_ty: void_ptr(),
            parameters: leak_params(vec![VOID_PTR_PARAM, SIZE_T_PARAM]),
            is_variadic: false,
        },
        // void free(void *p);
        ApiSignature {
            name: "free",
            library: ApiLibrary::Libc,
            return_ty: Type::Unknown,
            parameters: leak_params(vec![VOID_PTR_PARAM]),
            is_variadic: false,
        },
        // int printf(const char *fmt, ...);
        ApiSignature {
            name: "printf",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![ApiParameter {
                name: "fmt",
                ty: const_char_ptr(),
            }]),
            is_variadic: true,
        },
        // int puts(const char *s);
        ApiSignature {
            name: "puts",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![ApiParameter {
                name: "s",
                ty: const_char_ptr(),
            }]),
            is_variadic: false,
        },
        // FILE *fopen(const char *path, const char *mode);
        ApiSignature {
            name: "fopen",
            library: ApiLibrary::Libc,
            return_ty: file_ptr(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "path",
                    ty: const_char_ptr(),
                },
                ApiParameter {
                    name: "mode",
                    ty: const_char_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // int fclose(FILE *fp);
        ApiSignature {
            name: "fclose",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![ApiParameter {
                name: "fp",
                ty: file_ptr(),
            }]),
            is_variadic: false,
        },
        // size_t fread(void *p, size_t size, size_t n, FILE *fp);
        ApiSignature {
            name: "fread",
            library: ApiLibrary::Libc,
            return_ty: SIZE_T,
            parameters: leak_params(vec![
                VOID_PTR_PARAM,
                ApiParameter {
                    name: "size",
                    ty: SIZE_T,
                },
                SIZE_T_PARAM,
                ApiParameter {
                    name: "fp",
                    ty: file_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // size_t fwrite(const void *p, size_t size, size_t n, FILE *fp);
        ApiSignature {
            name: "fwrite",
            library: ApiLibrary::Libc,
            return_ty: SIZE_T,
            parameters: leak_params(vec![
                VOID_PTR_PARAM,
                ApiParameter {
                    name: "size",
                    ty: SIZE_T,
                },
                SIZE_T_PARAM,
                ApiParameter {
                    name: "fp",
                    ty: file_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // ssize_t read(int fd, void *buf, size_t n);
        ApiSignature {
            name: "read",
            library: ApiLibrary::Libc,
            return_ty: SSIZE_T,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "fd",
                    ty: INT32_S,
                },
                ApiParameter {
                    name: "buf",
                    ty: void_ptr(),
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // ssize_t write(int fd, const void *buf, size_t n);
        ApiSignature {
            name: "write",
            library: ApiLibrary::Libc,
            return_ty: SSIZE_T,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "fd",
                    ty: INT32_S,
                },
                ApiParameter {
                    name: "buf",
                    ty: void_ptr(),
                },
                SIZE_T_PARAM,
            ]),
            is_variadic: false,
        },
        // int open(const char *path, int flags);
        ApiSignature {
            name: "open",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "path",
                    ty: const_char_ptr(),
                },
                ApiParameter {
                    name: "flags",
                    ty: INT32_S,
                },
            ]),
            is_variadic: true,
        },
        // int close(int fd);
        ApiSignature {
            name: "close",
            library: ApiLibrary::Libc,
            return_ty: INT32_S,
            parameters: leak_params(vec![ApiParameter {
                name: "fd",
                ty: INT32_S,
            }]),
            is_variadic: false,
        },
        // _Noreturn void exit(int status);
        ApiSignature {
            name: "exit",
            library: ApiLibrary::Libc,
            return_ty: Type::Unknown,
            parameters: leak_params(vec![ApiParameter {
                name: "status",
                ty: INT32_S,
            }]),
            is_variadic: false,
        },
        // _Noreturn void abort(void);
        ApiSignature {
            name: "abort",
            library: ApiLibrary::Libc,
            return_ty: Type::Unknown,
            parameters: leak_params(vec![]),
            is_variadic: false,
        },
        // char *getenv(const char *name);
        ApiSignature {
            name: "getenv",
            library: ApiLibrary::Libc,
            return_ty: char_ptr(),
            parameters: leak_params(vec![ApiParameter {
                name: "name",
                ty: const_char_ptr(),
            }]),
            is_variadic: false,
        },
    ]
}

fn win32_signatures() -> Vec<ApiSignature> {
    vec![
        // HANDLE CreateFileA(
        //   LPCSTR lpFileName,
        //   DWORD dwDesiredAccess,
        //   DWORD dwShareMode,
        //   LPSECURITY_ATTRIBUTES lpSecurityAttributes,
        //   DWORD dwCreationDisposition,
        //   DWORD dwFlagsAndAttributes,
        //   HANDLE hTemplateFile,
        // );
        ApiSignature {
            name: "CreateFileA",
            library: ApiLibrary::Win32,
            return_ty: handle_ty(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "lpFileName",
                    ty: const_char_ptr(),
                },
                ApiParameter {
                    name: "dwDesiredAccess",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "dwShareMode",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "lpSecurityAttributes",
                    ty: void_ptr(),
                },
                ApiParameter {
                    name: "dwCreationDisposition",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "dwFlagsAndAttributes",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "hTemplateFile",
                    ty: handle_ty(),
                },
            ]),
            is_variadic: false,
        },
        // BOOL CloseHandle(HANDLE hObject);
        ApiSignature {
            name: "CloseHandle",
            library: ApiLibrary::Win32,
            return_ty: INT32_S,
            parameters: leak_params(vec![ApiParameter {
                name: "hObject",
                ty: handle_ty(),
            }]),
            is_variadic: false,
        },
        // BOOL ReadFile(HANDLE, LPVOID, DWORD, LPDWORD, LPOVERLAPPED);
        ApiSignature {
            name: "ReadFile",
            library: ApiLibrary::Win32,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "hFile",
                    ty: handle_ty(),
                },
                ApiParameter {
                    name: "lpBuffer",
                    ty: void_ptr(),
                },
                ApiParameter {
                    name: "nNumberOfBytesToRead",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "lpNumberOfBytesRead",
                    ty: Type::ptr_to(INT32_U),
                },
                ApiParameter {
                    name: "lpOverlapped",
                    ty: void_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // BOOL WriteFile(HANDLE, LPCVOID, DWORD, LPDWORD, LPOVERLAPPED);
        ApiSignature {
            name: "WriteFile",
            library: ApiLibrary::Win32,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "hFile",
                    ty: handle_ty(),
                },
                ApiParameter {
                    name: "lpBuffer",
                    ty: void_ptr(),
                },
                ApiParameter {
                    name: "nNumberOfBytesToWrite",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "lpNumberOfBytesWritten",
                    ty: Type::ptr_to(INT32_U),
                },
                ApiParameter {
                    name: "lpOverlapped",
                    ty: void_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // DWORD GetLastError(void);
        ApiSignature {
            name: "GetLastError",
            library: ApiLibrary::Win32,
            return_ty: INT32_U,
            parameters: leak_params(vec![]),
            is_variadic: false,
        },
        // LPVOID HeapAlloc(HANDLE hHeap, DWORD dwFlags, SIZE_T dwBytes);
        ApiSignature {
            name: "HeapAlloc",
            library: ApiLibrary::Win32,
            return_ty: void_ptr(),
            parameters: leak_params(vec![
                ApiParameter {
                    name: "hHeap",
                    ty: handle_ty(),
                },
                ApiParameter {
                    name: "dwFlags",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "dwBytes",
                    ty: SIZE_T,
                },
            ]),
            is_variadic: false,
        },
        // BOOL HeapFree(HANDLE hHeap, DWORD dwFlags, LPVOID lpMem);
        ApiSignature {
            name: "HeapFree",
            library: ApiLibrary::Win32,
            return_ty: INT32_S,
            parameters: leak_params(vec![
                ApiParameter {
                    name: "hHeap",
                    ty: handle_ty(),
                },
                ApiParameter {
                    name: "dwFlags",
                    ty: INT32_U,
                },
                ApiParameter {
                    name: "lpMem",
                    ty: void_ptr(),
                },
            ]),
            is_variadic: false,
        },
        // HANDLE GetProcessHeap(void);
        ApiSignature {
            name: "GetProcessHeap",
            library: ApiLibrary::Win32,
            return_ty: handle_ty(),
            parameters: leak_params(vec![]),
            is_variadic: false,
        },
        // _Noreturn void ExitProcess(UINT uExitCode);
        ApiSignature {
            name: "ExitProcess",
            library: ApiLibrary::Win32,
            return_ty: Type::Unknown,
            parameters: leak_params(vec![ApiParameter {
                name: "uExitCode",
                ty: INT32_U,
            }]),
            is_variadic: false,
        },
    ]
}

/// Convert an owned `Vec<ApiParameter>` into a leaked `&'static
/// [ApiParameter]` so the [`ApiSignature::parameters`] slice can be
/// `&'static`. This runs exactly once per signature thanks to the
/// outer [`LazyLock`], so the leaked bytes (~1 page total across the
/// whole catalogue) are de-facto allocated once for the lifetime of
/// the process.
fn leak_params(params: Vec<ApiParameter>) -> &'static [ApiParameter] {
    Box::leak(params.into_boxed_slice())
}

// Suppress dead-code warnings for the const items that are not used
// by the current table but are convenient seeds for the propagation
// pass (e.g. `INT_PARAM` will be reused by additional signatures in
// follow-up batches).
#[allow(dead_code)]
const _UNUSED: (&ApiParameter, &ApiParameter) = (&INT_PARAM, &VOID_PTR_PARAM);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_libc_signatures() {
        let s = lookup_api_signature("strlen").expect("strlen present");
        assert_eq!(s.name, "strlen");
        assert_eq!(s.library, ApiLibrary::Libc);
        assert_eq!(s.parameters.len(), 1);
        assert!(!s.is_variadic);
        assert_eq!(s.return_ty, Type::unsigned_int(64));
        assert_eq!(s.parameters[0].ty, Type::ptr_to(Type::unsigned_int(8)));
    }

    #[test]
    fn lookup_finds_win32_signatures() {
        let s = lookup_api_signature("CreateFileA").expect("CreateFileA present");
        assert_eq!(s.library, ApiLibrary::Win32);
        assert_eq!(s.parameters.len(), 7);
        assert!(!s.is_variadic);
    }

    #[test]
    fn variadic_flag_is_set_for_printf() {
        let s = lookup_api_signature("printf").expect("printf present");
        assert!(s.is_variadic);
        // The single fixed parameter is the format string.
        assert_eq!(s.parameters.len(), 1);
        assert_eq!(s.parameters[0].name, "fmt");
    }

    #[test]
    fn lookup_in_filters_by_library() {
        assert!(lookup_api_signature_in("strlen", ApiLibrary::Libc).is_some());
        assert!(lookup_api_signature_in("strlen", ApiLibrary::Win32).is_none());
        assert!(lookup_api_signature_in("CreateFileA", ApiLibrary::Win32).is_some());
        assert!(lookup_api_signature_in("CreateFileA", ApiLibrary::Libc).is_none());
    }

    #[test]
    fn lookup_returns_none_for_unknown_name() {
        assert!(lookup_api_signature("not_a_real_function").is_none());
    }

    #[test]
    fn api_signature_table_has_no_duplicate_names() {
        // Same (name, library) appearing twice would silently drop one
        // entry — both the lookup and the propagation pass walk the
        // table in declaration order.
        let mut seen = std::collections::BTreeSet::new();
        for sig in api_signatures() {
            assert!(
                seen.insert((sig.name, sig.library)),
                "duplicate signature: {} in {:?}",
                sig.name,
                sig.library
            );
        }
    }

    #[test]
    fn libc_minimal_set_is_complete() {
        for name in [
            "strlen", "strcmp", "strcpy", "strncpy", "memcpy", "memset", "memcmp", "malloc",
            "calloc", "realloc", "free", "printf", "puts", "fopen", "fclose", "fread", "fwrite",
            "read", "write", "open", "close", "exit", "abort", "getenv",
        ] {
            assert!(
                lookup_api_signature_in(name, ApiLibrary::Libc).is_some(),
                "missing libc signature for {name}"
            );
        }
    }

    #[test]
    fn win32_minimal_set_is_complete() {
        for name in [
            "CreateFileA",
            "CloseHandle",
            "ReadFile",
            "WriteFile",
            "GetLastError",
            "HeapAlloc",
            "HeapFree",
            "GetProcessHeap",
            "ExitProcess",
        ] {
            assert!(
                lookup_api_signature_in(name, ApiLibrary::Win32).is_some(),
                "missing win32 signature for {name}"
            );
        }
    }

    #[test]
    fn signatures_are_stable_across_lookups() {
        let a = lookup_api_signature("memcpy").unwrap();
        let b = lookup_api_signature("memcpy").unwrap();
        // Same backing static — pointer equality is meaningful for
        // the leaked parameter slices.
        assert!(std::ptr::eq(a, b));
        assert!(std::ptr::eq(a.parameters, b.parameters));
    }

    #[test]
    fn api_library_names_are_stable() {
        assert_eq!(ApiLibrary::Libc.name(), "libc");
        assert_eq!(ApiLibrary::Win32.name(), "win32");
    }
}
