//! Golden test for `dac --emit-cfg`.
//!
//! Closes the B2.1 done-when criterion at the CLI surface: running the
//! tool with `--emit-cfg` against a shared fixture produces a byte-stable
//! DOT file across re-runs (FR-28). The per-function CFG correctness
//! itself is covered by the hand-checked references in
//! `crates/dac-analysis/src/cfg.rs` (twelve `case_*` unit tests).
//!
//! What "stable" gates here:
//!
//! - The DOT sidecar (`<tmp>.cfg.dot`) is byte-identical across re-runs.
//! - The DOT output is valid-looking (contains `digraph "fn_…" {` and at
//!   least one `BB0` block — every function has an entry block).
//! - When the fixture has functions, the output also contains at least
//!   one edge label from the closed set (`fall` / `jmp` / `T` / `F`).

use std::fs;
use std::path::PathBuf;

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

fn run_emit_cfg(fixture: &str, output: &PathBuf) {
    let path = fixture_path(fixture);
    let mut cmd = Command::cargo_bin("dac").expect("dac binary present");
    cmd.arg("-O0")
        .arg("--emit-cfg")
        .arg("--output")
        .arg(output)
        .arg(&path)
        .assert()
        .success();
}

fn run_emit_cfg_twice(fixture: &str) -> (TempDir, String) {
    let dir = TempDir::new().expect("tempdir");
    let out_a = dir.path().join("a.listing");
    let out_b = dir.path().join("b.listing");
    run_emit_cfg(fixture, &out_a);
    run_emit_cfg(fixture, &out_b);
    let dot_a = fs::read_to_string(dir.path().join("a.listing.cfg.dot")).expect("dot a");
    let dot_b = fs::read_to_string(dir.path().join("b.listing.cfg.dot")).expect("dot b");
    assert_eq!(dot_a, dot_b, "{fixture}: CFG DOT drifted between two runs");
    (dir, dot_a)
}

#[test]
fn elf_emit_cfg_output_is_stable_across_reruns() {
    let (_dir, dot) = run_emit_cfg_twice("hello-x86_64");
    assert!(
        dot.contains("digraph \"fn_"),
        "DOT should contain at least one named function digraph: head={:?}",
        dot.lines().next().unwrap_or(""),
    );
    assert!(
        dot.contains("BB0 "),
        "DOT should contain at least one BB0 entry block declaration",
    );
    // hello-world programs have at least one block that ends in a real
    // edge. Any of the four canonical edge labels is acceptable.
    let has_any_edge = [
        "label=\"fall\"",
        "label=\"jmp\"",
        "label=\"T\"",
        "label=\"F\"",
    ]
    .iter()
    .any(|m| dot.contains(m));
    assert!(
        has_any_edge,
        "DOT should contain at least one classified edge"
    );
}

#[test]
fn pe_emit_cfg_output_is_stable_across_reruns() {
    let (_dir, dot) = run_emit_cfg_twice("hello-x86_64.exe");
    assert!(dot.contains("digraph \"fn_"));
    assert!(dot.contains("BB0 "));
}

#[test]
fn stripped_elf_emit_cfg_output_is_stable_across_reruns() {
    // Stripped ELF still has the entry point and prologue-discovered
    // functions, so the CFG export should be non-empty and stable.
    let (_dir, dot) = run_emit_cfg_twice("hello-x86_64-stripped");
    assert!(dot.contains("digraph \"fn_"));
}
