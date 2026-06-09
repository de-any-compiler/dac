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
use dac_backend_c::{CType, SynthesizedParam, SynthesizedSignature};
use dac_binfmt::{elf_x86_64_plt_stubs, BinaryFormat, BinaryModel};
use dac_core::{Confidence, EvidenceGraph, EvidenceNode, Source};
use dac_hints::{HintId, Hints};
use dac_ir::instr::InstructionIr;
use dac_ir::sem::{Block as SemBlock, SemFunction, Stmt as SemStmt, SwitchArm};
use dac_ir::ssa::{Operand, SsaFunction, SsaTerminator};
use dac_knowledge::{
    lookup_api_signature, lookup_canonical_entry, x86_64_convention_by_name, ApiSignature,
};
use dac_lift::lift_function;
use dac_recovery::{
    analyze_stack_frame, candidates_for, infer_calling_convention, propagate_types, recover_idioms,
    recover_names, recover_structs, resolve_switch_entries, simplify, ApiResolver,
    CallRenameResolver, ConventionMatch, Function, FunctionSet, LoopInfo, LoopShape, NameTable,
    RecoveredIdioms, RecoveredStructs, RegisterArg, SimplifyStats, StackConvention, StackFrame,
    StringResolver, SwitchTableIdiom, TypeMap, ValueType,
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
///
/// B3.6 adds the optional [`RecoveryFacts::user_hint`] field. When a
/// `--hints` file entry matched the function, the recovery passes'
/// outputs were overlaid in place: the [`TypeMap`] now carries
/// [`Source::UserHint`] confidences for the hinted argument /
/// return values, and `rename` (when set) supersedes the recovered
/// symbol on emit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecoveryFacts {
    pub stack_frame: StackFrame,
    pub convention: Option<ConventionMatch>,
    pub types: TypeMap,
    pub structs: RecoveredStructs,
    pub idioms: RecoveredIdioms,
    pub user_hint: Option<AppliedHint>,
    /// Variable-naming heuristic table (B3.7, FR-N spec §11.1).
    /// Maps SSA `ValueId`s to heuristic identifiers (`path`, `fmt`,
    /// `str_hello`, …) the C backend renders in place of `v<id>`.
    pub names: NameTable,
    /// Per-function simplifier counters (B3.26). Surfaces in
    /// `--emit-report` so a reader sees how many dead pure ops the
    /// pre-emit pass removed from the function. `Default` zeros for
    /// fixtures and tests that do not run the simplifier.
    pub simplify: SimplifyStats,
    /// Per-function C-canonical signature override (B3.28). When
    /// `Some`, the C backend prints the override's spellings instead
    /// of the convention-inferred shape — so `main` reads as
    /// `int main(int argc, char **argv)` instead of `int64_t main(…)`.
    /// Built by [`apply_canonical_entry`] when the function name
    /// matches the curated entry-point catalogue, and may be amended
    /// later by [`apply_function_hint`] when the hint declares arg
    /// slots past the convention-observed prefix (FR-12, FR-20, FR-21).
    pub canonical_signature: Option<SynthesizedSignature>,
    /// Width-derived return-type inference (B3.29, FR-18 / FR-21).
    /// Computed by [`infer_return_type`] after the convention pass
    /// pinned the return register. The C backend's `pick_return_type`
    /// consults this *after* the canonical override and *before* the
    /// lattice-driven path: a function the inference pass classified
    /// as `void` no longer renders as the legacy `int64_t` shape, and
    /// a function whose widest write to the return register is 32
    /// bits renders as `int` instead of `int64_t`. The annotation
    /// channel reads the same value so the `.annot.json` sidecar's
    /// `return_type` agrees with the emitted C function declarator
    /// (the "single source" the B3.29 spec calls for).
    pub inferred_return: InferredReturn,
    /// Whole-callgraph void-return demote (B3.34, FR-21). Set by
    /// [`apply_void_return_inference`] when every observed direct
    /// caller of this function drops the return register (or no caller
    /// was observed in the analyzed function set). When `true`, the C
    /// backend overrides [`Self::inferred_return`] and renders the
    /// declarator as `void`, with the Return-lowering arm degrading
    /// any `Return { value: Some(_) }` to a bare `return;`. Stays
    /// `false` for stubs and for functions whose return is observed.
    /// The per-function [`Self::inferred_return`] signal is preserved
    /// untouched so the annotation channel can attribute the demote
    /// to the callgraph pass without losing the width-derived
    /// observation.
    pub void_return_demoted: bool,
}

/// Summary of the user hint applied to a function. Surfaces in
/// `--emit-report` and the C lowering's leading comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppliedHint {
    pub id: HintId,
    pub rename: Option<String>,
    pub return_overridden: bool,
    /// Number of argument slots whose type the hint pinned.
    pub args_overridden: u32,
    /// Number of `RegisterArg` slots minted past the convention-
    /// observed prefix to satisfy the hint's declared arity (B3.28).
    /// Zero for hints whose arity is at most what the inference pass
    /// already observed.
    pub args_synthesized: u32,
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
    /// Maps a virtual address that appears as an
    /// [`dac_ir::ssa::Operand::Const`] to the extracted string
    /// content at that VA. Backs the B3.7 string-literal naming
    /// heuristic.
    string_resolver: BinaryStringResolver,
    /// User-supplied hints (B3.6, FR-20). Empty when `--hints` was
    /// not passed.
    hints: &'a Hints,
}

/// Run the per-function orchestrator across the whole recovered
/// function set. The returned vector is in the same order as
/// `functions.functions`, so callers can zip the two together.
///
/// `hints` carries the user-hint overlay (FR-20); pass an empty
/// [`Hints`] when `--hints` was not requested.
#[must_use]
pub(crate) fn lift_all(
    functions: &FunctionSet,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    register_file: &RegisterFile,
    hints: &Hints,
) -> Vec<LiftOutcome> {
    let ctx = LiftCtx {
        model,
        bytes,
        decoder,
        lifter,
        register_file,
        stack_convention: stack_convention_for(model),
        api_resolver: BinaryImportResolver::new(model, bytes),
        string_resolver: BinaryStringResolver::new(model),
        hints,
    };
    functions
        .functions
        .iter()
        .map(|f| lift_one(f, &ctx))
        .collect()
}

/// Register every hint in the catalogue as an
/// [`EvidenceNode::UserHint`] in `graph`. Returns the hint catalogue
/// with each entry's `evidence` populated, so downstream passes
/// (and the annotation channel) can cite the exact node.
pub(crate) fn register_hints(hints: Hints, graph: &mut EvidenceGraph) -> Hints {
    let mut out = hints;
    for h in out.functions.iter_mut() {
        h.evidence = Some(graph.add_node(EvidenceNode::UserHint(h.id)));
    }
    for h in out.structs.iter_mut() {
        h.evidence = Some(graph.add_node(EvidenceNode::UserHint(h.id)));
    }
    out
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
    /// Functions whose recovered facts were overlaid by a
    /// user-supplied `[[function]]` hint (B3.6, FR-20).
    pub user_hint_functions: u64,
    /// Sum of [`NameTable::named_count`] across every `Real`
    /// outcome (B3.7). Surfaces in `--emit-report`'s recovery row
    /// alongside [`Self::nameable_values`] to express the
    /// "heuristic-name coverage" rubric the B3.7 "done when"
    /// criterion measures.
    pub named_values: u64,
    /// Subset of [`Self::named_values`] whose
    /// [`dac_recovery::NameSource`] is `UserHint` (B3.22, FR-20).
    /// Surfaces in `--emit-report`'s naming row so a reader sees
    /// how many of the heuristic names came from `--hints` versus
    /// the deterministic pipeline.
    pub hint_named_values: u64,
    /// Sum of total non-parameter SSA values across every `Real`
    /// outcome — the denominator for the heuristic-name coverage
    /// fraction.
    pub nameable_values: u64,
    /// Pure SSA ops + phis the B3.26 simplifier removed across every
    /// `Real` outcome. Surfaces in `--emit-report` so a reader can
    /// see how much pre-emit noise the deterministic pipeline shed.
    pub simplifier_drops: u64,
    /// Constant + identity folds the simplifier performed, summed
    /// across functions. Surfaces alongside [`Self::simplifier_drops`]
    /// in the report so the two halves of the simplification budget
    /// (rewrites vs. removals) are visible.
    pub simplifier_folds: u64,
    /// Number of [`SemStmt::Unreachable`] markers the structurer
    /// emitted across every `Real` outcome (B3.27). The C backend's
    /// default emit collapses each marker to a single
    /// `/* dac: structuring fallback */` line; the report surfaces
    /// the raw count so a reader sees how often the structurer hit a
    /// recognised fallback regardless of `--debug` (FR-25).
    pub structuring_fallbacks: u64,
    /// Number of `Real` outcomes whose declarator was forced to
    /// `void` by the whole-callgraph void-return inference pass
    /// (B3.34, FR-21). Surfaces in `--emit-report` so a reader sees
    /// how many runtime-helper bodies the pass collapsed across the
    /// translation unit. Functions also covered by a canonical
    /// signature override (e.g. `main`) keep the override's
    /// catalogued return spelling — this counter only tracks the
    /// raw demote, not the final printed declarator.
    pub void_return_demotions: u64,
}

impl LiftStats {
    pub(crate) fn from(outcomes: &[LiftOutcome]) -> Self {
        let mut s = Self::default();
        for o in outcomes {
            match o {
                LiftOutcome::Real { ssa, sem, facts } => {
                    s.real += 1;
                    s.structuring_fallbacks += count_structuring_fallbacks(&sem.body) as u64;
                    if facts.void_return_demoted {
                        s.void_return_demotions += 1;
                    }
                    if recovered_convention_is_useful(facts.convention.as_ref()) {
                        s.typed_signatures += 1;
                    }
                    if !facts.structs.pointer_structs.is_empty() {
                        s.struct_field_functions += 1;
                    }
                    if !facts.idioms.switch_tables.is_empty() {
                        s.switch_functions += 1;
                    }
                    if facts.user_hint.is_some() {
                        s.user_hint_functions += 1;
                    }
                    s.named_values += facts.names.named_count() as u64;
                    s.hint_named_values += facts
                        .names
                        .provenance
                        .values()
                        .filter(|c| c.source == dac_recovery::NameSource::UserHint)
                        .count() as u64;
                    s.nameable_values += nameable_value_count(ssa, facts.as_ref()) as u64;
                    s.simplifier_drops += facts.simplify.dead_pure_dropped as u64;
                    s.simplifier_folds += (facts.simplify.constants_folded
                        + facts.simplify.identities_folded
                        + facts.simplify.moves_folded)
                        as u64;
                }
                LiftOutcome::Stub { .. } => s.stub += 1,
            }
        }
        s
    }

    /// Heuristic-name coverage fraction (B3.7, FR-N spec §11.1).
    /// `0.0` when no values are nameable (every function lifted to
    /// a stub or had no SSA values), saturating to `1.0` in the
    /// degenerate case the denominator is somehow exceeded.
    pub(crate) fn named_value_ratio(self) -> f32 {
        if self.nameable_values == 0 {
            0.0
        } else {
            (self.named_values as f32 / self.nameable_values as f32).clamp(0.0, 1.0)
        }
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

/// Count [`SemStmt::Unreachable`] markers in `body` (recursively).
/// Each marker is a structuring fallback the structurer emitted
/// because the source block's terminator was `Unreachable` /
/// `Indirect` and no further idiom recogniser claimed it. The C
/// backend's default emit collapses each occurrence to a single
/// `/* dac: structuring fallback */` line (B3.27); the report
/// surfaces the raw count regardless of `--debug` (FR-25).
fn count_structuring_fallbacks(block: &SemBlock) -> u32 {
    let mut total = 0u32;
    for stmt in &block.stmts {
        match stmt {
            SemStmt::Unreachable { .. } => total = total.saturating_add(1),
            SemStmt::If {
                then_body,
                else_body,
                ..
            } => {
                total = total.saturating_add(count_structuring_fallbacks(then_body));
                if let Some(eb) = else_body {
                    total = total.saturating_add(count_structuring_fallbacks(eb));
                }
            }
            SemStmt::While { body, .. }
            | SemStmt::DoWhile { body, .. }
            | SemStmt::Loop { body, .. } => {
                total = total.saturating_add(count_structuring_fallbacks(body));
            }
            SemStmt::Switch { arms, default, .. } => {
                for arm in arms {
                    total = total.saturating_add(count_structuring_fallbacks(&arm.body));
                }
                if let Some(d) = default {
                    total = total.saturating_add(count_structuring_fallbacks(d));
                }
            }
            _ => {}
        }
    }
    total
}

fn recovered_convention_is_useful(c: Option<&ConventionMatch>) -> bool {
    match c {
        Some(c) => !c.signature.int_args.is_empty() || c.signature.return_register.is_some(),
        None => false,
    }
}

/// Project a [`LoopForest`] into the smaller [`LoopInfo`] summary
/// the name-recovery pass consumes (B3.20). Keeps `dac-recovery`
/// off the `dac-analysis` dependency graph — `dac-analysis` already
/// pulls in `dac-recovery` for [`Function`] / [`FunctionSet`], so a
/// direct dependency the other way would close the cycle.
fn loop_info_from_forest(forest: &LoopForest) -> LoopInfo {
    let mut headers: BTreeMap<u32, LoopShape> = BTreeMap::new();
    for l in &forest.loops {
        headers.insert(
            l.header,
            LoopShape {
                depth: l.depth,
                back_edges: l.back_edges.iter().copied().collect(),
            },
        );
    }
    LoopInfo { headers }
}

/// Count SSA values that are eligible for heuristic naming —
/// i.e. every defined value the C backend emits as a local, minus
/// the convention's inferred parameter slots (which the backend
/// names `argN` from the signature, not via [`NameTable`]).
///
/// Orphan values (no defining instruction or phi after the B3.26
/// simplifier, or already orphaned by the earlier CSE pass) are
/// skipped so the denominator matches the locals the C backend
/// actually emits.
fn nameable_value_count(ssa: &SsaFunction, facts: &RecoveryFacts) -> usize {
    let mut params: std::collections::BTreeSet<dac_ir::ssa::ValueId> =
        std::collections::BTreeSet::new();
    if let Some(c) = facts.convention.as_ref() {
        for a in &c.signature.int_args {
            params.insert(a.value);
        }
    }
    ssa.values
        .iter()
        .filter(|v| !params.contains(&v.id) && dac_recovery::value_has_definition(ssa, v.id))
        .count()
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
    let mut ssa = construct_ssa(&cfg, &doms, &raw);
    let pdoms = PostDominatorTree::build(&cfg);
    let loops = LoopForest::build(&cfg, &doms);

    // B3.10: run the recovery side-table passes here so the C backend
    // can consume their results. Each pass is independent and pure;
    // ordering follows the data dependencies — stack frame seeds
    // convention, both seed types, types seeds structs.
    let stack_frame = analyze_stack_frame(&ssa, ctx.stack_convention);
    let mut convention = infer_calling_convention(
        &ssa,
        &stack_frame,
        candidates_for(ctx.model.format, ctx.model.architecture),
    )
    .into_iter()
    .next();
    let signature = convention.as_ref().map(|c| &c.signature);
    let mut types = propagate_types(&ssa, signature, Some(&stack_frame), &ctx.api_resolver);

    // B3.26: pre-emit simplifier. Constant-folds, identity-folds,
    // substitutes trivial Move chains, and drops dead pure ops + dead
    // phis. Runs after `propagate_types` so the type lattice was
    // seeded from the un-simplified IR (load / store widths and API
    // signatures still anchor it correctly); orphaned `TypeMap`
    // entries for now-dead values are harmless. Determinism: `Pure` —
    // same SSA in, same SSA out.
    let simplify_stats = simplify(&mut ssa);

    let structs = recover_structs(&ssa, Some(&stack_frame), Some(&types));
    let idioms = recover_idioms(&ssa);

    let sem = structure(&ssa, &cfg, &doms, &pdoms, &loops);
    let sem = lower_switch_idioms(sem, &idioms, &cfg, ctx.model, ctx.bytes);

    // B3.6: overlay the user-hint catalogue. Hints update `types`
    // with `Source::UserHint` confidences and may promote the
    // convention's `return_register` so the C backend's
    // `pick_return_type` path activates. They never mutate the
    // SSA / Semantic IR — the binary stays ground truth (I-1).
    //
    // B3.28 extends this pass with an arity-synthesis step: a hint
    // declaring more arg slots than the convention pass observed
    // mints synthetic `RegisterArg` entries for the remaining
    // registers so the rendered C signature carries the user-
    // declared arity even when the body doesn't read those slots.
    let user_hint = apply_function_hint(f, ctx.hints, &ssa, &mut convention, &mut types);

    // B3.28: overlay the canonical entry-point catalogue. Runs
    // *after* the user-hint pass so a hint that extended the
    // arg-register list (e.g. `args = ["int", "char**"]` on a
    // function that the convention pass observed reading no
    // registers) feeds the canonical-signature builder. The
    // recovered name (e.g. `main`) keys into curated runtime
    // contracts that pin the return type to `int` and the arg list
    // to either `(void)` or `(int argc, char **argv)` depending on
    // the (post-hint) observed arity. The overlay seeds the type
    // lattice with matching IR types so the annotation channel
    // agrees, and produces a [`SynthesizedSignature`] the C backend
    // prints directly — sidestepping the stdint-style spelling the
    // type lattice would otherwise yield.
    let canonical_signature = apply_canonical_entry(f, &ssa, &mut convention, &mut types);

    // B3.29: per-function return-type inference. Observes the widest
    // write to the convention's return register variable in the SSA
    // and publishes `void` / `int` / `long`. Runs *after* the
    // canonical / hint overlays so a function whose return register
    // was synthesised at hint time (e.g. a `[[function]]` hint
    // pinned `return`) is observed under the post-overlay convention.
    // The canonical override still wins at lowering time when both
    // channels publish — the inference is a fallback for non-canonical
    // callees (FR-18, FR-21).
    let inferred_return = infer_return_type(&ssa, convention.as_ref());

    // B3.7 + B3.20: deterministic variable-naming heuristics.
    // Consumes the recovered convention + API resolver + extracted
    // strings, plus a per-function loop summary derived from the
    // natural-loop forest (the `LoopInfo` indirection keeps
    // dac-recovery from depending on dac-analysis, which already
    // depends on us). The result threads into the C backend's
    // `Recovered` view in place of the `v<id>` fallback. Pure /
    // deterministic (NFR-9) — same SSA + same resolvers + same
    // summary → same names.
    let loop_info = loop_info_from_forest(&loops);
    let rename_resolver = HintRenameResolver::new(ctx.hints);
    let names = recover_names(
        &ssa,
        convention.as_ref().map(|c| &c.signature),
        &ctx.api_resolver,
        &ctx.string_resolver,
        &rename_resolver,
        &loop_info,
        &types,
    );

    let facts = Box::new(RecoveryFacts {
        stack_frame,
        convention,
        types,
        structs,
        idioms,
        user_hint,
        names,
        simplify: simplify_stats,
        canonical_signature,
        inferred_return,
        void_return_demoted: false,
    });
    LiftOutcome::Real { ssa, sem, facts }
}

/// Look the function up in the hint catalogue. When matched, mutate
/// the recovered convention's signature + the type lattice in place
/// so the C backend renders the hinted types unchanged, and return
/// an [`AppliedHint`] summary for the report / leading comment.
fn apply_function_hint(
    f: &Function,
    hints: &Hints,
    ssa: &SsaFunction,
    convention: &mut Option<ConventionMatch>,
    types: &mut TypeMap,
) -> Option<AppliedHint> {
    let hint = hints.find_function(f.address, f.name.as_deref())?;
    let conf = Confidence::new(USER_HINT_CONFIDENCE, Source::UserHint);

    // Argument retyping: stuff hint args into the TypeMap so the
    // C backend's `parameter_type` / `pick_return_type` paths pick
    // them up. The lattice join is componentwise max on
    // `Confidence`; `Source::UserHint` outranks `Source::Derived`,
    // so the hint wins even when the propagation pass already
    // seeded a derived type.
    let mut args_overridden: u32 = 0;
    let mut args_synthesized: u32 = 0;
    if let (Some(hint_args), Some(conv)) = (&hint.args, convention.as_mut()) {
        for (i, arg) in conv.signature.int_args.iter().enumerate() {
            let Some(ty) = hint_args.get(i) else { break };
            types.values.insert(
                arg.value,
                ValueType {
                    ty: ty.to_ir(),
                    confidence: conf,
                },
            );
            args_overridden += 1;
        }

        // B3.28 arity extension: when the hint declares more arg
        // slots than the convention pass observed, mint synthetic
        // `RegisterArg` entries for the missing tail. Each synthetic
        // slot picks the next register in the convention's
        // `int_arg_registers` table and an unused `ValueId` /
        // `VariableId` (high-bit space so it never collides with an
        // SSA value the lifter produced). The synthetic ValueId
        // anchors a `TypeMap` entry so the C backend's
        // `parameter_type` lookup resolves to the hint's spelling.
        let observed = conv.signature.int_args.len();
        if let Some(table) = x86_64_convention_by_name(conv.convention_name) {
            for (i, ty) in hint_args.iter().enumerate().skip(observed) {
                let Some(&reg) = table.int_arg_registers.get(i) else {
                    // Hint asks for more slots than the convention
                    // has integer-arg registers (>6 on SysV); fall
                    // back silently — the surplus slots would map
                    // to stack arguments, which B3.28 does not yet
                    // synthesise (residue-shelf follow-up).
                    break;
                };
                let synth_value = synthetic_arg_value_id(i);
                let synth_variable = synthetic_arg_variable_id(i);
                types.values.insert(
                    synth_value,
                    ValueType {
                        ty: ty.to_ir(),
                        confidence: conf,
                    },
                );
                conv.signature.int_args.push(RegisterArg {
                    register: reg,
                    index: i,
                    value: synth_value,
                    variable: synth_variable,
                });
                args_synthesized += 1;
            }
        }
    }

    // Return retyping: seed every `Return { value: Some(v) }`
    // operand's TypeMap entry. The C backend's `pick_return_type`
    // only consults the type map when the convention has a return
    // register; promote `None` to a synthetic `"hinted"` marker so
    // the path activates.
    let mut return_overridden = false;
    if let Some(ret_ty) = &hint.return_ty {
        if let Some(conv) = convention.as_mut() {
            if conv.signature.return_register.is_none() {
                conv.signature.return_register = Some(hinted_return_register(conv));
            }
        }
        for block in &ssa.blocks {
            if let SsaTerminator::Return {
                value: Some(Operand::Value(v)),
            } = &block.terminator
            {
                types.values.insert(
                    *v,
                    ValueType {
                        ty: ret_ty.to_ir(),
                        confidence: conf,
                    },
                );
            }
        }
        return_overridden = true;
    }

    Some(AppliedHint {
        id: hint.id,
        rename: hint.rename.clone(),
        return_overridden,
        args_overridden,
        args_synthesized,
    })
}

/// Stable synthetic [`dac_ir::ssa::ValueId`] for the `i`-th hint-
/// synthesised arg slot. Uses the high half of the `u32` space so it
/// never collides with a value the SSA constructor allocated
/// (functions with > ~2^31 SSA values are not representable in the
/// IR's `ValueDef` index anyway).
fn synthetic_arg_value_id(i: usize) -> dac_ir::ssa::ValueId {
    // Reserve `0xFFFF_FF00 + i` for synthetic args. The `0xFF` prefix
    // keeps the ids visually distinct in `--debug` dumps.
    (0xFFFF_FF00u32).saturating_add(i as u32)
}

/// Stable synthetic [`dac_ir::ssa::VariableId`] for the `i`-th hint-
/// synthesised arg slot. Mirrors [`synthetic_arg_value_id`] in spirit
/// — distinct high-range id so the SSA's `variables` table stays
/// untouched.
fn synthetic_arg_variable_id(i: usize) -> dac_ir::ssa::VariableId {
    (0xFFFF_FF00u32).saturating_add(i as u32)
}

/// Look the function name up in the canonical entry-point catalogue
/// (B3.28, FR-12 / FR-21). When matched, pin the function's return
/// type to the catalogue's spelling and clip its arg list to the
/// `min(observed, canonical_arity)` prefix — so `main` reads as
/// `int main(void)` on a binary whose `main` reads no arguments,
/// `int main(int argc, char **argv)` when `rdi`/`rsi` (SysV) or
/// `rcx`/`rdx` (MsX64) are live.
///
/// The TypeMap is seeded for both the kept arg slots and every
/// `Return { value: Some(_) }` operand so the annotation channel
/// reports the canonical IR types alongside the C backend's
/// catalogue-spelt rendering. Confidence is `Source::Derived` at
/// [`CANONICAL_ENTRY_CONFIDENCE`] — the runtime contract is a
/// curated fact, not an observation of the bytes themselves.
fn apply_canonical_entry(
    f: &Function,
    ssa: &SsaFunction,
    convention: &mut Option<ConventionMatch>,
    types: &mut TypeMap,
) -> Option<SynthesizedSignature> {
    let name = f.name.as_deref()?;
    let entry = lookup_canonical_entry(name)?;
    let conv = convention.as_mut()?;
    let conf = Confidence::new(CANONICAL_ENTRY_CONFIDENCE, Source::Derived);

    // Arity is liveness-gated: the catalogue declares the maximal
    // shape (e.g. main has 2 args), but a callee that never reads
    // `rdi` / `rsi` stays at `(void)`. The convention pass already
    // computed the observed prefix on `signature.int_args`; the
    // canonical override fires only when the observed arity fits the
    // catalogue's contract.
    //
    // When the function reads *more* arg registers than the canonical
    // entry permits, decline to apply the override entirely. A
    // function named `main` that reads `rcx, rdx, r8, r9` on PE is
    // either a CRT-side wrapper passing through more registers than
    // the canonical contract describes or a misclassification of
    // caller-saved register reads; either way, truncating the int-arg
    // list to the canonical arity would break the body's existing
    // references to the dropped slots (I-1 — the IR is ground truth).
    let observed_arity = conv.signature.int_args.len();
    if observed_arity > entry.args.len() {
        return None;
    }
    let kept_arity = observed_arity;

    // Pin each kept arg's IR type from the catalogue.
    for (i, arg) in conv.signature.int_args.iter().enumerate() {
        let canon = &entry.args[i];
        types.values.insert(
            arg.value,
            ValueType {
                ty: canon.ir_type.clone(),
                confidence: conf,
            },
        );
    }

    // Pin the return type and promote the convention's
    // `return_register` so the C backend's `pick_return_type` path
    // (and the annotation channel) treats the function as returning
    // via the conventional integer return register.
    if conv.signature.return_register.is_none() {
        conv.signature.return_register = Some(hinted_return_register(conv));
    }
    for block in &ssa.blocks {
        if let SsaTerminator::Return {
            value: Some(Operand::Value(v)),
        } = &block.terminator
        {
            types.values.insert(
                *v,
                ValueType {
                    ty: entry.return_ir_type.clone(),
                    confidence: conf,
                },
            );
        }
    }

    // Build the C-backend signature override. `kept_arity` slots
    // get the catalogue's name + C-spelling; the return type uses
    // the catalogue's spelling unconditionally.
    let params: Vec<SynthesizedParam> = entry.args[..kept_arity]
        .iter()
        .map(|a| SynthesizedParam {
            name: a.name.to_string(),
            ty: CType::Named(a.c_type.to_string()),
        })
        .collect();

    Some(SynthesizedSignature {
        return_type: Some(CType::Named(entry.return_c_type.to_string())),
        params,
    })
}

/// Confidence value canonical-entry overlay entries carry. The
/// catalogue is curated knowledge, not an observation of the bytes —
/// pinned at `Source::Derived` so it sits above the convention pass's
/// derived facts but below an explicit user hint.
pub(crate) const CANONICAL_ENTRY_CONFIDENCE: f32 = 0.90;

/// Confidence value `Source::UserHint` overlay entries carry.
///
/// Shared with the annotation channel (B3.19) so the `.annot.json`
/// sidecar reports the same value the lift overlay attached.
pub(crate) const USER_HINT_CONFIDENCE: f32 = 0.95;

/// Pick the convention's canonical integer return register so a
/// hint with a `return` override can activate the C backend's
/// `pick_return_type` path even when the inference pass left
/// `signature.return_register == None`. We borrow the convention's
/// own register table via the `int_args` register family.
fn hinted_return_register(c: &ConventionMatch) -> &'static str {
    match c.convention_name {
        "ms-x64" => "rax",
        _ => "rax",
    }
}

/// Per-function return-type inference result (B3.29, FR-18 / FR-21).
///
/// Computed by [`infer_return_type`] from the SSA layer: the function's
/// return register variable is read out of the convention pass's
/// inferred signature, every write to that variable in the SSA body
/// is inspected, and the widest source-op width wins.
///
/// Variants:
///
/// - [`InferredReturn::Void`] — the convention pass pinned no return
///   register, or no writes to the return register variable were
///   observed in the body. The C backend renders the function as
///   `void`.
/// - [`InferredReturn::Width32`] — the widest observed write to the
///   return register is 32 bits — typically a `mov eax, …` (canonicalised
///   to a Move whose source is a 4-byte `Load`) or a [`SsaOp::Compare`]
///   result. The C backend renders the function as `int`.
/// - [`InferredReturn::Width64`] — the widest observed write is 64
///   bits, or the source-op width could not be observed (conservative
///   default; covers Move-from-Const, arithmetic, indirect Call). The
///   C backend renders the function as `long`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InferredReturn {
    Void,
    Width32,
    Width64,
}

/// Final per-function return-type inference verdict the annotation
/// channel consumes (B3.34 follow-up to B3.29's `InferredReturn`).
///
/// Carries the per-function width-derived inference *plus* the
/// whole-callgraph void-return demote bit so the `.annot.json` sidecar
/// can either render the original width (the function's return is
/// observed by at least one caller) or report the demote with an
/// honest explanation pointing at the callgraph pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReturnInferenceVerdict {
    pub width: InferredReturn,
    /// Set by [`apply_void_return_inference`]; mirrors
    /// [`RecoveryFacts::void_return_demoted`].
    pub demoted: bool,
}

impl InferredReturn {
    /// C-canonical type spelling consumed by
    /// [`dac_backend_c::Recovered::inferred_return`]. The C backend
    /// uses this to fill `Function::return_type` when no canonical
    /// override applies.
    pub(crate) fn to_c_type(self) -> CType {
        match self {
            InferredReturn::Void => CType::Void,
            InferredReturn::Width32 => CType::Named("int".into()),
            InferredReturn::Width64 => CType::Named("long".into()),
        }
    }

    /// Plain-text label consumed by the annotation channel — the
    /// `.annot.json` sidecar's `return_type` field.
    pub(crate) fn label(self) -> &'static str {
        match self {
            InferredReturn::Void => "void",
            InferredReturn::Width32 => "int",
            InferredReturn::Width64 => "long",
        }
    }

    /// Single-line explanation rendered into the annotation channel's
    /// "Why this type?" trail. Retires the stale "default void return;
    /// calling-convention return-value inference lands with B3.6"
    /// message the pre-B3.29 annotator carried.
    pub(crate) fn explanation(self) -> &'static str {
        match self {
            InferredReturn::Void => {
                "no writes to the return register observed in the function body"
            }
            InferredReturn::Width32 => "widest observed write to the return register is 32 bits",
            InferredReturn::Width64 => {
                "widest observed write to the return register is 64 bits \
                 (or the source-op width could not be observed)"
            }
        }
    }
}

/// Confidence value B3.29's width-derived return-type inference carries.
///
/// `Source::Derived` — the value comes from observing the SSA, not
/// from an explicit user statement or a curated knowledge entry; pinned
/// just under the type-propagation tier so a hint or canonical entry
/// still outranks it.
pub(crate) const RETURN_INFERENCE_CONFIDENCE: f32 = 0.75;

/// Per-function return-type inference pass (B3.29, FR-18 / FR-21).
///
/// The pass:
///
/// 1. Reads the convention pass's pinned return register from
///    `convention.signature.return_register`. When no register is
///    pinned, the function returns void — degrade to
///    [`InferredReturn::Void`] without examining the body.
/// 2. Looks the return register up in `ssa.variables` (matched
///    case-insensitively, so the convention's `"rax"` finds the
///    `Variable` whose name the lifter recorded in any case).
/// 3. Walks every SSA instruction whose `dst` lands in the return
///    register variable, and every phi whose dst variable is the
///    return register. Tracks whether any write was observed and
///    the widest [`source_op_width`] across them.
/// 4. Returns [`InferredReturn::Void`] when no writes were observed,
///    [`InferredReturn::Width32`] when the widest observation is at
///    most 32 bits, [`InferredReturn::Width64`] otherwise (or when
///    no width was extractable from any observed write).
///
/// Determinism: the pass walks the SSA in block / instruction order,
/// which is itself byte-stable. `Pure` (NFR-9).
fn infer_return_type(ssa: &SsaFunction, convention: Option<&ConventionMatch>) -> InferredReturn {
    let Some(return_reg) = convention.and_then(|c| c.signature.return_register) else {
        return InferredReturn::Void;
    };
    let Some(return_var) = ssa
        .variables
        .iter()
        .find(|v| v.name.eq_ignore_ascii_case(return_reg))
    else {
        return InferredReturn::Void;
    };
    let return_var_id = return_var.id;

    // Stop B3.29 from collapsing a function whose body still carries
    // `Return { value: Some(_) }` into `void` — the structurer would
    // keep emitting `return v<id>;` against the void declarator and
    // the round-trip compile gate would refuse the output. The
    // narrower-than-64 inference only fires when we actually observed
    // a sub-64-bit write to the return register; the fallback for any
    // function that returns a value stays at 64 bits (= `long`).
    let mut any_return_value: Option<Operand> = None;
    for block in &ssa.blocks {
        if let dac_ir::ssa::SsaTerminator::Return { value: Some(v) } = &block.terminator {
            any_return_value = Some(*v);
            break;
        }
    }

    let mut any_write_to_return_reg = false;
    let mut max_observed: u16 = 0;
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let Some(dst) = ins.dst else { continue };
            if ssa.value(dst).variable != return_var_id {
                continue;
            }
            any_write_to_return_reg = true;
            if let Some(w) = source_op_width(ssa, &ins.op) {
                if w > max_observed {
                    max_observed = w;
                }
            }
        }
        for phi in &block.phis {
            if ssa.value(phi.dst).variable == return_var_id {
                any_write_to_return_reg = true;
            }
        }
    }

    // B3.26's simplifier folds the lifter's Move-into-rax sequence, so
    // a function whose only write to the return register was a Move
    // from a Load gets the Move dropped and the Return reads the
    // Load's dst directly. Probe the Return value's defining op width
    // here so the inference doesn't miss the signal the simplifier
    // collapsed.
    if let Some(Operand::Value(v)) = any_return_value {
        let def = ssa.value(v);
        if let dac_ir::ssa::ValueSource::Instruction { block, index } = def.source {
            let op = &ssa.block(block).instructions[index as usize].op;
            if let Some(w) = source_op_width(ssa, op) {
                if w > max_observed {
                    max_observed = w;
                }
            }
        }
    }

    if any_return_value.is_none() && !any_write_to_return_reg {
        return InferredReturn::Void;
    }

    if max_observed > 0 && max_observed <= 32 {
        InferredReturn::Width32
    } else {
        InferredReturn::Width64
    }
}

/// Width of the source operation that produces the value written into
/// the return register. The lifter erases the source-width signal on
/// 32-bit register writes (`mov eax, …` becomes a Move into the rax
/// parent variable), so the inference walks through one Move hop to
/// reach the underlying Load / Compare whose width is encoded on the
/// op itself.
///
/// Returns `None` when no width can be extracted — Move-from-Const,
/// Move-from-Parameter, arithmetic, indirect Call, and Opaque ops all
/// lose the signal. Callers default to 64-bit.
fn source_op_width(ssa: &SsaFunction, op: &dac_ir::ssa::SsaOp) -> Option<u16> {
    use dac_ir::ssa::SsaOp;
    match op {
        SsaOp::Load { width, .. } => {
            let bits = u16::from(*width).saturating_mul(8);
            (bits > 0).then_some(bits)
        }
        SsaOp::Compare { .. } => Some(8),
        SsaOp::Move {
            src: Operand::Value(v),
        } => {
            let def = ssa.value(*v);
            if let dac_ir::ssa::ValueSource::Instruction { block, index } = def.source {
                let inner = &ssa.block(block).instructions[index as usize].op;
                source_op_width(ssa, inner)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Whole-callgraph void-return inference (B3.34, FR-21).
///
/// Walks every `Real` outcome's SSA looking for direct calls
/// ([`dac_ir::ssa::SsaOp::Call`] with a resolved `target`), and for
/// each callee records whether at least one observed caller consumes
/// the returned register. After the scan, every analyzed function for
/// which no caller observes the return value has its
/// [`RecoveryFacts::void_return_demoted`] flag set to `true`.
///
/// The C backend's [`dac_backend_c::Recovered::void_return_demoted`]
/// channel then forces the function's declarator to `void` and drops
/// the value side of `Return { value: Some(_) }`, so the body stays
/// well-formed C even when the SSA-level Return still carries a value
/// operand. The canonical-signature override (B3.28) sits *above* this
/// channel in [`dac_backend_c::pick_return_type`], so curated entry
/// points such as `main` keep their catalogued return spelling even
/// when the callgraph pass demotes them.
///
/// "Observed by a caller" is conservative: an indirect call (no
/// resolvable `target`), a call whose `dst` is `None` (the caller
/// already chose to drop the return), and any call whose return-value
/// SSA `ValueId` has zero non-trivial uses in the caller all count as
/// *non*-observations. A call whose `dst` has any [`UseSite`] entry
/// in [`dac_analysis::dataflow::DependeUseChains`] counts as an
/// observation and is enough to keep the callee's return type.
///
/// Functions that no analyzed function calls — exported entry points
/// such as `_init`, `_fini`, `register_tm_clones`, … in the
/// `hello-x86_64` fixture — are demoted by vacuous truth: "every
/// observed caller drops the return" is trivially satisfied when no
/// caller was observed at all. This is the path the B3.34 done-when
/// list rests on, and the canonical-signature override above keeps
/// `main` rendering as `int` even when it falls in the same bucket.
///
/// Stubs ([`LiftOutcome::Stub`]) are skipped — there is no SSA to
/// observe.
///
/// Determinism: the pass is `Pure`. It walks the outcomes slice in its
/// existing address-sorted order, scans each SSA's blocks /
/// instructions in source order, and stores per-callee state in a
/// [`BTreeMap`] keyed by VA. Two runs on the same input mutate the
/// outcomes slice into the same byte-stable shape (NFR-9).
pub(crate) fn apply_void_return_inference(outcomes: &mut [LiftOutcome]) {
    use dac_analysis::dataflow::{compute_def_use, DefUseChains, UseSite};
    use dac_ir::ssa::{SsaOp, SsaTerminator, ValueId};
    use std::collections::BTreeSet;

    // Index every analyzed function by its address so the pass can
    // look up whose return value it should mark observed when it
    // walks a Call instruction.
    let mut analyzed: BTreeMap<u64, usize> = BTreeMap::new();
    for (i, o) in outcomes.iter().enumerate() {
        if let LiftOutcome::Real { ssa, .. } = o {
            analyzed.insert(ssa.function_address, i);
        }
    }

    // Per-function def-use chains, keyed by function VA. Cached
    // outside the fixpoint loop so the repeated escape walks below
    // don't recompute them.
    let mut def_use_table: BTreeMap<u64, DefUseChains> = BTreeMap::new();
    for o in outcomes.iter() {
        if let LiftOutcome::Real { ssa, .. } = o {
            def_use_table.insert(ssa.function_address, compute_def_use(ssa));
        }
    }

    // `observed[F]` = true once we've concluded F's return value
    // reaches a sink visible to the analyzed callgraph.
    //
    // Bootstrap: functions whose canonical-signature override pinned
    // a concrete (non-void) return type are seeded as observed —
    // their return spelling is fixed by curated knowledge, so the
    // callgraph pass must not contradict it. `main` on the
    // `hello-x86_64` fixture rides this seed.
    let mut observed: BTreeMap<u64, bool> = analyzed.keys().map(|&va| (va, false)).collect();
    for (&va, &idx) in &analyzed {
        if let LiftOutcome::Real { facts, .. } = &outcomes[idx] {
            let bootstrapped = facts
                .canonical_signature
                .as_ref()
                .and_then(|c| c.return_type.as_ref())
                .is_some_and(|ty| !matches!(ty, CType::Void));
            if bootstrapped {
                observed.insert(va, true);
            }
        }
    }

    // Fixpoint loop. Each pass walks every direct call site in every
    // analyzed function and decides whether the call's `dst` reaches
    // a sink that escapes the caller. The two transitive cases are:
    //
    // - the value flows through phis and Moves into the caller's
    //   `Return` operand. If the caller is itself observed, the value
    //   escapes; otherwise the use is deferred.
    // - the value reaches a "hard" sink (Store value, Call arg,
    //   Branch condition, arithmetic / compare / load address, …).
    //   These count as observations unconditionally.
    //
    // Convergence: every iteration either flips at least one entry
    // from `false` to `true` or the loop exits. The lattice has
    // monotone-rising values bounded by the number of analyzed
    // functions, so the worklist terminates in O(N) iterations
    // (NFR-9 deterministic).
    loop {
        let mut changed = false;
        for o in outcomes.iter() {
            let LiftOutcome::Real { ssa, .. } = o else {
                continue;
            };
            let caller_addr = ssa.function_address;
            let caller_observed = *observed.get(&caller_addr).unwrap_or(&false);
            let Some(def_use) = def_use_table.get(&caller_addr) else {
                continue;
            };
            for block in &ssa.blocks {
                for ins in &block.instructions {
                    let SsaOp::Call {
                        target: Some(target),
                        ..
                    } = &ins.op
                    else {
                        continue;
                    };
                    if !analyzed.contains_key(target) {
                        continue;
                    }
                    let Some(dst) = ins.dst else {
                        continue;
                    };
                    // Already known observed — skip the walk.
                    if observed.get(target).copied().unwrap_or(false) {
                        continue;
                    }
                    let mut visiting: BTreeSet<ValueId> = BTreeSet::new();
                    if value_escapes(ssa, def_use, dst, caller_observed, &mut visiting) {
                        if let Some(slot) = observed.get_mut(target) {
                            if !*slot {
                                *slot = true;
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Demote every function whose entry stayed `false`. The
    // per-function `inferred_return` channel is left untouched so the
    // annotation channel can still attribute the original width
    // observation; the void declarator is published through the new
    // `void_return_demoted` flag, which the C backend's
    // `pick_return_type` consults below the canonical override.
    for o in outcomes.iter_mut() {
        let LiftOutcome::Real { ssa, facts, .. } = o else {
            continue;
        };
        if matches!(observed.get(&ssa.function_address), Some(false)) {
            facts.void_return_demoted = true;
        }
    }

    /// Transitive escape predicate.
    ///
    /// Returns `true` if `v` (or any value transitively reachable
    /// through phi / Move propagation) reaches a sink other than a
    /// `Return` whose enclosing function's return is itself
    /// unobserved. The `visiting` cache breaks phi cycles by
    /// short-circuiting recursion when a value is already on the
    /// stack.
    fn value_escapes(
        ssa: &dac_ir::ssa::SsaFunction,
        def_use: &DefUseChains,
        v: ValueId,
        caller_observed: bool,
        visiting: &mut BTreeSet<ValueId>,
    ) -> bool {
        if !visiting.insert(v) {
            // Already on the recursion stack — break the cycle. The
            // value has no novel escape contribution along this path;
            // any non-phi escape would have been detected at the
            // originating site.
            return false;
        }
        for site in def_use.uses(v) {
            if site_escapes(ssa, def_use, *site, caller_observed, visiting) {
                visiting.remove(&v);
                return true;
            }
        }
        visiting.remove(&v);
        false
    }

    fn site_escapes(
        ssa: &dac_ir::ssa::SsaFunction,
        def_use: &DefUseChains,
        site: UseSite,
        caller_observed: bool,
        visiting: &mut BTreeSet<ValueId>,
    ) -> bool {
        match site {
            UseSite::Terminator { block } => match &ssa.block(block).terminator {
                SsaTerminator::Return { .. } => caller_observed,
                // Branch condition observes the value regardless of
                // the caller's return-observation status.
                _ => true,
            },
            UseSite::Phi {
                block, phi: idx, ..
            } => {
                // A phi propagates: the incoming value is observed
                // iff the phi's result is observed somewhere
                // downstream.
                let phi = &ssa.block(block).phis[idx as usize];
                value_escapes(ssa, def_use, phi.dst, caller_observed, visiting)
            }
            UseSite::Instruction { block, index } => {
                let instr = &ssa.block(block).instructions[index as usize];
                match &instr.op {
                    // Move propagates verbatim. Trace the dst.
                    SsaOp::Move { .. } => {
                        if let Some(dst) = instr.dst {
                            value_escapes(ssa, def_use, dst, caller_observed, visiting)
                        } else {
                            true
                        }
                    }
                    // Every other op (Store value, Load address,
                    // Call args, Compare, Add, Sub, And, Or, Xor,
                    // Shl, Shr, Opaque, …) treats v as a hard sink.
                    // Demoting the callee's return would erase a
                    // contribution the body actually depends on.
                    _ => true,
                }
            }
        }
    }
}

/// Post-pass on the structurer output: rewrite each
/// [`SemStmt::Unreachable`] whose source block matches a recognised
/// [`SwitchTableIdiom`] into [`SemStmt::Switch`], populating arms by
/// reading the table out of the binary section that backs the
/// recovered table base (B3.17, FR-18).
///
/// Resolution proceeds in three phases:
/// 1. **Resolve entries.** [`resolve_switch_entries`] reads the
///    binary at `idiom.table_base_const`, decoding the
///    absolute-pointer (`width == stride == 8`) or `int32_t`-relative
///    (`width == stride == 4`) shapes — bounded by the recovered
///    `bound` and capped by [`MAX_SWITCH_ENTRIES`].
/// 2. **Map VAs to blocks.** Every resolved target VA is looked up
///    in the CFG's block-address table; matches mint a per-block
///    [`LabelId`] from the function's existing label-id space so the
///    structurer-allocated labels and the switch-allocated labels
///    don't collide. Entries whose target VA doesn't hit a block
///    boundary are dropped (the idiom recognition was structurally
///    sound but the table contained a sentinel or jumped into the
///    middle of a known block — honest degradation, I-6).
/// 3. **Anchor labels.** The new [`SemStmt::Label`] markers are
///    appended at the function-body tail, *outside* the structurer's
///    recursive walk. They share the same orphan-anchor mechanism the
///    structurer already uses for goto targets it can't place inside
///    the structured tree, so an arm rewrite later in the pipeline
///    can't drop the label slot.
///
/// When the idiom carries no resolvable entries (no base, no bound,
/// or unsupported encoding), the switch surface degrades to the
/// B3.10 shape: a `Switch` with empty arms and an `Unreachable`
/// default body. The reader still sees the recognised idiom (I-6).
fn lower_switch_idioms(
    mut sem: SemFunction,
    idioms: &RecoveredIdioms,
    cfg: &dac_analysis::cfg::Cfg,
    model: &BinaryModel,
    bytes: &[u8],
) -> SemFunction {
    if idioms.switch_tables.is_empty() {
        return sem;
    }
    let resolved = build_resolved_tables(&idioms.switch_tables, cfg, model, bytes, &sem);
    rewrite_block(&mut sem.body, &resolved);
    append_orphan_labels(&mut sem.body, &resolved);
    sem
}

/// Per-switch resolution record threaded through the post-pass. Carries
/// the scrutinee, the recovered arms (case value paired with the
/// minted label id), and the distinct target blocks whose labels need
/// to be anchored at the function-body tail.
///
/// Every recognised [`SwitchTableIdiom`] gets a record — even when
/// `arms` is empty. The rewriter uses presence-in-map to decide whether
/// to demote a `SemStmt::Unreachable` into a `SemStmt::Switch`, which
/// keeps the B3.10 "switch with empty arms + Unreachable default"
/// surface alive for tables whose entry resolution failed.
struct ResolvedSwitch {
    scrutinee: dac_ir::ssa::ValueId,
    /// Case values paired with the [`LabelId`] of the target block's
    /// label marker. Deterministic order: ascending by case value.
    arms: Vec<(i64, dac_ir::sem::LabelId)>,
    /// Distinct target blocks whose labels need to be anchored at the
    /// function-body tail. Deterministic order: ascending by
    /// [`LabelId`].
    labels: Vec<(dac_ir::ssa::SsaBlockId, dac_ir::sem::LabelId)>,
}

type SwitchResolutions = BTreeMap<dac_ir::ssa::SsaBlockId, ResolvedSwitch>;

fn build_resolved_tables(
    tables: &BTreeMap<dac_ir::ssa::SsaBlockId, SwitchTableIdiom>,
    cfg: &dac_analysis::cfg::Cfg,
    model: &BinaryModel,
    bytes: &[u8],
    sem: &SemFunction,
) -> SwitchResolutions {
    let block_index: BTreeMap<u64, dac_ir::ssa::SsaBlockId> =
        cfg.blocks.iter().map(|b| (b.address, b.id)).collect();
    let mut next_label = next_label_id(&sem.body);
    let mut out = SwitchResolutions::new();
    for (source_block, idiom) in tables {
        let resolved = resolve_switch_entries(idiom, model, bytes);
        let mut block_label: BTreeMap<dac_ir::ssa::SsaBlockId, dac_ir::sem::LabelId> =
            BTreeMap::new();
        let mut arms: Vec<(i64, dac_ir::sem::LabelId)> = Vec::new();
        for entry in resolved {
            let Some(&target_block) = block_index.get(&entry.target_va) else {
                continue;
            };
            let lid = *block_label.entry(target_block).or_insert_with(|| {
                let id = next_label;
                next_label = next_label.saturating_add(1);
                id
            });
            arms.push((entry.case_value, lid));
        }
        let mut labels: Vec<(dac_ir::ssa::SsaBlockId, dac_ir::sem::LabelId)> =
            block_label.into_iter().collect();
        labels.sort_by_key(|(_, lid)| *lid);
        out.insert(
            *source_block,
            ResolvedSwitch {
                scrutinee: idiom.scrutinee,
                arms,
                labels,
            },
        );
    }
    out
}

/// Next free [`LabelId`] given the function body — one past the highest
/// id any existing [`SemStmt::Label`] or [`SemStmt::Goto`] references.
/// Conservative: scans both labels and goto targets so the allocator
/// stays above the structurer's range even when the structurer
/// pre-allocated a slot whose label marker hasn't been inserted yet.
fn next_label_id(body: &SemBlock) -> dac_ir::sem::LabelId {
    let mut max: Option<dac_ir::sem::LabelId> = None;
    walk_label_ids(body, &mut |id| {
        max = Some(match max {
            Some(prev) => prev.max(id),
            None => id,
        });
    });
    match max {
        Some(prev) => prev.saturating_add(1),
        None => 0,
    }
}

fn walk_label_ids(body: &SemBlock, f: &mut impl FnMut(dac_ir::sem::LabelId)) {
    for stmt in &body.stmts {
        match stmt {
            SemStmt::Label { id, .. } => f(*id),
            SemStmt::Goto { target, .. } => f(*target),
            SemStmt::If {
                then_body,
                else_body,
                ..
            } => {
                walk_label_ids(then_body, f);
                if let Some(eb) = else_body {
                    walk_label_ids(eb, f);
                }
            }
            SemStmt::While { body, .. }
            | SemStmt::DoWhile { body, .. }
            | SemStmt::Loop { body, .. } => walk_label_ids(body, f),
            SemStmt::Switch { arms, default, .. } => {
                for arm in arms {
                    walk_label_ids(&arm.body, f);
                }
                if let Some(d) = default {
                    walk_label_ids(d, f);
                }
            }
            _ => {}
        }
    }
}

fn append_orphan_labels(body: &mut SemBlock, resolved: &SwitchResolutions) {
    // Already-anchored label ids — the structurer's `insert_labels`
    // post-pass may have placed some of our newly-minted ids inside
    // the tree if a target block happened to be visited via another
    // path. Defensive: only emit a tail marker for label ids the
    // tree doesn't already carry a `Stmt::Label` for. Goto targets
    // do not count — a `goto` reference is the *consumer*, not the
    // anchor.
    let mut anchored: std::collections::BTreeSet<dac_ir::sem::LabelId> =
        std::collections::BTreeSet::new();
    walk_anchored_labels(body, &mut |id| {
        anchored.insert(id);
    });
    let mut entries: Vec<(dac_ir::ssa::SsaBlockId, dac_ir::sem::LabelId)> = Vec::new();
    for switch in resolved.values() {
        for (block, lid) in &switch.labels {
            if !entries.iter().any(|(_, existing)| *existing == *lid) {
                entries.push((*block, *lid));
            }
        }
    }
    entries.sort_by_key(|(_, lid)| *lid);
    for (block, lid) in entries {
        if anchored.contains(&lid) {
            continue;
        }
        body.stmts.push(SemStmt::Label {
            id: lid,
            source_block: block,
        });
    }
}

/// Like [`walk_label_ids`] but visits only `Stmt::Label` markers — the
/// anchors — and not `Stmt::Goto` targets. Used by
/// [`append_orphan_labels`] to decide which newly-minted label ids
/// still need a tail anchor.
fn walk_anchored_labels(body: &SemBlock, f: &mut impl FnMut(dac_ir::sem::LabelId)) {
    for stmt in &body.stmts {
        match stmt {
            SemStmt::Label { id, .. } => f(*id),
            SemStmt::If {
                then_body,
                else_body,
                ..
            } => {
                walk_anchored_labels(then_body, f);
                if let Some(eb) = else_body {
                    walk_anchored_labels(eb, f);
                }
            }
            SemStmt::While { body, .. }
            | SemStmt::DoWhile { body, .. }
            | SemStmt::Loop { body, .. } => walk_anchored_labels(body, f),
            SemStmt::Switch { arms, default, .. } => {
                for arm in arms {
                    walk_anchored_labels(&arm.body, f);
                }
                if let Some(d) = default {
                    walk_anchored_labels(d, f);
                }
            }
            _ => {}
        }
    }
}

fn rewrite_block(block: &mut SemBlock, resolved: &SwitchResolutions) {
    for stmt in block.stmts.iter_mut() {
        rewrite_stmt(stmt, resolved);
    }
}

fn rewrite_stmt(stmt: &mut SemStmt, resolved: &SwitchResolutions) {
    match stmt {
        SemStmt::Unreachable {
            source_block,
            evidence,
        } => {
            let Some(record) = resolved.get(source_block) else {
                return;
            };
            let switch_arms = record
                .arms
                .iter()
                .map(|(value, lid)| SwitchArm {
                    value: *value,
                    body: SemBlock {
                        stmts: vec![SemStmt::Goto {
                            target: *lid,
                            source_block: *source_block,
                            evidence: *evidence,
                        }],
                    },
                })
                .collect::<Vec<_>>();
            let mut default = SemBlock::empty();
            default.stmts.push(SemStmt::Unreachable {
                source_block: *source_block,
                evidence: *evidence,
            });
            *stmt = SemStmt::Switch {
                scrutinee: Operand::Value(record.scrutinee),
                arms: switch_arms,
                default: Some(default),
                source_block: *source_block,
                evidence: *evidence,
            };
        }
        SemStmt::If {
            then_body,
            else_body,
            ..
        } => {
            rewrite_block(then_body, resolved);
            if let Some(eb) = else_body.as_mut() {
                rewrite_block(eb, resolved);
            }
        }
        SemStmt::While { body, .. }
        | SemStmt::DoWhile { body, .. }
        | SemStmt::Loop { body, .. } => {
            rewrite_block(body, resolved);
        }
        SemStmt::Switch { arms, default, .. } => {
            for arm in arms.iter_mut() {
                rewrite_block(&mut arm.body, resolved);
            }
            if let Some(d) = default.as_mut() {
                rewrite_block(d, resolved);
            }
        }
        _ => {}
    }
}

/// `ApiResolver` backed by the binary's import / symbol table and,
/// on ELF, the PLT trampolines discovered by
/// [`elf_x86_64_plt_stubs`]. Only direct calls whose target VA
/// exactly matches an imported function (or a non-import named
/// symbol that resolves to a known API, or an ELF PLT stub for an
/// imported function) bind to a signature. The resolver consults
/// pre-built reverse maps so the lookup is `O(log n)`.
struct BinaryImportResolver {
    /// Map from import-target VA to signature. Populated from both
    /// the symbol-table side (PE / ELF binaries that still carry a
    /// `.symtab` with PLT-stub addresses) and the PLT walker
    /// (stripped or `.dynsym`-only ELF binaries — B3.21).
    imports_by_va: BTreeMap<u64, &'static ApiSignature>,
    /// Map from imported / exported symbol name to signature, used
    /// when the call site decodes a VA that lands on a named symbol
    /// (e.g. a direct call into libc statically linked in).
    name_index: BTreeMap<u64, &'static ApiSignature>,
}

impl BinaryImportResolver {
    /// `bytes` is the input image required by the PLT walker on
    /// ELF; on every other format it is unused and the
    /// `elf_x86_64_plt_stubs` call returns an empty vector.
    fn new(model: &BinaryModel, bytes: &[u8]) -> Self {
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
        // B3.21: ELF binaries don't surface PLT-stub VAs as named
        // symbols the way PE imports do, so we walk the trampolines
        // explicitly and bind each stub VA to the import it
        // resolves through `.rela.plt`. This lights up the
        // [`recover_names`] `ApiContext` heuristic (and the
        // [`propagate_types`] API-signature seed) on every
        // unstripped ELF that was 0% before.
        for (stub_va, name) in elf_x86_64_plt_stubs(model, bytes) {
            if let Some(sig) = lookup_api_signature(&name) {
                imports_by_va.entry(stub_va).or_insert(sig);
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

/// [`StringResolver`] backed by the binary model's extracted
/// strings. Pre-computes a `VA → &str` map so per-function
/// lookups stay `O(log n)` and the [`recover_names`] pass does not
/// re-scan the entire string table for each value.
///
/// Only strings located in read-only data sections contribute
/// candidates: a write-target VA matching a `.data` byte sequence
/// would just as easily be a number, an embedded struct, or
/// padding — naming after it would invent a fact the binary does
/// not support (I-6).
struct BinaryStringResolver {
    by_va: BTreeMap<u64, String>,
}

impl BinaryStringResolver {
    fn new(model: &BinaryModel) -> Self {
        let mut by_va: BTreeMap<u64, String> = BTreeMap::new();
        for s in &model.strings {
            let Some(section) = model.sections.get(s.section) else {
                continue;
            };
            if section.kind != dac_binfmt::SectionKind::ReadOnlyData {
                continue;
            }
            let va = section.address.saturating_add(s.offset);
            by_va.entry(va).or_insert_with(|| s.value.clone());
        }
        Self { by_va }
    }
}

impl StringResolver for BinaryStringResolver {
    fn resolve(&self, va: u64) -> Option<&str> {
        self.by_va.get(&va).map(String::as_str)
    }
}

/// [`CallRenameResolver`] backed by the user-supplied
/// `[[function]]` hint catalogue (B3.22, FR-20). Pre-computes a
/// `VA → rename` map at construction so per-call-site lookups stay
/// `O(log n)` even on hint files that list every binary import.
///
/// Hints with no `rename` field, and hints whose matcher is name-
/// only (no address — the recovery pass has no name index for
/// arbitrary call targets), contribute no entries: the rename
/// heuristic abstains for them and the deterministic name pipeline
/// runs unchanged.
struct HintRenameResolver {
    by_va: BTreeMap<u64, String>,
}

impl HintRenameResolver {
    fn new(hints: &Hints) -> Self {
        let mut by_va: BTreeMap<u64, String> = BTreeMap::new();
        for h in &hints.functions {
            let Some(rename) = h.rename.as_ref() else {
                continue;
            };
            match &h.matcher {
                dac_hints::HintMatcher::Address(va)
                | dac_hints::HintMatcher::Both { address: va, .. } => {
                    by_va.entry(*va).or_insert_with(|| rename.clone());
                }
                dac_hints::HintMatcher::Name(_) => {}
            }
        }
        Self { by_va }
    }
}

impl CallRenameResolver for HintRenameResolver {
    fn resolve(&self, target_va: u64) -> Option<&str> {
        self.by_va.get(&target_va).map(String::as_str)
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
            user_hint: None,
            names: NameTable::default(),
            simplify: SimplifyStats::default(),
            canonical_signature: None,
            inferred_return: InferredReturn::Void,
            void_return_demoted: false,
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

    // ---- B3.27 structuring-fallback counter ------------------------

    fn ev_node() -> dac_core::EvidenceId {
        dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
            layer: dac_core::IrLayer::Semantic,
            id: 0,
        })
    }

    #[test]
    fn b3_27_counts_lone_unreachable_marker() {
        let mut body = SemBlock::empty();
        body.stmts.push(SemStmt::Unreachable {
            source_block: 0,
            evidence: ev_node(),
        });
        assert_eq!(count_structuring_fallbacks(&body), 1);
    }

    #[test]
    fn b3_27_counts_unreachable_nested_in_if_arm() {
        // A nested fallback (the structurer reached an unreachable arm
        // of an If) must contribute exactly once to the count.
        let mut body = SemBlock::empty();
        let mut else_body = SemBlock::empty();
        else_body.stmts.push(SemStmt::Unreachable {
            source_block: 2,
            evidence: ev_node(),
        });
        body.stmts.push(SemStmt::If {
            cond: dac_ir::ssa::Operand::Const(0),
            then_body: SemBlock::empty(),
            else_body: Some(else_body),
            source_block: 0,
            evidence: ev_node(),
        });
        assert_eq!(count_structuring_fallbacks(&body), 1);
    }

    #[test]
    fn b3_27_counts_unreachable_inside_switch_default() {
        // B3.17 lowers switches whose entries fail to resolve by
        // wrapping the recognised idiom in a Switch with an
        // Unreachable default body. That default counts as one
        // fallback, plus any extras in arms.
        let mut body = SemBlock::empty();
        let mut default_body = SemBlock::empty();
        default_body.stmts.push(SemStmt::Unreachable {
            source_block: 4,
            evidence: ev_node(),
        });
        body.stmts.push(SemStmt::Switch {
            scrutinee: dac_ir::ssa::Operand::Value(0),
            arms: Vec::new(),
            default: Some(default_body),
            source_block: 4,
            evidence: ev_node(),
        });
        assert_eq!(count_structuring_fallbacks(&body), 1);
    }

    #[test]
    fn b3_27_counts_zero_when_body_has_no_unreachable() {
        // A real `Return None` body has no fallback markers.
        let mut body = SemBlock::empty();
        body.stmts.push(SemStmt::Return {
            value: None,
            evidence: ev_node(),
        });
        assert_eq!(count_structuring_fallbacks(&body), 0);
    }

    // ---- B3.17 switch-table lowering -------------------------------

    fn make_body_with_labels(label_ids: &[dac_ir::sem::LabelId]) -> SemBlock {
        let mut body = SemBlock::empty();
        for &id in label_ids {
            body.stmts.push(SemStmt::Label {
                id,
                source_block: 0,
            });
        }
        body
    }

    /// `next_label_id` reserves the slot one past the highest existing
    /// label or goto target so the switch-allocated ids can't collide
    /// with structurer-allocated ones.
    #[test]
    fn next_label_id_picks_one_past_the_highest_in_use() {
        let body = make_body_with_labels(&[0, 1, 4]);
        assert_eq!(next_label_id(&body), 5);
    }

    #[test]
    fn next_label_id_on_empty_body_starts_at_zero() {
        let body = SemBlock::empty();
        assert_eq!(next_label_id(&body), 0);
    }

    #[test]
    fn next_label_id_counts_goto_targets() {
        // A Goto stmt's target reserves the slot too — even before its
        // matching Label has been inserted by the structurer.
        let mut body = SemBlock::empty();
        body.stmts.push(SemStmt::Goto {
            target: 7,
            source_block: 0,
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Semantic,
                id: 0,
            }),
        });
        assert_eq!(next_label_id(&body), 8);
    }

    /// `walk_anchored_labels` reports only `Stmt::Label` markers — it
    /// excludes `Stmt::Goto` targets so `append_orphan_labels` knows
    /// which label ids still need a tail anchor.
    #[test]
    fn walk_anchored_labels_ignores_goto_targets() {
        let mut body = SemBlock::empty();
        let ev = dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
            layer: dac_core::IrLayer::Semantic,
            id: 0,
        });
        body.stmts.push(SemStmt::Label {
            id: 1,
            source_block: 0,
        });
        body.stmts.push(SemStmt::Goto {
            target: 2,
            source_block: 0,
            evidence: ev,
        });
        let mut seen: std::collections::BTreeSet<dac_ir::sem::LabelId> =
            std::collections::BTreeSet::new();
        walk_anchored_labels(&body, &mut |id| {
            seen.insert(id);
        });
        // Label id 1 is anchored; goto target 2 is *not* anchored.
        assert!(seen.contains(&1));
        assert!(!seen.contains(&2));
    }

    /// `append_orphan_labels` writes `Stmt::Label` markers at the
    /// function-body tail for every newly-minted switch label id that
    /// isn't already anchored inside the structured tree.
    #[test]
    fn append_orphan_labels_anchors_each_switch_label_at_body_tail() {
        let mut body = SemBlock::empty();
        // Pre-existing structurer-allocated label 0; switch will mint 1, 2.
        body.stmts.push(SemStmt::Label {
            id: 0,
            source_block: 0,
        });
        let mut resolved = SwitchResolutions::new();
        resolved.insert(
            7,
            ResolvedSwitch {
                scrutinee: 99,
                arms: vec![(0, 1), (1, 2)],
                labels: vec![(11, 1), (12, 2)],
            },
        );
        append_orphan_labels(&mut body, &resolved);
        // Body should now end with Label{1}, Label{2}.
        let tail_ids: Vec<dac_ir::sem::LabelId> = body
            .stmts
            .iter()
            .filter_map(|s| match s {
                SemStmt::Label { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(tail_ids, vec![0, 1, 2]);
    }

    // ---- B3.28 canonical-entry + hint-arity follow-up --------------

    use dac_core::{Confidence, Source};
    use dac_hints::{FunctionHint, HintMatcher, HintType, Hints};
    use dac_ir::ssa::SsaTerminator as IrSsaTerminator;
    use dac_recovery::{Function as RecoveredFunction, FunctionKind, SourceMask};

    fn b328_function_named(name: &str) -> RecoveredFunction {
        RecoveredFunction {
            address: 0x1000,
            end: Some(0x1010),
            name: Some(name.to_string()),
            confidence: Confidence::new(1.0, Source::Observed),
            sources: SourceMask::SYMBOL,
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Cfg,
                id: 0,
            }),
            kind: FunctionKind::User,
        }
    }

    fn b328_convention(name: &'static str, args: Vec<RegisterArg>) -> ConventionMatch {
        ConventionMatch {
            convention_name: name,
            signature: dac_recovery::InferredSignature {
                int_args: args,
                stack_args: vec![],
                return_register: None,
                variadic_call_sites: 0,
            },
            confidence: Confidence::new(0.5, Source::Derived),
        }
    }

    /// SSA with one block whose terminator returns a value of id `rv`.
    /// `rv` must already be present in `values`.
    fn b328_ssa_with_return(
        values: Vec<dac_ir::ssa::ValueDef>,
        rv: dac_ir::ssa::ValueId,
    ) -> SsaFunction {
        SsaFunction {
            function_address: 0x1000,
            function_name: Some("main".into()),
            blocks: vec![dac_ir::ssa::SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![],
                terminator: IrSsaTerminator::Return {
                    value: Some(Operand::Value(rv)),
                },
            }],
            entry: 0,
            variables: vec![dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values,
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Ssa,
                id: 0,
            }),
        }
    }

    /// `main` reading no arg registers resolves to `int main(void)`.
    /// The canonical override pins the return type to `"int"` and the
    /// param list to empty even though the lattice would have left
    /// the return at the i64 fallback.
    #[test]
    fn b3_28_main_zero_args_synthesizes_int_main_void() {
        let f = b328_function_named("main");
        let rv = 1;
        let ssa = b328_ssa_with_return(
            vec![
                dac_ir::ssa::ValueDef {
                    id: 0,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Parameter { variable: 0 },
                },
                dac_ir::ssa::ValueDef {
                    id: rv,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                },
            ],
            rv,
        );
        let mut convention = Some(b328_convention("sysv-amd64", vec![]));
        let mut types = TypeMap::default();
        let canon =
            apply_canonical_entry(&f, &ssa, &mut convention, &mut types).expect("main matches");
        assert_eq!(canon.return_type, Some(CType::Named("int".into())));
        assert!(canon.params.is_empty(), "void → 0 params");
        // Return value got the canonical IR type so the annotation
        // channel agrees with the C backend's spelling.
        let ret_ty = types.value_type(rv);
        assert_eq!(ret_ty, dac_ir::Type::signed_int(32));
    }

    /// `main` with `rdi` and `rsi` observed → canonical signature is
    /// `int main(int argc, char ** argv)`. The TypeMap is seeded for
    /// both args so the annotation channel reports the canonical IR
    /// types alongside the C backend's catalogue-spelt rendering.
    #[test]
    fn b3_28_main_two_args_observed_synthesizes_argc_argv() {
        let f = b328_function_named("main");
        // Two parameter values for rdi, rsi.
        let rdi_val: dac_ir::ssa::ValueId = 0;
        let rsi_val: dac_ir::ssa::ValueId = 1;
        let rv: dac_ir::ssa::ValueId = 2;
        let ssa = b328_ssa_with_return(
            vec![
                dac_ir::ssa::ValueDef {
                    id: rdi_val,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Parameter { variable: 0 },
                },
                dac_ir::ssa::ValueDef {
                    id: rsi_val,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Parameter { variable: 0 },
                },
                dac_ir::ssa::ValueDef {
                    id: rv,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                },
            ],
            rv,
        );
        let args = vec![
            RegisterArg {
                register: "rdi",
                index: 0,
                value: rdi_val,
                variable: 0,
            },
            RegisterArg {
                register: "rsi",
                index: 1,
                value: rsi_val,
                variable: 0,
            },
        ];
        let mut convention = Some(b328_convention("sysv-amd64", args));
        let mut types = TypeMap::default();
        let canon =
            apply_canonical_entry(&f, &ssa, &mut convention, &mut types).expect("canonical fires");
        assert_eq!(canon.return_type, Some(CType::Named("int".into())));
        assert_eq!(canon.params.len(), 2);
        assert_eq!(canon.params[0].name, "argc");
        assert_eq!(canon.params[0].ty, CType::Named("int".into()));
        assert_eq!(canon.params[1].name, "argv");
        assert_eq!(canon.params[1].ty, CType::Named("char **".into()));
        // Each kept slot's TypeMap entry agrees with the catalogue.
        assert_eq!(types.value_type(rdi_val), dac_ir::Type::signed_int(32));
        let argv_ir = dac_ir::Type::ptr_to(dac_ir::Type::ptr_to(dac_ir::Type::signed_int(8)));
        assert_eq!(types.value_type(rsi_val), argv_ir);
    }

    /// When the observed arity exceeds the canonical catalogue
    /// (e.g. a PE `main` reading `rcx, rdx, r8, r9` — typically a
    /// CRT-side misclassification), the canonical override declines
    /// to apply. The convention's `int_args` list is left untouched
    /// so the body's existing references to all four slots keep
    /// resolving (I-1 — the IR stays ground truth).
    #[test]
    fn b3_28_main_too_many_observed_args_skips_canonical() {
        let f = b328_function_named("main");
        let rv: dac_ir::ssa::ValueId = 4;
        let mut values: Vec<dac_ir::ssa::ValueDef> = (0u32..4)
            .map(|i| dac_ir::ssa::ValueDef {
                id: i,
                variable: 0,
                source: dac_ir::ssa::ValueSource::Parameter { variable: 0 },
            })
            .collect();
        values.push(dac_ir::ssa::ValueDef {
            id: rv,
            variable: 0,
            source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
        });
        let ssa = b328_ssa_with_return(values, rv);
        let args: Vec<RegisterArg> = ["rcx", "rdx", "r8", "r9"]
            .iter()
            .enumerate()
            .map(|(i, &reg)| RegisterArg {
                register: reg,
                index: i,
                value: i as u32,
                variable: 0,
            })
            .collect();
        let mut convention = Some(b328_convention("ms-x64", args.clone()));
        let mut types = TypeMap::default();
        let canon = apply_canonical_entry(&f, &ssa, &mut convention, &mut types);
        assert!(canon.is_none(), "observed > canonical max → no override");
        // The convention's int_args list is left exactly as the
        // inference pass produced it — no truncation, no retyping.
        assert_eq!(convention.as_ref().unwrap().signature.int_args.len(), 4);
        // No return-value type was pinned (canonical declined).
        assert_eq!(types.value_type(rv), dac_ir::Type::Unknown);
    }

    /// Functions whose name doesn't appear in the canonical catalogue
    /// (e.g. an ordinary user function `add_widget`) get no override
    /// — `apply_canonical_entry` returns `None` immediately and the
    /// type lattice / convention signature stay untouched.
    #[test]
    fn b3_28_non_canonical_name_skips_canonical_overlay() {
        let f = b328_function_named("add_widget");
        let ssa = b328_ssa_with_return(
            vec![dac_ir::ssa::ValueDef {
                id: 0,
                variable: 0,
                source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            }],
            0,
        );
        let mut convention = Some(b328_convention("sysv-amd64", vec![]));
        let mut types = TypeMap::default();
        let canon = apply_canonical_entry(&f, &ssa, &mut convention, &mut types);
        assert!(canon.is_none());
        assert_eq!(types.value_type(0), dac_ir::Type::Unknown);
    }

    /// A `[[function]]` hint that declares more arg slots than the
    /// convention observed mints synthetic `RegisterArg` entries for
    /// the missing tail. Each minted slot picks the next register in
    /// the convention's `int_arg_registers` table and seeds the
    /// `TypeMap` with the hint-specified IR type so the C backend's
    /// `parameter_type` lookup resolves to the hinted spelling.
    #[test]
    fn b3_28_hint_arity_extension_mints_missing_register_args() {
        let f = b328_function_named("user_function");
        let ssa = b328_ssa_with_return(
            vec![dac_ir::ssa::ValueDef {
                id: 0,
                variable: 0,
                source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            }],
            0,
        );
        let mut hints = Hints::default();
        hints.functions.push(FunctionHint {
            id: 1u64,
            line: 1,
            matcher: HintMatcher::Address(0x1000),
            rename: None,
            return_ty: None,
            // 3 args declared; convention observed 0.
            args: Some(vec![
                HintType::Int {
                    width_bits: 32,
                    signed: Some(true),
                },
                HintType::Int {
                    width_bits: 64,
                    signed: Some(false),
                },
                HintType::Ptr(Box::new(HintType::Int {
                    width_bits: 8,
                    signed: Some(false),
                })),
            ]),
            evidence: None,
        });
        let mut convention = Some(b328_convention("sysv-amd64", vec![]));
        let mut types = TypeMap::default();
        let applied = apply_function_hint(&f, &hints, &ssa, &mut convention, &mut types)
            .expect("hint matched");
        assert_eq!(applied.args_synthesized, 3, "all 3 slots minted");
        assert_eq!(applied.args_overridden, 0, "nothing to retype in-place");
        let int_args = &convention.as_ref().unwrap().signature.int_args;
        assert_eq!(int_args.len(), 3, "int_args grew to hint arity");
        assert_eq!(int_args[0].register, "rdi");
        assert_eq!(int_args[1].register, "rsi");
        assert_eq!(int_args[2].register, "rdx");
        // Each minted slot's TypeMap entry matches the hint's
        // declared IR type.
        for (i, arg) in int_args.iter().enumerate() {
            assert!(
                types.values.contains_key(&arg.value),
                "minted slot {i} got a TypeMap entry",
            );
        }
        // Synthetic value IDs sit in the high-bit reserved range so
        // they cannot collide with an SSA-allocated value.
        assert!(int_args[0].value >= 0xFFFF_FF00);
        assert!(int_args[1].value >= 0xFFFF_FF00);
        assert!(int_args[2].value >= 0xFFFF_FF00);
    }

    /// A hint whose arity matches or shrinks below the observed
    /// prefix synthesises nothing; only the existing slots get
    /// retyped. Sanity check that the arity-synthesis path doesn't
    /// fire when the hint doesn't ask for it.
    #[test]
    fn b3_28_hint_arity_at_or_below_observed_does_not_synthesize() {
        let f = b328_function_named("user_function");
        let ssa = b328_ssa_with_return(
            vec![
                dac_ir::ssa::ValueDef {
                    id: 0,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Parameter { variable: 0 },
                },
                dac_ir::ssa::ValueDef {
                    id: 1,
                    variable: 0,
                    source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
                },
            ],
            1,
        );
        let mut hints = Hints::default();
        hints.functions.push(FunctionHint {
            id: 1u64,
            line: 1,
            matcher: HintMatcher::Address(0x1000),
            rename: None,
            return_ty: None,
            args: Some(vec![HintType::Int {
                width_bits: 32,
                signed: Some(true),
            }]),
            evidence: None,
        });
        let observed = vec![RegisterArg {
            register: "rdi",
            index: 0,
            value: 0,
            variable: 0,
        }];
        let mut convention = Some(b328_convention("sysv-amd64", observed));
        let mut types = TypeMap::default();
        let applied = apply_function_hint(&f, &hints, &ssa, &mut convention, &mut types)
            .expect("hint matched");
        assert_eq!(applied.args_synthesized, 0);
        assert_eq!(applied.args_overridden, 1);
        assert_eq!(
            convention.as_ref().unwrap().signature.int_args.len(),
            1,
            "no synthesis when hint arity <= observed",
        );
    }

    // ---- B3.29 return-type inference -------------------------------

    fn b329_value(
        id: u32,
        variable: u32,
        source: dac_ir::ssa::ValueSource,
    ) -> dac_ir::ssa::ValueDef {
        dac_ir::ssa::ValueDef {
            id,
            variable,
            source,
        }
    }

    fn b329_block_with_terminator(
        instructions: Vec<dac_ir::ssa::SsaInstruction>,
        terminator: IrSsaTerminator,
    ) -> dac_ir::ssa::SsaBlock {
        dac_ir::ssa::SsaBlock {
            id: 0,
            predecessors: vec![],
            phis: vec![],
            instructions,
            terminator,
        }
    }

    fn b329_ssa(
        variables: Vec<dac_ir::ssa::Variable>,
        values: Vec<dac_ir::ssa::ValueDef>,
        block: dac_ir::ssa::SsaBlock,
    ) -> SsaFunction {
        SsaFunction {
            function_address: 0x1000,
            function_name: Some("fn".into()),
            blocks: vec![block],
            entry: 0,
            variables,
            values,
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Ssa,
                id: 0,
            }),
        }
    }

    fn b329_convention_with_rax_return() -> ConventionMatch {
        ConventionMatch {
            convention_name: "sysv-amd64",
            signature: dac_recovery::InferredSignature {
                int_args: vec![],
                stack_args: vec![],
                return_register: Some("rax"),
                variadic_call_sites: 0,
            },
            confidence: Confidence::new(0.5, Source::Derived),
        }
    }

    /// No convention return register, no return value → `Void`.
    #[test]
    fn b3_29_void_when_no_return_register_pinned() {
        let ssa = b329_ssa(
            vec![dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            vec![],
            b329_block_with_terminator(vec![], IrSsaTerminator::Return { value: None }),
        );
        let convention = ConventionMatch {
            convention_name: "sysv-amd64",
            signature: dac_recovery::InferredSignature {
                int_args: vec![],
                stack_args: vec![],
                return_register: None,
                variadic_call_sites: 0,
            },
            confidence: Confidence::new(0.0, Source::Derived),
        };
        assert_eq!(
            infer_return_type(&ssa, Some(&convention)),
            InferredReturn::Void,
        );
    }

    /// Convention pins rax but no `Return { Some(_) }` and no writes
    /// to rax in the body → `Void` (the spec's "void for none" case).
    #[test]
    fn b3_29_void_when_no_writes_and_no_value_return() {
        let ssa = b329_ssa(
            vec![dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            vec![],
            b329_block_with_terminator(vec![], IrSsaTerminator::Return { value: None }),
        );
        assert_eq!(
            infer_return_type(&ssa, Some(&b329_convention_with_rax_return())),
            InferredReturn::Void,
        );
    }

    /// A single Load width 4 written into rax (`mov eax, [mem]` shape)
    /// → `Width32` → `int` return spelling.
    #[test]
    fn b3_29_load_width_4_into_rax_renders_as_int() {
        // values:
        //   0 = Load { width: 4 } → temp (variable 1)
        //   1 = Move { src: Value(0) } → rax (variable 0)
        // Terminator returns value 1.
        let variables = vec![
            dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            },
            dac_ir::ssa::Variable {
                id: 1,
                name: "t0".into(),
                width_bits: 32,
            },
            dac_ir::ssa::Variable {
                id: 2,
                name: "rsp".into(),
                width_bits: 64,
            },
        ];
        let values = vec![
            // rsp parameter (used as address)
            b329_value(0, 2, dac_ir::ssa::ValueSource::Parameter { variable: 2 }),
            // Load dst
            b329_value(
                1,
                1,
                dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            ),
            // Move dst (writes rax)
            b329_value(
                2,
                0,
                dac_ir::ssa::ValueSource::Instruction { block: 0, index: 1 },
            ),
        ];
        let instructions = vec![
            dac_ir::ssa::SsaInstruction {
                dst: Some(1),
                op: dac_ir::ssa::SsaOp::Load {
                    address: Operand::Value(0),
                    width: 4,
                },
            },
            dac_ir::ssa::SsaInstruction {
                dst: Some(2),
                op: dac_ir::ssa::SsaOp::Move {
                    src: Operand::Value(1),
                },
            },
        ];
        let ssa = b329_ssa(
            variables,
            values,
            b329_block_with_terminator(
                instructions,
                IrSsaTerminator::Return {
                    value: Some(Operand::Value(2)),
                },
            ),
        );
        assert_eq!(
            infer_return_type(&ssa, Some(&b329_convention_with_rax_return())),
            InferredReturn::Width32,
        );
    }

    /// A single Load width 8 written into rax → `Width64` → `long`.
    #[test]
    fn b3_29_load_width_8_into_rax_renders_as_long() {
        let variables = vec![
            dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            },
            dac_ir::ssa::Variable {
                id: 1,
                name: "t0".into(),
                width_bits: 64,
            },
            dac_ir::ssa::Variable {
                id: 2,
                name: "rsp".into(),
                width_bits: 64,
            },
        ];
        let values = vec![
            b329_value(0, 2, dac_ir::ssa::ValueSource::Parameter { variable: 2 }),
            b329_value(
                1,
                1,
                dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            ),
            b329_value(
                2,
                0,
                dac_ir::ssa::ValueSource::Instruction { block: 0, index: 1 },
            ),
        ];
        let instructions = vec![
            dac_ir::ssa::SsaInstruction {
                dst: Some(1),
                op: dac_ir::ssa::SsaOp::Load {
                    address: Operand::Value(0),
                    width: 8,
                },
            },
            dac_ir::ssa::SsaInstruction {
                dst: Some(2),
                op: dac_ir::ssa::SsaOp::Move {
                    src: Operand::Value(1),
                },
            },
        ];
        let ssa = b329_ssa(
            variables,
            values,
            b329_block_with_terminator(
                instructions,
                IrSsaTerminator::Return {
                    value: Some(Operand::Value(2)),
                },
            ),
        );
        assert_eq!(
            infer_return_type(&ssa, Some(&b329_convention_with_rax_return())),
            InferredReturn::Width64,
        );
    }

    /// A function whose body returns a value but performs no
    /// observable writes to the return register (e.g. the lifter
    /// reads rax as a Parameter and returns it directly) stays at
    /// `Width64` rather than collapsing to Void — the structurer
    /// would otherwise emit `return v<id>;` against a `void`
    /// declarator and the compile gate would fail.
    #[test]
    fn b3_29_returns_value_without_writes_stays_at_long() {
        let variables = vec![dac_ir::ssa::Variable {
            id: 0,
            name: "rax".into(),
            width_bits: 64,
        }];
        let values = vec![
            // rax entry-value parameter.
            b329_value(0, 0, dac_ir::ssa::ValueSource::Parameter { variable: 0 }),
        ];
        let ssa = b329_ssa(
            variables,
            values,
            b329_block_with_terminator(
                vec![],
                IrSsaTerminator::Return {
                    value: Some(Operand::Value(0)),
                },
            ),
        );
        assert_eq!(
            infer_return_type(&ssa, Some(&b329_convention_with_rax_return())),
            InferredReturn::Width64,
        );
    }

    /// After [`dac_recovery::simplify`] folds the lifter's
    /// Move-into-rax sequence, the SSA carries no instruction whose
    /// dst variable is rax — the Return references the Load's dst
    /// (a synth temp) directly. The inference must still publish
    /// `Width32` by probing the Return value's source op.
    #[test]
    fn b3_29_post_simplify_move_into_rax_folded_still_publishes_width32() {
        // values:
        //   0 = rsp parameter (used as address)
        //   1 = Load { width: 4, address: v0 } → synth_temp (variable 1)
        // Terminator returns value 1 directly — no Move-into-rax (the
        // simplifier collapsed it). Conventions still pin `rax` as the
        // return register.
        let variables = vec![
            dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            },
            dac_ir::ssa::Variable {
                id: 1,
                name: "t0".into(),
                width_bits: 32,
            },
            dac_ir::ssa::Variable {
                id: 2,
                name: "rsp".into(),
                width_bits: 64,
            },
        ];
        let values = vec![
            b329_value(0, 2, dac_ir::ssa::ValueSource::Parameter { variable: 2 }),
            b329_value(
                1,
                1,
                dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            ),
        ];
        let instructions = vec![dac_ir::ssa::SsaInstruction {
            dst: Some(1),
            op: dac_ir::ssa::SsaOp::Load {
                address: Operand::Value(0),
                width: 4,
            },
        }];
        let ssa = b329_ssa(
            variables,
            values,
            b329_block_with_terminator(
                instructions,
                IrSsaTerminator::Return {
                    value: Some(Operand::Value(1)),
                },
            ),
        );
        assert_eq!(
            infer_return_type(&ssa, Some(&b329_convention_with_rax_return())),
            InferredReturn::Width32,
        );
    }

    /// `Width32` mints `CType::Named("int")`; `Width64` mints
    /// `CType::Named("long")`; `Void` mints `CType::Void`.
    #[test]
    fn b3_29_to_c_type_spellings_match_spec() {
        assert_eq!(InferredReturn::Void.to_c_type(), CType::Void);
        assert_eq!(
            InferredReturn::Width32.to_c_type(),
            CType::Named("int".into()),
        );
        assert_eq!(
            InferredReturn::Width64.to_c_type(),
            CType::Named("long".into()),
        );
    }

    // ---- B3.34 whole-callgraph void-return inference ---------------

    /// SSA function with no calls — a leaf that returns a value.
    /// Used as a callee target to verify the demote-when-unobserved
    /// path fires.
    fn b334_leaf_ssa(addr: u64) -> SsaFunction {
        SsaFunction {
            function_address: addr,
            function_name: Some(format!("fn_{:x}", addr)),
            blocks: vec![dac_ir::ssa::SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions: vec![],
                terminator: IrSsaTerminator::Return {
                    value: Some(Operand::Const(0)),
                },
            }],
            entry: 0,
            variables: vec![dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values: vec![],
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Ssa,
                id: 0,
            }),
        }
    }

    /// SSA function that issues a single direct call to `callee_addr`
    /// and `use_kind` describes what the caller does with the return:
    ///
    /// - `Drop`: instruction's `dst` is `None` — the lifter never
    ///   materialised the return slot.
    /// - `Unused`: `dst` is `Some(v)` but `v` has zero non-terminator
    ///   uses (def-use sees a 0-use entry).
    /// - `Returned`: `dst` is `Some(v)` and the SSA terminator returns
    ///   `v` straight back — feeds the caller's return value.
    /// - `Escapes`: `dst` is `Some(v)` and `v` is used as a Store's
    ///   value operand — a hard sink that always counts as observed.
    enum B334Use {
        Drop,
        Unused,
        Returned,
        Escapes,
    }

    fn b334_caller_ssa(caller_addr: u64, callee_addr: u64, use_kind: B334Use) -> SsaFunction {
        use dac_ir::ssa::{SsaOp as Op, ValueDef, ValueSource};
        let dst = match use_kind {
            B334Use::Drop => None,
            _ => Some(0u32),
        };
        let mut instructions = vec![dac_ir::ssa::SsaInstruction {
            dst,
            op: Op::Call {
                target: Some(callee_addr),
                args: vec![],
            },
        }];
        let mut values = vec![];
        if let Some(v) = dst {
            values.push(ValueDef {
                id: v,
                variable: 0,
                source: ValueSource::Instruction { block: 0, index: 0 },
            });
        }
        let terminator = match use_kind {
            B334Use::Returned => IrSsaTerminator::Return {
                value: Some(Operand::Value(0)),
            },
            B334Use::Escapes => {
                // A Store whose `value` operand consumes the call's
                // dst — observed sink.
                instructions.push(dac_ir::ssa::SsaInstruction {
                    dst: None,
                    op: Op::Store {
                        address: Operand::Const(0x100),
                        value: Operand::Value(0),
                        width: 8,
                    },
                });
                IrSsaTerminator::Return { value: None }
            }
            _ => IrSsaTerminator::Return { value: None },
        };
        SsaFunction {
            function_address: caller_addr,
            function_name: Some(format!("fn_{:x}", caller_addr)),
            blocks: vec![dac_ir::ssa::SsaBlock {
                id: 0,
                predecessors: vec![],
                phis: vec![],
                instructions,
                terminator,
            }],
            entry: 0,
            variables: vec![dac_ir::ssa::Variable {
                id: 0,
                name: "rax".into(),
                width_bits: 64,
            }],
            values,
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Ssa,
                id: 0,
            }),
        }
    }

    fn b334_real_outcome(ssa: SsaFunction) -> LiftOutcome {
        let sem = SemFunction {
            function_address: ssa.function_address,
            function_name: ssa.function_name.clone(),
            body: SemBlock::empty(),
            evidence: dac_core::EvidenceGraph::new().add_node(dac_core::EvidenceNode::IrNode {
                layer: dac_core::IrLayer::Semantic,
                id: 0,
            }),
            stats: dac_ir::sem::StructuringStats::default(),
        };
        let facts = Box::new(facts_default());
        LiftOutcome::Real { ssa, sem, facts }
    }

    /// A callee whose only direct caller drops the return register
    /// (the lifter emitted `Call { dst: None }`) demotes to `void`.
    #[test]
    fn b3_34_demotes_when_every_caller_drops_dst() {
        let caller = b334_caller_ssa(0x2000, 0x1000, B334Use::Drop);
        let callee = b334_leaf_ssa(0x1000);
        let mut outcomes = vec![b334_real_outcome(caller), b334_real_outcome(callee)];
        apply_void_return_inference(&mut outcomes);
        // Both demote — the caller's own return is also unobserved.
        for o in &outcomes {
            let LiftOutcome::Real { facts, .. } = o else {
                unreachable!()
            };
            assert!(facts.void_return_demoted, "every outcome should demote");
        }
    }

    /// A callee whose only direct caller materialises the return slot
    /// but never reads it (`dst = Some(v)` with zero uses) still
    /// demotes — the def-use chain reports `use_count(v) == 0`.
    #[test]
    fn b3_34_demotes_when_caller_dst_is_unused() {
        let caller = b334_caller_ssa(0x2000, 0x1000, B334Use::Unused);
        let callee = b334_leaf_ssa(0x1000);
        let mut outcomes = vec![b334_real_outcome(caller), b334_real_outcome(callee)];
        apply_void_return_inference(&mut outcomes);
        let LiftOutcome::Real { facts, .. } = &outcomes[1] else {
            unreachable!()
        };
        assert!(
            facts.void_return_demoted,
            "callee should demote when the only caller's call-dst is unused",
        );
    }

    /// A callee whose caller's return is observed — bootstrapped via
    /// the canonical signature override — stays non-void when the
    /// caller's body feeds the call's dst straight into its own
    /// `Return` operand.
    #[test]
    fn b3_34_keeps_return_when_caller_is_observed_and_passes_through() {
        let caller_ssa = b334_caller_ssa(0x2000, 0x1000, B334Use::Returned);
        let callee_ssa = b334_leaf_ssa(0x1000);
        let mut caller = b334_real_outcome(caller_ssa);
        // Bootstrap the caller as observed by attaching a canonical
        // signature pinning a non-void return type — mirrors what
        // `apply_canonical_entry` does for `main`.
        if let LiftOutcome::Real { facts, .. } = &mut caller {
            facts.canonical_signature = Some(SynthesizedSignature {
                return_type: Some(CType::Named("int".into())),
                params: vec![],
            });
        }
        let callee = b334_real_outcome(callee_ssa);
        let mut outcomes = vec![caller, callee];
        apply_void_return_inference(&mut outcomes);
        let LiftOutcome::Real {
            facts: caller_facts,
            ..
        } = &outcomes[0]
        else {
            unreachable!()
        };
        let LiftOutcome::Real {
            facts: callee_facts,
            ..
        } = &outcomes[1]
        else {
            unreachable!()
        };
        assert!(
            !caller_facts.void_return_demoted,
            "canonical-signature seed must keep the caller's return live",
        );
        assert!(
            !callee_facts.void_return_demoted,
            "callee's return flows into observed caller's return — must stay live",
        );
    }

    /// The escape predicate catches "hard sinks": a Store whose
    /// `value` operand consumes the call's dst observes the value
    /// regardless of the caller's own demote status.
    #[test]
    fn b3_34_store_value_use_keeps_callee_return_live() {
        let caller = b334_caller_ssa(0x2000, 0x1000, B334Use::Escapes);
        let callee = b334_leaf_ssa(0x1000);
        let mut outcomes = vec![b334_real_outcome(caller), b334_real_outcome(callee)];
        apply_void_return_inference(&mut outcomes);
        let LiftOutcome::Real {
            facts: callee_facts,
            ..
        } = &outcomes[1]
        else {
            unreachable!()
        };
        assert!(
            !callee_facts.void_return_demoted,
            "callee should stay live when the call's dst flows into a Store value",
        );
    }

    /// `Stub` outcomes have no SSA to walk and must be left alone —
    /// the pass touches neither their (non-existent) `facts` nor any
    /// surrounding outcome's state in response to seeing a stub.
    #[test]
    fn b3_34_skips_stub_outcomes() {
        let mut outcomes = vec![
            LiftOutcome::Stub {
                reason: "no end address".into(),
            },
            b334_real_outcome(b334_leaf_ssa(0x1000)),
        ];
        apply_void_return_inference(&mut outcomes);
        // Stub stays a stub; the real leaf with no observed callers
        // demotes by vacuous truth.
        assert!(matches!(&outcomes[0], LiftOutcome::Stub { .. }));
        let LiftOutcome::Real { facts, .. } = &outcomes[1] else {
            unreachable!()
        };
        assert!(facts.void_return_demoted);
    }

    /// Stays-untouched: `wmain` and `WinMain` are recognised the
    /// same way `main` is. Smoke check that the catalogue lookup
    /// honours every entry we ship in B3.28.
    #[test]
    fn b3_28_wmain_and_winmain_resolve_via_catalogue() {
        let f_wmain = b328_function_named("wmain");
        let ssa = b328_ssa_with_return(
            vec![dac_ir::ssa::ValueDef {
                id: 0,
                variable: 0,
                source: dac_ir::ssa::ValueSource::Instruction { block: 0, index: 0 },
            }],
            0,
        );
        let mut convention = Some(b328_convention("sysv-amd64", vec![]));
        let mut types = TypeMap::default();
        let canon = apply_canonical_entry(&f_wmain, &ssa, &mut convention, &mut types);
        let canon = canon.expect("wmain matches catalogue");
        assert_eq!(canon.return_type, Some(CType::Named("int".into())));

        let f_winmain = b328_function_named("WinMain");
        let mut convention = Some(b328_convention("ms-x64", vec![]));
        let mut types = TypeMap::default();
        let canon = apply_canonical_entry(&f_winmain, &ssa, &mut convention, &mut types);
        let canon = canon.expect("WinMain matches catalogue");
        assert_eq!(canon.return_type, Some(CType::Named("int".into())));
    }
}
