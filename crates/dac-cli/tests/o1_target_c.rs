//! End-to-end test for `dac -O<n> --target c` (B2.8, FR-21).
//!
//! Runs the CLI against the shared x86-64 ELF fixture with the
//! `-O<n> --target c` flag combination and asserts:
//!
//! 1. The CLI exits successfully.
//! 2. A `<output>.c` sidecar appears.
//! 3. The sidecar starts with the canonical dac C-reconstruction
//!    banner comment whose `-O<n>` level matches the invocation
//!    (B3.31 — the banner was previously hardcoded to `-O1`).
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

fn run_c(opt: &str, output: &PathBuf) {
    let path = fixture_path("hello-x86_64");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(opt)
        .arg("--target")
        .arg("c")
        .arg("--output")
        .arg(output)
        .arg(&path)
        .assert()
        .success();
}

fn run_o1_c(output: &PathBuf) {
    run_c("-O1", output);
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

/// B3.31: the reconstruction banner reflects the active `-O<n>`
/// invocation rather than the historical hardcoded `-O1`. The done-when
/// for B3.31 is "invocation at `-O3` prints `-O3` in the header"
/// (FR-21). We also assert `-O2` to confirm every non-`-O1` level
/// reaches the format string the same way.
#[test]
fn b3_31_banner_reflects_active_opt_level() {
    for opt in ["-O2", "-O3"] {
        let dir = TempDir::new().expect("tempdir");
        let out = dir.path().join("a.listing");
        run_c(opt, &out);
        let source = fs::read_to_string(sidecar_with_suffix(&out, ".c")).expect("source sidecar");
        let banner = format!("dac --target c {opt} reconstruction");
        assert!(source.contains(&banner), "missing `{banner}` in:\n{source}");
        // And the previously hardcoded level must not leak when the
        // run is at a different level.
        assert!(
            !source.contains("dac --target c -O1 reconstruction"),
            "stale -O1 banner leaked at {opt}:\n{source}"
        );
    }
}

/// B3.32: a constant pointer operand whose value matches an extracted
/// `BinaryModel::strings` entry surfaces as a `"…"` literal in the C
/// output instead of a bare integer address. The `hello-x86_64`
/// fixture's `.rodata` contains `"hello\n\0"` at virtual address
/// `0x2004`; the recovered `write(1, 0x2004, 6)` therefore renders
/// with the literal next to the call. The literal is wrapped in an
/// explicit `(int64_t)` cast so it keeps the integer slot the
/// surrounding call cast expects (round-trip compile gate stays
/// green).
#[test]
fn b3_32_string_literal_surfaces_in_write_call() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_c(&out);

    let source = fs::read_to_string(sidecar_with_suffix(&out, ".c")).expect("source sidecar");
    assert!(
        source.contains("\"hello\\n\""),
        "expected `\"hello\\n\"` literal in:\n{source}"
    );
    assert!(
        source.contains("((int64_t)(\"hello\\n\"))"),
        "expected explicit `(int64_t)` cast around the literal in:\n{source}"
    );
    // The bare address constant must no longer leak; if it does, the
    // string-substitution didn't fire.
    assert!(
        !source.contains("8196LL"),
        "stale `8196LL` constant leaked next to the recovered write call:\n{source}"
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
        // `-Dmain=__dac_main__` dodges macOS clang's hard error on
        // the recovered `int64_t main(int64_t)` signature. See the
        // sibling probe in `annotations_cli.rs` for the full
        // rationale.
        .args([
            "-x",
            "c",
            "-c",
            "-",
            "-o",
            "/dev/null",
            "-w",
            "-Dmain=__dac_main__",
        ])
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
