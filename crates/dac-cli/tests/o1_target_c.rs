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

/// B3.33: PLT-bound libc imports surface as `extern <canonical sig>`
/// using the POSIX typedef vocabulary instead of the lattice-driven
/// width-tagged spelling. The done-when for B3.33 is that the
/// extern declaration for `write` on the `hello-x86_64` fixture
/// reads `extern ssize_t write(int fd, const void *buf, size_t n);`
/// — and the translation unit grows the `<sys/types.h>` /
/// `<unistd.h>` includes the typedef requires.
#[test]
fn b3_33_canonical_extern_for_write_uses_posix_typedefs() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_c(&out);

    let source = fs::read_to_string(sidecar_with_suffix(&out, ".c")).expect("source sidecar");
    assert!(
        source.contains("extern ssize_t write(int fd, const void * buf, size_t n);"),
        "expected canonical `extern ssize_t write(...)` decl in:\n{source}"
    );
    // The stdint spelling must no longer leak — if it does, the
    // canonical-extern table didn't drive the lowering.
    assert!(
        !source.contains("extern int64_t write("),
        "stale stdint-spelled `extern int64_t write(...)` decl leaked:\n{source}"
    );
    // The required-headers accumulator must have prepended both
    // includes; the canonical-extern table reports them in
    // declaration order via a deterministic BTreeSet.
    assert!(
        source.contains("#include <sys/types.h>"),
        "missing `<sys/types.h>` include for ssize_t / size_t in:\n{source}"
    );
    assert!(
        source.contains("#include <unistd.h>"),
        "missing `<unistd.h>` include for write in:\n{source}"
    );
}

/// B3.34: the whole-callgraph void-return inference pass demotes
/// every CRT-helper on the `hello-x86_64` fixture whose return value
/// is dropped by every observed caller (or for which no caller was
/// observed in the analyzed function set). The done-when for B3.34
/// lists `_init`, `_fini`, `register_tm_clones`, `deregister_tm_clones`,
/// and `__do_global_dtors_aux` — each should emit with a `void`
/// declarator (B3.29's per-function inference left them at `long`)
/// without changing functions whose return is observed (e.g. `main`,
/// which the canonical-signature override keeps at `int`).
#[test]
fn b3_34_void_return_inference_demotes_unobserved_helpers() {
    let dir = TempDir::new().expect("tempdir");
    let out = dir.path().join("a.listing");
    run_o1_c(&out);

    let source = fs::read_to_string(sidecar_with_suffix(&out, ".c")).expect("source sidecar");
    for fn_name in [
        "_init",
        "_fini",
        "register_tm_clones",
        "deregister_tm_clones",
        "__do_global_dtors_aux",
    ] {
        let expect_void = format!("void {fn_name}(");
        assert!(
            source.contains(&expect_void),
            "expected `{expect_void}` declarator in:\n{source}",
        );
        let stale_long = format!("long {fn_name}(");
        assert!(
            !source.contains(&stale_long),
            "stale `{stale_long}` declarator must not survive B3.34 in:\n{source}",
        );
    }
    // The canonical override above B3.34 keeps `main`'s `int`.
    assert!(
        source.contains("int main("),
        "main must remain `int main(…)` even though no analyzed caller observes its return:\n{source}",
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
