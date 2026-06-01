//! `iced-x86`-backed implementation of [`dac_arch::InstructionDecoder`].
//!
//! ADR-0004 picks `iced-x86`: rich `flow_control()` + `near_branch_target()`
//! metadata maps directly onto [`dac_arch::ControlFlow`], invalid encodings
//! are reported explicitly (matching I-6: degrade, don't invent), and the
//! library is pure Rust with no FFI.
//!
//! Everything iced-specific stays in this file. Downstream passes consume
//! only [`dac_arch::DecodedInstruction`].

use dac_arch::{ControlFlow, DecodeError, DecodedInstruction, InstructionDecoder};
use iced_x86::{
    Decoder, DecoderOptions, FlowControl, Formatter, Instruction, IntelFormatter, OpKind,
};

/// iced-x86–backed single-pass decoder.
///
/// `bitness` is `16`, `32`, or `64`. Each ISA backend constructs the
/// decoder with the appropriate bitness; callers never see it.
pub struct IcedDecoder {
    bitness: u32,
}

impl IcedDecoder {
    /// Construct a decoder for the given bitness. Panics in debug builds
    /// if `bitness` is not one of `16`, `32`, `64` — the only legal
    /// values for `iced_x86::Decoder`.
    #[must_use]
    pub fn new(bitness: u32) -> Self {
        debug_assert!(
            matches!(bitness, 16 | 32 | 64),
            "iced-x86 only supports 16/32/64-bit; got {bitness}",
        );
        Self { bitness }
    }
}

impl InstructionDecoder for IcedDecoder {
    fn decode_one(&self, bytes: &[u8], address: u64) -> Result<DecodedInstruction, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::Truncated { offset: 0 });
        }
        let mut decoder = Decoder::with_ip(self.bitness, bytes, address, DecoderOptions::NONE);
        // `can_decode` is true while the decoder has bytes to consume.
        // Non-empty input implies at least one decode attempt.
        let instr = decoder.decode();
        Ok(project(&instr, bytes, address))
    }

    fn iter<'a>(
        &'a self,
        bytes: &'a [u8],
        address: u64,
    ) -> Box<dyn Iterator<Item = DecodedInstruction> + 'a> {
        Box::new(IcedIter {
            bitness: self.bitness,
            bytes,
            address,
            offset: 0,
        })
    }
}

/// Linear sweep iterator. Re-creates a `Decoder` per step so the iter
/// remains lifetime-free of any iced cursor state; the cost is a few
/// extra cycles per instruction, but the decoder construction is
/// trivial and a sweep is dominated by the actual decode anyway.
struct IcedIter<'a> {
    bitness: u32,
    bytes: &'a [u8],
    address: u64,
    offset: usize,
}

impl Iterator for IcedIter<'_> {
    type Item = DecodedInstruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.bytes.len() {
            return None;
        }
        let here = &self.bytes[self.offset..];
        let address = self.address.wrapping_add(self.offset as u64);
        let mut decoder = Decoder::with_ip(self.bitness, here, address, DecoderOptions::NONE);
        let instr = decoder.decode();
        let mut decoded = project(&instr, here, address);
        // iced sets len=1 on invalid encodings, but defensively clamp
        // so a stray len=0 cannot wedge the iterator.
        if decoded.length == 0 {
            decoded.length = 1;
            decoded.bytes = here[..1].to_vec();
        }
        // Clamp to remaining bytes — for a partial trailing instruction
        // iced still returns `len = instr_size`, which can exceed the
        // remaining buffer. We surface that as a truncated invalid
        // record and stop the sweep on the next step.
        if decoded.length > here.len() {
            decoded.length = here.len();
            decoded.bytes = here.to_vec();
            decoded.valid = false;
            decoded.mnemonic = "(bad)".to_string();
            decoded.operands.clear();
            decoded.flow = ControlFlow::Invalid;
        }
        self.offset += decoded.length;
        Some(decoded)
    }
}

/// Build a [`DecodedInstruction`] from an iced [`Instruction`] and the
/// surrounding context (slice + base address). Centralised so the
/// single-shot and iterator paths cannot drift.
fn project(instr: &Instruction, bytes: &[u8], address: u64) -> DecodedInstruction {
    let valid = !instr.is_invalid();
    let length = instr.len();
    let captured_len = length.min(bytes.len());
    let captured = bytes[..captured_len].to_vec();
    let (mnemonic, operands) = if valid {
        format_instr(instr)
    } else {
        ("(bad)".to_string(), String::new())
    };
    let flow = if valid {
        control_flow(instr)
    } else {
        ControlFlow::Invalid
    };
    DecodedInstruction {
        address,
        length: captured_len,
        bytes: captured,
        mnemonic,
        operands,
        flow,
        valid,
    }
}

/// Format an iced instruction into `(mnemonic, operands)` using the
/// Intel-syntax formatter. iced emits a single line like
/// `"mov     rax,rbx"`; we split on the first whitespace and trim the
/// remainder so passes can pattern-match on the mnemonic without parsing.
fn format_instr(instr: &Instruction) -> (String, String) {
    let mut formatter = IntelFormatter::new();
    let opts = formatter.options_mut();
    opts.set_first_operand_char_index(0);
    opts.set_space_after_operand_separator(false);
    let mut s = String::new();
    formatter.format(instr, &mut s);
    match s.find(|c: char| c.is_whitespace()) {
        Some(idx) => {
            let (mnem, rest) = s.split_at(idx);
            (mnem.to_string(), rest.trim_start().to_string())
        }
        None => (s, String::new()),
    }
}

/// Project iced's `FlowControl` onto our arch-neutral [`ControlFlow`].
/// Direct branches and calls carry their resolved virtual-address target
/// (computed by iced from the instruction's IP at decode time); indirect
/// variants surface as the `Indirect*` arms with no target.
fn control_flow(instr: &Instruction) -> ControlFlow {
    match instr.flow_control() {
        FlowControl::Next => ControlFlow::Sequential,
        FlowControl::ConditionalBranch => ControlFlow::ConditionalBranch {
            target: near_branch_target(instr),
        },
        FlowControl::UnconditionalBranch => ControlFlow::UnconditionalBranch {
            target: near_branch_target(instr),
        },
        FlowControl::IndirectBranch => ControlFlow::IndirectBranch,
        FlowControl::Call => ControlFlow::Call {
            target: near_branch_target(instr),
        },
        FlowControl::IndirectCall => ControlFlow::IndirectCall,
        FlowControl::Return => ControlFlow::Return,
        FlowControl::Interrupt => ControlFlow::Interrupt,
        // Hardware transactional memory and decoder-recognized exception
        // paths fall through structurally; CFG construction will refine
        // them later if needed.
        FlowControl::XbeginXabortXend => ControlFlow::Sequential,
        FlowControl::Exception => ControlFlow::Invalid,
    }
}

/// Recover the near-branch target VA for a direct branch / call. iced
/// computes this from the IP we supplied at decoder construction time.
fn near_branch_target(instr: &Instruction) -> Option<u64> {
    for i in 0..instr.op_count() {
        match instr.op_kind(i) {
            OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64 => {
                return Some(instr.near_branch_target());
            }
            _ => {}
        }
    }
    None
}
