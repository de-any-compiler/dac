//! Variable-naming heuristics (B3.7, FR-N spec §11.1).
//!
//! [`recover_names`] walks a [`SsaFunction`] and emits a [`NameTable`]
//! that maps SSA `ValueId`s to human-readable candidate names. The C
//! backend reads through the table when emitting locals so values
//! that match a heuristic surface as `path`, `fmt`, `len`, … instead
//! of the generic `v<id>` fallback (B2.8).
//!
//! ## Heuristics shipping at B3.7
//!
//! 1. **API-context naming.** When a value `v` is the i-th `Operand`
//!    of a [`SsaOp::Call`] whose target VA resolves to a known
//!    [`dac_knowledge::ApiSignature`], the i-th parameter's catalogue
//!    name becomes a candidate. `strlen(v3)` therefore proposes `s`
//!    for `v3`; `open(path=v5, flags=v6)` proposes `path` / `flags`.
//!    Variadic API tails are ignored — values past the fixed arity
//!    have no catalogue name to inherit.
//! 2. **String-literal naming.** When a value's defining op is
//!    [`SsaOp::Move`] of an [`Operand::Const`] whose immediate equals
//!    the virtual address of an extracted [`dac_binfmt::StringRef`]
//!    in a read-only section, the string content is slugified into
//!    a candidate (e.g. `"Hello, world!\n"` → `str_hello_world`).
//!    Strings shorter than [`MIN_STRING_LEN_FOR_NAME`] characters or
//!    longer than [`MAX_STRING_LEN_FOR_NAME`] do not contribute a
//!    candidate — the first because the slug carries no signal, the
//!    second because the resulting identifier would dominate the
//!    line.
//!
//! ## What does not ship
//!
//! Loop-induction naming (`i` / `j` / `k`), allocator-size naming
//! (`size` from arithmetic adjacent to a `malloc` call), and
//! counter-pattern naming (`count` for `+= 1` lhs values) are all
//! deferred — each requires extra dataflow reasoning that the spec
//! §11.1 list anticipates landing on the B3 follow-up shelf
//! (recorded in `PLAN.md`).
//!
//! ## Conflict resolution and disambiguation
//!
//! When several heuristics agree on a value, the highest-precedence
//! source wins; precedence follows [`NameSource`]'s declaration
//! order (`ApiContext > StringRef`). When multiple values share the
//! same base candidate (`strlen` called three times → three
//! candidate-`s` values), the table mints unique identifiers by
//! appending `_1`, `_2`, … in ascending `ValueId` order so iteration
//! is deterministic across runs.
//!
//! ## Confidence + invariants
//!
//! Each candidate carries a [`Confidence`] sourced from
//! [`Source::Derived`] (I-3). Parameter values that the caller has
//! already named via the convention list are skipped — the C
//! backend names parameters as `argN` and does not look up
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
use dac_ir::ssa::{Operand, SsaFunction, SsaOp, ValueId};

use crate::convention::InferredSignature;
use crate::types::ApiResolver;

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

/// Why a particular name was proposed. Higher variants outrank lower
/// ones when multiple heuristics fire on the same value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameSource {
    /// String-literal slug (e.g. `str_hello`).
    StringRef,
    /// API parameter name from [`dac_knowledge`] (e.g. `path`, `fmt`).
    ApiContext,
}

impl NameSource {
    /// Stable lowercase identifier for diagnostics and the
    /// `--debug` "why this name?" trail.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
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
/// `argN` by the C backend) are not named again. Both resolvers
/// are optional via their `Null*` defaults.
#[must_use]
pub fn recover_names(
    ssa: &SsaFunction,
    signature: Option<&InferredSignature>,
    api_resolver: &dyn ApiResolver,
    strings: &dyn StringResolver,
) -> NameTable {
    let parameters = parameter_value_set(signature);
    let mut candidates: BTreeMap<ValueId, NameCandidate> = BTreeMap::new();

    for block in &ssa.blocks {
        for instr in &block.instructions {
            collect_api_candidates(&instr.op, &parameters, api_resolver, &mut candidates);
            collect_string_candidate(instr.dst, &instr.op, &parameters, strings, &mut candidates);
        }
    }

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

    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{Variable, VariableId};
    use dac_knowledge::{lookup_api_signature, ApiSignature};

    use super::*;
    use crate::convention::{InferredSignature, RegisterArg};
    use crate::types::NullApiResolver;

    // --- helpers (mirror the convention/types test scaffold) -----

    fn synthetic_cfg(n: usize) -> Cfg {
        let blocks: Vec<BasicBlock> = (0..n)
            .map(|i| BasicBlock {
                id: i as u32,
                address: 0x1000 + 0x10 * i as u64,
                end: 0x1000 + 0x10 * (i + 1) as u64,
                instructions: Vec::new(),
                terminator: Terminator::Fall,
            })
            .collect();
        let edges: Vec<Edge> = Vec::new();
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
        let cfg = synthetic_cfg(raw.blocks.len());
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
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
        let table = recover_names(&ssa, None, &libc_resolver, &null_strings());
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
        let table = recover_names(&ssa, None, &libc_resolver, &null_strings());
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
        let table = recover_names(&ssa, Some(&sig), &libc_resolver, &null_strings());
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
        let table = recover_names(&ssa, None, &libc_resolver, &null_strings());
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
        let table = recover_names(&ssa, None, &libc_resolver, &null_strings());
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
        let table = recover_names(&ssa, None, &NullApiResolver, &resolver);
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
        let table = recover_names(&ssa, None, &libc_resolver, &resolver);
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
        let a = recover_names(&ssa, None, &libc_resolver, &resolver);
        let b = recover_names(&ssa, None, &libc_resolver, &resolver);
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
        let table = recover_names(&ssa, None, &NullApiResolver, &null_strings());
        assert_eq!(table.named_count(), 0);
        assert!(table.is_empty());
    }
}
