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
//! - **B2.4.** [`stack`] — stack-frame recovery (FR-12): identify
//!   stack locals, incoming stack arguments, and (on Windows x64)
//!   the home/shadow space, from SSA address arithmetic anchored at
//!   the function's entry `rsp`.
//! - **B2.5.** [`convention`] — calling-convention inference
//!   (FR-13). Scores candidate ABIs from `dac-knowledge` against
//!   observed argument-register reads, return-register definitions,
//!   and the stack-frame layout from B2.4.
//! - **B2.6 (this batch).** [`types`] — type lattice propagation
//!   (FR-14, FR-16). Seeds values from load/store widths, API
//!   signatures in `dac-knowledge`, and the convention-inferred
//!   parameter list; iterates [`dac_ir::ty::Type::join`] through
//!   Move / arithmetic / phi to a fixed point.
//! - **B3.2.** struct / array recovery.
//! - **B3.3.** [`idioms`] — proposal-style idiom recognition
//!   (FR-18, spec §11.4). Side-table only: scans the SSA function
//!   for compiler-emitted jump tables on x86-64 (terminator
//!   [`dac_ir::ssa::SsaTerminator::Indirect`] anchoring a `Load`
//!   from `Add(base, Mul(idx, c))` / `Add(base, Shl(idx, k))`) and
//!   surfaces them as [`idioms::SwitchTableIdiom`] records,
//!   optionally pinned by a bounding
//!   [`dac_ir::ssa::CompareKind::Ult`] /
//!   [`dac_ir::ssa::CompareKind::Ule`] in the predecessor.
//! - **B3.7.** [`names`] — variable-naming heuristics
//!   (FR-N spec §11.1). Two deterministic heuristics shipped at
//!   B3.7: API-context names from [`dac_knowledge::ApiSignature`]
//!   catalogues and string-literal slugs read out of the binary's
//!   `.rodata`.
//! - **B3.20 (this batch).** [`names`] — three additional dataflow
//!   heuristics: loop-induction (`i` / `j` / `k` by nesting depth),
//!   counter (`count` for non-induction `+= 1` phis), and
//!   allocator-size (`size` for arithmetic feeding a `malloc` /
//!   `calloc` / `realloc` size argument). The CLI passes a
//!   [`names::LoopInfo`] summary derived from
//!   `dac_analysis::loops::LoopForest`; the summary is lifted out
//!   into a small POD so `dac-recovery` does not need to depend on
//!   `dac-analysis` (which already depends on us). The
//!   [`names::NameTable`] is consumed by the C backend in place of
//!   the `v<id>` fallback.

#![forbid(unsafe_code)]

pub mod convention;
pub mod functions;
pub mod idioms;
pub mod names;
pub mod simplify;
pub mod stack;
pub mod structs;
pub mod types;

pub use convention::{
    infer_calling_convention, pick_best, ConventionMatch, InferredSignature, RegisterArg, StackArg,
};
pub use functions::{
    discover_functions, DiscoveryStats, Function, FunctionKind, FunctionSet, SourceMask,
    CALL_EDGE_CONFIDENCE, ENTRY_CONFIDENCE, PLT_BINDING_CONFIDENCE, PROLOGUE_CONFIDENCE,
    SYMBOL_CONFIDENCE,
};
pub use idioms::{
    recover_idioms, resolve_switch_entries, RecoveredIdioms, ResolvedSwitchEntry, SwitchBound,
    SwitchTableIdiom, MAX_SWITCH_ENTRIES, SWITCH_TABLE_CONFIDENCE,
};
pub use names::{
    recover_names, CallRenameResolver, LoopInfo, LoopShape, NameCandidate, NameSource, NameTable,
    NullCallRenameResolver, NullStringResolver, StringResolver, NAME_CONFIDENCE,
};
pub use simplify::{simplify, value_has_definition, SimplifyStats};
pub use stack::{
    analyze_stack_frame, FramePointer, StackConvention, StackFrame, StackLocal, StackLocalKind,
};
pub use structs::{
    recover_structs, ArrayLayout, FieldCandidate, RecoveredStructs, StructLayout,
    ARRAY_INDEXED_CONFIDENCE, POINTER_BASE_CONFIDENCE, STACK_CLUSTER_CONFIDENCE,
};
pub use types::{propagate_types, ApiResolver, LocalType, NullApiResolver, TypeMap, ValueType};
