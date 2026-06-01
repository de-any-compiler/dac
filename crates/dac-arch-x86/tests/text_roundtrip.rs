//! Integration test: decode the entire `.text` section of real binary
//! fixtures end-to-end.
//!
//! Closes the B1.3 done-when ("decoder round-trips a real `.text`
//! section"). The strict invariants are:
//!
//! - Every byte of `.text` is consumed by the iterator.
//! - Decoded addresses are strictly monotonically increasing.
//! - At least one `ret` is observed.
//! - At least one `call` (direct or indirect) is observed.
//! - At least 95% of decoded instructions are valid — a linear sweep
//!   through compiler-generated x86-64 should be near-perfect; the 5%
//!   tolerance covers occasional padding / jump-table interleaving in
//!   stripped binaries.
//!
//! Both the ELF and PE fixtures used elsewhere in the workspace are
//! exercised, proving that the same iced-x86 decoder rides cleanly on
//! top of either binary parser.

use std::path::PathBuf;

use dac_arch::{Architecture, ControlFlow};
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

fn round_trip(fixture: &str) {
    let (bytes, model) = load(fixture);
    let (text, va) = text_bytes(&model, &bytes);
    assert!(!text.is_empty(), "{fixture}: .text non-empty");

    let arch = X86_64;
    let decoder = arch.decoder();

    let mut total_bytes = 0usize;
    let mut count = 0usize;
    let mut valid_count = 0usize;
    let mut prev_addr: Option<u64> = None;
    let mut saw_ret = false;
    let mut saw_call = false;

    for instr in decoder.iter(text, va) {
        if let Some(p) = prev_addr {
            assert!(
                instr.address > p,
                "{fixture}: addresses must increase ({p:#x} -> {:#x})",
                instr.address,
            );
        }
        prev_addr = Some(instr.address);
        total_bytes += instr.length;
        count += 1;
        if instr.valid {
            valid_count += 1;
        }
        match instr.flow {
            ControlFlow::Return => saw_ret = true,
            ControlFlow::Call { .. } | ControlFlow::IndirectCall => saw_call = true,
            _ => {}
        }
    }

    assert_eq!(
        total_bytes,
        text.len(),
        "{fixture}: .text fully consumed — decoded {total_bytes} of {} bytes ({count} insts)",
        text.len(),
    );
    assert!(
        count > 10,
        "{fixture}: expected ≥ 10 instructions, got {count}"
    );
    let validity = (valid_count as f64) / (count as f64);
    assert!(
        validity >= 0.95,
        "{fixture}: validity {:.1}% under 95% ({valid_count}/{count})",
        validity * 100.0,
    );
    assert!(saw_ret, "{fixture}: expected at least one ret");
    assert!(saw_call, "{fixture}: expected at least one call");
}

#[test]
fn elf_hello_text_round_trips() {
    round_trip("hello-x86_64");
}

#[test]
fn pe_hello_text_round_trips() {
    round_trip("hello-x86_64.exe");
}
