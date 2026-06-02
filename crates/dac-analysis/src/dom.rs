//! Dominator and post-dominator trees (B2.2, FR-10).
//!
//! Computes the immediate-dominator vector for a [`Cfg`] using the
//! Cooper-Harvey-Kennedy iterative algorithm (Cooper, Harvey, Kennedy,
//! *A Simple, Fast Dominance Algorithm*, 2001). Post-dominators are
//! computed by running the same algorithm on the reverse CFG with a
//! virtual exit node that merges all real CFG exits.
//!
//! Both trees are deterministic: blocks are visited in reverse
//! postorder, predecessor lists are derived from the CFG's already-
//! sorted [`Cfg::edges`] vector, and any auxiliary ordering breaks ties
//! by [`BlockId`].
//!
//! ## Domain conventions
//!
//! - [`DominatorTree::idom`] returns `Some(entry)` for the entry block
//!   (entry dominates itself), `Some(parent)` for any reachable non-
//!   entry block, and `None` for blocks unreachable from the entry.
//! - [`PostDominatorTree::ipostdom`] distinguishes three states with
//!   the [`PostDom`] enum: a real block id, the synthetic [`PostDom::Exit`]
//!   for CFG exit blocks (their only "post-dominator" is the virtual
//!   exit, which is not exposed), and [`PostDom::Unreachable`] for
//!   blocks with no path to any exit (e.g. infinite loops).
//!
//! ## Determinism
//!
//! Both builders are [`Determinism::Pure`](dac_core::Determinism::Pure) —
//! same `&Cfg` in produces the same trees out. No environment, RNG, or
//! map-iteration assumptions leak in (we use `BTreeMap` / `BTreeSet` or
//! sorted `Vec` everywhere).

use crate::cfg::{BlockId, Cfg};

/// Dominator tree for a single function's CFG.
///
/// Built from a [`Cfg`] using [`DominatorTree::build`]. The immediate
/// dominator of every reachable non-entry block points to one block
/// strictly closer to the entry; the entry's immediate dominator is
/// itself; unreachable blocks have no immediate dominator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DominatorTree {
    /// `idoms[i]` is the immediate dominator of block `i`. `Some(i)`
    /// for the entry block, `Some(parent)` for a reachable non-entry
    /// block, `None` for unreachable blocks.
    idoms: Vec<Option<BlockId>>,
    /// Cached entry block id so [`DominatorTree::idom`] can answer
    /// without a `&Cfg`.
    entry: BlockId,
}

impl DominatorTree {
    /// Build the dominator tree for `cfg`.
    #[must_use]
    pub fn build(cfg: &Cfg) -> Self {
        let n = cfg.blocks.len();
        if n == 0 {
            return Self {
                idoms: Vec::new(),
                entry: 0,
            };
        }

        let preds = predecessors_of(cfg);
        let succs = successors_of(cfg);
        let rpo = compute_rpo(cfg.entry, &succs, n);
        let rpo_index = invert_order(&rpo, n);

        let mut idoms: Vec<Option<BlockId>> = vec![None; n];
        idoms[cfg.entry as usize] = Some(cfg.entry);

        let mut changed = true;
        while changed {
            changed = false;
            for &b in rpo.iter().skip(1) {
                let bi = b as usize;
                let mut new_idom: Option<BlockId> = None;
                for &p in &preds[bi] {
                    if idoms[p as usize].is_some() {
                        new_idom = Some(match new_idom {
                            None => p,
                            Some(cur) => intersect(p, cur, &idoms, &rpo_index),
                        });
                    }
                }
                if idoms[bi] != new_idom {
                    idoms[bi] = new_idom;
                    changed = true;
                }
            }
        }

        Self {
            idoms,
            entry: cfg.entry,
        }
    }

    /// Immediate dominator of `b`. Returns `Some(b)` for the entry,
    /// `Some(parent)` for any other reachable block, and `None` for
    /// blocks unreachable from the entry.
    #[must_use]
    pub fn idom(&self, b: BlockId) -> Option<BlockId> {
        self.idoms.get(b as usize).copied().flatten()
    }

    /// Entry block of the CFG this tree was built from.
    #[must_use]
    pub fn entry(&self) -> BlockId {
        self.entry
    }

    /// Does `a` dominate `b`? Reflexive: `dominates(a, a)` is true when
    /// `a` is reachable. Returns false when either block is
    /// unreachable.
    #[must_use]
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        if (a as usize) >= self.idoms.len() || (b as usize) >= self.idoms.len() {
            return false;
        }
        if self.idoms[a as usize].is_none() || self.idoms[b as usize].is_none() {
            return false;
        }
        let mut cur = b;
        loop {
            if cur == a {
                return true;
            }
            let Some(p) = self.idoms[cur as usize] else {
                return false;
            };
            if p == cur {
                // Reached the entry without finding `a`.
                return false;
            }
            cur = p;
        }
    }

    /// Does `a` strictly dominate `b`? Equivalent to `a != b &&
    /// dominates(a, b)`.
    #[must_use]
    pub fn strictly_dominates(&self, a: BlockId, b: BlockId) -> bool {
        a != b && self.dominates(a, b)
    }

    /// Direct children of `a` in the dominator tree, ascending by
    /// [`BlockId`]. The entry block is never returned as its own child.
    #[must_use]
    pub fn children(&self, a: BlockId) -> Vec<BlockId> {
        let mut out: Vec<BlockId> = Vec::new();
        for (i, slot) in self.idoms.iter().enumerate() {
            let id = i as BlockId;
            if id == self.entry {
                continue;
            }
            if *slot == Some(a) {
                out.push(id);
            }
        }
        // Already ascending since we iterate in index order.
        out
    }
}

/// One of three states for an immediate post-dominator query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostDom {
    /// `b`'s immediate post-dominator is a real CFG block.
    Block(BlockId),
    /// `b` is a CFG exit (no outgoing edges inside the function); the
    /// only thing post-dominating it is the synthetic virtual exit,
    /// which the public API does not expose.
    Exit,
    /// `b` has no path to any CFG exit (e.g. it sits in an infinite
    /// loop). No post-dominator exists.
    Unreachable,
}

/// Post-dominator tree for a single function's CFG.
///
/// The tree's notional root is a synthetic virtual exit that merges
/// every real CFG exit. Callers see the result through [`PostDom`],
/// which keeps the virtual exit out of the public surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostDominatorTree {
    /// `ipdoms[i]` is the immediate post-dominator of block `i`. The
    /// underlying representation uses the synthetic virtual exit at
    /// index `n_blocks`; entries are sized `n_blocks + 1`.
    ipdoms: Vec<Option<BlockId>>,
    /// Number of real blocks in the source CFG.
    n_blocks: usize,
}

impl PostDominatorTree {
    /// Build the post-dominator tree for `cfg`.
    #[must_use]
    pub fn build(cfg: &Cfg) -> Self {
        let n = cfg.blocks.len();
        if n == 0 {
            return Self {
                ipdoms: Vec::new(),
                n_blocks: 0,
            };
        }

        let virtual_exit: BlockId = n as BlockId;
        let total = n + 1;

        let fwd_preds = predecessors_of(cfg);
        let fwd_succs = successors_of(cfg);

        // Reverse-CFG predecessors for each real block: forward
        // successors. Real exits gain the virtual exit as their sole
        // reverse predecessor. The virtual exit has no reverse
        // predecessors (it is the root).
        let mut rev_preds: Vec<Vec<BlockId>> = Vec::with_capacity(total);
        for succs in fwd_succs.iter().take(n) {
            let mut p = succs.clone();
            if succs.is_empty() {
                p.push(virtual_exit);
            }
            rev_preds.push(p);
        }
        rev_preds.push(Vec::new());

        // Reverse-CFG successors: forward predecessors for real blocks.
        // The virtual exit's reverse successors are the real exits.
        let mut rev_succs: Vec<Vec<BlockId>> = Vec::with_capacity(total);
        for preds in fwd_preds.iter().take(n) {
            rev_succs.push(preds.clone());
        }
        let mut ve_succs: Vec<BlockId> = (0..n as BlockId)
            .filter(|&i| fwd_succs[i as usize].is_empty())
            .collect();
        ve_succs.sort_unstable();
        rev_succs.push(ve_succs);

        let rpo = compute_rpo(virtual_exit, &rev_succs, total);
        let rpo_index = invert_order(&rpo, total);

        let mut ipdoms: Vec<Option<BlockId>> = vec![None; total];
        ipdoms[virtual_exit as usize] = Some(virtual_exit);

        let mut changed = true;
        while changed {
            changed = false;
            for &b in rpo.iter().skip(1) {
                let bi = b as usize;
                let mut new_ipdom: Option<BlockId> = None;
                for &p in &rev_preds[bi] {
                    if ipdoms[p as usize].is_some() {
                        new_ipdom = Some(match new_ipdom {
                            None => p,
                            Some(cur) => intersect(p, cur, &ipdoms, &rpo_index),
                        });
                    }
                }
                if ipdoms[bi] != new_ipdom {
                    ipdoms[bi] = new_ipdom;
                    changed = true;
                }
            }
        }

        Self {
            ipdoms,
            n_blocks: n,
        }
    }

    /// Immediate post-dominator state of `b`. See [`PostDom`] for the
    /// three possible outcomes.
    #[must_use]
    pub fn ipostdom(&self, b: BlockId) -> PostDom {
        if (b as usize) >= self.n_blocks {
            return PostDom::Unreachable;
        }
        let virtual_exit = self.n_blocks as BlockId;
        match self.ipdoms[b as usize] {
            None => PostDom::Unreachable,
            Some(p) if p == virtual_exit => PostDom::Exit,
            Some(p) => PostDom::Block(p),
        }
    }

    /// Does `a` post-dominate `b`? Reflexive: `post_dominates(a, a)` is
    /// always true. Returns false when `a != b` and there is no chain
    /// of immediate post-dominators leading from `b` through `a`.
    #[must_use]
    pub fn post_dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        if (a as usize) >= self.n_blocks || (b as usize) >= self.n_blocks {
            return false;
        }
        let virtual_exit = self.n_blocks as BlockId;
        let mut cur = b;
        loop {
            let Some(p) = self.ipdoms[cur as usize] else {
                return false;
            };
            if p == a {
                return true;
            }
            if p == virtual_exit {
                return false;
            }
            if p == cur {
                return false;
            }
            cur = p;
        }
    }
}

/// Forward predecessor lists indexed by `BlockId`. Each list is sorted
/// and de-duplicated so the caller never has to worry about parallel
/// edges with different [`EdgeKind`]s collapsing or re-ordering.
pub(crate) fn predecessors_of(cfg: &Cfg) -> Vec<Vec<BlockId>> {
    let mut preds: Vec<Vec<BlockId>> = vec![Vec::new(); cfg.blocks.len()];
    for e in &cfg.edges {
        preds[e.to as usize].push(e.from);
    }
    for list in &mut preds {
        list.sort_unstable();
        list.dedup();
    }
    preds
}

/// Forward successor lists indexed by `BlockId`. Sorted and
/// de-duplicated; see [`predecessors_of`].
pub(crate) fn successors_of(cfg: &Cfg) -> Vec<Vec<BlockId>> {
    let mut succs: Vec<Vec<BlockId>> = vec![Vec::new(); cfg.blocks.len()];
    for e in &cfg.edges {
        succs[e.from as usize].push(e.to);
    }
    for list in &mut succs {
        list.sort_unstable();
        list.dedup();
    }
    succs
}

/// Reverse postorder over an arbitrary directed graph, starting at
/// `start` and following `succs[node]` for each node. Iterative; safe
/// on deep CFGs.
fn compute_rpo(start: BlockId, succs: &[Vec<BlockId>], total: usize) -> Vec<BlockId> {
    if total == 0 || (start as usize) >= total {
        return Vec::new();
    }
    let mut visited = vec![false; total];
    let mut post: Vec<BlockId> = Vec::new();
    let mut work: Vec<(BlockId, usize)> = Vec::new();

    visited[start as usize] = true;
    work.push((start, 0));

    while let Some((node, idx)) = work.pop() {
        let s = &succs[node as usize];
        if idx < s.len() {
            let next = s[idx];
            work.push((node, idx + 1));
            if !visited[next as usize] {
                visited[next as usize] = true;
                work.push((next, 0));
            }
        } else {
            post.push(node);
        }
    }

    post.reverse();
    post
}

fn invert_order(rpo: &[BlockId], total: usize) -> Vec<Option<usize>> {
    let mut out = vec![None; total];
    for (i, &b) in rpo.iter().enumerate() {
        out[b as usize] = Some(i);
    }
    out
}

/// The Cooper-Harvey-Kennedy `intersect` step. Walks the two finger
/// pointers up their respective immediate-dominator chains until they
/// meet at the nearest common ancestor.
fn intersect(
    b1: BlockId,
    b2: BlockId,
    idoms: &[Option<BlockId>],
    rpo_index: &[Option<usize>],
) -> BlockId {
    let mut f1 = b1;
    let mut f2 = b2;
    while f1 != f2 {
        while rpo_at(f1, rpo_index) > rpo_at(f2, rpo_index) {
            let Some(p) = idoms[f1 as usize] else {
                return f2;
            };
            if p == f1 {
                break;
            }
            f1 = p;
        }
        while rpo_at(f2, rpo_index) > rpo_at(f1, rpo_index) {
            let Some(p) = idoms[f2 as usize] else {
                return f1;
            };
            if p == f2 {
                break;
            }
            f2 = p;
        }
        if f1 == f2 {
            break;
        }
        // If the two fingers carry the same RPO index but are different
        // nodes (can happen across unreachable blocks with sentinel
        // `usize::MAX`), step the larger-id one up by its idom to make
        // progress without overshooting either chain.
        let Some(p1) = idoms[f1 as usize] else {
            return f2;
        };
        if p1 == f1 {
            return f2;
        }
        f1 = p1;
    }
    f1
}

fn rpo_at(b: BlockId, rpo_index: &[Option<usize>]) -> usize {
    rpo_index
        .get(b as usize)
        .copied()
        .flatten()
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::EdgeKind;
    use crate::test_support::synthetic_cfg as cfg;

    // ---------- DominatorTree ----------

    #[test]
    fn dom_01_single_block_dominates_itself() {
        let cfg = cfg(1, 0, &[]);
        let doms = DominatorTree::build(&cfg);
        assert_eq!(doms.idom(0), Some(0));
        assert!(doms.dominates(0, 0));
        assert!(!doms.strictly_dominates(0, 0));
    }

    #[test]
    fn dom_02_diamond_idoms_converge_at_entry() {
        // 0 → 1, 0 → 2, 1 → 3, 2 → 3
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        assert_eq!(doms.idom(0), Some(0));
        assert_eq!(doms.idom(1), Some(0));
        assert_eq!(doms.idom(2), Some(0));
        assert_eq!(doms.idom(3), Some(0));
        assert!(doms.dominates(0, 3));
        assert!(!doms.dominates(1, 3));
        assert!(!doms.dominates(2, 3));
    }

    #[test]
    fn dom_03_chain_idoms_chain_back() {
        // 0 → 1 → 2 → 3
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        assert_eq!(doms.idom(0), Some(0));
        assert_eq!(doms.idom(1), Some(0));
        assert_eq!(doms.idom(2), Some(1));
        assert_eq!(doms.idom(3), Some(2));
        assert!(doms.strictly_dominates(0, 3));
        assert!(doms.strictly_dominates(1, 3));
        assert!(doms.strictly_dominates(2, 3));
    }

    #[test]
    fn dom_04_unreachable_block_has_no_idom() {
        // 0 → 2; block 1 is orphaned.
        let cfg = cfg(3, 0, &[(0, 2, EdgeKind::Branch)]);
        let doms = DominatorTree::build(&cfg);
        assert_eq!(doms.idom(0), Some(0));
        assert_eq!(doms.idom(1), None);
        assert_eq!(doms.idom(2), Some(0));
        // Out-of-range queries return false without panicking.
        assert!(!doms.dominates(5, 0));
        assert!(!doms.dominates(0, 5));
    }

    #[test]
    fn dom_05_children_of_root_are_immediate_descendants() {
        // 0 → 1, 0 → 2, 1 → 3
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Fall),
            ],
        );
        let doms = DominatorTree::build(&cfg);
        // 0 dominates 1, 2, 3 transitively, but 3's only predecessor
        // is 1, so its immediate dominator is 1. Immediate children of
        // 0 are therefore {1, 2}; block 1 has {3} as its child.
        assert_eq!(doms.children(0), vec![1, 2]);
        assert_eq!(doms.children(1), vec![3]);
        assert_eq!(doms.children(2), Vec::<BlockId>::new());
        assert_eq!(doms.idom(3), Some(1));
    }

    #[test]
    fn dom_06_loop_header_dominates_body() {
        // 0 → 1; 1 → 2 (body); 2 → 1 (back-edge); 1 → 3 (exit).
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
        let doms = DominatorTree::build(&cfg);
        assert!(doms.dominates(1, 2));
        assert!(doms.dominates(1, 3));
        assert!(!doms.dominates(2, 1));
    }

    // ---------- PostDominatorTree ----------

    #[test]
    fn pdom_01_diamond_merges_at_join() {
        // Same diamond: 3 post-dominates 0, 1, 2.
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::NotTaken),
                (0, 2, EdgeKind::Taken),
                (1, 3, EdgeKind::Branch),
                (2, 3, EdgeKind::Fall),
            ],
        );
        let pdoms = PostDominatorTree::build(&cfg);
        assert_eq!(pdoms.ipostdom(0), PostDom::Block(3));
        assert_eq!(pdoms.ipostdom(1), PostDom::Block(3));
        assert_eq!(pdoms.ipostdom(2), PostDom::Block(3));
        assert_eq!(pdoms.ipostdom(3), PostDom::Exit);
        assert!(pdoms.post_dominates(3, 0));
        assert!(pdoms.post_dominates(3, 1));
        assert!(!pdoms.post_dominates(1, 0));
        assert!(!pdoms.post_dominates(2, 0));
    }

    #[test]
    fn pdom_02_infinite_loop_block_has_no_postdom() {
        // Block 0 has a self-edge only; no path to any exit.
        let cfg = cfg(1, 0, &[(0, 0, EdgeKind::Branch)]);
        let pdoms = PostDominatorTree::build(&cfg);
        assert_eq!(pdoms.ipostdom(0), PostDom::Unreachable);
        assert!(pdoms.post_dominates(0, 0)); // reflexive
        assert!(!pdoms.post_dominates(0, 1)); // out-of-range
    }

    #[test]
    fn pdom_03_chain_postdoms_chain_forward() {
        // 0 → 1 → 2 → 3 (exit)
        let cfg = cfg(
            4,
            0,
            &[
                (0, 1, EdgeKind::Fall),
                (1, 2, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
            ],
        );
        let pdoms = PostDominatorTree::build(&cfg);
        assert_eq!(pdoms.ipostdom(0), PostDom::Block(1));
        assert_eq!(pdoms.ipostdom(1), PostDom::Block(2));
        assert_eq!(pdoms.ipostdom(2), PostDom::Block(3));
        assert_eq!(pdoms.ipostdom(3), PostDom::Exit);
    }

    #[test]
    fn pdom_04_two_exits_have_no_common_real_postdom() {
        // 0 branches to two distinct exits 1 and 2.
        // Neither 1 nor 2 post-dominates 0 — 0's only post-dominator
        // is the virtual exit.
        let cfg = cfg(3, 0, &[(0, 1, EdgeKind::NotTaken), (0, 2, EdgeKind::Taken)]);
        let pdoms = PostDominatorTree::build(&cfg);
        assert_eq!(pdoms.ipostdom(0), PostDom::Exit);
        assert_eq!(pdoms.ipostdom(1), PostDom::Exit);
        assert_eq!(pdoms.ipostdom(2), PostDom::Exit);
    }
}
