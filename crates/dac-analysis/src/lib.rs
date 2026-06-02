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
//!
//! ## What's coming
//!
//! SSA (B2.3), dataflow (B2.4), calling-convention inference (B2.5),
//! and type lattice / propagation (B2.6) all land into this crate
//! behind their own modules and milestones.

#![forbid(unsafe_code)]

pub mod cfg;
pub mod dom;
pub mod loops;

#[cfg(test)]
mod test_support;
