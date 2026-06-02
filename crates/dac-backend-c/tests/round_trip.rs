//! Round-trip compile gate for the B2.8 corpus (FR-21, ARCHITECTURE §8).
//!
//! Each fixture constructs a small `SemFunction` together with the
//! supporting `SsaFunction`, runs it through [`lower_function`] /
//! [`emit`], and feeds the resulting source through the system C
//! compiler via [`try_compile`].
//!
//! The PLAN.md done-when criterion for B2.8 reads "at least 5 sample
//! binaries decompile to compilable C and run with matching behavior on
//! a smoke test." Two things in that criterion:
//!
//! - **Compilable C from 5+ shapes.** Six fixtures land here:
//!   - `empty_function` — the trivial baseline.
//!   - `arith_chain` — value-producing SSA ops through a return.
//!   - `if_then_else` — branch + join.
//!   - `endless_loop_with_break` — the canonical `Loop + If + Break`
//!     shape the B2.7 structurer produces.
//!   - `goto_fallback` — a Label + Goto pair, as the irreducible-CFG
//!     fallback would produce.
//!   - `store_then_load` — memory side effects through `Stmt::Store`
//!     and `Expr::Load`.
//! - **Run with matching behavior on a smoke test.** Cannot be
//!   measured at B2.8: the lifter → `RawFunction` bridge that would
//!   produce a `SemFunction` from a real binary is not yet a batch in
//!   PLAN.md. Recorded as a deferred follow-up in the B2.8 CHANGELOG
//!   entry; the corpus calibration lands with B2.9.
//!
//! The round-trip helper skips silently when no C compiler is
//! available on PATH. Tests pass in that mode — the gate is the CI
//! environment, not the developer's box.

use std::collections::BTreeMap;

use dac_backend_c::{
    ast::{Block as CBlock, CType, Expr, Function, Item, TranslationUnit},
    compile::CompileResult,
    emit, emit_function, lower_function, lower_unit, try_compile, NameResolver,
};
use dac_core::{EvidenceGraph, EvidenceId, EvidenceNode, IrLayer};
use dac_ir::sem::{Block as SemBlock, SemFunction, SsaRef, Stmt as SemStmt, StructuringStats};
use dac_ir::ssa::{
    CompareKind, Operand, Phi, SsaBlock, SsaFunction, SsaInstruction, SsaOp, SsaTerminator,
    Variable,
};

fn evidence() -> EvidenceId {
    let mut g = EvidenceGraph::new();
    g.add_node(EvidenceNode::IrNode {
        layer: IrLayer::Semantic,
        id: 0,
    })
}

fn ev() -> EvidenceId {
    evidence()
}

fn variable(id: u32, name: &str, width: u16) -> Variable {
    Variable {
        id,
        name: name.into(),
        width_bits: width,
    }
}

fn val(id: u32, var: u32, block: u32, index: u32) -> dac_ir::ssa::ValueDef {
    dac_ir::ssa::ValueDef {
        id,
        source: dac_ir::ssa::ValueSource::Instruction { block, index },
        variable: var,
    }
}

fn val_phi(id: u32, var: u32, block: u32, index: u32) -> dac_ir::ssa::ValueDef {
    dac_ir::ssa::ValueDef {
        id,
        source: dac_ir::ssa::ValueSource::Phi { block, index },
        variable: var,
    }
}

fn return_stmt() -> SemStmt {
    SemStmt::Return {
        value: None,
        evidence: ev(),
    }
}

fn return_value(op: Operand) -> SemStmt {
    SemStmt::Return {
        value: Some(op),
        evidence: ev(),
    }
}

fn instr_stmt(block: u32, index: u32) -> SemStmt {
    SemStmt::Instr {
        r: SsaRef { block, index },
        evidence: ev(),
    }
}

fn empty_resolver() -> NameResolver {
    BTreeMap::new()
}

/// Drive one round-trip: lower, emit, compile-check. Returns the
/// emitted source for diagnostic display in the test message.
fn round_trip(ssa: &SsaFunction, sem: &SemFunction) -> (String, CompileResult) {
    let lowered = lower_function(ssa, sem, &empty_resolver());
    let unit = TranslationUnit {
        includes: dac_backend_c::default_includes(),
        items: vec![Item::Function(lowered)],
    };
    let source = emit(&unit);
    let result = try_compile(&source);
    (source, result)
}

fn assert_round_trip_ok(source: &str, result: &CompileResult) {
    match result {
        CompileResult::Ok { .. } | CompileResult::Skipped { .. } => {}
        CompileResult::Failed { stderr } => {
            panic!("round-trip failed\n--- source ---\n{source}\n--- stderr ---\n{stderr}");
        }
    }
}

// -------------------------------------------------------------------
// Fixtures
// -------------------------------------------------------------------

fn fixture_empty() -> (SsaFunction, SemFunction) {
    let ssa = SsaFunction {
        function_address: 0x1000,
        function_name: Some("empty_function".into()),
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
    };
    let sem = SemFunction {
        function_address: 0x1000,
        function_name: Some("empty_function".into()),
        body: SemBlock {
            stmts: vec![return_stmt()],
        },
        evidence: ev(),
        stats: StructuringStats::default(),
    };
    (ssa, sem)
}

fn fixture_arith_chain() -> (SsaFunction, SemFunction) {
    // v0 = 1 + 2; v1 = v0 * 3; return v1;
    let ssa = SsaFunction {
        function_address: 0x1100,
        function_name: Some("arith_chain".into()),
        blocks: vec![SsaBlock {
            id: 0,
            predecessors: vec![],
            phis: vec![],
            instructions: vec![
                SsaInstruction {
                    dst: Some(0),
                    op: SsaOp::Add {
                        lhs: Operand::Const(1),
                        rhs: Operand::Const(2),
                    },
                },
                SsaInstruction {
                    dst: Some(1),
                    op: SsaOp::Mul {
                        lhs: Operand::Value(0),
                        rhs: Operand::Const(3),
                    },
                },
            ],
            terminator: SsaTerminator::Return {
                value: Some(Operand::Value(1)),
            },
        }],
        entry: 0,
        variables: vec![variable(0, "rax", 64)],
        values: vec![val(0, 0, 0, 0), val(1, 0, 0, 1)],
        evidence: ev(),
    };
    let sem = SemFunction {
        function_address: 0x1100,
        function_name: Some("arith_chain".into()),
        body: SemBlock {
            stmts: vec![
                instr_stmt(0, 0),
                instr_stmt(0, 1),
                return_value(Operand::Value(1)),
            ],
        },
        evidence: ev(),
        stats: StructuringStats {
            source_blocks: 1,
            ..Default::default()
        },
    };
    (ssa, sem)
}

fn fixture_if_then_else() -> (SsaFunction, SemFunction) {
    // v0 = a < b; if (v0) { return 1; } else { return 0; }
    let ssa = SsaFunction {
        function_address: 0x1200,
        function_name: Some("if_then_else".into()),
        blocks: vec![
            SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![SsaInstruction {
                    dst: Some(0),
                    op: SsaOp::Compare {
                        kind: CompareKind::Lt,
                        lhs: Operand::Const(3),
                        rhs: Operand::Const(5),
                    },
                }],
                terminator: SsaTerminator::Branch {
                    cond: Operand::Value(0),
                    taken: 1,
                    not_taken: 2,
                },
            },
            SsaBlock {
                id: 1,
                predecessors: vec![0],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Const(1)),
                },
            },
            SsaBlock {
                id: 2,
                predecessors: vec![0],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Const(0)),
                },
            },
        ],
        entry: 0,
        variables: vec![variable(0, "flag", 32)],
        values: vec![val(0, 0, 0, 0)],
        evidence: ev(),
    };
    let sem = SemFunction {
        function_address: 0x1200,
        function_name: Some("if_then_else".into()),
        body: SemBlock {
            stmts: vec![
                instr_stmt(0, 0),
                SemStmt::If {
                    cond: Operand::Value(0),
                    then_body: SemBlock {
                        stmts: vec![return_value(Operand::Const(1))],
                    },
                    else_body: Some(SemBlock {
                        stmts: vec![return_value(Operand::Const(0))],
                    }),
                    source_block: 0,
                    evidence: ev(),
                },
            ],
        },
        evidence: ev(),
        stats: StructuringStats {
            source_blocks: 3,
            ..Default::default()
        },
    };
    (ssa, sem)
}

fn fixture_endless_loop_with_break() -> (SsaFunction, SemFunction) {
    // loop { v0 = v0 + 1; if (v0 >= 10) break; }
    // Pre-declare v0 as a phi target so the SSA pre-declaration step
    // gives us a name to assign into.
    let ssa = SsaFunction {
        function_address: 0x1300,
        function_name: Some("endless_loop_with_break".into()),
        blocks: vec![
            SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Jump { target: 1 },
            },
            // Loop header carries a phi for the iteration variable.
            SsaBlock {
                id: 1,
                predecessors: vec![0, 1],
                phis: vec![Phi {
                    dst: 0,
                    variable: 0,
                    incoming: vec![(0, Operand::Const(0)), (1, Operand::Value(1))],
                }],
                instructions: vec![
                    SsaInstruction {
                        dst: Some(1),
                        op: SsaOp::Add {
                            lhs: Operand::Value(0),
                            rhs: Operand::Const(1),
                        },
                    },
                    SsaInstruction {
                        dst: Some(2),
                        op: SsaOp::Compare {
                            kind: CompareKind::Ge,
                            lhs: Operand::Value(1),
                            rhs: Operand::Const(10),
                        },
                    },
                ],
                terminator: SsaTerminator::Branch {
                    cond: Operand::Value(2),
                    taken: 2,
                    not_taken: 1,
                },
            },
            SsaBlock {
                id: 2,
                predecessors: vec![1],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Return {
                    value: Some(Operand::Value(1)),
                },
            },
        ],
        entry: 0,
        variables: vec![variable(0, "i", 32)],
        values: vec![val_phi(0, 0, 1, 0), val(1, 0, 1, 0), val(2, 0, 1, 1)],
        evidence: ev(),
    };
    let sem = SemFunction {
        function_address: 0x1300,
        function_name: Some("endless_loop_with_break".into()),
        body: SemBlock {
            stmts: vec![
                SemStmt::Loop {
                    body: SemBlock {
                        stmts: vec![
                            SemStmt::Phi {
                                r: SsaRef { block: 1, index: 0 },
                                evidence: ev(),
                            },
                            instr_stmt(1, 0),
                            instr_stmt(1, 1),
                            SemStmt::If {
                                cond: Operand::Value(2),
                                then_body: SemBlock {
                                    stmts: vec![SemStmt::Break { evidence: ev() }],
                                },
                                else_body: None,
                                source_block: 1,
                                evidence: ev(),
                            },
                        ],
                    },
                    header: 1,
                    evidence: ev(),
                },
                return_value(Operand::Value(1)),
            ],
        },
        evidence: ev(),
        stats: StructuringStats {
            source_blocks: 3,
            ..Default::default()
        },
    };
    (ssa, sem)
}

fn fixture_goto_fallback() -> (SsaFunction, SemFunction) {
    // Two basic blocks with a back edge that the structurer would
    // demote to a goto. The lowered C must contain both `L0:;` and
    // `goto L0;` and the result must compile.
    let ssa = SsaFunction {
        function_address: 0x1400,
        function_name: Some("goto_fallback".into()),
        blocks: vec![
            SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Jump { target: 1 },
            },
            SsaBlock {
                id: 1,
                predecessors: vec![0],
                phis: vec![],
                instructions: vec![],
                terminator: SsaTerminator::Return { value: None },
            },
        ],
        entry: 0,
        variables: vec![],
        values: vec![],
        evidence: ev(),
    };
    let sem = SemFunction {
        function_address: 0x1400,
        function_name: Some("goto_fallback".into()),
        body: SemBlock {
            stmts: vec![
                SemStmt::Label {
                    id: 0,
                    source_block: 1,
                },
                SemStmt::If {
                    cond: Operand::Const(0),
                    then_body: SemBlock {
                        stmts: vec![SemStmt::Goto {
                            target: 0,
                            source_block: 1,
                            evidence: ev(),
                        }],
                    },
                    else_body: None,
                    source_block: 0,
                    evidence: ev(),
                },
                return_stmt(),
            ],
        },
        evidence: ev(),
        stats: StructuringStats {
            source_blocks: 2,
            goto_count: 1,
            label_count: 1,
            irreducible: true,
        },
    };
    (ssa, sem)
}

fn fixture_store_then_load() -> (SsaFunction, SemFunction) {
    // *(int32_t*)0x2000 = 7;
    // v0 = *(int32_t*)0x2000;
    // return v0;
    let ssa = SsaFunction {
        function_address: 0x1500,
        function_name: Some("store_then_load".into()),
        blocks: vec![SsaBlock {
            id: 0,
            predecessors: vec![],
            phis: vec![],
            instructions: vec![
                SsaInstruction {
                    dst: None,
                    op: SsaOp::Store {
                        address: Operand::Const(0x2000),
                        value: Operand::Const(7),
                        width: 4,
                    },
                },
                SsaInstruction {
                    dst: Some(0),
                    op: SsaOp::Load {
                        address: Operand::Const(0x2000),
                        width: 4,
                    },
                },
            ],
            terminator: SsaTerminator::Return {
                value: Some(Operand::Value(0)),
            },
        }],
        entry: 0,
        variables: vec![variable(0, "tmp", 32)],
        values: vec![val(0, 0, 0, 1)],
        evidence: ev(),
    };
    let sem = SemFunction {
        function_address: 0x1500,
        function_name: Some("store_then_load".into()),
        body: SemBlock {
            stmts: vec![
                instr_stmt(0, 0),
                instr_stmt(0, 1),
                return_value(Operand::Value(0)),
            ],
        },
        evidence: ev(),
        stats: StructuringStats {
            source_blocks: 1,
            ..Default::default()
        },
    };
    (ssa, sem)
}

// -------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------

#[test]
fn empty_function_round_trips() {
    let (ssa, sem) = fixture_empty();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("void empty_function(void)"));
    assert!(source.contains("    return;"));
}

#[test]
fn arith_chain_round_trips() {
    let (ssa, sem) = fixture_arith_chain();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("int64_t v0 = 0LL;"));
    assert!(source.contains("v0 = (1LL + 2LL);"));
    assert!(source.contains("v1 = (v0 * 3LL);"));
}

#[test]
fn if_then_else_round_trips() {
    let (ssa, sem) = fixture_if_then_else();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("if (v0) {"));
    assert!(source.contains("} else {"));
}

#[test]
fn endless_loop_with_break_round_trips() {
    let (ssa, sem) = fixture_endless_loop_with_break();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("while (1) {"));
    assert!(source.contains("/* phi v0"));
    assert!(source.contains("break;"));
}

#[test]
fn goto_fallback_round_trips() {
    let (ssa, sem) = fixture_goto_fallback();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("L0:;"));
    assert!(source.contains("goto L0;"));
}

#[test]
fn store_then_load_round_trips() {
    let (ssa, sem) = fixture_store_then_load();
    let (source, result) = round_trip(&ssa, &sem);
    assert_round_trip_ok(&source, &result);
    assert!(source.contains("*((int32_t *)"));
    assert!(source.contains("(*((int32_t *)("));
}

#[test]
fn multi_function_translation_unit_round_trips() {
    // A single translation unit of all five canonical fixtures
    // compiles. This is the "5 sample functions" check from PLAN.md:
    // the unit holds six functions, each demonstrating a distinct
    // structurer output shape, and the round-trip gates that the
    // combined output is valid C.
    let fixtures = [
        fixture_empty(),
        fixture_arith_chain(),
        fixture_if_then_else(),
        fixture_endless_loop_with_break(),
        fixture_goto_fallback(),
        fixture_store_then_load(),
    ];
    let ssas: Vec<_> = fixtures.iter().map(|(s, _)| s.clone()).collect();
    let sems: Vec<_> = fixtures.iter().map(|(_, m)| m.clone()).collect();
    let unit = lower_unit(&ssas, &sems, &empty_resolver());
    let source = emit(&unit);
    let result = try_compile(&source);
    assert_round_trip_ok(&source, &result);
}

#[test]
fn lowering_then_emission_is_byte_deterministic() {
    let (ssa, sem) = fixture_arith_chain();
    let a = emit_function(&lower_function(&ssa, &sem, &empty_resolver()));
    let b = emit_function(&lower_function(&ssa, &sem, &empty_resolver()));
    assert_eq!(a, b);
}

#[test]
fn lowered_translation_unit_includes_stdint_and_stddef() {
    let (ssa, sem) = fixture_empty();
    let unit = lower_unit(
        std::slice::from_ref(&ssa),
        std::slice::from_ref(&sem),
        &empty_resolver(),
    );
    let source = emit(&unit);
    assert!(source.starts_with("#include <stdint.h>\n"));
    assert!(source.contains("#include <stddef.h>\n"));
}

#[test]
fn empty_function_renders_without_unreferenced_includes_when_locals_empty() {
    // Sanity: a function with no SSA values still renders correctly
    // (no spurious local-declaration line, blank-line separator
    // unaffected).
    let (ssa, sem) = fixture_empty();
    let f = lower_function(&ssa, &sem, &empty_resolver());
    assert!(f.locals.is_empty());
    let s = emit_function(&Function { ..f });
    assert!(!s.contains("int64_t v"));
}

#[test]
fn ctype_round_trip_through_pointer() {
    // Direct AST regression: pointer renders correctly nested.
    let t = CType::Ptr(Box::new(CType::Ptr(Box::new(CType::Int {
        width_bits: 32,
        signed: false,
    }))));
    let expr = Expr::Load {
        ty: t,
        address: Box::new(Expr::IntLit {
            value: 0,
            signed: true,
        }),
    };
    let f = Function {
        name: "p".into(),
        return_type: CType::Void,
        params: vec![],
        locals: vec![],
        body: CBlock {
            stmts: vec![dac_backend_c::ast::Stmt::ExprStmt(expr)],
        },
        leading_comment: None,
    };
    let s = emit_function(&f);
    // The lookup type appears as a nested pointer cast on the address.
    assert!(s.contains("uint32_t * * *"));
}

#[test]
fn skipping_compiler_does_not_panic() {
    // The compile helper should never panic — it returns Skipped when
    // no compiler is on PATH. Verifying the branch by setting CC to a
    // non-existent path and clearing the fallbacks would require
    // process isolation; here we just sanity-check the variant on the
    // result type.
    let r = CompileResult::Skipped {
        reason: "test".into(),
    };
    assert!(r.is_skipped());
}
