//! `dac-backend-cpp` — C++ target-language backend for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §8 in the workspace
//! root.
//!
//! ## What ships at B3.5
//!
//! - [`mangle`] — a tiny Itanium-ABI mangled-name reader covering
//!   nested-name methods (`_ZN…E…`), const members (`_ZNK…`),
//!   ctor/dtor variants (`C[123]E…/D[012]E…`), free functions
//!   (`_Z<name>…`), and the special data symbols every polymorphic
//!   class produces (`_ZTV` / `_ZTI` / `_ZTS` / `_ZTT`).
//! - [`class_recovery`] — symbol-driven class recovery (FR-21).
//!   Groups mangled symbols from the binary's symbol table into a
//!   [`class_recovery::RecoveredClasses`] table. `_ZTV<class>` symbols
//!   promote a class to polymorphic; ctor / dtor variants land as
//!   distinct member entries that the lowering pass collapses for
//!   emission. Each recovered class is minted into the
//!   [`dac_core::EvidenceGraph`] as a `Source`-layer
//!   [`dac_core::EvidenceNode::IrNode`].
//! - [`ast`] — a closed C++ AST covering everything the symbol-driven
//!   recovery produces: a translation unit with includes and items, a
//!   class with optional public bases and a virtuality bit, in-class
//!   member functions with `virtual` / `const` keywords, free
//!   functions, and a coarse [`ast::CppType`] vocabulary.
//! - [`lower`] — [`lower::lower_unit`] turns
//!   [`class_recovery::RecoveredClasses`] plus a
//!   [`dac_recovery::FunctionSet`] into a [`ast::TranslationUnit`].
//!   Multiple ctor / dtor variants collapse to a single member, the
//!   variant addresses land in the leading comment so the annotation
//!   channel surfaces them, and a `virtual ~Class()` is synthesised
//!   for polymorphic classes whose dtor is not in the symbol table.
//! - [`emit`] — hand-rolled deterministic pretty-printer.
//!   [`ast::TranslationUnit`] → formatted C++ source. Same byte-stable
//!   contract as `dac-backend-c::emit`.
//! - [`compile`] — best-effort round-trip helper that pipes the
//!   emitted source through `c++ -x c++ -std=c++17 -c -` to check the
//!   output compiles. Returns [`compile::CompileResult::Skipped`] when
//!   no compiler is on `PATH`.
//!
//! ## What ships later
//!
//! - **Base-class recovery.** B3.5's lowering reserves
//!   [`ast::Class::bases`] but always leaves it empty: identifying
//!   bases requires a typeinfo-relocation walker that reads
//!   `__si_class_type_info` / `__vmi_class_type_info` shapes out of
//!   `.data.rel.ro`. That lands in a follow-up batch when the
//!   relocation reader exists.
//! - **Signature recovery.** Methods, ctors, dtors, and free
//!   functions all emit `()` parameter lists today. B3.6's user-hint
//!   plumbing brings parameter inference, and the AST already has
//!   `Param` / `CppType::Ref` / `CppType::Const` slots for it.
//! - **Real bodies.** The lifter → SSA bridge that feeds the
//!   structurer from x86-64 bytes is not yet a batch in PLAN.md, so
//!   every emitted member / free function has a stub body. The
//!   leading comment makes the degradation explicit (I-6).
//! - **Namespace lowering.** Scope chains are flattened into the
//!   class leading comment until B3.6 can ground them; the AST
//!   already carries [`ast::Class::scope_chain`] so the change is
//!   additive.
//! - **Stripped-binary recovery.** B3.5 is symbol-driven; a stripped
//!   C++ binary with no `_Z…` symbols falls through to an empty
//!   class table. A byte-level vtable scanner across `.data.rel.ro`
//!   reservation patterns lands in a later batch.
//!
//! ## Determinism
//!
//! Every public function in this crate is
//! [`Pure`](dac_core::Determinism::Pure). The compile helper shells
//! out to the system C++ compiler — that is a side effect, but it
//! never influences the emitted source, only the round-trip outcome.

#![forbid(unsafe_code)]

pub mod ast;
pub mod class_recovery;
pub mod compile;
pub mod emit;
pub mod lower;
pub mod mangle;

pub use ast::{
    AccessSpec, BaseSpec, Class, CppType, FreeFunction, Item, MemberFunction, MemberFunctionKind,
    Param, TranslationUnit,
};
pub use class_recovery::{
    recover_classes, ClassRecoveryStats, MemberCategory, RecoveredClass, RecoveredClasses,
    RecoveredFreeFunction, RecoveredMember, CLASS_SYMBOL_CONFIDENCE,
};
pub use compile::{try_compile, CompileResult};
pub use emit::{emit, emit_class, emit_free_function};
pub use lower::{default_includes, lower_unit};
pub use mangle::{parse as parse_mangled, ItaniumSymbol, MemberKind};
