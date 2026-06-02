//! Instruction IR → RawFunction bridge (B3.8, FR-8, FR-11).
//!
//! [`lift_function`] consumes a [`Cfg`] together with the per-block
//! [`InstructionIr`] streams produced by the architecture lifter and
//! produces the [`RawFunction`] that
//! [`dac_analysis::ssa::construct_ssa`] is documented to take. Until
//! this batch landed, the bridge lived only inside the dac-analysis SSA
//! tests — every B2.x / B3.x downstream pass existed but never fired on
//! real bytes because nothing translated the lifter's output into the
//! SSA constructor's input.
//!
//! ## What this batch covers
//!
//! - Register variable model. One [`VariableId`] per *canonical*
//!   register: sub-register operands (`eax`, `ax`, `al`) are
//!   normalised through [`RegisterFile::parent`] back to the 64-bit
//!   GPR (`rax`).
//! - Arithmetic / data-movement / stack-manipulation translation:
//!   `Move`, `Add`, `Sub`, `Mul`, `And`, `Or`, `Xor`, `Shl`, `Shr`,
//!   `Sar` (lossy → `Shr`), `Neg`, `Not`, `LoadAddress`, `Push`,
//!   `Pop` all land on the matching [`RawOpKind`].
//! - Memory operand expansion. `[base + index*scale + disp]`
//!   addressing modes expand inline into a chain of synthetic
//!   `Add` / `Mul` ops that drive `Load` / `Store` raw ops.
//! - `Compare` / `Test` are *stashed*, not emitted: the next
//!   [`Operation::Jump`] consumes the pending flag setter, mints a
//!   [`RawOpKind::Compare`] with the Jcc-derived [`CompareKind`], and
//!   wires the resulting value into [`RawTerminator::Branch`].
//! - `Return` reads the return register (`rax` on x86-64) and lands
//!   on [`RawTerminator::Return`].
//! - `Call` translates as [`RawOpKind::Call`] with the resolved
//!   target VA, conservatively reads every SysV argument register
//!   (`rdi`–`r9`) so liveness analysis stays sound, and conservatively
//!   *defines* `rax` so callees that return a value get a fresh SSA
//!   name there.
//! - `Opaque`, `Interrupt`, `Syscall`, `Div`, `Nop` and decoder-invalid
//!   instructions degrade honestly: either dropped (`Nop`) or wrapped
//!   in [`RawOpKind::Opaque`] so the SSA constructor still sees a
//!   side-effect node (I-6).
//!
//! ## What this batch deliberately doesn't cover
//!
//! - **Subreg-aliasing precision.** A 32-bit write under x86-64 zeroes
//!   the upper 32 of the 64-bit parent; a 16/8-bit write preserves
//!   them. We treat every sub-register write as a full 64-bit write
//!   under the canonical variable id. The known-loss is documented at
//!   the call site and is the first follow-up listed in the PLAN.md
//!   "B3 follow-up shelf".
//! - **Stack-slot lowering.** Memory accesses through `rsp` / `rbp`
//!   land as ordinary `Load` / `Store` ops with synthetic
//!   address-compute temporaries. The B2.4 stack-frame pass runs
//!   *after* SSA construction (it reads the SSA function), so
//!   detecting stack slots before SSA isn't this batch's job.
//! - **Architecture other than x86-64.** The bridge takes a generic
//!   [`RegisterFile`] for canonicalisation, but the return register
//!   and call-argument register list are hard-coded to System V
//!   AMD64. AArch64 lands with a parameterised convention table in
//!   B5.2 / a later refinement.
//!
//! ## Determinism (NFR-9)
//!
//! `lift_function` is `Determinism::Pure`. Variable ids are minted in
//! first-encounter order driven by the input's ascending block-id and
//! source-order iteration. Synthetic temporaries are minted by a
//! monotonic counter. Same input → same `RawFunction`, always.

use std::collections::BTreeMap;

use dac_analysis::cfg::{BasicBlock, Cfg, EdgeKind, Terminator};
use dac_analysis::ssa::{RawBlock, RawFunction, RawOp, RawOpKind, RawOperand, RawTerminator};
use dac_arch::RegisterFile;
use dac_ir::instr::{Condition, InstructionIr, Operand, Operation, Target};
use dac_ir::ssa::{CompareKind, Variable, VariableId};

/// SysV AMD64 integer return register.
const RETURN_REGISTER_X86_64: &str = "rax";

/// SysV AMD64 integer argument register sequence. Read at every call
/// site so liveness analysis sees the conservative caller-clobber set;
/// B3.10's argument-count inference narrows this when it lands.
const CALL_ARG_REGISTERS_X86_64: &[&str] = &["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

/// Lift a CFG plus its per-block [`InstructionIr`] streams into a
/// [`RawFunction`] ready for [`dac_analysis::ssa::construct_ssa`].
///
/// `instructions_per_block[i]` must list the lifted instructions for
/// `cfg.blocks[i]` in source order. The caller is responsible for the
/// per-instruction lift (typically by pairing each
/// [`BasicBlock::instructions`] entry with
/// [`dac_arch::InstructionLifter::lift`]). Block indices are preserved
/// — `raw.blocks[i]` is the SSA-constructor input for the CFG block
/// whose id is `i`.
///
/// `register_file` is consulted for sub-register canonicalisation; on
/// x86-64 it is the file returned by
/// [`dac_arch_x86::X86_64::register_file`].
///
/// # Panics
///
/// Panics if `instructions_per_block.len() != cfg.blocks.len()`. Every
/// other failure mode degrades to honest IR shapes
/// ([`RawOpKind::Opaque`], [`RawTerminator::Indirect`],
/// [`RawTerminator::Unreachable`]) rather than panicking, per I-6.
#[must_use]
pub fn lift_function(
    cfg: &Cfg,
    instructions_per_block: &[Vec<InstructionIr>],
    register_file: &RegisterFile,
) -> RawFunction {
    assert_eq!(
        cfg.blocks.len(),
        instructions_per_block.len(),
        "lift_function: instructions_per_block must mirror cfg.blocks",
    );

    let mut builder = Builder::new(register_file);
    let blocks: Vec<RawBlock> = cfg
        .blocks
        .iter()
        .enumerate()
        .map(|(i, cfg_block)| builder.translate_block(cfg, cfg_block, &instructions_per_block[i]))
        .collect();

    RawFunction {
        variables: builder.variables,
        blocks,
    }
}

/// Translation state. Holds the variable table, the canonical-name
/// cache, and the pending flag setter from the previous instruction in
/// the block currently being walked.
struct Builder<'rf> {
    register_file: &'rf RegisterFile,
    variables: Vec<Variable>,
    by_name: BTreeMap<String, VariableId>,
    pending: Option<PendingFlag>,
    synth_counter: u32,
}

/// One arm of the `Compare` / `Test` → `Jcc` collapse: the most recent
/// flag-setting instruction in the current block, retained until the
/// terminator decides what to do with it.
#[derive(Debug, Clone, Copy)]
enum PendingFlag {
    Compare { lhs: RawOperand, rhs: RawOperand },
    Test { lhs: RawOperand, rhs: RawOperand },
}

impl<'rf> Builder<'rf> {
    fn new(register_file: &'rf RegisterFile) -> Self {
        Self {
            register_file,
            variables: Vec::new(),
            by_name: BTreeMap::new(),
            pending: None,
            synth_counter: 0,
        }
    }

    /// Variable id for `name`, canonicalised through the
    /// register-file's parent chain. Mints a fresh entry on first
    /// encounter; subsequent encounters return the same id.
    fn var_for_register(&mut self, name: &str) -> VariableId {
        let (canon_name, width) = self.canonical_register(name);
        if let Some(&id) = self.by_name.get(&canon_name) {
            return id;
        }
        let id = self.variables.len() as VariableId;
        self.variables.push(Variable {
            id,
            name: canon_name.clone(),
            width_bits: width,
        });
        self.by_name.insert(canon_name, id);
        id
    }

    fn canonical_register(&self, name: &str) -> (String, u16) {
        match self.register_file.by_name(name) {
            Some(r) => match r.parent {
                Some(parent_id) => match self.register_file.register(parent_id) {
                    Some(parent) => (parent.name.to_string(), parent.size_bits),
                    None => (r.name.to_string(), r.size_bits),
                },
                None => (r.name.to_string(), r.size_bits),
            },
            None => (name.to_string(), 0),
        }
    }

    /// Mint a fresh synthetic variable (address temps, compare
    /// results). Names are deterministic ("t0", "t1", …).
    fn synth_temp(&mut self, width_bits: u16) -> VariableId {
        let id = self.variables.len() as VariableId;
        let name = format!("t{}", self.synth_counter);
        self.synth_counter += 1;
        self.variables.push(Variable {
            id,
            name: name.clone(),
            width_bits,
        });
        self.by_name.insert(name, id);
        id
    }

    fn translate_block(
        &mut self,
        cfg: &Cfg,
        cfg_block: &BasicBlock,
        instrs: &[InstructionIr],
    ) -> RawBlock {
        self.pending = None;
        let mut ops: Vec<RawOp> = Vec::new();

        // Decide how many leading instructions to translate as body
        // ops. The trailing terminator-style instruction (Jump,
        // Return, Indirect, Interrupt) is consumed when we synthesise
        // the RawTerminator; a Call terminator is *body* (it falls
        // through), so all instructions translate.
        let body_len = match cfg_block.terminator {
            Terminator::Fall | Terminator::Call { .. } => instrs.len(),
            Terminator::Invalid => 0,
            Terminator::Branch { .. }
            | Terminator::Conditional { .. }
            | Terminator::Return
            | Terminator::Indirect
            | Terminator::Interrupt => instrs.len().saturating_sub(1),
        };
        for instr in instrs.iter().take(body_len) {
            self.translate_op(&instr.op, &mut ops);
        }

        let terminator = self.translate_terminator(cfg, cfg_block, instrs, &mut ops);
        RawBlock { ops, terminator }
    }

    fn translate_terminator(
        &mut self,
        cfg: &Cfg,
        cfg_block: &BasicBlock,
        instrs: &[InstructionIr],
        ops: &mut Vec<RawOp>,
    ) -> RawTerminator {
        match cfg_block.terminator {
            Terminator::Fall => self
                .first_successor(cfg, cfg_block.id, EdgeKind::Fall)
                .map_or(RawTerminator::Indirect, |target| RawTerminator::Jump {
                    target,
                }),
            Terminator::Branch { .. } => self
                .first_successor(cfg, cfg_block.id, EdgeKind::Branch)
                .map_or(RawTerminator::Indirect, |target| RawTerminator::Jump {
                    target,
                }),
            Terminator::Conditional { .. } => {
                let taken = self.first_successor(cfg, cfg_block.id, EdgeKind::Taken);
                let not_taken = self.first_successor(cfg, cfg_block.id, EdgeKind::NotTaken);
                let condition = instrs.last().and_then(|i| match &i.op {
                    Operation::Jump {
                        condition: Some(c), ..
                    } => condition_to_compare_kind(*c),
                    _ => None,
                });
                match (taken, not_taken, condition, self.pending.take()) {
                    (Some(t), Some(nt), Some(kind), Some(flag)) => {
                        let cond = self.emit_compare(kind, flag, ops);
                        RawTerminator::Branch {
                            cond,
                            taken: t,
                            not_taken: nt,
                        }
                    }
                    _ => RawTerminator::Indirect,
                }
            }
            Terminator::Return => {
                let ret_var = self.var_for_register(RETURN_REGISTER_X86_64);
                RawTerminator::Return {
                    value: Some(RawOperand::Variable(ret_var)),
                }
            }
            Terminator::Call { .. } => self
                .first_successor(cfg, cfg_block.id, EdgeKind::Fall)
                .map_or(RawTerminator::Indirect, |target| RawTerminator::Jump {
                    target,
                }),
            Terminator::Indirect => RawTerminator::Indirect,
            Terminator::Interrupt | Terminator::Invalid => RawTerminator::Unreachable,
        }
    }

    /// Look up the first successor of `block_id` whose `EdgeKind`
    /// matches `kind`. The CFG sorts edges by `(from, kind, to)`, so
    /// "first" is a stable choice.
    fn first_successor(&self, cfg: &Cfg, block_id: u32, kind: EdgeKind) -> Option<u32> {
        cfg.edges
            .iter()
            .find(|e| e.from == block_id && e.kind == kind)
            .map(|e| e.to)
    }

    fn emit_compare(
        &mut self,
        kind: CompareKind,
        flag: PendingFlag,
        ops: &mut Vec<RawOp>,
    ) -> RawOperand {
        let (lhs, rhs) = match flag {
            PendingFlag::Compare { lhs, rhs } => (lhs, rhs),
            PendingFlag::Test { lhs, rhs } => {
                let and_tmp = self.synth_temp(64);
                ops.push(RawOp {
                    dst: Some(and_tmp),
                    kind: RawOpKind::And { lhs, rhs },
                });
                (RawOperand::Variable(and_tmp), RawOperand::Const(0))
            }
        };
        let cmp_tmp = self.synth_temp(8);
        ops.push(RawOp {
            dst: Some(cmp_tmp),
            kind: RawOpKind::Compare { kind, lhs, rhs },
        });
        RawOperand::Variable(cmp_tmp)
    }

    fn translate_op(&mut self, op: &Operation, ops: &mut Vec<RawOp>) {
        match op {
            Operation::Move { dst, src } => {
                let src_val = self.translate_read(src, ops);
                self.write_register_or_store(dst, src_val, ops);
                self.pending = None;
            }
            Operation::LoadAddress { dst, src } => {
                let (addr_op, _) = self.translate_memory_address(src, ops);
                self.write_register_or_store(dst, addr_op, ops);
                self.pending = None;
            }
            Operation::Add { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Add, ops);
                self.pending = None;
            }
            Operation::Sub { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Sub, ops);
                self.pending = None;
            }
            Operation::Mul { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Mul, ops);
                self.pending = None;
            }
            Operation::And { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::And, ops);
                self.pending = None;
            }
            Operation::Or { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Or, ops);
                self.pending = None;
            }
            Operation::Xor { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Xor, ops);
                self.pending = None;
            }
            Operation::Shl { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Shl, ops);
                self.pending = None;
            }
            // B3.8 known-loss: Sar (arithmetic) and Shr (logical) collapse
            // onto RawOpKind::Shr. The signed-shift distinction is a
            // follow-up.
            Operation::Shr { dst, src } | Operation::Sar { dst, src } => {
                self.translate_binop_inplace(dst, src, BinaryKind::Shr, ops);
                self.pending = None;
            }
            Operation::Neg { dst } => {
                self.translate_unop_inplace(dst, UnaryKind::Neg, ops);
                self.pending = None;
            }
            Operation::Not { dst } => {
                self.translate_unop_inplace(dst, UnaryKind::Not, ops);
                self.pending = None;
            }
            Operation::Compare { lhs, rhs } => {
                let l = self.translate_read(lhs, ops);
                let r = self.translate_read(rhs, ops);
                self.pending = Some(PendingFlag::Compare { lhs: l, rhs: r });
            }
            Operation::Test { lhs, rhs } => {
                let l = self.translate_read(lhs, ops);
                let r = self.translate_read(rhs, ops);
                self.pending = Some(PendingFlag::Test { lhs: l, rhs: r });
            }
            Operation::Push { src } => {
                let value = self.translate_read(src, ops);
                let rsp = self.var_for_register("rsp");
                ops.push(RawOp {
                    dst: Some(rsp),
                    kind: RawOpKind::Sub {
                        lhs: RawOperand::Variable(rsp),
                        rhs: RawOperand::Const(8),
                    },
                });
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Store {
                        address: RawOperand::Variable(rsp),
                        value,
                        width: 8,
                    },
                });
                self.pending = None;
            }
            Operation::Pop { dst } => {
                let rsp = self.var_for_register("rsp");
                let loaded = self.synth_temp(64);
                ops.push(RawOp {
                    dst: Some(loaded),
                    kind: RawOpKind::Load {
                        address: RawOperand::Variable(rsp),
                        width: 8,
                    },
                });
                ops.push(RawOp {
                    dst: Some(rsp),
                    kind: RawOpKind::Add {
                        lhs: RawOperand::Variable(rsp),
                        rhs: RawOperand::Const(8),
                    },
                });
                self.write_register_or_store(dst, RawOperand::Variable(loaded), ops);
                self.pending = None;
            }
            Operation::Call { target } => {
                let resolved = match target {
                    Target::Direct(addr) => Some(*addr),
                    Target::Indirect(_) => None,
                };
                let args: Vec<RawOperand> = CALL_ARG_REGISTERS_X86_64
                    .iter()
                    .map(|name| RawOperand::Variable(self.var_for_register(name)))
                    .collect();
                let rax_var = self.var_for_register(RETURN_REGISTER_X86_64);
                ops.push(RawOp {
                    dst: Some(rax_var),
                    kind: RawOpKind::Call {
                        target: resolved,
                        args,
                    },
                });
                self.pending = None;
            }
            // The terminator-shaped Operations only appear in the
            // last slot of a block, and `translate_block` truncates
            // them before reaching `translate_op`. If one slips in
            // through some other path (mid-block return, fused
            // syscall), surface it as Opaque so the SSA still has a
            // side-effect node.
            Operation::Return | Operation::Jump { .. } => {
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Opaque {
                        mnemonic: opcode_label(op).to_string(),
                        args: Vec::new(),
                    },
                });
                self.pending = None;
            }
            Operation::Nop => {
                // Drop. Nop has no SSA effect and CSE would remove it
                // immediately anyway.
            }
            Operation::Interrupt { .. } | Operation::Syscall | Operation::Div { .. } => {
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Opaque {
                        mnemonic: opcode_label(op).to_string(),
                        args: Vec::new(),
                    },
                });
                self.pending = None;
            }
            Operation::Opaque { mnemonic } => {
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Opaque {
                        mnemonic: mnemonic.clone(),
                        args: Vec::new(),
                    },
                });
                self.pending = None;
            }
        }
    }

    /// Read an Operand into a [`RawOperand`]. Memory operands expand
    /// into address-compute ops followed by a [`RawOpKind::Load`].
    fn translate_read(&mut self, operand: &Operand, ops: &mut Vec<RawOp>) -> RawOperand {
        match operand {
            Operand::Register { name, .. } => RawOperand::Variable(self.var_for_register(name)),
            Operand::Immediate { value, .. } => RawOperand::Const(*value),
            Operand::Memory { .. } => {
                let (addr_op, width) = self.translate_memory_address(operand, ops);
                let loaded = self.synth_temp(u16::from(width) * 8);
                ops.push(RawOp {
                    dst: Some(loaded),
                    kind: RawOpKind::Load {
                        address: addr_op,
                        width,
                    },
                });
                RawOperand::Variable(loaded)
            }
            Operand::Branch { target } => RawOperand::Const(*target as i64),
        }
    }

    /// Write the produced value into a destination operand. Register
    /// destinations land as `Move`; memory destinations land as
    /// `Store` preceded by address-compute ops.
    fn write_register_or_store(&mut self, dst: &Operand, value: RawOperand, ops: &mut Vec<RawOp>) {
        match dst {
            Operand::Register { name, .. } => {
                let dst_var = self.var_for_register(name);
                ops.push(RawOp {
                    dst: Some(dst_var),
                    kind: RawOpKind::Move { src: value },
                });
            }
            Operand::Memory { .. } => {
                let (addr_op, width) = self.translate_memory_address(dst, ops);
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Store {
                        address: addr_op,
                        value,
                        width,
                    },
                });
            }
            Operand::Immediate { .. } | Operand::Branch { .. } => {
                // Writing into an immediate or a branch operand is
                // nonsensical at this layer. Drop the op so the
                // bridge stays total (I-6).
            }
        }
    }

    /// Translate a read-modify-write Operation (`dst = dst <op> src`).
    fn translate_binop_inplace(
        &mut self,
        dst: &Operand,
        src: &Operand,
        kind: BinaryKind,
        ops: &mut Vec<RawOp>,
    ) {
        let lhs = self.translate_read(dst, ops);
        let rhs = self.translate_read(src, ops);
        let result_kind = match kind {
            BinaryKind::Add => RawOpKind::Add { lhs, rhs },
            BinaryKind::Sub => RawOpKind::Sub { lhs, rhs },
            BinaryKind::Mul => RawOpKind::Mul { lhs, rhs },
            BinaryKind::And => RawOpKind::And { lhs, rhs },
            BinaryKind::Or => RawOpKind::Or { lhs, rhs },
            BinaryKind::Xor => RawOpKind::Xor { lhs, rhs },
            BinaryKind::Shl => RawOpKind::Shl { lhs, rhs },
            BinaryKind::Shr => RawOpKind::Shr { lhs, rhs },
        };
        match dst {
            Operand::Register { name, .. } => {
                let dst_var = self.var_for_register(name);
                ops.push(RawOp {
                    dst: Some(dst_var),
                    kind: result_kind,
                });
            }
            Operand::Memory { .. } => {
                let result = self.synth_temp(64);
                ops.push(RawOp {
                    dst: Some(result),
                    kind: result_kind,
                });
                let (addr_op, width) = self.translate_memory_address(dst, ops);
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Store {
                        address: addr_op,
                        value: RawOperand::Variable(result),
                        width,
                    },
                });
            }
            Operand::Immediate { .. } | Operand::Branch { .. } => {
                // Invalid destination — drop (I-6).
            }
        }
    }

    fn translate_unop_inplace(&mut self, dst: &Operand, kind: UnaryKind, ops: &mut Vec<RawOp>) {
        let src = self.translate_read(dst, ops);
        let result_kind = match kind {
            UnaryKind::Neg => RawOpKind::Neg { src },
            UnaryKind::Not => RawOpKind::Not { src },
        };
        match dst {
            Operand::Register { name, .. } => {
                let dst_var = self.var_for_register(name);
                ops.push(RawOp {
                    dst: Some(dst_var),
                    kind: result_kind,
                });
            }
            Operand::Memory { .. } => {
                let result = self.synth_temp(64);
                ops.push(RawOp {
                    dst: Some(result),
                    kind: result_kind,
                });
                let (addr_op, width) = self.translate_memory_address(dst, ops);
                ops.push(RawOp {
                    dst: None,
                    kind: RawOpKind::Store {
                        address: addr_op,
                        value: RawOperand::Variable(result),
                        width,
                    },
                });
            }
            Operand::Immediate { .. } | Operand::Branch { .. } => {}
        }
    }

    /// Compute the byte address described by a Memory operand, leaving
    /// the result as a [`RawOperand`] that any later op can consume.
    /// Returns `(address, width_in_bytes)`. `width_in_bytes` falls
    /// back to `1` when the operand carries no size (e.g. the operand
    /// for `lea`).
    fn translate_memory_address(
        &mut self,
        operand: &Operand,
        ops: &mut Vec<RawOp>,
    ) -> (RawOperand, u8) {
        let Operand::Memory {
            base,
            index,
            scale,
            displacement,
            size_bits,
            ..
        } = operand
        else {
            // Caller should only invoke this on Memory operands. Be
            // permissive and emit a zero address rather than panic.
            return (RawOperand::Const(0), 1);
        };
        let width = mem_width_bytes(*size_bits);

        // Start from the base register if present; otherwise from the
        // displacement directly.
        let mut current = match base {
            Some(name) => {
                let base_var = self.var_for_register(name);
                if *displacement != 0 {
                    let temp = self.synth_temp(64);
                    ops.push(RawOp {
                        dst: Some(temp),
                        kind: RawOpKind::Add {
                            lhs: RawOperand::Variable(base_var),
                            rhs: RawOperand::Const(*displacement),
                        },
                    });
                    RawOperand::Variable(temp)
                } else {
                    RawOperand::Variable(base_var)
                }
            }
            None => RawOperand::Const(*displacement),
        };

        if let Some(idx_name) = index {
            let idx_var = self.var_for_register(idx_name);
            let scaled = if *scale > 1 {
                let temp = self.synth_temp(64);
                ops.push(RawOp {
                    dst: Some(temp),
                    kind: RawOpKind::Mul {
                        lhs: RawOperand::Variable(idx_var),
                        rhs: RawOperand::Const(i64::from(*scale)),
                    },
                });
                RawOperand::Variable(temp)
            } else {
                RawOperand::Variable(idx_var)
            };
            let temp = self.synth_temp(64);
            ops.push(RawOp {
                dst: Some(temp),
                kind: RawOpKind::Add {
                    lhs: current,
                    rhs: scaled,
                },
            });
            current = RawOperand::Variable(temp);
        }

        (current, width)
    }
}

#[derive(Debug, Clone, Copy)]
enum BinaryKind {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy)]
enum UnaryKind {
    Neg,
    Not,
}

/// Map an Instruction-IR conditional code onto the SSA-layer
/// [`CompareKind`]. Returns `None` for codes that aren't expressible as
/// a two-operand compare under the present SSA vocabulary (sign,
/// overflow, parity, the CX-zero idiom). The bridge degrades those
/// blocks to [`RawTerminator::Indirect`] so structuring doesn't invent
/// a comparison it can't justify.
#[must_use]
fn condition_to_compare_kind(condition: Condition) -> Option<CompareKind> {
    match condition {
        Condition::Equal => Some(CompareKind::Eq),
        Condition::NotEqual => Some(CompareKind::Ne),
        Condition::Less => Some(CompareKind::Lt),
        Condition::LessEqual => Some(CompareKind::Le),
        Condition::Greater => Some(CompareKind::Gt),
        Condition::GreaterEqual => Some(CompareKind::Ge),
        Condition::Below => Some(CompareKind::Ult),
        Condition::BelowEqual => Some(CompareKind::Ule),
        Condition::Above => Some(CompareKind::Ugt),
        Condition::AboveEqual => Some(CompareKind::Uge),
        Condition::Sign
        | Condition::NotSign
        | Condition::Overflow
        | Condition::NotOverflow
        | Condition::Parity
        | Condition::NotParity
        | Condition::CxZero => None,
    }
}

fn mem_width_bytes(size_bits: u16) -> u8 {
    if size_bits == 0 {
        1
    } else {
        size_bits.div_ceil(8).min(8) as u8
    }
}

fn opcode_label(op: &Operation) -> &'static str {
    match op {
        Operation::Move { .. } => "mov",
        Operation::LoadAddress { .. } => "lea",
        Operation::Add { .. } => "add",
        Operation::Sub { .. } => "sub",
        Operation::Mul { .. } => "mul",
        Operation::Div { .. } => "div",
        Operation::And { .. } => "and",
        Operation::Or { .. } => "or",
        Operation::Xor { .. } => "xor",
        Operation::Shl { .. } => "shl",
        Operation::Shr { .. } => "shr",
        Operation::Sar { .. } => "sar",
        Operation::Not { .. } => "not",
        Operation::Neg { .. } => "neg",
        Operation::Compare { .. } => "cmp",
        Operation::Test { .. } => "test",
        Operation::Push { .. } => "push",
        Operation::Pop { .. } => "pop",
        Operation::Jump { .. } => "jmp",
        Operation::Call { .. } => "call",
        Operation::Return => "ret",
        Operation::Nop => "nop",
        Operation::Interrupt { .. } => "int",
        Operation::Syscall => "syscall",
        Operation::Opaque { .. } => "opaque",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_analysis::cfg::{BasicBlock, Cfg, Edge, EdgeKind, Terminator};
    use dac_analysis::dom::DominatorTree;
    use dac_analysis::ssa::construct_ssa;
    use dac_arch::Architecture;
    use dac_core::{EvidenceGraph, EvidenceId, EvidenceNode, IrLayer};
    use dac_ir::instr::{Condition, InstructionIr, Operand, Operation, Target};

    fn rf_x86_64() -> &'static RegisterFile {
        // The bridge only consults `by_name` + `register` + `parent`,
        // so we lean on the x86-64 backend's Architecture impl for
        // the real register file. Tests depend on dac-arch-x86 as a
        // dev-dep so this stays out of the production graph.
        let arch: &'static dac_arch_x86::X86_64 = &dac_arch_x86::X86_64;
        arch.register_file()
    }

    fn fresh_evidence() -> EvidenceId {
        // Mint a one-off EvidenceId for tests via a local graph. The
        // bridge doesn't read the evidence at all, but the Cfg struct
        // requires one — every node in the real pipeline carries
        // provenance (I-2).
        let mut g = EvidenceGraph::new();
        g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        })
    }

    fn block(
        id: u32,
        address: u64,
        end: u64,
        terminator: Terminator,
        instr_count: usize,
    ) -> BasicBlock {
        BasicBlock {
            id,
            address,
            end,
            instructions: vec![stub_decoded_instr(); instr_count],
            terminator,
        }
    }

    fn stub_decoded_instr() -> dac_arch::DecodedInstruction {
        dac_arch::DecodedInstruction {
            address: 0,
            length: 1,
            bytes: vec![0x90],
            mnemonic: "nop".to_string(),
            operands: String::new(),
            flow: dac_arch::ControlFlow::Sequential,
            valid: true,
        }
    }

    fn lifted_move(dst: &str, src_value: i64) -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Move {
                dst: Operand::Register {
                    name: dst.to_string(),
                    size_bits: 64,
                },
                src: Operand::Immediate {
                    value: src_value,
                    size_bits: 64,
                },
            },
        }
    }

    fn lifted_xor(dst: &str, src: &str) -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Xor {
                dst: Operand::Register {
                    name: dst.to_string(),
                    size_bits: 32,
                },
                src: Operand::Register {
                    name: src.to_string(),
                    size_bits: 32,
                },
            },
        }
    }

    fn lifted_compare_imm(reg: &str, value: i64) -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Compare {
                lhs: Operand::Register {
                    name: reg.to_string(),
                    size_bits: 64,
                },
                rhs: Operand::Immediate {
                    value,
                    size_bits: 32,
                },
            },
        }
    }

    fn lifted_jcc(cond: Condition, target: u64) -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Jump {
                target: Target::Direct(target),
                condition: Some(cond),
            },
        }
    }

    fn lifted_ret() -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Return,
        }
    }

    fn lifted_jmp(target: u64) -> InstructionIr {
        InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Jump {
                target: Target::Direct(target),
                condition: None,
            },
        }
    }

    fn empty_cfg(blocks: Vec<BasicBlock>, edges: Vec<Edge>) -> Cfg {
        let entry = blocks.first().map(|b| b.id).unwrap_or(0);
        let exits: Vec<u32> = blocks
            .iter()
            .filter(|b| !edges.iter().any(|e| e.from == b.id))
            .map(|b| b.id)
            .collect();
        Cfg {
            function_address: blocks.first().map(|b| b.address).unwrap_or(0),
            function_end: blocks.iter().map(|b| b.end).max().unwrap_or(0),
            function_name: Some("test".to_string()),
            blocks,
            entry,
            exits,
            edges,
            unreachable: Vec::new(),
            evidence: fresh_evidence(),
        }
    }

    #[test]
    fn subreg_writes_canonicalise_to_64bit_parent() {
        // xor eax, eax: dst and src are both "eax", canonical "rax".
        let cfg = empty_cfg(vec![block(0, 0x0, 0x4, Terminator::Return, 2)], vec![]);
        let instrs = vec![vec![lifted_xor("eax", "eax"), lifted_ret()]];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());

        // Only "rax" should appear in the variable table; "eax" must
        // collapse onto its parent. The canonical name is "rax".
        let rax = raw.variables.iter().find(|v| v.name == "rax").unwrap();
        assert!(raw.variables.iter().all(|v| v.name != "eax"));
        // The single Xor op writes the canonical rax variable id.
        let block = &raw.blocks[0];
        assert_eq!(block.ops.len(), 1);
        assert_eq!(block.ops[0].dst, Some(rax.id));
    }

    #[test]
    fn return_terminator_reads_rax_value() {
        let cfg = empty_cfg(vec![block(0, 0x0, 0x2, Terminator::Return, 1)], vec![]);
        // Just `ret`. The block has no body; the terminator is built
        // from the CFG metadata, not the instruction list.
        let instrs = vec![vec![lifted_ret()]];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        match &raw.blocks[0].terminator {
            RawTerminator::Return {
                value: Some(RawOperand::Variable(v)),
            } => {
                let rax = raw.variables.iter().find(|var| var.name == "rax").unwrap();
                assert_eq!(*v, rax.id);
            }
            other => panic!("expected Return with rax operand, got {other:?}"),
        }
    }

    #[test]
    fn compare_then_jcc_collapses_into_branch_terminator() {
        // Block 0: cmp rax, 0; je 0x10
        //   taken → block 1; not_taken → block 2
        // Block 1: ret
        // Block 2: ret
        let blocks = vec![
            block(
                0,
                0x0,
                0x6,
                Terminator::Conditional { target: Some(0x10) },
                2,
            ),
            block(1, 0x6, 0x8, Terminator::Return, 1),
            block(2, 0x10, 0x12, Terminator::Return, 1),
        ];
        let edges = vec![
            Edge {
                from: 0,
                to: 2,
                kind: EdgeKind::Taken,
            },
            Edge {
                from: 0,
                to: 1,
                kind: EdgeKind::NotTaken,
            },
        ];
        let cfg = empty_cfg(blocks, edges);
        let instrs = vec![
            vec![
                lifted_compare_imm("rax", 0),
                lifted_jcc(Condition::Equal, 0x10),
            ],
            vec![lifted_ret()],
            vec![lifted_ret()],
        ];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());

        match &raw.blocks[0].terminator {
            RawTerminator::Branch {
                taken, not_taken, ..
            } => {
                assert_eq!(*taken, 2);
                assert_eq!(*not_taken, 1);
            }
            other => panic!("expected Branch terminator, got {other:?}"),
        }
        // The Compare op landed as the last (or only) op in block 0.
        let last_op = raw.blocks[0].ops.last().expect("compare op");
        assert!(matches!(
            last_op.kind,
            RawOpKind::Compare {
                kind: CompareKind::Eq,
                ..
            }
        ));
    }

    #[test]
    fn unsupported_condition_degrades_to_indirect() {
        // jp 0x10 — Condition::Parity has no two-operand compare
        // counterpart in the SSA vocabulary. The terminator falls
        // back to Indirect (the block stops being a structured
        // branch), which is the I-6-honest degradation.
        let blocks = vec![
            block(
                0,
                0x0,
                0x4,
                Terminator::Conditional { target: Some(0x10) },
                2,
            ),
            block(1, 0x4, 0x6, Terminator::Return, 1),
            block(2, 0x10, 0x12, Terminator::Return, 1),
        ];
        let edges = vec![
            Edge {
                from: 0,
                to: 2,
                kind: EdgeKind::Taken,
            },
            Edge {
                from: 0,
                to: 1,
                kind: EdgeKind::NotTaken,
            },
        ];
        let cfg = empty_cfg(blocks, edges);
        let instrs = vec![
            vec![
                lifted_compare_imm("rax", 0),
                lifted_jcc(Condition::Parity, 0x10),
            ],
            vec![lifted_ret()],
            vec![lifted_ret()],
        ];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        assert!(matches!(raw.blocks[0].terminator, RawTerminator::Indirect));
    }

    #[test]
    fn nop_does_not_emit_a_raw_op() {
        let cfg = empty_cfg(vec![block(0, 0x0, 0x3, Terminator::Return, 2)], vec![]);
        let nop = InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Nop,
        };
        let instrs = vec![vec![nop, lifted_ret()]];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        // Block has 0 body ops — Nop produced nothing, Return is the
        // terminator.
        assert_eq!(raw.blocks[0].ops.len(), 0);
    }

    #[test]
    fn opaque_passes_through_with_preserved_mnemonic() {
        let cfg = empty_cfg(vec![block(0, 0x0, 0x4, Terminator::Return, 2)], vec![]);
        let opaque = InstructionIr {
            address: 0,
            length: 3,
            op: Operation::Opaque {
                mnemonic: "vpcmpeqq".to_string(),
            },
        };
        let instrs = vec![vec![opaque, lifted_ret()]];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        let RawOpKind::Opaque { ref mnemonic, .. } = raw.blocks[0].ops[0].kind else {
            panic!("expected Opaque, got {:?}", raw.blocks[0].ops[0].kind);
        };
        assert_eq!(mnemonic, "vpcmpeqq");
    }

    #[test]
    fn jcc_without_prior_compare_degrades_to_indirect() {
        let blocks = vec![
            block(
                0,
                0x0,
                0x2,
                Terminator::Conditional { target: Some(0x10) },
                1,
            ),
            block(1, 0x2, 0x4, Terminator::Return, 1),
            block(2, 0x10, 0x12, Terminator::Return, 1),
        ];
        let edges = vec![
            Edge {
                from: 0,
                to: 2,
                kind: EdgeKind::Taken,
            },
            Edge {
                from: 0,
                to: 1,
                kind: EdgeKind::NotTaken,
            },
        ];
        let cfg = empty_cfg(blocks, edges);
        let instrs = vec![
            vec![lifted_jcc(Condition::Equal, 0x10)],
            vec![lifted_ret()],
            vec![lifted_ret()],
        ];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        assert!(matches!(raw.blocks[0].terminator, RawTerminator::Indirect));
    }

    #[test]
    fn unconditional_jump_resolves_to_branch_edge_target() {
        let blocks = vec![
            block(0, 0x0, 0x2, Terminator::Branch { target: Some(0x10) }, 1),
            block(1, 0x10, 0x12, Terminator::Return, 1),
        ];
        let edges = vec![Edge {
            from: 0,
            to: 1,
            kind: EdgeKind::Branch,
        }];
        let cfg = empty_cfg(blocks, edges);
        let instrs = vec![vec![lifted_jmp(0x10)], vec![lifted_ret()]];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        assert!(matches!(
            raw.blocks[0].terminator,
            RawTerminator::Jump { target: 1 }
        ));
    }

    #[test]
    fn lift_function_is_deterministic_across_runs() {
        let cfg = empty_cfg(vec![block(0, 0x0, 0x6, Terminator::Return, 3)], vec![]);
        let instrs = vec![vec![
            lifted_move("rax", 7),
            lifted_xor("ebx", "ebx"),
            lifted_ret(),
        ]];
        let a = lift_function(&cfg, &instrs, rf_x86_64());
        let b = lift_function(&cfg, &instrs, rf_x86_64());
        assert_eq!(a, b);
    }

    #[test]
    fn end_to_end_diamond_construct_ssa_then_structure_produces_if() {
        // The B3.8 done-when rubric, distilled into a single test:
        // a hand-crafted if-then-else CFG, lifted to RawFunction,
        // run through construct_ssa, then run through
        // dac_analysis::structuring::structure → SemFunction.
        // The body should be a Stmt::If with two Stmt::Return arms.
        use dac_analysis::dom::PostDominatorTree;
        use dac_analysis::loops::LoopForest;
        use dac_analysis::structuring::structure;
        use dac_ir::sem::Stmt;

        // Block 0: cmp rax, 0; jne 0x10
        // Block 1: mov rax, 1; ret
        // Block 2: mov rax, 2; ret
        let blocks = vec![
            block(
                0,
                0x0,
                0x6,
                Terminator::Conditional { target: Some(0x10) },
                2,
            ),
            block(1, 0x6, 0xC, Terminator::Return, 2),
            block(2, 0x10, 0x16, Terminator::Return, 2),
        ];
        let edges = vec![
            Edge {
                from: 0,
                to: 2,
                kind: EdgeKind::Taken,
            },
            Edge {
                from: 0,
                to: 1,
                kind: EdgeKind::NotTaken,
            },
        ];
        let cfg = empty_cfg(blocks, edges);
        let instrs = vec![
            vec![
                lifted_compare_imm("rax", 0),
                lifted_jcc(Condition::NotEqual, 0x10),
            ],
            vec![lifted_move("rax", 1), lifted_ret()],
            vec![lifted_move("rax", 2), lifted_ret()],
        ];
        let raw = lift_function(&cfg, &instrs, rf_x86_64());
        let doms = DominatorTree::build(&cfg);
        let ssa = construct_ssa(&cfg, &doms, &raw);
        let pdoms = PostDominatorTree::build(&cfg);
        let loops = LoopForest::build(&cfg, &doms);
        let sem = structure(&ssa, &cfg, &doms, &pdoms, &loops);
        // The structurer should recognise this as an If.
        let stmts = &sem.body.stmts;
        let has_if = stmts.iter().any(|s| matches!(s, Stmt::If { .. }));
        assert!(
            has_if,
            "expected Stmt::If in structured body, got {stmts:?}"
        );
    }
}
