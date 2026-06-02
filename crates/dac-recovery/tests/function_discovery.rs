//! Integration test: function discovery on the workspace sample
//! corpus.
//!
//! Closes the B1.5 done-when ("function discovery matches symbol tables
//! ≥ 98% on the sample corpus"). The strict invariants are:
//!
//! - On the **unstripped** ELF and PE fixtures, every text-kind symbol
//!   with a defined address is rediscovered, so recall against the
//!   symbol-table ground truth clears `0.98`. Symbol-derived discovery
//!   alone makes this trivial; the test guards against future
//!   regressions that might drop symbol entries from the accumulator.
//! - On the **stripped** variants (symbols stripped), discovery still
//!   recovers a non-trivial set through the call-edge and prologue
//!   signals. The numbers are recorded by the test but not gated — the
//!   plan calls these out as "tracked but not gated".
//! - Independent signals routinely agree: the recall test asserts that
//!   the entry-point and at least one call-edge signal contribute on
//!   the unstripped fixtures, so a regression that disables a signal
//!   while keeping symbol-derived discovery alive still fails loudly.

use std::collections::BTreeSet;
use std::path::PathBuf;

use dac_arch::Architecture as _;
use dac_arch_x86::X86_64;
use dac_binfmt::{BinaryModel, Section, SymbolKind};
use dac_core::EvidenceGraph;
use dac_recovery::{discover_functions, FunctionSet, SourceMask};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load(name: &str) -> (Vec<u8>, BinaryModel) {
    let path = fixture_path(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    let model = dac_binfmt::load_from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()));
    (bytes, model)
}

fn discover(name: &str) -> (BinaryModel, FunctionSet) {
    let (bytes, model) = load(name);
    let arch = X86_64;
    let decoder = arch.decoder();
    let mut graph = EvidenceGraph::new();
    let set = discover_functions(&model, &bytes, decoder.as_ref(), &mut graph);
    (model, set)
}

/// Ground-truth function starts derived from the symbol table: every
/// `Text` symbol that resolves to a defined address inside an
/// executable section. Mirrors the same filter the discoverer uses
/// internally so the test compares like with like.
fn symbol_ground_truth(model: &BinaryModel) -> BTreeSet<u64> {
    let exec: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();
    let mut out = BTreeSet::new();
    for sym in &model.symbols {
        if sym.kind != SymbolKind::Text || sym.undefined || sym.address == 0 {
            continue;
        }
        if exec.iter().any(|s| {
            let start = s.address;
            let end = start.saturating_add(s.size);
            sym.address >= start && sym.address < end
        }) {
            out.insert(sym.address);
        }
    }
    out
}

fn recall(ground: &BTreeSet<u64>, set: &FunctionSet) -> f64 {
    if ground.is_empty() {
        return 1.0;
    }
    let discovered: BTreeSet<u64> = set.addresses().collect();
    let hit = ground.intersection(&discovered).count();
    (hit as f64) / (ground.len() as f64)
}

fn assert_unstripped_recall(fixture: &str) {
    let (model, set) = discover(fixture);
    let ground = symbol_ground_truth(&model);
    assert!(
        !ground.is_empty(),
        "{fixture}: expected symbol-table ground truth",
    );
    let r = recall(&ground, &set);
    assert!(
        r >= 0.98,
        "{fixture}: recall {r:.3} < 0.98 (ground={}, discovered={}, stats={:?})",
        ground.len(),
        set.functions.len(),
        set.stats,
    );

    // The discoverer should be picking up multiple independent signals
    // on every unstripped binary, so a future change that silently
    // disables one (e.g. the entry-point signal) does not slip past
    // the recall gate alone.
    assert!(
        set.stats.from_symbol > 0,
        "{fixture}: symbol signal silent ({:?})",
        set.stats,
    );
    assert!(
        set.stats.from_entry > 0,
        "{fixture}: entry signal silent ({:?})",
        set.stats,
    );
    assert!(
        set.stats.from_call > 0,
        "{fixture}: call signal silent ({:?})",
        set.stats,
    );

    // Confidence + source-mask invariants on the union.
    for f in &set.functions {
        assert!(
            !f.sources.is_empty(),
            "function at {:#x} has no sources",
            f.address
        );
        assert!(
            f.confidence.value() > 0.0,
            "function at {:#x} has zero confidence",
            f.address,
        );
    }
}

fn assert_stripped_signal(fixture: &str) {
    let (_model, set) = discover(fixture);
    // The "tracked but not gated" branch. We assert the discoverer
    // produces *some* output on a stripped binary — anything else
    // would mean both the entry-point and call-edge signals went
    // silent.
    assert!(
        !set.functions.is_empty(),
        "{fixture}: stripped discovery produced zero functions (stats={:?})",
        set.stats,
    );
    // Stripped fixtures keep their entry point, so the entry signal
    // must still contribute even when the symbol table is empty.
    assert!(
        set.stats.from_entry > 0,
        "{fixture}: entry signal silent on stripped fixture ({:?})",
        set.stats,
    );
}

#[test]
fn elf_unstripped_meets_recall_gate() {
    assert_unstripped_recall("hello-x86_64");
}

#[test]
fn pe_unstripped_meets_recall_gate() {
    assert_unstripped_recall("hello-x86_64.exe");
}

#[test]
fn elf_stripped_still_yields_functions() {
    assert_stripped_signal("hello-x86_64-stripped");
}

#[test]
fn pe_stripped_still_yields_functions() {
    assert_stripped_signal("hello-x86_64-stripped.exe");
}

#[test]
fn unstripped_functions_intersect_with_call_sources() {
    // Tighter check that the call-edge signal is actually hitting
    // symbol-known addresses — exercises the "merge" path inside the
    // discoverer (same address discovered via both symbol and call).
    let (_model, set) = discover("hello-x86_64");
    let merged = set
        .functions
        .iter()
        .filter(|f| f.sources.contains(SourceMask::SYMBOL) && f.sources.contains(SourceMask::CALL))
        .count();
    assert!(
        merged > 0,
        "no symbol-and-call merges observed; signals do not agree ({:?})",
        set.stats,
    );
}
