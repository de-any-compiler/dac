//! Integration tests for the B3.1 CLI surface: `--xrefs <subject>`
//! (FR-26, FR-31) and `--callgraph` (FR-27).
//!
//! The "done when" criterion for B3.1 is: `dac sample.elf --xrefs sym`
//! prints sane results. The tests below pin that the report
//!
//! 1) emits the right top-of-block markers (so a reader can `grep`),
//! 2) names the resolved subject (by name when known, by hex
//!    otherwise),
//! 3) records at least one direct-call site for a function we know is
//!    called in the corpus,
//! 4) records the binary's entry as an `EXP` xref against `_start`,
//! 5) lands the call graph DOT under `<output>.callgraph.dot` with the
//!    expected `digraph "callgraph_..."` header.

use std::path::PathBuf;

use assert_cmd::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn dac_xrefs_by_name_lists_direct_call_sites() {
    // `deregister_tm_clones` is called by `__do_global_dtors_aux` at
    // 0x1128 in the corpus ELF. The xrefs report must list that edge
    // with kind CALL and the caller annotated.
    let path = fixture_path("hello-x86_64");
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--xrefs", "deregister_tm_clones"])
        .arg(&path)
        .output()
        .expect("run dac --xrefs");
    assert!(output.status.success(), "dac --xrefs exits 0");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    assert!(stdout.contains(";; ---- xrefs (FR-26, FR-31) ----"));
    assert!(stdout.contains(";; subject:   deregister_tm_clones"));
    assert!(stdout.contains(";; address:   0x1090"));
    assert!(
        stdout.contains("CALL"),
        "expected a CALL xref edge in:\n{stdout}"
    );
    assert!(
        stdout.contains("from 0x1128"),
        "expected caller site 0x1128 in:\n{stdout}"
    );
    assert!(
        stdout.contains("__do_global_dtors_aux"),
        "expected caller name annotation in:\n{stdout}"
    );
}

#[test]
fn dac_xrefs_by_hex_address_matches_by_name_subject() {
    // 0x1090 == deregister_tm_clones; resolving by either spelling
    // should produce the same xref table (subject preamble differs).
    let path = fixture_path("hello-x86_64");
    let by_name = Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--xrefs", "deregister_tm_clones"])
        .arg(&path)
        .output()
        .expect("run dac --xrefs by-name");
    let by_hex = Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--xrefs", "0x1090"])
        .arg(&path)
        .output()
        .expect("run dac --xrefs by-hex");
    assert!(by_name.status.success());
    assert!(by_hex.status.success());
    let s_name = String::from_utf8(by_name.stdout).expect("utf-8");
    let s_hex = String::from_utf8(by_hex.stdout).expect("utf-8");
    // Both reports must mention the same CALL edge.
    for needle in ["CALL", "from 0x1128"] {
        assert!(s_name.contains(needle), "by-name missing {needle}");
        assert!(s_hex.contains(needle), "by-hex missing {needle}");
    }
}

#[test]
fn dac_xrefs_for_entry_records_external_export_edge() {
    // The binary's entry point flows in as an Export xref from the
    // synthetic external VA, so `--xrefs _start` lists `EXP` with the
    // `<external>` marker.
    let path = fixture_path("hello-x86_64");
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--xrefs", "_start"])
        .arg(&path)
        .output()
        .expect("run dac --xrefs _start");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    assert!(stdout.contains(";; subject:   _start"));
    assert!(stdout.contains("EXP"));
    assert!(stdout.contains("from <external>"));
}

#[test]
fn dac_xrefs_for_unknown_subject_emits_unresolved_block() {
    let path = fixture_path("hello-x86_64");
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--xrefs", "nope_does_not_exist"])
        .arg(&path)
        .output()
        .expect("run dac --xrefs nope");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    assert!(stdout.contains("(unresolved: no matching symbol or address)"));
}

#[test]
fn dac_callgraph_sidecar_lands_with_expected_dot_header() {
    let path = fixture_path("hello-x86_64");
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out_path = out_dir.path().join("dac-cg");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["-O0", "--callgraph", "--output"])
        .arg(&out_path)
        .arg(&path)
        .assert()
        .success();
    let cg_path = out_path.with_extension(""); // strip nothing
    let cg_path = PathBuf::from(format!("{}.callgraph.dot", cg_path.display()));
    let contents = std::fs::read_to_string(&cg_path)
        .unwrap_or_else(|e| panic!("read {} failed: {e}", cg_path.display()));
    assert!(contents.starts_with("digraph \"callgraph_"));
    assert!(
        contents.contains("shape=box"),
        "must list at least one function node"
    );
    assert!(contents.contains("style=solid"));
    assert!(contents.ends_with("}\n"));
}
