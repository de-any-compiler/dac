//! Append-only collection of [`dac_core::EvidenceId`]s that a prompt
//! claims to be conditioned on.
//!
//! Bundles are pure references — they do *not* re-host evidence node
//! payloads. The evidence graph is owned by `dac-core`; the bundle
//! just records the slice of handles that justify a prompt. When a
//! delta comes back from an [`crate::AiProvider`], it carries the same
//! handles (FR-37), so a verifier can rebuild the "model saw X, asked
//! about Y" link without consulting the provider's transcript.

use dac_core::EvidenceId;

/// A list of evidence handles the proposer is allowed to cite.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvidenceBundle {
    ids: Vec<EvidenceId>,
}

impl EvidenceBundle {
    /// Empty bundle. Used by tests and for "no available evidence"
    /// prompts — providers always return zero deltas in that case
    /// because the [`crate::Delta`] constructors reject empty
    /// `evidence` lists.
    #[must_use]
    pub fn new() -> Self {
        Self { ids: Vec::new() }
    }

    /// Append an evidence handle. Duplicates are kept; callers that
    /// care about uniqueness should de-duplicate upstream.
    pub fn push(&mut self, id: EvidenceId) {
        self.ids.push(id);
    }

    /// Number of handles in the bundle.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// `true` if the bundle has no handles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Handles in insertion order.
    #[must_use]
    pub fn ids(&self) -> &[EvidenceId] {
        &self.ids
    }
}

impl FromIterator<EvidenceId> for EvidenceBundle {
    fn from_iter<I: IntoIterator<Item = EvidenceId>>(iter: I) -> Self {
        Self {
            ids: iter.into_iter().collect(),
        }
    }
}

impl Extend<EvidenceId> for EvidenceBundle {
    fn extend<I: IntoIterator<Item = EvidenceId>>(&mut self, iter: I) {
        self.ids.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_core::{EvidenceGraph, EvidenceNode};

    fn three_ids() -> [EvidenceId; 3] {
        let mut g = EvidenceGraph::new();
        [
            g.add_node(EvidenceNode::Bytes { start: 0, end: 4 }),
            g.add_node(EvidenceNode::Instruction(42)),
            g.add_node(EvidenceNode::KnowledgeFact(7)),
        ]
    }

    #[test]
    fn new_bundle_is_empty() {
        let b = EvidenceBundle::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        assert!(b.ids().is_empty());
    }

    #[test]
    fn push_preserves_insertion_order() {
        let [a, b, c] = three_ids();
        let mut bundle = EvidenceBundle::new();
        bundle.push(a);
        bundle.push(b);
        bundle.push(c);
        assert_eq!(bundle.ids(), &[a, b, c]);
        assert_eq!(bundle.len(), 3);
        assert!(!bundle.is_empty());
    }

    #[test]
    fn from_iter_matches_push() {
        let [a, b, c] = three_ids();
        let bundle = EvidenceBundle::from_iter([a, b, c]);
        assert_eq!(bundle.ids(), &[a, b, c]);
    }

    #[test]
    fn duplicates_are_preserved() {
        let [a, b, _] = three_ids();
        let bundle = EvidenceBundle::from_iter([a, a, b]);
        assert_eq!(bundle.ids(), &[a, a, b]);
    }

    #[test]
    fn equality_is_order_sensitive() {
        let [a, b, _] = three_ids();
        assert_ne!(
            EvidenceBundle::from_iter([a, b]),
            EvidenceBundle::from_iter([b, a]),
        );
    }
}
