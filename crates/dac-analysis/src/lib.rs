//! `dac-analysis` — CFG, SSA, dataflow, and type analyses for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What's landed
//!
//! - [`cfg`] — per-function control-flow graph construction (B2.1, FR-10).
//!   Basic blocks, edges, entry / exits, unreachable detection, plus a
//!   deterministic DOT renderer for `--emit-cfg` (FR-28).
//! - [`dom`] — dominator and post-dominator trees (B2.2, FR-10).
//!   Cooper-Harvey-Kennedy iterative dominance with a synthetic
//!   virtual exit for the post-dominator computation.
//! - [`loops`] — natural loops, loop nest forest, reducibility (B2.2,
//!   FR-10). Back-edge detection via the dominator tree, body
//!   construction by reverse BFS, irreducibility flagged via SCC
//!   entry-point counts.
//! - [`ssa`] — SSA construction (B2.3, FR-11). Pruned phi placement
//!   via Cytron-Ferrante-Rosen-Wegman-Zadeck with liveness, dominator-
//!   tree rename, and a local value-numbering pass for trivial CSE.
//! - [`dataflow`] — SSA-level def-use chains and per-block liveness
//!   (B2.4, FR-11). Use-def is implicit in SSA so the module exposes
//!   a thin [`dataflow::def_of`] wrapper rather than a separate table.
//! - [`structuring`] — control-flow structuring (B2.7, FR-18,
//!   spec §11.3). Folds SSA + CFG + dominators + post-dominators +
//!   loop forest into a [`dac_ir::sem::SemFunction`] tree
//!   (`if` / `loop` / `break` / `continue` / `return`), with a goto
//!   fallback for irreducible CFGs.
//! - [`xrefs`] — whole-program call graph and cross-reference index
//!   (B3.1, FR-26, FR-27). Reads `ControlFlow::{Call, IndirectCall,
//!   UnconditionalBranch}` per function plus the relocation table and
//!   export table to produce a [`CallGraph`] and an [`XrefIndex`].
//!
//! ## What's coming
//!
//! Calling-convention inference (B2.5) and type lattice / propagation
//! (B2.6) live in `dac-recovery` instead — the doc comment in the
//! diagram above is the architectural intent, the actual home is the
//! crate that already consumes `dac-knowledge`.

#![forbid(unsafe_code)]

pub mod cfg;
pub mod dataflow;
pub mod dom;
pub mod loops;
pub mod ssa;
pub mod structuring;
pub mod xrefs;

pub use structuring::structure;
pub use xrefs::{
    build_call_graph, build_xref_index, render_callgraph_dot, resolve_subject, CallEdge, CallGraph,
    CallNode, CallNodeKind, Xref, XrefIndex, XrefKind, DIRECT_CALL_CONFIDENCE, EXPORT_CONFIDENCE,
    EXTERNAL_VA, INDIRECT_CALL_CONFIDENCE, RELOC_CONFIDENCE, TAIL_CALL_CONFIDENCE,
    UNRESOLVED_DIRECT_CALL_CONFIDENCE,
};

#[cfg(test)]
mod test_support;
