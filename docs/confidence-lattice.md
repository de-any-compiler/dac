# Confidence lattice

This page is the long-form companion to the `dac-core::confidence` module.
It explains the algebra dac uses to combine evidence about recovered facts,
why that algebra is a lattice, and how the lattice interacts with the
evidence graph (`dac-core::evidence`).

Spec / invariant: I-3 (every recovered fact has a confidence value and a
source).

---

## Why a lattice

dac recovers facts incrementally. A type for a single variable might be
learned from:

- a load width (`mov eax, [rsi]` says "4 bytes are read from `rsi`"),
- a calling-convention signature in `dac-knowledge` (`printf`'s first
  argument is `const char*`),
- a user hint (`signatures.toml` says "this is `pid_t`"),
- an AI suggestion (`SuggestStructLayout`).

These sources may agree, refine each other, or contradict. dac needs a
deterministic, associative, and commutative way to combine them so that
the recovered fact does not depend on the order passes ran in (NFR-9). A
lattice is the standard algebraic structure that fits.

## The carrier

```rust
pub struct Confidence {
    value: f32,     // clamped to [0.0, 1.0]
    source: Source, // Speculative < Derived < UserHint < Observed
}
```

`Source` is totally ordered from least to most authoritative.
Contradictions between sources are *not* re-ordered in this enum — they
are recorded as `EdgeKind::Contradicts` edges in the evidence graph. A
single `Confidence` value is always self-consistent.

`f32::NaN` is rejected at construction (mapped to `0.0`); negative zero is
normalized. After construction every `Confidence` lies on a totally
ordered numeric axis.

## The order

The natural order is componentwise:

> `a ≤ b` iff `a.source ≤ b.source` AND `a.value ≤ b.value`.

This is a *partial* order: `(0.9, Speculative)` and `(0.4, Observed)` are
incomparable (one has higher numeric weight, the other higher
authoritativeness). The lattice is the product of two total orders, so it
is still a lattice — every pair has a unique meet and join.

## Join (`∨`): supports

When two independent pieces of evidence support the same fact, take the
componentwise maximum:

```
(0.5, Derived) ∨ (0.7, Speculative) = (0.7, Derived)
(0.6, Observed) ∨ (0.9, UserHint)  = (0.9, Observed)
```

Intuition: "A or B implies the fact" → confidence is at least as strong
as either input. The source of the join is the most authoritative of the
two; the value is the larger of the two.

## Meet (`∧`): requires

When a fact requires several inputs to all hold, take the componentwise
minimum:

```
(0.5, Derived) ∧ (0.7, Speculative) = (0.5, Speculative)
(0.6, Observed) ∧ (0.9, UserHint)  = (0.6, UserHint)
```

Intuition: "A and B both required" → confidence is at most as strong as
either input ("weakest link").

## Laws (the part property tests check)

For every `a`, `b`, `c` drawn from the lattice:

| Law            | Statement                                |
| -------------- | ---------------------------------------- |
| Idempotence    | `a ∨ a = a` and `a ∧ a = a`              |
| Commutativity  | `a ∨ b = b ∨ a` and `a ∧ b = b ∧ a`      |
| Associativity  | `(a ∨ b) ∨ c = a ∨ (b ∨ c)` (same `∧`)   |
| Absorption     | `a ∨ (a ∧ b) = a` and `a ∧ (a ∨ b) = a`  |

`crates/dac-core/src/confidence.rs` exercises each of these with
`proptest`; CI fails on any violation.

## Interaction with the evidence graph

The lattice operates on `Confidence` *values*; the graph operates on the
*reasons*. A typical pattern when two passes propose the same fact:

1. Each pass calls `EvidenceGraph::add_node` to record what it observed
   (bytes, instruction, knowledge fact, …) and links its proposal to that
   node with `EdgeKind::Supports`.
2. The merge step combines the two proposals' `Confidence` values via
   `Confidence::join`.
3. If a later pass *contradicts* an earlier one, it records the conflict
   with `EdgeKind::Contradicts` and the merge resolves with `Confidence::meet`
   (or drops the lower-confidence side, depending on the policy of the
   consuming pass).

In every case the lattice answers "how strong is the combined fact?" and
the graph answers "where does each component come from?". Both are
required to satisfy invariant I-3 and to render the "Why this name?" /
"Why this type?" output the spec asks for in §12.

## Source ranking — clarifications

The total order on `Source` reads:

```
Speculative < Derived < UserHint < Observed
```

- `Speculative` is AI / heuristic. Always the lowest.
- `Derived` is deterministic analysis output. The default for almost
  every pass.
- `UserHint` is user-supplied. Outranks deterministic analysis because
  the user is, by stipulation, more authoritative than the tool — but is
  still subject to the evidence graph: a `UserHint` that contradicts an
  `Observed` debug symbol is recorded with a `Contradicts` edge and the
  consuming pass decides which to render.
- `Observed` is direct binary evidence (debug symbol, explicit type from
  DWARF / PDB, etc.). Highest authoritativeness.

`--ai-strict` (spec §13) constrains AI deltas so they can never lower
confidence on an `Observed` fact; this is enforced in `dac-verify`, not
by the lattice itself.

## Not-a-goals

- **Probabilities.** The numeric `value` field is not a Bayesian
  probability. Adding evidence does not multiply or sum; it joins. The
  lattice is about *certainty ordering*, not probability arithmetic.
- **Cross-source arithmetic.** There is no formula that turns three
  `Speculative` votes into one `Derived` value. If you want to escalate
  a `Source`, write the deterministic analysis that does it; the lattice
  refuses to invent authority.
- **Persistence.** Confidences are recomputed from the evidence graph on
  every run. Cached pass output keys `Confidence` by `(input_hash,
  settings_hash)`; nothing about the lattice itself is on-disk state.
