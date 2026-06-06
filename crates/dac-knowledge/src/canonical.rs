//! Canonical entry-point signatures (B3.28, FR-12 / FR-21).
//!
//! A small curated table of process entry-point signatures keyed by
//! symbol. Unlike [`crate::api`], whose entries describe *callees* the
//! lifter can recognise at call sites, canonical entries describe
//! *the function being lifted itself* — the runtime-mandated shape of
//! `main`, `wmain`, and `WinMain`.
//!
//! ## Why a separate table
//!
//! The convention inference pass scores arg-register prefixes and
//! return-register usage from observed SSA reads. That works on every
//! interior function, but it under-claims for `main`: the C standard
//! pins `main`'s return type to `int` and its argument list to either
//! `(void)` or `(int argc, char **argv)`, regardless of whether the
//! body actually reads `rdi` / `rsi`. The convention pass cannot
//! invent that knowledge.
//!
//! The orchestrator overlays this table after convention inference
//! (and after the user-hint catalogue), so the recovered signature
//! reflects the runtime contract for canonical entries while leaving
//! every other function untouched.
//!
//! ## Shape
//!
//! Each entry carries:
//! - A name keyed against the recovered symbol (case-sensitive — the
//!   compiler does not lower-case `WinMain`).
//! - A C-canonical return-type spelling (`"int"`) and the matching
//!   [`Type`] lattice element so both the C backend and the type
//!   propagation pass see the same fact (I-3).
//! - An ordered list of [`CanonicalArg`]s. Each arg pairs a name
//!   (`"argc"`, `"argv"`) with a C-canonical spelling and an IR-type
//!   marker. The argument list is *maximal* — `main`'s table entry
//!   has 2 args even though `main(void)` is legal. The orchestrator
//!   clips the list to the prefix the convention pass actually
//!   observed, so a binary whose `main` reads no arguments still
//!   renders as `int main(void)`.
//!
//! ## Determinism
//!
//! All entries are constructed at compile time (or behind a
//! [`std::sync::LazyLock`] when the type lattice contains `Ptr`).
//! Iteration order is stable — [`lookup_canonical_entry`] walks the
//! table in declaration order.

use std::sync::LazyLock;

use dac_ir::Type;

/// One canonical entry-point signature.
///
/// The C-canonical spellings (`"int"`, `"char **"`) feed the C
/// backend's `CType::Named` channel directly so the rendered
/// signature reads the way a C programmer would have written it
/// rather than the stdint-style `int32_t` / `int8_t **` the type
/// propagation pass would otherwise produce.
#[derive(Debug, Clone)]
pub struct CanonicalEntry {
    /// Symbol name as it appears in the binary's symbol table.
    /// Case-sensitive: `main`, `wmain`, `WinMain`.
    pub name: &'static str,
    /// C-canonical return-type spelling (`"int"`).
    pub return_c_type: &'static str,
    /// Lattice element matching `return_c_type`. The orchestrator
    /// seeds this into the [`TypeMap`](crate) entry for every
    /// `Return { value: Some(_) }` operand so the annotation channel
    /// and the C backend agree on the type (I-3).
    pub return_ir_type: Type,
    /// Maximal argument list in source order. The orchestrator
    /// truncates this to the observed convention-arg prefix length.
    pub args: Vec<CanonicalArg>,
}

/// One canonical-entry argument slot.
#[derive(Debug, Clone)]
pub struct CanonicalArg {
    /// Idiomatic parameter name (`"argc"`, `"argv"`).
    pub name: &'static str,
    /// C-canonical type spelling (`"int"`, `"char **"`).
    pub c_type: &'static str,
    /// Lattice element matching `c_type`.
    pub ir_type: Type,
}

/// Look up a canonical entry by name. Returns `None` when no entry
/// matches — the convention-inferred signature stays in force.
#[must_use]
pub fn lookup_canonical_entry(name: &str) -> Option<&'static CanonicalEntry> {
    CANONICAL_ENTRIES.iter().find(|e| e.name == name)
}

/// All canonical entries in declaration order.
#[must_use]
pub fn canonical_entries() -> &'static [CanonicalEntry] {
    CANONICAL_ENTRIES.as_slice()
}

static CANONICAL_ENTRIES: LazyLock<Vec<CanonicalEntry>> = LazyLock::new(build_canonical_entries);

fn build_canonical_entries() -> Vec<CanonicalEntry> {
    vec![
        CanonicalEntry {
            name: "main",
            return_c_type: "int",
            return_ir_type: Type::signed_int(32),
            args: vec![
                CanonicalArg {
                    name: "argc",
                    c_type: "int",
                    ir_type: Type::signed_int(32),
                },
                CanonicalArg {
                    name: "argv",
                    c_type: "char **",
                    ir_type: Type::ptr_to(Type::ptr_to(Type::signed_int(8))),
                },
            ],
        },
        CanonicalEntry {
            name: "wmain",
            return_c_type: "int",
            return_ir_type: Type::signed_int(32),
            args: vec![
                CanonicalArg {
                    name: "argc",
                    c_type: "int",
                    ir_type: Type::signed_int(32),
                },
                CanonicalArg {
                    name: "argv",
                    // `wchar_t` is platform-dependent; on Windows the
                    // Microsoft C runtime defines it as `unsigned short`
                    // but spell it as the typedef so the round-trip
                    // compile gate uses the platform's actual width.
                    c_type: "wchar_t **",
                    ir_type: Type::ptr_to(Type::ptr_to(Type::unsigned_int(16))),
                },
            ],
        },
        CanonicalEntry {
            name: "WinMain",
            return_c_type: "int",
            return_ir_type: Type::signed_int(32),
            args: vec![
                CanonicalArg {
                    name: "hInstance",
                    c_type: "HINSTANCE",
                    ir_type: Type::ptr_to(Type::Unknown),
                },
                CanonicalArg {
                    name: "hPrevInstance",
                    c_type: "HINSTANCE",
                    ir_type: Type::ptr_to(Type::Unknown),
                },
                CanonicalArg {
                    name: "lpCmdLine",
                    c_type: "LPSTR",
                    ir_type: Type::ptr_to(Type::signed_int(8)),
                },
                CanonicalArg {
                    name: "nCmdShow",
                    c_type: "int",
                    ir_type: Type::signed_int(32),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_ir::{IntType, Signedness};

    #[test]
    fn lookup_main_returns_canonical_entry() {
        let e = lookup_canonical_entry("main").expect("main present");
        assert_eq!(e.name, "main");
        assert_eq!(e.return_c_type, "int");
        assert_eq!(
            e.return_ir_type,
            Type::Int(IntType {
                width_bits: 32,
                signedness: Signedness::Signed,
            })
        );
        assert_eq!(e.args.len(), 2);
        assert_eq!(e.args[0].name, "argc");
        assert_eq!(e.args[1].name, "argv");
        assert_eq!(e.args[1].c_type, "char **");
    }

    #[test]
    fn lookup_wmain_returns_canonical_entry() {
        let e = lookup_canonical_entry("wmain").expect("wmain present");
        assert_eq!(e.return_c_type, "int");
        assert_eq!(e.args[1].c_type, "wchar_t **");
    }

    #[test]
    fn lookup_winmain_returns_canonical_entry() {
        let e = lookup_canonical_entry("WinMain").expect("WinMain present");
        assert_eq!(e.args.len(), 4);
        assert_eq!(e.args[3].c_type, "int");
    }

    #[test]
    fn lookup_is_case_sensitive() {
        assert!(lookup_canonical_entry("Main").is_none());
        assert!(lookup_canonical_entry("WINMAIN").is_none());
        assert!(lookup_canonical_entry("winmain").is_none());
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_canonical_entry("strlen").is_none());
        assert!(lookup_canonical_entry("frame_dummy").is_none());
    }

    #[test]
    fn canonical_entries_listed_in_table_order() {
        let names: Vec<&str> = canonical_entries().iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["main", "wmain", "WinMain"]);
    }
}
