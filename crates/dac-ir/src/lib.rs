//! `dac-ir` — intermediate representations for dac.
//!
//! Houses every IR layer: Instruction IR, CFG IR, SSA IR, Semantic IR, and
//! Source IR. See `ARCHITECTURE.md` §4 in the workspace root.
//!
//! ## What ships when
//!
//! - **B1.4 (this batch).** [`instr`] — Instruction IR.
//! - **B2.1.** `cfg` — basic blocks, edges, dominance, loop nest.
//! - **B2.3.** `ssa` — phi nodes, def-use chains.
//! - **B2.7.** `sem` — semantic IR (typed, structured).
//! - **B2.7 / M3.** `src` — language-neutral source AST.

#![forbid(unsafe_code)]

pub mod instr;

pub use instr::{Condition, InstructionIr, Operand, Operation, Target};
