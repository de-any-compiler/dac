//! Type propagation (B2.6, FR-14, FR-16).
//!
//! Given an SSA function and three optional pieces of side information
//! — a convention-inferred signature ([`crate::convention::InferredSignature`]
//! from B2.5), the recovered stack frame ([`crate::stack::StackFrame`]
//! from B2.4), and a way to resolve call-target addresses to
//! [`dac_knowledge::ApiSignature`] entries — propagate a type from
//! [`dac_ir::ty`]'s lattice to every SSA value the analyzer can
//! constrain.
//!
//! ## Signals
//!
//! Four sources feed the lattice:
//!
//! 1. **Inferred-signature parameter values.** Each
//!    `RegisterArg.value` enters the worklist with an integer type of
//!    the function's pointer width and signedness `Unknown` (a
//!    pointer-vs-int discrimination is the propagation's downstream
//!    job, not a fact the convention inference itself recovers).
//! 2. **Load / Store widths.** A `Load { width: w }` constrains its
//!    destination to `Int(w * 8, Unknown)`; a `Store { value, width }`
//!    constrains `value` to `Int(w * 8, Unknown)`. The address operand
//!    of both is constrained to `Ptr(Unknown)`.
//! 3. **Call sites.** When `Call { target: Some(va) }` resolves to
//!    a known API signature, its `args[i]` operands are joined with
//!    the signature's parameter `i` type and its destination (when
//!    present) is joined with the return type.
//! 4. **Stack locals.** Each
//!    [`crate::stack::StackLocal`] contributes a width observation —
//!    the access width seen on its loads and stores — which the pass
//!    publishes as a `locals` entry in the result.
//!
//! Arithmetic ops ([`SsaOp::Add`], [`Sub`](SsaOp::Sub), …) propagate
//! the **width** of their operands to their destination but make no
//! claim about signedness or pointer-ness. `Move` is a pure passthrough.
//!
//! ## Confidence
//!
//! Every recovered type carries a [`Confidence`] of
//! [`Source::Derived`]. The numeric values land in
//! `[0.0, 1.0]` based on how directly each fact was observed:
//!
//! | Seed                       | Confidence value |
//! | -------------------------- | ---------------- |
//! | API signature call site    | 0.90             |
//! | Load / Store width         | 0.80             |
//! | Stack-local width          | 0.75             |
//! | Inferred-signature parameter | 0.70           |
//! | Arithmetic propagation     | 0.60             |
//! | Move passthrough           | 0.85             |
//!
//! Multiple seeds for one value combine through [`Confidence::join`],
//! which is the componentwise max — the strongest observation wins.
//!
//! ## What this pass does not do
//!
//! - **Recover composite types.** Structs and arrays are
//!   [`Type::Struct`] / [`Type::Array`] at the lattice layer but the
//!   pass never produces those variants — that is B3.2's job
//!   (clustering stack offsets / pointer chases).
//! - **Promote `Unknown` to a concrete pointer or integer when
//!   constraints disagree.** A value with no seed stays at
//!   [`Type::Unknown`].
//! - **Touch the IR.** The output is a side table; the IR is the
//!   source of truth (I-1).
//! - **Cross function boundaries.** Inter-procedural propagation
//!   (the call-graph type flow promised in B3.1) is out of scope.
//!
//! ## Determinism (NFR-9)
//!
//! Iteration walks blocks in ascending `SsaBlockId`, instructions in
//! their source-order index, phis after instructions, and the
//! [`Operand`] → [`ValueId`] map is a [`BTreeMap`]. The output
//! [`TypeMap::values`] / [`TypeMap::locals`] are also [`BTreeMap`]s.
//! Same inputs → same output.

use std::collections::BTreeMap;

use dac_core::{Confidence, Source};
use dac_ir::ssa::{Operand, SsaFunction, SsaOp, SsaTerminator, ValueId};
use dac_ir::ty::Type;
use dac_knowledge::ApiSignature;

use crate::convention::InferredSignature;
use crate::stack::StackFrame;

/// Side table of recovered types.
///
/// `Eq` is intentionally not derived: [`ValueType::confidence`]
/// holds a [`Confidence`] (f32-backed), which only implements
/// [`PartialEq`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeMap {
    /// Recovered type for every SSA value the analyzer could
    /// constrain. Values that received no signal are absent from the
    /// map; callers can treat absence as [`Type::Unknown`].
    pub values: BTreeMap<ValueId, ValueType>,
    /// Recovered type for every stack local the analyzer touched.
    pub locals: BTreeMap<i64, LocalType>,
}

impl TypeMap {
    /// Look up the recovered type for `value`, returning
    /// [`Type::Unknown`] when nothing was inferred.
    #[must_use]
    pub fn value_type(&self, value: ValueId) -> Type {
        self.values
            .get(&value)
            .map(|v| v.ty.clone())
            .unwrap_or(Type::Unknown)
    }

    /// Look up the recovered type for the stack local at `offset`,
    /// returning [`Type::Unknown`] when nothing was inferred.
    #[must_use]
    pub fn local_type(&self, offset: i64) -> Type {
        self.locals
            .get(&offset)
            .map(|l| l.ty.clone())
            .unwrap_or(Type::Unknown)
    }

    /// Fraction of `values` keys whose type is more specific than
    /// `Unknown` (i.e. concretely recovered). Used by the corpus
    /// rubric ("≥ 70% of locals in the corpus").
    #[must_use]
    pub fn value_recovery_ratio(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        let known = self.values.values().filter(|v| !v.ty.is_unknown()).count();
        known as f32 / self.values.len() as f32
    }

    /// Fraction of stack locals with a recovered type more specific
    /// than `Unknown`.
    #[must_use]
    pub fn local_recovery_ratio(&self) -> f32 {
        if self.locals.is_empty() {
            return 0.0;
        }
        let known = self.locals.values().filter(|l| !l.ty.is_unknown()).count();
        known as f32 / self.locals.len() as f32
    }
}

/// One entry in [`TypeMap::values`].
#[derive(Debug, Clone, PartialEq)]
pub struct ValueType {
    /// Recovered type.
    pub ty: Type,
    /// Confidence in the recovery — always [`Source::Derived`].
    pub confidence: Confidence,
}

/// One entry in [`TypeMap::locals`].
#[derive(Debug, Clone, PartialEq)]
pub struct LocalType {
    /// Recovered type.
    pub ty: Type,
    /// Confidence in the recovery — always [`Source::Derived`].
    pub confidence: Confidence,
}

/// Resolves a call-target virtual address to a [`ApiSignature`].
///
/// The propagation pass does not own the binary's import table; the
/// caller provides this resolver so the pass can stay
/// architecture- and binary-format-agnostic.
pub trait ApiResolver {
    /// Return the signature for a direct-call target VA, or `None`
    /// when the target is not a known imported API.
    fn resolve(&self, target_va: u64) -> Option<&'static ApiSignature>;
}

impl<F> ApiResolver for F
where
    F: Fn(u64) -> Option<&'static ApiSignature>,
{
    fn resolve(&self, target_va: u64) -> Option<&'static ApiSignature> {
        (self)(target_va)
    }
}

/// No-op resolver — every call goes unmodeled.
///
/// Useful for tests that exercise only the load/store/stack signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullApiResolver;

impl ApiResolver for NullApiResolver {
    fn resolve(&self, _target_va: u64) -> Option<&'static ApiSignature> {
        None
    }
}

/// Pointer width used to type parameter slots and pointer leaves on
/// the only architecture B2.6 supports (x86-64). When AArch64 / 32-bit
/// architectures arrive, this constant moves into an
/// `Architecture` query.
const POINTER_WIDTH_BITS: u16 = 64;

const CONF_API: f32 = 0.90;
const CONF_MOVE: f32 = 0.85;
const CONF_LOAD_STORE: f32 = 0.80;
const CONF_STACK_LOCAL: f32 = 0.75;
const CONF_PARAMETER: f32 = 0.70;
const CONF_ARITH: f32 = 0.60;

/// Run type propagation.
///
/// `inferred_signature` and `stack_frame` are both optional so the
/// pass can be run before either of its B2.x dependencies; the result
/// degrades gracefully (fewer seeds → less concrete types) rather
/// than refusing to produce a [`TypeMap`].
#[must_use]
pub fn propagate_types(
    ssa: &SsaFunction,
    inferred_signature: Option<&InferredSignature>,
    stack_frame: Option<&StackFrame>,
    api_resolver: &dyn ApiResolver,
) -> TypeMap {
    let mut values: BTreeMap<ValueId, ValueType> = BTreeMap::new();
    let mut locals: BTreeMap<i64, LocalType> = BTreeMap::new();

    // ---- seed: convention-inferred parameters ----
    if let Some(sig) = inferred_signature {
        for arg in &sig.int_args {
            seed_value(
                &mut values,
                arg.value,
                Type::int_of_width(POINTER_WIDTH_BITS),
                Confidence::new(CONF_PARAMETER, Source::Derived),
            );
        }
    }

    // ---- seed: stack-local widths (publish only) ----
    if let Some(frame) = stack_frame {
        for (&offset, local) in &frame.locals {
            let width_bits = u16::from(local.width).saturating_mul(8);
            if width_bits == 0 {
                continue;
            }
            let ty = Type::int_of_width(width_bits);
            seed_local(
                &mut locals,
                offset,
                ty,
                Confidence::new(CONF_STACK_LOCAL, Source::Derived),
            );
        }
    }

    // ---- seed: load / store widths and API call sites ----
    for block in &ssa.blocks {
        for ins in &block.instructions {
            match &ins.op {
                SsaOp::Load { address, width } => {
                    if let Some(dst) = ins.dst {
                        let bits = u16::from(*width).saturating_mul(8);
                        if bits != 0 {
                            seed_value(
                                &mut values,
                                dst,
                                Type::int_of_width(bits),
                                Confidence::new(CONF_LOAD_STORE, Source::Derived),
                            );
                        }
                    }
                    if let Operand::Value(v) = address {
                        seed_value(
                            &mut values,
                            *v,
                            Type::ptr_to(Type::Unknown),
                            Confidence::new(CONF_LOAD_STORE, Source::Derived),
                        );
                    }
                }
                SsaOp::Store {
                    address,
                    value,
                    width,
                } => {
                    let bits = u16::from(*width).saturating_mul(8);
                    if let Operand::Value(v) = value {
                        if bits != 0 {
                            seed_value(
                                &mut values,
                                *v,
                                Type::int_of_width(bits),
                                Confidence::new(CONF_LOAD_STORE, Source::Derived),
                            );
                        }
                    }
                    if let Operand::Value(v) = address {
                        seed_value(
                            &mut values,
                            *v,
                            Type::ptr_to(Type::Unknown),
                            Confidence::new(CONF_LOAD_STORE, Source::Derived),
                        );
                    }
                }
                SsaOp::Call { target, args } => {
                    let Some(target_va) = target else { continue };
                    let Some(sig) = api_resolver.resolve(*target_va) else {
                        continue;
                    };
                    if let Some(dst) = ins.dst {
                        if !sig.return_ty.is_unknown() {
                            seed_value(
                                &mut values,
                                dst,
                                sig.return_ty.clone(),
                                Confidence::new(CONF_API, Source::Derived),
                            );
                        }
                    }
                    for (i, arg) in args.iter().enumerate() {
                        let Some(param) = sig.parameters.get(i) else {
                            break;
                        };
                        if let Operand::Value(v) = arg {
                            seed_value(
                                &mut values,
                                *v,
                                param.ty.clone(),
                                Confidence::new(CONF_API, Source::Derived),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ---- seed: return register match from convention inference ----
    // The convention pass tells us *which* register the function
    // returns through; here we mark the value(s) actually returned
    // with that integer width so callers querying the function's
    // return get a concrete answer.
    if let Some(sig) = inferred_signature {
        if sig.return_register.is_some() {
            for block in &ssa.blocks {
                if let SsaTerminator::Return {
                    value: Some(Operand::Value(v)),
                } = &block.terminator
                {
                    seed_value(
                        &mut values,
                        *v,
                        Type::int_of_width(POINTER_WIDTH_BITS),
                        Confidence::new(CONF_PARAMETER, Source::Derived),
                    );
                }
            }
        }
    }

    // ---- fixed-point propagation through Move / arithmetic / phi ----
    let mut changed = true;
    while changed {
        changed = false;
        for block in &ssa.blocks {
            for ins in &block.instructions {
                let Some(dst) = ins.dst else { continue };
                let propagated = propagate_op(&ins.op, &values);
                if let Some((ty, conf)) = propagated {
                    if seed_value(&mut values, dst, ty, conf) {
                        changed = true;
                    }
                }
            }
            for phi in &block.phis {
                let mut merged = Type::Unknown;
                let mut conf = Confidence::new(0.0, Source::Derived);
                for &(_, opnd) in &phi.incoming {
                    if let Some((t, c)) = type_of_operand(opnd, &values) {
                        merged = merged.join(&t);
                        conf = conf.join(c);
                    }
                }
                if !merged.is_unknown() && seed_value(&mut values, phi.dst, merged, conf) {
                    changed = true;
                }
            }
        }
    }

    TypeMap { values, locals }
}

/// Compute the propagated type for `op`'s destination, when one can
/// be derived from the current value table. Returns the propagated
/// `(type, confidence)` pair.
fn propagate_op(op: &SsaOp, values: &BTreeMap<ValueId, ValueType>) -> Option<(Type, Confidence)> {
    match op {
        SsaOp::Move { src } => {
            let (t, c) = type_of_operand(*src, values)?;
            Some((t, with_value_at_most(c, CONF_MOVE)))
        }
        SsaOp::Add { lhs, rhs }
        | SsaOp::Sub { lhs, rhs }
        | SsaOp::Mul { lhs, rhs }
        | SsaOp::And { lhs, rhs }
        | SsaOp::Or { lhs, rhs }
        | SsaOp::Xor { lhs, rhs }
        | SsaOp::Shl { lhs, rhs }
        | SsaOp::Shr { lhs, rhs } => {
            let l = type_of_operand(*lhs, values);
            let r = type_of_operand(*rhs, values);
            arith_join(l, r)
        }
        SsaOp::Neg { src } | SsaOp::Not { src } => {
            let (t, c) = type_of_operand(*src, values)?;
            Some((t, with_value_at_most(c, CONF_ARITH)))
        }
        SsaOp::Compare { .. } => Some((
            Type::int_of_width(1),
            Confidence::new(CONF_ARITH, Source::Derived),
        )),
        // Load / Store / Call are seeded earlier; Opaque is the
        // intentional pressure-release valve and contributes no
        // constraint.
        SsaOp::Load { .. } | SsaOp::Store { .. } | SsaOp::Call { .. } | SsaOp::Opaque { .. } => {
            None
        }
    }
}

/// Lattice-join the operand types for a binary arithmetic op. Returns
/// the joined type and a confidence reduced to the arithmetic-tier
/// cap.
fn arith_join(
    lhs: Option<(Type, Confidence)>,
    rhs: Option<(Type, Confidence)>,
) -> Option<(Type, Confidence)> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some((t, c)), None) | (None, Some((t, c))) => Some((t, with_value_at_most(c, CONF_ARITH))),
        (Some((tl, cl)), Some((tr, cr))) => {
            let joined = tl.join(&tr);
            // Conflict at the integer-width level — the dst is
            // unconstrained at this layer. Don't publish anything.
            if joined.is_top() {
                return None;
            }
            let conf = with_value_at_most(cl.join(cr), CONF_ARITH);
            Some((joined, conf))
        }
    }
}

/// Type of an operand at the current point in the fixed-point
/// iteration. Constants contribute no info (B2.6 does not classify
/// constants — that lives in a later range-inference batch); `Undef`
/// also contributes none.
fn type_of_operand(
    opnd: Operand,
    values: &BTreeMap<ValueId, ValueType>,
) -> Option<(Type, Confidence)> {
    match opnd {
        Operand::Value(v) => values.get(&v).map(|vt| (vt.ty.clone(), vt.confidence)),
        Operand::Const(_) | Operand::Undef => None,
    }
}

/// Merge a new fact into `values` at `id`. Returns true when the
/// stored entry changed.
fn seed_value(
    values: &mut BTreeMap<ValueId, ValueType>,
    id: ValueId,
    ty: Type,
    confidence: Confidence,
) -> bool {
    match values.get(&id) {
        Some(existing) => {
            let merged = existing.ty.join(&ty);
            let new_conf = existing.confidence.join(confidence);
            if merged == existing.ty && new_conf == existing.confidence {
                false
            } else {
                values.insert(
                    id,
                    ValueType {
                        ty: merged,
                        confidence: new_conf,
                    },
                );
                true
            }
        }
        None => {
            values.insert(id, ValueType { ty, confidence });
            true
        }
    }
}

fn seed_local(
    locals: &mut BTreeMap<i64, LocalType>,
    offset: i64,
    ty: Type,
    confidence: Confidence,
) {
    match locals.get(&offset) {
        Some(existing) => {
            let merged = existing.ty.join(&ty);
            let new_conf = existing.confidence.join(confidence);
            locals.insert(
                offset,
                LocalType {
                    ty: merged,
                    confidence: new_conf,
                },
            );
        }
        None => {
            locals.insert(offset, LocalType { ty, confidence });
        }
    }
}

/// Cap `conf.value` at `max` while preserving its source class. Used
/// to prevent propagation tiers from inheriting an unrealistically
/// high confidence from their seeds (e.g. an arithmetic op on a
/// 0.90-API-derived value should publish at most `CONF_ARITH`).
fn with_value_at_most(conf: Confidence, max: f32) -> Confidence {
    Confidence::new(conf.value().min(max), conf.source())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, EdgeKind, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::ssa::{
        construct_ssa, RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator,
    };
    use dac_core::{EvidenceGraph, EvidenceNode, IrLayer};
    use dac_ir::ssa::{Variable, VariableId};
    use dac_ir::ty::{IntType, Signedness};
    use dac_knowledge::{lookup_api_signature, X86_64_CONVENTIONS};

    use super::*;
    use crate::convention::infer_calling_convention;
    use crate::stack::{analyze_stack_frame, StackConvention};

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

    fn add_vv(dst: VariableId, lhs: VariableId, rhs: VariableId) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Variable(rhs),
            },
        }
    }

    fn add_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn sub_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Sub {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
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

    fn load(dst: VariableId, addr: VariableId, width: u8) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Load {
                address: RawOperand::Variable(addr),
                width,
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

    fn call(dst: Option<VariableId>, target_va: u64, args: Vec<VariableId>) -> RawOp {
        RawOp {
            dst,
            kind: RawOpKind::Call {
                target: Some(target_va),
                args: args.into_iter().map(RawOperand::Variable).collect(),
            },
        }
    }

    fn build(raw: RawFunction, n_blocks: usize, edges: &[(u32, u32, EdgeKind)]) -> SsaFunction {
        let cfg = synthetic_cfg(n_blocks, 0, edges);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    /// Look up the SSA value defined by the i'th instruction in the
    /// k'th block (after any phis are minted). Useful for poking at
    /// specific destinations.
    fn ins_value(ssa: &SsaFunction, block: usize, ins: usize) -> ValueId {
        ssa.blocks[block].instructions[ins]
            .dst
            .expect("instruction defines a value")
    }

    fn parameter_value_for(ssa: &SsaFunction, name: &str) -> ValueId {
        let var_id = ssa
            .variables
            .iter()
            .find(|v| v.name == name)
            .expect("variable present")
            .id;
        ssa.values
            .iter()
            .find_map(|v| match v.source {
                dac_ir::ssa::ValueSource::Parameter { variable } if variable == var_id => {
                    Some(v.id)
                }
                _ => None,
            })
            .expect("parameter value for variable")
    }

    // --- load / store seeding ----------------------------------

    #[test]
    fn load_seeds_destination_with_int_of_width() {
        // variables: 0 = rsp, 1 = v
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "v")],
            blocks: vec![RawBlock {
                ops: vec![load(1, 0, 4)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &NullApiResolver);
        let dst = ins_value(&ssa, 0, 0);
        assert_eq!(tm.value_type(dst), Type::int_of_width(32));
    }

    #[test]
    fn store_constrains_value_operand() {
        // variables: 0 = rsp, 1 = v (parameter), store [rsp] = v(8)
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![store(0, 1, 8)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &NullApiResolver);
        let rdi_param = parameter_value_for(&ssa, "rdi");
        assert_eq!(tm.value_type(rdi_param), Type::int_of_width(64));
    }

    #[test]
    fn load_marks_address_operand_as_pointer() {
        // addr = rsp + 8; v = [addr]
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "addr"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![add_vc(1, 0, 8), load(2, 1, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &NullApiResolver);
        // The `addr` ssa value (defined by the add) is the address
        // of the load.
        let addr_val = ins_value(&ssa, 0, 0);
        assert_eq!(tm.value_type(addr_val), Type::ptr_to(Type::Unknown));
    }

    // --- API signature seeding ---------------------------------

    /// A toy resolver that maps two addresses to libc strlen and
    /// memcpy.
    fn libc_resolver(va: u64) -> Option<&'static ApiSignature> {
        match va {
            0xA1 => lookup_api_signature("strlen"),
            0xA2 => lookup_api_signature("memcpy"),
            _ => None,
        }
    }

    #[test]
    fn api_call_seeds_argument_types_and_return_type() {
        // call strlen(rdi) -> rax  (lifter expresses as a Call op)
        // rdi parameter should be typed Ptr(uint8); return value
        // should be uint64.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0])],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);

        // The argument value: parameter of rdi.
        let rdi_param = parameter_value_for(&ssa, "rdi");
        assert_eq!(
            tm.value_type(rdi_param),
            Type::ptr_to(Type::unsigned_int(8))
        );

        // The return value: dst of the call instruction.
        let call_dst = ins_value(&ssa, 0, 0);
        assert_eq!(tm.value_type(call_dst), Type::unsigned_int(64));
    }

    #[test]
    fn api_arity_mismatch_still_types_known_arguments() {
        // call memcpy(rdi, rsi, rdx, rcx) — extra arg ignored.
        let raw = RawFunction {
            variables: vec![
                var(0, "rdi"),
                var(1, "rsi"),
                var(2, "rdx"),
                var(3, "rcx"),
                var(4, "rax"),
            ],
            blocks: vec![RawBlock {
                ops: vec![call(Some(4), 0xA2, vec![0, 1, 2, 3])],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);

        assert_eq!(
            tm.value_type(parameter_value_for(&ssa, "rdi")),
            Type::ptr_to(Type::Unknown),
        );
        assert_eq!(
            tm.value_type(parameter_value_for(&ssa, "rdx")),
            Type::unsigned_int(64),
        );
        // The trailing rcx had no parameter to bind against and so
        // stays Unknown.
        assert_eq!(
            tm.value_type(parameter_value_for(&ssa, "rcx")),
            Type::Unknown,
        );
    }

    // --- arithmetic + Move propagation -------------------------

    #[test]
    fn move_passes_through_seeded_type() {
        // call strlen(rdi) -> rax; mov v, rax; return v
        // The Move's destination inherits the API return type.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0]), mov(2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);
        let mov_dst = ins_value(&ssa, 0, 1);
        assert_eq!(tm.value_type(mov_dst), Type::unsigned_int(64));
    }

    #[test]
    fn arithmetic_join_propagates_width() {
        // call strlen(rdi) -> v; w = v + v (Add); should be u64.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "w")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0]), add_vv(2, 1, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);
        let add_dst = ins_value(&ssa, 0, 1);
        assert_eq!(add_dst, ssa.blocks[0].instructions[1].dst.unwrap());
        assert_eq!(
            tm.value_type(add_dst),
            Type::Int(IntType {
                width_bits: 64,
                signedness: Signedness::Unsigned,
            }),
        );
    }

    // --- phi merging --------------------------------------------

    #[test]
    fn phi_joins_incoming_types() {
        // b0: call strlen(rdi) -> v0; jump b2
        // b1: load r1 = [rsp + 0] width 4; jump b2  (only reachable
        //     via the entry through a branch we add for completeness)
        // b2: phi(v0, r1); return phi
        //
        // The phi merges Int(64, Unsigned) and Int(32, Unknown). Widths
        // differ — the join is Top, so the phi's type stays Unknown.

        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rsp"), var(2, "cond"), var(3, "tmp")],
            blocks: vec![
                // b0: branch
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Branch {
                        cond: RawOperand::Variable(2),
                        taken: 1,
                        not_taken: 2,
                    },
                },
                // b1: tmp = call strlen(rdi); jump b3
                RawBlock {
                    ops: vec![call(Some(3), 0xA1, vec![0])],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                // b2: tmp = load [rsp] width 8; jump b3
                RawBlock {
                    ops: vec![load(3, 1, 8)],
                    terminator: RawTerminator::Jump { target: 3 },
                },
                // b3: return tmp
                RawBlock {
                    ops: vec![],
                    terminator: RawTerminator::Return {
                        value: Some(RawOperand::Variable(3)),
                    },
                },
            ],
        };
        let ssa = build(
            raw,
            4,
            &[
                (0, 1, EdgeKind::Taken),
                (0, 2, EdgeKind::NotTaken),
                (1, 3, EdgeKind::Fall),
                (2, 3, EdgeKind::Fall),
            ],
        );
        let tm = propagate_types(&ssa, None, None, &libc_resolver);

        // Both `tmp` definitions resolve to width-64 unsigned (one
        // via the strlen return, one via the 8-byte load with
        // signedness Unknown). Same width, signedness join produces
        // Unsigned vs Unknown = Unsigned.
        let phi_block = &ssa.blocks[3];
        let phi = phi_block
            .phis
            .iter()
            .find(|p| p.variable == 3)
            .expect("phi for tmp");
        let joined = tm.value_type(phi.dst);
        assert!(matches!(joined, Type::Int(IntType { width_bits: 64, .. })));
    }

    // --- inferred-signature parameter seeding ------------------

    #[test]
    fn inferred_signature_seeds_parameter_values() {
        // SysV-shaped function: reads rdi, rsi, rdx.
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "rdx"),
                var(4, "rax"),
                var(5, "t"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // Force rdi/rsi/rdx to appear as parameter reads
                    // by adding them into rax.
                    add_vv(4, 1, 2),
                    add_vv(5, 4, 3),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(5)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let cm = infer_calling_convention(&ssa, &frame, X86_64_CONVENTIONS);
        let best = cm.into_iter().next().expect("at least one match");
        let tm = propagate_types(&ssa, Some(&best.signature), Some(&frame), &NullApiResolver);

        for arg in &best.signature.int_args {
            let ty = tm.value_type(arg.value);
            // Parameter seed publishes Int(64, Unknown); no other
            // signal applies here.
            assert_eq!(ty, Type::int_of_width(64));
        }
    }

    // --- stack-local widths ------------------------------------

    #[test]
    fn stack_locals_pick_up_widths_from_frame() {
        // Single-block SysV-style frame storing rdi at [rsp - 16].
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![sub_vc(0, 0, 16), store(0, 1, 8)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let tm = propagate_types(&ssa, None, Some(&frame), &NullApiResolver);
        let local = tm.local_type(-16);
        assert_eq!(local, Type::int_of_width(64));
    }

    // --- confidence ---------------------------------------------

    #[test]
    fn every_recovered_type_is_derived() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0]), mov(2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);
        for vt in tm.values.values() {
            assert_eq!(vt.confidence.source(), Source::Derived);
        }
        for lt in tm.locals.values() {
            assert_eq!(lt.confidence.source(), Source::Derived);
        }
    }

    #[test]
    fn api_seed_outranks_arithmetic_propagation_in_confidence() {
        // call strlen(rdi) -> v; w = v + v
        // v's confidence should be CONF_API; w's should be CONF_ARITH.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "w")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0]), add_vv(2, 1, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);
        let v = ins_value(&ssa, 0, 0);
        let w = ins_value(&ssa, 0, 1);
        let cv = tm.values.get(&v).unwrap().confidence.value();
        let cw = tm.values.get(&w).unwrap().confidence.value();
        assert!(
            cv >= cw,
            "API seed {cv} should outrank arithmetic propagation {cw}"
        );
    }

    // --- determinism --------------------------------------------

    #[test]
    fn propagation_is_deterministic_across_runs() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax"), var(2, "v")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0]), mov(2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm1 = propagate_types(&ssa, None, None, &libc_resolver);
        let tm2 = propagate_types(&ssa, None, None, &libc_resolver);
        assert_eq!(tm1, tm2);
    }

    // --- recovery-ratio helper ---------------------------------

    #[test]
    fn recovery_ratio_reflects_concrete_entries() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rax")],
            blocks: vec![RawBlock {
                ops: vec![call(Some(1), 0xA1, vec![0])],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &libc_resolver);
        // Both the rdi parameter and the call's dst are concretely
        // typed by the strlen signature.
        assert!(tm.value_recovery_ratio() > 0.0);
    }

    // --- nullary signature + Opaque ----------------------------

    #[test]
    fn opaque_operations_contribute_no_constraint() {
        // dst = Opaque("xchg", [rdi]) — modelled with raw mnemonic.
        // The pass should leave dst untyped.
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "v")],
            blocks: vec![RawBlock {
                ops: vec![RawOp {
                    dst: Some(1),
                    kind: RawOpKind::Opaque {
                        mnemonic: "xchg".to_string(),
                        args: vec![RawOperand::Variable(0)],
                    },
                }],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let tm = propagate_types(&ssa, None, None, &NullApiResolver);
        let dst = ins_value(&ssa, 0, 0);
        assert_eq!(tm.value_type(dst), Type::Unknown);
    }
}
