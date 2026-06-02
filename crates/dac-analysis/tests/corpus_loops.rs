//! Corpus invariants for B2.2 (FR-10).
//!
//! Runs the dominator + loop-forest passes over every function in the
//! shared fixture corpus and asserts the structural invariants that
//! [`dac_analysis::dom::DominatorTree`] and
//! [`dac_analysis::loops::LoopForest`] are supposed to guarantee:
//!
//! - Every block in a loop body is dominated by the loop's header.
//! - Every back-edge source is dominated by its header.
//! - The forest is byte-stable across re-runs (NFR-9).
//!
//! The unit tests in `crates/dac-analysis/src/loops.rs` cover the
//! hand-checked reference topologies (linear, self-loop, while-style,
//! do-while, nested, sibling, multi-back-edge, irreducible, etc.); this
//! file ensures the same builder survives byte-level CFGs from real
//! ELFs and PEs.

use std::path::PathBuf;

use dac_analysis::cfg::build_cfgs;
use dac_analysis::dom::DominatorTree;
use dac_analysis::loops::LoopForest;
use dac_arch::Architecture as _;
use dac_arch_x86::X86_64;
use dac_binfmt::load_from_bytes;
use dac_core::EvidenceGraph;
use dac_recovery::discover_functions;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn analyse(fixture: &str) -> Vec<LoopForest> {
    let bytes = std::fs::read(fixture_path(fixture)).expect("read fixture");
    let model = load_from_bytes(&bytes).expect("load binary model");
    let decoder = X86_64.decoder();
    let mut graph = EvidenceGraph::new();
    let functions = discover_functions(&model, &bytes, decoder.as_ref(), &mut graph);
    let cfgs = build_cfgs(&functions.functions, &model, &bytes, decoder.as_ref());

    let mut forests = Vec::with_capacity(cfgs.len());
    for cfg in &cfgs {
        let doms = DominatorTree::build(cfg);
        let forest = LoopForest::build(cfg, &doms);

        for l in &forest.loops {
            for &b in &l.body {
                assert!(
                    doms.dominates(l.header, b),
                    "{fixture}: loop {} header {} does not dominate body block {}",
                    l.id,
                    l.header,
                    b
                );
            }
            for &s in &l.back_edges {
                assert!(
                    doms.dominates(l.header, s),
                    "{fixture}: loop {} back-edge source {} is not dominated by header {}",
                    l.id,
                    s,
                    l.header
                );
            }
            assert!(
                l.body.binary_search(&l.header).is_ok(),
                "{fixture}: loop {} body must contain its header {}",
                l.id,
                l.header
            );
        }

        forests.push(forest);
    }

    forests
}

#[test]
fn elf_loop_invariants_hold() {
    let _ = analyse("hello-x86_64");
}

#[test]
fn pe_loop_invariants_hold() {
    let _ = analyse("hello-x86_64.exe");
}

#[test]
fn stripped_elf_loop_invariants_hold() {
    let _ = analyse("hello-x86_64-stripped");
}

#[test]
fn loop_forest_is_byte_stable_across_reruns() {
    // Determinism gate (NFR-9): same fixture, two runs, equal forests.
    let first = analyse("hello-x86_64");
    let second = analyse("hello-x86_64");
    assert_eq!(
        first, second,
        "loop forest drifted between two runs over the same input"
    );
}
