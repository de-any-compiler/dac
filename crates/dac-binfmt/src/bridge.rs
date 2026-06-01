//! Format-neutral bridge from `object`'s trait API into [`BinaryModel`].
//!
//! ELF and PE differ only in the variants of [`SectionFlags`], [`SegmentFlags`],
//! and [`RelocationFlags`] their parsers populate — every other piece of
//! information dac needs at this layer already flows through the
//! `Object` / `ObjectSection` / `ObjectSegment` / `ObjectSymbol` traits.
//! Putting the generic walk here keeps `elf.rs` and `pe.rs` to a handful of
//! lines each (just the format tag) and ensures both formats stay in lock-step
//! when a new field lands on `BinaryModel`.

use std::collections::HashMap;

use dac_core::{Error, Result};
use object::{
    Object, ObjectSection, ObjectSegment, ObjectSymbol, RelocationFlags, RelocationTarget,
    SectionFlags, SectionIndex, SegmentFlags, SymbolSection,
};

use crate::model::{
    Architecture, BinaryFormat, BinaryModel, Bits, Endian, Export, Import, Permissions, Relocation,
    RelocationKind, Section, SectionKind, Segment, StringRef, Symbol, SymbolBinding, SymbolKind,
    SymbolSource,
};

/// Minimum number of consecutive printable ASCII bytes to treat as a
/// string. Tuned to suppress noise from short opcodes that happen to be
/// printable. Matches the GNU `strings(1)` default.
const MIN_STRING_LEN: usize = 4;

/// Parse `bytes` through `object::File::parse` and project the result into
/// dac's format-agnostic [`BinaryModel`].
///
/// `format` and `format_tag` are carried through so that downstream
/// invariants (which format the model belongs to, what string to use in
/// error messages) stay stable even as `object` evolves.
pub(crate) fn parse_object(
    bytes: &[u8],
    format: BinaryFormat,
    format_tag: &'static str,
) -> Result<BinaryModel> {
    let obj = object::File::parse(bytes).map_err(|e| malformed(format_tag, "invalid header", e))?;

    let architecture = map_architecture(obj.architecture());
    let endian = if obj.is_little_endian() {
        Endian::Little
    } else {
        Endian::Big
    };
    let bits = if obj.is_64() {
        Bits::Bits64
    } else {
        Bits::Bits32
    };
    let entry = if obj.entry() == 0 {
        None
    } else {
        Some(obj.entry())
    };

    let mut sections = Vec::new();
    let mut section_lookup: HashMap<SectionIndex, usize> = HashMap::new();
    for raw in obj.sections() {
        let idx = sections.len();
        section_lookup.insert(raw.index(), idx);
        sections.push(read_section(&raw, format_tag)?);
    }

    let segments: Vec<Segment> = obj.segments().map(|s| read_segment(&s)).collect();

    let mut symbols = Vec::new();
    let mut static_symbol_lookup: HashMap<object::SymbolIndex, usize> = HashMap::new();
    for raw in obj.symbols() {
        let idx = symbols.len();
        static_symbol_lookup.insert(raw.index(), idx);
        symbols.push(read_symbol(&raw, &section_lookup, SymbolSource::Symtab));
    }
    let mut dynamic_symbol_lookup: HashMap<object::SymbolIndex, usize> = HashMap::new();
    for raw in obj.dynamic_symbols() {
        let idx = symbols.len();
        dynamic_symbol_lookup.insert(raw.index(), idx);
        symbols.push(read_symbol(&raw, &section_lookup, SymbolSource::Dynsym));
    }

    let imports: Vec<Import> = obj
        .imports()
        .map_err(|e| malformed(format_tag, "read imports", e))?
        .into_iter()
        .map(|i| Import {
            name: bytes_to_string(i.name()),
            library: Some(bytes_to_string(i.library())).filter(|s| !s.is_empty()),
        })
        .collect();

    let exports: Vec<Export> = obj
        .exports()
        .map_err(|e| malformed(format_tag, "read exports", e))?
        .into_iter()
        .map(|e| Export {
            name: bytes_to_string(e.name()),
            address: e.address(),
        })
        .collect();

    let mut relocations = Vec::new();
    // Static relocations live in `.rela.<section>` (ELF) or `.<section>$reloc`
    // attached COFF relocations (PE/COFF object files). They apply to the
    // owning section and are typical for `.o` files; linked executables
    // and shared libraries usually have none here.
    for raw_section in obj.sections() {
        let Some(&section_idx) = section_lookup.get(&raw_section.index()) else {
            continue;
        };
        for (offset, raw) in raw_section.relocations() {
            relocations.push(Relocation {
                section: Some(section_idx),
                offset,
                kind: map_relocation_flags(raw.flags(), architecture),
                symbol: relocation_symbol(
                    &raw.target(),
                    &static_symbol_lookup,
                    &dynamic_symbol_lookup,
                ),
                addend: raw.addend(),
            });
        }
    }
    // Dynamic relocations are ELF-only at the `object` trait level. For PE
    // images, base relocations live in `.reloc` and resolve image rebasing
    // rather than symbol bindings — they are deliberately not surfaced
    // here; the import table already covers the symbol-resolution view.
    if let Some(dyn_relocs) = obj.dynamic_relocations() {
        for (vaddr, raw) in dyn_relocs {
            relocations.push(Relocation {
                section: address_to_section(vaddr, &sections),
                offset: vaddr,
                kind: map_relocation_flags(raw.flags(), architecture),
                symbol: relocation_symbol(
                    &raw.target(),
                    &static_symbol_lookup,
                    &dynamic_symbol_lookup,
                ),
                addend: raw.addend(),
            });
        }
    }

    let strings = scan_strings(bytes, &sections);

    let needed_libraries = collect_needed_libraries(&imports);

    Ok(BinaryModel {
        format,
        architecture,
        endian,
        bits,
        entry,
        size: bytes.len(),
        sections,
        segments,
        symbols,
        imports,
        exports,
        relocations,
        strings,
        needed_libraries,
    })
}

fn read_section<'data, S>(raw: &S, format_tag: &'static str) -> Result<Section>
where
    S: ObjectSection<'data>,
{
    let name = raw
        .name()
        .map_err(|e| malformed(format_tag, "section name", e))?
        .to_owned();
    let (file_offset, file_size) = raw.file_range().unzip();
    // file_size is currently unused but kept for symmetry; suppress the
    // unused-binding warning without dropping the local.
    let _ = file_size;
    Ok(Section {
        name,
        address: raw.address(),
        size: raw.size(),
        file_offset,
        perms: section_permissions(raw.kind(), raw.flags()),
        kind: map_section_kind(raw.kind()),
    })
}

fn read_segment<'data, S>(raw: &S) -> Segment
where
    S: ObjectSegment<'data>,
{
    let (file_offset, file_size) = raw.file_range();
    Segment {
        name: raw.name().ok().flatten().map(str::to_owned),
        address: raw.address(),
        file_offset,
        file_size,
        mem_size: raw.size(),
        perms: segment_permissions(raw.flags()),
    }
}

fn read_symbol<'data, S>(
    raw: &S,
    section_lookup: &HashMap<SectionIndex, usize>,
    source: SymbolSource,
) -> Symbol
where
    S: ObjectSymbol<'data>,
{
    let name = raw.name().map(str::to_owned).unwrap_or_default();
    let section = match raw.section() {
        SymbolSection::Section(idx) => section_lookup.get(&idx).copied(),
        _ => None,
    };
    Symbol {
        name,
        address: raw.address(),
        size: raw.size(),
        kind: map_symbol_kind(raw.kind()),
        binding: map_symbol_binding(raw),
        section,
        source,
        undefined: raw.is_undefined(),
    }
}

fn address_to_section(addr: u64, sections: &[Section]) -> Option<usize> {
    sections.iter().position(|s| {
        s.address != 0 && addr >= s.address && addr < s.address.saturating_add(s.size)
    })
}

fn relocation_symbol(
    target: &RelocationTarget,
    static_symbols: &HashMap<object::SymbolIndex, usize>,
    dynamic_symbols: &HashMap<object::SymbolIndex, usize>,
) -> Option<usize> {
    match target {
        RelocationTarget::Symbol(idx) => static_symbols
            .get(idx)
            .copied()
            .or_else(|| dynamic_symbols.get(idx).copied()),
        _ => None,
    }
}

fn map_architecture(a: object::Architecture) -> Architecture {
    use object::Architecture as A;
    match a {
        A::I386 => Architecture::I386,
        A::X86_64 | A::X86_64_X32 => Architecture::X86_64,
        A::Arm => Architecture::Arm,
        A::Aarch64 | A::Aarch64_Ilp32 => Architecture::Aarch64,
        A::Riscv32 => Architecture::Riscv32,
        A::Riscv64 => Architecture::Riscv64,
        A::Mips => Architecture::Mips,
        A::Mips64 => Architecture::Mips64,
        A::PowerPc => Architecture::PowerPc,
        A::PowerPc64 => Architecture::PowerPc64,
        _ => Architecture::Unknown,
    }
}

fn map_section_kind(k: object::SectionKind) -> SectionKind {
    use object::SectionKind as K;
    match k {
        K::Text => SectionKind::Text,
        K::ReadOnlyData | K::ReadOnlyString | K::ReadOnlyDataWithRel => SectionKind::ReadOnlyData,
        K::Data => SectionKind::Data,
        K::UninitializedData => SectionKind::UninitializedData,
        K::Tls | K::TlsVariables | K::UninitializedTls => SectionKind::Tls,
        K::Debug | K::DebugString => SectionKind::Metadata,
        K::Note => SectionKind::Note,
        K::Metadata | K::Linker | K::Elf(_) | K::Common | K::OtherString | K::Other => {
            SectionKind::Other
        }
        K::Unknown => SectionKind::Unknown,
        _ => SectionKind::Unknown,
    }
}

fn section_permissions(kind: object::SectionKind, flags: SectionFlags) -> Permissions {
    match flags {
        SectionFlags::Elf { sh_flags } => {
            const SHF_WRITE: u64 = 0x1;
            const SHF_ALLOC: u64 = 0x2;
            const SHF_EXECINSTR: u64 = 0x4;
            Permissions {
                readable: sh_flags & SHF_ALLOC != 0,
                writable: sh_flags & SHF_WRITE != 0,
                executable: sh_flags & SHF_EXECINSTR != 0,
            }
        }
        SectionFlags::Coff { characteristics } => coff_perms(characteristics),
        _ => fallback_permissions(kind),
    }
}

fn segment_permissions(flags: SegmentFlags) -> Permissions {
    match flags {
        SegmentFlags::Elf { p_flags } => {
            const PF_X: u32 = 0x1;
            const PF_W: u32 = 0x2;
            const PF_R: u32 = 0x4;
            Permissions {
                readable: p_flags & PF_R != 0,
                writable: p_flags & PF_W != 0,
                executable: p_flags & PF_X != 0,
            }
        }
        SegmentFlags::Coff { characteristics } => coff_perms(characteristics),
        _ => Permissions::default(),
    }
}

fn coff_perms(characteristics: u32) -> Permissions {
    // PE/COFF section characteristics — `IMAGE_SCN_MEM_*`.
    const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
    const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
    const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;
    Permissions {
        readable: characteristics & IMAGE_SCN_MEM_READ != 0,
        writable: characteristics & IMAGE_SCN_MEM_WRITE != 0,
        executable: characteristics & IMAGE_SCN_MEM_EXECUTE != 0,
    }
}

fn fallback_permissions(kind: object::SectionKind) -> Permissions {
    Permissions {
        readable: matches!(
            kind,
            object::SectionKind::Text
                | object::SectionKind::Data
                | object::SectionKind::ReadOnlyData
                | object::SectionKind::ReadOnlyString
                | object::SectionKind::UninitializedData
                | object::SectionKind::Tls
        ),
        writable: matches!(
            kind,
            object::SectionKind::Data
                | object::SectionKind::UninitializedData
                | object::SectionKind::Tls
                | object::SectionKind::UninitializedTls
        ),
        executable: matches!(kind, object::SectionKind::Text),
    }
}

fn map_symbol_kind(k: object::SymbolKind) -> SymbolKind {
    use object::SymbolKind as K;
    match k {
        K::Text => SymbolKind::Text,
        K::Data => SymbolKind::Data,
        K::Section => SymbolKind::Section,
        K::File => SymbolKind::File,
        K::Tls => SymbolKind::Tls,
        K::Label => SymbolKind::Label,
        _ => SymbolKind::Unknown,
    }
}

fn map_symbol_binding<'data, S>(raw: &S) -> SymbolBinding
where
    S: ObjectSymbol<'data>,
{
    if raw.is_weak() {
        return SymbolBinding::Weak;
    }
    if raw.is_global() {
        return SymbolBinding::Global;
    }
    SymbolBinding::Local
}

/// COFF relocation type spaces overlap across architectures (each starts
/// at `0x0000`), so we need the architecture to decide which constant
/// table to consult. ELF embeds the architecture into the `r_type`
/// numeric layout, so the ELF arm is arch-agnostic.
fn map_relocation_flags(flags: RelocationFlags, arch: Architecture) -> RelocationKind {
    match flags {
        RelocationFlags::Elf { r_type } => map_elf_reloc(r_type),
        RelocationFlags::Coff { typ } => match arch {
            Architecture::X86_64 => map_coff_amd64_reloc(typ),
            Architecture::I386 => map_coff_i386_reloc(typ),
            Architecture::Aarch64 => map_coff_arm64_reloc(typ),
            _ => RelocationKind::Unknown,
        },
        _ => RelocationKind::Unknown,
    }
}

fn map_elf_reloc(r_type: u32) -> RelocationKind {
    use object::elf;
    match r_type {
        elf::R_X86_64_RELATIVE | elf::R_AARCH64_RELATIVE => RelocationKind::Relative,
        elf::R_X86_64_GOTPCREL | elf::R_X86_64_GOTPCRELX | elf::R_X86_64_REX_GOTPCRELX => {
            RelocationKind::GotRelative
        }
        elf::R_X86_64_PLT32 => RelocationKind::PltRelative,
        elf::R_X86_64_GLOB_DAT | elf::R_X86_64_JUMP_SLOT | elf::R_AARCH64_GLOB_DAT => {
            RelocationKind::Glob
        }
        elf::R_X86_64_COPY => RelocationKind::Copy,
        elf::R_X86_64_DTPMOD64
        | elf::R_X86_64_DTPOFF64
        | elf::R_X86_64_TPOFF64
        | elf::R_X86_64_TLSGD
        | elf::R_X86_64_TLSLD
        | elf::R_X86_64_DTPOFF32
        | elf::R_X86_64_GOTTPOFF
        | elf::R_X86_64_TPOFF32 => RelocationKind::Tls,
        elf::R_X86_64_64
        | elf::R_X86_64_32
        | elf::R_X86_64_32S
        | elf::R_X86_64_16
        | elf::R_X86_64_8
        | elf::R_AARCH64_ABS64
        | elf::R_AARCH64_ABS32 => RelocationKind::Absolute,
        elf::R_X86_64_PC32 | elf::R_X86_64_PC64 => RelocationKind::Relative,
        _ => RelocationKind::Unknown,
    }
}

fn map_coff_amd64_reloc(typ: u16) -> RelocationKind {
    use object::pe;
    match typ {
        pe::IMAGE_REL_AMD64_ADDR64 | pe::IMAGE_REL_AMD64_ADDR32 | pe::IMAGE_REL_AMD64_ADDR32NB => {
            RelocationKind::Absolute
        }
        pe::IMAGE_REL_AMD64_REL32
        | pe::IMAGE_REL_AMD64_REL32_1
        | pe::IMAGE_REL_AMD64_REL32_2
        | pe::IMAGE_REL_AMD64_REL32_3
        | pe::IMAGE_REL_AMD64_REL32_4
        | pe::IMAGE_REL_AMD64_REL32_5 => RelocationKind::Relative,
        pe::IMAGE_REL_AMD64_SECTION | pe::IMAGE_REL_AMD64_SECREL | pe::IMAGE_REL_AMD64_SECREL7 => {
            RelocationKind::Section
        }
        _ => RelocationKind::Unknown,
    }
}

fn map_coff_i386_reloc(typ: u16) -> RelocationKind {
    use object::pe;
    match typ {
        pe::IMAGE_REL_I386_DIR32 | pe::IMAGE_REL_I386_DIR32NB | pe::IMAGE_REL_I386_DIR16 => {
            RelocationKind::Absolute
        }
        pe::IMAGE_REL_I386_REL32 | pe::IMAGE_REL_I386_REL16 => RelocationKind::Relative,
        pe::IMAGE_REL_I386_SECTION | pe::IMAGE_REL_I386_SECREL | pe::IMAGE_REL_I386_SECREL7 => {
            RelocationKind::Section
        }
        _ => RelocationKind::Unknown,
    }
}

fn map_coff_arm64_reloc(typ: u16) -> RelocationKind {
    use object::pe;
    match typ {
        pe::IMAGE_REL_ARM64_ADDR64 | pe::IMAGE_REL_ARM64_ADDR32 | pe::IMAGE_REL_ARM64_ADDR32NB => {
            RelocationKind::Absolute
        }
        pe::IMAGE_REL_ARM64_REL32
        | pe::IMAGE_REL_ARM64_BRANCH26
        | pe::IMAGE_REL_ARM64_BRANCH19
        | pe::IMAGE_REL_ARM64_BRANCH14 => RelocationKind::Relative,
        pe::IMAGE_REL_ARM64_SECTION | pe::IMAGE_REL_ARM64_SECREL => RelocationKind::Section,
        _ => RelocationKind::Unknown,
    }
}

/// Walk read-only-data sections looking for null-terminated printable
/// ASCII strings of at least [`MIN_STRING_LEN`] bytes. Each match becomes
/// a [`StringRef`] pointing at the originating section. The cost is
/// linear in the read-only-data byte count; we deliberately scan only
/// `ReadOnlyData` to avoid spurious matches from code or relocations.
fn scan_strings(bytes: &[u8], sections: &[Section]) -> Vec<StringRef> {
    let mut out = Vec::new();
    for (idx, section) in sections.iter().enumerate() {
        if section.kind != SectionKind::ReadOnlyData {
            continue;
        }
        let Some(start) = section.file_offset else {
            continue;
        };
        let end = start.saturating_add(section.size);
        let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
            continue;
        };
        let Some(data) = bytes.get(start..end.min(bytes.len())) else {
            continue;
        };
        scan_section_strings(data, idx, &mut out);
    }
    out
}

fn scan_section_strings(data: &[u8], section: usize, out: &mut Vec<StringRef>) {
    let mut run_start: Option<usize> = None;
    for (i, &b) in data.iter().enumerate() {
        let is_printable = (0x20..=0x7E).contains(&b);
        if is_printable {
            if run_start.is_none() {
                run_start = Some(i);
            }
        } else {
            if let Some(start) = run_start.take() {
                let run = &data[start..i];
                if b == 0 && run.len() >= MIN_STRING_LEN {
                    if let Ok(value) = std::str::from_utf8(run) {
                        out.push(StringRef {
                            section,
                            offset: start as u64,
                            value: value.to_owned(),
                        });
                    }
                }
            }
        }
    }
}

fn collect_needed_libraries(imports: &[Import]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for import in imports {
        if let Some(lib) = &import.library {
            if !lib.is_empty() && !seen.iter().any(|s| s == lib) {
                seen.push(lib.clone());
            }
        }
    }
    seen
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn malformed(format_tag: &'static str, reason: &str, e: object::Error) -> Error {
    Error::MalformedBinary {
        format: format_tag,
        reason: format!("{reason}: {e}"),
    }
}
