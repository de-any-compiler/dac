//! C AST → formatted source string.
//!
//! The emitter is a hand-rolled pretty-printer. It is deliberately
//! simple — no external formatter is invoked — so the output is
//! byte-deterministic and the backend depends on no system tools
//! beyond the round-trip `cc` check in [`crate::compile`].
//!
//! ## Style
//!
//! - 4-space indent.
//! - K&R braces: `if (cond) {` on the same line as the keyword,
//!   `}` aligned with the keyword.
//! - One statement per line. Empty lines only between top-level items.
//! - Parenthesise both children of every [`crate::ast::Expr::Binary`]
//!   so the emitter never has to reason about operator precedence.
//!   The result is verbose but unambiguous; B3.3's idiom recogniser
//!   gets a separate pass when style matters more than safety.
//! - Integer literals are emitted in decimal with `LL` / `ULL` suffix
//!   for `i64` / `u64` widths.
//!
//! ## Determinism
//!
//! Pure function from [`TranslationUnit`] to `String`. No iteration
//! over hashed containers. Same AST in → same string out.

use std::fmt::Write as _;

use crate::ast::{
    BinaryOp, Block, CType, Expr, Function, Item, Local, Param, Stmt, SwitchArm, TranslationUnit,
    UnaryOp,
};

/// Render a [`TranslationUnit`] as formatted C source.
#[must_use]
pub fn emit(unit: &TranslationUnit) -> String {
    let mut out = String::new();
    let mut p = Printer::new(&mut out);
    for inc in &unit.includes {
        p.line(inc);
    }
    if !unit.includes.is_empty() && !unit.items.is_empty() {
        p.blank();
    }
    for (i, item) in unit.items.iter().enumerate() {
        if i > 0 {
            p.blank();
        }
        emit_item(&mut p, item);
    }
    out
}

/// Render a [`Function`] standalone. Convenience for tests that
/// construct a function in isolation.
#[must_use]
pub fn emit_function(f: &Function) -> String {
    let mut out = String::new();
    emit_item(&mut Printer::new(&mut out), &Item::Function(f.clone()));
    out
}

fn emit_item(p: &mut Printer<'_>, item: &Item) {
    match item {
        Item::Function(f) => emit_function_into(p, f),
    }
}

fn emit_function_into(p: &mut Printer<'_>, f: &Function) {
    if let Some(comment) = &f.leading_comment {
        for line in comment.lines() {
            p.write_line(&format!("/* {line} */"));
        }
    }
    let mut signature = String::new();
    signature.push_str(&render_type_prefix(&f.return_type));
    signature.push(' ');
    signature.push_str(&f.name);
    signature.push('(');
    if f.params.is_empty() {
        signature.push_str("void");
    } else {
        for (i, param) in f.params.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&render_param(param));
        }
    }
    signature.push_str(") {");
    p.write_line(&signature);
    p.indent();
    for local in &f.locals {
        emit_local(p, local);
    }
    if !f.locals.is_empty() && !f.body.stmts.is_empty() {
        p.blank();
    }
    emit_block_inner(p, &f.body);
    p.dedent();
    p.write_line("}");
}

fn emit_local(p: &mut Printer<'_>, local: &Local) {
    let mut line = render_decl(&local.ty, &local.name);
    if let Some(init) = &local.init {
        line.push_str(" = ");
        line.push_str(&render_expr(init));
    }
    line.push(';');
    p.write_line(&line);
}

fn emit_block_inner(p: &mut Printer<'_>, block: &Block) {
    for stmt in &block.stmts {
        emit_stmt(p, stmt);
    }
}

fn emit_stmt(p: &mut Printer<'_>, stmt: &Stmt) {
    match stmt {
        Stmt::Decl { ty, name, init } => {
            let mut line = render_decl(ty, name);
            if let Some(init) = init {
                line.push_str(" = ");
                line.push_str(&render_expr(init));
            }
            line.push(';');
            p.write_line(&line);
        }
        Stmt::Assign { name, value } => {
            p.write_line(&format!("{name} = {};", render_expr(value)));
        }
        Stmt::Store { ty, address, value } => {
            // `*((ty *)(address)) = value;`
            let ty_ptr = render_type_prefix(&CType::Ptr(Box::new(ty.clone())));
            p.write_line(&format!(
                "*(({ty_ptr})({})) = {};",
                render_expr(address),
                render_expr(value)
            ));
        }
        Stmt::FieldStore {
            base,
            field,
            arrow,
            value,
        } => {
            let op = if *arrow { "->" } else { "." };
            p.write_line(&format!(
                "{}{op}{field} = {};",
                render_expr(base),
                render_expr(value)
            ));
        }
        Stmt::ExprStmt(expr) => {
            p.write_line(&format!("{};", render_expr(expr)));
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
        } => {
            p.write_line(&format!("if ({}) {{", render_expr(cond)));
            p.indent();
            emit_block_inner(p, then_body);
            p.dedent();
            match else_body {
                Some(eb) => {
                    p.write_line("} else {");
                    p.indent();
                    emit_block_inner(p, eb);
                    p.dedent();
                    p.write_line("}");
                }
                None => p.write_line("}"),
            }
        }
        Stmt::Loop { body } => {
            p.write_line("while (1) {");
            p.indent();
            emit_block_inner(p, body);
            p.dedent();
            p.write_line("}");
        }
        Stmt::While { cond, body } => {
            p.write_line(&format!("while ({}) {{", render_expr(cond)));
            p.indent();
            emit_block_inner(p, body);
            p.dedent();
            p.write_line("}");
        }
        Stmt::DoWhile { body, cond } => {
            p.write_line("do {");
            p.indent();
            emit_block_inner(p, body);
            p.dedent();
            p.write_line(&format!("}} while ({});", render_expr(cond)));
        }
        Stmt::Break => p.write_line("break;"),
        Stmt::Continue => p.write_line("continue;"),
        Stmt::Return(None) => p.write_line("return;"),
        Stmt::Return(Some(e)) => p.write_line(&format!("return {};", render_expr(e))),
        // Labels are emitted at column 0 (one indent less than the
        // surrounding statements) so the C compiler accepts them. We
        // achieve this by dedenting one step for the label line.
        Stmt::Label(id) => p.write_label(*id),
        Stmt::Goto(id) => p.write_line(&format!("goto L{id};")),
        Stmt::Switch {
            scrutinee,
            arms,
            default,
        } => {
            p.write_line(&format!("switch ({}) {{", render_expr(scrutinee)));
            p.indent();
            for arm in arms {
                emit_switch_arm(p, arm);
            }
            if let Some(def) = default {
                p.write_line("default: {");
                p.indent();
                emit_block_inner(p, def);
                p.dedent();
                p.write_line("}");
            }
            p.dedent();
            p.write_line("}");
        }
        Stmt::Comment(text) => {
            for line in text.lines() {
                p.write_line(&format!("/* {line} */"));
            }
        }
        Stmt::Unreachable => p.write_line("__builtin_unreachable();"),
    }
}

fn emit_switch_arm(p: &mut Printer<'_>, arm: &SwitchArm) {
    p.write_line(&format!("case {}LL: {{", arm.value));
    p.indent();
    emit_block_inner(p, &arm.body);
    p.dedent();
    p.write_line("}");
}

fn render_param(p: &Param) -> String {
    render_decl(&p.ty, &p.name)
}

fn render_decl(ty: &CType, name: &str) -> String {
    // For B2.8 every declarator is simple (no arrays, no function
    // pointers), so the spelling is `<type> <name>`.
    format!("{} {}", render_type_prefix(ty), name)
}

fn render_type_prefix(ty: &CType) -> String {
    match ty {
        CType::Void => "void".to_string(),
        CType::Int { width_bits, signed } => render_int_type(*width_bits, *signed),
        CType::Ptr(inner) => format!("{} *", render_type_prefix(inner)),
    }
}

fn render_int_type(width_bits: u16, signed: bool) -> String {
    // Round to the nearest standard width — anything smaller than 8
    // becomes 8 (C requires int8_t etc.); anything wider than 64 falls
    // back to `int64_t` with an annotation. The Source IR will refuse
    // to express > 64-bit integers anyway for B2.8.
    let normalised = match width_bits {
        0..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        _ => 64,
    };
    if signed {
        format!("int{normalised}_t")
    } else {
        format!("uint{normalised}_t")
    }
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Var(name) => name.clone(),
        Expr::IntLit { value, signed } => {
            if *signed {
                format!("{value}LL")
            } else {
                let u = *value as u64;
                format!("{u}ULL")
            }
        }
        Expr::Undef => "0 /* undef */".to_string(),
        Expr::Binary { op, lhs, rhs } => {
            format!(
                "({} {} {})",
                render_expr(lhs),
                render_binary_op(*op),
                render_expr(rhs)
            )
        }
        Expr::Unary { op, expr } => {
            format!("{}({})", render_unary_op(*op), render_expr(expr))
        }
        Expr::Load { ty, address } => {
            let ty_ptr = render_type_prefix(&CType::Ptr(Box::new(ty.clone())));
            format!("(*(({ty_ptr})({})))", render_expr(address))
        }
        Expr::Call { target, args } => {
            // Until B3.10 plumbs `dac-recovery::infer_calling_convention`
            // through the C lowering, every recovered function lowers
            // with an empty parameter list (`void f(void)`) while the
            // bridge (B3.8) reads all six SysV AMD64 call-arg
            // registers at every call site. The recovered arity at
            // the call therefore doesn't match the declared
            // signature, *and* modern C interprets empty function-
            // pointer parens `()` as `(void)` under C23 — so a
            // K&R-style cast no longer accepts variadic actuals.
            // Cast every call target through an arity-matched
            // `void (*)(long long, …)` signature so the compiler
            // accepts the call regardless of what the callee
            // ultimately turns out to be. B3.10 collapses the cast
            // back into a typed direct call once the recovered
            // convention reaches the emitter.
            let mut s = String::new();
            let callee = match target.as_ref() {
                Expr::Var(name) => name.clone(),
                Expr::AddrLit(addr) => format!("{addr:#x}"),
                _ => render_expr(target),
            };
            let sig = render_call_target_cast(args.len());
            let _ = write!(&mut s, "(({sig}){callee})");
            s.push('(');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&render_expr(arg));
            }
            s.push(')');
            s
        }
        Expr::AddrLit(addr) => {
            // Bare `AddrLit` outside of a `Call` lowers to the
            // integer literal so an `int64_t` slot can hold it. The
            // `Call` path above synthesises its own cast.
            format!("{addr:#x}")
        }
        Expr::Field { base, field, arrow } => {
            let op = if *arrow { "->" } else { "." };
            format!("{}{op}{field}", render_expr(base))
        }
        Expr::Cast { ty, expr } => {
            format!("(({})({}))", render_type_prefix(ty), render_expr(expr))
        }
        Expr::Opaque(text) => {
            // Compile-safe placeholder. Wrapping in `(int)0` so the
            // expression has a definite type. The lowering pass uses
            // this for SsaOp::Opaque (I-6).
            format!("(/* opaque: {} */ 0)", sanitize_comment(text))
        }
    }
}

/// Build the function-pointer cast used in front of every call
/// target. Until B3.10 plumbs the recovered calling convention
/// through to the emitter, the cast is purely arity-matched: every
/// argument slot is `long long` and the return type is `long long`
/// so the cast value is type-compatible whether the call's result is
/// assigned to an `int64_t` local (`v0 = call(…)`) or discarded
/// (`call(…);`). Returning `void` here would break the assignment
/// case; returning a wider integer is safe in both call sites
/// because the C compiler accepts implicit narrowing/discarding of
/// the result. The arg-list spelling is `(void)` for zero args.
fn render_call_target_cast(argc: usize) -> String {
    if argc == 0 {
        return "long long (*)(void)".to_string();
    }
    let mut s = String::from("long long (*)(");
    for i in 0..argc {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str("long long");
    }
    s.push(')');
    s
}

fn render_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
    }
}

fn render_unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::BitNot => "~",
        UnaryOp::LogicalNot => "!",
    }
}

/// Strip `*/` from a comment body so we never emit a sequence that
/// closes the comment early.
fn sanitize_comment(s: &str) -> String {
    s.replace("*/", "* /")
}

/// Indenting line printer.
struct Printer<'a> {
    out: &'a mut String,
    depth: usize,
}

impl<'a> Printer<'a> {
    fn new(out: &'a mut String) -> Self {
        Self { out, depth: 0 }
    }

    fn indent(&mut self) {
        self.depth += 1;
    }

    fn dedent(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn write_line(&mut self, line: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn write_label(&mut self, id: u32) {
        // Labels render at column 0 (gcc accepts them anywhere, but
        // C requires a statement after the label — the emitter
        // currently expects the lowering pass to place a statement
        // right after; if the label is the last statement in the
        // block, the dead `;` makes it well-formed).
        self.out.push_str(&format!("L{id}:;\n"));
    }

    fn line(&mut self, raw: &str) {
        // Bypass indenting — used for include directives.
        self.out.push_str(raw);
        self.out.push('\n');
    }

    fn blank(&mut self) {
        self.out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(value: i64) -> Expr {
        Expr::IntLit {
            value,
            signed: true,
        }
    }

    #[test]
    fn empty_translation_unit_renders_blank() {
        let u = TranslationUnit::default();
        assert_eq!(emit(&u), "");
    }

    #[test]
    fn includes_then_blank_line_then_items() {
        let f = Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block::empty(),
            leading_comment: None,
        };
        let u = TranslationUnit {
            includes: vec!["#include <stdint.h>".into()],
            items: vec![Item::Function(f)],
        };
        let s = emit(&u);
        assert_eq!(s, "#include <stdint.h>\n\nvoid f(void) {\n}\n");
    }

    #[test]
    fn function_with_void_return_and_no_params_uses_void() {
        let f = Function {
            name: "main".into(),
            return_type: CType::Int {
                width_bits: 32,
                signed: true,
            },
            params: vec![],
            locals: vec![],
            body: Block {
                stmts: vec![Stmt::Return(Some(lit(0)))],
            },
            leading_comment: None,
        };
        let s = emit_function(&f);
        assert_eq!(s, "int32_t main(void) {\n    return 0LL;\n}\n");
    }

    #[test]
    fn if_else_renders_kr_style() {
        let f = Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block {
                stmts: vec![Stmt::If {
                    cond: Expr::Var("c".into()),
                    then_body: Block {
                        stmts: vec![Stmt::Return(None)],
                    },
                    else_body: Some(Block {
                        stmts: vec![Stmt::Break],
                    }),
                }],
            },
            leading_comment: None,
        };
        let s = emit_function(&f);
        let want = "\
void f(void) {
    if (c) {
        return;
    } else {
        break;
    }
}
";
        assert_eq!(s, want);
    }

    #[test]
    fn endless_loop_emits_while_one() {
        let f = Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block {
                stmts: vec![Stmt::Loop {
                    body: Block {
                        stmts: vec![Stmt::Break],
                    },
                }],
            },
            leading_comment: None,
        };
        let s = emit_function(&f);
        let want = "\
void f(void) {
    while (1) {
        break;
    }
}
";
        assert_eq!(s, want);
    }

    #[test]
    fn label_renders_with_trailing_semicolon() {
        // `L0:` alone is not a statement; the emitter appends `;` so
        // the label is followed by an empty statement and the result
        // compiles.
        let f = Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block {
                stmts: vec![Stmt::Label(0), Stmt::Return(None)],
            },
            leading_comment: None,
        };
        let s = emit_function(&f);
        assert!(s.contains("L0:;\n"));
        assert!(s.contains("    return;\n"));
    }

    #[test]
    fn binary_expression_parenthesises_both_children() {
        let e = Expr::Binary {
            op: BinaryOp::Add,
            lhs: Box::new(Expr::Var("a".into())),
            rhs: Box::new(Expr::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(Expr::Var("b".into())),
                rhs: Box::new(Expr::Var("c".into())),
            }),
        };
        assert_eq!(render_expr(&e), "(a + (b * c))");
    }

    #[test]
    fn int_type_normalises_to_standard_widths() {
        assert_eq!(render_int_type(1, true), "int8_t");
        assert_eq!(render_int_type(8, false), "uint8_t");
        assert_eq!(render_int_type(12, true), "int16_t");
        assert_eq!(render_int_type(32, true), "int32_t");
        assert_eq!(render_int_type(48, true), "int64_t");
        assert_eq!(render_int_type(64, false), "uint64_t");
        assert_eq!(render_int_type(128, true), "int64_t");
    }

    #[test]
    fn load_expression_casts_address_to_pointer() {
        let e = Expr::Load {
            ty: CType::Int {
                width_bits: 32,
                signed: false,
            },
            address: Box::new(Expr::Var("addr".into())),
        };
        assert_eq!(render_expr(&e), "(*((uint32_t *)(addr)))");
    }

    #[test]
    fn opaque_expression_sanitizes_close_comment() {
        let e = Expr::Opaque("foo */ bar".into());
        // The "*/" must not appear in the comment body.
        let s = render_expr(&e);
        assert!(!s.contains("*/ bar"));
        assert!(s.contains("foo * / bar"));
    }

    #[test]
    fn leading_comment_renders_above_signature() {
        let f = Function {
            name: "g".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![],
            body: Block::empty(),
            leading_comment: Some("provenance: 0x1234\nstructured".into()),
        };
        let s = emit_function(&f);
        assert_eq!(
            s,
            "\
/* provenance: 0x1234 */
/* structured */
void g(void) {
}
"
        );
    }

    #[test]
    fn locals_render_above_body_separated_by_blank() {
        let f = Function {
            name: "f".into(),
            return_type: CType::Void,
            params: vec![],
            locals: vec![Local {
                name: "v0".into(),
                ty: CType::i64(),
                init: Some(lit(0)),
            }],
            body: Block {
                stmts: vec![Stmt::Return(None)],
            },
            leading_comment: None,
        };
        let s = emit_function(&f);
        let want = "\
void f(void) {
    int64_t v0 = 0LL;

    return;
}
";
        assert_eq!(s, want);
    }
}
