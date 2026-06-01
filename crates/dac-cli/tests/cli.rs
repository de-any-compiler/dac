//! End-to-end tests for the `dac` binary.
//!
//! Closes the B0.2 done-when (dac returns a clean error on random input,
//! never crashes) and the B0.5 done-when (`dac --help` matches a snapshot
//! that mirrors spec §10.1). After B1.1 the `success`-path tests use a
//! real ELF fixture so they exercise the full parser; B1.2 adds a PE
//! fixture path to cover the second format dispatch.

use std::io::Write;
use std::path::PathBuf;

use assert_cmd::Command;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tempfile::NamedTempFile;

const HELP_SNAPSHOT: &str = include_str!("snapshots/help.txt");

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn elf_fixture() -> PathBuf {
    fixture_path("hello-x86_64")
}

fn pe_fixture() -> PathBuf {
    fixture_path("hello-x86_64.exe")
}

#[test]
fn dac_returns_clean_error_on_random_input() {
    let mut rng = StdRng::seed_from_u64(0xDAC0_5EED);
    let mut buf = vec![0u8; 4096];
    rng.fill(&mut buf[..]);

    let mut file = NamedTempFile::new().expect("create tempfile");
    file.write_all(&buf).expect("write tempfile");

    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn dac_parses_elf_fixture() {
    let path = elf_fixture();
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn dac_parses_pe_fixture() {
    let path = pe_fixture();
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn dac_rejects_elf_magic_without_valid_header() {
    let mut buf = vec![0u8; 64];
    buf[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
    let mut file = NamedTempFile::new().expect("create tempfile");
    file.write_all(&buf).expect("write tempfile");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn dac_with_no_args_prints_usage_and_exits_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_help_exits_zero() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn dac_help_matches_snapshot() {
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--help")
        .output()
        .expect("run dac --help");
    assert!(output.status.success(), "dac --help should exit 0");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(stdout, HELP_SNAPSHOT);
}

#[test]
fn dac_short_help_matches_snapshot() {
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-h")
        .output()
        .expect("run dac -h");
    assert!(output.status.success(), "dac -h should exit 0");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(stdout, HELP_SNAPSHOT);
}

#[test]
fn dac_version_prints_version_and_build_id() {
    let output = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--version")
        .output()
        .expect("run dac --version");
    assert!(output.status.success(), "dac --version should exit 0");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let expected_prefix = format!("dac {} (", env!("CARGO_PKG_VERSION"));
    assert!(
        stdout.starts_with(&expected_prefix),
        "stdout {stdout:?} should start with {expected_prefix:?}"
    );
    assert!(
        stdout.trim_end().ends_with(')'),
        "stdout {stdout:?} should end with `)` after trimming"
    );
}

#[test]
fn dac_short_version_matches_long_version() {
    let long = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--version")
        .output()
        .expect("run dac --version");
    let short = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-V")
        .output()
        .expect("run dac -V");
    assert!(long.status.success());
    assert!(short.status.success());
    assert_eq!(long.stdout, short.stdout);
}

#[test]
fn dac_accepts_deterministic_flag() {
    let path = elf_fixture();
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--deterministic")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn dac_rejects_unknown_flag_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--nonsense")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_accepts_full_spec_flag_surface() {
    let path = elf_fixture();
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args([
            "-O2",
            "--arch",
            "x86-64",
            "--format",
            "elf",
            "--target",
            "c",
            "--output",
            "/tmp/dac-test-out",
            "--emit-ir",
            "--emit-cfg",
            "--emit-report",
            "--emit-annotations",
            "--no-ai",
            "--ai-provider",
            "local",
            "--deterministic",
            "--threads",
            "4",
            "--json",
            "--debug",
            "--plugin",
            "/tmp/dac-test-plugin.so",
        ])
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn dac_accepts_each_opt_level() {
    let path = elf_fixture();
    for level in ["-O0", "-O1", "-O2", "-O3"] {
        Command::cargo_bin("dac")
            .expect("dac binary present")
            .arg(level)
            .arg(&path)
            .assert()
            .success();
    }
}

#[test]
fn dac_accepts_each_format_value() {
    let path = elf_fixture();
    for fmt in ["elf", "pe", "mach-o", "auto"] {
        Command::cargo_bin("dac")
            .expect("dac binary present")
            .args(["--format", fmt])
            .arg(&path)
            .assert()
            .success();
    }
}

#[test]
fn dac_rejects_invalid_format_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--format", "nonsense"])
        .arg("/dev/null")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_rejects_invalid_target_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--target", "fortran"])
        .arg("/dev/null")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_rejects_zero_threads_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--threads", "0"])
        .arg("/dev/null")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_rejects_nonnumeric_threads_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .args(["--threads", "lots"])
        .arg("/dev/null")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dac_rejects_missing_value_with_exit_2() {
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--arch")
        .assert()
        .failure()
        .code(2);
}
