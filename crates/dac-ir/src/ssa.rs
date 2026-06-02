//! SSA IR (B2.3, FR-11).
//!
//! The SSA layer follows the CFG layer in dac's IR stack
//! (`ARCHITECTURE.md` §4). Each [`SsaFunction`] is built from a CFG plus
//! a per-block stream of `(def, uses)` operations supplied by the lifter
//! — see `dac_analysis::ssa::construct_ssa` for the construction.
//!
//! ## What lands at B2.3
//!
//! - [`SsaFunction`] / [`SsaBlock`] — the per-function SSA graph. One
//!   block carries the same id as its CFG counterpart so the lifter
//!   does not have to maintain a parallel mapping. Predecessors are
//!   stored on the block (sorted ascending) so SSA passes do not have
//!   to re-walk the CFG edges.
//! - [`Phi`] — block parameter expressed in traditional dac syntax:
//!   `dst = phi (pred_i, op_i)`. Multiple back-edges with the same
//!   header land in a single phi node per variable; the incoming
//!   list is sorted by predecessor block id.
//! - [`SsaInstruction`] / [`SsaOp`] — closed enum of SSA operations.
//!   Operations new to a later batch land as new variants; the
//!   [`SsaOp::Opaque`] arm is the pressure-release valve mirroring
//!   [`crate::instr::Operation::Opaque`] (I-6).
//! - [`Operand`] — value-typed operand vocabulary: a defined SSA
//!   value, an integer constant, or [`Operand::Undef`] for reads of
//!   variables that have no reaching definition.
//! - [`Variable`] / [`ValueDef`] — bookkeeping that ties an SSA
//!   value back to the abstract "register" the lifter was tracking
//!   (so name-recovery passes can group all values for `rax`
//!   together) and to its defining site for evidence rendering.
//!
//! ## What deliberately doesn't land yet
//!
//! - Width-aware SSA. Operands carry no width attribute at this layer
//!   — type recovery is B2.6's job and lives on the [`Variable`]
//!   instead of being threaded through every operand.
//! - Memory SSA. Loads and stores are modelled as instructions but
//!   there is no `memory` token chaining them; dataflow + alias
//!   analysis comes at B2.4.
//! - Side-effect tracking on calls. [`SsaOp::Call`] records its
//!   target and argument list only; flag and clobber analysis lands
//!   alongside calling-convention inference (B2.5).

use dac_core::EvidenceId;

/// Numeric handle for a value definition inside a single
/// [`SsaFunction`]. Values are dense indices into
/// [`SsaFunction::values`], so `func.values[id as usize].id == id` for
/// every well-formed function.
pub type ValueId = u32;

/// Numeric handle for a block inside a single [`SsaFunction`]. Equals
/// the corresponding CFG `BlockId`, so consumers that already track
/// a `BlockId` can index into the SSA representation without a
/// translation table.
pub type SsaBlockId = u32;

/// Numeric handle for an abstract variable (the "register" the lifter
/// was tracking — `rax`, `rbx`, a stack slot). Dense indices into
/// [`SsaFunction::variables`].
pub type VariableId = u32;

/// An SSA-form function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaFunction {
    /// Function entry virtual address. Inherited from the source CFG so
    /// the evidence graph and emitted reports can address the function
    /// the same way both layers do.
    pub function_address: u64,
    /// Symbolic function name when known.
    pub function_name: Option<String>,
    /// Blocks indexed by [`SsaBlockId`], same numbering as the source
    /// CFG.
    pub blocks: Vec<SsaBlock>,
    /// Entry block id.
    pub entry: SsaBlockId,
    /// Variable table. The constructor populates this from the lifter's
    /// abstract-register set.
    pub variables: Vec<Variable>,
    /// Value definition table — one entry per [`ValueId`].
    pub values: Vec<ValueDef>,
    /// Evidence-graph handle inherited from the source CFG.
    pub evidence: EvidenceId,
}

impl SsaFunction {
    /// Look up a value definition by id.
    #[must_use]
    pub fn value(&self, id: ValueId) -> &ValueDef {
        &self.values[id as usize]
    }

    /// Look up a variable by id.
    #[must_use]
    pub fn variable(&self, id: VariableId) -> &Variable {
        &self.variables[id as usize]
    }

    /// Look up a block by id.
    #[must_use]
    pub fn block(&self, id: SsaBlockId) -> &SsaBlock {
        &self.blocks[id as usize]
    }
}

/// One basic block in SSA form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaBlock {
    /// Block id — also the index in [`SsaFunction::blocks`].
    pub id: SsaBlockId,
    /// Predecessor block ids, sorted ascending. Matches the source
    /// CFG's predecessor set.
    pub predecessors: Vec<SsaBlockId>,
    /// Phi nodes at block entry. One phi per `(variable)` that needs
    /// an `incoming` choice across predecessors.
    pub phis: Vec<Phi>,
    /// Block instructions in source order.
    pub instructions: Vec<SsaInstruction>,
    /// Block terminator. Drives the outgoing edges.
    pub terminator: SsaTerminator,
}

/// A phi node — `dst = phi (pred_0, op_0), (pred_1, op_1), …`.
///
/// The `incoming` list is sorted by predecessor block id so the
/// representation is byte-stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phi {
    /// Value defined by the phi.
    pub dst: ValueId,
    /// Variable this phi merges.
    pub variable: VariableId,
    /// `(predecessor block id, operand)` pairs, sorted by predecessor.
    /// Every predecessor block of the containing block appears exactly
    /// once. [`Operand::Undef`] marks a predecessor along which the
    /// variable has no reaching definition.
    pub incoming: Vec<(SsaBlockId, Operand)>,
}

/// One SSA instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaInstruction {
    /// Defined value when the instruction produces one; `None` for
    /// pure side-effect instructions (stores, void calls, …).
    pub dst: Option<ValueId>,
    /// Operation kind.
    pub op: SsaOp,
}

/// Closed enumeration of SSA operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaOp {
    /// `dst = src`. Coalesced copy. Trivial CSE collapses redundant
    /// moves, but the move itself is retained as a first-class op so
    /// the lifter can express a renamed `mov`.
    Move {
        src: Operand,
    },
    /// `dst = lhs + rhs`.
    Add {
        lhs: Operand,
        rhs: Operand,
    },
    Sub {
        lhs: Operand,
        rhs: Operand,
    },
    Mul {
        lhs: Operand,
        rhs: Operand,
    },
    And {
        lhs: Operand,
        rhs: Operand,
    },
    Or {
        lhs: Operand,
        rhs: Operand,
    },
    Xor {
        lhs: Operand,
        rhs: Operand,
    },
    Shl {
        lhs: Operand,
        rhs: Operand,
    },
    Shr {
        lhs: Operand,
        rhs: Operand,
    },
    Neg {
        src: Operand,
    },
    Not {
        src: Operand,
    },
    /// `dst = (lhs <kind> rhs)` — boolean comparison, value is 0 or 1.
    Compare {
        kind: CompareKind,
        lhs: Operand,
        rhs: Operand,
    },
    /// `dst = mem[address]` reading `width` bytes.
    Load {
        address: Operand,
        width: u8,
    },
    /// `mem[address] = value` writing `width` bytes.
    Store {
        address: Operand,
        value: Operand,
        width: u8,
    },
    /// Direct or indirect call. The result, when modelled, lands in
    /// `dst`; otherwise the call is treated as side-effect only.
    Call {
        target: Option<u64>,
        args: Vec<Operand>,
    },
    /// Operation the lifter does not yet model; preserved so the
    /// pipeline keeps moving (I-6). Operands are carried through so
    /// dataflow remains conservative-correct.
    Opaque {
        mnemonic: String,
        args: Vec<Operand>,
    },
}

/// Comparison kinds for [`SsaOp::Compare`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompareKind {
    Eq,
    Ne,
    /// Signed less-than.
    Lt,
    /// Signed less-than-or-equal.
    Le,
    /// Signed greater-than.
    Gt,
    /// Signed greater-than-or-equal.
    Ge,
    /// Unsigned less-than.
    Ult,
    /// Unsigned less-than-or-equal.
    Ule,
    /// Unsigned greater-than.
    Ugt,
    /// Unsigned greater-than-or-equal.
    Uge,
}

/// Operand vocabulary for SSA instructions.
///
/// `Ord` is implemented with a stable total order — `Undef <
/// Const(_) < Value(_)`, breaking ties by inner value — so operand
/// triples can serve as keys in a `BTreeMap`. Value-numbering passes
/// rely on this to canonicalize equivalent instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operand {
    /// A previously defined SSA value.
    Value(ValueId),
    /// Integer immediate.
    Const(i64),
    /// Read of a variable that has no reaching definition along this
    /// path. The interpreter treats it as zero; later passes can
    /// refine — but the marker is retained so we never silently
    /// invent a value (I-6).
    Undef,
}

impl Operand {
    /// Stable order key for use in BTreeMap-based value numbering and
    /// canonicalization. The order is purely structural and does not
    /// imply numeric comparison on `Const` values.
    fn order_key(self) -> (u8, i64, ValueId) {
        match self {
            Operand::Undef => (0, 0, 0),
            Operand::Const(c) => (1, c, 0),
            Operand::Value(v) => (2, 0, v),
        }
    }
}

impl PartialOrd for Operand {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Operand {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.order_key().cmp(&other.order_key())
    }
}

/// Block terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaTerminator {
    /// Unconditional jump to `target`.
    Jump { target: SsaBlockId },
    /// Conditional branch: `taken` when `cond` is non-zero,
    /// `not_taken` otherwise.
    Branch {
        cond: Operand,
        taken: SsaBlockId,
        not_taken: SsaBlockId,
    },
    /// Return; `value` carries the value placed in the return slot
    /// when the lifter knows it.
    Return { value: Option<Operand> },
    /// Indirect branch / `jmp rax` / computed jump table that has not
    /// been resolved yet. Treated as an exit by the constructor.
    Indirect,
    /// Block was unreachable from the entry, or its CFG terminator
    /// was decoded as invalid. Retained so the SSA function still
    /// covers every CFG block (I-2 traceability).
    Unreachable,
}

/// Variable metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub id: VariableId,
    /// Canonical name of the abstract register (lowercased), e.g.
    /// `"rax"`. Matches the [`crate::instr::Operand::Register::name`]
    /// the lifter emits.
    pub name: String,
    /// Width in bits. `0` when unknown; otherwise typically
    /// `8 / 16 / 32 / 64`.
    pub width_bits: u16,
}

/// Provenance for one SSA value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueDef {
    pub id: ValueId,
    /// What defined this value.
    pub source: ValueSource,
    /// The abstract variable this value belongs to. Multiple values
    /// can share a variable when the lifter renames it across
    /// definitions.
    pub variable: VariableId,
}

/// Where an SSA value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    /// Defined by an instruction at `blocks[block].instructions[index]`.
    Instruction { block: SsaBlockId, index: u32 },
    /// Defined by a phi at `blocks[block].phis[index]`.
    Phi { block: SsaBlockId, index: u32 },
    /// Function entry value of a variable — read but never previously
    /// written on the path to entry.
    Parameter { variable: VariableId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_kind_round_trip_through_pattern_match() {
        // Compile-time guard: every variant is reachable. Update this
        // when adding a new compare kind.
        let kinds = [
            CompareKind::Eq,
            CompareKind::Ne,
            CompareKind::Lt,
            CompareKind::Le,
            CompareKind::Gt,
            CompareKind::Ge,
            CompareKind::Ult,
            CompareKind::Ule,
            CompareKind::Ugt,
            CompareKind::Uge,
        ];
        for k in kinds {
            match k {
                CompareKind::Eq
                | CompareKind::Ne
                | CompareKind::Lt
                | CompareKind::Le
                | CompareKind::Gt
                | CompareKind::Ge
                | CompareKind::Ult
                | CompareKind::Ule
                | CompareKind::Ugt
                | CompareKind::Uge => {}
            }
        }
    }

    #[test]
    fn operand_is_copy_and_hashable() {
        // Compile-time check: callers can stash operands in HashMaps
        // for value numbering without a clone.
        fn _takes_copy(_: Operand) {}
        let o = Operand::Value(0);
        _takes_copy(o);
        _takes_copy(o);
    }
}
