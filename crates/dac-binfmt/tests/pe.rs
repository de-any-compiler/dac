//! PE round-trip tests against the shared fixtures.
//!
//! Closes the B1.2 done-when: the parser handles a sample console exe,
//! a stripped variant, and a sample DLL with exports, all from real
//! mingw-w64 output. The same `bridge::parse_object` machinery that
//! drives ELF is being exercised here through PE flag variants, so the
//! tests focus on PE-specific behaviour (DLL imports, export table,
//! `IMAGE_SCN_*` permissions, COFF symbol-table presence).

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
fn hello_pe_basic_shape() {
    let m = load("hello-x86_64.exe");
    assert_eq!(m.format, BinaryFormat::Pe);
    assert_eq!(m.architecture, Architecture::X86_64);
    assert_eq!(m.endian, Endian::Little);
    assert_eq!(m.bits, Bits::Bits64);
    assert!(m.entry.is_some(), "PE executable has an entry point");
    assert!(!m.sections.is_empty(), "PE has sections");
    assert!(!m.segments.is_empty(), "PE has segments");
    assert!(
        m.sections.iter().any(|s| s.kind == SectionKind::Text),
        "PE has a .text section"
    );
}

#[test]
fn hello_pe_has_canonical_section_names() {
    let m = load("hello-x86_64.exe");
    let names: Vec<&str> = m.sections.iter().map(|s| s.name.as_str()).collect();
    for expected in [".text", ".data", ".rdata", ".idata"] {
        assert!(
            names.contains(&expected),
            "missing section {expected:?} in {names:?}"
        );
    }
}

#[test]
fn hello_pe_text_section_is_executable() {
    let m = load("hello-x86_64.exe");
    let text = m
        .sections
        .iter()
        .find(|s| s.name == ".text")
        .expect(".text present");
    assert!(
        text.perms.readable && text.perms.executable && !text.perms.writable,
        "PE .text should be R+X (IMAGE_SCN_MEM_READ|EXECUTE), got {:?}",
        text.perms
    );
    let data = m
        .sections
        .iter()
        .find(|s| s.name == ".data")
        .expect(".data present");
    assert!(
        data.perms.readable && data.perms.writable && !data.perms.executable,
        "PE .data should be R+W (IMAGE_SCN_MEM_READ|WRITE), got {:?}",
        data.perms
    );
}

#[test]
fn hello_pe_has_main_in_symtab() {
    let m = load("hello-x86_64.exe");
    assert!(
        m.symbols
            .iter()
            .any(|s| s.source == SymbolSource::Symtab && s.name == "main"),
        "expected `main` in COFF symbol table"
    );
}

#[test]
fn hello_pe_imports_kernel32_and_a_known_function() {
    let m = load("hello-x86_64.exe");
    // FR-6 (linkage info): DLL grouping reaches `needed_libraries`.
    assert!(
        m.needed_libraries
            .iter()
            .any(|lib| lib.eq_ignore_ascii_case("KERNEL32.dll")),
        "expected KERNEL32.dll in needed_libraries, got {:?}",
        m.needed_libraries
    );
    // The minimal CRT pulled in by mingw-w64 always lands at least one
    // KERNEL32 function in the import table.
    assert!(
        m.imports.iter().any(|i| matches!(i.library.as_deref(),
                Some(lib) if lib.eq_ignore_ascii_case("KERNEL32.dll"))),
        "expected at least one KERNEL32.dll import"
    );
}

#[test]
fn stripped_pe_has_no_coff_symbol_table_but_keeps_imports() {
    let m = load("hello-x86_64-stripped.exe");
    assert!(
        m.symbols.is_empty(),
        "stripped PE should not retain COFF symbols; got {} entries",
        m.symbols.len()
    );
    // Imports must survive — they are required at load time, not optional
    // debug info. The DLL set should match the unstripped binary.
    assert!(
        !m.imports.is_empty(),
        "stripped PE keeps its imports (load-time requirement)"
    );
    let full = std::fs::metadata(fixture_path("hello-x86_64.exe"))
        .unwrap()
        .len();
    let stripped = std::fs::metadata(fixture_path("hello-x86_64-stripped.exe"))
        .unwrap()
        .len();
    assert!(
        stripped < full,
        "stripped fixture should be smaller than the unstripped (was {stripped} vs {full})"
    );
}

#[test]
fn sample_dll_exports_three_symbols() {
    let m = load("sample.dll");
    assert_eq!(m.format, BinaryFormat::Pe);
    let exported_names: Vec<&str> = m.exports.iter().map(|e| e.name.as_str()).collect();
    for expected in ["sample_add", "sample_greeting", "sample_value"] {
        assert!(
            exported_names.contains(&expected),
            "missing export {expected:?} in {exported_names:?}"
        );
    }
}

#[test]
fn sample_dll_has_function_symbols_for_exported_routines() {
    let m = load("sample.dll");
    let has_add = m
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Text && s.name == "sample_add");
    let has_greeting = m
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Text && s.name == "sample_greeting");
    assert!(has_add, "expected FUNC sample_add");
    assert!(has_greeting, "expected FUNC sample_greeting");
}

#[test]
fn sample_dll_extracts_embedded_string() {
    let m = load("sample.dll");
    let found = m
        .strings
        .iter()
        .any(|s| s.value.contains("hello from sample DLL"));
    assert!(
        found,
        "expected 'hello from sample DLL' in extracted strings; got {:?}",
        m.strings
            .iter()
            .map(|s| &s.value)
            .take(20)
            .collect::<Vec<_>>()
    );
}

/// FR-2 (auto-detection): the same byte buffer that the magic-byte
/// detector tags as PE must reach the PE parser, not the ELF parser.
/// We assert the round-trip through [`dac_binfmt::detect_format`] and
/// [`dac_binfmt::load_from_bytes`] both agree on `BinaryFormat::Pe`.
#[test]
fn pe_fixture_is_auto_detected_and_dispatched() {
    let bytes = std::fs::read(fixture_path("hello-x86_64.exe")).unwrap();
    assert_eq!(dac_binfmt::detect_format(&bytes).unwrap(), BinaryFormat::Pe);
    let m = dac_binfmt::load_from_bytes(&bytes).unwrap();
    assert_eq!(m.format, BinaryFormat::Pe);

    let elf_bytes = std::fs::read(fixture_path("hello-x86_64")).unwrap();
    assert_eq!(
        dac_binfmt::detect_format(&elf_bytes).unwrap(),
        BinaryFormat::Elf
    );
    let elf_model = dac_binfmt::load_from_bytes(&elf_bytes).unwrap();
    assert_eq!(elf_model.format, BinaryFormat::Elf);
}
