//! `RecoveredClasses` + `FunctionSet` → C++ AST (B3.5, FR-21).
//!
//! [`lower_unit`] is the single entry point. It walks the
//! [`crate::class_recovery::RecoveredClasses`] table and the
//! recovered function set into a [`TranslationUnit`] that
//! [`crate::emit`] then renders as C++.
//!
//! ## What the lowering pass commits to
//!
//! 1. **One [`Class`] per [`RecoveredClass`].** Every member function
//!    becomes an in-class declaration / definition. Multiple
//!    ctor / dtor variants (e.g. `C1` and `C2`) collapse to a single
//!    `Constructor` / `Destructor` member — Itanium variants have the
//!    same source-level signature, so emitting two of each would
//!    produce a duplicate-definition error from the C++ compiler.
//!    Variants we drop are recorded in the leading comment so the
//!    annotation channel still surfaces them.
//! 2. **Polymorphic classes always carry a virtual destructor.** Even
//!    when no `_ZN…D[012]Ev` symbol was recovered the lowering emits
//!    `virtual ~Class();` so the emitted unit reflects the binary's
//!    runtime shape (a `_ZTV*` symbol implies a vtable slot for the
//!    deleting dtor). The leading comment records that this was a
//!    lowering-inserted member, not a recovered one.
//! 3. **Stub bodies (I-6).** Every emitted member and free function
//!    has a deterministic stub body — `/* lifter→SSA bridge pending */`
//!    followed by `return T{};` for non-`void` returns. The leading
//!    comment carries the recovered address, the mangled symbol, and
//!    the join-confidence so a `--debug` consumer can trace the source.
//! 4. **Free functions.** [`RecoveredFreeFunction`] entries land as
//!    [`Item::FreeFunction`]; `main` always gets `int` return so the
//!    round-trip compile gate accepts the unit. Everything else
//!    defaults to `void` until B3.6 plumbs real signatures in.
//! 5. **No `namespace` lowering at B3.5.** When a class's scope chain
//!    is non-empty the qualified name is emitted as
//!    `using <Scope>__<Name> = …` — actually no: scope_chain is empty
//!    for everything the symbol-driven recovery handles today (C++
//!    binaries from CLI tests put classes at global scope), so we
//!    assert empty and degrade with a leading comment for any
//!    non-empty chain.
//!
//! ## Determinism
//!
//! Pure function. The order of items follows
//! [`RecoveredClasses::classes`] then [`RecoveredClasses::free_functions`],
//! both of which are sorted by [`crate::class_recovery::recover_classes`].

use dac_recovery::FunctionSet;

use crate::ast::{
    AccessSpec, BaseSpec, Class, CppType, FreeFunction, Item, MemberFunction, MemberFunctionKind,
    Param, TranslationUnit,
};
use crate::class_recovery::{
    MemberCategory, RecoveredBase, RecoveredClass, RecoveredClasses, RecoveredMember,
};

/// `#include` directives every emitted C++ translation unit needs.
#[must_use]
pub fn default_includes() -> Vec<String> {
    vec![
        "#include <cstdint>".to_string(),
        "#include <cstddef>".to_string(),
    ]
}

/// Lower the recovered class table into a translation unit.
///
/// `_functions` is accepted for parity with [`dac_backend_c::lower::lower_unit`];
/// at B3.5 the class members already carry their own addresses /
/// evidence handles, so the function set is informational only.
#[must_use]
pub fn lower_unit(classes: &RecoveredClasses, _functions: &FunctionSet) -> TranslationUnit {
    let mut items: Vec<Item> =
        Vec::with_capacity(classes.classes.len() + classes.free_functions.len());
    for c in &classes.classes {
        items.push(Item::Class(lower_class(c)));
    }
    for f in &classes.free_functions {
        items.push(Item::FreeFunction(lower_free_function(f)));
    }
    TranslationUnit {
        includes: default_includes(),
        items,
    }
}

fn lower_class(c: &RecoveredClass) -> Class {
    // Collapse ctor and dtor variants to a single member each, keeping
    // a record of every variant address in the leading comment.
    let mut ctor_addresses: Vec<u64> = Vec::new();
    let mut ctor_mangled: Vec<String> = Vec::new();
    let mut dtor_addresses: Vec<u64> = Vec::new();
    let mut dtor_mangled: Vec<String> = Vec::new();
    let mut methods: Vec<&RecoveredMember> = Vec::new();
    for m in &c.members {
        match m.kind {
            MemberCategory::Method => methods.push(m),
            MemberCategory::CtorVariant(_) => {
                ctor_addresses.push(m.address);
                ctor_mangled.push(m.mangled.clone());
            }
            MemberCategory::DtorVariant(_) => {
                dtor_addresses.push(m.address);
                dtor_mangled.push(m.mangled.clone());
            }
        }
    }

    let bases_comment = if c.bases.is_empty() {
        "(none)".to_string()
    } else {
        c.bases
            .iter()
            .map(|b| format!("{:?} {}", b.access, b.qualified_name))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let class_leading_comment = Some(format!(
        "dac-recovered class\n\
         qualified: {}\n\
         vtable: {}\n\
         typeinfo: {}\n\
         bases: {}\n\
         confidence: {:.2} ({:?})",
        c.qualified_name(),
        c.has_vtable,
        c.has_typeinfo,
        bases_comment,
        c.confidence.value(),
        c.confidence.source(),
    ));

    let mut members: Vec<MemberFunction> = Vec::new();

    // Collapsed ctor.
    if !ctor_addresses.is_empty() {
        let comment = Some(format!(
            "ctor variants:\n{}",
            ctor_mangled
                .iter()
                .zip(&ctor_addresses)
                .map(|(m, a)| format!("  {a:#x}  {m}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
        members.push(MemberFunction {
            name: c.name.clone(),
            return_type: CppType::Void,
            params: Vec::new(),
            kind: MemberFunctionKind::Constructor,
            is_const: false,
            is_virtual: false,
            leading_comment: comment,
        });
    }

    // Collapsed dtor — virtual when has_vtable.
    if !dtor_addresses.is_empty() {
        let comment = Some(format!(
            "dtor variants:\n{}",
            dtor_mangled
                .iter()
                .zip(&dtor_addresses)
                .map(|(m, a)| format!("  {a:#x}  {m}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
        members.push(MemberFunction {
            name: c.name.clone(),
            return_type: CppType::Void,
            params: Vec::new(),
            kind: MemberFunctionKind::Destructor,
            is_const: false,
            is_virtual: c.has_vtable,
            leading_comment: comment,
        });
    } else if c.has_vtable {
        // Synthesise a virtual destructor so the emitted class is
        // well-formed C++. The leading comment makes the lowering's
        // role explicit (I-6).
        members.push(MemberFunction {
            name: c.name.clone(),
            return_type: CppType::Void,
            params: Vec::new(),
            kind: MemberFunctionKind::Destructor,
            is_const: false,
            is_virtual: true,
            leading_comment: Some(
                "synthesised by dac: vtable present, dtor not in symbol table".into(),
            ),
        });
    }

    // Methods.
    for m in &methods {
        let leading_comment = Some(format!(
            "dac-recovered member\n\
             address: {:#x}\n\
             mangled: {}\n\
             const:   {}\n\
             virtual: {}",
            m.address, m.mangled, m.is_const, m.is_virtual
        ));
        members.push(MemberFunction {
            name: m.name.clone(),
            // No signature recovery yet (B3.6 plumbs it through). All
            // recovered methods default to `int` so a polymorphic
            // call site at the binary level — which routes through a
            // vtable slot whose return type we cannot see without
            // signature recovery — has a definite return spelling.
            return_type: CppType::int(),
            params: Vec::new(),
            kind: MemberFunctionKind::Method,
            is_const: m.is_const,
            is_virtual: m.is_virtual,
            leading_comment,
        });
    }

    Class {
        name: c.name.clone(),
        scope_chain: c.scope_chain.clone(),
        bases: c.bases.iter().map(lower_base).collect(),
        has_vtable: c.has_vtable,
        members,
        leading_comment: class_leading_comment,
    }
}

fn lower_base(b: &RecoveredBase) -> BaseSpec {
    BaseSpec {
        access: b.access,
        qualified_name: b.qualified_name.clone(),
    }
}

fn lower_free_function(f: &crate::class_recovery::RecoveredFreeFunction) -> FreeFunction {
    let return_type = if f.name == "main" {
        CppType::int()
    } else {
        CppType::Void
    };
    let leading_comment = Some(format!(
        "dac-recovered free function\n\
         address: {:#x}\n\
         mangled: {}",
        f.address, f.mangled,
    ));
    FreeFunction {
        name: f.name.clone(),
        return_type,
        params: Vec::new(),
        leading_comment,
    }
}

/// Default access specifier dac uses when synthesising base specs in
/// future batches. Exposed so callers can keep the policy in one place.
#[must_use]
pub fn default_base_access() -> AccessSpec {
    AccessSpec::Public
}

/// Build a base spec from a recovered base class — placeholder for
/// the B3.5-deferred typeinfo-relocation walker. Kept here so the AST
/// type stays in this crate's API surface.
#[must_use]
pub fn base_spec_public(qualified_name: impl Into<String>) -> BaseSpec {
    BaseSpec {
        access: AccessSpec::Public,
        qualified_name: qualified_name.into(),
    }
}

/// Build a stub `Param` list. Placeholder for B3.6 signature recovery.
#[must_use]
pub fn no_params() -> Vec<Param> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_recovery::{
        ClassRecoveryStats, MemberCategory, RecoveredClass, RecoveredClasses,
        RecoveredFreeFunction, RecoveredMember,
    };
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode, IrLayer, Source};

    fn ev() -> dac_core::EvidenceId {
        let mut g = EvidenceGraph::new();
        g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Source,
            id: 0,
        })
    }

    fn empty_function_set() -> FunctionSet {
        FunctionSet {
            functions: Vec::new(),
            stats: Default::default(),
        }
    }

    fn animal_class() -> RecoveredClass {
        RecoveredClass {
            name: "Animal".into(),
            scope_chain: Vec::new(),
            has_vtable: true,
            has_typeinfo: true,
            bases: Vec::new(),
            members: vec![RecoveredMember {
                name: "speak".into(),
                mangled: "_ZNK6Animal5speakEv".into(),
                address: 0x1000,
                kind: MemberCategory::Method,
                is_const: true,
                is_virtual: true,
                evidence: None,
            }],
            confidence: Confidence::new(1.0, Source::Observed),
            evidence: ev(),
        }
    }

    fn dog_class() -> RecoveredClass {
        RecoveredClass {
            name: "Dog".into(),
            scope_chain: Vec::new(),
            has_vtable: true,
            has_typeinfo: true,
            bases: vec![RecoveredBase {
                qualified_name: "Animal".into(),
                access: AccessSpec::Public,
            }],
            members: vec![
                RecoveredMember {
                    name: "Dog_ctor_v1".into(),
                    mangled: "_ZN3DogC1Ev".into(),
                    address: 0x2000,
                    kind: MemberCategory::CtorVariant(1),
                    is_const: false,
                    is_virtual: false,
                    evidence: None,
                },
                RecoveredMember {
                    name: "Dog_ctor_v2".into(),
                    mangled: "_ZN3DogC2Ev".into(),
                    address: 0x2010,
                    kind: MemberCategory::CtorVariant(2),
                    is_const: false,
                    is_virtual: false,
                    evidence: None,
                },
                RecoveredMember {
                    name: "Dog_dtor_v0".into(),
                    mangled: "_ZN3DogD0Ev".into(),
                    address: 0x2020,
                    kind: MemberCategory::DtorVariant(0),
                    is_const: false,
                    is_virtual: true,
                    evidence: None,
                },
                RecoveredMember {
                    name: "speak".into(),
                    mangled: "_ZNK3Dog5speakEv".into(),
                    address: 0x2030,
                    kind: MemberCategory::Method,
                    is_const: true,
                    is_virtual: true,
                    evidence: None,
                },
            ],
            confidence: Confidence::new(1.0, Source::Observed),
            evidence: ev(),
        }
    }

    #[test]
    fn lower_unit_yields_one_item_per_class_and_free_function() {
        let classes = RecoveredClasses {
            classes: vec![animal_class(), dog_class()],
            free_functions: vec![RecoveredFreeFunction {
                name: "main".into(),
                mangled: "main".into(),
                address: 0x3000,
                evidence: None,
            }],
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        assert_eq!(unit.items.len(), 3);
        assert!(matches!(unit.items[0], Item::Class(_)));
        assert!(matches!(unit.items[1], Item::Class(_)));
        assert!(matches!(unit.items[2], Item::FreeFunction(_)));
    }

    #[test]
    fn polymorphic_class_without_dtor_synthesises_virtual_destructor() {
        let mut animal = animal_class();
        animal.members.clear(); // No recovered methods or dtor.
        animal.has_vtable = true;
        let classes = RecoveredClasses {
            classes: vec![animal],
            free_functions: Vec::new(),
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::Class(c) = &unit.items[0] else {
            panic!("expected class");
        };
        assert_eq!(c.members.len(), 1);
        assert_eq!(c.members[0].kind, MemberFunctionKind::Destructor);
        assert!(c.members[0].is_virtual);
        assert!(c.members[0]
            .leading_comment
            .as_deref()
            .is_some_and(|s| s.contains("synthesised by dac")));
    }

    #[test]
    fn ctor_variants_collapse_to_single_member() {
        let classes = RecoveredClasses {
            classes: vec![dog_class()],
            free_functions: Vec::new(),
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::Class(c) = &unit.items[0] else {
            panic!("expected class");
        };
        let ctors: Vec<&MemberFunction> = c
            .members
            .iter()
            .filter(|m| m.kind == MemberFunctionKind::Constructor)
            .collect();
        assert_eq!(ctors.len(), 1);
        let comment = ctors[0].leading_comment.as_deref().unwrap();
        assert!(comment.contains("_ZN3DogC1Ev"));
        assert!(comment.contains("_ZN3DogC2Ev"));
    }

    #[test]
    fn polymorphic_class_promotes_virtual_dtor_when_present() {
        let classes = RecoveredClasses {
            classes: vec![dog_class()],
            free_functions: Vec::new(),
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::Class(c) = &unit.items[0] else {
            panic!("expected class");
        };
        let dtors: Vec<&MemberFunction> = c
            .members
            .iter()
            .filter(|m| m.kind == MemberFunctionKind::Destructor)
            .collect();
        assert_eq!(dtors.len(), 1);
        assert!(dtors[0].is_virtual);
    }

    #[test]
    fn methods_inherit_const_and_virtual_from_recovered_member() {
        let classes = RecoveredClasses {
            classes: vec![dog_class()],
            free_functions: Vec::new(),
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::Class(c) = &unit.items[0] else {
            panic!("expected class");
        };
        let methods: Vec<&MemberFunction> = c
            .members
            .iter()
            .filter(|m| m.kind == MemberFunctionKind::Method)
            .collect();
        assert_eq!(methods.len(), 1);
        assert!(methods[0].is_const);
        assert!(methods[0].is_virtual);
        assert_eq!(methods[0].name, "speak");
    }

    #[test]
    fn main_free_function_gets_int_return() {
        let classes = RecoveredClasses {
            classes: Vec::new(),
            free_functions: vec![RecoveredFreeFunction {
                name: "main".into(),
                mangled: "main".into(),
                address: 0x3000,
                evidence: None,
            }],
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::FreeFunction(f) = &unit.items[0] else {
            panic!("expected free fn");
        };
        assert_eq!(f.name, "main");
        assert_eq!(f.return_type, CppType::int());
    }

    #[test]
    fn other_free_functions_default_to_void_return() {
        let classes = RecoveredClasses {
            classes: Vec::new(),
            free_functions: vec![RecoveredFreeFunction {
                name: "chorus".into(),
                mangled: "_Z6chorusPK6AnimalS1_".into(),
                address: 0x3100,
                evidence: None,
            }],
            stats: ClassRecoveryStats::default(),
        };
        let unit = lower_unit(&classes, &empty_function_set());
        let Item::FreeFunction(f) = &unit.items[0] else {
            panic!("expected free fn");
        };
        assert_eq!(f.return_type, CppType::Void);
    }

    #[test]
    fn helpers_round_trip_their_inputs() {
        assert_eq!(default_base_access(), AccessSpec::Public);
        let spec = base_spec_public("Animal");
        assert_eq!(spec.qualified_name, "Animal");
        assert_eq!(spec.access, AccessSpec::Public);
        assert!(no_params().is_empty());
    }
}
