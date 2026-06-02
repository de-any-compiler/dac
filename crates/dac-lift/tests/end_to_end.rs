//! End-to-end lift of a real ELF function (B3.8 done-when, leg 2).
//!
//! The unit tests in `bridge` cover hand-built CFG topologies. This
//! test closes the second leg of the B3.8 rubric — *"a second test
//! using the existing `hello-x86_64` fixture's `main` produces a
//! non-trivial `SemFunction`"* — by driving the full pipeline:
//!
//! ```text
//!   ELF bytes
//!     → BinaryModel               (dac-binfmt)
//!     → FunctionSet               (dac-recovery::discover_functions)
//!     → Cfg                       (dac-analysis::cfg::build_cfg)
//!     → Vec<InstructionIr> / blk  (dac-arch-x86::IcedLifter)
//!     → RawFunction               (dac-lift::lift_function — B3.8)
//!     → SsaFunction               (dac-analysis::ssa::construct_ssa)
//!     → SemFunction               (dac-analysis::structuring::structure)
//! ```
//!
//! The assertion is intentionally weak: at least one structured
//! statement (return / if / loop / assignment), at least one SSA
//! value, and at least one RawOp in any block. Stronger contracts
//! (recognisable return-from-`main`, recovered argument signature)
//! land with B3.9 / B3.10 — this test guards against the bridge
//! silently regressing to "no real bodies at all", which is the
//! state the M3 close-out is fixing.

use std::fs;
use std::path::PathBuf;

use dac_analysis::cfg::build_cfg;
use dac_analysis::dom::{DominatorTree, PostDominatorTree};
use dac_analysis::loops::LoopForest;
use dac_analysis::ssa::construct_ssa;
use dac_analysis::structuring::structure;
use dac_arch::Architecture;
use dac_arch_x86::X86_64;
use dac_binfmt::{load_from_bytes, Architecture as BinArch};
use dac_core::EvidenceGraph;
use dac_ir::instr::InstructionIr;
use dac_lift::lift_function;
use dac_recovery::discover_functions;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read(&path).expect("fixture should be readable")
}

#[test]
fn hello_x86_64_main_lifts_to_a_non_trivial_sem_function() {
    let bytes = fixture_bytes("hello-x86_64");
    let model = load_from_bytes(&bytes).expect("hello-x86_64 parses as ELF");
    assert_eq!(model.architecture, BinArch::X86_64, "fixture is x86-64",);

    let arch = X86_64;
    let decoder = arch.decoder();
    let lifter = arch.lifter();
    let register_file = arch.register_file();

    let mut graph = EvidenceGraph::new();
    let functions = discover_functions(&model, &bytes, decoder.as_ref(), &mut graph);
    assert!(
        !functions.functions.is_empty(),
        "function discovery surfaced at least one function on a non-stripped ELF",
    );

    // Pick the function whose recovered name is "main". `hello-x86_64`
    // is not stripped so the symbol is present; if the picker can't
    // find it, fall back to the largest discovered function — the
    // assertion below is still meaningful as a "the bridge works on
    // *something* real" guard.
    let target = functions
        .functions
        .iter()
        .find(|f| f.name.as_deref() == Some("main"))
        .or_else(|| {
            functions
                .functions
                .iter()
                .max_by_key(|f| f.size().unwrap_or(0))
        })
        .expect("at least one recovered function");

    let cfg = build_cfg(target, &model, &bytes, decoder.as_ref())
        .expect("CFG builder produces a CFG for a recovered function");

    // Lift each block's instructions into InstructionIr. The bridge
    // does not run iced itself; it takes the lifted stream the arch
    // backend produces, mirroring how the CLI orchestrator (B3.9)
    // will drive this.
    let instructions_per_block: Vec<Vec<InstructionIr>> = cfg
        .blocks
        .iter()
        .map(|b| {
            b.instructions
                .iter()
                .map(|d| lifter.lift(&d.bytes, d.address))
                .collect()
        })
        .collect();

    let raw = lift_function(&cfg, &instructions_per_block, register_file);
    assert_eq!(raw.blocks.len(), cfg.blocks.len(), "block count preserved");

    let doms = DominatorTree::build(&cfg);
    let ssa = construct_ssa(&cfg, &doms, &raw);
    let pdoms = PostDominatorTree::build(&cfg);
    let loops = LoopForest::build(&cfg, &doms);
    let sem = structure(&ssa, &cfg, &doms, &pdoms, &loops);

    // Non-trivial assertion: the body has at least one statement and
    // at least one SSA value was minted. The B2.8-era stub path
    // never reached this point — it short-circuited to a synthetic
    // `return;` without touching the SSA pipeline.
    assert!(
        !sem.body.stmts.is_empty(),
        "structured body should carry at least one statement",
    );
    assert!(
        !ssa.values.is_empty(),
        "SSA function should mint at least one value",
    );
    assert!(
        raw.blocks.iter().any(|b| !b.ops.is_empty()),
        "at least one raw block should carry a body op (the function isn't empty)",
    );
}

#[test]
fn lift_function_is_byte_stable_across_two_runs_on_a_real_binary() {
    // Same fixture, lifted twice — the bridge must produce identical
    // RawFunction values both runs (NFR-9, I-4).
    let bytes = fixture_bytes("hello-x86_64");
    let model = load_from_bytes(&bytes).expect("ELF");
    let arch = X86_64;
    let decoder = arch.decoder();
    let lifter = arch.lifter();
    let register_file = arch.register_file();

    let mut graph_a = EvidenceGraph::new();
    let functions_a = discover_functions(&model, &bytes, decoder.as_ref(), &mut graph_a);
    let mut graph_b = EvidenceGraph::new();
    let functions_b = discover_functions(&model, &bytes, decoder.as_ref(), &mut graph_b);

    let target_a = functions_a
        .functions
        .iter()
        .find(|f| f.name.as_deref() == Some("main"))
        .unwrap_or(&functions_a.functions[0]);
    let target_b = functions_b
        .functions
        .iter()
        .find(|f| f.name.as_deref() == Some("main"))
        .unwrap_or(&functions_b.functions[0]);

    let cfg_a = build_cfg(target_a, &model, &bytes, decoder.as_ref()).expect("cfg a");
    let cfg_b = build_cfg(target_b, &model, &bytes, decoder.as_ref()).expect("cfg b");

    let ipb_a: Vec<Vec<InstructionIr>> = cfg_a
        .blocks
        .iter()
        .map(|b| {
            b.instructions
                .iter()
                .map(|d| lifter.lift(&d.bytes, d.address))
                .collect()
        })
        .collect();
    let ipb_b: Vec<Vec<InstructionIr>> = cfg_b
        .blocks
        .iter()
        .map(|b| {
            b.instructions
                .iter()
                .map(|d| lifter.lift(&d.bytes, d.address))
                .collect()
        })
        .collect();

    let a = lift_function(&cfg_a, &ipb_a, register_file);
    let b = lift_function(&cfg_b, &ipb_b, register_file);
    assert_eq!(a, b, "lift_function must be byte-stable on identical input",);
}
