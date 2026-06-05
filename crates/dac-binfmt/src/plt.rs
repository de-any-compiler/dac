//! Procedure Linkage Table (PLT) trampoline walker for ELF x86-64
//! (B3.21, FR-N spec §11.1).
//!
//! On a dynamically-linked ELF binary, a call to an imported function
//! (e.g. `write`, `malloc`) is compiled as a direct call to a small
//! trampoline in the `.plt` / `.plt.sec` / `.plt.got` sections. The
//! trampoline's first instruction is `jmp qword ptr [rip + disp32]`,
//! reading a slot in `.got.plt` that the dynamic loader patches at
//! resolution time. The matching `R_X86_64_JUMP_SLOT` relocation in
//! `.rela.plt` says which import name belongs to which GOT slot.
//!
//! This walker stitches those two pieces together and returns a
//! `Vec<(plt_stub_va, import_name)>`. The orchestrator (in the CLI)
//! merges that into [`crate::Symbol`]-derived `va → ApiSignature`
//! map, so direct calls into a PLT stub bind to the import the same
//! way they already bind on PE through the IAT.
//!
//! ## Invariants
//!
//! - **I-1 (binary is ground truth).** The walker reads section
//!   bytes and the relocation table from [`BinaryModel`]; it never
//!   invents stubs that the binary does not contain.
//! - **I-2 (provenance).** Each returned `(stub_va, name)` is
//!   derivable from the JUMP_SLOT relocation that bound the GOT
//!   slot. The CLI threads the names into the recovery layer's
//!   `ApiContext` heuristic, which already records `NameSource`.
//! - **I-3 (confidence).** PLT-stub naming is `Observed` quality —
//!   the binary's relocation table is the authoritative source, no
//!   guesswork.
//! - **NFR-9 (deterministic).** Pure function of the
//!   [`BinaryModel`] and input bytes; no global state, no RNG.
//!
//! ## What is matched
//!
//! Every stub the walker recognises ends in `ff 25 disp32`
//! (`jmp qword ptr [rip + disp32]`). The same two-byte opcode is
//! used by:
//!
//! - Canonical `.plt` stubs (16 bytes each, second through last
//!   entries — the PLT0 header reads from `_GLOBAL_OFFSET_TABLE_+8`,
//!   which is not a JUMP_SLOT target and is therefore filtered).
//! - `-fcf-protection=branch` `.plt.sec` stubs which prepend
//!   `endbr64` (`f3 0f 1e fa`) before the `jmp` — recognised at the
//!   leading-endbr offset.
//! - `.plt.got` stubs (8 bytes each), used for symbols also reached
//!   from a `R_X86_64_GLOB_DAT` slot.
//!
//! Probing at 8-byte granularity covers every layout above; the
//! "must resolve to a JUMP_SLOT GOT VA" filter suppresses spurious
//! hits at mid-instruction boundaries.

use std::collections::BTreeMap;

use crate::model::{Architecture, BinaryFormat, BinaryModel, RelocationKind};

/// PLT-stub VA → import name pairs discovered for an ELF x86-64
/// binary. Empty for any other format / architecture and for
/// statically-linked binaries.
///
/// The result is sorted by `plt_stub_va` and deduplicated so the
/// caller can fold it into a `BTreeMap` without further work.
#[must_use]
pub fn elf_x86_64_plt_stubs(model: &BinaryModel, bytes: &[u8]) -> Vec<(u64, String)> {
    if model.format != BinaryFormat::Elf || model.architecture != Architecture::X86_64 {
        return Vec::new();
    }

    let got_to_import = build_jump_slot_index(model);
    if got_to_import.is_empty() {
        return Vec::new();
    }

    let mut stubs: Vec<(u64, String)> = Vec::new();
    for section in &model.sections {
        if !is_plt_section(&section.name) {
            continue;
        }
        if !section.perms.executable {
            continue;
        }
        let Some(file_offset) = section.file_offset else {
            continue;
        };
        let Ok(start) = usize::try_from(file_offset) else {
            continue;
        };
        let Ok(size) = usize::try_from(section.size) else {
            continue;
        };
        let end = start.saturating_add(size).min(bytes.len());
        let Some(data) = bytes.get(start..end) else {
            continue;
        };
        scan_plt_section(section.address, data, &got_to_import, &mut stubs);
    }

    stubs.sort_by_key(|(va, _)| *va);
    stubs.dedup_by_key(|&mut (va, _)| va);
    stubs
}

/// `R_X86_64_JUMP_SLOT` lives under [`RelocationKind::Glob`] in the
/// shared model. Pairs every JUMP_SLOT GOT VA with the symbol name
/// the dynamic loader binds into it.
fn build_jump_slot_index(model: &BinaryModel) -> BTreeMap<u64, String> {
    let mut out: BTreeMap<u64, String> = BTreeMap::new();
    for reloc in &model.relocations {
        if reloc.kind != RelocationKind::Glob {
            continue;
        }
        let Some(sym_idx) = reloc.symbol else {
            continue;
        };
        let Some(sym) = model.symbols.get(sym_idx) else {
            continue;
        };
        let name = strip_version_suffix(&sym.name);
        if name.is_empty() {
            continue;
        }
        out.entry(reloc.offset).or_insert_with(|| name.to_owned());
    }
    out
}

/// Strip an `@<version>` suffix from a symbolic-versioned dynamic
/// symbol name (e.g. `write@GLIBC_2.2.5` → `write`). The bare base
/// name is what `dac-knowledge`'s API catalogue keys on.
fn strip_version_suffix(name: &str) -> &str {
    match name.find('@') {
        Some(i) => &name[..i],
        None => name,
    }
}

/// Walk a single PLT section in 8-byte strides. At each stride
/// position, accept either:
///
/// - `ff 25 <disp32>` (canonical `jmp [rip+disp32]`, 6 bytes), or
/// - `f3 0f 1e fa ff 25 <disp32>` (endbr64 + jmp, 10 bytes).
///
/// Compute the GOT VA the jump targets; if it lives in the
/// JUMP_SLOT relocation index, record `(stub_va, import_name)`.
fn scan_plt_section(
    section_va: u64,
    data: &[u8],
    got_to_import: &BTreeMap<u64, String>,
    out: &mut Vec<(u64, String)>,
) {
    let mut offset: usize = 0;
    while offset + 6 <= data.len() {
        if let Some((stub_va, got_va)) = decode_jump_thunk(section_va, offset, data)
            .filter(|(_, g)| got_to_import.contains_key(g))
        {
            if let Some(name) = got_to_import.get(&got_va) {
                out.push((stub_va, name.clone()));
            }
        }
        offset += 8;
    }
}

/// Recognise an `ff 25 disp32` jump (optionally `endbr64`-prefixed)
/// at `data[offset..]` and return the `(stub_va, got_va)` pair.
///
/// Returns `None` for any other byte sequence, including mid-stub
/// strides that happen to land on unrelated opcodes.
fn decode_jump_thunk(section_va: u64, offset: usize, data: &[u8]) -> Option<(u64, u64)> {
    let payload = data.get(offset..)?;
    let (stub_offset_in_section, rip_relative_to_section, disp) =
        if payload.starts_with(&[0xff, 0x25]) {
            let disp = i32::from_le_bytes(payload.get(2..6)?.try_into().ok()?);
            (offset, offset.checked_add(6)?, disp)
        } else if payload.starts_with(&[0xf3, 0x0f, 0x1e, 0xfa, 0xff, 0x25]) {
            let disp = i32::from_le_bytes(payload.get(6..10)?.try_into().ok()?);
            (offset, offset.checked_add(10)?, disp)
        } else {
            return None;
        };
    let stub_va = section_va.checked_add(stub_offset_in_section as u64)?;
    let rip = section_va.checked_add(rip_relative_to_section as u64)?;
    let got_va = if disp >= 0 {
        rip.checked_add(disp as u64)?
    } else {
        rip.checked_sub(i64::from(disp).unsigned_abs())?
    };
    Some((stub_va, got_va))
}

/// PLT-bearing section names on ELF x86-64. The standard layout
/// uses `.plt` and (when CFI is enabled or symbols are referenced
/// from both PLT and GOT-indirect calls) `.plt.sec` / `.plt.got`.
fn is_plt_section(name: &str) -> bool {
    name == ".plt" || name == ".plt.sec" || name == ".plt.got"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Bits, Endian, Permissions, Relocation, Section, SectionKind, Symbol, SymbolBinding,
        SymbolKind, SymbolSource,
    };

    fn make_section(name: &str, address: u64, size: u64, file_offset: u64) -> Section {
        Section {
            name: name.to_owned(),
            address,
            size,
            file_offset: Some(file_offset),
            perms: Permissions {
                readable: true,
                writable: false,
                executable: true,
            },
            kind: SectionKind::Text,
        }
    }

    fn make_symbol(name: &str) -> Symbol {
        Symbol {
            name: name.to_owned(),
            address: 0,
            size: 0,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: None,
            source: SymbolSource::Dynsym,
            undefined: true,
        }
    }

    fn make_jump_slot(got_va: u64, symbol_idx: usize) -> Relocation {
        Relocation {
            section: None,
            offset: got_va,
            kind: RelocationKind::Glob,
            symbol: Some(symbol_idx),
            addend: 0,
        }
    }

    fn empty_model(format: BinaryFormat, arch: Architecture) -> BinaryModel {
        BinaryModel {
            format,
            architecture: arch,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: Vec::new(),
            segments: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
            needed_libraries: Vec::new(),
        }
    }

    /// Encode `jmp qword ptr [rip + disp32]` (6 bytes) where the
    /// effective address resolves to `got_va` when the instruction
    /// is decoded at virtual address `stub_va`.
    fn encode_jmp_indirect(stub_va: u64, got_va: u64) -> [u8; 6] {
        let rip = stub_va + 6;
        let disp = (got_va as i64 - rip as i64) as i32;
        let mut bytes = [0u8; 6];
        bytes[0] = 0xff;
        bytes[1] = 0x25;
        bytes[2..6].copy_from_slice(&disp.to_le_bytes());
        bytes
    }

    /// Same as [`encode_jmp_indirect`] but prepended with the
    /// 4-byte `endbr64` opcode — the `.plt.sec` layout introduced
    /// for `-fcf-protection=branch`. Total 10 bytes.
    fn encode_endbr_jmp_indirect(stub_va: u64, got_va: u64) -> [u8; 10] {
        let mut bytes = [0u8; 10];
        bytes[0..4].copy_from_slice(&[0xf3, 0x0f, 0x1e, 0xfa]);
        let rip = stub_va + 10;
        let disp = (got_va as i64 - rip as i64) as i32;
        bytes[4] = 0xff;
        bytes[5] = 0x25;
        bytes[6..10].copy_from_slice(&disp.to_le_bytes());
        bytes
    }

    /// Synthesise a 16-byte PLT stub (jmp-indirect + 5-byte `push` +
    /// 5-byte tail `jmp`), mirroring the canonical layout emitted by
    /// GNU ld for entries beyond PLT[0].
    fn canonical_plt_stub(stub_va: u64, got_va: u64) -> [u8; 16] {
        let mut out = [0x90u8; 16];
        out[0..6].copy_from_slice(&encode_jmp_indirect(stub_va, got_va));
        out[6..11].copy_from_slice(&[0x68, 0x00, 0x00, 0x00, 0x00]); // push imm32
        out[11..16].copy_from_slice(&[0xe9, 0xe0, 0xff, 0xff, 0xff]); // jmp PLT[0]
        out
    }

    /// Synthesise an 8-byte `.plt.got` stub: just `jmp-indirect` +
    /// 2 bytes of `nop` padding so each stub sits on an 8-byte
    /// boundary.
    fn plt_got_stub(stub_va: u64, got_va: u64) -> [u8; 8] {
        let mut out = [0x90u8; 8];
        out[0..6].copy_from_slice(&encode_jmp_indirect(stub_va, got_va));
        out
    }

    #[test]
    fn empty_model_returns_no_stubs() {
        let model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        assert!(elf_x86_64_plt_stubs(&model, &[]).is_empty());
    }

    #[test]
    fn non_elf_returns_no_stubs() {
        let model = empty_model(BinaryFormat::Pe, Architecture::X86_64);
        assert!(elf_x86_64_plt_stubs(&model, &[]).is_empty());
    }

    #[test]
    fn non_x86_64_returns_no_stubs() {
        let model = empty_model(BinaryFormat::Elf, Architecture::Aarch64);
        assert!(elf_x86_64_plt_stubs(&model, &[]).is_empty());
    }

    #[test]
    fn binary_without_jump_slots_yields_no_stubs() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        // .plt exists but no JUMP_SLOT relocations.
        let mut bytes = vec![0u8; 0x40];
        let stub = canonical_plt_stub(0x1030, 0x4000);
        bytes[0x30..0x40].copy_from_slice(&stub);
        model.sections.push(make_section(".plt", 0x1000, 0x40, 0));
        assert!(elf_x86_64_plt_stubs(&model, &bytes).is_empty());
    }

    #[test]
    fn canonical_plt_stub_is_named_from_jump_slot() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write"));
        model.relocations.push(make_jump_slot(0x4000, 0));
        // Section layout: PLT[0] header at 0x1020, write@plt at 0x1030.
        let mut bytes = vec![0u8; 0x40];
        let plt0 = canonical_plt_stub(0x1020, 0x3ff8); // PLT[0] reads GOT[0x10], not a JUMP_SLOT
        let write_plt = canonical_plt_stub(0x1030, 0x4000);
        bytes[0x20..0x30].copy_from_slice(&plt0);
        bytes[0x30..0x40].copy_from_slice(&write_plt);
        model
            .sections
            .push(make_section(".plt", 0x1020, 0x20, 0x20));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs, vec![(0x1030, "write".to_owned())]);
    }

    #[test]
    fn endbr_prefixed_plt_sec_stub_is_recognised() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("malloc"));
        model.relocations.push(make_jump_slot(0x4008, 0));
        let stub = encode_endbr_jmp_indirect(0x1040, 0x4008);
        let mut bytes = vec![0x90u8; 0x60];
        bytes[0x40..0x4a].copy_from_slice(&stub);
        model
            .sections
            .push(make_section(".plt.sec", 0x1000, 0x50, 0));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs, vec![(0x1040, "malloc".to_owned())]);
    }

    #[test]
    fn plt_got_eight_byte_stubs_are_recognised() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("free"));
        model.symbols.push(make_symbol("memcpy"));
        model.relocations.push(make_jump_slot(0x5000, 0));
        model.relocations.push(make_jump_slot(0x5008, 1));
        let mut bytes = vec![0x90u8; 0x40];
        // Two .plt.got stubs at 0x1000 and 0x1008.
        bytes[0x00..0x08].copy_from_slice(&plt_got_stub(0x1000, 0x5000));
        bytes[0x08..0x10].copy_from_slice(&plt_got_stub(0x1008, 0x5008));
        model
            .sections
            .push(make_section(".plt.got", 0x1000, 0x10, 0));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(
            stubs,
            vec![(0x1000, "free".to_owned()), (0x1008, "memcpy".to_owned())]
        );
    }

    #[test]
    fn versioned_symbol_name_is_stripped() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write@GLIBC_2.2.5"));
        model.relocations.push(make_jump_slot(0x4000, 0));
        let mut bytes = vec![0u8; 0x20];
        bytes[0x10..0x16].copy_from_slice(&encode_jmp_indirect(0x1010, 0x4000));
        model
            .sections
            .push(make_section(".plt.sec", 0x1000, 0x20, 0));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs, vec![(0x1010, "write".to_owned())]);
    }

    #[test]
    fn non_executable_section_named_dot_plt_is_ignored() {
        // A linker that stripped the X bit (or a tampered binary)
        // should not be treated as a PLT region.
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write"));
        model.relocations.push(make_jump_slot(0x4000, 0));
        let mut bytes = vec![0u8; 0x20];
        bytes[0x10..0x16].copy_from_slice(&encode_jmp_indirect(0x1010, 0x4000));
        let mut section = make_section(".plt", 0x1000, 0x20, 0);
        section.perms.executable = false;
        model.sections.push(section);
        assert!(elf_x86_64_plt_stubs(&model, &bytes).is_empty());
    }

    #[test]
    fn jump_to_unknown_got_va_is_filtered() {
        // A `ff 25` sequence whose target VA does not match any
        // JUMP_SLOT relocation must not yield a stub. This is the
        // safeguard against mid-instruction matches and against
        // GLOB_DAT slots that are not call-site trampolines.
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write"));
        model.relocations.push(make_jump_slot(0x4000, 0));
        let mut bytes = vec![0u8; 0x20];
        // Stub jumps to 0x9999 — not in the JUMP_SLOT table.
        bytes[0x10..0x16].copy_from_slice(&encode_jmp_indirect(0x1010, 0x9999));
        model.sections.push(make_section(".plt", 0x1000, 0x20, 0));

        assert!(elf_x86_64_plt_stubs(&model, &bytes).is_empty());
    }

    #[test]
    fn negative_displacement_resolves_correctly() {
        // Some toolchains place `.got` ahead of `.plt`, producing
        // negative disp32 values. The walker must handle the
        // subtractive case without panicking.
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write"));
        model.relocations.push(make_jump_slot(0x0500, 0));
        let mut bytes = vec![0u8; 0x20];
        bytes[0x10..0x16].copy_from_slice(&encode_jmp_indirect(0x1010, 0x0500));
        model.sections.push(make_section(".plt", 0x1000, 0x20, 0));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs, vec![(0x1010, "write".to_owned())]);
    }

    #[test]
    fn results_are_sorted_and_deduplicated() {
        let mut model = empty_model(BinaryFormat::Elf, Architecture::X86_64);
        model.symbols.push(make_symbol("write"));
        model.relocations.push(make_jump_slot(0x4000, 0));
        // Place the same stub bytes in both .plt and .plt.sec; the
        // dedup pass must keep one entry per stub VA.
        let stub = encode_jmp_indirect(0x1010, 0x4000);
        let mut bytes = vec![0u8; 0x40];
        bytes[0x10..0x16].copy_from_slice(&stub);
        bytes[0x30..0x36].copy_from_slice(&stub);
        model.sections.push(make_section(".plt", 0x1000, 0x20, 0));
        // Second section duplicates the same VA so dedup is exercised.
        model
            .sections
            .push(make_section(".plt.sec", 0x1000, 0x20, 0));

        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs.len(), 1);
        assert_eq!(stubs[0], (0x1010, "write".to_owned()));
    }

    /// End-to-end regression: the canonical hello-world ELF fixture
    /// must surface its single PLT stub (`0x1030 → write`). Catches
    /// future regressions to either the walker or the
    /// `bridge::relocation_symbol` static/dynamic-table split (B3.21).
    #[test]
    fn real_hello_elf_binds_write_plt_stub() {
        let bytes = std::fs::read("../../tests/fixtures/hello-x86_64")
            .expect("hello-x86_64 fixture is required by the test suite");
        let model = crate::load_from_bytes(&bytes).unwrap();
        let stubs = elf_x86_64_plt_stubs(&model, &bytes);
        assert_eq!(stubs, vec![(0x1030, "write".to_owned())]);
    }
}
