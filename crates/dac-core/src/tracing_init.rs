//! Tracing initialization.
//!
//! dac uses [`tracing`] for structured diagnostics. Per-pass spans land
//! with the pass manager (B0.4); this module exists so the CLI and any
//! library embedder can set up a subscriber consistently.

use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing for the dac CLI / library.
///
/// Reads `RUST_LOG` for filter configuration, falling back to `"info"`.
/// When `json` is `true`, emits structured JSON events (FR per spec
/// §10.1 `--json`); otherwise emits the default human-readable format.
///
/// Safe to call multiple times: only the first call has effect (subsequent
/// calls are silently ignored). This keeps test setups idempotent.
pub fn init_tracing(json: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if json {
        let _ = fmt().with_env_filter(filter).json().try_init();
    } else {
        let _ = fmt().with_env_filter(filter).try_init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_is_idempotent() {
        // Two calls back-to-back must not panic; subsequent calls are no-ops.
        init_tracing(false);
        init_tracing(false);
        init_tracing(true);
    }
}
