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

#### B0.3 — Core types, evidence graph, confidence lattice (2026-06-01)

The provenance + confidence substrate every later batch depends on
(invariants I-2 and I-3). Confidence is a product lattice over a numeric
value and a totally ordered source class; the evidence graph is an
append-only directed graph whose handles travel with IR nodes.

- `dac-core::confidence`:
  - `Source` enum (`Speculative < Derived < UserHint < Observed`) with
    `name()` for diagnostics. The order is total; contradictions
    between sources are modeled with `EdgeKind::Contradicts` edges,
    not by re-ranking variants.
  - `Confidence { value: f32, source: Source }`. `new` clamps `value`
    to `[0.0, 1.0]`, maps `NaN` to `0.0`, and normalizes `-0.0`. After
    construction every value lies on a totally ordered numeric axis.
  - `join` / `meet` are componentwise max / min. Partial-order
    `PartialOrd` impl: `a ≤ b` iff both axes are `≤`; incomparable
    cases yield `None`.
  - Property tests (`proptest`) for idempotence, commutativity,
    associativity, absorption, monotonicity, and the
    least-upper-bound / greatest-lower-bound laws on comparable pairs.
- `dac-core::evidence`:
  - `EvidenceId(NonZeroU32)` so `Option<EvidenceId>` has the same size
    as `EvidenceId` when threaded through IR nodes.
  - `EvidenceNode` variants for `Bytes`, `Instruction`, `IrNode`,
    `KnowledgeFact`, `UserHint`, `AiSuggestion { prompt_hash }`. Inner
    ids are opaque `u64` until their owning crates exist (B1.x / B2.x).
  - `IrLayer` (`Instruction` / `Cfg` / `Ssa` / `Semantic` / `Source`)
    addresses which IR an `IrNode` points into.
  - `EdgeKind`: `Supports` / `Contradicts` / `Refines`.
  - `EvidenceGraph` is append-only — facts are superseded with
    `Contradicts` / `Refines` edges, never deleted, so the audit
    trail for `--debug` and `--emit-report` stays intact.
  - Unit tests cover sequential handles, foreign-handle safety,
    insertion-order edge ordering, self-loops, and round-trip
    payloads.
- `dac-api`: re-exports `Confidence`, `Source`, `EvidenceGraph`,
  `EvidenceId`, `EvidenceNode`, `IrLayer`, `Edge`, `EdgeKind`, plus
  `Error` / `Result`. This is the start of the stable public surface
  (FR-41); the wider surface fills in batch-by-batch.
- `docs/confidence-lattice.md`: long-form treatment of the algebra,
  the laws, and how the lattice meshes with the evidence graph. Linked
  from `ARCHITECTURE.md §5`.
- `Cargo.toml`: `proptest = "1"` added to `[workspace.dependencies]`
  as a dev dep.

Closes: I-2, I-3 (graph + lattice plumbing in place; concrete fact
production lands as later batches populate them). Sets up the
provenance machinery the pass manager (B0.4) and AI delta protocol
(M4) build on.

#### B0.4 — Pass manager skeleton (2026-06-01)

The skeleton orchestrator every later batch hangs off (invariant I-5
and NFR-9). `dac-core` gains a `Pass` trait, a topological scheduler,
and a `PassManager` that caches outputs into the new `dac-artifact`
content-addressed store. The `--deterministic` flag is wired through
to the manager and rejects `NonDeterministic` passes at registration.

- `dac-core::pass`:
  - `Pass` trait (`id`, `inputs`, `outputs`, `determinism`, `run`).
  - `PassId(&'static str)` and `ArtifactKind(&'static str)` newtypes —
    open by design so out-of-tree and test pipelines can declare
    their own kinds.
  - `Determinism` enum (`Pure` / `SeededPure` / `NonDeterministic`).
  - `PassContext` with `input(kind) -> Option<&[u8]>` and
    `produce(kind, bytes)`; writes to undeclared kinds are dropped.
  - `ArtifactStore` thin wrapper over `HashMap<ArtifactKind, Vec<u8>>`
    threaded through the pipeline.
  - `PassManager::new()` (lax) and `PassManager::deterministic()`
    (NFR-9). `register` rejects non-deterministic passes in the latter
    mode with a structured `Error::PassManager` message.
  - `with_settings_hash(u64)` partitions the cache so `-O1` and `-O2`
    runs cannot collide.
  - Topological scheduler (Kahn) with deterministic root tiebreak by
    registration order. Surfaces cycles, missing producers, duplicate
    producers, and self-input/output passes through `Error::PassManager`.
  - Cache key is the byte concatenation
    `pass_id || 0 || input_hash_le || settings_hash_le`. Input hash is
    an inline FNV-1a-64 over `(kind, bytes)*` in declared order — a
    stable in-process hash, replaceable later when on-disk caching
    needs cross-process stability.
  - Cache value is a length-prefixed `(kind, bytes)*` blob; nothing
    else reads the format.
  - `RunReport { passes: Vec<(PassId, PassOutcome)> }` with
    `executed()`, `cached()`, and `fully_cached()` helpers.
  - `Error::PassManager(String)` covers cycles, missing/duplicate
    producers, deterministic-mode violations, and corrupt-cache
    decode errors.
- `dac-artifact`:
  - `ArtifactCache` — in-memory `HashMap<Vec<u8>, Vec<u8>>` with
    `get` / `put` / `len` / `is_empty`. The cache is intentionally
    opaque: it stores and retrieves; the pass manager owns key
    construction. On-disk persistence lands later.
- `dac-cli`:
  - `--deterministic` flag parsed and surfaced through tracing. The
    pipeline plumbing (passing the flag into a real `PassManager`)
    lands once the CLI starts driving pipelines in B1.6; the
    rejection of non-deterministic passes is fully covered by
    `dac-core` unit tests today.
  - Two new integration tests: `--deterministic` is accepted on a
    valid input, and unknown flags still exit `2`.
- `dac-api`: re-exports the pass-manager surface
  (`Pass`, `PassContext`, `PassId`, `ArtifactKind`, `Determinism`,
  `PassManager`, `ArtifactStore`, `PassOutcome`, `RunReport`,
  `ArtifactCache`).

The toy-pipeline done-when is covered by
`dac-core::pass::tests::three_pass_pipeline_runs_and_caches`: three
`Pure` passes (`alpha → beta → gamma`) run once with counter `= 3`,
then a fresh store + same cache replays every pass from cache with
counter still `= 3` and all three outputs restored.

Closes: I-5 (pass declares inputs/outputs/determinism, manager
enforces), NFR-9 (`--deterministic` rejects non-deterministic
passes), NFR-5 (cache stub keyed by
`hash(pass_id || input_hash || settings_hash)`). Sets up the
plumbing the first real passes (B1.x) drop into.

#### B0.5 — CLI surface (2026-06-01)

Full `dac` CLI declared from spec §10.1. Every flag in the spec is now
parsed and validated; most do not yet drive behavior and become active
milestone by milestone. `dac --help` is a stable snapshot, guarded by an
integration test, and `dac --version` reports `dac <version> (<build-id>)`
for NFR-10.

- `dac-cli`:
  - Flags parsed: `-O0`/`-O1`/`-O2`/`-O3`, `--arch <a>`, `--format <fmt>`,
    `--target <lang>`, `--output <path>`, `--emit-ir`, `--emit-cfg`,
    `--emit-report`, `--emit-annotations`, `--no-ai`,
    `--ai-provider <name>`, `--deterministic`, `--threads <n>`,
    `--json`, `--debug`, `--plugin <path>`, `--version`/`-V`,
    `--help`/`-h`.
  - `Format` and `Target` are typed enums; invalid values fail at parse
    time with exit `2`. `--threads` is parsed as a positive `u32`; `0`
    or non-numeric values exit `2`. `--arch`, `--output`,
    `--ai-provider`, `--plugin` are accepted as opaque values for now.
  - `--help`/`-h` prints the snapshot to stdout (not stderr) and exits
    `0`. `--version`/`-V` prints `dac <CARGO_PKG_VERSION> (<BUILD_ID>)`
    and exits `0`.
  - `BUILD_ID` resolves from the compile-time `DAC_BUILD_ID` env var,
    defaulting to `"dev"` for local builds. CI / release builds inject
    the commit SHA. This is the first piece of NFR-10 reproducibility
    metadata; pipeline runs will pick it up once they have a manifest
    (B1.6).
  - Errors print a one-line hint (`dac: try \`dac --help\` for usage.`)
    rather than dumping the whole help text on every typo.
  - `tracing::debug!` at startup records the parsed argument set, so
    `--debug` (and `RUST_LOG=debug`) trace runs surface every flag.
- `dac-core::init_tracing` now takes `(json, debug)`. When `RUST_LOG`
  is unset, `debug = true` defaults the filter to `"debug"` instead of
  `"info"`. Existing callers updated; idempotence test now exercises
  all four combinations.
- `crates/dac-cli/tests/snapshots/help.txt` is the golden help text,
  included into the binary via `include_str!` and into the test via
  the same macro, so the binary cannot drift from the test.
- `crates/dac-cli/tests/cli.rs` grows to 18 integration tests covering:
  the help / short-help snapshot, version + short-version equality and
  format, the full §10.1 flag surface accepted together, each `-O`
  level, every `--format` value, and exit-`2` paths for invalid
  `--format` / `--target` / `--threads` and missing values.

Closes: NFR-10 (tool version + build id surfaced through `--version`;
the reproducibility manifest emitted alongside pipeline runs lands
with B1.6). Lays the spec-§10.1 surface every later batch hangs flags
off of.


### Milestone 1 — Foundation

#### B1.1 — Binary model and ELF parser (2026-06-01)

Format-agnostic `BinaryModel` vocabulary plus a real ELF parser. ADR-0003
closes on `object` (see [DECISIONS.md](./DECISIONS.md)); every later
format (PE in B1.2, Mach-O later) bridges into the same `BinaryModel`
shape, so downstream crates never see format-specific types.

- `dac-binfmt::model`:
  - `BinaryFormat`, `Architecture`, `Endian`, `Bits`, `Permissions`.
  - `Section { name, address, size, file_offset, perms, kind }` with
    `SectionKind` covering text / read-only data / data / bss / TLS /
    metadata / note / other / unknown.
  - `Segment { name, address, file_offset, file_size, mem_size, perms }`
    (program-header view; `name` carries the parser's `PT_*` label when
    available).
  - `Symbol { name, address, size, kind, binding, section, source,
    undefined }` with `SymbolKind` (text / data / section / file / TLS
    / label / unknown), `SymbolBinding` (local / global / weak / unique),
    `SymbolSource` (`Symtab` for `.symtab`, `Dynsym` for `.dynsym`).
  - `Import { name, library }` and `Export { name, address }` capture
    the dynamic linkage view.
  - `Relocation { section: Option<usize>, offset, kind, symbol, addend }`
    with `RelocationKind` (absolute / relative / GOT-relative /
    PLT-relative / glob / copy / TLS / unknown). `section` is `None`
    when a dynamic relocation patches an address outside every recorded
    section; `offset` is bytes-into-section for static relocations and
    a virtual address for dynamic ones.
  - `StringRef { section, offset, value }` for printable-ASCII runs
    ≥ 4 bytes, NUL-terminated, scanned from read-only-data sections.
  - `BinaryModel` aggregates all of the above plus `format`, `architecture`,
    `endian`, `bits`, `entry`, `size`, and `needed_libraries` (DT_NEEDED
    sonames on ELF; the same field carries PE DLL imports and Mach-O
    `LC_LOAD_DYLIB` install names once those parsers land).
- `dac-binfmt::elf`:
  - Wraps `object::File` and maps everything into the model types.
    Static `.rela.<section>` entries (relocatable objects) flow through
    per-section `relocations()`; dynamic `.rela.dyn` / `.rela.plt`
    entries (executables / shared libraries) flow through
    `Object::dynamic_relocations()`.
  - `map_relocation_flags` maps the common x86-64 and AArch64 `R_TYPE`
    families (`R_X86_64_RELATIVE`, `R_X86_64_GLOB_DAT`, `R_X86_64_PLT32`,
    GOT-relative, copy, TLS, absolute, PC-relative; `R_AARCH64_ABS*`,
    `R_AARCH64_GLOB_DAT`, `R_AARCH64_RELATIVE`). Unknown types collapse
    to `RelocationKind::Unknown`; raw type fidelity stays inside
    `object` for now.
  - String scan walks `SectionKind::ReadOnlyData` sections, emits any
    NUL-terminated printable-ASCII run ≥ 4 bytes as a `StringRef`. Code
    and writable sections are intentionally skipped to suppress
    relocation / opcode false positives.
  - All error paths return `Error::MalformedBinary { format: "ELF", reason }`;
    no `unwrap` / `expect` outside test code.
- `dac-binfmt::lib`:
  - `load_from_bytes` now dispatches: ELF → full parse, PE / Mach-O →
    `UnsupportedFormat` until B1.2 / later. Format detection
    (`detect_format`) keeps the cheap-path magic check that was added
    in B0.2.
  - 10 unit tests cover the boundary cases (empty input, ELF magic
    without a valid header → `MalformedBinary`, PE / Mach-O magic →
    `UnsupportedFormat`, 512 deterministic-PRNG inputs → no panic).
- `dac-binfmt::fuzz`:
  - libFuzzer crate at `crates/dac-binfmt/fuzz/`, scoped out of the
    parent workspace via its own empty `[workspace]` block.
  - Single target `fuzz_elf_parse` hits both `detect_format` and
    `load_from_bytes`. Run via
    `cargo install cargo-fuzz && cargo +nightly fuzz run fuzz_elf_parse -- -max_total_time=300`
    from `crates/dac-binfmt/`; the 5-minute total-time cap matches the
    B1.1 done-when. The in-tree 512-iteration deterministic-PRNG smoke
    keeps the same invariant green in stable CI.
- `tests/fixtures/` (workspace-root, shared across crates):
  - `hello-x86_64` — PIE executable, dynamic, with `.symtab`.
  - `hello-x86_64-stripped` — same input `strip -s`-ed.
  - `libsample.so` — shared library exporting `sample_add`,
    `sample_greeting`, `sample_value` plus an embedded string literal.
  - `README.md` documenting the source and build recipe so the fixtures
    are reproducible.
- `crates/dac-binfmt/tests/elf.rs` — round-trip integration tests:
  hello-x86_64 shape (entry, sections, segments, `.text`), `main` in
  `.symtab`, `libc.so` in `needed_libraries`, `write` in imports;
  stripped variant has zero `Symtab` symbols but keeps `Dynsym`;
  `libsample.so` exposes the three exports with the right `SymbolKind`
  and surfaces the embedded string; relocations all resolve to valid
  symbol indices when symbol-bound. A best-effort `system_libc_parses_when_present`
  probes `/lib/x86_64-linux-gnu/libc.so.6` and friends — runs on Linux
  CI, skips silently elsewhere.
- `dac-cli`:
  - The `dac_recognizes_elf_magic` test (which fed 64 magic-bytes-only
    bytes) is replaced by `dac_parses_elf_fixture` against the real
    fixture, plus a negative `dac_rejects_elf_magic_without_valid_header`
    that asserts the new parser produces a clean exit-1, not a panic.
  - Success-path tracing now emits `arch`, `sections`, `segments`,
    `symbols`, `imports`, `exports`, `relocations`, `strings`,
    `needed_libraries`, and `entry` alongside `format` / `size`. The
    full-flag-surface test still passes; tests that needed an ELF input
    now point at the shared fixture.
- `DECISIONS.md`: ADR-0003 closes with the rationale, the rejected
  alternatives, and the boundary the choice draws (`object` types do
  not leak past `dac-binfmt`).
- `Cargo.toml`: `object = { version = "0.36", default-features = false,
  features = ["read", "std"] }` added to `[workspace.dependencies]`.

Closes: FR-3 (ELF supported in the initial release; PE / Mach-O follow),
FR-5 (stripped and unstripped binaries both round-trip), FR-6
(import / export information preserved through `Import`, `Export`,
`Symbol`, and `needed_libraries`), partial NFR-4 (parser robustness
covered by the in-tree stress test and the fuzz target — the 5-minute
fuzz run remains a manual gate per the B1.1 done-when).

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
