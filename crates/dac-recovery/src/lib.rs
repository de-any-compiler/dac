//! `dac-recovery` — function, name, struct, and idiom recovery for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What ships when
//!
//! - **B1.5.** [`functions`] — function-boundary recovery from
//!   symbols, the entry point, direct-call edges, and x86 prologue
//!   patterns (FR-9). Each discovered function is recorded in the
//!   evidence graph as a `Cfg`-layer [`dac_core::EvidenceNode::IrNode`]
//!   supported by a [`dac_core::EvidenceNode::Bytes`] node covering its
//!   byte span.
//! - **B2.4 (this batch).** [`stack`] — stack-frame recovery (FR-12):
//!   identify stack locals, incoming stack arguments, and (on Windows
//!   x64) the home/shadow space, from SSA address arithmetic anchored
//!   at the function's entry `rsp`.
//! - **B3.2.** struct / array recovery.
//! - **B3.3.** idiom recognition.
//! - **B3.7.** variable-naming heuristics.

#![forbid(unsafe_code)]

pub mod functions;
pub mod stack;

pub use functions::{
    discover_functions, DiscoveryStats, Function, FunctionSet, SourceMask, CALL_EDGE_CONFIDENCE,
    ENTRY_CONFIDENCE, PROLOGUE_CONFIDENCE, SYMBOL_CONFIDENCE,
};
pub use stack::{
    analyze_stack_frame, FramePointer, StackConvention, StackFrame, StackLocal, StackLocalKind,
};
