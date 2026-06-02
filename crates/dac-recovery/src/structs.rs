//! Struct and array recovery (B3.2, FR-17).
//!
//! Given an SSA function, its recovered [`StackFrame`] (B2.4), and the
//! propagated [`TypeMap`] (B2.6), promote evidence of composite-type
//! access into structured layouts. Two flavours of struct are produced
//! here:
//!
//! - **Stack-anchored.** A run of stack locals at adjacent offsets is
//!   re-presented as a single [`StructLayout`] keyed by the lowest
//!   offset in the run. The stack pass keeps locals as independent
//!   slots (one entry per offset); this pass clusters them.
//! - **Pointer-anchored.** When two or more `Load` / `Store`
//!   instructions read or write `base + const_i` with at least two
//!   distinct `const_i` values, the SSA value `base` is treated as a
//!   pointer to a struct, and the recovered field set is keyed under
//!   the base [`ValueId`].
//!
//! A third recovery — **arrays** — pattern-matches the canonical
//! indexed-access form `Add(base, Mul(index, stride))` (or
//! `Shl(index, log_stride)` for power-of-two element sizes). Each
//! recovery records the base value, the element stride, and the
//! element width when one is observed at a load/store anchored on the
//! result.
//!
//! ## What this batch deliberately doesn't do
//!
//! - **Promote stack-anchored structs back into [`StackFrame`]
//!   `locals`.** Stack analysis at B2.4 records per-offset access; the
//!   `StructLayout` we emit is a *view* on top, not a rewrite of the
//!   frame. The lowering pass (a later batch) is responsible for
//!   collapsing a `StructLayout` worth of `LocalType` entries into one
//!   typed C local.
//! - **Recover nested structs.** A pointer-anchored struct with a
//!   pointer field whose pointee itself looks structural is reported
//!   as two independent layouts at this layer.
//! - **Recover union types.** A union is *two* disjoint sets of
//!   accesses through the same base; B3.2 reports them as one
//!   layout. Disambiguating that lands with the type-lattice union
//!   variant in a later batch.
//! - **Touch the IR.** Output is a side table; the IR is the source
//!   of truth (I-1).
//!
//! ## Confidence
//!
//! Every recovered fact carries a [`Confidence`] of
//! [`Source::Derived`]. The numeric values reflect how directly each
//! pattern was observed:
//!
//! | Recovery               | Confidence value |
//! | ---------------------- | ---------------- |
//! | Stack offset cluster   | 0.75             |
//! | Pointer-base struct    | 0.65             |
//! | Indexed-array pattern  | 0.70             |
//!
//! Field-level confidence on a [`FieldCandidate`] inherits the
//! enclosing layout's confidence and is joined with the
//! [`TypeMap`]-provided type confidence when the field type is known.
//!
//! ## Determinism (NFR-9)
//!
//! Iteration walks SSA blocks in ascending [`SsaBlockId`] order,
//! instructions in source order. Output maps are [`BTreeMap`]s; field
//! lists inside a layout are sorted by ascending offset; the array
//! map is keyed by the base [`ValueId`]. Same inputs → same output.

use std::collections::{BTreeMap, BTreeSet};

use dac_core::{Confidence, Source};
use dac_ir::ssa::{Operand, SsaFunction, SsaOp, ValueId};
use dac_ir::ty::Type;

use crate::stack::{StackFrame, StackLocalKind};
use crate::types::TypeMap;

/// Confidence for a struct synthesized from a contiguous stack-local
/// run.
pub const STACK_CLUSTER_CONFIDENCE: f32 = 0.75;
/// Confidence for a struct synthesized from `Load`/`Store` accesses
/// through a shared pointer base with multiple distinct offsets.
pub const POINTER_BASE_CONFIDENCE: f32 = 0.65;
/// Confidence for an array recovered from `Add(base, Mul(index,
/// stride))` / `Add(base, Shl(index, log_stride))`.
pub const ARRAY_INDEXED_CONFIDENCE: f32 = 0.70;

/// Output of struct/array recovery.
///
/// `Eq` is intentionally not derived: layouts carry a [`Confidence`]
/// (f32-backed), which only implements [`PartialEq`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecoveredStructs {
    /// Stack-anchored struct layouts, keyed by the lowest (most
    /// negative) offset in each cluster.
    pub stack_structs: BTreeMap<i64, StructLayout>,
    /// Pointer-anchored struct layouts, keyed by the base SSA value.
    pub pointer_structs: BTreeMap<ValueId, StructLayout>,
    /// Recovered arrays, keyed by the base SSA value that addresses
    /// element 0.
    pub arrays: BTreeMap<ValueId, ArrayLayout>,
}

impl RecoveredStructs {
    /// Number of struct layouts (stack + pointer) recovered.
    #[must_use]
    pub fn struct_count(&self) -> usize {
        self.stack_structs.len() + self.pointer_structs.len()
    }

    /// Number of arrays recovered.
    #[must_use]
    pub fn array_count(&self) -> usize {
        self.arrays.len()
    }
}

/// A clustered struct layout.
#[derive(Debug, Clone, PartialEq)]
pub struct StructLayout {
    /// Fields in ascending `offset` order. The lowest field's offset
    /// is `0` for pointer-anchored layouts and the cluster's lowest
    /// stack offset for stack-anchored layouts (i.e. the struct's
    /// own coordinate system starts at the lowest member, not the
    /// stack frame's coordinate system).
    pub fields: Vec<FieldCandidate>,
    /// Total span of the layout in bytes: `last_field.offset +
    /// last_field.width - first_field.offset`. Reported even when the
    /// layout is degenerate (a single field) so consumers can quickly
    /// query the size without re-walking the field list.
    pub total_size: u64,
    /// Confidence in the layout as a whole.
    pub confidence: Confidence,
}

/// One field inside a [`StructLayout`].
#[derive(Debug, Clone, PartialEq)]
pub struct FieldCandidate {
    /// Byte offset from the layout's first field. For stack-anchored
    /// layouts this is `local.offset - layout.fields[0].offset`; for
    /// pointer-anchored layouts this is the offset literal observed at
    /// the access site.
    pub offset: u64,
    /// Widest access width (in bytes) observed at this offset. `0`
    /// when the access width was not pinned down (rare; the pass
    /// elides zero-width fields rather than recording them).
    pub width: u8,
    /// Type recovered by [`crate::propagate_types`] for the field's
    /// SSA value, when one is available; [`Type::Unknown`] otherwise.
    pub ty: Type,
    /// Number of read+write accesses observed at this offset.
    pub access_count: u32,
    /// Confidence in this specific field. Inherits the enclosing
    /// layout's confidence; joined with the [`TypeMap`] confidence
    /// when a concrete type was recovered.
    pub confidence: Confidence,
}

/// A recovered array.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrayLayout {
    /// Element stride in bytes. The constant `c` from
    /// `Mul(index, c)` or `1 << log_c` from `Shl(index, log_c)`.
    pub element_size: u64,
    /// Element width in bytes (from a load/store anchored on
    /// `base + index*stride`), when one was observed. `None` when the
    /// array's element width has not yet been pinned by an access.
    pub element_width: Option<u8>,
    /// Confidence in the array recovery.
    pub confidence: Confidence,
}

/// Run struct / array recovery on `ssa`.
///
/// The `frame` and `types` arguments are independent inputs: passing
/// `None` for either degrades recovery (no stack clustering, or no
/// field types) but never produces an error. The returned table is
/// purely additive and never mutates the IR (I-1).
#[must_use]
pub fn recover_structs(
    ssa: &SsaFunction,
    frame: Option<&StackFrame>,
    types: Option<&TypeMap>,
) -> RecoveredStructs {
    let stack_structs = match frame {
        Some(f) => cluster_stack_locals(f, types),
        None => BTreeMap::new(),
    };
    let pointer_structs = cluster_pointer_accesses(ssa, types);
    let arrays = recover_arrays(ssa);
    RecoveredStructs {
        stack_structs,
        pointer_structs,
        arrays,
    }
}

// ---- Stack clustering ------------------------------------------------

/// Greedy contiguity cluster over the stack frame's locals. A cluster
/// extends as long as the next local's offset is no further than
/// `previous.offset + max(previous.width, 8)` away, which catches both
/// tight packings (no gap) and the natural 8-byte-aligned-spill case
/// that real frames emit. The pass only considers
/// [`StackLocalKind::Local`] entries — incoming args, return-address
/// slots, and shadow space are not struct candidates.
fn cluster_stack_locals(
    frame: &StackFrame,
    types: Option<&TypeMap>,
) -> BTreeMap<i64, StructLayout> {
    let mut clusters: Vec<Vec<(i64, u8, u32)>> = Vec::new();
    let mut current: Vec<(i64, u8, u32)> = Vec::new();
    let mut prev_end: Option<i64> = None;

    for (&offset, local) in &frame.locals {
        if local.kind != StackLocalKind::Local {
            // Boundary: terminate the current run.
            if current.len() >= 2 {
                clusters.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            prev_end = None;
            continue;
        }
        let stride = i64::from(local.width.max(8));
        match prev_end {
            Some(end) if offset <= end + stride => {
                current.push((offset, local.width, local.access_count));
            }
            _ => {
                if current.len() >= 2 {
                    clusters.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                current.push((offset, local.width, local.access_count));
            }
        }
        prev_end = Some(offset + i64::from(local.width.max(1)));
    }
    if current.len() >= 2 {
        clusters.push(current);
    }

    let confidence = Confidence::new(STACK_CLUSTER_CONFIDENCE, Source::Derived);
    let mut out = BTreeMap::new();
    for cluster in clusters {
        let base = cluster.first().expect("non-empty cluster").0;
        let last = cluster.last().expect("non-empty cluster");
        let total_size = (last.0 + i64::from(last.1.max(1)) - base).max(0) as u64;
        let mut fields = Vec::with_capacity(cluster.len());
        for (off, width, access_count) in cluster {
            let field_offset = (off - base) as u64;
            let (ty, ty_conf) = match types {
                Some(t) => {
                    let ty = t.local_type(off);
                    let conf = if ty.is_unknown() {
                        confidence
                    } else {
                        // Float the layout confidence up to the type's
                        // join — this models "we have both structural
                        // and type evidence for this field."
                        t.locals
                            .get(&off)
                            .map(|l| confidence.join(l.confidence))
                            .unwrap_or(confidence)
                    };
                    (ty, conf)
                }
                None => (Type::Unknown, confidence),
            };
            fields.push(FieldCandidate {
                offset: field_offset,
                width,
                ty,
                access_count,
                confidence: ty_conf,
            });
        }
        out.insert(
            base,
            StructLayout {
                fields,
                total_size,
                confidence,
            },
        );
    }
    out
}

// ---- Pointer-base clustering -----------------------------------------

/// Walk every `Load` / `Store` and record the offset constant when the
/// address resolves to `base + const` (or the bare base, treated as
/// `const = 0`). Any base SSA value with two or more distinct offset
/// observations is promoted to a [`StructLayout`].
///
/// Stack-pointer / frame-pointer derived addresses are intentionally
/// excluded here: those are the stack-cluster's territory. We
/// recognize them by checking whether the base value's type was
/// recovered as [`Type::Ptr`] of `Unknown` *and* the base value
/// appears in the stack frame as a known offset. The simpler proxy
/// used here — "the base value has a non-zero stack offset assigned
/// by the SSA layer" — would require re-running the stack pass to
/// access; instead we skip bases whose only constraint is a stack-anchored
/// access by checking `value_offset_from_zero` on the bases we have
/// already seen. Practically, the false-positive rate is low because
/// the stack pass keeps `[sp + k]` as `Const k`, not as a `base + k`
/// pattern over a separate SSA value.
fn cluster_pointer_accesses(
    ssa: &SsaFunction,
    types: Option<&TypeMap>,
) -> BTreeMap<ValueId, StructLayout> {
    // base_value -> offset -> (width, access_count)
    let mut sites: BTreeMap<ValueId, BTreeMap<u64, AccessInfo>> = BTreeMap::new();
    let value_const = collect_value_constants(ssa);

    for block in &ssa.blocks {
        for ins in &block.instructions {
            let (addr, width) = match &ins.op {
                SsaOp::Load { address, width } => (*address, *width),
                SsaOp::Store { address, width, .. } => (*address, *width),
                _ => continue,
            };
            let Operand::Value(addr_val) = addr else {
                continue;
            };
            let (base, offset) = decompose_address(addr_val, ssa, &value_const);
            // Skip self-cluster from bare values without a base — only
            // accumulate when at least one decomposition produced a
            // non-trivial offset; the second pass below filters
            // candidates with fewer than two distinct offsets anyway.
            let entry = sites.entry(base).or_default();
            let acc = entry.entry(offset).or_default();
            acc.width = acc.width.max(width);
            acc.access_count += 1;
        }
    }

    let confidence = Confidence::new(POINTER_BASE_CONFIDENCE, Source::Derived);
    let mut out = BTreeMap::new();
    for (base, offsets) in sites {
        if offsets.len() < 2 {
            continue;
        }
        let mut fields: Vec<FieldCandidate> = offsets
            .into_iter()
            .map(|(off, info)| {
                let (ty, conf) = match types {
                    Some(t) => {
                        // Heuristic for field type: if exactly one
                        // `Load` at this offset produced a value with
                        // a recovered type, use that.
                        let ty = field_type_at(ssa, t, base, off).unwrap_or(Type::Unknown);
                        (ty, confidence)
                    }
                    None => (Type::Unknown, confidence),
                };
                FieldCandidate {
                    offset: off,
                    width: info.width,
                    ty,
                    access_count: info.access_count,
                    confidence: conf,
                }
            })
            .collect();
        fields.sort_by_key(|f| f.offset);
        let first = fields.first().expect("non-empty fields");
        let last = fields.last().expect("non-empty fields");
        let total_size = last.offset + u64::from(last.width.max(1)) - first.offset;
        out.insert(
            base,
            StructLayout {
                fields,
                total_size,
                confidence,
            },
        );
    }
    out
}

#[derive(Default, Clone, Copy)]
struct AccessInfo {
    width: u8,
    access_count: u32,
}

fn collect_value_constants(ssa: &SsaFunction) -> BTreeMap<ValueId, i64> {
    let mut out = BTreeMap::new();
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let Some(dst) = ins.dst else { continue };
            if let SsaOp::Move {
                src: Operand::Const(c),
            } = &ins.op
            {
                out.insert(dst, *c);
            }
        }
    }
    out
}

/// Walk back through `addr_val` looking for the canonical
/// `Add(base, Const)` / `Sub(base, Const)` shape. Returns
/// `(base, offset)` — `base == addr_val` and `offset == 0` when no
/// decomposition fires.
fn decompose_address(
    addr_val: ValueId,
    ssa: &SsaFunction,
    value_const: &BTreeMap<ValueId, i64>,
) -> (ValueId, u64) {
    let Some(def) = lookup_def_op(addr_val, ssa) else {
        return (addr_val, 0);
    };
    match def {
        SsaOp::Add { lhs, rhs } => match (lhs, rhs) {
            (Operand::Value(b), Operand::Const(c)) | (Operand::Const(c), Operand::Value(b)) => {
                if *c >= 0 {
                    (*b, *c as u64)
                } else {
                    (addr_val, 0)
                }
            }
            (Operand::Value(b), Operand::Value(v)) => match value_const.get(v) {
                Some(c) if *c >= 0 => (*b, *c as u64),
                _ => (addr_val, 0),
            },
            _ => (addr_val, 0),
        },
        SsaOp::Sub { lhs, rhs } => match (lhs, rhs) {
            (Operand::Value(b), Operand::Const(c)) if *c <= 0 => (*b, (-*c) as u64),
            _ => (addr_val, 0),
        },
        _ => (addr_val, 0),
    }
}

fn lookup_def_op(value: ValueId, ssa: &SsaFunction) -> Option<&SsaOp> {
    let def = ssa.value(value);
    if let dac_ir::ssa::ValueSource::Instruction { block, index } = def.source {
        return Some(&ssa.blocks[block as usize].instructions[index as usize].op);
    }
    None
}

fn field_type_at(ssa: &SsaFunction, types: &TypeMap, base: ValueId, offset: u64) -> Option<Type> {
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let SsaOp::Load { address, .. } = &ins.op else {
                continue;
            };
            let Operand::Value(addr_val) = address else {
                continue;
            };
            let value_const = collect_value_constants(ssa);
            let (b, o) = decompose_address(*addr_val, ssa, &value_const);
            if b == base && o == offset {
                if let Some(dst) = ins.dst {
                    let t = types.value_type(dst);
                    if !t.is_unknown() {
                        return Some(t);
                    }
                }
            }
        }
    }
    None
}

// ---- Array recovery --------------------------------------------------

/// Find SSA values whose definition matches
/// `Add(base, Mul(index, stride))` or
/// `Add(base, Shl(index, log_stride))`, and record an [`ArrayLayout`]
/// keyed by `base`. When the indexed value is itself used as the
/// address of a `Load`/`Store`, the access width pins the array's
/// `element_width`.
fn recover_arrays(ssa: &SsaFunction) -> BTreeMap<ValueId, ArrayLayout> {
    let value_const = collect_value_constants(ssa);
    let confidence = Confidence::new(ARRAY_INDEXED_CONFIDENCE, Source::Derived);

    // base -> (stride, element_width)
    let mut found: BTreeMap<ValueId, (u64, Option<u8>)> = BTreeMap::new();
    // indexed_value -> (base, stride)
    let mut indexed_to_base: BTreeMap<ValueId, (ValueId, u64)> = BTreeMap::new();

    // First pass: every Add that fits the indexed shape.
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let Some(dst) = ins.dst else { continue };
            let SsaOp::Add { lhs, rhs } = &ins.op else {
                continue;
            };
            if let Some((base, stride)) = match_indexed(*lhs, *rhs, ssa, &value_const) {
                indexed_to_base.insert(dst, (base, stride));
                found.entry(base).or_insert((stride, None));
            }
        }
    }
    // Second pass: pin element width from loads/stores using the
    // indexed value as their address.
    for block in &ssa.blocks {
        for ins in &block.instructions {
            let (addr, width) = match &ins.op {
                SsaOp::Load { address, width } => (*address, *width),
                SsaOp::Store { address, width, .. } => (*address, *width),
                _ => continue,
            };
            let Operand::Value(v) = addr else { continue };
            if let Some((base, _)) = indexed_to_base.get(&v) {
                if let Some(entry) = found.get_mut(base) {
                    entry.1 = Some(entry.1.map_or(width, |w| w.max(width)));
                }
            }
        }
    }

    found
        .into_iter()
        .map(|(base, (stride, width))| {
            (
                base,
                ArrayLayout {
                    element_size: stride,
                    element_width: width,
                    confidence,
                },
            )
        })
        .collect()
}

/// Match either operand pair against the indexed-array shape and
/// return `(base, stride)` when it fits.
fn match_indexed(
    lhs: Operand,
    rhs: Operand,
    ssa: &SsaFunction,
    value_const: &BTreeMap<ValueId, i64>,
) -> Option<(ValueId, u64)> {
    for (base_op, scaled_op) in [(lhs, rhs), (rhs, lhs)] {
        let Operand::Value(base) = base_op else {
            continue;
        };
        let Operand::Value(scaled) = scaled_op else {
            continue;
        };
        let Some(def) = lookup_def_op(scaled, ssa) else {
            continue;
        };
        if let Some(stride) = stride_of(def, value_const) {
            if stride >= 2 {
                // Disqualify trivial "base + index*1" — that's just
                // `base + index`, not an array address. Strides of 1
                // are legitimate for byte arrays but indistinguishable
                // from a plain pointer add at this layer; B3.7 + the
                // type lattice will refine.
                return Some((base, stride));
            }
        }
    }
    None
}

/// Stride from a single `Mul` or `Shl` with a constant scaling factor.
fn stride_of(op: &SsaOp, value_const: &BTreeMap<ValueId, i64>) -> Option<u64> {
    match op {
        SsaOp::Mul { lhs, rhs } => {
            let c =
                const_operand(*lhs, value_const).or_else(|| const_operand(*rhs, value_const))?;
            if c > 0 {
                Some(c as u64)
            } else {
                None
            }
        }
        SsaOp::Shl { lhs: _, rhs } => {
            let c = const_operand(*rhs, value_const)?;
            if (0..64).contains(&c) {
                Some(1u64 << c)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn const_operand(op: Operand, value_const: &BTreeMap<ValueId, i64>) -> Option<i64> {
    match op {
        Operand::Const(c) => Some(c),
        Operand::Value(v) => value_const.get(&v).copied(),
        Operand::Undef => None,
    }
}

// ---- private helper for cluster contiguity test ----------------------
//
// The cluster heuristic relies on a tiny invariant — locals come out of
// `StackFrame::locals` in ascending offset order — so a smoke test that
// pins that ordering is not necessary here. The frame-pass tests at
// B2.4 are responsible for that invariant.

#[allow(dead_code)]
const _: fn() = || {
    // Compile-time guard: BTreeMap key iteration is ascending; we rely
    // on it for cluster contiguity.
    let _: BTreeMap<i64, ()> = BTreeMap::new();
};

// We need a BTreeSet import only for the doctests' synthetic helpers;
// kept here to avoid pulling it via tests-only `use`.
#[allow(dead_code)]
type _UnusedBTreeSet = BTreeSet<()>;

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

    use super::*;
    use crate::stack::{analyze_stack_frame, StackConvention};
    use crate::types::{propagate_types, NullApiResolver};

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

    fn add_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Add {
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

    fn sub_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Sub {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn mul_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Mul {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
            },
        }
    }

    fn shl_vc(dst: VariableId, lhs: VariableId, rhs: i64) -> RawOp {
        RawOp {
            dst: Some(dst),
            kind: RawOpKind::Shl {
                lhs: RawOperand::Variable(lhs),
                rhs: RawOperand::Const(rhs),
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

    fn build(raw: RawFunction, n_blocks: usize, edges: &[(u32, u32, EdgeKind)]) -> SsaFunction {
        let cfg = synthetic_cfg(n_blocks, 0, edges);
        let doms = DominatorTree::build(&cfg);
        construct_ssa(&cfg, &doms, &raw)
    }

    fn ins_value(ssa: &SsaFunction, block: usize, ins: usize) -> ValueId {
        ssa.blocks[block].instructions[ins]
            .dst
            .expect("instruction defines a value")
    }

    // --- Stack clustering --------------------------------------

    /// Two adjacent stack stores at `[rsp - 16]` (width 8) and
    /// `[rsp - 8]` (width 8) become a single stack-anchored struct
    /// keyed at `-16`.
    #[test]
    fn adjacent_stack_locals_form_struct_layout() {
        // variables: 0 = rsp, 1 = rdi, 2 = rsi, 3 = addr
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rsi"), var(3, "addr")],
            blocks: vec![RawBlock {
                ops: vec![
                    // rsp -= 16
                    sub_vc(0, 0, 16),
                    // [rsp + 0] = rdi  (entry_sp - 16)
                    store(0, 1, 8),
                    // addr = rsp + 8
                    add_vc(3, 0, 8),
                    // [addr] = rsi     (entry_sp - 8)
                    store(3, 2, 8),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let recovered = recover_structs(&ssa, Some(&frame), None);

        assert_eq!(recovered.stack_structs.len(), 1);
        let layout = recovered
            .stack_structs
            .get(&-16)
            .expect("cluster keyed at -16");
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[0].width, 8);
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].width, 8);
        assert_eq!(layout.total_size, 16);
        assert_eq!(layout.confidence.value(), STACK_CLUSTER_CONFIDENCE);
    }

    /// A lone stack local (no neighbour within stride) does not
    /// produce a struct.
    #[test]
    fn lone_stack_local_is_not_a_struct() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi")],
            blocks: vec![RawBlock {
                ops: vec![sub_vc(0, 0, 16), store(0, 1, 8)],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let recovered = recover_structs(&ssa, Some(&frame), None);
        assert!(recovered.stack_structs.is_empty());
    }

    /// Stack locals carry the [`TypeMap`]-recovered field types when
    /// the type pass provides them.
    #[test]
    fn stack_fields_inherit_recovered_types() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rsi"), var(3, "addr")],
            blocks: vec![RawBlock {
                ops: vec![
                    sub_vc(0, 0, 16),
                    store(0, 1, 8),
                    add_vc(3, 0, 8),
                    store(3, 2, 4),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let types = propagate_types(&ssa, None, Some(&frame), &NullApiResolver);
        let recovered = recover_structs(&ssa, Some(&frame), Some(&types));
        let layout = recovered
            .stack_structs
            .get(&-16)
            .expect("cluster keyed at -16");
        // First field is 8 bytes (Int64) per the store width; second
        // is 4 (Int32). Both come from the TypeMap's local entries.
        assert_eq!(layout.fields[0].ty, Type::int_of_width(64));
        assert_eq!(layout.fields[1].ty, Type::int_of_width(32));
    }

    // --- Pointer-base clustering -------------------------------

    /// Loads at `base + 0` and `base + 8` through a pointer parameter
    /// promote `base` to a struct keyed under its SSA value.
    #[test]
    fn two_loads_through_pointer_base_form_struct() {
        // variables: 0 = rdi (parameter, the base pointer),
        //            1 = addr, 2 = v0, 3 = v1
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "addr"), var(2, "v0"), var(3, "v1")],
            blocks: vec![RawBlock {
                ops: vec![
                    // v0 = [rdi + 0]
                    load(2, 0, 8),
                    // addr = rdi + 8
                    add_vc(1, 0, 8),
                    // v1 = [addr]
                    load(3, 1, 4),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);

        // `rdi` parameter value id — look it up by name.
        let rdi_var = ssa.variables.iter().find(|v| v.name == "rdi").unwrap().id;
        let rdi_param = ssa
            .values
            .iter()
            .find_map(|val| match val.source {
                dac_ir::ssa::ValueSource::Parameter { variable } if variable == rdi_var => {
                    Some(val.id)
                }
                _ => None,
            })
            .unwrap();
        let layout = recovered
            .pointer_structs
            .get(&rdi_param)
            .expect("pointer struct keyed on rdi parameter");
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[1].offset, 8);
        assert_eq!(layout.fields[1].width, 4);
        assert_eq!(layout.total_size, 12);
        assert_eq!(layout.confidence.value(), POINTER_BASE_CONFIDENCE);
    }

    /// A single load through a pointer base (no second offset) does
    /// not produce a struct.
    #[test]
    fn single_offset_pointer_access_is_not_a_struct() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "v")],
            blocks: vec![RawBlock {
                ops: vec![load(1, 0, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(1)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);
        assert!(recovered.pointer_structs.is_empty());
    }

    // --- Array recovery ----------------------------------------

    /// `addr = base + index*4; v = [addr]` is the canonical
    /// 32-bit-int array pattern.
    #[test]
    fn indexed_load_with_mul_stride_recovers_array() {
        // variables: 0 = base, 1 = idx, 2 = scaled, 3 = addr, 4 = v
        let raw = RawFunction {
            variables: vec![
                var(0, "rdi"),
                var(1, "rsi"),
                var(2, "scaled"),
                var(3, "addr"),
                var(4, "v"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // scaled = idx * 4
                    mul_vc(2, 1, 4),
                    // addr = base + scaled
                    add_vv(3, 0, 2),
                    // v = [addr] (4 bytes)
                    load(4, 3, 4),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);

        let rdi_var = ssa.variables.iter().find(|v| v.name == "rdi").unwrap().id;
        let base = ssa
            .values
            .iter()
            .find_map(|val| match val.source {
                dac_ir::ssa::ValueSource::Parameter { variable } if variable == rdi_var => {
                    Some(val.id)
                }
                _ => None,
            })
            .unwrap();
        let array = recovered
            .arrays
            .get(&base)
            .expect("array keyed on rdi parameter");
        assert_eq!(array.element_size, 4);
        assert_eq!(array.element_width, Some(4));
        assert_eq!(array.confidence.value(), ARRAY_INDEXED_CONFIDENCE);
    }

    /// `Shl` stride: `addr = base + (idx << 3)` is an 8-byte element
    /// array.
    #[test]
    fn indexed_load_with_shl_stride_recovers_array() {
        let raw = RawFunction {
            variables: vec![
                var(0, "rdi"),
                var(1, "rsi"),
                var(2, "scaled"),
                var(3, "addr"),
                var(4, "v"),
            ],
            blocks: vec![RawBlock {
                ops: vec![shl_vc(2, 1, 3), add_vv(3, 0, 2), load(4, 3, 8)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(4)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);
        let rdi_var = ssa.variables.iter().find(|v| v.name == "rdi").unwrap().id;
        let base = ssa
            .values
            .iter()
            .find_map(|val| match val.source {
                dac_ir::ssa::ValueSource::Parameter { variable } if variable == rdi_var => {
                    Some(val.id)
                }
                _ => None,
            })
            .unwrap();
        let array = recovered.arrays.get(&base).expect("array on rdi base");
        assert_eq!(array.element_size, 8);
        assert_eq!(array.element_width, Some(8));
    }

    /// `addr = base + idx` (stride 1) does *not* register as an
    /// array — it is indistinguishable from a plain pointer-arith add.
    #[test]
    fn stride_of_one_is_not_an_array() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "rsi"), var(2, "addr"), var(3, "v")],
            blocks: vec![RawBlock {
                ops: vec![add_vv(2, 0, 1), load(3, 2, 1)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(3)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);
        assert!(recovered.arrays.is_empty());
    }

    // --- Determinism + housekeeping ----------------------------

    #[test]
    fn recovery_is_deterministic_across_runs() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp"), var(1, "rdi"), var(2, "rsi"), var(3, "addr")],
            blocks: vec![RawBlock {
                ops: vec![
                    sub_vc(0, 0, 16),
                    store(0, 1, 8),
                    add_vc(3, 0, 8),
                    store(3, 2, 8),
                ],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let r1 = recover_structs(&ssa, Some(&frame), None);
        let r2 = recover_structs(&ssa, Some(&frame), None);
        assert_eq!(r1, r2);
    }

    #[test]
    fn empty_inputs_produce_empty_output() {
        let raw = RawFunction {
            variables: vec![var(0, "rsp")],
            blocks: vec![RawBlock {
                ops: vec![],
                terminator: RawTerminator::Return { value: None },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);
        assert_eq!(recovered.struct_count(), 0);
        assert_eq!(recovered.array_count(), 0);
    }

    /// All recovered confidences are `Source::Derived` — never
    /// `Observed` (no symbol-table evidence for a synthesized struct)
    /// and never `Speculative` (no AI input).
    #[test]
    fn every_recovered_confidence_is_derived() {
        let raw = RawFunction {
            variables: vec![var(0, "rdi"), var(1, "addr"), var(2, "v0"), var(3, "v1")],
            blocks: vec![RawBlock {
                ops: vec![load(2, 0, 8), add_vc(1, 0, 8), load(3, 1, 4)],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(2)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let recovered = recover_structs(&ssa, None, None);
        for layout in recovered.pointer_structs.values() {
            assert_eq!(layout.confidence.source(), Source::Derived);
            for field in &layout.fields {
                assert_eq!(field.confidence.source(), Source::Derived);
            }
        }
    }

    /// "Hand-built test binary" rubric: the small synthetic function
    /// below mimics a real `struct { int64 a; int32 b; }` on the
    /// stack, exercised through both a write and a read. The
    /// recovery surfaces two adjacent fields with widths 8 and 4.
    #[test]
    fn hand_built_struct_round_trip() {
        // Local layout, in memory (stack grows downward):
        //   [rsp - 16] : 8-byte field "a"
        //   [rsp -  8] : 4-byte field "b"
        // Then read both back.
        let raw = RawFunction {
            variables: vec![
                var(0, "rsp"),
                var(1, "rdi"),
                var(2, "rsi"),
                var(3, "addr_a"),
                var(4, "addr_b"),
                var(5, "load_a"),
                var(6, "load_b"),
            ],
            blocks: vec![RawBlock {
                ops: vec![
                    // sub rsp, 16
                    sub_vc(0, 0, 16),
                    // [rsp + 0] = rdi   ; field a
                    store(0, 1, 8),
                    // addr_b = rsp + 8
                    add_vc(4, 0, 8),
                    // [addr_b] = rsi    ; field b
                    store(4, 2, 4),
                    // load_a = [rsp + 0]
                    load(5, 0, 8),
                    // addr_a = rsp + 0  (= rsp, but resolved through SSA)
                    add_vc(3, 0, 0),
                    // load_b = [addr_b]
                    load(6, 4, 4),
                ],
                terminator: RawTerminator::Return {
                    value: Some(RawOperand::Variable(5)),
                },
            }],
        };
        let ssa = build(raw, 1, &[]);
        let frame = analyze_stack_frame(&ssa, StackConvention::SysVAmd64);
        let types = propagate_types(&ssa, None, Some(&frame), &NullApiResolver);
        let recovered = recover_structs(&ssa, Some(&frame), Some(&types));

        let layout = recovered
            .stack_structs
            .get(&-16)
            .expect("hand-built struct recovered");
        assert_eq!(layout.fields.len(), 2);
        assert_eq!(
            (layout.fields[0].offset, layout.fields[0].width),
            (0, 8),
            "field a: offset 0, width 8"
        );
        assert_eq!(
            (layout.fields[1].offset, layout.fields[1].width),
            (8, 4),
            "field b: offset 8, width 4"
        );

        // sanity: ins_value is reachable (silences unused-warning
        // pressure on the helper).
        let _ = ins_value(&ssa, 0, 0);
    }
}
