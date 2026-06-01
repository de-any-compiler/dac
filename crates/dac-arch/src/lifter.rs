//! Architecture-neutral lifter surface.
//!
//! A *lifter* turns the raw byte stream of an instruction (plus its
//! address) into an arch-neutral [`dac_ir::instr::InstructionIr`] node.
//! Each ISA backend (`dac-arch-x86`, `dac-arch-aarch64`, …) provides one;
//! the rest of the pipeline talks to all of them through this trait.
//!
//! The lifter trait is intentionally narrow:
//!
//! - **No mutable evidence-graph state.** Wiring lifted instructions
//!   into `dac_core::EvidenceGraph` is the orchestrator's job
//!   (B1.5+). The IR carries the byte span that the orchestrator turns
//!   into evidence; the lifter stays pure so it is trivial to unit-test
//!   and to call from the coverage reporter.
//! - **One instruction at a time.** Callers that want a sweep can pair
//!   the lifter with the decoder's [`crate::InstructionDecoder::iter`].
//!   Future batches may add a fused decode-and-lift entry point if the
//!   double-decode cost becomes load-bearing.
//! - **Always returns IR.** Unsupported opcodes are not an error — they
//!   land as [`dac_ir::instr::Operation::Opaque`]. The whole point of
//!   that arm is that CFG construction (B2.1) and function discovery
//!   (B1.5) still see a node, with `address + length` describing the
//!   bytes, so control flow can still be recovered (I-6: degrade,
//!   don't invent).

use core::fmt;
use std::collections::BTreeMap;

use dac_ir::instr::{InstructionIr, Operation};

/// Lifter trait. See module docs for the contract.
///
/// The trait is `Send + Sync` for the same reason [`crate::Architecture`]
/// is: the pass manager parallelizes architecture-aware passes (NFR-7).
pub trait InstructionLifter: Send + Sync {
    /// Lift the instruction at the start of `bytes` (addressed at
    /// `address`). Returns a single [`InstructionIr`]; encoding errors
    /// surface as [`Operation::Opaque`] with mnemonic `"(bad)"`.
    fn lift(&self, bytes: &[u8], address: u64) -> InstructionIr;
}

/// Per-batch lifter coverage report.
///
/// Closes the B1.4 "coverage report: which opcodes are lifted vs not"
/// deliverable. Callers feed each lifted [`InstructionIr`] through
/// [`Coverage::record`]; `Coverage` then exposes the totals and a
/// histogram of opaque mnemonics so the missing set is visible to the
/// reviewer rather than hidden behind the aggregate percentage.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Coverage {
    /// Every instruction the report has seen.
    pub total: u64,
    /// Instructions whose [`Operation`] is anything other than
    /// [`Operation::Opaque`].
    pub lifted: u64,
    /// Instructions that landed as [`Operation::Opaque`].
    pub opaque: u64,
    /// Per-mnemonic opaque counts, sorted lexicographically for stable
    /// reporting (NFR-9: determinism).
    pub opaque_mnemonics: BTreeMap<String, u64>,
}

impl Coverage {
    /// Fold one instruction into the running totals.
    pub fn record(&mut self, ir: &InstructionIr) {
        self.total += 1;
        match &ir.op {
            Operation::Opaque { mnemonic } => {
                self.opaque += 1;
                *self.opaque_mnemonics.entry(mnemonic.clone()).or_insert(0) += 1;
            }
            _ => {
                self.lifted += 1;
            }
        }
    }

    /// Lifted fraction in `0.0..=1.0`. Returns `0.0` on an empty report
    /// instead of a NaN so callers can compare unconditionally.
    #[must_use]
    pub fn lifted_fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.lifted as f64) / (self.total as f64)
        }
    }
}

impl fmt::Display for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "coverage: {} / {} lifted ({:.2}% opaque)",
            self.lifted,
            self.total,
            100.0 - self.lifted_fraction() * 100.0,
        )?;
        for (mnem, n) in &self.opaque_mnemonics {
            writeln!(f, "  opaque {mnem}: {n}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_ir::instr::{InstructionIr, Operand, Operation};

    fn lifted(addr: u64) -> InstructionIr {
        InstructionIr {
            address: addr,
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

    fn opaque(addr: u64, mnem: &str) -> InstructionIr {
        InstructionIr {
            address: addr,
            length: 4,
            op: Operation::Opaque {
                mnemonic: mnem.to_string(),
            },
        }
    }

    #[test]
    fn empty_coverage_is_zero_fraction() {
        let cov = Coverage::default();
        assert_eq!(cov.total, 0);
        assert_eq!(cov.lifted_fraction(), 0.0);
    }

    #[test]
    fn coverage_counts_lifted_and_opaque_separately() {
        let mut cov = Coverage::default();
        cov.record(&lifted(0x100));
        cov.record(&lifted(0x103));
        cov.record(&opaque(0x106, "vfmadd"));
        cov.record(&opaque(0x10A, "vfmadd"));
        cov.record(&opaque(0x10E, "vpcmpeqq"));
        assert_eq!(cov.total, 5);
        assert_eq!(cov.lifted, 2);
        assert_eq!(cov.opaque, 3);
        assert_eq!(cov.lifted_fraction(), 0.4);
        assert_eq!(cov.opaque_mnemonics.get("vfmadd"), Some(&2));
        assert_eq!(cov.opaque_mnemonics.get("vpcmpeqq"), Some(&1));
    }

    #[test]
    fn coverage_display_lists_opaque_mnemonics_in_sorted_order() {
        let mut cov = Coverage::default();
        cov.record(&opaque(0, "b"));
        cov.record(&opaque(0, "a"));
        cov.record(&opaque(0, "c"));
        let s = format!("{cov}");
        // Lexicographic order on opaque mnemonics is what makes the
        // report reproducible across runs.
        let a = s.find("opaque a").unwrap();
        let b = s.find("opaque b").unwrap();
        let c = s.find("opaque c").unwrap();
        assert!(a < b && b < c, "alphabetical order: {s}");
    }
}
