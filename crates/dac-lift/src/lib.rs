//! `dac-lift` — bridge from per-instruction [`dac_ir::instr::InstructionIr`]
//! into the per-function [`dac_analysis::ssa::RawFunction`] consumed by
//! the SSA constructor (B3.8, FR-8, FR-11).
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace
//! root.
//!
//! ## What the crate does
//!
//! The lifter is the missing leg in the per-function pipeline:
//!
//! ```text
//!   bytes
//!     → InstructionIr   (dac-arch-x86::IcedLifter, B1.4)
//!     → RawFunction     (this crate, B3.8)
//!     → SsaFunction     (dac-analysis::ssa::construct_ssa, B2.3)
//!     → SemFunction     (dac-analysis::structuring::structure, B2.7)
//!     → C / C++ AST     (dac-backend-{c,cpp}::lower_*, B2.8 / B3.5)
//!     → emitted source  (dac-backend-{c,cpp}::emit::emit, B2.8 / B3.5)
//! ```
//!
//! Every step except this one shipped at its scheduled batch; until
//! B3.8, both backends emitted stub bodies because nothing translated
//! the lifter's output into the SSA constructor's input. See the B2.8 /
//! B3.4 / B3.5 CHANGELOG entries for the explicit deferral trail this
//! crate closes.
//!
//! ## What it doesn't do
//!
//! - It is not the *byte → InstructionIr* lifter. That lives in
//!   `dac-arch-x86::IcedLifter` (and the corresponding crate per ISA).
//! - It does not run SSA construction, structuring, or lowering — it
//!   only produces the input the SSA pass already knows how to
//!   consume.
//! - It does not consult the evidence graph. The graph is threaded
//!   through `Cfg::evidence`; the constructed `RawFunction` will be
//!   handed back to `construct_ssa`, which inherits the same evidence
//!   handle (I-2 traceability stays unbroken).
//!
//! See [`bridge`] for the per-instruction translation rules and the
//! list of B3.8-known-losses (subreg aliasing, x86-64-only conventions)
//! that B3.9 / B3.10 follow-ups will refine.

#![forbid(unsafe_code)]

pub mod bridge;

pub use bridge::lift_function;
