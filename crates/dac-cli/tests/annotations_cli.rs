//! End-to-end test for `dac --emit-annotations` and the `--debug`
//! evidence trail (B3.4, FR-19 / FR-23 / FR-25, spec §10.4 / §12).
//!
//! Closes the B3.4 done-when "every name and type in emitted C is
//! traceable through the evidence graph in `--debug`". The test:
//!
//! 1. runs `dac -O1 --target c --emit-annotations <fixture> --output <tmp>`,
//!    and asserts the `<tmp>.annot.json` sidecar exists with the
//!    minimum-required structural fields;
//! 2. re-runs the same command and asserts the sidecar is
//!    byte-identical (deterministic, NFR-9);
//! 3. runs `dac -O1 --target c --debug <fixture> --output <tmp>` and
//!    asserts the emitted C unit's per-function leading comments
//!    contain the "Why this name?" and "Why this return type?"
//!    blocks (spec §12 trace mode).

use std::fs;
use std::path::{Path, PathBuf};

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

fn run(args: &[&str], output: &Path, debug: bool, emit_annotations: bool) {
    let path = fixture_path("hello-x86_64");
    let mut cmd = Command::cargo_bin("dac").expect("dac binary present");
    cmd.args(args).arg("--output").arg(output);
    if debug {
        cmd.arg("--debug");
    }
    if emit_annotations {
        cmd.arg("--emit-annotations");
    }
    cmd.arg(&path).assert().success();
}

fn sidecar(base: &Path, suffix: &str) -> PathBuf {
    let mut s = base.to_path_buf().into_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

#[test]
fn emit_annotations_writes_a_structured_sidecar() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run(&["-O1", "--target", "c"], &out, false, true);

    let annot = fs::read_to_string(sidecar(&out, ".annot.json")).expect("annot.json sidecar");
    assert!(annot.starts_with("{\n"), "JSON should start with {{");
    assert!(annot.contains("\"tool\""));
    assert!(annot.contains("\"input\""));
    assert!(annot.contains("\"settings\""));
    assert!(annot.contains("\"evidence\""));
    assert!(annot.contains("\"functions\""));
    assert!(annot.contains("\"notes\""));
    assert!(
        annot.contains("\"name\": \"dac\""),
        "tool name must be present"
    );
    assert!(
        annot.contains("\"format\": \"ELF\""),
        "input format must be present"
    );
    // The fixture has recovered functions, so the surface fact set
    // should mention at least one `name` block with a confidence
    // record.
    assert!(annot.contains("\"return_type\""));
    assert!(annot.contains("\"confidence\""));
    assert!(annot.contains("\"explanation\""));
    assert!(annot.contains("\"signals\""));
}

#[test]
fn emit_annotations_output_is_byte_stable_across_reruns() {
    let dir = TempDir::new().expect("tempdir");
    let out_a = dir.path().join("a.listing");
    let out_b = dir.path().join("b.listing");
    run(&["-O1", "--target", "c"], &out_a, false, true);
    run(&["-O1", "--target", "c"], &out_b, false, true);

    let a = fs::read_to_string(sidecar(&out_a, ".annot.json")).expect("a annot.json");
    let b = fs::read_to_string(sidecar(&out_b, ".annot.json")).expect("b annot.json");
    assert_eq!(a, b, "annotations sidecar drifted between two runs");
}

#[test]
fn debug_mode_embeds_evidence_trail_in_c_function_comments() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run(&["-O1", "--target", "c"], &out, true, false);

    let source = fs::read_to_string(sidecar(&out, ".c")).expect(".c sidecar");
    // The fixture has at least one recovered function, so the trail
    // text appears at least once.
    assert!(
        source.contains("Why this name?"),
        "expected `Why this name?` in emitted C:\n{source}"
    );
    assert!(
        source.contains("Why this return type?"),
        "expected `Why this return type?` in emitted C:\n{source}"
    );
    assert!(
        source.contains("explanation:"),
        "expected per-fact explanation line in:\n{source}"
    );
    assert!(
        source.contains("evidence:"),
        "expected per-fact evidence line in:\n{source}"
    );
}

#[test]
fn debug_mode_emitted_c_still_compiles() {
    // The `--debug` augmentation only touches leading comments inside
    // `/* … */` blocks, so the C unit must still round-trip through
    // the system compiler. Mirrors the contract in o1_target_c.rs.
    let cc = match find_cc() {
        Some(p) => p,
        None => {
            eprintln!("debug_mode_emitted_c_still_compiles: no cc on PATH, skipping");
            return;
        }
    };
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run(&["-O1", "--target", "c"], &out, true, false);

    let source = fs::read_to_string(sidecar(&out, ".c")).expect(".c sidecar");

    use std::io::Write as _;
    use std::process::{Command as Sys, Stdio};
    let mut child = Sys::new(&cc)
        // `-Dmain=__dac_main__` dodges macOS clang's hard error
        // ("first parameter of 'main' (argument count) must be of
        // type 'int'") on the recovered `int64_t main(int64_t)`
        // signature. The emitted source on disk is unchanged; only
        // the preprocessor sees the rename, so the compile probe
        // stops tripping clang's special-case rule. Linux gcc/clang
        // accept the define silently.
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
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(source.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait cc");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("debug-augmented C did not compile:\n--- source ---\n{source}\n--- stderr ---\n{stderr}");
    }
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
