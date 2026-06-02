//! `dac-analysis` — CFG, SSA, dataflow, and type analyses for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What's landed
//!
//! - [`cfg`] — per-function control-flow graph construction (B2.1, FR-10).
//!   Basic blocks, edges, entry / exits, unreachable detection, plus a
//!   deterministic DOT renderer for `--emit-cfg` (FR-28).
//!
//! ## What's coming
//!
//! Dominators / loops (B2.2), SSA (B2.3), dataflow (B2.4), calling-
//! convention inference (B2.5), and type lattice / propagation (B2.6) all
//! land into this crate behind their own modules and milestones.

#![forbid(unsafe_code)]

pub mod cfg;
