//! Per-function end-to-end lift orchestration (B3.9, FR-21) and
//! per-function recovery-fact bundling (B3.10, FR-14 / FR-16 / FR-17 /
//! FR-18).
//!
//! The CLI runs the deterministic pipeline once per recovered
//! function:
//!
//! ```text
//!   Function
//!     → build_cfg                  (dac-analysis, B1.7)
//!     → InstructionIr per block    (dac-arch-x86::IcedLifter, B1.4)
//!     → RawFunction                (dac-lift::lift_function, B3.8)
//!     → SsaFunction                (dac-analysis::ssa::construct_ssa, B2.3)
//!     → StackFrame                 (dac-recovery::stack, B2.4)
//!     → ConventionMatch            (dac-recovery::convention, B2.5)
//!     → TypeMap                    (dac-recovery::types, B2.6)
//!     → RecoveredStructs           (dac-recovery::structs, B3.2)
//!     → RecoveredIdioms            (dac-recovery::idioms, B3.3)
//!     → SemFunction                (dac-analysis::structuring, B2.7)
//!     → lower_switch_idioms        (B3.10 post-pass)
//! ```
//!
//! When any step short-circuits (no recovered `end`, `build_cfg`
//! returns `None`, etc.) the per-function outcome is a [`LiftOutcome::Stub`]
//! with a human-readable reason that the source-emitting code
//! surfaces in the leading comment (I-6: degrade visibly, never
//! invent semantics).
//!
//! ## Determinism
//!
//! Every constituent pass is `Determinism::Pure` (NFR-9). The
//! orchestrator iterates `FunctionSet::functions` in its existing
//! address-sorted order and threads the same register file into every
//! call, so two runs on the same bytes produce identical
//! `LiftOutcome` vectors.

use std::collections::BTreeMap;

use dac_analysis::cfg::build_cfg;
use dac_analysis::dom::{DominatorTree, PostDominatorTree};
use dac_analysis::loops::LoopForest;
use dac_analysis::ssa::construct_ssa;
use dac_analysis::structuring::structure;
use dac_arch::{InstructionDecoder, InstructionLifter, RegisterFile};
use dac_binfmt::{BinaryFormat, BinaryModel};
use dac_ir::instr::InstructionIr;
use dac_ir::sem::{Block as SemBlock, SemFunction, Stmt as SemStmt, SwitchArm};
use dac_ir::ssa::{Operand, SsaFunction};
use dac_knowledge::{lookup_api_signature, ApiSignature, X86_64_CONVENTIONS};
use dac_lift::lift_function;
use dac_recovery::{
    analyze_stack_frame, infer_calling_convention, propagate_types, recover_idioms,
    recover_structs, ApiResolver, ConventionMatch, Function, FunctionSet, RecoveredIdioms,
    RecoveredStructs, StackConvention, StackFrame, SwitchTableIdiom, TypeMap,
};

/// Per-function outcome of the orchestrator.
pub(crate) enum LiftOutcome {
    /// Pipeline ran end-to-end; both the SSA and Semantic IR
    /// representations are populated, alongside the recovery-side-
    /// table facts (B3.10). The facts bundle is boxed to keep the
    /// enum's `Real` variant from dominating sizeof in the
    /// `Vec<LiftOutcome>` the orchestrator returns (the `Stub` arm
    /// only carries a small `String`).
    Real {
        ssa: SsaFunction,
        sem: SemFunction,
        facts: Box<RecoveryFacts>,
    },
    /// Pipeline could not produce a Semantic IR function. `reason` is
    /// rendered into the leading comment of the emitted stub.
    Stub { reason: String },
}

/// Per-function recovery-side-table bundle threaded into the C
/// backend at B3.10 (FR-14 / FR-16 / FR-17 / FR-18).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecoveryFacts {
    pub stack_frame: StackFrame,
    pub convention: Option<ConventionMatch>,
    pub types: TypeMap,
    pub structs: RecoveredStructs,
    pub idioms: RecoveredIdioms,
}

/// Shared per-binary inputs to the orchestrator: bound up so
/// [`lift_one`] takes a single context reference instead of a long
/// argument list.
struct LiftCtx<'a> {
    model: &'a BinaryModel,
    bytes: &'a [u8],
    decoder: &'a dyn InstructionDecoder,
    lifter: &'a dyn InstructionLifter,
    register_file: &'a RegisterFile,
    stack_convention: StackConvention,
    api_resolver: BinaryImportResolver,
}

/// Run the per-function orchestrator across the whole recovered
/// function set. The returned vector is in the same order as
/// `functions.functions`, so callers can zip the two together.
#[must_use]
pub(crate) fn lift_all(
    functions: &FunctionSet,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    register_file: &RegisterFile,
) -> Vec<LiftOutcome> {
    let ctx = LiftCtx {
        model,
        bytes,
        decoder,
        lifter,
        register_file,
        stack_convention: stack_convention_for(model),
        api_resolver: BinaryImportResolver::new(model),
    };
    functions
        .functions
        .iter()
        .map(|f| lift_one(f, &ctx))
        .collect()
}

/// Pick the stack convention for the binary's format. ELF and Mach-O
/// follow SysV-AMD64 on x86-64; PE follows MS-X64. The convention
/// inference layer (B2.5) then picks the *calling* convention from a
/// menu using observed register / stack signals — the *stack*
/// convention is purely about where the immediate frame's home slots
/// live.
fn stack_convention_for(model: &BinaryModel) -> StackConvention {
    match model.format {
        BinaryFormat::Pe => StackConvention::MsX64,
        _ => StackConvention::SysVAmd64,
    }
}

/// Aggregate lift statistics. The CLI threads this into the
/// `--emit-report` output so a reader can tell how much of the binary
/// the deterministic pipeline reconstructed end-to-end.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LiftStats {
    pub real: u64,
    pub stub: u64,
    /// Functions whose body lowered with a recovered convention
    /// (i.e. signature has at least one inferred int arg or a return
    /// register). B3.10 surfaces this in `--emit-report`.
    pub typed_signatures: u64,
    /// Functions in which at least one Load / Store address matched a
    /// recovered struct-field shape (B3.10 lowering hit).
    pub struct_field_functions: u64,
    /// Functions in which at least one recognised
    /// [`SwitchTableIdiom`] was lowered to `Stmt::Switch` (B3.10).
    pub switch_functions: u64,
}

impl LiftStats {
    pub(crate) fn from(outcomes: &[LiftOutcome]) -> Self {
        let mut s = Self::default();
        for o in outcomes {
            match o {
                LiftOutcome::Real { facts, .. } => {
                    s.real += 1;
                    if recovered_convention_is_useful(facts.convention.as_ref()) {
                        s.typed_signatures += 1;
                    }
                    if !facts.structs.pointer_structs.is_empty() {
                        s.struct_field_functions += 1;
                    }
                    if !facts.idioms.switch_tables.is_empty() {
                        s.switch_functions += 1;
                    }
                }
                LiftOutcome::Stub { .. } => s.stub += 1,
            }
        }
        s
    }

    pub(crate) fn total(self) -> u64 {
        self.real + self.stub
    }

    pub(crate) fn fraction(self) -> f32 {
        let t = self.total();
        if t == 0 {
            0.0
        } else {
            self.real as f32 / t as f32
        }
    }
}

fn recovered_convention_is_useful(c: Option<&ConventionMatch>) -> bool {
    match c {
        Some(c) => !c.signature.int_args.is_empty() || c.signature.return_register.is_some(),
        None => false,
    }
}

fn lift_one(f: &Function, ctx: &LiftCtx<'_>) -> LiftOutcome {
    if f.end.is_none() {
        return LiftOutcome::Stub {
            reason: "no recovered end address".into(),
        };
    }
    let Some(cfg) = build_cfg(f, ctx.model, ctx.bytes, ctx.decoder) else {
        return LiftOutcome::Stub {
            reason: "cfg-build failed (byte range unreachable or empty)".into(),
        };
    };

    let instructions_per_block: Vec<Vec<InstructionIr>> = cfg
        .blocks
        .iter()
        .map(|b| {
            b.instructions
                .iter()
                .map(|d| ctx.lifter.lift(&d.bytes, d.address))
                .collect()
        })
        .collect();

    let raw = lift_function(&cfg, &instructions_per_block, ctx.register_file);
    let doms = DominatorTree::build(&cfg);
    let ssa = construct_ssa(&cfg, &doms, &raw);
    let pdoms = PostDominatorTree::build(&cfg);
    let loops = LoopForest::build(&cfg, &doms);

    // B3.10: run the recovery side-table passes here so the C backend
    // can consume their results. Each pass is independent and pure;
    // ordering follows the data dependencies — stack frame seeds
    // convention, both seed types, types seeds structs.
    let stack_frame = analyze_stack_frame(&ssa, ctx.stack_convention);
    let convention = infer_calling_convention(&ssa, &stack_frame, X86_64_CONVENTIONS)
        .into_iter()
        .next();
    let signature = convention.as_ref().map(|c| &c.signature);
    let types = propagate_types(&ssa, signature, Some(&stack_frame), &ctx.api_resolver);
    let structs = recover_structs(&ssa, Some(&stack_frame), Some(&types));
    let idioms = recover_idioms(&ssa);

    let sem = structure(&ssa, &cfg, &doms, &pdoms, &loops);
    let sem = lower_switch_idioms(sem, &idioms);

    let facts = Box::new(RecoveryFacts {
        stack_frame,
        convention,
        types,
        structs,
        idioms,
    });
    LiftOutcome::Real { ssa, sem, facts }
}

/// Post-pass on the structurer output: rewrite each
/// [`SemStmt::Unreachable`] whose source block matches a recognised
/// [`SwitchTableIdiom`] into [`SemStmt::Switch`].
///
/// **Scope at B3.10.** Arms are left empty and the default body
/// preserves the `Unreachable` shape; per-entry resolution that
/// reads `.rodata` (and PE relocations) and mints labelled goto
/// arms is on the B3 follow-up shelf. The visible change is that
/// the C backend now emits `switch (scrutinee) { default: __builtin_unreachable(); }`
/// with a comment describing the recovered table — instead of a bare
/// `__builtin_unreachable();` — so a reader sees the recognised
/// idiom even when the arms cannot yet be materialised (I-6).
fn lower_switch_idioms(mut sem: SemFunction, idioms: &RecoveredIdioms) -> SemFunction {
    if idioms.switch_tables.is_empty() {
        return sem;
    }
    rewrite_block(&mut sem.body, &idioms.switch_tables);
    sem
}

fn rewrite_block(
    block: &mut SemBlock,
    tables: &BTreeMap<dac_ir::ssa::SsaBlockId, SwitchTableIdiom>,
) {
    for stmt in block.stmts.iter_mut() {
        rewrite_stmt(stmt, tables);
    }
}

fn rewrite_stmt(stmt: &mut SemStmt, tables: &BTreeMap<dac_ir::ssa::SsaBlockId, SwitchTableIdiom>) {
    match stmt {
        SemStmt::Unreachable {
            source_block,
            evidence,
        } => {
            if let Some(table) = tables.get(source_block) {
                let scrutinee = Operand::Value(table.scrutinee);
                let mut default = SemBlock::empty();
                default.stmts.push(SemStmt::Unreachable {
                    source_block: *source_block,
                    evidence: *evidence,
                });
                *stmt = SemStmt::Switch {
                    scrutinee,
                    arms: Vec::<SwitchArm>::new(),
                    default: Some(default),
                    source_block: *source_block,
                    evidence: *evidence,
                };
            }
        }
        SemStmt::If {
            then_body,
            else_body,
            ..
        } => {
            rewrite_block(then_body, tables);
            if let Some(eb) = else_body.as_mut() {
                rewrite_block(eb, tables);
            }
        }
        SemStmt::While { body, .. }
        | SemStmt::DoWhile { body, .. }
        | SemStmt::Loop { body, .. } => {
            rewrite_block(body, tables);
        }
        SemStmt::Switch { arms, default, .. } => {
            for arm in arms.iter_mut() {
                rewrite_block(&mut arm.body, tables);
            }
            if let Some(d) = default.as_mut() {
                rewrite_block(d, tables);
            }
        }
        _ => {}
    }
}

/// `ApiResolver` backed by the binary's import / symbol table. Only
/// direct calls whose target VA exactly matches an imported function
/// (or a non-import named symbol that resolves to a known API) bind
/// to a signature. PLT-stub resolution lives in the PE / ELF
/// binfmt layer; the resolver consults pre-built reverse maps so the
/// lookup is `O(log n)`.
struct BinaryImportResolver {
    /// Map from import-target VA to signature.
    imports_by_va: BTreeMap<u64, &'static ApiSignature>,
    /// Map from imported / exported symbol name to signature, used
    /// when the call site decodes a VA that lands on a named symbol
    /// (e.g. a direct call into libc statically linked in).
    name_index: BTreeMap<u64, &'static ApiSignature>,
}

impl BinaryImportResolver {
    fn new(model: &BinaryModel) -> Self {
        let mut imports_by_va: BTreeMap<u64, &'static ApiSignature> = BTreeMap::new();
        // The `Import` records do not carry a VA on every format; the
        // PLT-stub VA lives on the matching `Symbol` entry produced
        // by the binfmt bridge (`object` exposes the dynsym stub via
        // the section table). We index by name to recover the VA
        // from `Symbol::address` below.
        let imports_by_name: BTreeMap<&str, &'static ApiSignature> = model
            .imports
            .iter()
            .filter_map(|imp| lookup_api_signature(&imp.name).map(|sig| (imp.name.as_str(), sig)))
            .collect();
        let mut name_index: BTreeMap<u64, &'static ApiSignature> = BTreeMap::new();
        for sym in &model.symbols {
            if sym.address == 0 {
                continue;
            }
            if let Some(sig) = imports_by_name.get(sym.name.as_str()) {
                imports_by_va.insert(sym.address, *sig);
            }
            if let Some(sig) = lookup_api_signature(&sym.name) {
                name_index.insert(sym.address, sig);
            }
        }
        Self {
            imports_by_va,
            name_index,
        }
    }
}

impl ApiResolver for BinaryImportResolver {
    fn resolve(&self, target_va: u64) -> Option<&'static ApiSignature> {
        self.imports_by_va
            .get(&target_va)
            .copied()
            .or_else(|| self.name_index.get(&target_va).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_recovery::{StackFrame, TypeMap};

    fn dummy_ssa() -> SsaFunction {
        SsaFunction {
            function_address: 0,
            function_name: None,
            blocks: Vec::new(),
            entry: 0,
            variables: Vec::new(),
            values: Vec::new(),
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Ssa,
                id: 0,
            }),
        }
    }

    fn dummy_sem() -> SemFunction {
        SemFunction {
            function_address: 0,
            function_name: None,
            body: SemBlock::empty(),
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Semantic,
                id: 0,
            }),
            stats: dac_ir::sem::StructuringStats::default(),
        }
    }

    fn facts_default() -> RecoveryFacts {
        RecoveryFacts {
            stack_frame: StackFrame {
                convention: StackConvention::SysVAmd64,
                stack_pointer: None,
                frame_pointer: None,
                locals: BTreeMap::new(),
                confidence: dac_core::Confidence::new(0.0, dac_core::Source::Derived),
            },
            convention: None,
            types: TypeMap::default(),
            structs: RecoveredStructs::default(),
            idioms: RecoveredIdioms::default(),
        }
    }

    #[test]
    fn lift_stats_round_trip() {
        let outcomes = vec![
            LiftOutcome::Stub {
                reason: "r1".into(),
            },
            LiftOutcome::Real {
                ssa: dummy_ssa(),
                sem: dummy_sem(),
                facts: Box::new(facts_default()),
            },
        ];
        let s = LiftStats::from(&outcomes);
        assert_eq!(s.real, 1);
        assert_eq!(s.stub, 1);
        assert_eq!(s.total(), 2);
        assert!((s.fraction() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn empty_outcomes_have_zero_fraction() {
        let s = LiftStats::from(&[]);
        assert_eq!(s.total(), 0);
        assert_eq!(s.fraction(), 0.0);
    }
}
