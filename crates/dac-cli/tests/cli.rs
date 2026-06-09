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

/// Stripped variant of [`elf_fixture`] — same binary, symbol table
/// gone. Used by B4.5 / B4.4-migrated tests because the AI proposal
/// pass only issues prompts for functions with synthesised `fn_<addr>`
/// names; an un-stripped fixture would skip every function and yield
/// `total=0`.
fn elf_stripped_fixture() -> PathBuf {
    fixture_path("hello-x86_64-stripped")
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

/// B4.3 — `--ai-strict` is accepted and the run succeeds end-to-end.
///
/// At B4.3 the verifier rejects every proposal as `unknown-target`
/// (the world model is still empty — populated in B4.4 / B4.5). The
/// CLI surface we lock in here is that the flag parses, the binary
/// exits 0, and the strict-mode choice does not perturb the manifest
/// (it is a behavioural switch, not a settings stamp).
#[test]
fn b4_3_ai_strict_flag_is_accepted_and_run_succeeds() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-strict")
        .arg(&path)
        .output()
        .expect("run dac --ai-strict --ai-provider local");
    assert!(
        out.status.success(),
        "--ai-strict --ai-provider local should exit 0",
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"provider\": \"local\""),
        "manifest must keep requested provider name `local`:\n{stdout}",
    );
}

/// B4.3 — `--ai-strict --no-ai` is a no-op combination: `--no-ai`
/// outranks every provider selection, so the verifier never sees a
/// delta and strict mode has nothing to gate. The run still succeeds
/// and the manifest's `no_ai: true` flag survives the combination.
#[test]
fn b4_3_ai_strict_is_inert_when_no_ai_is_set() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--no-ai")
        .arg("--ai-strict")
        .arg(&path)
        .output()
        .expect("run dac --no-ai --ai-strict");
    assert!(out.status.success(), "combination should still succeed");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("\"no_ai\": true"),
        "manifest must record no_ai=true even with --ai-strict:\n{stdout}",
    );
}

/// B4.3 — `--ai-strict` works without any `--ai-provider` selection.
/// The default `NullProvider` returns zero deltas, so strict mode has
/// nothing to gate, but the flag must still parse and the run must
/// still succeed.
#[test]
fn b4_3_ai_strict_works_with_default_provider() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("--ai-strict")
        .arg(&path)
        .output()
        .expect("run dac --ai-strict (default provider)");
    assert!(
        out.status.success(),
        "--ai-strict alone should still exit 0",
    );
}

/// B4.4 (migrated by B4.5) — `--ai-review` adds a review-mode side
/// artifact (spec §13.6, FR-33). The block contains the verifier's
/// verdict for every provider proposal as a before/after diff. B4.5
/// gates the AI pass behind `-O3` (spec §5) and populates the world
/// model from the recovered [`FunctionSet`]; the test runs at `-O3`
/// against a stripped fixture so the per-function loop has
/// synthesised `fn_<addr>` placeholders to propose against, and the
/// verifier accepts the local-stub renames (no name collisions,
/// lenient mode). The flag is still behavioural — the manifest
/// stays unchanged.
#[test]
fn b4_4_ai_review_emits_review_block_on_stdout() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-review")
        .arg(&path)
        .output()
        .expect("run dac -O3 --ai-review --ai-provider local");
    assert!(
        out.status.success(),
        "-O3 --ai-review --ai-provider local should exit 0",
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains(";; ---- ai review (FR-33) ----"),
        "review block delimiter must appear on stdout:\n{stdout}",
    );
    assert!(
        stdout.contains(";; dac --ai-review (spec §13.6)"),
        "review block must include the spec-§13.6 header:\n{stdout}",
    );
    assert!(
        stdout.contains(";; provider: local:stub"),
        "review block must cite the resolved provider name:\n{stdout}",
    );
    assert!(
        stdout.contains(";; mode:     lenient"),
        "default mode must render as lenient:\n{stdout}",
    );
    // Stripped hello fixture has 4 synthesised `fn_<addr>`
    // placeholders; the local stub returns one rename each, the
    // verifier's world recognises every target, and every rename
    // passes lenient verification (no name collisions).
    assert!(
        stdout.contains(";; deltas:   total=4 accepted=4 rejected=0"),
        "review block must summarise total/accepted/rejected:\n{stdout}",
    );
    assert!(
        stdout.contains("accept"),
        "review block must surface accepted outcomes:\n{stdout}",
    );
    assert!(
        stdout.contains("+   name:"),
        "diff `+` side must surface the proposed new name:\n{stdout}",
    );
}

/// B4.4 (migrated by B4.5) — `--ai-review` is omitted by default. No
/// review block, no `.review.txt` sidecar, no manifest churn.
#[test]
fn b4_4_review_block_absent_without_flag() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-provider")
        .arg("local")
        .arg(&path)
        .output()
        .expect("run dac -O3 --ai-provider local");
    assert!(out.status.success(), "default run should still exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        !stdout.contains(";; ---- ai review"),
        "review block must not appear without --ai-review:\n{stdout}",
    );
    assert!(
        !stdout.contains(";; dac --ai-review"),
        "review header must not appear without --ai-review:\n{stdout}",
    );
}

/// B4.4 (migrated by B4.5) — `--ai-review --ai-strict` composes:
/// strict mode flips the header without changing the side-artifact
/// shape.
#[test]
fn b4_4_review_renders_strict_header_when_strict_is_set() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-review")
        .arg("--ai-strict")
        .arg(&path)
        .output()
        .expect("run dac -O3 --ai-review --ai-strict --ai-provider local");
    assert!(out.status.success(), "combination should still exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains(";; mode:     strict"),
        "strict mode must surface in the review header:\n{stdout}",
    );
}

/// B4.4 (migrated by B4.5) — review block is stable across two runs
/// against the same input (deterministic side artifact — spec §13.6
/// "human-readable and stable").
#[test]
fn b4_4_review_block_is_stable_across_two_runs() {
    let path = elf_stripped_fixture();
    let first = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-review")
        .arg(&path)
        .output()
        .expect("first run");
    let second = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-review")
        .arg(&path)
        .output()
        .expect("second run");
    assert!(first.status.success());
    assert!(second.status.success());
    let a = String::from_utf8(first.stdout).expect("utf-8 stdout");
    let b = String::from_utf8(second.stdout).expect("utf-8 stdout");
    let extract = |s: &str| -> String {
        let marker = ";; ---- ai review";
        let pos = s.find(marker).expect("review block present");
        s[pos..].to_string()
    };
    assert_eq!(
        extract(&a),
        extract(&b),
        "review block must be byte-identical across two runs",
    );
}

/// B4.4 (migrated by B4.5) — with `--no-ai`, review mode has nothing
/// to record but must still produce a side artifact with `total=0`, so
/// a reviewer can see "review mode was on, no proposals were
/// returned".
#[test]
fn b4_4_review_block_renders_empty_under_no_ai() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--no-ai")
        .arg("--ai-review")
        .arg(&path)
        .output()
        .expect("run dac -O3 --no-ai --ai-review");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains(";; deltas:   total=0 accepted=0 rejected=0"),
        "empty review block must still summarise zero counts:\n{stdout}",
    );
}

/// B4.5 — at `-O3` the AI proposal pass runs, the verifier accepts
/// the local stub's renames against the synthesised `fn_<addr>`
/// world, and the C backend prefixes each accepted name with `ai_`
/// (FR-32, ARCHITECTURE §13). The done-when "AI is consulted only at
/// `-O3`" is observable here: the prefix would not appear if the pass
/// had been skipped.
#[test]
fn b4_5_o3_applies_ai_prefixed_renames_to_synthesised_function_names() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--target")
        .arg("c")
        .arg("--ai-provider")
        .arg("local")
        .arg(&path)
        .output()
        .expect("run dac -O3 --target c --ai-provider local");
    assert!(out.status.success(), "-O3 run should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("ai_dac_local_sub_"),
        "at least one ai_-prefixed function name must appear in C output:\n{stdout}",
    );
    assert!(
        stdout.contains("dac: ai-suggested rename (Speculative, conf="),
        "each AI-renamed function must carry an ai-suggested banner:\n{stdout}",
    );
}

/// B4.5 — the AI pass is **not** run at `-O2`. The corollary is
/// observable as the absence of any `ai_`-prefixed function name and
/// the absence of the ai-suggested banner. Demonstrates the "AI is
/// consulted only at `-O3`" gate (spec §5).
#[test]
fn b4_5_o2_does_not_consult_ai() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O2")
        .arg("--target")
        .arg("c")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-review")
        .arg(&path)
        .output()
        .expect("run dac -O2 --target c --ai-provider local --ai-review");
    assert!(out.status.success(), "-O2 run should exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        !stdout.contains("ai_dac_local_sub_"),
        "no ai_-prefixed names at -O2:\n{stdout}",
    );
    assert!(
        !stdout.contains("dac: ai-suggested"),
        "no ai-suggested banner at -O2:\n{stdout}",
    );
    // The review block is inert below -O3 (the AI pass returns an
    // empty result and the review_text is `None`) so `--ai-review`
    // does not produce a side artifact.
    assert!(
        !stdout.contains(";; ---- ai review"),
        "review block must not appear at -O2 even with --ai-review:\n{stdout}",
    );
}

/// B4.5 — `--ai-strict` preserves observed names: the symbol-table
/// fixture's `main` is recorded as `Source::Observed` in the world
/// model and strict mode rejects any rename against an observed
/// target (ARCHITECTURE §13.5). Note the proposal-pass policy also
/// skips functions with a recovered symbol-table name, so `main`
/// keeps its name regardless of `--ai-strict`; this test asserts the
/// invariant rather than the policy step that enforces it.
#[test]
fn b4_5_ai_strict_preserves_observed_main_at_o3() {
    let path = elf_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--target")
        .arg("c")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-strict")
        .arg(&path)
        .output()
        .expect("run dac -O3 --ai-strict --ai-provider local");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("int main(void)") || stdout.contains("int main("),
        "observed `main` symbol must survive --ai-strict at -O3:\n{stdout}",
    );
    assert!(
        !stdout.contains("ai_dac_local_sub_"),
        "no synthesised symbols in the symbol-table fixture → no ai_ renames:\n{stdout}",
    );
}

/// B4.5 — `-O3` AI output is deterministic. The local stub is a pure
/// function of `(prompt, bundle)` and the per-function loop iterates
/// over `FunctionSet::functions` in ascending-address order; two runs
/// against the same input must produce a byte-identical C
/// translation unit (NFR-9).
#[test]
fn b4_5_o3_c_unit_is_stable_across_two_runs() {
    let path = elf_stripped_fixture();
    let run = || -> String {
        let out = Command::cargo_bin("dac")
            .expect("dac binary present")
            .arg("-O3")
            .arg("--target")
            .arg("c")
            .arg("--ai-provider")
            .arg("local")
            .arg(&path)
            .output()
            .expect("run dac");
        assert!(out.status.success());
        String::from_utf8(out.stdout).expect("utf-8 stdout")
    };
    let a = run();
    let b = run();
    let extract_source = |s: &str| -> String {
        // The C unit is delimited by `;; ---- target source` on
        // stdout. The tracing INFO lines that precede the listing
        // carry timestamps so we cannot diff full stdout — extract
        // the source block instead.
        let marker = ";; ---- target source";
        let pos = s.find(marker).expect("source block present");
        s[pos..].to_string()
    };
    assert_eq!(
        extract_source(&a),
        extract_source(&b),
        "C unit must be byte-identical across two -O3 runs",
    );
}

/// B4.5 — `--ai-name-prefix` overrides the default `ai_` prefix.
/// Visible end-to-end: AI-renamed function names land with the
/// configured prefix instead of `ai_`.
#[test]
fn b4_5_ai_name_prefix_overrides_default() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--target")
        .arg("c")
        .arg("--ai-provider")
        .arg("local")
        .arg("--ai-name-prefix")
        .arg("synth_")
        .arg(&path)
        .output()
        .expect("run dac with custom --ai-name-prefix");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(
        stdout.contains("synth_dac_local_sub_"),
        "custom prefix must surface on renamed functions:\n{stdout}",
    );
    assert!(
        !stdout.contains("ai_dac_local_sub_"),
        "default prefix must not appear when overridden:\n{stdout}",
    );
}

/// B4.5 — `--ai-name-prefix` rejects values that would produce an
/// invalid C identifier when concatenated with the stub's name. The
/// CLI must surface a clean error before running the pipeline so the
/// user does not waste a full analysis on a malformed prefix.
#[test]
fn b4_5_ai_name_prefix_rejects_invalid_identifier() {
    let path = elf_stripped_fixture();
    let out = Command::cargo_bin("dac")
        .expect("dac binary present")
        .arg("-O3")
        .arg("--ai-name-prefix")
        .arg("3bad")
        .arg(&path)
        .output()
        .expect("run dac with invalid --ai-name-prefix");
    assert!(!out.status.success(), "must reject leading-digit prefix");
    let stderr = String::from_utf8(out.stderr).expect("utf-8 stderr");
    assert!(
        stderr.contains("invalid --ai-name-prefix"),
        "stderr must explain why the prefix was rejected:\n{stderr}",
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
