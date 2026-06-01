//! Confidence lattice (invariant I-3).
//!
//! Every fact dac recovers carries a [`Confidence`] — a numeric `value` in
//! `[0.0, 1.0]` paired with a [`Source`] class. Confidences combine through
//! a join-semilattice ([`Confidence::join`]) when multiple pieces of
//! evidence independently support a fact, and a meet-semilattice
//! ([`Confidence::meet`]) when a fact requires several inputs to hold
//! jointly. Both ops are deterministic and satisfy the standard lattice
//! laws (idempotence, commutativity, associativity, absorption) — checked
//! by the property tests in the `tests` module.
//!
//! Long-form treatment: see `docs/confidence-lattice.md`.

use std::cmp::Ordering;

/// Origin of a recovered fact, per invariant I-3.
///
/// Ordered from least to most authoritative:
/// `Speculative < Derived < UserHint < Observed`. The order is total so the
/// lattice has clean meet/join behavior; contradictions between sources are
/// modeled with [`crate::EdgeKind::Contradicts`] edges in the evidence
/// graph, not by re-ranking variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// Produced by an AI provider or a non-deterministic heuristic guess.
    Speculative,
    /// Produced by deterministic analysis.
    Derived,
    /// Supplied by the user (annotation, signature file, type override).
    UserHint,
    /// Present in the binary directly (e.g. a debug symbol).
    Observed,
}

impl Source {
    /// Human-readable name suitable for diagnostics and `--debug` output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Speculative => "speculative",
            Self::Derived => "derived",
            Self::UserHint => "user-hint",
            Self::Observed => "observed",
        }
    }
}

/// A piece of confidence about a recovered fact: numeric strength plus
/// provenance class.
///
/// `value` is clamped to `[0.0, 1.0]` on construction; `NaN` is mapped to
/// `0.0` and negative zero is normalized to positive zero so equality is
/// well-behaved. With these guarantees `f32` is totally ordered here, so
/// [`Confidence`] participates in the product lattice cleanly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Confidence {
    value: f32,
    source: Source,
}

impl Confidence {
    /// Build a confidence. `value` is clamped to `[0.0, 1.0]`; `NaN` becomes
    /// `0.0`.
    #[must_use]
    pub fn new(value: f32, source: Source) -> Self {
        let clamped = if value.is_nan() {
            0.0
        } else {
            value.clamp(0.0, 1.0)
        };
        // Normalize `-0.0` so equality / hashing stay consistent if we ever
        // need them later.
        let value = if clamped == 0.0 { 0.0 } else { clamped };
        Self { value, source }
    }

    /// The numeric confidence in `[0.0, 1.0]`.
    #[must_use]
    pub fn value(self) -> f32 {
        self.value
    }

    /// The provenance class.
    #[must_use]
    pub fn source(self) -> Source {
        self.source
    }

    /// Lattice join: the stronger of two confidences supporting a fact.
    ///
    /// Componentwise max — take the higher source and the higher value.
    /// "A or B implies the fact" → the joint confidence is at least as
    /// strong as each input.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        Self {
            value: max_f32(self.value, other.value),
            source: self.source.max(other.source),
        }
    }

    /// Lattice meet: the weaker of two confidences both required for a fact.
    ///
    /// Componentwise min — take the lower source and the lower value. "A
    /// and B both required" → the joint confidence is at most as strong as
    /// either input.
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        Self {
            value: min_f32(self.value, other.value),
            source: self.source.min(other.source),
        }
    }
}

impl PartialOrd for Confidence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Product lattice: `a ≤ b` iff `a.source ≤ b.source` AND
        // `a.value ≤ b.value`. Incomparable cases (one axis greater, the
        // other less) yield `None`.
        let v = self.value.partial_cmp(&other.value)?;
        let s = self.source.cmp(&other.source);
        match (v, s) {
            (Ordering::Equal, x) | (x, Ordering::Equal) => Some(x),
            (a, b) if a == b => Some(a),
            _ => None,
        }
    }
}

// Safe because `Confidence::new` excludes NaN and normalizes -0.0.
fn max_f32(a: f32, b: f32) -> f32 {
    if a >= b {
        a
    } else {
        b
    }
}

fn min_f32(a: f32, b: f32) -> f32 {
    if a <= b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn source_strategy() -> impl Strategy<Value = Source> {
        prop_oneof![
            Just(Source::Speculative),
            Just(Source::Derived),
            Just(Source::UserHint),
            Just(Source::Observed),
        ]
    }

    fn conf_strategy() -> impl Strategy<Value = Confidence> {
        (0.0f32..=1.0f32, source_strategy()).prop_map(|(v, s)| Confidence::new(v, s))
    }

    #[test]
    fn new_clamps_value_into_unit_interval() {
        assert_eq!(Confidence::new(-1.0, Source::Derived).value(), 0.0);
        assert_eq!(Confidence::new(2.5, Source::Derived).value(), 1.0);
        assert_eq!(Confidence::new(f32::NAN, Source::Derived).value(), 0.0);
        assert_eq!(Confidence::new(f32::INFINITY, Source::Derived).value(), 1.0);
        assert_eq!(
            Confidence::new(f32::NEG_INFINITY, Source::Derived).value(),
            0.0
        );
    }

    #[test]
    fn source_ordering_is_total_and_documented() {
        assert!(Source::Speculative < Source::Derived);
        assert!(Source::Derived < Source::UserHint);
        assert!(Source::UserHint < Source::Observed);
    }

    #[test]
    fn negative_zero_is_normalized() {
        let a = Confidence::new(-0.0, Source::Derived);
        let b = Confidence::new(0.0, Source::Derived);
        assert_eq!(a, b);
    }

    proptest! {
        #[test]
        fn join_is_idempotent(a in conf_strategy()) {
            prop_assert_eq!(a.join(a), a);
        }

        #[test]
        fn meet_is_idempotent(a in conf_strategy()) {
            prop_assert_eq!(a.meet(a), a);
        }

        #[test]
        fn join_is_commutative(a in conf_strategy(), b in conf_strategy()) {
            prop_assert_eq!(a.join(b), b.join(a));
        }

        #[test]
        fn meet_is_commutative(a in conf_strategy(), b in conf_strategy()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        #[test]
        fn join_is_associative(
            a in conf_strategy(),
            b in conf_strategy(),
            c in conf_strategy(),
        ) {
            prop_assert_eq!(a.join(b).join(c), a.join(b.join(c)));
        }

        #[test]
        fn meet_is_associative(
            a in conf_strategy(),
            b in conf_strategy(),
            c in conf_strategy(),
        ) {
            prop_assert_eq!(a.meet(b).meet(c), a.meet(b.meet(c)));
        }

        #[test]
        fn absorption_join_meet(a in conf_strategy(), b in conf_strategy()) {
            prop_assert_eq!(a.join(a.meet(b)), a);
        }

        #[test]
        fn absorption_meet_join(a in conf_strategy(), b in conf_strategy()) {
            prop_assert_eq!(a.meet(a.join(b)), a);
        }

        #[test]
        fn join_dominates_inputs(a in conf_strategy(), b in conf_strategy()) {
            let j = a.join(b);
            prop_assert!(j.source() >= a.source());
            prop_assert!(j.source() >= b.source());
            prop_assert!(j.value() >= a.value());
            prop_assert!(j.value() >= b.value());
        }

        #[test]
        fn meet_is_dominated_by_inputs(a in conf_strategy(), b in conf_strategy()) {
            let m = a.meet(b);
            prop_assert!(m.source() <= a.source());
            prop_assert!(m.source() <= b.source());
            prop_assert!(m.value() <= a.value());
            prop_assert!(m.value() <= b.value());
        }

        #[test]
        fn join_is_least_upper_bound_when_comparable(
            a in conf_strategy(),
            b in conf_strategy(),
        ) {
            // If a ≤ b in the partial order, then join(a, b) == b.
            if a.partial_cmp(&b) == Some(Ordering::Less)
                || a.partial_cmp(&b) == Some(Ordering::Equal)
            {
                prop_assert_eq!(a.join(b), b);
                prop_assert_eq!(a.meet(b), a);
            }
        }
    }
}
