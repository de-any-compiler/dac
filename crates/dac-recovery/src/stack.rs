//! Stack-frame recovery (B2.4, FR-12).
//!
//! Reconstructs the layout of a function's stack frame from its
//! SSA form, identifying every memory location addressed as
//! `entry_sp + k` (a *stack local*, a stack-passed *incoming
//! argument*, or — on Windows — the *shadow space*) and folding the
//! frame pointer (if any) back onto the same anchor.
//!
//! ## What the pass actually does
//!
//! The SSA construction in [`dac_analysis::ssa`] mints a `Parameter`
//! value for every variable that is read without first being
//! written; for the architecture's stack pointer (`rsp` on x86-64)
//! that parameter is *the* `entry_sp` — the anchor for the whole
//! frame. The pass then:
//!
//! 1. Resolves the stack pointer's parameter value and seeds an
//!    [`Offset`] map with `entry_sp -> 0`.
//! 2. Walks instructions, propagating `(anchor, offset)` through
//!    `Move`, `Add`, and `Sub` whose other operand is a constant.
//!    A short fix-point loop catches values whose definitions are
//!    visited after their phi-resolved consumers (the analyzer does
//!    not require a topological sort by id).
//! 3. Folds phi nodes: when every incoming edge resolves to the
//!    same offset, the phi destination inherits it. This handles
//!    loop bodies that reload `sp + k` on the back edge.
//! 4. Picks up the *frame pointer* alias: the first instruction
//!    whose destination variable matches the convention's
//!    nominated FP register (e.g. `rbp`) and whose resolved offset
//!    is known. Accesses through that variable inherit the same
//!    anchor with the right offset; no separate mechanism is
//!    needed.
//! 5. Collects every `Load` and `Store` whose address resolves to
//!    `(entry_sp + k)` into a [`StackLocal`] keyed by `k`. Widths
//!    accumulate as the maximum observed access width; access
//!    counts accumulate as the total number of read+write sites.
//! 6. Classifies each offset against the
//!    [`StackConvention`]-specific layout into [`StackLocalKind`].
//!
//! What the pass *does not* do:
//!
//! - **Cluster offsets into composite locals.** A struct local
//!   touched at `+0`, `+4`, `+8` lands as three independent locals
//!   here; struct synthesis is B3.2.
//! - **Resolve aliasing between two stack accesses.** Memory-SSA
//!   is a later concern; the pass only records *that* a slot was
//!   touched.
//! - **Recover incoming-argument types.** Convention inference
//!   (B2.5) consumes [`StackFrame::locals`] to do this.
//! - **Propagate offsets through `And` / `Or` / `Shl`**. Alignment
//!   masks (`and rsp, -16`) are common in real prologues but
//!   require either an alignment hint or symbolic interval
//!   tracking; conservatively, the pass treats the result as
//!   unknown rather than guessing (I-6).
//!
//! ## Conventions modelled
//!
//! Both x86-64 conventions agree on the immediate frame at function
//! entry: `[entry_sp + 0]` is the return address, the *negative*
//! half is the callee's local frame, the *positive* half is the
//! caller's frame. They differ on the positive half's layout:
//!
//! | Convention                  | `[entry_sp + 0]` | `[entry_sp + 8 .. 40)`         | `[entry_sp + 40+]` |
//! | --------------------------- | ---------------- | ------------------------------ | ------------------ |
//! | [`StackConvention::SysVAmd64`] | ret address      | stack-passed args (7th, 8th, …) | stack-passed args  |
//! | [`StackConvention::MsX64`]   | ret address      | home space (RCX/RDX/R8/R9)      | stack args (5th+)  |
//!
//! Both place callee locals at negative offsets. The frame pointer
//! register is `rbp` for both conventions when present.
//!
//! ## Determinism (NFR-9)
//!
//! The pass is [`dac_core::Source::Derived`]-class and
//! deterministic. Iteration order is by ascending block id and
//! instruction index, and the output [`StackFrame::locals`] is a
//! [`BTreeMap`] keyed by offset.

use std::collections::BTreeMap;

use dac_core::{Confidence, Source};
use dac_ir::ssa::{Operand, SsaFunction, SsaOp, ValueId, ValueSource, VariableId};

/// Calling-convention-driven stack layout.
///
/// The convention determines two things: which architectural
/// register the analyzer treats as the stack pointer (always `rsp`
/// for the conventions modelled today), and how positive offsets
/// from `entry_sp` are classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackConvention {
    /// SysV AMD64 ABI (Linux, macOS, BSD on x86-64).
    SysVAmd64,
    /// Microsoft x64 ABI (Windows on x86-64).
    MsX64,
}

impl StackConvention {
    /// Canonical lowercased name of the stack-pointer register for
    /// this convention.
    #[must_use]
    pub const fn stack_pointer_name(self) -> &'static str {
        match self {
            StackConvention::SysVAmd64 | StackConvention::MsX64 => "rsp",
        }
    }

    /// Canonical lowercased name of the frame-pointer register for
    /// this convention. Both x86-64 conventions nominate `rbp`; this
    /// returns `Some` even when the function omits the frame
    /// pointer (the analyzer simply never sees an FP-anchoring
    /// `Move`).
    #[must_use]
    pub const fn frame_pointer_name(self) -> Option<&'static str> {
        match self {
            StackConvention::SysVAmd64 | StackConvention::MsX64 => Some("rbp"),
        }
    }

    /// Highest positive offset (from `entry_sp`) classified as
    /// home/shadow space on this convention. Args lie at `offset >=
    /// shadow_end`; anything strictly below this and above `0` is
    /// home/shadow space (when present).
    const fn shadow_end(self) -> i64 {
        match self {
            StackConvention::SysVAmd64 => 8, // only the return-address slot
            StackConvention::MsX64 => 40,    // ret addr + 4 home slots
        }
    }
}

/// Recovered stack frame for a function.
///
/// All offsets are *signed*, measured from the value of the stack
/// pointer at function entry. The negative half is the callee's
/// local frame; the positive half is the caller's.
///
/// `Eq` is intentionally not derived: [`Confidence`] holds an `f32`
/// and only implements [`PartialEq`]. Callers comparing frames in
/// tests use `==`, which suffices for byte-stable bit-equal floats.
#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    /// Convention used to classify offsets.
    pub convention: StackConvention,
    /// Stack-pointer variable resolved by name lookup. `None` when
    /// the SSA function never references the architecture's stack
    /// pointer — the frame degenerates to empty.
    pub stack_pointer: Option<VariableId>,
    /// Frame-pointer variable, when a `mov fp, sp + k` was
    /// recognized in the prologue. `None` when the function omits
    /// the frame pointer.
    pub frame_pointer: Option<FramePointer>,
    /// Recovered locations on the stack, keyed by signed offset
    /// from `entry_sp`. Iterating yields offsets in ascending order.
    pub locals: BTreeMap<i64, StackLocal>,
    /// Confidence the analyzer assigns to the frame as a whole. The
    /// source is always [`Source::Derived`]; the value is
    /// `0.9` when the stack pointer was located, `0.0` otherwise.
    pub confidence: Confidence,
}

/// Recovered frame-pointer alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramePointer {
    /// Variable serving as the frame pointer.
    pub variable: VariableId,
    /// The constant `k` from `mov fp, sp + k`. Negative on SysV
    /// (`rbp = rsp - 8` after a notional `push rbp`), zero or
    /// positive otherwise.
    pub offset: i64,
}

/// One recovered stack location.
///
/// A *location* is a (offset, kind) pair. The pass collapses
/// multiple accesses at the same offset — distinguished by access
/// width — into a single entry whose `width` is the widest observed.
/// Struct synthesis (B3.2) is responsible for clustering adjacent
/// offsets into composite locals. `Eq` is intentionally omitted for
/// the same reason as [`StackFrame`] — [`Confidence`] is f32-typed.
#[derive(Debug, Clone, PartialEq)]
pub struct StackLocal {
    /// Signed offset from `entry_sp`. Negative for callee locals.
    pub offset: i64,
    /// Widest access width observed at this offset, in bytes.
    pub width: u8,
    /// Convention-specific classification.
    pub kind: StackLocalKind,
    /// Number of (read + write) accesses observed at this offset.
    pub access_count: u32,
    /// Confidence the analyzer assigns to *this* location.
    pub confidence: Confidence,
}

/// What kind of slot the offset falls into for the convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackLocalKind {
    /// Callee local (`offset < 0`).
    Local,
    /// Return address slot (`offset == 0`).
    ReturnAddress,
    /// Shadow / home space (Windows x64 only, `0 < offset < 40`).
    ShadowSpace,
    /// Stack-passed incoming argument from the caller. SysV: any
    /// 8-byte-aligned offset above the return address. MsX64: any
    /// 8-byte-aligned offset at or above 40.
    IncomingArgument,
    /// An offset that fell outside any known slot for the
    /// convention. Retained rather than dropped so the reviewer can
    /// see what the pass touched.
    Unclassified,
}

/// Run stack-frame recovery on `ssa`.
///
/// Returns an empty-but-well-formed [`StackFrame`] when the SSA
/// function does not reference the architecture's stack pointer —
/// the analyzer never raises an error (a missing stack pointer is a
/// legitimate state for synthetic functions and leaf no-ops).
#[must_use]
pub fn analyze_stack_frame(ssa: &SsaFunction, convention: StackConvention) -> StackFrame {
    let sp_var = lookup_variable(ssa, convention.stack_pointer_name());
    let fp_var = convention
        .frame_pointer_name()
        .and_then(|name| lookup_variable(ssa, name));

    let Some(sp_var) = sp_var else {
        return StackFrame {
            convention,
            stack_pointer: None,
            frame_pointer: None,
            locals: BTreeMap::new(),
            confidence: Confidence::new(0.0, Source::Derived),
        };
    };

    let sp_entry = parameter_value(ssa, sp_var);

    // ValueId -> offset relative to entry_sp. The parameter value
    // seeds the table at offset 0; everything else is derived.
    let mut value_offset: BTreeMap<ValueId, i64> = BTreeMap::new();
    if let Some(sp_entry) = sp_entry {
        value_offset.insert(sp_entry, 0);
    }

    // Propagate offsets through Move / Add / Sub to a fixed point.
    // SSA defs precede uses in dominator order, but block ids do
    // not always respect that order, so we iterate.
    let mut changed = true;
    while changed {
        changed = false;
        for block in &ssa.blocks {
            for ins in &block.instructions {
                let Some(d) = ins.dst else { continue };
                if value_offset.contains_key(&d) {
                    continue;
                }
                if let Some(off) = resolve_op_offset(&ins.op, &value_offset) {
                    value_offset.insert(d, off);
                    changed = true;
                }
            }
            for phi in &block.phis {
                if value_offset.contains_key(&phi.dst) {
                    continue;
                }
                if let Some(off) = resolve_phi_offset(&phi.incoming, &value_offset) {
                    value_offset.insert(phi.dst, off);
                    changed = true;
                }
            }
        }
    }

    // Locate the frame pointer: the first `Move`/`Add`/`Sub` whose
    // destination is the FP-named variable and whose resolved offset
    // is known. Walking in block-id, then instruction-index order
    // mirrors how a real prologue lays out the move.
    let mut frame_pointer: Option<FramePointer> = None;
    if let Some(fp_var) = fp_var {
        'outer: for block in &ssa.blocks {
            for ins in &block.instructions {
                let Some(d) = ins.dst else { continue };
                if ssa.value(d).variable != fp_var {
                    continue;
                }
                if let Some(&off) = value_offset.get(&d) {
                    frame_pointer = Some(FramePointer {
                        variable: fp_var,
                        offset: off,
                    });
                    break 'outer;
                }
            }
        }
    }

    // Collect Load/Store accesses whose address resolved to a known
    // offset.
    let mut accesses: BTreeMap<i64, AccessAccumulator> = BTreeMap::new();
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let (addr, width) = match &ins.op {
                SsaOp::Load { address, width } => (*address, *width),
                SsaOp::Store { address, width, .. } => (*address, *width),
                _ => continue,
            };
            let Some(off) = operand_offset(addr, &value_offset) else {
                continue;
            };
            let entry = accesses.entry(off).or_default();
            entry.access_count += 1;
            entry.width = entry.width.max(width);
        }
    }

    let local_conf = Confidence::new(0.85, Source::Derived);
    let mut locals = BTreeMap::new();
    for (off, info) in accesses {
        locals.insert(
            off,
            StackLocal {
                offset: off,
                width: info.width,
                kind: classify_offset(convention, off),
                access_count: info.access_count,
                confidence: local_conf,
            },
        );
    }

    StackFrame {
        convention,
        stack_pointer: Some(sp_var),
        frame_pointer,
        locals,
        confidence: Confidence::new(if sp_entry.is_some() { 0.9 } else { 0.0 }, Source::Derived),
    }
}

#[derive(Default)]
struct AccessAccumulator {
    width: u8,
    access_count: u32,
}

fn lookup_variable(ssa: &SsaFunction, name: &str) -> Option<VariableId> {
    ssa.variables
        .iter()
        .find(|v| v.name.eq_ignore_ascii_case(name))
        .map(|v| v.id)
}

fn parameter_value(ssa: &SsaFunction, var: VariableId) -> Option<ValueId> {
    ssa.values.iter().find_map(|val| match val.source {
        ValueSource::Parameter { variable } if variable == var => Some(val.id),
        _ => None,
    })
}

fn operand_offset(opnd: Operand, value_offset: &BTreeMap<ValueId, i64>) -> Option<i64> {
    match opnd {
        Operand::Value(v) => value_offset.get(&v).copied(),
        Operand::Const(_) | Operand::Undef => None,
    }
}

fn const_of(opnd: Operand) -> Option<i64> {
    match opnd {
        Operand::Const(c) => Some(c),
        _ => None,
    }
}

/// Resolve the offset (relative to `entry_sp`) of the value produced
/// by `op`, if any. Returns `None` for non-arithmetic ops and for
/// arithmetic ops whose operands do not pin down an offset.
fn resolve_op_offset(op: &SsaOp, value_offset: &BTreeMap<ValueId, i64>) -> Option<i64> {
    match op {
        SsaOp::Move { src } => operand_offset(*src, value_offset),
        SsaOp::Add { lhs, rhs } => {
            let l_off = operand_offset(*lhs, value_offset);
            let r_off = operand_offset(*rhs, value_offset);
            let l_const = const_of(*lhs);
            let r_const = const_of(*rhs);
            match (l_off, r_off, l_const, r_const) {
                (Some(a), _, _, Some(c)) => a.checked_add(c),
                (_, Some(a), Some(c), _) => a.checked_add(c),
                _ => None,
            }
        }
        SsaOp::Sub { lhs, rhs } => {
            // Only `(anchor + k_lhs) - k_rhs` resolves. `k_lhs - anchor`
            // would invert the anchor and is treated as unknown.
            let l_off = operand_offset(*lhs, value_offset);
            let r_const = const_of(*rhs);
            match (l_off, r_const) {
                (Some(a), Some(c)) => a.checked_sub(c),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Phi-resolution: every incoming entry must resolve to the same
/// offset.
fn resolve_phi_offset(
    incoming: &[(u32, Operand)],
    value_offset: &BTreeMap<ValueId, i64>,
) -> Option<i64> {
    let mut shared: Option<i64> = None;
    for &(_, opnd) in incoming {
        let off = operand_offset(opnd, value_offset)?;
        match shared {
            None => shared = Some(off),
            Some(s) if s == off => {}
            Some(_) => return None,
        }
    }
    shared
}

fn classify_offset(convention: StackConvention, offset: i64) -> StackLocalKind {
    if offset < 0 {
        return StackLocalKind::Local;
    }
    if offset == 0 {
        return StackLocalKind::ReturnAddress;
    }
    let shadow_end = convention.shadow_end();
    if offset < shadow_end {
        return match convention {
            StackConvention::SysVAmd64 => {
                // SysV has no shadow space; positive offsets below
                // `shadow_end` would only land if the lifter accessed
                // bytes inside the return-address slot.
                StackLocalKind::Unclassified
            }
            StackConvention::MsX64 => StackLocalKind::ShadowSpace,
        };
    }
    if offset % 8 == 0 {
        StackLocalKind::IncomingArgument
    } else {
        StackLocalKind::Unclassified
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, EdgeKind, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::Variable;

    use super::*;

    // --- helpers ------------------------------------------------

    fn edge_kind_key(k: EdgeKind) -> u8 {
        match k {
            EdgeKind::Fall => 0,
            EdgeKind::Branch => 1,
            EdgeKind::Taken => 2,
            EdgeKind::NotTaken => 3,
        }
    }

    fn synthetic_cfg(n: usize, entry: u32, raw_edges: &[(u32, u32, EdgeKind)]) -> Cfg {
        let blocks: Vec<BasicBlock> = (0..n)
            .map(|i| BasicBlock {
                id: i as u32,
                address: 0x1000 + 0x10 * i as u64,
                end: 0x1000 + 0x10 * (i + 1) as u64,
                instructions: Vec::new(),
                terminator: Terminator::Fall,
            })
            .collect();
        let mut edges: Vec<Edge> = raw_edges
            .iter()
            .map(|&(from, to, kind)| Edge { from, to, kind })
            .collect();
        edges.sort_by_key(|e| (e.from, edge_kind_key(e.kind), e.to));

        let has_succ: BTreeSet<u32> = edges.iter().map(|e| e.from).collect();
        let exits: Vec<u32> = (0..n as u32).filter(|id| !has_succ.contains(id)).collect();

        let mut reachable: BTreeSet<u32> = BTreeSet::new();
        reachable.insert(entry);
        let mut queue: VecDeque<u32> = VecDeque::from([entry]);
        while let Some(b) = queue.pop_front() {
            for e in &edges {
                if e.from == b && reachable.insert(e.to) {
                    queue.push_back(e.to);
                }
            }
        }
        let unreachable: Vec<u32> = (0..n as u32).filter(|id| !reachable.contains(id)).collect();

        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });

        Cfg {
            function_address: 0x1000,
            function_end: 0x1000 + 0x10 * n as u64,
            function_name: None,
            blocks,
            entry,
            exits,
            edges,
            unreachable,
            evidence: ev,
        }
    }

    fn var(id: VariableId, name: &str) -> Variable {
        Variable {
            id,
            name: name.to_string(),
            width_bits: 64,
        }
    }

    fn sub(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Sub {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn add(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn mov(dst: VariableId, src: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Variable(src),
            },
        }
    }

    fn store(addr: VariableId, value: VariableId, width: u8) -> RawOp {
        RawOp {
            dst: None,
            kind: RawOpKind::Store {
                address: RawOperand::Variable(addr),
                value: RawOperand::Variable(value),
                width,
            },
        }
    }

    fn load(dst: VariableId, addr: VariableId, width: u8) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Load {
                address: RawOperand::Variable(addr),
                width,
            },
        }
    }

    fn build(raw: RawFunction, n_blocks: usize, edges: &[(u32, u32, EdgeKind)]) -> SsaFunction {
        let cfg = synthetic_cfg(n_blocks, 0, edges);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    // --- SysV x86-64 patterns -----------------------------------

    /// SysV with frame pointer omitted: a single `sub rsp, N`, then
    /// stores into `[rsp + k]`, then `add rsp, N` and return.
    #[test]
    fn sysv_no_fp_locals_at_rsp_plus_k_resolve_to_negative_offsets() {
        // variables: 0 = rsp, 1 = rdi, 2 = rsi, 3 = addr, 4 = v
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "addr"),
                var(4, "v"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // rsp = rsp - 16
                    sub(0, 0, 16),
                    // [rsp + 0] = rdi   -> entry_sp - 16
                    store(0, 1, 8),
                    // addr = rsp + 8
                    add(3, 0, 8),
                    // [addr] = rsi      -> entry_sp - 8
                    store(3, 2, 8),
                    // v = [rsp + 0]     -> entry_sp - 16
                    load(4, 0, 8),
                    // rsp = rsp + 16
                    add(0, 0, 16),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);

        assert_eq!(frame.convention, StackConvention::SysVAmd64);
        assert_eq!(frame.stack_pointer, Some(0));
        assert!(frame.frame_pointer.is_none());

        // Locals at -16 (touched twice: store + load) and -8.
        let l16 = frame.locals.get(&-16).expect("local at -16");
        assert_eq!(l16.kind, StackLocalKind::Local);
        assert_eq!(l16.width, 8);
        assert_eq!(l16.access_count, 2);

        let l8 = frame.locals.get(&-8).expect("local at -8");
        assert_eq!(l8.kind, StackLocalKind::Local);
        assert_eq!(l8.access_count, 1);

        // No other locals.
        assert_eq!(frame.locals.len(), 2);
    }

    /// SysV with frame pointer: `push rbp; mov rbp, rsp; sub rsp, N`
    /// modelled as `rsp -= 8; rbp = rsp; rsp -= 16`. Accesses through
    /// `rbp - k` resolve onto the same anchor.
    #[test]
    fn sysv_with_fp_resolves_rbp_minus_offsets() {
        // variables: 0 = rsp, 1 = rbp, 2 = rdi, 3 = addr, 4 = v
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rbp"),
                var(2, "rdi"),
                var(3, "addr"),
                var(4, "v"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // push rbp: rsp = rsp - 8
                    sub(0, 0, 8),
                    // mov rbp, rsp -> rbp at entry_sp - 8
                    mov(1, 0),
                    // sub rsp, 16
                    sub(0, 0, 16),
                    // addr = rbp - 8
                    sub(3, 1, 8),
                    // [addr] = rdi  -> entry_sp - 16
                    store(3, 2, 8),
                    // addr = rbp - 16
                    sub(3, 1, 16),
                    // [addr] = rdi  -> entry_sp - 24
                    store(3, 2, 8),
                    // v = [rbp - 8] -> entry_sp - 16
                    sub(3, 1, 8),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);

        let fp = frame.frame_pointer.expect("frame pointer recognized");
        assert_eq!(fp.variable, 1);
        assert_eq!(fp.offset, -8);

        let l16 = frame.locals.get(&-16).expect("local at -16");
        assert_eq!(l16.access_count, 2); // store + load
        let l24 = frame.locals.get(&-24).expect("local at -24");
        assert_eq!(l24.access_count, 1);
        assert_eq!(frame.locals.len(), 2);
    }

    /// SysV incoming stack args: `[rbp + 16]` after the standard
    /// prologue lands at `entry_sp + 8` (return address is at +0).
    #[test]
    fn sysv_classifies_positive_offsets_as_incoming_arguments() {
        // variables: 0 = rsp, 1 = rbp, 2 = addr, 3 = v
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rbp"), var(2, "addr"), var(3, "v")],
            blocks: vec![RawBlock {
                ops: vec![
                    sub(0, 0, 8),
                    mov(1, 0),
                    sub(0, 0, 16),
                    // addr = rbp + 16 -> entry_sp + 8 (first stack arg)
                    add(2, 1, 16),
                    load(3, 2, 8),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);

        let arg = frame.locals.get(&8).expect("incoming arg at +8 recognized");
        assert_eq!(arg.kind, StackLocalKind::IncomingArgument);
        assert_eq!(arg.width, 8);
    }

    // --- Microsoft x64 (Win64) patterns -------------------------

    /// Win64 callee saves register args into its home space.
    /// `[rsp + 8]` and `[rsp + 16]` at function entry are home-space
    /// slots; classify them as [`StackLocalKind::ShadowSpace`].
    #[test]
    fn ms_x64_classifies_home_space_writes() {
        // variables: 0 = rsp, 1 = rcx, 2 = rdx, 3 = addr
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rcx"), var(2, "rdx"), var(3, "addr")],
            blocks: vec![RawBlock {
                ops: vec![
                    // mov [rsp + 8], rcx
                    add(3, 0, 8),
                    store(3, 1, 8),
                    // mov [rsp + 16], rdx
                    add(3, 0, 16),
                    store(3, 2, 8),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);

        let s8 = frame.locals.get(&8).expect("home slot at +8");
        let s16 = frame.locals.get(&16).expect("home slot at +16");
        assert_eq!(s8.kind, StackLocalKind::ShadowSpace);
        assert_eq!(s16.kind, StackLocalKind::ShadowSpace);
    }

    /// Win64 callee reserves a stack frame, spills args into local
    /// slots, then loads them back. Locals land at negative offsets.
    #[test]
    fn ms_x64_locals_below_reserved_frame() {
        // variables: 0 = rsp, 1 = rcx, 2 = addr, 3 = v
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rcx"), var(2, "addr"), var(3, "v")],
            blocks: vec![RawBlock {
                ops: vec![
                    // sub rsp, 40   ; 32 shadow + 8 alignment
                    sub(0, 0, 40),
                    // [rsp + 0] = rcx  -> entry_sp - 40
                    store(0, 1, 8),
                    // v = [rsp + 0]    -> entry_sp - 40
                    load(3, 0, 8),
                    // add rsp, 40
                    add(0, 0, 40),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);

        let local = frame.locals.get(&-40).expect("local at -40");
        assert_eq!(local.kind, StackLocalKind::Local);
        assert_eq!(local.access_count, 2);
    }

    /// Win64 incoming stack arg: at function entry, the 5th
    /// integer arg sits at `[rsp + 40]` (8 ret addr + 32 shadow).
    #[test]
    fn ms_x64_classifies_fifth_arg_at_plus_40() {
        // variables: 0 = rsp, 1 = addr, 2 = v
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "addr"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![add(1, 0, 40), load(2, 1, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);

        let arg = frame
            .locals
            .get(&40)
            .expect("fifth incoming arg at +40 recognized");
        assert_eq!(arg.kind, StackLocalKind::IncomingArgument);
    }

    // --- Cross-cutting behavior ---------------------------------

    /// Access width accumulates as the maximum observed across
    /// load/store widths at the same offset.
    #[test]
    fn widest_access_width_wins_at_each_offset() {
        // variables: 0 = rsp, 1 = v8, 2 = v4
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "v8"), var(2, "v4")],
            blocks: vec![RawBlock {
                ops: vec![sub(0, 0, 16), load(1, 0, 4), load(2, 0, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let local = frame.locals.get(&-16).unwrap();
        assert_eq!(local.width, 8);
        assert_eq!(local.access_count, 2);
    }

    /// Offsets propagated through a loop-header phi: the phi
    /// destination resolves when every incoming side resolves to the
    /// same offset.
    #[test]
    fn phi_propagates_offset_when_all_incoming_agree() {
        // b0: rsp = rsp - 16; goto b1
        // b1: phi rsp from b0 and b2; if cond goto b2 else b3
        // b2: addr = rsp + 0; store [addr]; goto b1   (rsp unchanged)
        // b3: rsp = rsp + 16; return
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "cond"), var(2, "addr"), var(3, "val")],
            blocks: vec![
                RawBlock {
                    ops: vec![sub(0, 0, 16)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(1),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![add(2, 0, 0), store(2, 3, 8)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![add(0, 0, 16)],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let ssa = build(
            raw,
            4,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::NotTaken),
                (2, 1, EdgeKind::Fall),
            ],
        );
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let local = frame
            .locals
            .get(&-16)
            .expect("loop-body store resolves through phi");
        assert_eq!(local.kind, StackLocalKind::Local);
    }

    /// Missing stack pointer variable degrades gracefully — no
    /// locals, zero confidence.
    #[test]
    fn no_stack_pointer_yields_empty_frame() {
        let raw = RawFunction {
            variables: vec![var(0, "x")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(0)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        assert!(frame.stack_pointer.is_none());
        assert!(frame.locals.is_empty());
        assert_eq!(frame.confidence.value(), 0.0);
        assert_eq!(frame.confidence.source(), Source::Derived);
    }

    /// Analyzer is deterministic across runs (NFR-9).
    #[test]
    fn analyze_is_deterministic_across_runs() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![sub(0, 0, 32), store(0, 1, 8), load(2, 0, 8), add(0, 0, 32)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame1 = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let frame2 = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        assert_eq!(frame1, frame2);
    }

    /// SysV: a store inside the return-address slot (between 0 and
    /// 8, exclusive) is `Unclassified` rather than misidentified as
    /// an argument.
    #[test]
    fn sysv_offset_inside_return_address_slot_is_unclassified() {
        // Synthesize an access at +4 (mid-return-address); the
        // analyzer should not promote it to IncomingArgument.
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "addr"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![add(1, 0, 4), load(2, 1, 4)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let acc = frame.locals.get(&4).unwrap();
        assert_eq!(acc.kind, StackLocalKind::Unclassified);
    }

    /// MsX64: an unaligned offset in the home-space window is
    /// `Unclassified`.
    #[test]
    fn ms_x64_unaligned_arg_offset_is_unclassified() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "addr"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![add(1, 0, 41), load(2, 1, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);
        let acc = frame.locals.get(&41).unwrap();
        assert_eq!(acc.kind, StackLocalKind::Unclassified);
    }

    /// Confidence sources: `frame.confidence.source()` is always
    /// [`Source::Derived`]; per-local confidence is also Derived.
    #[test]
    fn all_confidences_are_derived() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![sub(0, 0, 16), store(0, 1, 8)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        assert_eq!(frame.confidence.source(), Source::Derived);
        for local in frame.locals.values() {
            assert_eq!(local.confidence.source(), Source::Derived);
        }
    }
}
