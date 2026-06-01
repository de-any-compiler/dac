//! Architecture-neutral instruction decoder surface.
//!
//! The trait and types here are everything downstream passes (CFG
//! construction, function discovery, the `-O0` text backend) need to
//! reason about decoded instructions without depending on any one ISA's
//! library types. ISA-specific decoders implement [`InstructionDecoder`]
//! inside their own crate (`dac-arch-x86`, …) and project the rich
//! per-instruction state into the shared [`DecodedInstruction`] view.

use core::fmt;

/// A single decoded instruction in arch-neutral form.
///
/// This is the boundary at which iced-x86 (or any other concrete decoder)
/// stops being visible to the rest of dac. Anything an upstream pass
/// needs to do with an instruction — address it, measure it, capture its
/// bytes for evidence, classify its control-flow contribution, print it —
/// goes through this struct.
///
/// Rich per-operand state (operand kinds, register reads/writes, memory
/// modes) stays inside the lifter (B1.4), which is also architecture-
/// local. The decoded form here is deliberately stringly-typed for
/// operands because the only consumer at B1.3 is a textual disassembly
/// view; the lifter consumes the ISA-specific instruction directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedInstruction {
    /// Virtual address where the first byte of the instruction lives.
    pub address: u64,
    /// Encoded length in bytes (`1..=15` for x86, `4` for AArch64 …).
    pub length: usize,
    /// Captured copy of the encoded bytes. Costs a small allocation per
    /// instruction but lets the lifter mint a `Bytes` evidence node
    /// without holding a reference to the original section buffer.
    pub bytes: Vec<u8>,
    /// Mnemonic (e.g. `"mov"`). Lowercase, no padding.
    pub mnemonic: String,
    /// Formatted operand string (e.g. `"rax,rbx"`). Empty for
    /// zero-operand instructions.
    pub operands: String,
    /// Control-flow contribution of this instruction. Used by CFG
    /// construction (B2.1) and the linear-sweep disassembler today.
    pub flow: ControlFlow,
    /// `true` if the encoder recognized this byte sequence as a legal
    /// instruction. `false` indicates either truly invalid bytes (data
    /// interleaved with code, mid-instruction sweep restart) or an
    /// instruction the decoder does not yet support. Either way the
    /// pipeline degrades — it does not invent semantics (I-6).
    pub valid: bool,
}

impl fmt::Display for DecodedInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.operands.is_empty() {
            write!(f, "{:#x}: {}", self.address, self.mnemonic)
        } else {
            write!(
                f,
                "{:#x}: {} {}",
                self.address, self.mnemonic, self.operands
            )
        }
    }
}

/// Arch-neutral classification of an instruction's effect on control
/// flow. CFG construction (B2.1) and the function-discovery heuristics
/// (B1.5) both read through this enum, so it is the single point at
/// which "did this jump?" becomes a pipeline-wide concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlow {
    /// Falls through to the next instruction.
    Sequential,
    /// Conditional near branch with an immediate target (when known).
    ConditionalBranch {
        /// Resolved target VA, when the branch is direct. Indirect
        /// conditional branches still expose this as `None`.
        target: Option<u64>,
    },
    /// Unconditional near branch with an immediate target (when known).
    UnconditionalBranch { target: Option<u64> },
    /// Register- or memory-indirect branch (`jmp rax`, `jmp [rax]`, …).
    IndirectBranch,
    /// Direct call.
    Call { target: Option<u64> },
    /// Indirect call (`call rax`, `call [rip+disp]`, …).
    IndirectCall,
    /// Procedure return.
    Return,
    /// Interrupt / trap / system call.
    Interrupt,
    /// Decoder produced an invalid instruction; flow is unknowable here.
    Invalid,
}

/// Errors a decoder can report from [`InstructionDecoder::decode_one`].
///
/// The iterator API ([`InstructionDecoder::iter`]) deals with invalid
/// bytes by emitting a `DecodedInstruction { valid: false, … }` and
/// advancing — it never errors mid-stream. This enum is reserved for the
/// single-shot path's "you didn't give me enough bytes to try at all"
/// failure mode, which is distinct from "I tried and the bytes were
/// garbage".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Caller passed an empty byte slice. `offset` is always `0` and is
    /// kept for parity with future variants that may carry positional
    /// information.
    Truncated { offset: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { offset } => write!(f, "truncated instruction at offset {offset}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Single-pass instruction decoder for one ISA.
///
/// Implementations are stateless from the caller's point of view: every
/// call decodes a fresh region, no cursor is held across calls. The
/// implementation may cache internal lookup tables; sharing a decoder
/// across threads is acceptable when `Self: Send + Sync`.
pub trait InstructionDecoder {
    /// Decode the instruction at the start of `bytes`, treating that
    /// first byte as virtual address `address`. Returns the decoded
    /// instruction (which may be `valid: false` if the encoding is
    /// illegal) or [`DecodeError::Truncated`] if the buffer is empty.
    fn decode_one(&self, bytes: &[u8], address: u64) -> Result<DecodedInstruction, DecodeError>;

    /// Linear sweep starting at `address`. Yields instructions in
    /// address order, including invalid ones (which the iterator skips
    /// past by advancing one byte). Stops when the underlying buffer is
    /// exhausted.
    fn iter<'a>(
        &'a self,
        bytes: &'a [u8],
        address: u64,
    ) -> Box<dyn Iterator<Item = DecodedInstruction> + 'a>;
}
