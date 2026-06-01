//! `dac-arch` — architecture trait, registry, and shared decoder/lifter
//! types for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## Scope
//!
//! This crate exposes the architecture-neutral vocabulary the rest of the
//! pipeline reads through. Per ARCHITECTURE.md §7, every concrete ISA
//! lands as a separate crate implementing [`Architecture`]; the pipeline
//! then talks to all of them through trait objects. The trait surface is
//! kept narrow on purpose — anything that needs ISA-specific shapes
//! belongs *inside* the implementing crate, not on this trait.
//!
//! ## What lands when
//!
//! - **B1.3 (this batch).** [`Architecture`], [`InstructionDecoder`],
//!   [`DecodedInstruction`], [`ControlFlow`], [`Endianness`], [`Isa`],
//!   [`RegisterFile`].
//! - **B1.4.** `InstructionLifter` trait (decoder → Instruction IR).
//! - **B2.5.** `CallingConvention` shape consumed by convention
//!   inference.

#![forbid(unsafe_code)]

mod decoder;
mod registers;

pub use decoder::{ControlFlow, DecodeError, DecodedInstruction, InstructionDecoder};
pub use registers::{Register, RegisterClass, RegisterFile, RegisterId};

/// Byte ordering of integer-typed values in memory and registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

/// Instruction-set architecture identifier. Used by the orchestrator to
/// look up an [`Architecture`] implementation for a parsed binary.
///
/// The variants are intentionally narrow: this is the set of ISAs dac has
/// or plans to have a backend for. Other ISAs surface from the binary
/// parser as `dac_binfmt::Architecture::Unknown` and never reach this
/// type. New ISAs land here when their crate lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Isa {
    /// 32-bit Intel / AMD.
    I386,
    /// 64-bit Intel / AMD.
    X86_64,
    /// 64-bit ARM (lands in M5).
    Aarch64,
}

impl Isa {
    /// Human-readable identifier suitable for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::I386 => "i386",
            Self::X86_64 => "x86-64",
            Self::Aarch64 => "aarch64",
        }
    }
}

/// Architecture backend.
///
/// Every concrete ISA implements this trait in its own crate
/// (`dac-arch-x86`, `dac-arch-aarch64`, …). Implementations are usually
/// zero-sized marker structs; the trait methods produce the decoder /
/// register file / metadata on demand, so callers hold a
/// `Box<dyn Architecture>` without dragging along ISA-specific state.
///
/// The trait is `Send + Sync` because the pass manager parallelizes
/// architecture-aware passes across cores (NFR-7). Implementations must
/// be self-contained — they may consult thread-local caches but must not
/// share mutable state across instances.
pub trait Architecture: Send + Sync {
    /// Short human-readable name (e.g. `"x86-64"`).
    fn name(&self) -> &'static str;

    /// ISA tag, for registry lookup and diagnostics.
    fn isa(&self) -> Isa;

    /// Pointer width in bytes (4 for 32-bit, 8 for 64-bit).
    fn pointer_size(&self) -> usize;

    /// Byte ordering of integer-typed values.
    fn endianness(&self) -> Endianness;

    /// Construct a fresh instruction decoder. Each call returns an owned
    /// decoder so callers can hand it to a pass without lifetime drama.
    fn decoder(&self) -> Box<dyn InstructionDecoder>;

    /// Register file metadata. The reference is `'static`-equivalent —
    /// implementations typically return a `&'static RegisterFile` lazily
    /// initialised by `OnceLock`.
    fn register_file(&self) -> &RegisterFile;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isa_names_are_stable() {
        assert_eq!(Isa::I386.name(), "i386");
        assert_eq!(Isa::X86_64.name(), "x86-64");
        assert_eq!(Isa::Aarch64.name(), "aarch64");
    }

    #[test]
    fn endianness_is_copy() {
        let e = Endianness::Little;
        let e2 = e;
        assert_eq!(e, e2);
    }
}
