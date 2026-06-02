//! C++ AST — the intermediate representation produced by
//! [`crate::lower`] and consumed by [`crate::emit`].
//!
//! Mirrors [`dac_backend_c::ast`] in spirit, but the surface is
//! narrower: at B3.5 the backend's job is to materialise the class
//! hierarchy `dac_backend_cpp::class_recovery` discovered on top of
//! the (still-stubbed) function bodies. The AST consequently only
//! needs to describe:
//!
//! - Translation-unit-level `#include` directives and items
//!   ([`TranslationUnit`], [`Item`]).
//! - A C++ `class` with optional public bases, virtuality, and a flat
//!   list of in-class member function declarations
//!   ([`Class`], [`BaseSpec`], [`AccessSpec`], [`MemberFunction`],
//!   [`MemberFunctionKind`]).
//! - Free functions ([`FreeFunction`]) — for `main` and the residual
//!   `_Z<name>` symbols.
//! - A coarse [`CppType`] vocabulary (`void`, fixed-width ints,
//!   pointer, reference, const, and a `Class { qualified_name }`
//!   spelling for backend-recovered class types).
//!
//! Bodies are not modelled here. Every emitted definition is a stub
//! — a `/* dac C++ stub */` comment plus a `return …;` for non-`void`
//! returns — because the lifter → `RawFunction` bridge feeding the
//! structurer is not yet a batch in PLAN.md. The AST consumer takes
//! this on faith and the lowering pass attaches a leading-comment
//! receipt so a reader knows why the bodies are empty (I-6 — degrade
//! visibly, never invent semantics).
//!
//! ## Determinism
//!
//! Every node is a pure data value. `Vec<…>` orderings are the
//! lowering-pass order, which is itself derived from the deterministic
//! ordering of [`crate::class_recovery::RecoveredClasses`]. There is
//! no interior mutability and no hashing; same AST in → same string
//! out.

/// A complete C++ translation unit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranslationUnit {
    /// `#include` directives in source order. The lowering pass
    /// populates the standard set (`<cstdint>`, `<cstddef>`); callers
    /// can prepend a banner comment as the first entry.
    pub includes: Vec<String>,
    /// Top-level items in source order.
    pub items: Vec<Item>,
}

/// One top-level item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A class definition.
    Class(Class),
    /// A free (non-member) function definition.
    FreeFunction(FreeFunction),
}

/// A C++ `class` definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    /// Class name (innermost segment of the qualified name).
    pub name: String,
    /// Outer scope chain — emit wraps the class declaration in
    /// `namespace … { … }` blocks when non-empty.
    pub scope_chain: Vec<String>,
    /// Public base classes recovered from typeinfo relocations.
    /// Empty at B3.5 because we have not yet wired the relocation
    /// reader — the field is in the AST so the lowering pass can
    /// populate it without a schema change.
    pub bases: Vec<BaseSpec>,
    /// `true` when the class carries a vtable. Emit promotes the
    /// dtor to `virtual` automatically when this is set, even if no
    /// dtor was recovered, so the resulting C++ is well-formed.
    pub has_vtable: bool,
    /// Member function declarations, in lowering-pass order.
    pub members: Vec<MemberFunction>,
    /// Optional leading comment rendered above `class Foo { … };`.
    pub leading_comment: Option<String>,
}

/// One base-class specifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseSpec {
    pub access: AccessSpec,
    /// Fully qualified name of the base class.
    pub qualified_name: String,
}

/// Access specifier (`public:` / `protected:` / `private:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSpec {
    Public,
    Protected,
    Private,
}

/// One in-class member function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberFunction {
    /// Member name as it appears in the emitted source. For
    /// constructors and destructors this is the class name (without
    /// `~` for ctors — emit adds the tilde from `kind`).
    pub name: String,
    /// Return type. Constructors and destructors must use
    /// [`CppType::Void`]; emit suppresses the return-type spelling
    /// for those kinds.
    pub return_type: CppType,
    /// Parameter list. Empty means `()`.
    pub params: Vec<Param>,
    /// What kind of member function this is.
    pub kind: MemberFunctionKind,
    /// `const` member function (`int speak() const`).
    pub is_const: bool,
    /// `virtual` (declarations only emit `virtual …;` when this is
    /// set; bodies are still stubs).
    pub is_virtual: bool,
    /// Optional `/* … */` leading comment rendered above the member.
    pub leading_comment: Option<String>,
}

/// Coarse member-function classification used by emit to decide the
/// leading keyword (`virtual`), the tilde for destructors, and
/// whether to suppress the return type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberFunctionKind {
    /// A named method.
    Method,
    /// Constructor.
    Constructor,
    /// Destructor.
    Destructor,
}

/// One function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: CppType,
}

/// A free (non-member) function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeFunction {
    pub name: String,
    pub return_type: CppType,
    pub params: Vec<Param>,
    pub leading_comment: Option<String>,
}

/// C++ type spelling.
///
/// The lowering pass produces these from the recovered class table
/// and from whatever signature information the symbol carries; at
/// B3.5 the only recovered signature is `main`'s `int` return, so
/// most non-`void` slots fall back to [`CppType::Void`] via the
/// emit-time stub-body rules. The vocabulary is wider than that to
/// keep the AST stable when B3.6's signature recovery starts feeding
/// real types in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CppType {
    /// `void`.
    Void,
    /// Fixed-width integer — emits `std::intNN_t` / `std::uintNN_t`.
    Int { width_bits: u16, signed: bool },
    /// Pointer to another type.
    Ptr(Box<CppType>),
    /// Reference to another type.
    Ref(Box<CppType>),
    /// `const T`.
    Const(Box<CppType>),
    /// Class type spelled by its fully qualified name (e.g. `Foo::Bar`).
    Class { qualified_name: String },
}

impl CppType {
    /// Convenience: `int` (32-bit signed, the C++ `int` default for
    /// the targets dac supports today).
    #[must_use]
    pub const fn int() -> Self {
        Self::Int {
            width_bits: 32,
            signed: true,
        }
    }

    /// Convenience: `std::int64_t`.
    #[must_use]
    pub const fn i64() -> Self {
        Self::Int {
            width_bits: 64,
            signed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_variants_are_exhaustively_matchable() {
        // Compile-time guard: every Item variant is reachable.
        let i = Item::FreeFunction(FreeFunction {
            name: "f".into(),
            return_type: CppType::Void,
            params: Vec::new(),
            leading_comment: None,
        });
        match i {
            Item::Class(_) | Item::FreeFunction(_) => {}
        }
    }

    #[test]
    fn member_function_kind_variants_are_matchable() {
        let k = MemberFunctionKind::Method;
        match k {
            MemberFunctionKind::Method
            | MemberFunctionKind::Constructor
            | MemberFunctionKind::Destructor => {}
        }
    }

    #[test]
    fn cpptype_helpers_round_trip() {
        assert_eq!(
            CppType::int(),
            CppType::Int {
                width_bits: 32,
                signed: true
            }
        );
        assert_eq!(
            CppType::i64(),
            CppType::Int {
                width_bits: 64,
                signed: true
            }
        );
    }

    #[test]
    fn access_spec_variants_are_matchable() {
        let a = AccessSpec::Public;
        match a {
            AccessSpec::Public | AccessSpec::Protected | AccessSpec::Private => {}
        }
    }

    #[test]
    fn class_default_is_empty_struct_shape() {
        let c = Class {
            name: "Dog".into(),
            scope_chain: Vec::new(),
            bases: Vec::new(),
            has_vtable: false,
            members: Vec::new(),
            leading_comment: None,
        };
        assert_eq!(c.name, "Dog");
        assert!(c.members.is_empty());
        assert!(!c.has_vtable);
    }
}
