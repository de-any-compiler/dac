//! Natural loops, loop nest forest, and reducibility (B2.2, FR-10).
//!
//! Builds a [`LoopForest`] for a [`Cfg`] given its [`DominatorTree`]:
//!
//! 1. Enumerate back-edges (CFG edges `u → v` where `v` dominates `u`).
//! 2. Group back-edges by header `v`, then construct each natural
//!    loop body by walking predecessors backward from every source
//!    `u` until the header is reached.
//! 3. Build the nest forest by header containment: a loop `L` is the
//!    child of the smallest loop whose body contains `L.header`.
//! 4. Flag the CFG as **irreducible** when at least one non-trivial
//!    SCC has more than one entry point — a node inside the SCC that
//!    has a predecessor outside it (or the CFG entry).
//!
//! Reducible CFGs structure cleanly into `if` / `while` / `for` at
//! B2.7. Irreducible CFGs fall back to `goto` in the C backend
//! (`I-6`, spec §11.3).
//!
//! Determinism: back-edges are enumerated in [`Cfg::edges`] order
//! (already sorted); loop ids are assigned in ascending-header order;
//! `body`, `back_edges`, `children`, and `roots` are sorted; the SCC
//! computation walks vertices and successors in ascending-id order.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::cfg::{BlockId, Cfg};
use crate::dom::{predecessors_of, successors_of, DominatorTree};

/// Numeric handle for a [`Loop`] inside a [`LoopForest`]. Loop ids are
/// dense indices into [`LoopForest::loops`] in ascending-header order.
pub type LoopId = u32;

/// One natural loop.
///
/// A natural loop is the maximal set of blocks that can reach the
/// header without going through it. The header dominates every block
/// in [`Loop::body`]. Multiple back-edges with the same header merge
/// into a single `Loop`, recorded in [`Loop::back_edges`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loop {
    /// Loop id — also the index in [`LoopForest::loops`].
    pub id: LoopId,
    /// Header block: the single entry point to the loop. Dominates
    /// every block in [`Loop::body`].
    pub header: BlockId,
    /// Blocks in the loop body, including the header, ascending.
    pub body: Vec<BlockId>,
    /// Block ids whose terminator branches back to [`Loop::header`]
    /// while being dominated by it, ascending.
    pub back_edges: Vec<BlockId>,
    /// Containing loop in the nest forest, when this loop is nested
    /// inside another. `None` for outermost loops.
    pub parent: Option<LoopId>,
    /// Directly nested loops, ascending by id.
    pub children: Vec<LoopId>,
    /// Nesting depth — 0 for outermost loops, `parent.depth + 1`
    /// otherwise.
    pub depth: u32,
}

/// The complete loop nest forest for a single function.
///
/// Built from a [`Cfg`] + [`DominatorTree`] via [`LoopForest::build`].
/// Empty when the function contains no back-edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopForest {
    /// Every natural loop, indexed by [`LoopId`].
    pub loops: Vec<Loop>,
    /// Outermost loops — loops with no parent, ascending by id.
    pub roots: Vec<LoopId>,
    /// Map from header [`BlockId`] to the [`LoopId`] of the loop with
    /// that header.
    pub header_of: BTreeMap<BlockId, LoopId>,
    /// `innermost[i]` is the deepest loop containing block `i`, or
    /// `None` when block `i` is not in any loop.
    pub innermost: Vec<Option<LoopId>>,
    /// True when at least one non-trivial SCC has more than one entry
    /// point. Irreducible CFGs cannot be expressed with structured
    /// control flow; the backend uses `goto` for them (spec §11.3).
    pub irreducible: bool,
}

impl LoopForest {
    /// Build the loop nest forest for `cfg` using its dominator tree.
    #[must_use]
    pub fn build(cfg: &Cfg, doms: &DominatorTree) -> Self {
        let n = cfg.blocks.len();
        if n == 0 {
            return Self {
                loops: Vec::new(),
                roots: Vec::new(),
                header_of: BTreeMap::new(),
                innermost: Vec::new(),
                irreducible: false,
            };
        }

        let preds = predecessors_of(cfg);

        // 1. Group back-edges by header. A back-edge `u → v` requires
        //    `v` to dominate `u`; both endpoints must therefore be
        //    reachable from the function entry. Skip edges out of
        //    unreachable blocks — they leak into the natural loop
        //    body otherwise, since the BFS would walk an
        //    entry-disconnected predecessor chain without ever passing
        //    through the header (NFR-4, I-6).
        let mut back_by_header: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for e in &cfg.edges {
            if doms.idom(e.from).is_none() || doms.idom(e.to).is_none() {
                continue;
            }
            if doms.dominates(e.to, e.from) {
                back_by_header.entry(e.to).or_default().insert(e.from);
            }
        }

        // 2. Materialise each natural loop body. Predecessor expansion
        //    skips unreachable blocks so the body stays inside the
        //    dominator-tree subgraph rooted at the header.
        let mut loops: Vec<Loop> = Vec::with_capacity(back_by_header.len());
        let mut header_of: BTreeMap<BlockId, LoopId> = BTreeMap::new();
        for (header, sources) in &back_by_header {
            let id = loops.len() as LoopId;
            header_of.insert(*header, id);

            let mut body: BTreeSet<BlockId> = BTreeSet::new();
            body.insert(*header);
            let mut queue: VecDeque<BlockId> = VecDeque::new();
            for &s in sources {
                if body.insert(s) {
                    queue.push_back(s);
                }
            }
            while let Some(node) = queue.pop_front() {
                if node == *header {
                    continue;
                }
                for &p in &preds[node as usize] {
                    if doms.idom(p).is_none() {
                        continue;
                    }
                    if body.insert(p) {
                        queue.push_back(p);
                    }
                }
            }
            let body_vec: Vec<BlockId> = body.into_iter().collect();
            let back_vec: Vec<BlockId> = sources.iter().copied().collect();

            loops.push(Loop {
                id,
                header: *header,
                body: body_vec,
                back_edges: back_vec,
                parent: None,
                children: Vec::new(),
                depth: 0,
            });
        }

        // 3. Parent = smallest loop (other than L) whose body contains
        //    L.header. Walks every pair (O(L^2) — fine for the typical
        //    handful of loops per function).
        let parents: Vec<Option<LoopId>> = loops
            .iter()
            .enumerate()
            .map(|(i, l_i)| {
                let mut best: Option<LoopId> = None;
                let mut best_size: usize = usize::MAX;
                for (j, candidate) in loops.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if candidate.body.binary_search(&l_i.header).is_ok() {
                        let size_j = candidate.body.len();
                        if size_j < best_size {
                            best_size = size_j;
                            best = Some(candidate.id);
                        }
                    }
                }
                best
            })
            .collect();
        for (i, p) in parents.into_iter().enumerate() {
            loops[i].parent = p;
        }

        // 4. Children + roots + depth.
        let mut roots: Vec<LoopId> = Vec::new();
        for i in 0..loops.len() {
            if let Some(p) = loops[i].parent {
                let child_id = loops[i].id;
                loops[p as usize].children.push(child_id);
            } else {
                roots.push(loops[i].id);
            }
        }
        for l in &mut loops {
            l.children.sort_unstable();
        }
        roots.sort_unstable();

        let mut queue: VecDeque<LoopId> = VecDeque::new();
        for &r in &roots {
            loops[r as usize].depth = 0;
            queue.push_back(r);
        }
        while let Some(l) = queue.pop_front() {
            let d = loops[l as usize].depth;
            let children: Vec<LoopId> = loops[l as usize].children.clone();
            for c in children {
                loops[c as usize].depth = d + 1;
                queue.push_back(c);
            }
        }

        // 5. Innermost loop per block — the loop with the greatest
        //    depth whose body contains the block.
        let mut innermost: Vec<Option<LoopId>> = vec![None; n];
        for b in 0..n as BlockId {
            let mut best: Option<LoopId> = None;
            let mut best_depth: u32 = 0;
            for l in &loops {
                if l.body.binary_search(&b).is_ok() && (best.is_none() || l.depth > best_depth) {
                    best = Some(l.id);
                    best_depth = l.depth;
                }
            }
            innermost[b as usize] = best;
        }

        let irreducible = detect_irreducibility(cfg);

        Self {
            loops,
            roots,
            header_of,
            innermost,
            irreducible,
        }
    }
}

/// True when the CFG has at least one non-trivial SCC with more than
/// one entry point. See [`LoopForest::irreducible`] for the precise
/// definition.
fn detect_irreducibility(cfg: &Cfg) -> bool {
    let succs = successors_of(cfg);
    let sccs = compute_sccs(cfg.blocks.len(), &succs);
    for scc in &sccs {
        // Trivial SCC (single node, no self-loop) is not a cycle.
        if scc.len() == 1 {
            let b = scc[0];
            let has_self_loop = succs[b as usize].contains(&b);
            if !has_self_loop {
                continue;
            }
        }

        let scc_set: BTreeSet<BlockId> = scc.iter().copied().collect();
        let mut entries = 0;
        for &node in scc {
            let entered_from_outside = cfg
                .edges
                .iter()
                .any(|e| e.to == node && !scc_set.contains(&e.from));
            let is_function_entry = node == cfg.entry;
            if entered_from_outside || is_function_entry {
                entries += 1;
                if entries > 1 {
                    return true;
                }
            }
        }
    }
    false
}

/// Iterative Tarjan's strongly-connected-components algorithm. Returns
/// SCCs sorted internally by ascending [`BlockId`], with the outer
/// vector sorted by each SCC's first member.
fn compute_sccs(n: usize, succs: &[Vec<BlockId>]) -> Vec<Vec<BlockId>> {
    if n == 0 {
        return Vec::new();
    }
    let mut index: Vec<Option<usize>> = vec![None; n];
    let mut lowlink: Vec<usize> = vec![0; n];
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut stack: Vec<BlockId> = Vec::new();
    let mut counter: usize = 0;
    let mut sccs: Vec<Vec<BlockId>> = Vec::new();

    for v in 0..n as BlockId {
        if index[v as usize].is_some() {
            continue;
        }
        let mut work: Vec<(BlockId, usize)> = Vec::new();
        index[v as usize] = Some(counter);
        lowlink[v as usize] = counter;
        counter += 1;
        stack.push(v);
        on_stack[v as usize] = true;
        work.push((v, 0));

        while let Some((node, idx)) = work.pop() {
            let s = &succs[node as usize];
            if idx < s.len() {
                let w = s[idx];
                work.push((node, idx + 1));
                match index[w as usize] {
                    None => {
                        index[w as usize] = Some(counter);
                        lowlink[w as usize] = counter;
                        counter += 1;
                        stack.push(w);
                        on_stack[w as usize] = true;
                        work.push((w, 0));
                    }
                    Some(w_idx) => {
                        if on_stack[w as usize] {
                            let cur = lowlink[node as usize];
                            lowlink[node as usize] = cur.min(w_idx);
                        }
                    }
                }
            } else {
                // Finished visiting `node`. If it is the SCC root,
                // peel the stack.
                if Some(lowlink[node as usize]) == index[node as usize] {
                    let mut scc: Vec<BlockId> = Vec::new();
                    while let Some(w) = stack.pop() {
                        on_stack[w as usize] = false;
                        scc.push(w);
                        if w == node {
                            break;
                        }
                    }
                    scc.sort_unstable();
                    sccs.push(scc);
                }
                if let Some(&(parent, _)) = work.last() {
                    let parent_low = lowlink[parent as usize];
                    let child_low = lowlink[node as usize];
                    lowlink[parent as usize] = parent_low.min(child_low);
                }
            }
        }
    }

    sccs.sort_by_key(|s| s.first().copied().unwrap_or(BlockId::MAX));
    sccs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::EdgeKind;
    use crate::test_support::synthetic_cfg as cfg;

    fn forest(cfg: &Cfg) -> LoopForest {
        let doms = DominatorTree::build(cfg);
        LoopForest::build(cfg, &doms)
    }

    #[test]
    fn loop_01_linear_function_has_no_loops() {
        // 0 → 1 → 2
        let cfg = cfg(3, 0, &[(0, 1, EdgeKind::Fall), (1, 2, EdgeKind::Fall)]);
        let f = forest(&cfg);
        assert!(f.loops.is_empty());
        assert!(f.roots.is_empty());
        assert!(f.header_of.is_empty());
        assert_eq!(f.innermost, vec![None, None, None]);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_02_self_loop_is_one_natural_loop() {
        // 0 → 0
        let cfg = cfg(1, 0, &[(0, 0, EdgeKind::Branch)]);
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        let l = &f.loops[0];
        assert_eq!(l.header, 0);
        assert_eq!(l.body, vec![0]);
        assert_eq!(l.back_edges, vec![0]);
        assert_eq!(l.depth, 0);
        assert_eq!(l.parent, None);
        assert_eq!(f.roots, vec![0]);
        assert_eq!(f.innermost[0], Some(0));
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_03_while_loop_body_collected_via_back_edge() {
        // 0 (pre) → 1 (header); 1 → 2 (body); 2 → 1 (back); 1 → 3 (exit)
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Taken),
                (2, 1, EdgeKind::Branch),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        let l = &f.loops[0];
        assert_eq!(l.header, 1);
        assert_eq!(l.body, vec![1, 2]);
        assert_eq!(l.back_edges, vec![2]);
        assert_eq!(f.innermost, vec![None, Some(0), Some(0), None]);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_04_do_while_uses_back_target_as_header() {
        // 0 (pre) → 1 (body/header); 1 → 1 cond? Use:
        // 0 → 1; 1 → 1 (back-edge — body is the header); 1 → 2 (exit)
        let cfg = cfg(
            3,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 1, EdgeKind::Taken),
                (1, 2, EdgeKind::NotTaken),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        assert_eq!(f.loops[0].header, 1);
        assert_eq!(f.loops[0].body, vec![1]);
        assert_eq!(f.loops[0].back_edges, vec![1]);
    }

    #[test]
    fn loop_05_nested_loops_form_a_two_level_forest() {
        // 0 → 1 (outer header)
        // 1 → 2 (inner header)
        // 2 → 3 (inner body)
        // 3 → 2 (inner back-edge)
        // 3 → 4 (inner exit, still in outer)
        // 4 → 1 (outer back-edge)
        // 4 → 5 (outer exit)
        let cfg = cfg(
            6,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
                (3, 2, EdgeKind::Taken),
                (3, 4, EdgeKind::NotTaken),
                (4, 1, EdgeKind::Taken),
                (4, 5, EdgeKind::NotTaken),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 2);
        // Loop ids are assigned in ascending-header order: outer
        // header 1 → id 0, inner header 2 → id 1.
        let outer = &f.loops[0];
        let inner = &f.loops[1];
        assert_eq!(outer.header, 1);
        assert_eq!(inner.header, 2);
        assert_eq!(outer.body, vec![1, 2, 3, 4]);
        assert_eq!(inner.body, vec![2, 3]);
        assert_eq!(inner.parent, Some(0));
        assert_eq!(outer.parent, None);
        assert_eq!(outer.children, vec![1]);
        assert_eq!(inner.depth, 1);
        assert_eq!(outer.depth, 0);
        assert_eq!(f.roots, vec![0]);
        assert_eq!(f.innermost[2], Some(1)); // inner
        assert_eq!(f.innermost[4], Some(0)); // outer body, not in inner
        assert_eq!(f.innermost[5], None);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_06_sibling_loops_share_no_parent() {
        // 0 → 1 (first loop header)
        // 1 → 1 (back-edge — single-block loop)
        // 1 → 2 (exit first loop)
        // 2 → 2 (second self-loop)
        // 2 → 3 (exit second loop)
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 1, EdgeKind::Taken),
                (1, 2, EdgeKind::NotTaken),
                (2, 2, EdgeKind::Taken),
                (2, 3, EdgeKind::NotTaken),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 2);
        assert_eq!(f.loops[0].header, 1);
        assert_eq!(f.loops[1].header, 2);
        assert_eq!(f.loops[0].parent, None);
        assert_eq!(f.loops[1].parent, None);
        assert_eq!(f.roots, vec![0, 1]);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_07_multiple_back_edges_merge_into_one_loop() {
        // 0 → 1 (header)
        // 1 → 2; 1 → 3
        // 2 → 1 (back-edge A)
        // 3 → 1 (back-edge B)
        // 1 → 4 (exit)
        let cfg = cfg(
            5,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Taken),
                (1, 4, EdgeKind::Branch),
                (2, 1, EdgeKind::Branch),
                (3, 1, EdgeKind::Branch),
            ],
        );
        // Block 1 has three outgoing edges in this construction
        // (NotTaken, Taken, Branch); the synthetic CFG allows this for
        // the test but it would be unusual in real x86. Verify
        // back-edges merge into a single loop.
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        let l = &f.loops[0];
        assert_eq!(l.header, 1);
        assert_eq!(l.body, vec![1, 2, 3]);
        assert_eq!(l.back_edges, vec![2, 3]);
    }

    #[test]
    fn loop_08_irreducible_cfg_is_flagged_without_natural_loop() {
        // 0 → 1; 0 → 2; 1 → 2; 2 → 1 — two-entry cycle between {1, 2}.
        let cfg = cfg(
            3,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 2, EdgeKind::Fall),
                (2, 1, EdgeKind::Branch),
            ],
        );
        let f = forest(&cfg);
        // No back-edges (neither 1 nor 2 dominates the other), so no
        // natural loops.
        assert!(f.loops.is_empty());
        // But the SCC {1, 2} has two external entry points (both from
        // block 0), so the CFG is flagged irreducible.
        assert!(f.irreducible);
    }

    #[test]
    fn loop_09_break_out_of_loop_does_not_split_body() {
        // 0 → 1 (header)
        // 1 → 2 (body)
        // 2 → 3 (early exit — break)
        // 2 → 1 (back-edge)
        // 3 → ret  (out of the loop, but block 1 dominates neither)
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 1, EdgeKind::Taken),
                (2, 3, EdgeKind::NotTaken),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        let l = &f.loops[0];
        assert_eq!(l.header, 1);
        assert_eq!(l.body, vec![1, 2]);
        assert_eq!(l.back_edges, vec![2]);
        assert_eq!(f.innermost, vec![None, Some(0), Some(0), None]);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_10_unreachable_self_loop_is_not_flagged_irreducible() {
        // 0 → 2; orphan block 1 self-loops to itself.
        // The CFG is technically irreducible in the strict graph-theory
        // sense (block 1's SCC has no entry from the function entry).
        // We treat unreachable blocks conservatively — the SCC has
        // zero external entry edges, so entries == 0, not > 1, and
        // we do not flag it irreducible.
        let cfg = cfg(3, 0, &[(0, 2, EdgeKind::Branch), (1, 1, EdgeKind::Branch)]);
        let f = forest(&cfg);
        assert!(!f.irreducible);
        // The unreachable self-loop is still detected as a natural
        // loop (back-edge target dominance is vacuously satisfied:
        // block 1 dominates itself in its trivial idom chain).
        // But block 1 is unreachable in the dominator tree, so the
        // back-edge check returns false — no loop is created.
        assert!(f.loops.is_empty());
    }

    #[test]
    fn loop_11_back_edge_is_required_for_natural_loop() {
        // 0 → 1; 1 → 2; 2 → 1 is a back-edge (1 dominates 2). One loop.
        let cfg = cfg(
            3,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 1, EdgeKind::Branch),
            ],
        );
        let f = forest(&cfg);
        assert_eq!(f.loops.len(), 1);
        let l = &f.loops[0];
        assert_eq!(l.header, 1);
        assert_eq!(l.body, vec![1, 2]);
        assert!(!f.irreducible);
    }

    #[test]
    fn loop_12_loop_forest_is_deterministic_across_rebuilds() {
        // Same inputs → identical output. Determinism gate (NFR-9).
        let cfg = cfg(
            6,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
                (3, 2, EdgeKind::Taken),
                (3, 4, EdgeKind::NotTaken),
                (4, 1, EdgeKind::Taken),
                (4, 5, EdgeKind::NotTaken),
            ],
        );
        let f1 = forest(&cfg);
        let f2 = forest(&cfg);
        assert_eq!(f1, f2);
    }

    #[test]
    fn loop_13_scc_with_self_loop_is_treated_as_non_trivial() {
        // Single block with a self-loop and external entry from 0.
        // SCC = {1}, non-trivial because of the self-loop. Single
        // entry (from 0), so not irreducible. The natural-loop pass
        // detects it as a self-loop.
        let cfg = cfg(2, 0, &[(0, 1, EdgeKind::Fall), (1, 1, EdgeKind::Branch)]);
        let f = forest(&cfg);
        assert!(!f.irreducible);
        assert_eq!(f.loops.len(), 1);
        assert_eq!(f.loops[0].header, 1);
    }
}
