//! Integration test: assert lifter coverage on the workspace's sample
//! `.text` sections meets the B1.4 ≥ 95% gate.
//!
//! Both the ELF and PE fixtures used by the decoder round-trip test
//! are lifted instruction-by-instruction; every node is fed to a
//! [`Coverage`] report. The report's `lifted / total` ratio is the
//! number the batch is graded on. Opaque-mnemonic histograms are
//! printed on failure so the next pass knows what to model next.
//!
//! Why both fixtures: the ELF fixture covers SysV-style compiler
//! output (push/pop/lea/mov/cmp/jcc/call/ret); the PE fixture covers
//! a Windows-style prelude. A single fixture would let an opcode
//! oversight pass unnoticed.

use std::path::PathBuf;

use dac_arch::{Architecture, ControlFlow, Coverage};
use dac_arch_x86::X86_64;
use dac_binfmt::{BinaryModel, SectionKind};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load(name: &str) -> (Vec<u8>, BinaryModel) {
    let path = fixture_path(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    let model = dac_binfmt::load_from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()));
    (bytes, model)
}

fn text_bytes<'a>(model: &BinaryModel, bytes: &'a [u8]) -> (&'a [u8], u64) {
    let section = model
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Text && s.name == ".text")
        .expect(".text section present");
    let offset = section.file_offset.expect(".text has file backing") as usize;
    let size = section.size as usize;
    assert!(
        offset + size <= bytes.len(),
        ".text [{offset}+{size}] must fit in fixture ({} bytes)",
        bytes.len(),
    );
    (&bytes[offset..offset + size], section.address)
}

/// Lift every instruction the decoder produces for the fixture's
/// `.text` and assert the resulting [`Coverage`] meets the gate. The
/// decoder is the source of truth for instruction boundaries; the
/// lifter is the thing under test. Pairing them this way means a
/// regression in either crate fails this test loudly.
fn assert_corpus_coverage(fixture: &str, floor: f64) {
    let (bytes, model) = load(fixture);
    let (text, va) = text_bytes(&model, &bytes);
    assert!(!text.is_empty(), "{fixture}: .text non-empty");

    let arch = X86_64;
    let decoder = arch.decoder();
    let lifter = arch.lifter();

    let mut cov = Coverage::default();
    let mut decoder_invalid = 0u64;

    for decoded in decoder.iter(text, va) {
        // Track decoder invalids separately so we can distinguish a
        // bad fixture from a lifter coverage gap on failure.
        if matches!(decoded.flow, ControlFlow::Invalid) {
            decoder_invalid += 1;
        }
        let ir = lifter.lift(&decoded.bytes, decoded.address);
        cov.record(&ir);
    }

    let pct = cov.lifted_fraction() * 100.0;
    let floor_pct = floor * 100.0;
    assert!(
        cov.lifted_fraction() >= floor,
        "{fixture}: lifter coverage {pct:.1}% < {floor_pct:.1}% (lifted={lifted}, opaque={opaque}, total={total}, decoder_invalid={decoder_invalid})\n{cov}",
        lifted = cov.lifted,
        opaque = cov.opaque,
        total = cov.total,
    );
}

#[test]
fn elf_hello_meets_coverage_floor() {
    // The B1.4 done-when is "≥ 95% by instruction count" on the
    // sample corpus's `.text`.
    assert_corpus_coverage("hello-x86_64", 0.95);
}

#[test]
fn pe_hello_meets_coverage_floor() {
    assert_corpus_coverage("hello-x86_64.exe", 0.95);
}
