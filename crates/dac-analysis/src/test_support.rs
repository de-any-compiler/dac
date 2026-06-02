//! Test-only helpers shared between the analysis modules.
//!
//! Most analysis passes (dominators, loops, …) test cleanest against
//! hand-built CFGs rather than the full byte-level [`crate::cfg::build_cfg`]
//! pipeline. This module centralizes the construction so each pass
//! can declare its topology in a single edge list and the helper
//! recomputes the [`Cfg::exits`] and [`Cfg::unreachable`] fields the
//! same way `build_cfg` would.

#![cfg(test)]

use std::collections::{BTreeSet, VecDeque};

use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};

use crate::cfg::{BasicBlock, BlockId, Cfg, Edge, EdgeKind, Terminator};

/// Stable u8 sort key for [`EdgeKind`] in tests. The production
/// `EdgeKind::sort_key` is private — tests inline a matching
/// implementation so synthetic CFGs respect the same canonical edge
/// ordering as the real builder.
pub(crate) fn edge_kind_key(k: EdgeKind) -> u8 {
    match k {
        EdgeKind::Fall => 0,
        EdgeKind::Branch => 1,
        EdgeKind::Taken => 2,
        EdgeKind::NotTaken => 3,
    }
}

/// Build a synthetic `Cfg` with the given edges.
///
/// Blocks are numbered `0..n` with stub addresses `0x1000 + 0x10*i`.
/// `exits` and `unreachable` are recomputed from the edge list so the
/// resulting `Cfg` satisfies the same invariants `build_cfg` produces.
pub(crate) fn synthetic_cfg(
    n: usize,
    entry: BlockId,
    raw_edges: &[(BlockId, BlockId, EdgeKind)],
) -> Cfg {
    let blocks: Vec<BasicBlock> = (0..n)
        .map(|i| BasicBlock {
            id: i as BlockId,
            address: 0x1000 + 0x10 * i as u64,
            end: 0x1000 + 0x10 * (i + 1) as u64,
            instructions: Vec::new(),
            terminator: Terminator::Fall,
        })
        .collect();
    let mut edges: Vec<Edge> = raw_edges
        .iter()
        .map(|&(from, to, kind)| Edge { from, to, kind })
        .collect();
    edges.sort_by_key(|e| (e.from, edge_kind_key(e.kind), e.to));

    let has_succ: BTreeSet<BlockId> = edges.iter().map(|e| e.from).collect();
    let exits: Vec<BlockId> = (0..n as BlockId)
        .filter(|id| !has_succ.contains(id))
        .collect();

    let mut reachable: BTreeSet<BlockId> = BTreeSet::new();
    reachable.insert(entry);
    let mut queue: VecDeque<BlockId> = VecDeque::from([entry]);
    while let Some(b) = queue.pop_front() {
        for e in &edges {
            if e.from == b && reachable.insert(e.to) {
                queue.push_back(e.to);
            }
        }
    }
    let unreachable: Vec<BlockId> = (0..n as BlockId)
        .filter(|id| !reachable.contains(id))
        .collect();

    let mut g = EvidenceGraph::new();
    let ev = g.add_node(EvidenceNode::IrNode {
        layer: IrLayer::Cfg,
        id: 0,
    });

    Cfg {
        function_address: 0x1000,
        function_end: 0x1000 + 0x10 * n as u64,
        function_name: None,
        blocks,
        entry,
        exits,
        edges,
        unreachable,
        evidence: ev,
    }
}
