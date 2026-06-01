//! Tracing initialization.
//!
//! dac uses [`tracing`] for structured diagnostics. Per-pass spans land
//! with the pass manager (B0.4); this module exists so the CLI and any
//! library embedder can set up a subscriber consistently.

use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing for the dac CLI / library.
///
/// Reads `RUST_LOG` for filter configuration. When `RUST_LOG` is unset, the
/// default filter is `"debug"` if `debug` is `true` and `"info"` otherwise
/// (spec §10.1 `--debug`). When `json` is `true`, emits structured JSON
/// events (spec §10.1 `--json`); otherwise emits the default human-readable
/// format.
///
/// Safe to call multiple times: only the first call has effect (subsequent
/// calls are silently ignored). This keeps test setups idempotent.
pub fn init_tracing(json: bool, debug: bool) {
    let default_level = if debug { "debug" } else { "info" };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
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
        init_tracing(false, false);
        init_tracing(false, true);
        init_tracing(true, false);
    }
}
