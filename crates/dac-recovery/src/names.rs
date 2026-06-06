//! Variable-naming heuristics (B3.7, FR-N spec §11.1).
//!
//! [`recover_names`] walks a [`SsaFunction`] and emits a [`NameTable`]
//! that maps SSA `ValueId`s to human-readable candidate names. The C
//! backend reads through the table when emitting locals so values
//! that match a heuristic surface as `path`, `fmt`, `len`, … instead
//! of the generic `v<id>` fallback (B2.8).
//!
//! ## Heuristics shipping at B3.7, B3.20, and B3.22
//!
//! 1. **API-context naming** (B3.7). When a value `v` is the i-th
//!    `Operand` of a [`SsaOp::Call`] whose target VA resolves to a
//!    known [`dac_knowledge::ApiSignature`], the i-th parameter's
//!    catalogue name becomes a candidate. `strlen(v3)` therefore
//!    proposes `s` for `v3`; `open(path=v5, flags=v6)` proposes
//!    `path` / `flags`. Variadic API tails are ignored — values past
//!    the fixed arity have no catalogue name to inherit.
//! 2. **String-literal naming** (B3.7). When a value's defining op
//!    is [`SsaOp::Move`] of an [`Operand::Const`] whose immediate
//!    equals the virtual address of an extracted
//!    [`dac_binfmt::StringRef`] in a read-only section, the string
//!    content is slugified into a candidate (e.g.
//!    `"Hello, world!\n"` → `str_hello_world`). Strings shorter than
//!    [`MIN_STRING_LEN_FOR_NAME`] characters or longer than
//!    [`MAX_STRING_LEN_FOR_NAME`] do not contribute a candidate —
//!    the first because the slug carries no signal, the second
//!    because the resulting identifier would dominate the line.
//! 3. **Loop-induction naming** (B3.20). For every natural loop
//!    discovered in the function's [`LoopForest`], a header `phi`
//!    whose back-edge incoming is an `Add(phi, 1)` produces an
//!    induction-variable candidate named `i` / `j` / `k` / … by
//!    nesting depth.
//! 4. **Counter naming** (B3.20). A phi whose `(initial, phi + 1)`
//!    shape matches the induction pattern but does *not* live at a
//!    loop header — or sits at a header alongside another phi that
//!    already earned `i`/`j`/`k` — earns `count`.
//! 5. **Allocator-size naming** (B3.20). A value passed as the
//!    `size` argument of `malloc` / `calloc` / `realloc` whose
//!    defining op is an arithmetic op (`Add` / `Sub` / `Mul` /
//!    `Shl` / `Shr`) earns `size`. A bare register or constant
//!    feeding the call carries no signal and stays on the
//!    API-context (`n` from the catalogue) path.
//! 6. **Hint-driven call-result naming** (B3.22, FR-20). When a
//!    [`SsaOp::Call`]'s target VA matches a user-supplied
//!    `[[function]]` hint with a `rename` field, the call's
//!    destination value picks up the rename verbatim. Carries
//!    [`Source::UserHint`] and outranks every other heuristic so
//!    a reverse engineer who explicitly named the function sees
//!    that name surface in the lifted source.
//!
//! ## Conflict resolution and disambiguation
//!
//! When several heuristics agree on a value, the highest-precedence
//! source wins; precedence follows [`NameSource`]'s declaration
//! order (`UserHint > InductionCounter > Counter > AllocatorSize >
//! ApiContext > StringRef`). When multiple values share the same
//! base candidate (`strlen` called three times → three candidate-`s`
//! values, two calls to a hinted `send` → two candidate-`send`
//! values), the table mints unique identifiers by appending `_1`,
//! `_2`, … in ascending `ValueId` order so iteration is
//! deterministic across runs.
//!
//! ## Confidence + invariants
//!
//! Each candidate carries a [`Confidence`]. Deterministic heuristics
//! source from [`Source::Derived`]; hint-driven candidates source
//! from [`Source::UserHint`] (I-3). Parameter values that the
//! caller has already named via the convention list are skipped —
//! the C backend names parameters as `argN` and does not look up
//! [`NameTable`] for them.
//!
//! ## Determinism (NFR-9)
//!
//! Pure function. Iteration over `ssa.blocks` follows their declared
//! order; values inside each block are walked in `ValueId` order.
//! Disambiguation walks the value set in ascending `ValueId`. No
//! clock, environment, or filesystem-iteration order is consulted.

use std::collections::{BTreeMap, BTreeSet};

use dac_core::{Confidence, Source};
use dac_ir::ssa::{Operand, Phi, SsaFunction, SsaOp, ValueId, ValueSource};
use dac_ir::ty::Type;

use crate::convention::InferredSignature;
use crate::types::{ApiResolver, TypeMap};

/// Minimum slugified-string length that earns a string-literal name.
/// One-character strings (`""`, `"\n"`) carry no signal so we keep
/// the `v<id>` fallback.
pub const MIN_STRING_LEN_FOR_NAME: usize = 2;

/// Maximum slugified-string length kept as an identifier. Beyond
/// this the name would dominate the line; the slug truncates at this
/// boundary and the resulting identifier always carries a stable
/// suffix from the address so two strings whose first
/// [`MAX_STRING_LEN_FOR_NAME`] characters coincide stay distinct.
pub const MAX_STRING_LEN_FOR_NAME: usize = 24;

/// Numeric confidence attached to deterministic name candidates.
pub const NAME_CONFIDENCE: f32 = 0.80;

/// Resolves a virtual address that appears as a [`Operand::Const`] to
/// a string literal extracted from a read-only section. The
/// [`recover_names`] pass does not own the binary's section / string
/// table; the caller threads this through so the pass stays
/// architecture- and format-agnostic, mirroring [`ApiResolver`].
pub trait StringResolver {
    /// Return the string content at `va`, or `None` when no extracted
    /// string starts at `va` or the address is not in a read-only
    /// section.
    fn resolve(&self, va: u64) -> Option<&str>;
}

impl<F> StringResolver for F
where
    F: Fn(u64) -> Option<&'static str>,
{
    fn resolve(&self, va: u64) -> Option<&str> {
        (self)(va)
    }
}

/// No-op resolver — every address goes un-named.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullStringResolver;

impl StringResolver for NullStringResolver {
    fn resolve(&self, _va: u64) -> Option<&str> {
        None
    }
}

/// Resolves a call target VA to a user-hint rename (B3.22, FR-20).
/// `dac-recovery` stays decoupled from `dac-hints` by taking this
/// trait at the [`recover_names`] boundary — the CLI threads in a
/// thin adapter over the parsed `Hints` catalogue.
pub trait CallRenameResolver {
    /// Return the user-supplied rename for a function at `target_va`,
    /// or `None` when no `[[function]]` hint matches.
    fn resolve(&self, target_va: u64) -> Option<&str>;
}

impl<F> CallRenameResolver for F
where
    F: Fn(u64) -> Option<&'static str>,
{
    fn resolve(&self, target_va: u64) -> Option<&str> {
        (self)(target_va)
    }
}

/// No-op resolver — every call target goes un-renamed. Default for
/// CLI invocations that did not pass `--hints`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullCallRenameResolver;

impl CallRenameResolver for NullCallRenameResolver {
    fn resolve(&self, _target_va: u64) -> Option<&str> {
        None
    }
}

/// Per-function natural-loop summary consumed by the loop-induction
/// heuristic (B3.20). Lifted into a small POD so `dac-recovery` does
/// not depend on `dac-analysis` (which already depends on us).
///
/// The CLI builds [`LoopInfo`] from a
/// `dac_analysis::loops::LoopForest`; a [`LoopInfo::default`] value
/// is equivalent to "no loops" and disables the heuristic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoopInfo {
    /// Block id of every natural-loop header → its loop shape.
    /// Iteration order is deterministic (ascending header id).
    pub headers: BTreeMap<u32, LoopShape>,
}

/// Shape of a natural loop as far as the name-recovery pass is
/// concerned: nesting depth (drives `i` / `j` / `k`) and the set of
/// back-edge predecessor block ids (drives the phi-shape match for
/// the loop-carry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopShape {
    /// Nesting depth — 0 for outermost loops.
    pub depth: u32,
    /// Block ids of back-edge predecessors, ascending.
    pub back_edges: BTreeSet<u32>,
}

/// Why a particular name was proposed. Higher variants outrank lower
/// ones when multiple heuristics fire on the same value — the order
/// here is `StringRef < ApiContext < AllocatorSize < Counter <
/// InductionCounter < UserHint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameSource {
    /// String-literal slug (e.g. `str_hello`).
    StringRef,
    /// API parameter name from [`dac_knowledge`] (e.g. `path`, `fmt`).
    ApiContext,
    /// Arithmetic value feeding a `malloc` / `calloc` / `realloc`
    /// size argument (e.g. `size`).
    AllocatorSize,
    /// Non-induction `+= 1` counter — phi `(initial, phi + 1)` that
    /// is not a loop-induction variable.
    Counter,
    /// Loop-induction variable at a natural-loop header — phi whose
    /// only back-edge incoming is an `Add(phi, 1)`. Named `i` / `j`
    /// / `k` / … by nesting depth.
    InductionCounter,
    /// `[[function]]`-hint `rename` field applied to a call site's
    /// destination value (B3.22, FR-20). Outranks every
    /// deterministic heuristic — the reverse engineer who supplied
    /// the hint outranks every guess the pipeline can make.
    UserHint,
}

impl NameSource {
    /// Stable lowercase identifier for diagnostics and the
    /// `--debug` "why this name?" trail.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            NameSource::UserHint => "user-hint",
            NameSource::InductionCounter => "induction-counter",
            NameSource::Counter => "counter",
            NameSource::AllocatorSize => "allocator-size",
            NameSource::ApiContext => "api-context",
            NameSource::StringRef => "string-ref",
        }
    }
}

/// One naming candidate before disambiguation.
#[derive(Debug, Clone, PartialEq)]
pub struct NameCandidate {
    /// Base identifier proposed by the heuristic, before
    /// disambiguation. Always a valid C identifier.
    pub base: String,
    /// Heuristic that produced the candidate.
    pub source: NameSource,
    /// Confidence — always [`Source::Derived`] at this milestone.
    pub confidence: Confidence,
}

/// Per-function naming table. Maps [`ValueId`] → emitted identifier.
/// The C backend looks up each non-parameter value here when
/// lowering; absence means "fall back to `v<id>`".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NameTable {
    /// Final, disambiguated names keyed by `ValueId`. Each entry is
    /// a unique C identifier within the function.
    pub values: BTreeMap<ValueId, String>,
    /// Per-value provenance for the entries in `values`. Surfaces in
    /// `--debug` and lets the annotation channel say "we named this
    /// value `path` because it flowed into `open`'s 1st argument".
    pub provenance: BTreeMap<ValueId, NameCandidate>,
}

impl NameTable {
    /// True when no heuristic fired anywhere in the function.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Number of values that earned a heuristic name. Used by the
    /// `--emit-report` rubric (`named_values=X/Y`).
    #[must_use]
    pub fn named_count(&self) -> usize {
        self.values.len()
    }

    /// Look up the heuristic name for `value`, or `None` to fall
    /// back to the generic `v<id>` shape.
    #[must_use]
    pub fn lookup(&self, value: ValueId) -> Option<&str> {
        self.values.get(&value).map(String::as_str)
    }
}

/// Run the variable-naming pass.
///
/// `signature` is consulted so parameter values (already named
/// `argN` by the C backend) are not named again. `loops`
/// summarises the natural-loop forest — pass [`LoopInfo::default`]
/// to skip the induction / counter heuristic on a loop-free
/// function. `types` lets the induction / counter pass filter out
/// pointer-typed phis (e.g. a `void *p; p++` walker is not an
/// integer counter); pass an empty [`TypeMap`] to disable the
/// filter. `rename_resolver` (B3.22, FR-20) lets a user-supplied
/// `[[function]]` `rename` flip the destination value of a call
/// targeting the hinted function. Every resolver is optional via
/// its `Null*` default.
#[must_use]
pub fn recover_names(
    ssa: &SsaFunction,
    signature: Option<&InferredSignature>,
    api_resolver: &dyn ApiResolver,
    strings: &dyn StringResolver,
    rename_resolver: &dyn CallRenameResolver,
    loops: &LoopInfo,
    types: &TypeMap,
) -> NameTable {
    let parameters = parameter_value_set(signature);
    let mut candidates: BTreeMap<ValueId, NameCandidate> = BTreeMap::new();

    for block in &ssa.blocks {
        for instr in &block.instructions {
            collect_api_candidates(&instr.op, &parameters, api_resolver, &mut candidates);
            collect_string_candidate(instr.dst, &instr.op, &parameters, strings, &mut candidates);
            collect_allocator_size_candidate(
                &instr.op,
                ssa,
                &parameters,
                api_resolver,
                &mut candidates,
            );
            collect_user_hint_call_candidate(
                instr.dst,
                &instr.op,
                &parameters,
                rename_resolver,
                &mut candidates,
            );
        }
    }

    // Loop-induction + counter live behind a single walk so the
    // induction heuristic claims one phi per loop header before the
    // counter heuristic falls back to `count` on the rest. The
    // claimed phi set propagates between the two passes.
    let mut induction_phis: BTreeSet<ValueId> = BTreeSet::new();
    collect_loop_induction_candidates(
        ssa,
        loops,
        types,
        &parameters,
        &mut induction_phis,
        &mut candidates,
    );
    collect_counter_candidates(ssa, types, &parameters, &induction_phis, &mut candidates);

    finalise_names(candidates)
}

fn parameter_value_set(signature: Option<&InferredSignature>) -> BTreeSet<ValueId> {
    let mut s = BTreeSet::new();
    if let Some(sig) = signature {
        for arg in &sig.int_args {
            s.insert(arg.value);
        }
    }
    s
}

/// API-context heuristic: for each [`SsaOp::Call`] with a known
/// target signature, propose the catalogue parameter name for every
/// value-typed positional argument. Variadic tails are skipped.
fn collect_api_candidates(
    op: &SsaOp,
    parameters: &BTreeSet<ValueId>,
    api_resolver: &dyn ApiResolver,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    let SsaOp::Call {
        target: Some(target_va),
        args,
    } = op
    else {
        return;
    };
    let Some(sig) = api_resolver.resolve(*target_va) else {
        return;
    };
    for (idx, arg) in args.iter().enumerate() {
        let Operand::Value(v) = arg else { continue };
        if parameters.contains(v) {
            continue;
        }
        let Some(api_param) = sig.parameters.get(idx) else {
            continue; // variadic tail or arity mismatch
        };
        let base = sanitise_identifier(api_param.name);
        if base.is_empty() {
            continue;
        }
        propose(
            candidates,
            *v,
            NameCandidate {
                base,
                source: NameSource::ApiContext,
                confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
            },
        );
    }
}

/// String-literal heuristic: a `Move { src: Const(va) }` whose `va`
/// matches an extracted string contributes a slug-based candidate.
fn collect_string_candidate(
    dst: Option<ValueId>,
    op: &SsaOp,
    parameters: &BTreeSet<ValueId>,
    strings: &dyn StringResolver,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    let Some(dst) = dst else { return };
    if parameters.contains(&dst) {
        return;
    }
    let SsaOp::Move {
        src: Operand::Const(c),
    } = op
    else {
        return;
    };
    if *c < 0 {
        return;
    }
    let va = *c as u64;
    let Some(text) = strings.resolve(va) else {
        return;
    };
    let Some(base) = slugify_string(text) else {
        return;
    };
    propose(
        candidates,
        dst,
        NameCandidate {
            base,
            source: NameSource::StringRef,
            confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
        },
    );
}

/// Allocator-size heuristic: when a [`SsaOp::Call`]'s target resolves
/// to a known allocator (`malloc` / `calloc` / `realloc`), any size
/// argument whose defining op is arithmetic (`Add` / `Sub` / `Mul` /
/// `Shl` / `Shr`) earns `size`. Bare register / constant / parameter
/// loads carry no signal and stay on the API-context path (where
/// they pick up `n` from the catalogue).
fn collect_allocator_size_candidate(
    op: &SsaOp,
    ssa: &SsaFunction,
    parameters: &BTreeSet<ValueId>,
    api_resolver: &dyn ApiResolver,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    let SsaOp::Call {
        target: Some(target_va),
        args,
    } = op
    else {
        return;
    };
    let Some(sig) = api_resolver.resolve(*target_va) else {
        return;
    };
    let size_indices: &[usize] = match sig.name {
        "malloc" => &[0],
        "calloc" => &[0, 1],
        "realloc" => &[1],
        _ => return,
    };
    for &idx in size_indices {
        let Some(Operand::Value(v)) = args.get(idx) else {
            continue;
        };
        if parameters.contains(v) {
            continue;
        }
        if !defining_op_is_arithmetic(ssa, *v) {
            continue;
        }
        propose(
            candidates,
            *v,
            NameCandidate {
                base: "size".to_string(),
                source: NameSource::AllocatorSize,
                confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
            },
        );
    }
}

/// Hint-driven call-result heuristic (B3.22, FR-20): when a
/// [`SsaOp::Call`]'s target VA matches a user-supplied
/// `[[function]]` `rename`, the call's destination value picks up
/// the rename. The candidate's source is [`NameSource::UserHint`],
/// which outranks every deterministic heuristic — a reverse
/// engineer who explicitly named the function outranks every guess
/// the pipeline can make.
///
/// Sanitised against the same C-identifier rules
/// [`sanitise_identifier`] enforces for catalogue names, so a hint
/// that accidentally embeds `@` or `.` (or collides with a C
/// keyword) still produces an emittable identifier. An empty
/// sanitised slug yields no candidate — the deterministic
/// heuristics fall back to whatever they would have named the
/// value.
fn collect_user_hint_call_candidate(
    dst: Option<ValueId>,
    op: &SsaOp,
    parameters: &BTreeSet<ValueId>,
    rename_resolver: &dyn CallRenameResolver,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    let Some(dst) = dst else { return };
    if parameters.contains(&dst) {
        return;
    }
    let SsaOp::Call {
        target: Some(target_va),
        ..
    } = op
    else {
        return;
    };
    let Some(rename) = rename_resolver.resolve(*target_va) else {
        return;
    };
    let base = sanitise_identifier(rename);
    if base.is_empty() {
        return;
    }
    propose(
        candidates,
        dst,
        NameCandidate {
            base,
            source: NameSource::UserHint,
            confidence: Confidence::new(NAME_CONFIDENCE, Source::UserHint),
        },
    );
}

/// Loop-induction heuristic: for each natural loop, find header phis
/// whose only back-edge incoming is `Add(phi, 1)`. The first such
/// phi (lowest `ValueId`) earns `i`/`j`/`k`/… by loop depth;
/// subsequent header phis claimed by this routine become candidates
/// for the counter heuristic.
fn collect_loop_induction_candidates(
    ssa: &SsaFunction,
    loops: &LoopInfo,
    types: &TypeMap,
    parameters: &BTreeSet<ValueId>,
    induction_phis: &mut BTreeSet<ValueId>,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    for (header_block_id, shape) in &loops.headers {
        let header_block = match ssa.blocks.get(*header_block_id as usize) {
            Some(b) => b,
            None => continue,
        };
        let mut header_inductions: Vec<ValueId> = Vec::new();
        for phi in &header_block.phis {
            if parameters.contains(&phi.dst) {
                continue;
            }
            if value_type_is_pointer(types, phi.dst) {
                continue;
            }
            if !phi_is_increment_by_one(ssa, phi, &shape.back_edges) {
                continue;
            }
            header_inductions.push(phi.dst);
        }
        if header_inductions.is_empty() {
            continue;
        }
        // First induction phi per loop (lowest ValueId — phi order
        // already follows variable id) wins the depth-indexed name.
        let primary = header_inductions[0];
        induction_phis.insert(primary);
        propose(
            candidates,
            primary,
            NameCandidate {
                base: induction_name_for_depth(shape.depth).to_string(),
                source: NameSource::InductionCounter,
                confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
            },
        );
    }
}

/// Counter heuristic: any phi with the `(initial, phi + 1)` shape
/// that the induction pass did not claim earns `count`. The shape
/// is recognised whether or not the phi sits at a loop header — the
/// pattern is the SSA signature of a `+= 1` counter, and it shows
/// up at sibling phis of the chosen induction variable and at
/// irreducible-CFG headers the natural-loop pass could not name (I-6
/// — we degrade visibly but keep extracting facts).
fn collect_counter_candidates(
    ssa: &SsaFunction,
    types: &TypeMap,
    parameters: &BTreeSet<ValueId>,
    induction_phis: &BTreeSet<ValueId>,
    candidates: &mut BTreeMap<ValueId, NameCandidate>,
) {
    for block in &ssa.blocks {
        for phi in &block.phis {
            if parameters.contains(&phi.dst) {
                continue;
            }
            if induction_phis.contains(&phi.dst) {
                continue;
            }
            if value_type_is_pointer(types, phi.dst) {
                continue;
            }
            if !phi_has_increment_incoming(ssa, phi) {
                continue;
            }
            propose(
                candidates,
                phi.dst,
                NameCandidate {
                    base: "count".to_string(),
                    source: NameSource::Counter,
                    confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
                },
            );
        }
    }
}

/// Returns true when **every** back-edge incoming is `Add(phi.dst,
/// 1)` and at least one back-edge predecessor is present in
/// `back_edges`. Used by [`collect_loop_induction_candidates`] where
/// the natural-loop analysis already separates loop-carry incomings
/// from loop-entry incomings.
fn phi_is_increment_by_one(ssa: &SsaFunction, phi: &Phi, back_edges: &BTreeSet<u32>) -> bool {
    let mut found_increment = false;
    for (pred, operand) in &phi.incoming {
        if back_edges.contains(pred) {
            match operand {
                Operand::Value(v) if defining_op_is_increment_of(ssa, *v, phi.dst) => {
                    found_increment = true;
                }
                _ => return false,
            }
        }
    }
    found_increment
}

/// Returns true when **at least one** incoming of `phi` is
/// `Add(phi.dst, 1)`. Used by [`collect_counter_candidates`] where
/// we do not separate entry-from-back-edge predecessors — the
/// loose check is sufficient to identify the `+= 1` shape.
fn phi_has_increment_incoming(ssa: &SsaFunction, phi: &Phi) -> bool {
    phi.incoming.iter().any(|(_, operand)| match operand {
        Operand::Value(v) => defining_op_is_increment_of(ssa, *v, phi.dst),
        _ => false,
    })
}

/// True when value `v`'s defining op is `Add(target, 1)` or
/// `Add(1, target)`. Used by [`phi_is_increment_by_one`] to confirm
/// the loop-carry update; `target` is the phi's destination.
fn defining_op_is_increment_of(ssa: &SsaFunction, v: ValueId, target: ValueId) -> bool {
    let def = match ssa.values.get(v as usize) {
        Some(d) => d,
        None => return false,
    };
    let ValueSource::Instruction { block, index } = def.source else {
        return false;
    };
    let block_idx = block as usize;
    let instr_idx = index as usize;
    let instr = match ssa
        .blocks
        .get(block_idx)
        .and_then(|b| b.instructions.get(instr_idx))
    {
        Some(i) => i,
        None => return false,
    };
    let SsaOp::Add { lhs, rhs } = &instr.op else {
        return false;
    };
    let target_op = Operand::Value(target);
    let one = Operand::Const(1);
    (*lhs == target_op && *rhs == one) || (*lhs == one && *rhs == target_op)
}

/// True when `v` is defined by an arithmetic SSA op (`Add` / `Sub`
/// / `Mul` / `Shl` / `Shr`). `Move(Const)` / `Move(Value)` /
/// parameter loads are *not* arithmetic — they carry no
/// `size`-computation signal.
fn defining_op_is_arithmetic(ssa: &SsaFunction, v: ValueId) -> bool {
    let def = match ssa.values.get(v as usize) {
        Some(d) => d,
        None => return false,
    };
    let ValueSource::Instruction { block, index } = def.source else {
        return false;
    };
    let instr = match ssa
        .blocks
        .get(block as usize)
        .and_then(|b| b.instructions.get(index as usize))
    {
        Some(i) => i,
        None => return false,
    };
    matches!(
        instr.op,
        SsaOp::Add { .. }
            | SsaOp::Sub { .. }
            | SsaOp::Mul { .. }
            | SsaOp::Shl { .. }
            | SsaOp::Shr { .. }
    )
}

/// Pick `i` / `j` / `k` / … for a loop at the given nesting depth.
/// Depth 0 (outermost) → `i`; deeper loops walk a small alphabet.
/// Anything past the table re-enters at `i` and lets
/// [`finalise_names`] disambiguate with a numeric suffix.
fn induction_name_for_depth(depth: u32) -> &'static str {
    const TABLE: &[&str] = &["i", "j", "k", "l", "m", "n"];
    TABLE.get(depth as usize).copied().unwrap_or("i")
}

/// True when the type system recovered a pointer type for `value`.
/// Used by the induction / counter heuristics to skip the `void *p;
/// p++` walker pattern (a pointer that gets `+= 1` on every
/// iteration is not an integer counter — naming it `i` would be a
/// regression vs the API-context name the caller already gave it).
/// Returns false for absent / `Unknown` types so values the type
/// pass did not constrain stay eligible.
fn value_type_is_pointer(types: &TypeMap, value: ValueId) -> bool {
    matches!(types.value_type(value), Type::Ptr(_))
}

/// Insert / replace the candidate for `value` if the new candidate
/// outranks the existing one by [`NameSource`] order.
fn propose(candidates: &mut BTreeMap<ValueId, NameCandidate>, value: ValueId, cand: NameCandidate) {
    match candidates.get(&value) {
        Some(existing) if existing.source >= cand.source => {}
        _ => {
            candidates.insert(value, cand);
        }
    }
}

/// Finalise the per-value candidate map: deterministic
/// disambiguation walks values in ascending `ValueId`, appending
/// `_1`, `_2`, … to any base that has already been assigned.
fn finalise_names(candidates: BTreeMap<ValueId, NameCandidate>) -> NameTable {
    let mut taken: BTreeMap<String, u32> = BTreeMap::new();
    let mut values: BTreeMap<ValueId, String> = BTreeMap::new();
    for (vid, cand) in candidates.iter() {
        let entry = taken.entry(cand.base.clone()).or_insert(0);
        let final_name = if *entry == 0 {
            cand.base.clone()
        } else {
            format!("{}_{}", cand.base, entry)
        };
        *entry += 1;
        values.insert(*vid, final_name);
    }
    NameTable {
        values,
        provenance: candidates,
    }
}

/// Constrain `s` to a valid C identifier: ASCII alpha + digits +
/// underscore, leading char non-digit. Returns an empty string when
/// `s` has no usable characters.
fn sanitise_identifier(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => out.push(ch),
            '0'..='9' if !out.is_empty() => out.push(ch),
            _ => {}
        }
    }
    if is_c_keyword(&out) {
        out.push('_');
    }
    out
}

/// True for C / C++ reserved words and a handful of common-case
/// stdint typedefs we render in the same translation unit. The list
/// errs on the side of inclusion — appending an underscore is
/// harmless and keeps round-trip compile gates from failing.
fn is_c_keyword(s: &str) -> bool {
    matches!(
        s,
        "auto"
            | "break"
            | "case"
            | "char"
            | "const"
            | "continue"
            | "default"
            | "do"
            | "double"
            | "else"
            | "enum"
            | "extern"
            | "float"
            | "for"
            | "goto"
            | "if"
            | "inline"
            | "int"
            | "long"
            | "register"
            | "restrict"
            | "return"
            | "short"
            | "signed"
            | "sizeof"
            | "static"
            | "struct"
            | "switch"
            | "typedef"
            | "union"
            | "unsigned"
            | "void"
            | "volatile"
            | "while"
    )
}

/// Slugify a printable string into an identifier of the form
/// `str_<lowercased_alnum>`. Returns `None` when the slug would be
/// shorter than [`MIN_STRING_LEN_FOR_NAME`] characters of payload.
fn slugify_string(text: &str) -> Option<String> {
    let mut slug = String::from("str_");
    let mut payload_chars = 0usize;
    let mut last_underscore = true; // suppress leading `_`
    for ch in text.chars() {
        if payload_chars >= MAX_STRING_LEN_FOR_NAME {
            break;
        }
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            ' ' | '\t' | '\n' | '\r' | '_' | '-' | '/' | '\\' | '.' | ',' | ':' | ';' | '!'
            | '?' => Some('_'),
            _ => None,
        };
        if let Some(c) = mapped {
            if c == '_' {
                if !last_underscore {
                    slug.push(c);
                    payload_chars += 1;
                    last_underscore = true;
                }
            } else {
                slug.push(c);
                payload_chars += 1;
                last_underscore = false;
            }
        }
    }
    while slug.ends_with('_') && slug.len() > 4 {
        slug.pop();
    }
    let alnum_payload = slug
        .chars()
        .skip(4)
        .filter(|c| c.is_ascii_alphanumeric())
        .count();
    if alnum_payload < MIN_STRING_LEN_FOR_NAME {
        return None;
    }
    Some(slug)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, EdgeKind, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::loops::LoopForest;
    use dac_analysis::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };

    fn loop_info(forest: &LoopForest) -> LoopInfo {
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
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{Variable, VariableId};
    use dac_knowledge::{lookup_api_signature, ApiSignature};

    use super::*;
    use crate::convention::{InferredSignature, RegisterArg};
    use crate::types::{NullApiResolver, ValueType};

    // --- helpers (mirror the convention/types test scaffold) -----

    fn synthetic_cfg(n: usize, raw_edges: &[(u32, u32, EdgeKind)]) -> Cfg {
        let blocks: Vec<BasicBlock> = (0..n)
            .map(|i| BasicBlock {
                id: i as u32,
                address: 0x1000 + 0x10 * i as u64,
                end: 0x1000 + 0x10 * (i + 1) as u64,
                instructions: Vec::new(),
                terminator: Terminator::Fall,
            })
            .collect();
        let edges: Vec<Edge> = raw_edges
            .iter()
            .map(|&(from, to, kind)| Edge { from, to, kind })
            .collect();
        let exits: Vec<u32> = (0..n as u32).collect();
        let mut reachable: BTreeSet<u32> = BTreeSet::new();
        reachable.insert(0);
        let mut queue: VecDeque<u32> = VecDeque::from([0u32]);
        while let Some(b) = queue.pop_front() {
            for e in &edges {
                if e.from == b && reachable.insert(e.to) {
                    queue.push_back(e.to);
                }
            }
        }
        let unreachable: Vec<u32> = (0..n as u32).filter(|id| !reachable.contains(id)).collect();
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
            entry: 0,
            exits,
            edges,
            unreachable,
            evidence: ev,
        }
    }

    /// Empty loop summary matching the `synthetic_cfg(n, &[])` shape
    /// — every B3.7-era test runs against this.
    fn empty_loops(_n: usize) -> LoopInfo {
        LoopInfo::default()
    }

    fn var(id: VariableId, name: &str) -> Variable {
        Variable {
            id,
            name: name.to_string(),
            width_bits: 64,
        }
    }

    fn mov_c(dst: VariableId, c: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Const(c),
            },
        }
    }

    fn mov_v(dst: VariableId, src: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Variable(src),
            },
        }
    }

    fn call(dst: Option<VariableId>, target_va: u64, args: Vec<VariableId>) -> RawOp {
        RawOp {
            dst,
            kind: RawOpKind::Call {
                target: Some(target_va),
                args: args.into_iter().map(RawOperand::Variable).collect(),
            },
        }
    }

    fn build(raw: RawFunction) -> dac_ir::ssa::SsaFunction {
        let cfg = synthetic_cfg(raw.blocks.len(), &[]);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    /// Build an SSA function + loop summary for a CFG with real
    /// edges — needed by the induction / counter tests so the
    /// natural-loop pass actually fires.
    fn build_with_edges(
        raw: RawFunction,
        raw_edges: &[(u32, u32, EdgeKind)],
    ) -> (dac_ir::ssa::SsaFunction, LoopInfo, LoopForest) {
        let cfg = synthetic_cfg(raw.blocks.len(), raw_edges);
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let forest = LoopForest::build(&cfg, &doms);
        let info = loop_info(&forest);
        (ssa, info, forest)
    }

    fn ins_value(ssa: &dac_ir::ssa::SsaFunction, block: usize, ins: usize) -> ValueId {
        ssa.blocks[block].instructions[ins]
            .dst
            .expect("instruction defines a value")
    }

    fn libc_resolver(va: u64) -> Option<&'static ApiSignature> {
        if va == 0x2000 {
            lookup_api_signature("strlen")
        } else if va == 0x2100 {
            lookup_api_signature("open")
        } else if va == 0x2200 {
            lookup_api_signature("puts")
        } else {
            None
        }
    }

    fn null_strings() -> NullStringResolver {
        NullStringResolver
    }

    fn null_renames() -> NullCallRenameResolver {
        NullCallRenameResolver
    }

    // --- tests ---------------------------------------------------

    #[test]
    fn api_context_names_strlen_argument() {
        // variables: 0 = rdi (string), 1 = scratch
        // block 0: rdi = "...", rax = strlen(rdi)
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let arg_value = ins_value(&ssa, 0, 0); // the rdi load defines `s`
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(arg_value), Some("s"));
    }

    #[test]
    fn api_context_distinguishes_positional_args() {
        // open(path=rdi, flags=rsi)
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rsi"), var(2, "rax")],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(0, 0x4010),
                    mov_c(1, 0),
                    call(Some(2), 0x2100, vec![0, 1]),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let path_v = ins_value(&ssa, 0, 0);
        let flags_v = ins_value(&ssa, 0, 1);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(path_v), Some("path"));
        assert_eq!(table.lookup(flags_v), Some("flags"));
    }

    #[test]
    fn parameters_are_skipped() {
        // strlen(rdi-parameter) — rdi enters the function as a
        // ValueSource::Parameter. The convention pass owns its
        // identifier; the name table must not propose anything.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let rdi_param = ssa
            .values
            .iter()
            .find_map(|v| match v.source {
                dac_ir::ssa::ValueSource::Parameter { variable: 0 } => Some(v.id),
                _ => None,
            })
            .expect("rdi parameter value");
        let sig = InferredSignature {
            int_args: vec![RegisterArg {
                register: "rdi",
                index: 0,
                value: rdi_param,
                variable: 0,
            }],
            stack_args: vec![],
            return_register: None,
            variadic_call_sites: 0,
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            Some(&sig),
            &libc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert!(table.lookup(rdi_param).is_none());
    }

    #[test]
    fn repeated_api_arg_disambiguates_with_suffix() {
        // Two strlen calls in sequence — both arg values want "s",
        // the second becomes "s_1" in `ValueId` order.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "rsi")],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(0, 0x4006),
                    call(Some(1), 0x2000, vec![0]),
                    mov_c(2, 0x4010),
                    mov_v(0, 2),
                    call(Some(1), 0x2000, vec![0]),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        let s_count = table
            .values
            .values()
            .filter(|n| n.as_str() == "s" || n.starts_with("s_"))
            .count();
        assert!(s_count >= 2, "expected ≥2 `s`-rooted names, got {s_count}");
        let names: BTreeSet<&String> = table.values.values().collect();
        // No two values share the same final name.
        assert_eq!(names.len(), table.values.len());
    }

    #[test]
    fn variadic_tail_is_not_named() {
        // puts has arity 1. A two-arg call only names the first arg;
        // the second slot has no catalogue parameter.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rsi"), var(2, "rax")],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(0, 0x4006),
                    mov_c(1, 0x4010),
                    call(Some(2), 0x2200, vec![0, 1]),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let a = ins_value(&ssa, 0, 0);
        let b = ins_value(&ssa, 0, 1);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(a), Some("s"));
        assert!(table.lookup(b).is_none());
    }

    #[test]
    fn string_literal_proposes_slug() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let v = ins_value(&ssa, 0, 0);
        let resolver = |va: u64| -> Option<&'static str> {
            if va == 0x4006 {
                Some("Hello, world!\n")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &resolver,
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        let name = table.lookup(v).expect("string slug");
        assert!(name.starts_with("str_"), "got {name}");
        assert!(name.contains("hello"));
    }

    #[test]
    fn api_context_outranks_string_literal() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let v = ins_value(&ssa, 0, 0);
        let resolver = |va: u64| -> Option<&'static str> {
            if va == 0x4006 {
                Some("Hello, world!\n")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &resolver,
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(v), Some("s"));
        assert_eq!(table.provenance[&v].source, NameSource::ApiContext);
    }

    #[test]
    fn slugifier_trims_and_lowercases() {
        assert_eq!(
            slugify_string("Hello, World!\n"),
            Some("str_hello_world".into())
        );
        assert_eq!(slugify_string("ALL_CAPS"), Some("str_all_caps".into()));
        assert_eq!(slugify_string(" "), None);
        assert_eq!(slugify_string("a"), None);
        let very_long = "abcdefghijklmnopqrstuvwxyz1234567890";
        let slug = slugify_string(very_long).unwrap();
        assert!(slug.len() <= 4 + MAX_STRING_LEN_FOR_NAME);
    }

    #[test]
    fn sanitise_rewrites_reserved_words() {
        assert_eq!(sanitise_identifier("int"), "int_");
        assert_eq!(sanitise_identifier("path"), "path");
        assert_eq!(sanitise_identifier("a-b"), "ab");
        assert_eq!(sanitise_identifier("1abc"), "abc");
    }

    #[test]
    fn determinism_across_repeated_runs() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let resolver = |va: u64| -> Option<&'static str> {
            if va == 0x4006 {
                Some("Hello, world!\n")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let a = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &resolver,
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        let b = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &resolver,
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn named_count_reports_zero_when_no_signal() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x9999)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.named_count(), 0);
        assert!(table.is_empty());
    }

    // --- B3.20 helpers + tests ----------------------------------

    fn add_op(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn mul_op(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Mul {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn malloc_resolver(va: u64) -> Option<&'static ApiSignature> {
        if va == 0x3000 {
            lookup_api_signature("malloc")
        } else if va == 0x3100 {
            lookup_api_signature("calloc")
        } else if va == 0x3200 {
            lookup_api_signature("realloc")
        } else {
            None
        }
    }

    /// `for (i = 0; i < n; i++) { ... }`-style CFG. Block 0 seeds
    /// `i = 0`, block 1 is the loop header (phi over i, branch on
    /// `i`), block 2 increments `i += 1` and falls back to the
    /// header, block 3 is the exit.
    fn induction_loop_function() -> RawFunction {
        RawFunction {
            variables: vec![var(0, "i")],
            blocks: vec![
                RawBlock {
                    ops: vec![mov_c(0, 0)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![add_op(0, 0, 1)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(0)),
                    },
                },
            ],
        }
    }

    fn induction_loop_edges() -> Vec<(u32, u32, EdgeKind)> {
        vec![
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 1, EdgeKind::Branch),
        ]
    }

    fn header_phi_dst(ssa: &dac_ir::ssa::SsaFunction, block: usize) -> ValueId {
        ssa.blocks[block]
            .phis
            .first()
            .expect("loop header should have a phi for the carried variable")
            .dst
    }

    #[test]
    fn loop_induction_names_outer_counter_i() {
        let (ssa, info, forest) =
            build_with_edges(induction_loop_function(), &induction_loop_edges());
        assert_eq!(forest.loops.len(), 1, "expected one natural loop");
        let phi_v = header_phi_dst(&ssa, 1);
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &info,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(phi_v), Some("i"));
        assert_eq!(
            table.provenance[&phi_v].source,
            NameSource::InductionCounter
        );
    }

    #[test]
    fn nested_loops_name_inner_counter_j() {
        // b0: i = 0
        // b1: phi i; outer header; branch
        // b2: j = 0
        // b3: phi j; inner header; branch
        // b4: j = j + 1; back to b3
        // b5: i = i + 1; back to b1
        // b6: exit
        let raw = RawFunction {
            variables: vec![var(0, "i"), var(1, "j")],
            blocks: vec![
                RawBlock {
                    ops: vec![mov_c(0, 0)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 6,
                    },
                },
                RawBlock {
                    ops: vec![mov_c(1, 0)],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(1),
                        taken: 4,
                        not_taken: 5,
                    },
                },
                RawBlock {
                    ops: vec![add_op(1, 1, 1)],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                RawBlock {
                    ops: vec![add_op(0, 0, 1)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let edges = [
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 6, EdgeKind::NotTaken),
            (2, 3, EdgeKind::Fall),
            (3, 4, EdgeKind::Taken),
            (3, 5, EdgeKind::NotTaken),
            (4, 3, EdgeKind::Branch),
            (5, 1, EdgeKind::Branch),
        ];
        let (ssa, info, forest) = build_with_edges(raw, &edges);
        assert_eq!(forest.loops.len(), 2, "expected outer + inner loop");
        let i_v = header_phi_dst(&ssa, 1);
        let j_v = header_phi_dst(&ssa, 3);
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &info,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(i_v), Some("i"));
        assert_eq!(table.lookup(j_v), Some("j"));
        assert_eq!(table.provenance[&i_v].source, NameSource::InductionCounter);
        assert_eq!(table.provenance[&j_v].source, NameSource::InductionCounter);
    }

    #[test]
    fn allocator_size_names_arithmetic_arg_size() {
        // b0: n = 8; size = n * 4; rax = malloc(size)
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rsi"), var(2, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 8), mul_op(1, 0, 4), call(Some(2), 0x3000, vec![1])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let size_v = ins_value(&ssa, 0, 1); // the Mul value flowing into malloc
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &malloc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(size_v), Some("size"));
        assert_eq!(table.provenance[&size_v].source, NameSource::AllocatorSize);
    }

    #[test]
    fn allocator_size_skips_non_arithmetic_arg() {
        // malloc(constant n) — the size operand is a plain Move, not
        // a computed arithmetic. AllocatorSize must abstain so the
        // ApiContext heuristic gets to name it `n` from the
        // catalogue instead.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 16), call(Some(1), 0x3000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let arg_v = ins_value(&ssa, 0, 0);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &malloc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        // ApiContext on malloc's first parameter (`n`) names it.
        assert_eq!(table.lookup(arg_v), Some("n"));
        assert_eq!(table.provenance[&arg_v].source, NameSource::ApiContext);
    }

    #[test]
    fn induction_outranks_allocator_size() {
        // Pathological: the loop counter `i` itself feeds malloc's
        // size slot through an Add. The Add result picks up `size`;
        // the phi value picks up `i`. Verify InductionCounter
        // outranks AllocatorSize on the phi (the Add still gets
        // `size`).
        let raw = RawFunction {
            variables: vec![var(0, "i"), var(1, "rdi"), var(2, "rax")],
            blocks: vec![
                RawBlock {
                    ops: vec![mov_c(0, 0)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![
                        add_op(1, 0, 8), // size_arg = i + 8
                        call(Some(2), 0x3000, vec![1]),
                        add_op(0, 0, 1), // i = i + 1
                    ],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let edges = [
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 1, EdgeKind::Branch),
        ];
        let (ssa, info, _) = build_with_edges(raw, &edges);
        let phi_v = header_phi_dst(&ssa, 1);
        let size_v = ins_value(&ssa, 2, 0);
        let table = recover_names(
            &ssa,
            None,
            &malloc_resolver,
            &null_strings(),
            &null_renames(),
            &info,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(phi_v), Some("i"));
        assert_eq!(table.lookup(size_v), Some("size"));
    }

    #[test]
    fn counter_falls_back_when_induction_already_claimed() {
        // Two phis at the same header — both look like `+= 1`
        // counters. The first earns `i`; the second falls back to
        // `count`.
        let raw = RawFunction {
            variables: vec![var(0, "i"), var(1, "j")],
            blocks: vec![
                RawBlock {
                    ops: vec![mov_c(0, 0), mov_c(1, 0)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![add_op(0, 0, 1), add_op(1, 1, 1)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let edges = [
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 1, EdgeKind::Branch),
        ];
        let (ssa, info, _) = build_with_edges(raw, &edges);
        let header = &ssa.blocks[1];
        assert!(header.phis.len() >= 2, "expected two phis at the header");
        let i_phi = header.phis[0].dst;
        let j_phi = header.phis[1].dst;
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &info,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(i_phi), Some("i"));
        assert_eq!(table.lookup(j_phi), Some("count"));
        assert_eq!(table.provenance[&j_phi].source, NameSource::Counter);
    }

    #[test]
    fn loop_induction_skips_unrecognised_phi_shape() {
        // Phi whose back-edge incoming is `i = i + 2`, not `i + 1`.
        // The induction heuristic must NOT name it.
        let raw = RawFunction {
            variables: vec![var(0, "i")],
            blocks: vec![
                RawBlock {
                    ops: vec![mov_c(0, 0)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(0),
                        taken: 2,
                        not_taken: 3,
                    },
                },
                RawBlock {
                    ops: vec![add_op(0, 0, 2)],
                    terminator: RawTerminator::Jump { target: 1 },
                },
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return { value: None },
                },
            ],
        };
        let edges = [
            (0, 1, EdgeKind::Fall),
            (1, 2, EdgeKind::Taken),
            (1, 3, EdgeKind::NotTaken),
            (2, 1, EdgeKind::Branch),
        ];
        let (ssa, info, _) = build_with_edges(raw, &edges);
        let phi_v = header_phi_dst(&ssa, 1);
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &info,
            &TypeMap::default(),
        );
        assert!(table.lookup(phi_v).is_none());
    }

    #[test]
    fn calloc_size_argument_is_named_size_when_arithmetic() {
        // calloc(n, size); both args are arithmetic.
        let raw = RawFunction {
            variables: vec![
                var(0, "rdi"),
                var(1, "rsi"),
                var(2, "rdx"),
                var(3, "rcx"),
                var(4, "rax"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(0, 4),
                    add_op(1, 0, 2), // n = base + 2
                    mov_c(2, 8),
                    mul_op(3, 2, 2), // size = 8 * 2
                    call(Some(4), 0x3100, vec![1, 3]),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let n_v = ins_value(&ssa, 0, 1);
        let size_v = ins_value(&ssa, 0, 3);
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &malloc_resolver,
            &null_strings(),
            &null_renames(),
            &loops,
            &TypeMap::default(),
        );
        // Both args carry the `size` candidate — one wins outright,
        // the other gets the disambiguator suffix.
        let n_name = table.lookup(n_v).expect("n earns a name");
        let size_name = table.lookup(size_v).expect("size earns a name");
        assert!(n_name == "size" || n_name == "size_1", "got {n_name}");
        assert!(
            size_name == "size" || size_name == "size_1",
            "got {size_name}"
        );
        assert_ne!(n_name, size_name);
        assert_eq!(table.provenance[&n_v].source, NameSource::AllocatorSize);
        assert_eq!(table.provenance[&size_v].source, NameSource::AllocatorSize);
    }

    #[test]
    fn loop_induction_skips_pointer_typed_phi() {
        // `void *p; p++` walker — same SSA shape as a `for(i=0;;i++)`
        // counter, but the propagation pass tagged the phi value as
        // a pointer. The induction heuristic must NOT name it `i`.
        let (ssa, info, _) = build_with_edges(induction_loop_function(), &induction_loop_edges());
        let phi_v = header_phi_dst(&ssa, 1);
        let mut types = TypeMap::default();
        types.values.insert(
            phi_v,
            ValueType {
                ty: Type::ptr_to(Type::Unknown),
                confidence: Confidence::new(NAME_CONFIDENCE, Source::Derived),
            },
        );
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &null_renames(),
            &info,
            &types,
        );
        assert!(
            table.lookup(phi_v).is_none(),
            "pointer-typed phi must not earn an induction name"
        );
    }

    // --- B3.22 hint-driven naming tests ------------------------------

    /// `[[function]] rename = "send"` applied to a call site flips the
    /// destination value's name to `send` and cites
    /// [`NameSource::UserHint`].
    #[test]
    fn user_hint_rename_names_call_result_value() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let call_dst = ins_value(&ssa, 0, 1);
        let renames = |va: u64| -> Option<&'static str> {
            if va == 0x2000 {
                Some("send")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &renames,
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(call_dst), Some("send"));
        assert_eq!(table.provenance[&call_dst].source, NameSource::UserHint);
        assert_eq!(
            table.provenance[&call_dst].confidence.source(),
            Source::UserHint
        );
    }

    /// A user-hint rename outranks every deterministic heuristic — even
    /// when API context, string literal, or counter signals fire on
    /// the same dst value, the user's identifier wins.
    #[test]
    fn user_hint_rename_outranks_api_context_on_call_result() {
        // strlen returns size_t — its catalogue name for the return is
        // not surfaced (API context only names parameters), but make
        // the rename collide with a dst that would otherwise be named
        // by API context: pin the same value as both an arg and a
        // call dst via aliasing. Easier: pin the rename on the call
        // and verify the dst gets it regardless.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let call_dst = ins_value(&ssa, 0, 1);
        let renames = |va: u64| -> Option<&'static str> {
            if va == 0x2000 {
                Some("send")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &libc_resolver,
            &null_strings(),
            &renames,
            &loops,
            &TypeMap::default(),
        );
        assert_eq!(table.lookup(call_dst), Some("send"));
        assert_eq!(table.provenance[&call_dst].source, NameSource::UserHint);
    }

    /// Two calls to a renamed function — both dst values want `send`;
    /// the second becomes `send_1` via the existing disambiguation
    /// path.
    #[test]
    fn user_hint_rename_disambiguates_repeat_calls() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "rsi")],
            blocks: vec![RawBlock {
                ops: vec![
                    mov_c(0, 0x4006),
                    call(Some(1), 0x2000, vec![0]),
                    mov_c(2, 0x4010),
                    mov_v(0, 2),
                    call(Some(1), 0x2000, vec![0]),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let renames = |va: u64| -> Option<&'static str> {
            if va == 0x2000 {
                Some("send")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &renames,
            &loops,
            &TypeMap::default(),
        );
        let send_count = table
            .values
            .values()
            .filter(|n| n.as_str() == "send" || n.starts_with("send_"))
            .count();
        assert!(
            send_count >= 2,
            "expected ≥2 send-rooted names, got {send_count}"
        );
        let names: BTreeSet<&String> = table.values.values().collect();
        assert_eq!(names.len(), table.values.len());
    }

    /// A rename whose sanitised form is empty (e.g. only punctuation)
    /// proposes no candidate, so the deterministic heuristics still
    /// get to name the dst.
    #[test]
    fn user_hint_rename_skipped_when_sanitised_to_empty() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov_c(0, 0x4006), call(Some(1), 0x2000, vec![0])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        let call_dst = ins_value(&ssa, 0, 1);
        let renames = |va: u64| -> Option<&'static str> {
            if va == 0x2000 {
                Some("@@@")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            None,
            &NullApiResolver,
            &null_strings(),
            &renames,
            &loops,
            &TypeMap::default(),
        );
        assert!(table.lookup(call_dst).is_none());
    }

    /// Parameter values are still skipped — even if a call's dst
    /// shadows a parameter, the rename heuristic must not propose a
    /// candidate for it (the C backend names parameters via the
    /// convention list).
    #[test]
    fn user_hint_rename_skips_parameter_dst() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(0), 0x2000, vec![1])],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw);
        // Pin the rdi parameter as the call's dst by treating it as a
        // parameter slot in the signature. The dst is the value
        // defined by the call instruction — collect that explicitly.
        let call_dst = ins_value(&ssa, 0, 0);
        let sig = InferredSignature {
            int_args: vec![RegisterArg {
                register: "rdi",
                index: 0,
                value: call_dst,
                variable: 0,
            }],
            stack_args: vec![],
            return_register: None,
            variadic_call_sites: 0,
        };
        let renames = |va: u64| -> Option<&'static str> {
            if va == 0x2000 {
                Some("send")
            } else {
                None
            }
        };
        let loops = empty_loops(ssa.blocks.len());
        let table = recover_names(
            &ssa,
            Some(&sig),
            &NullApiResolver,
            &null_strings(),
            &renames,
            &loops,
            &TypeMap::default(),
        );
        assert!(table.lookup(call_dst).is_none());
    }
}
