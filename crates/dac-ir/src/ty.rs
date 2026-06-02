//! Type lattice (B2.6, FR-14, FR-16).
//!
//! The recovered type of an SSA value, stack local, or struct field
//! is one element of this lattice. Type recovery walks the lattice
//! from the bottom (`Unknown` — "we know nothing yet") toward more
//! specific shapes (`Int(32, Signed)`, `Ptr(Int(8, Unsigned))`,
//! `Struct{…}`, …) by [`Type::join`]ing independent observations.
//! Contradictory observations land at the top of the lattice
//! ([`Type::Top`]) so downstream passes can flag them rather than
//! silently picking a winner (I-6).
//!
//! ## Lattice shape
//!
//! ```text
//!                       Top                 (contradiction)
//!                        |
//!     +----+----+-------+--------+----+
//!     |    |    |       |        |    |
//!  Int(N) Ptr  Struct Array    …
//!     |    |
//!  Int(N, sgn)  Ptr<T>
//!     …
//!                        |
//!                     Unknown                (no information)
//! ```
//!
//! - [`Type::Unknown`] is the bottom: identity for [`Type::join`].
//! - [`Type::Top`] is the top: absorbing for [`Type::join`].
//! - The major variants ([`Type::Int`], [`Type::Ptr`],
//!   [`Type::Struct`], [`Type::Array`]) are mutually incomparable;
//!   any join across two of them yields [`Type::Top`]. Inside one
//!   variant, sub-lattices refine further (e.g. signedness for
//!   integers).
//!
//! ## What B2.6 lands
//!
//! - The lattice itself ([`Type`], [`IntType`], [`Signedness`],
//!   [`StructType`], [`StructField`], [`ArrayType`]).
//! - [`Type::join`] / [`Signedness::join`] — combine independent
//!   observations about the same value.
//! - [`Type::is_top`] / [`Type::is_unknown`] convenience predicates.
//! - [`Type::int_width_bits`] for callers that want to query the
//!   integer width without matching every variant.
//!
//! ## What deliberately doesn't land yet
//!
//! - **Sub-typing / coercion.** The lattice is pure structural
//!   information. Implicit coercions (`int -> ptr` on a cast, sign
//!   extension on a load) are the propagation pass's job, not the
//!   lattice's.
//! - **Union / enum recovery.** Both fall out of struct recovery in
//!   B3.2. The variants are absent here so the propagation pass does
//!   not invent them.
//! - **Function pointers.** A `Ptr(Function{…})` variant lands when
//!   the C backend (B2.8) needs it.
//! - **Float / vector types.** Float types land alongside FP-aware
//!   SSA in a later batch.
//!
//! ## Determinism
//!
//! Every type is a pure value (no interior mutability, no hashed
//! addresses). [`Type::join`] is a total function on the algebra
//! above and is byte-deterministic.

/// One type-lattice element.
///
/// Constructors that hold owned data (`Ptr`, `Struct`, `Array`) own
/// their children behind a `Box` so the enum stays a thin
/// discriminant-plus-pointer. `Eq` and `Hash` are exact equality,
/// which is the right semantics for the propagation pass's worklist
/// deduplication: "same type" means "same shape".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Bottom of the lattice — no information yet.
    Unknown,
    /// Integer / pointer-sized scalar.
    Int(IntType),
    /// Pointer to a known target type. `Ptr(Unknown)` is a generic
    /// pointer of unknown pointee.
    Ptr(Box<Type>),
    /// Aggregate with named or unnamed fields at known offsets.
    Struct(StructType),
    /// Fixed-element-type array; length is optional for arrays whose
    /// extent has not yet been recovered.
    Array(ArrayType),
    /// Top of the lattice — two observations disagree.
    Top,
}

impl Type {
    /// True for [`Type::Unknown`].
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    /// True for [`Type::Top`].
    #[must_use]
    pub const fn is_top(&self) -> bool {
        matches!(self, Type::Top)
    }

    /// Width in bits when the type is an integer; `None` for every
    /// other variant. Pointers are deliberately *not* mapped to the
    /// architecture's pointer width here — the propagation pass knows
    /// the pointer width from the architecture and assigns it
    /// explicitly when it lowers `Ptr` into a concrete machine word.
    #[must_use]
    pub fn int_width_bits(&self) -> Option<u16> {
        match self {
            Type::Int(t) => Some(t.width_bits),
            _ => None,
        }
    }

    /// Lattice join: combine two independent observations about the
    /// same value.
    ///
    /// The propagation pass calls this whenever multiple constraints
    /// (a load width, an API-signature return slot, a phi merge)
    /// pin down the same SSA value. The result is the most specific
    /// type consistent with both inputs, or [`Type::Top`] when the
    /// two inputs are contradictory.
    ///
    /// Laws:
    /// - Idempotent: `t.join(&t) == t`.
    /// - Commutative: `a.join(&b) == b.join(&a)`.
    /// - Associative: `a.join(&b.join(&c)) == a.join(&b).join(&c)`.
    /// - `Unknown` is the identity: `t.join(&Unknown) == t`.
    /// - `Top` is absorbing: `t.join(&Top) == Top`.
    #[must_use]
    pub fn join(&self, other: &Type) -> Type {
        match (self, other) {
            (Type::Unknown, x) | (x, Type::Unknown) => x.clone(),
            (Type::Top, _) | (_, Type::Top) => Type::Top,
            (Type::Int(a), Type::Int(b)) => {
                if a.width_bits != b.width_bits {
                    Type::Top
                } else {
                    Type::Int(IntType {
                        width_bits: a.width_bits,
                        signedness: a.signedness.join(b.signedness),
                    })
                }
            }
            (Type::Ptr(a), Type::Ptr(b)) => Type::Ptr(Box::new(a.join(b))),
            (Type::Struct(a), Type::Struct(b)) => {
                if a == b {
                    Type::Struct(a.clone())
                } else {
                    Type::Top
                }
            }
            (Type::Array(a), Type::Array(b)) => {
                if a.length != b.length {
                    Type::Top
                } else {
                    Type::Array(ArrayType {
                        element: Box::new(a.element.join(&b.element)),
                        length: a.length,
                    })
                }
            }
            // Cross-variant joins are always contradictory.
            _ => Type::Top,
        }
    }

    /// Construct a signed integer of `width_bits`.
    #[must_use]
    pub const fn signed_int(width_bits: u16) -> Type {
        Type::Int(IntType {
            width_bits,
            signedness: Signedness::Signed,
        })
    }

    /// Construct an unsigned integer of `width_bits`.
    #[must_use]
    pub const fn unsigned_int(width_bits: u16) -> Type {
        Type::Int(IntType {
            width_bits,
            signedness: Signedness::Unsigned,
        })
    }

    /// Construct an integer of unknown signedness — what a `Load` /
    /// `Store` width observation produces before any signed-vs-
    /// unsigned discriminator runs.
    #[must_use]
    pub const fn int_of_width(width_bits: u16) -> Type {
        Type::Int(IntType {
            width_bits,
            signedness: Signedness::Unknown,
        })
    }

    /// Construct `Ptr<pointee>` ergonomically.
    #[must_use]
    pub fn ptr_to(pointee: Type) -> Type {
        Type::Ptr(Box::new(pointee))
    }
}

/// Integer-type leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntType {
    /// Width in bits. Typically `1` (bool), `8`, `16`, `32`, `64`.
    /// `0` is reserved for "unknown width" and is currently never
    /// produced by the lattice itself.
    pub width_bits: u16,
    /// Signedness sub-lattice — `Unknown` is the bottom, `Signed` and
    /// `Unsigned` are incomparable peaks. A join that conflicts
    /// degrades the *enclosing* [`Type`] to [`Type::Top`] when widths
    /// match but signs disagree.
    pub signedness: Signedness,
}

/// Signed / unsigned / not-yet-known.
///
/// `join` semantics inside the integer leaf:
/// - `Unknown` is the identity (any sign refines it).
/// - `Signed.join(Signed) = Signed`, `Unsigned.join(Unsigned) = Unsigned`.
/// - `Signed.join(Unsigned)` is the only conflicting case; it
///   propagates outward by returning [`Signedness::Conflict`], which
///   the enclosing [`Type::join`] interprets as a [`Type::Top`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signedness {
    /// Sign bit's meaning has not yet been pinned down.
    Unknown,
    /// Two's-complement signed interpretation.
    Signed,
    /// Unsigned interpretation.
    Unsigned,
    /// Two observations disagree on sign. Distinguished from
    /// `Unknown` so the propagation pass can surface the contradiction
    /// in `--debug`.
    Conflict,
}

impl Signedness {
    /// Sub-lattice join.
    #[must_use]
    pub const fn join(self, other: Signedness) -> Signedness {
        match (self, other) {
            (Signedness::Unknown, x) | (x, Signedness::Unknown) => x,
            (Signedness::Conflict, _) | (_, Signedness::Conflict) => Signedness::Conflict,
            (Signedness::Signed, Signedness::Signed) => Signedness::Signed,
            (Signedness::Unsigned, Signedness::Unsigned) => Signedness::Unsigned,
            (Signedness::Signed, Signedness::Unsigned)
            | (Signedness::Unsigned, Signedness::Signed) => Signedness::Conflict,
        }
    }
}

/// A recovered struct. Fields are kept in ascending-offset order so
/// equality and hashing are deterministic regardless of discovery
/// order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructType {
    /// Optional struct tag. `None` for anonymous aggregates.
    pub name: Option<String>,
    /// Fields in ascending `offset` order.
    pub fields: Vec<StructField>,
}

/// One field inside a [`StructType`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructField {
    /// Byte offset from the start of the enclosing struct.
    pub offset: u64,
    /// Field type.
    pub ty: Type,
    /// Optional field name.
    pub name: Option<String>,
}

/// A recovered array.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType {
    /// Element type.
    pub element: Box<Type>,
    /// Number of elements, when known. `None` denotes a flexible
    /// array — recovered by B3.2 when adjacent struct fields suggest
    /// a trailing array but no terminating bound is observed.
    pub length: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i32_signed() -> Type {
        Type::signed_int(32)
    }
    fn i32_unsigned() -> Type {
        Type::unsigned_int(32)
    }
    fn i64_signed() -> Type {
        Type::signed_int(64)
    }

    #[test]
    fn unknown_is_join_identity() {
        let t = i32_signed();
        assert_eq!(Type::Unknown.join(&t), t);
        assert_eq!(t.join(&Type::Unknown), t);
    }

    #[test]
    fn top_is_join_absorbing() {
        let t = i32_signed();
        assert_eq!(Type::Top.join(&t), Type::Top);
        assert_eq!(t.join(&Type::Top), Type::Top);
    }

    #[test]
    fn join_is_idempotent_on_each_variant() {
        for t in [
            Type::Unknown,
            Type::Top,
            i32_signed(),
            i32_unsigned(),
            Type::ptr_to(i32_signed()),
            Type::Struct(StructType {
                name: Some("S".into()),
                fields: vec![],
            }),
            Type::Array(ArrayType {
                element: Box::new(i32_signed()),
                length: Some(4),
            }),
        ] {
            assert_eq!(t.join(&t), t, "join not idempotent for {t:?}");
        }
    }

    #[test]
    fn join_is_commutative_for_disjoint_variants() {
        let a = i32_signed();
        let b = Type::ptr_to(Type::Unknown);
        assert_eq!(a.join(&b), b.join(&a));
        // Cross-variant joins land at Top.
        assert_eq!(a.join(&b), Type::Top);
    }

    #[test]
    fn signedness_unknown_widens_to_observed_sign() {
        let unknown_width = Type::int_of_width(32);
        assert_eq!(unknown_width.join(&i32_signed()), i32_signed());
        assert_eq!(unknown_width.join(&i32_unsigned()), i32_unsigned());
    }

    #[test]
    fn signedness_conflict_propagates_to_top() {
        let merged = i32_signed().join(&i32_unsigned());
        // Width matches; signs disagree.
        let Type::Int(IntType {
            width_bits,
            signedness,
        }) = merged
        else {
            panic!("expected Int, got {merged:?}");
        };
        assert_eq!(width_bits, 32);
        assert_eq!(signedness, Signedness::Conflict);
    }

    #[test]
    fn integer_width_mismatch_lands_at_top() {
        assert_eq!(i32_signed().join(&i64_signed()), Type::Top);
    }

    #[test]
    fn ptr_join_recurses_into_pointee() {
        let a = Type::ptr_to(Type::int_of_width(8));
        let b = Type::ptr_to(Type::unsigned_int(8));
        let joined = a.join(&b);
        assert_eq!(joined, Type::ptr_to(Type::unsigned_int(8)));
    }

    #[test]
    fn array_with_different_lengths_is_top() {
        let a = Type::Array(ArrayType {
            element: Box::new(i32_signed()),
            length: Some(4),
        });
        let b = Type::Array(ArrayType {
            element: Box::new(i32_signed()),
            length: Some(8),
        });
        assert_eq!(a.join(&b), Type::Top);
    }

    #[test]
    fn array_same_length_joins_element() {
        let a = Type::Array(ArrayType {
            element: Box::new(Type::int_of_width(8)),
            length: Some(4),
        });
        let b = Type::Array(ArrayType {
            element: Box::new(Type::unsigned_int(8)),
            length: Some(4),
        });
        let joined = a.join(&b);
        let Type::Array(ArrayType { element, length }) = joined else {
            panic!("expected Array, got something else");
        };
        assert_eq!(length, Some(4));
        assert_eq!(*element, Type::unsigned_int(8));
    }

    #[test]
    fn struct_join_only_collapses_when_equal() {
        let s1 = Type::Struct(StructType {
            name: Some("S".into()),
            fields: vec![StructField {
                offset: 0,
                ty: i32_signed(),
                name: None,
            }],
        });
        let s1_again = s1.clone();
        let s2 = Type::Struct(StructType {
            name: Some("T".into()),
            fields: vec![],
        });
        assert_eq!(s1.join(&s1_again), s1);
        assert_eq!(s1.join(&s2), Type::Top);
    }

    #[test]
    fn int_width_bits_helper() {
        assert_eq!(i32_signed().int_width_bits(), Some(32));
        assert_eq!(Type::Unknown.int_width_bits(), None);
        assert_eq!(Type::ptr_to(i32_signed()).int_width_bits(), None);
    }

    #[test]
    fn predicates_distinguish_endpoints() {
        assert!(Type::Unknown.is_unknown());
        assert!(!Type::Unknown.is_top());
        assert!(Type::Top.is_top());
        assert!(!Type::Top.is_unknown());
        assert!(!i32_signed().is_unknown());
        assert!(!i32_signed().is_top());
    }

    #[test]
    fn signedness_join_is_idempotent_and_commutative() {
        let kinds = [
            Signedness::Unknown,
            Signedness::Signed,
            Signedness::Unsigned,
            Signedness::Conflict,
        ];
        for a in kinds {
            assert_eq!(a.join(a), a);
            for b in kinds {
                assert_eq!(a.join(b), b.join(a));
            }
        }
    }

    #[test]
    fn signedness_conflict_absorbs() {
        for s in [
            Signedness::Unknown,
            Signedness::Signed,
            Signedness::Unsigned,
            Signedness::Conflict,
        ] {
            assert_eq!(Signedness::Conflict.join(s), Signedness::Conflict);
        }
    }
}
