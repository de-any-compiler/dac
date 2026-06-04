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
//!    the function with zero initialisers. Locals stay typed by the
//!    SSA variable's `width_bits` (the B2.8 fallback); we deliberately
//!    do **not** retype them from the [`TypeMap`] because the lifter's
//!    sub-register arithmetic mixes pointer-typed and int-typed values
//!    in ways the round-trip `cc` gate would reject under
//!    `-Wint-conversion`. Refining local types is a B3 follow-up
//!    shelf entry.
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
//! 7. **Recovered struct-field accesses surface as a comment**
//!    (B3.10, FR-17). The orchestrator detects pointer-anchored
//!    struct layouts via [`RecoveredStructs`] but the emitted source
//!    still uses the bare `*(int*)(base + 0xN)` shape because the C
//!    AST does not yet model struct typedefs in scope. The lowering
//!    pass records `/* recovered field: base=v_<id> offset=0x<hex>
//!    field=field_<hex> */` above each matching access so the reader
//!    sees where the field surface would land. Promoting these to
//!    `base->field` requires translation-unit-level struct
//!    declarations and is a B3 follow-up shelf entry.
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
    TranslationUnit, UnaryOp,
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
#[must_use]
pub fn lower_function(
    ssa: &SsaFunction,
    sem: &SemFunction,
    resolver: &NameResolver,
    recovered: &Recovered<'_>,
) -> Function {
    debug_assert_eq!(ssa.function_address, sem.function_address);
    let name = sem
        .function_name
        .clone()
        .unwrap_or_else(|| format!("fn_{:x}", sem.function_address));
    let params = build_params(recovered);
    let parameter_inits = parameter_initialisers(recovered);
    let locals = lower_locals(ssa, recovered, &parameter_inits);
    let return_type = pick_return_type(&sem.body, ssa, recovered);
    let ctx = LowerCtx {
        ssa,
        resolver,
        recovered,
        return_type: return_type.clone(),
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
    Function {
        name,
        return_type,
        params,
        locals,
        body,
        leading_comment,
    }
}

/// Map from parameter `ValueId` to the [`ParameterInit`] describing
/// the matching [`Param`] (its name and type).
type ParameterInits = BTreeMap<ValueId, ParameterInit>;

/// Build the C parameter list from the convention's `int_args` slot.
/// The N-th argument register becomes `argN`; its type comes from
/// the [`TypeMap`] when available, falling back to the SSA variable's
/// width.
fn build_params(recovered: &Recovered<'_>) -> Vec<Param> {
    let Some(sig) = recovered.signature else {
        return Vec::new();
    };
    sig.int_args
        .iter()
        .map(|arg| Param {
            name: parameter_name(arg),
            ty: parameter_type(arg, recovered.types),
        })
        .collect()
}

fn parameter_initialisers(recovered: &Recovered<'_>) -> ParameterInits {
    let mut m = ParameterInits::new();
    let Some(sig) = recovered.signature else {
        return m;
    };
    for arg in &sig.int_args {
        m.insert(
            arg.value,
            ParameterInit {
                arg_name: parameter_name(arg),
                arg_type: parameter_type(arg, recovered.types),
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

fn parameter_type(arg: &RegisterArg, types: Option<&TypeMap>) -> CType {
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
#[must_use]
pub fn lower_unit(
    ssa_funcs: &[SsaFunction],
    sem_funcs: &[SemFunction],
    resolver: &NameResolver,
    recovered: &[Recovered<'_>],
) -> TranslationUnit {
    debug_assert_eq!(ssa_funcs.len(), sem_funcs.len());
    let default = Recovered::default();
    let items = ssa_funcs
        .iter()
        .zip(sem_funcs.iter())
        .enumerate()
        .map(|(i, (s, sem))| {
            let r = recovered.get(i).copied().unwrap_or(default);
            Item::Function(lower_function(s, sem, resolver, &r))
        })
        .collect();
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
}

fn lower_locals(
    ssa: &SsaFunction,
    recovered: &Recovered<'_>,
    parameter_inits: &ParameterInits,
) -> Vec<Local> {
    let mut locals = Vec::with_capacity(ssa.values.len());
    for def in &ssa.values {
        let ty = local_type(ssa, def);
        let init = match parameter_inits.get(&def.id) {
            Some(p) => Some(cast_if_needed(
                &p.arg_type,
                &ty,
                Expr::Var(p.arg_name.clone()),
            )),
            None => Some(Expr::IntLit {
                value: 0,
                signed: true,
            }),
        };
        locals.push(Local {
            name: value_name(def.id, recovered.names),
            ty,
            init,
        });
    }
    locals
}

fn local_type(ssa: &SsaFunction, def: &ValueDef) -> CType {
    let var = ssa.variable(def.variable);
    match var.width_bits {
        0 => CType::i64(),
        w => CType::Int {
            width_bits: w,
            signed: true,
        },
    }
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

impl<'a> LowerCtx<'a> {
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
                    cond: lower_operand(cond, self.recovered.names),
                    then_body: self.lower_block(then_body),
                    else_body: else_body.as_ref().map(|b| self.lower_block(b)),
                });
            }
            SemStmt::While { cond, body, .. } => {
                out.push(CStmt::While {
                    cond: lower_operand(cond, self.recovered.names),
                    body: self.lower_block(body),
                });
            }
            SemStmt::DoWhile { cond, body, .. } => {
                out.push(CStmt::DoWhile {
                    body: self.lower_block(body),
                    cond: lower_operand(cond, self.recovered.names),
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
                let arms_c = arms
                    .iter()
                    .map(|a| crate::ast::SwitchArm {
                        value: a.value,
                        body: self.lower_block(&a.body),
                    })
                    .collect();
                let default_c = default.as_ref().map(|d| self.lower_block(d));
                out.push(CStmt::Comment(format!(
                    "recovered switch table at block {source_block} (arm resolution pending)"
                )));
                out.push(CStmt::Switch {
                    scrutinee: lower_operand(scrutinee, self.recovered.names),
                    arms: arms_c,
                    default: default_c,
                });
            }
            SemStmt::Break { .. } => out.push(CStmt::Break),
            SemStmt::Continue { .. } => out.push(CStmt::Continue),
            SemStmt::Return { value, .. } => {
                let v = value.as_ref().map(|op| {
                    let raw = lower_operand(op, self.recovered.names);
                    cast_if_needed(&CType::i64(), &self.return_type, raw)
                });
                out.push(CStmt::Return(v));
            }
            SemStmt::Label { id, .. } => out.push(CStmt::Label(*id)),
            SemStmt::Goto { target, .. } => out.push(CStmt::Goto(*target)),
            SemStmt::Unreachable { source_block, .. } => {
                out.push(CStmt::Comment(format!(
                    "structurally unreachable: block {source_block}"
                )));
                out.push(CStmt::Unreachable);
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
                if let Some(field) = self.match_struct_field(address) {
                    out.push(CStmt::Comment(field_provenance_comment(&field)));
                }
                out.push(CStmt::Store {
                    ty: int_type_for_width(*width),
                    address: lower_operand(address, self.recovered.names),
                    value: lower_operand(value, self.recovered.names),
                });
            }
            (None, SsaOp::Call { target, args }) => {
                out.push(CStmt::ExprStmt(call_expr(
                    *target,
                    args,
                    self.resolver,
                    self.recovered.names,
                )));
            }
            (None, SsaOp::Opaque { mnemonic, args }) => {
                out.push(CStmt::ExprStmt(opaque_expr(
                    mnemonic,
                    args,
                    self.recovered.names,
                )));
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
                out.push(CStmt::Assign {
                    name: value_name(*dst, self.recovered.names),
                    value: expr,
                });
            }
        }
    }

    fn lower_value_op(&self, _dst: ValueId, op: &SsaOp) -> Expr {
        let names = self.recovered.names;
        match op {
            SsaOp::Move { src } => lower_operand(src, names),
            SsaOp::Add { lhs, rhs } => binary(BinaryOp::Add, lhs, rhs, names),
            SsaOp::Sub { lhs, rhs } => binary(BinaryOp::Sub, lhs, rhs, names),
            SsaOp::Mul { lhs, rhs } => binary(BinaryOp::Mul, lhs, rhs, names),
            SsaOp::And { lhs, rhs } => binary(BinaryOp::BitAnd, lhs, rhs, names),
            SsaOp::Or { lhs, rhs } => binary(BinaryOp::BitOr, lhs, rhs, names),
            SsaOp::Xor { lhs, rhs } => binary(BinaryOp::BitXor, lhs, rhs, names),
            SsaOp::Shl { lhs, rhs } => binary(BinaryOp::Shl, lhs, rhs, names),
            SsaOp::Shr { lhs, rhs } => binary(BinaryOp::Shr, lhs, rhs, names),
            SsaOp::Neg { src } => Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(lower_operand(src, names)),
            },
            SsaOp::Not { src } => Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(lower_operand(src, names)),
            },
            SsaOp::Compare { kind, lhs, rhs } => Expr::Binary {
                op: compare_to_binary_op(*kind),
                lhs: Box::new(lower_operand(lhs, names)),
                rhs: Box::new(lower_operand(rhs, names)),
            },
            SsaOp::Load { address, width } => Expr::Load {
                ty: int_type_for_width(*width),
                address: Box::new(lower_operand(address, names)),
            },
            SsaOp::Call { target, args } => call_expr(*target, args, self.resolver, names),
            SsaOp::Opaque { mnemonic, args } => opaque_expr(mnemonic, args, names),
            SsaOp::Store { .. } => {
                // Stores have no `dst`, handled in the caller. If we get
                // here the IR is inconsistent — render an Opaque so the
                // output compiles.
                Expr::Opaque("store-with-dst".to_string())
            }
        }
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
fn map_ir_type(ty: &IrType) -> Option<CType> {
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

fn binary(op: BinaryOp, lhs: &Operand, rhs: &Operand, names: Option<&NameTable>) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lower_operand(lhs, names)),
        rhs: Box::new(lower_operand(rhs, names)),
    }
}

fn call_expr(
    target: Option<u64>,
    args: &[Operand],
    resolver: &NameResolver,
    names: Option<&NameTable>,
) -> Expr {
    let target_expr = match target {
        Some(addr) => match resolver.get(&addr) {
            Some(name) => Expr::Var(name.clone()),
            None => Expr::AddrLit(addr),
        },
        None => Expr::Opaque("indirect-call".into()),
    };
    Expr::Call {
        target: Box::new(target_expr),
        args: args.iter().map(|a| lower_operand(a, names)).collect(),
    }
}

fn opaque_expr(mnemonic: &str, args: &[Operand], names: Option<&NameTable>) -> Expr {
    let mut body = String::from(mnemonic);
    for arg in args {
        body.push(' ');
        body.push_str(&format_operand(arg, names));
    }
    Expr::Opaque(body)
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
}
