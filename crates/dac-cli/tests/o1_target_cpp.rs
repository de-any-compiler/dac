//! End-to-end test for `dac -O<n> --target cpp` (B3.5, FR-21).
//!
//! Closes the B3.5 done-when: "a sample C++ binary with a small class
//! hierarchy decompiles to C++ that compiles". Runs the CLI against
//! the `cpp-hierarchy-x86_64` ELF fixture and asserts:
//!
//! 1. A `<output>.cpp` sidecar is written.
//! 2. The sidecar contains the canonical dac C++ banner comment whose
//!    `-O<n>` level matches the invocation (B3.31 — was hardcoded to
//!    `-O1`) plus `class Dog`, `class Cat`, and `class Animal`
//!    definitions — the three classes the fixture's source declares.
//! 3. Each polymorphic class carries a `virtual ~Class` declaration
//!    (every `_ZTV*` symbol promotes its class to polymorphic and the
//!    lowering pass synthesises the dtor when none was recovered).
//! 4. A `main` free function lands with `std::int32_t` return.
//! 5. The sidecar compiles cleanly when `c++` is on PATH; the test is
//!    silently skipped otherwise (mirrors the round-trip helper's
//!    skip-when-no-compiler contract).
//! 6. Two runs produce byte-identical `.cpp` output — the backend is
//!    deterministic across re-runs (NFR-9).

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

fn run_cpp(opt: &str, output: &PathBuf) {
    let path = fixture_path("cpp-hierarchy-x86_64");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(opt)
        .arg("--target")
        .arg("cpp")
        .arg("--output")
        .arg(output)
        .arg(&path)
        .assert()
        .success();
}

fn run_o1_cpp(output: &PathBuf) {
    run_cpp("-O1", output);
}

fn sidecar(base: &Path, suffix: &str) -> PathBuf {
    let mut s = base.to_path_buf().into_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

#[test]
fn o1_target_cpp_emits_cpp_sidecar_with_recovered_classes() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_cpp(&out);

    let source = fs::read_to_string(sidecar(&out, ".cpp")).expect("cpp sidecar");
    assert!(
        source.contains("dac --target cpp -O1 reconstruction"),
        "missing banner in:\n{source}"
    );
    // The fixture's three user-defined classes. Animal has no
    // base; Dog and Cat each carry `: public Animal` because
    // B3.11's typeinfo walker recovered the inheritance from
    // `__si_class_type_info` shapes in `.data.rel.ro`.
    assert!(
        source.contains("class Animal {"),
        "missing Animal class in:\n{source}"
    );
    assert!(
        source.contains("class Dog : public Animal {"),
        "missing Dog : public Animal inheritance clause in:\n{source}"
    );
    assert!(
        source.contains("class Cat : public Animal {"),
        "missing Cat : public Animal inheritance clause in:\n{source}"
    );
    // Polymorphism: Dog and Cat carry a virtual dtor.
    assert!(
        source.contains("virtual ~Dog()"),
        "Dog should have a virtual dtor in:\n{source}"
    );
    assert!(
        source.contains("virtual ~Cat()"),
        "Cat should have a virtual dtor in:\n{source}"
    );
    // The recovered methods.
    assert!(
        source.contains("virtual std::int32_t speak() const"),
        "expected `virtual std::int32_t speak() const` declaration in:\n{source}"
    );
    // main always lands as a free function with int return.
    assert!(
        source.contains("std::int32_t main()"),
        "expected `std::int32_t main()` in:\n{source}"
    );
}

/// B3.31: the C++ reconstruction banner reflects the active `-O<n>`
/// invocation rather than the historical hardcoded `-O1`. The done-when
/// for B3.31 is "invocation at `-O3` prints `-O3` in the header"
/// (FR-21). We also assert `-O2` to confirm every non-`-O1` level
/// reaches the format string the same way.
#[test]
fn b3_31_cpp_banner_reflects_active_opt_level() {
    for opt in ["-O2", "-O3"] {
        let dir = TempDir::new().expect("tempdir");
        let out = dir.path().join("a.listing");
        run_cpp(opt, &out);
        let source = fs::read_to_string(sidecar(&out, ".cpp")).expect("cpp sidecar");
        let banner = format!("dac --target cpp {opt} reconstruction");
        assert!(source.contains(&banner), "missing `{banner}` in:\n{source}");
        assert!(
            !source.contains("dac --target cpp -O1 reconstruction"),
            "stale -O1 banner leaked at {opt}:\n{source}"
        );
    }
}

#[test]
fn o1_target_cpp_round_trips_through_system_compiler() {
    let cxx = match find_cxx() {
        Some(p) => p,
        None => {
            eprintln!("o1_target_cpp_round_trips: no c++ on PATH, skipping");
            return;
        }
    };
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_cpp(&out);

    let source = fs::read_to_string(sidecar(&out, ".cpp")).expect("cpp sidecar");

    use std::io::Write as _;
    let mut child = Sys::new(&cxx)
        .args([
            "-x",
            "c++",
            "-std=c++17",
            "-c",
            "-",
            "-o",
            "/dev/null",
            "-w",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn c++");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(source.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait c++");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("emitted C++ did not compile:\n--- source ---\n{source}\n--- stderr ---\n{stderr}");
    }
}

#[test]
fn o1_target_cpp_output_is_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    let out_a = dir.path().join("a.listing");
    let out_b = dir.path().join("b.listing");
    run_o1_cpp(&out_a);
    run_o1_cpp(&out_b);
    let a = fs::read_to_string(sidecar(&out_a, ".cpp")).expect("source a");
    let b = fs::read_to_string(sidecar(&out_b, ".cpp")).expect("source b");
    assert_eq!(a, b, "C++ source drifted between two runs");
}

#[test]
fn o1_target_c_still_emits_dot_c_sidecar_against_cpp_fixture() {
    // Sanity check that the fixture works against `--target c` too —
    // the C backend's class-blind recovery still produces a valid
    // sidecar with one `void <name>(void)` per recovered function.
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    let path = fixture_path("cpp-hierarchy-x86_64");
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O1")
        .arg("--target")
        .arg("c")
        .arg("--output")
        .arg(&out)
        .arg(&path)
        .assert()
        .success();
    let c_source = fs::read_to_string(sidecar(&out, ".c")).expect("c sidecar");
    assert!(c_source.contains("dac --target c -O1 reconstruction"));
    assert!(!sidecar(&out, ".cpp").exists());
}

fn find_cxx() -> Option<String> {
    if let Ok(cxx) = std::env::var("CXX") {
        if !cxx.trim().is_empty() && which(&cxx).is_some() {
            return Some(cxx);
        }
    }
    for c in ["c++", "g++", "clang++"] {
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
