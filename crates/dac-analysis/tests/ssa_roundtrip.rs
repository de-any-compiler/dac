//! SSA round-trip semantic equivalence (B2.3, FR-11 done-when).
//!
//! For a small vector of synthetic functions, this test:
//!
//! 1. Interprets the raw, register-based form directly. This is the
//!    ground truth — what the lifter believes the function computes.
//! 2. Constructs SSA with `dac_analysis::ssa::construct_ssa` (which
//!    runs pruned phi placement + dominator-tree rename + local CSE).
//! 3. Interprets the SSA form, threading phi-arg selection through
//!    the predecessor block id at each block transition.
//! 4. Asserts both interpretations agree on the observable output —
//!    the value returned from the function.
//!
//! Hitting the same return value at every input both proves that
//! renaming preserved the original dataflow and that the pruned phi
//! placement plus value numbering did not drop a definition that the
//! return depended on.
//!
//! These small functions exercise:
//!
//! - Linear arithmetic (renaming chain, no phi).
//! - Branch-merge with phi at the join.
//! - Nested branches where the inner join is itself an argument to
//!   the outer join's phi.
//! - A while-style loop with phi at the header.
//! - A function with a redundant computation that CSE should collapse
//!   without changing semantics.

use std::collections::BTreeMap;

use dac_analysis::cfg::{BasicBlock, BlockId, Cfg, Edge, EdgeKind, Terminator};
use dac_analysis::dom::DominatorTree;
use dac_analysis::ssa::{
    construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
};
use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
use dac_ir::ssa::{Operand, SsaFunction, SsaOp, SsaTerminator, ValueId, Variable, VariableId};

// ---- shared CFG construction (the workspace shares synthetic_cfg
// inside the crate, but tests live outside it and need their own
// copy; this mirrors the production sort logic) ----

fn edge_sort_key(k: EdgeKind) -> u8 {
    match k {
        EdgeKind::Fall => 0,
        EdgeKind::Branch => 1,
        EdgeKind::Taken => 2,
        EdgeKind::NotTaken => 3,
    }
}

fn build_cfg_topology(n: usize, entry: BlockId, raw_edges: &[(BlockId, BlockId, EdgeKind)]) -> Cfg {
    let blocks: Vec<BasicBlock> = (0..n)
        .map(|i| BasicBlock {
            id: i as BlockId,
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
    edges.sort_by_key(|e| (e.from, edge_sort_key(e.kind), e.to));
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
        exits: Vec::new(),
        edges,
        unreachable: Vec::new(),
        evidence: ev,
    }
}

// ---- raw (variable-based) interpreter ----

#[derive(Default)]
struct RawState {
    vars: BTreeMap<VariableId, i64>,
}

fn read_raw(op: RawOperand, state: &RawState) -> i64 {
    match op {
        RawOperand::Variable(v) => state.vars.get(&v).copied().unwrap_or(0),
        RawOperand::Const(c) => c,
    }
}

fn eval_raw_kind(kind: &RawOpKind, state: &RawState) -> Option<i64> {
    Some(match kind {
        RawOpKind::Move { src } => read_raw(*src, state),
        RawOpKind::Add { lhs, rhs } => read_raw(*lhs, state).wrapping_add(read_raw(*rhs, state)),
        RawOpKind::Sub { lhs, rhs } => read_raw(*lhs, state).wrapping_sub(read_raw(*rhs, state)),
        RawOpKind::Mul { lhs, rhs } => read_raw(*lhs, state).wrapping_mul(read_raw(*rhs, state)),
        RawOpKind::And { lhs, rhs } => read_raw(*lhs, state) & read_raw(*rhs, state),
        RawOpKind::Or { lhs, rhs } => read_raw(*lhs, state) | read_raw(*rhs, state),
        RawOpKind::Xor { lhs, rhs } => read_raw(*lhs, state) ^ read_raw(*rhs, state),
        RawOpKind::Shl { lhs, rhs } => {
            read_raw(*lhs, state).wrapping_shl((read_raw(*rhs, state) & 63) as u32)
        }
        RawOpKind::Shr { lhs, rhs } => {
            (read_raw(*lhs, state) as u64).wrapping_shr((read_raw(*rhs, state) & 63) as u32) as i64
        }
        RawOpKind::Neg { src } => 0i64.wrapping_sub(read_raw(*src, state)),
        RawOpKind::Not { src } => !read_raw(*src, state),
        // Other ops (Compare, Load, Store, Call, Opaque) aren't used in
        // these round-trip programs.
        _ => return None,
    })
}

fn interpret_raw(raw: &RawFunction, initial: &BTreeMap<VariableId, i64>) -> i64 {
    let mut state = RawState {
        vars: initial.clone(),
    };
    let mut bid: BlockId = 0;
    loop {
        let block = &raw.blocks[bid as usize];
        for op in &block.ops {
            let v = eval_raw_kind(&op.kind, &state).expect("unsupported raw op in round-trip");
            if let Some(d) = op.dst {
                state.vars.insert(d, v);
            }
        }
        match &block.terminator {
            RawTerminator::Jump { target } => bid = *target as BlockId,
            RawTerminator::Branch {
                cond,
                taken,
                not_taken,
            } => {
                let c = read_raw(*cond, &state);
                bid = if c != 0 {
                    *taken as BlockId
                } else {
                    *not_taken as BlockId
                };
            }
            RawTerminator::Return { value } => {
                return value.map(|v| read_raw(v, &state)).unwrap_or(0);
            }
            RawTerminator::Indirect | RawTerminator::Unreachable => {
                panic!("interpreter hit non-deterministic terminator");
            }
        }
    }
}

// ---- SSA interpreter ----

#[derive(Default)]
struct SsaState {
    values: BTreeMap<ValueId, i64>,
}

fn read_ssa(op: Operand, state: &SsaState) -> i64 {
    match op {
        Operand::Value(v) => state.values.get(&v).copied().unwrap_or(0),
        Operand::Const(c) => c,
        Operand::Undef => 0,
    }
}

fn eval_ssa_op(op: &SsaOp, state: &SsaState) -> Option<i64> {
    Some(match op {
        SsaOp::Move { src } => read_ssa(*src, state),
        SsaOp::Add { lhs, rhs } => read_ssa(*lhs, state).wrapping_add(read_ssa(*rhs, state)),
        SsaOp::Sub { lhs, rhs } => read_ssa(*lhs, state).wrapping_sub(read_ssa(*rhs, state)),
        SsaOp::Mul { lhs, rhs } => read_ssa(*lhs, state).wrapping_mul(read_ssa(*rhs, state)),
        SsaOp::And { lhs, rhs } => read_ssa(*lhs, state) & read_ssa(*rhs, state),
        SsaOp::Or { lhs, rhs } => read_ssa(*lhs, state) | read_ssa(*rhs, state),
        SsaOp::Xor { lhs, rhs } => read_ssa(*lhs, state) ^ read_ssa(*rhs, state),
        SsaOp::Shl { lhs, rhs } => {
            read_ssa(*lhs, state).wrapping_shl((read_ssa(*rhs, state) & 63) as u32)
        }
        SsaOp::Shr { lhs, rhs } => {
            (read_ssa(*lhs, state) as u64).wrapping_shr((read_ssa(*rhs, state) & 63) as u32) as i64
        }
        SsaOp::Neg { src } => 0i64.wrapping_sub(read_ssa(*src, state)),
        SsaOp::Not { src } => !read_ssa(*src, state),
        _ => return None,
    })
}

fn interpret_ssa(ssa: &SsaFunction, initial: &BTreeMap<VariableId, i64>) -> i64 {
    let mut state = SsaState::default();
    // Seed Parameter values from the initial map. Parameters appear as
    // values with `ValueSource::Parameter { variable }`.
    for vd in &ssa.values {
        if let dac_ir::ssa::ValueSource::Parameter { variable } = vd.source {
            let initial_value = initial.get(&variable).copied().unwrap_or(0);
            state.values.insert(vd.id, initial_value);
        }
    }

    let mut prev: Option<u32> = None;
    let mut bid: u32 = ssa.entry;
    loop {
        let block = ssa.block(bid);
        // Phi selection: walk each phi and pick the operand whose
        // predecessor matches the block we just came from. The very
        // first iteration has no predecessor — the entry block must
        // not carry any phis (Cytron + dom-tree guarantees this).
        for phi in &block.phis {
            let pred = prev.expect("phi found at entry block — invalid SSA");
            let pick = phi
                .incoming
                .iter()
                .find(|(p, _)| *p == pred)
                .map(|(_, op)| *op)
                .expect("no phi incoming for current predecessor");
            let v = read_ssa(pick, &state);
            state.values.insert(phi.dst, v);
        }
        for ins in &block.instructions {
            let v = eval_ssa_op(&ins.op, &state).expect("unsupported SSA op in round-trip");
            if let Some(d) = ins.dst {
                state.values.insert(d, v);
            }
        }
        match &block.terminator {
            SsaTerminator::Jump { target } => {
                prev = Some(bid);
                bid = *target;
            }
            SsaTerminator::Branch {
                cond,
                taken,
                not_taken,
            } => {
                let c = read_ssa(*cond, &state);
                prev = Some(bid);
                bid = if c != 0 { *taken } else { *not_taken };
            }
            SsaTerminator::Return { value } => {
                return value.map(|v| read_ssa(v, &state)).unwrap_or(0);
            }
            SsaTerminator::Indirect | SsaTerminator::Unreachable => {
                panic!("interpreter hit non-deterministic terminator");
            }
        }
    }
}

// ---- helpers for building raw programs ----

fn var_table(names: &[&str]) -> Vec<Variable> {
    names
        .iter()
        .enumerate()
        .map(|(i, n)| Variable {
            id: i as VariableId,
            name: (*n).to_string(),
            width_bits: 64,
        })
        .collect()
}

fn assert_round_trip(
    cfg: &Cfg,
    raw: &RawFunction,
    inputs: &[BTreeMap<VariableId, i64>],
    label: &str,
) {
    let doms = DominatorTree::build(cfg);
    let ssa = construct_ssa(cfg, &doms, raw);
    for (i, init) in inputs.iter().enumerate() {
        let expected = interpret_raw(raw, init);
        let actual = interpret_ssa(&ssa, init);
        assert_eq!(
            actual, expected,
            "{label}: input #{i} {init:?} disagreed (raw={expected}, ssa={actual})"
        );
    }
}

// ---- the round-trip cases ----

#[test]
fn linear_chain_renames_preserve_semantics() {
    // Block 0:
    //   t = a + b
    //   t = t * c
    //   ret t
    let cfg = build_cfg_topology(1, 0, &[]);
    let raw = RawFunction {
        variables: var_table(&["a", "b", "c", "t"]),
        blocks: vec![RawBlock {
            ops: vec![
                RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Variable(1),
                    },
                },
                RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Mul {
                        lhs: RawOperand::Variable(3),
                        rhs: RawOperand::Variable(2),
                    },
                },
            ],
            terminator: RawTerminator::Return {
                value: Some(RawOperand::Variable(3)),
            },
        }],
    };
    let inputs: Vec<BTreeMap<VariableId, i64>> = vec![
        BTreeMap::from([(0, 0), (1, 0), (2, 0)]),
        BTreeMap::from([(0, 1), (1, 2), (2, 3)]),
        BTreeMap::from([(0, -1), (1, -2), (2, 3)]),
        BTreeMap::from([(0, 5), (1, 7), (2, 11)]),
    ];
    assert_round_trip(&cfg, &raw, &inputs, "linear_chain");
}

#[test]
fn diamond_phi_picks_correct_arm() {
    // Block 0: if (c != 0) goto 1 else goto 2
    // Block 1: t = a + 10; jmp 3
    // Block 2: t = a - 10; jmp 3
    // Block 3: ret t   <-- phi merges the two arms
    let cfg = build_cfg_topology(
        4,
        0,
        &[
            (0, 1, EdgeKind::Taken),
            (0, 2, EdgeKind::NotTaken),
            (1, 3, EdgeKind::Branch),
            (2, 3, EdgeKind::Branch),
        ],
    );
    let raw = RawFunction {
        variables: var_table(&["a", "c", "t"]),
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
                    dst: Some(2),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Const(10),
                    },
                }],
                terminator: RawTerminator::Jump { target: 3 },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(2),
                    kind: RawOpKind::Sub {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Const(10),
                    },
                }],
                terminator: RawTerminator::Jump { target: 3 },
            },
            RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            },
        ],
    };
    let inputs: Vec<BTreeMap<VariableId, i64>> = vec![
        BTreeMap::from([(0, 0), (1, 0)]),
        BTreeMap::from([(0, 0), (1, 1)]),
        BTreeMap::from([(0, 42), (1, 0)]),
        BTreeMap::from([(0, 42), (1, 1)]),
        BTreeMap::from([(0, -100), (1, 7)]),
    ];
    assert_round_trip(&cfg, &raw, &inputs, "diamond_phi");
}

#[test]
fn nested_branches_phi_at_outer_join() {
    // Block 0: if (c1) goto 1 else goto 5  (outer if)
    // Block 1: if (c2) goto 2 else goto 3  (inner if)
    // Block 2: t = a + 1; jmp 4
    // Block 3: t = a + 2; jmp 4
    // Block 4: t = t * 10; jmp 6   (inner join, then continue)
    // Block 5: t = a - 1; jmp 6
    // Block 6: ret t              (outer join — phi over 4 and 5)
    let cfg = build_cfg_topology(
        7,
        0,
        &[
            (0, 1, EdgeKind::Taken),
            (0, 5, EdgeKind::NotTaken),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 4, EdgeKind::Branch),
            (3, 4, EdgeKind::Branch),
            (4, 6, EdgeKind::Branch),
            (5, 6, EdgeKind::Branch),
        ],
    );
    let raw = RawFunction {
        variables: var_table(&["a", "c1", "c2", "t"]),
        blocks: vec![
            RawBlock {
                ops: vec![],
                terminator: RawTerminator::Branch {
                    cond: RawOperand::Variable(1),
                    taken: 1,
                    not_taken: 5,
                },
            },
            RawBlock {
                ops: vec![],
                terminator: RawTerminator::Branch {
                    cond: RawOperand::Variable(2),
                    taken: 2,
                    not_taken: 3,
                },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Const(1),
                    },
                }],
                terminator: RawTerminator::Jump { target: 4 },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Const(2),
                    },
                }],
                terminator: RawTerminator::Jump { target: 4 },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Mul {
                        lhs: RawOperand::Variable(3),
                        rhs: RawOperand::Const(10),
                    },
                }],
                terminator: RawTerminator::Jump { target: 6 },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Sub {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Const(1),
                    },
                }],
                terminator: RawTerminator::Jump { target: 6 },
            },
            RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            },
        ],
    };
    let inputs: Vec<BTreeMap<VariableId, i64>> = vec![
        BTreeMap::from([(0, 5), (1, 0), (2, 0)]),
        BTreeMap::from([(0, 5), (1, 1), (2, 0)]),
        BTreeMap::from([(0, 5), (1, 1), (2, 1)]),
        BTreeMap::from([(0, 100), (1, 1), (2, 1)]),
        BTreeMap::from([(0, -42), (1, 0), (2, 1)]),
    ];
    assert_round_trip(&cfg, &raw, &inputs, "nested_branches");
}

#[test]
fn while_loop_phi_preserves_iteration() {
    // Pseudocode:
    //   sum = 0
    //   i = 0
    //   while (i < n) {
    //       sum = sum + i
    //       i = i + 1
    //   }
    //   ret sum
    //
    // CFG:
    //   0: sum = 0; i = 0; jmp 1
    //   1: (header) — phi for sum and i
    //      cond = (i < n)
    //      if (cond) jmp 2 else jmp 3
    //   2: sum = sum + i; i = i + 1; jmp 1
    //   3: ret sum
    //
    // We don't have Compare implemented in the interpreter, but the
    // cond can come from `n - i` as a stand-in: if positive, we go
    // around (Branch's "taken" treats non-zero as true). For a small
    // test we use `n` as the cond directly and `n` is decremented
    // each iter to make the loop terminate.
    let cfg = build_cfg_topology(
        4,
        0,
        &[
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 1, EdgeKind::Branch),
        ],
    );
    // Variables: 0=n_initial (input), 1=sum, 2=i_remaining, 3=cond
    let raw = RawFunction {
        variables: var_table(&["n", "sum", "i_remaining", "cond"]),
        blocks: vec![
            RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Move {
                            src: RawOperand::Const(0),
                        },
                    },
                    RawOp {
                        dst: Some(2),
                        kind: RawOpKind::Move {
                            src: RawOperand::Variable(0),
                        },
                    },
                ],
                terminator: RawTerminator::Jump { target: 1 },
            },
            RawBlock {
                ops: vec![RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Move {
                        src: RawOperand::Variable(2),
                    },
                }],
                terminator: RawTerminator::Branch {
                    cond: RawOperand::Variable(3),
                    taken: 2,
                    not_taken: 3,
                },
            },
            RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Add {
                            lhs: RawOperand::Variable(1),
                            rhs: RawOperand::Variable(2),
                        },
                    },
                    RawOp {
                        dst: Some(2),
                        kind: RawOpKind::Sub {
                            lhs: RawOperand::Variable(2),
                            rhs: RawOperand::Const(1),
                        },
                    },
                ],
                terminator: RawTerminator::Jump { target: 1 },
            },
            RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            },
        ],
    };
    // Sum of 1..=n. For n=0,1,5,10 → 0, 1, 15, 55.
    let inputs: Vec<BTreeMap<VariableId, i64>> = vec![
        BTreeMap::from([(0, 0)]),
        BTreeMap::from([(0, 1)]),
        BTreeMap::from([(0, 5)]),
        BTreeMap::from([(0, 10)]),
    ];
    assert_round_trip(&cfg, &raw, &inputs, "while_loop");
}

#[test]
fn cse_does_not_change_observable_output() {
    // Block 0:
    //   t0 = a + b
    //   t1 = a + b    (redundant — CSE)
    //   t2 = t0 - t1  (folds to 0)
    //   ret t2
    let cfg = build_cfg_topology(1, 0, &[]);
    let raw = RawFunction {
        variables: var_table(&["a", "b", "t0", "t1", "t2"]),
        blocks: vec![RawBlock {
            ops: vec![
                RawOp {
                    dst: Some(2),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Variable(1),
                    },
                },
                RawOp {
                    dst: Some(3),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(0),
                        rhs: RawOperand::Variable(1),
                    },
                },
                RawOp {
                    dst: Some(4),
                    kind: RawOpKind::Sub {
                        lhs: RawOperand::Variable(2),
                        rhs: RawOperand::Variable(3),
                    },
                },
            ],
            terminator: RawTerminator::Return {
                value: Some(RawOperand::Variable(4)),
            },
        }],
    };
    let inputs: Vec<BTreeMap<VariableId, i64>> = vec![
        BTreeMap::from([(0, 0), (1, 0)]),
        BTreeMap::from([(0, 7), (1, 9)]),
        BTreeMap::from([(0, -3), (1, 100)]),
    ];
    assert_round_trip(&cfg, &raw, &inputs, "cse_round_trip");
}
