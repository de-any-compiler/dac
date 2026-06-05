//! SSA construction (B2.3, FR-11).
//!
//! [`construct_ssa`] turns a [`Cfg`] plus a lifter-supplied
//! [`RawFunction`] (per-block raw operation streams keyed by abstract
//! variable id) into an [`SsaFunction`] in pruned SSA form:
//!
//! 1. **Dominance frontiers.** Computed from the dominator tree using
//!    the standard Cytron-Ferrante-Rosen-Wegman-Zadeck walk:
//!    `for each block b with > 1 predecessor, for each predecessor p,
//!    runner = p; while runner != idom(b): DF[runner].insert(b);
//!    runner = idom(runner)`.
//! 2. **Liveness.** Per-block use/def sets feed a backward dataflow
//!    that converges on `LiveIn` per block. Pruning uses this to
//!    drop phi nodes for variables that are not live at the join.
//! 3. **Phi placement.** Worklist over variables: for each defining
//!    block `x` and each `y ∈ DF(x)`, insert a phi for the variable
//!    if it is live-in to `y` and not already placed. New phis count
//!    as definitions, so `y` joins the worklist.
//! 4. **Renaming.** Pre-order dominator-tree DFS, maintaining one
//!    stack per variable. Each phi and each defining instruction
//!    pushes a fresh [`ValueId`]; uses read top-of-stack. After
//!    visiting children, the block pops everything it pushed.
//! 5. **Phi fill-in.** While renaming a block, every successor's
//!    phi-for-`v` gets an incoming entry `(this_block, top_of_stack(v))`.
//!    Variables with an empty stack at that point contribute
//!    [`Operand::Undef`] (no reaching definition along this edge).
//! 6. **Trivial CSE.** A local value-numbering pass per block hashes
//!    instructions by (op kind, operands). The second occurrence
//!    drops its instruction and rewrites every downstream operand
//!    pointing at its destination to the first occurrence.
//!
//! ## Decoupling from the lifter
//!
//! The construction takes [`RawFunction`] rather than reading
//! `InstructionIr` directly. That keeps the algorithm testable in
//! isolation (the dac-analysis crate doesn't have to pull in an
//! architecture lifter to unit-test SSA correctness) and lets future
//! lifters express subsets of the SSA op vocabulary without changing
//! this module. The bridge from `InstructionIr` to [`RawFunction`]
//! is a B2.4-or-later concern.
//!
//! ## Determinism
//!
//! `construct_ssa` is [`Determinism::Pure`](dac_core::Determinism::Pure):
//! variable iteration order is the input order, dominator-tree
//! traversal is pre-order by ascending block id, phi `incoming` lists
//! are sorted by predecessor, and value numbering uses a `BTreeMap`
//! keyed by a structural hash key. Same inputs → same SsaFunction,
//! always.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dac_ir::ssa::{
    CompareKind, Operand, Phi, SsaBlock, SsaBlockId, SsaFunction, SsaInstruction, SsaOp,
    SsaTerminator, ValueDef, ValueId, ValueSource, Variable, VariableId,
};

use crate::cfg::{BlockId, Cfg};
use crate::dom::{predecessors_of, DominatorTree};

/// Per-function raw input to the SSA constructor.
///
/// Each [`RawBlock`] mirrors a CFG basic block in source order,
/// referring to variables by [`VariableId`]. The number of blocks must
/// equal `cfg.blocks.len()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFunction {
    /// Variable table, indexed by [`VariableId`].
    pub variables: Vec<Variable>,
    /// Raw blocks, indexed by [`SsaBlockId`] (matches `Cfg::blocks`).
    pub blocks: Vec<RawBlock>,
}

/// One basic block of raw operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBlock {
    /// Operations in source order.
    pub ops: Vec<RawOp>,
    /// Block terminator.
    pub terminator: RawTerminator,
}

/// One raw operation: an optional definition plus an operation kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawOp {
    /// Variable defined by the op; `None` for side-effect-only ops
    /// (stores, void calls, …).
    pub dst: Option<VariableId>,
    /// Operation kind.
    pub kind: RawOpKind,
}

/// Closed enumeration of raw operation kinds. Mirrors [`SsaOp`] but
/// with [`RawOperand`] instead of [`Operand`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawOpKind {
    Move {
        src: RawOperand,
    },
    Add {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Sub {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Mul {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    And {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Or {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Xor {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Shl {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Shr {
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Neg {
        src: RawOperand,
    },
    Not {
        src: RawOperand,
    },
    Compare {
        kind: CompareKind,
        lhs: RawOperand,
        rhs: RawOperand,
    },
    Load {
        address: RawOperand,
        width: u8,
    },
    Store {
        address: RawOperand,
        value: RawOperand,
        width: u8,
    },
    Call {
        target: Option<u64>,
        args: Vec<RawOperand>,
    },
    Opaque {
        mnemonic: String,
        args: Vec<RawOperand>,
    },
}

/// Raw operand vocabulary — variables before SSA naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawOperand {
    /// Read of a variable.
    Variable(VariableId),
    /// Integer immediate.
    Const(i64),
}

/// Raw block terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawTerminator {
    /// Unconditional jump.
    Jump { target: SsaBlockId },
    /// Conditional branch.
    Branch {
        cond: RawOperand,
        taken: SsaBlockId,
        not_taken: SsaBlockId,
    },
    /// Return — value carries the return slot when known.
    Return { value: Option<RawOperand> },
    /// Indirect / unresolved branch — treated as an exit by the
    /// constructor.
    Indirect,
    /// CFG-level unreachable. Used for blocks the lifter could not
    /// translate (decode-invalid, syscall-as-exit). Retained so the
    /// SSA function shape mirrors the CFG (I-2).
    Unreachable,
}

/// Build an [`SsaFunction`] in pruned SSA form.
///
/// `raw.blocks.len()` must equal `cfg.blocks.len()`. The function's
/// evidence handle is inherited from the source CFG so dataflow
/// passes can attach further facts to the same node (I-2).
#[must_use]
pub fn construct_ssa(cfg: &Cfg, doms: &DominatorTree, raw: &RawFunction) -> SsaFunction {
    assert_eq!(
        raw.blocks.len(),
        cfg.blocks.len(),
        "raw function block count must match CFG block count",
    );

    let n = cfg.blocks.len();
    let nvars = raw.variables.len();

    if n == 0 {
        return SsaFunction {
            function_address: cfg.function_address,
            function_name: cfg.function_name.clone(),
            blocks: Vec::new(),
            entry: 0,
            variables: raw.variables.clone(),
            values: Vec::new(),
            evidence: cfg.evidence,
        };
    }

    let preds = predecessors_of(cfg);
    let df = dominance_frontiers(doms, &preds, n);
    let DefUseSets {
        def_sites,
        uses_before_def,
        def_in_block,
    } = collect_def_use(raw, nvars);
    let live_in = compute_live_in(&preds, &uses_before_def, &def_in_block, n);

    // Placeholder phis. We track only "which variables have a phi at
    // this block" until renaming assigns ValueIds and fills incoming.
    let mut placed_phis: Vec<BTreeSet<VariableId>> = vec![BTreeSet::new(); n];
    place_phis(
        nvars,
        &def_sites,
        &def_in_block,
        &df,
        &live_in,
        &mut placed_phis,
    );

    let mut state = RenameState::new(cfg, doms, &preds, raw, &placed_phis);
    state.rename();
    state.build(cfg)
}

/// Compute dominance frontiers using Cytron's classic algorithm.
///
/// `df[b]` is the set of blocks `y` such that `b` dominates some
/// predecessor of `y` but does not strictly dominate `y`. Sets are
/// returned as sorted, deduplicated `Vec<BlockId>` for byte-stability.
fn dominance_frontiers(
    doms: &DominatorTree,
    preds: &[Vec<BlockId>],
    n: usize,
) -> Vec<Vec<BlockId>> {
    let mut df: Vec<BTreeSet<BlockId>> = vec![BTreeSet::new(); n];
    for b in 0..n as BlockId {
        let bi = b as usize;
        if doms.idom(b).is_none() {
            // Unreachable — contributes no DF entries since no
            // dominator chain to walk.
            continue;
        }
        if preds[bi].len() < 2 {
            continue;
        }
        let Some(idom_b) = doms.idom(b) else {
            continue;
        };
        for &p in &preds[bi] {
            // Skip predecessors that are themselves unreachable —
            // their idom chain does not exist.
            if doms.idom(p).is_none() {
                continue;
            }
            let mut runner = p;
            while runner != idom_b {
                df[runner as usize].insert(b);
                let Some(next) = doms.idom(runner) else {
                    break;
                };
                if next == runner {
                    break;
                }
                runner = next;
            }
        }
    }
    df.into_iter().map(|s| s.into_iter().collect()).collect()
}

/// Bundle of the three per-block sets used during phi placement and
/// liveness.
struct DefUseSets {
    /// `def_sites[v]` = blocks that define variable `v`.
    def_sites: Vec<BTreeSet<BlockId>>,
    /// `uses_before_def[b]` = variables read in `b` before being
    /// (re-)defined inside `b`. These are exactly the variables for
    /// which `b` consumes an in-edge value.
    uses_before_def: Vec<BTreeSet<VariableId>>,
    /// `def_in_block[b]` = variables that have a definition somewhere
    /// in `b`.
    def_in_block: Vec<BTreeSet<VariableId>>,
}

/// Per-block use-before-def and definition sets.
fn collect_def_use(raw: &RawFunction, nvars: usize) -> DefUseSets {
    let n = raw.blocks.len();
    let mut def_sites: Vec<BTreeSet<BlockId>> = vec![BTreeSet::new(); nvars];
    let mut uses_before_def: Vec<BTreeSet<VariableId>> = vec![BTreeSet::new(); n];
    let mut def_in_block: Vec<BTreeSet<VariableId>> = vec![BTreeSet::new(); n];

    for (bid, block) in raw.blocks.iter().enumerate() {
        let mut defined_here: BTreeSet<VariableId> = BTreeSet::new();
        for op in &block.ops {
            for opnd in operands_of(&op.kind) {
                if let RawOperand::Variable(v) = opnd {
                    if !defined_here.contains(&v) {
                        uses_before_def[bid].insert(v);
                    }
                }
            }
            if let Some(d) = op.dst {
                defined_here.insert(d);
                def_sites[d as usize].insert(bid as BlockId);
            }
        }
        // The terminator is a use site (its operands are read before
        // we leave the block). Defs from the terminator are not
        // modelled — terminators produce no values.
        for opnd in operands_of_terminator(&block.terminator) {
            if let RawOperand::Variable(v) = opnd {
                if !defined_here.contains(&v) {
                    uses_before_def[bid].insert(v);
                }
            }
        }
        def_in_block[bid] = defined_here;
    }
    DefUseSets {
        def_sites,
        uses_before_def,
        def_in_block,
    }
}

fn operands_of(kind: &RawOpKind) -> Vec<RawOperand> {
    match kind {
        RawOpKind::Move { src } | RawOpKind::Neg { src } | RawOpKind::Not { src } => vec![*src],
        RawOpKind::Add { lhs, rhs }
        | RawOpKind::Sub { lhs, rhs }
        | RawOpKind::Mul { lhs, rhs }
        | RawOpKind::And { lhs, rhs }
        | RawOpKind::Or { lhs, rhs }
        | RawOpKind::Xor { lhs, rhs }
        | RawOpKind::Shl { lhs, rhs }
        | RawOpKind::Shr { lhs, rhs }
        | RawOpKind::Compare { lhs, rhs, .. } => vec![*lhs, *rhs],
        RawOpKind::Load { address, .. } => vec![*address],
        RawOpKind::Store { address, value, .. } => vec![*address, *value],
        RawOpKind::Call { args, .. } | RawOpKind::Opaque { args, .. } => args.clone(),
    }
}

fn operands_of_terminator(t: &RawTerminator) -> Vec<RawOperand> {
    match t {
        RawTerminator::Branch { cond, .. } => vec![*cond],
        RawTerminator::Return { value: Some(v) } => vec![*v],
        RawTerminator::Jump { .. }
        | RawTerminator::Return { value: None }
        | RawTerminator::Indirect
        | RawTerminator::Unreachable => Vec::new(),
    }
}

/// Backward-dataflow liveness over the CFG.
///
/// `LiveIn[b] = UsesBeforeDef[b] ∪ (LiveOut[b] - DefInBlock[b])`,
/// `LiveOut[b] = ⋃ LiveIn[s] over successors s`.
///
/// Iterates over blocks in descending id order; converges in O(n²)
/// worst case, which is fine for the per-function workloads dac
/// targets. We do not iterate over unreachable blocks specially —
/// their LiveIn is whatever the equation says.
fn compute_live_in(
    preds: &[Vec<BlockId>],
    uses_before_def: &[BTreeSet<VariableId>],
    def_in_block: &[BTreeSet<VariableId>],
    n: usize,
) -> Vec<BTreeSet<VariableId>> {
    let mut succs: Vec<Vec<BlockId>> = vec![Vec::new(); n];
    for (b, ps) in preds.iter().enumerate() {
        for &p in ps {
            succs[p as usize].push(b as BlockId);
        }
    }
    for s in &mut succs {
        s.sort_unstable();
        s.dedup();
    }

    let mut live_in: Vec<BTreeSet<VariableId>> = uses_before_def.to_vec();
    let mut changed = true;
    while changed {
        changed = false;
        for b in (0..n).rev() {
            let mut live_out: BTreeSet<VariableId> = BTreeSet::new();
            for &s in &succs[b] {
                live_out.extend(live_in[s as usize].iter().copied());
            }
            let mut new_in: BTreeSet<VariableId> =
                live_out.difference(&def_in_block[b]).copied().collect();
            new_in.extend(uses_before_def[b].iter().copied());
            if new_in != live_in[b] {
                live_in[b] = new_in;
                changed = true;
            }
        }
    }
    live_in
}

/// Place pruned phis. After this call, `placed_phis[b]` lists every
/// variable that needs a phi at the start of block `b`.
fn place_phis(
    nvars: usize,
    def_sites: &[BTreeSet<BlockId>],
    def_in_block: &[BTreeSet<VariableId>],
    df: &[Vec<BlockId>],
    live_in: &[BTreeSet<VariableId>],
    placed_phis: &mut [BTreeSet<VariableId>],
) {
    for v in 0..nvars as VariableId {
        let mut work: VecDeque<BlockId> = def_sites[v as usize].iter().copied().collect();
        let mut already_has_phi: BTreeSet<BlockId> = BTreeSet::new();
        let mut already_in_work: BTreeSet<BlockId> = def_sites[v as usize].clone();
        while let Some(x) = work.pop_front() {
            for &y in &df[x as usize] {
                if already_has_phi.contains(&y) {
                    continue;
                }
                if !live_in[y as usize].contains(&v) {
                    continue;
                }
                placed_phis[y as usize].insert(v);
                already_has_phi.insert(y);
                if !def_in_block[y as usize].contains(&v) && !already_in_work.contains(&y) {
                    work.push_back(y);
                    already_in_work.insert(y);
                }
            }
        }
    }
}

/// Renaming state: maintains the per-variable stack and emits the
/// final blocks + values.
struct RenameState<'a> {
    cfg: &'a Cfg,
    doms: &'a DominatorTree,
    raw: &'a RawFunction,

    /// Per-variable stack of currently-visible ValueIds.
    stacks: Vec<Vec<ValueId>>,
    /// Output value table.
    values: Vec<ValueDef>,
    /// Output blocks.
    blocks: Vec<SsaBlock>,
    /// Per-variable Parameter ValueId at function entry, lazily
    /// minted when the variable is first read with an empty stack.
    parameter_values: Vec<Option<ValueId>>,
}

impl<'a> RenameState<'a> {
    fn new(
        cfg: &'a Cfg,
        doms: &'a DominatorTree,
        preds: &[Vec<BlockId>],
        raw: &'a RawFunction,
        placed_phis: &[BTreeSet<VariableId>],
    ) -> Self {
        let n = cfg.blocks.len();
        let nvars = raw.variables.len();

        // Pre-seed phi slots in each block so we can grow `incoming`
        // during the rename walk.
        let mut blocks: Vec<SsaBlock> = Vec::with_capacity(n);
        for b in 0..n {
            let mut phis: Vec<Phi> = Vec::new();
            for &v in &placed_phis[b] {
                phis.push(Phi {
                    dst: 0, // filled during rename
                    variable: v,
                    incoming: Vec::new(),
                });
            }
            blocks.push(SsaBlock {
                id: b as SsaBlockId,
                predecessors: preds[b].iter().map(|&p| p as SsaBlockId).collect(),
                phis,
                instructions: Vec::new(),
                terminator: SsaTerminator::Unreachable,
            });
        }

        Self {
            cfg,
            doms,
            raw,
            stacks: vec![Vec::new(); nvars],
            values: Vec::new(),
            blocks,
            parameter_values: vec![None; nvars],
        }
    }

    fn fresh_value(&mut self, source: ValueSource, variable: VariableId) -> ValueId {
        let id = self.values.len() as ValueId;
        self.values.push(ValueDef {
            id,
            source,
            variable,
        });
        id
    }

    /// Top of the variable stack, minting a Parameter ValueId when
    /// the variable has no reaching definition. Parameters share one
    /// id per variable so two reads of an unwritten register hash
    /// equal in value numbering.
    fn read_var(&mut self, var: VariableId) -> Operand {
        if let Some(&v) = self.stacks[var as usize].last() {
            return Operand::Value(v);
        }
        if let Some(p) = self.parameter_values[var as usize] {
            return Operand::Value(p);
        }
        let id = self.fresh_value(ValueSource::Parameter { variable: var }, var);
        self.parameter_values[var as usize] = Some(id);
        // Parameter values are not pushed onto the stack — they are
        // implicit and reachable from anywhere via parameter_values.
        Operand::Value(id)
    }

    fn lower_operand(&mut self, opnd: RawOperand) -> Operand {
        match opnd {
            RawOperand::Variable(v) => self.read_var(v),
            RawOperand::Const(c) => Operand::Const(c),
        }
    }

    fn rename(&mut self) {
        // Pre-order traversal of the dominator tree. We need the
        // children of each block sorted ascending to keep the order
        // deterministic — DominatorTree::children already returns
        // ascending.
        let entry = self.cfg.entry;
        if self.doms.idom(entry).is_none() {
            return;
        }
        let mut work: Vec<(BlockId, usize)> = Vec::new();
        self.enter_block(entry);
        work.push((entry, 0));

        while let Some(&(node, idx)) = work.last() {
            let children = self.doms.children(node);
            if idx < children.len() {
                let next = children[idx];
                let last_idx = work.len() - 1;
                work[last_idx].1 = idx + 1;
                self.enter_block(next);
                work.push((next, 0));
            } else {
                self.leave_block(node);
                work.pop();
            }
        }
    }

    fn enter_block(&mut self, b: BlockId) {
        let bi = b as usize;
        // Phi definitions: assign ValueIds and push onto stack.
        for phi_idx in 0..self.blocks[bi].phis.len() {
            let var = self.blocks[bi].phis[phi_idx].variable;
            let id = self.fresh_value(
                ValueSource::Phi {
                    block: b as SsaBlockId,
                    index: phi_idx as u32,
                },
                var,
            );
            self.blocks[bi].phis[phi_idx].dst = id;
            self.stacks[var as usize].push(id);
        }

        // Instructions: lower operands, then mint dst.
        let raw_ops = self.raw.blocks[bi].ops.clone();
        let mut emitted: Vec<SsaInstruction> = Vec::with_capacity(raw_ops.len());
        for (i, op) in raw_ops.into_iter().enumerate() {
            let ssa_op = self.lower_kind(op.kind);
            let dst = if let Some(var) = op.dst {
                let id = self.fresh_value(
                    ValueSource::Instruction {
                        block: b as SsaBlockId,
                        index: i as u32,
                    },
                    var,
                );
                self.stacks[var as usize].push(id);
                Some(id)
            } else {
                None
            };
            emitted.push(SsaInstruction { dst, op: ssa_op });
        }
        self.blocks[bi].instructions = emitted;

        // Terminator: lower operands.
        let raw_term = self.raw.blocks[bi].terminator.clone();
        self.blocks[bi].terminator = self.lower_terminator(raw_term);

        // For each CFG successor, populate phi incoming entries with
        // the current top-of-stack for each phi variable.
        let mut successors: Vec<BlockId> = self
            .cfg
            .successors(b as BlockId)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        successors.sort_unstable();
        for s in successors {
            let si = s as usize;
            for phi_idx in 0..self.blocks[si].phis.len() {
                let var = self.blocks[si].phis[phi_idx].variable;
                let operand = self.read_var(var);
                self.blocks[si].phis[phi_idx]
                    .incoming
                    .push((b as SsaBlockId, operand));
            }
        }
    }

    fn leave_block(&mut self, b: BlockId) {
        let bi = b as usize;
        // Pop in the reverse of `enter_block`'s push order: ops first
        // (in reverse), then phis. Within a single variable, a phi may
        // be followed by several op-defs, so popping in the wrong
        // order would surface the phi's id beneath a still-live op id.
        for op in self.raw.blocks[bi].ops.iter().rev() {
            if let Some(var) = op.dst {
                self.stacks[var as usize].pop();
            }
        }
        for phi in &self.blocks[bi].phis {
            let stack = &mut self.stacks[phi.variable as usize];
            let last = stack.pop();
            debug_assert_eq!(last, Some(phi.dst));
        }
    }

    fn lower_kind(&mut self, kind: RawOpKind) -> SsaOp {
        match kind {
            RawOpKind::Move { src } => SsaOp::Move {
                src: self.lower_operand(src),
            },
            RawOpKind::Add { lhs, rhs } => SsaOp::Add {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Sub { lhs, rhs } => SsaOp::Sub {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Mul { lhs, rhs } => SsaOp::Mul {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::And { lhs, rhs } => SsaOp::And {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Or { lhs, rhs } => SsaOp::Or {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Xor { lhs, rhs } => SsaOp::Xor {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Shl { lhs, rhs } => SsaOp::Shl {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Shr { lhs, rhs } => SsaOp::Shr {
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Neg { src } => SsaOp::Neg {
                src: self.lower_operand(src),
            },
            RawOpKind::Not { src } => SsaOp::Not {
                src: self.lower_operand(src),
            },
            RawOpKind::Compare { kind, lhs, rhs } => SsaOp::Compare {
                kind,
                lhs: self.lower_operand(lhs),
                rhs: self.lower_operand(rhs),
            },
            RawOpKind::Load { address, width } => SsaOp::Load {
                address: self.lower_operand(address),
                width,
            },
            RawOpKind::Store {
                address,
                value,
                width,
            } => SsaOp::Store {
                address: self.lower_operand(address),
                value: self.lower_operand(value),
                width,
            },
            RawOpKind::Call { target, args } => {
                let args = args.into_iter().map(|a| self.lower_operand(a)).collect();
                SsaOp::Call { target, args }
            }
            RawOpKind::Opaque { mnemonic, args } => {
                let args = args.into_iter().map(|a| self.lower_operand(a)).collect();
                SsaOp::Opaque { mnemonic, args }
            }
        }
    }

    fn lower_terminator(&mut self, t: RawTerminator) -> SsaTerminator {
        match t {
            RawTerminator::Jump { target } => SsaTerminator::Jump { target },
            RawTerminator::Branch {
                cond,
                taken,
                not_taken,
            } => SsaTerminator::Branch {
                cond: self.lower_operand(cond),
                taken,
                not_taken,
            },
            RawTerminator::Return { value } => SsaTerminator::Return {
                value: value.map(|v| self.lower_operand(v)),
            },
            RawTerminator::Indirect => SsaTerminator::Indirect,
            RawTerminator::Unreachable => SsaTerminator::Unreachable,
        }
    }

    fn build(mut self, cfg: &Cfg) -> SsaFunction {
        // Sort each phi's incoming list by predecessor id for byte
        // stability. The rename walk inserts in DFS order, which is
        // not predecessor-ordered.
        for block in &mut self.blocks {
            for phi in &mut block.phis {
                phi.incoming.sort_by_key(|(p, _)| *p);
            }
        }

        let mut ssa = SsaFunction {
            function_address: cfg.function_address,
            function_name: cfg.function_name.clone(),
            blocks: std::mem::take(&mut self.blocks),
            entry: cfg.entry as SsaBlockId,
            variables: self.raw.variables.clone(),
            values: std::mem::take(&mut self.values),
            evidence: cfg.evidence,
        };

        // Trivial local CSE pass (value numbering). The pass may
        // leave orphan `ValueDef`s in the value table — values whose
        // defining instruction has been folded away. Consumers reach
        // values through phi/instr `dst` fields, so the orphan entries
        // are harmless and keep `values[id].id == id` invariant.
        local_value_number(&mut ssa);
        ssa
    }
}

/// One-pass block-local value numbering / trivial CSE.
///
/// Within each block:
/// - Each value-producing instruction is hashed by a structural key
///   (op discriminant + operand list).
/// - The first instruction with a given key claims the key in a
///   per-block map; subsequent matches are dropped and their dst
///   ValueId is rewritten to the first dst.
///
/// After all blocks have been processed once, the cumulative remap is
/// applied globally to every operand (phi incoming, instruction
/// operands, terminator operands). This is sufficient for the
/// "trivial CSE" deliverable; cross-block CSE and constant folding
/// are out of scope for B2.3.
fn local_value_number(ssa: &mut SsaFunction) {
    let mut remap: BTreeMap<ValueId, ValueId> = BTreeMap::new();

    for bi in 0..ssa.blocks.len() {
        let mut seen: BTreeMap<VnKey, ValueId> = BTreeMap::new();
        let mut kept: Vec<SsaInstruction> = Vec::with_capacity(ssa.blocks[bi].instructions.len());
        for ins in ssa.blocks[bi].instructions.drain(..) {
            // Apply current remap to operands first so equivalence
            // is detected after upstream redundancies are folded.
            let ins = remap_instruction(ins, &remap);
            if let Some(dst) = ins.dst {
                if let Some(key) = vn_key(&ins.op) {
                    if let Some(&existing) = seen.get(&key) {
                        remap.insert(dst, existing);
                        // Drop the redundant instruction.
                        continue;
                    }
                    seen.insert(key, dst);
                }
            }
            kept.push(ins);
        }
        // Reindex surviving dsts so each `ValueSource::Instruction { index }`
        // points at the value's new position in the compacted list. The
        // invariant `blocks[block].instructions[index].dst == Some(id)`
        // must hold for every live value — downstream passes (idiom
        // recognition, struct-field recovery, C lowering) reach the
        // defining op through this index without re-scanning the block.
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

    if remap.is_empty() {
        return;
    }

    // Second pass: rewrite all remaining operand references.
    for block in &mut ssa.blocks {
        for phi in &mut block.phis {
            for (_, op) in &mut phi.incoming {
                *op = remap_operand(*op, &remap);
            }
            if let Some(&new) = remap.get(&phi.dst) {
                phi.dst = new;
            }
        }
        for ins in &mut block.instructions {
            ins.op = remap_op(
                std::mem::replace(
                    &mut ins.op,
                    SsaOp::Move {
                        src: Operand::Undef,
                    },
                ),
                &remap,
            );
            if let Some(d) = ins.dst {
                if let Some(&new) = remap.get(&d) {
                    ins.dst = Some(new);
                }
            }
        }
        block.terminator = remap_terminator(
            std::mem::replace(&mut block.terminator, SsaTerminator::Unreachable),
            &remap,
        );
    }

    // Note: ValueDef entries for folded ids are left in place so
    // `values[id].id == id` remains stable for downstream passes.
    // Callers reach values through phi/instr dst fields, never by
    // iterating the table looking for a specific id.
}

fn remap_instruction(
    mut ins: SsaInstruction,
    remap: &BTreeMap<ValueId, ValueId>,
) -> SsaInstruction {
    ins.op = remap_op(ins.op, remap);
    ins
}

fn remap_op(op: SsaOp, remap: &BTreeMap<ValueId, ValueId>) -> SsaOp {
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

fn remap_operand(op: Operand, remap: &BTreeMap<ValueId, ValueId>) -> Operand {
    match op {
        Operand::Value(v) => {
            let mut cur = v;
            while let Some(&next) = remap.get(&cur) {
                if next == cur {
                    break;
                }
                cur = next;
            }
            Operand::Value(cur)
        }
        other => other,
    }
}

fn remap_terminator(t: SsaTerminator, remap: &BTreeMap<ValueId, ValueId>) -> SsaTerminator {
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

/// Structural key for block-local value numbering.
///
/// Only pure value-producing ops get a key. [`SsaOp::Load`],
/// [`SsaOp::Store`], [`SsaOp::Call`], and [`SsaOp::Opaque`] are
/// excluded because their result is not a function of their operands
/// alone (memory state / side effects).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum VnKey {
    Move(Operand),
    Add(Operand, Operand),
    Sub(Operand, Operand),
    Mul(Operand, Operand),
    And(Operand, Operand),
    Or(Operand, Operand),
    Xor(Operand, Operand),
    Shl(Operand, Operand),
    Shr(Operand, Operand),
    Neg(Operand),
    Not(Operand),
    Compare(CompareKind, Operand, Operand),
}

fn vn_key(op: &SsaOp) -> Option<VnKey> {
    Some(match op {
        SsaOp::Move { src } => VnKey::Move(*src),
        SsaOp::Add { lhs, rhs } => VnKey::Add(*lhs, *rhs),
        SsaOp::Sub { lhs, rhs } => VnKey::Sub(*lhs, *rhs),
        SsaOp::Mul { lhs, rhs } => VnKey::Mul(*lhs, *rhs),
        SsaOp::And { lhs, rhs } => VnKey::And(*lhs, *rhs),
        SsaOp::Or { lhs, rhs } => VnKey::Or(*lhs, *rhs),
        SsaOp::Xor { lhs, rhs } => VnKey::Xor(*lhs, *rhs),
        SsaOp::Shl { lhs, rhs } => VnKey::Shl(*lhs, *rhs),
        SsaOp::Shr { lhs, rhs } => VnKey::Shr(*lhs, *rhs),
        SsaOp::Neg { src } => VnKey::Neg(*src),
        SsaOp::Not { src } => VnKey::Not(*src),
        SsaOp::Compare { kind, lhs, rhs } => VnKey::Compare(*kind, *lhs, *rhs),
        SsaOp::Load { .. } | SsaOp::Store { .. } | SsaOp::Call { .. } | SsaOp::Opaque { .. } => {
            return None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::EdgeKind;
    use crate::test_support::synthetic_cfg as cfg;

    fn var(name: &str) -> Variable {
        Variable {
            id: 0, // overwritten below by vars()
            name: name.to_string(),
            width_bits: 64,
        }
    }

    fn vars(names: &[&str]) -> Vec<Variable> {
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

    fn op_move_const(dst: VariableId, c: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Const(c),
            },
        }
    }

    fn op_add(dst: VariableId, lhs: VariableId, rhs: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Variable(rhs),
            },
        }
    }

    fn op_add_const(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn term_jump(t: SsaBlockId) -> RawTerminator {
        RawTerminator::Jump { target: t }
    }

    fn term_branch(cond: VariableId, taken: SsaBlockId, not_taken: SsaBlockId) -> RawTerminator {
        RawTerminator::Branch {
            cond: RawOperand::Variable(cond),
            taken,
            not_taken,
        }
    }

    fn term_return(value: Option<VariableId>) -> RawTerminator {
        RawTerminator::Return {
            value: value.map(RawOperand::Variable),
        }
    }

    // ---------- pure types ----------

    #[test]
    fn ssa_01_variable_helper_overwrites_id() {
        // Documents that the `var()` test helper does not pre-fill id —
        // `vars(...)` is the constructor that does, so callers writing
        // test inputs see one canonical place where ids land.
        let v = var("rax");
        assert_eq!(v.id, 0);
        let vs = vars(&["rax", "rbx", "rcx"]);
        assert_eq!(vs.iter().map(|v| v.id).collect::<Vec<_>>(), vec![0, 1, 2]);
    }

    // ---------- construction ----------

    #[test]
    fn ssa_02_linear_function_renames_each_def_uniquely() {
        // Single block: rax = 1; rax = rax + 1; ret rax
        // Two definitions of rax → two distinct ValueIds.
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax"]),
            blocks: vec![RawBlock {
                ops: vec![op_move_const(0, 1), op_add_const(0, 0, 1)],
                terminator: term_return(Some(0)),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        assert_eq!(ssa.blocks.len(), 1);
        assert!(ssa.blocks[0].phis.is_empty());
        assert_eq!(ssa.blocks[0].instructions.len(), 2);
        let d0 = ssa.blocks[0].instructions[0].dst.expect("first def");
        let d1 = ssa.blocks[0].instructions[1].dst.expect("second def");
        assert_ne!(d0, d1, "two defs must produce distinct ValueIds");
        // The second instruction reads the first's ValueId.
        if let SsaOp::Add { lhs, .. } = &ssa.blocks[0].instructions[1].op {
            assert_eq!(*lhs, Operand::Value(d0));
        } else {
            panic!("expected Add at index 1");
        }
        // The return reads the second def.
        if let SsaTerminator::Return { value: Some(v) } = ssa.blocks[0].terminator {
            assert_eq!(v, Operand::Value(d1));
        } else {
            panic!("expected return");
        }
    }

    #[test]
    fn ssa_03_diamond_join_inserts_phi_for_redefined_variable() {
        // 0: rax = 1; branch
        // 1: rax = 2; jump 3
        // 2: rax = 3; jump 3
        // 3: ret rax     <-- phi for rax here
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax", "cond"]),
            blocks: vec![
                RawBlock {
                    ops: vec![op_move_const(0, 1), op_move_const(1, 0)],
                    terminator: term_branch(1, 2, 1),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 2)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 3)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![],
                    terminator: term_return(Some(0)),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Block 3 carries a phi for `rax`.
        assert_eq!(ssa.blocks[3].phis.len(), 1, "expected one phi at join");
        let phi = &ssa.blocks[3].phis[0];
        assert_eq!(phi.variable, 0);
        assert_eq!(phi.incoming.len(), 2);
        // Phi incoming is sorted by predecessor.
        assert_eq!(phi.incoming[0].0, 1);
        assert_eq!(phi.incoming[1].0, 2);
        // The return uses the phi's dst.
        if let SsaTerminator::Return { value: Some(v) } = ssa.blocks[3].terminator {
            assert_eq!(v, Operand::Value(phi.dst));
        } else {
            panic!("expected return");
        }
    }

    #[test]
    fn ssa_04_diamond_no_phi_for_dead_variable() {
        // Same diamond, but rax is defined in 1 and 2 yet never used at 3.
        // Pruned SSA should drop the phi.
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax", "cond"]),
            blocks: vec![
                RawBlock {
                    ops: vec![op_move_const(1, 0)],
                    terminator: term_branch(1, 2, 1),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 2)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 3)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![],
                    terminator: term_return(None),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Pruned: no phi because rax is not live at block 3.
        assert!(
            ssa.blocks[3].phis.is_empty(),
            "pruned SSA must omit phi for dead variable, got {:?}",
            ssa.blocks[3].phis
        );
    }

    #[test]
    fn ssa_05_loop_header_phi_carries_initial_and_back_edge_values() {
        // 0: i = 0; jmp 1
        // 1: (phi candidate)
        //    j = i + 1; (use i, def j)
        //    i = j;     (rename i to j's value)
        //    branch back to 1 or fall to 2
        // 2: ret i
        //
        // Block 1 must have a phi for `i` with incoming from 0 (i=0) and
        // 1 (i = renamed `j`).
        let cfg = cfg(
            3,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 1, EdgeKind::Taken),
                (1, 2, EdgeKind::NotTaken),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["i", "j", "cond"]),
            blocks: vec![
                RawBlock {
                    ops: vec![op_move_const(0, 0)],
                    terminator: term_jump(1),
                },
                RawBlock {
                    ops: vec![
                        op_add_const(1, 0, 1),
                        RawOp {
                            dst: Some(0),
                            kind: RawOpKind::Move {
                                src: RawOperand::Variable(1),
                            },
                        },
                        op_move_const(2, 1),
                    ],
                    terminator: term_branch(2, 1, 2),
                },
                RawBlock {
                    ops: vec![],
                    terminator: term_return(Some(0)),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Block 1 should have a phi for variable `i` (id 0).
        let phis_for_i: Vec<&Phi> = ssa.blocks[1]
            .phis
            .iter()
            .filter(|p| p.variable == 0)
            .collect();
        assert_eq!(
            phis_for_i.len(),
            1,
            "expected one phi for `i` at loop header"
        );
        let phi_i = phis_for_i[0];
        // The phi has two incoming entries: from block 0 (initial) and
        // block 1 (back-edge).
        assert_eq!(phi_i.incoming.len(), 2);
        assert_eq!(phi_i.incoming[0].0, 0);
        assert_eq!(phi_i.incoming[1].0, 1);
    }

    #[test]
    fn ssa_06_use_without_def_produces_parameter_value() {
        // 0: ret rax   (rax never defined → Parameter value)
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax"]),
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: term_return(Some(0)),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        if let SsaTerminator::Return {
            value: Some(Operand::Value(v)),
        } = ssa.blocks[0].terminator
        {
            assert_eq!(
                ssa.value(v).source,
                ValueSource::Parameter { variable: 0 },
                "use without def must mint a Parameter value"
            );
        } else {
            panic!("expected Return with a parameter value");
        }
    }

    #[test]
    fn ssa_07_trivial_cse_collapses_repeated_add() {
        // Single block:
        //   t0 = a + b
        //   t1 = a + b    <-- redundant, should fold to t0
        //   ret t1
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["a", "b", "t0", "t1"]),
            blocks: vec![RawBlock {
                ops: vec![op_add(2, 0, 1), op_add(3, 0, 1)],
                terminator: term_return(Some(3)),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Only one Add instruction should remain.
        let adds = ssa.blocks[0]
            .instructions
            .iter()
            .filter(|i| matches!(i.op, SsaOp::Add { .. }))
            .count();
        assert_eq!(adds, 1, "CSE must fold the duplicate add");
        // And the return reads the surviving dst.
        let surviving = ssa.blocks[0].instructions[0].dst.unwrap();
        if let SsaTerminator::Return { value: Some(v) } = ssa.blocks[0].terminator {
            assert_eq!(v, Operand::Value(surviving));
        } else {
            panic!("expected return");
        }
    }

    #[test]
    fn ssa_07b_cse_reindexes_surviving_value_sources() {
        // Block 0:
        //   t0 = a + b      (survives at index 0)
        //   t1 = a + b      (redundant, dropped)
        //   t2 = t0 + 1     (originally index 2; after compaction at index 1)
        //   ret t2
        //
        // The compaction has to rewrite t2's ValueSource::Instruction.index
        // from 2 → 1, or downstream callers that follow the index back to
        // the defining op (struct-field recovery, idiom matching, C
        // lowering) read past the end of the block's compacted list.
        // Documented in CHANGELOG B3.18 as the PE-corpus out-of-bounds.
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["a", "b", "t0", "t1", "t2"]),
            blocks: vec![RawBlock {
                ops: vec![op_add(2, 0, 1), op_add(3, 0, 1), op_add_const(4, 2, 1)],
                terminator: term_return(Some(4)),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        assert_eq!(
            ssa.blocks[0].instructions.len(),
            2,
            "CSE must drop the duplicate add"
        );
        let surviving_t2 = ssa.blocks[0].instructions[1]
            .dst
            .expect("compacted index 1 must hold the t2 def");
        assert_eq!(
            ssa.value(surviving_t2).source,
            ValueSource::Instruction { block: 0, index: 1 },
            "reindex must point at the value's new compacted position"
        );
        // The walk-back used by downstream passes must land on the live
        // instruction without an out-of-bounds index.
        let ValueSource::Instruction { block, index } = ssa.value(surviving_t2).source else {
            panic!("surviving dst must be an Instruction-sourced value");
        };
        assert!((index as usize) < ssa.blocks[block as usize].instructions.len());
    }

    #[test]
    fn ssa_08_cse_does_not_cross_block_boundary() {
        // Block 0: t0 = a + b; jmp 1
        // Block 1: t1 = a + b; ret t1
        // The second Add is in a different block, so local CSE keeps it.
        let cfg = cfg(2, 0, &[(0, 1, EdgeKind::Fall)]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["a", "b", "t0", "t1"]),
            blocks: vec![
                RawBlock {
                    ops: vec![op_add(2, 0, 1)],
                    terminator: term_jump(1),
                },
                RawBlock {
                    ops: vec![op_add(3, 0, 1)],
                    terminator: term_return(Some(3)),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Each block still has its own Add — local CSE only.
        assert_eq!(ssa.blocks[0].instructions.len(), 1);
        assert_eq!(ssa.blocks[1].instructions.len(), 1);
    }

    #[test]
    fn ssa_09_cse_preserves_side_effecting_ops() {
        // Loads at the same address must not be folded: an intervening
        // store could change the memory. The local pass excludes Load,
        // Store, Call, and Opaque from value-numbering keys.
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["addr", "v0", "v1"]),
            blocks: vec![RawBlock {
                ops: vec![
                    RawOp {
                        dst: Some(1),
                        kind: RawOpKind::Load {
                            address: RawOperand::Variable(0),
                            width: 8,
                        },
                    },
                    RawOp {
                        dst: Some(2),
                        kind: RawOpKind::Load {
                            address: RawOperand::Variable(0),
                            width: 8,
                        },
                    },
                ],
                terminator: term_return(None),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let loads = ssa.blocks[0]
            .instructions
            .iter()
            .filter(|i| matches!(i.op, SsaOp::Load { .. }))
            .count();
        assert_eq!(loads, 2, "loads must not be CSE'd by the trivial pass");
    }

    #[test]
    fn ssa_10_phi_incoming_is_sorted_by_predecessor() {
        // Three-way join: 0 → 1, 0 → 2, 0 → 3 (via jumps), then all
        // converge on 4. Predecessor list of 4 is {1, 2, 3}, phi
        // incoming must be sorted in that order regardless of DFS
        // rename order.
        let cfg = cfg(
            5,
            0,
            &[
                (0, 1, EdgeKind::Branch),
                (0, 2, EdgeKind::Branch),
                (0, 3, EdgeKind::Branch),
                (1, 4, EdgeKind::Branch),
                (2, 4, EdgeKind::Branch),
                (3, 4, EdgeKind::Branch),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax"]),
            blocks: vec![
                RawBlock {
                    ops: vec![],
                    terminator: term_jump(1),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 1)],
                    terminator: term_jump(4),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 2)],
                    terminator: term_jump(4),
                },
                RawBlock {
                    ops: vec![op_move_const(0, 3)],
                    terminator: term_jump(4),
                },
                RawBlock {
                    ops: vec![],
                    terminator: term_return(Some(0)),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let phi = ssa.blocks[4]
            .phis
            .first()
            .expect("expected phi at three-way join");
        let preds: Vec<SsaBlockId> = phi.incoming.iter().map(|(p, _)| *p).collect();
        assert_eq!(preds, vec![1, 2, 3]);
    }

    #[test]
    fn ssa_11_function_address_and_evidence_inherited_from_cfg() {
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: Vec::new(),
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: term_return(None),
            }],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        assert_eq!(ssa.function_address, cfg.function_address);
        assert_eq!(ssa.evidence, cfg.evidence);
        assert_eq!(ssa.entry, cfg.entry as SsaBlockId);
    }

    #[test]
    fn ssa_12_construct_is_deterministic_across_runs() {
        // Two passes over the same input must produce equal SsaFunctions
        // (NFR-9).
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax", "rbx", "cond"]),
            blocks: vec![
                RawBlock {
                    ops: vec![
                        op_move_const(0, 1),
                        op_move_const(1, 2),
                        op_move_const(2, 0),
                    ],
                    terminator: term_branch(2, 2, 1),
                },
                RawBlock {
                    ops: vec![op_add_const(0, 0, 7), op_add_const(1, 1, 9)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![op_add_const(0, 0, 11), op_add_const(1, 1, 13)],
                    terminator: term_jump(3),
                },
                RawBlock {
                    ops: vec![op_add(0, 0, 1)],
                    terminator: term_return(Some(0)),
                },
            ],
        };
        let a = construct_ssa(&cfg, &doms, &raw);
        let b = construct_ssa(&cfg, &doms, &raw);
        assert_eq!(a, b);
    }

    #[test]
    fn ssa_13_unreachable_block_terminator_is_preserved() {
        // 0 → 2 only; block 1 is unreachable. Its raw terminator says
        // Return, which we want to survive into SSA as Return so the
        // pipeline can still surface the block (I-2: traceability).
        let cfg = cfg(3, 0, &[(0, 2, EdgeKind::Branch)]);
        let doms = DominatorTree::build(&cfg);
        let raw = RawFunction {
            variables: vars(&["rax"]),
            blocks: vec![
                RawBlock {
                    ops: vec![op_move_const(0, 1)],
                    terminator: term_jump(2),
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Unreachable,
                },
                RawBlock {
                    ops: vec![],
                    terminator: term_return(Some(0)),
                },
            ],
        };
        let ssa = construct_ssa(&cfg, &doms, &raw);
        // Block 1 was unreachable; its terminator stayed Unreachable.
        assert!(matches!(
            ssa.blocks[1].terminator,
            SsaTerminator::Unreachable
        ));
        // Block 0 still ends in a Jump; block 2 still ends in a Return.
        assert!(matches!(
            ssa.blocks[0].terminator,
            SsaTerminator::Jump { target: 2 }
        ));
        assert!(matches!(
            ssa.blocks[2].terminator,
            SsaTerminator::Return { value: Some(_) }
        ));
    }

    #[test]
    fn ssa_14_dominance_frontier_diamond_matches_cytron() {
        // Diamond: DF(0) = ∅, DF(1) = {3}, DF(2) = {3}, DF(3) = ∅.
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        let preds = predecessors_of(&cfg);
        let df = dominance_frontiers(&doms, &preds, 4);
        assert!(df[0].is_empty());
        assert_eq!(df[1], vec![3]);
        assert_eq!(df[2], vec![3]);
        assert!(df[3].is_empty());
    }

    #[test]
    fn ssa_15_liveness_tracks_use_before_def() {
        // 0: t0 = a (use a)
        //    jmp 1
        // 1: t1 = t0 + b (uses t0, b)
        //    ret t1
        // a is live-in at 0 (used before def). b is live-in at 1.
        // t0 is live-out of 0, live-in to 1.
        let cfg = cfg(2, 0, &[(0, 1, EdgeKind::Fall)]);
        let raw = RawFunction {
            variables: vars(&["a", "b", "t0", "t1"]),
            blocks: vec![
                RawBlock {
                    ops: vec![RawOp {
                        dst: Some(2),
                        kind: RawOpKind::Move {
                            src: RawOperand::Variable(0),
                        },
                    }],
                    terminator: term_jump(1),
                },
                RawBlock {
                    ops: vec![op_add(3, 2, 1)],
                    terminator: term_return(Some(3)),
                },
            ],
        };
        let preds = predecessors_of(&cfg);
        let DefUseSets {
            uses_before_def,
            def_in_block,
            ..
        } = collect_def_use(&raw, 4);
        let live_in = compute_live_in(&preds, &uses_before_def, &def_in_block, 2);
        // Block 0 live-in: {a, b, t0} — a is used before def in 0; b
        // and t0 are live-in to 1 and not defined in 0.
        assert!(live_in[0].contains(&0)); // a
        assert!(live_in[0].contains(&1)); // b (propagated from 1)
                                          // Block 1 live-in: {b, t0}.
        assert!(live_in[1].contains(&1));
        assert!(live_in[1].contains(&2));
        // t1 is not live anywhere (it's only defined and immediately
        // returned).
        assert!(!live_in[0].contains(&3));
        assert!(!live_in[1].contains(&3));
    }
}
