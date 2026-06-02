//! `dac-backend-c` — C target-language backend for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §8 in the workspace
//! root.
//!
//! ## What ships at B2.8
//!
//! - [`ast`] — a small closed C AST (translation units, functions,
//!   statements, expressions, types) covering everything the B2.7
//!   structurer can produce. Variants the structurer does not emit yet
//!   (`While` / `DoWhile`) are present in the vocabulary so consumers
//!   pattern-match exhaustively when later batches start producing them.
//! - [`lower`] — [`lower_function`] consumes a
//!   [`dac_ir::sem::SemFunction`] together with the underlying
//!   [`dac_ir::ssa::SsaFunction`] and produces an [`ast::Function`].
//!   [`lower_unit`] wraps a slice of lowered functions in the standard
//!   `#include` directives.
//! - [`emit`] — hand-rolled pretty-printer: [`ast::TranslationUnit`] →
//!   formatted C source string. Pure function, byte-deterministic, no
//!   external formatter.
//! - [`compile`] — best-effort round-trip helper that pipes the
//!   emitted source through `cc -x c -c -` to check the output
//!   compiles. Returns [`compile::CompileResult::Skipped`] when no
//!   compiler is on `PATH`, so unit tests stay green on machines
//!   without a toolchain.
//!
//! ## What ships later
//!
//! - **Backend trait wiring.** ARCHITECTURE.md §8 specifies a
//!   `Backend` trait that owns the lowering + emission pipeline. The
//!   trait lands when a second backend (`dac-backend-cpp`, B3.5)
//!   appears; for B2.8 there is only one consumer and a free function
//!   keeps the surface honest.
//! - **Source IR.** The architecture diagram places a language-neutral
//!   Source IR between the Semantic IR and the backend. The B2.7
//!   semantic IR is structural enough to feed the backend directly;
//!   the explicit Source IR layer lands when the C++ backend needs a
//!   different lowering policy (B3.5).
//! - **Real SSA destruction with phi-edge copies.** B2.8 sidesteps the
//!   problem by pre-declaring every SSA value as a function-local with
//!   a zero initialiser; that compiles but does not preserve iteration
//!   semantics for loop variables. The structurer-aware destructor
//!   lands once B3.3 starts emitting `While { … }` shapes that demand
//!   it.
//! - **Type-aware lowering.** [`lower_function`] currently uses each
//!   SSA variable's `width_bits` and falls back to `int64_t`. Threading
//!   `dac_recovery::TypeMap` (B2.6) through is the next refinement and
//!   lands when the orchestrator plumbs it into the call site.
//!
//! ## Determinism
//!
//! Every public function in this crate is
//! [`Pure`](dac_core::Determinism::Pure). The compile helper shells
//! out to the system compiler — that is a side effect, but it never
//! influences the emitted source, only the round-trip outcome.

#![forbid(unsafe_code)]

pub mod ast;
pub mod compile;
pub mod emit;
pub mod lower;

pub use ast::{
    BinaryOp, Block, CType, Expr, Function, Item, Local, Param, Stmt, SwitchArm, TranslationUnit,
    UnaryOp,
};
pub use compile::{try_compile, CompileResult};
pub use emit::{emit, emit_function};
pub use lower::{default_includes, lower_function, lower_unit, NameResolver, Recovered};
