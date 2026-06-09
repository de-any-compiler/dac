//! Error type for AI provider calls.
//!
//! Providers do not own the deterministic pipeline (I-4), so a provider
//! failure must be recoverable — the orchestrator records the error,
//! drops the proposal, and continues. [`ProviderError`] carries enough
//! detail for the manifest / report to attribute the failure without
//! leaking host-specific paths.

use std::fmt;

/// What can go wrong when asking an [`crate::AiProvider`] to propose
/// deltas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// Provider was selected but is not configured for this run
    /// (missing model file, missing endpoint). The orchestrator
    /// should fall back to [`crate::NullProvider`] and warn.
    Unavailable(String),
    /// Provider was reachable but transport-level errors prevented a
    /// response (timeout, connection reset, bad HTTP status). String
    /// is the short diagnostic; the manifest still records the
    /// provider name so the failure is auditable.
    Transport(String),
    /// Model refused to answer (safety stop, length cap, malformed
    /// output that couldn't be repaired). Distinct from `Transport`
    /// so the report can keep the provider listed as healthy.
    Refused(String),
}

impl ProviderError {
    /// Short stable tag suitable for log fields. One word, kebab-cased.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "unavailable",
            Self::Transport(_) => "transport",
            Self::Refused(_) => "refused",
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(msg) => write!(f, "AI provider unavailable: {msg}"),
            Self::Transport(msg) => write!(f, "AI provider transport error: {msg}"),
            Self::Refused(msg) => write!(f, "AI provider refused: {msg}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Result alias for provider calls.
pub type ProviderResult<T> = std::result::Result<T, ProviderError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_tag_is_stable_per_variant() {
        assert_eq!(ProviderError::Unavailable("x".into()).kind(), "unavailable",);
        assert_eq!(ProviderError::Transport("x".into()).kind(), "transport");
        assert_eq!(ProviderError::Refused("x".into()).kind(), "refused");
    }

    #[test]
    fn display_prepends_classification() {
        let e = ProviderError::Transport("timeout".to_string());
        assert_eq!(e.to_string(), "AI provider transport error: timeout");
    }

    #[test]
    fn provider_error_is_send_sync_static() {
        fn check<E: Send + Sync + 'static>() {}
        check::<ProviderError>();
    }
}
