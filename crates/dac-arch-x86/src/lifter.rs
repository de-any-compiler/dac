//! `iced-x86`-backed implementation of [`dac_arch::InstructionLifter`].
//!
//! Decoder + lifter are sibling iced consumers: the decoder
//! ([`crate::decoder`]) produces the arch-neutral textual view used by
//! CFG construction and the `-O0` listing; the lifter here produces the
//! arch-neutral *semantic* view used by every later pass. Both go
//! through the same iced `Decoder::with_ip`, so the iced API surface
//! stays contained inside `dac-arch-x86` per ADR-0004.
//!
//! ## Coverage strategy
//!
//! 1. **Control flow first.** Every `Jcc`, `Jmp`, `Call`, `Ret`, `Int`,
//!    and `Syscall` lifts through [`Instruction::flow_control`] —
//!    matches the source of truth the decoder already uses, so the IR
//!    and the listing agree on classifications.
//! 2. **Common arithmetic / data-movement / stack subset.** Modeled
//!    explicitly: `mov`, `lea`, `add`, `sub`, `mul`, `imul`, `div`,
//!    `idiv`, `and`, `or`, `xor`, `not`, `neg`, `shl`, `shr`, `sar`,
//!    `inc`, `dec`, `cmp`, `test`, `push`, `pop`, `nop`, ENDBR
//!    landing pads.
//! 3. **Everything else falls through to [`Operation::Opaque`]** with
//!    the iced mnemonic preserved. The [`dac_arch::Coverage`] report
//!    surfaces the histogram so the next pass to extend the lifter
//!    knows which mnemonics to prioritise.
//!
//! ## Why opaque rather than error?
//!
//! Function discovery (B1.5) and CFG construction (B2.1) need to walk
//! the full `.text` byte by byte to recover boundaries; they cannot
//! skip an instruction because its semantics are not modelled. Opaque
//! nodes still expose `address` + `length` for byte provenance and
//! still flow through the decoder's `ControlFlow` projection, so CFG
//! edges remain intact (I-6).

use dac_arch::InstructionLifter;
use dac_ir::instr::{Condition, InstructionIr, Operand, Operation, Target};
use iced_x86::{
    ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, Mnemonic, OpKind, Register,
};

/// iced-x86–backed lifter.
///
/// `bitness` mirrors the decoder's: `16`, `32`, or `64`. Each ISA
/// backend constructs a lifter with the appropriate value; callers
/// never see it.
pub struct IcedLifter {
    bitness: u32,
}

impl IcedLifter {
    /// Construct a lifter for the given bitness. Panics in debug builds
    /// if `bitness` is not one of `16`, `32`, `64` — matching the
    /// decoder's contract.
    #[must_use]
    pub fn new(bitness: u32) -> Self {
        debug_assert!(
            matches!(bitness, 16 | 32 | 64),
            "iced-x86 only supports 16/32/64-bit; got {bitness}",
        );
        Self { bitness }
    }
}

impl InstructionLifter for IcedLifter {
    fn lift(&self, bytes: &[u8], address: u64) -> InstructionIr {
        if bytes.is_empty() {
            // The caller has nothing for us to decode. Emit an opaque
            // zero-length record so iterators that pair us with the
            // decoder still get a node back rather than a panic.
            return InstructionIr {
                address,
                length: 0,
                op: Operation::Opaque {
                    mnemonic: "(empty)".to_string(),
                },
            };
        }
        let mut decoder = Decoder::with_ip(self.bitness, bytes, address, DecoderOptions::NONE);
        let instr = decoder.decode();
        let length = u32::try_from(instr.len()).unwrap_or(0);
        let op = if instr.is_invalid() {
            Operation::Opaque {
                mnemonic: "(bad)".to_string(),
            }
        } else {
            lift_op(&instr)
        };
        InstructionIr {
            address,
            length,
            op,
        }
    }
}

/// Project a decoded iced [`Instruction`] into the arch-neutral
/// [`Operation`] vocabulary. The matching order is:
///
/// 1. **Syscalls** — `syscall`, `sysenter`, `sysexit`, `sysret`.
///    iced does not put these under `FlowControl::Interrupt`, so we
///    pre-match on mnemonic to avoid landing them in the data-op
///    fallthrough.
/// 2. **Control flow** — `flow_control()` is the single source of
///    truth, matching the decoder's `ControlFlow` projection so the
///    IR and the listing classify branches the same way.
/// 3. **Per-mnemonic data / arithmetic / stack ops**.
fn lift_op(instr: &Instruction) -> Operation {
    if let Some(syscall) = lift_syscall(instr) {
        return syscall;
    }
    if let Some(cf) = lift_control_flow(instr) {
        return cf;
    }
    lift_data_op(instr)
}

/// Catch the `syscall` family up-front. Returns `None` for anything
/// else so the regular control-flow / data-op pipeline runs.
fn lift_syscall(instr: &Instruction) -> Option<Operation> {
    matches!(
        instr.mnemonic(),
        Mnemonic::Syscall
            | Mnemonic::Sysenter
            | Mnemonic::Sysexit
            | Mnemonic::Sysretq
            | Mnemonic::Sysret
    )
    .then_some(Operation::Syscall)
}

/// Handle every instruction whose semantics are dominated by its
/// effect on control flow. Returns `None` for `FlowControl::Next` and
/// for the edge cases that need per-mnemonic disambiguation
/// (`Exception`, `XbeginXabortXend`).
fn lift_control_flow(instr: &Instruction) -> Option<Operation> {
    match instr.flow_control() {
        FlowControl::Next => None,
        FlowControl::ConditionalBranch => Some(Operation::Jump {
            target: branch_target(instr),
            condition: condition_from(instr),
        }),
        FlowControl::UnconditionalBranch => Some(Operation::Jump {
            target: branch_target(instr),
            condition: None,
        }),
        FlowControl::IndirectBranch => Some(Operation::Jump {
            target: Target::Indirect(operand(instr, 0)),
            condition: None,
        }),
        FlowControl::Call => Some(Operation::Call {
            target: branch_target(instr),
        }),
        FlowControl::IndirectCall => Some(Operation::Call {
            target: Target::Indirect(operand(instr, 0)),
        }),
        FlowControl::Return => Some(Operation::Return),
        FlowControl::Interrupt => Some(lift_interrupt(instr)),
        // Hardware transactional memory (xbegin/xabort/xend) and decoder-
        // recognised exception paths fall through to per-mnemonic
        // handling — they may still match `nop` (e.g. ENDBR64) or
        // legitimate opcodes the lifter models.
        FlowControl::XbeginXabortXend | FlowControl::Exception => None,
    }
}

/// Distinguish `int N`, `int3`, and `into`. (`syscall` and friends are
/// caught up-front by [`lift_syscall`].)
fn lift_interrupt(instr: &Instruction) -> Operation {
    match instr.mnemonic() {
        Mnemonic::Int3 => Operation::Interrupt { vector: Some(3) },
        Mnemonic::Into => Operation::Interrupt { vector: Some(4) },
        Mnemonic::Int1 => Operation::Interrupt { vector: Some(1) },
        Mnemonic::Int => {
            // `int N` carries the immediate in op 0.
            let vector = if instr.op_count() >= 1 {
                Some(instr.immediate8())
            } else {
                None
            };
            Operation::Interrupt { vector }
        }
        _ => Operation::Interrupt { vector: None },
    }
}

/// Per-mnemonic dispatch for non-control-flow instructions.
fn lift_data_op(instr: &Instruction) -> Operation {
    use Mnemonic as M;
    match instr.mnemonic() {
        M::Mov | M::Movzx | M::Movsx | M::Movsxd => Operation::Move {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Lea => Operation::LoadAddress {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Xchg => {
            // `xchg dst, src` swaps. The Move arm preserves enough of
            // the operand state for CFG / function-discovery; a
            // dedicated `Swap` op can land if a later pass needs it.
            Operation::Opaque {
                mnemonic: "xchg".to_string(),
            }
        }
        M::Add | M::Adc => Operation::Add {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Sub | M::Sbb => Operation::Sub {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Inc => Operation::Add {
            dst: operand(instr, 0),
            src: Operand::Immediate {
                value: 1,
                size_bits: 8,
            },
        },
        M::Dec => Operation::Sub {
            dst: operand(instr, 0),
            src: Operand::Immediate {
                value: 1,
                size_bits: 8,
            },
        },
        M::Mul | M::Imul => lift_mul(instr),
        M::Div | M::Idiv => Operation::Div {
            dst: implicit_a(instr),
            src: operand(instr, 0),
        },
        M::And => Operation::And {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Or => Operation::Or {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Xor => Operation::Xor {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Not => Operation::Not {
            dst: operand(instr, 0),
        },
        M::Neg => Operation::Neg {
            dst: operand(instr, 0),
        },
        M::Shl | M::Sal => Operation::Shl {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Shr => Operation::Shr {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Sar => Operation::Sar {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        M::Cmp => Operation::Compare {
            lhs: operand(instr, 0),
            rhs: operand(instr, 1),
        },
        M::Test => Operation::Test {
            lhs: operand(instr, 0),
            rhs: operand(instr, 1),
        },
        M::Push => Operation::Push {
            src: operand(instr, 0),
        },
        M::Pop => Operation::Pop {
            dst: operand(instr, 0),
        },
        M::Nop | M::Endbr32 | M::Endbr64 => Operation::Nop,
        _ => Operation::Opaque {
            mnemonic: mnemonic_name(instr.mnemonic()),
        },
    }
}

/// `imul` has three forms on x86 (1-, 2-, and 3-operand). The 1-op
/// form writes the implicit A register; the 2- and 3-op forms write
/// the first operand. We project all three onto `Mul { dst, src }`
/// with the convention "`dst` is the destination operand".
fn lift_mul(instr: &Instruction) -> Operation {
    match instr.op_count() {
        1 => Operation::Mul {
            dst: implicit_a(instr),
            src: operand(instr, 0),
        },
        2 => Operation::Mul {
            dst: operand(instr, 0),
            src: operand(instr, 1),
        },
        _ => Operation::Mul {
            dst: operand(instr, 0),
            // 3-op imul: dst = src1 * imm. The second source is the
            // immediate factor; the first source equals dst's reload
            // and is implicit in the dst operand at this layer.
            src: operand(instr, 2),
        },
    }
}

/// Project an iced operand into the arch-neutral [`Operand`].
fn operand(instr: &Instruction, idx: u32) -> Operand {
    if idx >= instr.op_count() {
        // Out-of-range request — return a zero-bit placeholder so
        // pattern-matching downstream stays total without panicking.
        return Operand::Immediate {
            value: 0,
            size_bits: 0,
        };
    }
    match instr.op_kind(idx) {
        OpKind::Register => {
            let r = instr.op_register(idx);
            register_operand(r)
        }
        OpKind::Immediate8 => {
            // 8-bit immediates sign-extend through `i8` for `add/sub`
            // and friends; unsigned interpretation is the consumer's
            // call.
            Operand::Immediate {
                value: i64::from(instr.immediate8() as i8),
                size_bits: 8,
            }
        }
        OpKind::Immediate8_2nd => Operand::Immediate {
            value: i64::from(instr.immediate8_2nd() as i8),
            size_bits: 8,
        },
        OpKind::Immediate16 => Operand::Immediate {
            value: i64::from(instr.immediate16() as i16),
            size_bits: 16,
        },
        OpKind::Immediate32 => Operand::Immediate {
            value: i64::from(instr.immediate32() as i32),
            size_bits: 32,
        },
        OpKind::Immediate64 => Operand::Immediate {
            value: instr.immediate64() as i64,
            size_bits: 64,
        },
        OpKind::Immediate8to16 => Operand::Immediate {
            value: i64::from(instr.immediate8to16()),
            size_bits: 16,
        },
        OpKind::Immediate8to32 => Operand::Immediate {
            value: i64::from(instr.immediate8to32()),
            size_bits: 32,
        },
        OpKind::Immediate8to64 => Operand::Immediate {
            value: instr.immediate8to64(),
            size_bits: 64,
        },
        OpKind::Immediate32to64 => Operand::Immediate {
            value: instr.immediate32to64(),
            size_bits: 64,
        },
        OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64 => Operand::Branch {
            target: instr.near_branch_target(),
        },
        OpKind::FarBranch16 | OpKind::FarBranch32 => Operand::Branch {
            target: u64::from(instr.far_branch32()),
        },
        OpKind::Memory => memory_operand(instr),
        // The few remaining OpKind variants describe operand classes
        // for AVX-512 mask registers, SIB-encoded addresses with no
        // base, and a couple of esoterica. They land as memory with
        // best-effort fields; the lifter does not synthesize
        // semantics it cannot defend.
        OpKind::MemorySegSI
        | OpKind::MemorySegESI
        | OpKind::MemorySegRSI
        | OpKind::MemorySegDI
        | OpKind::MemorySegEDI
        | OpKind::MemorySegRDI
        | OpKind::MemoryESDI
        | OpKind::MemoryESEDI
        | OpKind::MemoryESRDI => memory_operand(instr),
    }
}

fn register_operand(r: Register) -> Operand {
    Operand::Register {
        name: reg_name(r),
        // `Register::size()` returns bytes; promote to bits for the
        // IR. Returns `0` for `Register::None`, which we keep so the
        // caller can spot junk operands rather than silently rewrite
        // them.
        size_bits: (r.size() as u16) * 8,
    }
}

fn memory_operand(instr: &Instruction) -> Operand {
    // RIP-relative addressing is a special case: iced's
    // `memory_displacement64` returns the already-resolved absolute
    // target VA (not the raw displacement), and `memory_base` reports
    // `Register::RIP`. The lift bridge would otherwise emit
    // `rip_var + absolute_va`, which is semantically wrong and hides
    // the constant from downstream constant-folding (B3.17: switch
    // table resolution needs to see the table base as a concrete
    // value, not as a sum involving an SSA parameter for RIP).
    // Drop the base for RIP-relative operands and let the displacement
    // carry the absolute VA on its own.
    let is_rip_relative = instr.is_ip_rel_memory_operand();
    let base = if is_rip_relative {
        None
    } else {
        nonzero_register(instr.memory_base())
    };
    let index = nonzero_register(instr.memory_index());
    Operand::Memory {
        base,
        index,
        scale: instr.memory_index_scale() as u8,
        // `memory_displacement64` returns the displacement already
        // sign-extended to 64 bits. Cast through `i64` so negative
        // displacements (`[rbp-0x10]`) survive the trip intact. For
        // RIP-relative this is the resolved absolute target.
        displacement: instr.memory_displacement64() as i64,
        size_bits: (instr.memory_size().size() as u16) * 8,
        segment: nonzero_register(instr.memory_segment()),
    }
}

fn nonzero_register(r: Register) -> Option<String> {
    if r == Register::None {
        None
    } else {
        Some(reg_name(r))
    }
}

/// Lowercase the iced `Debug` variant name. iced does not expose a
/// stable `name()`-style accessor on `Register`, but its `Debug` impl
/// emits the canonical mnemonic name (`RAX`, `R8D`, `XMM0`).
fn reg_name(r: Register) -> String {
    format!("{r:?}").to_lowercase()
}

/// Lowercase mnemonic name for the opaque fallback. We strip iced's
/// `Mnemonic::` prefix by using the variant's `Debug` form, same as
/// for registers.
fn mnemonic_name(m: Mnemonic) -> String {
    format!("{m:?}").to_lowercase()
}

/// Direct branch target as a [`Target::Direct`]. Falls back to
/// `Target::Indirect` with the first operand if iced did not classify
/// the branch as direct — keeps the IR honest for hand-crafted
/// edge cases.
fn branch_target(instr: &Instruction) -> Target {
    for i in 0..instr.op_count() {
        match instr.op_kind(i) {
            OpKind::NearBranch16 | OpKind::NearBranch32 | OpKind::NearBranch64 => {
                return Target::Direct(instr.near_branch_target());
            }
            OpKind::FarBranch16 | OpKind::FarBranch32 => {
                return Target::Direct(u64::from(instr.far_branch32()));
            }
            _ => {}
        }
    }
    Target::Indirect(operand(instr, 0))
}

/// Map iced's `ConditionCode` onto the arch-neutral [`Condition`].
fn condition_from(instr: &Instruction) -> Option<Condition> {
    // `JCXZ`/`JECXZ`/`JRCXZ` are not modelled by `ConditionCode` — they
    // surface as separate mnemonics in iced.
    match instr.mnemonic() {
        Mnemonic::Jcxz | Mnemonic::Jecxz | Mnemonic::Jrcxz => return Some(Condition::CxZero),
        _ => {}
    }
    match instr.condition_code() {
        ConditionCode::None => None,
        ConditionCode::o => Some(Condition::Overflow),
        ConditionCode::no => Some(Condition::NotOverflow),
        ConditionCode::b => Some(Condition::Below),
        ConditionCode::ae => Some(Condition::AboveEqual),
        ConditionCode::e => Some(Condition::Equal),
        ConditionCode::ne => Some(Condition::NotEqual),
        ConditionCode::be => Some(Condition::BelowEqual),
        ConditionCode::a => Some(Condition::Above),
        ConditionCode::s => Some(Condition::Sign),
        ConditionCode::ns => Some(Condition::NotSign),
        ConditionCode::p => Some(Condition::Parity),
        ConditionCode::np => Some(Condition::NotParity),
        ConditionCode::l => Some(Condition::Less),
        ConditionCode::ge => Some(Condition::GreaterEqual),
        ConditionCode::le => Some(Condition::LessEqual),
        ConditionCode::g => Some(Condition::Greater),
    }
}

/// The implicit accumulator operand for `mul`, `imul` (1-op), `div`,
/// `idiv`. Width follows the explicit operand: `r/m8 -> al`, `r/m16
/// -> ax`, `r/m32 -> eax`, `r/m64 -> rax`.
fn implicit_a(instr: &Instruction) -> Operand {
    // Use the explicit operand's size to pick the right `a`-register.
    let bits = explicit_operand_bits(instr).unwrap_or(64);
    let (name, size_bits) = match bits {
        8 => ("al", 8),
        16 => ("ax", 16),
        32 => ("eax", 32),
        _ => ("rax", 64),
    };
    Operand::Register {
        name: name.to_string(),
        size_bits,
    }
}

fn explicit_operand_bits(instr: &Instruction) -> Option<u16> {
    if instr.op_count() == 0 {
        return None;
    }
    match instr.op_kind(0) {
        OpKind::Register => {
            let r = instr.op_register(0);
            Some((r.size() as u16) * 8)
        }
        OpKind::Memory => Some((instr.memory_size().size() as u16) * 8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_arch::Architecture;

    fn lift_one(bitness: u32, bytes: &[u8], address: u64) -> InstructionIr {
        IcedLifter::new(bitness).lift(bytes, address)
    }

    #[test]
    fn lifts_mov_rax_rbx() {
        // 48 89 D8 = mov rax, rbx
        let ir = lift_one(64, &[0x48, 0x89, 0xD8], 0x1000);
        assert_eq!(ir.address, 0x1000);
        assert_eq!(ir.length, 3);
        match ir.op {
            Operation::Move {
                dst:
                    Operand::Register {
                        name: dn,
                        size_bits: ds,
                    },
                src:
                    Operand::Register {
                        name: sn,
                        size_bits: ss,
                    },
            } => {
                assert_eq!(dn, "rax");
                assert_eq!(ds, 64);
                assert_eq!(sn, "rbx");
                assert_eq!(ss, 64);
            }
            other => panic!("expected Move(rax, rbx), got {other:?}"),
        }
    }

    #[test]
    fn lifts_lea_with_memory_operand() {
        // 48 8D 44 24 10 = lea rax, [rsp+0x10]
        // Using rsp+disp rather than rip+disp because for RIP-relative
        // addressing iced's `memory_displacement64` returns the
        // resolved absolute address rather than the raw displacement,
        // so the displacement assertion would need to know the base
        // VA. The structural test (base register + displacement) is
        // what we care about here.
        let ir = lift_one(64, &[0x48, 0x8D, 0x44, 0x24, 0x10], 0);
        match ir.op {
            Operation::LoadAddress {
                dst: Operand::Register { name, .. },
                src:
                    Operand::Memory {
                        base,
                        displacement,
                        index,
                        ..
                    },
            } => {
                assert_eq!(name, "rax");
                assert_eq!(base.as_deref(), Some("rsp"));
                assert_eq!(displacement, 0x10);
                assert!(index.is_none());
            }
            other => panic!("expected LoadAddress(rax, [rsp+0x10]), got {other:?}"),
        }
    }

    #[test]
    fn lifts_rip_relative_lea_as_constant_displacement() {
        // 48 8D 05 04 00 00 00 = lea rax, [rip + 4] at 0x1000.
        // iced resolves the target to 0x1000 + 7 (instruction length) +
        // 4 = 0x100B. The lifter must surface that as a constant
        // displacement with no base register so downstream constant
        // folding (B3.17 switch-table resolution) sees the concrete VA.
        let ir = lift_one(64, &[0x48, 0x8D, 0x05, 0x04, 0x00, 0x00, 0x00], 0x1000);
        match ir.op {
            Operation::LoadAddress {
                dst: Operand::Register { name, .. },
                src:
                    Operand::Memory {
                        base,
                        displacement,
                        index,
                        ..
                    },
            } => {
                assert_eq!(name, "rax");
                assert!(
                    base.is_none(),
                    "RIP-relative addressing must drop the base register, got base={base:?}"
                );
                assert_eq!(displacement, 0x100B);
                assert!(index.is_none());
            }
            other => panic!("expected LoadAddress(rax, [Const(0x100B)]), got {other:?}"),
        }
    }

    #[test]
    fn lifts_call_with_direct_target() {
        // E8 05 00 00 00 = call +5 from 0x1000 -> 0x100A
        let ir = lift_one(64, &[0xE8, 0x05, 0x00, 0x00, 0x00], 0x1000);
        match ir.op {
            Operation::Call {
                target: Target::Direct(t),
            } => assert_eq!(t, 0x100A),
            other => panic!("expected Call(Direct(0x100A)), got {other:?}"),
        }
    }

    #[test]
    fn lifts_indirect_call() {
        // FF D0 = call rax
        let ir = lift_one(64, &[0xFF, 0xD0], 0x3000);
        match ir.op {
            Operation::Call {
                target: Target::Indirect(Operand::Register { name, size_bits }),
            } => {
                assert_eq!(name, "rax");
                assert_eq!(size_bits, 64);
            }
            other => panic!("expected Call(Indirect(rax)), got {other:?}"),
        }
    }

    #[test]
    fn lifts_conditional_branch_with_signed_codes() {
        // 7C 04 = jl +4 from 0x4000 -> 0x4006
        let ir = lift_one(64, &[0x7C, 0x04], 0x4000);
        match ir.op {
            Operation::Jump {
                target: Target::Direct(t),
                condition: Some(Condition::Less),
            } => assert_eq!(t, 0x4006),
            other => panic!("expected Jump(Direct(0x4006), Less), got {other:?}"),
        }
    }

    #[test]
    fn lifts_unconditional_branch() {
        // EB 02 = jmp +2 from 0x5000 -> 0x5004
        let ir = lift_one(64, &[0xEB, 0x02], 0x5000);
        match ir.op {
            Operation::Jump {
                target: Target::Direct(t),
                condition: None,
            } => assert_eq!(t, 0x5004),
            other => panic!("expected Jump(Direct(0x5004), None), got {other:?}"),
        }
    }

    #[test]
    fn lifts_ret() {
        let ir = lift_one(64, &[0xC3], 0);
        assert!(matches!(ir.op, Operation::Return));
        assert_eq!(ir.length, 1);
    }

    #[test]
    fn lifts_push_pop() {
        let push = lift_one(64, &[0x50], 0);
        match push.op {
            Operation::Push {
                src: Operand::Register { name, .. },
            } => assert_eq!(name, "rax"),
            other => panic!("expected Push(rax), got {other:?}"),
        }
        let pop = lift_one(64, &[0x5D], 0);
        match pop.op {
            Operation::Pop {
                dst: Operand::Register { name, .. },
            } => assert_eq!(name, "rbp"),
            other => panic!("expected Pop(rbp), got {other:?}"),
        }
    }

    #[test]
    fn lifts_add_with_immediate() {
        // 48 83 C0 05 = add rax, 5
        let ir = lift_one(64, &[0x48, 0x83, 0xC0, 0x05], 0);
        match ir.op {
            Operation::Add {
                dst: Operand::Register { name, .. },
                src: Operand::Immediate { value, .. },
            } => {
                assert_eq!(name, "rax");
                assert_eq!(value, 5);
            }
            other => panic!("expected Add(rax, 5), got {other:?}"),
        }
    }

    #[test]
    fn lifts_xor_self() {
        // 31 C0 = xor eax, eax
        let ir = lift_one(64, &[0x31, 0xC0], 0);
        match ir.op {
            Operation::Xor {
                dst: Operand::Register { name: dn, .. },
                src: Operand::Register { name: sn, .. },
            } => {
                assert_eq!(dn, "eax");
                assert_eq!(sn, "eax");
            }
            other => panic!("expected Xor(eax, eax), got {other:?}"),
        }
    }

    #[test]
    fn lifts_inc_as_add_one() {
        // FF C0 = inc eax
        let ir = lift_one(64, &[0xFF, 0xC0], 0);
        match ir.op {
            Operation::Add {
                dst: Operand::Register { name, .. },
                src: Operand::Immediate { value: 1, .. },
            } => assert_eq!(name, "eax"),
            other => panic!("expected Add(eax, 1), got {other:?}"),
        }
    }

    #[test]
    fn lifts_cmp_then_jne_sequence() {
        // 48 39 D8 = cmp rax, rbx ; 75 02 = jne +2
        let bytes = [0x48, 0x39, 0xD8, 0x75, 0x02];
        let cmp = IcedLifter::new(64).lift(&bytes[..3], 0x1000);
        let jne = IcedLifter::new(64).lift(&bytes[3..], 0x1003);
        match cmp.op {
            Operation::Compare {
                lhs: Operand::Register { name: lhs, .. },
                rhs: Operand::Register { name: rhs, .. },
            } => {
                assert_eq!(lhs, "rax");
                assert_eq!(rhs, "rbx");
            }
            other => panic!("expected Compare(rax, rbx), got {other:?}"),
        }
        match jne.op {
            Operation::Jump {
                target: Target::Direct(t),
                condition: Some(Condition::NotEqual),
            } => assert_eq!(t, 0x1007),
            other => panic!("expected Jump(Direct(0x1007), NotEqual), got {other:?}"),
        }
    }

    #[test]
    fn lifts_syscall() {
        // 0F 05 = syscall
        let ir = lift_one(64, &[0x0F, 0x05], 0);
        assert!(matches!(ir.op, Operation::Syscall));
    }

    #[test]
    fn lifts_int3_with_known_vector() {
        let ir = lift_one(64, &[0xCC], 0);
        match ir.op {
            Operation::Interrupt { vector: Some(3) } => {}
            other => panic!("expected Interrupt(3), got {other:?}"),
        }
    }

    #[test]
    fn lifts_nop_and_endbr64_alike() {
        let nop = lift_one(64, &[0x90], 0);
        assert!(matches!(nop.op, Operation::Nop));
        // F3 0F 1E FA = endbr64
        let endbr = lift_one(64, &[0xF3, 0x0F, 0x1E, 0xFA], 0);
        assert!(matches!(endbr.op, Operation::Nop));
    }

    #[test]
    fn invalid_bytes_lift_to_opaque_bad() {
        // 0x06 is invalid in 64-bit mode.
        let ir = lift_one(64, &[0x06], 0);
        match ir.op {
            Operation::Opaque { mnemonic } => assert_eq!(mnemonic, "(bad)"),
            other => panic!("expected Opaque(bad), got {other:?}"),
        }
    }

    #[test]
    fn empty_buffer_lifts_to_opaque_empty() {
        let ir = lift_one(64, &[], 0x9000);
        assert_eq!(ir.length, 0);
        assert_eq!(ir.address, 0x9000);
        match ir.op {
            Operation::Opaque { mnemonic } => assert_eq!(mnemonic, "(empty)"),
            other => panic!("expected Opaque(empty), got {other:?}"),
        }
    }

    #[test]
    fn unmodelled_opcode_falls_through_to_opaque() {
        // F3 0F 58 C1 = addss xmm0, xmm1 — SSE single-precision float
        // add, deliberately outside the B1.4 subset.
        let ir = lift_one(64, &[0xF3, 0x0F, 0x58, 0xC1], 0);
        match ir.op {
            Operation::Opaque { mnemonic } => assert!(mnemonic.contains("addss")),
            other => panic!("expected Opaque(addss), got {other:?}"),
        }
    }

    #[test]
    fn architecture_lifter_returns_working_instance() {
        let arch = crate::X86_64;
        let lifter = arch.lifter();
        let ir = lifter.lift(&[0xC3], 0);
        assert!(matches!(ir.op, Operation::Return));
    }
}
