//! CRT / startup helper catalogue (B3.30, FR-21).
//!
//! Curated table of C runtime / startup helper symbols. Recognising
//! these by name lets the C backend tag the matched function with a
//! `/* runtime support (<runtime>) — not user code */` banner and the
//! report carry a `crt_support=N` counter so a reviewer can tell at a
//! glance how many emitted bodies are scaffolding (not the user's
//! code).
//!
//! ## What gets in
//!
//! Entries for the GNU C library's startup family (`_init`, `_fini`,
//! `_start`, `frame_dummy`, `register_tm_clones`, `deregister_tm_clones`,
//! `__do_global_dtors_aux`, the libgcc-eh and libitm helpers that
//! `-no-pie` emits) and the MinGW-w64 runtime entries that ship on
//! every `-static-libgcc` Windows binary (the `__tmainCRTStartup` /
//! `mainCRTStartup` / `WinMainCRTStartup` family, the `__do_global_*`
//! pair, the `__mingw_*` family, the PE-helper functions
//! `_FindPESection`, `_GetPEImageBase`, …).
//!
//! ## Determinism
//!
//! All entries are constructed at compile time as `&'static`
//! references. [`lookup_crt_entry`] performs an in-order linear scan
//! (the table is small — sub-100 entries — and the lookup happens at
//! most once per discovered function).

/// One CRT / startup-helper signature.
#[derive(Debug, Clone, Copy)]
pub struct CrtEntry {
    /// Symbol name as it appears in the binary's symbol table.
    /// Case-sensitive — `_init` and `_INIT` are distinct.
    pub name: &'static str,
    /// Runtime family the helper belongs to. Drives the banner text
    /// the C backend prints above the matched function.
    pub runtime: CrtRuntime,
    /// One-line description of the helper's role. Surfaces in the
    /// annotation channel (FR-21) so a reader inspecting the sidecar
    /// can read *what* the helper does without leaving dac.
    pub role: &'static str,
}

/// CRT runtime families dac currently recognises.
///
/// The banner text the C backend prints uses [`CrtRuntime::label`] so
/// adding a new family stays a one-line change here. The enum is
/// `#[non_exhaustive]` so downstream code matching on it has to keep
/// a `_` arm and dac can grow the catalogue without breaking
/// out-of-tree consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CrtRuntime {
    /// GNU C library startup helpers (`_init`, `_fini`, `_start`,
    /// `frame_dummy`, …). The most common ELF surface.
    Glibc,
    /// MinGW-w64 startup helpers (`__tmainCRTStartup`,
    /// `__do_global_ctors`, the `__mingw_*` family). The most common
    /// PE surface produced by `x86_64-w64-mingw32-gcc`.
    MingwW64,
}

impl CrtRuntime {
    /// Short human-readable label used in the `/* runtime support …
    /// */` banner the C backend prints above a CRT body. Stable
    /// across versions so a reader who has memorised the phrasing
    /// does not have to re-learn it.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            CrtRuntime::Glibc => "glibc startup",
            CrtRuntime::MingwW64 => "mingw-w64 startup",
        }
    }
}

/// Look up a CRT entry by name. Returns `None` when no entry matches.
///
/// The match is exact (case-sensitive); MinGW-w64 mangles a handful
/// of helpers with a leading underscore on PE / COFF but not on ELF,
/// so callers do not need to strip prefixes.
#[must_use]
pub fn lookup_crt_entry(name: &str) -> Option<&'static CrtEntry> {
    CRT_ENTRIES.iter().find(|e| e.name == name)
}

/// All CRT entries in declaration order.
///
/// Used by `dac-cli::report` to render a stable `;; taxonomy:` row
/// histogram, and by the test suite to verify the table covers a
/// fixture's full CRT surface in one shot.
#[must_use]
pub fn crt_entries() -> &'static [CrtEntry] {
    CRT_ENTRIES
}

const CRT_ENTRIES: &[CrtEntry] = &[
    // ---- glibc startup family (ELF) ----
    CrtEntry {
        name: "_start",
        runtime: CrtRuntime::Glibc,
        role: "process entry point; sets up argc/argv and calls __libc_start_main",
    },
    CrtEntry {
        name: "_init",
        runtime: CrtRuntime::Glibc,
        role: "DT_INIT array runner; called by the dynamic loader before main",
    },
    CrtEntry {
        name: "_fini",
        runtime: CrtRuntime::Glibc,
        role: "DT_FINI array runner; called by the dynamic loader at process exit",
    },
    CrtEntry {
        name: "frame_dummy",
        runtime: CrtRuntime::Glibc,
        role: "crtbegin.o helper that registers .eh_frame with __register_frame_info",
    },
    CrtEntry {
        name: "register_tm_clones",
        runtime: CrtRuntime::Glibc,
        role: "crtbegin.o transactional-memory clone registrar (no-op when libitm is absent)",
    },
    CrtEntry {
        name: "deregister_tm_clones",
        runtime: CrtRuntime::Glibc,
        role: "crtbegin.o transactional-memory clone deregistrar",
    },
    CrtEntry {
        name: "__do_global_dtors_aux",
        runtime: CrtRuntime::Glibc,
        role: "crtbegin.o destructor-list runner invoked from _fini",
    },
    CrtEntry {
        name: "__do_global_ctors_aux",
        runtime: CrtRuntime::Glibc,
        role: "crtbegin.o constructor-list runner invoked from _init",
    },
    CrtEntry {
        name: "__libc_csu_init",
        runtime: CrtRuntime::Glibc,
        role: "glibc's csu/csu-init runner; iterates __init_array between _init and main",
    },
    CrtEntry {
        name: "__libc_csu_fini",
        runtime: CrtRuntime::Glibc,
        role: "glibc's csu/csu-fini runner; iterates __fini_array after main returns",
    },
    CrtEntry {
        name: "_dl_relocate_static_pie",
        runtime: CrtRuntime::Glibc,
        role: "static-PIE relocation helper run before main when the loader is the binary itself",
    },
    // ---- MinGW-w64 startup family (PE) ----
    CrtEntry {
        name: "__tmainCRTStartup",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW common CRT entry shared by mainCRTStartup and WinMainCRTStartup",
    },
    CrtEntry {
        name: "mainCRTStartup",
        runtime: CrtRuntime::MingwW64,
        role: "console-subsystem entry point; dispatches into __tmainCRTStartup",
    },
    CrtEntry {
        name: "WinMainCRTStartup",
        runtime: CrtRuntime::MingwW64,
        role: "GUI-subsystem entry point; dispatches into __tmainCRTStartup",
    },
    CrtEntry {
        name: "__do_global_dtors",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW destructor-list runner invoked from atexit",
    },
    CrtEntry {
        name: "__do_global_ctors",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW constructor-list runner invoked from __tmainCRTStartup",
    },
    CrtEntry {
        name: "__main",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW C++-constructor bootstrap called from the first line of main",
    },
    CrtEntry {
        name: "_setargv",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW argv-expansion hook (weak by default; user can override)",
    },
    CrtEntry {
        name: "_pei386_runtime_relocator",
        runtime: CrtRuntime::MingwW64,
        role: "PE-image runtime relocation runner for pseudo-reloc fixups",
    },
    CrtEntry {
        name: "__gcc_register_frame",
        runtime: CrtRuntime::MingwW64,
        role: "libgcc-eh helper that registers .eh_frame_hdr with the unwinder",
    },
    CrtEntry {
        name: "__gcc_deregister_frame",
        runtime: CrtRuntime::MingwW64,
        role: "libgcc-eh helper that deregisters .eh_frame_hdr at process exit",
    },
    CrtEntry {
        name: "register_frame_ctor",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW DllMain-style ctor that calls __gcc_register_frame",
    },
    CrtEntry {
        name: "__dyn_tls_init",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW thread-local-storage initialiser (per-thread callback)",
    },
    CrtEntry {
        name: "__dyn_tls_dtor",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW thread-local-storage destructor (per-thread callback)",
    },
    CrtEntry {
        name: "__tlregdtor",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW TLS-destructor registrar called from C++ destructors",
    },
    CrtEntry {
        name: "__mingw_TLScallback",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW image-TLS-directory callback dispatched by the loader",
    },
    CrtEntry {
        name: "__mingw_SEH_error_handler",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW structured-exception handler emitted into .pdata",
    },
    CrtEntry {
        name: "__mingw_invalidParameterHandler",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW handler that swallows MSVCRT invalid-parameter aborts",
    },
    CrtEntry {
        name: "__mingw_raise_matherr",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW shim that forwards matherr() into the user's __setusermatherr hook",
    },
    CrtEntry {
        name: "__mingw_setusermatherr",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW _matherr-installer; called from the user once or default to no-op",
    },
    CrtEntry {
        name: "__setusermatherr",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW wrapper that installs the user _matherr through MSVCRT",
    },
    CrtEntry {
        name: "_matherr",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW default matherr() implementation; weak so users can override",
    },
    CrtEntry {
        name: "_fpreset",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW x87 FPU reset stub invoked by signal() and longjmp()",
    },
    CrtEntry {
        name: "__mingwthr_run_key_dtors_part_0",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW per-thread TLS-key destructor runner (part 0 of the chain)",
    },
    CrtEntry {
        name: "___w64_mingwthr_add_key_dtor",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW thread-key destructor registrar (w64 thunk)",
    },
    CrtEntry {
        name: "___w64_mingwthr_remove_key_dtor",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW thread-key destructor deregistrar (w64 thunk)",
    },
    CrtEntry {
        name: "__mingw_enum_import_library_names",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW IAT-walking helper that enumerates the import-table DLLs",
    },
    CrtEntry {
        name: "__mingw_GetSectionCount",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW PE-section-count helper consumed by the relocator",
    },
    CrtEntry {
        name: "__mingw_GetSectionForAddress",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW reverse PE-section lookup keyed by virtual address",
    },
    CrtEntry {
        name: "_GetPEImageBase",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW helper that returns the image base of the current PE",
    },
    CrtEntry {
        name: "_IsNonwritableInCurrentImage",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW page-permission check used by the pseudo-reloc runner",
    },
    CrtEntry {
        name: "_FindPESection",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW PE-section locator keyed by RVA",
    },
    CrtEntry {
        name: "_FindPESectionByName",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW PE-section locator keyed by section name",
    },
    CrtEntry {
        name: "_FindPESectionExec",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW PE-section locator restricted to executable sections",
    },
    CrtEntry {
        name: "mark_section_writable",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW VirtualProtect-style helper used by the pseudo-reloc runner",
    },
    CrtEntry {
        name: "_ValidateImageBase",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW MZ-magic-check helper for the loaded image base",
    },
    CrtEntry {
        name: "_gnu_exception_handler",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW glue between Windows SEH and POSIX signal() handlers",
    },
    CrtEntry {
        name: "_amsg_exit",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW MSVCRT-style fatal-error exit forwarder",
    },
    CrtEntry {
        name: "_cexit",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW C-runtime cleanup invoked from exit() and _exit()",
    },
    CrtEntry {
        name: "safe_flush",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW atexit-registered stdio flush wrapper",
    },
    CrtEntry {
        name: "__report_error",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW fatal-error printer used by the pseudo-reloc runner",
    },
    CrtEntry {
        name: "__local_stdio_printf_options",
        runtime: CrtRuntime::MingwW64,
        role: "MinGW UCRT printf-options accessor injected by stdio.h",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity check: every entry's `name` is non-empty and its
    /// `role` is descriptive. Lookups in declaration order match the
    /// returned reference.
    #[test]
    fn crt_entries_are_well_formed() {
        let entries = crt_entries();
        assert!(!entries.is_empty());
        for e in entries {
            assert!(!e.name.is_empty(), "empty name in CRT table");
            assert!(!e.role.is_empty(), "empty role for {}", e.name);
        }
    }

    /// The 7 glibc-startup helpers exercised by the hello-x86_64 ELF
    /// fixture must all match — this is the done-when criterion for
    /// B3.30.
    #[test]
    fn b3_30_hello_x86_64_crt_helpers_all_resolve() {
        for name in [
            "_init",
            "_start",
            "deregister_tm_clones",
            "register_tm_clones",
            "__do_global_dtors_aux",
            "frame_dummy",
            "_fini",
        ] {
            let e =
                lookup_crt_entry(name).unwrap_or_else(|| panic!("expected CRT entry for {name}"));
            assert_eq!(e.runtime, CrtRuntime::Glibc);
            assert_eq!(e.name, name);
        }
    }

    /// The MinGW-w64 startup trio used by the hello-x86_64 PE fixture
    /// (`__tmainCRTStartup`, `mainCRTStartup`, `WinMainCRTStartup`)
    /// all resolve to a `MingwW64` runtime so the banner reads
    /// "mingw-w64 startup" on a PE binary.
    #[test]
    fn b3_30_mingw_startup_trio_resolves() {
        for name in ["__tmainCRTStartup", "mainCRTStartup", "WinMainCRTStartup"] {
            let e =
                lookup_crt_entry(name).unwrap_or_else(|| panic!("expected CRT entry for {name}"));
            assert_eq!(e.runtime, CrtRuntime::MingwW64);
        }
    }

    /// A user-program symbol should not match. Negative tests are
    /// cheap and lock in the contract that the table is curated, not
    /// pattern-based.
    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_crt_entry("main").is_none());
        assert!(lookup_crt_entry("strlen").is_none());
        assert!(lookup_crt_entry("").is_none());
    }

    /// Case sensitivity: `_INIT` is not `_init`.
    #[test]
    fn lookup_is_case_sensitive() {
        assert!(lookup_crt_entry("_INIT").is_none());
        assert!(lookup_crt_entry("MAIN_CRT_STARTUP").is_none());
    }

    /// The label is what the C backend prints in the banner. Lock it
    /// in so a future rename of the variant does not silently shift
    /// the rendered string.
    #[test]
    fn crt_runtime_label_is_stable() {
        assert_eq!(CrtRuntime::Glibc.label(), "glibc startup");
        assert_eq!(CrtRuntime::MingwW64.label(), "mingw-w64 startup");
    }
}
