//! Format-agnostic binary model.
//!
//! The types here are the shared vocabulary every later layer uses to talk
//! about a loaded binary. They sit above the `object` crate, so callers
//! never need to think about whether a section came from an ELF program
//! header or a PE section table.

/// A binary executable format dac recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// ELF (Linux, BSD, embedded).
    Elf,
    /// Portable Executable (Windows).
    Pe,
    /// Mach-O (macOS, iOS).
    MachO,
}

impl BinaryFormat {
    /// Human-readable name suitable for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Elf => "ELF",
            Self::Pe => "PE",
            Self::MachO => "Mach-O",
        }
    }
}

/// Instruction-set architecture recognized at parse time. Decoder
/// support is independent and lives in `dac-arch` (B1.3+).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    Unknown,
    I386,
    X86_64,
    Arm,
    Aarch64,
    Riscv32,
    Riscv64,
    Mips,
    Mips64,
    PowerPc,
    PowerPc64,
}

impl Architecture {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::I386 => "i386",
            Self::X86_64 => "x86-64",
            Self::Arm => "arm",
            Self::Aarch64 => "aarch64",
            Self::Riscv32 => "riscv32",
            Self::Riscv64 => "riscv64",
            Self::Mips => "mips",
            Self::Mips64 => "mips64",
            Self::PowerPc => "powerpc",
            Self::PowerPc64 => "powerpc64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bits {
    Bits32,
    Bits64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Permissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Unknown,
    /// `.text` or any executable code section.
    Text,
    /// `.rodata` / read-only initialized data.
    ReadOnlyData,
    /// `.data` / writable initialized data.
    Data,
    /// `.bss` / writable zero-initialized data.
    UninitializedData,
    /// Thread-local data.
    Tls,
    /// Linker / debugger metadata (`.debug_*`, `.symtab`, …).
    Metadata,
    /// Notes (`.note.*`).
    Note,
    /// Other / unrecognized.
    Other,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub address: u64,
    pub size: u64,
    /// Offset into the input bytes. `None` for `bss`-like sections that
    /// occupy memory but no file space.
    pub file_offset: Option<u64>,
    pub perms: Permissions,
    pub kind: SectionKind,
}

#[derive(Debug)]
pub struct Segment {
    /// `PT_*` name (`"LOAD"`, `"DYNAMIC"`, …) when the parser supplies one,
    /// or `None` for formats without a textual name.
    pub name: Option<String>,
    pub address: u64,
    pub file_offset: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub perms: Permissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Unknown,
    /// Code symbol (function, label).
    Text,
    /// Data symbol (object).
    Data,
    /// Section symbol.
    Section,
    /// File symbol.
    File,
    /// Thread-local symbol.
    Tls,
    /// Label without size (e.g. assembly label).
    Label,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    Unique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolSource {
    /// From the static `.symtab` (only present when not stripped).
    Symtab,
    /// From the dynamic symbol table (`.dynsym`).
    Dynsym,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub kind: SymbolKind,
    pub binding: SymbolBinding,
    /// Index into `BinaryModel::sections` if the symbol is bound to a
    /// section in this binary; `None` for undefined symbols.
    pub section: Option<usize>,
    pub source: SymbolSource,
    /// `true` when this symbol references an external definition (an
    /// import).
    pub undefined: bool,
}

#[derive(Debug)]
pub struct Import {
    pub name: String,
    /// Library hint (DT_NEEDED soname for ELF, DLL name for PE, install
    /// name for Mach-O). `None` if the format does not provide one.
    pub library: Option<String>,
}

#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub address: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationKind {
    Unknown,
    Absolute,
    Relative,
    GotRelative,
    PltRelative,
    /// Imported function address resolved through the PLT.
    Glob,
    /// Address-copy relocation.
    Copy,
    /// Thread-local storage relocation.
    Tls,
    /// Section-relative or other format-specific.
    Section,
}

#[derive(Debug)]
pub struct Relocation {
    /// Index into `BinaryModel::sections` for the section being patched,
    /// when knowable. `None` for dynamic relocations whose target virtual
    /// address falls outside every section we recorded.
    pub section: Option<usize>,
    /// For static (`.o`) relocations this is the byte offset within
    /// `section`. For dynamic relocations (executables, shared libraries)
    /// this is the virtual address being patched.
    pub offset: u64,
    pub kind: RelocationKind,
    /// Index into `BinaryModel::symbols` if the relocation references a
    /// symbol. `None` for relocations that compute purely from the load
    /// address (e.g. `R_X86_64_RELATIVE`).
    pub symbol: Option<usize>,
    pub addend: i64,
}

#[derive(Debug)]
pub struct StringRef {
    /// Index into `BinaryModel::sections`.
    pub section: usize,
    /// Offset, in bytes, within the section.
    pub offset: u64,
    pub value: String,
}

/// Format-agnostic view of a loaded binary.
///
/// This is the substrate every later layer reads. Field ordering follows
/// the conceptual flow of analysis (format → arch → layout → symbol info →
/// extracted strings), so debug-formatting a `BinaryModel` is a halfway
/// readable summary.
#[derive(Debug)]
pub struct BinaryModel {
    pub format: BinaryFormat,
    pub architecture: Architecture,
    pub endian: Endian,
    pub bits: Bits,
    pub entry: Option<u64>,
    pub size: usize,
    pub sections: Vec<Section>,
    pub segments: Vec<Segment>,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub relocations: Vec<Relocation>,
    pub strings: Vec<StringRef>,
    /// DT_NEEDED (ELF), DLL import names (PE), `LC_LOAD_DYLIB` (Mach-O).
    pub needed_libraries: Vec<String>,
}
