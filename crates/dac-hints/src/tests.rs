//! Unit tests for the hint-file parser + the IR-lowering.

use dac_ir::ty::{IntType, Signedness, Type as IrType};

use super::*;

#[test]
fn parse_address_only_function_hint_with_return_override() {
    let src = r#"
[[function]]
address = "0x1040"
return = "int"
"#;
    let hints = parse_toml(src).expect("parse");
    assert_eq!(hints.functions.len(), 1);
    let h = &hints.functions[0];
    assert_eq!(h.matcher, HintMatcher::Address(0x1040));
    assert_eq!(h.rename, None);
    assert_eq!(
        h.return_ty,
        Some(HintType::Int {
            width_bits: 32,
            signed: Some(true)
        })
    );
    assert!(h.args.is_none());
}

#[test]
fn parse_name_only_function_hint_with_args() {
    let src = r#"
[[function]]
name = "main"
args = ["int", "char**"]
"#;
    let h = &parse_toml(src).expect("parse").functions[0];
    assert_eq!(h.matcher, HintMatcher::Name("main".into()));
    let args = h.args.as_ref().expect("args present");
    assert_eq!(
        args[0],
        HintType::Int {
            width_bits: 32,
            signed: Some(true)
        }
    );
    assert_eq!(
        args[1],
        HintType::Ptr(Box::new(HintType::Ptr(Box::new(HintType::Int {
            width_bits: 8,
            signed: Some(true)
        }))))
    );
}

#[test]
fn both_matchers_require_both_to_match() {
    let src = r#"
[[function]]
address = "0x1040"
name = "main"
rename = "user_main"
"#;
    let h = &parse_toml(src).expect("parse").functions[0];
    assert!(h.matcher.matches(0x1040, Some("main")));
    assert!(!h.matcher.matches(0x1040, Some("other")));
    assert!(!h.matcher.matches(0x9999, Some("main")));
    assert!(!h.matcher.matches(0x1040, None));
}

#[test]
fn rejects_function_hint_with_no_effect() {
    let src = r#"
[[function]]
address = "0x1040"
"#;
    let err = parse_toml(src).expect_err("must reject");
    assert!(matches!(err, HintError::Semantic { .. }));
    assert!(err.message().contains("no effect"));
}

#[test]
fn rejects_function_hint_with_no_matcher() {
    let src = r#"
[[function]]
return = "int"
"#;
    let err = parse_toml(src).expect_err("must reject");
    assert!(err.message().contains("requires `address` or `name`"));
}

#[test]
fn rejects_void_argument() {
    let src = r#"
[[function]]
name = "main"
args = ["void"]
"#;
    let err = parse_toml(src).expect_err("must reject");
    assert!(err
        .message()
        .contains("`void` is not a valid argument type"));
}

#[test]
fn parses_struct_hint_with_named_fields() {
    let src = r#"
[[struct]]
name = "Point"
fields = [
    { name = "x", offset = "0x0", ty = "int32" },
    { name = "y", offset = 4, ty = "int32" },
]
"#;
    let hints = parse_toml(src).expect("parse");
    assert_eq!(hints.structs.len(), 1);
    let s = &hints.structs[0];
    assert_eq!(s.name, "Point");
    assert_eq!(s.fields.len(), 2);
    assert_eq!(s.fields[0].name, "x");
    assert_eq!(s.fields[0].offset, 0);
    assert_eq!(s.fields[1].offset, 4);
}

#[test]
fn hint_ids_are_assigned_in_source_order() {
    let src = r#"
[[function]]
name = "a"
return = "int"

[[function]]
name = "b"
return = "int"

[[struct]]
name = "S"
fields = [ { name = "f", offset = 0, ty = "int32" } ]
"#;
    let hints = parse_toml(src).expect("parse");
    assert_eq!(hints.functions[0].id, 1);
    assert_eq!(hints.functions[1].id, 2);
    assert_eq!(hints.structs[0].id, 3);
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let src = "
# top-level comment
[[function]] # trailing comment
name = \"main\"  # inline
return = \"int\"

# more comments
";
    let hints = parse_toml(src).expect("parse");
    assert_eq!(hints.functions.len(), 1);
}

#[test]
fn hint_type_lowers_to_ir_type() {
    let t = HintType::Int {
        width_bits: 32,
        signed: Some(true),
    };
    assert_eq!(
        t.to_ir(),
        IrType::Int(IntType {
            width_bits: 32,
            signedness: Signedness::Signed
        })
    );
    let ptr = HintType::Ptr(Box::new(HintType::Int {
        width_bits: 8,
        signed: Some(true),
    }));
    assert_eq!(
        ptr.to_ir(),
        IrType::Ptr(Box::new(IrType::Int(IntType {
            width_bits: 8,
            signedness: Signedness::Signed
        })))
    );
}

#[test]
fn type_parser_handles_pointer_chains_and_whitespace() {
    let t = HintType::parse(" int  **").expect("parse");
    assert_eq!(
        t,
        HintType::Ptr(Box::new(HintType::Ptr(Box::new(HintType::Int {
            width_bits: 32,
            signed: Some(true)
        }))))
    );
}

#[test]
fn type_parser_rejects_unknown_atom() {
    let err = HintType::parse("frob").expect_err("must reject");
    assert!(err.message().contains("unknown type atom"));
}

#[test]
fn parser_rejects_single_table_header() {
    let src = "[function]\nname = \"main\"\n";
    let err = parse_toml(src).expect_err("must reject");
    assert!(err.message().contains("single-table headers"));
}

#[test]
fn find_function_prefers_first_match() {
    let src = r#"
[[function]]
name = "main"
rename = "first"

[[function]]
name = "main"
rename = "second"
"#;
    let hints = parse_toml(src).expect("parse");
    let h = hints.find_function(0x1040, Some("main")).unwrap();
    assert_eq!(h.rename.as_deref(), Some("first"));
}

#[test]
fn parse_round_trip_is_deterministic() {
    let src = r#"
[[function]]
address = "0x1040"
return = "int"
args = ["int", "char**"]
"#;
    let a = parse_toml(src).expect("parse");
    let b = parse_toml(src).expect("parse");
    assert_eq!(a, b);
}
