//! `dac-knowledge` — calling conventions, standard library signatures, and
//! pattern catalogues for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! ## What ships when
//!
//! - **B2.5.** [`convention`] — calling-convention table for x86-64
//!   (SysV AMD64 and Microsoft x64). Consumed by
//!   `dac_recovery::convention` to infer per-function conventions
//!   (FR-13).
//! - **B2.6 (this batch).** [`api`] — minimal libc and Win32 API
//!   signature catalogue. Consumed by `dac_recovery::types` to seed
//!   the type lattice at known call sites (FR-14, FR-16).
//! - **B3.3.** Pattern catalogue for idiom recognition.

#![forbid(unsafe_code)]

pub mod api;
pub mod convention;

pub use api::{
    api_signatures, lookup_api_signature, lookup_api_signature_in, ApiLibrary, ApiParameter,
    ApiSignature,
};
pub use convention::{
    x86_64_convention_by_name, CallingConvention, ConventionKind, MS_X64, SYSV_AMD64,
    SYSV_AMD64_SYSCALL, X86_64_CONVENTIONS,
};
