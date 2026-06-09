//! The verifier's view of recovered facts (spec §13.4).
//!
//! Before an AI [`Delta`](dac_ai::Delta) is judged, the orchestrator
//! populates a [`KnownFacts`] with the symbols, slots, and regions that
//! the deterministic pipeline has already produced. The verifier then
//! cross-references the delta against this world model:
//!
//! - "rename to a name a different symbol already owns" — caught by
//!   the reverse `names_by_symbol` map.
//! - "retype an int slot to a pointer without observed pointer use" —
//!   caught by comparing the slot's recorded [`SlotType`] against the
//!   requested type string.
//! - "strict mode forbids overwriting an Observed fact" — caught by
//!   inspecting the target's recorded [`Source`].
//!
//! The world is deliberately minimal at B4.3. It carries only the
//! shape the two PLAN.md done-when cases need plus the strict-mode
//! gate; broader IR consistency (struct-overlap checks driven by the
//! recovered layout table, idiom whitelists drawn from
//! `dac-knowledge`, etc.) lands with B4.4 / B4.5 as those passes
//! produce the facts that warrant checking.

use std::collections::BTreeMap;

use dac_ai::{RegionRef, SlotRef, SymbolRef};
use dac_core::Source;

/// A recovered symbol's name and confidence source, as the verifier
/// sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownSymbol {
    pub name: String,
    pub source: Source,
}

/// What the verifier knows about a slot's recovered type.
///
/// The shape is intentionally coarse — verification only needs to
/// decide "would this retype contradict observed evidence?". Finer
/// type detail lives in `dac-ir` and is not re-encoded here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotType {
    /// Recovered as an integer of the given physical width.
    Integer { width_bits: u32 },
    /// Recovered as a pointer (load/store with `[base+off]`, pointer
    /// arithmetic, address-of operand, etc.).
    Pointer,
    /// Recovered as a floating-point value.
    Float,
    /// Recovered as an aggregate (struct, array, union) — typically
    /// from a `[[struct]]` hint or layout-recognition pass.
    Aggregate,
    /// The slot is known to the world model but no type evidence has
    /// been recorded yet. Treated as "any retype is plausible" so the
    /// verifier does not over-reject.
    Unknown,
}

impl SlotType {
    /// Stable kebab-case tag — useful for log lines and rejection
    /// messages.
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::Integer { .. } => "integer",
            Self::Pointer => "pointer",
            Self::Float => "float",
            Self::Aggregate => "aggregate",
            Self::Unknown => "unknown",
        }
    }

    /// `true` iff a retype to this slot type would be consistent with
    /// observed pointer evidence.
    #[must_use]
    pub fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer)
    }
}

/// A recovered slot's known type and confidence source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownSlot {
    pub ty: SlotType,
    pub source: Source,
}

/// A recovered region's confidence source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownRegion {
    pub source: Source,
}

/// The verifier's snapshot of recovered facts.
///
/// Built from the recovery output (or empty by default — an empty
/// world makes every [`Delta`](dac_ai::Delta) reject as
/// [`UnknownTarget`](crate::verify::DeltaRejection::UnknownTarget),
/// which is the safe default until the orchestrator wires the world
/// model from real recovered state in B4.4 / B4.5).
///
/// The maps are `BTreeMap`s so iteration order is deterministic
/// (NFR-9).
#[derive(Debug, Clone, Default)]
pub struct KnownFacts {
    symbols: BTreeMap<SymbolRef, KnownSymbol>,
    slots: BTreeMap<SlotRef, KnownSlot>,
    regions: BTreeMap<RegionRef, KnownRegion>,
    names_by_symbol: BTreeMap<String, SymbolRef>,
}

impl KnownFacts {
    /// Empty world. The orchestrator can call this and add facts via
    /// the `insert_*` methods.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a known symbol. The reverse name index is kept in sync
    /// so collision checks are O(log n).
    ///
    /// If `name` already maps to a *different* symbol, the new symbol
    /// overrides the index entry. This matches the orchestrator's
    /// "last writer wins" intent — there is no scenario where two
    /// real recovered symbols share a name, so the conflict can only
    /// arise from a buggy test fixture.
    pub fn insert_symbol(&mut self, id: SymbolRef, name: impl Into<String>, source: Source) {
        let name = name.into();
        self.names_by_symbol.insert(name.clone(), id);
        self.symbols.insert(id, KnownSymbol { name, source });
    }

    /// Record a known slot.
    pub fn insert_slot(&mut self, id: SlotRef, ty: SlotType, source: Source) {
        self.slots.insert(id, KnownSlot { ty, source });
    }

    /// Record a known region.
    pub fn insert_region(&mut self, id: RegionRef, source: Source) {
        self.regions.insert(id, KnownRegion { source });
    }

    /// Look up a known symbol by handle.
    #[must_use]
    pub fn symbol(&self, id: SymbolRef) -> Option<&KnownSymbol> {
        self.symbols.get(&id)
    }

    /// Look up a known slot by handle.
    #[must_use]
    pub fn slot(&self, id: SlotRef) -> Option<&KnownSlot> {
        self.slots.get(&id)
    }

    /// Look up a known region by handle.
    #[must_use]
    pub fn region(&self, id: RegionRef) -> Option<&KnownRegion> {
        self.regions.get(&id)
    }

    /// Look up the symbol that currently owns `name`, if any.
    #[must_use]
    pub fn symbol_by_name(&self, name: &str) -> Option<SymbolRef> {
        self.names_by_symbol.get(name).copied()
    }

    /// Total number of symbols in the world.
    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Total number of slots in the world.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Total number of regions in the world.
    #[must_use]
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_world_has_no_facts() {
        let w = KnownFacts::new();
        assert_eq!(w.symbol_count(), 0);
        assert_eq!(w.slot_count(), 0);
        assert_eq!(w.region_count(), 0);
        assert!(w.symbol(SymbolRef(1)).is_none());
        assert!(w.slot(SlotRef(1)).is_none());
        assert!(w.region(RegionRef(1)).is_none());
        assert!(w.symbol_by_name("missing").is_none());
    }

    #[test]
    fn insert_symbol_updates_both_maps() {
        let mut w = KnownFacts::new();
        w.insert_symbol(SymbolRef(7), "checksum", Source::Derived);
        assert_eq!(
            w.symbol(SymbolRef(7)).map(|s| s.name.as_str()),
            Some("checksum")
        );
        assert_eq!(w.symbol_by_name("checksum"), Some(SymbolRef(7)));
        assert_eq!(w.symbol_count(), 1);
    }

    #[test]
    fn slot_type_tags_are_distinct_kebab_case() {
        let tags = [
            SlotType::Integer { width_bits: 32 }.tag(),
            SlotType::Pointer.tag(),
            SlotType::Float.tag(),
            SlotType::Aggregate.tag(),
            SlotType::Unknown.tag(),
        ];
        for t in tags {
            assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len());
    }

    #[test]
    fn is_pointer_only_holds_for_pointer_variant() {
        assert!(SlotType::Pointer.is_pointer());
        assert!(!SlotType::Integer { width_bits: 32 }.is_pointer());
        assert!(!SlotType::Float.is_pointer());
        assert!(!SlotType::Aggregate.is_pointer());
        assert!(!SlotType::Unknown.is_pointer());
    }

    #[test]
    fn inserted_facts_are_observable() {
        let mut w = KnownFacts::new();
        w.insert_slot(
            SlotRef(3),
            SlotType::Integer { width_bits: 32 },
            Source::Observed,
        );
        w.insert_region(RegionRef(9), Source::Derived);
        assert_eq!(
            w.slot(SlotRef(3)).map(|s| (s.ty.clone(), s.source)),
            Some((SlotType::Integer { width_bits: 32 }, Source::Observed))
        );
        assert_eq!(
            w.region(RegionRef(9)).map(|r| r.source),
            Some(Source::Derived)
        );
    }

    #[test]
    fn name_index_follows_latest_writer() {
        // The orchestrator never collides real recovered names, but if
        // a test inserts the same name twice the index points to the
        // most recent symbol. Both KnownSymbol entries survive, so a
        // verifier can still see "two symbols named X" via the symbol
        // table — the collision check only consults the index.
        let mut w = KnownFacts::new();
        w.insert_symbol(SymbolRef(1), "main", Source::Observed);
        w.insert_symbol(SymbolRef(2), "main", Source::Derived);
        assert_eq!(w.symbol_by_name("main"), Some(SymbolRef(2)));
        assert_eq!(w.symbol_count(), 2);
    }
}
