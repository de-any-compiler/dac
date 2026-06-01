# Changelog

All notable changes to dac are recorded here. The format is loosely based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), adapted for dac's
batch-based development model from [PLAN.md](./PLAN.md).

## Recording rules

- **One entry per finished batch.** When a batch from `PLAN.md` lands, its
  goal + deliverables + closed requirement IDs move here.
- **Group by milestone.** Inside a milestone, list batches in completion order.
- **Reference IDs.** Always include the batch ID (`B1.4`), and the
  FR/NFR/I numbers the batch closed, so the spec stays traceable.
- **Releases live at the top.** Tagged releases get a `## [x.y.z] — YYYY-MM-DD`
  heading above the in-progress section.
- **The "Unreleased" section is the live one.** Move entries out of it when
  cutting a release.

---

## [Unreleased]

### Milestone 0 — Project skeleton

#### B0.1 — Workspace bootstrap (2026-06-01)

Cargo workspace with the 17-crate layout from `ARCHITECTURE.md` §2, plus
`xtask`. All crates compile as stubs and `cargo xtask ci` is green locally
on Linux. CI matrix runs on Linux / macOS / Windows in
`.github/workflows/ci.yml`; cross-platform confirmation lands the first
time the workflow runs in CI.

- Workspace `Cargo.toml` with `resolver = "2"`, workspace-wide
  `version`/`edition`/`license`/`rust-version`, and workspace lints
  (`rust_2018_idioms`, `clippy::all`).
- `rust-toolchain.toml` pinned to `stable` with `rustfmt` + `clippy`.
- `rustfmt.toml`, `clippy.toml` (`msrv = "1.85"`), `deny.toml` (license +
  source allow-lists; not yet wired into `xtask ci`).
- `.cargo/config.toml` aliases `cargo xtask` to `cargo run --package xtask`.
- `xtask` crate with subcommands `ci` / `fmt` / `clippy` / `test` / `help`.
  `ci` runs `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `cargo test --workspace` in order.
- GitHub Actions workflow runs `cargo xtask ci` on
  `ubuntu-latest` / `macos-latest` / `windows-latest`, with
  `Swatinem/rust-cache@v2`.
- License chosen: Apache-2.0 (ADR-0001 closed in
  [DECISIONS.md](./DECISIONS.md)). Canonical text in `LICENSE`.
- Each stub crate carries `#![forbid(unsafe_code)]` and `[lints] workspace
  = true`. `dac-cli` is the `dac` binary; for now it prints a "not yet
  implemented" message and exits 2.

Closes: NFR-19 (cross-platform CI scaffolding).

#### B0.2 — Logging, errors, and panic policy (2026-06-01)

Tracing infrastructure, project-wide `Error` enum, and the panic-policy
smoke test that proves dac returns a clean error on garbage input rather
than crashing (NFR-4). End-to-end check: `dac` invoked on a 4 KiB random
buffer exits with code 1, not a signal.

- `dac-core`:
  - `Error` enum (`thiserror`, `#[non_exhaustive]`) with `Io`,
    `UnsupportedFormat`, `MalformedBinary { format, reason }`,
    `InvariantViolation`, `Other` variants; `Result<T>` alias.
  - `init_tracing(json: bool)` sets up `tracing_subscriber` with
    `EnvFilter` (defaults to `info`, honors `RUST_LOG`) and toggles
    JSON output. Idempotent — safe in tests.
- `dac-binfmt`:
  - `BinaryFormat` (`Elf` / `Pe` / `MachO`) with `name()`.
  - `BinaryModel { format, size }` (placeholder; full fields land
    with B1.1).
  - `detect_format(&[u8])` does magic-byte detection for ELF, PE
    (DOS-stub-relative PE header pointer), and Mach-O (thin LE/BE +
    fat, both endians).
  - `load_from_bytes(&[u8])` wraps detection and returns a
    `BinaryModel` or `Error::UnsupportedFormat`.
  - Smoke test runs 512 deterministic-PRNG inputs through both
    entrypoints; asserts no panic.
- `dac-cli`:
  - Hand-rolled arg parser: `<input>`, `--json`, `--help`/`-h`.
  - Reads the input file, calls `load_from_bytes`, emits structured
    `tracing` events on success and failure.
  - Exit codes: `0` (success), `1` (clean failure), `2` (usage error).
  - Integration tests (`crates/dac-cli/tests/cli.rs`) cover: random-byte
    input → exit 1; ELF magic → exit 0; no-args → exit 2; `--help` →
    exit 0. All run as part of `cargo test --workspace`, which is the
    `xtask ci` fuzz smoke per the batch spec.
- `Cargo.toml`: `[workspace.dependencies]` centralizes versions for
  internal crates and the new external deps (`tracing`,
  `tracing-subscriber` w/ `env-filter` + `fmt` + `json`, `thiserror`,
  `rand`, `assert_cmd`, `tempfile`).

Closes: NFR-4 (safe handling of malformed binaries — for the format
detector layer). Lays the tracing groundwork for FR-29 / NFR-8 / spec
§10.1 `--json`. Spec §13.7-style prompt-template versioning is not yet
in scope; B4.x will use the `tracing` infrastructure for AI provenance.


### Milestone 1 — Foundation
*(not started)*

### Milestone 2 — Core decompilation
*(not started)*

### Milestone 3 — Usable RE tool
*(not started)*

### Milestone 4 — Human-oriented reconstruction
*(not started)*

### Milestone 5 — Ecosystem
*(not started)*

---

## Project bootstrap (2026-06-01)

- Imported the design notes / requirements spec
  (`dac_design_notes_requirements_spec.md`).
- Authored [README.md](./README.md), [ARCHITECTURE.md](./ARCHITECTURE.md),
  [PLAN.md](./PLAN.md), [DECISIONS.md](./DECISIONS.md),
  [CONTRIBUTING.md](./CONTRIBUTING.md), this file.
- Locked in two foundational decisions:
  - **Rust** as the implementation language ([ADR-0002](./DECISIONS.md)).
  - **Custom SSA-based decompilation IR** ([ADR-0002](./DECISIONS.md)).
- No code yet; M0 batches are the first implementation work.
