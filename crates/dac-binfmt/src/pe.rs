//! PE (Portable Executable / COFF) entry point into [`crate::bridge`].
//!
//! `object`'s trait-uniform read API (ADR-0003) means PE parsing reuses
//! the same generic walk that drives ELF. Format-specific decoding lives
//! in the helpers in `bridge.rs`:
//!
//! * `SectionFlags::Coff { characteristics }` and
//!   `SegmentFlags::Coff { characteristics }` are mapped through the
//!   PE/COFF `IMAGE_SCN_MEM_*` bits.
//! * `RelocationFlags::Coff { typ }` is mapped through `IMAGE_REL_AMD64_*`,
//!   `IMAGE_REL_I386_*`, and `IMAGE_REL_ARM64_*` constants.
//! * PE base relocations (the `.reloc` table) describe image rebasing, not
//!   symbol bindings, and are intentionally not surfaced as
//!   [`crate::model::Relocation`] entries — the import table already
//!   gives the symbol-resolution view through `obj.imports()`.

use dac_core::Result;

use crate::bridge;
use crate::model::{BinaryFormat, BinaryModel};

/// Parse `bytes` as a PE binary (PE32 or PE32+) and produce a [`BinaryModel`].
pub(crate) fn parse(bytes: &[u8]) -> Result<BinaryModel> {
    bridge::parse_object(bytes, BinaryFormat::Pe, "PE")
}
