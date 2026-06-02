//! `dac-knowledge` — calling conventions, standard library signatures, and
//! pattern catalogues for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What ships when
//!
//! - **B2.5 (this batch).** [`convention`] — calling-convention
//!   table for x86-64 (SysV AMD64 and Microsoft x64). Consumed by
//!   `dac_recovery::convention` to infer per-function conventions
//!   (FR-13).
//! - **B2.6.** API signatures (libc, Win32 minimal set) for type
//!   propagation.
//! - **B3.3.** Pattern catalogue for idiom recognition.

#![forbid(unsafe_code)]

pub mod convention;

pub use convention::{
    x86_64_convention_by_name, CallingConvention, MS_X64, SYSV_AMD64, X86_64_CONVENTIONS,
};
