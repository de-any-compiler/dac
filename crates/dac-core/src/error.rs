//! Project-wide error type for dac.

use std::io;

/// The project-wide error type for dac.
///
/// Crates surface errors through this enum (or convert into it via
/// [`From`] impls). Variants will be added as new error kinds appear;
/// callers should match non-exhaustively.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O failure (reading an input binary, writing an artifact, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The input does not match any supported binary format.
    #[error("input does not match any supported binary format")]
    UnsupportedFormat,

    /// The input matched a known format but its structure is malformed.
    #[error("malformed {format}: {reason}")]
    MalformedBinary {
        /// Name of the format that recognized the magic bytes.
        format: &'static str,
        /// Human-readable explanation of what was wrong.
        reason: String,
    },

    /// A pass produced a result that violates an IR invariant.
    #[error("invariant violation: {0}")]
    InvariantViolation(String),

    /// The pass manager rejected a pipeline configuration (cycle, missing
    /// producer, duplicate producer, `--deterministic` violation, …).
    #[error("pass manager: {0}")]
    PassManager(String),

    /// Catch-all. Prefer a structured variant whenever possible.
    #[error("{0}")]
    Other(String),
}

/// Project-wide [`Result`] alias defaulting to [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_format_displays_known_message() {
        let s = format!("{}", Error::UnsupportedFormat);
        assert!(s.contains("supported binary format"));
    }

    #[test]
    fn malformed_binary_displays_format_and_reason() {
        let err = Error::MalformedBinary {
            format: "ELF",
            reason: "truncated header".into(),
        };
        let s = format!("{err}");
        assert!(s.contains("ELF"));
        assert!(s.contains("truncated header"));
    }

    #[test]
    fn io_error_converts_via_from() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "nope");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
