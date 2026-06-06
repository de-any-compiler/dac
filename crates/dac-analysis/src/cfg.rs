//! Control-flow graph construction (B2.1, FR-10).
//!
//! Given a [`Function`] recovered by `dac-recovery` and the raw bytes of
//! the binary, [`build_cfg`] reconstructs a per-function CFG:
//!
//! - Basic blocks split at branch targets, the instruction after every
//!   non-sequential terminator, and the function entry.
//! - Edges classified by [`EdgeKind`] so downstream passes (dominators,
//!   structuring) can distinguish a conditional's taken and not-taken
//!   sides without re-deriving them.
//! - Entry and exit blocks recorded explicitly; unreachable blocks listed.
//!
//! The builder is purely deterministic — it reads through the existing
//! [`InstructionDecoder`] trait so no architecture knowledge leaks into
//! this crate. The CFG itself records nothing it cannot prove from the
//! decoder's [`ControlFlow`] classification, so per I-6 the output never
//! invents an edge: if the target of a direct branch is unknown or out
//! of function range, no edge is minted, and the block is exposed as a
//! CFG exit instead.
//!
//! The DOT renderer at [`render_dot`] / [`render_dot_all`] is the
//! `--emit-cfg` (FR-28) backend. The format is one `digraph` per
//! function, sorted by function address, with stable node ids
//! (`BB<index>`) and a fixed attribute order, so the rendered output is
//! byte-stable across re-runs.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;

use dac_arch::{ControlFlow, DecodedInstruction, InstructionDecoder};
use dac_binfmt::{BinaryModel, Section};
use dac_core::EvidenceId;
use dac_recovery::Function;

/// Numeric handle for a [`BasicBlock`] inside a [`Cfg`]. Block ids are
/// dense indices into [`Cfg::blocks`] in ascending-address order, so
/// `cfg.blocks[id as usize].id == id` holds for every well-formed CFG.
pub type BlockId = u32;

/// A per-function control-flow graph.
///
/// The CFG describes the function as a directed graph of [`BasicBlock`]
/// nodes connected by [`Edge`] arcs. The [`entry`](Self::entry) block is
/// the one starting at [`function_address`](Self::function_address); the
/// [`exits`](Self::exits) list every block whose terminator produces no
/// outgoing edge inside the function (returns, indirect branches,
/// interrupts, invalid bytes, or branches that leave the function
/// range). Any block not reachable from the entry by following
/// [`edges`](Self::edges) is listed in
/// [`unreachable`](Self::unreachable).
#[derive(Debug, Clone)]
pub struct Cfg {
    /// Function entry virtual address.
    pub function_address: u64,
    /// Exclusive end virtual address of the function's byte span.
    pub function_end: u64,
    /// Function name when known (symbol-derived); `None` otherwise. Used
    /// to name the DOT `digraph` and to label the entry block.
    pub function_name: Option<String>,
    /// Basic blocks in ascending address order. The block id is the
    /// vector index, so [`Edge::from`] and [`Edge::to`] index into this
    /// vector directly.
    pub blocks: Vec<BasicBlock>,
    /// Entry block id — always the block whose
    /// [`address`](BasicBlock::address) equals
    /// [`function_address`](Self::function_address).
    pub entry: BlockId,
    /// Block ids with no outgoing edge inside the function — ascending.
    pub exits: Vec<BlockId>,
    /// Edges, sorted by `(from, kind, to)` for deterministic output.
    pub edges: Vec<Edge>,
    /// Block ids unreachable from [`entry`](Self::entry), ascending.
    pub unreachable: Vec<BlockId>,
    /// Evidence-graph handle for the function. Inherited from
    /// [`Function::evidence`] so callers can attach further facts (e.g.
    /// dominator results) to the same node without re-minting it.
    pub evidence: EvidenceId,
}

impl Cfg {
    /// Look up a block by id. Panics if `id` is out of range, which is
    /// not reachable through any public API on this crate — block ids
    /// are always allocated by the builder.
    #[must_use]
    pub fn block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id as usize]
    }

    /// Successors of `block` as block ids. Computed by scanning
    /// [`edges`](Self::edges); cheap for small CFGs and avoids a parallel
    /// index for the typical decompilation workload.
    pub fn successors(&self, block: BlockId) -> impl Iterator<Item = BlockId> + '_ {
        self.edges
            .iter()
            .filter(move |e| e.from == block)
            .map(|e| e.to)
    }
}

/// One basic block.
///
/// A block is a maximal run of decoded instructions inside which control
/// flow is straight-line: every instruction except the last is
/// [`ControlFlow::Sequential`], and the last instruction's
/// [`Terminator`] classification determines the block's outgoing edges.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block id — also the index in [`Cfg::blocks`].
    pub id: BlockId,
    /// VA of the first instruction in the block.
    pub address: u64,
    /// Exclusive end VA. Equals either the next leader's address or
    /// [`Cfg::function_end`].
    pub end: u64,
    /// Decoded instructions in address order. May be empty when a leader
    /// landed on an address the linear sweep could not decode (e.g.
    /// post-`Invalid` resync); the block still exists so reachability is
    /// honest about it.
    pub instructions: Vec<DecodedInstruction>,
    /// Terminator classification — the categorical reason this block
    /// ended. Drives the edge-minting logic in [`build_cfg`].
    pub terminator: Terminator,
}

/// Why a basic block ends. Derived from the last instruction's
/// [`ControlFlow`] (or [`Terminator::Fall`] when the block ended only
/// because the next address is a leader for some other reason).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminator {
    /// No terminator instruction — the block ended because the next
    /// address is a leader for some other reason (typically a branch
    /// target). Always produces a fall-through edge to the next block.
    Fall,
    /// Unconditional direct branch.
    Branch {
        /// Target VA as supplied by the decoder, or `None` when the
        /// decoder could not resolve it. The address is preserved even
        /// when out-of-function so call-graph passes (B3.1) can detect
        /// tail calls; the CFG itself only mints an edge when a
        /// block starts at this address inside the function.
        target: Option<u64>,
    },
    /// Conditional direct branch.
    Conditional {
        /// Taken-side target VA as supplied by the decoder. The
        /// not-taken side always falls through to the next block; the
        /// taken side gets a CFG edge only when a block starts at this
        /// address inside the function.
        target: Option<u64>,
    },
    /// Indirect unconditional branch (`jmp rax`, `jmp [rax]`, …). No
    /// edges minted — the block is an exit.
    Indirect,
    /// Direct or indirect call. Falls through intra-procedurally; the
    /// callee target is recorded for later passes (call-graph at B3.1)
    /// but does not become a CFG successor.
    Call {
        /// Resolved callee VA when the call is direct, else `None`.
        target: Option<u64>,
    },
    /// Procedure return.
    Return,
    /// Interrupt / trap / syscall. Conservatively treated as a CFG exit
    /// — some syscalls do not return, and the structuring pass can
    /// refine if needed.
    Interrupt,
    /// Decoder produced an invalid instruction. No edges minted.
    Invalid,
}

/// One directed CFG edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub from: BlockId,
    pub to: BlockId,
    pub kind: EdgeKind,
}

/// Classification of a CFG edge.
///
/// The kind is what tells structuring (B2.7) whether an edge is the
/// taken or not-taken side of a conditional, or an unconditional
/// fall-through after a call. Downstream consumers should pattern-match
/// the closed enum; new variants land alongside new terminator kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Unconditional fall-through. Produced by [`Terminator::Fall`] and
    /// [`Terminator::Call`].
    Fall,
    /// Unconditional branch ([`Terminator::Branch`] with a resolved
    /// in-function target).
    Branch,
    /// Taken side of a conditional branch.
    Taken,
    /// Not-taken side of a conditional branch — falls through to the
    /// next block.
    NotTaken,
}

impl EdgeKind {
    /// Stable discriminant for sort keys. Kept separate from the enum
    /// definition so adding a variant does not silently re-order
    /// previously written DOT output.
    fn sort_key(self) -> u8 {
        match self {
            Self::Fall => 0,
            Self::Branch => 1,
            Self::Taken => 2,
            Self::NotTaken => 3,
        }
    }

    /// Short label used in DOT output.
    fn dot_label(self) -> &'static str {
        match self {
            Self::Fall => "fall",
            Self::Branch => "jmp",
            Self::Taken => "T",
            Self::NotTaken => "F",
        }
    }
}

/// Build a CFG for `function` from the decoded bytes of the binary.
///
/// Returns `None` when the function's byte span cannot be resolved —
/// either [`Function::end`] is `None`, the function range is empty, or
/// the executable section that should contain it is missing or
/// truncated. The builder never panics on garbage input; an
/// undecodable byte range turns into an empty block with
/// [`Terminator::Invalid`] (NFR-4, I-6).
///
/// The CFG inherits the function's evidence handle so subsequent passes
/// can attach facts (dominators, types, hints, …) to the same node
/// without re-minting it.
#[must_use]
pub fn build_cfg(
    function: &Function,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
) -> Option<Cfg> {
    let end = function.end?;
    if end <= function.address {
        return None;
    }

    let exec_sections: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();

    let (slice, slice_addr) = function_bytes(function.address, end, &exec_sections, bytes)?;
    let instructions: Vec<DecodedInstruction> = decoder.iter(slice, slice_addr).collect();
    if instructions.is_empty() {
        return None;
    }

    let addr_to_idx: BTreeMap<u64, usize> = instructions
        .iter()
        .enumerate()
        .map(|(i, ins)| (ins.address, i))
        .collect();

    let mut leaders: BTreeSet<u64> = BTreeSet::new();
    leaders.insert(function.address);

    for (i, ins) in instructions.iter().enumerate() {
        let next_addr = instructions.get(i + 1).map(|n| n.address);
        match ins.flow {
            ControlFlow::Sequential => {}
            ControlFlow::ConditionalBranch { target }
            | ControlFlow::UnconditionalBranch { target } => {
                if let Some(t) = target {
                    if (function.address..end).contains(&t) && addr_to_idx.contains_key(&t) {
                        leaders.insert(t);
                    }
                }
                if let Some(na) = next_addr {
                    leaders.insert(na);
                }
            }
            ControlFlow::IndirectBranch
            | ControlFlow::Call { .. }
            | ControlFlow::IndirectCall
            | ControlFlow::Return
            | ControlFlow::Interrupt
            | ControlFlow::Invalid => {
                if let Some(na) = next_addr {
                    leaders.insert(na);
                }
            }
        }
    }

    let leader_vec: Vec<u64> = leaders.into_iter().collect();
    let mut blocks: Vec<BasicBlock> = Vec::with_capacity(leader_vec.len());

    for (i, &leader_addr) in leader_vec.iter().enumerate() {
        let next_leader = leader_vec.get(i + 1).copied().unwrap_or(end);
        let block_id = i as BlockId;

        let mut block_instrs = Vec::new();
        if let Some(&start_idx) = addr_to_idx.get(&leader_addr) {
            let mut idx = start_idx;
            while idx < instructions.len() && instructions[idx].address < next_leader {
                block_instrs.push(instructions[idx].clone());
                idx += 1;
            }
        }

        let terminator = match block_instrs.last() {
            Some(last) => classify_terminator(last),
            None => Terminator::Invalid,
        };

        blocks.push(BasicBlock {
            id: block_id,
            address: leader_addr,
            end: next_leader,
            instructions: block_instrs,
            terminator,
        });
    }

    let addr_to_block: BTreeMap<u64, BlockId> = blocks.iter().map(|b| (b.address, b.id)).collect();

    let mut edges: Vec<Edge> = Vec::new();
    for block in &blocks {
        let next_block = next_block_id(&blocks, block.id);
        match block.terminator {
            Terminator::Fall => {
                if let Some(to) = next_block {
                    edges.push(Edge {
                        from: block.id,
                        to,
                        kind: EdgeKind::Fall,
                    });
                }
            }
            Terminator::Branch { target } => {
                if let Some(t) = target {
                    if let Some(&to) = addr_to_block.get(&t) {
                        edges.push(Edge {
                            from: block.id,
                            to,
                            kind: EdgeKind::Branch,
                        });
                    }
                }
            }
            Terminator::Conditional { target } => {
                if let Some(t) = target {
                    if let Some(&to) = addr_to_block.get(&t) {
                        edges.push(Edge {
                            from: block.id,
                            to,
                            kind: EdgeKind::Taken,
                        });
                    }
                }
                if let Some(to) = next_block {
                    edges.push(Edge {
                        from: block.id,
                        to,
                        kind: EdgeKind::NotTaken,
                    });
                }
            }
            Terminator::Call { .. } => {
                if let Some(to) = next_block {
                    edges.push(Edge {
                        from: block.id,
                        to,
                        kind: EdgeKind::Fall,
                    });
                }
            }
            Terminator::Indirect
            | Terminator::Return
            | Terminator::Interrupt
            | Terminator::Invalid => {}
        }
    }

    edges.sort_by_key(|e| (e.from, e.kind.sort_key(), e.to));

    let entry_id = addr_to_block.get(&function.address).copied()?;

    let mut successors: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for e in &edges {
        successors.entry(e.from).or_default().push(e.to);
    }

    let mut reachable: BTreeSet<BlockId> = BTreeSet::new();
    reachable.insert(entry_id);
    let mut queue: VecDeque<BlockId> = VecDeque::from([entry_id]);
    while let Some(b) = queue.pop_front() {
        if let Some(succs) = successors.get(&b) {
            for &s in succs {
                if reachable.insert(s) {
                    queue.push_back(s);
                }
            }
        }
    }

    let unreachable: Vec<BlockId> = blocks
        .iter()
        .map(|b| b.id)
        .filter(|id| !reachable.contains(id))
        .collect();

    let has_succ: BTreeSet<BlockId> = edges.iter().map(|e| e.from).collect();
    let exits: Vec<BlockId> = blocks
        .iter()
        .map(|b| b.id)
        .filter(|id| !has_succ.contains(id))
        .collect();

    Some(Cfg {
        function_address: function.address,
        function_end: end,
        function_name: function.name.clone(),
        blocks,
        entry: entry_id,
        exits,
        edges,
        unreachable,
        evidence: function.evidence,
    })
}

/// Build CFGs for every function in `functions` for which a CFG can be
/// constructed. Functions whose byte range cannot be resolved (no end,
/// truncated section, …) are skipped silently; the returned vector is
/// in ascending function-address order, matching the input.
#[must_use]
pub fn build_cfgs(
    functions: &[Function],
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
) -> Vec<Cfg> {
    let mut cfgs = Vec::with_capacity(functions.len());
    for f in functions {
        if let Some(cfg) = build_cfg(f, model, bytes, decoder) {
            cfgs.push(cfg);
        }
    }
    cfgs
}

fn classify_terminator(ins: &DecodedInstruction) -> Terminator {
    match ins.flow {
        ControlFlow::Sequential => Terminator::Fall,
        ControlFlow::ConditionalBranch { target } => Terminator::Conditional { target },
        ControlFlow::UnconditionalBranch { target } => Terminator::Branch { target },
        ControlFlow::IndirectBranch => Terminator::Indirect,
        ControlFlow::Call { target } => Terminator::Call { target },
        ControlFlow::IndirectCall => Terminator::Call { target: None },
        ControlFlow::Return => Terminator::Return,
        ControlFlow::Interrupt => Terminator::Interrupt,
        ControlFlow::Invalid => Terminator::Invalid,
    }
}

fn next_block_id(blocks: &[BasicBlock], id: BlockId) -> Option<BlockId> {
    let next = id.checked_add(1)?;
    if (next as usize) < blocks.len() {
        Some(next)
    } else {
        None
    }
}

fn function_bytes<'a>(
    address: u64,
    end: u64,
    exec_sections: &[&Section],
    bytes: &'a [u8],
) -> Option<(&'a [u8], u64)> {
    let sec = exec_sections.iter().find(|s| {
        let start = s.address;
        let send = start.saturating_add(s.size);
        address >= start && address < send
    })?;
    let sec_end = sec.address.saturating_add(sec.size);
    let real_end = end.min(sec_end);
    if real_end <= address {
        return None;
    }
    let offset_in_sec = address - sec.address;
    let length = real_end - address;
    let file_off = usize::try_from(sec.file_offset?).ok()?;
    let off_in_sec = usize::try_from(offset_in_sec).ok()?;
    let len = usize::try_from(length).ok()?;
    let start_off = file_off.checked_add(off_in_sec)?;
    let end_off = start_off.checked_add(len)?;
    if end_off > bytes.len() {
        return None;
    }
    Some((&bytes[start_off..end_off], address))
}

/// Render `cfg` as a single Graphviz `digraph`.
///
/// The graph name is derived from the function name (sanitized to a DOT
/// identifier) and the function address, so two functions with the same
/// name still produce distinct graph names. Block nodes are `BB<id>`,
/// labelled with their address and decoded instructions; edges carry a
/// short label (`fall` / `jmp` / `T` / `F`). The entry block is
/// highlighted; unreachable blocks are drawn dashed.
///
/// Output is byte-stable: blocks and edges are emitted in the order they
/// appear in [`Cfg::blocks`] / [`Cfg::edges`], which the builder sorts
/// canonically.
#[must_use]
pub fn render_dot(cfg: &Cfg) -> String {
    let mut s = String::new();
    let name = dot_graph_name(cfg);
    writeln!(s, "digraph \"{name}\" {{").unwrap();
    writeln!(s, "    rankdir=TB;").unwrap();
    writeln!(s, "    node [shape=box, fontname=\"monospace\"];").unwrap();
    for block in &cfg.blocks {
        let label = block_label(block);
        let escaped = escape_dot_label(&label);
        let attrs = if block.id == cfg.entry {
            ", style=filled, fillcolor=\"#e0e0e0\""
        } else if cfg.unreachable.binary_search(&block.id).is_ok() {
            ", style=dashed, color=\"#808080\""
        } else {
            ""
        };
        writeln!(s, "    BB{} [label=\"{escaped}\"{attrs}];", block.id).unwrap();
    }
    for edge in &cfg.edges {
        writeln!(
            s,
            "    BB{} -> BB{} [label=\"{}\"];",
            edge.from,
            edge.to,
            edge.kind.dot_label()
        )
        .unwrap();
    }
    writeln!(s, "}}").unwrap();
    s
}

/// Render every CFG in `cfgs` as one DOT file, sorted by function
/// address. Each `digraph` block is separated by a blank line so the
/// output can be split on `\n\n` by ad-hoc tooling.
#[must_use]
pub fn render_dot_all(cfgs: &[Cfg]) -> String {
    let mut order: Vec<&Cfg> = cfgs.iter().collect();
    order.sort_by_key(|c| c.function_address);
    let mut out = String::new();
    for (i, cfg) in order.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&render_dot(cfg));
    }
    out
}

fn block_label(block: &BasicBlock) -> String {
    let mut s = String::new();
    writeln!(s, "BB{} @ {:#x}", block.id, block.address).unwrap();
    for ins in &block.instructions {
        if ins.operands.is_empty() {
            writeln!(s, "{:#x}: {}", ins.address, ins.mnemonic).unwrap();
        } else {
            writeln!(s, "{:#x}: {} {}", ins.address, ins.mnemonic, ins.operands).unwrap();
        }
    }
    if block.instructions.is_empty() {
        writeln!(s, "(no decoded instructions)").unwrap();
    }
    s
}

fn dot_graph_name(cfg: &Cfg) -> String {
    let mut out = String::new();
    out.push_str("fn_");
    if let Some(name) = cfg.function_name.as_deref() {
        if !name.is_empty() {
            for c in name.chars() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    out.push(c);
                } else {
                    out.push('_');
                }
            }
            out.push('_');
        }
    }
    write!(out, "{:x}", cfg.function_address).unwrap();
    out
}

fn escape_dot_label(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\l"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_binfmt::{Architecture, BinaryFormat, Bits, Endian, Permissions, Section, SectionKind};
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode, IrLayer, Source};
    use dac_recovery::SourceMask;

    /// Build a `DecodedInstruction` with the supplied address, length,
    /// mnemonic, and control flow. Bytes/operand strings are placeholders
    /// — every CFG decision in the builder reads `flow`, not the text.
    fn ins(address: u64, length: u32, mnemonic: &str, flow: ControlFlow) -> DecodedInstruction {
        DecodedInstruction {
            address,
            length: length as usize,
            bytes: vec![0u8; length as usize],
            mnemonic: mnemonic.to_string(),
            operands: String::new(),
            flow,
            valid: !matches!(flow, ControlFlow::Invalid),
        }
    }

    /// Decoder that yields the recorded instructions whose address falls
    /// inside the slice range. Used to drive [`build_cfg`] in tests
    /// without depending on a real ISA decoder.
    struct FakeDecoder {
        instrs: Vec<DecodedInstruction>,
    }

    impl InstructionDecoder for FakeDecoder {
        fn decode_one(
            &self,
            _bytes: &[u8],
            address: u64,
        ) -> Result<DecodedInstruction, dac_arch::DecodeError> {
            self.instrs
                .iter()
                .find(|i| i.address == address)
                .cloned()
                .ok_or(dac_arch::DecodeError::Truncated { offset: 0 })
        }

        fn iter<'a>(
            &'a self,
            bytes: &'a [u8],
            address: u64,
        ) -> Box<dyn Iterator<Item = DecodedInstruction> + 'a> {
            let end = address + bytes.len() as u64;
            let it = self
                .instrs
                .iter()
                .filter(move |i| i.address >= address && i.address < end)
                .cloned();
            Box::new(it)
        }
    }

    fn model_with_text(text_address: u64, text_size: u64) -> BinaryModel {
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

    fn function(name: &str, address: u64, end: u64) -> (Function, EvidenceGraph) {
        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        let f = Function {
            address,
            end: Some(end),
            name: if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
            confidence: Confidence::new(1.0, Source::Observed),
            sources: SourceMask::SYMBOL,
            evidence: ev,
            kind: Default::default(),
        };
        (f, g)
    }

    fn run(
        function_addr: u64,
        function_end: u64,
        name: &str,
        text_size: u64,
        instrs: Vec<DecodedInstruction>,
    ) -> Cfg {
        let model = model_with_text(function_addr, text_size);
        let bytes = vec![0u8; text_size as usize];
        let decoder = FakeDecoder { instrs };
        let (f, _g) = function(name, function_addr, function_end);
        build_cfg(&f, &model, &bytes, &decoder).expect("CFG build")
    }

    #[test]
    fn case_01_single_return_block() {
        // One instruction: ret. One block, no edges.
        let cfg = run(
            0x1000,
            0x1001,
            "ret_only",
            0x10,
            vec![ins(0x1000, 1, "ret", ControlFlow::Return)],
        );
        assert_eq!(cfg.blocks.len(), 1);
        assert_eq!(cfg.entry, 0);
        assert_eq!(cfg.exits, vec![0]);
        assert!(cfg.edges.is_empty());
        assert!(cfg.unreachable.is_empty());
        assert_eq!(cfg.blocks[0].terminator, Terminator::Return);
    }

    #[test]
    fn case_02_linear_function_is_one_block() {
        // mov; mov; ret. All sequential except the ret. One block.
        let cfg = run(
            0x1000,
            0x1006,
            "linear",
            0x10,
            vec![
                ins(0x1000, 2, "mov", ControlFlow::Sequential),
                ins(0x1002, 2, "mov", ControlFlow::Sequential),
                ins(0x1004, 2, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 1);
        assert_eq!(cfg.blocks[0].instructions.len(), 3);
        assert!(cfg.edges.is_empty());
        assert_eq!(cfg.exits, vec![0]);
    }

    #[test]
    fn case_03_conditional_branch_creates_diamond() {
        // 0x1000: test
        // 0x1002: jz 0x1008 (conditional)
        // 0x1004: mov  (not-taken side)
        // 0x1006: jmp 0x100a
        // 0x1008: mov  (taken side)
        // 0x100a: ret
        let cfg = run(
            0x1000,
            0x100b,
            "diamond",
            0x20,
            vec![
                ins(0x1000, 2, "test", ControlFlow::Sequential),
                ins(
                    0x1002,
                    2,
                    "jz",
                    ControlFlow::ConditionalBranch {
                        target: Some(0x1008),
                    },
                ),
                ins(0x1004, 2, "mov", ControlFlow::Sequential),
                ins(
                    0x1006,
                    2,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(0x100a),
                    },
                ),
                ins(0x1008, 2, "mov", ControlFlow::Sequential),
                ins(0x100a, 1, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 4, "test/jz | mov/jmp | mov | ret");
        // Block 0 ends with the conditional.
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::Conditional {
                target: Some(0x1008)
            }
        ));
        // Two edges out of block 0: Taken + NotTaken.
        let out0: Vec<_> = cfg.edges.iter().filter(|e| e.from == 0).collect();
        assert_eq!(out0.len(), 2);
        assert!(out0.iter().any(|e| e.kind == EdgeKind::Taken));
        assert!(out0.iter().any(|e| e.kind == EdgeKind::NotTaken));
        // The unconditional branch in block 1 also reaches block 3 (ret).
        assert!(matches!(
            cfg.blocks[1].terminator,
            Terminator::Branch {
                target: Some(0x100a)
            }
        ));
        assert!(cfg.edges.iter().any(|e| e.from == 1
            && cfg.blocks[e.to as usize].address == 0x100a
            && e.kind == EdgeKind::Branch));
        // Every block is reachable.
        assert!(cfg.unreachable.is_empty());
    }

    #[test]
    fn case_04_unconditional_branch_skips_unreachable_block() {
        // 0x1000: jmp 0x1006
        // 0x1003: mov   <-- orphan, no in-edge
        // 0x1006: ret
        let cfg = run(
            0x1000,
            0x1007,
            "orphan",
            0x10,
            vec![
                ins(
                    0x1000,
                    3,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(0x1006),
                    },
                ),
                ins(0x1003, 3, "mov", ControlFlow::Sequential),
                ins(0x1006, 1, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 3);
        // The middle block has no predecessor.
        let orphan_id = cfg.blocks.iter().find(|b| b.address == 0x1003).unwrap().id;
        assert!(cfg.unreachable.contains(&orphan_id));
    }

    #[test]
    fn case_05_back_edge_forms_a_loop() {
        // Tight loop:
        // 0x1000: mov
        // 0x1002: jmp 0x1000   (back-edge)
        let cfg = run(
            0x1000,
            0x1004,
            "spin",
            0x10,
            vec![
                ins(0x1000, 2, "mov", ControlFlow::Sequential),
                ins(
                    0x1002,
                    2,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(0x1000),
                    },
                ),
            ],
        );
        assert_eq!(cfg.blocks.len(), 1);
        // The single block has a self-edge.
        let e = &cfg.edges[0];
        assert_eq!(e.from, 0);
        assert_eq!(e.to, 0);
        assert_eq!(e.kind, EdgeKind::Branch);
        // The block has a successor (itself), so it's not an exit.
        assert!(cfg.exits.is_empty());
    }

    #[test]
    fn case_06_branch_out_of_function_is_a_tail_exit() {
        // 0x1000: jmp 0x9000  (out of range — tail call / external jump)
        let cfg = run(
            0x1000,
            0x1003,
            "tail",
            0x10,
            vec![ins(
                0x1000,
                3,
                "jmp",
                ControlFlow::UnconditionalBranch {
                    target: Some(0x9000),
                },
            )],
        );
        assert_eq!(cfg.blocks.len(), 1);
        // The terminator carries the address even though the target is
        // out-of-range, so call-graph passes can see the tail jump.
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::Branch {
                target: Some(0x9000)
            }
        ));
        // No edge minted because the target is outside the function.
        assert!(cfg.edges.is_empty());
        assert_eq!(cfg.exits, vec![0]);
    }

    #[test]
    fn case_07_call_falls_through_to_next_block() {
        // 0x1000: call 0x9000
        // 0x1005: ret
        let cfg = run(
            0x1000,
            0x1006,
            "after_call",
            0x10,
            vec![
                ins(
                    0x1000,
                    5,
                    "call",
                    ControlFlow::Call {
                        target: Some(0x9000),
                    },
                ),
                ins(0x1005, 1, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 2);
        let e = &cfg.edges[0];
        assert_eq!(e.from, 0);
        assert_eq!(e.to, 1);
        assert_eq!(e.kind, EdgeKind::Fall);
        // Callee address is preserved on the terminator for the call
        // graph pass.
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::Call {
                target: Some(0x9000)
            }
        ));
    }

    #[test]
    fn case_08_indirect_branch_has_no_outgoing_edges() {
        // 0x1000: jmp rax
        let cfg = run(
            0x1000,
            0x1002,
            "indirect",
            0x10,
            vec![ins(0x1000, 2, "jmp", ControlFlow::IndirectBranch)],
        );
        assert_eq!(cfg.blocks.len(), 1);
        assert!(cfg.edges.is_empty());
        assert_eq!(cfg.exits, vec![0]);
        assert_eq!(cfg.blocks[0].terminator, Terminator::Indirect);
    }

    #[test]
    fn case_09_conditional_with_out_of_range_target_keeps_fall_through() {
        // 0x1000: jz 0x9000  (taken side is out-of-range)
        // 0x1002: ret        (not-taken side falls through here)
        let cfg = run(
            0x1000,
            0x1003,
            "skip",
            0x10,
            vec![
                ins(
                    0x1000,
                    2,
                    "jz",
                    ControlFlow::ConditionalBranch {
                        target: Some(0x9000),
                    },
                ),
                ins(0x1002, 1, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 2);
        // Only the NotTaken edge survives — Taken would have left the
        // function range.
        let from0: Vec<_> = cfg.edges.iter().filter(|e| e.from == 0).collect();
        assert_eq!(from0.len(), 1);
        assert_eq!(from0[0].kind, EdgeKind::NotTaken);
    }

    #[test]
    fn case_10_dot_is_byte_stable_for_a_diamond() {
        // Same diamond as case 3, but we check the DOT output. Stability
        // and a few structural markers: the digraph name, the entry
        // styling, every edge appears exactly once.
        let cfg = run(
            0x1000,
            0x100b,
            "diamond",
            0x20,
            vec![
                ins(0x1000, 2, "test", ControlFlow::Sequential),
                ins(
                    0x1002,
                    2,
                    "jz",
                    ControlFlow::ConditionalBranch {
                        target: Some(0x1008),
                    },
                ),
                ins(0x1004, 2, "mov", ControlFlow::Sequential),
                ins(
                    0x1006,
                    2,
                    "jmp",
                    ControlFlow::UnconditionalBranch {
                        target: Some(0x100a),
                    },
                ),
                ins(0x1008, 2, "mov", ControlFlow::Sequential),
                ins(0x100a, 1, "ret", ControlFlow::Return),
            ],
        );
        let dot1 = render_dot(&cfg);
        let dot2 = render_dot(&cfg);
        assert_eq!(dot1, dot2, "DOT output must be byte-stable");
        assert!(dot1.contains("digraph \"fn_diamond_1000\""));
        // The entry block is filled.
        assert!(dot1.contains("fillcolor=\"#e0e0e0\""));
        // Edge labels render.
        assert!(dot1.contains("label=\"T\""));
        assert!(dot1.contains("label=\"F\""));
        assert!(dot1.contains("label=\"jmp\""));
    }

    #[test]
    fn case_11_unresolved_conditional_target_emits_only_not_taken_edge() {
        // 0x1000: jz [rax]   (indirect-like conditional with no
        //                     decoder-supplied target)
        // 0x1002: ret
        let cfg = run(
            0x1000,
            0x1003,
            "guess",
            0x10,
            vec![
                ins(
                    0x1000,
                    2,
                    "jz",
                    ControlFlow::ConditionalBranch { target: None },
                ),
                ins(0x1002, 1, "ret", ControlFlow::Return),
            ],
        );
        assert_eq!(cfg.blocks.len(), 2);
        let from0: Vec<_> = cfg.edges.iter().filter(|e| e.from == 0).collect();
        assert_eq!(from0.len(), 1);
        assert_eq!(from0[0].kind, EdgeKind::NotTaken);
    }

    #[test]
    fn case_12_render_dot_all_sorts_by_address() {
        // Two synthetic CFGs; render_dot_all should emit the lower-address
        // function first regardless of input order.
        let a = run(
            0x2000,
            0x2001,
            "a",
            0x10,
            vec![ins(0x2000, 1, "ret", ControlFlow::Return)],
        );
        let b = run(
            0x1000,
            0x1001,
            "b",
            0x10,
            vec![ins(0x1000, 1, "ret", ControlFlow::Return)],
        );
        let combined = render_dot_all(&[a, b]);
        let lower = combined.find("fn_b_1000").expect("b at 0x1000");
        let upper = combined.find("fn_a_2000").expect("a at 0x2000");
        assert!(lower < upper, "lower address should render first");
    }

    #[test]
    fn dot_label_escaping_handles_quotes_and_backslashes() {
        // Direct unit check on the escape function — the renderer relies
        // on it for correctness when a mnemonic or operand string carries
        // backslashes or quotes.
        assert_eq!(escape_dot_label("a\"b"), "a\\\"b");
        assert_eq!(escape_dot_label("a\\b"), "a\\\\b");
        assert_eq!(escape_dot_label("a\nb"), "a\\lb");
    }

    #[test]
    fn target_not_on_instruction_boundary_does_not_mint_an_edge() {
        // 0x1000: jz 0x1003   (target is in-range but the decoder
        //                      never produced an instruction at 0x1003;
        //                      we conservatively decline to invent a
        //                      block there — I-6.)
        let cfg = run(
            0x1000,
            0x1005,
            "gap",
            0x10,
            vec![ins(
                0x1000,
                2,
                "jz",
                ControlFlow::ConditionalBranch {
                    target: Some(0x1003),
                },
            )],
        );
        assert_eq!(cfg.blocks.len(), 1);
        // The terminator preserves the decoder-supplied address even
        // though no block was minted at the target — call-graph passes
        // can still see it.
        assert!(matches!(
            cfg.blocks[0].terminator,
            Terminator::Conditional {
                target: Some(0x1003)
            }
        ));
        // No edges minted at all (no NotTaken either: only one block,
        // and it has no successor).
        assert!(cfg.edges.is_empty());
    }
}
