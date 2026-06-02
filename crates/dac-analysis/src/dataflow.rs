//! Dataflow over SSA (B2.4, FR-11).
//!
//! The SSA construction in [`crate::ssa`] embeds use-def directly:
//! every [`Operand::Value`] reference names its single definition, and
//! [`SsaFunction::value`] resolves it in O(1). The remaining classical
//! analyses are the *inverse* direction (def-use chains) and the
//! per-value liveness sets a register allocator or type-propagation
//! pass would consult.
//!
//! ## What this module computes
//!
//! - [`compute_def_use`] inverts the SSA value graph, returning a
//!   [`DefUseChains`] table keyed by [`ValueId`]. Every site that
//!   consumes a value (phi incoming edge, instruction operand, block
//!   terminator) appears once per occurrence, in source order.
//! - [`compute_liveness`] computes per-block [`SsaLiveness`] — the
//!   sets of value ids live on entry to and exit from each block.
//!   This is a backward dataflow over the SSA CFG, with the crucial
//!   twist that *phi operands are use sites on the predecessor edge*,
//!   not on the join block itself (otherwise loop-carried values
//!   spuriously appear live in their join's live-in set).
//!
//! ## What this module deliberately does *not* compute
//!
//! - **Reaching definitions.** In SSA every use already names its
//!   single reaching definition; a separate reaching-definitions
//!   table would duplicate state without buying anything new.
//!   Callers that want "the definition reaching this operand" should
//!   call [`SsaFunction::value`] directly, or use [`def_of`] for
//!   symmetry with [`compute_def_use`]. The interesting reaching
//!   problem — *which store reaches this load* — is a memory-SSA
//!   concern that B2.4 explicitly does not tackle; it lands later
//!   alongside alias-aware dataflow.
//! - **Use-def chains.** Same reason: SSA gives them away for free.
//!
//! ## Determinism (NFR-9)
//!
//! Both passes are deterministic. Iteration order is by ascending
//! block id and instruction index; outputs use [`BTreeMap`] /
//! [`BTreeSet`] containers keyed by these ids. The same
//! [`SsaFunction`] always produces the same chains and liveness sets,
//! and the on-disk representation is byte-stable.

use std::collections::{BTreeMap, BTreeSet};

use dac_ir::ssa::{Operand, SsaBlockId, SsaFunction, SsaOp, SsaTerminator, ValueDef, ValueId};

/// Inverted def-use graph.
///
/// For each [`ValueId`] referenced anywhere in the SSA function, the
/// table records every site that consumes it. Values defined but
/// never consumed produce no entry — callers can detect dead values
/// by querying [`DefUseChains::uses`] and checking for an empty
/// slice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DefUseChains {
    sites: BTreeMap<ValueId, Vec<UseSite>>,
}

impl DefUseChains {
    /// Use sites of `value` in source order. Returns an empty slice
    /// for values that are defined but never read.
    #[must_use]
    pub fn uses(&self, value: ValueId) -> &[UseSite] {
        self.sites
            .get(&value)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// True when the value has no use site. Includes both values that
    /// were never referenced and values whose only definitions were
    /// folded out by trivial CSE (the canonical id keeps the uses,
    /// the folded ids do not).
    #[must_use]
    pub fn is_dead(&self, value: ValueId) -> bool {
        self.sites.get(&value).is_none_or(|s| s.is_empty())
    }

    /// Number of distinct use sites of `value`.
    #[must_use]
    pub fn use_count(&self, value: ValueId) -> usize {
        self.sites.get(&value).map_or(0, Vec::len)
    }

    /// Iterate over every (value, sites) pair in ascending value-id
    /// order.
    pub fn iter(&self) -> impl Iterator<Item = (ValueId, &[UseSite])> + '_ {
        self.sites.iter().map(|(v, s)| (*v, s.as_slice()))
    }
}

/// Where an SSA value is consumed.
///
/// Locations are coarse: they identify the syntactic site, not which
/// operand within the site. An instruction such as `Add { lhs: v,
/// rhs: v }` records two [`UseSite::Instruction`] entries for `v`
/// because two distinct operand slots reference it. This matches the
/// classic "number of uses" semantics callers (DCE, copy propagation)
/// expect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UseSite {
    /// `blocks[block].phis[phi].incoming[incoming]` consumes the value.
    Phi {
        block: SsaBlockId,
        phi: u32,
        incoming: u32,
    },
    /// `blocks[block].instructions[index]` consumes the value in one
    /// of its operand slots.
    Instruction { block: SsaBlockId, index: u32 },
    /// `blocks[block].terminator` consumes the value (a branch
    /// condition or a returned value).
    Terminator { block: SsaBlockId },
}

/// Compute the def-use chain for every value referenced in `ssa`.
///
/// The forward direction (use → def) is implicit in SSA: every
/// `Operand::Value(v)` already names its single definition, and
/// [`SsaFunction::value`] (or [`def_of`]) resolves it in O(1).
#[must_use]
pub fn compute_def_use(ssa: &SsaFunction) -> DefUseChains {
    let mut sites: BTreeMap<ValueId, Vec<UseSite>> = BTreeMap::new();
    let record = |v: ValueId, site: UseSite, sites: &mut BTreeMap<ValueId, Vec<UseSite>>| {
        sites.entry(v).or_default().push(site);
    };

    for block in &ssa.blocks {
        let bid = block.id;
        for (phi_idx, phi) in block.phis.iter().enumerate() {
            for (inc_idx, &(_, opnd)) in phi.incoming.iter().enumerate() {
                if let Operand::Value(v) = opnd {
                    record(
                        v,
                        UseSite::Phi {
                            block: bid,
                            phi: phi_idx as u32,
                            incoming: inc_idx as u32,
                        },
                        &mut sites,
                    );
                }
            }
        }
        for (idx, ins) in block.instructions.iter().enumerate() {
            for opnd in operands_of(&ins.op) {
                if let Operand::Value(v) = opnd {
                    record(
                        v,
                        UseSite::Instruction {
                            block: bid,
                            index: idx as u32,
                        },
                        &mut sites,
                    );
                }
            }
        }
        for opnd in operands_of_terminator(&block.terminator) {
            if let Operand::Value(v) = opnd {
                record(v, UseSite::Terminator { block: bid }, &mut sites);
            }
        }
    }

    DefUseChains { sites }
}

/// Resolve the single definition of `value`. Kept as a thin wrapper
/// around [`SsaFunction::value`] for symmetry with [`compute_def_use`].
#[must_use]
pub fn def_of(ssa: &SsaFunction, value: ValueId) -> &ValueDef {
    ssa.value(value)
}

/// Per-block SSA liveness — sets of [`ValueId`]s live on entry and
/// exit from each block.
///
/// Indexed by [`SsaBlockId`]; the vectors have length
/// `ssa.blocks.len()`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SsaLiveness {
    live_in: Vec<BTreeSet<ValueId>>,
    live_out: Vec<BTreeSet<ValueId>>,
}

impl SsaLiveness {
    /// Values live on entry to `block`.
    #[must_use]
    pub fn live_in(&self, block: SsaBlockId) -> &BTreeSet<ValueId> {
        &self.live_in[block as usize]
    }

    /// Values live on exit from `block`.
    #[must_use]
    pub fn live_out(&self, block: SsaBlockId) -> &BTreeSet<ValueId> {
        &self.live_out[block as usize]
    }

    /// True when `value` is live on entry to `block`.
    #[must_use]
    pub fn is_live_in(&self, block: SsaBlockId, value: ValueId) -> bool {
        self.live_in[block as usize].contains(&value)
    }

    /// True when `value` is live on exit from `block`.
    #[must_use]
    pub fn is_live_out(&self, block: SsaBlockId, value: ValueId) -> bool {
        self.live_out[block as usize].contains(&value)
    }
}

/// Compute per-block liveness via backward dataflow.
///
/// The equation system is the classical one:
///
/// ```text
/// LiveIn[b]  = UsesBeforeDef[b] ∪ (LiveOut[b] - DefInBlock[b])
/// LiveOut[b] = ⋃ LiveIn[s] over successors s     (instruction uses)
///            ∪ ⋃ PhiUseOnEdge[b → s]            (phi-incoming uses)
/// ```
///
/// The phi-incoming term is the subtle bit. A phi at `s` whose
/// incoming entry on edge `(b, s)` is `Operand::Value(v)` represents
/// a *use* of `v` along that specific edge, not in `s`'s live-in. If
/// we naively rolled phi-incoming uses into `s`'s live-in, then `v`
/// would also become live on every *other* predecessor edge into `s`,
/// inflating live ranges spuriously. Treating phi operands as
/// per-edge uses on the predecessor's live-out side is the standard
/// fix and matches Cooper & Torczon §9.2.
#[must_use]
pub fn compute_liveness(ssa: &SsaFunction) -> SsaLiveness {
    let n = ssa.blocks.len();

    // Derive successors from each block's predecessor list.
    let mut succs: Vec<Vec<SsaBlockId>> = vec![Vec::new(); n];
    for b in 0..n {
        for &p in &ssa.blocks[b].predecessors {
            succs[p as usize].push(b as SsaBlockId);
        }
    }

    // Per-block uses-before-def and defs.
    let mut uses_before_def: Vec<BTreeSet<ValueId>> = vec![BTreeSet::new(); n];
    let mut def_in_block: Vec<BTreeSet<ValueId>> = vec![BTreeSet::new(); n];
    // Per-edge phi uses: (pred, succ) -> set of values consumed.
    let mut phi_use_on_edge: BTreeMap<(SsaBlockId, SsaBlockId), BTreeSet<ValueId>> =
        BTreeMap::new();

    for block in &ssa.blocks {
        let b = block.id as usize;
        let mut local_defs: BTreeSet<ValueId> = BTreeSet::new();

        // Phi destinations are defined at block start; their operands
        // are recorded as edge uses, not as uses-before-def of the
        // join block.
        for phi in &block.phis {
            local_defs.insert(phi.dst);
            for &(pred, opnd) in &phi.incoming {
                if let Operand::Value(v) = opnd {
                    phi_use_on_edge
                        .entry((pred, block.id))
                        .or_default()
                        .insert(v);
                }
            }
        }

        // Walk instructions, classifying each value reference as
        // use-before-def or use-after-def-in-same-block.
        for ins in &block.instructions {
            for opnd in operands_of(&ins.op) {
                if let Operand::Value(v) = opnd {
                    if !local_defs.contains(&v) {
                        uses_before_def[b].insert(v);
                    }
                }
            }
            if let Some(d) = ins.dst {
                local_defs.insert(d);
            }
        }

        // Terminator operands are uses after every instruction in the
        // block has had its chance to define.
        for opnd in operands_of_terminator(&block.terminator) {
            if let Operand::Value(v) = opnd {
                if !local_defs.contains(&v) {
                    uses_before_def[b].insert(v);
                }
            }
        }

        def_in_block[b] = local_defs;
    }

    // Iterate to fixed point. Descending block-id sweep is the usual
    // heuristic for fast convergence on reverse-postorder-ish layouts.
    let mut live_in: Vec<BTreeSet<ValueId>> = uses_before_def.clone();
    let mut live_out: Vec<BTreeSet<ValueId>> = vec![BTreeSet::new(); n];
    let mut changed = true;
    while changed {
        changed = false;
        for b in (0..n).rev() {
            let mut out: BTreeSet<ValueId> = BTreeSet::new();
            for &s in &succs[b] {
                out.extend(live_in[s as usize].iter().copied());
                if let Some(set) = phi_use_on_edge.get(&(b as SsaBlockId, s)) {
                    out.extend(set.iter().copied());
                }
            }
            let mut new_in: BTreeSet<ValueId> = out.difference(&def_in_block[b]).copied().collect();
            new_in.extend(uses_before_def[b].iter().copied());
            if new_in != live_in[b] || out != live_out[b] {
                live_in[b] = new_in;
                live_out[b] = out;
                changed = true;
            }
        }
    }

    SsaLiveness { live_in, live_out }
}

/// Enumerate the value operands of an [`SsaOp`].
///
/// Lifted into a free function so [`compute_def_use`] and
/// [`compute_liveness`] share the same traversal. Add a new op kind
/// here when extending [`SsaOp`]; the exhaustive match keeps the
/// compiler honest.
fn operands_of(op: &SsaOp) -> Vec<Operand> {
    match op {
        SsaOp::Move { src } | SsaOp::Neg { src } | SsaOp::Not { src } => vec![*src],
        SsaOp::Add { lhs, rhs }
        | SsaOp::Sub { lhs, rhs }
        | SsaOp::Mul { lhs, rhs }
        | SsaOp::And { lhs, rhs }
        | SsaOp::Or { lhs, rhs }
        | SsaOp::Xor { lhs, rhs }
        | SsaOp::Shl { lhs, rhs }
        | SsaOp::Shr { lhs, rhs }
        | SsaOp::Compare { lhs, rhs, .. } => vec![*lhs, *rhs],
        SsaOp::Load { address, .. } => vec![*address],
        SsaOp::Store { address, value, .. } => vec![*address, *value],
        SsaOp::Call { args, .. } | SsaOp::Opaque { args, .. } => args.clone(),
    }
}

fn operands_of_terminator(t: &SsaTerminator) -> Vec<Operand> {
    match t {
        SsaTerminator::Branch { cond, .. } => vec![*cond],
        SsaTerminator::Return { value: Some(v) } => vec![*v],
        SsaTerminator::Jump { .. }
        | SsaTerminator::Return { value: None }
        | SsaTerminator::Indirect
        | SsaTerminator::Unreachable => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{Phi, SsaBlock, SsaInstruction, Variable, VariableId};

    use crate::cfg::{BasicBlock, Cfg, Edge, EdgeKind as CfgEdgeKind, Terminator};
    use crate::dom::DominatorTree;
    use crate::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };
    use crate::test_support::edge_kind_key;

    // --- helpers (mirrors crate::test_support; kept local because that
    // module is cfg(test) and not re-exported across modules cleanly). ---

    fn synthetic_cfg(n: usize, entry: u32, raw_edges: &[(u32, u32, CfgEdgeKind)]) -> Cfg {
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
        let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::from([entry]);
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

    fn parameter_of(ssa: &SsaFunction, name: &str) -> ValueId {
        let var = ssa
            .variables
            .iter()
            .find(|v| v.name == name)
            .expect("variable exists");
        ssa.values
            .iter()
            .find(|val| {
                matches!(
                    val.source,
                    dac_ir::ssa::ValueSource::Parameter { variable } if variable == var.id
                )
            })
            .expect("parameter exists")
            .id
    }

    // --- tests --------------------------------------------------

    #[test]
    fn def_use_records_each_operand_occurrence() {
        // a = 1
        // b = a + a  (two uses of `a` in one instruction)
        // ret b
        let raw = RawFunction {
            variables: vec![var(0, "a"), var(1, "b")],
            blocks: vec![RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    },
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Add {
                            lhs: RawOperand::Variable(0),
                            rhs: RawOperand::Variable(0),
                        },
                    },
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let cfg = synthetic_cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let du = compute_def_use(&ssa);

        // `a`'s value is consumed twice by the same instruction.
        let a_val = ssa.blocks[0].instructions[0].dst.unwrap();
        assert_eq!(du.use_count(a_val), 2);

        // `b`'s value is consumed once by the terminator.
        let b_val = ssa.blocks[0].instructions[1].dst.unwrap();
        assert_eq!(du.use_count(b_val), 1);
        assert!(matches!(
            du.uses(b_val)[0],
            UseSite::Terminator { block: 0 }
        ));
    }

    #[test]
    fn def_use_marks_dead_values() {
        // a = 1                (dead)
        // b = 2
        // ret b
        let raw = RawFunction {
            variables: vec![var(0, "a"), var(1, "b")],
            blocks: vec![RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    },
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(2),
                        },
                    },
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let cfg = synthetic_cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let du = compute_def_use(&ssa);

        let a_val = ssa.blocks[0].instructions[0].dst.unwrap();
        let b_val = ssa.blocks[0].instructions[1].dst.unwrap();
        assert!(du.is_dead(a_val));
        assert!(!du.is_dead(b_val));
    }

    #[test]
    fn def_use_records_phi_incoming_per_edge() {
        // diamond: b0 -> b1, b0 -> b2, b1 -> b3, b2 -> b3
        // b0: a = 1
        // b1: a = 2
        // b2: a = 3
        // b3: ret a
        let raw = RawFunction {
            variables: vec![var(0, "a")],
            blocks: vec![
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    }],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(2),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(3),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
            ],
        };
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, CfgEdgeKind::Taken),
                (0, 2, CfgEdgeKind::NotTaken),
                (1, 3, CfgEdgeKind::Fall),
                (2, 3, CfgEdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let du = compute_def_use(&ssa);

        // The two predecessor defs are each used by exactly one phi
        // incoming edge.
        let b1_def = ssa.blocks[1].instructions[0].dst.unwrap();
        let b2_def = ssa.blocks[2].instructions[0].dst.unwrap();
        assert_eq!(du.use_count(b1_def), 1);
        assert_eq!(du.use_count(b2_def), 1);
        assert!(matches!(du.uses(b1_def)[0], UseSite::Phi { block: 3, .. }));
        assert!(matches!(du.uses(b2_def)[0], UseSite::Phi { block: 3, .. }));
    }

    #[test]
    fn liveness_carries_value_across_branches() {
        // b0: a = 1; if cond goto b1 else b2
        // b1: ret a
        // b2: ret a
        let raw = RawFunction {
            variables: vec![var(0, "a"), var(1, "cond")],
            blocks: vec![
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    }],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(1),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
            ],
        };
        let cfg = synthetic_cfg(
            3,
            0,
            &[(0, 1, CfgEdgeKind::Taken), (0, 2, CfgEdgeKind::NotTaken)],
        );
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let live = compute_liveness(&ssa);

        let a_val = ssa.blocks[0].instructions[0].dst.unwrap();
        // `a` was just defined in b0, so it is live on exit of b0,
        // and live on entry to both successors.
        assert!(live.is_live_out(0, a_val));
        assert!(live.is_live_in(1, a_val));
        assert!(live.is_live_in(2, a_val));
    }

    #[test]
    fn liveness_does_not_inflate_phi_join_live_in() {
        // diamond producing a phi at b3. Each side-defined value is
        // live on the *edge* to b3, but NOT in b3's live-in (that
        // would inflate the live range onto the other predecessor).
        let raw = RawFunction {
            variables: vec![var(0, "a"), var(1, "c")],
            blocks: vec![
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(1),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(10),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(20),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
            ],
        };
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, CfgEdgeKind::Taken),
                (0, 2, CfgEdgeKind::NotTaken),
                (1, 3, CfgEdgeKind::Fall),
                (2, 3, CfgEdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let live = compute_liveness(&ssa);

        let v_b1 = ssa.blocks[1].instructions[0].dst.unwrap();
        let v_b2 = ssa.blocks[2].instructions[0].dst.unwrap();
        // Each side's value is live on exit of its own block.
        assert!(live.is_live_out(1, v_b1));
        assert!(live.is_live_out(2, v_b2));
        // But neither is live on entry to b3 — the join consumes the
        // phi's destination, not the per-side definitions.
        assert!(!live.is_live_in(3, v_b1));
        assert!(!live.is_live_in(3, v_b2));
    }

    #[test]
    fn liveness_keeps_loop_carry_live_through_back_edge() {
        // b0: i = 0; goto b1
        // b1: phi i; if i < 10 goto b2 else b3
        // b2: i = i + 1; goto b1
        // b3: ret i
        let raw = RawFunction {
            variables: vec![var(0, "i")],
            blocks: vec![
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(0),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Add {
                            lhs: RawOperand::Variable(0),
                            rhs: RawOperand::Const(1),
                        },
                    }],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
            ],
        };
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, CfgEdgeKind::Fall),
                (1, 2, CfgEdgeKind::Taken),
                (1, 3, CfgEdgeKind::NotTaken),
                (2, 1, CfgEdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let live = compute_liveness(&ssa);

        // Phi at b1 defines the loop's i. Its destination is live on
        // exit of b1 (used in the branch and consumed by b2's add).
        let phi_dst: ValueId = ssa.blocks[1].phis[0].dst;
        assert!(live.is_live_out(1, phi_dst));
        assert!(live.is_live_in(1, phi_dst) || live.is_live_in(2, phi_dst));
        // The back-edge value defined in b2 must be live on exit of b2
        // (it flows into b1's phi).
        let v_b2 = ssa.blocks[2].instructions[0].dst.unwrap();
        assert!(live.is_live_out(2, v_b2));
    }

    #[test]
    fn def_use_and_liveness_are_deterministic() {
        let raw = RawFunction {
            variables: vec![var(0, "a"), var(1, "b")],
            blocks: vec![RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    },
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Add {
                            lhs: RawOperand::Variable(0),
                            rhs: RawOperand::Const(2),
                        },
                    },
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let cfg = synthetic_cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);

        let du1 = compute_def_use(&ssa);
        let du2 = compute_def_use(&ssa);
        assert_eq!(du1, du2);

        let l1 = compute_liveness(&ssa);
        let l2 = compute_liveness(&ssa);
        assert_eq!(l1, l2);
    }

    #[test]
    fn liveness_handles_empty_function() {
        let raw = RawFunction {
            variables: vec![],
            blocks: vec![],
        };
        let cfg = synthetic_cfg(0, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let live = compute_liveness(&ssa);
        let du = compute_def_use(&ssa);
        // Sanity: no blocks, no values, no panics.
        assert_eq!(live.live_in.len(), 0);
        assert_eq!(du.iter().count(), 0);
        // parameter_of helper would panic; just touch SsaFunction directly.
        let _ = &ssa;
    }

    #[test]
    fn def_use_records_terminator_uses() {
        // Sanity: a Branch terminator records its cond use.
        let raw = RawFunction {
            variables: vec![var(0, "cond")],
            blocks: vec![
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(0),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(1),
                        },
                    }],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let cfg = synthetic_cfg(
            3,
            0,
            &[(0, 1, CfgEdgeKind::Taken), (0, 2, CfgEdgeKind::NotTaken)],
        );
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let du = compute_def_use(&ssa);

        let cond_val = ssa.blocks[0].instructions[0].dst.unwrap();
        assert_eq!(du.use_count(cond_val), 1);
        assert!(matches!(
            du.uses(cond_val)[0],
            UseSite::Terminator { block: 0 }
        ));
    }

    #[test]
    #[allow(dead_code)]
    fn parameter_of_does_not_panic_on_present_var() {
        // Exercise the test helper too.
        let raw = RawFunction {
            variables: vec![var(0, "x")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(0)),
                },
            }],
        };
        let cfg = synthetic_cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let _ = parameter_of(&ssa, "x");
        // Suppress unused imports/types in this test scope.
        let _ = SsaBlock {
            id: 0,
            predecessors: vec![],
            phis: vec![] as Vec<Phi>,
            instructions: vec![] as Vec<SsaInstruction>,
            terminator: dac_ir::ssa::SsaTerminator::Return { value: None },
        };
    }
}
