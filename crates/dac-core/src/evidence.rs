//! Evidence graph (invariant I-2).
//!
//! Every IR node and recovered fact carries an [`EvidenceId`] back into an
//! [`EvidenceGraph`]; walking the edges answers "why is this here?" for the
//! user (`--debug`, `--emit-report`). The AI delta protocol's
//! `EvidenceBundle` (M4) is a subgraph of this.
//!
//! Status: B0.3 ships the types and the graph itself. Concrete instruction
//! / IR / knowledge-fact identities are opaque `u64`s until the layers
//! that own them exist (B1.x / B2.x).

use std::num::NonZeroU32;

/// A stable handle to a node in an [`EvidenceGraph`].
///
/// Backed by `NonZeroU32` so `Option<EvidenceId>` and `EvidenceId` have the
/// same size — convenient when threading provenance through IR nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EvidenceId(NonZeroU32);

impl EvidenceId {
    /// Raw numeric value of the handle. Stable within one graph instance,
    /// 1-based, monotonically increasing in insertion order.
    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0.get()
    }
}

/// A node in the evidence graph.
///
/// Concrete ids for instructions, IR nodes, knowledge facts, etc. are
/// opaque `u64`s; the crates that own them (`dac-ir`, `dac-knowledge`, …)
/// supply meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceNode {
    /// A span of input bytes (file offset; half-open `[start, end)`).
    Bytes { start: u64, end: u64 },
    /// A decoded instruction.
    Instruction(u64),
    /// A node in some IR layer.
    IrNode { layer: IrLayer, id: u64 },
    /// A fact from `dac-knowledge` (e.g. a calling-convention rule).
    KnowledgeFact(u64),
    /// A user-supplied hint (signature file, type override, …).
    UserHint(u64),
    /// A proposal from an AI provider, keyed by prompt hash for FR-37
    /// reproducibility. The full prompt/response is recorded in the
    /// manifest.
    AiSuggestion { prompt_hash: [u8; 32] },
}

/// IR layer addressed by [`EvidenceNode::IrNode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrLayer {
    Instruction,
    Cfg,
    Ssa,
    Semantic,
    Source,
}

/// Directed edge label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// `from` is evidence for `to`.
    Supports,
    /// `from` contradicts `to`; surfaced by `--debug` to explain conflicts.
    Contradicts,
    /// `from` refines `to` (e.g. a more specific type subsumes a coarser
    /// one).
    Refines,
}

/// An outgoing edge in the evidence graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub target: EvidenceId,
    pub kind: EdgeKind,
}

/// Append-only directed graph of evidence facts.
///
/// Nodes are inserted with [`EvidenceGraph::add_node`] and never removed —
/// stale facts are superseded by adding `Contradicts` or `Refines` edges,
/// not by deletion, so the audit trail stays intact (spec §12).
#[derive(Debug, Default)]
pub struct EvidenceGraph {
    nodes: Vec<EvidenceNode>,
    edges: Vec<Vec<Edge>>,
}

impl EvidenceGraph {
    /// An empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a node and return its handle.
    ///
    /// Handles are 1-based and assigned in insertion order. The graph
    /// supports up to `u32::MAX` nodes; insertions beyond that ceiling
    /// would collide on the saturated id, which no real decompilation
    /// workload approaches.
    pub fn add_node(&mut self, node: EvidenceNode) -> EvidenceId {
        self.nodes.push(node);
        self.edges.push(Vec::new());
        let raw = u32::try_from(self.nodes.len()).unwrap_or(u32::MAX);
        EvidenceId(NonZeroU32::new(raw).unwrap_or(NonZeroU32::MIN))
    }

    /// Add a directed edge from `from` to `to` labeled `kind`. Returns
    /// `false` if either handle is unknown to this graph; the graph is
    /// unchanged on `false`.
    pub fn add_edge(&mut self, from: EvidenceId, to: EvidenceId, kind: EdgeKind) -> bool {
        let from_idx = (from.0.get() - 1) as usize;
        let to_raw = to.0.get() as usize;
        if from_idx < self.edges.len() && to_raw <= self.nodes.len() {
            self.edges[from_idx].push(Edge { target: to, kind });
            true
        } else {
            false
        }
    }

    /// Look up a node by handle. Returns `None` for foreign handles.
    #[must_use]
    pub fn node(&self, id: EvidenceId) -> Option<&EvidenceNode> {
        self.nodes.get((id.0.get() - 1) as usize)
    }

    /// Outgoing edges from `id`, in insertion order. Returns an empty
    /// slice for foreign handles.
    #[must_use]
    pub fn out_edges(&self, id: EvidenceId) -> &[Edge] {
        self.edges
            .get((id.0.get() - 1) as usize)
            .map_or(&[][..], Vec::as_slice)
    }

    /// Number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Iterate `(id, node)` pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (EvidenceId, &EvidenceNode)> + '_ {
        self.nodes.iter().enumerate().map(|(i, n)| {
            let raw = u32::try_from(i + 1).unwrap_or(u32::MAX);
            let id = EvidenceId(NonZeroU32::new(raw).unwrap_or(NonZeroU32::MIN));
            (id, n)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (EvidenceGraph, EvidenceId, EvidenceId, EvidenceId) {
        let mut g = EvidenceGraph::new();
        let a = g.add_node(EvidenceNode::Bytes { start: 0, end: 4 });
        let b = g.add_node(EvidenceNode::Instruction(42));
        let c = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Ssa,
            id: 7,
        });
        (g, a, b, c)
    }

    #[test]
    fn handles_are_sequential_and_one_based() {
        let (_g, a, b, c) = fixture();
        assert_eq!(a.as_u32(), 1);
        assert_eq!(b.as_u32(), 2);
        assert_eq!(c.as_u32(), 3);
    }

    #[test]
    fn node_lookup_returns_inserted_payload() {
        let (g, a, b, c) = fixture();
        assert_eq!(g.node(a), Some(&EvidenceNode::Bytes { start: 0, end: 4 }));
        assert_eq!(g.node(b), Some(&EvidenceNode::Instruction(42)));
        assert_eq!(
            g.node(c),
            Some(&EvidenceNode::IrNode {
                layer: IrLayer::Ssa,
                id: 7,
            })
        );
    }

    #[test]
    fn foreign_handle_yields_none() {
        let (g, _, _, _) = fixture();
        let foreign = EvidenceId(NonZeroU32::new(999).expect("nonzero"));
        assert!(g.node(foreign).is_none());
        assert!(g.out_edges(foreign).is_empty());
    }

    #[test]
    fn add_edge_records_outgoing_in_insertion_order() {
        let (mut g, a, b, c) = fixture();
        assert!(g.add_edge(a, b, EdgeKind::Supports));
        assert!(g.add_edge(a, c, EdgeKind::Refines));
        let edges = g.out_edges(a);
        assert_eq!(edges.len(), 2);
        assert_eq!(
            edges[0],
            Edge {
                target: b,
                kind: EdgeKind::Supports,
            }
        );
        assert_eq!(
            edges[1],
            Edge {
                target: c,
                kind: EdgeKind::Refines,
            }
        );
    }

    #[test]
    fn add_edge_rejects_foreign_endpoints() {
        let (mut g, a, _, _) = fixture();
        let foreign = EvidenceId(NonZeroU32::new(999).expect("nonzero"));
        assert!(!g.add_edge(foreign, a, EdgeKind::Supports));
        assert!(!g.add_edge(a, foreign, EdgeKind::Supports));
        assert!(g.out_edges(a).is_empty());
    }

    #[test]
    fn self_loops_are_permitted() {
        // Self-loops aren't useful in practice but the graph doesn't reject
        // them; the structuring crate decides what's semantically valid.
        let (mut g, a, _, _) = fixture();
        assert!(g.add_edge(a, a, EdgeKind::Refines));
        assert_eq!(g.out_edges(a).len(), 1);
    }

    #[test]
    fn iter_visits_every_node_once_in_insertion_order() {
        let (g, a, b, c) = fixture();
        let ids: Vec<EvidenceId> = g.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![a, b, c]);
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn ai_suggestion_carries_prompt_hash() {
        let mut g = EvidenceGraph::new();
        let h = [7u8; 32];
        let id = g.add_node(EvidenceNode::AiSuggestion { prompt_hash: h });
        assert_eq!(
            g.node(id),
            Some(&EvidenceNode::AiSuggestion { prompt_hash: h })
        );
    }
}
