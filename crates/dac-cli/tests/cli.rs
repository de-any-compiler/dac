//! End-to-end tests for the `dac` binary.
//!
//! Closes the B0.2 done-when: dac must return a clean error on random
//! input, not crash (NFR-4).

use std::io::Write;

use assert_cmd::Command;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tempfile::NamedTempFile;

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
fn dac_recognizes_elf_magic() {
    let mut buf = vec![0u8; 64];
    buf[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);

    let mut file = NamedTempFile::new().expect("create tempfile");
    file.write_all(&buf).expect("write tempfile");

    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(file.path())
        .assert()
        .success();
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
fn dac_accepts_deterministic_flag() {
    let mut buf = vec![0u8; 64];
    buf[..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);

    let mut file = NamedTempFile::new().expect("create tempfile");
    file.write_all(&buf).expect("write tempfile");

    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--deterministic")
        .arg(file.path())
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
