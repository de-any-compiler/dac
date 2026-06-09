//! Canonical C-typedef spellings for known libc / POSIX APIs (B3.33,
//! FR-21).
//!
//! The `dac-knowledge::api` catalogue types every API entry in the
//! lattice's [`dac_ir::Type`] vocabulary — width-tagged integers and
//! `Ptr<T>` chains. That vocabulary cannot distinguish `int` from
//! `int32_t`, `ssize_t` from `int64_t`, or `const void *` from
//! `void *`: the lattice deliberately erases the typedef-level
//! distinctions a C source program carries.
//!
//! For most lifter consumers that erasure is the right call — the
//! recovered body has no source-level evidence that `0x3` is "an `fd`"
//! versus "an `int32_t` that happens to be 3". But the *extern
//! forward declaration* for a known libc symbol is the one place
//! where the source-level spelling is fully determined by the API
//! contract: the linker resolves `write` to the system's `write`,
//! whose signature is fixed by POSIX. Rendering the extern as
//! `extern int64_t write(int32_t fd, void *buf, uint64_t n);` is
//! lossy: a reverse-engineer reading the output has to know to
//! translate the integer widths back into the POSIX typedefs.
//!
//! This module ships a parallel table keyed by API name that returns
//! the canonical C-side spelling — typedef names like `ssize_t` and
//! `size_t`, plus the standard headers each typedef requires. The
//! CLI's PLT-extern lowering path consults this table first; on hit
//! the rendered extern matches the system header verbatim. On miss
//! the lowering falls back to the lattice-driven shape so unknown
//! imports keep working without a manual table entry.
//!
//! The mapping is one-way (canonical → render). The lifter's type
//! propagation and the recovery facts continue to operate over the
//! lattice; the canonical spelling is purely a backend rendering
//! choice for the extern decl.
//!
//! ## Determinism
//!
//! All entries are constructed at lookup time from `&'static str`
//! constants; iteration order across the table is stable.

use crate::ast::CType;

/// One canonical extern signature plus the headers its rendering
/// requires. Constructed by [`canonical_extern_signature`] for the
/// libc / POSIX entries this batch ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExtern {
    /// Function return type, spelled with the canonical typedef the
    /// system header uses (`ssize_t`, `int`, `size_t`, `void`, …).
    pub return_type: CType,
    /// Parameters in source order. The `String` is the parameter
    /// name as the API documents it; the `CType` carries the
    /// canonical typedef spelling.
    pub params: Vec<(String, CType)>,
    /// True when the API accepts a trailing `...`.
    pub is_variadic: bool,
    /// Standard headers the rendered extern's typedefs reference,
    /// e.g. `<sys/types.h>` for `ssize_t`. The CLI deduplicates and
    /// prepends these so the translation unit compiles standalone.
    pub headers: Vec<&'static str>,
}

impl CanonicalExtern {
    fn new(
        return_type: CType,
        params: Vec<(&str, CType)>,
        is_variadic: bool,
        headers: &[&'static str],
    ) -> Self {
        Self {
            return_type,
            params: params
                .into_iter()
                .map(|(n, t)| (n.to_string(), t))
                .collect(),
            is_variadic,
            headers: headers.to_vec(),
        }
    }
}

/// Look up the canonical extern signature for a known libc / POSIX
/// API by name. Returns `None` for imports the table does not cover —
/// callers fall back to the lattice-driven shape via
/// [`crate::map_ir_type`] in that case.
///
/// The match is name-exact and case-sensitive; `write` matches POSIX
/// `write`, but `WriteFile` does not (Win32 entries arrive in a
/// follow-up batch once the PE PLT/IAT lowering walk needs them).
#[must_use]
pub fn canonical_extern_signature(name: &str) -> Option<CanonicalExtern> {
    let int_ = || CType::Named("int".to_string());
    let size_t = || CType::Named("size_t".to_string());
    let ssize_t = || CType::Named("ssize_t".to_string());
    let void_ptr = || CType::Ptr(Box::new(CType::Void));
    let const_void_ptr = || CType::Const(Box::new(CType::Ptr(Box::new(CType::Void))));
    let char_ptr = || {
        CType::Ptr(Box::new(CType::Int {
            width_bits: 8,
            signed: true,
        }))
    };
    let const_char_ptr = || CType::Const(Box::new(char_ptr()));
    let file_ptr = || CType::Ptr(Box::new(CType::Named("FILE".to_string())));

    // Header bundles. Each entry lists the headers the typedefs in
    // the signature require; the canonical `<stdint.h>` / `<stddef.h>`
    // includes always emitted by `default_includes` cover the
    // width-tagged integer / `size_t` baseline.
    let sys_types = &["<sys/types.h>"][..];
    let unistd = &["<unistd.h>"][..];
    let stdio = &["<stdio.h>"][..];
    let stdlib = &["<stdlib.h>"][..];
    let string_h = &["<string.h>"][..];
    let fcntl = &["<fcntl.h>"][..];

    match name {
        // ssize_t write(int fd, const void *buf, size_t n);
        "write" => Some(CanonicalExtern::new(
            ssize_t(),
            vec![("fd", int_()), ("buf", const_void_ptr()), ("n", size_t())],
            false,
            &concat_headers(&[sys_types, unistd]),
        )),
        // ssize_t read(int fd, void *buf, size_t n);
        "read" => Some(CanonicalExtern::new(
            ssize_t(),
            vec![("fd", int_()), ("buf", void_ptr()), ("n", size_t())],
            false,
            &concat_headers(&[sys_types, unistd]),
        )),
        // int open(const char *path, int flags, ...);
        "open" => Some(CanonicalExtern::new(
            int_(),
            vec![("path", const_char_ptr()), ("flags", int_())],
            true,
            fcntl,
        )),
        // int close(int fd);
        "close" => Some(CanonicalExtern::new(
            int_(),
            vec![("fd", int_())],
            false,
            unistd,
        )),
        // size_t strlen(const char *s);
        "strlen" => Some(CanonicalExtern::new(
            size_t(),
            vec![("s", const_char_ptr())],
            false,
            string_h,
        )),
        // int strcmp(const char *a, const char *b);
        "strcmp" => Some(CanonicalExtern::new(
            int_(),
            vec![("a", const_char_ptr()), ("b", const_char_ptr())],
            false,
            string_h,
        )),
        // char *strcpy(char *dst, const char *src);
        "strcpy" => Some(CanonicalExtern::new(
            char_ptr(),
            vec![("dst", char_ptr()), ("src", const_char_ptr())],
            false,
            string_h,
        )),
        // char *strncpy(char *dst, const char *src, size_t n);
        "strncpy" => Some(CanonicalExtern::new(
            char_ptr(),
            vec![
                ("dst", char_ptr()),
                ("src", const_char_ptr()),
                ("n", size_t()),
            ],
            false,
            string_h,
        )),
        // void *memcpy(void *dst, const void *src, size_t n);
        "memcpy" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![
                ("dst", void_ptr()),
                ("src", const_void_ptr()),
                ("n", size_t()),
            ],
            false,
            string_h,
        )),
        // void *memmove(void *dst, const void *src, size_t n);
        "memmove" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![
                ("dst", void_ptr()),
                ("src", const_void_ptr()),
                ("n", size_t()),
            ],
            false,
            string_h,
        )),
        // void *memset(void *p, int c, size_t n);
        "memset" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![("p", void_ptr()), ("c", int_()), ("n", size_t())],
            false,
            string_h,
        )),
        // int memcmp(const void *a, const void *b, size_t n);
        "memcmp" => Some(CanonicalExtern::new(
            int_(),
            vec![
                ("a", const_void_ptr()),
                ("b", const_void_ptr()),
                ("n", size_t()),
            ],
            false,
            string_h,
        )),
        // void *malloc(size_t n);
        "malloc" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![("n", size_t())],
            false,
            stdlib,
        )),
        // void *calloc(size_t n, size_t size);
        "calloc" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![("n", size_t()), ("size", size_t())],
            false,
            stdlib,
        )),
        // void *realloc(void *p, size_t n);
        "realloc" => Some(CanonicalExtern::new(
            void_ptr(),
            vec![("p", void_ptr()), ("n", size_t())],
            false,
            stdlib,
        )),
        // void free(void *p);
        "free" => Some(CanonicalExtern::new(
            CType::Void,
            vec![("p", void_ptr())],
            false,
            stdlib,
        )),
        // int printf(const char *fmt, ...);
        "printf" => Some(CanonicalExtern::new(
            int_(),
            vec![("fmt", const_char_ptr())],
            true,
            stdio,
        )),
        // int puts(const char *s);
        "puts" => Some(CanonicalExtern::new(
            int_(),
            vec![("s", const_char_ptr())],
            false,
            stdio,
        )),
        // FILE *fopen(const char *path, const char *mode);
        "fopen" => Some(CanonicalExtern::new(
            file_ptr(),
            vec![("path", const_char_ptr()), ("mode", const_char_ptr())],
            false,
            stdio,
        )),
        // int fclose(FILE *fp);
        "fclose" => Some(CanonicalExtern::new(
            int_(),
            vec![("fp", file_ptr())],
            false,
            stdio,
        )),
        // size_t fread(void *p, size_t size, size_t n, FILE *fp);
        "fread" => Some(CanonicalExtern::new(
            size_t(),
            vec![
                ("p", void_ptr()),
                ("size", size_t()),
                ("n", size_t()),
                ("fp", file_ptr()),
            ],
            false,
            stdio,
        )),
        // size_t fwrite(const void *p, size_t size, size_t n, FILE *fp);
        "fwrite" => Some(CanonicalExtern::new(
            size_t(),
            vec![
                ("p", const_void_ptr()),
                ("size", size_t()),
                ("n", size_t()),
                ("fp", file_ptr()),
            ],
            false,
            stdio,
        )),
        // _Noreturn void exit(int status);
        "exit" => Some(CanonicalExtern::new(
            CType::Void,
            vec![("status", int_())],
            false,
            stdlib,
        )),
        // _Noreturn void abort(void);
        "abort" => Some(CanonicalExtern::new(CType::Void, vec![], false, stdlib)),
        // char *getenv(const char *name);
        "getenv" => Some(CanonicalExtern::new(
            char_ptr(),
            vec![("name", const_char_ptr())],
            false,
            stdlib,
        )),
        _ => None,
    }
}

/// Concatenate header bundles while preserving declaration order.
/// Duplicates are dropped — the CLI's accumulator deduplicates too,
/// but keeping the per-entry list tidy makes the table-level tests
/// easier to write.
fn concat_headers(bundles: &[&[&'static str]]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for b in bundles {
        for h in *b {
            if !out.contains(h) {
                out.push(*h);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_signature_matches_posix() {
        let s = canonical_extern_signature("write").expect("write present");
        assert_eq!(s.return_type, CType::Named("ssize_t".into()));
        assert_eq!(s.params.len(), 3);
        assert_eq!(s.params[0].0, "fd");
        assert_eq!(s.params[0].1, CType::Named("int".into()));
        assert_eq!(s.params[1].0, "buf");
        assert_eq!(
            s.params[1].1,
            CType::Const(Box::new(CType::Ptr(Box::new(CType::Void))))
        );
        assert_eq!(s.params[2].0, "n");
        assert_eq!(s.params[2].1, CType::Named("size_t".into()));
        assert!(!s.is_variadic);
        assert!(s.headers.contains(&"<sys/types.h>"));
    }

    #[test]
    fn read_uses_non_const_void_ptr_for_buf() {
        // The POSIX prototype has `read(int, void *, size_t)` — `buf`
        // is *not* const, unlike `write(int, const void *, size_t)`.
        let s = canonical_extern_signature("read").unwrap();
        assert_eq!(s.params[1].0, "buf");
        assert_eq!(s.params[1].1, CType::Ptr(Box::new(CType::Void)));
    }

    #[test]
    fn printf_is_variadic_with_const_char_ptr_fmt() {
        let s = canonical_extern_signature("printf").unwrap();
        assert_eq!(s.return_type, CType::Named("int".into()));
        assert_eq!(s.params.len(), 1);
        assert_eq!(s.params[0].0, "fmt");
        assert!(s.is_variadic);
        assert!(s.headers.contains(&"<stdio.h>"));
    }

    #[test]
    fn malloc_returns_void_ptr_with_stdlib_header() {
        let s = canonical_extern_signature("malloc").unwrap();
        assert_eq!(s.return_type, CType::Ptr(Box::new(CType::Void)));
        assert_eq!(s.params[0].0, "n");
        assert_eq!(s.params[0].1, CType::Named("size_t".into()));
        assert!(s.headers.contains(&"<stdlib.h>"));
    }

    #[test]
    fn fopen_uses_file_pointer_typedef() {
        let s = canonical_extern_signature("fopen").unwrap();
        assert_eq!(
            s.return_type,
            CType::Ptr(Box::new(CType::Named("FILE".into())))
        );
        assert_eq!(s.params.len(), 2);
        assert_eq!(s.params[0].0, "path");
    }

    #[test]
    fn unknown_name_returns_none() {
        assert!(canonical_extern_signature("not_a_real_api").is_none());
    }

    #[test]
    fn case_sensitive_match() {
        // POSIX names are case-sensitive; the Win32 catalogue lives in
        // a separate space that this table will not match.
        assert!(canonical_extern_signature("Write").is_none());
        assert!(canonical_extern_signature("WriteFile").is_none());
    }

    #[test]
    fn libc_minimal_set_has_canonical_entries() {
        for name in [
            "write", "read", "open", "close", "strlen", "strcmp", "strcpy", "strncpy", "memcpy",
            "memmove", "memset", "memcmp", "malloc", "calloc", "realloc", "free", "printf", "puts",
            "fopen", "fclose", "fread", "fwrite", "exit", "abort", "getenv",
        ] {
            assert!(
                canonical_extern_signature(name).is_some(),
                "missing canonical entry for {name}"
            );
        }
    }

    #[test]
    fn header_bundles_are_deduplicated_per_entry() {
        // `write` lists both `<sys/types.h>` (for ssize_t / size_t)
        // and `<unistd.h>` (for the API itself); neither header is
        // repeated inside the entry.
        let s = canonical_extern_signature("write").unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for h in &s.headers {
            assert!(seen.insert(*h), "duplicate header: {h}");
        }
    }
}
