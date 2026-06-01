//! ELF round-trip tests against the shared fixtures.
//!
//! Closes the B1.1 done-when: the parser handles a sample hello-world,
//! a stripped variant, and a shared library, all from real GCC output.
//! A best-effort probe of the system `libc.so.6` runs on Linux when
//! present (skipped silently elsewhere) so the parser sees a non-trivial
//! shared library on at least one CI runner.

use std::path::PathBuf;

use dac_binfmt::{
    Architecture, BinaryFormat, BinaryModel, Bits, Endian, SectionKind, SymbolKind, SymbolSource,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load(name: &str) -> BinaryModel {
    let path = fixture_path(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    dac_binfmt::load_from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()))
}

#[test]
fn hello_x86_64_basic_shape() {
    let m = load("hello-x86_64");
    assert_eq!(m.format, BinaryFormat::Elf);
    assert_eq!(m.architecture, Architecture::X86_64);
    assert_eq!(m.endian, Endian::Little);
    assert_eq!(m.bits, Bits::Bits64);
    assert!(m.entry.is_some(), "hello-x86_64 has an entry point");
    assert!(!m.sections.is_empty(), "hello-x86_64 has sections");
    assert!(!m.segments.is_empty(), "hello-x86_64 has program headers");
    assert!(
        m.sections.iter().any(|s| s.kind == SectionKind::Text),
        "hello-x86_64 has a .text section"
    );
}

#[test]
fn hello_x86_64_has_static_symbol_table_with_main() {
    let m = load("hello-x86_64");
    assert!(
        m.symbols
            .iter()
            .any(|s| s.source == SymbolSource::Symtab && s.name == "main"),
        "expected `main` in .symtab"
    );
}

#[test]
fn hello_x86_64_needs_libc() {
    let m = load("hello-x86_64");
    assert!(
        m.needed_libraries
            .iter()
            .any(|lib| lib == "libc.so.6" || lib.starts_with("libc.so")),
        "expected libc.so.6 in needed_libraries, got {:?}",
        m.needed_libraries
    );
}

#[test]
fn hello_x86_64_imports_write_through_libc() {
    let m = load("hello-x86_64");
    assert!(
        m.imports.iter().any(|i| i.name == "write"),
        "expected write@libc in imports"
    );
}

#[test]
fn stripped_hello_has_no_symtab_but_keeps_dynamic_symbols() {
    let m = load("hello-x86_64-stripped");
    assert!(
        m.symbols.iter().all(|s| s.source != SymbolSource::Symtab),
        "stripped binary must not retain .symtab entries; got {} of them",
        m.symbols
            .iter()
            .filter(|s| s.source == SymbolSource::Symtab)
            .count()
    );
    assert!(
        m.symbols.iter().any(|s| s.source == SymbolSource::Dynsym),
        "stripped binary still keeps .dynsym (it is needed at runtime)"
    );
    // Sanity: the file shrinks. Both binaries are in the workspace.
    let full = std::fs::metadata(fixture_path("hello-x86_64"))
        .unwrap()
        .len();
    let stripped = std::fs::metadata(fixture_path("hello-x86_64-stripped"))
        .unwrap()
        .len();
    assert!(
        stripped < full,
        "stripped fixture should be smaller than the unstripped (was {stripped} vs {full})"
    );
}

#[test]
fn libsample_so_exports_sample_symbols() {
    let m = load("libsample.so");
    assert_eq!(m.format, BinaryFormat::Elf);
    let exported_names: Vec<&str> = m.exports.iter().map(|e| e.name.as_str()).collect();
    for expected in ["sample_add", "sample_greeting", "sample_value"] {
        assert!(
            exported_names.contains(&expected),
            "missing export {expected:?} in {exported_names:?}"
        );
    }
}

#[test]
fn libsample_so_has_function_and_object_symbols() {
    let m = load("libsample.so");
    let has_function = m
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Text && s.name == "sample_add");
    let has_object = m
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Data && s.name == "sample_value");
    assert!(has_function, "expected FUNC sample_add");
    assert!(has_object, "expected OBJECT sample_value");
}

#[test]
fn libsample_so_relocations_reference_known_symbols() {
    let m = load("libsample.so");
    assert!(
        !m.relocations.is_empty(),
        "libsample.so should have dynamic relocations"
    );
    for r in &m.relocations {
        if let Some(idx) = r.symbol {
            assert!(
                idx < m.symbols.len(),
                "relocation symbol index out of bounds: {idx} vs {}",
                m.symbols.len()
            );
        }
    }
}

#[test]
fn libsample_so_extracts_the_embedded_string() {
    let m = load("libsample.so");
    let found = m
        .strings
        .iter()
        .any(|s| s.value.contains("hello from libsample"));
    assert!(
        found,
        "expected 'hello from libsample' in extracted strings; got {:?}",
        m.strings.iter().map(|s| &s.value).collect::<Vec<_>>()
    );
}

/// Best-effort smoke against the system libc shared object — the
/// canonical "real" shared library called out by the B1.1 done-when.
/// Skips silently when the file is absent (non-Linux runners, musl, etc.).
#[test]
fn system_libc_parses_when_present() {
    let candidates = [
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib64/libc.so.6",
        "/usr/lib/libc.so.6",
        "/usr/lib64/libc.so.6",
    ];
    let Some(path) = candidates.iter().find(|p| std::path::Path::new(p).exists()) else {
        eprintln!("skipping system libc test: no libc.so.6 at known paths");
        return;
    };
    let bytes = std::fs::read(path).expect("read system libc");
    let m = dac_binfmt::load_from_bytes(&bytes).expect("parse system libc");
    assert_eq!(m.format, BinaryFormat::Elf);
    assert!(
        !m.exports.is_empty(),
        "libc.so.6 should export thousands of symbols"
    );
    assert!(!m.sections.is_empty(), "libc.so.6 should have sections");
    // libc's relocation count is in the thousands; just assert it's non-empty
    // so the test does not depend on a specific glibc build.
    assert!(
        !m.relocations.is_empty(),
        "libc.so.6 should have relocations"
    );
}
