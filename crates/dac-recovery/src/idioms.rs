//! Idiom recognition (B3.3, FR-18, spec §11.4).
//!
//! A side-table pass that scans an [`SsaFunction`] and surfaces
//! source-level idioms the deterministic pipeline can already justify
//! from structural evidence — without rewriting the IR. Per PLAN.md the
//! whole point of this batch is that "each idiom is a pass that proposes
//! annotations; non-matches do not rewrite the IR" — output is a
//! proposal table only.
//!
//! ## What ships in this batch
//!
//! - **Switch-table recognition.** Compiler-emitted jump tables on
//!   x86-64. These appear in SSA as a block whose terminator is
//!   [`SsaTerminator::Indirect`] and whose tail computes the indirect
//!   target as `Load(width = w, Add(table_base, Mul(index, stride)))`
//!   or the [`SsaOp::Shl`] variant for power-of-two strides. Both the
//!   "absolute target" table (`stride == sizeof(ptr) == 8`) and the
//!   "32-bit relative offset" table (`stride == 4`) match.
//! - **Bound detection.** When a single predecessor branches into the
//!   indirect block on a [`SsaOp::Compare`] of [`CompareKind::Ult`] or
//!   [`CompareKind::Ule`] against the scrutinee, the constant bound is
//!   recorded. Backends can render `case 0:` … `case N-1:` ranges from
//!   the bound when subsequent passes resolve individual table entries.
//!
//! Together these satisfy the PLAN.md "Done when": switch recovery
//! handles compiler-emitted jump tables on x86-64.
//!
//! ## What deliberately doesn't land yet
//!
//! The PLAN.md deliverables list for B3.3 also names error-handling
//! patterns, ref-counting, and simple state machines. Per the standing
//! pattern from B3.2 ("union recovery deferred", "nested structs not
//! chased"), those land as separate functions on this same
//! [`RecoveredIdioms`] table in subsequent batches:
//!
//! | Idiom kind          | Status | Notes                                                                            |
//! | ------------------- | ------ | -------------------------------------------------------------------------------- |
//! | Switch tables       | this   | Pattern-matches `Indirect` + indexed `Load`; the rubric.                         |
//! | Error guard returns | next   | `Compare(result, 0) → Return` shape, seeded once dac-knowledge errno is wired.   |
//! | Ref-counting        | M3 end | Needs atomic / lock-prefix modelling at the SSA layer first.                     |
//! | State machines      | M3 end | Needs phi-of-state-constants tracking on top of the type lattice (B2.6).         |
//!
//! Nothing here precludes them: each future detector adds a new field
//! to [`RecoveredIdioms`] and its own builder; non-matches return an
//! empty map, so the channel degrades gracefully.
//!
//! ## What this pass never does
//!
//! - **Mutate IR.** Output is purely additive — the Instruction IR and
//!   SSA IR remain the source of truth (I-1). A separate lowering pass
//!   is responsible for collapsing a [`SwitchTableIdiom`] into a
//!   [`dac_ir::sem::Stmt::Switch`] in the Semantic IR.
//! - **Resolve table entries.** A `SwitchTableIdiom` records the
//!   *shape* of the jump table (base, stride, scrutinee). Resolving
//!   the actual entry addresses requires reading the binary's `.rodata`
//!   (or the relocation table) and lives in a downstream pass — likely
//!   B3.4 once the annotation channel can carry table data.
//! - **Touch confidence sources.** Every proposal carries
//!   [`Source::Derived`] — the structural shape is observable but the
//!   claim "this is a switch statement" is derived from it (I-3). A
//!   later pass that has resolved the table entries may join with a
//!   higher-source confidence; this pass never does so on its own.
//!
//! ## Determinism (NFR-9, I-4)
//!
//! Iteration walks SSA blocks in ascending [`SsaBlockId`] order. The
//! output [`RecoveredIdioms::switch_tables`] is a [`BTreeMap`], so the
//! same SSA function produces the same byte-for-byte output across
//! runs.

use std::collections::BTreeMap;

use dac_core::{Confidence, Source};
use dac_ir::ssa::{
    CompareKind, Operand, SsaBlockId, SsaFunction, SsaOp, SsaTerminator, ValueId, ValueSource,
};

/// Confidence value attached to a switch-table proposal recovered from
/// the indirect-jump + indexed-load shape on x86-64.
pub const SWITCH_TABLE_CONFIDENCE: f32 = 0.70;

/// Output of [`recover_idioms`].
///
/// `Eq` is intentionally not derived: idiom records carry a
/// [`Confidence`] (f32-backed), which only implements [`PartialEq`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecoveredIdioms {
    /// Switch-table proposals, keyed by the [`SsaBlockId`] of the block
    /// whose [`SsaTerminator::Indirect`] terminator anchors the
    /// dispatch.
    pub switch_tables: BTreeMap<SsaBlockId, SwitchTableIdiom>,
}

impl RecoveredIdioms {
    /// Total number of idiom proposals across every kind.
    #[must_use]
    pub fn len(&self) -> usize {
        self.switch_tables.len()
    }

    /// True when no proposals were recovered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// One recovered switch-table dispatch.
///
/// Records the *shape* of the table — base, stride, scrutinee, optional
/// upper bound — but not the resolved entry addresses. Resolving entries
/// requires reading the binary section that backs `table_base_const`
/// and lives in a downstream pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchTableIdiom {
    /// Block whose [`SsaTerminator::Indirect`] terminator dispatches
    /// via the table.
    pub source_block: SsaBlockId,
    /// SSA value the dispatch indexes on — typically the function's
    /// `case` expression after any normalising offset.
    pub scrutinee: ValueId,
    /// Table base, when the address decomposition produced a constant
    /// offset (`Add(table_base_const, Mul(scrutinee, stride))`).
    /// `None` for PIC-style tables whose base is itself an SSA value
    /// (the constant lives at the relocation site, not in the
    /// instruction stream).
    pub table_base_const: Option<i64>,
    /// Stride between consecutive table entries in bytes — the `c`
    /// from `Mul(idx, c)` or `1 << k` from `Shl(idx, k)`.
    pub element_stride: u64,
    /// Width of the load that reads one entry. Typically equal to
    /// [`Self::element_stride`] for absolute-pointer tables and `4` for
    /// `int32_t`-relative tables.
    pub element_width: u8,
    /// Upper bound from a preceding [`SsaOp::Compare`] of
    /// [`CompareKind::Ult`] or [`CompareKind::Ule`] in a unique
    /// predecessor block — `Some(N)` means the scrutinee is provably
    /// in `[0, N)` (Ult) or `[0, N]` (Ule). `None` when no such bound
    /// was found in a single-predecessor chain.
    pub bound: Option<i64>,
    /// Confidence in the proposal. Always [`Source::Derived`] from this
    /// pass (I-3).
    pub confidence: Confidence,
}

/// Run idiom recognition on `ssa`.
///
/// The function is total: it walks every block and emits whatever the
/// pattern matchers fire on. Functions with no idioms produce an empty
/// [`RecoveredIdioms`]. The IR is never mutated (I-1).
#[must_use]
pub fn recover_idioms(ssa: &SsaFunction) -> RecoveredIdioms {
    RecoveredIdioms {
        switch_tables: recover_switch_tables(ssa),
    }
}

// ---- Switch-table recognition ---------------------------------------

/// Scan every block whose terminator is [`SsaTerminator::Indirect`]
/// and pattern-match the last [`SsaOp::Load`] in its tail against the
/// indexed-address shape. When the match fires, walk the predecessor
/// graph one hop back to pick up the bound from a guarding
/// [`SsaOp::Compare`].
fn recover_switch_tables(ssa: &SsaFunction) -> BTreeMap<SsaBlockId, SwitchTableIdiom> {
    let value_const = collect_value_constants(ssa);
    let confidence = Confidence::new(SWITCH_TABLE_CONFIDENCE, Source::Derived);
    let mut out = BTreeMap::new();

    for block in &ssa.blocks {
        if !matches!(block.terminator, SsaTerminator::Indirect) {
            continue;
        }
        let Some((scrutinee, table_base_const, stride, width)) =
            last_indexed_load(block, ssa, &value_const)
        else {
            continue;
        };
        let bound = lookup_bound(block.id, scrutinee, ssa, &value_const);
        out.insert(
            block.id,
            SwitchTableIdiom {
                source_block: block.id,
                scrutinee,
                table_base_const,
                element_stride: stride,
                element_width: width,
                bound,
                confidence,
            },
        );
    }
    out
}

/// Find the last [`SsaOp::Load`] in the block whose address decomposes
/// to `Add(base, scaled_index)` with `scaled_index` matching
/// `Mul(idx, c)` or `Shl(idx, k)` (stride ≥ 2). When the base resolves
/// to a constant we return it; otherwise the table base lives in an
/// SSA value and we record `None`.
fn last_indexed_load(
    block: &dac_ir::ssa::SsaBlock,
    ssa: &SsaFunction,
    value_const: &BTreeMap<ValueId, i64>,
) -> Option<(ValueId, Option<i64>, u64, u8)> {
    for ins in block.instructions.iter().rev() {
        let SsaOp::Load { address, width } = &ins.op else {
            continue;
        };
        let Operand::Value(addr_val) = address else {
            continue;
        };
        let Some(def) = lookup_def_op(*addr_val, ssa) else {
            continue;
        };
        let SsaOp::Add { lhs, rhs } = def else {
            continue;
        };
        if let Some((scrutinee, stride, base_const)) =
            split_indexed_add(*lhs, *rhs, ssa, value_const)
        {
            return Some((scrutinee, base_const, stride, *width));
        }
    }
    None
}

/// Inspect the two operands of an `Add` and try to identify one as the
/// scaled-index leg (`Mul(idx, c)` or `Shl(idx, k)`) and the other as
/// the table base. Returns `(scrutinee, stride, table_base_const)`.
/// `table_base_const` is `Some(c)` only when the base operand is a
/// constant (or a `Move` of a constant).
fn split_indexed_add(
    lhs: Operand,
    rhs: Operand,
    ssa: &SsaFunction,
    value_const: &BTreeMap<ValueId, i64>,
) -> Option<(ValueId, u64, Option<i64>)> {
    for (scaled_op, base_op) in [(lhs, rhs), (rhs, lhs)] {
        let Operand::Value(scaled) = scaled_op else {
            continue;
        };
        let Some(def) = lookup_def_op(scaled, ssa) else {
            continue;
        };
        let Some((scrutinee, stride)) = scaled_index(def, value_const) else {
            continue;
        };
        if stride < 2 {
            continue;
        }
        let base_const = const_operand(base_op, value_const);
        return Some((scrutinee, stride, base_const));
    }
    None
}

/// Decompose `def` against `Mul(idx, c)` / `Shl(idx, k)`. Returns
/// `(idx, stride)` when the shape matches.
fn scaled_index(op: &SsaOp, value_const: &BTreeMap<ValueId, i64>) -> Option<(ValueId, u64)> {
    match op {
        SsaOp::Mul { lhs, rhs } => {
            for (idx_op, const_op) in [(lhs, rhs), (rhs, lhs)] {
                let Operand::Value(idx) = idx_op else {
                    continue;
                };
                if let Some(c) = const_operand(*const_op, value_const) {
                    if c > 0 {
                        return Some((*idx, c as u64));
                    }
                }
            }
            None
        }
        SsaOp::Shl { lhs, rhs } => {
            let Operand::Value(idx) = lhs else {
                return None;
            };
            let c = const_operand(*rhs, value_const)?;
            if (0..64).contains(&c) {
                Some((*idx, 1u64 << c))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Walk one hop back to a predecessor that branches into `target_block`
/// on a [`CompareKind::Ult`] / [`CompareKind::Ule`] check against
/// `scrutinee`. When found, return the constant bound. When the
/// predecessor set is empty or ambiguous, return `None`.
fn lookup_bound(
    target_block: SsaBlockId,
    scrutinee: ValueId,
    ssa: &SsaFunction,
    value_const: &BTreeMap<ValueId, i64>,
) -> Option<i64> {
    let preds = &ssa.block(target_block).predecessors;
    if preds.len() != 1 {
        return None;
    }
    let pred_id = preds[0];
    let pred = ssa.block(pred_id);
    let SsaTerminator::Branch {
        cond,
        taken,
        not_taken,
    } = pred.terminator
    else {
        return None;
    };
    // The bound only constrains the scrutinee when the *taken* edge
    // leads to the dispatch block. A `not_taken` arrival means the
    // compare excluded the in-range case, which is the default-arm
    // path — no bound carried on this edge.
    if taken != target_block || not_taken == target_block {
        return None;
    }
    let Operand::Value(cond_val) = cond else {
        return None;
    };
    let def = lookup_def_op(cond_val, ssa)?;
    let SsaOp::Compare { kind, lhs, rhs } = def else {
        return None;
    };
    if !matches!(kind, CompareKind::Ult | CompareKind::Ule) {
        return None;
    }
    let Operand::Value(idx) = lhs else {
        return None;
    };
    if *idx != scrutinee {
        return None;
    }
    const_operand(*rhs, value_const)
}

// ---- Shared SSA helpers ---------------------------------------------

/// Collect every SSA value defined as `Move { src: Const(c) }` so the
/// pattern matchers can treat "constant materialised by a Move" the
/// same as a literal [`Operand::Const`].
fn collect_value_constants(ssa: &SsaFunction) -> BTreeMap<ValueId, i64> {
    let mut out = BTreeMap::new();
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let Some(dst) = ins.dst else { continue };
            if let SsaOp::Move {
                src: Operand::Const(c),
            } = &ins.op
            {
                out.insert(dst, *c);
            }
        }
    }
    out
}

/// Look up the defining [`SsaOp`] for `value`, when it has one.
/// Returns `None` for phi-defined and parameter-defined values — neither
/// participates in the patterns matched here.
fn lookup_def_op(value: ValueId, ssa: &SsaFunction) -> Option<&SsaOp> {
    if let ValueSource::Instruction { block, index } = ssa.value(value).source {
        return Some(&ssa.blocks[block as usize].instructions[index as usize].op);
    }
    None
}

/// Resolve `op` to a concrete `i64` constant when one is observable —
/// either a literal [`Operand::Const`] or a value defined by
/// `Move { src: Const(c) }`. Returns `None` for `Undef`, phi-defined
/// values, and parameters.
fn const_operand(op: Operand, value_const: &BTreeMap<ValueId, i64>) -> Option<i64> {
    match op {
        Operand::Const(c) => Some(c),
        Operand::Value(v) => value_const.get(&v).copied(),
        Operand::Undef => None,
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
    use dac_ir::ssa::{Variable, VariableId};

    use super::*;

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

    fn mov_c(dst: VariableId, c: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Const(c),
            },
        }
    }

    fn add_vv(dst: VariableId, lhs: VariableId, rhs: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Variable(rhs),
            },
        }
    }

    fn mul_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Mul {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn shl_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Shl {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
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

    fn cmp_vc(dst: VariableId, kind: CompareKind, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Compare {
                kind,
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn build(raw: RawFunction, n_blocks: usize, edges: &[(u32, u32, EdgeKind)]) -> SsaFunction {
        let cfg = synthetic_cfg(n_blocks, 0, edges);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    // --- Switch-table recognition ------------------------------------

    /// Canonical x86-64 jump table: `table_base + idx * 8`, then load
    /// 8 bytes, then `jmp`. This is the PLAN rubric.
    #[test]
    fn indirect_block_with_mul_indexed_load_is_a_switch_table() {
        // variables: 0 = idx, 1 = table_base, 2 = scaled, 3 = addr, 4 = target
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404020), // table_base
                    mul_vc(2, 0, 8),    // idx * 8
                    add_vv(3, 1, 2),    // table_base + idx*8
                    load(4, 3, 8),      // *(target ptr)
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_idioms(&ssa);
        assert_eq!(recovered.switch_tables.len(), 1);
        let s = recovered.switch_tables.get(&0).expect("block 0 switch");
        assert_eq!(s.source_block, 0);
        assert_eq!(s.element_stride, 8);
        assert_eq!(s.element_width, 8);
        assert_eq!(s.table_base_const, Some(0x404020));
        assert_eq!(s.bound, None);
        assert_eq!(s.confidence.value(), SWITCH_TABLE_CONFIDENCE);
        assert_eq!(s.confidence.source(), Source::Derived);
    }

    /// Power-of-two stride via `Shl(idx, 3) == idx * 8`.
    #[test]
    fn indirect_block_with_shl_indexed_load_is_a_switch_table() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404100),
                    shl_vc(2, 0, 3), // idx << 3 == idx * 8
                    add_vv(3, 1, 2),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_idioms(&ssa);
        let s = recovered
            .switch_tables
            .get(&0)
            .expect("block 0 switch via Shl");
        assert_eq!(s.element_stride, 8);
        assert_eq!(s.element_width, 8);
        assert_eq!(s.table_base_const, Some(0x404100));
    }

    /// `int32_t`-relative tables — stride 4, width 4.
    #[test]
    fn indirect_block_with_stride_4_table_records_width_4() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "off"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404200),
                    mul_vc(2, 0, 4),
                    add_vv(3, 1, 2),
                    load(4, 3, 4),
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        let s = recover_idioms(&ssa)
            .switch_tables
            .remove(&0)
            .expect("block 0 stride-4 table");
        assert_eq!(s.element_stride, 4);
        assert_eq!(s.element_width, 4);
    }

    /// A bounded `cmp idx, N; ja default` predecessor pins the upper
    /// bound on the in-range arrival edge.
    #[test]
    fn predecessor_compare_supplies_upper_bound() {
        // Block 0: cmp idx, 16; if Ult goto block 1 (dispatch) else
        //                       goto block 2 (default).
        // Block 1: load table[idx*8]; jmp [load result]
        // Block 2: return
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
                var(5, "cmp"),
            ],
            blocks: vec![
                RawBlock {
                    ops: vec![cmp_vc(5, CompareKind::Ult, 0, 16)],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(5),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![
                        mov_c(1, 0x404300),
                        mul_vc(2, 0, 8),
                        add_vv(3, 1, 2),
                        load(4, 3, 8),
                    ],
                    terminator: RawTerminator::Indirect,
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let ssa = build(
            raw,
            3,
            &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)],
        );
        let s = recover_idioms(&ssa)
            .switch_tables
            .remove(&1)
            .expect("dispatch in block 1");
        assert_eq!(s.bound, Some(16));
    }

    /// `Ule` is also a valid bounding compare (`<=` rather than `<`).
    #[test]
    fn ule_compare_also_supplies_bound() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
                var(5, "cmp"),
            ],
            blocks: vec![
                RawBlock {
                    ops: vec![cmp_vc(5, CompareKind::Ule, 0, 7)],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(5),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![
                        mov_c(1, 0x404400),
                        mul_vc(2, 0, 8),
                        add_vv(3, 1, 2),
                        load(4, 3, 8),
                    ],
                    terminator: RawTerminator::Indirect,
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let ssa = build(
            raw,
            3,
            &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)],
        );
        let s = recover_idioms(&ssa)
            .switch_tables
            .remove(&1)
            .expect("dispatch in block 1");
        assert_eq!(s.bound, Some(7));
    }

    /// A signed `Lt` is not a bounding check — the dispatch could be
    /// entered with a negative index, which a `Lt` against `N` does
    /// not forbid. The bound must be absent.
    #[test]
    fn signed_lt_does_not_supply_bound() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
                var(5, "cmp"),
            ],
            blocks: vec![
                RawBlock {
                    ops: vec![cmp_vc(5, CompareKind::Lt, 0, 16)],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(5),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![
                        mov_c(1, 0x404500),
                        mul_vc(2, 0, 8),
                        add_vv(3, 1, 2),
                        load(4, 3, 8),
                    ],
                    terminator: RawTerminator::Indirect,
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let ssa = build(
            raw,
            3,
            &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)],
        );
        let s = recover_idioms(&ssa)
            .switch_tables
            .remove(&1)
            .expect("dispatch in block 1");
        assert_eq!(s.bound, None);
    }

    /// A non-`Indirect` terminator never produces a switch proposal,
    /// even if the block contains an indexed load.
    #[test]
    fn return_terminator_does_not_produce_switch() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404600),
                    mul_vc(2, 0, 8),
                    add_vv(3, 1, 2),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        assert!(recover_idioms(&ssa).switch_tables.is_empty());
    }

    /// An `Indirect` block whose tail has no indexed load (e.g. a
    /// bare `jmp rax` from a function pointer) produces no proposal.
    #[test]
    fn indirect_without_indexed_load_produces_no_proposal() {
        let raw = RawFunction {
            variables: vec![var(0, "fp")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        assert!(recover_idioms(&ssa).switch_tables.is_empty());
    }

    /// Stride 1 is rejected — indistinguishable from plain pointer
    /// arithmetic, mirroring the array-recovery rule in [`super::super::structs`].
    #[test]
    fn stride_one_is_not_a_switch_table() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404700),
                    mul_vc(2, 0, 1), // stride 1
                    add_vv(3, 1, 2),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        assert!(recover_idioms(&ssa).switch_tables.is_empty());
    }

    /// Same SSA input → same idiom output, byte-for-byte (NFR-9).
    #[test]
    fn recovery_is_deterministic_across_runs() {
        let mk = || RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404800),
                    mul_vc(2, 0, 8),
                    add_vv(3, 1, 2),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa_a = build(mk(), 1, &[]);
        let ssa_b = build(mk(), 1, &[]);
        let a = recover_idioms(&ssa_a);
        let b = recover_idioms(&ssa_b);
        assert_eq!(a, b);
    }

    /// Empty function → empty output. Degraded inputs never error
    /// (I-4 graceful degradation).
    #[test]
    fn empty_function_produces_empty_output() {
        let raw = RawFunction {
            variables: vec![var(0, "rax")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let r = recover_idioms(&ssa);
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    /// Every proposal carries [`Source::Derived`] — this pass never
    /// claims [`Source::Observed`] (I-3).
    #[test]
    fn every_recovered_confidence_is_derived() {
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(1, 0x404900),
                    mul_vc(2, 0, 8),
                    add_vv(3, 1, 2),
                    load(4, 3, 8),
                ],
                terminator: RawTerminator::Indirect,
            }],
        };
        let ssa = build(raw, 1, &[]);
        let r = recover_idioms(&ssa);
        for s in r.switch_tables.values() {
            assert_eq!(s.confidence.source(), Source::Derived);
            assert!(s.confidence.value() > 0.0 && s.confidence.value() < 1.0);
        }
    }

    /// PLAN.md rubric: a hand-built jump-table-style function decompiles
    /// to a recovered switch with the right shape. This is the
    /// "compiler-emitted jump tables on x86-64" line in the batch's
    /// done-when.
    #[test]
    fn hand_built_jump_table_round_trip() {
        // Approximates:
        //   if (idx < 4) { jmp table[idx]; } else { return; }
        //
        // Block 0: cmp idx, 4 (Ult); taken -> block 1, not_taken -> block 2
        // Block 1: mov tbl, 0x404000; addr = tbl + idx*8; target = *addr; jmp target
        // Block 2: return
        let raw = RawFunction {
            variables: vec![
                var(0, "idx"),
                var(1, "tbl"),
                var(2, "scl"),
                var(3, "adr"),
                var(4, "tgt"),
                var(5, "cmp"),
            ],
            blocks: vec![
                RawBlock {
                    ops: vec![cmp_vc(5, CompareKind::Ult, 0, 4)],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(5),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                RawBlock {
                    ops: vec![
                        mov_c(1, 0x404000),
                        mul_vc(2, 0, 8),
                        add_vv(3, 1, 2),
                        load(4, 3, 8),
                    ],
                    terminator: RawTerminator::Indirect,
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let ssa = build(
            raw,
            3,
            &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)],
        );
        let r = recover_idioms(&ssa);
        let s = r.switch_tables.get(&1).expect("dispatch at block 1");
        assert_eq!(s.element_stride, 8);
        assert_eq!(s.element_width, 8);
        assert_eq!(s.table_base_const, Some(0x404000));
        assert_eq!(s.bound, Some(4));
        // 1 proposal — only block 1 carries the dispatch.
        assert_eq!(r.len(), 1);
    }
}
