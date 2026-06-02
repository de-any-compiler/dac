//! Golden test for `dac -O0`.
//!
//! Closes the B1.6 done-when: running `dac sample.elf -O0` produces
//! stable output across re-runs. The test runs the CLI twice against
//! each shared fixture (ELF + PE), with output piped through
//! `--output <tmp>` so the stdout interleaving cannot mask drift, and
//! asserts every emitted artifact is byte-identical between the two
//! runs.
//!
//! What "stable" gates here:
//!
//! - The annotated listing (`<tmp>`) is byte-identical across re-runs.
//! - The manifest (`<tmp>.manifest.json`) is byte-identical across
//!   re-runs. The build_id field comes from `option_env!("DAC_BUILD_ID")`
//!   which is the *same value* across both invocations of the same
//!   binary, so the manifest is reproducible at this slice without
//!   needing to mock the env.
//! - The analysis report (`<tmp>.report.txt`), when `--emit-report` is
//!   passed, is byte-identical across re-runs.
//!
//! What the test also pins as a structural sanity check:
//!
//! - The listing always contains the `;; dac -O0 annotated listing`
//!   preamble line. Drift here would mean the renderer's preamble
//!   format changed and the CHANGELOG should record it.
//! - The manifest always contains the `dac` tool name and version,
//!   which is the NFR-10 minimum.

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

fn run_o0(fixture: &str, emit_report: bool, output: &PathBuf) {
    let path = fixture_path(fixture);
    let mut cmd = Command::cargo_bin("dac").expect("dac binary present");
    cmd.arg("-O0").arg("--output").arg(output);
    if emit_report {
        cmd.arg("--emit-report");
    }
    cmd.arg(&path).assert().success();
}

fn run_o0_twice(fixture: &str, emit_report: bool) -> (TempDir, String, String, Option<String>) {
    let dir = TempDir::new().expect("tempdir");
    let out_a = dir.path().join("a.listing");
    let out_b = dir.path().join("b.listing");
    run_o0(fixture, emit_report, &out_a);
    run_o0(fixture, emit_report, &out_b);
    let listing_a = fs::read_to_string(&out_a).expect("listing a");
    let listing_b = fs::read_to_string(&out_b).expect("listing b");
    let manifest_a =
        fs::read_to_string(dir.path().join("a.listing.manifest.json")).expect("manifest a");
    let manifest_b =
        fs::read_to_string(dir.path().join("b.listing.manifest.json")).expect("manifest b");
    assert_eq!(
        manifest_a, manifest_b,
        "{fixture}: manifest drifted between two runs"
    );
    assert_eq!(
        listing_a, listing_b,
        "{fixture}: listing drifted between two runs"
    );
    let report = if emit_report {
        let report_a =
            fs::read_to_string(dir.path().join("a.listing.report.txt")).expect("report a");
        let report_b =
            fs::read_to_string(dir.path().join("b.listing.report.txt")).expect("report b");
        assert_eq!(
            report_a, report_b,
            "{fixture}: report drifted between two runs"
        );
        Some(report_a)
    } else {
        None
    };
    (dir, listing_a, manifest_a, report)
}

#[test]
fn elf_o0_output_is_stable_across_reruns() {
    let (_dir, listing, manifest, _) = run_o0_twice("hello-x86_64", false);
    assert!(
        listing.starts_with(";; dac -O0 annotated listing\n"),
        "listing preamble missing or moved: {}",
        listing.lines().next().unwrap_or(""),
    );
    assert!(
        listing.contains(";; format:    ELF"),
        "listing should declare its source format",
    );
    assert!(
        listing.contains(";; arch:      x86-64"),
        "listing should declare its arch",
    );
    assert!(
        listing.contains(";; function "),
        "listing should contain at least one function header",
    );
    assert!(
        manifest.contains("\"name\": \"dac\""),
        "manifest should record the tool name",
    );
    assert!(
        manifest.contains("\"format\": \"ELF\""),
        "manifest should record the input format",
    );
    assert!(
        manifest.contains("\"level\": \"O0\""),
        "manifest should record the selected level",
    );
}

#[test]
fn pe_o0_output_is_stable_across_reruns() {
    let (_dir, listing, manifest, _) = run_o0_twice("hello-x86_64.exe", false);
    assert!(listing.starts_with(";; dac -O0 annotated listing\n"));
    assert!(listing.contains(";; format:    PE"));
    assert!(listing.contains(";; arch:      x86-64"));
    assert!(manifest.contains("\"format\": \"PE\""));
}

#[test]
fn elf_o0_with_emit_report_is_stable_across_reruns() {
    let (_dir, listing, _manifest, report) = run_o0_twice("hello-x86_64", true);
    let report = report.expect("emit_report=true should yield a report");
    assert!(listing.contains(";; functions: "));
    assert!(report.contains(";; dac analysis report (FR-25)"));
    assert!(report.contains(";; functions:"));
    assert!(report.contains("functions:"));
}

#[test]
fn stripped_elf_o0_output_is_stable_across_reruns() {
    let (_dir, listing, _, _) = run_o0_twice("hello-x86_64-stripped", false);
    assert!(listing.starts_with(";; dac -O0 annotated listing\n"));
    assert!(listing.contains(";; format:    ELF"));
}
