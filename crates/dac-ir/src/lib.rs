//! `dac-ir` — intermediate representations for dac.
//!
//! Houses every IR layer: Instruction IR, CFG IR, SSA IR, Semantic IR, and
//! Source IR. See `ARCHITECTURE.md` §4 in the workspace root.
//!
//! ## What ships when
//!
//! - **B1.4.** [`instr`] — Instruction IR.
//! - **B2.1.** CFG IR lives in `dac-analysis::cfg` for now; the
//!   long-term home is `dac-ir::cfg`, but no consumer has needed the
//!   split yet.
//! - **B2.3 (this batch).** [`ssa`] — phi nodes, def-use chains.
//! - **B2.7.** `sem` — semantic IR (typed, structured).
//! - **B2.7 / M3.** `src` — language-neutral source AST.

#![forbid(unsafe_code)]

pub mod instr;
pub mod ssa;

pub use instr::{Condition, InstructionIr, Operand, Operation, Target};
