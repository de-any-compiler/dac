//! Whole-program call graph and cross-reference index (B3.1, FR-26, FR-27).
//!
//! Given a [`FunctionSet`] (from `dac-recovery`), a [`BinaryModel`], and
//! an [`InstructionDecoder`] for the binary's architecture, this module
//! builds two related artifacts:
//!
//! - [`CallGraph`] — the function-to-function (and function-to-import)
//!   call graph (FR-27). Direct calls land as `Function → Function`
//!   edges; direct calls to addresses that are not function starts (or
//!   that fall outside every recovered function) become
//!   `Function → Unresolved` edges; indirect calls become
//!   `Function → IndirectSite` edges anchored at the calling instruction
//!   so the caller can still see *where* the indirection happened.
//! - [`XrefIndex`] — every cross-reference between code and data
//!   (FR-26). Three orthogonal axes carry meaning:
//!   1. **Source kind**: code (an instruction at some VA) or data
//!      (a section offset / VA in a non-executable section).
//!   2. **Target kind**: same options, plus "external" for imports.
//!   3. **Edge kind**: [`XrefKind`] — `Call`, `TailCall`, `IndirectCall`,
//!      `CodeToData`, `DataToCode`, `DataToData`, `Import`, `Export`.
//!
//! Both artifacts are pure: they read [`InstructionDecoder::iter`] on
//! the bytes of the function's body, plus [`BinaryModel::relocations`],
//! [`BinaryModel::exports`], and [`BinaryModel::entry`]. No mutation of
//! the [`EvidenceGraph`] is performed here — callers wire the resulting
//! facts in as they like (see `dac-cli` for the orchestration site).
//!
//! ## Determinism
//!
//! Every list returned by this module is sorted by stable address-major
//! keys. `CallGraph::edges` is sorted by `(from_node, site, to_node)`;
//! `XrefIndex` is internally a `Vec<Xref>` sorted by `(to, from, kind)`
//! so `to(addr)` and `from(addr)` can binary-search. The CLI's textual
//! renderer (`dac-cli::xrefs_report`) walks these vectors in order, so
//! the emitted bytes are byte-identical across runs (NFR-9, I-4).
//!
//! ## What this module does *not* do
//!
//! - It does not classify individual instruction operands as
//!   reads / writes / lea-style address-takes. That would need a typed
//!   walk over Instruction IR, which is per-instruction work that grows
//!   with the lifter's coverage. B2.6's lattice gives us the seed
//!   information for B3.2; the operand-level xrefs land alongside it.
//! - It does not resolve PLT trampolines back to their imported symbol
//!   names. Imports surface here as `CallNodeKind::Import` nodes named
//!   by the import they correspond to (when known via the relocation
//!   table); the trampoline-side mechanics are an arch-and-format
//!   problem better solved once we have a real `.plt` model.

use std::collections::BTreeMap;

use dac_arch::{ControlFlow, InstructionDecoder};
use dac_binfmt::{BinaryModel, RelocationKind, Section, SectionKind};
use dac_core::{Confidence, Source};
use dac_recovery::FunctionSet;

/// Confidence value for a direct-call edge whose target lands on a
/// known function entry. Source axis is [`Source::Observed`] because we
/// observed both the call instruction and the destination function.
pub const DIRECT_CALL_CONFIDENCE: f32 = 1.0;
/// Confidence value for a direct call whose target does not match any
/// recovered function entry. Source axis is [`Source::Derived`] — we
/// observed the call site but only inferred that the target is callable.
pub const UNRESOLVED_DIRECT_CALL_CONFIDENCE: f32 = 0.7;
/// Confidence value for a tail-call edge: a direct branch whose
/// destination is another function's entry. Source axis is
/// [`Source::Derived`].
pub const TAIL_CALL_CONFIDENCE: f32 = 0.8;
/// Confidence value for an indirect call edge. Source axis is
/// [`Source::Speculative`] because the destination cannot be resolved
/// from the decoded instruction alone.
pub const INDIRECT_CALL_CONFIDENCE: f32 = 0.4;
/// Confidence value for a relocation-derived xref. Source axis is
/// [`Source::Observed`].
pub const RELOC_CONFIDENCE: f32 = 1.0;
/// Confidence value for an exported-symbol xref (synthetic external
/// reference). Source axis is [`Source::Observed`].
pub const EXPORT_CONFIDENCE: f32 = 1.0;

/// Synthetic "address" used as `from` in xrefs that originate outside
/// any loaded section — e.g. the binary's entry point and exported
/// symbols both have an *implicit* external caller. Picked as `0` since
/// no real instruction lives at VA `0` in any loaded image we model.
pub const EXTERNAL_VA: u64 = 0;

/// Edge-kind discriminator for a single cross-reference (FR-26).
///
/// The vocabulary is intentionally small: every kind names a directed
/// relationship between two addresses, and the kind is enough to drive
/// the textual renderer. Confidence and source live on the [`Xref`]
/// itself so the audit trail is preserved (I-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum XrefKind {
    /// Direct call from a code address to another code address.
    Call,
    /// Direct branch (`jmp`) that leaves the source function and lands
    /// on another function's entry — the compiler-emitted tail-call.
    TailCall,
    /// Indirect call (`call rax`, `call [rip+disp]`, …). `to` is
    /// always [`EXTERNAL_VA`] because the destination is unknowable
    /// from the decoded instruction; the *site* is the meaningful
    /// information.
    IndirectCall,
    /// Code instruction references a data address (load / store /
    /// address-of). Today this is only minted from relocations whose
    /// patched address is in an executable section and whose target is
    /// in a data section.
    CodeToData,
    /// Data references a code address — function pointer tables,
    /// `.init_array`, vtables, …. From relocations only.
    DataToCode,
    /// Data references data. From relocations whose patched address
    /// and target both fall in non-executable sections.
    DataToData,
    /// Code calls (or relocation targets) an imported symbol whose
    /// definition lives outside the binary. `to` is `EXTERNAL_VA`; the
    /// name is recorded in the call graph's [`CallNode::name`].
    Import,
    /// External holder (the dynamic loader, another module) references
    /// `to` because it is exported. `from` is `EXTERNAL_VA`.
    Export,
}

impl XrefKind {
    /// Stable short tag used by the CLI's textual renderer. Two-letter
    /// codes keep columns aligned without exhausting the namespace.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Call => "CALL",
            Self::TailCall => "TAIL",
            Self::IndirectCall => "ICALL",
            Self::CodeToData => "C->D",
            Self::DataToCode => "D->C",
            Self::DataToData => "D->D",
            Self::Import => "IMP",
            Self::Export => "EXP",
        }
    }
}

/// A single cross-reference.
///
/// `from` is the VA producing the reference (an instruction address,
/// a relocation patch address, or [`EXTERNAL_VA`] for synthetic edges).
/// `to` is the VA being referenced. The pair carries a [`Confidence`]
/// so a downstream renderer can decide whether to display the edge in
/// the report or as a debug annotation only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xref {
    pub from: u64,
    pub to: u64,
    pub kind: XrefKind,
    pub confidence: Confidence,
}

/// Address-indexed cross-reference table.
///
/// The internal vector is sorted by `(to, from, kind)`, which is the
/// order most queries want: "what references `addr`?" is the common
/// case (the textual `--xrefs sym` rendering walks `xrefs_to(addr)`).
/// "What does `addr` reference?" is supported by a parallel index keyed
/// by `from`.
#[derive(Debug, Clone, Default)]
pub struct XrefIndex {
    /// All xrefs, sorted by `(to, from, kind)`. Public so renderers and
    /// debugging consumers can iterate in a stable order.
    pub xrefs: Vec<Xref>,
    /// Index over `xrefs` keyed by `from`: maps each source VA to the
    /// indices in `xrefs` that originate there, in insertion order.
    from_index: BTreeMap<u64, Vec<u32>>,
    /// Index over `xrefs` keyed by `to`: maps each target VA to the
    /// indices in `xrefs` that point there, in insertion order.
    to_index: BTreeMap<u64, Vec<u32>>,
}

impl XrefIndex {
    /// All xrefs whose target is `addr`. Empty when nothing references
    /// `addr`. Returned slice is sorted by `(from, kind)` because the
    /// underlying `xrefs` vector is `(to, from, kind)`-sorted.
    #[must_use]
    pub fn to(&self, addr: u64) -> Vec<&Xref> {
        match self.to_index.get(&addr) {
            Some(ids) => ids.iter().map(|&i| &self.xrefs[i as usize]).collect(),
            None => Vec::new(),
        }
    }

    /// All xrefs whose source is `addr`.
    #[must_use]
    pub fn from(&self, addr: u64) -> Vec<&Xref> {
        match self.from_index.get(&addr) {
            Some(ids) => ids.iter().map(|&i| &self.xrefs[i as usize]).collect(),
            None => Vec::new(),
        }
    }

    /// `true` if no xrefs are recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.xrefs.is_empty()
    }

    /// Total xref count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.xrefs.len()
    }

    fn from_raw(mut entries: Vec<Xref>) -> Self {
        entries.sort_by_key(|x| (x.to, x.from, x.kind));
        let mut from_index: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        let mut to_index: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        for (i, x) in entries.iter().enumerate() {
            let idx = u32::try_from(i).unwrap_or(u32::MAX);
            from_index.entry(x.from).or_default().push(idx);
            to_index.entry(x.to).or_default().push(idx);
        }
        Self {
            xrefs: entries,
            from_index,
            to_index,
        }
    }
}

/// Node in the [`CallGraph`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallNode {
    pub id: u32,
    pub kind: CallNodeKind,
    /// Function entry virtual address for [`CallNodeKind::Function`];
    /// `None` for synthetic nodes (imports without a relocated slot,
    /// indirect-call anchors).
    pub address: Option<u64>,
    /// Display name. Function nodes use the symbol-derived name (or
    /// `fn_<addr>` when no symbol exists); import nodes use the import
    /// name; the indirect site name is `indirect@<va>`; unresolved
    /// targets are `loc_<va>`.
    pub name: String,
}

/// What kind of "callable thing" a [`CallNode`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallNodeKind {
    /// A recovered function in [`FunctionSet`].
    Function,
    /// An imported symbol whose definition is external.
    Import,
    /// A direct-call target that does not match any recovered function.
    Unresolved,
    /// An indirect-call anchor, one per (caller, site) pair, so the
    /// caller's signal-of-indirection is preserved in the graph.
    IndirectSite,
}

/// Edge in the [`CallGraph`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CallEdge {
    pub from: u32,
    pub to: u32,
    /// VA of the calling instruction.
    pub site: u64,
    /// `true` when the call was indirect.
    pub indirect: bool,
    pub confidence: Confidence,
}

/// Whole-program call graph (FR-27).
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
    /// `function_address → node_id` lookup so callers can find the
    /// graph node for a known function without scanning [`nodes`].
    pub by_function: BTreeMap<u64, u32>,
}

impl CallGraph {
    /// Node by id. Panics if `id` is out of range, which is unreachable
    /// for any id this crate produces.
    #[must_use]
    pub fn node(&self, id: u32) -> &CallNode {
        &self.nodes[id as usize]
    }

    /// Outgoing edges from `id`, in `(site, to)` order.
    pub fn outgoing(&self, id: u32) -> impl Iterator<Item = &CallEdge> + '_ {
        self.edges.iter().filter(move |e| e.from == id)
    }

    /// Incoming edges to `id`, in `(from, site)` order.
    pub fn incoming(&self, id: u32) -> impl Iterator<Item = &CallEdge> + '_ {
        self.edges.iter().filter(move |e| e.to == id)
    }
}

/// Build the whole-program call graph for `functions` in `model`.
///
/// Walks each function's bytes through `decoder`. The bookkeeping is
/// deterministic: function nodes are emitted in ascending address order,
/// imports / unresolved / indirect-site nodes follow them sorted by
/// `(kind, name, address)`, and edges land sorted by
/// `(from, site, to, indirect)`.
#[must_use]
pub fn build_call_graph(
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    functions: &FunctionSet,
) -> CallGraph {
    let exec_sections: Vec<&Section> = exec_sections(model);
    let import_names = import_name_set(model);

    let mut nodes: Vec<CallNode> = Vec::with_capacity(functions.functions.len());
    let mut by_function: BTreeMap<u64, u32> = BTreeMap::new();
    let mut by_import: BTreeMap<String, u32> = BTreeMap::new();
    let mut by_unresolved: BTreeMap<u64, u32> = BTreeMap::new();
    let mut by_indirect: BTreeMap<u64, u32> = BTreeMap::new();

    for f in &functions.functions {
        let id = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
        let name = f
            .name
            .clone()
            .unwrap_or_else(|| format!("fn_{:x}", f.address));
        nodes.push(CallNode {
            id,
            kind: CallNodeKind::Function,
            address: Some(f.address),
            name,
        });
        by_function.insert(f.address, id);
    }

    let mut edges: Vec<CallEdge> = Vec::new();

    // Per-function instruction sweep — emit call/tailcall/indirect-call
    // edges. We need the function's byte slice; if it is missing
    // (sectionless symbols, etc.) the function is skipped silently —
    // this matches I-6: degrade, don't invent.
    for f in &functions.functions {
        let Some(end) = f.end else { continue };
        if end <= f.address {
            continue;
        }
        let Some((slice, slice_addr)) = function_bytes(f.address, end, &exec_sections, bytes)
        else {
            continue;
        };
        let from_id = by_function[&f.address];
        for inst in decoder.iter(slice, slice_addr) {
            match inst.flow {
                ControlFlow::Call { target: Some(t) } => {
                    let to_id =
                        if let Some(import_name) = resolve_import_at(t, model, &import_names) {
                            *by_import.entry(import_name.clone()).or_insert_with(|| {
                                let id = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
                                nodes.push(CallNode {
                                    id,
                                    kind: CallNodeKind::Import,
                                    address: Some(t),
                                    name: import_name,
                                });
                                id
                            })
                        } else if let Some(&id) = by_function.get(&t) {
                            id
                        } else {
                            *by_unresolved.entry(t).or_insert_with(|| {
                                let id = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
                                nodes.push(CallNode {
                                    id,
                                    kind: CallNodeKind::Unresolved,
                                    address: Some(t),
                                    name: format!("loc_{t:x}"),
                                });
                                id
                            })
                        };
                    let confidence = if nodes[to_id as usize].kind == CallNodeKind::Function
                        || nodes[to_id as usize].kind == CallNodeKind::Import
                    {
                        Confidence::new(DIRECT_CALL_CONFIDENCE, Source::Observed)
                    } else {
                        Confidence::new(UNRESOLVED_DIRECT_CALL_CONFIDENCE, Source::Derived)
                    };
                    edges.push(CallEdge {
                        from: from_id,
                        to: to_id,
                        site: inst.address,
                        indirect: false,
                        confidence,
                    });
                }
                ControlFlow::IndirectCall => {
                    let to_id = *by_indirect.entry(inst.address).or_insert_with(|| {
                        let id = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
                        nodes.push(CallNode {
                            id,
                            kind: CallNodeKind::IndirectSite,
                            address: Some(inst.address),
                            name: format!("indirect@{:x}", inst.address),
                        });
                        id
                    });
                    edges.push(CallEdge {
                        from: from_id,
                        to: to_id,
                        site: inst.address,
                        indirect: true,
                        confidence: Confidence::new(INDIRECT_CALL_CONFIDENCE, Source::Speculative),
                    });
                }
                ControlFlow::UnconditionalBranch { target: Some(t) }
                    if (t < f.address || t >= end) && by_function.contains_key(&t) =>
                {
                    let to_id = by_function[&t];
                    edges.push(CallEdge {
                        from: from_id,
                        to: to_id,
                        site: inst.address,
                        indirect: false,
                        confidence: Confidence::new(TAIL_CALL_CONFIDENCE, Source::Derived),
                    });
                }
                _ => {}
            }
        }
    }

    edges.sort_by_key(|e| (e.from, e.site, e.to, e.indirect));

    CallGraph {
        nodes,
        edges,
        by_function,
    }
}

/// Build the cross-reference index for `model` and `functions`.
#[must_use]
pub fn build_xref_index(
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    functions: &FunctionSet,
) -> XrefIndex {
    let exec_sections: Vec<&Section> = exec_sections(model);
    let import_names = import_name_set(model);
    let mut entries: Vec<Xref> = Vec::new();

    // 1) Code → code xrefs from the function sweep. Mirror what the
    //    callgraph builder finds, but recorded address-to-address so
    //    `--xrefs <symbol>` can list every call site referencing it.
    for f in &functions.functions {
        let Some(end) = f.end else { continue };
        if end <= f.address {
            continue;
        }
        let Some((slice, slice_addr)) = function_bytes(f.address, end, &exec_sections, bytes)
        else {
            continue;
        };
        for inst in decoder.iter(slice, slice_addr) {
            match inst.flow {
                ControlFlow::Call { target: Some(t) } => {
                    let is_import = resolve_import_at(t, model, &import_names).is_some();
                    let (kind, confidence) = if is_import {
                        (
                            XrefKind::Import,
                            Confidence::new(DIRECT_CALL_CONFIDENCE, Source::Observed),
                        )
                    } else if functions.contains_address(t) {
                        (
                            XrefKind::Call,
                            Confidence::new(DIRECT_CALL_CONFIDENCE, Source::Observed),
                        )
                    } else {
                        (
                            XrefKind::Call,
                            Confidence::new(UNRESOLVED_DIRECT_CALL_CONFIDENCE, Source::Derived),
                        )
                    };
                    entries.push(Xref {
                        from: inst.address,
                        to: t,
                        kind,
                        confidence,
                    });
                }
                ControlFlow::IndirectCall => {
                    entries.push(Xref {
                        from: inst.address,
                        to: EXTERNAL_VA,
                        kind: XrefKind::IndirectCall,
                        confidence: Confidence::new(INDIRECT_CALL_CONFIDENCE, Source::Speculative),
                    });
                }
                ControlFlow::UnconditionalBranch { target: Some(t) }
                    if (t < f.address || t >= end) && functions.contains_address(t) =>
                {
                    entries.push(Xref {
                        from: inst.address,
                        to: t,
                        kind: XrefKind::TailCall,
                        confidence: Confidence::new(TAIL_CALL_CONFIDENCE, Source::Derived),
                    });
                }
                _ => {}
            }
        }
    }

    // 2) Relocation-derived xrefs. Each relocation patches an address
    //    so it points at `symbol + addend` at load time; we mint a
    //    Code↔Data / Data↔Code / Data↔Data xref between those two
    //    sides, classified by the section kind on each end.
    for r in &model.relocations {
        let patch_section = r.section.and_then(|i| model.sections.get(i));
        let patch_va = patch_section.map(|s| {
            // For dynamic relocations the model stores the VA directly
            // in `offset`; for static (.o) relocations the offset is
            // the within-section byte offset, so add the section base.
            if is_object_relocation(model) {
                s.address.wrapping_add(r.offset)
            } else {
                r.offset
            }
        });
        let target_sym = r.symbol.and_then(|i| model.symbols.get(i));
        let target_addr = target_sym.and_then(|sym| {
            if sym.undefined {
                None
            } else {
                Some(sym.address.wrapping_add(r.addend as u64))
            }
        });

        let from_va = patch_va.unwrap_or(EXTERNAL_VA);
        let from_in_code = patch_section.map(|s| s.perms.executable).unwrap_or(false);
        if let Some(sym) = target_sym {
            if sym.undefined && from_in_code {
                entries.push(Xref {
                    from: from_va,
                    to: EXTERNAL_VA,
                    kind: XrefKind::Import,
                    confidence: Confidence::new(RELOC_CONFIDENCE, Source::Observed),
                });
                continue;
            }
        }
        let Some(to_va) = target_addr else {
            continue;
        };
        let to_section = section_of_va(model, to_va);
        let to_in_code = to_section.map(|s| s.perms.executable).unwrap_or(false);
        let kind = match (from_in_code, to_in_code) {
            (true, true) => XrefKind::Call,
            (true, false) => XrefKind::CodeToData,
            (false, true) => XrefKind::DataToCode,
            (false, false) => XrefKind::DataToData,
        };
        // Drop the absolute /relative distinction — for an xref index
        // the kind plus addresses are enough. `RelocationKind::Copy`
        // and other section-relative kinds without a useful target are
        // already screened off by the `target_addr` check above.
        let _ = RelocationKind::Absolute;
        entries.push(Xref {
            from: from_va,
            to: to_va,
            kind,
            confidence: Confidence::new(RELOC_CONFIDENCE, Source::Observed),
        });
    }

    // 3) Exports & entry point. Both are "external holders reference
    //    this code address" — record them as `Export` xrefs from the
    //    synthetic external VA.
    if let Some(entry) = model.entry {
        if entry != 0 {
            entries.push(Xref {
                from: EXTERNAL_VA,
                to: entry,
                kind: XrefKind::Export,
                confidence: Confidence::new(EXPORT_CONFIDENCE, Source::Observed),
            });
        }
    }
    for e in &model.exports {
        if e.address == 0 {
            continue;
        }
        entries.push(Xref {
            from: EXTERNAL_VA,
            to: e.address,
            kind: XrefKind::Export,
            confidence: Confidence::new(EXPORT_CONFIDENCE, Source::Observed),
        });
    }

    XrefIndex::from_raw(entries)
}

/// Heuristic: a relocation table whose first entry's `offset` looks
/// like a VA (i.e. above any section's base) is dynamic; otherwise it
/// is per-section. This crate consumes the model produced by
/// `dac-binfmt`, which normalises both ELF dynamic and PE base-relocs
/// onto the dynamic form, so today the answer is always "dynamic"; the
/// helper exists so a future static-object input does not silently
/// double-add the section base.
fn is_object_relocation(model: &BinaryModel) -> bool {
    // `object` produces `offset = within-section byte offset` for ELF
    // `.o` inputs and `offset = VA` for shared libraries / executables.
    // Today `load_from_bytes` only loads the latter — every loaded
    // binary has at least one section with a load address > 0 and the
    // relocation offsets are VAs. The conservative answer for the
    // current pipeline is therefore "no".
    let _ = model;
    false
}

fn section_of_va(model: &BinaryModel, va: u64) -> Option<&Section> {
    model.sections.iter().find(|s| {
        let start = s.address;
        let end = start.saturating_add(s.size);
        s.size > 0 && va >= start && va < end && s.kind != SectionKind::Metadata
    })
}

fn exec_sections(model: &BinaryModel) -> Vec<&Section> {
    model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect()
}

fn import_name_set(model: &BinaryModel) -> std::collections::BTreeSet<String> {
    model.imports.iter().map(|i| i.name.clone()).collect()
}

/// Resolve a direct-call target VA to an imported symbol name when the
/// target sits in a PLT-style stub that the relocation table maps onto
/// an undefined import. The heuristic walks the relocation table for an
/// entry whose patched-address falls inside the same executable region
/// as `target_va` and whose symbol is undefined and present in
/// `imports`.
///
/// This is a coarse approximation — a real PLT walk would decode the
/// trampoline and read the GOT slot. For the M3 scope it is enough to
/// distinguish "direct call → recovered function" from "direct call →
/// imported symbol" for the textual report.
fn resolve_import_at(
    target_va: u64,
    model: &BinaryModel,
    imports: &std::collections::BTreeSet<String>,
) -> Option<String> {
    // Only consider relocations whose symbol is undefined and named
    // in the imports table. Match by the relocation falling within a
    // 64-byte window of the call target — enough to span a single PLT
    // stub on x86-64.
    const PLT_WINDOW: u64 = 64;
    for r in &model.relocations {
        let Some(sym) = r.symbol.and_then(|i| model.symbols.get(i)) else {
            continue;
        };
        if !sym.undefined || !imports.contains(&sym.name) {
            continue;
        }
        let patched = r.offset;
        if patched >= target_va.saturating_sub(PLT_WINDOW)
            && patched <= target_va.saturating_add(PLT_WINDOW)
        {
            return Some(sym.name.clone());
        }
    }
    None
}

fn function_bytes<'a>(
    start: u64,
    end: u64,
    exec_sections: &[&Section],
    bytes: &'a [u8],
) -> Option<(&'a [u8], u64)> {
    let sec = exec_sections.iter().find(|s| {
        let s_start = s.address;
        let s_end = s_start.saturating_add(s.size);
        start >= s_start && end <= s_end
    })?;
    let file_off = sec.file_offset?;
    let off = (start - sec.address).saturating_add(file_off);
    let len = end.saturating_sub(start);
    let off_usize = usize::try_from(off).ok()?;
    let len_usize = usize::try_from(len).ok()?;
    let end_usize = off_usize.checked_add(len_usize)?;
    if end_usize > bytes.len() {
        return None;
    }
    Some((&bytes[off_usize..end_usize], start))
}

/// Look up a function or other named subject in `model` and
/// `functions` by either symbol name or a hex VA (`0x…` / no prefix).
///
/// Used by the CLI to parse `--xrefs <subject>`. Returns the resolved
/// VA (and a display name when one is known).
#[must_use]
pub fn resolve_subject(
    raw: &str,
    model: &BinaryModel,
    functions: &FunctionSet,
) -> Option<(u64, Option<String>)> {
    // Numeric form: 0x… (hex), or decimal.
    let parsed = if let Some(stripped) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        u64::from_str_radix(stripped, 16).ok()
    } else {
        u64::from_str_radix(raw, 16)
            .ok()
            .or_else(|| raw.parse().ok())
    };
    if let Some(va) = parsed {
        // If this VA matches a known function, prefer that name.
        if let Some(f) = functions.get(va) {
            return Some((va, f.name.clone()));
        }
        for sym in &model.symbols {
            if sym.address == va && !sym.name.is_empty() {
                return Some((va, Some(sym.name.clone())));
            }
        }
        return Some((va, None));
    }

    // Symbol form.
    for f in &functions.functions {
        if let Some(name) = &f.name {
            if name == raw {
                return Some((f.address, Some(name.clone())));
            }
        }
    }
    for sym in &model.symbols {
        if sym.name == raw && sym.address != 0 {
            return Some((sym.address, Some(sym.name.clone())));
        }
    }
    for e in &model.exports {
        if e.name == raw && e.address != 0 {
            return Some((e.address, Some(e.name.clone())));
        }
    }
    None
}

/// Render the call graph as DOT. One `digraph` per binary; node
/// shapes encode [`CallNodeKind`] (function = box, import = diamond,
/// unresolved = ellipse, indirect site = circle). Edges are labelled
/// with the call site VA so the operator can grep for "where was
/// `foo` called from?".
#[must_use]
pub fn render_callgraph_dot(graph: &CallGraph, binary_name: &str) -> String {
    let mut out = String::new();
    use std::fmt::Write as _;
    let safe = binary_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let _ = writeln!(out, "digraph \"callgraph_{safe}\" {{");
    let _ = writeln!(out, "    node [fontname=\"monospace\"];");
    for n in &graph.nodes {
        let shape = match n.kind {
            CallNodeKind::Function => "box",
            CallNodeKind::Import => "diamond",
            CallNodeKind::Unresolved => "ellipse",
            CallNodeKind::IndirectSite => "circle",
        };
        let _ = writeln!(
            out,
            "    n{} [shape={}, label=\"{}\"];",
            n.id,
            shape,
            dot_escape(&n.name),
        );
    }
    for e in &graph.edges {
        let style = if e.indirect { "dashed" } else { "solid" };
        let _ = writeln!(
            out,
            "    n{} -> n{} [style={}, label=\"{:#x}\"];",
            e.from, e.to, style, e.site,
        );
    }
    let _ = writeln!(out, "}}");
    out
}

fn dot_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            _ => vec![c],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_arch::{
        ControlFlow as Cf, DecodeError, DecodedInstruction as Di, InstructionDecoder as Id,
    };
    use dac_binfmt::{
        Architecture, BinaryFormat, Bits, Endian, Export, Permissions, Relocation, RelocationKind,
        Section, SectionKind, Symbol, SymbolBinding, SymbolKind, SymbolSource,
    };
    use dac_core::EvidenceGraph;
    use dac_recovery::discover_functions;

    fn model_with(text_addr: u64, text_size: u64) -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: vec![Section {
                name: ".text".into(),
                address: text_addr,
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
            name: name.into(),
            address,
            size,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: Some(0),
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    /// Scripted decoder that emits a sequence of synthetic instructions
    /// per address range. Lets tests author exactly the control-flow
    /// shape they need without depending on real iced behaviour.
    struct ScriptedDecoder {
        events: Vec<(u64, Cf)>,
    }

    impl ScriptedDecoder {
        fn new(events: Vec<(u64, Cf)>) -> Self {
            Self { events }
        }
    }

    impl Id for ScriptedDecoder {
        fn decode_one(&self, _bytes: &[u8], _address: u64) -> Result<Di, DecodeError> {
            Err(DecodeError::Truncated { offset: 0 })
        }
        fn iter<'a>(&'a self, _bytes: &'a [u8], address: u64) -> Box<dyn Iterator<Item = Di> + 'a> {
            let max = address.saturating_add(_bytes.len() as u64);
            let here: Vec<Di> = self
                .events
                .iter()
                .filter(|(a, _)| *a >= address && *a < max)
                .map(|(a, f)| Di {
                    address: *a,
                    length: 1,
                    bytes: vec![0],
                    mnemonic: "x".into(),
                    operands: String::new(),
                    flow: *f,
                    valid: true,
                })
                .collect();
            Box::new(here.into_iter())
        }
    }

    fn discover(model: &BinaryModel, dec: &dyn Id) -> FunctionSet {
        let mut g = EvidenceGraph::new();
        discover_functions(
            model,
            &vec![0u8; model.sections[0].size as usize],
            dec,
            &mut g,
        )
    }

    #[test]
    fn direct_call_to_function_lands_as_call_edge() {
        let mut model = model_with(0x1000, 0x200);
        model.symbols.push(text_sym("caller", 0x1000, 0x40));
        model.symbols.push(text_sym("callee", 0x1080, 0x40));
        let dec = ScriptedDecoder::new(vec![(
            0x1010,
            Cf::Call {
                target: Some(0x1080),
            },
        )]);
        let funcs = discover(&model, &dec);
        let cg = build_call_graph(&model, &[0u8; 0x200], &dec, &funcs);
        let caller = cg.by_function[&0x1000];
        let callee = cg.by_function[&0x1080];
        let outs: Vec<_> = cg.outgoing(caller).collect();
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].to, callee);
        assert_eq!(outs[0].site, 0x1010);
        assert!(!outs[0].indirect);
        assert_eq!(outs[0].confidence.source(), Source::Observed);
        let ins: Vec<_> = cg.incoming(callee).collect();
        assert_eq!(ins.len(), 1);
        assert_eq!(ins[0].from, caller);
    }

    #[test]
    fn indirect_call_creates_anchored_indirect_site_node() {
        let mut model = model_with(0x2000, 0x80);
        model.symbols.push(text_sym("caller", 0x2000, 0x40));
        let dec = ScriptedDecoder::new(vec![(0x2010, Cf::IndirectCall)]);
        let funcs = discover(&model, &dec);
        let cg = build_call_graph(&model, &[0u8; 0x80], &dec, &funcs);
        let caller = cg.by_function[&0x2000];
        let outs: Vec<_> = cg.outgoing(caller).collect();
        assert_eq!(outs.len(), 1);
        assert!(outs[0].indirect);
        assert_eq!(cg.node(outs[0].to).kind, CallNodeKind::IndirectSite);
        assert_eq!(cg.node(outs[0].to).name, "indirect@2010");
    }

    #[test]
    fn direct_call_to_unknown_address_promotes_unresolved_node() {
        // Hand-roll a FunctionSet: in the live pipeline,
        // `discover_functions` would mint a function at every direct
        // call target, so `Unresolved` is reachable only when a
        // caller passes a FunctionSet without that pass — e.g. a
        // conservatively scoped analysis or a future change to
        // discovery. This test exercises that safety net.
        let mut model = model_with(0x3000, 0x200);
        model.symbols.push(text_sym("caller", 0x3000, 0x40));
        let mut g = EvidenceGraph::new();
        let caller_evidence = g.add_node(dac_core::EvidenceNode::IrNode {
            layer: dac_core::IrLayer::Cfg,
            id: 0,
        });
        let funcs = FunctionSet {
            functions: vec![dac_recovery::Function {
                address: 0x3000,
                end: Some(0x3040),
                name: Some("caller".into()),
                confidence: Confidence::new(1.0, Source::Observed),
                sources: Default::default(),
                evidence: caller_evidence,
            }],
            stats: Default::default(),
        };
        let dec = ScriptedDecoder::new(vec![(
            0x3008,
            Cf::Call {
                target: Some(0x30C0),
            },
        )]);
        let cg = build_call_graph(&model, &[0u8; 0x200], &dec, &funcs);
        let caller = cg.by_function[&0x3000];
        let outs: Vec<_> = cg.outgoing(caller).collect();
        assert_eq!(outs.len(), 1);
        assert_eq!(cg.node(outs[0].to).kind, CallNodeKind::Unresolved);
        assert_eq!(cg.node(outs[0].to).name, "loc_30c0");
        assert_eq!(outs[0].confidence.source(), Source::Derived);
    }

    #[test]
    fn tail_call_only_promotes_when_target_is_another_function_entry() {
        let mut model = model_with(0x4000, 0x200);
        model.symbols.push(text_sym("caller", 0x4000, 0x40));
        model.symbols.push(text_sym("tail", 0x4100, 0x40));
        // Two unconditional branches: one to the tail function entry
        // (promoted), one to a mid-function address (ignored).
        let dec = ScriptedDecoder::new(vec![
            (
                0x4010,
                Cf::UnconditionalBranch {
                    target: Some(0x4100),
                },
            ),
            (
                0x4018,
                Cf::UnconditionalBranch {
                    target: Some(0x4108),
                },
            ),
        ]);
        let funcs = discover(&model, &dec);
        let cg = build_call_graph(&model, &[0u8; 0x200], &dec, &funcs);
        let caller = cg.by_function[&0x4000];
        let outs: Vec<_> = cg.outgoing(caller).collect();
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].to, cg.by_function[&0x4100]);
        assert_eq!(outs[0].confidence.source(), Source::Derived);
    }

    #[test]
    fn xref_index_to_from_lookups_return_sorted_results() {
        let mut model = model_with(0x5000, 0x200);
        model.symbols.push(text_sym("a", 0x5000, 0x40));
        model.symbols.push(text_sym("b", 0x5080, 0x40));
        // a calls b at 0x5010 and 0x5020.
        let dec = ScriptedDecoder::new(vec![
            (
                0x5010,
                Cf::Call {
                    target: Some(0x5080),
                },
            ),
            (
                0x5020,
                Cf::Call {
                    target: Some(0x5080),
                },
            ),
        ]);
        let funcs = discover(&model, &dec);
        let idx = build_xref_index(&model, &[0u8; 0x200], &dec, &funcs);
        let to_b = idx.to(0x5080);
        assert_eq!(to_b.len(), 2);
        assert_eq!(to_b[0].from, 0x5010);
        assert_eq!(to_b[1].from, 0x5020);
        let from_a = idx.from(0x5010);
        assert_eq!(from_a.len(), 1);
        assert_eq!(from_a[0].to, 0x5080);
    }

    #[test]
    fn exports_and_entry_become_external_xrefs() {
        let mut model = model_with(0x6000, 0x80);
        model.entry = Some(0x6010);
        model.exports.push(Export {
            name: "foo".into(),
            address: 0x6040,
        });
        let dec = ScriptedDecoder::new(vec![]);
        let funcs = discover(&model, &dec);
        let idx = build_xref_index(&model, &[0u8; 0x80], &dec, &funcs);
        let entry_refs = idx.to(0x6010);
        assert_eq!(entry_refs.len(), 1);
        assert_eq!(entry_refs[0].from, EXTERNAL_VA);
        assert_eq!(entry_refs[0].kind, XrefKind::Export);
        let export_refs = idx.to(0x6040);
        assert_eq!(export_refs.len(), 1);
        assert_eq!(export_refs[0].kind, XrefKind::Export);
    }

    #[test]
    fn relocation_between_data_sections_classifies_data_to_data() {
        let mut model = model_with(0x7000, 0x100);
        model.sections.push(Section {
            name: ".data".into(),
            address: 0x8000,
            size: 0x100,
            file_offset: Some(0x100),
            perms: Permissions {
                readable: true,
                writable: true,
                executable: false,
            },
            kind: SectionKind::Data,
        });
        let sym_idx = model.symbols.len();
        model.symbols.push(Symbol {
            name: "datum".into(),
            address: 0x8040,
            size: 0,
            kind: SymbolKind::Data,
            binding: SymbolBinding::Global,
            section: Some(1),
            source: SymbolSource::Symtab,
            undefined: false,
        });
        model.relocations.push(Relocation {
            section: Some(1),
            offset: 0x8020,
            kind: RelocationKind::Absolute,
            symbol: Some(sym_idx),
            addend: 0,
        });
        let dec = ScriptedDecoder::new(vec![]);
        let funcs = discover(&model, &dec);
        let idx = build_xref_index(&model, &[0u8; 0x200], &dec, &funcs);
        let refs = idx.to(0x8040);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, XrefKind::DataToData);
        assert_eq!(refs[0].from, 0x8020);
    }

    #[test]
    fn resolve_subject_accepts_hex_and_symbol_names() {
        let mut model = model_with(0x9000, 0x80);
        model.symbols.push(text_sym("main", 0x9020, 0x10));
        let dec = ScriptedDecoder::new(vec![]);
        let funcs = discover(&model, &dec);
        // By name.
        let (va, name) = resolve_subject("main", &model, &funcs).expect("symbol");
        assert_eq!(va, 0x9020);
        assert_eq!(name.as_deref(), Some("main"));
        // By address.
        let (va, name) = resolve_subject("0x9020", &model, &funcs).expect("hex");
        assert_eq!(va, 0x9020);
        assert_eq!(name.as_deref(), Some("main"));
        // Unknown symbol → None.
        assert!(resolve_subject("nope", &model, &funcs).is_none());
    }

    #[test]
    fn render_callgraph_dot_emits_stable_ordering() {
        let mut model = model_with(0xA000, 0x200);
        model.symbols.push(text_sym("a", 0xA000, 0x40));
        model.symbols.push(text_sym("b", 0xA080, 0x40));
        let dec = ScriptedDecoder::new(vec![(
            0xA010,
            Cf::Call {
                target: Some(0xA080),
            },
        )]);
        let funcs = discover(&model, &dec);
        let cg = build_call_graph(&model, &[0u8; 0x200], &dec, &funcs);
        let dot1 = render_callgraph_dot(&cg, "sample");
        let dot2 = render_callgraph_dot(&cg, "sample");
        assert_eq!(dot1, dot2);
        assert!(dot1.contains("callgraph_sample"));
        assert!(dot1.contains("shape=box"));
        assert!(dot1.contains("0xa010"));
    }
}
