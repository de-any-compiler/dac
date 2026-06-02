//! One-off stat dump used while writing the B1.5 CHANGELOG entry.
//! Run with `cargo run -p dac-recovery --example discovery_stats`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use dac_arch::Architecture;
use dac_arch_x86::X86_64;
use dac_binfmt::{BinaryModel, Section, SymbolKind};
use dac_core::EvidenceGraph;
use dac_recovery::{discover_functions, SourceMask};

fn ground(model: &BinaryModel) -> BTreeSet<u64> {
    let exec: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();
    model
        .symbols
        .iter()
        .filter(|s| {
            s.kind == SymbolKind::Text
                && !s.undefined
                && s.address != 0
                && exec.iter().any(|sec| {
                    let st = sec.address;
                    let en = st.saturating_add(sec.size);
                    s.address >= st && s.address < en
                })
        })
        .map(|s| s.address)
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures");
    for fixture in [
        "hello-x86_64",
        "hello-x86_64.exe",
        "hello-x86_64-stripped",
        "hello-x86_64-stripped.exe",
    ] {
        let bytes = std::fs::read(root.join(fixture)).expect("read");
        let model = dac_binfmt::load_from_bytes(&bytes).expect("parse");
        let arch = X86_64;
        let dec = arch.decoder();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &bytes, dec.as_ref(), &mut g);
        let gt = ground(&model);
        let discovered: BTreeSet<u64> = set.addresses().collect();
        let hit = gt.intersection(&discovered).count();
        let recall = if gt.is_empty() {
            f64::NAN
        } else {
            (hit as f64) / (gt.len() as f64)
        };
        let merges = set
            .functions
            .iter()
            .filter(|f| {
                let mut c = 0;
                if f.sources.contains(SourceMask::SYMBOL) {
                    c += 1
                }
                if f.sources.contains(SourceMask::ENTRY) {
                    c += 1
                }
                if f.sources.contains(SourceMask::CALL) {
                    c += 1
                }
                if f.sources.contains(SourceMask::PROLOGUE) {
                    c += 1
                }
                c >= 2
            })
            .count();
        println!(
            "{fixture}: discovered={} ground={} recall={:.3} stats={:?} merges>=2sig={}",
            set.functions.len(),
            gt.len(),
            recall,
            set.stats,
            merges,
        );
    }
}
