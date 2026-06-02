//! Semantic IR → C AST (B2.8, FR-21).
//!
//! [`lower_function`] consumes the structurer's [`SemFunction`] together
//! with the underlying [`SsaFunction`] that holds the actual ops the
//! Semantic IR references via [`SsaRef`], and produces a [`Function`]
//! in the C AST. [`lower_unit`] is the translation-unit-level entry
//! point that wraps a slice of lowered functions in the standard
//! include directives.
//!
//! ## What the lowering pass commits to
//!
//! 1. **Every SSA value becomes a pre-declared C local.** The naming
//!    convention is `v<id>` — dense, deterministic, source-block-
//!    independent. All locals are declared at the top of the function
//!    with zero initialisers. This sidesteps the SSA-destruction
//!    problem at B2.8: phi nodes do not have to be moved into their
//!    predecessors because every name is already live everywhere. The
//!    cost is loss of fidelity for loop iteration variables — a
//!    correct lowering needs phi-out copies on every back edge, which
//!    is the structurer-aware destructor that lands when M3's idiom
//!    recogniser (B3.3) starts producing `While { … }` shapes.
//!    Compilability — the B2.8 done-when criterion — does not require
//!    semantic fidelity here.
//! 2. **The phi carrier renders as a `/* phi: … */` comment.** The
//!    underlying SSA value is already declared, so the comment is
//!    purely informational — but it is recorded so a reader can trace
//!    the value back to its SSA-layer definition (I-2).
//! 3. **Branch conditions are rendered as the raw operand.** The C
//!    compiler treats nonzero as true, matching the SSA semantics. The
//!    recovered Compare instruction whose result feeds the branch is
//!    already lowered as an assignment higher up, so we just reference
//!    the resulting boolean by name.
//! 4. **Side-effectful ops** (stores, calls without `dst`, opaque
//!    instructions) become [`Stmt::Store`] / [`Stmt::ExprStmt`].
//!    Value-producing ops become [`Stmt::Assign`].
//! 5. **Unknown call targets** lower through [`Expr::AddrLit`] so the
//!    cast-and-call shape compiles even when the symbol is not
//!    recoverable.
//!
//! ## What lands later
//!
//! - Real SSA destruction with phi-edge copies (post-B2.8, paired with
//!   the loop-shape recogniser).
//! - Type-aware lowering: thread [`dac_recovery::TypeMap`] through so
//!   each `vN` gets its recovered type instead of the `int64_t` fallback.
//!   This lands once the recovery pipeline plumbs the type map into
//!   the orchestrator and into here.
//! - Parameter and return type inference: today every lowered function
//!   is `void f(void)`; the calling-convention inference (B2.5,
//!   `dac_recovery::infer_calling_convention`) is not yet threaded.
//!
//! ## Determinism
//!
//! Pure function. Iteration over `ssa.values` is in ascending `ValueId`
//! order. Iteration over `body.stmts` is the structurer's order.

use std::collections::BTreeMap;

use dac_ir::sem::{Block as SemBlock, SemFunction, SsaRef, Stmt as SemStmt};
use dac_ir::ssa::{CompareKind, Operand, Phi, SsaFunction, SsaOp, ValueDef, ValueId};

use crate::ast::{
    BinaryOp, Block as CBlock, CType, Expr, Function, Item, Local, Stmt as CStmt, TranslationUnit,
    UnaryOp,
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

/// Lower one Semantic IR function to a C AST function.
///
/// The pass walks `sem.body` recursively, looking up referenced
/// SSA ops in `ssa.blocks`. Both arguments must describe the same
/// source function — the structurer's `ssa.function_address` must
/// equal `sem.function_address`.
#[must_use]
pub fn lower_function(ssa: &SsaFunction, sem: &SemFunction, resolver: &NameResolver) -> Function {
    debug_assert_eq!(ssa.function_address, sem.function_address);
    let name = sem
        .function_name
        .clone()
        .unwrap_or_else(|| format!("fn_{:x}", sem.function_address));
    let locals = lower_locals(ssa);
    let body = lower_block(ssa, &sem.body, resolver);
    // B2.5's calling-convention inference is not yet threaded through
    // here, so the return type comes from a much weaker signal: if any
    // `Stmt::Return` in the body carries a value, the function returns
    // `int64_t`; otherwise `void`. The recovered type lattice (B2.6)
    // will refine this when the orchestrator plumbs the TypeMap in.
    let return_type = if returns_value(&sem.body) {
        CType::i64()
    } else {
        CType::Void
    };
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
        params: Vec::new(),
        locals,
        body,
        leading_comment,
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
/// [`TranslationUnit`] suitable for emission.
#[must_use]
pub fn lower_unit(
    ssa_funcs: &[SsaFunction],
    sem_funcs: &[SemFunction],
    resolver: &NameResolver,
) -> TranslationUnit {
    debug_assert_eq!(ssa_funcs.len(), sem_funcs.len());
    let items = ssa_funcs
        .iter()
        .zip(sem_funcs.iter())
        .map(|(s, sem)| Item::Function(lower_function(s, sem, resolver)))
        .collect();
    TranslationUnit {
        includes: default_includes(),
        items,
    }
}

fn lower_locals(ssa: &SsaFunction) -> Vec<Local> {
    let mut locals = Vec::with_capacity(ssa.values.len());
    for def in &ssa.values {
        let ty = local_type(ssa, def);
        locals.push(Local {
            name: value_name(def.id),
            ty,
            init: Some(Expr::IntLit {
                value: 0,
                signed: true,
            }),
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

fn lower_block(ssa: &SsaFunction, block: &SemBlock, resolver: &NameResolver) -> CBlock {
    let mut stmts = Vec::with_capacity(block.stmts.len());
    for stmt in &block.stmts {
        lower_stmt(ssa, stmt, resolver, &mut stmts);
    }
    CBlock { stmts }
}

fn lower_stmt(ssa: &SsaFunction, stmt: &SemStmt, resolver: &NameResolver, out: &mut Vec<CStmt>) {
    match stmt {
        SemStmt::Phi { r, .. } => {
            let phi = phi_at(ssa, *r);
            out.push(CStmt::Comment(format_phi_comment(phi)));
        }
        SemStmt::Instr { r, .. } => {
            lower_instr(ssa, *r, resolver, out);
        }
        SemStmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            out.push(CStmt::If {
                cond: lower_operand(cond),
                then_body: lower_block(ssa, then_body, resolver),
                else_body: else_body.as_ref().map(|b| lower_block(ssa, b, resolver)),
            });
        }
        SemStmt::While { cond, body, .. } => {
            out.push(CStmt::While {
                cond: lower_operand(cond),
                body: lower_block(ssa, body, resolver),
            });
        }
        SemStmt::DoWhile { cond, body, .. } => {
            out.push(CStmt::DoWhile {
                body: lower_block(ssa, body, resolver),
                cond: lower_operand(cond),
            });
        }
        SemStmt::Loop { body, .. } => {
            out.push(CStmt::Loop {
                body: lower_block(ssa, body, resolver),
            });
        }
        SemStmt::Switch { source_block, .. } => {
            // B2.7 does not emit Switch; the variant exists so backends
            // pattern-match exhaustively. Degrade with a clear comment
            // until B3.3's recogniser starts producing arms.
            out.push(CStmt::Comment(format!(
                "switch from block {source_block} (idiom recogniser pending)"
            )));
        }
        SemStmt::Break { .. } => out.push(CStmt::Break),
        SemStmt::Continue { .. } => out.push(CStmt::Continue),
        SemStmt::Return { value, .. } => {
            out.push(CStmt::Return(value.as_ref().map(lower_operand)));
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

fn lower_instr(ssa: &SsaFunction, r: SsaRef, resolver: &NameResolver, out: &mut Vec<CStmt>) {
    let block = ssa.block(r.block);
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
            out.push(CStmt::Store {
                ty: int_type_for_width(*width),
                address: lower_operand(address),
                value: lower_operand(value),
            });
        }
        (None, SsaOp::Call { target, args }) => {
            out.push(CStmt::ExprStmt(call_expr(*target, args, resolver)));
        }
        (None, SsaOp::Opaque { mnemonic, args }) => {
            out.push(CStmt::ExprStmt(opaque_expr(mnemonic, args)));
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
            let expr = lower_value_op(*dst, op, resolver);
            out.push(CStmt::Assign {
                name: value_name(*dst),
                value: expr,
            });
        }
    }
}

fn lower_value_op(_dst: ValueId, op: &SsaOp, resolver: &NameResolver) -> Expr {
    match op {
        SsaOp::Move { src } => lower_operand(src),
        SsaOp::Add { lhs, rhs } => binary(BinaryOp::Add, lhs, rhs),
        SsaOp::Sub { lhs, rhs } => binary(BinaryOp::Sub, lhs, rhs),
        SsaOp::Mul { lhs, rhs } => binary(BinaryOp::Mul, lhs, rhs),
        SsaOp::And { lhs, rhs } => binary(BinaryOp::BitAnd, lhs, rhs),
        SsaOp::Or { lhs, rhs } => binary(BinaryOp::BitOr, lhs, rhs),
        SsaOp::Xor { lhs, rhs } => binary(BinaryOp::BitXor, lhs, rhs),
        SsaOp::Shl { lhs, rhs } => binary(BinaryOp::Shl, lhs, rhs),
        SsaOp::Shr { lhs, rhs } => binary(BinaryOp::Shr, lhs, rhs),
        SsaOp::Neg { src } => Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(lower_operand(src)),
        },
        SsaOp::Not { src } => Expr::Unary {
            op: UnaryOp::BitNot,
            expr: Box::new(lower_operand(src)),
        },
        SsaOp::Compare { kind, lhs, rhs } => Expr::Binary {
            op: compare_to_binary_op(*kind),
            lhs: Box::new(lower_operand(lhs)),
            rhs: Box::new(lower_operand(rhs)),
        },
        SsaOp::Load { address, width } => Expr::Load {
            ty: int_type_for_width(*width),
            address: Box::new(lower_operand(address)),
        },
        SsaOp::Call { target, args } => call_expr(*target, args, resolver),
        SsaOp::Opaque { mnemonic, args } => opaque_expr(mnemonic, args),
        SsaOp::Store { .. } => {
            // Stores have no `dst`, handled in the caller. If we get
            // here the IR is inconsistent — render an Opaque so the
            // output compiles.
            Expr::Opaque("store-with-dst".to_string())
        }
    }
}

fn lower_operand(op: &Operand) -> Expr {
    match op {
        Operand::Value(v) => Expr::Var(value_name(*v)),
        Operand::Const(c) => Expr::IntLit {
            value: *c,
            signed: true,
        },
        Operand::Undef => Expr::Undef,
    }
}

fn binary(op: BinaryOp, lhs: &Operand, rhs: &Operand) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lower_operand(lhs)),
        rhs: Box::new(lower_operand(rhs)),
    }
}

fn call_expr(target: Option<u64>, args: &[Operand], resolver: &NameResolver) -> Expr {
    let target_expr = match target {
        Some(addr) => match resolver.get(&addr) {
            Some(name) => Expr::Var(name.clone()),
            None => Expr::AddrLit(addr),
        },
        None => Expr::Opaque("indirect-call".into()),
    };
    Expr::Call {
        target: Box::new(target_expr),
        args: args.iter().map(lower_operand).collect(),
    }
}

fn opaque_expr(mnemonic: &str, args: &[Operand]) -> Expr {
    let mut body = String::from(mnemonic);
    for arg in args {
        body.push(' ');
        body.push_str(&format_operand(arg));
    }
    Expr::Opaque(body)
}

fn format_operand(op: &Operand) -> String {
    match op {
        Operand::Value(v) => value_name(*v),
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

fn format_phi_comment(phi: &Phi) -> String {
    let mut s = format!("phi {} <-", value_name(phi.dst));
    for (pred, op) in &phi.incoming {
        s.push_str(&format!(" (bb{pred}: {})", format_operand(op)));
    }
    s
}

#[must_use]
pub(crate) fn value_name(id: ValueId) -> String {
    format!("v{id}")
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
        let f = lower_function(&ssa, &sem, &NameResolver::new());
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
        let f = lower_function(&ssa, &sem, &NameResolver::new());
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
        let f = lower_function(&ssa, &sem, &NameResolver::new());
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
        let f = lower_function(&ssa, &sem, &r);
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
        let f = lower_function(&ssa, &sem, &NameResolver::new());
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
        let a = lower_function(&ssa, &sem, &NameResolver::new());
        let b = lower_function(&ssa, &sem, &NameResolver::new());
        assert_eq!(a, b);
    }
}
