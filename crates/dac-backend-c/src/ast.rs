//! C AST — the intermediate representation produced by [`crate::lower`]
//! and consumed by [`crate::emit`].
//!
//! The AST is intentionally small. Every node corresponds to a
//! syntactic C construct that the [`crate::lower`] pass can synthesise
//! from a [`dac_ir::sem::SemFunction`] without inventing semantics
//! (I-6). Constructs the backend cannot express end up as
//! [`Stmt::Comment`] / [`Expr::Opaque`] entries that compile to a
//! comment or a placeholder identifier rather than silently
//! disappearing.
//!
//! ## Scope
//!
//! B2.8 is the `-O1` backend, so the vocabulary covers:
//!
//! - Translation-unit-level **#include** directives and function
//!   definitions ([`TranslationUnit`], [`Item`], [`Function`]).
//! - Per-function declarations, assignments, control flow, returns,
//!   `break` / `continue`, `goto` / labels, and a `/* … */` comment
//!   carrier ([`Stmt`]).
//! - Integer and pointer type spellings ([`CType`]).
//! - Variable references, integer literals, binary / unary operations,
//!   memory loads, and function calls ([`Expr`]).
//!
//! Structs, arrays, `switch`, function pointers, and floating-point
//! types are deliberately absent — they enter the AST when the matching
//! Semantic IR shapes start producing them (B3.2 / B3.3 / B3.5).
//!
//! ## Determinism
//!
//! Every node is a pure data value. `Vec<…>` orderings are the source
//! order produced by the lowering pass. There is no interior mutability
//! and no hashing in the data model, so the emitter renders the same
//! AST byte-for-byte every time.

use dac_ir::sem::LabelId;

/// A whole C translation unit — top of the AST. The lowering pass emits
/// one of these per binary; in B2.8 the unit is `dac-recovered.c` and
/// contains the include directives the backend always needs (`<stdint.h>`
/// for the integer-width typedefs, `<stddef.h>` for `NULL` and
/// `size_t`) plus one [`Item::Function`] per recovered function.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranslationUnit {
    /// `#include` directives in source order. The lowering pass
    /// populates the standard set; callers can append additional
    /// includes when they know the binary needs them.
    pub includes: Vec<String>,
    /// Top-level items in source order.
    pub items: Vec<Item>,
}

/// One top-level item in a [`TranslationUnit`]. The enum is closed so
/// future kinds (globals, function-pointer typedefs) land as new
/// variants and break the emitter at compile time rather than
/// silently producing nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A function definition.
    Function(Function),
    /// `typedef struct { … } NAME;` — surface for a recovered
    /// pointer-anchored struct layout (B3.16, FR-17). Each typedef is
    /// rendered with `__attribute__((packed))` so the per-field
    /// `offset` produced by `dac-recovery::structs` survives lowering
    /// to C: a `field_<hex>` reference resolves to the same byte
    /// position the recovery observed at the SSA layer.
    StructDecl(StructDecl),
}

/// A `typedef struct { … } NAME;` declaration emitted at translation-
/// unit scope (B3.16). The lowering pass synthesises one of these per
/// pointer-anchored layout in [`dac_recovery::RecoveredStructs::pointer_structs`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    /// Typedef'd name (e.g. `S_140001234_v5_t`). Emitted both as the
    /// `struct` tag-less typedef and as the C-side identifier other
    /// items reference via [`CType::Named`].
    pub name: String,
    /// Fields in source order. The emitter adds padding fields
    /// (`uint8_t __pad_<from>_<to>[N];`) where the recovered layout
    /// has gaps so the offsets the recovery pass observed (e.g.
    /// `field_60` lives at byte 0x60) survive the round-trip compile
    /// gate.
    pub fields: Vec<StructField>,
    /// Optional `/* … */` comment rendered above the typedef.
    pub leading_comment: Option<String>,
}

/// One member of a [`StructDecl`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ty: CType,
}

/// A C function definition.
///
/// The lowering pass produces one of these per [`dac_ir::sem::SemFunction`].
/// `leading_comment` records the provenance (function address, source
/// signals, structurer stats) so a human reviewer can trace any
/// emitted function back to the binary even when the source map is
/// absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    /// Function name as it appears in the emitted source.
    pub name: String,
    /// Return type.
    pub return_type: CType,
    /// Parameter list. Empty means `(void)`.
    pub params: Vec<Param>,
    /// Local declarations to lift to the top of the function body.
    /// These come from the SSA destruction step (phi targets, address-
    /// taken stack slots). Variables that the lowering pass can
    /// declare inline are emitted as [`Stmt::Decl`] inside `body`.
    pub locals: Vec<Local>,
    /// Function body.
    pub body: Block,
    /// Optional `/* provenance: … */` comment rendered above the
    /// signature.
    pub leading_comment: Option<String>,
}

/// One parameter of a [`Function`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: CType,
}

/// A top-of-function local declaration. Initialiser is `None` for
/// uninitialised locals (the SSA destructor decides what initial value
/// is safe; without that pass, B2.8 emits `0` for any local it has to
/// pre-declare).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    pub name: String,
    pub ty: CType,
    pub init: Option<Expr>,
}

/// A block — `{ … }`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    /// Statements in source order.
    pub stmts: Vec<Stmt>,
}

impl Block {
    /// Empty block.
    #[must_use]
    pub fn empty() -> Self {
        Self { stmts: Vec::new() }
    }

    /// True when the block contains no statements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stmts.is_empty()
    }
}

/// One C statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    /// `ty name = init;` (or `ty name;` when `init` is `None`).
    Decl {
        ty: CType,
        name: String,
        init: Option<Expr>,
    },
    /// `name = value;`
    Assign { name: String, value: Expr },
    /// `*(ty *)(address) = value;`
    Store {
        ty: CType,
        address: Expr,
        value: Expr,
    },
    /// `base.field = value;` (when `arrow` is false) or
    /// `base->field = value;` (when `arrow` is true). Produced by
    /// the B3.10 lowering when a `Store` address decomposes to a
    /// known struct-field offset (FR-17).
    FieldStore {
        base: Expr,
        field: String,
        arrow: bool,
        value: Expr,
    },
    /// `expr;` — for side-effect expressions (calls, opaque ops).
    ExprStmt(Expr),
    /// `if (cond) { then_body } [else { else_body }]`.
    If {
        cond: Expr,
        then_body: Block,
        else_body: Option<Block>,
    },
    /// `while (1) { body }` — endless loop. The B2.7 structurer always
    /// emits this shape, paired with explicit `if (!cond) break;` /
    /// `continue;` inside the body. The recogniser that promotes this
    /// to a `while`/`for` form lives in B3.3.
    Loop { body: Block },
    /// `while (cond) { body }` — produced once the recogniser at B3.3
    /// lands. Present in the AST so backends pattern-match exhaustively
    /// from day one.
    While { cond: Expr, body: Block },
    /// `do { body } while (cond);`
    DoWhile { body: Block, cond: Expr },
    /// `break;`
    Break,
    /// `continue;`
    Continue,
    /// `return [value];`
    Return(Option<Expr>),
    /// `L<id>:` label.
    Label(LabelId),
    /// `goto L<id>;`
    Goto(LabelId),
    /// `switch (scrutinee) { case v: …; … [default: …] }`. Arms are
    /// emitted in `SwitchArm.value` order; `default` lowers to a
    /// `default:` arm. The B3.10 switch post-pass produces this when
    /// it recognises a [`dac_recovery::SwitchTableIdiom`]; arms are
    /// empty at B3.10 (table-entry resolution is a follow-up) but the
    /// scrutinee is surfaced so the reader sees the recognised idiom
    /// (FR-18, I-6).
    Switch {
        scrutinee: Expr,
        arms: Vec<SwitchArm>,
        default: Option<Block>,
    },
    /// `/* … */` comment carrier.
    Comment(String),
    /// `__builtin_unreachable();` — emitted for SSA blocks whose
    /// terminator was decoded as `Unreachable` or `Indirect`. The
    /// preceding [`Stmt::Comment`] holds the original reason so the
    /// reader knows whether the structurer reached a `ud2`, an
    /// unresolved indirect jump, or an unreachable block (I-6).
    Unreachable,
}

/// A C expression. Side-effectful expressions (calls, stores) embed
/// here too — the lowering pass decides whether to attach them via
/// [`Stmt::ExprStmt`] or as the RHS of a [`Stmt::Decl`] / [`Stmt::Assign`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Variable reference: `v0`, `rax_3`, …
    Var(String),
    /// Integer literal. The emitter picks the suffix from `signed`
    /// (`-1` vs `0xff u`).
    IntLit { value: i64, signed: bool },
    /// `(void)0` placeholder for `Operand::Undef` reads — the lowering
    /// pass renders these as `0 /* undef */` rather than silently
    /// inventing a value (I-6).
    Undef,
    /// `lhs <op> rhs`. The emitter inserts parentheses around both
    /// children so precedence is preserved without the lowering pass
    /// having to reason about it.
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `<op>expr`.
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// `*(ty *)(address)` — typed memory load.
    Load { ty: CType, address: Box<Expr> },
    /// `target(arg0, arg1, …)`. `target` is typically an
    /// [`Expr::Var`] for direct calls and an [`Expr::AddrLit`] for
    /// indirect calls whose target address is known.
    Call { target: Box<Expr>, args: Vec<Expr> },
    /// `((void (*)())0xdeadbeef)` — call-target literal for direct
    /// calls whose name we don't know yet.
    AddrLit(u64),
    /// `base.field` (when `arrow` is false) or `base->field` (when
    /// `arrow` is true). Produced by the B3.10 lowering when a
    /// `Load` address decomposes to a known struct-field offset
    /// (FR-17). The `field` string is the field name as it appears
    /// in the emitted source — the lowering pass synthesises
    /// `field_<offset>` for recovered fields without a recovered
    /// name.
    Field {
        base: Box<Expr>,
        field: String,
        arrow: bool,
    },
    /// `((ty)(expr))` — explicit C cast. Used to bridge the boundary
    /// between recovery-typed parameters and width-typed locals so
    /// the round-trip compile gate stays green when the recovered
    /// parameter type doesn't match the local's width-based type
    /// (B3.10).
    Cast { ty: CType, expr: Box<Expr> },
    /// `(/* opaque: <text> */ 0)` — opaque operation whose semantics
    /// the lowering pass can't faithfully render. The compile pipeline
    /// degrades by emitting a comment-wrapped zero so the C compiler
    /// has something with the right type (NFR-7, I-6).
    Opaque(String),
}

/// One arm of a [`Stmt::Switch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchArm {
    /// Constant matched by this arm. Emitter renders as
    /// `case <value>:`.
    pub value: i64,
    /// Body executed when the scrutinee equals `value`. The lowering
    /// pass terminates each arm with the appropriate `break;` /
    /// `return;` / `goto` to keep fall-through explicit.
    pub body: Block,
}

/// Binary operators. The set mirrors the SSA-layer
/// [`dac_ir::ssa::SsaOp`] arithmetic and comparison ops plus a few
/// shorthand combinations the lowering pass needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// `-expr` — two's-complement negation.
    Neg,
    /// `~expr` — bitwise NOT.
    BitNot,
    /// `!expr` — logical NOT. Used for the `if (!cond) break;` shape
    /// the lowering pass emits inside `Loop`.
    LogicalNot,
}

/// C type spelling.
///
/// The lowering pass produces these from the SSA value's
/// [`dac_ir::ssa::Variable::width_bits`] (defaulting to the integer
/// fallback `int64_t` when no information is available) and from the
/// recovered return / parameter types when [`dac_recovery::TypeMap`]
/// is threaded through (B2.6 in dac-recovery).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CType {
    /// `void`.
    Void,
    /// Fixed-width integer — emits `intNN_t` / `uintNN_t` based on
    /// [`CType::Int { signed }`].
    Int { width_bits: u16, signed: bool },
    /// Pointer to another type — emits `<inner> *`.
    Ptr(Box<CType>),
    /// Reference to a `typedef`'d name (e.g. a [`StructDecl`]'s
    /// identifier). The emitter renders the name verbatim — no
    /// `struct` keyword is added, so the typedef must already be
    /// `typedef struct { … } NAME;`-shaped (B3.16).
    Named(String),
    /// Fixed-extent array — emits `<element>` as the prefix and
    /// `<name>[count]` at the declarator site. Used inside
    /// [`StructField`] for the padding members the B3.16 lowering pass
    /// emits between recovered struct fields.
    Array { element: Box<CType>, count: u64 },
}

impl CType {
    /// Convenience: `int64_t`.
    #[must_use]
    pub const fn i64() -> Self {
        Self::Int {
            width_bits: 64,
            signed: true,
        }
    }

    /// Convenience: `uint8_t`.
    #[must_use]
    pub const fn u8() -> Self {
        Self::Int {
            width_bits: 8,
            signed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_empty_is_default() {
        let a: Block = Block::default();
        let b: Block = Block::empty();
        assert_eq!(a, b);
        assert!(a.is_empty());
    }

    #[test]
    fn ctype_helpers_round_trip() {
        assert_eq!(
            CType::i64(),
            CType::Int {
                width_bits: 64,
                signed: true
            }
        );
        assert_eq!(
            CType::u8(),
            CType::Int {
                width_bits: 8,
                signed: false
            }
        );
    }

    #[test]
    fn stmt_variants_are_exhaustively_matchable() {
        // Compile-time guard: every Stmt variant is reachable. Adding
        // a new variant breaks this test, which is the signal we want
        // to see in downstream consumers (the emitter).
        let s = Stmt::Break;
        match s {
            Stmt::Decl { .. }
            | Stmt::Assign { .. }
            | Stmt::Store { .. }
            | Stmt::FieldStore { .. }
            | Stmt::ExprStmt(_)
            | Stmt::If { .. }
            | Stmt::Loop { .. }
            | Stmt::While { .. }
            | Stmt::DoWhile { .. }
            | Stmt::Break
            | Stmt::Continue
            | Stmt::Return(_)
            | Stmt::Label(_)
            | Stmt::Goto(_)
            | Stmt::Switch { .. }
            | Stmt::Comment(_)
            | Stmt::Unreachable => {}
        }
    }

    #[test]
    fn expr_variants_are_exhaustively_matchable() {
        let e = Expr::Undef;
        match e {
            Expr::Var(_)
            | Expr::IntLit { .. }
            | Expr::Undef
            | Expr::Binary { .. }
            | Expr::Unary { .. }
            | Expr::Load { .. }
            | Expr::Call { .. }
            | Expr::AddrLit(_)
            | Expr::Field { .. }
            | Expr::Cast { .. }
            | Expr::Opaque(_) => {}
        }
    }

    #[test]
    fn item_variants_are_exhaustively_matchable() {
        let i = Item::Function(Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block::empty(),
            leading_comment: None,
        });
        match i {
            Item::Function(_) | Item::StructDecl(_) => {}
        }
    }

    #[test]
    fn ctype_variants_are_exhaustively_matchable() {
        let t = CType::Void;
        match t {
            CType::Void
            | CType::Int { .. }
            | CType::Ptr(_)
            | CType::Named(_)
            | CType::Array { .. } => {}
        }
    }
}
