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

use std::collections::{BTreeMap, BTreeSet};

use dac_arch::{ControlFlow, DecodedInstruction, InstructionDecoder};
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

/// Coarse taxonomy for a recovered function (B3.23, B3.25).
///
/// `User` covers any function whose body should be lowered to source.
/// `PltStub` marks a Procedure Linkage Table trampoline bound to a
/// concrete imported symbol — the C backend renders these as `extern`
/// forward declarations instead of bodies, and call sites resolve
/// through the import name. `Thunk` marks an in-binary forwarding
/// thunk whose entire body is `[endbr64?]; jmp <known function>` —
/// the C backend renders these as a one-line tail call to the target
/// instead of stubbing the body with a structuring fallback (B3.25,
/// FR-21).
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
    /// An in-binary forwarding thunk whose entire body is an optional
    /// `endbr64` landing pad followed by an unconditional jump to
    /// another recovered function's entry. `target` is the
    /// jump-target virtual address (B3.25, FR-21).
    Thunk { target: u64 },
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

    /// Coarse human-readable classification (B3.30, FR-21).
    ///
    /// Distinct from [`FunctionKind`], which records the *structural*
    /// fact ("this body is a PLT trampoline / a forwarding thunk /
    /// ordinary code"). [`FunctionTaxonomy`] is the *intent* a
    /// reviewer cares about ("this function is runtime scaffolding I
    /// can skip / it's imported / it's a thunk / it's user code").
    ///
    /// Priority order, highest first:
    /// 1. **`CrtSupport`** when the recovered name matches an entry in
    ///    [`dac_knowledge::lookup_crt_entry`]. Wins over the
    ///    structural classifications because a CRT helper that
    ///    happens to be implemented as a thunk (e.g. glibc's
    ///    `frame_dummy`, recovered by [`detect_thunks`] as
    ///    [`FunctionKind::Thunk`]) is still runtime scaffolding from
    ///    the reviewer's point of view.
    /// 2. **`Imported`** for [`FunctionKind::PltStub`]. The body is a
    ///    PLT trampoline; the import lives in another image.
    /// 3. **`Thunk`** for [`FunctionKind::Thunk`] not matched by the
    ///    CRT catalogue. Forwarding shape that is not runtime
    ///    scaffolding (e.g. compiler-emitted same-image thunks).
    /// 4. **`User`** for everything else.
    #[must_use]
    pub fn taxonomy(&self) -> FunctionTaxonomy {
        if let Some(name) = self.name.as_deref() {
            if dac_knowledge::lookup_crt_entry(name).is_some() {
                return FunctionTaxonomy::CrtSupport;
            }
        }
        match &self.kind {
            FunctionKind::PltStub { .. } => FunctionTaxonomy::Imported,
            FunctionKind::Thunk { .. } => FunctionTaxonomy::Thunk,
            FunctionKind::User => FunctionTaxonomy::User,
        }
    }
}

/// Reviewer-facing classification of a recovered function (B3.30,
/// FR-21).
///
/// Derived from a [`Function`]'s name + [`FunctionKind`] through
/// [`Function::taxonomy`]; this enum exists so the C backend and the
/// report agree on a single classification axis when deciding what
/// banner to print and how to count "this is runtime scaffolding"
/// versus "this is user code".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionTaxonomy {
    /// Ordinary user-program function. The default classification.
    User,
    /// Runtime / startup helper recognised by
    /// [`dac_knowledge::lookup_crt_entry`]. The C backend prints a
    /// "runtime support — not user code" banner above the body and
    /// the `--hide-crt` flag collapses it to a forward declaration.
    CrtSupport,
    /// In-binary forwarding thunk (B3.25). The body is a one-line
    /// tail call to the recovered target.
    Thunk,
    /// PLT-bound import (B3.23). The body lives in another image; the
    /// source file just states the signature.
    Imported,
}

impl FunctionTaxonomy {
    /// Lowercase snake-case label used by the report's `;; taxonomy:`
    /// histogram row. Stable across versions of this crate.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            FunctionTaxonomy::User => "user",
            FunctionTaxonomy::CrtSupport => "crt_support",
            FunctionTaxonomy::Thunk => "thunk",
            FunctionTaxonomy::Imported => "imported",
        }
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
    /// Number of functions reclassified as forwarding thunks by
    /// [`detect_thunks`] (B3.25). Counts the
    /// `[endbr64?]; jmp <known function>` shape only — indirect
    /// IAT-slot jumps are handled by the PLT-binding path instead.
    pub from_thunk: u64,
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

/// Reclassify forwarding thunks in `set` (B3.25, FR-21).
///
/// A *forwarding thunk* is a function whose entire body is
///
/// ```text
/// [endbr64?] ; jmp <target>
/// ```
///
/// where `<target>` is itself the entry of another recovered function
/// in `set`. Trailing nop padding is tolerated (the unconditional jmp
/// ends control flow, so anything after it is dead). The classic
/// example is `frame_dummy` on ELF, which is `endbr64; jmp
/// register_tm_clones`; on PE, mingw's `atexit` is `jmp _crt_atexit`.
///
/// Matched functions transition from [`FunctionKind::User`] to
/// [`FunctionKind::Thunk`] with `target` set to the jump destination.
/// The C backend then renders the body as a one-line tail call to the
/// target instead of a `/* dac: structuring fallback */` stub, which
/// is what every other path would produce for a function whose
/// terminator is a tail-jump out of itself (B3.27 collapsed
/// `__builtin_unreachable();` to the structuring-fallback line, and a
/// thunk's body is exactly that case).
///
/// Functions already reclassified as [`FunctionKind::PltStub`] are
/// left untouched — a PLT trampoline is also a forwarding thunk in
/// spirit, but it binds to an *external* import and the extern-decl
/// rendering already conveys the binding more precisely than a
/// `return imp();` body would. Functions with an unknown `end` are
/// skipped because the byte range that bounds the pattern match is
/// not known.
///
/// Determinism: [`Pure`](dac_core::Source) — the only inputs are the
/// (already-deterministic) function set and the (already-deterministic)
/// decoded instruction stream, and the only side-effect is mutating
/// `set.functions[i].kind` plus the
/// [`DiscoveryStats::from_thunk`] counter. The evidence graph is
/// untouched because the byte-range and CFG-layer nodes the
/// discoverer minted already cover the thunk's span.
pub fn detect_thunks(
    set: &mut FunctionSet,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
) {
    let exec_sections: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();
    let entries: BTreeSet<u64> = set.functions.iter().map(|f| f.address).collect();
    for f in set.functions.iter_mut() {
        if !matches!(f.kind, FunctionKind::User) {
            continue;
        }
        let Some(end) = f.end else { continue };
        let Some(slice) = function_slice(f.address, end, &exec_sections, bytes) else {
            continue;
        };
        let instructions: Vec<DecodedInstruction> =
            decoder.iter(slice, f.address).take(3).collect();
        if let Some(target) = match_thunk_pattern(&instructions, &entries) {
            f.kind = FunctionKind::Thunk { target };
            set.stats.from_thunk += 1;
        }
    }
}

/// Match the canonical forwarding-thunk shape in `body`. Returns the
/// jump target when the body is `[endbr64?]; jmp <known function>`
/// (with `<known function>` ∈ `entries`); returns `None` otherwise.
///
/// `body` is the head of the function's instruction stream (the
/// caller passes the first three decoded instructions, which is
/// enough to cover the longest accepted prefix). Trailing instructions
/// after the unconditional jmp are dead and are not inspected.
fn match_thunk_pattern(body: &[DecodedInstruction], entries: &BTreeSet<u64>) -> Option<u64> {
    let mut iter = body.iter();
    let first = iter.next()?;
    if !first.valid {
        return None;
    }
    let jmp = if first.mnemonic == "endbr64" {
        let second = iter.next()?;
        if !second.valid {
            return None;
        }
        second
    } else {
        first
    };
    let ControlFlow::UnconditionalBranch { target: Some(t) } = jmp.flow else {
        return None;
    };
    if !entries.contains(&t) {
        return None;
    }
    Some(t)
}

/// Slice `bytes` to the function's executable byte range, clamped to
/// the enclosing executable section. Returns `None` when the start /
/// end / section bounds disagree — the same defensive shape as
/// [`crate::convention`]-side helpers, lifted here so
/// [`detect_thunks`] can reuse it without leaking into the public
/// surface.
fn function_slice<'a>(
    start: u64,
    end: u64,
    exec_sections: &[&Section],
    bytes: &'a [u8],
) -> Option<&'a [u8]> {
    let sec = exec_sections.iter().find(|s| {
        let s_start = s.address;
        let s_end = s_start.saturating_add(s.size);
        start >= s_start && start < s_end
    })?;
    let s_start = sec.address;
    let s_end = s_start.saturating_add(sec.size);
    let clamped_end = end.min(s_end);
    if clamped_end <= start {
        return None;
    }
    let file_off = usize::try_from(sec.file_offset?).ok()?;
    let in_sec_off = usize::try_from(start - s_start).ok()?;
    let length = usize::try_from(clamped_end - start).ok()?;
    let begin = file_off.checked_add(in_sec_off)?;
    let finish = begin.checked_add(length)?;
    if finish > bytes.len() {
        return None;
    }
    Some(&bytes[begin..finish])
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

    // ---- B3.25 forwarding-thunk recognition ------------------------

    /// Build a `DecodedInstruction` carrying `mnemonic` / `flow` at
    /// `address`. The bytes / operands fields are irrelevant for the
    /// thunk detector (only `mnemonic`, `valid`, and `flow` are
    /// inspected), so the helper leaves them at their defaults so the
    /// test reads as a sequence of "endbr64 here, jmp there".
    fn decoded(address: u64, mnemonic: &str, flow: ControlFlow) -> dac_arch::DecodedInstruction {
        dac_arch::DecodedInstruction {
            address,
            length: 1,
            bytes: Vec::new(),
            mnemonic: mnemonic.to_string(),
            operands: String::new(),
            flow,
            valid: true,
        }
    }

    /// A decoder driven by a per-address script. `scripts` maps the
    /// requested decode address to the canned instruction stream the
    /// helper hands back. Useful for exercising
    /// [`detect_thunks`]'s pattern matcher without round-tripping
    /// through the real x86-64 decoder.
    struct ScriptedDecoder {
        scripts: std::collections::BTreeMap<u64, Vec<dac_arch::DecodedInstruction>>,
    }

    impl ScriptedDecoder {
        fn new() -> Self {
            Self {
                scripts: std::collections::BTreeMap::new(),
            }
        }

        fn at(mut self, address: u64, stream: Vec<dac_arch::DecodedInstruction>) -> Self {
            self.scripts.insert(address, stream);
            self
        }
    }

    impl InstructionDecoder for ScriptedDecoder {
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
            address: u64,
        ) -> Box<dyn Iterator<Item = dac_arch::DecodedInstruction> + 'a> {
            let stream = self.scripts.get(&address).cloned().unwrap_or_default();
            Box::new(stream.into_iter())
        }
    }

    /// Set up a two-function model: a candidate thunk at `thunk_va`
    /// and its target at `target_va` inside the same `.text` section.
    /// Returns the model alongside a byte buffer sized to cover both
    /// functions so the `function_slice` byte-range bookkeeping is
    /// happy (the scripted decoder ignores the contents).
    fn two_function_model(thunk_va: u64, target_va: u64) -> (BinaryModel, Vec<u8>) {
        let base = thunk_va.min(target_va);
        let end = thunk_va.max(target_va).saturating_add(0x20);
        let section_size = end - base;
        let mut model = empty_model(base, section_size);
        model.symbols.push(text_sym("thunk", thunk_va, 0x09));
        model.symbols.push(text_sym("target", target_va, 0x10));
        let bytes = vec![0u8; section_size as usize];
        (model, bytes)
    }

    /// `endbr64; jmp <target>` lowering thunk: the canonical
    /// `frame_dummy` shape. After [`detect_thunks`] the function
    /// reclassifies to `FunctionKind::Thunk { target }` and the
    /// discovery stats grow `from_thunk += 1`.
    #[test]
    fn b3_25_endbr64_jmp_known_function_reclassifies_as_thunk() {
        let thunk_va = 0x1150;
        let target_va = 0x10c0;
        let (model, bytes) = two_function_model(thunk_va, target_va);
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![
                decoded(thunk_va, "endbr64", ControlFlow::Sequential),
                decoded(
                    thunk_va + 4,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(target_va),
                    },
                ),
            ],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        detect_thunks(&mut set, &model, &bytes, &decoder);
        let f = set.get(thunk_va).expect("thunk function");
        assert_eq!(f.kind, FunctionKind::Thunk { target: target_va });
        assert_eq!(set.stats.from_thunk, 1);
        // The target stays a plain user function.
        let t = set.get(target_va).expect("target function");
        assert_eq!(t.kind, FunctionKind::User);
    }

    /// Bare `jmp <target>` (no `endbr64` prefix) — the mingw `atexit`
    /// shape on PE. Still a forwarding thunk; reclassifies the same
    /// way.
    #[test]
    fn b3_25_bare_jmp_known_function_reclassifies_as_thunk() {
        let thunk_va = 0x1460;
        let target_va = 0x29c8;
        let (model, bytes) = two_function_model(thunk_va, target_va);
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![decoded(
                thunk_va,
                "jmp",
                ControlFlow::UnconditionalBranch {
                    target: Some(target_va),
                },
            )],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        detect_thunks(&mut set, &model, &bytes, &decoder);
        assert_eq!(
            set.get(thunk_va).unwrap().kind,
            FunctionKind::Thunk { target: target_va }
        );
        assert_eq!(set.stats.from_thunk, 1);
    }

    /// `xor ecx, ecx; jmp <target>` (mingw `safe_flush` shape) is
    /// *not* a pure forwarding thunk: the body mutates a register
    /// before forwarding. Recognising it as a thunk would lose that
    /// behaviour, so the detector leaves the kind as `User`.
    #[test]
    fn b3_25_jmp_after_other_work_is_not_a_thunk() {
        let thunk_va = 0x1010;
        let target_va = 0x2990;
        let (model, bytes) = two_function_model(thunk_va, target_va);
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![
                decoded(thunk_va, "xor", ControlFlow::Sequential),
                decoded(
                    thunk_va + 2,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(target_va),
                    },
                ),
            ],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        detect_thunks(&mut set, &model, &bytes, &decoder);
        assert_eq!(set.get(thunk_va).unwrap().kind, FunctionKind::User);
        assert_eq!(set.stats.from_thunk, 0);
    }

    /// A jump whose target is *not* itself a recovered function entry
    /// is not a forwarding thunk (it might be a branch into the
    /// middle of another function, a PLT stub we haven't bound yet,
    /// or noise) — the detector leaves the kind as `User`.
    #[test]
    fn b3_25_jmp_to_non_entry_address_is_not_a_thunk() {
        let thunk_va = 0x1150;
        let (model, bytes) = two_function_model(thunk_va, 0x10c0);
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![decoded(
                thunk_va,
                "jmp",
                // 0xdead is not in the function set.
                ControlFlow::UnconditionalBranch {
                    target: Some(0xdead),
                },
            )],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        detect_thunks(&mut set, &model, &bytes, &decoder);
        assert_eq!(set.get(thunk_va).unwrap().kind, FunctionKind::User);
        assert_eq!(set.stats.from_thunk, 0);
    }

    /// An indirect jump (`jmp rax`, `jmp [rip+disp]`) is structurally
    /// a thunk shape on PE / ELF .got.plt, but B3.25 only handles
    /// *direct* jumps with a known immediate target. The PLT-binding
    /// path (B3.23) is what classifies indirect IAT jumps. The
    /// detector therefore leaves these as `User` (or as `PltStub`
    /// when the PLT walker matched them earlier).
    #[test]
    fn b3_25_indirect_jmp_is_not_classified_as_thunk() {
        let thunk_va = 0x2960;
        let (model, bytes) = two_function_model(thunk_va, 0x29c8);
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![decoded(thunk_va, "jmp", ControlFlow::IndirectBranch)],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        detect_thunks(&mut set, &model, &bytes, &decoder);
        assert_eq!(set.get(thunk_va).unwrap().kind, FunctionKind::User);
        assert_eq!(set.stats.from_thunk, 0);
    }

    /// A function already classified as a PLT stub stays a PLT stub
    /// even when its body trivially looks like a forwarding thunk.
    /// The extern-decl rendering carries more information than the
    /// thunk path (it binds to the relocation table), so the
    /// detector skips it.
    #[test]
    fn b3_25_plt_stub_kind_is_preserved() {
        let thunk_va = 0x1030;
        let (model, bytes) = make_plt_model(thunk_va, 0x4000, "write");
        let decoder = ScriptedDecoder::new().at(
            thunk_va,
            vec![decoded(
                thunk_va,
                "jmp",
                ControlFlow::UnconditionalBranch {
                    target: Some(0x4000),
                },
            )],
        );
        let mut g = EvidenceGraph::new();
        let mut set = discover_functions(&model, &bytes, &NullDecoder, &mut g);
        // Sanity: the PLT walker minted the stub as PltStub.
        assert!(matches!(
            set.get(thunk_va).unwrap().kind,
            FunctionKind::PltStub { .. }
        ));
        detect_thunks(&mut set, &model, &bytes, &decoder);
        assert!(matches!(
            set.get(thunk_va).unwrap().kind,
            FunctionKind::PltStub { .. }
        ));
        assert_eq!(set.stats.from_thunk, 0);
    }

    // ---- B3.30 ----

    fn function_with(name: Option<&str>, kind: FunctionKind) -> Function {
        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        Function {
            address: 0x1000,
            end: Some(0x1040),
            name: name.map(|s| s.to_string()),
            confidence: Confidence::new(1.0, Source::Observed),
            sources: SourceMask::SYMBOL,
            evidence: ev,
            kind,
        }
    }

    /// A glibc startup helper resolves to `CrtSupport` regardless of
    /// its structural kind. `frame_dummy` lives in the hello-x86_64
    /// fixture as a recovered thunk; the CRT classification wins.
    #[test]
    fn b3_30_glibc_helper_taxonomy_is_crt_support() {
        let user = function_with(Some("_init"), FunctionKind::User);
        assert_eq!(user.taxonomy(), FunctionTaxonomy::CrtSupport);
        let thunk_helper =
            function_with(Some("frame_dummy"), FunctionKind::Thunk { target: 0x10c0 });
        assert_eq!(thunk_helper.taxonomy(), FunctionTaxonomy::CrtSupport);
    }

    /// A MinGW startup helper resolves to `CrtSupport`. `mainCRTStartup`
    /// is the entry point on Windows PE binaries built with MinGW-w64.
    #[test]
    fn b3_30_mingw_helper_taxonomy_is_crt_support() {
        let f = function_with(Some("mainCRTStartup"), FunctionKind::User);
        assert_eq!(f.taxonomy(), FunctionTaxonomy::CrtSupport);
    }

    /// A PLT stub falls into `Imported`. The CRT catalogue does not
    /// include imported symbols.
    #[test]
    fn b3_30_plt_stub_taxonomy_is_imported() {
        let f = function_with(
            Some("write"),
            FunctionKind::PltStub {
                import: "write".to_string(),
            },
        );
        assert_eq!(f.taxonomy(), FunctionTaxonomy::Imported);
    }

    /// A forwarding thunk that is *not* a CRT helper falls into the
    /// `Thunk` bucket; the recovered structural fact stays visible.
    #[test]
    fn b3_30_non_crt_thunk_taxonomy_is_thunk() {
        let f = function_with(
            Some("my_user_thunk"),
            FunctionKind::Thunk { target: 0x2000 },
        );
        assert_eq!(f.taxonomy(), FunctionTaxonomy::Thunk);
    }

    /// A user-named `User`-kind function with no CRT match stays
    /// `User`. The default case the report's histogram counts.
    #[test]
    fn b3_30_user_function_taxonomy_is_user() {
        let f = function_with(Some("main"), FunctionKind::User);
        assert_eq!(f.taxonomy(), FunctionTaxonomy::User);
        let unnamed = function_with(None, FunctionKind::User);
        assert_eq!(unnamed.taxonomy(), FunctionTaxonomy::User);
    }

    /// Labels are the stable strings the report histogram emits and
    /// `--hide-crt` diagnostics quote. Lock them in here so a rename
    /// of the variant does not silently shift the rendered string.
    #[test]
    fn b3_30_function_taxonomy_labels_are_stable() {
        assert_eq!(FunctionTaxonomy::User.label(), "user");
        assert_eq!(FunctionTaxonomy::CrtSupport.label(), "crt_support");
        assert_eq!(FunctionTaxonomy::Thunk.label(), "thunk");
        assert_eq!(FunctionTaxonomy::Imported.label(), "imported");
    }
}
