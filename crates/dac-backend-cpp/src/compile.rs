//! Round-trip compilation check for C++ (ARCHITECTURE.md §8).
//!
//! Mirrors [`dac_backend_c::compile`]: a thin wrapper around the host
//! C++ compiler that the round-trip tests use to gate the corpus,
//! falling back to [`CompileResult::Skipped`] when no `c++` is on
//! `PATH` so unit tests stay green on a toolchain-free box. The
//! difference is solely the language: `-x c++` instead of `-x c`, and
//! the candidate list is `$CXX` / `c++` / `g++` / `clang++`.
//!
//! The same scope caveats apply: this checks "the emitted source
//! parses and type-checks", not "the result is semantically equivalent
//! to the binary it came from". Equivalence is the smoke-test tier
//! that B3.5's done-when criterion does not include.

use std::ffi::OsStr;
use std::io::Write as _;
use std::process::{Command, Stdio};

/// Outcome of a round-trip compile attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileResult {
    /// Compilation succeeded.
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

/// Try to compile `source` with the system C++ compiler.
#[must_use]
pub fn try_compile(source: &str) -> CompileResult {
    let candidates = compiler_candidates();
    let mut tried = Vec::new();
    for cxx in &candidates {
        match run_compiler(cxx, source) {
            RunOutcome::Ok { stderr } => return CompileResult::Ok { stderr },
            RunOutcome::Failed { stderr } => return CompileResult::Failed { stderr },
            RunOutcome::NotFound => tried.push(cxx.clone()),
        }
    }
    CompileResult::Skipped {
        reason: format!("no C++ compiler available (tried: {})", tried.join(", ")),
    }
}

fn compiler_candidates() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(env_cxx) = std::env::var("CXX") {
        if !env_cxx.trim().is_empty() {
            v.push(env_cxx);
        }
    }
    for fallback in ["c++", "g++", "clang++"] {
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

fn run_compiler(cxx: &str, source: &str) -> RunOutcome {
    let args: &[&OsStr] = &[
        OsStr::new("-x"),
        OsStr::new("c++"),
        OsStr::new("-std=c++17"),
        OsStr::new("-c"),
        OsStr::new("-"),
        OsStr::new("-o"),
        OsStr::new("/dev/null"),
        OsStr::new("-w"),
    ];
    let mut child = match Command::new(cxx)
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
                stderr: format!("failed to launch {cxx}: {err}"),
            };
        }
    };
    if let Some(stdin) = child.stdin.as_mut() {
        if let Err(err) = stdin.write_all(source.as_bytes()) {
            return RunOutcome::Failed {
                stderr: format!("failed to write to {cxx} stdin: {err}"),
            };
        }
    }
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(err) => {
            return RunOutcome::Failed {
                stderr: format!("failed to wait on {cxx}: {err}"),
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
    fn candidate_list_always_falls_back_to_cxx() {
        let prior = std::env::var("CXX").ok();
        std::env::remove_var("CXX");
        let cands = compiler_candidates();
        if let Some(p) = prior {
            std::env::set_var("CXX", p);
        }
        assert!(cands.iter().any(|c| c == "c++"));
        assert!(cands.iter().any(|c| c == "g++"));
        assert!(cands.iter().any(|c| c == "clang++"));
    }

    #[test]
    fn trivial_source_round_trips_when_compiler_available() {
        let r = try_compile("int main() { return 0; }\n");
        match r {
            CompileResult::Ok { .. } | CompileResult::Skipped { .. } => {}
            CompileResult::Failed { stderr } => panic!("trivial source failed: {stderr}"),
        }
    }

    #[test]
    fn malformed_source_fails_when_compiler_available() {
        let r = try_compile("class // unterminated\n");
        match r {
            CompileResult::Failed { .. } | CompileResult::Skipped { .. } => {}
            CompileResult::Ok { .. } => panic!("malformed source unexpectedly compiled"),
        }
    }

    #[test]
    fn class_with_virtual_dtor_round_trips() {
        let src = "\
#include <cstdint>
class Dog {
public:
    virtual ~Dog() {}
    virtual std::int32_t speak() const { return 0; }
};
";
        let r = try_compile(src);
        match r {
            CompileResult::Ok { .. } | CompileResult::Skipped { .. } => {}
            CompileResult::Failed { stderr } => panic!("class did not compile: {stderr}"),
        }
    }
}
