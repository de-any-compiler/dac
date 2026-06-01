//! `dac-arch-x86` — x86 and x86-64 architecture backend for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What B1.3 ships
//!
//! - [`X86_64`] and [`I386`] zero-sized [`dac_arch::Architecture`] impls.
//! - [`IcedDecoder`] — an [`dac_arch::InstructionDecoder`] that wraps
//!   `iced-x86` (ADR-0004). The iced types stay inside this module; the
//!   trait surface exposes only the arch-neutral
//!   [`dac_arch::DecodedInstruction`] view.
//! - Register-file metadata for both bitnesses (GPRs and their aliases,
//!   plus instruction-pointer / flags). Vector / FP registers come in
//!   later batches as the lifter starts to need them.
//!
//! The lifter (`B1.4`) will live in this crate too and consume the
//! iced `Instruction` directly for accurate operand semantics, while
//! still emitting the arch-neutral Instruction IR.

#![forbid(unsafe_code)]
#![allow(non_camel_case_types)]

mod decoder;
mod registers;

use dac_arch::{Architecture, Endianness, InstructionDecoder, Isa, RegisterFile};

pub use decoder::IcedDecoder;

/// 64-bit Intel / AMD architecture backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct X86_64;

impl Architecture for X86_64 {
    fn name(&self) -> &'static str {
        "x86-64"
    }

    fn isa(&self) -> Isa {
        Isa::X86_64
    }

    fn pointer_size(&self) -> usize {
        8
    }

    fn endianness(&self) -> Endianness {
        Endianness::Little
    }

    fn decoder(&self) -> Box<dyn InstructionDecoder> {
        Box::new(IcedDecoder::new(64))
    }

    fn register_file(&self) -> &RegisterFile {
        registers::x86_64_register_file()
    }
}

/// 32-bit Intel / AMD architecture backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct I386;

impl Architecture for I386 {
    fn name(&self) -> &'static str {
        "i386"
    }

    fn isa(&self) -> Isa {
        Isa::I386
    }

    fn pointer_size(&self) -> usize {
        4
    }

    fn endianness(&self) -> Endianness {
        Endianness::Little
    }

    fn decoder(&self) -> Box<dyn InstructionDecoder> {
        Box::new(IcedDecoder::new(32))
    }

    fn register_file(&self) -> &RegisterFile {
        registers::i386_register_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_arch::{ControlFlow, DecodeError};

    #[test]
    fn x86_64_arch_metadata_is_correct() {
        let arch = X86_64;
        assert_eq!(arch.name(), "x86-64");
        assert_eq!(arch.isa(), Isa::X86_64);
        assert_eq!(arch.pointer_size(), 8);
        assert_eq!(arch.endianness(), Endianness::Little);
        let rf = arch.register_file();
        assert!(rf.by_name("rax").is_some(), "x86-64 has rax");
        assert!(rf.by_name("r15").is_some(), "x86-64 has r15");
        assert!(rf.by_name("rip").is_some(), "x86-64 has rip");
        assert!(rf.by_name("rflags").is_some(), "x86-64 has rflags");
    }

    #[test]
    fn i386_arch_metadata_is_correct() {
        let arch = I386;
        assert_eq!(arch.name(), "i386");
        assert_eq!(arch.isa(), Isa::I386);
        assert_eq!(arch.pointer_size(), 4);
        assert_eq!(arch.endianness(), Endianness::Little);
        let rf = arch.register_file();
        assert!(rf.by_name("eax").is_some(), "i386 has eax");
        assert!(rf.by_name("ah").is_some(), "i386 has 8-bit high aliases");
        assert!(rf.by_name("eip").is_some(), "i386 has eip");
        // i386 must not advertise the 64-bit GP set.
        assert!(rf.by_name("rax").is_none(), "i386 must not expose rax");
    }

    #[test]
    fn decodes_mov_rax_rbx_snapshot() {
        // 48 89 D8 == `mov rax, rbx`. REX-prefixed two-operand form;
        // covers REX decoding + ModR/M handling in one shot.
        let bytes = [0x48, 0x89, 0xD8];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0x1000).expect("decodes cleanly");
        assert!(inst.valid);
        assert_eq!(inst.address, 0x1000);
        assert_eq!(inst.length, 3);
        assert_eq!(inst.bytes, bytes.to_vec());
        assert_eq!(inst.mnemonic, "mov");
        assert_eq!(inst.operands, "rax,rbx");
        assert_eq!(inst.flow, ControlFlow::Sequential);
    }

    #[test]
    fn decodes_ret_snapshot() {
        let bytes = [0xC3];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0).expect("ret decodes");
        assert!(inst.valid);
        assert_eq!(inst.mnemonic, "ret");
        assert_eq!(inst.flow, ControlFlow::Return);
    }

    #[test]
    fn decodes_call_with_near_target() {
        // E8 05 00 00 00 == `call +5` from a 0x1000 base.
        // Next IP after the 5-byte call is 0x1005; +5 displacement
        // resolves to 0x100A.
        let bytes = [0xE8, 0x05, 0x00, 0x00, 0x00];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0x1000).expect("call decodes");
        assert_eq!(inst.mnemonic, "call");
        match inst.flow {
            ControlFlow::Call { target: Some(t) } => assert_eq!(t, 0x100A),
            other => panic!("expected Call with target=0x100A, got {other:?}"),
        }
    }

    #[test]
    fn decodes_indirect_call() {
        // FF D0 == `call rax`.
        let bytes = [0xFF, 0xD0];
        let dec = X86_64.decoder();
        let inst = dec
            .decode_one(&bytes, 0x2000)
            .expect("indirect call decodes");
        assert_eq!(inst.mnemonic, "call");
        assert_eq!(inst.flow, ControlFlow::IndirectCall);
    }

    #[test]
    fn decodes_conditional_short_branch() {
        // 74 05 == `je short +5`. Next IP is 0x2002, +5 = 0x2007.
        let bytes = [0x74, 0x05];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0x2000).expect("je decodes");
        match inst.flow {
            ControlFlow::ConditionalBranch { target: Some(t) } => assert_eq!(t, 0x2007),
            other => panic!("expected ConditionalBranch target=0x2007, got {other:?}"),
        }
    }

    #[test]
    fn decodes_unconditional_short_branch() {
        // EB 02 == `jmp short +2`. Next IP is 0x3002, +2 = 0x3004.
        let bytes = [0xEB, 0x02];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0x3000).expect("jmp decodes");
        match inst.flow {
            ControlFlow::UnconditionalBranch { target: Some(t) } => assert_eq!(t, 0x3004),
            other => panic!("expected UnconditionalBranch target=0x3004, got {other:?}"),
        }
    }

    #[test]
    fn decodes_indirect_branch() {
        // FF E0 == `jmp rax`.
        let bytes = [0xFF, 0xE0];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0).expect("indirect jmp decodes");
        assert_eq!(inst.flow, ControlFlow::IndirectBranch);
    }

    #[test]
    fn iter_walks_a_known_sequence() {
        // mov eax, 1     (B8 01 00 00 00)            — 5 bytes
        // xor edi, edi   (31 FF)                     — 2 bytes
        // ret            (C3)                        — 1 byte
        // nop            (90)                        — 1 byte
        let bytes = [0xB8, 0x01, 0x00, 0x00, 0x00, 0x31, 0xFF, 0xC3, 0x90];
        let dec = X86_64.decoder();
        let insts: Vec<_> = dec.iter(&bytes, 0x4000).collect();
        assert_eq!(insts.len(), 4, "four instructions");
        assert_eq!(insts[0].address, 0x4000);
        assert_eq!(insts[0].length, 5);
        assert_eq!(insts[0].mnemonic, "mov");
        assert_eq!(insts[1].address, 0x4005);
        assert_eq!(insts[1].mnemonic, "xor");
        assert_eq!(insts[2].mnemonic, "ret");
        assert_eq!(insts[2].flow, ControlFlow::Return);
        assert_eq!(insts[3].mnemonic, "nop");
        // Sum of lengths equals input size — full consumption.
        let consumed: usize = insts.iter().map(|i| i.length).sum();
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn empty_bytes_decode_one_is_truncated() {
        let dec = X86_64.decoder();
        assert_eq!(
            dec.decode_one(&[], 0),
            Err(DecodeError::Truncated { offset: 0 })
        );
    }

    #[test]
    fn invalid_64bit_only_opcode_yields_invalid_instruction() {
        // `0x06` is `push es` on i386 but reserved / invalid in 64-bit
        // mode. iced reports `is_invalid`; we surface this as
        // `valid: false` with `ControlFlow::Invalid` and a `(bad)`
        // mnemonic (degrade, don't invent — I-6).
        let bytes = [0x06];
        let dec = X86_64.decoder();
        let inst = dec.decode_one(&bytes, 0).expect("returns a record");
        assert!(!inst.valid);
        assert_eq!(inst.mnemonic, "(bad)");
        assert_eq!(inst.flow, ControlFlow::Invalid);
    }

    #[test]
    fn i386_decoder_uses_32bit_bitness() {
        // 50 == `push eax` in 32-bit (valid), and `push rax` in 64-bit.
        // Either way it's a valid one-byte instruction; the difference
        // is purely operand width. We verify the i386 decoder picks the
        // 32-bit operand by way of the disassembly text.
        let bytes = [0x50];
        let dec = I386.decoder();
        let inst = dec.decode_one(&bytes, 0).expect("decodes");
        assert!(inst.valid);
        assert_eq!(inst.mnemonic, "push");
        assert_eq!(inst.operands, "eax");
    }

    #[test]
    fn iter_does_not_stall_on_invalid_bytes() {
        // `06` is invalid in 64-bit mode. Whether iced consumes it
        // alone (length 1) or absorbs trailing bytes as part of the
        // invalid encoding is an implementation detail; the invariant
        // dac relies on is that the iterator makes forward progress
        // and consumes every byte of the buffer (NFR-4 in spirit).
        let bytes = [0x06, 0xC3];
        let dec = X86_64.decoder();
        let insts: Vec<_> = dec.iter(&bytes, 0x5000).collect();
        assert!(!insts.is_empty(), "iter emitted at least one record");
        let total: usize = insts.iter().map(|i| i.length).sum();
        assert_eq!(total, bytes.len(), "iter consumed every input byte");
        assert!(
            insts.iter().any(|i| !i.valid),
            "at least one record is reported as invalid",
        );
    }
}
