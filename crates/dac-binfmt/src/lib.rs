//! `dac-binfmt` — binary format parsing (ELF, PE, Mach-O) for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! Status: B0.2 lands magic-byte format detection and the panic-policy
//! smoke test. Full ELF parsing lands with B1.1, PE with B1.2.

#![forbid(unsafe_code)]

use dac_core::{Error, Result};

/// A binary executable format dac recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// ELF (Linux, BSD, embedded).
    Elf,
    /// Portable Executable (Windows).
    Pe,
    /// Mach-O (macOS, iOS).
    MachO,
}

impl BinaryFormat {
    /// Human-readable name suitable for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Elf => "ELF",
            Self::Pe => "PE",
            Self::MachO => "Mach-O",
        }
    }
}

/// Minimal placeholder for the binary model.
///
/// Real fields (sections, symbols, imports, relocations, strings) land
/// with B1.1.
#[derive(Debug)]
pub struct BinaryModel {
    /// Detected format.
    pub format: BinaryFormat,
    /// Size of the input in bytes.
    pub size: usize,
}

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

/// Construct a minimal [`BinaryModel`] from the input bytes.
///
/// At B0.2 this only identifies the format. Field-level parsing
/// (sections, symbols, etc.) lands with B1.1.
pub fn load_from_bytes(bytes: &[u8]) -> Result<BinaryModel> {
    let format = detect_format(bytes)?;
    Ok(BinaryModel {
        format,
        size: bytes.len(),
    })
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
        // Exercise the short-buffer paths in PE/Mach-O detection.
        for b in [
            &[0x7F, b'E', b'L'][..],
            &[b'M', b'Z', 0][..],
            &[0xCA, 0xFE, 0xBA][..],
        ] {
            let _ = detect_format(b);
            let _ = load_from_bytes(b);
        }
    }

    /// Smoke check: feed deterministic random bytes through the format
    /// detector and assert that no input causes a panic (NFR-4). This is
    /// the placeholder for the per-parser cargo-fuzz targets that land in
    /// B1.1 / B1.2.
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
