//! Calling-convention inference (B2.5, FR-13).
//!
//! Given an SSA function and its recovered [`StackFrame`], score every
//! candidate convention against the observed register and stack usage
//! and return the ranked matches. The pass is purely consultative —
//! it never mutates the IR, only reports what each convention would
//! say about the function. Downstream signature-recovery passes
//! (B2.6 / B3.1) pick a winner and attach an [`crate::signature`]-style
//! summary to the function.
//!
//! ## Signals
//!
//! Four observations drive the score, in decreasing weight:
//!
//! 1. **Argument-register prefix.** A convention passes integer
//!    arguments in a fixed register sequence (`rdi, rsi, rdx, …` on
//!    SysV; `rcx, rdx, r8, r9` on MsX64). The SSA constructor mints
//!    a `Parameter` [`ValueId`] for every variable read without
//!    first being written — for an int argument register, that
//!    parameter *is* the incoming argument. The pass measures the
//!    longest contiguous prefix of the convention's argument
//!    sequence whose registers all appear as parameter reads, and
//!    counts isolated arg-register reads outside that prefix as
//!    soft contradictions.
//! 2. **Caller-saved non-arg reads.** A parameter read of a
//!    caller-saved register that the convention does *not* list as
//!    an argument register (e.g. `rax` on SysV) is a strong
//!    contradiction: the caller does not preserve those registers,
//!    so their value at entry is undefined under this convention.
//! 3. **Return-register definition.** If the function's
//!    `Return { value }` terminator carries a value whose underlying
//!    variable is the convention's integer return register, the
//!    convention gains a small boost.
//! 4. **Stack layout.** Stack accesses at positive offsets are
//!    cross-checked against the convention's `first_stack_arg_offset`
//!    and `shadow_space_bytes`:
//!    - Offsets `>= first_stack_arg_offset` line up with stack-
//!      passed arguments.
//!    - Offsets in `(0, first_stack_arg_offset)` line up with
//!      shadow / home space (MsX64 only).
//!    - An offset that falls in the shadow region is a *negative*
//!      signal for a convention with zero shadow space (SysV), and a
//!      *positive* signal for one that reserves it (MsX64).
//!
//! ## What this pass does not do
//!
//! - **Recover types.** B2.6's type lattice consumes the
//!   [`InferredSignature::int_args`] vector but the convention pass
//!   itself never assigns types.
//! - **Recover variadic-arg counts.** SysV's "rax = vector-arg
//!   count" convention is folded in alongside libc signature import
//!   later in M3.
//! - **Distinguish syscall conventions.** Inferring `syscall`-style
//!   arg layouts (`rdi, rsi, rdx, r10, r8, r9` on Linux) is a
//!   follow-up batch when the lifter starts surfacing `syscall` as a
//!   distinct call kind.
//! - **Modify the IR.** The score is reported, not applied. Callers
//!   that want to lock in a convention call [`pick_best`] and feed
//!   the result into their own signature-recovery pass.
//!
//! ## Determinism (NFR-9)
//!
//! Iteration walks SSA blocks, instructions, and the `StackFrame`'s
//! `BTreeMap` in ascending key order. Candidate scoring is pure
//! arithmetic over those iterations. Ties between candidates are
//! broken by their position in the input slice, which the caller
//! controls — [`infer_calling_convention`] does not silently reorder.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use dac_core::{Confidence, Source};
use dac_ir::ssa::{Operand, SsaFunction, SsaTerminator, ValueId, ValueSource, VariableId};
use dac_knowledge::CallingConvention;

use crate::stack::StackFrame;

/// Ranked candidate produced by [`infer_calling_convention`].
///
/// `Eq` is intentionally not derived: [`Confidence`] holds an `f32`
/// and only implements [`PartialEq`].
#[derive(Debug, Clone, PartialEq)]
pub struct ConventionMatch {
    /// Stable name of the matched convention
    /// (`CallingConvention::name`).
    pub convention_name: &'static str,
    /// Per-convention reading of the function's signature.
    pub signature: InferredSignature,
    /// How well this convention explains the observed evidence.
    /// Always [`Source::Derived`] at this layer.
    pub confidence: Confidence,
}

/// Inferred signature relative to one candidate convention.
///
/// All fields are *observations* under the candidate's assumptions —
/// a high-scoring match means the function's behavior is consistent
/// with the convention, not that the signature is correct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferredSignature {
    /// Integer arguments in convention order. Indexed by the
    /// convention's argument-register position; a gap (e.g. arg #2
    /// observed without arg #1) is preserved as a missing slot —
    /// see [`RegisterArg::index`].
    pub int_args: Vec<RegisterArg>,
    /// Stack-passed arguments in ascending offset order.
    pub stack_args: Vec<StackArg>,
    /// Integer return-register name when the function's `Return`
    /// terminator carries a value whose underlying variable matches
    /// the convention's integer return register; `None` otherwise.
    pub return_register: Option<&'static str>,
}

/// One register-passed integer argument as identified under a
/// candidate convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterArg {
    /// Lowercase ASCII register name, matching the convention's
    /// `int_arg_registers` entry.
    pub register: &'static str,
    /// Position in the convention's `int_arg_registers` slice — 0
    /// for the first arg register.
    pub index: usize,
    /// Parameter [`ValueId`] for this register's entry value.
    pub value: ValueId,
    /// Underlying [`VariableId`] for the register.
    pub variable: VariableId,
}

/// One stack-passed argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackArg {
    /// Signed offset from the function's entry stack pointer.
    /// Always `>= convention.first_stack_arg_offset`.
    pub offset: i64,
    /// Widest observed access width at this offset, in bytes.
    pub width: u8,
}

/// Score every candidate against the function and rank them.
///
/// Returns one [`ConventionMatch`] per candidate, sorted descending
/// by [`Confidence::value`]. Ties between candidates are broken by
/// their position in `candidates` — the earlier entry wins so the
/// caller controls precedence (e.g. by ordering `X86_64_CONVENTIONS`
/// to prefer SysV when evidence is ambiguous).
#[must_use]
pub fn infer_calling_convention(
    ssa: &SsaFunction,
    stack_frame: &StackFrame,
    candidates: &[&'static CallingConvention],
) -> Vec<ConventionMatch> {
    let parameters = collect_parameters(ssa);
    let return_var = return_value_variable(ssa);

    let mut matches: Vec<(usize, ConventionMatch)> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            (
                i,
                score_candidate(ssa, stack_frame, &parameters, return_var, c),
            )
        })
        .collect();

    matches.sort_by(|(ia, ma), (ib, mb)| {
        match mb
            .confidence
            .value()
            .partial_cmp(&ma.confidence.value())
            .unwrap_or(Ordering::Equal)
        {
            Ordering::Equal => ia.cmp(ib),
            other => other,
        }
    });

    matches.into_iter().map(|(_, m)| m).collect()
}

/// Convenience wrapper: rank candidates and return the top match.
///
/// Returns `None` when `candidates` is empty.
#[must_use]
pub fn pick_best(
    ssa: &SsaFunction,
    stack_frame: &StackFrame,
    candidates: &[&'static CallingConvention],
) -> Option<ConventionMatch> {
    infer_calling_convention(ssa, stack_frame, candidates)
        .into_iter()
        .next()
}

/// All parameter values in the function, paired with their
/// register name (lowercased via the SSA variable table).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParameterEntry {
    register: String,
    variable: VariableId,
    value: ValueId,
}

fn collect_parameters(ssa: &SsaFunction) -> Vec<ParameterEntry> {
    let mut out: Vec<ParameterEntry> = ssa
        .values
        .iter()
        .filter_map(|v| match v.source {
            ValueSource::Parameter { variable } => {
                let var = ssa.variable(variable);
                Some(ParameterEntry {
                    register: var.name.to_ascii_lowercase(),
                    variable,
                    value: v.id,
                })
            }
            _ => None,
        })
        .collect();
    out.sort_by_key(|a| a.variable);
    out
}

/// Variable id underlying the *value* returned by the function's
/// `Return` terminator, when there is exactly one and the operand is
/// a defined SSA value. Returns `None` for void returns, returns of
/// constants, or conflicting return-variables across blocks.
fn return_value_variable(ssa: &SsaFunction) -> Option<VariableId> {
    let mut seen: Option<VariableId> = None;
    for block in &ssa.blocks {
        if let SsaTerminator::Return {
            value: Some(Operand::Value(v)),
        } = &block.terminator
        {
            let var = ssa.value(*v).variable;
            match seen {
                None => seen = Some(var),
                Some(prev) if prev == var => {}
                Some(_) => return None,
            }
        }
    }
    seen
}

fn score_candidate(
    ssa: &SsaFunction,
    stack_frame: &StackFrame,
    parameters: &[ParameterEntry],
    return_var: Option<VariableId>,
    convention: &'static CallingConvention,
) -> ConventionMatch {
    // ---- argument-register prefix ----
    let arg_param_index: Vec<(usize, &ParameterEntry)> = parameters
        .iter()
        .filter_map(|p| convention.int_arg_index(&p.register).map(|i| (i, p)))
        .collect();

    let observed_indices: BTreeSet<usize> = arg_param_index.iter().map(|(i, _)| *i).collect();
    let mut prefix_len = 0usize;
    while observed_indices.contains(&prefix_len) {
        prefix_len += 1;
    }
    let total_observed = observed_indices.len();
    // Indices observed beyond the contiguous prefix from 0 are gaps.
    let gap_count = total_observed - prefix_len;

    let prefix_capacity = convention.int_arg_registers.len().max(1);
    let prefix_score = (prefix_len as f32) / (prefix_capacity as f32);
    let prefix_bonus = 0.30 * prefix_score.min(1.0);
    let gap_penalty = 0.10 * gap_count as f32;

    // Build the int_args list in convention order, restricted to the
    // contiguous prefix so a half-observed signature is not over-
    // claimed.
    let mut int_args: Vec<RegisterArg> = Vec::with_capacity(prefix_len);
    for (idx, &reg) in convention
        .int_arg_registers
        .iter()
        .enumerate()
        .take(prefix_len)
    {
        if let Some((_, p)) = arg_param_index.iter().find(|(i, _)| *i == idx) {
            int_args.push(RegisterArg {
                register: reg,
                index: idx,
                value: p.value,
                variable: p.variable,
            });
        }
    }

    // ---- caller-saved non-arg reads (contradictions) ----
    let caller_saved_read = parameters
        .iter()
        .filter(|p| {
            convention.is_caller_saved(&p.register) && !convention.is_int_arg_register(&p.register)
        })
        .count();
    let caller_saved_penalty = 0.15 * caller_saved_read as f32;

    // ---- return-register usage ----
    let return_register = return_var
        .map(|v| ssa.variable(v).name.to_ascii_lowercase())
        .filter(|name| convention.is_int_return_register(name))
        .map(|_| convention.int_return_register);
    let return_bonus = if return_register.is_some() { 0.15 } else { 0.0 };

    // ---- stack-arg / shadow-space layout ----
    let mut stack_args: Vec<StackArg> = Vec::new();
    let mut shadow_hits = 0u32;
    let mut shadow_misses = 0u32;
    let has_shadow = convention.shadow_space_bytes > 0;
    let shadow_end = convention.shadow_space_bytes.saturating_add(8) as i64; // 8 = return-address slot
    for (&offset, local) in &stack_frame.locals {
        if offset <= 0 {
            continue;
        }
        let aligned = offset % convention.stack_arg_alignment as i64 == 0;
        if offset >= convention.first_stack_arg_offset && aligned {
            stack_args.push(StackArg {
                offset,
                width: local.width,
            });
        } else if has_shadow && (8..shadow_end).contains(&offset) && aligned {
            shadow_hits += 1;
        } else if !has_shadow && (8..40).contains(&offset) && aligned {
            // Positive offset inside the MsX64 shadow window but
            // SysV reserves no shadow space; treat as a contradiction.
            shadow_misses += 1;
        }
    }
    let stack_bonus = if !stack_args.is_empty() { 0.05 } else { 0.0 };
    let shadow_bonus = if has_shadow && shadow_hits > 0 {
        0.10
    } else {
        0.0
    };
    let shadow_penalty = 0.10 * shadow_misses as f32;

    // ---- combine ----
    let raw = 0.40 + prefix_bonus + return_bonus + stack_bonus + shadow_bonus
        - gap_penalty
        - caller_saved_penalty
        - shadow_penalty;
    let value = raw.clamp(0.0, 1.0);

    ConventionMatch {
        convention_name: convention.name,
        signature: InferredSignature {
            int_args,
            stack_args,
            return_register,
        },
        confidence: Confidence::new(value, Source::Derived),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, EdgeKind, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{SsaFunction, Variable, VariableId};
    use dac_knowledge::{SYSV_AMD64, X86_64_CONVENTIONS};

    use crate::stack::{analyze_stack_frame, StackConvention};

    use super::*;

    // --- helpers ------------------------------------------------

    fn edge_kind_key(k: EdgeKind) -> u8 {
        match k {
            EdgeKind::Fall => 0,
            EdgeKind::Branch => 1,
            EdgeKind::Taken => 2,
            EdgeKind::NotTaken => 3,
        }
    }

    fn synthetic_cfg(n: usize, entry: u32, raw_edges: &[(u32, u32, EdgeKind)]) -> Cfg {
        let blocks: Vec<BasicBlock> = (0..n)
            .map(|i| BasicBlock {
                id: i as u32,
                address: 0x1000 + 0x10 * i as u64,
                end: 0x1000 + 0x10 * (i + 1) as u64,
                instructions: Vec::new(),
                terminator: Terminator::Fall,
            })
            .collect();
        let mut edges: Vec<Edge> = raw_edges
            .iter()
            .map(|&(from, to, kind)| Edge { from, to, kind })
            .collect();
        edges.sort_by_key(|e| (e.from, edge_kind_key(e.kind), e.to));

        let has_succ: BTreeSet<u32> = edges.iter().map(|e| e.from).collect();
        let exits: Vec<u32> = (0..n as u32).filter(|id| !has_succ.contains(id)).collect();

        let mut reachable: BTreeSet<u32> = BTreeSet::new();
        reachable.insert(entry);
        let mut queue: VecDeque<u32> = VecDeque::from([entry]);
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
        let _ = (BTreeMap::<u32, u32>::new(), unreachable.clone());

        Cfg {
            function_address: 0x1000,
            function_end: 0x1000 + 0x10 * n as u64,
            function_name: None,
            blocks,
            entry,
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

    fn mov(dst: VariableId, src: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Move {
                src: RawOperand::Variable(src),
            },
        }
    }

    fn add(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn sub(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Sub {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn add_vv(dst: VariableId, lhs: VariableId, rhs: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Variable(rhs),
            },
        }
    }

    fn store(addr: VariableId, value: VariableId, width: u8) -> RawOp {
        RawOp {
            dst: None,
            kind: RawOpKind::Store {
                address: RawOperand::Variable(addr),
                value: RawOperand::Variable(value),
                width,
            },
        }
    }

    fn load(dst: VariableId, addr: VariableId, width: u8) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Load {
                address: RawOperand::Variable(addr),
                width,
            },
        }
    }

    fn build(raw: RawFunction, n_blocks: usize, edges: &[(u32, u32, EdgeKind)]) -> SsaFunction {
        let cfg = synthetic_cfg(n_blocks, 0, edges);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    // --- inference --------------------------------------------

    /// `int f(int a, int b, int c)` SysV: reads rdi, rsi, rdx and
    /// returns rax. SysV should win cleanly over MsX64.
    #[test]
    fn sysv_three_int_args_outranks_msx64() {
        // 0 rsp, 1 rdi, 2 rsi, 3 rdx, 4 rax, 5 t (rdi+rsi), 6 t2 (t+rdx)
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "rdx"),
                var(4, "rax"),
                var(5, "t"),
                var(6, "t2"),
            ],
            blocks: vec![RawBlock {
                ops: vec![add_vv(5, 1, 2), add_vv(6, 5, 3), mov(4, 6)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].convention_name, "sysv-amd64");
        assert_eq!(ranked[1].convention_name, "ms-x64");
        assert!(
            ranked[0].confidence.value() > ranked[1].confidence.value(),
            "sysv conf {} should beat ms-x64 conf {}",
            ranked[0].confidence.value(),
            ranked[1].confidence.value(),
        );

        let sig = &ranked[0].signature;
        let regs: Vec<&str> = sig.int_args.iter().map(|a| a.register).collect();
        assert_eq!(regs, vec!["rdi", "rsi", "rdx"]);
        assert_eq!(sig.return_register, Some("rax"));
        assert!(sig.stack_args.is_empty());
    }

    /// MsX64 `int f(int a, int b)` reads rcx, rdx and returns rax.
    /// rdi and rsi (callee-saved on MsX64, arg-regs on SysV) are
    /// not read. MsX64 should win.
    #[test]
    fn msx64_two_int_args_outranks_sysv() {
        // 0 rsp, 1 rcx, 2 rdx, 3 rax, 4 t
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rcx"),
                var(2, "rdx"),
                var(3, "rax"),
                var(4, "t"),
            ],
            blocks: vec![RawBlock {
                ops: vec![add_vv(4, 1, 2), mov(3, 4)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);

        assert_eq!(ranked[0].convention_name, "ms-x64");
        assert_eq!(ranked[1].convention_name, "sysv-amd64");
        let sig = &ranked[0].signature;
        let regs: Vec<&str> = sig.int_args.iter().map(|a| a.register).collect();
        assert_eq!(regs, vec!["rcx", "rdx"]);
        assert_eq!(sig.return_register, Some("rax"));
    }

    /// Shadow-space writes at +8 and +16 (MsX64 home-saving prologue)
    /// boost MsX64 and contradict SysV.
    #[test]
    fn msx64_shadow_space_breaks_tie_in_favor_of_msx64() {
        // Simulate a MsX64 prologue: rcx and rdx are spilled to
        // [rsp + 8] / [rsp + 16]. Both regs are also args under both
        // conventions, so without shadow signals the inputs are
        // ambiguous between MsX64 and the "rcx, rdx are arg slots 3-4
        // on SysV with a gap" reading.
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rcx"),
                var(2, "rdx"),
                var(3, "home1"),
                var(4, "home2"),
            ],
            blocks: vec![RawBlock {
                ops: vec![add(3, 0, 8), store(3, 1, 8), add(4, 0, 16), store(4, 2, 8)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame_ms = analyze_stack_frame(&ssa, StackConvention::MsX64);
        let ranked = infer_calling_convention(&ssa, &frame_ms, X86_64_CONVENTIONS);
        assert_eq!(ranked[0].convention_name, "ms-x64");
        // The MsX64 confidence must exceed SysV by at least the
        // shadow_bonus + shadow_penalty differential, even when both
        // see rcx and rdx as arg reads.
        assert!(
            ranked[0].confidence.value() > ranked[1].confidence.value() + 0.10,
            "shadow signal must dominate (sysv={}, msx64={})",
            ranked[1].confidence.value(),
            ranked[0].confidence.value(),
        );
    }

    /// SysV reading rax at entry would be a hard contradiction (rax
    /// is caller-saved scratch / return register, not an argument).
    /// Reading rdi instead is normal. The presence of an unexpected
    /// caller-saved read should drag SysV's score down.
    #[test]
    fn caller_saved_non_arg_read_penalizes_convention() {
        // Read rax (caller-saved on SysV, NOT in arg list) as if it
        // were a parameter. This should crash SysV's score.
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, &[&SYSV_AMD64]);
        // The return read counts as a return-register match (+0.15)
        // but the parameter read of rax is *also* a caller-saved
        // non-arg read (-0.15). Net: confidence ~= base 0.40.
        let conf = ranked[0].confidence.value();
        assert!(
            (conf - 0.40).abs() < 1e-3,
            "expected confidence near base (got {conf})",
        );
    }

    /// No parameters read, no return value: only the base score.
    /// Both conventions should report identical confidence and the
    /// tie should break toward SysV (first in X86_64_CONVENTIONS).
    #[test]
    fn leaf_function_returns_tie_broken_by_input_order() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        assert_eq!(ranked[0].convention_name, "sysv-amd64");
        assert_eq!(ranked[1].convention_name, "ms-x64");
        assert_eq!(
            ranked[0].confidence.value(),
            ranked[1].confidence.value(),
            "leaf ties must compare bit-equal",
        );
    }

    /// A SysV function with arg 0 and arg 2 but missing arg 1 (e.g.
    /// the lifter saw `rdi` and `rdx` reads but not `rsi`). The
    /// signature should report a single-element prefix (just arg 0)
    /// and the gap_penalty should keep confidence modest.
    #[test]
    fn discontiguous_args_truncate_signature_to_prefix() {
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rdx"),
                var(3, "rax"),
                var(4, "t"),
            ],
            blocks: vec![RawBlock {
                ops: vec![add_vv(4, 1, 2), mov(3, 4)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, &[&SYSV_AMD64]);
        let sig = &ranked[0].signature;
        assert_eq!(sig.int_args.len(), 1, "only rdi belongs to the prefix");
        assert_eq!(sig.int_args[0].register, "rdi");
        assert_eq!(sig.int_args[0].index, 0);
        assert_eq!(sig.return_register, Some("rax"));
    }

    /// Stack-passed args show up at the convention's positive offsets
    /// and feed `signature.stack_args`. The pass picks them up from
    /// the StackFrame's locals map, classifying by offset against the
    /// candidate convention rather than trusting the StackFrame's own
    /// classification.
    #[test]
    fn sysv_seventh_arg_at_plus_8_lands_in_stack_args() {
        // Six int-arg regs (rdi..r9) are read into a running sum,
        // plus a stack arg accessed via `mov rax, [rsp + 8]`. SysV
        // places the 7th int arg at exactly that offset.
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "rdx"),
                var(4, "rcx"),
                var(5, "r8"),
                var(6, "r9"),
                var(7, "addr"),
                var(8, "rax"),
                var(9, "t1"),
                var(10, "t2"),
                var(11, "t3"),
                var(12, "t4"),
                var(13, "t5"),
                var(14, "v"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // accumulate rdi..r9 so every arg reg becomes a Parameter read
                    add_vv(9, 1, 2),
                    add_vv(10, 9, 3),
                    add_vv(11, 10, 4),
                    add_vv(12, 11, 5),
                    add_vv(13, 12, 6),
                    // stack-arg load: [rsp + 8]
                    add(7, 0, 8),
                    load(14, 7, 8),
                    add_vv(8, 13, 14),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(8)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, &[&SYSV_AMD64]);
        let sig = &ranked[0].signature;
        assert_eq!(sig.int_args.len(), 6);
        assert_eq!(sig.stack_args.len(), 1);
        assert_eq!(sig.stack_args[0].offset, 8);
        assert_eq!(sig.stack_args[0].width, 8);
    }

    /// MsX64 fifth arg lives at `[rsp + 40]`, past the 32-byte home
    /// space. SysV doesn't reserve home space, so the same offset
    /// also resolves to a stack arg under SysV — both conventions
    /// list it. Use the rcx/rdx/r8/r9 arg pattern to keep MsX64 on
    /// top.
    #[test]
    fn msx64_fifth_arg_at_plus_40_lands_in_stack_args() {
        // Read rcx, rdx, r8, r9 into a running sum so MsX64's int_arg
        // prefix scores 4/4. The fifth arg is loaded from [rsp + 40].
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rcx"),
                var(2, "rdx"),
                var(3, "r8"),
                var(4, "r9"),
                var(5, "addr"),
                var(6, "v5"),
                var(7, "rax"),
                var(8, "t1"),
                var(9, "t2"),
                var(10, "t3"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    add_vv(8, 1, 2),
                    add_vv(9, 8, 3),
                    add_vv(10, 9, 4),
                    add(5, 0, 40),
                    load(6, 5, 8),
                    add_vv(7, 10, 6),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(7)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::MsX64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        assert_eq!(ranked[0].convention_name, "ms-x64");
        let sig = &ranked[0].signature;
        assert_eq!(
            sig.stack_args.iter().map(|a| a.offset).collect::<Vec<_>>(),
            vec![40]
        );
    }

    /// Convention inference is deterministic across runs (NFR-9).
    #[test]
    fn inference_is_deterministic_across_runs() {
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "rdx"),
                var(4, "rcx"),
                var(5, "rax"),
                var(6, "t1"),
                var(7, "t2"),
                var(8, "t3"),
            ],
            blocks: vec![RawBlock {
                ops: vec![add_vv(6, 1, 2), add_vv(7, 6, 3), add_vv(8, 7, 4), mov(5, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(5)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let a = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        let b = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        assert_eq!(a, b);
    }

    /// `pick_best` returns the same head as the ranked list, and
    /// `None` when the candidate list is empty.
    #[test]
    fn pick_best_matches_head_of_ranking() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov(2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        let best = pick_best(&ssa, &frame, X86_64_CONVENTIONS).unwrap();
        assert_eq!(best, ranked[0]);
        assert!(pick_best(&ssa, &frame, &[]).is_none());
    }

    /// Every match produced carries `Source::Derived` confidence.
    #[test]
    fn every_match_source_is_derived() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rax")],
            blocks: vec![RawBlock {
                ops: vec![mov(2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        for m in &ranked {
            assert_eq!(m.confidence.source(), Source::Derived);
        }
    }

    /// A return of a constant (or a void return) does not nominate a
    /// return register: the function might still be writing rax but
    /// the analyzer cannot prove it, so `return_register` stays
    /// `None`.
    #[test]
    fn return_of_constant_does_not_nominate_return_register() {
        // Note: SsaTerminator carries an Operand which may be Const.
        // The check in `return_value_variable` skips non-Value
        // operands.
        let raw = RawFunction {
            variables: vec![var(0, "rsp")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Const(0)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        for m in &ranked {
            assert!(
                m.signature.return_register.is_none(),
                "constant return must not nominate a return reg",
            );
        }
    }

    /// Inference handles a function with sub rsp / locals plus
    /// register args and a return without confusing the stack-frame
    /// locals for stack args.
    #[test]
    fn locals_are_not_misclassified_as_stack_args() {
        // sub rsp, 32; [rsp - 32] = rdi; rax = [rsp - 32]; ret rax
        // Variables: 0 rsp, 1 rdi, 2 rax, 3 t
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rax"), var(3, "t")],
            blocks: vec![RawBlock {
                ops: vec![sub(0, 0, 32), store(0, 1, 8), load(3, 0, 8), mov(2, 3)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let ranked = infer_calling_convention(&ssa, &frame, &[&SYSV_AMD64]);
        let sig = &ranked[0].signature;
        assert!(sig.stack_args.is_empty(), "locals (offset<0) are not args");
        assert_eq!(sig.int_args.len(), 1);
        assert_eq!(sig.int_args[0].register, "rdi");
    }
}
