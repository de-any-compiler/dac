//! ELF entry point into the shared [`crate::bridge`] parser.
//!
//! All format-neutral work (sections, segments, symbols, imports, exports,
//! relocations, strings) lives in `bridge.rs`. This module only carries
//! the format tag — keeping ELF and PE strictly aligned on what they
//! surface and how malformed-input errors are labelled (ADR-0003).

use dac_core::Result;

use crate::bridge;
use crate::model::{BinaryFormat, BinaryModel};

/// Parse `bytes` as an ELF binary and produce a [`BinaryModel`].
pub(crate) fn parse(bytes: &[u8]) -> Result<BinaryModel> {
    bridge::parse_object(bytes, BinaryFormat::Elf, "ELF")
}
