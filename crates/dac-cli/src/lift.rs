//! Per-function end-to-end lift orchestration (B3.9, FR-21).
//!
//! The CLI runs the deterministic pipeline once per recovered
//! function:
//!
//! ```text
//!   Function
//!     → build_cfg                  (dac-analysis, B1.7)
//!     → InstructionIr per block    (dac-arch-x86::IcedLifter, B1.4)
//!     → RawFunction                (dac-lift::lift_function, B3.8)
//!     → SsaFunction                (dac-analysis::ssa::construct_ssa, B2.3)
//!     → SemFunction                (dac-analysis::structuring::structure, B2.7)
//! ```
//!
//! When any step short-circuits (no recovered `end`, `build_cfg`
//! returns `None`, etc.) the per-function outcome is a [`LiftOutcome::Stub`]
//! with a human-readable reason that the source-emitting code
//! surfaces in the leading comment (I-6: degrade visibly, never
//! invent semantics).
//!
//! ## Determinism
//!
//! Every constituent pass is `Determinism::Pure` (NFR-9). The
//! orchestrator iterates `FunctionSet::functions` in its existing
//! address-sorted order and threads the same register file into every
//! call, so two runs on the same bytes produce identical
//! `LiftOutcome` vectors.

use dac_analysis::cfg::build_cfg;
use dac_analysis::dom::{DominatorTree, PostDominatorTree};
use dac_analysis::loops::LoopForest;
use dac_analysis::ssa::construct_ssa;
use dac_analysis::structuring::structure;
use dac_arch::{InstructionDecoder, InstructionLifter, RegisterFile};
use dac_binfmt::BinaryModel;
use dac_ir::instr::InstructionIr;
use dac_ir::sem::SemFunction;
use dac_ir::ssa::SsaFunction;
use dac_lift::lift_function;
use dac_recovery::{Function, FunctionSet};

/// Per-function outcome of the orchestrator.
pub(crate) enum LiftOutcome {
    /// Pipeline ran end-to-end; both the SSA and Semantic IR
    /// representations are populated.
    Real { ssa: SsaFunction, sem: SemFunction },
    /// Pipeline could not produce a Semantic IR function. `reason` is
    /// rendered into the leading comment of the emitted stub.
    Stub { reason: String },
}

/// Run the per-function orchestrator across the whole recovered
/// function set. The returned vector is in the same order as
/// `functions.functions`, so callers can zip the two together.
#[must_use]
pub(crate) fn lift_all(
    functions: &FunctionSet,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    register_file: &RegisterFile,
) -> Vec<LiftOutcome> {
    functions
        .functions
        .iter()
        .map(|f| lift_one(f, model, bytes, decoder, lifter, register_file))
        .collect()
}

/// Aggregate lift statistics. The CLI threads this into the
/// `--emit-report` output so a reader can tell how much of the binary
/// the deterministic pipeline reconstructed end-to-end.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LiftStats {
    pub real: u64,
    pub stub: u64,
}

impl LiftStats {
    pub(crate) fn from(outcomes: &[LiftOutcome]) -> Self {
        let mut s = Self::default();
        for o in outcomes {
            match o {
                LiftOutcome::Real { .. } => s.real += 1,
                LiftOutcome::Stub { .. } => s.stub += 1,
            }
        }
        s
    }

    pub(crate) fn total(self) -> u64 {
        self.real + self.stub
    }

    pub(crate) fn fraction(self) -> f32 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.real as f32 / t as f32
        }
    }
}

fn lift_one(
    f: &Function,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    register_file: &RegisterFile,
) -> LiftOutcome {
    if f.end.is_none() {
        return LiftOutcome::Stub {
            reason: "no recovered end address".into(),
        };
    }
    let Some(cfg) = build_cfg(f, model, bytes, decoder) else {
        return LiftOutcome::Stub {
            reason: "cfg-build failed (byte range unreachable or empty)".into(),
        };
    };

    let instructions_per_block: Vec<Vec<InstructionIr>> = cfg
        .blocks
        .iter()
        .map(|b| {
            b.instructions
                .iter()
                .map(|d| lifter.lift(&d.bytes, d.address))
                .collect()
        })
        .collect();

    let raw = lift_function(&cfg, &instructions_per_block, register_file);
    let doms = DominatorTree::build(&cfg);
    let ssa = construct_ssa(&cfg, &doms, &raw);
    let pdoms = PostDominatorTree::build(&cfg);
    let loops = LoopForest::build(&cfg, &doms);
    let sem = structure(&ssa, &cfg, &doms, &pdoms, &loops);

    LiftOutcome::Real { ssa, sem }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_stats_round_trip() {
        let outcomes = vec![
            LiftOutcome::Stub {
                reason: "r1".into(),
            },
            LiftOutcome::Real {
                ssa: SsaFunction {
                    function_address: 0,
                    function_name: None,
                    blocks: Vec::new(),
                    entry: 0,
                    variables: Vec::new(),
                    values: Vec::new(),
                    evidence: dac_core::EvidenceGraph::new().add_node(
                        dac_core::EvidenceNode::IrNode {
                            layer: dac_core::IrLayer::Ssa,
                            id: 0,
                        },
                    ),
                },
                sem: SemFunction {
                    function_address: 0,
                    function_name: None,
                    body: dac_ir::sem::Block::empty(),
                    evidence: dac_core::EvidenceGraph::new().add_node(
                        dac_core::EvidenceNode::IrNode {
                            layer: dac_core::IrLayer::Semantic,
                            id: 0,
                        },
                    ),
                    stats: dac_ir::sem::StructuringStats::default(),
                },
            },
        ];
        let s = LiftStats::from(&outcomes);
        assert_eq!(s.real, 1);
        assert_eq!(s.stub, 1);
        assert_eq!(s.total(), 2);
        assert!((s.fraction() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn empty_outcomes_have_zero_fraction() {
        let s = LiftStats::from(&[]);
        assert_eq!(s.total(), 0);
        assert_eq!(s.fraction(), 0.0);
    }
}
