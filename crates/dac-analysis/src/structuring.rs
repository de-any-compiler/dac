//! Control-flow structuring (B2.7, FR-18, spec §11.3).
//!
//! [`structure`] consumes a [`SsaFunction`] together with its
//! [`Cfg`] / [`DominatorTree`] / [`PostDominatorTree`] / [`LoopForest`]
//! and produces a [`SemFunction`] — a tree of structured statements
//! ([`Stmt::If`], [`Stmt::Loop`], [`Stmt::Break`], [`Stmt::Continue`],
//! [`Stmt::Return`], …) plus a goto fallback for irreducible CFGs.
//!
//! ## Algorithm sketch
//!
//! The pass is a top-down recursive walk over the dominator tree,
//! threaded by post-dominators for branch joins and by the loop
//! forest for header / break / continue resolution. The structurer
//! visits every block at most once; a second visit is the trigger
//! that demotes a block into a `goto` target with a synthesised
//! [`Stmt::Label`] (I-6, spec §11.3).
//!
//! 1. `structure_at(current, region_exit, loop_stack)` is the
//!    recursion. Bases:
//!    - `current == None` or `current == region_exit` → empty block
//!      (the caller continues from the region exit).
//!    - `current == loop_stack.last().header` (and the header has
//!      already been emitted) → [`Stmt::Continue`].
//!    - `current == loop_stack.last().exit` → [`Stmt::Break`].
//!    - `current` is already in [`State::emitted`] → [`Stmt::Goto`]
//!      and bump [`StructuringStats::goto_count`].
//! 2. Otherwise mark `current` emitted. If `current` is a loop header
//!    (and the enclosing loop_stack does not already contain it),
//!    enter [`structure_loop_entry`].
//! 3. Otherwise emit the block's phis and instructions in source
//!    order, then dispatch the terminator:
//!    - [`SsaTerminator::Jump`] → recurse into the target.
//!    - [`SsaTerminator::Branch`] → compute the join (IPDOM filtered
//!      against the region/loop), structure both arms, emit
//!      [`Stmt::If`], then recurse into the join.
//!    - [`SsaTerminator::Return`] → [`Stmt::Return`].
//!    - [`SsaTerminator::Indirect`] / [`SsaTerminator::Unreachable`]
//!      → [`Stmt::Unreachable`] (the block is honest about being
//!      structurally opaque; I-6).
//! 4. After the walk, a post-pass [`insert_labels`] inserts every
//!    required [`Stmt::Label`] at the first emission of its block.
//!    Labels are necessary only for blocks that were demoted to
//!    goto-targets in step 1; for reducible CFGs with single-exit
//!    loops the pass produces zero gotos and zero labels.
//!
//! ## What goto-free means
//!
//! [`StructuringStats::is_goto_free`] is the per-function rubric. The
//! corpus-level criterion in PLAN.md — "structuring is goto-free on
//! the sample corpus for at least the simple functions" — is the
//! aggregate of this per-function metric over the golden corpus and
//! lands when B2.9 wires it in.
//!
//! ## Determinism
//!
//! Inputs flow through dac-analysis structures that already enforce
//! ascending [`SsaBlockId`] ordering. The recursion is driven by
//! `Cfg::edges` (sorted) and `LoopForest::header_of` (`BTreeMap`).
//! Same SSA function in → same [`SemFunction`] out, byte-stable.

use std::collections::{BTreeMap, BTreeSet};

use dac_core::EvidenceId;
use dac_ir::sem::{Block, LabelId, SemFunction, SsaRef, Stmt, StructuringStats};
use dac_ir::ssa::{SsaBlockId, SsaFunction, SsaTerminator};

use crate::cfg::Cfg;
use crate::dom::{DominatorTree, PostDom, PostDominatorTree};
use crate::loops::{LoopForest, LoopId};

/// Produce a [`SemFunction`] by structuring the control flow of the
/// supplied SSA function.
///
/// `cfg`, `doms`, `pdoms`, and `loops` must all describe the same
/// function as `ssa` — call [`crate::cfg::build_cfg`],
/// [`DominatorTree::build`], [`PostDominatorTree::build`], and
/// [`LoopForest::build`] in order on the same function before
/// invoking this pass.
///
/// The pass is [`Determinism::Pure`](dac_core::Determinism::Pure) — no
/// hidden state, no RNG.
#[must_use]
pub fn structure(
    ssa: &SsaFunction,
    cfg: &Cfg,
    doms: &DominatorTree,
    pdoms: &PostDominatorTree,
    loops: &LoopForest,
) -> SemFunction {
    let body = if ssa.blocks.is_empty() {
        Block::empty()
    } else {
        let mut state = State {
            ssa,
            cfg,
            doms,
            pdoms,
            loops,
            emitted: BTreeSet::new(),
            labels: BTreeMap::new(),
            next_label: 0,
            goto_count: 0,
            evidence: ssa.evidence,
        };
        let mut loop_stack: Vec<LoopCtx> = Vec::new();
        let body = state.structure_at(Some(ssa.entry), None, &mut loop_stack);
        let labels = state.labels.clone();
        let mut stats = StructuringStats {
            source_blocks: state.emitted.len() as u32,
            goto_count: state.goto_count,
            label_count: state.labels.len() as u32,
            irreducible: loops.irreducible,
        };
        let mut body = insert_labels(body, &labels);
        // Any label that didn't get anchored during the walk (the
        // block was never the source of a stmt that survived the
        // recursion) gets appended at the tail of the body so every
        // `Stmt::Goto::target` resolves to at least one
        // `Stmt::Label::id` somewhere in the tree. Cross-scope gotos
        // are still ill-formed C — the structurer's job here is to
        // produce a tree the lowering pass (B2.8) can either accept
        // or further degrade; never to silently drop a Goto with no
        // Label (I-6).
        let anchored = collect_label_ids(&body);
        let mut orphans: Vec<(SsaBlockId, LabelId)> = labels
            .iter()
            .filter(|(_, id)| !anchored.contains(id))
            .map(|(b, id)| (*b, *id))
            .collect();
        orphans.sort_unstable_by_key(|(_, id)| *id);
        for (block, id) in orphans {
            body.stmts.push(Stmt::Label {
                id,
                source_block: block,
            });
        }
        // Re-count labels after insertion so the stat reflects what's
        // actually in the tree.
        stats.label_count = count_labels(&body);
        return SemFunction {
            function_address: ssa.function_address,
            function_name: ssa.function_name.clone(),
            body,
            evidence: ssa.evidence,
            stats,
        };
    };

    SemFunction {
        function_address: ssa.function_address,
        function_name: ssa.function_name.clone(),
        body,
        evidence: ssa.evidence,
        stats: StructuringStats {
            source_blocks: 0,
            goto_count: 0,
            label_count: 0,
            irreducible: loops.irreducible,
        },
    }
}

/// Walk state shared across recursive [`State::structure_at`] calls.
struct State<'a> {
    ssa: &'a SsaFunction,
    cfg: &'a Cfg,
    #[allow(dead_code)]
    doms: &'a DominatorTree,
    pdoms: &'a PostDominatorTree,
    loops: &'a LoopForest,
    /// Blocks the structurer has already placed into the [`Block`]
    /// tree. A second visit to a block in this set becomes a
    /// [`Stmt::Goto`].
    emitted: BTreeSet<SsaBlockId>,
    /// `block → label` for every block demoted into a goto target.
    /// The actual [`Stmt::Label`] anchors are inserted by
    /// [`insert_labels`] in a post-pass.
    labels: BTreeMap<SsaBlockId, LabelId>,
    next_label: LabelId,
    goto_count: u32,
    /// Evidence handle inherited from the SSA function. Every Sem
    /// statement carries this — per-statement evidence minting lands
    /// in a later batch when the orchestrator wires Sem nodes into
    /// the evidence graph.
    evidence: EvidenceId,
}

/// One level of the loop context stack. Pushed when the structurer
/// enters a loop header, popped when it leaves the loop.
#[derive(Debug)]
struct LoopCtx {
    /// Index into [`LoopForest::loops`] of the enclosing loop.
    loop_id: LoopId,
    /// Loop header block.
    header: SsaBlockId,
    /// Uniform exit block when the loop has one; `None` when the loop
    /// has no exit (endless) or multiple exits the pass declined to
    /// pick a representative for.
    exit: Option<SsaBlockId>,
}

impl<'a> State<'a> {
    fn structure_at(
        &mut self,
        current: Option<SsaBlockId>,
        region_exit: Option<SsaBlockId>,
        loop_stack: &mut Vec<LoopCtx>,
    ) -> Block {
        let Some(current) = current else {
            return Block::empty();
        };
        if Some(current) == region_exit {
            return Block::empty();
        }

        // Loop transitions: break / continue.
        if let Some(ctx) = loop_stack.last() {
            if Some(current) == ctx.exit {
                return Block {
                    stmts: vec![Stmt::Break {
                        evidence: self.evidence,
                    }],
                };
            }
            if current == ctx.header && self.emitted.contains(&current) {
                return Block {
                    stmts: vec![Stmt::Continue {
                        evidence: self.evidence,
                    }],
                };
            }
        }

        // Already emitted → goto fallback.
        if self.emitted.contains(&current) {
            let id = self.label_for(current);
            self.goto_count += 1;
            return Block {
                stmts: vec![Stmt::Goto {
                    target: id,
                    source_block: current,
                    evidence: self.evidence,
                }],
            };
        }
        self.emitted.insert(current);

        // Loop header — only treat as a new loop entry when we're not
        // already structuring inside this loop. The `loop_stack` walk
        // skips the loop-detection branch when the header is the
        // immediately-enclosing context.
        let in_loop_stack = loop_stack.iter().any(|c| c.header == current);
        if !in_loop_stack {
            if let Some(&lid) = self.loops.header_of.get(&current) {
                return self.structure_loop_entry(lid, current, region_exit, loop_stack);
            }
        }

        self.process_block(current, region_exit, loop_stack)
    }

    /// Emit phis + instructions for `current`, then dispatch its
    /// terminator into the structured form. Does not consult the
    /// loop-header detection — callers gate that.
    fn process_block(
        &mut self,
        current: SsaBlockId,
        region_exit: Option<SsaBlockId>,
        loop_stack: &mut Vec<LoopCtx>,
    ) -> Block {
        let block = self.ssa.block(current);
        let mut stmts: Vec<Stmt> = Vec::new();

        for idx in 0..block.phis.len() {
            stmts.push(Stmt::Phi {
                r: SsaRef {
                    block: current,
                    index: idx as u32,
                },
                evidence: self.evidence,
            });
        }
        for idx in 0..block.instructions.len() {
            stmts.push(Stmt::Instr {
                r: SsaRef {
                    block: current,
                    index: idx as u32,
                },
                evidence: self.evidence,
            });
        }

        let term = block.terminator.clone();
        match term {
            SsaTerminator::Jump { target } => {
                let tail = self.structure_at(Some(target), region_exit, loop_stack);
                stmts.extend(tail.stmts);
            }
            SsaTerminator::Branch {
                cond,
                taken,
                not_taken,
            } => {
                let join = self.find_join(current, region_exit, loop_stack);
                let then_body = self.structure_arm(taken, join, region_exit, loop_stack);
                let else_block = self.structure_arm(not_taken, join, region_exit, loop_stack);
                let else_body = if else_block.is_empty() {
                    None
                } else {
                    Some(else_block)
                };
                stmts.push(Stmt::If {
                    cond,
                    then_body,
                    else_body,
                    source_block: current,
                    evidence: self.evidence,
                });
                if let Some(j) = join {
                    let tail = self.structure_at(Some(j), region_exit, loop_stack);
                    stmts.extend(tail.stmts);
                }
            }
            SsaTerminator::Return { value } => {
                stmts.push(Stmt::Return {
                    value,
                    evidence: self.evidence,
                });
            }
            SsaTerminator::Indirect | SsaTerminator::Unreachable => {
                stmts.push(Stmt::Unreachable {
                    source_block: current,
                    evidence: self.evidence,
                });
            }
        }

        Block { stmts }
    }

    /// Structure one branch arm: target = the block the arm jumps to,
    /// `join` = the post-If merge point (or `None` for "no merge").
    /// If `target == join`, the arm is empty (the arm "falls through"
    /// straight to the merge); otherwise recurse with the join as the
    /// arm's region exit.
    fn structure_arm(
        &mut self,
        target: SsaBlockId,
        join: Option<SsaBlockId>,
        region_exit: Option<SsaBlockId>,
        loop_stack: &mut Vec<LoopCtx>,
    ) -> Block {
        if Some(target) == join {
            return Block::empty();
        }
        let arm_region = join.or(region_exit);
        self.structure_at(Some(target), arm_region, loop_stack)
    }

    /// Find the join point for a branch at `current`. The IPDOM is
    /// the default; it is suppressed when it equals the surrounding
    /// `region_exit` (the caller will continue from there anyway) or
    /// when it falls outside the enclosing loop body (the arms reach
    /// the loop exit via [`Stmt::Break`] instead of a structural
    /// merge).
    fn find_join(
        &self,
        current: SsaBlockId,
        region_exit: Option<SsaBlockId>,
        loop_stack: &[LoopCtx],
    ) -> Option<SsaBlockId> {
        let raw = match self.pdoms.ipostdom(current) {
            PostDom::Block(j) => j,
            PostDom::Exit | PostDom::Unreachable => return None,
        };
        if Some(raw) == region_exit {
            return None;
        }
        if let Some(ctx) = loop_stack.last() {
            let l = &self.loops.loops[ctx.loop_id as usize];
            if l.body.binary_search(&raw).is_err() {
                return None;
            }
        }
        Some(raw)
    }

    /// Enter a loop whose header is `header`. Builds the loop body by
    /// processing the header inside a pushed [`LoopCtx`], so that
    /// back-edges into the header turn into [`Stmt::Continue`] and
    /// edges to the loop exit turn into [`Stmt::Break`]. After the
    /// loop closes, recurse into the exit block (when known) to
    /// continue the surrounding region.
    fn structure_loop_entry(
        &mut self,
        lid: LoopId,
        header: SsaBlockId,
        region_exit: Option<SsaBlockId>,
        loop_stack: &mut Vec<LoopCtx>,
    ) -> Block {
        let exit = self.compute_loop_exit(lid);

        loop_stack.push(LoopCtx {
            loop_id: lid,
            header,
            exit,
        });
        let body = self.process_block(header, region_exit, loop_stack);
        loop_stack.pop();

        let mut outer: Vec<Stmt> = vec![Stmt::Loop {
            body,
            header,
            evidence: self.evidence,
        }];

        if let Some(exit_id) = exit {
            let tail = self.structure_at(Some(exit_id), region_exit, loop_stack);
            outer.extend(tail.stmts);
        }

        Block { stmts: outer }
    }

    /// Pick the loop's exit block when one can be identified.
    ///
    /// Preferred shape: the loop header is a conditional whose taken
    /// and not-taken sides split between "in the loop" and "outside".
    /// Falls back to "any single block outside the loop reached from
    /// the loop body". When multiple distinct exits are reached, the
    /// IPDOM of the header is preferred when it sits outside; failing
    /// that, the smallest-id exit block wins so other exits become
    /// goto-emitted (`I-6`, degrade not invent).
    fn compute_loop_exit(&self, lid: LoopId) -> Option<SsaBlockId> {
        let l = &self.loops.loops[lid as usize];
        let header_block = self.ssa.block(l.header);
        if let SsaTerminator::Branch {
            taken, not_taken, ..
        } = &header_block.terminator
        {
            let taken_in = l.body.binary_search(taken).is_ok();
            let not_taken_in = l.body.binary_search(not_taken).is_ok();
            match (taken_in, not_taken_in) {
                (true, false) => return Some(*not_taken),
                (false, true) => return Some(*taken),
                _ => {}
            }
        }
        let mut exits: BTreeSet<SsaBlockId> = BTreeSet::new();
        for &b in &l.body {
            for e in &self.cfg.edges {
                if e.from == b && l.body.binary_search(&e.to).is_err() {
                    exits.insert(e.to);
                }
            }
        }
        match exits.len() {
            0 => None,
            1 => exits.into_iter().next(),
            _ => match self.pdoms.ipostdom(l.header) {
                PostDom::Block(j) if l.body.binary_search(&j).is_err() => Some(j),
                _ => exits.into_iter().next(),
            },
        }
    }

    fn label_for(&mut self, b: SsaBlockId) -> LabelId {
        if let Some(&id) = self.labels.get(&b) {
            return id;
        }
        let id = self.next_label;
        self.next_label = self.next_label.checked_add(1).unwrap_or(LabelId::MAX);
        self.labels.insert(b, id);
        id
    }
}

/// Post-pass: walk the structured body and insert [`Stmt::Label`]
/// markers at the first emission of every labelled block.
fn insert_labels(body: Block, labels: &BTreeMap<SsaBlockId, LabelId>) -> Block {
    if labels.is_empty() {
        return body;
    }
    let mut seen: BTreeSet<SsaBlockId> = BTreeSet::new();
    insert_labels_in(body, labels, &mut seen)
}

fn insert_labels_in(
    body: Block,
    labels: &BTreeMap<SsaBlockId, LabelId>,
    seen: &mut BTreeSet<SsaBlockId>,
) -> Block {
    let mut out: Vec<Stmt> = Vec::with_capacity(body.stmts.len());
    for stmt in body.stmts {
        if let Some(source) = stmt_source_block(&stmt) {
            if !seen.contains(&source) {
                if let Some(&lid) = labels.get(&source) {
                    out.push(Stmt::Label {
                        id: lid,
                        source_block: source,
                    });
                }
                seen.insert(source);
            }
        }
        out.push(recurse_labels(stmt, labels, seen));
    }
    Block { stmts: out }
}

fn recurse_labels(
    stmt: Stmt,
    labels: &BTreeMap<SsaBlockId, LabelId>,
    seen: &mut BTreeSet<SsaBlockId>,
) -> Stmt {
    match stmt {
        Stmt::If {
            cond,
            then_body,
            else_body,
            source_block,
            evidence,
        } => Stmt::If {
            cond,
            then_body: insert_labels_in(then_body, labels, seen),
            else_body: else_body.map(|b| insert_labels_in(b, labels, seen)),
            source_block,
            evidence,
        },
        Stmt::While {
            cond,
            body,
            header,
            evidence,
        } => Stmt::While {
            cond,
            body: insert_labels_in(body, labels, seen),
            header,
            evidence,
        },
        Stmt::DoWhile {
            cond,
            body,
            header,
            latch,
            evidence,
        } => Stmt::DoWhile {
            cond,
            body: insert_labels_in(body, labels, seen),
            header,
            latch,
            evidence,
        },
        Stmt::Loop {
            body,
            header,
            evidence,
        } => Stmt::Loop {
            body: insert_labels_in(body, labels, seen),
            header,
            evidence,
        },
        Stmt::Switch {
            scrutinee,
            arms,
            default,
            source_block,
            evidence,
        } => Stmt::Switch {
            scrutinee,
            arms: arms
                .into_iter()
                .map(|arm| dac_ir::sem::SwitchArm {
                    value: arm.value,
                    body: insert_labels_in(arm.body, labels, seen),
                })
                .collect(),
            default: default.map(|b| insert_labels_in(b, labels, seen)),
            source_block,
            evidence,
        },
        other => other,
    }
}

fn stmt_source_block(stmt: &Stmt) -> Option<SsaBlockId> {
    match stmt {
        Stmt::Phi { r, .. } | Stmt::Instr { r, .. } => Some(r.block),
        Stmt::If { source_block, .. } => Some(*source_block),
        Stmt::While { header, .. } => Some(*header),
        Stmt::DoWhile { header, .. } => Some(*header),
        Stmt::Loop { header, .. } => Some(*header),
        Stmt::Switch { source_block, .. } => Some(*source_block),
        Stmt::Unreachable { source_block, .. } => Some(*source_block),
        Stmt::Label { source_block, .. } => Some(*source_block),
        Stmt::Goto { source_block, .. } => Some(*source_block),
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Return { .. } => None,
    }
}

fn collect_label_ids(body: &Block) -> BTreeSet<LabelId> {
    let mut out = BTreeSet::new();
    walk_collect_label_ids(body, &mut out);
    out
}

fn walk_collect_label_ids(body: &Block, out: &mut BTreeSet<LabelId>) {
    for s in &body.stmts {
        if let Stmt::Label { id, .. } = s {
            out.insert(*id);
        }
        match s {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                walk_collect_label_ids(then_body, out);
                if let Some(eb) = else_body {
                    walk_collect_label_ids(eb, out);
                }
            }
            Stmt::While { body, .. } => walk_collect_label_ids(body, out),
            Stmt::DoWhile { body, .. } => walk_collect_label_ids(body, out),
            Stmt::Loop { body, .. } => walk_collect_label_ids(body, out),
            Stmt::Switch { arms, default, .. } => {
                for a in arms {
                    walk_collect_label_ids(&a.body, out);
                }
                if let Some(d) = default {
                    walk_collect_label_ids(d, out);
                }
            }
            _ => {}
        }
    }
}

fn count_labels(body: &Block) -> u32 {
    let mut n = 0u32;
    for s in &body.stmts {
        if matches!(s, Stmt::Label { .. }) {
            n += 1;
        }
        match s {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                n += count_labels(then_body);
                if let Some(eb) = else_body {
                    n += count_labels(eb);
                }
            }
            Stmt::While { body, .. } => n += count_labels(body),
            Stmt::DoWhile { body, .. } => n += count_labels(body),
            Stmt::Loop { body, .. } => n += count_labels(body),
            Stmt::Switch { arms, default, .. } => {
                for a in arms {
                    n += count_labels(&a.body);
                }
                if let Some(d) = default {
                    n += count_labels(d);
                }
            }
            _ => {}
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{BasicBlock, BlockId, Edge, EdgeKind, Terminator};
    use crate::dom::DominatorTree;
    use crate::loops::LoopForest;
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{
        CompareKind, Operand, Phi, SsaBlock, SsaInstruction, SsaOp, SsaTerminator, ValueDef,
        ValueSource, Variable,
    };
    use std::collections::{BTreeSet, VecDeque};

    fn ev() -> EvidenceId {
        let mut g = EvidenceGraph::new();
        g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Semantic,
            id: 0,
        })
    }

    /// Build a synthetic CFG identical in shape to `test_support::synthetic_cfg`
    /// but inline so the `structuring` test module doesn't depend on
    /// `test_support` (which is private to dac-analysis but only
    /// re-used from within the crate).
    fn synthetic_cfg(n: usize, entry: BlockId, raw_edges: &[(BlockId, BlockId, EdgeKind)]) -> Cfg {
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
        edges.sort_by_key(|e| {
            (
                e.from,
                match e.kind {
                    EdgeKind::Fall => 0u8,
                    EdgeKind::Branch => 1,
                    EdgeKind::Taken => 2,
                    EdgeKind::NotTaken => 3,
                },
                e.to,
            )
        });

        let has_succ: BTreeSet<BlockId> = edges.iter().map(|e| e.from).collect();
        let exits: Vec<BlockId> = (0..n as BlockId)
            .filter(|id| !has_succ.contains(id))
            .collect();

        let mut reachable: BTreeSet<BlockId> = BTreeSet::new();
        reachable.insert(entry);
        let mut queue: VecDeque<BlockId> = VecDeque::from([entry]);
        while let Some(b) = queue.pop_front() {
            for e in &edges {
                if e.from == b && reachable.insert(e.to) {
                    queue.push_back(e.to);
                }
            }
        }
        let unreachable: Vec<BlockId> = (0..n as BlockId)
            .filter(|id| !reachable.contains(id))
            .collect();

        let mut g = EvidenceGraph::new();
        let evid = g.add_node(EvidenceNode::IrNode {
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
            evidence: evid,
        }
    }

    /// Build an empty-instruction SSA function with the supplied
    /// terminators. The variables / values tables are minimal — only
    /// what the structurer reads (block phis/instrs lengths and
    /// terminators) matters.
    fn synthetic_ssa(cfg: &Cfg, terminators: Vec<SsaTerminator>) -> SsaFunction {
        assert_eq!(terminators.len(), cfg.blocks.len());
        let blocks: Vec<SsaBlock> = cfg
            .blocks
            .iter()
            .zip(terminators)
            .map(|(b, t)| SsaBlock {
                id: b.id,
                predecessors: Vec::new(),
                phis: Vec::new(),
                instructions: Vec::new(),
                terminator: t,
            })
            .collect();
        SsaFunction {
            function_address: cfg.function_address,
            function_name: cfg.function_name.clone(),
            blocks,
            entry: cfg.entry,
            variables: vec![Variable {
                id: 0,
                name: "rax".to_string(),
                width_bits: 64,
            }],
            values: vec![ValueDef {
                id: 0,
                source: ValueSource::Parameter { variable: 0 },
                variable: 0,
            }],
            evidence: cfg.evidence,
        }
    }

    fn run(cfg: &Cfg, ssa: &SsaFunction) -> SemFunction {
        let doms = DominatorTree::build(cfg);
        let pdoms = PostDominatorTree::build(cfg);
        let loops = LoopForest::build(cfg, &doms);
        structure(ssa, cfg, &doms, &pdoms, &loops)
    }

    #[test]
    fn s_01_single_return_block_emits_one_return() {
        // 0: ret.
        let cfg = synthetic_cfg(1, 0, &[]);
        let ssa = synthetic_ssa(&cfg, vec![SsaTerminator::Return { value: None }]);
        let sem = run(&cfg, &ssa);
        assert_eq!(sem.body.stmts.len(), 1);
        assert!(matches!(
            sem.body.stmts[0],
            Stmt::Return { value: None, .. }
        ));
        assert_eq!(sem.stats.source_blocks, 1);
        assert!(sem.stats.is_goto_free());
        assert_eq!(sem.stats.label_count, 0);
        assert!(!sem.stats.irreducible);
    }

    #[test]
    fn s_02_linear_chain_emits_a_flat_sequence() {
        // 0 → 1 → 2 (ret)
        let cfg = synthetic_cfg(3, 0, &[(0, 1, EdgeKind::Fall), (1, 2, EdgeKind::Fall)]);
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Jump { target: 1 },
                SsaTerminator::Jump { target: 2 },
                SsaTerminator::Return { value: None },
            ],
        );
        let sem = run(&cfg, &ssa);
        assert_eq!(sem.body.stmts.len(), 1, "single Return after Jump chain");
        assert!(matches!(sem.body.stmts[0], Stmt::Return { .. }));
        assert_eq!(sem.stats.source_blocks, 3);
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_03_diamond_becomes_if_else_with_merge() {
        // 0: brcond 1, 2 ; 1: jmp 3 ; 2: jmp 3 ; 3: ret
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Const(1),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Return { value: None },
            ],
        );
        let sem = run(&cfg, &ssa);
        // Expect: [If { then: [], else: None or [] }, Return]
        // The arms are empty because both jumps land at the join (block 3).
        assert_eq!(sem.body.stmts.len(), 2);
        match &sem.body.stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                source_block,
                ..
            } => {
                assert_eq!(*source_block, 0);
                assert!(then_body.is_empty(), "taken arm goes straight to join");
                assert!(
                    else_body.is_none() || else_body.as_ref().unwrap().is_empty(),
                    "not-taken arm goes straight to join"
                );
            }
            other => panic!("expected Stmt::If, got {other:?}"),
        }
        assert!(matches!(sem.body.stmts[1], Stmt::Return { .. }));
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_04_diamond_with_arm_contents() {
        // 0: brcond 1, 2 ; 1: instr; jmp 3 ; 2: instr; jmp 3 ; 3: ret
        // Add one fake instruction to each arm.
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let mut ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Return { value: None },
            ],
        );
        ssa.blocks[1].instructions.push(SsaInstruction {
            dst: None,
            op: SsaOp::Opaque {
                mnemonic: "stub".to_string(),
                args: Vec::new(),
            },
        });
        ssa.blocks[2].instructions.push(SsaInstruction {
            dst: None,
            op: SsaOp::Opaque {
                mnemonic: "stub".to_string(),
                args: Vec::new(),
            },
        });
        let sem = run(&cfg, &ssa);
        assert_eq!(sem.body.stmts.len(), 2);
        match &sem.body.stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.stmts.len(), 1, "taken arm = [Instr]");
                assert!(matches!(then_body.stmts[0], Stmt::Instr { .. }));
                let else_body = else_body.as_ref().expect("else arm");
                assert_eq!(else_body.stmts.len(), 1, "not-taken arm = [Instr]");
                assert!(matches!(else_body.stmts[0], Stmt::Instr { .. }));
            }
            other => panic!("expected Stmt::If, got {other:?}"),
        }
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_05_early_return_on_taken_side_no_join_needed() {
        // 0: brcond 1, 2 ; 1: ret ; 2: ret
        let cfg = synthetic_cfg(3, 0, &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)]);
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Return {
                    value: Some(Operand::Const(0)),
                },
                SsaTerminator::Return {
                    value: Some(Operand::Const(1)),
                },
            ],
        );
        let sem = run(&cfg, &ssa);
        // Expect: [If { then: [Return], else: Some([Return]) }]
        assert_eq!(sem.body.stmts.len(), 1);
        match &sem.body.stmts[0] {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.stmts.len(), 1);
                assert!(matches!(then_body.stmts[0], Stmt::Return { .. }));
                let else_body = else_body.as_ref().expect("else arm with Return");
                assert_eq!(else_body.stmts.len(), 1);
                assert!(matches!(else_body.stmts[0], Stmt::Return { .. }));
            }
            other => panic!("expected Stmt::If, got {other:?}"),
        }
        assert!(sem.stats.is_goto_free());
        assert_eq!(sem.stats.source_blocks, 3);
    }

    #[test]
    fn s_06_while_loop_becomes_loop_with_if_break() {
        // 0: jmp 1 ; 1 (header): brcond 2, 3 ; 2: jmp 1 ; 3: ret
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::NotTaken),
                (2, 1, EdgeKind::Branch),
            ],
        );
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Jump { target: 1 },
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 2,
                    not_taken: 3,
                },
                SsaTerminator::Jump { target: 1 },
                SsaTerminator::Return { value: None },
            ],
        );
        let sem = run(&cfg, &ssa);
        // Expected outer shape: [Loop { ... }, Return]
        assert_eq!(sem.body.stmts.len(), 2);
        match &sem.body.stmts[0] {
            Stmt::Loop { body, header, .. } => {
                assert_eq!(*header, 1);
                // Body should contain an If with one arm Continue, other Break.
                let mut saw_continue = false;
                let mut saw_break = false;
                for s in &body.stmts {
                    if let Stmt::If {
                        then_body,
                        else_body,
                        ..
                    } = s
                    {
                        for t in &then_body.stmts {
                            if matches!(t, Stmt::Continue { .. }) {
                                saw_continue = true;
                            }
                            if matches!(t, Stmt::Break { .. }) {
                                saw_break = true;
                            }
                        }
                        if let Some(eb) = else_body {
                            for t in &eb.stmts {
                                if matches!(t, Stmt::Continue { .. }) {
                                    saw_continue = true;
                                }
                                if matches!(t, Stmt::Break { .. }) {
                                    saw_break = true;
                                }
                            }
                        }
                    }
                }
                assert!(saw_continue, "loop body should contain Continue");
                assert!(saw_break, "loop body should contain Break");
            }
            other => panic!("expected Stmt::Loop, got {other:?}"),
        }
        assert!(matches!(sem.body.stmts[1], Stmt::Return { .. }));
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_07_self_loop_becomes_endless_loop() {
        // 0: instr; jmp 0 (no exit)
        let cfg = synthetic_cfg(1, 0, &[(0, 0, EdgeKind::Branch)]);
        let ssa = synthetic_ssa(&cfg, vec![SsaTerminator::Jump { target: 0 }]);
        let sem = run(&cfg, &ssa);
        assert_eq!(sem.body.stmts.len(), 1);
        match &sem.body.stmts[0] {
            Stmt::Loop { body, header, .. } => {
                assert_eq!(*header, 0);
                // Body: just a Continue (the Jump target is the header).
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0], Stmt::Continue { .. }));
            }
            other => panic!("expected Stmt::Loop, got {other:?}"),
        }
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_08_nested_while_loops() {
        // Outer loop with inner loop:
        // 0 → 1 (outer header)
        // 1 → 2 (inner header)
        // 2 → 3 (body); 3 → 2 (inner back); 3 → 4
        // 4 → 1 (outer back); 4 → 5 (outer exit)
        // 5: ret
        let cfg = synthetic_cfg(
            6,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
                (3, 2, EdgeKind::Taken),
                (3, 4, EdgeKind::NotTaken),
                (4, 1, EdgeKind::Taken),
                (4, 5, EdgeKind::NotTaken),
            ],
        );
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Jump { target: 1 },
                SsaTerminator::Jump { target: 2 },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 2,
                    not_taken: 4,
                },
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 5,
                },
                SsaTerminator::Return { value: None },
            ],
        );
        let sem = run(&cfg, &ssa);
        // Top-level: [OuterLoop, Return]
        assert_eq!(sem.body.stmts.len(), 2);
        let outer_body = match &sem.body.stmts[0] {
            Stmt::Loop { body, header, .. } => {
                assert_eq!(*header, 1);
                body
            }
            other => panic!("expected outer Stmt::Loop, got {other:?}"),
        };
        // Inner Loop appears somewhere in outer_body.
        let saw_inner_loop = outer_body
            .stmts
            .iter()
            .any(|s| matches!(s, Stmt::Loop { header, .. } if *header == 2));
        assert!(saw_inner_loop, "inner loop should be present");
        assert!(matches!(sem.body.stmts[1], Stmt::Return { .. }));
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_09_irreducible_cfg_uses_goto_fallback() {
        // 0 → 1; 0 → 2; 1 → 2; 2 → 1 — two-entry cycle.
        let cfg = synthetic_cfg(
            3,
            0,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 2, EdgeKind::Branch),
                (2, 1, EdgeKind::Branch),
            ],
        );
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Jump { target: 2 },
                SsaTerminator::Jump { target: 1 },
            ],
        );
        let sem = run(&cfg, &ssa);
        assert!(sem.stats.irreducible);
        assert!(
            sem.stats.goto_count > 0,
            "irreducible CFG should produce at least one goto"
        );
        // Every Goto's target should be matched by exactly one Label
        // in the body.
        let mut goto_targets: Vec<LabelId> = Vec::new();
        let mut label_ids: Vec<LabelId> = Vec::new();
        collect_goto_label(&sem.body, &mut goto_targets, &mut label_ids);
        for t in &goto_targets {
            assert!(
                label_ids.contains(t),
                "goto target {t} should be anchored by a Label"
            );
        }
        assert_eq!(sem.stats.label_count, label_ids.len() as u32);
    }

    fn collect_goto_label(body: &Block, gotos: &mut Vec<LabelId>, labels: &mut Vec<LabelId>) {
        for s in &body.stmts {
            match s {
                Stmt::Goto { target, .. } => gotos.push(*target),
                Stmt::Label { id, .. } => labels.push(*id),
                Stmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    collect_goto_label(then_body, gotos, labels);
                    if let Some(eb) = else_body {
                        collect_goto_label(eb, gotos, labels);
                    }
                }
                Stmt::While { body, .. } => collect_goto_label(body, gotos, labels),
                Stmt::DoWhile { body, .. } => collect_goto_label(body, gotos, labels),
                Stmt::Loop { body, .. } => collect_goto_label(body, gotos, labels),
                Stmt::Switch { arms, default, .. } => {
                    for a in arms {
                        collect_goto_label(&a.body, gotos, labels);
                    }
                    if let Some(d) = default {
                        collect_goto_label(d, gotos, labels);
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn s_10_structuring_is_byte_deterministic() {
        // Same SSA function in twice → identical SemFunction out.
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Return { value: None },
            ],
        );
        let a = run(&cfg, &ssa);
        let b = run(&cfg, &ssa);
        assert_eq!(a, b);
    }

    #[test]
    fn s_11_block_with_phis_emits_them_before_instrs() {
        // 0: brcond 1, 2 ; 1: jmp 3 ; 2: jmp 3 ; 3: phi + ret.
        let cfg = synthetic_cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Branch),
            ],
        );
        let mut ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Jump { target: 3 },
                SsaTerminator::Return {
                    value: Some(Operand::Value(0)),
                },
            ],
        );
        ssa.blocks[3].phis.push(Phi {
            dst: 0,
            variable: 0,
            incoming: vec![(1, Operand::Undef), (2, Operand::Undef)],
        });
        let sem = run(&cfg, &ssa);
        // Top-level: [If, Phi, Return]
        assert_eq!(sem.body.stmts.len(), 3);
        assert!(matches!(sem.body.stmts[0], Stmt::If { .. }));
        match &sem.body.stmts[1] {
            Stmt::Phi { r, .. } => {
                assert_eq!(r.block, 3);
                assert_eq!(r.index, 0);
            }
            other => panic!("expected Stmt::Phi, got {other:?}"),
        }
        assert!(matches!(sem.body.stmts[2], Stmt::Return { .. }));
    }

    #[test]
    fn s_12_indirect_terminator_is_marked_unreachable() {
        let cfg = synthetic_cfg(1, 0, &[]);
        let ssa = synthetic_ssa(&cfg, vec![SsaTerminator::Indirect]);
        let sem = run(&cfg, &ssa);
        assert_eq!(sem.body.stmts.len(), 1);
        match &sem.body.stmts[0] {
            Stmt::Unreachable { source_block, .. } => {
                assert_eq!(*source_block, 0);
            }
            other => panic!("expected Stmt::Unreachable, got {other:?}"),
        }
    }

    #[test]
    fn s_13_empty_function_produces_empty_body() {
        let cfg = synthetic_cfg(0, 0, &[]);
        let ssa = SsaFunction {
            function_address: 0x1000,
            function_name: None,
            blocks: Vec::new(),
            entry: 0,
            variables: Vec::new(),
            values: Vec::new(),
            evidence: ev(),
        };
        let sem = run(&cfg, &ssa);
        assert!(sem.body.is_empty());
        assert_eq!(sem.stats.source_blocks, 0);
        assert!(sem.stats.is_goto_free());
    }

    #[test]
    fn s_14_compare_terminator_preserves_cond_operand() {
        // Verify the cond operand round-trips through the structurer
        // without modification. This pins the relationship between
        // SsaTerminator::Branch::cond and Stmt::If::cond.
        let cfg = synthetic_cfg(3, 0, &[(0, 1, EdgeKind::Taken), (0, 2, EdgeKind::NotTaken)]);
        let ssa = synthetic_ssa(
            &cfg,
            vec![
                SsaTerminator::Branch {
                    cond: Operand::Value(42),
                    taken: 1,
                    not_taken: 2,
                },
                SsaTerminator::Return { value: None },
                SsaTerminator::Return { value: None },
            ],
        );
        let sem = run(&cfg, &ssa);
        match &sem.body.stmts[0] {
            Stmt::If { cond, .. } => assert_eq!(*cond, Operand::Value(42)),
            other => panic!("expected Stmt::If, got {other:?}"),
        }
    }

    #[test]
    fn s_15_compare_kind_unused_but_module_compiles() {
        // Compile-time guard: `CompareKind` is referenced by the
        // tests so removing it from the SSA layer breaks this module
        // at compile time, surfacing the dependency.
        let _ = CompareKind::Eq;
    }
}
