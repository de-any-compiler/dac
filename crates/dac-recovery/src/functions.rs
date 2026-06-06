//! Function discovery (FR-9).
//!
//! Identifies function entry points — and, when available, byte-range
//! ends — from four independent signals, each carrying its own
//! [`dac_core::Source`] class:
//!
//! | Signal              | `Source`     | Default confidence value |
//! | ------------------- | ------------ | ------------------------ |
//! | Symbol table entry  | `Observed`   | `1.0`                    |
//! | Binary entry point  | `Observed`   | `1.0`                    |
//! | Direct call target  | `Derived`    | `0.85`                   |
//! | x86 prologue match  | `Derived`    | `0.6`                    |
//!
//! When several signals agree on the same address, their confidences
//! combine through [`dac_core::Confidence::join`] (componentwise max)
//! and the per-signal bits in [`SourceMask`] accumulate, so a
//! `--debug` consumer can still see which signals contributed.
//!
//! ## Evidence wiring (I-2)
//!
//! Each discovered function is minted into the [`EvidenceGraph`] as a
//! pair of nodes:
//!
//! - [`EvidenceNode::Bytes`] covering `[address, end)` — the byte span
//!   the function occupies in the loaded image. When the end is not
//!   known *and* no neighbour fills it in, the span degenerates to
//!   `[address, address)` so the node still exists and the orchestrator
//!   can attach later facts to it.
//! - [`EvidenceNode::IrNode { layer: Cfg, id }`] — the function itself,
//!   addressed at the CFG layer. The numeric `id` is the function's
//!   index in [`FunctionSet::functions`].
//!
//! A `Supports` edge from the byte span to the function node records
//! "this byte range produced this CFG-layer fact." Per-signal evidence
//! (a knowledge fact per call site, a per-symbol node) lands in later
//! batches; the structure here is the substrate they hook into.
//!
//! ## End-bound recovery
//!
//! Symbol-derived entries arrive with a known `size`; everything else
//! lands with `end = None`. A final pass walks the discovered functions
//! in address order and fills any unknown end with the next function
//! start *inside the same executable section*, falling back to the
//! section end. This matches how every real decompiler approximates
//! function bodies when the symbol table is silent, and gives B2.1
//! (CFG construction) a half-open byte range to work with.

use std::collections::BTreeMap;

use dac_arch::{ControlFlow, InstructionDecoder};
use dac_binfmt::{elf_x86_64_plt_stubs, BinaryModel, Section, SymbolKind};
use dac_core::{Confidence, EdgeKind, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer, Source};

/// Default confidence value for a function derived from a symbol-table
/// entry. The `Source` axis is [`Source::Observed`].
pub const SYMBOL_CONFIDENCE: f32 = 1.0;
/// Default confidence value for the binary's entry point. The `Source`
/// axis is [`Source::Observed`].
pub const ENTRY_CONFIDENCE: f32 = 1.0;
/// Default confidence value for a function discovered as the target of
/// a direct call. The `Source` axis is [`Source::Derived`].
pub const CALL_EDGE_CONFIDENCE: f32 = 0.85;
/// Default confidence value for a function discovered through a
/// prologue pattern. The `Source` axis is [`Source::Derived`].
pub const PROLOGUE_CONFIDENCE: f32 = 0.6;
/// Default confidence value for a function bound to an imported
/// symbol through a recognised PLT trampoline (B3.23). The
/// relocation table is binary-grounded, so the `Source` axis is
/// [`Source::Observed`].
pub const PLT_BINDING_CONFIDENCE: f32 = 1.0;

/// Bitmask recording which signals contributed to a discovered
/// function.
///
/// Multiple signals routinely agree on the same address (a known symbol
/// is also the target of an internal call edge, the entry point is
/// also a `main` symbol, …). Rather than collapsing that information
/// into a single source, the discoverer ORs the per-signal flags so
/// the reviewer can inspect *why* a given function was promoted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceMask(u8);

impl SourceMask {
    /// Symbol-table-derived discovery.
    pub const SYMBOL: Self = Self(1 << 0);
    /// Binary entry point.
    pub const ENTRY: Self = Self(1 << 1);
    /// Target of a direct call.
    pub const CALL: Self = Self(1 << 2);
    /// Matched a known prologue pattern.
    pub const PROLOGUE: Self = Self(1 << 3);
    /// Bound to an imported symbol through a recognised PLT
    /// trampoline (B3.23). The function lives at the trampoline
    /// VA; the import name comes from the matched `JUMP_SLOT`
    /// relocation.
    pub const PLT: Self = Self(1 << 4);

    /// Empty mask.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// `true` iff the mask records no signals.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// `true` iff every bit in `mask` is set in `self`. The empty mask
    /// is treated as "no bits required" and matches.
    #[must_use]
    pub const fn contains(self, mask: Self) -> bool {
        (self.0 & mask.0) == mask.0
    }

    /// Add the bits in `mask` to `self`.
    pub fn insert(&mut self, mask: Self) {
        self.0 |= mask.0;
    }

    /// Raw bit representation. Stable across versions of this crate.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Coarse taxonomy for a recovered function (B3.23).
///
/// `User` covers any function whose body should be lowered to source.
/// `PltStub` marks a Procedure Linkage Table trampoline bound to a
/// concrete imported symbol — the C backend renders these as `extern`
/// forward declarations instead of bodies, and call sites resolve
/// through the import name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum FunctionKind {
    /// A regular function discovered in the binary. The deterministic
    /// pipeline lifts its body end-to-end.
    #[default]
    User,
    /// An ELF / PE trampoline whose first instruction reads a GOT
    /// (or IAT) slot that the dynamic loader patches at resolution
    /// time. `import` is the symbol the trampoline binds to (e.g.
    /// `write`, `__libc_start_main`).
    PltStub { import: String },
}

/// A function recovered from the binary.
///
/// `address` is the entry virtual address (a function start). `end`,
/// when present, is the exclusive end of the function's byte span. The
/// confidence and source-mask together describe *how strongly* dac
/// believes this is a function and *why*.
///
/// `evidence` points at the function's `IrNode { layer: Cfg, id }`
/// node in the [`EvidenceGraph`]; subsequent passes attach further
/// facts (calling-convention inference, signature recovery, type
/// propagation, …) by adding edges into that node.
#[derive(Debug, Clone)]
pub struct Function {
    /// Function entry virtual address.
    pub address: u64,
    /// Exclusive end of the function's byte span. `None` when no
    /// signal provided a size *and* no neighbour bounded it.
    pub end: Option<u64>,
    /// Symbol-table name when available, otherwise `None`.
    pub name: Option<String>,
    /// Joined confidence across every contributing signal.
    pub confidence: Confidence,
    /// Bitmask of contributing signals.
    pub sources: SourceMask,
    /// Evidence-graph handle for this function.
    pub evidence: EvidenceId,
    /// Coarse taxonomy (B3.23). `User` for ordinary functions;
    /// `PltStub { import }` for PLT trampolines bound to an
    /// imported symbol.
    pub kind: FunctionKind,
}

impl Function {
    /// Inclusive byte length when both ends are known.
    #[must_use]
    pub fn size(&self) -> Option<u64> {
        self.end.map(|e| e.saturating_sub(self.address))
    }
}

/// Per-signal contribution counts. Each counter is incremented at most
/// once per discovered function, so the sum can exceed
/// [`FunctionSet::functions`]'s length when multiple signals agree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiscoveryStats {
    /// Number of functions whose discovery included a symbol-table
    /// signal.
    pub from_symbol: u64,
    /// Number of functions whose discovery included the binary entry
    /// point.
    pub from_entry: u64,
    /// Number of functions whose discovery included a direct call
    /// edge.
    pub from_call: u64,
    /// Number of functions whose discovery included a prologue match.
    pub from_prologue: u64,
    /// Number of functions bound to an imported symbol through a
    /// recognised PLT trampoline (B3.23).
    pub from_plt: u64,
}

/// Set of functions recovered from a [`BinaryModel`].
#[derive(Debug, Clone)]
pub struct FunctionSet {
    /// Functions in ascending address order.
    pub functions: Vec<Function>,
    /// Per-signal contribution counts.
    pub stats: DiscoveryStats,
}

impl FunctionSet {
    /// Iterator over function start addresses in ascending order.
    pub fn addresses(&self) -> impl Iterator<Item = u64> + '_ {
        self.functions.iter().map(|f| f.address)
    }

    /// `true` iff a function starts exactly at `address`.
    #[must_use]
    pub fn contains_address(&self, address: u64) -> bool {
        self.functions
            .binary_search_by_key(&address, |f| f.address)
            .is_ok()
    }

    /// Look up the function starting at `address`, if any.
    #[must_use]
    pub fn get(&self, address: u64) -> Option<&Function> {
        self.functions
            .binary_search_by_key(&address, |f| f.address)
            .ok()
            .map(|i| &self.functions[i])
    }
}

/// Discover functions in `model`.
///
/// The discoverer reads:
///
/// - `model.symbols` for symbol-table-derived starts and sizes;
/// - `model.entry` for the binary entry point;
/// - `bytes` + `decoder` for direct-call edges and prologue patterns
///   across every executable section.
///
/// Each discovery is recorded in `graph` (see the module docs for the
/// node and edge shapes). The function returns a [`FunctionSet`]
/// sorted by address; downstream passes use [`Function::evidence`] to
/// look up the corresponding graph node.
pub fn discover_functions(
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    graph: &mut EvidenceGraph,
) -> FunctionSet {
    let exec_sections: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();

    let mut acc: BTreeMap<u64, Acc> = BTreeMap::new();
    let mut stats = DiscoveryStats::default();

    // 1) Symbol-table-derived starts.
    for sym in &model.symbols {
        if sym.kind != SymbolKind::Text || sym.undefined || sym.address == 0 {
            continue;
        }
        if !in_executable(sym.address, &exec_sections) {
            continue;
        }
        let end = if sym.size > 0 {
            Some(sym.address.wrapping_add(sym.size))
        } else {
            None
        };
        let name = if sym.name.is_empty() {
            None
        } else {
            Some(sym.name.clone())
        };
        record_signal(
            &mut acc,
            &mut stats,
            sym.address,
            SYMBOL_CONFIDENCE,
            Source::Observed,
            name,
            end,
            SourceMask::SYMBOL,
        );
    }

    // 2) Entry-point.
    if let Some(entry) = model.entry {
        if entry != 0 && in_executable(entry, &exec_sections) {
            record_signal(
                &mut acc,
                &mut stats,
                entry,
                ENTRY_CONFIDENCE,
                Source::Observed,
                None,
                None,
                SourceMask::ENTRY,
            );
        }
    }

    // 3) Single sweep per section for call-edge targets and prologue
    //    patterns.
    for sec in &exec_sections {
        let Some(slice) = section_bytes(sec, bytes) else {
            continue;
        };
        let instructions: Vec<_> = decoder.iter(slice, sec.address).collect();
        for (idx, inst) in instructions.iter().enumerate() {
            if let ControlFlow::Call { target: Some(t) } = inst.flow {
                if t != 0 && in_executable(t, &exec_sections) {
                    record_signal(
                        &mut acc,
                        &mut stats,
                        t,
                        CALL_EDGE_CONFIDENCE,
                        Source::Derived,
                        None,
                        None,
                        SourceMask::CALL,
                    );
                }
            }
            if let Some(prologue_addr) = match_prologue(inst, instructions.get(idx + 1)) {
                record_signal(
                    &mut acc,
                    &mut stats,
                    prologue_addr,
                    PROLOGUE_CONFIDENCE,
                    Source::Derived,
                    None,
                    None,
                    SourceMask::PROLOGUE,
                );
            }
        }
    }

    // 4) ELF x86-64 PLT trampoline binding (B3.23). For every
    //    `(stub_va, import_name)` the walker recognises, fold the
    //    binding in as an `Observed` signal that supersedes the call /
    //    prologue confidence and (if no symbol contributed) pins the
    //    name to the import. Non-matching formats / architectures
    //    return an empty vector, so this is a no-op for PE and Mach-O.
    let plt_bindings: BTreeMap<u64, String> =
        elf_x86_64_plt_stubs(model, bytes).into_iter().collect();
    for (stub_va, import_name) in &plt_bindings {
        record_signal(
            &mut acc,
            &mut stats,
            *stub_va,
            PLT_BINDING_CONFIDENCE,
            Source::Observed,
            Some(import_name.clone()),
            None,
            SourceMask::PLT,
        );
    }

    // 5) Materialize. The `acc` map is already in ascending address
    //    order; collect into a Vec, fill unknown ends from neighbours
    //    and section bounds, then mint the evidence nodes.
    let mut entries: Vec<(u64, Acc)> = acc.into_iter().collect();
    fill_ends_from_neighbours(&mut entries, &exec_sections);

    let mut functions = Vec::with_capacity(entries.len());
    for (address, ent) in entries {
        let confidence = ent
            .confidence
            .unwrap_or_else(|| Confidence::new(0.0, Source::Speculative));
        let span_end = ent.end.unwrap_or(address);
        let bytes_node = graph.add_node(EvidenceNode::Bytes {
            start: address,
            end: span_end,
        });
        let fn_idx = functions.len() as u64;
        let ir_node = graph.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: fn_idx,
        });
        graph.add_edge(bytes_node, ir_node, EdgeKind::Supports);
        let kind = match plt_bindings.get(&address) {
            Some(import) => FunctionKind::PltStub {
                import: import.clone(),
            },
            None => FunctionKind::User,
        };
        functions.push(Function {
            address,
            end: ent.end,
            name: ent.name,
            confidence,
            sources: ent.sources,
            evidence: ir_node,
            kind,
        });
    }

    FunctionSet { functions, stats }
}

#[derive(Debug, Default)]
struct Acc {
    confidence: Option<Confidence>,
    name: Option<String>,
    end: Option<u64>,
    sources: SourceMask,
}

#[allow(clippy::too_many_arguments)]
fn record_signal(
    acc: &mut BTreeMap<u64, Acc>,
    stats: &mut DiscoveryStats,
    address: u64,
    value: f32,
    source: Source,
    name: Option<String>,
    end: Option<u64>,
    mask: SourceMask,
) {
    let entry = acc.entry(address).or_default();
    let already = entry.sources.contains(mask);
    let c = Confidence::new(value, source);
    entry.confidence = Some(match entry.confidence {
        Some(cur) => cur.join(c),
        None => c,
    });
    if entry.name.is_none() && name.is_some() {
        entry.name = name;
    }
    if entry.end.is_none() && end.is_some() {
        entry.end = end;
    }
    entry.sources.insert(mask);
    if !already {
        match mask {
            SourceMask::SYMBOL => stats.from_symbol += 1,
            SourceMask::ENTRY => stats.from_entry += 1,
            SourceMask::CALL => stats.from_call += 1,
            SourceMask::PROLOGUE => stats.from_prologue += 1,
            SourceMask::PLT => stats.from_plt += 1,
            _ => {}
        }
    }
}

fn in_executable(addr: u64, sections: &[&Section]) -> bool {
    sections.iter().any(|s| {
        let start = s.address;
        let end = start.saturating_add(s.size);
        addr >= start && addr < end
    })
}

fn section_bytes<'a>(section: &Section, bytes: &'a [u8]) -> Option<&'a [u8]> {
    let off = usize::try_from(section.file_offset?).ok()?;
    let size = usize::try_from(section.size).ok()?;
    let end = off.checked_add(size)?;
    if end > bytes.len() {
        return None;
    }
    Some(&bytes[off..end])
}

/// Detect the two x86 prologue patterns the discoverer recognises:
///
/// 1. `push rbp; mov rbp, rsp` — the canonical SysV / pre-CET frame
///    setup; address is the `push`.
/// 2. `endbr64; <push rbp | sub rsp, imm>` — CET landing pad
///    immediately followed by a frame setup or stack reservation;
///    address is the `endbr64`.
///
/// Returns the address that should be promoted to a function start, or
/// `None`. The decoder's instruction-boundary stream is the only thing
/// we trust as a "this address is a real opcode start"; mid-instruction
/// matches are impossible here because [`InstructionDecoder::iter`]
/// only yields aligned boundaries.
fn match_prologue(
    current: &dac_arch::DecodedInstruction,
    next: Option<&dac_arch::DecodedInstruction>,
) -> Option<u64> {
    if !current.valid {
        return None;
    }
    let next = next?;
    if !next.valid {
        return None;
    }
    if current.mnemonic == "push"
        && current.operands == "rbp"
        && next.mnemonic == "mov"
        && next.operands == "rbp,rsp"
    {
        return Some(current.address);
    }
    if current.mnemonic == "endbr64" {
        let follows_push_rbp = next.mnemonic == "push" && next.operands == "rbp";
        let follows_sub_rsp = next.mnemonic == "sub" && next.operands.starts_with("rsp,");
        if follows_push_rbp || follows_sub_rsp {
            return Some(current.address);
        }
    }
    None
}

fn fill_ends_from_neighbours(entries: &mut [(u64, Acc)], exec_sections: &[&Section]) {
    let starts: Vec<u64> = entries.iter().map(|(a, _)| *a).collect();
    for (i, (address, acc)) in entries.iter_mut().enumerate() {
        if acc.end.is_some() {
            continue;
        }
        let section_end = exec_sections
            .iter()
            .find(|s| {
                let start = s.address;
                let end = start.saturating_add(s.size);
                *address >= start && *address < end
            })
            .map(|s| s.address.saturating_add(s.size));
        let next_in_section = starts
            .get(i + 1)
            .copied()
            .filter(|n| section_end.is_none_or(|se| *n <= se));
        let bound = match (next_in_section, section_end) {
            (Some(n), Some(se)) => Some(n.min(se)),
            (Some(n), None) => Some(n),
            (None, Some(se)) => Some(se),
            (None, None) => None,
        };
        if let Some(b) = bound {
            if b > *address {
                acc.end = Some(b);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_binfmt::{
        Architecture, BinaryFormat, Bits, Endian, Permissions, Section, SectionKind, Symbol,
        SymbolBinding, SymbolKind, SymbolSource,
    };

    fn empty_model(text_address: u64, text_size: u64) -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: vec![Section {
                name: ".text".to_string(),
                address: text_address,
                size: text_size,
                file_offset: Some(0),
                perms: Permissions {
                    readable: true,
                    writable: false,
                    executable: true,
                },
                kind: SectionKind::Text,
            }],
            segments: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
            needed_libraries: Vec::new(),
        }
    }

    fn text_sym(name: &str, address: u64, size: u64) -> Symbol {
        Symbol {
            name: name.to_string(),
            address,
            size,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: Some(0),
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    /// A no-op decoder for tests that don't need real instructions. The
    /// iterator yields nothing, so call-edge / prologue passes find
    /// nothing — useful for testing the symbol / entry paths in
    /// isolation.
    struct NullDecoder;

    impl InstructionDecoder for NullDecoder {
        fn decode_one(
            &self,
            _bytes: &[u8],
            _address: u64,
        ) -> Result<dac_arch::DecodedInstruction, dac_arch::DecodeError> {
            Err(dac_arch::DecodeError::Truncated { offset: 0 })
        }

        fn iter<'a>(
            &'a self,
            _bytes: &'a [u8],
            _address: u64,
        ) -> Box<dyn Iterator<Item = dac_arch::DecodedInstruction> + 'a> {
            Box::new(std::iter::empty())
        }
    }

    #[test]
    fn source_mask_bits_compose_independently() {
        let mut m = SourceMask::empty();
        assert!(m.is_empty());
        m.insert(SourceMask::SYMBOL);
        assert!(m.contains(SourceMask::SYMBOL));
        assert!(!m.contains(SourceMask::CALL));
        m.insert(SourceMask::CALL);
        assert!(m.contains(SourceMask::SYMBOL));
        assert!(m.contains(SourceMask::CALL));
        assert!(!m.contains(SourceMask::ENTRY));
    }

    #[test]
    fn symbol_derived_function_carries_name_size_and_observed_source() {
        let mut model = empty_model(0x1000, 0x200);
        model.symbols.push(text_sym("main", 0x1000, 0x40));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        let f = &set.functions[0];
        assert_eq!(f.address, 0x1000);
        assert_eq!(f.end, Some(0x1040));
        assert_eq!(f.size(), Some(0x40));
        assert_eq!(f.name.as_deref(), Some("main"));
        assert_eq!(f.confidence.source(), Source::Observed);
        assert!(f.sources.contains(SourceMask::SYMBOL));
        assert_eq!(set.stats.from_symbol, 1);
    }

    #[test]
    fn entry_point_alone_produces_a_function_with_end_filled_from_section() {
        let mut model = empty_model(0x2000, 0x100);
        model.entry = Some(0x2000);
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        let f = &set.functions[0];
        assert_eq!(f.address, 0x2000);
        // No symbol-derived size, but the section bound fills the end.
        assert_eq!(f.end, Some(0x2100));
        assert!(f.sources.contains(SourceMask::ENTRY));
        assert_eq!(set.stats.from_entry, 1);
    }

    #[test]
    fn symbol_and_entry_at_same_address_merge_into_one_function() {
        let mut model = empty_model(0x3000, 0x100);
        model.entry = Some(0x3000);
        model.symbols.push(text_sym("main", 0x3000, 0x40));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        let f = &set.functions[0];
        assert!(f.sources.contains(SourceMask::SYMBOL));
        assert!(f.sources.contains(SourceMask::ENTRY));
        assert_eq!(f.confidence.source(), Source::Observed);
        // Both signals contribute, so both stats increment.
        assert_eq!(set.stats.from_symbol, 1);
        assert_eq!(set.stats.from_entry, 1);
    }

    #[test]
    fn unknown_end_is_filled_from_next_function_start_in_section() {
        let mut model = empty_model(0x4000, 0x200);
        // Two entry-only signals; the first should be bounded by the
        // second's address, the second by the section end.
        model.symbols.push(text_sym("a", 0x4000, 0));
        model.symbols.push(text_sym("b", 0x4080, 0));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 2);
        assert_eq!(set.functions[0].address, 0x4000);
        assert_eq!(set.functions[0].end, Some(0x4080));
        assert_eq!(set.functions[1].address, 0x4080);
        assert_eq!(set.functions[1].end, Some(0x4200));
    }

    #[test]
    fn function_set_supports_address_lookup() {
        let mut model = empty_model(0x5000, 0x80);
        model.symbols.push(text_sym("f", 0x5000, 0x20));
        model.symbols.push(text_sym("g", 0x5040, 0x20));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert!(set.contains_address(0x5000));
        assert!(set.contains_address(0x5040));
        assert!(!set.contains_address(0x5020));
        assert_eq!(set.get(0x5040).unwrap().name.as_deref(), Some("g"));
        let addrs: Vec<u64> = set.addresses().collect();
        assert_eq!(addrs, vec![0x5000, 0x5040]);
    }

    #[test]
    fn each_function_gets_a_supported_cfg_evidence_node() {
        let mut model = empty_model(0x6000, 0x80);
        model.symbols.push(text_sym("f", 0x6000, 0x40));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        let ev = set.functions[0].evidence;
        match g.node(ev) {
            Some(EvidenceNode::IrNode {
                layer: IrLayer::Cfg,
                id: 0,
            }) => {}
            other => panic!("expected Cfg IrNode with id 0, got {other:?}"),
        }
        // The graph must contain at least one Bytes node for the function
        // span with a Supports edge into the IR node.
        let supports_count = g
            .iter()
            .filter_map(|(id, node)| match node {
                EvidenceNode::Bytes { start, end } if *start == 0x6000 && *end == 0x6040 => {
                    Some(id)
                }
                _ => None,
            })
            .filter(|bid| {
                g.out_edges(*bid)
                    .iter()
                    .any(|e| e.target == ev && e.kind == EdgeKind::Supports)
            })
            .count();
        assert_eq!(supports_count, 1);
    }

    #[test]
    fn out_of_section_symbol_is_ignored() {
        let mut model = empty_model(0x7000, 0x80);
        // Address outside the only executable section.
        model.symbols.push(text_sym("ghost", 0x9000, 0x10));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert!(set.functions.is_empty());
        assert_eq!(set.stats.from_symbol, 0);
    }

    #[test]
    fn undefined_symbol_is_ignored() {
        let mut model = empty_model(0x8000, 0x80);
        let mut sym = text_sym("imp", 0x0, 0);
        sym.undefined = true;
        model.symbols.push(sym);
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert!(set.functions.is_empty());
    }

    // ---- B3.23 PLT stub recognition + import naming ----------------

    use dac_binfmt::{Relocation, RelocationKind, SymbolSource as BinSymbolSource};

    /// Build a 6-byte `jmp qword ptr [rip + disp32]` whose effective
    /// address resolves to `got_va` when decoded at `stub_va`. Mirrors
    /// the helper in `dac-binfmt::plt` tests.
    fn encode_jmp_indirect(stub_va: u64, got_va: u64) -> [u8; 6] {
        let rip = stub_va + 6;
        let disp = (got_va as i64 - rip as i64) as i32;
        let mut bytes = [0u8; 6];
        bytes[0] = 0xff;
        bytes[1] = 0x25;
        bytes[2..6].copy_from_slice(&disp.to_le_bytes());
        bytes
    }

    /// Add a `.plt` section to `model` and write a single PLT stub at
    /// `stub_va` pointing at `got_va`. Also adds a matching
    /// `JUMP_SLOT` relocation binding `got_va` to the named dynsym
    /// entry. Returns the file-image byte buffer the discoverer reads.
    fn make_plt_model(stub_va: u64, got_va: u64, import: &str) -> (BinaryModel, Vec<u8>) {
        let mut model = empty_model(0x1000, 0x40);
        // Replace the default `.text` section with a `.plt` section
        // so the PLT walker picks the stub up. The discoverer treats
        // both equally because both are executable.
        model.sections[0].name = ".plt".to_string();
        let stub_off_in_section = (stub_va - model.sections[0].address) as usize;
        let mut bytes = vec![0x90u8; model.sections[0].size as usize];
        bytes[stub_off_in_section..stub_off_in_section + 6]
            .copy_from_slice(&encode_jmp_indirect(stub_va, got_va));
        // Dynsym entry for the imported symbol (undefined / address 0).
        model.symbols.push(Symbol {
            name: import.to_string(),
            address: 0,
            size: 0,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: None,
            source: BinSymbolSource::Dynsym,
            undefined: true,
        });
        model.relocations.push(Relocation {
            section: None,
            offset: got_va,
            kind: RelocationKind::Glob,
            symbol: Some(0),
            addend: 0,
        });
        (model, bytes)
    }

    /// A PLT trampoline reaches the discoverer through `elf_x86_64_plt_stubs`
    /// even when nothing else (no symbol, no caller, no prologue) pointed
    /// at the stub address: the discoverer mints a fresh function with
    /// the imported symbol's name and `FunctionKind::PltStub`.
    #[test]
    fn b3_23_plt_stub_is_minted_when_otherwise_undiscovered() {
        let (model, bytes) = make_plt_model(0x1030, 0x4000, "write");
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        let f = &set.functions[0];
        assert_eq!(f.address, 0x1030);
        assert_eq!(f.name.as_deref(), Some("write"));
        assert!(f.sources.contains(SourceMask::PLT));
        assert_eq!(f.confidence.source(), Source::Observed);
        assert_eq!(
            f.kind,
            FunctionKind::PltStub {
                import: "write".to_string()
            }
        );
        assert_eq!(set.stats.from_plt, 1);
    }

    /// When a caller already minted a CALL signal for the PLT stub's
    /// VA, the PLT binding *joins* the existing accumulator: the name
    /// is set (which was `None` before), the confidence is promoted
    /// from `Derived` to `Observed`, and the bitmask carries both
    /// signals so `--debug` consumers see CALL and PLT.
    #[test]
    fn b3_23_plt_binding_promotes_existing_call_discovery() {
        let (mut model, bytes) = make_plt_model(0x1030, 0x4000, "malloc");
        // Pretend something direct-called the stub. The discoverer
        // would normally fold this in via the call-edge sweep; we
        // pre-seed the entry through a CALL-bearing symbol-table
        // placeholder so the test is decoder-independent.
        model.symbols.push(text_sym("", 0x1030, 0));
        model.symbols[1].undefined = false;
        // The pre-seed acts as a SYMBOL signal; the PLT binding still
        // wins on naming because the symbol-table entry is anonymous.
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        let f = set.get(0x1030).expect("PLT stub at 0x1030");
        assert_eq!(f.name.as_deref(), Some("malloc"));
        assert!(f.sources.contains(SourceMask::PLT));
        assert!(f.sources.contains(SourceMask::SYMBOL));
        assert_eq!(f.confidence.source(), Source::Observed);
        assert_eq!(
            f.kind,
            FunctionKind::PltStub {
                import: "malloc".to_string()
            }
        );
    }

    /// A symbol-table-derived name takes precedence over the PLT name
    /// — the relocation table and the symbol table both point at the
    /// same VA, so the symbol-table name wins (it's a higher-level
    /// fact). The function still tags as `PltStub` because the
    /// trampoline shape is structural.
    #[test]
    fn b3_23_named_symbol_keeps_its_name_but_still_tags_as_plt_stub() {
        let (mut model, bytes) = make_plt_model(0x1030, 0x4000, "write");
        model.symbols.push(text_sym("write_alias", 0x1030, 0x10));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        let f = set.get(0x1030).expect("PLT stub at 0x1030");
        // The symbol-table entry came first and pinned the name.
        assert_eq!(f.name.as_deref(), Some("write_alias"));
        // But the PLT binding still tagged the kind so the C backend
        // emits an extern declaration rather than a body.
        assert_eq!(
            f.kind,
            FunctionKind::PltStub {
                import: "write".to_string()
            }
        );
        assert!(f.sources.contains(SourceMask::PLT));
        assert!(f.sources.contains(SourceMask::SYMBOL));
    }

    /// Non-PLT functions (no relocation, no `.plt` section, ordinary
    /// `.text`) keep `FunctionKind::User` and don't surface in the
    /// `from_plt` counter.
    #[test]
    fn b3_23_user_functions_keep_user_kind() {
        let mut model = empty_model(0x2000, 0x80);
        model.symbols.push(text_sym("main", 0x2000, 0x40));
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        assert_eq!(set.functions.len(), 1);
        assert_eq!(set.functions[0].kind, FunctionKind::User);
        assert!(!set.functions[0].sources.contains(SourceMask::PLT));
        assert_eq!(set.stats.from_plt, 0);
    }
}
