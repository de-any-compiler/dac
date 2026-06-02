//! End-to-end test for `dac -O1 --target c` (B2.8, FR-21).
//!
//! Runs the CLI against the shared x86-64 ELF fixture with the
//! `-O1 --target c` flag combination and asserts:
//!
//! 1. The CLI exits successfully.
//! 2. A `<output>.c` sidecar appears.
//! 3. The sidecar starts with the canonical dac C-reconstruction
//!    banner comment.
//! 4. The sidecar compiles cleanly when `cc` is on PATH; the test is
//!    silently skipped otherwise (matching the round-trip helper's
//!    skip-when-no-compiler contract).
//! 5. A second run produces a byte-identical `.c` sidecar — the
//!    backend is deterministic across re-runs (NFR-9).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as Sys, Stdio};

use assert_cmd::Command;
use tempfile::TempDir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run_o1_c(output: &PathBuf) {
    let path = fixture_path("hello-x86_64");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O1")
        .arg("--target")
        .arg("c")
        .arg("--output")
        .arg(output)
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn o1_target_c_emits_c_sidecar_with_banner() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_c(&out);

    let source_path = sidecar_with_suffix(&out, ".c");
    let source = fs::read_to_string(&source_path).expect("source sidecar");
    assert!(
        source.contains("dac --target c -O1 reconstruction"),
        "missing banner in:\n{source}"
    );
    // The fixture has at least one recovered function, so the unit
    // contains at least one `void <name>(void)` definition.
    assert!(
        source.contains("(void) {"),
        "no function definition in:\n{source}"
    );
}

#[test]
fn o1_target_c_round_trips_through_system_compiler() {
    // Best-effort: run `cc -x c -c -` on the emitted source. If no
    // compiler is on PATH, the test passes (mirrors `try_compile`'s
    // skip contract).
    let cc = match find_cc() {
        Some(p) => p,
        None => {
            eprintln!("o1_target_c_round_trips: no cc on PATH, skipping");
            return;
        }
    };
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_c(&out);

    let source_path = sidecar_with_suffix(&out, ".c");
    let source = fs::read_to_string(&source_path).expect("source sidecar");

    let mut child = Sys::new(&cc)
        .args(["-x", "c", "-c", "-", "-o", "/dev/null", "-w"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cc");
    use std::io::Write as _;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(source.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait cc");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("emitted C did not compile:\n--- source ---\n{source}\n--- stderr ---\n{stderr}");
    }
}

#[test]
fn o1_target_c_output_is_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    let out_a = dir.path().join("a.listing");
    let out_b = dir.path().join("b.listing");
    run_o1_c(&out_a);
    run_o1_c(&out_b);
    let a = fs::read_to_string(sidecar_with_suffix(&out_a, ".c")).expect("source a");
    let b = fs::read_to_string(sidecar_with_suffix(&out_b, ".c")).expect("source b");
    assert_eq!(a, b, "C source drifted between two runs");
}

fn sidecar_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    let mut s = base.to_path_buf().into_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

fn find_cc() -> Option<String> {
    if let Ok(cc) = std::env::var("CC") {
        if !cc.trim().is_empty() && which(&cc).is_some() {
            return Some(cc);
        }
    }
    for c in ["cc", "gcc", "clang"] {
        if which(c).is_some() {
            return Some(c.to_string());
        }
    }
    None
}

fn which(prog: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(prog);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
