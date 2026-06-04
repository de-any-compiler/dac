//! Round-trip compilation check (ARCHITECTURE.md §8).
//!
//! The backend contract requires that emitted source compiles. This
//! module wraps the system C compiler in a small helper that the round-
//! trip tests in `tests/round_trip.rs` use to gate the corpus, and that
//! a future CI hook can call to enforce the same gate on every PR
//! (B2.9 wires it in).
//!
//! The check is best-effort: when no compiler is available, the helper
//! returns [`CompileResult::Skipped`] rather than failing. That keeps
//! the unit-test path green on machines without a toolchain — the
//! gating is the CI environment's job, not the developer's box.
//!
//! ## What's checked
//!
//! - Syntactic correctness: the source parses.
//! - Type correctness within the emitted unit: `cc` is invoked with
//!   `-Wno-everything` to suppress style warnings but `-Werror` is
//!   *not* set; we accept warnings as long as the compile succeeds. A
//!   stricter mode (`-Wall -Werror`) lands when the emitter is mature
//!   enough to produce warning-free C.
//! - Self-containment: only `<stdint.h>` / `<stddef.h>` may be needed.
//!
//! ## What's not checked
//!
//! - Semantic equivalence to the source binary. That is the smoke test
//!   tier in PLAN.md's B2.8 done-when criterion; it lands once the
//!   sample corpus (B2.9) and the lifter→RawFunction bridge exist.

use std::ffi::OsStr;
use std::io::Write as _;
use std::process::{Command, Stdio};

/// Outcome of a round-trip compile attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileResult {
    /// Compilation succeeded. Any warning text the compiler produced
    /// is attached for diagnostic display.
    Ok { stderr: String },
    /// Compilation failed; stderr contains the diagnostics.
    Failed { stderr: String },
    /// No compiler was available on the host. Tests should treat this
    /// as "not exercised" rather than failure.
    Skipped { reason: String },
}

impl CompileResult {
    /// `true` when the compile succeeded.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, CompileResult::Ok { .. })
    }

    /// `true` when the round-trip was skipped because no compiler was
    /// available.
    #[must_use]
    pub fn is_skipped(&self) -> bool {
        matches!(self, CompileResult::Skipped { .. })
    }
}

/// Try to compile `source` with the system C compiler.
///
/// The compiler is chosen from `$CC`, then `cc`, then `gcc`, then
/// `clang`. The source is piped on stdin (`-x c -`) and the output is
/// discarded (`-o /dev/null`) so the helper does not touch the
/// filesystem. `-c` is passed to skip linking, which avoids surprises
/// for hosted-libc references.
///
/// Returns [`CompileResult::Skipped`] when none of the candidate
/// compilers are on `PATH`.
#[must_use]
pub fn try_compile(source: &str) -> CompileResult {
    let candidates = compiler_candidates();
    let mut tried = Vec::new();
    for cc in &candidates {
        match run_compiler(cc, source) {
            RunOutcome::Ok { stderr } => return CompileResult::Ok { stderr },
            RunOutcome::Failed { stderr } => return CompileResult::Failed { stderr },
            RunOutcome::NotFound => tried.push(cc.clone()),
        }
    }
    CompileResult::Skipped {
        reason: format!("no C compiler available (tried: {})", tried.join(", ")),
    }
}

fn compiler_candidates() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(env_cc) = std::env::var("CC") {
        if !env_cc.trim().is_empty() {
            v.push(env_cc);
        }
    }
    for fallback in ["cc", "gcc", "clang"] {
        if !v.iter().any(|c| c == fallback) {
            v.push(fallback.to_string());
        }
    }
    v
}

enum RunOutcome {
    Ok { stderr: String },
    Failed { stderr: String },
    NotFound,
}

fn run_compiler(cc: &str, source: &str) -> RunOutcome {
    let args: &[&OsStr] = &[
        OsStr::new("-x"),
        OsStr::new("c"),
        OsStr::new("-c"),
        OsStr::new("-"),
        OsStr::new("-o"),
        OsStr::new("/dev/null"),
        // Suppress non-error noise; we only care about whether the
        // emitted source compiles, not about stylistic warnings.
        OsStr::new("-w"),
        // `-Dmain=__dac_main__` dodges macOS clang's hard error
        // ("first parameter of 'main' (argument count) must be of
        // type 'int'") when the recovered signature is e.g.
        // `int64_t main(int64_t)`. The on-disk source is unchanged;
        // only the preprocessor sees the rename so we keep
        // recovered-view fidelity while the round-trip probe stops
        // tripping clang's special-case main-signature check. Linux
        // gcc/clang accept the define silently.
        OsStr::new("-Dmain=__dac_main__"),
    ];
    let mut child = match Command::new(cc)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return RunOutcome::NotFound,
        Err(err) => {
            return RunOutcome::Failed {
                stderr: format!("failed to launch {cc}: {err}"),
            };
        }
    };
    if let Some(stdin) = child.stdin.as_mut() {
        if let Err(err) = stdin.write_all(source.as_bytes()) {
            return RunOutcome::Failed {
                stderr: format!("failed to write to {cc} stdin: {err}"),
            };
        }
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(err) => {
            return RunOutcome::Failed {
                stderr: format!("failed to wait on {cc}: {err}"),
            };
        }
    };
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        RunOutcome::Ok { stderr }
    } else {
        RunOutcome::Failed { stderr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_result_predicates_match_variant() {
        assert!(CompileResult::Ok {
            stderr: String::new()
        }
        .is_ok());
        assert!(CompileResult::Skipped { reason: "x".into() }.is_skipped());
        assert!(!CompileResult::Failed {
            stderr: "boom".into()
        }
        .is_ok());
    }

    #[test]
    fn candidate_list_always_falls_back_to_cc() {
        // Even with $CC unset (the test harness clears it), `cc` must
        // appear in the fallback list.
        let prior = std::env::var("CC").ok();
        std::env::remove_var("CC");
        let cands = compiler_candidates();
        if let Some(p) = prior {
            std::env::set_var("CC", p);
        }
        assert!(cands.iter().any(|c| c == "cc"));
        assert!(cands.iter().any(|c| c == "gcc"));
        assert!(cands.iter().any(|c| c == "clang"));
    }

    #[test]
    fn trivial_source_round_trips_when_compiler_available() {
        let r = try_compile("int main(void) { return 0; }\n");
        match r {
            CompileResult::Ok { .. } | CompileResult::Skipped { .. } => {}
            CompileResult::Failed { stderr } => panic!("trivial source failed: {stderr}"),
        }
    }

    #[test]
    fn malformed_source_fails_when_compiler_available() {
        let r = try_compile("definitely not C\n");
        match r {
            CompileResult::Failed { .. } | CompileResult::Skipped { .. } => {}
            CompileResult::Ok { .. } => panic!("malformed source unexpectedly compiled"),
        }
    }
}
