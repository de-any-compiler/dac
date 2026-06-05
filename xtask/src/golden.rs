//! Golden corpus harness (B2.9, NFR-9, spec §16).
//!
//! Runs each `Case` through the `dac` CLI under the workspace root and
//! compares the resulting sidecars against the recorded bytes under
//! `tests/golden/<case_name>/<output_file>`. `Mode::Check` fails on the
//! first drift; `Mode::Update` overwrites the recorded bytes so a
//! developer can refresh the corpus after an intentional change.
//!
//! The corpus is declared in the static [`CASES`] array — adding a case
//! is a one-line change plus `cargo xtask golden update` to seed the
//! expected files. The array shape catches typos at compile time and
//! keeps xtask dependency-free; if the corpus ever grows past a handful
//! of cases the array can be lifted into a TOML manifest behind a
//! workspace-`toml` dep without changing the public xtask surface.
//!
//! Determinism contract:
//!
//! - `dac` itself is byte-deterministic at every supported
//!   `--target` / `-O` combination
//!   (`crates/dac-cli/tests/o0_golden.rs`,
//!   `crates/dac-cli/tests/o1_target_c.rs`).
//! - The CLI is invoked with workspace-relative fixture paths so the
//!   manifest's `input.path` field does not depend on the developer's
//!   home directory.
//! - The `dac` binary is built into `target/debug/dac` once per
//!   `xtask golden` run and then reused for every case.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What the harness should do when a sidecar differs from the
/// recorded golden.
#[derive(Copy, Clone, Debug)]
pub(crate) enum Mode {
    /// Diff actual vs. expected; fail on any drift.
    Check,
    /// Overwrite (or create) the golden file with the actual output.
    Update,
}

/// One golden case: a fixture + CLI flag set + the sidecars to capture.
/// The fixture name resolves to `tests/fixtures/<fixture>` from the
/// workspace root; the case name maps to `tests/golden/<name>/`.
struct Case {
    name: &'static str,
    fixture: &'static str,
    args: &'static [&'static str],
    outputs: &'static [OutputKind],
}

#[derive(Copy, Clone)]
enum OutputKind {
    Listing,
    Manifest,
    Report,
    Cfg,
    Source,
    CppSource,
}

impl OutputKind {
    /// The file name under `tests/golden/<case>/` — purely a label;
    /// the dac CLI does not produce a file with this name.
    fn file_name(self) -> &'static str {
        match self {
            Self::Listing => "listing.txt",
            Self::Manifest => "manifest.json",
            Self::Report => "report.txt",
            Self::Cfg => "cfg.dot",
            Self::Source => "source.c",
            Self::CppSource => "source.cpp",
        }
    }

    /// Where the `dac` invocation writes this output given the
    /// `--output` base path. Mirrors the contract documented in
    /// `crates/dac-cli/src/main.rs::emit_outputs` — any change to that
    /// function must update both sides in lockstep.
    fn produced_path(self, base: &Path) -> PathBuf {
        let suffix = match self {
            Self::Listing => "",
            Self::Manifest => ".manifest.json",
            Self::Report => ".report.txt",
            Self::Cfg => ".cfg.dot",
            Self::Source => ".c",
            Self::CppSource => ".cpp",
        };
        if suffix.is_empty() {
            base.to_path_buf()
        } else {
            let mut s: OsString = base.as_os_str().to_owned();
            s.push(suffix);
            PathBuf::from(s)
        }
    }
}

/// The corpus. Adding a case here + running `cargo xtask golden update`
/// seeds the recorded bytes; CI then gates drift via `golden check`.
const CASES: &[Case] = &[
    Case {
        name: "hello-elf-o0",
        fixture: "hello-x86_64",
        args: &["-O0"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest],
    },
    Case {
        name: "hello-elf-o0-report",
        fixture: "hello-x86_64",
        args: &["-O0", "--emit-report"],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::Report,
        ],
    },
    Case {
        name: "hello-elf-o0-cfg",
        fixture: "hello-x86_64",
        args: &["-O0", "--emit-cfg"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest, OutputKind::Cfg],
    },
    Case {
        name: "hello-elf-o1-c",
        fixture: "hello-x86_64",
        args: &["-O1", "--target", "c"],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::Source,
        ],
    },
    Case {
        name: "hello-elf-o1-c-hints",
        fixture: "hello-x86_64",
        args: &[
            "-O1",
            "--target",
            "c",
            "--emit-report",
            "--hints",
            "tests/fixtures/hello-x86_64.hints.toml",
        ],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::Source,
            OutputKind::Report,
        ],
    },
    Case {
        name: "hello-elf-stripped-o0",
        fixture: "hello-x86_64-stripped",
        args: &["-O0"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest],
    },
    Case {
        name: "hello-pe-o0",
        fixture: "hello-x86_64.exe",
        args: &["-O0"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest],
    },
    Case {
        name: "hello-pe-o1-c",
        fixture: "hello-x86_64.exe",
        args: &["-O1", "--target", "c"],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::Source,
        ],
    },
    Case {
        name: "cpp-hierarchy-o1-cpp",
        fixture: "cpp-hierarchy-x86_64",
        args: &["-O1", "--target", "cpp"],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::CppSource,
        ],
    },
    Case {
        name: "libsample-o0",
        fixture: "libsample.so",
        args: &["-O0"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest],
    },
    // B3.13: a small PIE that issues a `syscall` directly from
    // user code. The convention pass should pick the
    // `sysv-amd64-syscall` reading on the function holding the
    // opcode; the manifest's `convention` field captures the
    // winning candidate per function so the golden gates the
    // signal against drift.
    Case {
        name: "syscall-hello-elf-o1-c",
        fixture: "syscall-hello-x86_64",
        args: &["-O1", "--target", "c"],
        outputs: &[
            OutputKind::Listing,
            OutputKind::Manifest,
            OutputKind::Source,
        ],
    },
    Case {
        name: "sample-dll-o0",
        fixture: "sample.dll",
        args: &["-O0"],
        outputs: &[OutputKind::Listing, OutputKind::Manifest],
    },
];

pub(crate) fn run(mode: Mode) -> Result<(), String> {
    let root = workspace_root();

    let mut build = crate::cargo();
    build.args(["build", "--quiet", "--bin", "dac"]);
    crate::run("cargo build --bin dac", build)?;
    let dac = dac_binary_path(&root);
    if !dac.is_file() {
        return Err(format!("dac binary not found at {}", dac.display()));
    }

    let golden_root = root.join("tests").join("golden");
    let scratch = root.join("target").join("xtask").join("golden");
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch)
            .map_err(|e| format!("clearing {}: {e}", scratch.display()))?;
    }
    std::fs::create_dir_all(&scratch)
        .map_err(|e| format!("creating {}: {e}", scratch.display()))?;

    let mut failures: Vec<String> = Vec::new();
    let mut updated = 0usize;
    let mut matched = 0usize;

    eprintln!("xtask: golden — {} case(s)", CASES.len());

    for case in CASES {
        let case_dir = scratch.join(case.name);
        std::fs::create_dir_all(&case_dir)
            .map_err(|e| format!("creating {}: {e}", case_dir.display()))?;
        let out_base = case_dir.join("out");

        let mut cmd = Command::new(&dac);
        cmd.current_dir(&root);
        for a in case.args {
            cmd.arg(a);
        }
        cmd.arg("--output").arg(&out_base);
        // Workspace-relative fixture path keeps the manifest's
        // `input.path` portable across developers.
        let fixture_rel = format!("tests/fixtures/{}", case.fixture);
        cmd.arg(&fixture_rel);

        let status = cmd
            .status()
            .map_err(|e| format!("case `{}`: spawning dac: {e}", case.name))?;
        if !status.success() {
            failures.push(format!("case `{}`: dac exited with {status}", case.name));
            continue;
        }

        for kind in case.outputs {
            let actual_path = kind.produced_path(&out_base);
            let actual = match std::fs::read(&actual_path) {
                Ok(b) => b,
                Err(e) => {
                    failures.push(format!(
                        "case `{}`: dac did not produce {}: {e}",
                        case.name,
                        actual_path.display()
                    ));
                    continue;
                }
            };
            let expected_path = golden_root.join(case.name).join(kind.file_name());
            match mode {
                Mode::Check => match std::fs::read(&expected_path) {
                    Ok(expected) => {
                        if expected != actual {
                            failures.push(render_drift(
                                case.name,
                                kind.file_name(),
                                &expected,
                                &actual,
                                &relative_to(&expected_path, &root),
                                &actual_path,
                            ));
                        } else {
                            matched += 1;
                        }
                    }
                    Err(_) => {
                        failures.push(format!(
                            "case `{}` / `{}`: missing golden at {} (run `cargo xtask golden update`)",
                            case.name,
                            kind.file_name(),
                            relative_to(&expected_path, &root).display()
                        ));
                    }
                },
                Mode::Update => {
                    if let Some(p) = expected_path.parent() {
                        std::fs::create_dir_all(p)
                            .map_err(|e| format!("creating {}: {e}", p.display()))?;
                    }
                    let changed = match std::fs::read(&expected_path) {
                        Ok(b) => b != actual,
                        Err(_) => true,
                    };
                    std::fs::write(&expected_path, &actual).map_err(|e| {
                        format!(
                            "writing {}: {e}",
                            relative_to(&expected_path, &root).display()
                        )
                    })?;
                    if changed {
                        updated += 1;
                    } else {
                        matched += 1;
                    }
                }
            }
        }
    }

    match mode {
        Mode::Check => {
            if !failures.is_empty() {
                for f in &failures {
                    eprintln!("{f}");
                }
                eprintln!(
                    "xtask: golden — {} drift(s), {} match(es) across {} case(s)",
                    failures.len(),
                    matched,
                    CASES.len()
                );
                eprintln!(
                    "xtask: hint: run `cargo xtask golden update` to refresh after intentional changes."
                );
                return Err(format!("{} golden output(s) drifted", failures.len()));
            }
            eprintln!(
                "xtask: golden — all {matched} output(s) across {} case(s) match",
                CASES.len()
            );
        }
        Mode::Update => {
            if !failures.is_empty() {
                for f in &failures {
                    eprintln!("{f}");
                }
                eprintln!(
                    "xtask: golden update — {} failure(s), {updated} updated, {matched} unchanged",
                    failures.len()
                );
                return Err(format!(
                    "{} golden output(s) failed to update",
                    failures.len()
                ));
            }
            eprintln!(
                "xtask: golden update — {updated} updated, {matched} unchanged, across {} case(s)",
                CASES.len()
            );
        }
    }
    Ok(())
}

fn dac_binary_path(root: &Path) -> PathBuf {
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let mut bin = target_dir.join("debug").join("dac");
    if cfg!(windows) {
        bin.set_extension("exe");
    }
    bin
}

fn workspace_root() -> PathBuf {
    let xtask_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_manifest
        .parent()
        .expect("xtask is a sub-directory of the workspace root")
        .to_path_buf()
}

fn relative_to(p: &Path, root: &Path) -> PathBuf {
    p.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| p.to_path_buf())
}

fn render_drift(
    case_name: &str,
    output_name: &str,
    expected: &[u8],
    actual: &[u8],
    expected_path: &Path,
    actual_path: &Path,
) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(s, "case `{case_name}` / `{output_name}` drifted:");
    let _ = writeln!(
        s,
        "  expected: {} ({} bytes)",
        expected_path.display(),
        expected.len()
    );
    let _ = writeln!(
        s,
        "  actual:   {} ({} bytes)",
        actual_path.display(),
        actual.len()
    );
    match (std::str::from_utf8(expected), std::str::from_utf8(actual)) {
        (Ok(e), Ok(a)) => {
            let mut exp_lines = e.lines();
            let mut act_lines = a.lines();
            let mut idx = 0usize;
            loop {
                idx += 1;
                match (exp_lines.next(), act_lines.next()) {
                    (Some(l), Some(r)) if l == r => continue,
                    (Some(l), Some(r)) => {
                        let _ = writeln!(s, "  first diff at line {idx}:");
                        let _ = writeln!(s, "    - {l}");
                        let _ = writeln!(s, "    + {r}");
                        return s;
                    }
                    (Some(l), None) => {
                        let _ = writeln!(s, "  expected has extra line {idx}:");
                        let _ = writeln!(s, "    - {l}");
                        return s;
                    }
                    (None, Some(r)) => {
                        let _ = writeln!(s, "  actual has extra line {idx}:");
                        let _ = writeln!(s, "    + {r}");
                        return s;
                    }
                    (None, None) => {
                        let _ = writeln!(
                            s,
                            "  contents identical line-by-line but differ in trailing bytes"
                        );
                        return s;
                    }
                }
            }
        }
        _ => {
            for (i, (l, r)) in expected.iter().zip(actual.iter()).enumerate() {
                if l != r {
                    let _ = writeln!(
                        s,
                        "  first diff at byte {i} (expected {l:#04x}, actual {r:#04x})"
                    );
                    return s;
                }
            }
            let _ = writeln!(
                s,
                "  contents differ past byte {} (expected {} bytes, actual {} bytes)",
                expected.len().min(actual.len()),
                expected.len(),
                actual.len()
            );
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn case_names_are_unique() {
        let mut seen: HashSet<&'static str> = HashSet::new();
        for case in CASES {
            assert!(seen.insert(case.name), "duplicate case name: {}", case.name);
        }
    }

    #[test]
    fn case_outputs_are_non_empty_and_unique() {
        for case in CASES {
            assert!(
                !case.outputs.is_empty(),
                "case `{}` has no outputs",
                case.name
            );
            let mut seen: HashSet<&'static str> = HashSet::new();
            for k in case.outputs {
                let name = k.file_name();
                assert!(
                    seen.insert(name),
                    "case `{}` has duplicate output `{}`",
                    case.name,
                    name
                );
            }
        }
    }

    #[test]
    fn output_paths_match_dac_sidecar_format() {
        let base = PathBuf::from("/tmp/out");
        assert_eq!(
            OutputKind::Listing.produced_path(&base),
            PathBuf::from("/tmp/out")
        );
        assert_eq!(
            OutputKind::Manifest.produced_path(&base),
            PathBuf::from("/tmp/out.manifest.json")
        );
        assert_eq!(
            OutputKind::Report.produced_path(&base),
            PathBuf::from("/tmp/out.report.txt")
        );
        assert_eq!(
            OutputKind::Cfg.produced_path(&base),
            PathBuf::from("/tmp/out.cfg.dot")
        );
        assert_eq!(
            OutputKind::Source.produced_path(&base),
            PathBuf::from("/tmp/out.c")
        );
        assert_eq!(
            OutputKind::CppSource.produced_path(&base),
            PathBuf::from("/tmp/out.cpp")
        );
    }

    #[test]
    fn case_fixtures_exist() {
        let root = workspace_root();
        for case in CASES {
            let p = root.join("tests").join("fixtures").join(case.fixture);
            assert!(
                p.is_file(),
                "case `{}`: fixture {} missing",
                case.name,
                p.display()
            );
        }
    }

    #[test]
    fn render_drift_points_to_first_changed_line() {
        let exp = b"alpha\nbeta\ngamma\n";
        let act = b"alpha\nBETA\ngamma\n";
        let s = render_drift(
            "case-x",
            "listing.txt",
            exp,
            act,
            Path::new("tests/golden/case-x/listing.txt"),
            Path::new("/tmp/out"),
        );
        assert!(s.contains("first diff at line 2"), "report:\n{s}");
        assert!(s.contains("- beta"), "report:\n{s}");
        assert!(s.contains("+ BETA"), "report:\n{s}");
    }

    #[test]
    fn render_drift_reports_extra_lines() {
        let exp = b"alpha\nbeta\n";
        let act = b"alpha\nbeta\ngamma\n";
        let s = render_drift(
            "case-y",
            "listing.txt",
            exp,
            act,
            Path::new("tests/golden/case-y/listing.txt"),
            Path::new("/tmp/out"),
        );
        assert!(s.contains("actual has extra line"), "report:\n{s}");
        assert!(s.contains("+ gamma"), "report:\n{s}");
    }
}
