//! Instruction IR — the first arch-neutral IR layer (`ARCHITECTURE.md` §4).
//!
//! `dac-arch-x86` (and any future architecture crate) lifts decoded
//! instructions into this vocabulary. Downstream passes — CFG (B2.1),
//! function discovery (B1.5), the `-O0` text backend (B1.6) — read through
//! it without depending on any ISA-specific decoder.
//!
//! ## What lands at B1.4
//!
//! - [`InstructionIr`] — the per-instruction node. Address + length
//!   together describe the bytes it was lifted from, which is the
//!   provenance hook for I-2: any orchestrator can mint an
//!   `EvidenceNode::Bytes { start, end }` plus an `EvidenceNode::IrNode`
//!   from these two fields and wire them in `dac-core`'s evidence graph.
//! - [`Operation`] — a closed enum of lifted ops. Unsupported opcodes
//!   become [`Operation::Opaque`] so later passes still see CFG edges
//!   (I-6: degrade, don't invent).
//! - [`Operand`] — typed operand vocabulary (register, immediate, memory,
//!   branch target).
//! - [`Target`] / [`Condition`] — branch / call destinations and
//!   conditional-branch codes.
//!
//! ## What deliberately doesn't land yet
//!
//! - Provenance as a stored `EvidenceId`. The IR carries the byte span
//!   that *will* be wired into the evidence graph, but the wiring itself
//!   is the job of an orchestrator (B1.5+). That keeps the lifter pure —
//!   it does not need a mutable graph to do its job — and lets unit
//!   tests construct IR without spinning up `dac-core`.
//! - Per-operand size attributes beyond what an x86 lifter needs to round-
//!   trip immediates. Type recovery is B2.6's problem.

use core::fmt;

/// A single lifted instruction.
///
/// The address + length pair *is* the provenance: it names the byte
/// range in the source binary that produced this node. An orchestrator
/// linking the IR into a `dac_core::EvidenceGraph` can construct an
/// `EvidenceNode::Bytes { start: address, end: address + length as u64 }`
/// and a corresponding `EvidenceNode::IrNode { layer: Instruction, … }`
/// from this struct alone — see invariant I-2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionIr {
    /// Virtual address of the first byte of the instruction.
    pub address: u64,
    /// Encoded length in bytes. `1..=15` on x86; fixed widths on
    /// AArch64. Together with [`address`](Self::address), names the
    /// byte span this node was lifted from.
    pub length: u32,
    /// Lifted operation. `Opaque` for opcodes the lifter does not yet
    /// model — the CFG edges are still recoverable from the byte span
    /// because control-flow extraction lives on the decoder, not on
    /// this layer.
    pub op: Operation,
}

impl InstructionIr {
    /// `true` if this instruction was lifted into a modelled operation.
    /// `false` only for [`Operation::Opaque`], so this is the predicate
    /// the coverage report counts against.
    #[must_use]
    pub fn is_lifted(&self) -> bool {
        !matches!(self.op, Operation::Opaque { .. })
    }

    /// Byte range `[address, address + length)`. Convenience for
    /// orchestrators wiring provenance into the evidence graph.
    #[must_use]
    pub fn byte_range(&self) -> (u64, u64) {
        (
            self.address,
            self.address.wrapping_add(u64::from(self.length)),
        )
    }
}

/// Closed enumeration of operations the Instruction IR can carry.
///
/// New operations land as new variants; existing IR consumers that
/// pattern-match must explicitly handle them. The `Opaque` arm is the
/// pressure-release valve: any opcode the lifter does not yet model goes
/// here with its mnemonic preserved, so the CFG / function-discovery
/// passes still see something at this address (I-6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// `dst = src`. Covers `mov`, `movzx`, `movsx`, `movsxd` on x86 —
    /// the zero/sign-extending forms are tracked by their respective
    /// `Operand::Register::size_bits` widths on dst and src; the
    /// extension policy is a type-recovery concern (B2.6) and is not
    /// represented here.
    Move {
        dst: Operand,
        src: Operand,
    },
    /// `dst = &src` (address-of). x86's `lea`.
    LoadAddress {
        dst: Operand,
        src: Operand,
    },
    /// `dst = dst + src`. Covers `add`, `adc` (carry-in for `adc` is
    /// implicit and surfaces in semantic IR later).
    Add {
        dst: Operand,
        src: Operand,
    },
    /// `dst = dst - src`. Covers `sub`, `sbb`.
    Sub {
        dst: Operand,
        src: Operand,
    },
    /// `dst = dst * src`. Covers `mul`, `imul`.
    Mul {
        dst: Operand,
        src: Operand,
    },
    /// `dst = dst / src`. Covers `div`, `idiv`.
    Div {
        dst: Operand,
        src: Operand,
    },
    And {
        dst: Operand,
        src: Operand,
    },
    Or {
        dst: Operand,
        src: Operand,
    },
    Xor {
        dst: Operand,
        src: Operand,
    },
    Shl {
        dst: Operand,
        src: Operand,
    },
    Shr {
        dst: Operand,
        src: Operand,
    },
    Sar {
        dst: Operand,
        src: Operand,
    },
    Not {
        dst: Operand,
    },
    Neg {
        dst: Operand,
    },
    /// `flags = lhs - rhs` (result discarded). The lifter emits this
    /// for `cmp`; structuring (B2.7) pairs it with the next `Jcc`.
    Compare {
        lhs: Operand,
        rhs: Operand,
    },
    /// `flags = lhs & rhs` (result discarded). x86 `test`.
    Test {
        lhs: Operand,
        rhs: Operand,
    },
    Push {
        src: Operand,
    },
    Pop {
        dst: Operand,
    },
    /// Direct, indirect, conditional, and unconditional branches all
    /// land here; the [`Target`] and [`Condition`] distinguish them.
    /// CFG construction (B2.1) walks this arm.
    Jump {
        target: Target,
        condition: Option<Condition>,
    },
    /// Direct or indirect call.
    Call {
        target: Target,
    },
    /// Procedure return.
    Return,
    /// No-op. Includes ENDBR32 / ENDBR64 (CET landing pads) since their
    /// semantic effect at this layer is nil.
    Nop,
    /// Interrupt / trap. `int N`, `int3`, `into`. `vector` is `Some`
    /// when the immediate is known, `None` otherwise.
    Interrupt {
        vector: Option<u8>,
    },
    /// Linux / SysV `syscall` (and Windows `sysenter`-likes). Treated
    /// as a distinct op from interrupts because calling-convention
    /// inference (B2.5) reads them differently.
    Syscall,
    /// Opcode the lifter does not yet model. The mnemonic is preserved
    /// for the coverage report and for the `-O0` listing backend. CFG
    /// edges still come from the decoder's `ControlFlow` so the rest
    /// of the pipeline keeps working (I-6).
    Opaque {
        mnemonic: String,
    },
}

/// Operand vocabulary. Stays string-keyed for register names so the IR
/// is decoupled from any ISA's [`dac_arch::RegisterFile`]; orchestrators
/// who want a typed lookup can pass the name through
/// `RegisterFile::by_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Register {
        /// Lowercase canonical name (e.g. `"rax"`, `"r8d"`). Matches
        /// the keys in the architecture's `RegisterFile`.
        name: String,
        /// Width in bits. `8 / 16 / 32 / 64` on x86, plus `128 / 256 /
        /// 512` once vector registers land.
        size_bits: u16,
    },
    Immediate {
        /// Sign-extended to `i64` so the lifter can use a single
        /// arm for all integer immediates. Consumers that need the
        /// narrower view consult `size_bits`.
        value: i64,
        size_bits: u16,
    },
    /// `[segment:base + index*scale + displacement]`.
    Memory {
        base: Option<String>,
        index: Option<String>,
        /// `1`, `2`, `4`, or `8` on x86; `0` when no index is present.
        scale: u8,
        displacement: i64,
        /// Access width in bits (`8 / 16 / 32 / 64 / …`). Zero when the
        /// addressing mode does not imply a width (e.g. `lea`).
        size_bits: u16,
        /// Segment override (`fs`, `gs`, …). `None` when the default
        /// segment applies; consumers treat it as the platform's
        /// default.
        segment: Option<String>,
    },
    /// Direct branch target encoded inline in the instruction.
    Branch { target: u64 },
}

/// Branch / call destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Resolved virtual address (direct near branch / direct call).
    Direct(u64),
    /// Register- or memory-indirect (`jmp rax`, `call [rip+disp]`, …).
    /// The operand is the addressing form supplied by the decoder.
    Indirect(Operand),
}

/// Arch-neutral conditional-branch codes.
///
/// Maps onto x86's Jcc set; AArch64's `B.cond` lands on the same vocabulary
/// when that backend arrives. Signed vs unsigned comparisons are split
/// because structuring (B2.7) needs to know which `Compare` flag it is
/// looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Condition {
    Equal,
    NotEqual,
    /// Signed less-than.
    Less,
    /// Signed less-than-or-equal.
    LessEqual,
    /// Signed greater-than.
    Greater,
    /// Signed greater-than-or-equal.
    GreaterEqual,
    /// Unsigned less-than.
    Below,
    /// Unsigned less-than-or-equal.
    BelowEqual,
    /// Unsigned greater-than.
    Above,
    /// Unsigned greater-than-or-equal.
    AboveEqual,
    Sign,
    NotSign,
    Overflow,
    NotOverflow,
    Parity,
    NotParity,
    /// `(e/r)cx == 0` — the JCXZ / JECXZ / JRCXZ x86 idiom.
    CxZero,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Equal => "eq",
            Self::NotEqual => "ne",
            Self::Less => "lt",
            Self::LessEqual => "le",
            Self::Greater => "gt",
            Self::GreaterEqual => "ge",
            Self::Below => "b",
            Self::BelowEqual => "be",
            Self::Above => "a",
            Self::AboveEqual => "ae",
            Self::Sign => "s",
            Self::NotSign => "ns",
            Self::Overflow => "o",
            Self::NotOverflow => "no",
            Self::Parity => "p",
            Self::NotParity => "np",
            Self::CxZero => "cxz",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op_mov_rax_rbx() -> InstructionIr {
        InstructionIr {
            address: 0x1000,
            length: 3,
            op: Operation::Move {
                dst: Operand::Register {
                    name: "rax".to_string(),
                    size_bits: 64,
                },
                src: Operand::Register {
                    name: "rbx".to_string(),
                    size_bits: 64,
                },
            },
        }
    }

    #[test]
    fn lifted_instructions_report_lifted() {
        let ir = op_mov_rax_rbx();
        assert!(ir.is_lifted());
        assert_eq!(ir.byte_range(), (0x1000, 0x1003));
    }

    #[test]
    fn opaque_instructions_report_not_lifted() {
        let ir = InstructionIr {
            address: 0x2000,
            length: 4,
            op: Operation::Opaque {
                mnemonic: "vpmovzxbw".to_string(),
            },
        };
        assert!(!ir.is_lifted());
        assert_eq!(ir.byte_range(), (0x2000, 0x2004));
    }

    #[test]
    fn condition_display_uses_shortest_canonical_names() {
        assert_eq!(format!("{}", Condition::Equal), "eq");
        assert_eq!(format!("{}", Condition::NotEqual), "ne");
        assert_eq!(format!("{}", Condition::Below), "b");
        assert_eq!(format!("{}", Condition::AboveEqual), "ae");
        assert_eq!(format!("{}", Condition::CxZero), "cxz");
    }

    #[test]
    fn target_indirect_carries_addressing_form() {
        let t = Target::Indirect(Operand::Register {
            name: "rax".to_string(),
            size_bits: 64,
        });
        match t {
            Target::Indirect(Operand::Register { name, size_bits }) => {
                assert_eq!(name, "rax");
                assert_eq!(size_bits, 64);
            }
            _ => panic!("indirect should preserve the operand"),
        }
    }

    #[test]
    fn byte_range_does_not_overflow_at_u64_top() {
        // address + length cannot panic on overflow because the lifter
        // is allowed to produce IR for any reachable address range.
        let ir = InstructionIr {
            address: u64::MAX - 1,
            length: 4,
            op: Operation::Nop,
        };
        let (start, end) = ir.byte_range();
        assert_eq!(start, u64::MAX - 1);
        // Wrapping is the documented contract — the caller decides
        // whether to treat the range as invalid.
        assert_eq!(end, 2);
    }
}
