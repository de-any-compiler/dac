//! B3.26 — pre-emit SSA simplifier (FR-21, NFR-9).
//!
//! [`simplify`] runs after [`crate::types::propagate_types`] and before
//! the structurer / C backend. It collapses the three classes of noise
//! that dominate the post-SSA listing on small fixtures:
//!
//! 1. **Trivial Move folding.** `Move { src: Const(c) | Value(s) }`
//!    defs are inlined at every use of their `dst`, then dropped.
//! 2. **Constant folding.** `(Const a) op (Const b)` rewrites in place
//!    to `Move { src: Const(a op b) }`, which the next round inlines.
//! 3. **Identity folding.** `(x op x)` patterns reduce to `Move`s:
//!    - `Sub`, `Xor` → `Move { Const(0) }`
//!    - `And`, `Or`  → `Move { src: x }`
//! 4. **Dead-pure elimination.** After the rewrite reaches a fixed
//!    point, any pure instruction whose `dst` is not transitively
//!    reachable from a side-effect (Store, Call, Opaque, Load) or a
//!    terminator operand is removed from its block, and the
//!    `ValueSource::Instruction.index` of every surviving instruction
//!    is reindexed to its new position. Dead phis are pruned the same
//!    way.
//!
//! ## What "pure" means
//!
//! Pure ops: [`SsaOp::Move`], [`SsaOp::Add`], [`SsaOp::Sub`],
//! [`SsaOp::Mul`], [`SsaOp::And`], [`SsaOp::Or`], [`SsaOp::Xor`],
//! [`SsaOp::Shl`], [`SsaOp::Shr`], [`SsaOp::Neg`], [`SsaOp::Not`],
//! [`SsaOp::Compare`].
//!
//! Loads are *not* treated as pure — a load from a volatile / MMIO
//! address would change observable behaviour if elided. Calls and
//! opaque ops always have side effects (I-6 conservatism). Stores are
//! always side-effectful.
//!
//! ## Determinism (NFR-9)
//!
//! The pass is a pure function of its input. All substitution maps are
//! [`BTreeMap`]s; all set iteration is in `ValueId` ascending order.
//! Block / instruction traversal is index order. The fixed-point loop
//! is bounded — each iteration either grows the substitution map or
//! exits, and the map is bounded by the SSA value count.
//!
//! ## What deliberately doesn't land yet
//!
//! - Cross-block code motion. The pass is local: it doesn't sink loads
//!   or hoist invariants.
//! - Algebraic identities beyond `(x op x)` (e.g. `x + 0`, `x * 1`).
//!   Adding those is mechanical but each one is a small risk; B3.26
//!   keeps the scope tight to the patterns the hello-x86_64 corpus
//!   actually produces.

use std::collections::{BTreeMap, BTreeSet};

use dac_ir::ssa::{
    Operand, Phi, SsaFunction, SsaInstruction, SsaOp, SsaTerminator, ValueId, ValueSource,
};

/// Per-function simplifier statistics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SimplifyStats {
    /// `Move` defs whose `dst` was folded into uses across the function.
    pub moves_folded: u32,
    /// `(Const a) op (Const b)` rewrites that produced a literal.
    pub constants_folded: u32,
    /// `(x op x)` identity rewrites to a `Move`.
    pub identities_folded: u32,
    /// Pure instructions (including phis) whose `dst` was dead and
    /// therefore removed from the function.
    pub dead_pure_dropped: u32,
}

impl SimplifyStats {
    /// Total rewrites + drops the pass performed. Useful for reports.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.moves_folded + self.constants_folded + self.identities_folded + self.dead_pure_dropped
    }
}

/// Simplify the SSA function in place. Returns per-function statistics.
///
/// Safe to call after any of the recovery side-table passes
/// ([`crate::convention::infer_calling_convention`],
/// [`crate::types::propagate_types`], …). The pass mutates only
/// instruction / phi operands, terminator operands, and the
/// `ValueSource::Instruction.index` / `ValueSource::Phi.index` of
/// surviving definitions — `ssa.values[id].id` and `.variable` are
/// preserved so the C backend's per-value `value_name` /
/// `width_ctype` lookups remain valid.
#[must_use]
pub fn simplify(ssa: &mut SsaFunction) -> SimplifyStats {
    let mut stats = SimplifyStats::default();

    // Fixed-point: each round canonicalises trivial folds in place,
    // applies the resulting Move-driven remap to every use site, then
    // drops the now-redundant Move definitions so the next round sees
    // strictly fewer foldable defs. The loop terminates because each
    // iteration either removes ≥1 Move from a block or exits — the
    // Move count is a strict decreasing measure bounded by the SSA
    // value count.
    loop {
        canonicalize_in_place(ssa, &mut stats);
        let remap = build_move_remap(ssa);
        if remap.is_empty() {
            break;
        }
        let folded = remap.len() as u32;
        apply_remap(ssa, &remap);
        drop_folded_moves(ssa, &remap);
        stats.moves_folded += folded;
    }

    drop_dead(ssa, &mut stats);

    stats
}

/// Remove every `Move` instruction whose `dst` was just folded into a
/// substitution map. The remap consumers (phi incoming, instruction
/// ops, terminator ops) were rewritten in [`apply_remap`]; the Move's
/// own use sites are therefore gone, and keeping the Move would only
/// reintroduce the same entry on the next iteration of the fixed
/// point. Surviving instructions are reindexed so
/// `ValueSource::Instruction.index` continues to point at the
/// defining op.
fn drop_folded_moves(ssa: &mut SsaFunction, remap: &BTreeMap<ValueId, Operand>) {
    for bi in 0..ssa.blocks.len() {
        let drained: Vec<SsaInstruction> = std::mem::take(&mut ssa.blocks[bi].instructions);
        let mut kept = Vec::with_capacity(drained.len());
        for ins in drained {
            let drop = matches!(
                (ins.dst, &ins.op),
                (Some(d), SsaOp::Move { .. }) if remap.contains_key(&d)
            );
            if !drop {
                kept.push(ins);
            }
        }
        for (new_index, ins) in kept.iter().enumerate() {
            if let Some(dst) = ins.dst {
                if let ValueSource::Instruction { index, .. } = &mut ssa.values[dst as usize].source
                {
                    *index = new_index as u32;
                }
            }
        }
        ssa.blocks[bi].instructions = kept;
    }
}

/// Walk every instruction and rewrite the op in place when its
/// arguments match a constant- or identity-fold pattern. Each rewrite
/// turns the instruction into a `Move`, which the next call to
/// [`build_move_remap`] picks up as a substitution candidate.
fn canonicalize_in_place(ssa: &mut SsaFunction, stats: &mut SimplifyStats) {
    for block in &mut ssa.blocks {
        for ins in &mut block.instructions {
            let Some((new_op, kind)) = canonical_form(&ins.op) else {
                continue;
            };
            match kind {
                FoldKind::Constant => stats.constants_folded += 1,
                FoldKind::Identity => stats.identities_folded += 1,
            }
            ins.op = new_op;
        }
    }
}

#[derive(Clone, Copy)]
enum FoldKind {
    Constant,
    Identity,
}

/// Return a simplified op + the fold class, or `None` when nothing
/// changes. The simplified op is always a `Move` carrying either a
/// constant or a value operand, so a follow-up [`build_move_remap`]
/// can substitute it out at every use site.
fn canonical_form(op: &SsaOp) -> Option<(SsaOp, FoldKind)> {
    use Operand::{Const, Value};
    match op {
        // Constant folding on binary integer ops. Arithmetic uses
        // wrapping semantics — SSA values carry width metadata on the
        // variable, not the operand, so widening past 64 bits never
        // happens.
        SsaOp::Add {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((
            SsaOp::Move {
                src: Const(a.wrapping_add(*b)),
            },
            FoldKind::Constant,
        )),
        SsaOp::Sub {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((
            SsaOp::Move {
                src: Const(a.wrapping_sub(*b)),
            },
            FoldKind::Constant,
        )),
        SsaOp::Mul {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((
            SsaOp::Move {
                src: Const(a.wrapping_mul(*b)),
            },
            FoldKind::Constant,
        )),
        SsaOp::And {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((SsaOp::Move { src: Const(a & b) }, FoldKind::Constant)),
        SsaOp::Or {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((SsaOp::Move { src: Const(a | b) }, FoldKind::Constant)),
        SsaOp::Xor {
            lhs: Const(a),
            rhs: Const(b),
        } => Some((SsaOp::Move { src: Const(a ^ b) }, FoldKind::Constant)),
        SsaOp::Shl {
            lhs: Const(a),
            rhs: Const(b),
        } if (0..64).contains(b) => Some((
            SsaOp::Move {
                src: Const(((*a as u64).wrapping_shl(*b as u32)) as i64),
            },
            FoldKind::Constant,
        )),
        SsaOp::Shr {
            lhs: Const(a),
            rhs: Const(b),
        } if (0..64).contains(b) => Some((
            SsaOp::Move {
                src: Const(((*a as u64) >> (*b as u32)) as i64),
            },
            FoldKind::Constant,
        )),
        SsaOp::Neg { src: Const(a) } => Some((
            SsaOp::Move {
                src: Const(a.wrapping_neg()),
            },
            FoldKind::Constant,
        )),
        SsaOp::Not { src: Const(a) } => Some((SsaOp::Move { src: Const(!a) }, FoldKind::Constant)),

        // Identities `(x op x)` — only fire when both operands are the
        // *same* SSA value or the *same* constant. `Operand` derives
        // `PartialEq` over its discriminant + payload so `Const(0) ==
        // Const(0)` and `Value(v) == Value(v)` decide correctly.
        SsaOp::Sub { lhs, rhs } if lhs == rhs => {
            Some((SsaOp::Move { src: Const(0) }, FoldKind::Identity))
        }
        SsaOp::Xor { lhs, rhs } if lhs == rhs => {
            Some((SsaOp::Move { src: Const(0) }, FoldKind::Identity))
        }
        SsaOp::And { lhs, rhs } if lhs == rhs => match lhs {
            Value(_) | Const(_) => Some((SsaOp::Move { src: *lhs }, FoldKind::Identity)),
            Operand::Undef => None,
        },
        SsaOp::Or { lhs, rhs } if lhs == rhs => match lhs {
            Value(_) | Const(_) => Some((SsaOp::Move { src: *lhs }, FoldKind::Identity)),
            Operand::Undef => None,
        },

        _ => None,
    }
}

/// Build the substitution map: every `dst` whose def is a `Move` with
/// a foldable source maps to that source. Transitive chains
/// (`v → w → x`) are closed so a single application substitutes the
/// final operand.
fn build_move_remap(ssa: &SsaFunction) -> BTreeMap<ValueId, Operand> {
    let mut raw: BTreeMap<ValueId, Operand> = BTreeMap::new();
    for block in &ssa.blocks {
        for ins in &block.instructions {
            if let (Some(dst), SsaOp::Move { src }) = (ins.dst, &ins.op) {
                match src {
                    Operand::Value(_) | Operand::Const(_) => {
                        raw.insert(dst, *src);
                    }
                    Operand::Undef => {}
                }
            }
        }
    }
    let mut closed: BTreeMap<ValueId, Operand> = BTreeMap::new();
    for (&v, &op) in &raw {
        let resolved = follow(op, &raw);
        // Skip identity entries: if `dst` resolves to `Value(dst)` (a
        // self-loop, only possible through a malformed input), drop it
        // so the apply step makes no change.
        if resolved == Operand::Value(v) {
            continue;
        }
        closed.insert(v, resolved);
    }
    closed
}

/// Walk a `Value` operand through `remap` until it bottoms out at a
/// `Const`, `Undef`, or a `Value` not present in the map. Bounded by
/// the map size so a malformed cycle cannot diverge.
fn follow(mut op: Operand, remap: &BTreeMap<ValueId, Operand>) -> Operand {
    for _ in 0..(remap.len() + 1) {
        let Operand::Value(v) = op else {
            return op;
        };
        match remap.get(&v) {
            Some(&next) if next != op => op = next,
            _ => return op,
        }
    }
    op
}

/// Apply `remap` to every operand in the function — phis, instruction
/// ops, terminators.
fn apply_remap(ssa: &mut SsaFunction, remap: &BTreeMap<ValueId, Operand>) {
    for block in &mut ssa.blocks {
        for phi in &mut block.phis {
            for (_, op) in &mut phi.incoming {
                *op = remap_operand(*op, remap);
            }
        }
        for ins in &mut block.instructions {
            let old = std::mem::replace(
                &mut ins.op,
                SsaOp::Move {
                    src: Operand::Undef,
                },
            );
            ins.op = remap_op(old, remap);
        }
        let old_term = std::mem::replace(&mut block.terminator, SsaTerminator::Unreachable);
        block.terminator = remap_terminator(old_term, remap);
    }
}

fn remap_operand(op: Operand, remap: &BTreeMap<ValueId, Operand>) -> Operand {
    match op {
        Operand::Value(v) => remap.get(&v).copied().unwrap_or(op),
        _ => op,
    }
}

fn remap_op(op: SsaOp, remap: &BTreeMap<ValueId, Operand>) -> SsaOp {
    match op {
        SsaOp::Move { src } => SsaOp::Move {
            src: remap_operand(src, remap),
        },
        SsaOp::Add { lhs, rhs } => SsaOp::Add {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Sub { lhs, rhs } => SsaOp::Sub {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Mul { lhs, rhs } => SsaOp::Mul {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::And { lhs, rhs } => SsaOp::And {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Or { lhs, rhs } => SsaOp::Or {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Xor { lhs, rhs } => SsaOp::Xor {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Shl { lhs, rhs } => SsaOp::Shl {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Shr { lhs, rhs } => SsaOp::Shr {
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Neg { src } => SsaOp::Neg {
            src: remap_operand(src, remap),
        },
        SsaOp::Not { src } => SsaOp::Not {
            src: remap_operand(src, remap),
        },
        SsaOp::Compare { kind, lhs, rhs } => SsaOp::Compare {
            kind,
            lhs: remap_operand(lhs, remap),
            rhs: remap_operand(rhs, remap),
        },
        SsaOp::Load { address, width } => SsaOp::Load {
            address: remap_operand(address, remap),
            width,
        },
        SsaOp::Store {
            address,
            value,
            width,
        } => SsaOp::Store {
            address: remap_operand(address, remap),
            value: remap_operand(value, remap),
            width,
        },
        SsaOp::Call { target, args } => SsaOp::Call {
            target,
            args: args.into_iter().map(|a| remap_operand(a, remap)).collect(),
        },
        SsaOp::Opaque { mnemonic, args } => SsaOp::Opaque {
            mnemonic,
            args: args.into_iter().map(|a| remap_operand(a, remap)).collect(),
        },
    }
}

fn remap_terminator(t: SsaTerminator, remap: &BTreeMap<ValueId, Operand>) -> SsaTerminator {
    match t {
        SsaTerminator::Branch {
            cond,
            taken,
            not_taken,
        } => SsaTerminator::Branch {
            cond: remap_operand(cond, remap),
            taken,
            not_taken,
        },
        SsaTerminator::Return { value } => SsaTerminator::Return {
            value: value.map(|v| remap_operand(v, remap)),
        },
        other => other,
    }
}

/// Compute the set of live SSA values, then prune dead pure
/// instructions and dead phis. Surviving instructions are reindexed
/// so `ValueSource::Instruction { block, index }` continues to point
/// at the defining op; surviving phis likewise.
fn drop_dead(ssa: &mut SsaFunction, stats: &mut SimplifyStats) {
    let live = compute_live(ssa);

    for bi in 0..ssa.blocks.len() {
        let drained: Vec<SsaInstruction> = std::mem::take(&mut ssa.blocks[bi].instructions);
        let mut kept = Vec::with_capacity(drained.len());
        for ins in drained {
            let keep = match ins.dst {
                Some(d) => live.contains(&d) || !is_pure(&ins.op),
                None => true,
            };
            if keep {
                kept.push(ins);
            } else {
                stats.dead_pure_dropped += 1;
            }
        }
        for (new_index, ins) in kept.iter().enumerate() {
            if let Some(dst) = ins.dst {
                if let ValueSource::Instruction { index, .. } = &mut ssa.values[dst as usize].source
                {
                    *index = new_index as u32;
                }
            }
        }
        ssa.blocks[bi].instructions = kept;
    }

    for bi in 0..ssa.blocks.len() {
        let drained: Vec<Phi> = std::mem::take(&mut ssa.blocks[bi].phis);
        let mut kept = Vec::with_capacity(drained.len());
        for phi in drained {
            if live.contains(&phi.dst) {
                kept.push(phi);
            } else {
                stats.dead_pure_dropped += 1;
            }
        }
        for (new_index, phi) in kept.iter().enumerate() {
            if let ValueSource::Phi { index, .. } = &mut ssa.values[phi.dst as usize].source {
                *index = new_index as u32;
            }
        }
        ssa.blocks[bi].phis = kept;
    }
}

fn is_pure(op: &SsaOp) -> bool {
    matches!(
        op,
        SsaOp::Move { .. }
            | SsaOp::Add { .. }
            | SsaOp::Sub { .. }
            | SsaOp::Mul { .. }
            | SsaOp::And { .. }
            | SsaOp::Or { .. }
            | SsaOp::Xor { .. }
            | SsaOp::Shl { .. }
            | SsaOp::Shr { .. }
            | SsaOp::Neg { .. }
            | SsaOp::Not { .. }
            | SsaOp::Compare { .. }
    )
}

/// Mark every value transitively reachable from a side-effectful op
/// or a terminator operand as live.
fn compute_live(ssa: &SsaFunction) -> BTreeSet<ValueId> {
    let mut live: BTreeSet<ValueId> = BTreeSet::new();
    let mut worklist: Vec<ValueId> = Vec::new();

    // Seed: terminator operands + every operand of a side-effectful
    // instruction. Side-effectful instructions also seed their own
    // `dst` so consumers downstream stay live.
    for block in &ssa.blocks {
        match &block.terminator {
            SsaTerminator::Branch { cond, .. } => mark(cond, &mut live, &mut worklist),
            SsaTerminator::Return { value: Some(v) } => mark(v, &mut live, &mut worklist),
            _ => {}
        }
        for ins in &block.instructions {
            let has_side_effect = !is_pure(&ins.op);
            if has_side_effect {
                if let Some(d) = ins.dst {
                    if live.insert(d) {
                        worklist.push(d);
                    }
                }
                for op in iter_op_operands(&ins.op) {
                    mark(&op, &mut live, &mut worklist);
                }
            }
        }
    }

    // Transitive closure: for every live value, the operands of its
    // defining site become live.
    while let Some(v) = worklist.pop() {
        let source = ssa.values[v as usize].source;
        match source {
            ValueSource::Instruction { block, index } => {
                if let Some(ins) = ssa
                    .blocks
                    .get(block as usize)
                    .and_then(|b| b.instructions.get(index as usize))
                {
                    for op in iter_op_operands(&ins.op) {
                        mark(&op, &mut live, &mut worklist);
                    }
                }
            }
            ValueSource::Phi { block, index } => {
                if let Some(phi) = ssa
                    .blocks
                    .get(block as usize)
                    .and_then(|b| b.phis.get(index as usize))
                {
                    for (_, op) in &phi.incoming {
                        mark(op, &mut live, &mut worklist);
                    }
                }
            }
            ValueSource::Parameter { .. } => {}
        }
    }

    live
}

fn mark(op: &Operand, live: &mut BTreeSet<ValueId>, worklist: &mut Vec<ValueId>) {
    if let Operand::Value(v) = op {
        if live.insert(*v) {
            worklist.push(*v);
        }
    }
}

fn iter_op_operands(op: &SsaOp) -> Vec<Operand> {
    match op {
        SsaOp::Move { src } | SsaOp::Neg { src } | SsaOp::Not { src } => vec![*src],
        SsaOp::Add { lhs, rhs }
        | SsaOp::Sub { lhs, rhs }
        | SsaOp::Mul { lhs, rhs }
        | SsaOp::And { lhs, rhs }
        | SsaOp::Or { lhs, rhs }
        | SsaOp::Xor { lhs, rhs }
        | SsaOp::Shl { lhs, rhs }
        | SsaOp::Shr { lhs, rhs } => vec![*lhs, *rhs],
        SsaOp::Compare { lhs, rhs, .. } => vec![*lhs, *rhs],
        SsaOp::Load { address, .. } => vec![*address],
        SsaOp::Store { address, value, .. } => vec![*address, *value],
        SsaOp::Call { args, .. } | SsaOp::Opaque { args, .. } => args.clone(),
    }
}

/// Returns `true` when the value has a defining instruction or phi at
/// its recorded `ValueSource`. Used by the C backend to skip orphan
/// locals — values whose def was elided by [`simplify`] (or by an
/// earlier CSE pass) and which therefore should not be declared.
///
/// `Parameter` sources always return `true` so the call boundary's
/// `arg<n> → v<id>` init still emits.
#[must_use]
pub fn value_has_definition(ssa: &SsaFunction, id: ValueId) -> bool {
    let def = &ssa.values[id as usize];
    match def.source {
        ValueSource::Parameter { .. } => true,
        ValueSource::Instruction { block, index } => {
            ssa.blocks
                .get(block as usize)
                .and_then(|b| b.instructions.get(index as usize))
                .and_then(|ins| ins.dst)
                == Some(id)
        }
        ValueSource::Phi { block, index } => {
            ssa.blocks
                .get(block as usize)
                .and_then(|b| b.phis.get(index as usize))
                .map(|phi| phi.dst)
                == Some(id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{
        CompareKind, SsaBlock, SsaFunction, SsaInstruction, SsaTerminator, ValueDef, Variable,
    };

    // ---- builders ----------------------------------------------------

    fn make_ssa(blocks: Vec<SsaBlock>, values: Vec<ValueDef>) -> SsaFunction {
        let mut g = EvidenceGraph::new();
        let evidence = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Ssa,
            id: 0,
        });
        SsaFunction {
            function_address: 0x1000,
            function_name: Some("t".into()),
            blocks,
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values,
            evidence,
        }
    }

    fn val(id: ValueId, block: u32, index: u32) -> ValueDef {
        ValueDef {
            id,
            source: ValueSource::Instruction { block, index },
            variable: 0,
        }
    }

    fn ins(dst: ValueId, op: SsaOp) -> SsaInstruction {
        SsaInstruction { dst: Some(dst), op }
    }

    fn ins_void(op: SsaOp) -> SsaInstruction {
        SsaInstruction { dst: None, op }
    }

    fn single_block(
        id: u32,
        instructions: Vec<SsaInstruction>,
        terminator: SsaTerminator,
    ) -> SsaBlock {
        SsaBlock {
            id,
            predecessors: Vec::new(),
            phis: Vec::new(),
            instructions,
            terminator,
        }
    }

    // ---- folding tests -----------------------------------------------

    #[test]
    fn constant_folding_collapses_add_of_two_constants() {
        // v0 = Add(Const 3, Const 4)
        // return v0
        let block = single_block(
            0,
            vec![ins(
                0,
                SsaOp::Add {
                    lhs: Operand::Const(3),
                    rhs: Operand::Const(4),
                },
            )],
            SsaTerminator::Return {
                value: Some(Operand::Value(0)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0)]);

        let stats = simplify(&mut ssa);

        assert_eq!(stats.constants_folded, 1);
        // After folding + DCE, v0's def was a Move(Const(7)) which got
        // folded into the terminator; the dead pure Move is dropped.
        assert!(ssa.blocks[0].instructions.is_empty());
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Const(7)),
            }
        );
    }

    #[test]
    fn identity_xor_self_collapses_to_zero() {
        // v0 = Move(Const 5)
        // v1 = Xor(v0, v0)
        // return v1
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Move {
                        src: Operand::Const(5),
                    },
                ),
                ins(
                    1,
                    SsaOp::Xor {
                        lhs: Operand::Value(0),
                        rhs: Operand::Value(0),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Value(1)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1)]);

        let stats = simplify(&mut ssa);

        assert!(stats.identities_folded >= 1);
        // v0 + v1 are both dead pure Moves at the end.
        assert!(ssa.blocks[0].instructions.is_empty());
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Const(0)),
            }
        );
    }

    #[test]
    fn identity_and_self_collapses_to_self() {
        // v0 = <call result, side-effect>
        // v1 = And(v0, v0)
        // return v1
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Call {
                        target: Some(0x2000),
                        args: vec![],
                    },
                ),
                ins(
                    1,
                    SsaOp::And {
                        lhs: Operand::Value(0),
                        rhs: Operand::Value(0),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Value(1)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1)]);

        let stats = simplify(&mut ssa);

        assert!(stats.identities_folded >= 1);
        // v0's call is kept (side effect); v1's Move(v0) is folded into
        // the terminator and the dead pure Move is dropped.
        assert_eq!(ssa.blocks[0].instructions.len(), 1);
        assert!(matches!(
            ssa.blocks[0].instructions[0].op,
            SsaOp::Call { .. }
        ));
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Value(0)),
            }
        );
    }

    // ---- move-folding tests ------------------------------------------

    #[test]
    fn move_of_constant_inlines_at_use_sites() {
        // v0 = Move(Const 8)
        // v1 = Add(v0, Const 1)
        // return v1
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Move {
                        src: Operand::Const(8),
                    },
                ),
                ins(
                    1,
                    SsaOp::Add {
                        lhs: Operand::Value(0),
                        rhs: Operand::Const(1),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Value(1)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1)]);

        let stats = simplify(&mut ssa);

        assert!(stats.moves_folded >= 1);
        // After constant folding closes, the return carries the literal.
        assert!(ssa.blocks[0].instructions.is_empty());
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Const(9)),
            }
        );
    }

    #[test]
    fn move_chain_collapses_transitively() {
        // v0 = Move(Const 1)
        // v1 = Move(v0)
        // v2 = Move(v1)
        // return v2
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Move {
                        src: Operand::Const(1),
                    },
                ),
                ins(
                    1,
                    SsaOp::Move {
                        src: Operand::Value(0),
                    },
                ),
                ins(
                    2,
                    SsaOp::Move {
                        src: Operand::Value(1),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Value(2)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1), val(2, 0, 2)]);

        let stats = simplify(&mut ssa);

        assert!(stats.moves_folded >= 1);
        assert!(ssa.blocks[0].instructions.is_empty());
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Const(1)),
            }
        );
    }

    // ---- liveness / DCE tests ----------------------------------------

    #[test]
    fn dead_zero_init_is_dropped() {
        // v0 = Move(Const 0)
        // v1 = Move(Const 0)         <-- dead
        // v2 = Add(v0, Const 5)
        // return v2
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Move {
                        src: Operand::Const(0),
                    },
                ),
                ins(
                    1,
                    SsaOp::Move {
                        src: Operand::Const(0),
                    },
                ),
                ins(
                    2,
                    SsaOp::Add {
                        lhs: Operand::Value(0),
                        rhs: Operand::Const(5),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Value(2)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1), val(2, 0, 2)]);

        let stats = simplify(&mut ssa);

        // v1 has no live use; whether it left via the Move-fold path
        // (its `Move(Const 0)` participated in the remap and got
        // dropped) or the dead-pure path is implementation detail —
        // the C backend's `lower_locals` skip catches it either way.
        assert!(stats.total() >= 3);
        assert!(!value_has_definition(&ssa, 1));
        // v0 and v2 collapsed into the terminator's Const(5).
        assert_eq!(
            ssa.blocks[0].terminator,
            SsaTerminator::Return {
                value: Some(Operand::Const(5)),
            }
        );
    }

    #[test]
    fn dead_pure_add_with_no_consumer_is_dropped_by_dce() {
        // v0 = Load(Const 0x4000, 8)        <-- side-effect, kept
        // v1 = Add(v0, Const 5)             <-- pure, no consumer
        // return Const 0
        //
        // v1 reaches the DCE path because it is not a Move (so the
        // remap path skips it) and its dst has no live use. Asserts
        // that `dead_pure_dropped` actually moves on the DCE-only
        // path.
        let block = single_block(
            0,
            vec![
                ins(
                    0,
                    SsaOp::Load {
                        address: Operand::Const(0x4000),
                        width: 8,
                    },
                ),
                ins(
                    1,
                    SsaOp::Add {
                        lhs: Operand::Value(0),
                        rhs: Operand::Const(5),
                    },
                ),
            ],
            SsaTerminator::Return {
                value: Some(Operand::Const(0)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1)]);

        let stats = simplify(&mut ssa);

        assert_eq!(stats.dead_pure_dropped, 1);
        assert!(value_has_definition(&ssa, 0));
        assert!(!value_has_definition(&ssa, 1));
    }

    #[test]
    fn side_effectful_call_is_not_dropped_even_when_dst_is_dead() {
        // v0 = Call(0x2000)          <-- side-effect; result unused
        // return Const 0
        let block = single_block(
            0,
            vec![ins(
                0,
                SsaOp::Call {
                    target: Some(0x2000),
                    args: vec![],
                },
            )],
            SsaTerminator::Return {
                value: Some(Operand::Const(0)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0)]);

        let stats = simplify(&mut ssa);

        assert_eq!(stats.dead_pure_dropped, 0);
        assert_eq!(ssa.blocks[0].instructions.len(), 1);
        assert!(matches!(
            ssa.blocks[0].instructions[0].op,
            SsaOp::Call { .. }
        ));
    }

    #[test]
    fn store_is_never_dropped() {
        // *Const(0x1000) = Const(5)
        let block = single_block(
            0,
            vec![ins_void(SsaOp::Store {
                address: Operand::Const(0x1000),
                value: Operand::Const(5),
                width: 8,
            })],
            SsaTerminator::Return { value: None },
        );
        let mut ssa = make_ssa(vec![block], vec![]);

        let _ = simplify(&mut ssa);

        assert_eq!(ssa.blocks[0].instructions.len(), 1);
        assert!(matches!(
            ssa.blocks[0].instructions[0].op,
            SsaOp::Store { .. }
        ));
    }

    #[test]
    fn opaque_is_never_dropped() {
        let block = single_block(
            0,
            vec![ins_void(SsaOp::Opaque {
                mnemonic: "hlt".into(),
                args: vec![],
            })],
            SsaTerminator::Return { value: None },
        );
        let mut ssa = make_ssa(vec![block], vec![]);

        let _ = simplify(&mut ssa);

        assert_eq!(ssa.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn load_is_kept_conservatively() {
        // v0 = Load(Const 0x4000, 8)
        // return Const 0       <-- v0 has no consumer
        let block = single_block(
            0,
            vec![ins(
                0,
                SsaOp::Load {
                    address: Operand::Const(0x4000),
                    width: 8,
                },
            )],
            SsaTerminator::Return {
                value: Some(Operand::Const(0)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0)]);

        let _ = simplify(&mut ssa);

        // Load stays — its result going unused is fine, but eliding the
        // load could re-order across a volatile / MMIO read.
        assert_eq!(ssa.blocks[0].instructions.len(), 1);
    }

    // ---- branch-cond paths -------------------------------------------

    #[test]
    fn branch_cond_keeps_its_compare_alive() {
        // v0 = Compare(Eq, Const 1, Const 0)
        // branch v0 ? B1 : B2
        let block0 = SsaBlock {
            id: 0,
            predecessors: Vec::new(),
            phis: Vec::new(),
            instructions: vec![ins(
                0,
                SsaOp::Compare {
                    kind: CompareKind::Eq,
                    lhs: Operand::Const(1),
                    rhs: Operand::Const(0),
                },
            )],
            terminator: SsaTerminator::Branch {
                cond: Operand::Value(0),
                taken: 1,
                not_taken: 2,
            },
        };
        let block1 = single_block(1, vec![], SsaTerminator::Return { value: None });
        let block2 = single_block(2, vec![], SsaTerminator::Return { value: None });
        let mut ssa = make_ssa(vec![block0, block1, block2], vec![val(0, 0, 0)]);

        let _ = simplify(&mut ssa);

        // Compare is pure but its dst is used by the branch terminator,
        // so it survives liveness.
        assert_eq!(ssa.blocks[0].instructions.len(), 1);
    }

    // ---- value_has_definition ----------------------------------------

    #[test]
    fn value_has_definition_recognises_dropped_orphans() {
        // v0 = Move(Const 0); unused
        // return Const 1
        let block = single_block(
            0,
            vec![ins(
                0,
                SsaOp::Move {
                    src: Operand::Const(0),
                },
            )],
            SsaTerminator::Return {
                value: Some(Operand::Const(1)),
            },
        );
        let mut ssa = make_ssa(vec![block], vec![val(0, 0, 0)]);
        assert!(value_has_definition(&ssa, 0));

        let _ = simplify(&mut ssa);

        // After DCE, v0's source still says (block 0, index 0) but the
        // block at that index is empty — predicate returns false.
        assert!(!value_has_definition(&ssa, 0));
    }

    // ---- determinism (NFR-9) -----------------------------------------

    #[test]
    fn simplify_is_byte_stable_across_two_runs() {
        // Same SSA in, same SSA out, twice over.
        let make = || {
            let block = single_block(
                0,
                vec![
                    ins(
                        0,
                        SsaOp::Move {
                            src: Operand::Const(0),
                        },
                    ),
                    ins(
                        1,
                        SsaOp::Move {
                            src: Operand::Const(0),
                        },
                    ),
                    ins(
                        2,
                        SsaOp::Add {
                            lhs: Operand::Value(0),
                            rhs: Operand::Const(5),
                        },
                    ),
                ],
                SsaTerminator::Return {
                    value: Some(Operand::Value(2)),
                },
            );
            make_ssa(vec![block], vec![val(0, 0, 0), val(1, 0, 1), val(2, 0, 2)])
        };
        let mut a = make();
        let mut b = make();
        let _ = simplify(&mut a);
        let _ = simplify(&mut b);
        assert_eq!(a, b);
    }
}
