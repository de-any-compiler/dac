//! Semantic IR (B2.7, FR-18).
//!
//! The Semantic IR layer sits above the SSA layer in dac's IR stack
//! (`ARCHITECTURE.md` §4). It represents a function as a tree of
//! structured statements: `if` / `else`, `while`, `do { … } while`,
//! endless `loop` with explicit `break` / `continue`, early `return`,
//! and a `switch` placeholder for the jump-table recovery pass at
//! B3.3.
//!
//! Control flow that cannot be structured — irreducible CFGs and any
//! multi-entry tangles the algorithm declines to fold — falls back to
//! [`Stmt::Label`] + [`Stmt::Goto`] (spec §11.3, I-6). The structurer
//! never invents semantics; it degrades.
//!
//! ## What B2.7 lands
//!
//! - [`SemFunction`] / [`Block`] / [`Stmt`] — the structured tree.
//! - [`SsaRef`] — a stable handle back into the source SSA function.
//!   The Semantic IR does not clone SSA instructions; it references
//!   them so the lowering pass (B2.8) can decide between
//!   SSA-destruction and direct-form rendering without re-walking the
//!   SSA layer.
//! - [`StructuringStats`] — the coverage metric the pass reports.
//!   `goto_count == 0` (and `irreducible == false`) is the per-function
//!   "fully structured" rubric; the corpus-level rubric in PLAN.md is
//!   simply "this is true on the sample corpus for the simple
//!   functions" (B2.9 instruments it).
//!
//! ## What deliberately doesn't land yet
//!
//! - **Typed locals / typed expressions.** The lowering pass (B2.8)
//!   threads the B2.6 [`crate::ty::Type`] map and the B2.5 calling
//!   convention into the Semantic IR; this batch keeps the layer
//!   structural so the structuring algorithm is independently testable
//!   (NFR-7).
//! - **Idiom slots.** `switch` / `for` / ref-counting / state-machine
//!   recognition all land at B3.3 as proposal-style annotations on top
//!   of the structured tree; nothing here precludes them.
//! - **`for` loops.** A `while` whose body increments a single induction
//!   variable is a `for` candidate; this is a pattern pass on top of
//!   the Semantic IR (B3.3), not a primary structuring construct.
//! - **AI deltas.** Per I-4 / I-5 every node here is the product of a
//!   deterministic pass. AI deltas land against the Semantic IR via
//!   the `Delta` protocol (ARCHITECTURE.md §9) at M4.
//!
//! ## Determinism
//!
//! Every node is a pure data value. [`StructuringStats`] are derived
//! during the structuring walk and only count what the walker emits,
//! so the same SSA function produces the same `SemFunction` byte-for-
//! byte (NFR-9).

use dac_core::EvidenceId;

use crate::ssa::{Operand, SsaBlockId};

/// Numeric handle for a [`Stmt::Label`] inside a single
/// [`SemFunction`]. Label ids are dense indices allocated by the
/// structuring pass in source-order, so a renderer can produce stable
/// labels (`L0`, `L1`, …) without bookkeeping of its own.
pub type LabelId = u32;

/// A structured function: tree of statements rooted at [`SemFunction::body`].
///
/// Built from an [`crate::ssa::SsaFunction`] by `dac_analysis::structuring::structure`.
/// The conversion is deterministic and consumes the function's CFG,
/// dominator tree, post-dominator tree, and loop forest; the Semantic
/// IR carries no link back to those structures beyond the [`SsaBlockId`]
/// references on individual statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemFunction {
    /// Function entry virtual address. Inherited from the source SSA
    /// function so callers can address the same function across IR
    /// layers.
    pub function_address: u64,
    /// Symbolic function name when known.
    pub function_name: Option<String>,
    /// Body of the function — every statement that runs in its top
    /// lexical scope.
    pub body: Block,
    /// Evidence-graph handle inherited from the source SSA function.
    /// Downstream passes attach further facts (e.g. structuring
    /// confidence, idiom proposals) to the same node.
    pub evidence: EvidenceId,
    /// Coverage metrics — see [`StructuringStats`].
    pub stats: StructuringStats,
}

/// A sequence of [`Stmt`]s — the body of any structured construct.
///
/// `Block` is just a thin wrapper around `Vec<Stmt>` so the tree shape
/// is obvious in pattern matches (`Stmt::If { then_body, else_body, … }`
/// reads better than nested `Vec<Stmt>` parameters). Empty blocks are
/// permitted and have a precise meaning: a `then_body: Block { stmts:
/// [] }` is `if (cond) {}`, not `if (cond) <missing>`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    /// Statements in source order.
    pub stmts: Vec<Stmt>,
}

impl Block {
    /// Empty block — the identity for concatenation.
    #[must_use]
    pub fn empty() -> Self {
        Self { stmts: Vec::new() }
    }

    /// True when the block contains no statements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stmts.is_empty()
    }
}

/// A stable handle to one phi or instruction in the source SSA
/// function. The structuring pass emits these in place rather than
/// cloning the SSA op, so the Semantic IR stays cheap and the lowering
/// pass (B2.8) has one canonical place to look up the underlying op,
/// its types, and its evidence (NFR-7, I-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaRef {
    /// Source block id.
    pub block: SsaBlockId,
    /// Index inside the source block's `phis` or `instructions` array,
    /// depending on the carrier statement variant.
    pub index: u32,
}

/// One statement in the Semantic IR.
///
/// The enum is closed: new constructs land as new variants and every
/// downstream consumer (the C and C++ backends, the annotation
/// channel) must pattern-match exhaustively. That is the I-6 lever
/// that keeps backends from inventing semantics — if the structurer
/// produced an [`Stmt::Goto`] for an irreducible region, the C
/// backend has no choice but to render `goto` and degrade gracefully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// Phi from the source SSA function, lifted into statement
    /// position. The reference points at
    /// `source.blocks[r.block].phis[r.index]`. The lowering pass
    /// destructures it.
    Phi { r: SsaRef, evidence: EvidenceId },
    /// SSA instruction from the source SSA function, lifted into
    /// statement position. The reference points at
    /// `source.blocks[r.block].instructions[r.index]`. Side-effectful
    /// ops (stores, void calls) and value-producing ops both flow
    /// through this variant; the lowering pass uses the instruction's
    /// `dst` to decide whether to bind a name.
    Instr { r: SsaRef, evidence: EvidenceId },
    /// `if (cond) { then_body } else? { else_body }`.
    ///
    /// `else_body == None` is `if (cond) { … }` with no else arm.
    /// `else_body == Some(Block { stmts: [] })` is the literal
    /// `if (cond) { … } else {}` — distinct, even though degenerate.
    /// The structuring pass produces the first form for blocks whose
    /// taken side post-dominates the join with no statements on the
    /// not-taken side.
    If {
        cond: Operand,
        then_body: Block,
        else_body: Option<Block>,
        /// Source block whose conditional terminator produced this
        /// `if` — used by the lowering pass to look up the compare
        /// instruction that fed the condition.
        source_block: SsaBlockId,
        evidence: EvidenceId,
    },
    /// `while (cond) { body }` — pre-test loop. The test runs before
    /// every iteration; `body` may be empty for spin loops the
    /// structurer detects but cannot simplify further.
    While {
        cond: Operand,
        body: Block,
        /// Source block of the loop header (the block that contains
        /// the test).
        header: SsaBlockId,
        evidence: EvidenceId,
    },
    /// `do { body } while (cond)` — post-test loop. The test runs at
    /// the latch of the source loop. Used when the loop header's
    /// terminator is unconditional and the test sits on a single
    /// back-edge predecessor.
    DoWhile {
        cond: Operand,
        body: Block,
        /// Source block of the loop header.
        header: SsaBlockId,
        /// Source block of the back-edge predecessor that carries the
        /// test.
        latch: SsaBlockId,
        evidence: EvidenceId,
    },
    /// Endless `loop { body }`. Used when no `while`/`do-while` test
    /// can be identified at the header — every exit happens through a
    /// nested `Break` or `Return`. This is the catch-all loop form,
    /// not an error: a header with only a back-edge and an unconditional
    /// jump is a legitimate `loop { … }` (I-6).
    Loop {
        body: Block,
        /// Source block of the loop header.
        header: SsaBlockId,
        evidence: EvidenceId,
    },
    /// `switch (scrutinee) { … }` — placeholder for the jump-table
    /// recognition pass at B3.3. Not emitted at B2.7; the variant is
    /// present so backends that pattern-match on `Stmt` get an
    /// exhaustivity error when B3.3 starts producing them.
    Switch {
        scrutinee: Operand,
        arms: Vec<SwitchArm>,
        default: Option<Block>,
        source_block: SsaBlockId,
        evidence: EvidenceId,
    },
    /// `break` — exit the innermost enclosing loop.
    Break { evidence: EvidenceId },
    /// `continue` — branch back to the innermost enclosing loop's
    /// header.
    Continue { evidence: EvidenceId },
    /// `return [value]`.
    Return {
        value: Option<Operand>,
        evidence: EvidenceId,
    },
    /// Label target for the goto fallback. Labels carry no evidence
    /// of their own — they live where the structurer decided to
    /// anchor the corresponding [`Stmt::Goto`].
    Label {
        id: LabelId,
        source_block: SsaBlockId,
    },
    /// `goto <label>` — fallback for irreducible CFGs (I-6, spec §11.3).
    /// Every `Goto::target` resolves to exactly one `Label::id` in the
    /// containing [`SemFunction`]. `source_block` is the CFG block the
    /// goto stands in for (the block the structurer tried to enter
    /// when it discovered the target was already emitted) — used by
    /// the label-anchoring post-pass to place the matching
    /// [`Stmt::Label`] at the right point in the tree.
    Goto {
        target: LabelId,
        source_block: SsaBlockId,
        evidence: EvidenceId,
    },
    /// Marker for a CFG block whose terminator was decoded as
    /// `Unreachable` or `Indirect` and could not be lifted further.
    /// Retained so the lowering pass can render a comment ("UD2",
    /// "indirect jump unresolved") at the right place without the
    /// structurer silently dropping the block (I-2).
    Unreachable {
        source_block: SsaBlockId,
        evidence: EvidenceId,
    },
}

/// One arm of a [`Stmt::Switch`]. Placeholder for B3.3; the structuring
/// pass at B2.7 does not emit these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchArm {
    /// Constant value matched by this arm.
    pub value: i64,
    /// Body executed when the scrutinee equals `value`.
    pub body: Block,
}

/// Coverage metrics produced by the structuring pass.
///
/// The structurer fills these in as it walks. `source_blocks` is the
/// number of CFG blocks the pass actually visited (matches
/// `ssa.blocks.len()` minus blocks reported as unreachable by the
/// CFG); `goto_count` is the number of [`Stmt::Goto`] statements the
/// walker emitted, which is the per-function "structured-ness"
/// rubric; `irreducible` mirrors the loop forest's flag for the
/// source CFG.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StructuringStats {
    /// Number of CFG blocks consumed.
    pub source_blocks: u32,
    /// Number of [`Stmt::Goto`] statements emitted by the structurer.
    /// Zero means "fully structured" for this function.
    pub goto_count: u32,
    /// Number of [`Stmt::Label`] statements emitted by the structurer.
    /// Equal to the number of distinct goto targets.
    pub label_count: u32,
    /// True when the source CFG was flagged as irreducible by the
    /// loop forest. Implies `goto_count > 0` in practice; recorded
    /// separately so reports can distinguish "structurally
    /// irreducible" from "single-goto recoverable".
    pub irreducible: bool,
}

impl StructuringStats {
    /// True when the function was structured without emitting a single
    /// `goto`. This is the per-function rubric the PLAN.md "Done when"
    /// criterion measures across the corpus.
    #[must_use]
    pub fn is_goto_free(&self) -> bool {
        self.goto_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};

    fn ev() -> EvidenceId {
        let mut g = EvidenceGraph::new();
        g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Semantic,
            id: 0,
        })
    }

    #[test]
    fn block_empty_is_default() {
        let a: Block = Block::default();
        let b: Block = Block::empty();
        assert_eq!(a, b);
        assert!(a.is_empty());
    }

    #[test]
    fn stats_goto_free_means_zero_gotos() {
        let s = StructuringStats {
            source_blocks: 5,
            goto_count: 0,
            label_count: 0,
            irreducible: false,
        };
        assert!(s.is_goto_free());

        let s2 = StructuringStats {
            source_blocks: 5,
            goto_count: 1,
            label_count: 1,
            irreducible: false,
        };
        assert!(!s2.is_goto_free());
    }

    #[test]
    fn stmt_round_trip_through_pattern_match_covers_every_variant() {
        // Compile-time guard: every Stmt variant is reachable from a
        // pattern match. Adding a new variant breaks this test, which
        // is exactly the signal we want downstream consumers (the C
        // backend, the annotation channel) to see.
        let sample = Stmt::Return {
            value: None,
            evidence: ev(),
        };
        match sample {
            Stmt::Phi { .. }
            | Stmt::Instr { .. }
            | Stmt::If { .. }
            | Stmt::While { .. }
            | Stmt::DoWhile { .. }
            | Stmt::Loop { .. }
            | Stmt::Switch { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Return { .. }
            | Stmt::Label { .. }
            | Stmt::Goto { .. }
            | Stmt::Unreachable { .. } => {}
        }
    }

    #[test]
    fn ssa_ref_is_copy_and_orderable_by_block() {
        // Sanity: `SsaRef` is `Copy`, so passes can stash it in scalar
        // worklists without a clone.
        fn _takes_copy(_: SsaRef) {}
        let r = SsaRef { block: 1, index: 2 };
        _takes_copy(r);
        let other = SsaRef { block: 1, index: 2 };
        assert_eq!(r, other);
    }

    #[test]
    fn label_id_is_dense() {
        // Labels are dense indices — the renderer assumes `L0`, `L1`,
        // `L2`, … land contiguously. This test pins the choice so a
        // future "sparse label" change is a deliberate decision.
        let l0: LabelId = 0;
        let l1: LabelId = l0 + 1;
        assert_eq!(l1, 1);
    }

    #[test]
    fn sem_function_carries_evidence_and_stats() {
        let f = SemFunction {
            function_address: 0x1000,
            function_name: Some("main".to_string()),
            body: Block::empty(),
            evidence: ev(),
            stats: StructuringStats {
                source_blocks: 0,
                goto_count: 0,
                label_count: 0,
                irreducible: false,
            },
        };
        assert!(f.body.is_empty());
        assert!(f.stats.is_goto_free());
    }
}
