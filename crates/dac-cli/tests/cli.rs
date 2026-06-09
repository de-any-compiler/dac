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

fn pe_i386_fixture() -> PathBuf {
    fixture_path("hello-i386.exe")
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

/// B4.1 — AI adapter trait + offline default.
///
/// Ensures the M4 AI dispatch layer is wired through `run_pipeline`
/// (FR-32, FR-35, ARCHITECTURE §9). The CLI must:
///   * default to `NullProvider` (no `--ai-provider` arg, no
///     `--no-ai`),
///   * honour `--no-ai` (forces null),
///   * honour `--ai-provider null` / `--ai-provider none` (selects
///     null without warning),
///   * downgrade unknown provider names to null with a warning
///     instead of crashing (until B4.2 ships a real adapter).
///
/// In every case the run must succeed end-to-end and the manifest
/// must still render `"ai": { "provider": ... }` consistently — the
/// existing manifest goldens are unchanged by this batch.
#[test]
fn b4_1_default_run_succeeds_with_null_provider() {
    let path = elf_fixture();
    Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn b4_1_no_ai_flag_succeeds_and_keeps_provider_null_in_manifest() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--no-ai")
        .arg(&path)
        .output()
        .expect("run dac --no-ai");
    assert!(out.status.success(), "dac --no-ai should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"provider\": null"),
        "--no-ai must leave manifest provider null:\n{stdout}",
    );
}

#[test]
fn b4_1_ai_provider_null_is_routed_without_warning() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("null")
        .arg(&path)
        .output()
        .expect("run dac --ai-provider null");
    assert!(out.status.success(), "--ai-provider null should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    // The `null` name is preserved in the manifest so FR-37
    // attribution stays honest about what was requested.
    assert!(
        stdout.contains("\"provider\": \"null\""),
        "--ai-provider null must surface as the manifest provider:\n{stdout}",
    );
}

#[test]
fn b4_1_unknown_ai_provider_is_downgraded_to_null_without_failing() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("local:llama")
        .arg(&path)
        .output()
        .expect("run dac --ai-provider local:llama");
    assert!(
        out.status.success(),
        "unknown provider must downgrade, not fail",
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    // Manifest still records what the user asked for so the
    // attribution audit trail (FR-37) is preserved.
    assert!(
        stdout.contains("\"provider\": \"local:llama\""),
        "manifest must keep requested provider name:\n{stdout}",
    );
}

#[test]
fn b4_1_no_ai_overrides_explicit_ai_provider_arg() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--no-ai")
        .arg("--ai-provider")
        .arg("local:llama")
        .arg(&path)
        .output()
        .expect("run dac --no-ai --ai-provider local:llama");
    assert!(out.status.success(), "combination should still succeed");
}

/// B4.2 — `--ai-provider local` resolves through `select_provider` to
/// the rule-based [`dac_ai::LocalProvider`] (which advertises itself as
/// `"local:stub"`). The run must succeed end-to-end and the manifest
/// must preserve the user's requested provider name verbatim so the
/// FR-37 attribution trail records what was asked for.
#[test]
fn b4_2_ai_provider_local_alias_routes_to_local_stub_and_succeeds() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("local")
        .arg(&path)
        .output()
        .expect("run dac --ai-provider local");
    assert!(out.status.success(), "--ai-provider local should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"provider\": \"local\""),
        "manifest must keep requested provider name `local`:\n{stdout}",
    );
}

/// B4.2 — `--ai-provider local:stub` is the explicit spelling for the
/// rule-based local backend. Verifies the dispatch table's lower-case
/// match arm and that the manifest preserves the verbatim request.
#[test]
fn b4_2_ai_provider_local_stub_is_an_explicit_alias() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("local:stub")
        .arg(&path)
        .output()
        .expect("run dac --ai-provider local:stub");
    assert!(
        out.status.success(),
        "--ai-provider local:stub should exit 0",
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"provider\": \"local:stub\""),
        "manifest must keep requested provider name `local:stub`:\n{stdout}",
    );
}

/// B4.2 — `local:llama` is the canonical HTTP-backed local adapter
/// reserved for a follow-up batch. Until the HTTP wiring lands it stays
/// on the `Fallback` row of the dispatch table — the run must still
/// succeed and the manifest must preserve the requested name.
#[test]
fn b4_2_local_llama_remains_on_fallback_until_http_adapter_lands() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("local:llama")
        .arg(&path)
        .output()
        .expect("run dac --ai-provider local:llama");
    assert!(out.status.success(), "local:llama must downgrade, not fail",);
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"provider\": \"local:llama\""),
        "manifest must keep requested provider name `local:llama`:\n{stdout}",
    );
}

/// B4.2 — `--no-ai` still outranks the new `local` alias. The provider
/// is force-downgraded to null, the run succeeds, and the manifest
/// keeps the `no_ai: true` flag.
#[test]
fn b4_2_no_ai_overrides_local_provider_request() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--no-ai")
        .arg("--ai-provider")
        .arg("local")
        .arg(&path)
        .output()
        .expect("run dac --no-ai --ai-provider local");
    assert!(out.status.success(), "combination should still succeed");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"no_ai\": true"),
        "manifest must record no_ai=true even with explicit --ai-provider:\n{stdout}",
    );
}

/// B3.35 — i386 dispatch wiring.
///
/// Before this batch, `dac --target c <i386 PE>` printed the
/// `unsupported_arch_listing` stub ("no architecture backend
/// available; listing skipped"). After: the dispatch arm in
/// `pick_backend` routes i386 through the existing `dac-arch-x86`
/// 32-bit decoder / lifter / register file, so the listing carries
/// real recovered functions and the manifest reports
/// `architecture: i386`.
///
/// Done-when (PLAN.md §B3.35): listing has at least one recovered
/// function instead of the unsupported-arch stub, and the manifest
/// architecture field reads `i386`. (FR-3, FR-21)
#[test]
fn b3_35_i386_pe_listing_recovers_functions_and_manifest_reports_i386() {
    let path = pe_i386_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg(&path)
        .output()
        .expect("run dac on i386 PE");
    assert!(out.status.success(), "dac on i386 PE should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");

    // The pre-B3.35 stub leaves this exact substring; its absence is
    // the structural signal that the dispatch arm fired.
    assert!(
        !stdout.contains("no architecture backend available"),
        "i386 listing must not emit the unsupported-arch stub:\n{stdout}",
    );
    // At least one function header must appear. The annotated-listing
    // renderer prints `;; function <name>` per recovered entry; the
    // i386 PE fixture's symbol table seeds dozens of these.
    assert!(
        stdout.contains(";; function "),
        "i386 listing must include at least one recovered function header:\n{stdout}",
    );
    // Manifest is appended to stdout under the `;; ---- manifest`
    // banner and reports the architecture string as the
    // `dac-binfmt::Architecture::name` for I386, which is `"i386"`.
    assert!(
        stdout.contains("\"architecture\": \"i386\""),
        "manifest must report architecture: i386:\n{stdout}",
    );
}
