//! Semantic IR → C AST (B2.8, FR-21; B3.10 recovery-fact threading).
//!
//! [`lower_function`] consumes the structurer's [`SemFunction`] together
//! with the underlying [`SsaFunction`] that holds the actual ops the
//! Semantic IR references via [`SsaRef`], a [`NameResolver`] that maps
//! call-target VAs to C identifiers, and a [`Recovered`] view of the
//! per-function side tables produced by `dac-recovery`. The function
//! produces a [`Function`] in the C AST.
//!
//! ## What the lowering pass commits to
//!
//! 1. **Every non-parameter SSA value becomes a pre-declared C
//!    local.** Naming is `v<id>`. Locals are declared at the top of
//!    the function with zero initialisers. The declared type comes
//!    from the recovered [`TypeMap`] when [`Recovered::types`] is
//!    present and the lattice has a concrete entry (B3.15); the
//!    width-based `int<bits>_t` shape from the SSA variable's
//!    `width_bits` is the B2.8 fallback when no lattice entry exists.
//!    Lattice-typed locals (e.g. `int64_t *`, `int32_t`) are bridged
//!    with explicit casts at every define and every use so the
//!    round-trip `cc` gate stays clean under `-Wint-conversion` even
//!    though the lifter's sub-register arithmetic mixes pointer-typed
//!    and int-typed values:
//!    - **At the define site** (`Stmt::Assign`) the RHS is computed in
//!      the width-typed shape (the SSA op's natural result type) and
//!      cast into the local's declared type with
//!      `((<declared>)(<rhs>))`.
//!    - **At every use site** (operand in a binary/unary op, the
//!      address of a `Load`, the return value, …) the local's value
//!      is cast back from declared to the width-typed shape with
//!      `((<width>)(<local>))` so arithmetic and pointer arithmetic
//!      do not accidentally re-interpret each other (e.g. a pointer
//!      `+ 8LL` byte-add stays a byte add, not a 64-byte-stride
//!      pointer-element add).
//!
//!    When declared == width (no lattice entry, or lattice matches
//!    width) `cast_if_needed` is a no-op so the B2.8 output shape is
//!    preserved byte-for-byte.
//! 2. **The convention's parameter list materialises as named C
//!    parameters** (B3.10): each `RegisterArg` becomes `arg<index>`
//!    whose type comes from [`TypeMap::value_type`]. The pre-declared
//!    local `v<id>` for each parameter is initialised from the matching
//!    `arg<n>` through an explicit [`Expr::Cast`] so the int / pointer
//!    boundary between the typed parameter and the width-typed local
//!    is explicit (FR-13, FR-14).
//! 3. **Return types come from the recovered convention + type map**
//!    (B3.10, FR-14). A convention with `return_register: Some(_)`
//!    joins every `Return { value: Some(v) }` operand's type and
//!    emits the result as the function's return type. Each
//!    `Return { value: Some(_) }` operand is wrapped in an
//!    [`Expr::Cast`] to the declared return type so int / pointer
//!    boundaries stay explicit. A convention without a return
//!    register, or no convention at all, falls back to the B2.8
//!    "returns a value → `int64_t`, otherwise `void`" heuristic.
//! 4. **Phi nodes render as a `/* phi: … */` comment.** The underlying
//!    SSA value is already declared, so the comment is purely
//!    informational.
//! 5. **Side-effectful ops** (stores, calls without `dst`, opaque
//!    instructions) become [`Stmt::Store`] / [`Stmt::ExprStmt`].
//!    Value-producing ops become [`Stmt::Assign`].
//! 6. **Unknown call targets** lower through [`Expr::AddrLit`] so the
//!    cast-and-call shape compiles even when the symbol is not
//!    recoverable.
//! 7. **Recovered pointer-anchored struct layouts surface as real
//!    typedefs** (B3.16, FR-17). The orchestrator's
//!    [`RecoveredStructs::pointer_structs`] is walked at lowering
//!    time; each layout that fits within
//!    [`MAX_PROMOTED_STRUCT_SIZE`] mints a translation-unit-level
//!    `typedef struct { … } NAME;` and the base value is re-typed as
//!    `<typedef> *`. `Load` / `Store` whose address decomposes to
//!    `Add(base, Const(offset))` and matches a field in the layout
//!    rewrite to `base->field_<hex>` directly. Layouts above the size
//!    cap — almost always the absolute-address false-positive shape
//!    from `Add(base, Const(0x140008008))` — fall back to the B3.10
//!    `/* recovered field: … */` comment-on-top-of-bare-Store surface
//!    so the round-trip compile gate is not blown up by a multi-MB
//!    typedef.
//!
//! ## What lands later
//!
//! - Real SSA destruction with phi-edge copies (post-B3.3, paired
//!   with the loop-shape recogniser).
//! - Stack-anchored struct surface in the C AST. B3.10 only lowers
//!   pointer-anchored structs; stack-anchored structs from
//!   `RecoveredStructs::stack_structs` are still rendered as
//!   `*(int*)(rsp + k)` until the AST gains a `Stmt::Decl` shape for
//!   composite locals.
//! - Switch-arm resolution. B3.10's switch lowering rewrites
//!   recognised `Stmt::Unreachable` into `Stmt::Switch` with an empty
//!   arm list and a `default: __builtin_unreachable();` body; per-
//!   entry resolution that reads `.rodata` / relocations lives on the
//!   B3 follow-up shelf.
//!
//! ## Determinism
//!
//! Pure function. Iteration over `ssa.values` is in ascending `ValueId`
//! order. Iteration over `body.stmts` is the structurer's order.
//! Recovery facts are read through the [`Recovered`] view, which
//! itself wraps [`BTreeMap`]-backed tables.

use std::collections::BTreeMap;

use dac_ir::sem::{Block as SemBlock, SemFunction, SsaRef, Stmt as SemStmt};
use dac_ir::ssa::{CompareKind, Operand, Phi, SsaFunction, SsaOp, ValueDef, ValueId, ValueSource};
use dac_ir::ty::{Signedness, Type as IrType};
use dac_recovery::{
    InferredSignature, NameTable, RecoveredStructs, RegisterArg, StructLayout, TypeMap,
};

use crate::ast::{
    BinaryOp, Block as CBlock, CType, Expr, Function, Item, Local, Param, Stmt as CStmt,
    StructDecl, StructField, TranslationUnit, UnaryOp,
};

/// The set of `#include` directives every emitted translation unit
/// needs. Exposed so callers (e.g. the CLI driver) can prepend their
/// own without re-deriving the canonical list.
#[must_use]
pub fn default_includes() -> Vec<String> {
    vec![
        "#include <stdint.h>".to_string(),
        "#include <stddef.h>".to_string(),
    ]
}

/// Map from call-target virtual address to the C function name to emit
/// at the call site. The CLI threads the recovered
/// [`dac_recovery::FunctionSet`] through this; tests can pass an empty
/// map and accept the `AddrLit` fallback.
pub type NameResolver = BTreeMap<u64, String>;

/// View onto the per-function recovery side tables produced by
/// `dac-recovery`. The lowering pass reads through this rather than
/// owning the tables so callers can pass any subset of facts they
/// have recovered. `Default` yields the empty view — equivalent to the
/// B2.8 behaviour: no typed signatures, no struct fields, no typed
/// locals.
#[derive(Debug, Clone, Copy, Default)]
pub struct Recovered<'a> {
    /// Convention-inferred parameter / return register layout
    /// (B2.5). Used by [`lower_function`] to build the C parameter
    /// list and to pick the return type.
    pub signature: Option<&'a InferredSignature>,
    /// Per-SSA-value type lattice (B2.6). Used to refine the type of
    /// every pre-declared local and every parameter.
    pub types: Option<&'a TypeMap>,
    /// Pointer / stack struct layouts (B3.2). Used to rewrite
    /// `Load` / `Store` accesses whose address decomposes to
    /// `Add(base, Const(offset))` into `base->field` / `base.field`.
    pub structs: Option<&'a RecoveredStructs>,
    /// Heuristic name candidates (B3.7). Each SSA value present in
    /// the table emits with its heuristic identifier (`path`,
    /// `fmt`, `str_hello`, …) instead of the generic `v<id>`
    /// fallback. Values absent from the table keep `v<id>`.
    pub names: Option<&'a NameTable>,
}

impl<'a> Recovered<'a> {
    /// Convenience builder.
    #[must_use]
    pub fn new(
        signature: Option<&'a InferredSignature>,
        types: Option<&'a TypeMap>,
        structs: Option<&'a RecoveredStructs>,
        names: Option<&'a NameTable>,
    ) -> Self {
        Self {
            signature,
            types,
            structs,
            names,
        }
    }
}

/// Lower one Semantic IR function to a C AST function.
///
/// The pass walks `sem.body` recursively, looking up referenced
/// SSA ops in `ssa.blocks`. Both arguments must describe the same
/// source function — the structurer's `ssa.function_address` must
/// equal `sem.function_address`. The [`Recovered`] view threads the
/// per-function recovery facts (convention, types, structs) the
/// orchestrator computed (B3.10).
///
/// Returns only the lowered [`Function`]. Callers that also want the
/// translation-unit-level struct typedefs synthesised from
/// [`RecoveredStructs::pointer_structs`] (B3.16) should call
/// [`lower_function_with_typedefs`] instead.
#[must_use]
pub fn lower_function(
    ssa: &SsaFunction,
    sem: &SemFunction,
    resolver: &NameResolver,
    recovered: &Recovered<'_>,
) -> Function {
    lower_function_with_typedefs(ssa, sem, resolver, recovered).function
}

/// Output of [`lower_function_with_typedefs`]: the lowered C function
/// plus the translation-unit-level struct typedefs the lowering pass
/// synthesised for the recovered pointer-anchored layouts referenced
/// by this function (B3.16, FR-17).
///
/// `struct_decls` is sorted by typedef name so multi-function
/// translation units can dedup deterministically before prepending
/// them to the [`TranslationUnit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub function: Function,
    pub struct_decls: Vec<StructDecl>,
}

/// Per-lowering knobs the C backend honours (B3.27, FR-21, FR-25).
///
/// Default-constructible so existing call sites and tests stay green
/// without ceremony; callers that want the debug-only surface flip
/// the flag at the call site.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LowerOptions {
    /// When true, the C backend keeps the
    /// `/* structurally unreachable: block N */` + `__builtin_unreachable();`
    /// pair the structurer's [`SemStmt::Unreachable`] markers lower to.
    /// When false (the `--debug`-off default), the pair collapses to a
    /// single `/* dac: structuring fallback */` line — the reader still
    /// sees that the structurer hit a fallback, without the
    /// per-source-block citation that only matters when debugging the
    /// pipeline (B3.27).
    pub debug: bool,
}

/// Lower a function and also return the struct typedefs the lowering
/// pass needs at translation-unit scope. The typedef shape mirrors the
/// recovered [`StructLayout`]: leading padding bringing the first
/// field to its observed offset, then one field per observed offset
/// (B3.16, FR-17). Layouts whose `total_size` exceeds
/// [`MAX_PROMOTED_STRUCT_SIZE`] are skipped — they are almost always
/// the recovery clustering an absolute-address access (e.g. a PE
/// `[rip + 0x140008008]` rendered as `Add(base, Const(0x140008008))`)
/// rather than a real per-field offset, and emitting a multi-MB
/// typedef would silently break the round-trip compile gate. Those
/// bases stay declared with their B3.15 width-typed / lattice-driven
/// shape, and the B3.10 `recovered field` comment trail still surfaces
/// the access pattern in the body.
#[must_use]
pub fn lower_function_with_typedefs(
    ssa: &SsaFunction,
    sem: &SemFunction,
    resolver: &NameResolver,
    recovered: &Recovered<'_>,
) -> LoweredFunction {
    lower_function_with_options(ssa, sem, resolver, recovered, LowerOptions::default())
}

/// Like [`lower_function_with_typedefs`] but consumes a
/// [`LowerOptions`] knob bag. Added at B3.27 so the CLI can flip the
/// structuring-fallback surface between the debug-citation form and
/// the default collapsed form without breaking existing callers.
#[must_use]
pub fn lower_function_with_options(
    ssa: &SsaFunction,
    sem: &SemFunction,
    resolver: &NameResolver,
    recovered: &Recovered<'_>,
    options: LowerOptions,
) -> LoweredFunction {
    debug_assert_eq!(ssa.function_address, sem.function_address);
    let name = sem
        .function_name
        .clone()
        .unwrap_or_else(|| format!("fn_{:x}", sem.function_address));
    let struct_typedefs = build_struct_typedef_table(sem.function_address, recovered);
    let params = build_params(recovered, &struct_typedefs);
    let parameter_inits = parameter_initialisers(recovered, &struct_typedefs);
    let locals = lower_locals(ssa, recovered, &parameter_inits, &struct_typedefs);
    let return_type = pick_return_type(&sem.body, ssa, recovered);
    let value_types = build_value_types(ssa, recovered, &struct_typedefs);
    let ctx = LowerCtx {
        ssa,
        resolver,
        recovered,
        return_type: return_type.clone(),
        value_types,
        struct_typedefs: &struct_typedefs,
        options,
    };
    let body = ctx.lower_block(&sem.body);
    let leading_comment = Some(format!(
        "dac-recovered function\n\
         address: {:#x}\n\
         source_blocks: {}\n\
         goto_count: {}\n\
         label_count: {}\n\
         irreducible: {}",
        sem.function_address,
        sem.stats.source_blocks,
        sem.stats.goto_count,
        sem.stats.label_count,
        sem.stats.irreducible
    ));
    let mut struct_decls: Vec<StructDecl> = struct_typedefs
        .into_values()
        .map(|typedef| typedef.decl)
        .collect();
    struct_decls.sort_by(|a, b| a.name.cmp(&b.name));
    LoweredFunction {
        function: Function {
            name,
            return_type,
            params,
            locals,
            body,
            leading_comment,
        },
        struct_decls,
    }
}

/// Upper bound on a recovered struct's effective on-wire size (the
/// highest byte the layout touches, **including leading padding**)
/// before the typedef synthesis declines to emit it. Recovery
/// occasionally clusters absolute-address shapes such as
/// `Add(base, Const(0x140008008))` against a shared base — those land
/// with a tiny `last.offset - first.offset` span but a huge
/// `last.offset + width` upper bound. Emitting that as a typedef
/// would silently break the round-trip compile gate without adding
/// signal, so we gate on the upper bound instead of the span. 4 KiB
/// is generous for the real recovered layouts in the corpus (typical
/// structs are well under 1 KiB) and firmly excludes the
/// absolute-address false positives.
pub const MAX_PROMOTED_STRUCT_SIZE: u64 = 4 * 1024;

fn struct_effective_size(layout: &StructLayout) -> u64 {
    let Some(last) = layout.fields.last() else {
        return 0;
    };
    last.offset.saturating_add(u64::from(last.width.max(1)))
}

/// Map from parameter `ValueId` to the [`ParameterInit`] describing
/// the matching [`Param`] (its name and type).
type ParameterInits = BTreeMap<ValueId, ParameterInit>;

/// Per-function table of recovered pointer-anchored structs that the
/// B3.16 lowering promoted to a translation-unit-level typedef. Keyed
/// by the base [`ValueId`] the recovery observed.
type StructTypedefTable = BTreeMap<ValueId, StructTypedef>;

/// One entry in [`StructTypedefTable`]: the synthesised typedef and the
/// recovered layout it materialises. Keeping both around lets the
/// per-access matcher resolve a field by `offset` against the original
/// layout without re-walking the typedef's padded field list.
#[derive(Debug, Clone)]
struct StructTypedef {
    decl: StructDecl,
    layout: StructLayout,
}

/// Build the C parameter list from the convention's `int_args` slot.
/// The N-th argument register becomes `argN`; its type comes from
/// the [`TypeMap`] when available, falling back to the SSA variable's
/// width.
fn build_params(recovered: &Recovered<'_>, structs: &StructTypedefTable) -> Vec<Param> {
    let Some(sig) = recovered.signature else {
        return Vec::new();
    };
    sig.int_args
        .iter()
        .map(|arg| Param {
            name: parameter_name(arg),
            ty: parameter_type(arg, recovered.types, structs),
        })
        .collect()
}

fn parameter_initialisers(
    recovered: &Recovered<'_>,
    structs: &StructTypedefTable,
) -> ParameterInits {
    let mut m = ParameterInits::new();
    let Some(sig) = recovered.signature else {
        return m;
    };
    for arg in &sig.int_args {
        m.insert(
            arg.value,
            ParameterInit {
                arg_name: parameter_name(arg),
                arg_type: parameter_type(arg, recovered.types, structs),
            },
        );
    }
    m
}

/// Per-parameter init descriptor — the name and type of the C
/// [`Param`] so the local-init renderer can build an explicit
/// [`Expr::Cast`] from the parameter to the local's (width-typed)
/// shape.
#[derive(Debug, Clone)]
struct ParameterInit {
    arg_name: String,
    arg_type: CType,
}

fn parameter_name(arg: &RegisterArg) -> String {
    format!("arg{}", arg.index)
}

fn parameter_type(
    arg: &RegisterArg,
    types: Option<&TypeMap>,
    structs: &StructTypedefTable,
) -> CType {
    if let Some(typedef) = structs.get(&arg.value) {
        return CType::Ptr(Box::new(CType::Named(typedef.decl.name.clone())));
    }
    let inferred = types
        .map(|t| t.value_type(arg.value))
        .unwrap_or(IrType::Unknown);
    map_ir_type(&inferred).unwrap_or_else(CType::i64)
}

/// Pick the function's C return type from the recovered convention
/// and per-value types. When the convention has no return register
/// (or no convention was recovered) the B2.8 "any
/// `Stmt::Return { value: Some(_) }` → `int64_t`" heuristic stays in
/// force.
fn pick_return_type(body: &SemBlock, ssa: &SsaFunction, recovered: &Recovered<'_>) -> CType {
    let conv_has_return = recovered
        .signature
        .and_then(|s| s.return_register)
        .is_some();
    if !conv_has_return {
        return if returns_value(body) {
            CType::i64()
        } else {
            CType::Void
        };
    }
    // Convention says the function returns through a register. Take
    // the lattice-join of every `Return { value: Some(v) }` operand's
    // recovered type; degrade to `int64_t` when nothing was inferred.
    let mut joined = IrType::Unknown;
    collect_returned_value_types(body, ssa, recovered, &mut joined);
    map_ir_type(&joined).unwrap_or_else(CType::i64)
}

fn collect_returned_value_types(
    block: &SemBlock,
    ssa: &SsaFunction,
    recovered: &Recovered<'_>,
    out: &mut IrType,
) {
    for stmt in &block.stmts {
        match stmt {
            SemStmt::Return {
                value: Some(Operand::Value(v)),
                ..
            } => {
                if let Some(types) = recovered.types {
                    let ty = types.value_type(*v);
                    *out = out.join(&ty);
                } else {
                    // Width fallback from the SSA variable so a "returns
                    // a value but the type pass didn't run" case still
                    // produces a sensible type.
                    let def = ssa.value(*v);
                    let var = ssa.variable(def.variable);
                    *out = out.join(&IrType::int_of_width(var.width_bits.max(64)));
                }
            }
            SemStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_returned_value_types(then_body, ssa, recovered, out);
                if let Some(eb) = else_body {
                    collect_returned_value_types(eb, ssa, recovered, out);
                }
            }
            SemStmt::While { body, .. }
            | SemStmt::DoWhile { body, .. }
            | SemStmt::Loop { body, .. } => {
                collect_returned_value_types(body, ssa, recovered, out);
            }
            SemStmt::Switch { arms, default, .. } => {
                for arm in arms {
                    collect_returned_value_types(&arm.body, ssa, recovered, out);
                }
                if let Some(d) = default {
                    collect_returned_value_types(d, ssa, recovered, out);
                }
            }
            _ => {}
        }
    }
}

fn returns_value(block: &SemBlock) -> bool {
    block.stmts.iter().any(stmt_returns_value)
}

fn stmt_returns_value(stmt: &SemStmt) -> bool {
    match stmt {
        SemStmt::Return { value: Some(_), .. } => true,
        SemStmt::If {
            then_body,
            else_body,
            ..
        } => returns_value(then_body) || else_body.as_ref().is_some_and(returns_value),
        SemStmt::While { body, .. }
        | SemStmt::DoWhile { body, .. }
        | SemStmt::Loop { body, .. } => returns_value(body),
        SemStmt::Switch { arms, default, .. } => {
            arms.iter().any(|a| returns_value(&a.body))
                || default.as_ref().is_some_and(returns_value)
        }
        _ => false,
    }
}

/// Lower a sequence of Semantic IR functions into a single
/// [`TranslationUnit`] suitable for emission. `recovered` is indexed
/// alongside `ssa_funcs` / `sem_funcs`; pass an empty `&[]` to fall
/// back to the B2.8 behaviour (no recovery facts).
///
/// Translation-unit-level struct typedefs synthesised by
/// [`lower_function_with_typedefs`] (B3.16) are deduplicated by name
/// and prepended to the item list so each typedef is in scope at
/// every function that references it.
#[must_use]
pub fn lower_unit(
    ssa_funcs: &[SsaFunction],
    sem_funcs: &[SemFunction],
    resolver: &NameResolver,
    recovered: &[Recovered<'_>],
) -> TranslationUnit {
    debug_assert_eq!(ssa_funcs.len(), sem_funcs.len());
    let default = Recovered::default();
    let mut typedefs: BTreeMap<String, StructDecl> = BTreeMap::new();
    let mut function_items: Vec<Item> = Vec::with_capacity(ssa_funcs.len());
    for (i, (s, sem)) in ssa_funcs.iter().zip(sem_funcs.iter()).enumerate() {
        let r = recovered.get(i).copied().unwrap_or(default);
        let lowered = lower_function_with_typedefs(s, sem, resolver, &r);
        for decl in lowered.struct_decls {
            typedefs.entry(decl.name.clone()).or_insert(decl);
        }
        function_items.push(Item::Function(lowered.function));
    }
    let mut items: Vec<Item> = typedefs.into_values().map(Item::StructDecl).collect();
    items.extend(function_items);
    TranslationUnit {
        includes: default_includes(),
        items,
    }
}

/// Threading context passed through the recursive lowering walk so
/// every callee can consult the same SSA / resolver / recovery view
/// without re-deriving them.
struct LowerCtx<'a> {
    ssa: &'a SsaFunction,
    resolver: &'a NameResolver,
    recovered: &'a Recovered<'a>,
    /// Function's C return type. Used to cast `Return { value: Some(_) }`
    /// operands at every return site so the int / pointer boundary
    /// between the integer-typed locals and a pointer-typed return is
    /// explicit (B3.10).
    return_type: CType,
    /// Per-value `(declared, width)` C-type pair (B3.15). `declared`
    /// is the C type the pre-declared local was emitted with;
    /// `width` is the SSA variable's width-based shape. Both are
    /// indexed by [`ValueId`]; values absent from the SSA function
    /// fall through to `int64_t` (the B2.8 default).
    value_types: Vec<ValueTypePair>,
    /// Pointer-anchored struct typedefs the lowering pass promoted for
    /// this function (B3.16). Keyed by the base [`ValueId`]; used at
    /// every `Load` / `Store` address site to rewrite the access into
    /// `base->field_<hex>` form.
    struct_typedefs: &'a StructTypedefTable,
    /// Per-lowering knobs threaded from the CLI (B3.27). Currently
    /// the `debug` bit gates the structuring-fallback citation form;
    /// the struct is here so later knobs land without re-threading
    /// every call site.
    options: LowerOptions,
}

/// The `(declared, width)` pair the lowering pass tracks per SSA value.
///
/// `declared` is the C type the pre-declared local was rendered with —
/// either the recovered lattice's mapping (B3.15) or the width-based
/// fallback. `width` is always the width-based shape; we keep both so
/// the per-define / per-use cast pair stays consistent even when the
/// lattice promotes the local to a pointer or to a non-default integer
/// width.
#[derive(Debug, Clone)]
struct ValueTypePair {
    declared: CType,
    width: CType,
}

fn lower_locals(
    ssa: &SsaFunction,
    recovered: &Recovered<'_>,
    parameter_inits: &ParameterInits,
    struct_typedefs: &StructTypedefTable,
) -> Vec<Local> {
    let mut locals = Vec::with_capacity(ssa.values.len());
    for def in &ssa.values {
        // B3.26: skip values orphaned by the pre-emit simplifier
        // (and by the earlier B2.3 local CSE pass). Parameters always
        // emit so their `arg<n> → v<id>` init survives. A non-parameter
        // value with no defining instruction or phi never appears in
        // any kept operand — emitting it would leave a stale
        // `int64_t vN = 0LL;` declaration that the user perceives as
        // dead-store noise.
        let is_param = parameter_inits.contains_key(&def.id);
        if !is_param && !dac_recovery::value_has_definition(ssa, def.id) {
            continue;
        }
        let declared = declared_ctype(ssa, def, recovered, struct_typedefs);
        let zero_init = Expr::IntLit {
            value: 0,
            signed: true,
        };
        let init = match parameter_inits.get(&def.id) {
            // Parameter init keeps the B3.10 cast surface — any
            // change in declared shape (narrow int from a hint, ptr
            // from the lattice) is bridged explicitly so the type
            // shift is visible at the call boundary.
            Some(p) => Some(cast_if_needed(
                &p.arg_type,
                &declared,
                Expr::Var(p.arg_name.clone()),
            )),
            // Zero init: only cast when the declared type genuinely
            // disagrees with the `0LL` literal at the int↔ptr
            // boundary (B3.15). `int8_t = 0LL` etc. stays implicit.
            None => Some(ptr_cast_if_needed(&CType::i64(), &declared, zero_init)),
        };
        locals.push(Local {
            name: value_name(def.id, recovered.names),
            ty: declared,
            init,
        });
    }
    locals
}

/// The width-based C type for `def` — the B2.8 fallback shape. Used
/// at every SSA-op result site (arithmetic, loads, calls) as the type
/// the op naturally produces before any lattice-driven cast.
fn width_ctype(ssa: &SsaFunction, def: &ValueDef) -> CType {
    let var = ssa.variable(def.variable);
    match var.width_bits {
        0 => CType::i64(),
        w => CType::Int {
            width_bits: w,
            signed: true,
        },
    }
}

/// The declared C type for `def` — the type the pre-declared local
/// will be emitted with. Prefers the recovered lattice's mapping
/// (B3.15); falls back to the width-based shape when the lattice has
/// no concrete entry, returns [`None`] for that value, or yields a
/// composite the C AST cannot represent yet. When `def` is the base
/// of a recovered pointer-anchored struct that the lowering pass
/// promoted to a translation-unit-level typedef (B3.16), the declared
/// type becomes `<typedef> *` — overriding any lattice mapping —
/// so the body's `base->field_<hex>` accesses compile.
fn declared_ctype(
    ssa: &SsaFunction,
    def: &ValueDef,
    recovered: &Recovered<'_>,
    structs: &StructTypedefTable,
) -> CType {
    if let Some(typedef) = structs.get(&def.id) {
        return CType::Ptr(Box::new(CType::Named(typedef.decl.name.clone())));
    }
    let width = width_ctype(ssa, def);
    let Some(types) = recovered.types else {
        return width;
    };
    let ty = types.value_type(def.id);
    map_ir_type(&ty).unwrap_or(width)
}

fn build_value_types(
    ssa: &SsaFunction,
    recovered: &Recovered<'_>,
    structs: &StructTypedefTable,
) -> Vec<ValueTypePair> {
    ssa.values
        .iter()
        .map(|def| ValueTypePair {
            declared: declared_ctype(ssa, def, recovered, structs),
            width: width_ctype(ssa, def),
        })
        .collect()
}

/// Build the per-function pointer-anchored struct typedef table. One
/// entry per [`RecoveredStructs::pointer_structs`] layout whose
/// `total_size` is within [`MAX_PROMOTED_STRUCT_SIZE`]; layouts above
/// the cap are dropped so the B3.10 comment-only surface still covers
/// them without breaking the round-trip compile gate.
fn build_struct_typedef_table(
    function_address: u64,
    recovered: &Recovered<'_>,
) -> StructTypedefTable {
    let mut out = StructTypedefTable::new();
    let Some(structs) = recovered.structs else {
        return out;
    };
    for (base, layout) in &structs.pointer_structs {
        if struct_effective_size(layout) > MAX_PROMOTED_STRUCT_SIZE {
            continue;
        }
        let name = struct_typedef_name(function_address, *base);
        let fields = build_struct_fields(layout);
        let decl = StructDecl {
            name: name.clone(),
            fields,
            leading_comment: Some(format!(
                "dac-recovered struct\n\
                 base: v{base}\n\
                 total_size: {} bytes\n\
                 confidence: {:.2} ({:?})",
                layout.total_size,
                layout.confidence.value(),
                layout.confidence.source()
            )),
        };
        out.insert(
            *base,
            StructTypedef {
                decl,
                layout: layout.clone(),
            },
        );
    }
    out
}

/// Synthesise a stable typedef name for a recovered pointer-anchored
/// struct. The format `S_<funcaddr_hex>_v<base_id>_t` is per-function
/// and per-base — two functions that share the same SSA `ValueId` for
/// different bases land at different typedef names, and rerunning the
/// pass on the same function produces the same name.
fn struct_typedef_name(function_address: u64, base: ValueId) -> String {
    format!("S_{function_address:x}_v{base}_t")
}

/// Lay the [`StructLayout`] out as a packed C struct's field list. A
/// gap of `N` bytes between two adjacent fields (or between offset 0
/// and the first field) becomes a `uint8_t __pad_<from>_<to>[N];`
/// member so the field at offset `0xN` is rendered at byte `0xN`
/// inside the typedef and a recovered `field_<hex>` reference resolves
/// to the same byte position the recovery pass observed.
fn build_struct_fields(layout: &StructLayout) -> Vec<StructField> {
    let mut out = Vec::new();
    let mut cursor: u64 = 0;
    for f in &layout.fields {
        if f.offset > cursor {
            let gap = f.offset - cursor;
            out.push(StructField {
                name: format!("__pad_{:x}_{:x}", cursor, f.offset),
                ty: CType::Array {
                    element: Box::new(CType::u8()),
                    count: gap,
                },
            });
        }
        out.push(StructField {
            name: format!("field_{:x}", f.offset),
            ty: int_type_for_width(f.width),
        });
        cursor = f.offset + u64::from(f.width.max(1));
    }
    out
}

/// Build `((target)(expr))` only when `source` differs from `target`.
/// Used at parameter→local and return-value boundaries so the round-
/// trip compile gate accepts the assignment without
/// `-Wint-conversion` complaints (B3.10).
fn cast_if_needed(source: &CType, target: &CType, expr: Expr) -> Expr {
    if source == target {
        expr
    } else {
        Expr::Cast {
            ty: target.clone(),
            expr: Box::new(expr),
        }
    }
}

/// Build `((target)(expr))` only at the *int ↔ pointer* boundary
/// between `source` and `target`. Int → int and ptr → ptr-same-pointee
/// conversions are skipped because they are implicit in C and would
/// only add visual noise to the recovered output. Used by the B3.15
/// lattice-typed local code paths to avoid emitting redundant
/// `(int8_t)(0LL)` shapes for width-typed locals while still threading
/// `(void *)(...)` casts through pointer-typed locals.
fn ptr_cast_if_needed(source: &CType, target: &CType, expr: Expr) -> Expr {
    if source == target {
        return expr;
    }
    let is_ptr = |t: &CType| matches!(t, CType::Ptr(_));
    if is_ptr(source) == is_ptr(target) {
        // Same kind (int↔int or ptr↔ptr-same-shape). The C compiler
        // handles this implicitly; keep the surface readable.
        expr
    } else {
        Expr::Cast {
            ty: target.clone(),
            expr: Box::new(expr),
        }
    }
}

impl<'a> LowerCtx<'a> {
    /// Declared (= local-decl) C type for `v`. Falls back to
    /// [`CType::i64`] when the value is out of range (defensive — the
    /// SSA constructor produces a contiguous `ValueId` index space).
    fn declared(&self, v: ValueId) -> CType {
        self.value_types
            .get(v as usize)
            .map(|p| p.declared.clone())
            .unwrap_or_else(CType::i64)
    }

    /// Width-based C type for `v` — what the SSA op's natural result
    /// shape is before any lattice-driven cast.
    fn width(&self, v: ValueId) -> CType {
        self.value_types
            .get(v as usize)
            .map(|p| p.width.clone())
            .unwrap_or_else(CType::i64)
    }

    /// Lower an operand for use *as a value of the width-typed shape*.
    /// `Operand::Value(v)` whose declared type differs from its
    /// width-typed shape is wrapped in `((<width>)v)` so arithmetic
    /// and pointer arithmetic stay consistent (B3.15).
    fn lower_operand_for_use(&self, op: &Operand) -> Expr {
        let raw = lower_operand(op, self.recovered.names);
        match op {
            Operand::Value(v) => {
                let declared = self.declared(*v);
                let width = self.width(*v);
                ptr_cast_if_needed(&declared, &width, raw)
            }
            Operand::Const(_) | Operand::Undef => raw,
        }
    }

    /// Lower an operand and report its declared C type. Used at sites
    /// that compose against a different target shape (return values,
    /// for instance). For non-value operands (constants, undef) the
    /// declared type is the width-typed shape — `int64_t` — so the
    /// resulting cast pair is identity when the target is also
    /// `int64_t`.
    fn lower_operand_with_type(&self, op: &Operand) -> (Expr, CType) {
        let raw = lower_operand(op, self.recovered.names);
        let ty = match op {
            Operand::Value(v) => self.declared(*v),
            Operand::Const(_) | Operand::Undef => CType::i64(),
        };
        (raw, ty)
    }

    fn lower_block(&self, block: &SemBlock) -> CBlock {
        let mut stmts = Vec::with_capacity(block.stmts.len());
        for stmt in &block.stmts {
            self.lower_stmt(stmt, &mut stmts);
        }
        CBlock { stmts }
    }

    fn lower_stmt(&self, stmt: &SemStmt, out: &mut Vec<CStmt>) {
        match stmt {
            SemStmt::Phi { r, .. } => {
                let phi = phi_at(self.ssa, *r);
                out.push(CStmt::Comment(format_phi_comment(
                    phi,
                    self.recovered.names,
                )));
            }
            SemStmt::Instr { r, .. } => {
                self.lower_instr(*r, out);
            }
            SemStmt::If {
                cond,
                then_body,
                else_body,
                ..
            } => {
                out.push(CStmt::If {
                    cond: self.lower_operand_for_use(cond),
                    then_body: self.lower_block(then_body),
                    else_body: else_body.as_ref().map(|b| self.lower_block(b)),
                });
            }
            SemStmt::While { cond, body, .. } => {
                out.push(CStmt::While {
                    cond: self.lower_operand_for_use(cond),
                    body: self.lower_block(body),
                });
            }
            SemStmt::DoWhile { cond, body, .. } => {
                out.push(CStmt::DoWhile {
                    body: self.lower_block(body),
                    cond: self.lower_operand_for_use(cond),
                });
            }
            SemStmt::Loop { body, .. } => {
                out.push(CStmt::Loop {
                    body: self.lower_block(body),
                });
            }
            SemStmt::Switch {
                scrutinee,
                arms,
                default,
                source_block,
                ..
            } => {
                let arms_c: Vec<_> = arms
                    .iter()
                    .map(|a| crate::ast::SwitchArm {
                        value: a.value,
                        body: self.lower_block(&a.body),
                    })
                    .collect();
                let default_c = default.as_ref().map(|d| self.lower_block(d));
                // B3.17: when arms is non-empty, entry resolution
                // landed and the case-to-goto shape carries the
                // table; only flag "arm resolution pending" when the
                // table was recognised but no entries could be
                // resolved (B3.10 surface). The comment is the
                // reader's signal that the idiom is recognised
                // regardless (I-6).
                let comment = if arms.is_empty() {
                    format!(
                        "recovered switch table at block {source_block} (arm resolution pending)"
                    )
                } else {
                    format!("recovered switch table at block {source_block}")
                };
                out.push(CStmt::Comment(comment));
                out.push(CStmt::Switch {
                    scrutinee: self.lower_operand_for_use(scrutinee),
                    arms: arms_c,
                    default: default_c,
                });
            }
            SemStmt::Break { .. } => out.push(CStmt::Break),
            SemStmt::Continue { .. } => out.push(CStmt::Continue),
            SemStmt::Return { value, .. } => {
                let v = value.as_ref().map(|op| {
                    let (raw, source_ty) = self.lower_operand_with_type(op);
                    cast_if_needed(&source_ty, &self.return_type, raw)
                });
                out.push(CStmt::Return(v));
            }
            SemStmt::Label { id, .. } => out.push(CStmt::Label(*id)),
            SemStmt::Goto { target, .. } => out.push(CStmt::Goto(*target)),
            SemStmt::Unreachable { source_block, .. } => {
                // B3.27: gate the per-source-block citation +
                // `__builtin_unreachable();` pair behind `--debug`.
                // The default emit collapses to a single
                // `/* dac: structuring fallback */` line so the
                // reader sees a recognised structuring fallback
                // without the per-block decoder-trace noise. The
                // debug surface keeps the original pair so the
                // pipeline-debugging signal stays available.
                if self.options.debug {
                    out.push(CStmt::Comment(format!(
                        "structurally unreachable: block {source_block}"
                    )));
                    out.push(CStmt::Unreachable);
                } else {
                    out.push(CStmt::Comment("dac: structuring fallback".to_string()));
                }
            }
        }
    }

    fn lower_instr(&self, r: SsaRef, out: &mut Vec<CStmt>) {
        let block = self.ssa.block(r.block);
        let Some(instr) = block.instructions.get(r.index as usize) else {
            out.push(CStmt::Comment(format!(
                "missing SSA instr at block {} index {}",
                r.block, r.index
            )));
            return;
        };
        match (&instr.dst, &instr.op) {
            // Side-effect-only ops first.
            (
                _,
                SsaOp::Store {
                    address,
                    value,
                    width,
                },
            ) => {
                // B3.16: when the address decomposes to a recovered
                // pointer-anchored struct field that was promoted to a
                // translation-unit-level typedef, emit
                // `base->field_<hex> = value;` directly. Otherwise
                // keep the B3.10 comment-only marker on top of the
                // bare `*((<ty> *)(address)) = value;` Store so the
                // reader still sees where the field surface would
                // land.
                if let Some(field) = self.match_promoted_struct_field(address) {
                    out.push(CStmt::FieldStore {
                        base: Expr::Var(field.base_name),
                        field: field.field_name,
                        arrow: true,
                        value: self.lower_operand_for_use(value),
                    });
                    return;
                }
                if let Some(field) = self.match_struct_field(address) {
                    out.push(CStmt::Comment(field_provenance_comment(&field)));
                }
                out.push(CStmt::Store {
                    ty: int_type_for_width(*width),
                    address: self.lower_operand_for_use(address),
                    value: self.lower_operand_for_use(value),
                });
            }
            (None, SsaOp::Call { target, args }) => {
                out.push(CStmt::ExprStmt(self.lower_call(*target, args)));
            }
            (None, SsaOp::Opaque { mnemonic, args }) => {
                out.push(CStmt::ExprStmt(self.lower_opaque(mnemonic, args)));
            }
            (None, _) => {
                // Value-producing op with no destination — should not
                // happen in well-formed SSA, but I-6: degrade with a
                // comment instead of dropping silently.
                out.push(CStmt::Comment(format!(
                    "ssa op produced a value but has no dst at block {} index {}",
                    r.block, r.index
                )));
            }
            (Some(dst), op) => {
                let expr = self.lower_value_op(*dst, op);
                let value = ptr_cast_if_needed(&self.width(*dst), &self.declared(*dst), expr);
                out.push(CStmt::Assign {
                    name: value_name(*dst, self.recovered.names),
                    value,
                });
            }
        }
    }

    fn lower_value_op(&self, dst: ValueId, op: &SsaOp) -> Expr {
        match op {
            SsaOp::Move { src } => {
                // A `Move` is the one op whose RHS isn't naturally
                // width-typed: it preserves the source operand's
                // value. Project the source into the *destination's*
                // width-typed shape — but only when the boundary
                // crosses int↔ptr, so the common Move of one
                // width-typed value into another stays free of
                // identity casts. The outer Assign re-adds the
                // declared cast on the LHS if the lattice promoted
                // the dst (B3.15).
                let raw = lower_operand(src, self.recovered.names);
                let source_ty = match src {
                    Operand::Value(v) => self.declared(*v),
                    Operand::Const(_) | Operand::Undef => CType::i64(),
                };
                ptr_cast_if_needed(&source_ty, &self.width(dst), raw)
            }
            SsaOp::Add { lhs, rhs } => self.binary(BinaryOp::Add, lhs, rhs),
            SsaOp::Sub { lhs, rhs } => self.binary(BinaryOp::Sub, lhs, rhs),
            SsaOp::Mul { lhs, rhs } => self.binary(BinaryOp::Mul, lhs, rhs),
            SsaOp::And { lhs, rhs } => self.binary(BinaryOp::BitAnd, lhs, rhs),
            SsaOp::Or { lhs, rhs } => self.binary(BinaryOp::BitOr, lhs, rhs),
            SsaOp::Xor { lhs, rhs } => self.binary(BinaryOp::BitXor, lhs, rhs),
            SsaOp::Shl { lhs, rhs } => self.binary(BinaryOp::Shl, lhs, rhs),
            SsaOp::Shr { lhs, rhs } => self.binary(BinaryOp::Shr, lhs, rhs),
            SsaOp::Neg { src } => Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(self.lower_operand_for_use(src)),
            },
            SsaOp::Not { src } => Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(self.lower_operand_for_use(src)),
            },
            SsaOp::Compare { kind, lhs, rhs } => Expr::Binary {
                op: compare_to_binary_op(*kind),
                lhs: Box::new(self.lower_operand_for_use(lhs)),
                rhs: Box::new(self.lower_operand_for_use(rhs)),
            },
            SsaOp::Load { address, width } => {
                // B3.16: a load whose address decomposes to a
                // promoted struct field renders as `base->field_<hex>`
                // directly. Otherwise fall back to the B3.10
                // `*((<ty> *)(address))` shape.
                if let Some(field) = self.match_promoted_struct_field(address) {
                    Expr::Field {
                        base: Box::new(Expr::Var(field.base_name)),
                        field: field.field_name,
                        arrow: true,
                    }
                } else {
                    Expr::Load {
                        ty: int_type_for_width(*width),
                        address: Box::new(self.lower_operand_for_use(address)),
                    }
                }
            }
            SsaOp::Call { target, args } => self.lower_call(*target, args),
            SsaOp::Opaque { mnemonic, args } => self.lower_opaque(mnemonic, args),
            SsaOp::Store { .. } => {
                // Stores have no `dst`, handled in the caller. If we get
                // here the IR is inconsistent — render an Opaque so the
                // output compiles.
                Expr::Opaque("store-with-dst".to_string())
            }
        }
    }

    fn binary(&self, op: BinaryOp, lhs: &Operand, rhs: &Operand) -> Expr {
        Expr::Binary {
            op,
            lhs: Box::new(self.lower_operand_for_use(lhs)),
            rhs: Box::new(self.lower_operand_for_use(rhs)),
        }
    }

    fn lower_call(&self, target: Option<u64>, args: &[Operand]) -> Expr {
        let target_expr = match target {
            Some(addr) => match self.resolver.get(&addr) {
                Some(name) => Expr::Var(name.clone()),
                None => Expr::AddrLit(addr),
            },
            None => Expr::Opaque("indirect-call".into()),
        };
        Expr::Call {
            target: Box::new(target_expr),
            args: args.iter().map(|a| self.lower_operand_for_use(a)).collect(),
        }
    }

    fn lower_opaque(&self, mnemonic: &str, args: &[Operand]) -> Expr {
        // Opaque ops carry their args verbatim — by definition the
        // backend cannot represent the operation faithfully, so it
        // does not know what type each arg should land at. Use the
        // raw operand form (no per-use cast) so the rendered text
        // still reads as `mnemonic <name> <name>` and the
        // `format_operand` round-trip stays well-formed.
        let mut body = String::from(mnemonic);
        for arg in args {
            body.push(' ');
            body.push_str(&format_operand(arg, self.recovered.names));
        }
        Expr::Opaque(body)
    }

    /// Recognise a `Load` / `Store` address operand as a recovered
    /// pointer-struct field access. The shape we match is
    /// `Add(base_value, Const(offset))` (or `Const + base_value`) where
    /// `base_value` is keyed in [`RecoveredStructs::pointer_structs`]
    /// and the layout has a field at `offset`. Returns the rendered
    /// base expression and the field name to use in the C output.
    fn match_struct_field(&self, address: &Operand) -> Option<MatchedField> {
        let structs = self.recovered.structs?;
        let addr_val = match address {
            Operand::Value(v) => *v,
            _ => return None,
        };
        let (base, offset) = decompose_add(self.ssa, addr_val)?;
        let layout = structs.pointer_structs.get(&base)?;
        let field_name = field_name_at(layout, offset)?;
        Some(MatchedField {
            base_name: value_name(base, self.recovered.names),
            offset,
            field_name,
        })
    }

    /// Like [`match_struct_field`] but gated on the layout actually
    /// having been promoted to a translation-unit-level typedef
    /// (B3.16). Layouts larger than [`MAX_PROMOTED_STRUCT_SIZE`] keep
    /// the comment-only surface even when the field-offset match
    /// fires, so the round-trip compile gate does not see a
    /// pseudo-typedef with gigabytes of padding.
    fn match_promoted_struct_field(&self, address: &Operand) -> Option<MatchedField> {
        let addr_val = match address {
            Operand::Value(v) => *v,
            _ => return None,
        };
        let (base, offset) = decompose_add(self.ssa, addr_val)?;
        let typedef = self.struct_typedefs.get(&base)?;
        let field_name = field_name_at(&typedef.layout, offset)?;
        Some(MatchedField {
            base_name: value_name(base, self.recovered.names),
            offset,
            field_name,
        })
    }
}

/// Recognised pointer-anchored struct-field access. Kept as a
/// side-table comment in the lowered body until the C AST gains a
/// translation-unit-level struct typedef surface (a B3 follow-up
/// shelf entry).
struct MatchedField {
    base_name: String,
    offset: u64,
    field_name: String,
}

fn field_provenance_comment(f: &MatchedField) -> String {
    format!(
        "recovered field: base={} offset={:#x} field={}",
        f.base_name, f.offset, f.field_name
    )
}

/// Decompose `addr_val` into `(base, offset)` when it was produced by
/// `Add(base, Const(offset))` or `Add(Const(offset), base)`. Returns
/// `None` for everything else — including `Add` of two variables, or
/// `Sub` (which never appears in field-access shapes the recovery
/// pass surfaces).
fn decompose_add(ssa: &SsaFunction, addr_val: ValueId) -> Option<(ValueId, u64)> {
    let def = ssa.value(addr_val);
    let ValueSource::Instruction { block, index } = def.source else {
        return None;
    };
    let op = &ssa.block(block).instructions.get(index as usize)?.op;
    let SsaOp::Add { lhs, rhs } = op else {
        return None;
    };
    match (lhs, rhs) {
        (Operand::Value(b), Operand::Const(c)) | (Operand::Const(c), Operand::Value(b)) => {
            if *c < 0 {
                None
            } else {
                Some((*b, *c as u64))
            }
        }
        _ => None,
    }
}

fn field_name_at(layout: &StructLayout, offset: u64) -> Option<String> {
    layout
        .fields
        .iter()
        .find(|f| f.offset == offset)
        .map(|_| format!("field_{:x}", offset))
}

/// Map a `dac_ir::Type` to a `CType` whenever the C AST can
/// faithfully represent it. Returns `None` when the result would
/// require a kind of type the C AST does not model yet (composite,
/// arrays, conflicting widths); callers fall back to a width-based
/// integer.
pub fn map_ir_type(ty: &IrType) -> Option<CType> {
    match ty {
        IrType::Unknown | IrType::Top => None,
        IrType::Int(i) => {
            let signed = matches!(i.signedness, Signedness::Signed | Signedness::Unknown);
            Some(CType::Int {
                width_bits: i.width_bits.max(8),
                signed,
            })
        }
        IrType::Ptr(inner) => Some(CType::Ptr(Box::new(
            map_ir_type(inner).unwrap_or(CType::Void),
        ))),
        IrType::Struct(_) | IrType::Array(_) => None,
    }
}

fn lower_operand(op: &Operand, names: Option<&NameTable>) -> Expr {
    match op {
        Operand::Value(v) => Expr::Var(value_name(*v, names)),
        Operand::Const(c) => Expr::IntLit {
            value: *c,
            signed: true,
        },
        Operand::Undef => Expr::Undef,
    }
}

fn format_operand(op: &Operand, names: Option<&NameTable>) -> String {
    match op {
        Operand::Value(v) => value_name(*v, names),
        Operand::Const(c) => format!("{c}"),
        Operand::Undef => "undef".into(),
    }
}

fn compare_to_binary_op(kind: CompareKind) -> BinaryOp {
    match kind {
        CompareKind::Eq => BinaryOp::Eq,
        CompareKind::Ne => BinaryOp::Ne,
        CompareKind::Lt | CompareKind::Ult => BinaryOp::Lt,
        CompareKind::Le | CompareKind::Ule => BinaryOp::Le,
        CompareKind::Gt | CompareKind::Ugt => BinaryOp::Gt,
        CompareKind::Ge | CompareKind::Uge => BinaryOp::Ge,
    }
}

/// Map a [`SsaOp::Load`] / [`SsaOp::Store`] byte width to a C integer
/// type. `width` is in *bytes* per the SSA-layer documentation; the
/// emitter renders the result as `int8_t` / `int16_t` / `int32_t` /
/// `int64_t`. A zero width (defensive — the SSA constructor never
/// produces one) falls back to `int64_t`.
fn int_type_for_width(width: u8) -> CType {
    let bits = match width {
        0 => 64,
        1 => 8,
        2 => 16,
        4 => 32,
        _ => 64,
    };
    CType::Int {
        width_bits: bits as u16,
        signed: true,
    }
}

fn phi_at(ssa: &SsaFunction, r: SsaRef) -> &Phi {
    &ssa.block(r.block).phis[r.index as usize]
}

fn format_phi_comment(phi: &Phi, names: Option<&NameTable>) -> String {
    let mut s = format!("phi {} <-", value_name(phi.dst, names));
    for (pred, op) in &phi.incoming {
        s.push_str(&format!(" (bb{pred}: {})", format_operand(op, names)));
    }
    s
}

/// Render the C identifier for an SSA value.
///
/// When a heuristic [`NameTable`] entry exists for `id` (B3.7) its
/// disambiguated name is returned verbatim; otherwise the call
/// falls back to the `v<id>` shape that B2.8 introduced. The
/// fallback path keeps the C output well-formed even when the
/// recovery pipeline produced no naming table at all (orchestrator
/// stub paths, tests, hand-built fixtures).
#[must_use]
pub(crate) fn value_name(id: ValueId, names: Option<&NameTable>) -> String {
    names
        .and_then(|t| t.lookup(id))
        .map(str::to_string)
        .unwrap_or_else(|| format!("v{id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::sem::{Block as SemBlock, SemFunction, SsaRef, Stmt as SemStmt, StructuringStats};
    use dac_ir::ssa::{
        Operand, Phi, SsaBlock, SsaFunction, SsaInstruction, SsaOp, SsaTerminator, Variable,
    };

    fn ev() -> dac_core::EvidenceId {
        let mut g = EvidenceGraph::new();
        g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Semantic,
            id: 0,
        })
    }

    fn empty_ssa(name: &str) -> SsaFunction {
        SsaFunction {
            function_address: 0x1000,
            function_name: Some(name.to_string()),
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Return { value: None },
            }],
            entry: 0,
            variables: vec![],
            values: vec![],
            evidence: ev(),
        }
    }

    fn empty_sem(name: &str) -> SemFunction {
        SemFunction {
            function_address: 0x1000,
            function_name: Some(name.to_string()),
            body: SemBlock {
                stmts: vec![SemStmt::Return {
                    value: None,
                    evidence: ev(),
                }],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        }
    }

    #[test]
    fn empty_function_lowers_to_void_return() {
        let ssa = empty_ssa("f");
        let sem = empty_sem("f");
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        assert_eq!(f.name, "f");
        assert!(f.params.is_empty());
        assert!(f.locals.is_empty());
        assert_eq!(f.body.stmts, vec![CStmt::Return(None)]);
    }

    #[test]
    fn instruction_lowers_to_assignment() {
        let ssa = SsaFunction {
            function_address: 0,
            function_name: Some("g".into()),
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![SsaInstruction {
                    dst: Some(0),
                    op: SsaOp::Add {
                        lhs: Operand::Const(1),
                        rhs: Operand::Const(2),
                    },
                }],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Value(0)),
                },
            }],
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values: vec![dac_ir::ssa::ValueDef {
                id: 0,
                source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                variable: 0,
            }],
            evidence: ev(),
        };
        let sem = SemFunction {
            function_address: 0,
            function_name: Some("g".into()),
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: Some(Operand::Value(0)),
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        // One pre-declared local for v0.
        assert_eq!(f.locals.len(), 1);
        assert_eq!(f.locals[0].name, "v0");
        // The instruction becomes `v0 = (1 + 2);`.
        assert_eq!(
            f.body.stmts[0],
            CStmt::Assign {
                name: "v0".into(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expr::IntLit {
                        value: 1,
                        signed: true
                    }),
                    rhs: Box::new(Expr::IntLit {
                        value: 2,
                        signed: true
                    }),
                },
            }
        );
        // The return references v0.
        assert_eq!(f.body.stmts[1], CStmt::Return(Some(Expr::Var("v0".into()))));
    }

    #[test]
    fn store_lowers_to_store_stmt() {
        let ssa = SsaFunction {
            function_address: 0,
            function_name: None,
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![SsaInstruction {
                    dst: None,
                    op: SsaOp::Store {
                        address: Operand::Const(0x1000),
                        value: Operand::Const(7),
                        width: 4,
                    },
                }],
                terminator: SsaTerminator::Return { value: None },
            }],
            entry: 0,
            variables: vec![],
            values: vec![],
            evidence: ev(),
        };
        let sem = SemFunction {
            function_address: 0,
            function_name: None,
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: None,
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        let CStmt::Store { ty, .. } = &f.body.stmts[0] else {
            panic!("expected Store, got {:?}", f.body.stmts[0]);
        };
        assert_eq!(
            *ty,
            CType::Int {
                width_bits: 32,
                signed: true
            }
        );
    }

    #[test]
    fn call_uses_resolver_when_available() {
        let ssa = SsaFunction {
            function_address: 0,
            function_name: None,
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![SsaInstruction {
                    dst: None,
                    op: SsaOp::Call {
                        target: Some(0xdead_beef),
                        args: vec![Operand::Const(1)],
                    },
                }],
                terminator: SsaTerminator::Return { value: None },
            }],
            entry: 0,
            variables: vec![],
            values: vec![],
            evidence: ev(),
        };
        let sem = SemFunction {
            function_address: 0,
            function_name: None,
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: None,
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        };
        let mut r = NameResolver::new();
        r.insert(0xdead_beef, "puts".into());
        let f = lower_function(&ssa, &sem, &r, &Recovered::default());
        let CStmt::ExprStmt(Expr::Call { target, .. }) = &f.body.stmts[0] else {
            panic!("expected Call");
        };
        assert_eq!(**target, Expr::Var("puts".into()));
    }

    #[test]
    fn phi_lowers_to_comment() {
        let ssa = SsaFunction {
            function_address: 0,
            function_name: None,
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![Phi {
                    dst: 5,
                    variable: 0,
                    incoming: vec![(1, Operand::Const(0)), (2, Operand::Value(3))],
                }],
                instructions: vec![],
                terminator: SsaTerminator::Return { value: None },
            }],
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values: (0..=5)
                .map(|id| dac_ir::ssa::ValueDef {
                    id,
                    source: dac_ir::ssa::ValueSource::Phi { block: 0, index: 0 },
                    variable: 0,
                })
                .collect(),
            evidence: ev(),
        };
        let sem = SemFunction {
            function_address: 0,
            function_name: None,
            body: SemBlock {
                stmts: vec![
                    SemStmt::Phi {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: None,
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        match &f.body.stmts[0] {
            CStmt::Comment(text) => {
                assert!(text.contains("phi v5"));
                assert!(text.contains("(bb1: 0)"));
                assert!(text.contains("(bb2: v3)"));
            }
            other => panic!("expected Comment, got {other:?}"),
        }
    }

    #[test]
    fn lowering_is_deterministic() {
        let ssa = empty_ssa("d");
        let sem = empty_sem("d");
        let a = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        let b = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        assert_eq!(a, b);
    }

    // -----------------------------------------------------------------
    // B3.27 — Structuring-fallback suppression in non-debug emit.
    // -----------------------------------------------------------------

    /// Build a one-block sem function whose body is a lone
    /// `SemStmt::Unreachable` — the shape the structurer produces for a
    /// function whose source block's terminator decoded as
    /// `Unreachable` / `Indirect` and no further idiom recogniser
    /// claimed it.
    fn sem_with_lone_unreachable(name: &str) -> SemFunction {
        SemFunction {
            function_address: 0x1000,
            function_name: Some(name.to_string()),
            body: SemBlock {
                stmts: vec![SemStmt::Unreachable {
                    source_block: 7,
                    evidence: ev(),
                }],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        }
    }

    #[test]
    fn b3_27_default_emit_collapses_unreachable_to_single_fallback_comment() {
        let ssa = empty_ssa("frame_dummy_like");
        let sem = sem_with_lone_unreachable("frame_dummy_like");
        let lowered = lower_function_with_options(
            &ssa,
            &sem,
            &NameResolver::new(),
            &Recovered::default(),
            LowerOptions { debug: false },
        );
        let body = lowered.function.body.stmts;
        assert_eq!(
            body,
            vec![CStmt::Comment("dac: structuring fallback".into())],
            "default emit must collapse Unreachable to a single fallback comment",
        );
        assert!(
            !body.iter().any(|s| matches!(s, CStmt::Unreachable)),
            "non-debug emit must not surface __builtin_unreachable()",
        );
    }

    #[test]
    fn b3_27_debug_emit_keeps_per_block_citation_and_unreachable_call() {
        let ssa = empty_ssa("frame_dummy_like");
        let sem = sem_with_lone_unreachable("frame_dummy_like");
        let lowered = lower_function_with_options(
            &ssa,
            &sem,
            &NameResolver::new(),
            &Recovered::default(),
            LowerOptions { debug: true },
        );
        let body = lowered.function.body.stmts;
        assert_eq!(
            body,
            vec![
                CStmt::Comment("structurally unreachable: block 7".into()),
                CStmt::Unreachable,
            ],
            "debug emit must keep the per-block citation + __builtin_unreachable pair",
        );
    }

    #[test]
    fn b3_27_default_options_match_lower_function_with_typedefs() {
        // The wrapper `lower_function_with_typedefs` continues to use
        // `LowerOptions::default()` so existing callers and goldens
        // keep the new (suppressed) surface as the default — confirm
        // by emitting via both paths and comparing.
        let ssa = empty_ssa("dual_path");
        let sem = sem_with_lone_unreachable("dual_path");
        let via_typedefs =
            lower_function_with_typedefs(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        let via_options = lower_function_with_options(
            &ssa,
            &sem,
            &NameResolver::new(),
            &Recovered::default(),
            LowerOptions::default(),
        );
        assert_eq!(via_typedefs, via_options);
    }

    // -----------------------------------------------------------------
    // B3.15 — Lattice-typed local refinement.
    //
    // These tests pin the contract that the TypeMap is read at *every*
    // SSA value's declaration (not just at parameter / return type
    // sites the way B3.10 did), and that the define / use cast pair
    // bridges the declared / width-based shapes so the round-trip
    // compile gate stays clean even when the lattice flags a pointer.
    // -----------------------------------------------------------------

    use dac_core::{Confidence, Source};
    use dac_ir::ty::Type as IrType;
    use dac_recovery::{TypeMap, ValueType};

    /// Build a minimal SSA function with two values:
    ///   v0 = (Const(2) + Const(16336LL))   // address-style arithmetic
    ///   v1 = Load { address: v0, width: 8 }
    ///   return v1;
    /// Both values are backed by a single 64-bit variable so width
    /// stays `int64_t`. The caller injects whatever lattice it wants
    /// to test against the values.
    fn ssa_load_address_chain() -> SsaFunction {
        SsaFunction {
            function_address: 0x2000,
            function_name: Some("typed_locals".into()),
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![
                    SsaInstruction {
                        dst: Some(0),
                        op: SsaOp::Add {
                            lhs: Operand::Const(2),
                            rhs: Operand::Const(16336),
                        },
                    },
                    SsaInstruction {
                        dst: Some(1),
                        op: SsaOp::Load {
                            address: Operand::Value(0),
                            width: 8,
                        },
                    },
                ],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Value(1)),
                },
            }],
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values: vec![
                dac_ir::ssa::ValueDef {
                    id: 0,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                    variable: 0,
                },
                dac_ir::ssa::ValueDef {
                    id: 1,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 1 },
                    variable: 0,
                },
            ],
            evidence: ev(),
        }
    }

    fn sem_load_address_chain() -> SemFunction {
        SemFunction {
            function_address: 0x2000,
            function_name: Some("typed_locals".into()),
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 1 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: Some(Operand::Value(1)),
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        }
    }

    fn typemap_pointing_v0_to_ptr() -> TypeMap {
        // v0 is the Load's address — TypeMap::propagate_types seeds it
        // as Ptr(Unknown) via the load address rule. We mirror that
        // here without dragging the propagation pass in.
        let mut t = TypeMap::default();
        t.values.insert(
            0,
            ValueType {
                ty: IrType::ptr_to(IrType::Unknown),
                confidence: Confidence::new(0.80, Source::Derived),
            },
        );
        t.values.insert(
            1,
            ValueType {
                ty: IrType::int_of_width(64),
                confidence: Confidence::new(0.80, Source::Derived),
            },
        );
        t
    }

    #[test]
    fn b3_15_pointer_local_declared_with_pointer_ctype() {
        // Without a TypeMap, the existing B3.10 behaviour: v0 declared
        // as `int64_t` because the SSA variable is 64-bit.
        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let baseline = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        assert_eq!(baseline.locals[0].ty, CType::i64());

        // With a TypeMap that flags v0 as pointer, the local's
        // declared type becomes `int64_t *` while the SSA-variable's
        // width-typed shape stays `int64_t` (the width fallback).
        let types = typemap_pointing_v0_to_ptr();
        let recovered = Recovered {
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        assert_eq!(f.locals[0].ty, CType::Ptr(Box::new(CType::Void)));
        assert_eq!(f.locals[1].ty, CType::i64());
    }

    #[test]
    fn b3_15_pointer_local_zero_init_gets_cast() {
        // The `0LL` initialiser is `int64_t`; a pointer-typed local
        // needs an explicit `(int64_t *)` cast or the round-trip
        // compile gate would reject it.
        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let types = typemap_pointing_v0_to_ptr();
        let recovered = Recovered {
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        match &f.locals[0].init {
            Some(Expr::Cast { ty, expr }) => {
                assert_eq!(*ty, CType::Ptr(Box::new(CType::Void)));
                assert_eq!(
                    **expr,
                    Expr::IntLit {
                        value: 0,
                        signed: true
                    }
                );
            }
            other => panic!("expected Cast(ptr, 0LL), got {other:?}"),
        }
        // Width-typed local: no cast (declared == width).
        assert_eq!(
            f.locals[1].init,
            Some(Expr::IntLit {
                value: 0,
                signed: true
            })
        );
    }

    #[test]
    fn b3_15_pointer_assignment_rhs_gets_cast() {
        // v0 = (2LL + 16336LL); — the Add RHS is `int64_t`-typed but
        // the local is `int64_t *`, so the Assign must cast the RHS.
        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let types = typemap_pointing_v0_to_ptr();
        let recovered = Recovered {
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        let CStmt::Assign { name, value } = &f.body.stmts[0] else {
            panic!("expected Assign, got {:?}", f.body.stmts[0]);
        };
        assert_eq!(name, "v0");
        match value {
            Expr::Cast { ty, expr } => {
                assert_eq!(*ty, CType::Ptr(Box::new(CType::Void)));
                // The wrapped expression is the Add RHS unchanged.
                assert_eq!(
                    **expr,
                    Expr::Binary {
                        op: BinaryOp::Add,
                        lhs: Box::new(Expr::IntLit {
                            value: 2,
                            signed: true
                        }),
                        rhs: Box::new(Expr::IntLit {
                            value: 16336,
                            signed: true
                        }),
                    }
                );
            }
            other => panic!("expected Cast(ptr, add), got {other:?}"),
        }
    }

    #[test]
    fn b3_15_pointer_operand_use_gets_cast_back() {
        // v1 = *((int64_t *)(v0));
        //
        // The Load address is `v0` whose declared type is `int64_t *`,
        // but the Load's address operand is read through the
        // width-typed shape (`int64_t`) so subsequent pointer-style
        // casts in the emitter don't double up. We assert the read
        // wraps v0 in `((int64_t)v0)`.
        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let types = typemap_pointing_v0_to_ptr();
        let recovered = Recovered {
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        let CStmt::Assign { value, .. } = &f.body.stmts[1] else {
            panic!("expected Assign for v1, got {:?}", f.body.stmts[1]);
        };
        let Expr::Load { address, .. } = value else {
            panic!("expected Load, got {value:?}");
        };
        match &**address {
            Expr::Cast { ty, expr } => {
                assert_eq!(*ty, CType::i64());
                assert_eq!(**expr, Expr::Var("v0".into()));
            }
            other => panic!("expected Cast(int64_t, v0), got {other:?}"),
        }
    }

    #[test]
    fn b3_15_no_typemap_keeps_width_based_shape() {
        // The B2.8 / B3.10 path: no TypeMap, locals declared with
        // width-based types, no extra casts injected.
        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        // Local 0 is declared as `int64_t` (the width fallback).
        assert_eq!(f.locals[0].ty, CType::i64());
        // Local 0's init is the plain `0LL` literal (no cast).
        assert_eq!(
            f.locals[0].init,
            Some(Expr::IntLit {
                value: 0,
                signed: true
            })
        );
        // The Add assigns directly with no surrounding cast.
        let CStmt::Assign { value, .. } = &f.body.stmts[0] else {
            panic!("expected Assign");
        };
        assert!(
            !matches!(value, Expr::Cast { .. }),
            "expected width-based Assign with no cast, got {value:?}"
        );
        // The Load address is the bare Var(v0).
        let CStmt::Assign { value, .. } = &f.body.stmts[1] else {
            panic!("expected Assign for v1");
        };
        let Expr::Load { address, .. } = value else {
            panic!("expected Load");
        };
        assert_eq!(**address, Expr::Var("v0".into()));
    }

    #[test]
    fn b3_15_return_pointer_local_matches_pointer_return_type() {
        // Return v1 where v1 is declared as `int64_t *` and the
        // convention says the function returns `int64_t *`. The cast
        // pair at the return site bridges (declared, return_type),
        // which is identity here, so `return v1;` renders without a
        // surrounding cast.
        use dac_recovery::InferredSignature;

        let ssa = ssa_load_address_chain();
        let sem = sem_load_address_chain();
        let mut types = TypeMap::default();
        types.values.insert(
            1,
            ValueType {
                ty: IrType::ptr_to(IrType::Unknown),
                confidence: Confidence::new(0.80, Source::Derived),
            },
        );
        let signature = InferredSignature {
            int_args: vec![],
            stack_args: vec![],
            return_register: Some("rax"),
            variadic_call_sites: 0,
        };
        let recovered = Recovered {
            signature: Some(&signature),
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        assert_eq!(f.return_type, CType::Ptr(Box::new(CType::Void)));
        let CStmt::Return(Some(expr)) = &f.body.stmts[2] else {
            panic!("expected Return with value");
        };
        // v1 declared == return_type → no cast wrapper.
        assert_eq!(*expr, Expr::Var("v1".into()));
    }

    #[test]
    fn b3_15_move_with_pointer_dst_wraps_const_init() {
        // `Move { src: Const(0) }` into a pointer-typed local. The
        // assignment cast must lift the int-typed RHS to the local's
        // declared pointer shape so the round-trip compile gate stays
        // clean.
        let ssa = SsaFunction {
            function_address: 0x4000,
            function_name: Some("move_ptr".into()),
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![SsaInstruction {
                    dst: Some(0),
                    op: SsaOp::Move {
                        src: Operand::Const(0),
                    },
                }],
                terminator: SsaTerminator::Return { value: None },
            }],
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rdi".into(),
                width_bits: 64,
            }],
            values: vec![dac_ir::ssa::ValueDef {
                id: 0,
                source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                variable: 0,
            }],
            evidence: ev(),
        };
        let sem = SemFunction {
            function_address: 0x4000,
            function_name: Some("move_ptr".into()),
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: None,
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        };
        let mut types = TypeMap::default();
        types.values.insert(
            0,
            ValueType {
                ty: IrType::ptr_to(IrType::Unknown),
                confidence: Confidence::new(0.80, Source::Derived),
            },
        );
        let recovered = Recovered {
            types: Some(&types),
            ..Recovered::default()
        };
        let f = lower_function(&ssa, &sem, &NameResolver::new(), &recovered);
        assert_eq!(f.locals[0].ty, CType::Ptr(Box::new(CType::Void)));
        let CStmt::Assign { value, .. } = &f.body.stmts[0] else {
            panic!("expected Assign");
        };
        match value {
            Expr::Cast { ty, expr } => {
                assert_eq!(*ty, CType::Ptr(Box::new(CType::Void)));
                assert_eq!(
                    **expr,
                    Expr::IntLit {
                        value: 0,
                        signed: true
                    }
                );
            }
            other => panic!("expected Cast(ptr, 0LL), got {other:?}"),
        }
    }

    // -----------------------------------------------------------------
    // B3.16 — Pointer-anchored struct typedef surface.
    //
    // Pin the contract that the lowering pass synthesises a
    // translation-unit-level struct typedef per recovered pointer-
    // anchored layout (FR-17), that bases pointing into a promoted
    // layout are declared with the matching `<typedef> *` C type, and
    // that the per-access matcher rewrites the bare `*((<ty> *)…)`
    // form into `base->field_<hex>`. Layouts whose effective upper
    // bound exceeds [`MAX_PROMOTED_STRUCT_SIZE`] (the
    // absolute-address false-positive case) keep the B3.10 comment-
    // only surface so the round-trip compile gate is not broken by a
    // multi-MB typedef.
    // -----------------------------------------------------------------

    use dac_recovery::structs::{
        FieldCandidate, RecoveredStructs, StructLayout, POINTER_BASE_CONFIDENCE,
    };

    fn pointer_struct_at(base: ValueId, fields: Vec<(u64, u8)>) -> RecoveredStructs {
        let conf = Confidence::new(POINTER_BASE_CONFIDENCE, Source::Derived);
        let field_candidates: Vec<FieldCandidate> = fields
            .iter()
            .map(|(off, width)| FieldCandidate {
                offset: *off,
                width: *width,
                ty: IrType::Unknown,
                access_count: 1,
                confidence: conf,
            })
            .collect();
        let last = field_candidates.last().unwrap();
        let first = field_candidates.first().unwrap();
        let total_size = last.offset + u64::from(last.width.max(1)) - first.offset;
        let layout = StructLayout {
            fields: field_candidates,
            total_size,
            confidence: conf,
        };
        let mut out = RecoveredStructs::default();
        out.pointer_structs.insert(base, layout);
        out
    }

    /// SSA: `v0 = arg-shaped Move; addr = Add(v0, Const(8)); v1 = Load(addr, 8); return v1;`
    fn ssa_pointer_field_access() -> SsaFunction {
        SsaFunction {
            function_address: 0x3000,
            function_name: Some("struct_access".into()),
            blocks: vec![SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![
                    SsaInstruction {
                        dst: Some(0),
                        op: SsaOp::Move {
                            src: Operand::Const(0x4000),
                        },
                    },
                    SsaInstruction {
                        dst: Some(1),
                        op: SsaOp::Add {
                            lhs: Operand::Value(0),
                            rhs: Operand::Const(8),
                        },
                    },
                    SsaInstruction {
                        dst: Some(2),
                        op: SsaOp::Load {
                            address: Operand::Value(1),
                            width: 8,
                        },
                    },
                ],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Value(2)),
                },
            }],
            entry: 0,
            variables: vec![Variable {
                id: 0,
                name: "rdi".into(),
                width_bits: 64,
            }],
            values: vec![
                dac_ir::ssa::ValueDef {
                    id: 0,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                    variable: 0,
                },
                dac_ir::ssa::ValueDef {
                    id: 1,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 1 },
                    variable: 0,
                },
                dac_ir::ssa::ValueDef {
                    id: 2,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 2 },
                    variable: 0,
                },
            ],
            evidence: ev(),
        }
    }

    fn sem_pointer_field_access() -> SemFunction {
        SemFunction {
            function_address: 0x3000,
            function_name: Some("struct_access".into()),
            body: SemBlock {
                stmts: vec![
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 0 },
                        evidence: ev(),
                    },
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 1 },
                        evidence: ev(),
                    },
                    SemStmt::Instr {
                        r: SsaRef { block: 0, index: 2 },
                        evidence: ev(),
                    },
                    SemStmt::Return {
                        value: Some(Operand::Value(2)),
                        evidence: ev(),
                    },
                ],
            },
            evidence: ev(),
            stats: StructuringStats::default(),
        }
    }

    #[test]
    fn b3_16_pointer_struct_promotes_to_typedef_and_field_access() {
        let ssa = ssa_pointer_field_access();
        let sem = sem_pointer_field_access();
        let structs = pointer_struct_at(0, vec![(0, 8), (8, 8)]);
        let recovered = Recovered {
            structs: Some(&structs),
            ..Recovered::default()
        };
        let lowered = lower_function_with_typedefs(&ssa, &sem, &NameResolver::new(), &recovered);

        // The lowering pass synthesised a typedef for the recovered
        // layout anchored on v0.
        assert_eq!(lowered.struct_decls.len(), 1);
        let typedef = &lowered.struct_decls[0];
        assert_eq!(typedef.name, "S_3000_v0_t");
        // Two fields, both 8 bytes; no leading padding because
        // first.offset == 0.
        assert_eq!(typedef.fields.len(), 2);
        assert_eq!(typedef.fields[0].name, "field_0");
        assert_eq!(typedef.fields[1].name, "field_8");

        // The base local (`v0`) is declared as `S_3000_v0_t *`.
        let f = lowered.function;
        let expected_ty = CType::Ptr(Box::new(CType::Named("S_3000_v0_t".into())));
        assert_eq!(f.locals[0].ty, expected_ty);

        // The Load at v0+8 lowers to `v0->field_8` rather than
        // `*((int64_t *)(...))`.
        let load_assign = f
            .body
            .stmts
            .iter()
            .find_map(|s| match s {
                CStmt::Assign { name, value } if name == "v2" => Some(value.clone()),
                _ => None,
            })
            .expect("Assign for v2");
        match load_assign {
            Expr::Field { base, field, arrow } => {
                assert_eq!(*base, Expr::Var("v0".into()));
                assert_eq!(field, "field_8");
                assert!(arrow);
            }
            other => panic!("expected Field(v0, field_8, ->), got {other:?}"),
        }
    }

    #[test]
    fn b3_16_oversized_struct_declines_to_typedef() {
        // A recovered layout whose `last.offset + width` exceeds the
        // cap is the absolute-address false-positive shape — keep the
        // B3.10 comment-only surface and emit no typedef so the round-
        // trip compile gate is not blown up.
        let ssa = ssa_pointer_field_access();
        let sem = sem_pointer_field_access();
        // first.offset = MAX_PROMOTED_STRUCT_SIZE + 8 → effective_size
        // is well above the 4 KiB cap.
        let big = MAX_PROMOTED_STRUCT_SIZE + 8;
        let structs = pointer_struct_at(0, vec![(big, 8), (big + 8, 8)]);
        let recovered = Recovered {
            structs: Some(&structs),
            ..Recovered::default()
        };
        let lowered = lower_function_with_typedefs(&ssa, &sem, &NameResolver::new(), &recovered);
        assert!(lowered.struct_decls.is_empty());
        // The base local stays declared as `int64_t` (the width
        // fallback) — no promotion happened.
        assert_eq!(lowered.function.locals[0].ty, CType::i64());
    }

    #[test]
    fn b3_16_no_typemap_no_structs_keeps_b3_10_surface() {
        // With no RecoveredStructs at all the lowering pass emits no
        // typedefs and the v0 local stays width-typed.
        let ssa = ssa_pointer_field_access();
        let sem = sem_pointer_field_access();
        let lowered =
            lower_function_with_typedefs(&ssa, &sem, &NameResolver::new(), &Recovered::default());
        assert!(lowered.struct_decls.is_empty());
        assert_eq!(lowered.function.locals[0].ty, CType::i64());
    }

    #[test]
    fn b3_16_struct_field_padding_lays_out_to_observed_offset() {
        // A layout whose first field is at offset 0x10 emits a 16-byte
        // leading padding so `field_10` lives at byte 0x10 in the
        // typedef — matching the recovery's observation.
        let ssa = ssa_pointer_field_access();
        let sem = sem_pointer_field_access();
        let structs = pointer_struct_at(0, vec![(0x10, 8), (0x20, 4)]);
        let recovered = Recovered {
            structs: Some(&structs),
            ..Recovered::default()
        };
        let lowered = lower_function_with_typedefs(&ssa, &sem, &NameResolver::new(), &recovered);
        let typedef = &lowered.struct_decls[0];
        // padding(0..0x10) then field_10 then padding(0x18..0x20) then field_20.
        assert_eq!(typedef.fields.len(), 4);
        assert_eq!(typedef.fields[0].name, "__pad_0_10");
        match &typedef.fields[0].ty {
            CType::Array { count, .. } => assert_eq!(*count, 0x10),
            other => panic!("expected Array, got {other:?}"),
        }
        assert_eq!(typedef.fields[1].name, "field_10");
        assert_eq!(typedef.fields[2].name, "__pad_18_20");
        match &typedef.fields[2].ty {
            CType::Array { count, .. } => assert_eq!(*count, 8),
            other => panic!("expected Array, got {other:?}"),
        }
        assert_eq!(typedef.fields[3].name, "field_20");
    }
}
