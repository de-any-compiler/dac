//! C++ AST → formatted source string.
//!
//! The emitter is hand-rolled, deterministic, and aware of just enough
//! C++ syntax to keep the round-trip compile gate green:
//!
//! - `class <Name>[ : public <Base>, …] { … };`
//! - `public:` / `protected:` / `private:` blocks.
//! - In-class member function definitions with `virtual` / `const`
//!   keywords and stub bodies.
//! - Free function definitions with a stub body returning a default-
//!   initialised value of the return type for non-`void` returns.
//! - Pointer / reference / const type spellings.
//!
//! Things that intentionally never appear in the output:
//!
//! - `namespace` blocks. Scope chains are flattened into the class
//!   leading comment until B3.6's signature recovery can ground them.
//! - `template`, `enum class`, `using` declarations, exception specs.
//!   None of them are needed by the symbol-driven class recovery
//!   B3.5 ships, so they would only be unused vocabulary.
//!
//! ## Determinism
//!
//! Pure function from [`TranslationUnit`] to `String`. Iteration order
//! follows the AST's `Vec<…>` slots and never touches a hashed
//! container.

use std::fmt::Write as _;

use crate::ast::{
    AccessSpec, BaseSpec, Class, CppType, FreeFunction, Item, MemberFunction, MemberFunctionKind,
    Param, TranslationUnit,
};

/// Render a [`TranslationUnit`] as formatted C++ source.
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

/// Render a single class to a string. Convenience for unit tests that
/// build a class in isolation.
#[must_use]
pub fn emit_class(c: &Class) -> String {
    let mut out = String::new();
    emit_item(&mut Printer::new(&mut out), &Item::Class(c.clone()));
    out
}

/// Render a single free function to a string. Convenience for tests.
#[must_use]
pub fn emit_free_function(f: &FreeFunction) -> String {
    let mut out = String::new();
    emit_item(&mut Printer::new(&mut out), &Item::FreeFunction(f.clone()));
    out
}

fn emit_item(p: &mut Printer<'_>, item: &Item) {
    match item {
        Item::Class(c) => emit_class_into(p, c),
        Item::FreeFunction(f) => emit_free_function_into(p, f),
    }
}

fn emit_class_into(p: &mut Printer<'_>, c: &Class) {
    if let Some(comment) = &c.leading_comment {
        for line in comment.lines() {
            p.write_line(&format!("// {line}"));
        }
    }
    let mut header = format!("class {}", c.name);
    if !c.bases.is_empty() {
        header.push_str(" : ");
        for (i, b) in c.bases.iter().enumerate() {
            if i > 0 {
                header.push_str(", ");
            }
            header.push_str(&render_base(b));
        }
    }
    header.push_str(" {");
    p.write_line(&header);
    p.write_line("public:");
    p.indent();
    for (i, m) in c.members.iter().enumerate() {
        if i > 0 {
            p.blank();
        }
        emit_member(p, &c.name, m);
    }
    p.dedent();
    p.write_line("};");
}

fn render_base(b: &BaseSpec) -> String {
    let access = match b.access {
        AccessSpec::Public => "public",
        AccessSpec::Protected => "protected",
        AccessSpec::Private => "private",
    };
    format!("{access} {}", b.qualified_name)
}

fn emit_member(p: &mut Printer<'_>, class_name: &str, m: &MemberFunction) {
    if let Some(comment) = &m.leading_comment {
        for line in comment.lines() {
            p.write_line(&format!("// {line}"));
        }
    }
    // Signature.
    let mut sig = String::new();
    if m.is_virtual {
        sig.push_str("virtual ");
    }
    match m.kind {
        MemberFunctionKind::Method => {
            sig.push_str(&render_type(&m.return_type));
            sig.push(' ');
            sig.push_str(&m.name);
        }
        MemberFunctionKind::Constructor => {
            sig.push_str(class_name);
        }
        MemberFunctionKind::Destructor => {
            sig.push('~');
            sig.push_str(class_name);
        }
    }
    sig.push('(');
    sig.push_str(&render_params(&m.params));
    sig.push(')');
    if m.is_const {
        sig.push_str(" const");
    }
    sig.push_str(" {");
    p.write_line(&sig);
    p.indent();
    p.write_line("// dac C++ stub: lifter→SSA bridge pending; body intentionally empty");
    if matches!(m.kind, MemberFunctionKind::Method) && !matches!(m.return_type, CppType::Void) {
        p.write_line(&format!("return {}{{}};", render_type(&m.return_type)));
    }
    p.dedent();
    p.write_line("}");
}

fn emit_free_function_into(p: &mut Printer<'_>, f: &FreeFunction) {
    if let Some(comment) = &f.leading_comment {
        for line in comment.lines() {
            p.write_line(&format!("// {line}"));
        }
    }
    let mut sig = String::new();
    sig.push_str(&render_type(&f.return_type));
    sig.push(' ');
    sig.push_str(&f.name);
    sig.push('(');
    sig.push_str(&render_params(&f.params));
    sig.push_str(") {");
    p.write_line(&sig);
    p.indent();
    p.write_line("// dac C++ stub: lifter→SSA bridge pending; body intentionally empty");
    if !matches!(f.return_type, CppType::Void) {
        p.write_line(&format!("return {}{{}};", render_type(&f.return_type)));
    }
    p.dedent();
    p.write_line("}");
}

fn render_params(params: &[Param]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(&mut s, "{} {}", render_type(&param.ty), param.name);
    }
    s
}

fn render_type(ty: &CppType) -> String {
    match ty {
        CppType::Void => "void".to_string(),
        CppType::Int { width_bits, signed } => render_int_type(*width_bits, *signed),
        CppType::Ptr(inner) => format!("{}*", render_type(inner)),
        CppType::Ref(inner) => format!("{}&", render_type(inner)),
        CppType::Const(inner) => format!("{} const", render_type(inner)),
        CppType::Class { qualified_name } => qualified_name.clone(),
    }
}

fn render_int_type(width_bits: u16, signed: bool) -> String {
    let normalised = match width_bits {
        0..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        _ => 64,
    };
    if signed {
        format!("std::int{normalised}_t")
    } else {
        format!("std::uint{normalised}_t")
    }
}

/// Indenting line printer — same shape as
/// [`dac_backend_c::emit`]'s.
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

    fn line(&mut self, raw: &str) {
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
    use crate::ast::{
        AccessSpec, BaseSpec, Class, CppType, FreeFunction, MemberFunction, MemberFunctionKind,
        TranslationUnit,
    };

    fn animal_class() -> Class {
        Class {
            name: "Animal".into(),
            scope_chain: Vec::new(),
            bases: Vec::new(),
            has_vtable: true,
            members: vec![
                MemberFunction {
                    name: "Animal".into(),
                    return_type: CppType::Void,
                    params: Vec::new(),
                    kind: MemberFunctionKind::Destructor,
                    is_const: false,
                    is_virtual: true,
                    leading_comment: None,
                },
                MemberFunction {
                    name: "speak".into(),
                    return_type: CppType::int(),
                    params: Vec::new(),
                    kind: MemberFunctionKind::Method,
                    is_const: true,
                    is_virtual: true,
                    leading_comment: None,
                },
            ],
            leading_comment: None,
        }
    }

    #[test]
    fn empty_unit_renders_blank() {
        let u = TranslationUnit::default();
        assert_eq!(emit(&u), "");
    }

    #[test]
    fn includes_then_blank_then_items() {
        let u = TranslationUnit {
            includes: vec!["#include <cstdint>".into()],
            items: vec![Item::FreeFunction(FreeFunction {
                name: "f".into(),
                return_type: CppType::Void,
                params: Vec::new(),
                leading_comment: None,
            })],
        };
        let s = emit(&u);
        assert!(s.starts_with("#include <cstdint>\n\nvoid f() {\n"));
    }

    #[test]
    fn class_renders_with_virtual_dtor_and_const_method() {
        let s = emit_class(&animal_class());
        let want = "\
class Animal {
public:
    virtual ~Animal() {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
    }

    virtual std::int32_t speak() const {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
        return std::int32_t{};
    }
};
";
        assert_eq!(s, want);
    }

    #[test]
    fn class_with_public_base_renders_inheritance_clause() {
        let mut c = animal_class();
        c.name = "Dog".into();
        c.bases.push(BaseSpec {
            access: AccessSpec::Public,
            qualified_name: "Animal".into(),
        });
        c.members[0].name = "Dog".into();
        let s = emit_class(&c);
        assert!(s.contains("class Dog : public Animal {"));
    }

    #[test]
    fn constructor_emits_class_name_and_no_return_type() {
        let c = Class {
            name: "Dog".into(),
            scope_chain: Vec::new(),
            bases: Vec::new(),
            has_vtable: false,
            members: vec![MemberFunction {
                name: "Dog".into(),
                return_type: CppType::Void,
                params: Vec::new(),
                kind: MemberFunctionKind::Constructor,
                is_const: false,
                is_virtual: false,
                leading_comment: None,
            }],
            leading_comment: None,
        };
        let s = emit_class(&c);
        assert!(s.contains("    Dog() {\n"));
        assert!(!s.contains("void Dog()"));
    }

    #[test]
    fn free_function_with_int_return_emits_return_stub() {
        let f = FreeFunction {
            name: "main".into(),
            return_type: CppType::int(),
            params: Vec::new(),
            leading_comment: None,
        };
        let s = emit_free_function(&f);
        let want = "\
std::int32_t main() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
    return std::int32_t{};
}
";
        assert_eq!(s, want);
    }

    #[test]
    fn void_free_function_omits_return_statement() {
        let f = FreeFunction {
            name: "frame_dummy".into(),
            return_type: CppType::Void,
            params: Vec::new(),
            leading_comment: None,
        };
        let s = emit_free_function(&f);
        assert!(!s.contains("return"));
    }

    #[test]
    fn ptr_const_ref_render_in_canonical_form() {
        // const T*.
        assert_eq!(
            render_type(&CppType::Ptr(Box::new(CppType::Const(Box::new(
                CppType::Class {
                    qualified_name: "Animal".into()
                }
            ))))),
            "Animal const*"
        );
        // T&.
        assert_eq!(
            render_type(&CppType::Ref(Box::new(CppType::int()))),
            "std::int32_t&"
        );
    }

    #[test]
    fn int_type_normalises_to_standard_widths() {
        assert_eq!(render_int_type(1, true), "std::int8_t");
        assert_eq!(render_int_type(8, false), "std::uint8_t");
        assert_eq!(render_int_type(12, true), "std::int16_t");
        assert_eq!(render_int_type(32, true), "std::int32_t");
        assert_eq!(render_int_type(48, true), "std::int64_t");
        assert_eq!(render_int_type(64, false), "std::uint64_t");
        assert_eq!(render_int_type(128, true), "std::int64_t");
    }

    #[test]
    fn leading_comment_renders_with_double_slash() {
        let mut c = animal_class();
        c.leading_comment = Some("provenance: x\nvtable: y".into());
        let s = emit_class(&c);
        assert!(s.starts_with("// provenance: x\n// vtable: y\nclass Animal {"));
    }
}
