//! `dac-binfmt` — binary format parsing (ELF, PE, Mach-O) for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! Status: B0.2 landed magic-byte format detection. B1.1 added the full
//! ELF parser on top of `object` (ADR-0003) and the shared `BinaryModel`
//! vocabulary. B1.2 plugs PE into the same bridge — `elf.rs` and `pe.rs`
//! now both delegate to `bridge::parse_object`, so every model field stays
//! in lock-step across formats. Mach-O follows the same pattern later.

#![forbid(unsafe_code)]

mod bridge;
mod elf;
mod model;
mod pe;

use dac_core::{Error, Result};

pub use model::{
    Architecture, BinaryFormat, BinaryModel, Bits, Endian, Export, Import, Permissions, Relocation,
    RelocationKind, Section, SectionKind, Segment, StringRef, Symbol, SymbolBinding, SymbolKind,
    SymbolSource,
};

/// Identify the binary format of `bytes` by inspecting magic numbers.
///
/// Returns [`Error::UnsupportedFormat`] if no magic matches. Never panics
/// on any input, including empty or arbitrary-length garbage (NFR-4).
pub fn detect_format(bytes: &[u8]) -> Result<BinaryFormat> {
    if bytes.starts_with(&[0x7F, b'E', b'L', b'F']) {
        return Ok(BinaryFormat::Elf);
    }

    if bytes.starts_with(b"MZ") {
        if let Some(off_bytes) = bytes
            .get(0x3C..0x40)
            .and_then(|b| <[u8; 4]>::try_from(b).ok())
        {
            let pe_offset = u32::from_le_bytes(off_bytes) as usize;
            if bytes.get(pe_offset..pe_offset.saturating_add(4)) == Some(b"PE\0\0".as_slice()) {
                return Ok(BinaryFormat::Pe);
            }
        }
    }

    if let Some(magic) = bytes.get(..4).and_then(|b| <[u8; 4]>::try_from(b).ok()) {
        // Magic-byte sequences for Mach-O thin (LE + BE) and fat binaries.
        if matches!(
            magic,
            [0xFE, 0xED, 0xFA, 0xCE]
                | [0xCE, 0xFA, 0xED, 0xFE]
                | [0xFE, 0xED, 0xFA, 0xCF]
                | [0xCF, 0xFA, 0xED, 0xFE]
                | [0xCA, 0xFE, 0xBA, 0xBE]
                | [0xBE, 0xBA, 0xFE, 0xCA]
        ) {
            return Ok(BinaryFormat::MachO);
        }
    }

    Err(Error::UnsupportedFormat)
}

/// Construct a [`BinaryModel`] from the input bytes.
///
/// ELF inputs (B1.1) and PE inputs (B1.2) both produce fully populated
/// [`BinaryModel`]s with sections, segments, symbols, imports, exports,
/// relocations, strings, and needed libraries — the two formats share the
/// generic walk in [`bridge`]. Mach-O still returns
/// [`Error::UnsupportedFormat`] until its parser lands.
pub fn load_from_bytes(bytes: &[u8]) -> Result<BinaryModel> {
    let format = detect_format(bytes)?;
    match format {
        BinaryFormat::Elf => elf::parse(bytes),
        BinaryFormat::Pe => pe::parse(bytes),
        BinaryFormat::MachO => Err(Error::UnsupportedFormat),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn elf_magic_is_detected() {
        let mut buf = vec![0u8; 16];
        buf[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        assert_eq!(detect_format(&buf).unwrap(), BinaryFormat::Elf);
    }

    #[test]
    fn pe_with_valid_header_pointer_is_detected() {
        let mut buf = vec![0u8; 0x80];
        buf[..2].copy_from_slice(b"MZ");
        buf[0x3C..0x40].copy_from_slice(&0x40_u32.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(b"PE\0\0");
        assert_eq!(detect_format(&buf).unwrap(), BinaryFormat::Pe);
    }

    #[test]
    fn mz_without_pe_signature_is_unsupported() {
        let buf = b"MZ\0\0".to_vec();
        assert!(matches!(detect_format(&buf), Err(Error::UnsupportedFormat)));
    }

    #[test]
    fn macho_thin_le_magic_is_detected() {
        let buf = vec![0xCE, 0xFA, 0xED, 0xFE, 0, 0, 0, 0];
        assert_eq!(detect_format(&buf).unwrap(), BinaryFormat::MachO);
    }

    #[test]
    fn macho_fat_magic_is_detected() {
        let buf = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 0];
        assert_eq!(detect_format(&buf).unwrap(), BinaryFormat::MachO);
    }

    #[test]
    fn empty_input_is_unsupported() {
        assert!(matches!(detect_format(&[]), Err(Error::UnsupportedFormat)));
    }

    #[test]
    fn three_byte_input_is_unsupported_without_panic() {
        for b in [
            &[0x7F, b'E', b'L'][..],
            &[b'M', b'Z', 0][..],
            &[0xCA, 0xFE, 0xBA][..],
        ] {
            let _ = detect_format(b);
            let _ = load_from_bytes(b);
        }
    }

    /// Magic bytes without a valid ELF header reach the full `object`
    /// parser; that parser returns an error rather than panicking, and
    /// dac surfaces it as [`Error::MalformedBinary`].
    #[test]
    fn elf_magic_without_valid_header_is_malformed() {
        let mut buf = vec![0u8; 16];
        buf[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        match load_from_bytes(&buf) {
            Err(Error::MalformedBinary { format, .. }) => assert_eq!(format, "ELF"),
            other => panic!("expected MalformedBinary, got {other:?}"),
        }
    }

    /// Mach-O has no parser yet; calls to [`load_from_bytes`] must surface
    /// the dispatch decision as [`Error::UnsupportedFormat`] rather than
    /// panicking or being misclassified.
    #[test]
    fn macho_returns_unsupported_format() {
        let macho = vec![0xCF, 0xFA, 0xED, 0xFE, 0, 0, 0, 0];
        assert!(matches!(
            load_from_bytes(&macho),
            Err(Error::UnsupportedFormat)
        ));
    }

    /// A hand-built MZ + `PE\0\0` stub passes [`detect_format`] but is
    /// otherwise empty. The full PE parser must reject it cleanly through
    /// [`Error::MalformedBinary`] with the `"PE"` tag, not panic.
    #[test]
    fn pe_magic_without_valid_header_is_malformed() {
        let mut pe = vec![0u8; 0x80];
        pe[..2].copy_from_slice(b"MZ");
        pe[0x3C..0x40].copy_from_slice(&0x40_u32.to_le_bytes());
        pe[0x40..0x44].copy_from_slice(b"PE\0\0");
        match load_from_bytes(&pe) {
            Err(Error::MalformedBinary { format, .. }) => assert_eq!(format, "PE"),
            other => panic!("expected MalformedBinary, got {other:?}"),
        }
    }

    /// Smoke check: feed deterministic random bytes through the format
    /// detector and assert that no input causes a panic (NFR-4). The full
    /// parser is exercised by the libfuzzer target in `fuzz/`.
    #[test]
    fn random_input_never_panics() {
        let mut rng = StdRng::seed_from_u64(0xDAC0_5EED);
        for _ in 0..512 {
            let len = rng.gen_range(0..4096);
            let mut buf = vec![0u8; len];
            rng.fill(&mut buf[..]);
            let _ = detect_format(&buf);
            let _ = load_from_bytes(&buf);
        }
    }
}
