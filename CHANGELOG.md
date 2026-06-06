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

#### B1.2 — PE parser (2026-06-01)

Second binary format wired into the [`BinaryModel`] vocabulary. `elf.rs`
and the new `pe.rs` both delegate to a shared `bridge::parse_object`, so
every model field stays in lock-step across formats and the format-
specific work collapses to two thin modules. ADR-0003 (`object`)
continues to hold for PE: `Object::sections() / segments() / symbols() /
imports() / exports()` already drive the generic walk; the PE-specific
bits are confined to `IMAGE_SCN_MEM_*` permission decoding and the
`IMAGE_REL_AMD64_*` / `IMAGE_REL_I386_*` / `IMAGE_REL_ARM64_*` relocation
tables.

- `dac-binfmt::bridge` (new):
  - `parse_object(bytes, format, format_tag)` is the generic walk. It
    builds sections / segments / symbols / imports / exports / static and
    (ELF-only) dynamic relocations / strings / `needed_libraries`. The
    same code that produces an ELF `BinaryModel` now produces a PE one
    after dispatching through `bridge::parse_object(bytes, BinaryFormat::Pe, "PE")`.
  - `section_permissions` and `segment_permissions` understand both
    `SectionFlags::Elf { sh_flags }` and `SectionFlags::Coff { characteristics }`
    (and the matching `SegmentFlags` arms). PE permissions read from the
    canonical `IMAGE_SCN_MEM_READ` / `IMAGE_SCN_MEM_WRITE` /
    `IMAGE_SCN_MEM_EXECUTE` bits.
  - `map_relocation_flags(flags, arch)` takes the architecture because
    COFF relocation type spaces overlap across `IMAGE_FILE_MACHINE_*`
    targets (each starts at `0x0000`). Per-arch tables map the common
    AMD64 / i386 / ARM64 reloc kinds (`ADDR64`, `ADDR32`, `ADDR32NB`,
    `REL32*`, `BRANCH26`/19/14, `SECTION` / `SECREL` / `SECREL7`).
  - PE base relocations (`.reloc`) are intentionally not surfaced here:
    they describe image rebasing rather than symbol bindings, and the
    import table already covers the symbol-resolution view.
- `dac-binfmt::elf` and `dac-binfmt::pe`:
  - Both are now ≤ a dozen lines: each calls
    `bridge::parse_object(bytes, BinaryFormat::Elf, "ELF")` or
    `(bytes, BinaryFormat::Pe, "PE")` respectively. New fields on
    `BinaryModel` land once in the bridge and reach both formats.
- `dac-binfmt::lib`:
  - `load_from_bytes` now dispatches PE → `pe::parse`. Mach-O still
    returns `Error::UnsupportedFormat` until its parser lands.
  - The hand-built MZ + `PE\0\0` stub test now asserts
    `Error::MalformedBinary { format: "PE", .. }` instead of
    `UnsupportedFormat`, matching the new behaviour.
- `tests/fixtures/` (workspace-root):
  - `hello-x86_64.exe` — PE32+ console exe, debug stripped, COFF symbol
    table kept (~40 KiB). Built with mingw-w64 16.x:
    `x86_64-w64-mingw32-gcc -Os -ffunction-sections -fdata-sections
    -Wl,--gc-sections hello_pe.c` then `--strip-debug`.
  - `hello-x86_64-stripped.exe` — same with `--strip-all` (~16 KiB).
  - `sample.dll` — PE32+ DLL with `sample_add`, `sample_greeting`,
    `sample_value` exports plus an embedded string literal (~30 KiB).
  - `README.md` updated to document the build recipe alongside the ELF
    entries.
- `crates/dac-binfmt/tests/pe.rs` — 10 round-trip integration tests
  cover: PE32+ shape (entry, sections, segments, `.text`); canonical
  section names (`.text` / `.data` / `.rdata` / `.idata`); `IMAGE_SCN_*`
  permissions on `.text` (R+X) and `.data` (R+W); `main` in the COFF
  symbol table on the unstripped fixture; `KERNEL32.dll` in
  `needed_libraries`; stripped variant carries zero symbols but keeps
  its imports; DLL exports include all three names; DLL function
  symbols (`sample_add`, `sample_greeting`) surface with
  `SymbolKind::Text`; embedded string lands in the `StringRef` set;
  FR-2 auto-detection sends PE and ELF buffers to the right parser.
- `crates/dac-cli/tests/cli.rs` — adds `dac_parses_pe_fixture`, which
  runs `dac` against `hello-x86_64.exe` and asserts a clean exit. The
  full-flag-surface test continues to use the ELF fixture; both formats
  are now covered through the CLI end-to-end.
- `crates/dac-binfmt/fuzz/`:
  - New target `fuzz_pe_parse` driving both `detect_format` and
    `load_from_bytes`. Run via
    `cargo +nightly fuzz run fuzz_pe_parse -- -max_total_time=300`
    from `crates/dac-binfmt/`. The 5-minute total-time cap satisfies
    the B1.2 done-when. The in-tree 512-iteration deterministic-PRNG
    smoke in the lib unit tests continues to keep NFR-4 green in stable
    CI for both ELF and PE paths.

Closes: FR-2 (auto-detection of ELF vs PE proven by
`pe_fixture_is_auto_detected_and_dispatched`), FR-3 (PE supported in the
initial release alongside ELF), FR-5 (stripped and unstripped PE both
round-trip), FR-6 (DLL imports + exports preserved through `Import`,
`Export`, and `needed_libraries`). Continues NFR-4 (parser robustness):
PE shares the in-tree stress test through `load_from_bytes` and the
5-minute manual fuzz gate is matched by a dedicated PE target.

#### B1.3 — Architecture trait + x86-64 decoder (2026-06-01)

First step out of the binary-parser layer: the `dac-arch` trait surface
lands, and `dac-arch-x86` rides on top of `iced-x86` (ADR-0004 closes)
to decode every byte of the workspace's ELF and PE `.text` fixtures
end-to-end. Downstream passes see only the arch-neutral
`DecodedInstruction` view; iced types stay inside the decoder module so
the choice is contained to one crate.

- `dac-arch`:
  - `Architecture` trait per ARCHITECTURE.md §7: `name`, `isa`,
    `pointer_size`, `endianness`, `decoder`, `register_file`. `Send +
    Sync` so the pass manager can hand instances across cores
    (NFR-7).
  - `InstructionDecoder` trait with `decode_one(bytes, address) ->
    Result<DecodedInstruction, DecodeError>` and `iter(bytes, address)
    -> Box<dyn Iterator<Item = DecodedInstruction>>`. The iterator
    contract is "consume the whole buffer, emit invalid records inline,
    never stall"; the single-shot path errors only on empty input.
  - `DecodedInstruction { address, length, bytes, mnemonic, operands,
    flow, valid }` — the boundary at which iced (or any future decoder)
    stops being visible. `bytes` is captured so the lifter (B1.4) can
    mint a `Bytes` evidence node without holding a slice into the
    section buffer (I-2).
  - `ControlFlow` enum: `Sequential`, `ConditionalBranch { target }`,
    `UnconditionalBranch { target }`, `IndirectBranch`, `Call { target }`,
    `IndirectCall`, `Return`, `Interrupt`, `Invalid`. Direct branches
    carry their resolved target VA when iced computes one; indirect
    variants surface as the `Indirect*` arms with no target.
  - `DecodeError::Truncated { offset }` covers the "empty buffer"
    failure mode of the single-shot API. Invalid encodings are
    surfaced through `DecodedInstruction::valid = false`, not errors.
  - `Endianness`, `Isa { I386, X86_64, Aarch64 }`.
  - `Register`, `RegisterId`, `RegisterClass`, `RegisterFile` — flat
    register catalogue with id-based lookup, case-insensitive name
    lookup, and parent links for sub-register aliases.
  - 6 unit tests cover the catalogue and `Isa` names.
- `dac-arch-x86`:
  - `X86_64` and `I386` zero-sized `Architecture` impls.
  - `IcedDecoder` implementing `InstructionDecoder` for 16/32/64-bit
    iced. `decode_one` errors on empty input and surfaces invalid
    encodings as `valid = false` records; the linear-sweep `iter`
    re-creates a fresh `iced_x86::Decoder` per step (cheap), defensively
    clamps `length` to remaining buffer for trailing partial
    instructions, and is guaranteed to terminate (consumes every byte
    of the input).
  - Register files for both bitnesses: 16 GPRs + 32/16/8 aliases + RIP +
    RFLAGS for x86-64; 8 GPRs + 16/8-low/8-high aliases + EIP + EFLAGS
    for i386. Sub-register parents point at the 64- or 32-bit base.
    Vector / FP registers land with the lifter once it models them.
  - 19 unit tests cover snapshot decodes (`mov rax, rbx`, `ret`, direct
    + indirect call, conditional + unconditional short branch, indirect
    branch), the iterator (known sequence with full consumption,
    progress past invalid bytes), the empty-input error path, the
    invalid-encoding degradation path, and the i386 vs x86-64 bitness
    split.
- `crates/dac-arch-x86/tests/text_roundtrip.rs` — the B1.3 done-when.
  Two integration tests (`elf_hello_text_round_trips`,
  `pe_hello_text_round_trips`) load the shared ELF and PE fixtures
  through `dac-binfmt`, locate `.text`, run the decoder iterator across
  every byte, and assert: full byte consumption, strictly increasing
  addresses, ≥ 10 instructions, ≥ 95% validity, at least one `ret`,
  at least one `call` (direct or indirect).
- `crates/dac-arch-x86/fuzz/`:
  - libFuzzer crate scoped out of the parent workspace with its own
    empty `[workspace]` block (mirrors the binfmt fuzz layout).
  - Single target `fuzz_x86_decode` runs both `decode_one` and the
    linear-sweep iterator for both `X86_64` and `I386`, with a length-
    safety cap on iteration so adversarial inputs cannot OOM the
    fuzzer. Run via
    `cargo install cargo-fuzz && cargo +nightly fuzz run fuzz_x86_decode -- -max_total_time=300`
    from `crates/dac-arch-x86/`. The 5-minute total-time cap matches
    the B1.3 done-when. The deterministic snapshot path is covered by
    in-tree unit tests; this target covers the open-ended NFR-4-style
    robustness invariant for the decoder.
- `DECISIONS.md`: ADR-0004 closes on `iced-x86` with the full rationale,
  the rejected alternatives (`yaxpeax-x86`, `capstone-rs`, hand-rolled),
  and the boundary the choice draws (iced types do not leak past
  `dac-arch-x86`).
- `Cargo.toml`: `iced-x86 = { version = "1.21", default-features =
  false, features = ["std", "decoder", "instr_info", "intel"] }` added
  to `[workspace.dependencies]`.

Closes: ADR-0004 (x86 decoder library choice). Sets up the
Instruction-IR + lifter work in B1.4 by giving it a stable
`DecodedInstruction` to consume and a register file to bind operand
names against. Continues I-6 (decoder degrades to `(bad)` on invalid
input rather than inventing semantics).

#### B1.4 — Instruction IR + x86-64 lifter (2026-06-01)

First arch-neutral IR layer (`ARCHITECTURE.md` §4) plus an iced-backed
lifter that projects decoded x86 / x86-64 instructions onto it.
Coverage on the workspace's ELF and PE `.text` fixtures clears 98.5%
— comfortably above the 95% gate — with `hlt`, conditional set/move
(`sete`, `setne`, `cmove`), SSE moves (`movaps`, `movsd`, `movups`,
`unpcklpd`), `cdqe`, `cmpxchg`, `xchg`, and `fninit` landing as
`Opaque` for later batches to model. CFG construction (B2.1) and
function discovery (B1.5) still see every byte through the decoder's
`ControlFlow` projection, so the opaques do not break downstream
passes (I-6).

- `dac-ir::instr` (new):
  - `InstructionIr { address, length, op }` — the per-instruction
    node. Address + length together name the byte span the node was
    lifted from, which is the provenance hook an orchestrator turns
    into `EvidenceNode::Bytes` + `EvidenceNode::IrNode` edges
    (I-2). `is_lifted` is `false` only for `Opaque` so it is the
    predicate `Coverage` counts against; `byte_range` returns the
    half-open span the orchestrator needs.
  - `Operation` — closed enum: `Move`, `LoadAddress`, `Add`, `Sub`,
    `Mul`, `Div`, `And`, `Or`, `Xor`, `Shl`, `Shr`, `Sar`, `Not`,
    `Neg`, `Compare`, `Test`, `Push`, `Pop`, `Jump { target,
    condition }`, `Call { target }`, `Return`, `Nop`, `Interrupt
    { vector }`, `Syscall`, `Opaque { mnemonic }`. New ops land as
    new variants; existing consumers must explicitly handle them.
  - `Operand` — typed operand vocabulary: `Register { name,
    size_bits }`, `Immediate { value, size_bits }`, `Memory { base,
    index, scale, displacement, size_bits, segment }`, `Branch
    { target }`. Register names are lowercase canonical strings that
    match `RegisterFile::by_name`; the IR stays decoupled from any
    ISA's register catalogue.
  - `Target { Direct(u64), Indirect(Operand) }` and `Condition` (17
    arch-neutral codes covering the x86 `Jcc` set: `Equal`,
    `NotEqual`, signed `Less`/`Less­Equal`/`Greater`/`GreaterEqual`,
    unsigned `Below`/`BelowEqual`/`Above`/`AboveEqual`, `Sign`,
    `NotSign`, `Overflow`, `NotOverflow`, `Parity`, `NotParity`,
    `CxZero`). `Condition` carries a `Display` impl using the
    shortest canonical name (`eq`, `ne`, `b`, `ae`, …).
  - 5 unit tests cover `is_lifted` / `byte_range` for both lifted and
    opaque records, `Condition::Display`, `Target::Indirect`'s
    addressing-form preservation, and the wrapping-overflow
    `byte_range` contract at the top of `u64`.
- `dac-arch`:
  - `InstructionLifter` trait: `lift(bytes, address) ->
    InstructionIr`. Pure (no mutable evidence-graph state); callers
    that want a sweep pair it with `InstructionDecoder::iter`. Always
    returns IR — unsupported opcodes land as `Operation::Opaque`
    rather than errors, so function discovery and CFG construction
    never have to skip an instruction.
  - `Coverage { total, lifted, opaque, opaque_mnemonics: BTreeMap }`
    — fold one record at a time via `record`, surface the ratio with
    `lifted_fraction`. `Display` impl emits the report with opaque
    mnemonics in lexicographic order (NFR-9: deterministic).
  - `Architecture` trait grows `fn lifter(&self) -> Box<dyn
    InstructionLifter>`, matching ARCHITECTURE.md §7. Crate gains a
    `dac-ir` dependency so the lifter trait can reference the IR
    types.
  - 3 unit tests cover the empty-coverage zero-fraction edge case,
    lifted-vs-opaque counting + opaque histogram, and the
    sort-stability invariant on the `Display` impl.
- `dac-arch-x86`:
  - `IcedLifter` implementing `InstructionLifter` for 16/32/64-bit
    iced. Matching order: syscalls up-front (iced does not put them
    under `FlowControl::Interrupt`), control flow via
    `instr.flow_control()` (single source of truth, mirrors the
    decoder's `ControlFlow` projection), then per-mnemonic for the
    common arithmetic / data-movement / stack subset. Three-form
    `imul` projects onto a single `Mul { dst, src }` arm. `inc` and
    `dec` lower to `Add` / `Sub` of immediate `1`. `endbr32` /
    `endbr64` land as `Nop`. Empty input lifts to a zero-length
    `Opaque { mnemonic: "(empty)" }` so iterators that pair the
    lifter with the decoder never see a panic. Invalid encodings
    land as `Opaque { mnemonic: "(bad)" }` matching the decoder's
    degradation policy (I-6).
  - 19 unit tests cover snapshot lifts (`mov rax, rbx`, `lea` with
    memory operand, direct + indirect call, conditional branch with
    signed code, unconditional branch, `ret`, `push` / `pop`, `add`
    with immediate, `xor` self-zero, `inc` lowered to `add 1`,
    `cmp` + `jne` sequence, `syscall`, `int3` with vector, `nop` +
    `endbr64` parity, invalid `0x06` → `(bad)`, unmodelled `addss`
    → `Opaque`), the empty-input edge case, and the `Architecture::
    lifter` trait-object path.
  - `IcedLifter` exported alongside `IcedDecoder`; the lifter is
    wired into both `X86_64.lifter()` and `I386.lifter()`.
- `crates/dac-arch-x86/tests/lift_coverage.rs` — the B1.4 done-when.
  Two integration tests (`elf_hello_meets_coverage_floor`,
  `pe_hello_meets_coverage_floor`) load the shared ELF and PE
  fixtures through `dac-binfmt`, run the decoder iterator across
  every byte of `.text`, lift each record, and assert the resulting
  `Coverage::lifted_fraction` clears `0.95`. The `Coverage` value's
  `Display` impl is included in the assertion message so a regression
  surfaces the per-mnemonic histogram alongside the percentage.
- `Cargo.toml`: `dac-ir` added as a dependency to both `dac-arch`
  (so the lifter trait can name `InstructionIr`) and `dac-arch-x86`
  (so the iced lifter can produce one).

Closes: I-2 (instruction IR nodes carry the byte span that an
orchestrator wires into the evidence graph), I-6 (unsupported
opcodes degrade to `Opaque` so passes still see CFG edges). Closes
the B1.4 done-when by clearing 98.5% lifter coverage on the sample
corpus's `.text`. Sets up B1.5 (function discovery) by giving it
both control-flow classification (from the decoder) and a typed
operand view (from the lifter) per instruction.

#### B1.5 — Function discovery (2026-06-02)

First recovery pass on top of the Foundation: `dac-recovery::functions`
discovers function entry points (and best-effort end bounds) from four
independent signals and records each fact in the evidence graph as a
`Cfg`-layer node supported by a byte-range node (I-2). Recall on the
sample corpus's unstripped ELF and PE clears 100%; stripped variants
recover 5 (ELF) and 50 (PE) functions through entry + call-edge +
prologue alone — the "tracked but not gated" stripped branch the plan
calls out.

- `dac-recovery::functions` (new):
  - `discover_functions(model, bytes, decoder, graph) -> FunctionSet`
    is the single entry point. It walks `BinaryModel::symbols`
    (Source::Observed, value 1.0), `BinaryModel::entry`
    (Source::Observed, value 1.0), every direct-call edge through the
    decoder's `ControlFlow::Call { target: Some(_) }` projection
    (Source::Derived, value 0.85), and the
    `push rbp; mov rbp, rsp` + `endbr64; push rbp` / `endbr64; sub rsp,…`
    prologue patterns (Source::Derived, value 0.6). When several signals
    agree on the same address their `Confidence`s combine through
    `Confidence::join` (componentwise max) and a `SourceMask` bitset
    records every contributing signal, so a `--debug` consumer can
    inspect *why* a function was promoted.
  - `Function { address, end, name, confidence, sources, evidence }`
    is the discovered record. `evidence` points at an
    `EvidenceNode::IrNode { layer: Cfg, id }` whose `id` is the
    function's index in `FunctionSet::functions`. Each function also
    has a sibling `EvidenceNode::Bytes { start, end }` for its byte
    span with a `Supports` edge into the IR node — the substrate later
    batches attach signature, calling-convention, and type facts to.
  - End-bound recovery: symbol-derived entries arrive with a size,
    everything else lands with `end = None`. A final pass walks the
    discovered functions in address order and fills each unknown end
    with the next function start *inside the same executable section*,
    falling back to the section end. CFG construction (B2.1) consumes
    the resulting half-open byte ranges directly.
  - `DiscoveryStats { from_symbol, from_entry, from_call, from_prologue }`
    counts each signal's contribution exactly once per function, so
    the histogram never double-counts when signals merge.
  - `SourceMask` is a typed bit-flag wrapper (no `bitflags` dep) with
    associated constants `SYMBOL`, `ENTRY`, `CALL`, `PROLOGUE` and
    `empty` / `is_empty` / `contains` / `insert` / `bits`.
  - The discoverer filters to sections with `perms.executable &&
    file_offset.is_some() && size > 0`, so PLT stubs / `.init` /
    `.fini` are first-class function hosts alongside `.text` (call
    edges into the PLT surface as real `Function`s).
  - 9 unit tests cover: `SourceMask` independence, symbol-derived
    discovery (name / size / `Source::Observed` / mask), entry-point
    discovery with end filled from the section bound, symbol + entry
    merging at the same address (both signals contribute to stats but
    only one `Function` is emitted), neighbour-derived end filling
    across two same-section functions, `FunctionSet::contains_address`
    / `get` / `addresses` ordering, evidence-node wiring
    (`IrNode { layer: Cfg, id: 0 }` with a `Bytes` `Supports` edge),
    out-of-section symbols ignored, undefined symbols ignored. The
    unit tests use a `NullDecoder` so the symbol / entry paths are
    isolated from the iced sweep.
- `crates/dac-recovery/tests/function_discovery.rs` — the B1.5 done-when.
  Five integration tests load the shared ELF and PE fixtures through
  `dac-binfmt`, decode `.text` (and any other executable section)
  through the iced-backed `X86_64` lifter, and assert:
  - `elf_hello_meets_recall_gate`: unstripped ELF recall ≥ 0.98 against
    the symbol-table ground truth (`Text` symbols with defined
    addresses in executable sections). Actual: 100% (8 / 8). All four
    signals contribute (`from_symbol`, `from_entry`, `from_call`,
    `from_prologue` all > 0).
  - `pe_hello_meets_recall_gate`: unstripped PE recall ≥ 0.98. Actual:
    100% (75 / 75) with 48 call-derived merges across symbol-known
    functions.
  - `elf_stripped_still_yields_functions` /
    `pe_stripped_still_yields_functions`: stripped fixtures still emit
    a non-empty set through entry + call edges (5 ELF / 50 PE
    functions) — the "tracked but not gated" branch the plan calls
    out.
  - `unstripped_functions_intersect_with_call_sources`: at least one
    function on the unstripped ELF has both `SourceMask::SYMBOL` and
    `SourceMask::CALL` set, exercising the merge path inside the
    accumulator. Actual: 4 merges across 9 functions.
- `crates/dac-recovery/examples/discovery_stats.rs` — convenience
  example that dumps `discovered / ground / recall / stats / merges`
  for every fixture. Run with
  `cargo run -p dac-recovery --example discovery_stats`. Used while
  writing this CHANGELOG entry; future per-corpus runs reuse it.
- `crates/dac-recovery/Cargo.toml`: adds `dac-arch`, `dac-binfmt`, and
  `dac-core` as runtime deps, and `dac-arch-x86` as a dev-dep for the
  integration / example tests. No new external dependencies.

Closes: FR-9 (function-boundary recovery without symbols — entry,
call edges, and prologue heuristics carry stripped binaries through),
I-2 (per-function `Bytes` + `IrNode` evidence with `Supports` edges),
I-3 (every recovered function carries a `Confidence` value and a
`Source` class, joined deterministically across signals). Sets up
B2.1 (CFG construction) with a half-open byte range per function and
an evidence handle to attach per-block facts onto.

#### B1.6 — `-O0` text backend (2026-06-02)

Closes Milestone 1: the CLI now drives an end-to-end pipeline from
`dac-binfmt` through `dac-recovery` to a deterministic textual artifact
set. `-O0` is not a "real" backend — it emits an annotated listing of
the lifted IR — but it is the first user-visible end-to-end output and
the first place every prior batch (parser → decoder → lifter →
discovery) lands together. `dac sample.elf -O0` re-run twice produces
byte-identical listing + manifest + report; the new
`o0_golden.rs` integration test gates that for both ELF and PE
fixtures, with and without `--emit-report`.

- `dac-cli::listing` (new):
  - `render_listing(input_name, model, bytes, decoder, lifter, functions, opts)`
    produces a deterministic textual view of every discovered function.
    Per-function header records address range, joined confidence, and
    contributing signal mask (`symbol, entry, call, prologue`).
    Per-instruction lines carry virtual address, the first 8 encoded
    bytes (further bytes elided as ` .` so the column stays at fixed
    width), the disassembly text from
    `dac_arch::DecodedInstruction`, and the lifted-IR projection from
    [`format_ir`] in the comment column.
  - `ListingOptions { with_headers, with_bytes, with_ir }` for tighter
    diffs; defaults to all-on so the canonical listing carries the
    fullest provenance view.
  - `format_ir` projects an `InstructionIr` onto a one-line semantic
    form (e.g. `mov(rax:64, rbx:64)`, `call(@0x1030)`,
    `opaque(vfmadd)`). Reads each `Operation` variant explicitly; new
    variants get a new arm.
  - 4 unit tests cover the empty-function-set note, byte-stability
    across two renders of the same input, `format_ir` shape for lifted
    + opaque instructions, and `format_sources` canonical bit ordering.
- `dac-cli::manifest` (new):
  - `Manifest { tool, input, settings, ai_provider, plugins }` with
    sub-structs for each field group. `tool` records name + version +
    build_id (NFR-10); `input` records path + size + format +
    architecture; `settings` records `-O` level, target, and every
    user-visible flag from the spec §10.1 surface.
  - `render_manifest_json` hand-rolls the JSON (no `serde` /
    `serde_json` dep) with a fixed key order, sorted output, escape
    handling for control chars and quotes. Byte-stable across calls.
  - Optional fields (`ai.provider`, `settings.threads`) render as JSON
    `null` when absent so the schema does not shift between runs.
  - 5 unit tests cover byte-stability, NFR-10 field presence, AI
    provider rendering, threads rendering, and string escaping for the
    quote / newline / sub-0x20 cases.
- `dac-cli::report` (new):
  - `Report { function_count, from_*, coverage, functions }` with
    `FunctionSummary` for the per-function lines. `Report::build`
    folds a `dac_arch::Coverage` across every discovered function by
    pairing the decoder iterator with the lifter, so the report's
    lifted-fraction is computed against *only* the function bytes
    (not the full `.text`).
  - `render_report_text` emits a deterministic text form. Header
    declares function count + signals + lift coverage; per-function
    block prints `address..end  name  source/confidence  sources`;
    trailing "unresolved opaque mnemonics" block lists every opaque
    mnemonic by lexicographic order.
  - 3 unit tests cover the aggregate signal count, the per-function
    summary lines, and byte-stability across two renders.
- `dac-cli` orchestration changes in `main.rs`:
  - `run_pipeline(path, args)` replaces the prior log-only `run()`. It
    loads the binary, picks an architecture backend (currently
    `X86_64`, returns `None` for unsupported archs), discovers
    functions, builds the listing / manifest / optional report, and
    writes the outputs.
  - `Backend { decoder, lifter }` holds the architecture-specific
    trait objects; the only x86 dispatch in the pipeline is here.
  - Unsupported architectures land on `unsupported_arch_listing` /
    `unsupported_arch_report` — they still emit a manifest + a stub
    listing, never panic. This matches the format-detection contract
    from B0.2 (NFR-4 robustness).
  - Output routing: no `--output` writes listing + manifest (delimited
    with `;; ---- manifest (NFR-10) ----`) and optional report to
    stdout in one stream. `--output <path>` writes listing to
    `<path>`, manifest to `<path>.manifest.json`, and the report to
    `<path>.report.txt` when `--emit-report` is set. Sidecar paths
    are computed by appending the suffix to the raw `OsStr` so paths
    without extensions (`/tmp/out`) and paths with extensions
    (`/tmp/out.c`) both round-trip predictably.
- `crates/dac-cli/Cargo.toml`: adds `dac-arch`, `dac-arch-x86`,
  `dac-ir`, `dac-recovery` as runtime deps. No new external deps.
- `crates/dac-cli/tests/o0_golden.rs` (new) — the B1.6 done-when. Four
  integration tests run `dac` twice against shared fixtures and assert
  the listing, manifest, and (when emitted) report files are
  byte-identical between the two runs:
  - `elf_o0_output_is_stable_across_reruns` on `hello-x86_64`. Also
    pins the preamble line, the `;; format:    ELF` / `;; arch:
    x86-64` markers, and the NFR-10 manifest fields.
  - `pe_o0_output_is_stable_across_reruns` on `hello-x86_64.exe`.
    Pins the `;; format:    PE` marker.
  - `elf_o0_with_emit_report_is_stable_across_reruns` adds
    `--emit-report` to the run, asserts the listing still records the
    function count line, and the report carries the FR-25 header.
  - `stripped_elf_o0_output_is_stable_across_reruns` runs on the
    stripped fixture so the test covers the entry / call / prologue
    branch of the discoverer (the symbol set is empty here).
- Sample output preamble for `dac -O0 hello-x86_64`:
  ```text
  ;; dac -O0 annotated listing
  ;; input:     tests/fixtures/hello-x86_64
  ;; format:    ELF
  ;; arch:      x86-64
  ;; entry:     0x0000000000001060
  ;; size:      15968 bytes
  ;; functions: 9
  ;; signals:   symbol=8 entry=1 call=2 prologue=2

  ;; ============================================================
  ;; function _init [0x0000000000001000..0x000000000000101b) (27 bytes)
  ;;   confidence: observed 1.000
  ;;   sources:   symbol, prologue
  ;; ============================================================
  0x0000000000001000  f3 0f 1e fa                   endbr64    ; nop
  0x0000000000001004  48 83 ec 08                   sub rsp,8  ; sub(rsp:64, 8#64)
  …
  ```

Closes: FR-22 (`-O0` is wired end-to-end; the higher `-O` levels build
on the same orchestration in B2.x / M4), FR-25 (analysis report with
per-function confidence + source attribution + lift coverage +
opaque mnemonic histogram), NFR-9 (deterministic across re-runs gated
by `o0_golden.rs`), NFR-10 (reproducibility manifest records tool
version + build id + analysis settings + backend + AI + plugins).
Completes Milestone 1 of `PLAN.md` — every M1 batch (B1.1 through
B1.6) has now landed.

### Milestone 2 — Core decompilation

#### B2.1 — CFG construction (2026-06-02)

Per-function control-flow graph in `dac-analysis::cfg`: basic blocks
split at every leader (function entry, branch targets,
post-terminator addresses), edges classified into `Fall` / `Branch` /
`Taken` / `NotTaken`, entries / exits / unreachable blocks recorded
explicitly. The builder reads only through the existing
`dac_arch::ControlFlow` classification so no ISA knowledge leaks into
this layer, and refuses to invent edges when a target is unresolved
or out-of-function (I-6). `--emit-cfg` exports a deterministic
Graphviz DOT file, one `digraph` per function, sorted by function
address, with stable `BB<id>` node ids — byte-identical across
re-runs.

- `dac-analysis`:
  - `cfg::Cfg { function_address, function_end, function_name,
    blocks, entry, exits, edges, unreachable, evidence }` — the
    per-function CFG. Block ids are dense indices into `blocks`, so
    `Edge::from` / `Edge::to` index directly. The function's
    `EvidenceId` is inherited from `dac_recovery::Function::evidence`,
    keeping CFG facts attached to the same evidence node B1.5 minted.
  - `cfg::BasicBlock { id, address, end, instructions, terminator }`
    holds decoded instructions in address order. May be empty if a
    leader landed where the linear sweep produced no decode; the
    block still appears so reachability is honest about it.
  - `cfg::Terminator` closed enum — `Fall`, `Branch { target }`,
    `Conditional { target }`, `Indirect`, `Call { target }`, `Return`,
    `Interrupt`, `Invalid`. The branch / conditional / call targets
    preserve the decoder-supplied VA even when out-of-function so the
    future call-graph pass (B3.1) can detect tail calls.
  - `cfg::Edge { from, to, kind }` + `cfg::EdgeKind` — sorted by
    `(from, kind discriminant, to)` for deterministic output.
  - `cfg::build_cfg(function, model, bytes, decoder) -> Option<Cfg>`
    runs the leader-detection + block-building + BFS-reachability
    pipeline. Returns `None` only when the function span cannot be
    resolved (no `end`, truncated section, …); never panics on
    garbage input (NFR-4).
  - `cfg::build_cfgs(functions, …) -> Vec<Cfg>` — convenience over a
    `FunctionSet` slice; silently skips functions that can't be
    built.
  - `cfg::render_dot(cfg) -> String` + `cfg::render_dot_all(cfgs) ->
    String` — DOT exporters. Graph names are
    `fn_<sanitized_name>_<hex_addr>` so duplicate names cannot
    collide. Entry blocks are filled gray; unreachable blocks are
    dashed. Labels escape backslashes / quotes / newlines into DOT
    syntax (`\l` for left-justified line breaks).
  - 14 unit tests cover the hand-checked-reference deliverable:
    single-return, linear, conditional diamond, post-`jmp` orphan,
    self-edge loop, out-of-function tail exit, call-fall-through,
    indirect branch, conditional with out-of-range taken side,
    DOT byte-stability, unresolved conditional (`target: None`),
    `render_dot_all` address ordering, the escape function in
    isolation, and target-not-on-instruction-boundary (no edge
    minted).
- `dac-cli`:
  - `--emit-cfg` is now live. With `--output <path>` the DOT lands
    at `<path>.cfg.dot`; without `--output` it appends to stdout
    after the manifest / report, delimited by
    `;; ---- cfg (FR-28) ----`.
  - Non-x86_64 inputs still emit a (valid, empty) DOT digraph rather
    than failing — keeps the binary-format layer usable end-to-end
    regardless of arch backend availability.
  - New `dac-analysis` workspace dependency.
- Integration tests:
  - `crates/dac-cli/tests/cfg_emit.rs` runs `dac -O0 --emit-cfg
    --output <tmp>` twice on each of the ELF / PE / stripped-ELF
    fixtures and asserts the DOT sidecars are byte-identical (the
    determinism gate for `--emit-cfg`).
  - Structural sanity: every fixture's DOT contains at least one
    `digraph "fn_…"` and a `BB0` entry-block declaration; the
    hello-world ELF additionally contains at least one classified
    edge label (`fall` / `jmp` / `T` / `F`).

Test counts: `cargo xtask ci` reports 42 green `test result: ok`
lines (was 41 at end of B1.6) — +1 dac-analysis lib + 1 dac-cli
integration binary. No new warnings.

Closes: FR-10 (control-flow graphs for recovered functions), FR-28
(export CFGs as DOT). Determinism is gated by the new integration
test (NFR-9). The CFG carries the function's evidence handle, so
I-2 is preserved across the new layer.

#### B2.2 — Dominators + loop nest (2026-06-02)

Dominator and post-dominator trees, natural-loop detection, and a
loop nest forest land in `dac-analysis::dom` and
`dac-analysis::loops`. The dominator computation is the
Cooper-Harvey-Kennedy iterative algorithm walking blocks in reverse
postorder; the post-dominator computation runs the same algorithm
on the reverse CFG augmented with a synthetic virtual exit that
merges every CFG exit. Natural loops are derived from back-edges
(CFG edges `u → v` where `v` dominates `u`), their bodies
materialised by reverse BFS from each back-edge source. Reducibility
is detected by counting external entry points per non-trivial SCC;
irreducible CFGs are flagged so the structuring pass (B2.7) can
fall back to `goto` in the C backend.

- `dac-analysis::dom`:
  - `DominatorTree { idoms, entry }` with `build(&Cfg) -> Self`,
    `idom(b) -> Option<BlockId>`, `entry() -> BlockId`,
    `dominates(a, b) -> bool`, `strictly_dominates(a, b) -> bool`,
    `children(a) -> Vec<BlockId>`. Convention: `idom(entry) ==
    Some(entry)`; `idom(unreachable) == None`. The dominance check
    walks the idom chain so it never panics on out-of-range ids.
  - `PostDominatorTree { ipdoms, n_blocks }` with `build(&Cfg) ->
    Self`, `ipostdom(b) -> PostDom`, `post_dominates(a, b) -> bool`.
    The synthetic virtual exit is internal; callers see the three
    public states `PostDom::Block(id)` / `PostDom::Exit` /
    `PostDom::Unreachable`, so a CFG exit and an infinite-loop
    block are distinguishable without leaking sentinel ids.
  - Crate-private helpers `predecessors_of` / `successors_of` build
    sorted + de-duplicated adjacency from `Cfg::edges` so a block
    with parallel edges of different `EdgeKind`s does not double-
    count for dominance.
- `dac-analysis::loops`:
  - `LoopForest { loops, roots, header_of, innermost, irreducible }`
    with `build(&Cfg, &DominatorTree) -> Self`. Loop ids are
    assigned in ascending-header order; `roots` and `children` are
    sorted; `innermost[i]` records the deepest loop containing
    block `i` for cheap per-block queries.
  - `Loop { id, header, body, back_edges, parent, children, depth }`.
    `body` is sorted ascending and always contains `header`; the
    parent relation is the smallest enclosing loop whose body
    contains `header` (i.e. the natural nesting from header
    containment, not just dominance).
  - `LoopForest::irreducible` is true iff at least one non-trivial
    SCC has more than one entry point — a node inside the SCC with
    a predecessor outside it, or the CFG entry. Detected by an
    iterative Tarjan SCC pass; trivial SCCs (single nodes without
    self-loops) are skipped.
  - Back-edge enumeration and body BFS both skip blocks unreachable
    from the function entry — the dominance check is vacuously
    false for them, so without filtering the BFS would leak into
    orphan predecessor chains that never pass through the header.
  - 13 unit tests cover the canonical reference topologies:
    linear (no loops), self-loop, while-style, do-while with the
    body as header, nested two-level forest, sibling loops, multi-
    back-edge with merging, irreducible two-entry SCC,
    early-exit / break, unreachable self-loop (not flagged),
    plain natural-loop body, determinism across rebuilds, and a
    single-block SCC with a self-loop.
- Tests:
  - `crates/dac-analysis/tests/corpus_loops.rs` runs the entire
    discover → CFG → dom → loops pipeline on the ELF / PE /
    stripped-ELF fixtures. For every recovered function it asserts
    that every loop's header dominates every body block, every
    back-edge source is dominated by its header, and the body
    contains the header. A determinism test runs the pipeline
    twice on the ELF fixture and asserts the resulting `LoopForest`
    vectors are equal (NFR-9).

Test counts: `cargo xtask ci` reports 43 green `test result: ok`
lines (was 42 at end of B2.1) — +1 dac-analysis integration test
binary (`corpus_loops`). Lib test count grew from 14 to 37 inside
`dac-analysis` (the 14 existing CFG tests, plus 10 dominator /
post-dominator tests and 13 loop tests). No new warnings.

Closes: FR-10 (dominators / loops as part of the CFG analysis
layer). The reducibility flag covers the prerequisite I-6 fallback
the C backend will rely on at B2.7. Determinism is gated by the
loop-forest equality test in the corpus integration suite (NFR-9).
No new evidence nodes are minted — dominators and loops are
derived facts attached to the existing per-function CFG node
inherited from B1.5 (I-2).

#### B2.3 — SSA construction (2026-06-02)

Pruned SSA construction lands in `dac-ir::ssa` (the IR types) and
`dac-analysis::ssa` (the algorithm). Phi placement uses the standard
Cytron-Ferrante-Rosen-Wegman-Zadeck walk over dominance frontiers,
pruned by a backward liveness pass so dead variables do not collect
phi nodes at every merge. Renaming is a pre-order DFS of the
dominator tree with one ValueId stack per abstract variable; phi
incoming entries are filled in while each block's terminator is
processed so the operands seen at the join match the definition
visible to that predecessor at end-of-block. A block-local value-
numbering pass collapses trivial CSE candidates by hashing
instructions on `(op kind, operands)`. The construction is
decoupled from the lifter: it takes a `RawFunction` of per-block
variable-keyed operations so the SSA pass can be tested against
hand-built programs without dragging in a real architecture
lifter (the InstructionIR → `RawFunction` bridge is B2.4+ work).

- `dac-ir::ssa`:
  - `SsaFunction { function_address, function_name, blocks, entry,
    variables, values, evidence }` — the per-function SSA graph.
    Block ids match the source CFG; the `evidence` handle is
    inherited from the function's CFG node so dataflow passes can
    attach further facts to the same evidence id (I-2).
  - `SsaBlock { id, predecessors (sorted), phis, instructions,
    terminator }` and a closed `SsaTerminator` enum (`Jump`,
    `Branch`, `Return`, `Indirect`, `Unreachable`). The
    `Unreachable` variant is retained for CFG blocks the lifter
    could not translate, so the SSA function still has one block
    per CFG block (I-2 traceability).
  - `Phi { dst, variable, incoming }` — the incoming list is
    sorted by predecessor block id, with `Operand::Undef`
    inserted on predecessors where the variable has no reaching
    definition (rather than silently inventing a zero — I-6).
  - `SsaInstruction { dst, op }`, `SsaOp` closed enum covering
    arithmetic, bitwise, compare, load/store, call, and a final
    `Opaque` arm mirroring `Operation::Opaque` in the Instruction
    IR. New operations land as new variants.
  - `Operand` (`Value(id) | Const(c) | Undef`) implements `Ord`
    via a structural key (`Undef < Const < Value`) so passes can
    use operand sequences as `BTreeMap` keys without re-deriving
    the comparison.
  - `ValueDef { id, source, variable }` with `ValueSource ::=
    Instruction { block, index } | Phi { block, index } |
    Parameter { variable }`. Parameter values represent reads of
    a variable that has no reaching definition along the path
    from entry — one Parameter id per variable, shared across
    every use so value-numbering treats two reads of an unwritten
    register as equal.
- `dac-analysis::ssa`:
  - `RawFunction { variables, blocks }`, `RawBlock { ops,
    terminator }`, `RawOp { dst, kind }`, `RawOpKind`,
    `RawOperand`, `RawTerminator` — the lifter-facing input
    types. Mirrors the SSA op vocabulary but with `VariableId`
    operands in place of `ValueId`.
  - `construct_ssa(cfg, doms, raw) -> SsaFunction` — the whole
    pipeline in one entrypoint. Asserts that
    `raw.blocks.len() == cfg.blocks.len()` so the SSA function
    shape mirrors the CFG.
  - `dominance_frontiers(doms, preds, n)` — Cytron's iterative
    DF computation. Skips unreachable predecessors so an orphan
    block does not poison its ancestors' frontiers.
  - `compute_live_in(...)` — backward liveness over the CFG,
    returning per-block `LiveIn` sets. Drives the phi-pruning so
    only variables actually consumed at a join carry a phi
    there.
  - `place_phis(...)` — worklist over variables, placing phis at
    each `DF(defining_block)` whose `LiveIn` contains the
    variable. A new phi counts as a fresh definition, so its
    block joins the worklist.
  - `RenameState` (internal) — drives the dominator-tree DFS
    iteratively with explicit `(block, child_index)` work
    entries so it never panics on deep dominator trees.
    Pre-seeds phi slots in each block before rename so phi
    incoming entries can be appended in DFS order, then sorted
    by predecessor at build time for byte stability.
  - `local_value_number(ssa)` — block-local CSE. Hashes
    instructions by a `VnKey` (op discriminant + operands). The
    first occurrence claims the key; later matches drop their
    instruction and record a `ValueId → ValueId` remap. After
    every block has been processed, the remap is applied
    globally to phi incoming entries, instruction operands, and
    terminators. `ValueDef` entries for folded ids are kept in
    place so `values[id].id == id` stays stable; consumers reach
    values through phi/instruction `dst` fields and never see
    the orphan entries. Load/Store/Call/Opaque are excluded
    from value-numbering keys — their result is not a function
    of their operands alone (memory state / side effects).
  - 15 unit tests covering linear renaming, diamond-join phi,
    pruning (no phi for a dead variable), loop-header phi with
    initial + back-edge incoming, parameter creation for
    use-without-def, single-block and cross-block CSE behavior,
    side-effect preservation (Load not folded), three-way phi
    incoming sort order, evidence/address inheritance,
    determinism across rebuilds, unreachable-terminator
    preservation, dominance-frontier correctness on a diamond,
    and a direct liveness check.
- Tests:
  - `crates/dac-analysis/tests/ssa_roundtrip.rs` is the
    done-when. Five small functions are interpreted twice: once
    against the raw (variable-based) form and once against the
    constructed SSA form, with phi-arg selection threaded
    through the predecessor block id at each block transition.
    Every input must produce the same return value on both
    interpreters — linear chain, branch-merge with phi, nested
    branches with an inner join feeding the outer phi, a
    while-style loop with header phi, and a CSE case where two
    redundant adds must collapse without changing the
    observable return value.

Test counts: `cargo xtask ci` reports 44 green `test result: ok`
lines (was 43 at end of B2.2) — +1 dac-analysis integration test
binary (`ssa_roundtrip`). Lib test count grew from 37 to 52 inside
`dac-analysis` (the 37 existing + 15 SSA tests); `dac-ir` lib
tests grew from 5 to 7 (+2 for the new `ssa` module's basic
helpers). No new warnings.

Closes: FR-11 (use-def chains / SSA construction). Determinism
is gated by the `ssa_12_construct_is_deterministic_across_runs`
unit test (NFR-9). No new evidence nodes are minted — SSA is a
derived rewrite of the existing CFG, inheriting the function's
CFG evidence handle so the provenance chain at I-2 stays
intact. The `RawFunction` input layer keeps the algorithm
testable in isolation; wiring it to a real `InstructionIr`
stream is B2.4's problem.

#### B2.4 — Dataflow + stack-frame recovery (2026-06-02)

Two passes land together: SSA-level dataflow in
`dac-analysis::dataflow` (def-use chains, per-block liveness),
and stack-frame recovery in `dac-recovery::stack` (locals,
incoming args, and Windows shadow space recovered from SSA
address arithmetic anchored at the entry stack pointer). Both
are deterministic and depend only on outputs already produced
by B2.1–B2.3.

The SSA construction in B2.3 already gives use-def for free —
every operand directly names its defining value via [`ValueId`]
— so this batch ships the inverse direction (def-use) and a
liveness pass. Reaching definitions are deliberately not a
separate pass: in SSA each operand already names its single
reaching definition, and the interesting "which store reaches
this load" version is a memory-SSA concern that lives in a
later batch.

The stack-frame pass identifies every memory location a
function touches at a constant offset from the entry stack
pointer. The SSA constructor mints a `Parameter` value for
every variable read without first being written; for `rsp`
that parameter is *the* `entry_sp` anchor. The pass propagates
`(anchor, offset)` through `Move`/`Add`/`Sub` (constant
operand), folds phi nodes whose every incoming agrees on the
same offset, and collects every `Load`/`Store` whose address
resolved to `entry_sp + k`. The frame pointer (if any) is
detected as the first instruction whose destination variable
matches the convention's nominated FP register (e.g. `rbp`) and
whose offset is known; accesses through it fold back onto the
same anchor with no extra mechanism. Per I-6, alignment masks
(`and rsp, -16`) and other non-additive transforms do *not*
propagate — the pass treats them as unknown rather than
guessing.

- `dac-analysis::dataflow`:
  - `DefUseChains` — inverted SSA value graph keyed by
    [`ValueId`]. A use-site appears once per syntactic
    occurrence so `Add { lhs: v, rhs: v }` records two
    instruction uses for `v` (classic "number of uses"
    semantics for DCE / copy propagation).
  - `UseSite` enum: `Phi { block, phi, incoming } |
    Instruction { block, index } | Terminator { block }`.
    Implements `Ord` so callers can sort use lists. The
    location identifies the syntactic site, not which operand
    slot within it.
  - `compute_def_use(ssa) -> DefUseChains` — single forward
    walk over phi/instruction/terminator operands.
  - `def_of(ssa, value) -> &ValueDef` — thin wrapper over
    `SsaFunction::value` kept for symmetry with the def-use
    direction.
  - `SsaLiveness { live_in, live_out }` — per-block sets of
    live [`ValueId`]s.
  - `compute_liveness(ssa) -> SsaLiveness` — backward
    dataflow. The phi-incoming term is treated as a *per-edge*
    use on the predecessor's live-out side, not as a join-block
    live-in: otherwise a phi operand on edge (B1 → B3) would
    spuriously appear live on edge (B2 → B3) too, inflating
    live ranges. Matches Cooper & Torczon §9.2.
  - 10 unit tests: each-operand-occurrence counting, dead-
    value detection, phi-incoming recording per edge,
    cross-branch liveness, the phi-inflation guard, loop-
    carried back-edge liveness, determinism across runs,
    empty-function corner case, terminator-use recording, and
    a `parameter_of` test helper.
- `dac-recovery::stack`:
  - `StackConvention` closed enum: `SysVAmd64 | MsX64`. Names
    the stack-pointer register (`rsp` for both) and the
    frame-pointer register (`rbp` for both) per convention,
    and classifies positive offsets into the convention's
    layout zones.
  - `StackFrame { convention, stack_pointer, frame_pointer,
    locals, confidence }` — recovered frame, with
    `stack_pointer: Option<VariableId>` (None for synthetic
    leaf functions) and a [`BTreeMap`] of locals keyed by
    signed offset from `entry_sp`.
  - `FramePointer { variable, offset }` — populated when the
    pass recognized `mov fp, sp + k`. Negative on SysV
    (`rbp = rsp - 8` after a notional `push rbp`).
  - `StackLocal { offset, width, kind, access_count,
    confidence }` — one record per offset touched, with
    `width` accumulating as the maximum observed access width
    and `access_count` summing reads + writes.
  - `StackLocalKind` closed enum: `Local | ReturnAddress |
    ShadowSpace | IncomingArgument | Unclassified`.
    Convention-driven classification: SysV places ret addr at
    `+0`, stack args at any `8k > 0`; MsX64 has the same ret
    addr at `+0`, home space at `+8..+40`, and stack args at
    `>=+40`. Both treat negative offsets as callee locals.
    Unrecognized offsets land in `Unclassified` rather than
    being silently dropped — the reviewer should see what the
    pass touched.
  - `analyze_stack_frame(ssa, convention) -> StackFrame` — the
    entrypoint. Returns an empty-but-well-formed frame when
    the function never references `rsp` (legitimate for
    synthetic leaf no-ops; never raises).
  - All recovered facts carry [`Source::Derived`]
    confidence (I-3): `0.9` for the frame itself when the
    stack pointer was located, `0.85` per identified local.
  - 13 unit tests covering SysV-no-FP locals at `[rsp + k]`,
    SysV-with-FP `mov rbp, rsp` recognition + `[rbp - k]`
    locals, SysV incoming-arg classification at positive
    offsets, MsX64 home-space writes at `[rsp + 8]` /
    `[rsp + 16]`, MsX64 locals below a reserved frame, MsX64
    fifth-arg at `[rsp + 40]`, widest-access-width
    accumulation, phi-loop offset propagation, missing-sp
    degenerate case, determinism across runs, mid-return-
    address `Unclassified` classification, unaligned home-
    space `Unclassified`, and a `Source::Derived`
    confidence audit on every produced confidence.

Test counts: `cargo xtask ci` reports 44 green
`test result: ok` lines (unchanged from B2.3 — both new
modules are unit-tested inside their owning crates, no new
integration-test binaries). Lib test counts grew in
`dac-analysis` (52 → 62, +10 for dataflow) and in
`dac-recovery` (9 → 22, +13 for stack). No new warnings.

Closes: FR-11 (the def-use direction of use-def / def-use
chains and SSA liveness) and FR-12 (stack-variable
identification). Done-when satisfied: `dac-recovery::stack`
ships unit tests for both SysV x86-64 and Win64 stack
patterns — locals at negative offsets in both conventions,
arg classification in both, frame-pointer recognition on
SysV, and Win64 shadow-space recognition. No new evidence
nodes are minted; the dataflow passes derive from the same
SSA function (and thus the same CFG evidence handle as B2.3
introduced), keeping the I-2 provenance chain intact. The
stack pass's confidence (always `Source::Derived`) feeds
B2.5 (calling-convention inference) and B2.6 (type lattice +
propagation) directly.

#### B2.5 — Calling convention inference (2026-06-02)

Calling-convention table for x86-64 lands in `dac-knowledge`,
and a consultative inference pass lands in `dac-recovery`. The
pass scores every candidate ABI against four observed signals
— argument-register reads, caller-saved non-arg reads,
return-register definitions, and stack-frame layout — and
returns the candidates ranked by confidence. It is purely
consultative: the score is reported, never written back to the
IR.

The argument-register prefix is the dominant signal. SSA
construction (B2.3) mints a `Parameter` value for every
variable read without first being written; for an integer
argument register, that parameter *is* the incoming argument.
The pass measures the longest contiguous prefix of the
convention's argument-register sequence whose registers all
appear as parameter reads, and counts arg-register reads
beyond that prefix as soft contradictions (a function that
reads `rdx` and `rcx` but never `rdi` is unlikely to be using
SysV). Reading a caller-saved register that the convention
does *not* list as an argument register is a stronger
contradiction — the value at entry is undefined under that
ABI.

The stack layout (consumed from the B2.4 `StackFrame`) supplies
two finer signals. Positive offsets `>=
convention.first_stack_arg_offset` and 8-byte aligned line up
with stack-passed arguments and surface in
`InferredSignature::stack_args`. Offsets inside the convention's
`shadow_space_bytes` window are a positive signal for MsX64
(home-saving prologues spill `rcx`/`rdx`/`r8`/`r9` to
`[rsp+8..+40)`) and a negative signal for SysV (which reserves
no shadow space).

- `dac-knowledge::convention`:
  - `CallingConvention` — closed struct describing one ABI:
    `name`, `architecture`, `int_arg_registers`,
    `float_arg_registers`, `int_return_register`,
    `float_return_register`, `callee_saved`, `caller_saved`,
    `stack_pointer`, `frame_pointer`, `first_stack_arg_offset`,
    `stack_arg_alignment`, `shadow_space_bytes`.
  - `SYSV_AMD64` constant: int arg regs `rdi, rsi, rdx, rcx,
    r8, r9`; ret `rax`; callee-saved `rbx, rbp, r12..r15`;
    `first_stack_arg_offset = 8`; `shadow_space_bytes = 0`.
  - `MS_X64` constant: int arg regs `rcx, rdx, r8, r9`; ret
    `rax`; callee-saved `rbx, rbp, rdi, rsi, r12..r15`;
    `first_stack_arg_offset = 40` (past the 32-byte home
    space); `shadow_space_bytes = 32`.
  - `X86_64_CONVENTIONS` — the ranked candidate slice the
    inference pass scores by default; ties at the top of the
    ranking break toward SysV.
  - `x86_64_convention_by_name(name)` — case-insensitive
    lookup helper.
  - Predicate helpers (`is_int_arg_register`,
    `is_int_return_register`, `is_callee_saved`,
    `is_caller_saved`, `int_arg_index`) all match register
    names case-insensitively against the
    [`dac_ir::ssa::Variable::name`] vocabulary the lifter
    emits.
  - 6 unit tests: SysV table audit, MsX64 table audit,
    zero-based case-insensitive arg-index lookup, callee /
    caller predicate case-insensitivity, lookup by name
    returning the canonical entry, and the SysV-unique vs
    MsX64-unique register disjointness guard.
- `dac-recovery::convention`:
  - `ConventionMatch { convention_name, signature, confidence }`
    — one ranked candidate. `Eq` not derived because
    [`Confidence`] is f32-typed.
  - `InferredSignature { int_args, stack_args, return_register }`
    — per-convention reading of the function's signature.
    `int_args` is restricted to the contiguous arg-register
    prefix so a half-observed signature is not over-claimed.
  - `RegisterArg { register, index, value, variable }` and
    `StackArg { offset, width }` — element types.
  - `infer_calling_convention(ssa, stack_frame, candidates) ->
    Vec<ConventionMatch>` — entrypoint. Returns one match per
    candidate, sorted descending by `Confidence::value()`;
    ties break by the candidate's position in the input slice
    so the caller controls precedence (NFR-9 determinism).
  - `pick_best(ssa, stack_frame, candidates) ->
    Option<ConventionMatch>` — convenience wrapper around the
    head of the ranking.
  - All recovered facts carry [`Source::Derived`] (I-3).
    Scoring is `0.40` base + `0.30 × prefix_score` arg prefix
    bonus + `0.15` return-register match + `0.05` stack-args
    bonus + `0.10` shadow-space bonus − `0.10 × gap_count`
    arg-gap penalty − `0.15 × caller_saved_non_arg_reads` −
    `0.10 × shadow_misses`; the sum is clamped into `[0, 1]`.
  - 13 unit tests: SysV three-int-arg vs MsX64 outranking;
    MsX64 two-int-arg vs SysV outranking; shadow-space writes
    tipping the ranking even when arg lists overlap;
    caller-saved non-arg read penalty (SysV reading `rax` as
    a parameter); leaf-function tie broken by input order;
    discontiguous args truncating the signature to the
    contiguous prefix; SysV seventh arg at `[rsp + 8]` landing
    in `stack_args`; MsX64 fifth arg at `[rsp + 40]` landing
    in `stack_args`; determinism across runs; `pick_best`
    matching the head of the ranking; every match carrying
    `Source::Derived`; return of a constant not nominating a
    return register; locals at negative offsets not being
    misclassified as stack args.

Wiring: `dac-recovery` now depends on `dac-knowledge` (the
inference pass's only new dep). No public API of the existing
`stack` module changed.

Test counts: `cargo xtask ci` reports 44 green
`test result: ok` lines (unchanged from B2.4 — both new
modules are unit-tested inside their owning crates with no
new integration-test binaries). Lib test counts grew in
`dac-knowledge` (0 → 6, +6 for `convention`) and in
`dac-recovery` (22 → 35, +13 for `convention`). No new
warnings.

Closes: FR-13 (calling-convention inference). The "≥ 95% on
the sample corpus" criterion in PLAN.md cannot be measured
yet — the sample corpus itself lands as part of the
cross-cutting corpus-growth work alongside the type recovery
in B2.6 and the golden-test infrastructure in B2.9. The
inference algorithm and scoring are in place; corpus
calibration will be tracked as a follow-up once `tests/golden/`
exists.

#### B2.6 — Type lattice + propagation (2026-06-02)

A type lattice lands in `dac-ir::ty`, an API signature
catalogue lands in `dac-knowledge::api`, and a type-propagation
pass lands in `dac-recovery::types`. The propagation pass
consumes the SSA function from B2.3, the stack frame from
B2.4, the convention-inferred signature from B2.5, and a
caller-supplied resolver from call-target VAs to API
signatures; it seeds the lattice from four observation sources
and iterates `Type::join` through Move, arithmetic, and phi
nodes to a fixed point. The pass never mutates the IR — its
output is a side table of `(ValueId → Type, Confidence)` and
`(stack offset → Type, Confidence)` (I-1).

The lattice keeps `Unknown` as the bottom and `Top` as the
top: any cross-variant join (`Int` against `Ptr`, two `Int`s
with differing widths, two `Struct`s with differing shapes,
two `Array`s with differing lengths) lands at `Top` rather
than silently picking a winner (I-6). The signedness
sub-lattice within `Int` has its own `Unknown` bottom and a
`Conflict` element distinct from `Unknown` so the propagation
pass can surface contradictions in `--debug` later without
losing the width fact.

Seeds, in decreasing strength:

1. **API signature call sites.** When a `Call { target:
   Some(va) }` resolves via the caller-supplied
   `ApiResolver`, the destination joins the signature's
   `return_ty` and each `args[i]` operand joins the
   signature's `parameters[i].ty`. Confidence `0.90`.
2. **Load / Store widths.** A `Load { width: w }` constrains
   its destination to `Int(w*8, Unknown)`; a `Store { value,
   width: w }` constrains `value` likewise. Both ops also
   mark their address operand as `Ptr(Unknown)`. Confidence
   `0.80`.
3. **Stack-local widths.** Each `StackLocal` from the B2.4
   stack frame contributes its widest observed access width
   as a `locals` entry of `Int(width*8, Unknown)`. Confidence
   `0.75`.
4. **Inferred-signature parameter values.** Each
   `RegisterArg.value` from B2.5 enters as `Int(64, Unknown)`
   (the function's pointer width on x86-64). Confidence
   `0.70`.

Propagation steps are weaker than their seeds: `Move` is a
pure passthrough capped at `0.85`; arithmetic ops join their
operand types and cap the resulting confidence at `0.60`;
`Compare` always publishes `Int(1, Unknown)` (the boolean
result of a comparison) at `0.60`; `Opaque` is opaque (I-6) —
its destination stays unconstrained. Phi destinations join
the types of every incoming operand and surface only when the
joined type is more specific than `Unknown`.

- `dac-ir::ty`:
  - `Type` enum: `Unknown` (bottom), `Int(IntType)`,
    `Ptr(Box<Type>)`, `Struct(StructType)`,
    `Array(ArrayType)`, `Top`. `Eq` + `Hash` are exact
    equality — same shape, same type.
  - `IntType { width_bits: u16, signedness: Signedness }` and
    `Signedness { Unknown, Signed, Unsigned, Conflict }`.
    `Signedness::join` follows the sub-lattice exactly.
  - `StructType { name, fields: Vec<StructField> }`,
    `StructField { offset, ty, name }`, `ArrayType { element,
    length: Option<u64> }`. Fields are kept in ascending
    offset for byte-stable hashing.
  - `Type::join(other)` — lattice join, total, deterministic.
    Idempotent, commutative, associative; `Unknown` is the
    identity; `Top` is absorbing.
  - Ergonomic constructors: `Type::signed_int(width)`,
    `unsigned_int(width)`, `int_of_width(width)`,
    `ptr_to(t)`.
  - Predicates: `is_unknown()`, `is_top()`,
    `int_width_bits()`.
  - 13 unit tests covering each variant's join behavior,
    signedness sub-lattice laws, and the four lattice
    endpoints.
- `dac-knowledge::api`:
  - `ApiSignature { name, library, return_ty, parameters,
    is_variadic }` — one catalogue entry.
  - `ApiLibrary { Libc, Win32 }` — origin tag with
    case-insensitive ordering for stable diagnostics.
  - `ApiParameter { name, ty }` — positional formals.
  - `lookup_api_signature(name)`,
    `lookup_api_signature_in(name, library)`,
    `api_signatures()` — deterministic lookups walking the
    table in declaration order.
  - Libc minimal set (24 entries): `strlen`, `strcmp`,
    `strcpy`, `strncpy`, `memcpy`, `memset`, `memcmp`,
    `malloc`, `calloc`, `realloc`, `free`, `printf`, `puts`,
    `fopen`, `fclose`, `fread`, `fwrite`, `read`, `write`,
    `open`, `close`, `exit`, `abort`, `getenv`.
  - Win32 minimal set (9 entries): `CreateFileA`,
    `CloseHandle`, `ReadFile`, `WriteFile`, `GetLastError`,
    `HeapAlloc`, `HeapFree`, `GetProcessHeap`, `ExitProcess`.
  - Behind a `LazyLock<Vec<ApiSignature>>` because
    `Type::Ptr` boxes its pointee and is not `const`-
    constructible. Parameter slices are leaked once at first
    access (`Box::leak` on a stable, declaration-order list)
    so the public surface is `&'static [ApiParameter]`.
  - 10 unit tests: libc and Win32 lookups, variadic flag for
    `printf`, library-scoped lookup, miss returns `None`,
    no duplicate `(name, library)` pairs, minimal-set
    completeness for libc and Win32, signature stability
    across lookups (pointer-equal), library-name strings.
- `dac-recovery::types`:
  - `TypeMap { values: BTreeMap<ValueId, ValueType>, locals:
    BTreeMap<i64, LocalType> }` — side table of recovered
    types. Both maps are absent-as-`Unknown`.
  - `ValueType { ty, confidence }` and `LocalType { ty,
    confidence }`. `Eq` not derived (`Confidence` is f32-
    typed).
  - `propagate_types(ssa, inferred_signature: Option<&_>,
    stack_frame: Option<&_>, api_resolver: &dyn
    ApiResolver) -> TypeMap` — entrypoint. Both optional
    inputs let the pass run before its B2.x dependencies
    have lined up; degradation is graceful (fewer seeds, no
    refusal to produce).
  - `ApiResolver` trait + blanket impl for any `Fn(u64) ->
    Option<&'static ApiSignature>`, plus a `NullApiResolver`
    that always returns `None` for tests that exercise only
    load / store / stack signals.
  - `TypeMap::value_recovery_ratio()` and
    `local_recovery_ratio()` — recovery-coverage helpers used
    by the corpus rubric in PLAN.md ("≥ 70% of locals").
  - All recovered facts carry [`Source::Derived`] (I-3).
  - 15 unit tests covering load/store seeding, the
    Ptr(Unknown) address-operand mark, API call-site seeding
    of args and return, API arity-mismatch trimming, Move
    passthrough, arithmetic width propagation, phi merging
    across incoming SSA values, convention-inferred
    parameter seeding through `infer_calling_convention`,
    stack-local width pickup through `analyze_stack_frame`,
    `Source::Derived` on every recovered fact, API seeds
    outranking arithmetic propagation by confidence,
    determinism across repeated runs, the recovery-ratio
    helper, and `Opaque` contributing no constraint.

Wiring: `dac-knowledge` now depends on `dac-ir` (for `Type`);
`dac-ir` re-exports `Type`, `IntType`, `Signedness`,
`StructField`, `StructType`, `ArrayType` at the crate root.
`dac-recovery::types` consumes both `dac-knowledge` and
`dac-recovery::{convention, stack}` — no new external deps.

Test counts: `cargo xtask ci` reports 46 green
`test result: ok` lines (44 → 46, +2 for the new `dac-ir::ty`
and `dac-knowledge::api` doc-test slots; doctest lines stay
at 0 each since neither module ships executable doctests).
Lib test counts grew: `dac-ir` 9 → 22 (+13 for `ty`),
`dac-knowledge` 6 → 16 (+10 for `api`), `dac-recovery` 35 →
50 (+15 for `types`). No new warnings.

Closes: FR-14 (parameter and return-type inference where
evidence exists) and FR-16 (type propagation from API
signatures, instruction patterns, and memory usage). The
"≥ 70% of locals in the corpus" criterion in PLAN.md cannot
be measured yet — `tests/golden/` and the sample corpus arrive
in B2.9. The propagation algorithm, lattice, and signature
table are in place; corpus calibration follows.

#### B2.7 — Semantic IR + structuring (2026-06-02)

A Semantic IR module lands in `dac-ir::sem` and a control-
flow structuring pass lands in `dac-analysis::structuring`.
The structurer consumes the SSA function from B2.3 together
with its CFG (B2.1), dominator and post-dominator trees, and
loop forest (B2.2) and folds them into a tree of structured
statements — `if` / `else`, endless `loop` with explicit
`break` / `continue`, early `return`, and a `goto` /
`Label` fallback for irreducible CFGs. The Semantic IR also
defines `while`, `do { … } while`, and `switch` variants
for use by later batches (B3.3 idiom recognition / B2.8
lowering); the B2.7 structurer itself emits the canonical
`Loop` + `If` + `Break` + `Continue` shape, leaving the
`while` / `do-while` / `for` rewriting to a downstream
recognition pass that has access to inferred types.

The algorithm is a top-down recursive walk seeded at the SSA
entry. Each recursion takes a `region_exit` (the
post-dominator-derived merge point that caps the current
sub-tree) and a stack of enclosing `LoopCtx { header, exit
}`. Bases: `current == None` or `current == region_exit`
returns an empty block; `current == loop_stack.last().header`
becomes a `Continue`; `current == loop_stack.last().exit`
becomes a `Break`; an already-emitted block becomes a `Goto`
with a freshly allocated `LabelId`. Otherwise the block is
marked emitted, its phis and instructions are lifted into
statement-position `SsaRef` carriers, and the terminator is
dispatched: `Jump` recurses into the target; `Branch` builds
an `If` whose then/else arms are recursive structurings
toward the IPDOM-filtered join, then continues from the join
when one exists; `Return` becomes a `Return` statement;
`Indirect` / `Unreachable` produce a `Stmt::Unreachable`
marker (I-6, degrade — don't invent).

Loop entry pre-computes the loop's uniform exit (the
not-in-body side of the header's conditional terminator, or
the single CFG-level block any body member exits to) and
pushes a `LoopCtx`, then processes the header normally inside
the loop scope. Back-edges into the header automatically
become `Continue`; edges to the exit automatically become
`Break`; nested loops recurse with their own pushed
`LoopCtx`.

A label-anchoring post-pass walks the produced body once and
inserts `Stmt::Label` at the first emission of every block
demoted into a goto target. `Stmt::Goto` carries the
`source_block` of the demoted CFG block so the anchor can be
placed even when the block produced no other statements (the
irreducible-ping-pong case). Any label that survives the
walk without being anchored — possible when both a goto's
source and the structurer's recursion through the source's
target produced no anchor-eligible statements — is appended
at the tail of the body so every `Stmt::Goto::target`
resolves to a `Stmt::Label::id` somewhere in the tree
(degrade, never silently drop).

- `dac-ir::sem`:
  - `SemFunction { function_address, function_name, body,
    evidence, stats }` — the structured tree.
  - `Block { stmts: Vec<Stmt> }` with `Block::empty()` /
    `is_empty()` ergonomics.
  - `SsaRef { block: SsaBlockId, index: u32 }` — stable
    handle back into the source SSA function. Per I-2, the
    Sem layer references SSA instructions rather than
    cloning them, so the lowering pass (B2.8) has one
    canonical place to resolve types / evidence / dst.
  - `Stmt` enum (13 variants): `Phi`, `Instr`, `If`,
    `While`, `DoWhile`, `Loop`, `Switch`, `Break`,
    `Continue`, `Return`, `Label`, `Goto`, `Unreachable`.
    Closed and exhaustively pattern-matched downstream so
    new constructs surface as compile-time misses (the I-6
    lever that keeps backends from inventing semantics).
  - `SwitchArm { value: i64, body: Block }` — placeholder
    for the B3.3 jump-table recognizer.
  - `StructuringStats { source_blocks, goto_count,
    label_count, irreducible }` plus
    `StructuringStats::is_goto_free()` — the per-function
    rubric.
  - `LabelId` is `u32`. Labels are dense indices allocated
    in source-order so a renderer can produce stable `L0`,
    `L1`, … names without bookkeeping.
  - 6 unit tests covering the empty-block identity, the
    goto-free predicate, the exhaustive-match guard,
    `SsaRef` copy / equality, label-id density, and the
    `SemFunction` carrier shape.
- `dac-analysis::structuring`:
  - `structure(ssa, cfg, doms, pdoms, loops) -> SemFunction`
    — entrypoint, re-exported as `dac_analysis::structure`.
  - Recursive structuring with `region_exit` and a loop
    context stack; emits `Stmt::If` at conditional
    branches, `Stmt::Loop` at loop headers, `Stmt::Break` /
    `Stmt::Continue` at exit / back-edge transitions, and
    falls back to `Stmt::Goto` only when the recursion
    re-enters an already-emitted block.
  - `compute_loop_exit` picks the canonical exit when the
    loop header's conditional terminator splits one way
    into the body and the other way outside; falls back to
    "any single block successor outside the body", then to
    the header's IPDOM when neither shape applies.
  - `find_join` consults the post-dominator tree, filtered
    so the join is suppressed when it sits at the
    surrounding `region_exit` or outside the enclosing
    loop body (the arms reach the loop exit via `Break`
    rather than a structural merge).
  - `insert_labels` post-pass walks the body and prepends
    `Stmt::Label` at the first emission of every labelled
    block; orphan labels are appended at the body tail so
    every `Stmt::Goto::target` resolves.
  - 15 unit tests covering: single return, linear chain,
    diamond merge with empty / populated arms, early-return
    diamond, canonical while-loop (Loop + If + Break +
    Continue), self-loop endless `Loop`, nested while
    loops, irreducible CFG goto fallback (every Goto
    resolves to a Label), byte-determinism, phi-before-
    instr order, indirect terminator → Unreachable, empty
    function, and conditional cond preservation.

Wiring: `dac-ir` re-exports `Block`, `LabelId`,
`SemFunction`, `SsaRef`, `Stmt`, `StructuringStats`,
`SwitchArm` at the crate root. `dac-analysis::structuring`
consumes `dac_ir::sem` and `dac_ir::ssa`, plus the existing
`crate::{cfg, dom, loops}` modules — no new external deps.

Test counts: `cargo xtask ci` reports 44 green `test result:
ok` lines (clippy + tests + doc-tests across the workspace),
zero warnings, zero errors. New lib tests: `dac-ir` 22 → 28
(+6 for `sem`), `dac-analysis` 62 → 77 (+15 for
`structuring`). Cumulative test count climbs by 21.

Closes: FR-18 (control-flow structuring producing `if`,
`while`, `for`, `switch`, early returns plus a goto fallback
— the variants exist in the Semantic IR vocabulary; the
structurer emits `If` + `Loop` + `Break` + `Continue` +
`Return` reliably; `While` / `DoWhile` / `For` / `Switch`
recognition is layered on top in later batches with access
to inferred types). The "goto-free on the sample corpus for
at least the simple functions" criterion in PLAN.md cannot
be measured yet — `tests/golden/` and the sample corpus
arrive in B2.9. `StructuringStats::is_goto_free()` and
`StructuringStats::goto_count` are wired up so the rubric
can be evaluated immediately when the corpus lands.

#### B2.8 — C backend (`-O1`) (2026-06-02)

The C backend lands in `dac-backend-c` and `--target c` is
wired end-to-end through `dac-cli` at `-O1` (and above).
The backend is a four-module pipeline — AST, lowering,
pretty-printer, round-trip compile check — that consumes a
[`SemFunction`](dac_ir::sem::SemFunction) together with its
underlying [`SsaFunction`](dac_ir::ssa::SsaFunction) and
produces formatted C source. A best-effort `cc` round-trip
helper sits next to the pretty-printer so unit tests can
gate the corpus on compilability without forcing every
developer to install a toolchain.

- `dac-backend-c::ast`:
  - Closed C AST covering everything the B2.7 structurer
    can produce: `TranslationUnit { includes, items }`,
    `Item::Function { name, return_type, params, locals,
    body, leading_comment }`, `Block { stmts }`, and the
    13-variant `Stmt` enum (`Decl` / `Assign` / `Store` /
    `ExprStmt` / `If` / `Loop` / `While` / `DoWhile` /
    `Break` / `Continue` / `Return` / `Label` / `Goto` /
    `Comment` / `Unreachable`).
  - 9-variant `Expr` (`Var`, `IntLit`, `Undef`, `Binary`,
    `Unary`, `Load`, `Call`, `AddrLit`, `Opaque`).
  - `BinaryOp` (14 variants: arithmetic + bitwise + compare),
    `UnaryOp` (3 variants: `Neg` / `BitNot` / `LogicalNot`).
  - `CType` (`Void`, `Int { width_bits, signed }`, `Ptr`).
    `CType::i64()` / `CType::u8()` shortcuts.
  - 4 unit tests on the AST shape and exhaustivity guards.
- `dac-backend-c::lower`:
  - `lower_function(ssa, sem, resolver) -> Function` walks
    `sem.body`, resolves each `SsaRef` against
    `ssa.blocks[…].phis` / `instructions`, and emits the
    matching C statement. Side-effect ops (stores, calls
    without `dst`, opaque) lower to `Stmt::Store` /
    `Stmt::ExprStmt`; value-producing ops become
    `Stmt::Assign`. Phi statements lower to a
    `/* phi v<dst> <- (bb<p>: <op>) … */` comment carrier;
    every SSA value is pre-declared at the top of the
    function body as `int<width>_t v<id> = 0LL`, sidestepping
    SSA destruction at the cost of loop-iteration fidelity.
  - `lower_unit(ssa_funcs, sem_funcs, resolver)` wraps a
    slice of lowered functions in `default_includes()` —
    `#include <stdint.h>` + `#include <stddef.h>`.
  - `NameResolver = BTreeMap<u64, String>` threads recovered
    call-target names into `Expr::Call` — direct calls
    render `target(args)`; unknown addresses fall back to
    `((void (*)())0xNNNN)(args)`.
  - Return-type inference: scans the Sem body for any
    `Return { value: Some(_) }` and picks `int64_t` if
    found, `void` otherwise. The B2.6 type-lattice + B2.5
    convention threading lands in a later batch when the
    orchestrator plumbs `TypeMap` and `InferredSignature`
    into the call site.
  - Each lowered function carries a leading comment with
    the source address and the structurer's
    `StructuringStats` so any emitted function is traceable
    back to the binary (I-2).
  - 5 unit tests covering empty / arithmetic / store /
    resolver-injected call / phi lowering plus a
    byte-determinism guard.
- `dac-backend-c::emit`:
  - Hand-rolled pretty-printer: `emit(unit) -> String` and
    `emit_function(f) -> String`. 4-space indent, K&R
    braces, one statement per line. Binary expressions
    parenthesise both children (verbose but precedence-
    correct without the lowering pass having to reason
    about it). Integer literals carry `LL` / `ULL`
    suffixes. Labels render as `L<id>:;` so the trailing
    semicolon makes the result a valid empty-statement
    target.
  - `int<n>_t` widths normalise to the nearest standard
    width (8 / 16 / 32 / 64); anything beyond 64 falls
    back to `int64_t`.
  - 11 unit tests covering blank-unit / include / function
    signature / if-else / endless loop / label / binary
    precedence / int-type normalisation / load cast /
    opaque sanitisation / leading-comment / locals-before-
    body.
- `dac-backend-c::compile`:
  - `try_compile(source) -> CompileResult` shells out to
    the system C compiler (`$CC`, then `cc`, then `gcc`,
    then `clang`) with `-x c -c - -o /dev/null -w`.
    Returns `CompileResult::Ok` / `Failed` / `Skipped`;
    `Skipped` fires when no compiler is on PATH, so unit
    tests stay green on toolchain-less hosts.
  - 4 unit tests covering predicate / candidate-list /
    trivial / malformed-source cases.
- `dac-backend-c/tests/round_trip.rs`:
  - 12 round-trip tests building six SemFunction fixtures
    — empty function, arithmetic chain, if-then-else,
    endless loop with break, goto fallback, and store-then-
    load — and feeding the emitted C through `try_compile`.
    Each fixture covers one structurer output shape; the
    multi-function translation unit aggregates them.
    Determinism, the `#include <stdint.h>` / `<stddef.h>`
    presence, the empty-function shape, and the nested
    pointer cast all carry pinned assertions.
- `dac-cli` wiring:
  - New `--target c` / `-O1`+ code path renders a C
    translation unit through `dac-backend-c` and writes it
    to `<output>.c` alongside the listing (or appends it
    to stdout under a `;; ---- target source (FR-21) ----`
    divider when no `--output` is set).
  - Because the lifter → `RawFunction` bridge that would
    feed the structurer from real x86-64 bytes is not yet
    a batch in PLAN.md, the per-function body is a stub
    (`/* lifter→SSA bridge pending; body intentionally
    empty */`). The translation unit still compiles cleanly
    — the round-trip test gates it.
  - The leading banner records the input path and arch so
    the file is self-identifying.
  - `Target::Cpp` lands on a placeholder until the C++
    backend (B3.5).
  - 3 new end-to-end tests confirm the banner is present,
    the sidecar compiles through `cc`, and the output is
    byte-identical across two runs.

Wiring: `dac-backend-c` gains `dac-core` + `dac-ir` as
dependencies and `dac-analysis` + `dac-recovery` as dev
dependencies (the round-trip tests construct SemFunctions
by hand and don't need the recovery pipeline, but the
crates are wired for the next batch). `dac-cli` adds
`dac-backend-c` as a runtime dependency. The CLI's
`emit_outputs` gains a `source: Option<&str>` parameter.

Test counts: `cargo xtask ci` reports 46 green `test
result: ok` lines, zero warnings, zero errors. New tests:
`dac-backend-c` 0 → 38 (26 lib + 12 round-trip); `dac-cli`
24 → 27 (+3 for `o1_target_c`). Cumulative test count
climbs by 41.

Closes: FR-21 (target-language backend emits source from
the Semantic IR). The "5 sample binaries decompile to
compilable C and run with matching behavior on a smoke
test" rubric in PLAN.md is partially satisfied: the C
backend produces compilable C from 6 hand-built fixtures
each demonstrating a distinct structurer output shape
(empty / arith / if-else / loop-with-break / goto /
store-load), and `--target c -O1` produces a compilable
translation unit from the real ELF fixture. The
"run with matching behavior" leg cannot be measured until
the lifter → `RawFunction` bridge lands and feeds the
sample corpus (B2.9); it is recorded here as a deferred
follow-up rather than silently dropped.

#### B2.9 — Golden test infrastructure (2026-06-02)

Long-term drift gate for every deterministic `dac` output.
`tests/golden/` becomes the recorded corpus; a new
`cargo xtask golden {check, update}` runs each declared
case through the `dac` CLI under workspace-relative paths,
captures the produced sidecars, and either diffs (`check`)
or overwrites (`update`) the bytes stored on disk. The
canonical `cargo xtask ci` calls `golden check` after the
test suite, so any drift fails the same command developers
already run locally.

- `xtask/src/golden.rs`:
  - `Mode::{Check, Update}` switches the harness between
    diff-and-fail and overwrite-on-disk behavior.
  - `Case { name, fixture, args, outputs }` declares one
    corpus row; the static `CASES` array is the corpus.
    The array shape catches typos at compile time and
    keeps `xtask` dependency-free (no TOML crate today).
  - `OutputKind::{Listing, Manifest, Report, Cfg, Source}`
    maps each captured sidecar to its on-disk file name
    (`listing.txt`, `manifest.json`, `report.txt`,
    `cfg.dot`, `source.c`) and to the suffix the `dac`
    CLI writes (`""`, `.manifest.json`, `.report.txt`,
    `.cfg.dot`, `.c`) — mirroring the contract documented
    in `dac-cli::main::emit_outputs`.
  - `run(mode)` builds `target/debug/dac` once, clears a
    scratch dir under `target/xtask/golden/`, and for each
    case invokes the CLI with `current_dir = workspace
    root` plus the workspace-relative fixture path
    (`tests/fixtures/<file>`). Workspace-relative paths
    keep the manifest's `input.path` portable across
    developer machines.
  - `render_drift(...)` picks the first differing line
    (UTF-8 inputs) or first differing byte (otherwise) and
    formats a triage report: expected vs. actual paths,
    byte counts, the offending line with `-`/`+` markers,
    and a hint to re-run `cargo xtask golden update`.
  - 6 unit tests pin the invariants the harness depends on:
    case-name uniqueness, per-case output uniqueness, the
    `OutputKind → sidecar suffix` mapping, fixture existence,
    and the two drift-reporting paths.

- `xtask/src/main.rs`:
  - `cargo xtask golden check` (default sub) and
    `cargo xtask golden update` dispatched from the existing
    arg parser; unknown `golden` sub exits 2 with a hint.
  - `ci()` runs `fmt → clippy → test → golden::Check`, so
    the canonical CI command gates drift.
  - Usage banner updated to list the new subcommands.

- `tests/golden/`:
  - 9 cases, 22 captured outputs:
    - `hello-elf-o0` (listing, manifest),
    - `hello-elf-o0-report` (listing, manifest, report),
    - `hello-elf-o0-cfg` (listing, manifest, cfg),
    - `hello-elf-o1-c` (listing, manifest, source),
    - `hello-elf-stripped-o0` (listing, manifest),
    - `hello-pe-o0` (listing, manifest),
    - `hello-pe-o1-c` (listing, manifest, source),
    - `libsample-o0` (listing, manifest),
    - `sample-dll-o0` (listing, manifest).
  - `README.md` documents the layout, the update flow, the
    workspace-relative path contract, and how to add a
    case.

- The integration tests `crates/dac-cli/tests/o0_golden.rs`,
  `crates/dac-cli/tests/cfg_emit.rs`, and
  `crates/dac-cli/tests/o1_target_c.rs` continue to assert
  within-run determinism (run-twice, byte-identical). The
  goldens layer the across-run / across-PR check on top.

`cargo xtask ci`: green. Test groups climb by 1 (xtask
now ships 6 unit tests) and the golden harness re-runs the
nine-case corpus at the end of every CI invocation.

Closes: NFR-9 (same input + settings → same output, gated
in CI) at the CLI surface for the artifacts shipping today
(listing, manifest, report, CFG DOT, target source).
Satisfies spec §16 "golden-file tests for emitted source"
for the C backend. Closes the deferred B2.8 follow-up at
the corpus level: every `--target c -O1` reconstruction in
the corpus is now byte-pinned across runs, even though the
lifter → `RawFunction` bridge (the second leg of the B2.8
"run with matching behavior" rubric) remains future work.

### Milestone 3 — Usable RE tool

#### B3.1 — Call graph + xrefs (2026-06-02)

Whole-program call graph and an address-indexed cross-reference table
land in `dac-analysis`, with two new CLI surfaces (`--xrefs <subject>`,
`--callgraph`) wiring them through the existing sidecar machinery in
`dac-cli`. The B3.1 "done when" rubric — `dac sample.elf --xrefs sym`
prints sane results — is gated by an integration test that asserts the
expected `CALL` edges and caller annotations against the corpus ELF
fixture.

- `dac-analysis::xrefs` (new ~600-line module):
  - `XrefKind` — `Call`, `TailCall`, `IndirectCall`, `CodeToData`,
    `DataToCode`, `DataToData`, `Import`, `Export`. Each kind has a
    stable two-to-five-letter tag (`tag()`) for the textual renderer
    and an explicit doc-comment recording when it is minted.
  - `Xref { from, to, kind, confidence }` and `XrefIndex` with
    `from(addr)` / `to(addr)` lookups backed by `BTreeMap<u64,
    Vec<u32>>` parallel indices. The underlying `Vec<Xref>` is sorted
    `(to, from, kind)` so the renderer walks every report row in a
    byte-stable order (NFR-9, I-4).
  - `CallNode { id, kind, address, name }` with
    `CallNodeKind ∈ Function | Import | Unresolved | IndirectSite`,
    `CallEdge { from, to, site, indirect, confidence }`, and
    `CallGraph { nodes, edges, by_function }`. Function nodes land in
    ascending address order; imports / unresolved / indirect-site
    nodes follow as edges are discovered. Edges are sorted
    `(from, site, to, indirect)` so the DOT output is stable.
  - `build_call_graph(model, bytes, decoder, functions)` walks every
    function through `decoder.iter`, surfacing
    `ControlFlow::Call { target: Some }` (direct call → Function /
    Import / Unresolved), `ControlFlow::IndirectCall` (anchored at
    `indirect@<va>`), and `ControlFlow::UnconditionalBranch
    { target: Some }` (tail call only when the target leaves the
    source function *and* lands at another recovered function entry).
    Confidence is `Observed` for direct call→function / Import,
    `Derived` for unresolved-direct / tail-call, `Speculative` for
    indirect calls.
  - `build_xref_index(model, bytes, decoder, functions)` mirrors the
    callgraph walk for the code↔code half, then minters
    relocation-derived xrefs (`Code→Data`, `Data→Code`, `Data→Data`,
    `Import` for undefined-symbol relocations against code), and
    finally `Export` xrefs from `model.entry` and `model.exports`,
    rooted at the synthetic external VA `0`.
  - `resolve_subject(raw, model, functions)` accepts a hex VA
    (`0x...` / bare hex / decimal) or a symbol name, preferring an
    exact function-entry match before falling back to symbols and
    exports. Used by the CLI.
  - `render_callgraph_dot(graph, binary_name)` emits a single
    `digraph` with node shapes per kind (box / diamond / ellipse /
    circle) and dashed indirect edges; the call-site VA is the edge
    label.
- `dac-cli::xrefs` (new module): `render_xrefs_report(subject_raw,
  subject_va, subject_name, index, model, functions)` formats the
  textual `;;`-prefixed report — preamble + `;; xrefs to: N` block +
  `;; xrefs from: N` block — with two-line annotations
  (`->  fn <name>` / `-> sym <name>`) that fall back to the
  *containing* function so call sites inside a function body still
  surface the caller's name.
- `dac-cli::main`:
  - `Args` gains `emit_callgraph` (`--callgraph`) and
    `xrefs_subject` (`--xrefs <subject>`). The parser, the tracing
    debug line, and `tests/snapshots/help.txt` are updated in lock
    step.
  - `emit_outputs` grows two new sidecars: `<output>.callgraph.dot`
    and `<output>.xrefs.txt`, with matching delimited stdout blocks
    when `--output` is absent.
  - The unsupported-arch branch returns a valid-but-empty
    `digraph "callgraph_unsupported_arch_<arch>"` and an "unresolved
    subject" placeholder so downstream tooling never receives
    invalid DOT or empty text.
- Tests:
  - `dac_analysis::xrefs::tests` — 9 unit tests covering the direct
    call edge, indirect-site anchoring, unresolved-target safety
    net, tail-call promotion (positive + negative), xref-index
    `to/from` ordering, exports + entry → `Export` xrefs, data↔data
    relocations, subject resolution by name + hex, and DOT
    determinism.
  - `dac-cli::xrefs::tests` — 4 unit tests covering caller-symbol
    annotation, zero-xref subjects, the `<external>` marker, and the
    data-kind constant.
  - `crates/dac-cli/tests/xrefs_cli.rs` (new) — 5 integration tests
    against the corpus ELF: `--xrefs deregister_tm_clones` lists the
    `CALL` edge from `0x1128` with the
    `__do_global_dtors_aux` annotation; `--xrefs 0x1090` matches
    `--xrefs deregister_tm_clones` on every expected substring;
    `--xrefs _start` records the `EXP` xref from `<external>` (the
    binary's entry point); unknown subjects emit the
    `(unresolved: …)` block; `--callgraph --output <path>` lands a
    DOT file with the expected header, at least one
    `shape=box` function node, and a `style=solid` edge.

Closes FR-26 (cross-references), FR-27 (whole-program call graph),
FR-31 (CLI query interface for symbols / strings / refs). All goldens
unchanged — the new sidecars are opt-in and the listing / manifest /
report / CFG / source corpus rows remain byte-identical. The current
implementation is limited to the signals the decoder's `ControlFlow`
enum and the relocation table expose: per-instruction operand-level
code↔data xrefs (lea / mov of an absolute address) land alongside
B3.2's struct recovery, when the lifter's operand-walk becomes the
shared substrate.

#### B3.2 — Struct and array recovery (2026-06-02)

Struct and array recovery for `dac-recovery`. Lands as a new
`dac_recovery::structs` module that consumes the existing SSA, the
B2.4 [`StackFrame`], and the B2.6 [`TypeMap`] and emits a
[`RecoveredStructs`] side table — purely additive, no IR mutation
(I-1). The B3.2 "done when" rubric — *recovers known structs on a
hand-built test binary* — is gated by a unit test that builds a
two-field `{int64 a; int32 b;}` stack struct out of synthetic SSA
ops and asserts the recovered layout's field offsets and widths.

- `dac-recovery::structs` (new ~600-line module):
  - `RecoveredStructs { stack_structs, pointer_structs, arrays }` —
    three `BTreeMap`s for deterministic ordering. Stack-anchored
    layouts are keyed by their lowest (most negative) stack offset;
    pointer-anchored layouts are keyed by the base SSA value;
    arrays are keyed by the base SSA value addressing element 0.
  - `StructLayout { fields, total_size, confidence }` with fields
    sorted ascending by offset. Field offsets are *normalized* to
    start at zero, so the same struct shape compares byte-equal
    regardless of whether it lives on the stack or behind a pointer.
  - `FieldCandidate { offset, width, ty, access_count, confidence }`
    — the per-offset access record. `ty` is `Type::Unknown` when no
    `TypeMap` is supplied or the type pass failed to constrain the
    field's value.
  - `ArrayLayout { element_size, element_width, confidence }` —
    `element_size` is the stride from `Mul(idx, c)` /
    `Shl(idx, log_c)`; `element_width` is the access width observed
    at a load/store through the indexed value, when one fires.
  - `recover_structs(ssa, frame, types) -> RecoveredStructs` — the
    single public entry point. `frame` and `types` are independently
    optional; passing `None` degrades the corresponding recovery
    rather than refusing to produce output.
  - Confidence constants (all `Source::Derived`):
    `STACK_CLUSTER_CONFIDENCE = 0.75`,
    `POINTER_BASE_CONFIDENCE = 0.65`,
    `ARRAY_INDEXED_CONFIDENCE = 0.70`. Each value reflects how
    directly the pattern is observable in SSA.
  - Heuristics:
    - **Stack cluster.** Greedy contiguity walk over
      [`StackLocalKind::Local`] entries — a cluster extends as long
      as the next offset sits within `max(prev.width, 8)` of the
      previous one. Singleton clusters do not promote. The walk
      excludes return-address, shadow, and incoming-arg slots
      (those are not struct candidates).
    - **Pointer base.** For every `Load`/`Store`, decompose the
      address operand via `Add(base, Const)` / `Sub(base, Const)`
      / `Add(base, Move-of-const-value)`. Bases with two or more
      distinct offsets promote. Bases with a single observed
      offset are not surfaced (one read is not enough to claim a
      struct shape).
    - **Indexed array.** Match `Add(base, scaled)` where
      `scaled = Mul(idx, c)` or `Shl(idx, log_c)`. Stride 1 is
      rejected (indistinguishable from plain pointer arithmetic
      at this layer); strides ≥ 2 register.
- `dac-recovery::lib`: `pub mod structs;` plus re-exports of
  `recover_structs`, `ArrayLayout`, `FieldCandidate`,
  `RecoveredStructs`, `StructLayout`, and the three confidence
  constants.
- Tests (12 in `dac_recovery::structs::tests`):
  - `adjacent_stack_locals_form_struct_layout` — two adjacent
    stack stores cluster into a 16-byte struct.
  - `lone_stack_local_is_not_a_struct` — single stack local does
    not promote.
  - `stack_fields_inherit_recovered_types` — fields pick up
    `int_of_width` types from the `TypeMap`.
  - `two_loads_through_pointer_base_form_struct` — two distinct
    offsets through `rdi` register the pointer struct.
  - `single_offset_pointer_access_is_not_a_struct` — one load is
    not enough.
  - `indexed_load_with_mul_stride_recovers_array` — `Mul(idx, 4)`
    plus a 4-byte load registers `element_size = 4`,
    `element_width = Some(4)`.
  - `indexed_load_with_shl_stride_recovers_array` — `Shl(idx, 3)`
    plus an 8-byte load registers `element_size = 8`.
  - `stride_of_one_is_not_an_array` — `base + idx` (stride 1)
    deliberately rejected.
  - `recovery_is_deterministic_across_runs` and
    `empty_inputs_produce_empty_output` pin determinism + degraded
    inputs.
  - `every_recovered_confidence_is_derived` checks that no
    `Observed` / `Speculative` confidence leaks out of the pass
    (I-3).
  - `hand_built_struct_round_trip` — the PLAN rubric: a synthetic
    `struct { int64 a; int32 b; }` on the stack is exercised
    through both a store and a load and the recovered layout
    matches.

Closes FR-17 (struct / array recovery from offset clustering and
indexed access patterns). The pass is `Source::Derived` (no AI
input), deterministic (NFR-9, I-4), and additive (the IR remains
the source of truth, I-1). Union recovery, nested-struct chasing,
and feeding the recovered layouts back into the C backend at
emit time are deliberately deferred. The B3.3 idiom-recognition
pass and the B3.4 annotation channel will consume
`RecoveredStructs` directly when they land.

#### B3.3 — Idiom recognition (2026-06-02)

Idiom recognition for `dac-recovery`. Lands as a new
`dac_recovery::idioms` module that scans the SSA function for
compiler-emitted jump tables on x86-64 and surfaces them as
proposal-style annotations on a side table — never rewriting the IR
(I-1). The B3.3 "done when" rubric — *switch recovery handles
compiler-emitted jump tables on x86-64* — is gated by the
`hand_built_jump_table_round_trip` unit test, which assembles a
synthetic `if (idx < 4) { jmp table[idx*8]; } else { return; }`
function and asserts the recovered `SwitchTableIdiom` carries the
correct base, stride, width, and upper bound.

- `dac-recovery::idioms` (new ~400-line module):
  - `RecoveredIdioms { switch_tables }` — a single `BTreeMap` for
    deterministic ordering, keyed by the [`SsaBlockId`] of the
    [`SsaTerminator::Indirect`] dispatch block. Additional idiom
    families (error-guard returns, ref-counting, simple state
    machines) land as new sibling fields in later batches; the
    module docs map each future detector to its prerequisite.
  - `SwitchTableIdiom { source_block, scrutinee, table_base_const,
    element_stride, element_width, bound, confidence }` — records
    the *shape* of the dispatch without resolving the table's
    entries. Per-entry resolution requires reading `.rodata` /
    relocations and lives downstream (likely B3.4).
  - `recover_idioms(ssa) -> RecoveredIdioms` — the single public
    entry. Total: every block is walked; non-matches are silent.
  - `SWITCH_TABLE_CONFIDENCE = 0.70` — `Source::Derived`. The
    structural shape is observable but the *claim* "this is a
    switch" is derived from it (I-3).
  - Heuristics:
    - **Switch table.** Scan blocks whose terminator is
      [`SsaTerminator::Indirect`]; walk the block's instructions
      in reverse looking for a [`SsaOp::Load`] whose address
      decomposes via [`SsaOp::Add`] to `(table_base, scaled_idx)`
      with `scaled_idx` matching `Mul(idx, c)` or `Shl(idx, k)`.
      Stride 1 is rejected (mirrors the array-recovery rule from
      [B3.2](#b32--struct-and-array-recovery-2026-06-02)). The
      table base, when a constant or `Move`-of-const value, is
      recorded as `table_base_const`; PIC-style relative tables
      leave it as `None`.
    - **Bound.** When the dispatch block has exactly one
      predecessor whose terminator is
      [`SsaTerminator::Branch`] with `taken == dispatch_block` and
      `not_taken != dispatch_block`, and the branch condition is a
      [`CompareKind::Ult`] / [`CompareKind::Ule`] of the
      scrutinee against a constant, the constant is recorded as
      the upper bound. Signed compares (`Lt`, `Le`) deliberately
      do not contribute — they do not forbid a negative scrutinee.
- `dac-recovery::lib`: `pub mod idioms;` plus re-exports of
  `recover_idioms`, `RecoveredIdioms`, `SwitchTableIdiom`, and
  `SWITCH_TABLE_CONFIDENCE`.
- Tests (13 in `dac_recovery::idioms::tests`):
  - `indirect_block_with_mul_indexed_load_is_a_switch_table` —
    canonical `Add(base, Mul(idx, 8)) + Load(width=8)` shape.
  - `indirect_block_with_shl_indexed_load_is_a_switch_table` —
    `Shl(idx, 3)` (power-of-two stride) recognised.
  - `indirect_block_with_stride_4_table_records_width_4` — relative
    int32 tables register `stride = 4`, `width = 4`.
  - `predecessor_compare_supplies_upper_bound` — `Compare(Ult, idx,
    16)` in a single predecessor pins `bound = Some(16)`.
  - `ule_compare_also_supplies_bound` — `Ule` is treated as a
    valid bounding compare.
  - `signed_lt_does_not_supply_bound` — signed `Lt` does not.
  - `return_terminator_does_not_produce_switch` — non-`Indirect`
    terminators never surface a switch.
  - `indirect_without_indexed_load_produces_no_proposal` — a bare
    `jmp rax` from a function pointer does not falsely match.
  - `stride_one_is_not_a_switch_table` — stride 1 rejected.
  - `recovery_is_deterministic_across_runs` and
    `empty_function_produces_empty_output` pin determinism +
    degraded inputs.
  - `every_recovered_confidence_is_derived` checks that no
    `Observed` / `Speculative` confidence leaks out (I-3).
  - `hand_built_jump_table_round_trip` — the PLAN rubric: a
    synthetic `if (idx < 4) { jmp table[idx*8]; }` resolves to a
    `SwitchTableIdiom` with `table_base_const = Some(0x404000)`,
    `element_stride = 8`, `element_width = 8`, `bound = Some(4)`.

Closes FR-18 (idiom recognition for switches) and the relevant
slice of spec §11.4. The pass is `Source::Derived` (no AI input),
deterministic (NFR-9, I-4), and additive (the IR remains the
source of truth, I-1). Error-handling patterns, ref-counting, and
simple state-machine detection are deliberately deferred — each
needs additional infrastructure (errno tables from
`dac-knowledge`, atomic/lock-prefix modelling at the SSA layer,
phi-of-state-constants tracking) and lands in a follow-up batch
inside Milestone 3. Resolving individual jump-table entries (which
requires reading the binary's `.rodata` or its relocation table)
is left to the B3.4 annotation channel, which can carry table
data; the [`SwitchTableIdiom`] shape it consumes is the deliverable
here.

#### B3.4 — Annotation channel and confidence reporting (2026-06-02)

Annotation channel for `dac-cli`. Every name and type in the
emitted C unit becomes traceable through the evidence graph
(I-2, FR-19, FR-23, FR-25, spec §10.4 / §12). Lands as a new
`dac_cli::annotations` module that lifts the recovered facts
plus the `EvidenceGraph` predecessor chain into a structured
document, plus two views:

- `<output>.annot.json` — deterministic JSON sidecar written when
  `--emit-annotations` is set (spec §10.2 "annotations / notes"
  artifact). Hand-rolled writer with fixed key order; identical
  inputs → byte-identical output, validated by the
  `emit_annotations_output_is_byte_stable_across_reruns` golden.
- `--debug` augmentation of the C unit — each recovered function's
  `/* … */` leading comment gains a "Why this name?" /
  "Why this return type?" block listing value, source class,
  confidence, explanation, and the evidence-node chain. Emitted C
  still compiles end-to-end (round-trip gate in
  `debug_mode_emitted_c_still_compiles`).

The B3.4 "done when" rubric — *every name and type in emitted C
is traceable through the evidence graph in `--debug`* — is closed
by `debug_mode_embeds_evidence_trail_in_c_function_comments`,
which asserts both blocks plus the per-fact `explanation:` and
`evidence:` lines appear in the `.c` sidecar.

- `dac-cli::annotations` (new module):
  - `AnnotationDoc { tool, input, settings, evidence, functions,
    notes }` — top-level document. `EvidenceSummary` carries a
    fixed-key histogram so the count-by-variant rendering is
    byte-stable.
  - `FunctionAnnotation { address, end, signals, name,
    return_type }` — per-function fact bundle. Additional surface
    facts (recovered stack locals, inferred parameters, struct /
    array layouts, switch-table idioms) slot in as new fields when
    the lifter → SSA → structurer bridge starts feeding them into
    `TranslationUnit` in a later batch.
  - `FactAnnotation { value, confidence, explanation, evidence }`
    — single annotated fact. `confidence` is a `dac_core::Confidence`
    (value + `Source` class, I-3); `evidence` is a vector of
    `EvidenceRef` chained from the fact's own [`EvidenceId`]
    through every `Supports` predecessor in the [`EvidenceGraph`]
    via a single-pass reverse index.
  - **Name annotation.** Symbol-table-backed names render as
    `Source::Observed` with `SYMBOL_CONFIDENCE`; synthesized
    `fn_<hex>` fallbacks render as `Source::Derived` value `0.0`
    (the address carries no semantic content). Both cite the
    function's IR-node and supporting byte-span node in the chain.
  - **Return-type annotation.** All functions render `void`
    `Source::Derived` value `0.0` today; the explanation records
    "default void return; calling-convention return-value
    inference lands with B3.6" so a reader can distinguish
    *pending* void from *observed* void.
  - `render_annotations_json(doc) -> String` — deterministic JSON
    writer. Key order: tool → input → settings → evidence →
    functions → notes. Confidence values formatted as `{:.3}`.
  - `render_function_debug_block(fn_annot) -> String` — line-
    oriented plain text safe to drop into a C comment (no `*/`).
- `dac-cli::main`:
  - `mod annotations;` plus a `build_annotations_doc` helper that
    stamps the active CLI flags (`level`, `target`, `debug`) into
    the document header.
  - `run_pipeline` builds the doc both for the supported-arch
    path (after `discover_functions`) and the unsupported-arch
    path (with an empty graph), then routes it through
    `render_source_text` and `emit_outputs`.
  - `render_c_unit` consumes the doc plus the `args.debug` flag
    and, when `--debug` is set, appends the per-function debug
    block to each `leading_comment`.
  - `emit_outputs` gains an optional `annotations` parameter:
    written as `<output>.annot.json` when `--output` is set, or
    as a delimited `;; ---- annotations (FR-19, FR-23, FR-25) ----`
    block on stdout otherwise.
- Tests (11 in `dac_cli::annotations::tests`):
  - `symbol_derived_name_renders_as_observed_with_evidence_chain`
    — symbol-table source → `Observed`, value ≥ 0.9, chain
    contains both ir-node and bytes-node refs.
  - `synthesized_name_is_derived_with_zero_value` — `fn_<hex>`
    fallback → `Derived` value `0.0`.
  - `return_type_is_void_derived_pending_signature_inference` —
    return-type annotation explicitly cites B3.6 as the
    prerequisite for the next refinement.
  - `render_annotations_json_is_byte_stable_across_calls` and
    `render_annotations_json_carries_every_top_level_section`
    pin determinism (NFR-9) and the JSON contract.
  - `evidence_summary_counts_match_graph` — histogram totals
    equal `graph.node_count()`.
  - `debug_block_renders_why_this_name_and_why_this_type` — the
    `--debug` view contains both "Why this …?" headers, every
    per-fact line, and never produces `*/` (safe in a C comment).
  - `empty_function_set_produces_an_explanatory_note` covers the
    degraded path (no architecture backend).
  - `signals_list_is_alphabetical` pins the byte-order contract
    for the `signals` JSON array.
  - `json_string_escapes_quote_and_control` covers the JSON
    string escape table.
  - `evidence_chain_terminates_on_cycle` — the breadth-first walk
    over the append-only graph deduplicates by node id, so a
    `Supports` self-loop or `a ↔ b` cycle cannot loop the
    renderer.
- Integration tests (4 in `crates/dac-cli/tests/annotations_cli.rs`):
  - `emit_annotations_writes_a_structured_sidecar` — runs
    `dac -O1 --target c --emit-annotations --output <tmp>`
    against the ELF fixture and asserts every top-level section
    plus the `name` / `return_type` / `confidence` /
    `explanation` / `signals` keys appear.
  - `emit_annotations_output_is_byte_stable_across_reruns` —
    NFR-9 gate: two consecutive runs produce identical
    `.annot.json` sidecars.
  - `debug_mode_embeds_evidence_trail_in_c_function_comments` —
    the PLAN rubric: the `.c` sidecar contains
    `Why this name?`, `Why this return type?`, `explanation:`,
    and `evidence:` lines when `--debug` is set.
  - `debug_mode_emitted_c_still_compiles` — the debug-augmented
    C unit round-trips through `cc -x c -c -` (matches the
    skip-when-no-compiler contract from `o1_target_c.rs`).

Closes FR-19 (uncertainty annotation), FR-23 (separate annotation
channel), FR-25 (structured recovery report covers the
annotation-doc layer), and the relevant slice of spec §10.4
(annotation source-class taxonomy) and §12 (trace-mode "why"
rendering). Spec §10.3's `explanation` + `dependencies` fields
land as `FactAnnotation::explanation` and the `evidence` chain
respectively. The pass is `Source::Derived` (no AI input),
deterministic (NFR-9, I-4), and additive (I-1 — the IR remains
the source of truth; the doc is a strictly downstream artifact).

Inferred calling-convention parameter lists (B2.5), propagated
value types (B2.6), recovered struct / array layouts (B3.2), and
switch-table idioms (B3.3) all sit in `dac-recovery` side tables
today but do not yet surface in the emitted C, so annotating
them here would describe facts the reader cannot find in the `.c`
sidecar. They slot into `FunctionAnnotation` as additional fields
once the lifter → `RawFunction` bridge drives the structurer.
Jump-table entry resolution (the deferred follow-up from B3.3)
similarly waits for `.rodata` reading; the `evidence` chain
shape it will use is the deliverable here.

#### B3.5 — C++ backend (`-O2`) (2026-06-02)

C++ target-language backend lands as `dac-backend-cpp`, closing
FR-21's C++ slice and the relevant parts of spec §6. The recovery
side is symbol-driven at this batch: Itanium-mangled symbols
(`_ZN…`, `_ZNK…`, `_Z…`, `_ZTV…`, `_ZTI…`) feed a flat
`RecoveredClasses` table that the lowering pass turns into a
`TranslationUnit` of `class <Name> { … };` shapes. The B3.5
"done when" rubric — *a sample C++ binary with a small class
hierarchy decompiles to C++ that compiles* — is closed by
`o1_target_cpp_round_trips_through_system_compiler`, which pipes
the emitted `.cpp` through `c++ -std=c++17 -c -` on the
`cpp-hierarchy-x86_64` fixture (a 3-class Animal / Dog / Cat
hierarchy with virtual `speak()`).

- `dac-backend-cpp` (now non-stub):
  - `mangle` — a minimal Itanium-ABI reader covering nested-name
    methods (`_ZN…E…`), const members (`_ZNK…`), ctor / dtor
    variants (`C[123]E…`, `D[012]E…`), free functions
    (`_Z<name>…`), and the four special data symbols every
    polymorphic class produces (`_ZTV`, `_ZTI`, `_ZTS`, `_ZTT`).
    Templates, substitutions in the nested name, and operator
    overloads are explicit deferrals — the reader returns `None`
    and the recovery degrades by leaving the symbol on the free-
    function pile rather than guessing. 11 unit tests pin the
    accepted grammar and the `None`-on-garbled-input behaviour.
  - `class_recovery::recover_classes` — symbol-driven class
    discovery (FR-21). Member-function symbols populate a class
    bag; `_ZTV<class>` symbols promote the class to polymorphic
    (`has_vtable = true`); `_ZTI<class>` records typeinfo. Ctor
    and dtor variants land as distinct `RecoveredMember` entries
    sorted by `(MemberSortKey, address, mangled)`. Each class
    mints an `IrNode { layer: Source }` node in the
    `EvidenceGraph` (I-2) and links every member function's
    evidence handle into it via a `Supports` edge; polymorphic
    classes additionally link a `KnowledgeFact(FNV1a64(qualified
    name))` node to record the "we believe this is polymorphic
    because we saw a `_ZTV*` symbol" signal. 9 unit tests pin
    every path: single-method class, vtable promotion, ctor /
    dtor variant capture, nested scope chain, free-function
    sorting, address-based de-dup off the free pile, evidence-
    node layer, and run-to-run determinism.
  - `ast` — closed C++ AST: `TranslationUnit` →
    `Item::{Class, FreeFunction}`; `Class` carries `name`,
    `scope_chain`, `bases`, `has_vtable`, `members`; member
    functions carry `kind` (`Method` / `Constructor` /
    `Destructor`), `is_const`, `is_virtual`, return type;
    `CppType` covers `Void`, fixed-width `Int`, `Ptr`, `Ref`,
    `Const`, and `Class { qualified_name }`. 5 unit tests pin
    the variant set and the exhaustive-match contract.
  - `lower::lower_unit` — `RecoveredClasses` + `FunctionSet` →
    `TranslationUnit`. Ctor / dtor variants collapse to a single
    member (Itanium variants share the source-level signature, so
    emitting two would produce a duplicate-definition error); the
    leading comment records every variant's address + mangled
    symbol so the annotation channel surfaces them. Polymorphic
    classes without a recovered dtor get a synthesised
    `virtual ~Class();` so the emitted unit is well-formed C++
    (I-6 — the leading comment makes the synthesis explicit).
    `main` always lowers to `std::int32_t main()`; every other
    free function defaults to `void` until B3.6's signature
    recovery plumbs real types in. 7 unit tests pin the collapse,
    the dtor synthesis, the virtual-method promotion, and the
    `main` special case.
  - `emit` — hand-rolled deterministic pretty-printer. Renders
    leading comments as `// …`, classes with a `public:` block,
    `virtual` and `const` keywords in the canonical order, ctor /
    dtor name handling (no return type, tilde-prefix for dtors),
    pointer / reference / const type spellings, and a stub body
    that returns `return T{};` for non-`void` returns. 10 unit
    tests pin the byte-stable output across class / free-function
    / base-spec / type-spelling variants.
  - `compile::try_compile` — mirrors `dac_backend_c::compile` but
    invokes `c++ -x c++ -std=c++17 -c -`. Returns
    `CompileResult::Skipped` when no C++ compiler is on `PATH`.
    4 unit tests pin the candidate-list, the success and failure
    cases, and a class-with-virtual-dtor round-trip.

- `dac-cli`:
  - `--target cpp` at `-O1`+ now produces `<output>.cpp` (or a
    delimited stdout block). The CLI runs `recover_classes`
    against the binary's symbol table, feeds the result through
    `lower_unit`, and renders via `cpp_emit`. The banner comment
    surfaces the recovered counts (`classes`, polymorphic,
    member functions, free functions) so a `--debug` reader can
    see how many of the binary's symbols the recovery captured.
  - `render_source_text` now threads the `EvidenceGraph` so the
    C++ class-recovery pass can link evidence nodes (I-2).
  - The source sidecar suffix follows the target: `.c` for
    `--target c`, `.cpp` for `--target cpp`. The xtask golden
    harness gained a matching `OutputKind::CppSource` variant.

- Fixture and goldens:
  - `tests/fixtures/cpp-hierarchy-x86_64` — a 16 KiB PIE
    executable built from a 3-class Animal / Dog / Cat hierarchy
    with virtual dispatch. Built with
    `g++ -Os -fno-exceptions hello_cpp.cpp -o
    cpp-hierarchy-x86_64`; the source is reproduced in
    `tests/fixtures/README.md`.
  - `tests/golden/cpp-hierarchy-o1-cpp/` — listing + manifest +
    `source.cpp` capture. The new golden case is wired into
    `xtask::golden::CASES` so `cargo xtask ci`'s `golden check`
    gates drift across re-runs.

- Integration tests (`crates/dac-cli/tests/o1_target_cpp.rs`):
  - `o1_target_cpp_emits_cpp_sidecar_with_recovered_classes` —
    asserts `class Animal`, `class Dog`, `class Cat`,
    `virtual ~Dog`, `virtual ~Cat`,
    `virtual std::int32_t speak() const`, and
    `std::int32_t main()` all land in the emitted `.cpp`.
  - `o1_target_cpp_round_trips_through_system_compiler` — the
    PLAN.md done-when gate. Pipes the emitted `.cpp` through
    `c++ -std=c++17 -c -` and fails on any compiler diagnostic.
    Skips silently when no `c++` is on `PATH`.
  - `o1_target_cpp_output_is_deterministic` — two runs produce
    byte-identical `.cpp` (NFR-9).
  - `o1_target_c_still_emits_dot_c_sidecar_against_cpp_fixture`
    — sanity check that `--target c` continues to work against
    a C++ binary (the class-blind backend still produces a
    valid `.c` sidecar with one `void <name>(void)` per
    recovered function).

Explicit B3.5 deferrals — each is documented at the call site so
the next pass can pick them up without re-reading this entry:

- **Base-class recovery.** The lowering reserves `Class::bases`
  but always leaves it empty: identifying bases requires a
  typeinfo-relocation walker that reads
  `__si_class_type_info` / `__vmi_class_type_info` shapes out
  of `.data.rel.ro`. Lands when the relocation reader exists.
- **Signature recovery.** Every method, ctor, dtor, and free
  function emits an empty parameter list today. The AST already
  has `Param` / `CppType::Ref` / `CppType::Const` slots; B3.6's
  user-hint plumbing feeds them.
- **Real bodies.** The lifter → SSA bridge that drives the
  structurer from x86-64 bytes is not yet a batch in PLAN.md, so
  every emitted member / free function carries a deterministic
  stub body (`// dac C++ stub: lifter→SSA bridge pending` +
  `return T{};` for non-`void` returns). The leading comment
  makes the degradation explicit (I-6).
- **Namespace lowering.** Scope chains are flattened into the
  class leading comment until B3.6 can ground them; the AST
  already carries `Class::scope_chain` so adding `namespace`
  emission is additive.
- **Stripped-binary recovery.** A stripped C++ binary with no
  `_Z…` symbols falls through to an empty class table. A byte-
  level vtable scanner across `.data.rel.ro` reservation
  patterns lands in a later batch.

#### B3.8 — `dac-lift`: Instruction IR → RawFunction bridge (2026-06-02)

The missing leg in the per-function pipeline. Until this batch
landed, `dac-lift` had been a stub since M0 (`Status: stub. Real
lifting lands with B1.4.`) — B1.4 actually delivered the
`InstructionIr` decoder/lifter inside `dac-arch-x86`, so the
*bridge* the spec assigned to this crate (per-instruction `Operation`
→ per-block `RawFunction` for the SSA constructor) was never written.
The B2.x / B3.x deferral trail repeats the same phrase across
`dac-cli`, both backends, and the B2.8 / B3.4 / B3.5 CHANGELOG
entries: *"the lifter → `RawFunction` bridge is not yet a batch in
PLAN.md"*. This batch makes it one and closes it.

Closes both legs of the PLAN.md B3.8 done-when rubric:

- A hand-crafted x86-64 if-then-else CFG lifts through
  `lift_function` → `construct_ssa` → `structure` to a
  [`SemFunction`] whose body carries a `Stmt::If`
  (`end_to_end_diamond_construct_ssa_then_structure_produces_if`).
- The `hello-x86_64` fixture's `main` lifts end-to-end to a
  non-trivial `SemFunction` — at least one statement, at least one
  SSA value, at least one block with body ops
  (`hello_x86_64_main_lifts_to_a_non_trivial_sem_function`).

What landed in `dac-lift`:

- `bridge::lift_function(cfg, instructions_per_block, register_file)
  -> RawFunction` — the public entry. Asserts the
  `instructions_per_block.len() == cfg.blocks.len()` shape but
  degrades every other failure mode to honest IR rather than
  panicking (I-6). `must_use`.
- `Builder` — translation state. Holds the variable table, the
  canonical-name cache (`BTreeMap<String, VariableId>` for
  deterministic iteration), the pending flag-setter from the
  most-recent in-block `Compare`/`Test`, and a monotonic
  `synth_counter` for address / compare-result temporaries.
- Register variable model — one [`VariableId`] per *canonical*
  64-bit register. Sub-register operands (`eax`, `ax`, `al`, etc.)
  walk `RegisterFile::register(parent_id)` and land on the same
  variable as their 64-bit parent. The known-loss — that a 32-bit
  write doesn't zero the upper 32 in this representation — is
  documented at the call site and listed first in the PLAN.md
  "B3 follow-up shelf".
- Operation translation:
  - `Move`, `Add`, `Sub`, `Mul`, `And`, `Or`, `Xor`, `Shl`,
    `Shr`, `Sar` (lossy → `Shr`), `Neg`, `Not`, `LoadAddress`
    land on the corresponding `RawOpKind`. `dst = dst <op> src`
    read-modify-write semantics handled inline.
  - `Push` / `Pop` synthesise `rsp ±= 8` plus a `Store` / `Load`.
  - `Compare` and `Test` are *stashed* on the builder, not
    emitted. The next conditional `Operation::Jump` consumes the
    pending flag-setter at terminator-build time, emits a
    [`RawOpKind::Compare`] with the Jcc-derived [`CompareKind`],
    and wires the result into [`RawTerminator::Branch`].
  - `Return` reads the SysV return register (`rax`) and lands on
    [`RawTerminator::Return { value: Some(Variable(rax_var)) }`].
  - `Call` translates as [`RawOpKind::Call`] with the resolved
    target VA (or `None` for indirect), conservatively reads every
    SysV argument register (`rdi`, `rsi`, `rdx`, `rcx`, `r8`,
    `r9`) so liveness stays sound, and conservatively defines
    `rax` so the call-site gets a fresh SSA name for the return
    value. B3.10's argument-count inference narrows this when it
    lands.
  - `Opaque`, `Interrupt`, `Syscall`, `Div`, and any decode-error
    Operation surface as [`RawOpKind::Opaque`] with mnemonic
    preserved — the SSA constructor still sees a side-effect node
    rather than the lifter silently skipping it (I-6).
  - `Nop` is dropped (no SSA effect; CSE would erase it
    immediately).
- Memory-operand expansion. `[base + index*scale + disp]`
  addressing modes expand inline into a chain of synthetic `Add` /
  `Mul` raw ops that produce a single address [`RawOperand`]; that
  operand drives a [`RawOpKind::Load`] (read) or
  [`RawOpKind::Store`] (write) with the operand's width
  (`mem_width_bytes` rounds `size_bits` up to bytes, capped at 8).
- Branch-target resolution via the CFG. The bridge never re-parses
  the target VA out of the `Jcc` instruction; it walks
  `Cfg::edges` for the `EdgeKind::Taken` / `EdgeKind::NotTaken` /
  `EdgeKind::Branch` / `EdgeKind::Fall` neighbour. Edges are
  already sorted by the CFG builder, so the lookup is
  deterministic. Unresolved branch targets (no matching edge)
  degrade the terminator to [`RawTerminator::Indirect`].
- `condition_to_compare_kind` maps every `Condition` to a
  [`CompareKind`] when one exists. Sign / overflow / parity /
  `CxZero` have no two-operand-compare counterpart in the SSA
  vocabulary; their blocks fall back to [`RawTerminator::Indirect`]
  so the structurer doesn't see a comparison the bridge couldn't
  justify (I-6 honest degradation).

Unit tests in `bridge::tests` (10, all `Determinism::Pure`):

- `subreg_writes_canonicalise_to_64bit_parent` — `xor eax, eax`
  materialises a single `rax` variable; no separate `eax`.
- `return_terminator_reads_rax_value` — a bare `ret` block lands
  on `RawTerminator::Return { value: Some(Variable(rax)) }`.
- `compare_then_jcc_collapses_into_branch_terminator` —
  `cmp rax, 0; je 0x10` produces a `Branch` terminator whose
  `taken` / `not_taken` match the CFG's edge wiring and whose
  body's last op is a `RawOpKind::Compare { kind: Eq, … }`.
- `unsupported_condition_degrades_to_indirect` — `jp` degrades
  honestly.
- `nop_does_not_emit_a_raw_op` — `Nop` is dropped; the block has
  zero body ops.
- `opaque_passes_through_with_preserved_mnemonic` — an unmodelled
  iced mnemonic surfaces in `RawOpKind::Opaque::mnemonic`
  verbatim.
- `jcc_without_prior_compare_degrades_to_indirect` — Jcc with no
  pending `Compare`/`Test` becomes `Indirect`, never invents a
  comparison.
- `unconditional_jump_resolves_to_branch_edge_target` — `jmp 0x10`
  picks the `EdgeKind::Branch` successor from the CFG.
- `lift_function_is_deterministic_across_runs` — two runs over
  the same input produce byte-identical `RawFunction` (NFR-9).
- `end_to_end_diamond_construct_ssa_then_structure_produces_if`
  — the PLAN.md done-when rubric, leg 1.

Integration tests in `tests/end_to_end.rs` (2):

- `hello_x86_64_main_lifts_to_a_non_trivial_sem_function` — the
  PLAN.md done-when rubric, leg 2. Drives the whole pipeline on
  the existing `hello-x86_64` fixture and asserts the resulting
  `SemFunction` has at least one statement, the `SsaFunction`
  has at least one value, and at least one `RawBlock` carries
  body ops. Guards against the bridge silently regressing to the
  pre-B3.8 stub state.
- `lift_function_is_byte_stable_across_two_runs_on_a_real_binary`
  — NFR-9 / I-4 on a real ELF.

Wiring:

- `crates/dac-lift/Cargo.toml`: drops the lone `[lints]` block in
  favour of `[dependencies]` (dac-analysis, dac-arch, dac-ir
  workspace-pinned) + `[dev-dependencies]` (dac-arch-x86,
  dac-binfmt, dac-core, dac-recovery for the integration tests).
- `crates/dac-lift/src/lib.rs`: full module doc, `pub mod bridge`,
  `pub use bridge::lift_function`. Drops the `Status: stub` line.

Closes: FR-8 (the lifter's output is finally consumable by the
downstream pipeline), FR-11 (use-def / SSA actually reachable
from real binaries), partial FR-13 (calling-convention drives
call argument modelling at the bridge). Invariants: I-2 (the
`SsaFunction` produced by the constructor inherits its evidence
from the source CFG — no extra evidence-graph wiring needed
here), I-4 (`Determinism::Pure`, validated by two byte-identity
tests), I-6 (Opaque / Indirect / Unreachable degradations are
honest about what the bridge can't yet model).

Explicit B3.8 deferrals — each is documented at the call site
and most are now listed on the PLAN.md "B3 follow-up shelf":

- **Subreg-aliasing precision.** Sub-register writes land on the
  full 64-bit parent variable. The x86-64 "32-bit write zeroes
  the upper 32" semantics is dropped — a follow-up batch will
  refine.
- **Stack-slot detection before SSA.** The B2.4 stack-frame pass
  runs *after* SSA construction (it reads the SSA function), so
  pre-SSA stack-slot synthesis isn't this batch's job. `[rsp+N]`
  / `[rbp+N]` memory operands land as ordinary `Load` / `Store`
  with synthetic address-compute temporaries; B3.10's
  recovery-facts-into-source pass surfaces them as named locals.
- **Architecture other than x86-64.** The bridge takes a generic
  [`RegisterFile`] for canonicalisation, but the return register
  and call-argument register list are hard-coded to System V
  AMD64. AArch64 lands with a parameterised convention table in
  B5.2.
- **Mid-block terminators.** `Operation::Return` /
  `Operation::Jump` that appear mid-block (rather than as the
  block's last instruction) surface as `RawOpKind::Opaque` —
  the CFG builder already filters those into separate blocks in
  practice, but the bridge is defensive.

Closes B3.8. Test counts: `cargo xtask ci` reports green; 12 new
tests in `dac-lift` (10 unit + 2 integration); 25 golden outputs
across 10 cases still match without regeneration (the C / C++
backends still emit stubs because the orchestrator-side wiring
is B3.9's job).

#### B3.9 — End-to-end pipeline orchestration in `dac-cli` (2026-06-02)

The B2.8 / B3.4 / B3.5 deferral trail closed: `--target c -O1`
now emits real lowered bodies instead of the
`/* lifter→SSA bridge pending */` stubs. The CLI runs the full
deterministic pipeline once per recovered function
(`build_cfg → InstructionIr → lift_function → construct_ssa →
DominatorTree / PostDominatorTree / LoopForest → structure →
lower_function → emit`) and the recovered `FunctionSet` is now
threaded into the C backend's `NameResolver` so direct calls
resolve to `function_name(…)` instead of the
`((void (*)())0xNN)(…)` fallback.

##### New code

- **`dac-cli` crate.**
  - `crates/dac-cli/src/lift.rs` — new module. `LiftOutcome` enum
    (`Real { ssa, sem }` / `Stub { reason }`), `LiftStats`
    aggregator, `lift_all` / `lift_one` orchestrator. The
    orchestrator runs every constituent pass in fixed order so
    NFR-9 holds: same bytes in → identical `LiftOutcome` vectors
    out, byte-for-byte.
  - `crates/dac-cli/src/main.rs`:
    - `pick_backend` now returns a `&'static RegisterFile`
      alongside the decoder + lifter, recovered through a small
      `x86_64_register_file_static` helper that promotes a
      `static X86_64` to make the trait method's elided lifetime
      compatible with `'static`.
    - `render_source_text` takes the orchestrator's per-function
      outcome slice and threads it into `render_c_unit`.
    - `render_c_unit` now consumes `lift_outcomes`: on
      `LiftOutcome::Real { ssa, sem }` it calls
      `dac_backend_c::lower_function` and then post-processes the
      `name` through `sanitize_c_identifier` so symbols like
      `_GLOBAL__sub_I_…` still produce valid C; on
      `LiftOutcome::Stub` it falls back to the B2.8 stub shape
      and writes the degradation reason into the body's leading
      comment (I-6, FR-21).
    - `build_c_name_resolver` constructs the
      `BTreeMap<u64, String>` consumed by
      `dac_backend_c::lower::call_expr` so every recovered VA
      resolves to its sanitised symbol.
    - `real_body_leading_comment` builds a unified per-function
      comment that combines the recovered-function head
      (`address` / `end` / `confidence`) with the structurer's
      stats (`source_blocks` / `goto_count` / `label_count` /
      `irreducible`). `stub_body_leading_comment` (renamed from
      the previous `function_leading_comment`) covers the
      degraded path.
  - `crates/dac-cli/src/report.rs` — `Report` now carries a
    `LiftStats` and `render_report_text` prints a
    `;; body cover.: {real} / {total} ({pct:.2}% real bodies, {stub} stubs)`
    line directly below the existing instruction-level `lift cover.`
    line.

- **`dac-backend-c` crate.**
  - `crates/dac-backend-c/src/emit.rs` — `Expr::Call` now wraps
    every target in an arity-matched
    `long long (*)(long long, …)` cast. The recovered calling
    convention (B2.5 `dac_recovery::infer_calling_convention`)
    is not yet threaded into the C lowering pass, so every
    recovered function lowers with an empty parameter list
    (`void f(void)`) while the bridge (B3.8) reads all six SysV
    AMD64 call-arg registers at every call site. Modern C
    (C23) interprets empty function-pointer parens `()` as
    `(void)`, so the K&R-style fallback the original B2.8
    `AddrLit` rendering relied on no longer accepts variadic
    actuals. The arity-matched cast keeps the round-trip `cc`
    gate green regardless of whether the result is assigned
    (`v0 = call(…)`) or discarded (`call(…);`) — the return
    spelling is `long long` for both cases. `Expr::AddrLit`
    now renders as the bare integer literal; the
    Call-context cast is synthesised in the emit's call branch.

##### Wiring + plumbing

- `crates/dac-cli/Cargo.toml` adds the `dac-lift` workspace dep
  used by the new orchestrator.

##### Tests

- 2 new unit tests in `dac-cli/src/lift.rs`
  (`lift_stats_round_trip`, `empty_outcomes_have_zero_fraction`).
- Every existing C/C++ end-to-end test stays green
  (`o1_target_c_*` × 3, `o1_target_cpp_*` × 4) including the
  round-trip-compile gates against the system `cc` / `c++`.
- `cargo xtask ci` clean: fmt + clippy + 25 golden outputs
  across 10 cases match after `cargo xtask golden update` for
  the three drifted outputs:
  - `hello-elf-o0-report/report.txt` — gained the new
    `;; body cover.: 9 / 9 (100.00% real bodies, 0 stubs)`
    line.
  - `hello-elf-o1-c/source.c` — every recovered function now
    has a real lowered body. `main` lifts to an `int64_t main(void)`
    with a recognisable structure (the leading-comment trail
    surfaces the recovered SSA-value count, the address
    range, and the structurer's irreducible / goto-count
    statistics).
  - `hello-pe-o1-c/source.c` — same story on the PE corpus
    (162 KB of real C bodies for `__mingw_invalidParameterHandler`
    and friends).

##### Deferrals — recorded as B3 follow-up shelf entries

- **C++ body lowering.** The C++ AST in
  `dac-backend-cpp::ast` does not yet model function bodies;
  `--target cpp` continues to emit class-shape stubs. Extending
  the AST + emit to consume the C-side `SsaFunction →
  SemFunction` shape lands as the B3.9 follow-up
  ("C++ body lowering" entry in the B3 follow-up shelf in
  PLAN.md). The C++ docstring on `render_cpp_unit` is updated
  to surface this.
- **Signature recovery.** All emitted C functions still use
  `void f(void)` signatures; the arity-matched call-target cast
  is the I-6 honest workaround until B3.10 threads
  `dac_recovery::infer_calling_convention` →
  `pick_best` → `InferredSignature` through the C lowering pass.

Closes B3.9. Closes FR-21 round-trip on real binaries (the
follow-up explicitly recorded in the B2.8 CHANGELOG entry).
Closes NFR-9 because every constituent pass is `Determinism::Pure`
and the orchestrator's iteration order matches `FunctionSet`'s
address-sorted layout — `cargo xtask ci`'s
`hello-elf-o1-c/source.c` golden regenerates byte-identically
on two runs.

### B3.10 — Recovery facts → emitted source (FR-13, FR-14, FR-17, FR-18, NFR-9)

Surfaces the per-function recovery side tables in the C source the
orchestrator emits at `--target c -O1`. Closes the "facts in
`dac-recovery` don't surface in the emitted source" debt recorded
across the B2.5 / B2.6 / B3.2 / B3.3 / B3.4 CHANGELOG entries.

**Per-function orchestrator (`dac-cli::lift`).** `LiftOutcome::Real`
now carries a boxed `RecoveryFacts { stack_frame, convention,
types, structs, idioms }`. Each constituent pass runs in the
order its data dependencies require: `analyze_stack_frame` →
`infer_calling_convention` (picks the highest-scoring match from
`X86_64_CONVENTIONS`) → `propagate_types` (seeded by the
convention's `int_args` and the stack frame) → `recover_structs`
(consults the type map for field types) → `recover_idioms`. The
orchestrator picks the binary-format-correct `StackConvention`
(`SysVAmd64` for ELF / Mach-O, `MsX64` for PE) and builds a
`BinaryImportResolver` against the binary's `Import` / `Symbol`
tables so `propagate_types` can seed types at direct-call sites
whose target VA matches an entry in `dac-knowledge`'s libc /
Win32 API catalogue.

A new switch-idiom post-pass (`lower_switch_idioms`) rewrites
every `SemStmt::Unreachable` whose `source_block` matches a
recognised `SwitchTableIdiom` into `SemStmt::Switch { scrutinee,
arms: [], default: Some(<the original Unreachable>),
source_block }`. The scrutinee surfaces; per-entry resolution
(reading `.rodata` and minting per-arm goto targets) is recorded
as a B3 follow-up shelf entry — the structurer's recursive walk
doesn't naturally visit blocks reachable only through the
indirect dispatch, so resolving arms cleanly needs more
plumbing.

The `lift_one` helper now takes a single `LiftCtx` reference
(bundling model + bytes + decoder + lifter + register file +
stack convention + API resolver) instead of a long arg list.

**C backend surface (`dac-backend-c`).** Three new AST nodes land:

- `Expr::Field { base, field, arrow }` and
  `Stmt::FieldStore { base, field, arrow, value }` model
  `base->field` / `base.field` accesses. The lowering pass at
  B3.10 detects the matching shape via
  `RecoveredStructs::pointer_structs` but, until the AST grows
  translation-unit-level `struct` typedefs (a B3 follow-up shelf
  entry), surfaces the recovery as a `/* recovered field:
  base=v_<id> offset=0x<hex> field=field_<hex> */` comment above
  the bare `Load` / `Store`. The arrow / dot rendering is
  exercised by `dac-backend-c`'s AST exhaustivity tests so the
  path stays warm.
- `Stmt::Switch { scrutinee, arms, default }` plus `SwitchArm
  { value, body }` model `switch (s) { case N: …; default: … }`.
  The emitter renders the `default` arm as `default: { … }` and
  arms as `case <value>LL: { … }` so per-arm break / fall-through
  semantics stay explicit when arm bodies start landing in the
  follow-up.
- `Expr::Cast { ty, expr }` for `((ty)(expr))`. The lowering pass
  uses it at the two int / pointer boundaries B3.10 introduces:
  parameter → local init (`int64_t v0 = (int64_t)arg0;`) and
  `Return { value: Some(_) }` operands when the return type is
  not `int64_t`.

`lower_function` now takes a `Recovered<'a>` view that bundles
optional refs to `InferredSignature`, `TypeMap`, and
`RecoveredStructs`. The lowering pass commits to:

- Materialising the convention's `int_args` as named C parameters
  (`arg0, arg1, …`) whose types come from `TypeMap::value_type`.
  Pre-declared `v<id>` locals for each parameter initialise from
  the matching `arg<n>` through an explicit `Expr::Cast` so the
  int / pointer boundary is explicit (FR-13, FR-14).
- Picking the return type from the convention's
  `return_register` and the join of every `Return { value:
  Some(_) }` operand's recovered type. The B2.8 fallback
  (`value: Some(_)` → `int64_t`, otherwise `void`) stays in
  force when the convention has no return register.
- Keeping non-parameter locals typed by `width_bits` for now.
  Refining local types directly from the lattice exposes
  pointer / int mixes the lifter's sub-register arithmetic
  produces, so refining is a B3 follow-up shelf entry.

`lower_unit` takes a parallel `&[Recovered<'_>]` slice; passing
`&[]` falls back to the B2.8 behaviour.

**CLI plumbing (`dac-cli`).** `lift.rs` now imports `dac-knowledge`
and `dac-recovery` enough to build the orchestrator's
recovery-facts pipeline. `main.rs`'s `lower_one_c_function`
threads `facts.convention.signature`, `facts.types`, and
`facts.structs` into the C backend's `Recovered` view, and
`real_body_leading_comment` cites the chosen convention name +
score, the inferred arg-register sequence, the return register,
the stack-local count, the pointer / stack struct layout counts,
and the recognised switch-table count.

**Report (`dac-cli::report`).** `--emit-report` gains a new line:

```
;; recovery:    typed_sigs=7 struct_fields=1 switch_tables=0
```

…between `;; body cover.: …` and the per-function table.
`LiftStats` accumulates `typed_signatures`, `struct_field_functions`,
and `switch_functions` per the new criteria: a "useful" convention
(at least one inferred arg or a return register), a recovered
`pointer_structs` entry, and a recognised `SwitchTableIdiom`.

**Bugfix in `dac-recovery::structs::lookup_def_op`.** Running
`recover_structs` on the PE corpus surfaced an out-of-bounds
panic: a `ValueSource::Instruction { block, index }` pointed past
the end of `block.instructions`. The function now bounds-checks
defensively and returns `None`, matching the existing degradation
path for non-instruction sources. The underlying SSA-source
inconsistency is recorded as a follow-up shelf entry in PLAN.md.

**Tests.** `cargo xtask ci` is green:

- `dac-backend-c`'s 26 unit tests (including `Recovered::default`
  paths) and 12 round-trip cases pass.
- `dac-cli`'s `o1_target_c_round_trips_through_system_compiler`
  passes after the int / pointer boundary casts.
- 25 golden outputs across 10 cases match; three updated
  intentionally:
  - `hello-elf-o0-report/report.txt` gained the new
    `;; recovery: …` line.
  - `hello-elf-o1-c/source.c` now shows the recovered convention
    in the leading comment (`/* convention: sysv-amd64 (score
    0.85) */`), typed parameters
    (`int64_t _init(int64_t arg0, int64_t arg1, …)`), and
    parameter → local init casts.
  - `hello-pe-o1-c/source.c` regenerates with the same plumbing
    against the MS x64 corpus.

Closes B3.10, FR-13 (convention surfaced in the source), FR-14
(parameter + return inference reflected in the C signature),
FR-17 (struct field recovery comment-surfaced ahead of the
typedef follow-up), FR-18 (switch idiom lowered into
`Stmt::Switch`, scrutinee visible), and NFR-9 (every new pass is
`Determinism::Pure`; the corpus output is byte-stable across
runs).

#### B3.6 — User hints / signatures (2026-06-04)

Wires a user-supplied hint catalogue into the recovery side-table
pipeline so a reverse engineer can override the recovered name /
return type / argument types of any function (FR-20). Hints are
loaded from a strict-TOML subset, registered in the evidence graph
as `EvidenceNode::UserHint` nodes (I-2), and overlay the
`TypeMap` with `Source::UserHint`-sourced entries (I-3). Because
`Confidence::join` is componentwise max and `UserHint` outranks
`Derived`, the existing C lowering's `parameter_type` /
`pick_return_type` paths render hinted types without further
changes.

- **`dac-hints` crate.** New no-external-dep workspace member
  containing:
  - A constrained TOML reader (`crates/dac-hints/src/parse.rs`)
    that handles `[[table]]` headers, scalar / array / inline-
    table values, hash comments, and `\"` / `\\` / `\n` / `\t`
    string escapes. Anything outside that envelope is rejected
    with a `HintError::Syntax { line, message }`. The reader is
    intentionally hand-rolled — the schema is small enough that
    the dep cost of `toml` + `serde` was not justified.
  - A type-string grammar (`HintType::parse`) covering
    `void`, `char`, `int{,8,16,32,64}`, `uint{,8,16,32,64}`,
    `short` / `long`, plus pointer suffixes (`int**`). Lowers to
    the IR lattice through `HintType::to_ir`.
  - `Hints { functions, structs }` with `find_function(addr,
    name)` lookup. `FunctionHint` carries `rename`, `return_ty`,
    `args`, an `HintId` assigned in source order, and a slot for
    the `EvidenceId` the CLI mints during `register_hints`.
  - `HintError` with `Io` / `NotUtf8` / `Syntax` / `Semantic`
    variants and a `message()` helper the CLI surfaces verbatim.
  - 15 unit tests covering the grammar, matcher precedence,
    semantic rejections (`void` arg, no-effect hint, missing
    matcher), and determinism of repeated parses.
- **Overlay in `dac-cli/src/lift.rs`.** `LiftCtx` now carries a
  `&Hints` borrow; `lift_one` runs `apply_function_hint` after
  the deterministic recovery passes:
  - For each hinted argument, stuff a `ValueType { ty:
    HintType::to_ir(), confidence: UserHint(0.95) }` entry into
    `facts.types` keyed by the corresponding `RegisterArg.value`.
  - When `return` is hinted, promote the convention's
    `return_register` to `Some("rax")` if inference left it
    `None` so the backend's `pick_return_type` path activates,
    then seed every `Return { value: Some(v) }` operand's
    TypeMap entry.
  - `rename` flows through `RecoveryFacts::user_hint`, which the
    `lower_one_c_function` site reads before sanitising into a
    C identifier. The recovered name and the hint rename both
    pass through the existing sanitiser so a rename containing
    `@` / `.` cannot break the round-trip compile gate.
- **CLI plumbing.** `--hints PATH` parses a hint file with
  `Hints::load_from_path`. On parse failure the CLI surfaces the
  `HintError::message()` line and exits non-zero. Successfully
  loaded hints are registered in the evidence graph via
  `register_hints`, so the annotation summary's
  `evidence.by_kind.user_hint` counter ticks up to match the
  source file's hint count.
- **Report counter.** `;; recovery: …` in `--emit-report` gains a
  `user_hints={n}` column counting functions whose recovered
  facts were overlaid by a `[[function]]` hint (B3.6's
  "reflected in the confidence report" criterion).
- **C lowering leading comment.** Functions with an applied hint
  print a `user_hint: id=N rename=… return_override=… args_override=…`
  line above the body so a reader sees that the signature was
  pinned by `--hints`, not inferred from the bytes.
- **Help text.** `--hints <path>` added to the CLI usage
  snapshot.
- **Goldens.** New `hello-elf-o1-c-hints` corpus case
  (`tests/golden/hello-elf-o1-c-hints/`) demonstrates the
  end-to-end behaviour on `hello-x86_64`:
  - The fixture's `main` (recovered at `0x1040`) is renamed to
    `user_main` with return `int` and arg `int`.
  - The emitted `.c` shows `int32_t user_main(int32_t arg0)`
    with a cast init `int64_t v8 = ((int64_t)(arg0))` and a
    return cast `return ((int32_t)(v12))`.
  - The matching `.report.txt` shows `user_hints=1` on the
    recovery row.
  - The `hello-elf-o0-report` golden also gains the new column
    `user_hints=0` so existing cases stay byte-stable.
- **`hello-x86_64.hints.toml` fixture.** Workspace-relative TOML
  the corpus harness feeds to `--hints`. Documents the schema
  inline so a contributor reading the fixture sees the supported
  shape.

Limitations (folded into the B3 follow-up shelf):

- **Argument synthesis stops at the inferred prefix.** A hint
  whose `args` lists more positional types than the convention
  pass observed (e.g. `args = ["int", "char**"]` on a function
  that only reads `rdi`) retypes the inferred slots but does not
  mint additional `RegisterArg` entries — the second slot would
  have no SSA-side value to bind. Synthesising additional
  parameters needs the C backend to learn a "declared but
  unused" parameter shape; tracked as a B3 follow-up.
- **Struct hints are parsed and counted, not applied.** The
  lowering pass still surfaces struct fields as `/* recovered
  field: … */` comments (B3.10 deferral). Promoting hinted
  layouts to `s->field` lands with the struct-typedef-surface
  follow-up.
- **`Source::UserHint` does not yet bubble into the annotation
  channel's per-fact source.** The summary counter ticks via
  the evidence graph, but `annotate_name` / `annotate_return_type`
  in `dac-cli/src/annotations.rs` still report the recovery
  pipeline's classification. Threading the hint's `EvidenceId`
  into the matching `FactAnnotation` is a B3 follow-up so the
  `.annot.json` sidecar can name the hint that pinned the type.

Closes: B3.6, FR-19 (user-hint source surfaces in the report
counter), FR-20 (user-supplied signatures and type hints land
end-to-end), I-2 (every hint enters the evidence graph), I-3
(every hint-driven retyping carries `Source::UserHint`).

### B3.7 — Variable naming heuristics (FR-N spec §11.1, NFR-9, I-3)

**Closes B3.7 and clears the last numbered M3 batch.** Two
deterministic naming heuristics now ship in `dac-recovery::names`
and feed the C backend in place of the `v<id>` fallback.

- **`dac-recovery::names` (new module).** Exposes
  `recover_names(ssa, signature, &dyn ApiResolver, &dyn StringResolver)
  -> NameTable`. `NameTable` carries the disambiguated identifier
  per SSA `ValueId` plus a `provenance` table that records the
  candidate, the [`NameSource`] (`ApiContext` / `StringRef`), and a
  `Source::Derived` [`Confidence`] of `NAME_CONFIDENCE = 0.80`
  (I-3). The pass walks `ssa.blocks` in source order; values
  belonging to convention-inferred parameters are skipped so the
  backend's `argN` parameter naming still wins.
- **API-context heuristic.** For each `SsaOp::Call { target:
  Some(va), args }` whose `va` resolves through the supplied
  [`ApiResolver`] to a `dac_knowledge::ApiSignature`, the i-th
  positional argument that is an `Operand::Value(v)` earns the
  catalogue parameter name (`open` → `path`, `flags`; `memcpy` →
  `dst`, `src`; `malloc` → `n`; etc.). Variadic tails are
  skipped — out-of-arity slots have no catalogue name. Reserved
  C keywords get a trailing `_` so the round-trip compile gate
  stays green.
- **String-literal heuristic.** For each `SsaOp::Move { src:
  Operand::Const(c) }`, the new `StringResolver` looks `c` up in
  the binary's `.rodata` strings. When the lookup hits, the text
  is slugified into `str_<lowercased_alnum>` (camel-case &
  punctuation collapse to `_`; trailing underscores trimmed;
  slug truncates at `MAX_STRING_LEN_FOR_NAME = 24` payload
  chars). Payloads shorter than `MIN_STRING_LEN_FOR_NAME = 2` do
  not contribute — empty / whitespace strings carry no signal.
- **Conflict resolution.** When several heuristics agree on a
  value, `NameSource`'s declaration order wins (`ApiContext >
  StringRef`). When multiple values share the same base
  candidate, deterministic disambiguation walks the BTree in
  ascending `ValueId` and appends `_1`, `_2`, … to subsequent
  occurrences so the disambiguated map is byte-stable across
  runs.
- **C backend plumb-through.** `dac-backend-c::lower::Recovered<'a>`
  gains an `Option<&'a NameTable>` field. `value_name(id,
  names)` now reads through the table before falling back to
  `format!("v{id}")`. Every free helper that printed a value —
  `lower_operand`, `format_operand`, `binary`, `call_expr`,
  `opaque_expr`, `format_phi_comment` — takes the optional
  table and threads it through. `Recovered::new` gained a 4th
  parameter; the CLI is the only call-site so the breaking
  change is local.
- **CLI orchestration.** `LiftCtx` gained a `BinaryStringResolver`
  that pre-computes a `VA → &str` map over `BinaryModel::strings`
  filtered to `SectionKind::ReadOnlyData` — write-target VAs
  matching `.data` bytes are deliberately skipped because the
  bytes there might just as easily be numbers, padding, or
  struct payloads (I-6: never invent semantics). `lift_one`
  runs `recover_names(...)` after the type / convention /
  struct / idiom passes complete and after the user-hint
  overlay so heuristic names see the final post-overlay shape
  of the convention's parameter set. The new
  `RecoveryFacts::names` field is stored alongside the existing
  `types` / `structs` / `idioms` / `user_hint` channels.
- **Confidence-report rubric (the "done when").** `--emit-report`'s
  recovery row gained a `;; naming:` line counting
  `named_values={X} / {Y} ({pct}% heuristic coverage)`. `Y` is
  every defined SSA value minus the convention's inferred parameter
  slots (which the backend already names `argN`). `X` is the sum
  of `NameTable::named_count()` across every `LiftOutcome::Real`.
  The B3.7 corpus rubric is satisfied: the PE `hello-x86_64.exe`
  golden now emits twelve heuristic identifiers — `dst`, `src`,
  `size`, `status`, `n`, `n_1`, `n_2`, `n_3`, `n_4`, etc. — where
  B3.10 would have shown only `v<id>`s.
- **Golden corpus refresh.** `hello-elf-o0-report` and
  `hello-elf-o1-c-hints` reports gained the new `;; naming:`
  line. `hello-pe-o1-c/source.c` regenerated to show the
  recovered names threaded through the lowering output (98-line
  diff of `v206 → n`, `v207 → size`, `v208 → dst` shapes).
- **Testing.** Eleven unit tests in `dac-recovery::names` cover
  the API-context heuristic on `strlen` / `open` / `puts`,
  parameter-skip behaviour, repeated-`s` disambiguation, variadic
  tail handling, the string slugifier (`Hello, world! → str_hello_world`,
  `ALL_CAPS → str_all_caps`, empty / single-char payloads
  rejected), keyword sanitisation (`int → int_`), the
  `ApiContext > StringRef` precedence rule, and a determinism
  property that confirms two runs on identical SSA produce
  identical `NameTable`s.
- **Limitations (B3 follow-up shelf).**
  1. *Loop-induction and counter naming.* `i` / `j` / `k` for
     loop scrutinees, `count` for `+= 1` lhs values, and
     allocator-size naming all need per-function dataflow
     reasoning that this batch does not introduce.
  2. *PLT-stub naming on ELF.* `BinaryImportResolver::resolve`
     does not yet walk the `.plt.sec` trampoline, so direct
     ELF calls into the PLT lower as `fn_<va>` and earn no
     API-context names — the same gap the type-propagation pass
     has at B2.6 surfaces here. The PE corpus does not hit this
     because PE symbols carry the import VA directly.
  3. *Hint-driven naming.* A function `[[function]]` hint with
     `rename = "foo"` overrides the symbol but does not yet
     propagate that name into the surrounding call sites' arg
     values; threading `Hints::find_function` into
     `recover_names` is the obvious next step.

Closes: B3.7, FR-N (spec §11.1 — naming heuristics).
Touches: I-3 (every name carries a `Source::Derived`
[`Confidence`]), NFR-9 (`recover_names` is `Determinism::Pure`,
BTree-iterated, deterministic), FR-25 (`--emit-report` surfaces
the heuristic-name coverage metric).

### B3.11 — Base-class recovery (FR-N spec §10, I-3, NFR-9)

**First of the M3 shelf follow-ups promoted to a numbered
batch.** `dac-backend-cpp::class_recovery` now walks Itanium
typeinfo objects in `.data.rel.ro` and resolves single- and
multiple-inheritance parent links straight out of the relocation
table — no more "base recovery is a B3.5 deferral" comment.

- **`RecoveredClass::bases` (new field).** Each base carries a
  `qualified_name` (demangled from the parent `_ZTI<chain>`
  symbol) and an `AccessSpec`. B3.11 emits `AccessSpec::Public`
  for every base; the `offset_flags` lower-byte holding the
  private / virtual flags lands in a follow-up that needs the
  input buffer plumbed through the recovery pass.
- **`RecoveredBase` (new type).** Public schema so downstream
  consumers (lower / annotate / future C++ body lowering) can
  read the inheritance graph without re-running the walker.
- **Typeinfo walker (third pass in `recover_classes`).** Itanium
  layout pins the byte size: 16 B = `__class_type_info` (no
  bases), 24 B = `__si_class_type_info` (one base pointer at
  offset 16), `24 + 16 * count` B = `__vmi_class_type_info`
  (count base pointers starting at offset 24, every 16 bytes).
  The walker reads `Symbol::size` to discriminate the shape and
  follows the relocations at the matching offsets to map each
  parent pointer back to its `_ZTI<chain>` symbol. The fallback
  for size-zero / stripped weak symbols is the vptr relocation's
  target-name substring match (`__si_class_type_info` /
  `__vmi_class_type_info`); it is best-effort because dynamic
  relocations into `.data.rel.ro` carry a mis-mapped symbol
  index in the ELF parser today (B3.18 surfaced the underlying
  bookkeeping gap).
- **`AccessSpec` re-used from `crate::ast`** so the lower pass
  hands the base spec to the C++ AST without translation. The
  printer's existing `render_base` rule emits the inheritance
  clause; no emit changes were required.
- **Lower plumbing.** `lower_class` walks `c.bases.iter()` into
  `Class::bases` (replacing the hardcoded `Vec::new()`), and the
  per-class leading comment carries a new `bases: …` line so
  `--debug` runs surface the recovered chain (`bases: Public
  Animal` for Dog; `bases: (none)` for Animal).
- **Confidence stays `Observed`.** The typeinfo bytes are in the
  binary itself: a class with a `_ZTI<chain>` typeinfo whose
  parent relocation targets another recovered `_ZTI<parent>` is
  not a guess. Bases drop silently when the parent typeinfo is
  not in the recovery set so we'd rather emit no inheritance
  than invent one (I-3, I-6).
- **`cpp-hierarchy-o1-cpp` golden refresh.** Dog and Cat now
  emit `class Dog : public Animal {` and `class Cat : public
  Animal {`, replacing the flat `class Dog {` shape; their
  leading comments include `bases: Public Animal`. Animal and
  the two `__cxxabiv1::__…_type_info` synthetic classes still
  show `bases: (none)` because they have no `_ZTI` of their own
  (the latter are recovered from the vtable imports rather than
  a user-defined typeinfo).
- **Tests.** Seven new unit tests cover: size-16 `__class_type_info`
  → no bases; size-24 `__si_class_type_info` → one parent;
  multi-inheritance walk that stops at the first missing
  relocation; SI walker dropping its base when the parent
  typeinfo is unknown; undefined-symbol skip; and a determinism
  cross-run check.

#### Limitations carried forward as the new "B3 residue shelf"

The original B3.5 deferral becomes a smaller cluster:

- **Virtual / private / non-public bases** still emit as
  `public`. Decoding the `offset_flags` lower byte needs the
  `.data.rel.ro` bytes; the typeinfo walker only reads
  relocations, not source bytes. The field is on
  `RecoveredBase` already.
- **Stripped C++ binaries** still recover no bases — the
  byte-level vtable / typeinfo scanner is its own shelf entry
  ("Stripped-binary C++ recovery").
- **`offset` field of `__base_class_type_info`** is also unread;
  emitted bases land in declared (typeinfo) order, which matches
  the source order for non-virtual inheritance.

Closes: B3.11, FR-N (spec §10 — C++ inheritance surface).
Touches: I-3 (every base carries `Source::Observed`), NFR-9
(`recover_classes` stays `Determinism::Pure`, BTree-iterated),
I-6 (bases drop visibly rather than synthesise an unrecovered
parent), FR-25 (per-class leading comment surfaces the
recovered chain).

### B3.12 — Namespace lowering (FR-21, I-6, NFR-9)

**Second of the M3 shelf follow-ups.** `dac-backend-cpp::emit` now
wraps every recovered class whose `scope_chain` is non-empty in
nested `namespace S1 { … }` blocks; the chain segments are emitted
in order, opened with `namespace <seg> {` and closed with
`} // namespace <seg>` so the matching block is greppable in the
output. The lowering pass already carried `scope_chain` through to
the AST — only the printer changed.

- **`emit_class_into` wraps in nested namespaces.** One `namespace
  <seg> {` open per segment in chain order, one `} // namespace
  <seg>` close in reverse order, with the class declaration (and
  its leading provenance comment) emitted at the innermost
  indent level. The merge-consecutive-prefix optimisation is
  intentionally not done: emit stays a pure per-item walk, so the
  same class always renders to the same block shape regardless of
  its position in `RecoveredClasses::classes`.
- **`cpp-hierarchy-o1-cpp` golden.** The two `__cxxabiv1::*`
  synthetic ABI classes (the C++ ABI's typeinfo bases recovered
  from vtable imports) now emit inside `namespace __cxxabiv1 { …
  } // namespace __cxxabiv1` blocks; the user-defined Animal /
  Dog / Cat classes still emit at global scope because their
  recovered `scope_chain` is empty.
- **Lower-pass docstring** updated: the old "no `namespace`
  lowering at B3.5" caveat (which had grown out of date by B3.5
  itself) is replaced by the B3.12 contract — `scope_chain`
  passes through to the AST verbatim and emit owns the
  wrapping.
- **Tests.** Two new emit unit tests cover length-1 and length-2
  scope chains, exercising the open / inner-indent / close
  walk and asserting the exact rendered shape.

#### Limitations carried forward

- **Cross-class namespace merge.** Two consecutive classes sharing
  the same scope chain each open and close their own namespace
  block. A future printer pass could collapse them into one
  combined block; the current shape is C++-legal but slightly
  noisier than hand-written code.
- **`namespace` recovery for free functions.** `RecoveredFreeFunction`
  has no `scope_chain` yet — Itanium-mangled `_ZN<scope>…E`
  free functions parse the chain on the class-recovery side, but
  the free-function pile only retains the demangled leaf name.
  Lands when free-function-side mangling exposes the chain.

Closes: B3.12, FR-21 (C++ surface fidelity for namespaced
classes), I-6 (emit reproduces the binary's recovered scope
chain rather than flattening it into a comment).
Touches: NFR-9 (emit stays pure / deterministic; no hashed
containers added).

### B3.13 — Variadic + syscall conventions (FR-13, I-3, NFR-9, I-6)

**Third of the M3 shelf follow-ups.** `dac-knowledge::convention`
gains a third x86-64 entry — `SYSV_AMD64_SYSCALL`, the Linux
kernel's `syscall` instruction layout — and a `ConventionKind` enum
that flags it as the syscall variant. The recovery pass picks the
syscall reading on functions whose body contains a `syscall` opaque
op, leaving the SysV / MsX64 rankings untouched on every existing
fixture. SysV variadic call sites surface as an auxiliary counter
on the inferred signature rather than as a fourth convention.

- **`SYSV_AMD64_SYSCALL` (new constant).** `int_arg_registers =
  [rdi, rsi, rdx, r10, r8, r9]`; everything else matches
  `SYSV_AMD64` (return register, stack-arg layout, callee-saved
  set). `kind: ConventionKind::Syscall` flags the entry; the
  module's curated list `X86_64_CONVENTIONS` keeps SysV / MsX64
  at the head so tie-breaks still resolve to user-space when no
  syscall opcode is present.
- **`ConventionKind` (new enum).** `Normal` covers regular
  user-space calls; `Syscall` flags the kernel `syscall` ABI. The
  enum is the discriminator the recovery pass's scoring rule
  keys off — no name-string matching.
- **Syscall scoring rule (recovery side).** `score_candidate`
  counts `Opaque { mnemonic: "syscall" }` ops in the SSA:
    - **+0.20** to a `Syscall`-kind candidate when at least one
      `syscall` op is present (the prefix bonus alone, capped at
      ~0.30 for a 6/6 prefix match, is not enough to flip the
      rcx-vs-r10 explanation against SysV; this boost is).
    - **-0.05** to `Normal` candidates when a `syscall` op is
      present (the user-space reading is the less specific
      explanation, but never out of the ranking).
    - **-0.30** to a `Syscall`-kind candidate when no `syscall`
      op is observed — ensures the kernel reading never outranks
      SysV / MsX64 on a regular function.
- **Variadic call-site detection.** `detect_variadic_call_sites`
  walks each block and counts SSA `Call` ops whose immediately
  preceding op is `Move { dst = rax, src = Const(_) }` — the
  SysV variadic call-site shape. The count surfaces on
  `InferredSignature::variadic_call_sites` so downstream
  signature recovery can promote a hint or `ApiSignature` to its
  variadic shape *without* introducing a separate convention.
  The count populates only for `Normal` candidates whose
  return register is `rax` (the same register that holds the
  vector-arg count on SysV); other candidates report 0.
- **New ELF corpus fixture.** `tests/fixtures/syscall-hello-x86_64`
  is a small PIE that issues `sys_write(2)` from `main` via
  inline asm — GCC at `-Os` inlines the wrapper, so the `syscall`
  opcode lands inside `main` itself. The new golden
  `syscall-hello-elf-o1-c` records the convention pass picking
  `sysv-amd64-syscall` on `main` (score 0.75) while every other
  function in the binary keeps a user-space reading.
- **Tests.** Three new knowledge tests cover the `SYSV_AMD64_SYSCALL`
  arg-register prefix, the `Normal` default for SysV / MsX64, and
  the `X86_64_CONVENTIONS` ordering invariant. Four new recovery
  tests cover: syscall-present picks the syscall convention;
  syscall-absent drops it below SysV; the variadic counter fires
  on the SysV-style `mov rax, const; call` pattern; and a bare
  `call` (no rax-const preamble) doesn't.

#### Limitations carried forward

- **Variadic call-site detection is single-block.** A variadic
  call whose `mov rax, <count>` is in a different basic block
  than the matching `call` won't be picked up. Cross-block
  detection needs the SSA value-numbering pass to surface
  constant-folded `rax` reaching defs at the call site — a
  follow-up once the SSA constant lattice is in place.
- **Syscall convention picks per function only.** A single
  function that mixes user-space `call`s and direct `syscall`
  ops still gets the syscall reading on its parameters; the
  alternative — modelling the syscall as a distinct call-site
  property like variadic — is shelved alongside the cross-block
  variadic work.
- **No syscall-number → kernel-signature lookup.** The fixture's
  `main` calls `sys_write(2)` but the recovery report only
  surfaces "this function uses the syscall convention". Mapping
  `rax = 1` to the `sys_write` signature lives behind the
  symbol-driven hint catalogue that future B3 batches will
  layer on top.

Closes: B3.13, FR-13 (calling-convention recovery for the
syscall and variadic call-site shapes).
Touches: I-3 (every match still carries `Source::Derived`
confidence), NFR-9 (scoring stays pure / deterministic;
opaque-op count and variadic-call-site count are pure SSA
iterations), I-6 (syscall convention drops to last when no
`syscall` opcode is present rather than synthesise the layout).

#### B3.14 — Subreg-aliasing correctness in bridge (2026-06-05)

Refines `dac-lift::bridge`'s register-write rules so the x86-64
sub-register model is encoded faithfully rather than collapsed
onto a single "parent := value" Move. The bridge already
canonicalises every sub-register read and write to the 64-bit
parent's `VariableId`; B3.14 keeps that identity choice but
varies the *operation shape* by destination width so the
unmodified upper bits stay live in the SSA use-chain (I-2
evidence preserved per sub-write).

- **64-bit destination** (`rax`, `r8`, …) and **32-bit
  destination** (`eax`, `r8d`, …): unchanged — plain `Move` /
  `<binop>` straight into the parent. x86-64's implicit
  zero-extension on 32-bit writes is already encoded by the
  canonical parent identity (the value-side already represents
  a 32-bit-shaped result), so no prior-parent read is needed.
- **16-bit destination** (`ax`, `r8w`, …) and **8-bit
  destination** (`al`, `r8b`, …): three-op blend
  `(parent_old & ~mask) | (value & mask)` with `mask = 0xFFFF`
  for 16-bit and `mask = 0xFF` for 8-bit. The blend's high-half
  `And` reads the parent's prior SSA name, keeping every
  earlier definition that touched those bits live in the
  use-chain.
- **Read-modify-write** (`add al, 1`, `neg ax`, …): the binop /
  unop materialises into a synthetic temp, then the blend
  copies its low bits into the parent. Four ops total for an
  8/16-bit r-m-w (binop / unop into temp + three blend ops).
- **Helpers.** New `write_value_to_register` (plain Move path),
  `write_kind_to_register` (r-m-w path), and
  `blend_subreg_into_parent` (mask + And + Or sequence)
  centralise the dispatch. The free function
  `needs_subreg_blend(op_width, parent_width)` gates the blend
  to widths 8 and 16 under a strictly wider parent — 32-into-64
  takes the implicit-zero-extension path.
- **High-byte registers.** `ah` / `bh` / `ch` / `dh` aren't
  exposed by the x86-64 `RegisterFile` (only `al`–`r15b` are),
  so any stray operand referencing one degrades to a plain
  Move. They're still encodable on i386, but the bridge is
  x86-64-only (return register / call-arg registers are
  hardcoded SysV AMD64), so the correct shifted-mask handling
  is a follow-up alongside the architecture parameterisation.
- **Tests.** Six new bridge unit tests cover the predicate plus
  the three width classes (32-bit Move stays single-op; 16-bit
  / 8-bit Move blend; 8-bit binop and 8-bit unop materialise
  then blend) plus a use-chain test asserting that a prior
  `mov rax, 7` reaches the high-half mask of a following
  `mov al, 0` blend — the SSA edge that proves the lossy rule
  is gone. `cargo xtask ci` reports 565 tests passing (was 558
  at B3.13; +7 from the new tests including the predicate
  smoke test).

#### Limitations carried forward

- **Read-side imprecision.** A read of `eax` / `ax` / `al`
  still surfaces as a full read of the canonical `rax`
  variable; no truncation op is emitted on the read side.
  Operations whose semantics depend on the masked sub-register
  value (`cmp al, 0` against a `rax` whose upper 56 bits are
  non-zero) will read stale upper bits. Fixing this needs the
  same blend treatment in `translate_read` — deferred until a
  corpus binary actually exercises a wrong-result case.
- **No goldens drifted.** The existing ELF / PE corpus
  exercises 32-bit register writes (`mov eax, X` /
  `xor ebp, ebp`) heavily, plus byte memory stores
  (`mov byte ptr [...], 1` — handled by the Store path), but
  no 8/16-bit *register* destinations. So `xtask golden`
  reports 32/32 unchanged across all 12 cases. The "visibly
  tighter use-chains" claim from the PLAN entry is therefore
  forward-looking — it will fire the next time a fixture lands
  with byte-register r-m-w in it (`memchr`-shaped scanners,
  `strcasecmp`-shaped comparisons, etc.).
- **Stack-pointer width on `pop ax`.** `Pop` still hardcodes a
  64-bit `Load { width: 8 }` and an 8-byte `rsp` adjustment.
  The bridge now blends a 16-bit slice of that load into the
  parent, which masks the value-side error, but the
  stack-pointer increment remains too large. That's a B3.8
  scope item not in B3.14's PLAN entry; left for follow-up.

Closes: B3.14.
Touches: I-2 (every sub-register write now produces a `RawOp`
sequence whose use-chain preserves the unmodified bits, instead
of the previous "parent := value" overwrite that bled
provenance), NFR-9 (the dispatch is data-driven on operand
width — same input still produces the same `RawFunction`),
I-6 (unknown widths and AT&T high-byte registers degrade to a
plain Move rather than panicking or emitting a mis-aligned
mask).

#### B3.15 — Typed-local refinement (2026-06-05)

The B3.10 lowering pass declared every pre-declared C local
with the SSA variable's width-based shape (`int<bits>_t`) and
only consulted `dac_recovery::TypeMap` for the function's
parameters and return type. As a consequence, locals that the
lattice promoted to a pointer landed as `int64_t` and the
pointer / int boundary was invisible in the recovered source.
B3.15 retypes locals from the lattice directly while pairing
the declaration with explicit define / use casts so the
round-trip compile gate stays clean even though the lifter's
sub-register arithmetic still mixes pointer- and int-typed
values.

##### Declared local types

`dac-backend-c::lower::declared_ctype` is the new entry point.
For every `ValueDef` in the SSA function it asks the
[`Recovered`] view for a `TypeMap` entry; when one exists and
`map_ir_type` can render it, the local is declared with the
mapped C type. When no lattice entry exists, or the mapping
yields a composite the C AST cannot represent (`Struct`,
`Array`, conflicting widths), the width-based shape from
B3.10 is used as the fallback. `width_ctype` is the renamed,
unchanged width-fallback helper; `LowerCtx` now tracks a
`Vec<ValueTypePair>` of `(declared, width)` indexed by
`ValueId` so every cast site reads both shapes in O(1).

##### Define-side casts (`ptr_cast_if_needed`)

A new `ptr_cast_if_needed(source, target, expr)` helper fires
only at the *int ↔ pointer* boundary. Same-kind conversions
(int → int of any width, ptr → ptr of the same pointee) emit
no cast wrapper, which keeps the surface free of the
`(int8_t)(0LL)` noise that the equivalent `cast_if_needed`
would have produced for width-typed narrow-int locals.

Define sites that switched to the new helper:

- `lower_locals`: the zero initialiser of every non-parameter
  local. Pointer-typed locals render as `void * v = ((void
  *)(0LL));`; width-typed integer locals stay as `int<w>_t v
  = 0LL;`, byte-for-byte identical to B3.10.
- `lower_instr` Assign arm: the value of every `Stmt::Assign`
  is wrapped in `ptr_cast_if_needed(width, declared, expr)`
  so an arithmetic RHS lands cleanly in a pointer-typed
  local (`v3 = ((void *)((v2 + 16336LL)));`) but a
  width-typed local takes the RHS verbatim
  (`v3 = (v2 + 16336LL);`).
- `lower_value_op::Move`: the source operand is projected to
  the destination's width-typed shape, again at the int ↔ ptr
  boundary only. Identity moves (`v9 = v6;` where both are
  width-typed) stay raw.

Parameter init and `Stmt::Return` keep the existing B3.10
`cast_if_needed` shape so the int-width-narrowing surface the
hint pipeline depends on (`int64_t v8 = ((int64_t)(arg0));`,
`return ((int32_t)(v12));`) survives unchanged. The B3.15
helper is additive — it does *not* widen B3.10's cast surface,
only adds pointer-aware casts at the new sites.

##### Use-side casts

`LowerCtx::lower_operand_for_use` casts every `Value` operand
back from `declared` to `width` (via `ptr_cast_if_needed`)
before it composes against another expression. Concretely:

- Arithmetic and comparison operators (`Add`, `Sub`, `Mul`,
  `And`, `Or`, `Xor`, `Shl`, `Shr`, `Compare`) read each
  operand through `lower_operand_for_use`, so a pointer-typed
  local in a byte-arithmetic context renders as
  `(((int64_t)(v4)) + 8LL)` — byte arithmetic stays byte
  arithmetic even though the local is declared `void *`,
  preserving the binary's actual semantics rather than
  silently switching to C pointer-element scaling.
- Unary operators (`Neg`, `Not`) read their source through the
  same helper.
- `SsaOp::Load` / `SsaOp::Store` read their address through
  the helper, so the existing `(int64_t *)(...)` cast in the
  emitter sees the width-typed shape and does not double up.
- `Call` arguments and `If` / `While` / `DoWhile` / `Switch`
  conditions are also routed through the helper, so a
  pointer-typed scrutinee or call argument decays to its
  width-typed shape at the use site.

`Stmt::Return` uses a parallel `lower_operand_with_type`
helper that returns `(expr, operand_declared_type)`. The
returned operand is then cast through the B3.10
`cast_if_needed(operand_declared, return_type, ...)` shape —
the B3.10 contract is preserved, but the cast source is the
operand's *declared* type instead of the hard-coded
`int64_t`. A pointer-typed return operand against a
pointer-typed `return_type` is rendered without a cast
wrapper; the B3.10 narrowing surface remains intact.

##### Width fallback path

When `Recovered::types` is `None` (orchestrator stub paths,
hand-built fixtures, the B2.8 round-trip tests) every value's
declared type equals its width-typed shape, so every cast
helper returns the raw expression and the rendered output is
byte-for-byte identical to B3.14. The 12-fixture round-trip
test corpus continues to pass without any golden refresh.

##### Tests

Seven new unit tests in `dac-backend-c::lower::tests` cover
the contract:

- `b3_15_pointer_local_declared_with_pointer_ctype` — a
  TypeMap with `v0: Ptr(Unknown)` promotes the matching local
  to `void *`; values absent from the TypeMap keep the
  width-based `int64_t`.
- `b3_15_pointer_local_zero_init_gets_cast` — pointer-typed
  locals get `((void *)(0LL))`; width-typed locals keep the
  raw `0LL` literal.
- `b3_15_pointer_assignment_rhs_gets_cast` — `v0 = (2LL +
  16336LL);` against a pointer-typed `v0` becomes `v0 =
  ((void *)(...));`.
- `b3_15_pointer_operand_use_gets_cast_back` — reading a
  pointer-typed operand in a Load address yields
  `((int64_t)(v0))` so byte arithmetic stays correct.
- `b3_15_no_typemap_keeps_width_based_shape` — backward
  compatibility: without a TypeMap the B2.8 / B3.10 output
  shape is preserved exactly.
- `b3_15_return_pointer_local_matches_pointer_return_type` —
  identity-cast pair: declared == return_type → no wrapper.
- `b3_15_move_with_pointer_dst_wraps_const_init` — the Move
  op preserves the lattice surface even when the source is a
  constant.

The `dac-backend-c` round-trip gate continues to pass (12 / 12
fixtures compile through the system C compiler). The corpus
golden set picked up four refreshed sources where the lattice
genuinely promotes a local to pointer:
`hello-elf-o1-c/source.c`, `hello-elf-o1-c-hints/source.c`,
`hello-pe-o1-c/source.c`, `syscall-hello-elf-o1-c/source.c`.
Each refreshed golden shows the same pattern: locals the
lattice flags as `Ptr(Unknown)` now declare as `void *`,
their initialisers and define sites carry `((void *)(...))`,
and their use sites carry `((int64_t)(...))` for the
arithmetic paths. The eight remaining goldens (`hello-elf-o1-ir`
and the unaffected fixtures that did not exercise the
lattice's pointer rule) remain byte-for-byte identical.

572 workspace tests pass (+7 vs B3.14's 565); 32 goldens
across 12 cases match; fmt + clippy clean.

##### Limitations carried forward

- **Pointer pointee detail.** The C AST renders `Ptr(Unknown)`
  as `void *`. When the lattice has a concrete pointee
  (`Ptr(Int { 32, Signed })` etc.) the emitter would render
  `int32_t *`, but the seeding rules in
  `dac_recovery::types` currently never produce a non-
  `Unknown` pointee on the corpus. Refining the pointee
  inside the recovery pass is its own follow-up; the C
  backend already supports the round-trip.
- **Read-side `bridge` width imprecision.** The B3.14
  follow-up note still applies — the bridge reads parent
  registers at full width before sub-register operations
  consume them. B3.15 only touches the C lowering pass; the
  sub-register *read* model is unchanged.
- **Stack-anchored structs (B3.10 deferral) still surface as
  raw `*(int*)(rsp + k)` accesses.** B3.16 will lower these
  through a proper `Stmt::Decl` for composite locals.

Touches:
`crates/dac-backend-c/src/lower.rs`,
`tests/golden/hello-elf-o1-c/source.c`,
`tests/golden/hello-elf-o1-c-hints/source.c`,
`tests/golden/hello-pe-o1-c/source.c`,
`tests/golden/syscall-hello-elf-o1-c/source.c`,
`CHANGELOG.md`,
`PLAN.md`.

Closes: B3.15.
Touches: FR-13 (parameter / return-type cast pair now reads
the operand's *declared* lattice type rather than the
hard-coded `int64_t`), FR-14 (lattice-driven C type spelling
is visible at every pre-declared local, not only at the
signature), I-2 (each cast wraps an `Expr` whose underlying
operand still resolves through the SSA value's evidence node;
no provenance is dropped), NFR-9 (the new helpers are pure
data-driven functions over the SSA value index — same input
still produces the same C AST byte-for-byte), I-6 (composite
or unrepresentable lattice types degrade to the width-based
shape rather than producing an invalid C type spelling).

#### B3.16 — Struct typedef surface (2026-06-05)

The B3.10 lowering pass surfaced every recovered pointer-
anchored field as a `/* recovered field: base=… offset=…
field=… */` comment but kept the access spelled as
`*((<ty> *)(base + 0xN))` because the C AST had no
translation-unit-level typedef shape. B3.16 grows the AST a
`StructDecl` item, walks `dac_recovery::RecoveredStructs` at
lowering time to mint one typedef per promoted layout, and
rewrites the matching `Load` / `Store` accesses into
`base->field_<hex>` form.

##### C AST

`dac-backend-c::ast` gains three new shapes:

- `Item::StructDecl(StructDecl)` — translation-unit-level
  `typedef struct __attribute__((packed)) { … } NAME;`. The
  packed attribute is load-bearing: the per-field offsets
  come straight from the recovery pass, and a normal aligned
  layout would silently re-shape the byte positions the
  recovery observed.
- `StructDecl { name, fields, leading_comment }` and
  `StructField { name, ty }` — the typedef body. Field order
  is recovery's source order; the emitter renders each entry
  on its own line.
- `CType::Named(String)` — a reference to a typedef'd name.
  Used by the lowering pass to declare pointer-anchored
  locals as `<typedef> *`. Includes `CType::Array { element,
  count }` for the leading and inter-field padding members
  the typedef body needs.

The new variants land with exhaustive-match coverage tests
so any future variant addition breaks the emit / lowering
stages at compile time (I-6 carries forward from B2.8).

##### Lowering pass

`lower_function` is now a thin wrapper around the new
`lower_function_with_typedefs`. The richer entry point
returns a `LoweredFunction { function, struct_decls }`
where `struct_decls` is the deterministic, name-sorted list
of typedefs the function references. The translation-unit-
level entry point `lower_unit` aggregates `struct_decls`
across every lowered function, deduplicates by typedef name,
and prepends the resulting `Item::StructDecl`s so each
typedef is in scope at every function that references it.

The per-function plumbing:

- `build_struct_typedef_table` walks
  `RecoveredStructs::pointer_structs` and synthesises one
  `StructTypedef { decl, layout }` per layout whose
  *effective* size (`last.offset + last.width`) is within
  `MAX_PROMOTED_STRUCT_SIZE` (4 KiB). Layouts above the cap
  — almost always absolute-address false positives such as
  `Add(base, Const(0x140008008))` — are dropped so the
  comment-only surface still covers them without producing
  a multi-MB typedef that breaks the round-trip compile
  gate.
- `struct_typedef_name(function_address, base)` produces
  the deterministic name `S_<funcaddr_hex>_v<base_id>_t`.
  Per-function naming lets two functions that share the
  same `ValueId` for different bases land at distinct
  typedef names; `lower_unit`'s name-keyed dedup collapses
  the duplicates an inlined helper would produce.
- `build_struct_fields(layout)` emits padding (`uint8_t
  __pad_<from>_<to>[N];`) between adjacent recovered fields
  so the byte position the recovery observed survives the
  round trip. The first field's leading padding starts at
  byte 0 — `field_60` at offset 0x60 sits at byte 0x60 of
  the typedef.

`declared_ctype` (B3.15) gains a typedef short-circuit: a
base value present in the per-function table is declared as
`<typedef> *`, overriding any lattice mapping for that
value. The B3.15 cast pair (`ptr_cast_if_needed`) continues
to bridge the `int64_t` body arithmetic against the new
pointer-to-typedef declarations — every define-site cast
becomes `((S_…_t *)(<rhs>))` and every use-site cast becomes
`((int64_t)(<local>))`.

##### Per-access rewriting

`LowerCtx::match_promoted_struct_field` is the new matcher,
gated on the per-function typedef table rather than
`recovered.structs` directly. When a `Load` / `Store`
address decomposes to `Add(base, Const(offset))` and the
base appears in the typedef table at a field with the
matching offset:

- `Stmt::Store` becomes `Stmt::FieldStore { base, field,
  arrow: true, value }`, rendering as
  `v5->field_60 = v12;`.
- `SsaOp::Load`'s `lower_value_op` arm becomes
  `Expr::Field { base, field, arrow: true }`, rendering as
  `v26 = v5->field_28;`.

When the base does *not* appear in the typedef table (size
cap fired, or the lattice never produced the layout), the
B3.10 comment-on-top-of-Store / bare-Load surface is
unchanged. The matcher is layered, not exclusive — bases
the size cap skipped still receive the `/* recovered field:
… */` provenance comment.

##### CLI wiring

`render_c_unit` in `dac-cli/src/main.rs` is rewired to read
each `LoweredFunction`'s `struct_decls`, accumulate them
across the function set into a name-keyed `BTreeMap`, and
prepend the resulting `CItem::StructDecl` items to the
function items. Each typedef appears once in the
translation unit regardless of how many functions
reference it (the dedup mirrors `lower_unit`'s but lives in
the CLI so the per-function `LoweredFunction` shape stays
small and serialisable).

##### Corpus impact

`tests/golden/hello-pe-o1-c/source.c` now opens with 48
`typedef struct __attribute__((packed)) { … } S_<addr>_v<id>_t;`
items and 91 `<base>-><field_hex>` accesses scattered
through the recovered function bodies — both well above
B3.16's "at least one" success criterion. The ELF goldens
(`hello-elf-o1-c`, `hello-elf-o1-c-hints`,
`syscall-hello-elf-o1-c`) recovery only produced
absolute-address candidates, which the size cap correctly
declined, so those goldens carry zero typedefs and the
matching `/* recovered field: ... */` comments survive
verbatim — the surface stays B3.10-shaped where there is no
real layout to promote.

##### Tests

`dac-backend-c::ast::tests` — three new exhaustive-match
guards: `item_variants_are_exhaustively_matchable`,
`ctype_variants_are_exhaustively_matchable`, and the
existing expression / statement guards still pin every
variant.

`dac-backend-c::emit::tests` — two new tests:
`struct_decl_renders_typedef_with_packed_attribute` pins
the full rendered shape (packed attribute, padded fields,
trailing typedef name) and
`named_ctype_renders_as_typedef_name` pins that
`Ptr(Named("S_v5_t"))` renders as `S_v5_t *`.

`dac-backend-c::lower::tests` — four new tests:
- `b3_16_pointer_struct_promotes_to_typedef_and_field_access`
  exercises the end-to-end path: a layout at offsets 0/8
  produces a `S_3000_v0_t` typedef with two `field_*` members,
  the base local is declared as `S_3000_v0_t *`, and a
  `Load(v0 + 8)` lowers to `v0->field_8`.
- `b3_16_oversized_struct_declines_to_typedef` builds a
  layout whose first field is just past
  `MAX_PROMOTED_STRUCT_SIZE` and asserts the typedef is
  dropped and the base stays width-typed.
- `b3_16_no_typemap_no_structs_keeps_b3_10_surface` is the
  backward-compat guard: no recovery facts → no typedefs,
  no declared-type changes.
- `b3_16_struct_field_padding_lays_out_to_observed_offset`
  builds a layout whose first field is at byte 0x10 and
  asserts the padding member spans 0..0x10 so `field_10`
  lives at the recovery's observed byte position.

`cargo xtask ci` is green: 576 workspace tests pass
(was 572 at B3.15, +4 net), 32 goldens across 12 cases
match, fmt + clippy clean. The round-trip `cc` gate accepts
the new PE corpus output unchanged.

##### Limitations carried forward

- **Inferred field types are `int<width>_t`.** The
  recovered layout records each field's *access width* (8,
  4, 1 bytes) but the type lattice rarely reaches a
  pointer-typed field — even with `RecoveredStructs::ty`
  set per-field, the type pass leaves `Type::Unknown` on
  almost every field across the corpus. The typedef body
  uses the width fallback (`int_type_for_width`); a follow-
  up batch refining `dac_recovery::types`'s per-field
  inference will let the typedef carry pointer-typed and
  signed/unsigned-aware fields without changing the
  backend.
- **Stack-anchored structs still surface as
  `*(int*)(rsp + k)`.** B3.16 only promotes
  `RecoveredStructs::pointer_structs`. The stack-anchored
  variant needs a `Stmt::Decl` shape for composite locals
  (the AST currently models `Local::ty` as a scalar) and
  is on the B3 residue shelf alongside the union /
  nested-struct deferrals.
- **Field type recovery deferred.** Pointer-anchored
  layouts whose effective size exceeds 4 KiB (the
  absolute-address false-positive case the PE corpus
  exhibits at `Add(base, Const(0x140008008))`) keep the
  B3.10 comment-on-top-of-bare-Store surface. Tightening
  the recovery pass so it never produces these candidates
  is the B3.18 follow-up; B3.16's cap is the visible-
  degradation guard until that lands.
- **Struct hint application (B3.6 follow-up).** Now that
  the typedef surface exists, a `[[struct]]` hint can
  rename a recovered typedef and its fields. Wiring the
  hint through `apply_struct_hint` stays on the B3
  follow-up shelf — the substrate is in place.

Touches:
`crates/dac-backend-c/src/ast.rs`,
`crates/dac-backend-c/src/emit.rs`,
`crates/dac-backend-c/src/lower.rs`,
`crates/dac-backend-c/src/lib.rs`,
`crates/dac-cli/src/main.rs`,
`tests/golden/hello-pe-o1-c/source.c`,
`CHANGELOG.md`,
`PLAN.md`.

Closes: B3.16.
Touches: FR-17 (recovered struct layouts now surface as
real C `typedef` declarations and field accesses, not
comment markers), I-2 (each typedef carries the recovered
layout's confidence + provenance through its leading
comment; the underlying SSA evidence is unchanged), NFR-9
(typedef synthesis is a pure, deterministic walk over
`BTreeMap`-keyed recovery output; identical input still
yields identical C source), I-6 (absolute-address false
positives degrade visibly to the B3.10 comment-only
surface rather than producing a typedef the compile gate
would reject).

#### B3.17 — Switch-arm resolution (2026-06-05)

`dac-recovery::idioms::SwitchTableIdiom` records the
*shape* of a jump-table dispatch, and B3.10 lowered the
recognised idiom to a `Stmt::Switch` whose `arms` were
empty and whose `default` body wrapped an
`__builtin_unreachable()` carrying the original
`SemStmt::Unreachable`. The C printer surfaced that as
`switch (scrutinee) { default: { __builtin_unreachable(); } }`
with a `recovered switch table at block N (arm resolution
pending)` comment — recognised, but not yet usable.

B3.17 closes the loop. The post-pass that maps each
recognised idiom into a `SemStmt::Switch` now reads the
table out of the binary's `.rodata` (or `.rdata`), maps
each entry's target VA to a CFG block, mints
[`LabelId`](crates/dac-ir/src/sem.rs)s from the slot
immediately above the structurer's range, and writes
`case <const>: { goto L<n>; }` arms anchored against
those labels. A new label-marker post-pass (`append_orphan_labels`)
appends a `Stmt::Label` for every minted id at the
function-body tail — *outside* the structurer's recursive
walk — so the slots survive any subsequent arm-rewrite
pass.

#### What ships

- **`SwitchBound` enum.** `SwitchTableIdiom::bound` now
  encodes the `Ult` (`LessThan(n)`) vs `Ule` (`AtMost(n)`)
  distinction lost by B3.3's `Option<i64>` shape. The
  entry-resolution pass needs the inclusive-vs-exclusive
  distinction to read the right number of entries; the
  helper `SwitchBound::entry_count` returns `n` for
  `LessThan` and `n + 1` for `AtMost`.
- **`lookup_bound` recognises both branch polarities.**
  The B3.3 recogniser only fired when the dispatch sat on
  the `taken` edge of a guarding `Ult`/`Ule` compare. The
  PE corpus emits the inverse shape (`if (cmd > N) goto
  default; else goto dispatch`, where the dispatch is on
  the `not_taken` edge of a guarding `Ugt`/`Uge`); the
  recogniser now handles `(Ult, taken)`, `(Ule, taken)`,
  `(Ugt, not_taken)`, and `(Uge, not_taken)`. Signed
  compares stay rejected — `signed_lt_does_not_supply_bound`
  still holds, because a signed `<` against a positive
  limit could fire with a negative scrutinee that would
  index the table backwards (I-6).
- **Constant folder on the base leg.** PIC tables
  materialise the table base as a short chain of `lea`s
  (`Move(Const)` → `Add(base, Const)` → another `Move`),
  which B3.3 surfaced as `table_base_const: None`. A new
  `fold_constant_operand` helper walks up to four hops of
  `Move` / `Add` / `Sub` and collapses the chain to a
  single `Some(va)`. The fold is bounded so the SSA
  well-formedness invariant cannot deadlock the
  recogniser.
- **Iced RIP-relative addressing surfaces correctly.**
  `iced-x86` reports `Register::RIP` as the base for
  RIP-relative `lea` / `mov` operands *and* returns the
  already-resolved absolute target as the displacement.
  The x86 lifter previously emitted both, so the SSA
  carried `rip_param + abs_va` — semantically wrong and
  invisible to any downstream constant-folding pass. With
  `instr.is_ip_rel_memory_operand()` now driving a
  base-drop, the lifter emits `Move(Const(abs_va))` and
  the switch-table base lands in the recovery's
  `value_const` table without the fold even running.
- **`resolve_switch_entries` (dac-recovery).** Given an
  idiom with a recovered base and bound, the helper reads
  the bytes at `base + i * stride` for
  `i ∈ 0..entry_count.min(MAX_SWITCH_ENTRIES)` (4 KiB-style
  cap of 256 entries) and decodes them into target VAs:
  - **Absolute pointer table** (`width == stride == 8`):
    the entry *is* the target VA.
  - **`int32_t`-relative table** (`width == stride == 4`):
    the target is `(base as i64) + (entry as i32) as i64`
    — the GCC PIE / clang PIC / MSVC `/GS-` shape.
  Anything else (unsupported width/stride combo, base or
  bound missing, section without file backing, section-
  boundary overrun) returns an empty vector. Honest
  degradation, never an error (I-6).
- **`lower_switch_idioms` (dac-cli).** The post-pass now
  takes the CFG + `BinaryModel` + raw bytes, calls
  `resolve_switch_entries`, maps each target VA to a CFG
  block via an `address → SsaBlockId` index, mints one
  `LabelId` per *unique* target block (starting from
  `next_label_id(&sem.body)` so the slot can't collide
  with structurer-allocated ids or pre-existing goto
  targets), and produces a `SwitchArm { value, body:
  [Goto { target: lid }] }` for every resolved entry.
  Idioms whose entries can't be resolved still get a
  switch record — with empty arms — so the B3.10
  comment-marked empty-arms surface is preserved
  unchanged for the corpus shapes the resolver can't yet
  decode (e.g. signed-Gt-bounded dispatches).
- **Label anchoring at the function-body tail.** A new
  `append_orphan_labels` walk pushes `Stmt::Label` markers
  at `sem.body.stmts`' tail for every minted label id that
  no `Stmt::Label` elsewhere in the tree already anchors.
  Goto targets do not count as anchors — the original
  structurer-style `walk_label_ids` helper that conflated
  them was split, with `walk_anchored_labels` driving the
  orphan-anchor decision. This matches the PLAN.md
  requirement: "Anchor labels outside the structurer's
  recursive walk so the label slots survive arm
  rewriting."

#### Corpus impact

- The PE corpus (`hello-pe-o1-c`) has three recognised
  switch tables. Two now lower with populated arms:
  - `__mingw_SEH_error_handler` (0x140001de0) emits
    `case 5LL: { goto L3; }`, `case 9LL: { goto L4; }`,
    and anchors `L3:; L4:;` at the function tail.
  - `_FX_init_default` (0x140001f80) emits the identical
    `case 5LL` / `case 9LL` shape against its own
    function-local label ids.
- The third (`_matherr`, 0x140001730) keeps the B3.10
  empty-arms surface: the guarding compare is signed
  `Gt(v7, 6)` rather than `Ugt`, which the strict
  `signed_lt_does_not_supply_bound` invariant continues
  to reject (signed bounds are a known follow-up).
- ELF goldens (`hello-elf-o1-c`, `syscall-hello-elf-o1-c`)
  show no switch tables; the recovery is unaffected
  there. They *do* drift on the RIP-relative lifter
  change — listing rows now render
  `mov rax, [<abs_va>]` rather than `mov rax,
  [rip + <abs_va>]`, and per-value type recovery picks up
  the constants the SSA no longer hides behind a phantom
  `rip` parameter. The drift is uniform across both
  corpora and improves the surface (fewer `void *` casts
  for absolute-address loads).

#### Tests

- `dac-recovery::idioms::tests::fold_walks_short_add_chain_to_recover_table_base` — proves the folder catches the
  two-hop PIC base shape that B3.3 missed.
- `dac-recovery::idioms::tests::resolve_entries_reads_absolute_pointer_table` /
  `_reads_int32_relative_table` /
  `_at_most_reads_inclusive_count` /
  `_stops_at_section_boundary` /
  `_without_bound_returns_empty` /
  `_without_base_returns_empty` /
  `_rejects_unknown_encoding` /
  `_caps_at_max_switch_entries` — pin the resolver's
  contract for both encodings, both bound polarities, and
  every degradation path.
- `dac-recovery::idioms::tests::ule_compare_also_supplies_bound` / `predecessor_compare_supplies_upper_bound` updated
  to assert the new `SwitchBound::AtMost(n)` /
  `SwitchBound::LessThan(n)` variants and that
  `entry_count` returns `n + 1` for `AtMost`, `n` for
  `LessThan`.
- `dac-arch-x86::lifter::tests::lifts_rip_relative_lea_as_constant_displacement` — guards the RIP-relative
  base-drop so future iced upgrades can't regress the
  recovery substrate.
- `dac-cli::lift::tests::next_label_id_picks_one_past_the_highest_in_use` /
  `_on_empty_body_starts_at_zero` /
  `_counts_goto_targets` /
  `walk_anchored_labels_ignores_goto_targets` /
  `append_orphan_labels_anchors_each_switch_label_at_body_tail` — pin the label-allocation and
  body-tail anchoring contracts the lowering pass relies on.

#### Limitations carried forward

- **Signed-Gt-bounded dispatches.** The third PE corpus
  switch (`_matherr`) is guarded by a signed `Gt` rather
  than `Ugt`; the bound recogniser rejects it. Promoting
  signed compares to bounds requires arguing the
  scrutinee is non-negative — that's a Type-lattice
  upgrade (B3.18+) rather than an idiom-pass change.
- **Tables without a recovered bound.** Without a bound
  the resolver returns an empty entry list to avoid
  reading past the table into unrelated `.rodata`. A
  future batch could allow CFG-bounded fallback (stop
  reading at the first entry whose VA doesn't map to a
  CFG block within the same function) — the
  block-address index is already built; only the policy
  call is missing.
- **PE absolute-pointer tables with `.reloc` rebasing.**
  The resolver reads file bytes as-is, which gives the
  correct final VA for the corpus binaries (statically
  linked, no runtime relocation) but would mis-decode a
  table that ships with `IMAGE_REL_BASED_DIR64` entries
  the PE loader patches at runtime. Walking
  `BinaryModel::relocations` to apply the addends lands
  with the broader PE PIC handling.
- **Default-arm goto.** The `default` body still
  preserves the B3.10 `Unreachable` shape rather than
  pointing at a recovered "fall-through to next block"
  target. The structurer's region analysis would need to
  surface the dispatch-block's structural successor for
  this — a fold of `Stmt::Unreachable`-into-`Stmt::Goto`
  is the natural follow-up.

Closes: B3.17.
Touches: FR-18 (per-arm case-to-goto shape replaces the
B3.10 empty-arms surface for compiler-emitted jump tables
on x86-64), I-2 (every minted label carries the source
SSA block id, so the evidence graph can trace a case
label back to the original CFG block whose address fed
the switch entry), NFR-9 (resolution is a pure walk over
`BTreeMap`-keyed recovery output; identical idiom +
identical binary in ⇒ identical arm vector + label-id
allocation out), I-6 (every degradation path — bounded
without base, unsupported encoding, section overrun, VA
that doesn't map to a CFG block — falls back honestly to
the B3.10 surface rather than inventing arms).

#### B3.18 — SSA value-source bookkeeping after CSE (2026-06-05)

The trivial local CSE pass in
`dac_analysis::ssa::local_value_number` drops redundant
instructions from `block.instructions` but, before B3.18,
left every surviving dst's
`ValueDef.source = ValueSource::Instruction { index }`
unchanged. So a value defined at original index 2 that
ended up at compacted index 1 still claimed index 2 — the
backing op was one slot to the left of where the
value-source said it was. Any downstream pass that resolved
a `ValueId` to its defining `SsaOp` through the index path
(struct-field decomposition, idiom matching, the C
backend's operand printer) either landed on the wrong op or
fell off the end of the list, depending on how many
redundancies the block had.

The PE corpus surfaced this in B3.10 as an over-bound index
into `block.instructions`. B3.10 patched the visible site —
`dac_recovery::structs::lookup_def_op` — with an explicit
`get()` guard that turned the panic into a silent `None`.
That was the correct stopgap (the struct-field pass
gracefully degrades to the self-base shape on `None`), but
it also masked every other consumer of `ValueSource` that
*didn't* guard: they continued to read the wrong op.

B3.18 fixes the root cause:

- **Reindex on compaction.** After
  `local_value_number` finishes draining a block's
  instructions into the `kept: Vec<SsaInstruction>` list,
  it walks `kept` in order and rewrites each surviving
  `dst`'s `ValueDef.source` to
  `ValueSource::Instruction { block, index: new_index }`.
  The block index is unchanged (CSE never moves a value
  between blocks); only the slot index is rewritten.
  Phi-sourced and parameter-sourced values are untouched —
  CSE does not touch the phi list.
- **Promote the defensive guard.** With the invariant
  restored, `dac_recovery::structs::lookup_def_op`'s
  `?`-on-`get()` becomes a `debug_assert!` plus direct
  indexing, matching the style already used in
  `dac_recovery::idioms::lookup_def_op`. The assertion
  message names B3.18 so a future regression points at the
  CSE reindex rather than at the consumer.

The fix is small (one rewrite loop) but the effect on the
PE corpus is visible. Before B3.18,
`hello-pe-o1-c/source.c` had a single
`/* base: v17 */` struct that compressed three real fields
into a 60-byte opaque `__pad_0_3c` because every field
walk-back beyond the first hit the stale-index
degradation. After B3.18, that struct grows from 36 bytes
to 64 bytes with a recovered `int64_t field_20`, and two
new structs land at `/* base: v1 */` (16 bytes,
`field_20` + `field_28`) and `/* base: v1 */` (48 bytes,
`field_20` + `field_38` + `field_4c`).
`S_140002860_v7_t` also gains `field_38` and `field_78`,
growing from 88 to 96 bytes. Every gained field is a real
struct-recovery hit that the stale-index path was throwing
away.

### Tests added

- `dac_analysis::ssa::tests::ssa_07b_cse_reindexes_surviving_value_sources`
  constructs a 3-instruction block where CSE drops the
  non-last redundancy (`t1 = a + b` between
  `t0 = a + b` and `t2 = t0 + 1`). The test asserts both
  that the surviving `t2` lands at compacted index 1 and
  that `ssa.value(t2).source` reports
  `ValueSource::Instruction { block: 0, index: 1 }` rather
  than the pre-compaction `index: 2`. The walk-back used
  by downstream passes is exercised by the bound check at
  the end of the test, mirroring how
  `dac_recovery::structs::lookup_def_op` reaches the op.

### Limitations

- **CSE-only invariant.** The reindex only runs inside
  `local_value_number`. No other pass currently mutates
  `block.instructions` after SSA construction — `grep` of
  the workspace confirms it. If a later batch adds an
  instruction-removing pass (dead-store elimination,
  load-forwarding, a cross-block CSE) it has to adopt the
  same reindex contract. The `debug_assert!` in
  `lookup_def_op` catches a regression in dev builds; a
  release build still degrades silently to the
  "self-base" shape that B3.10 was already doing.

Closes: B3.18.
Touches: I-2 (`ValueSource::Instruction { block, index }`
once again denotes the *current* defining op for every
live value, not a stale pre-CSE slot — the evidence chain
from a struct-field read back to its base computation is
unbroken), NFR-9 (the rewrite is a pure pass over
`kept` in source order with no hashing or non-determinism
beyond what was already in `local_value_number`; same
SsaFunction in ⇒ same SsaFunction out), I-6 (the
`debug_assert!` makes a future bookkeeping regression
loud in dev while preserving the B3.10 graceful-degrade
shape in release).

#### B3.19 — Hint provenance in annotations (2026-06-05)

B3.6 landed user-hint application — `[[function]]`
entries retype arguments and the return slot in the
recovered TypeMap, and `rename` overrides the emitted
identifier. The hint's `EvidenceNode::UserHint` ID was
already minted into the same `EvidenceGraph` the
annotation channel walks (the CLI's `register_hints`
runs before `AnnotationDoc::build`), but the
`.annot.json` sidecar's `name` and `return_type` facts
ignored it: an applied rename was reported with
`Source::Observed` and a "symbol-table entry" chain that
described the *pre-hint* state, and an applied return
override silently kept the
"default void / B3.6 pending" annotation.

B3.19 closes that loop. When `Hints::find_function`
matches a recovered function, the annotators reach into
the matched [`FunctionHint`] and override:

- **`annotate_name`**: if the hint has `rename = Some(_)`,
  the `FactAnnotation` reports the hinted identifier
  with `Source::UserHint` confidence at
  `USER_HINT_CONFIDENCE` (the same constant the
  TypeMap overlay uses, now `pub(crate)` so both call
  sites read from one place), an explanation citing
  the hint's source-file line, and an evidence chain
  anchored on the hint's `EvidenceNode::UserHint`
  node. The pre-hint symbol-table chain is not
  appended underneath — the hint *replaces* the
  observation; the annotation reflects that
  faithfully so a reader inspecting the sidecar sees
  the hint as the sole supporting evidence.
- **`annotate_return_type`**: if the hint has
  `return_ty = Some(_)`, the annotation reports the
  hinted type's `Display` form (e.g. `int32_t`) with
  `Source::UserHint` confidence, an explanation
  citing the hint's line, and the same single-node
  chain rooted at the hint's evidence ID.

When the matched hint has *no* override active (a hint
that only fired for `args`, or a registered-but-passive
entry), neither annotation changes — the name still
renders as the observed symbol and the return type
still renders as the B3.4 / B3.6 "void / pending"
placeholder. The match is consulted; the override is
gated.

The plumbing is one new `&Hints` parameter on
`AnnotationDoc::build` (and on the CLI's
`build_annotations_doc` wrapper). The CLI passes the
already-registered catalogue on the supported-arch
path and an empty `Hints::new()` on the
unsupported-arch path, mirroring how the latter handles
the `FunctionSet` and `EvidenceGraph`.

A small helper, `hint_evidence_chain`, falls back to a
synthesised single-entry `EvidenceRef` when the hint
was not registered against the graph — the test
harness exercises this corner so a regression in
`register_hints` would not silently strip the hint
citation from the sidecar.

### Tests added

- `function_hint_rename_pins_name_with_user_hint_source`
  asserts that a `[[function]]` rename hint at address
  `0x1000` produces `name.value == "user_main"`,
  `Source::UserHint`, confidence
  `USER_HINT_CONFIDENCE`, an explanation mentioning
  the hint, and an evidence chain that contains the
  hint's `UserHint` node and *no* `Bytes` / `IrNode`
  refs from the discarded symbol-table chain.
- `function_hint_return_pins_return_type_with_user_hint_source`
  asserts that a `return = "int32"` hint produces
  `return_type.value == "int32_t"`, `Source::UserHint`,
  the matching confidence, and a chain rooted on the
  `UserHint` evidence ref.
- `function_hint_without_overrides_does_not_alter_annotations`
  asserts that a registered hint with neither
  `rename` nor `return` set leaves both annotations
  on the deterministic-pipeline path (Observed name,
  Derived void return). Guards against a regression
  where the match itself would be mistaken for an
  override.
- `rendered_json_cites_user_hint_id_under_name`
  asserts that the rendered JSON sidecar carries
  `"kind": "user_hint"` and `"hint_id": 99` inside
  the name fact's evidence list so a tool consuming
  the `.annot.json` can cross-reference back to the
  hint catalogue without re-walking the evidence
  graph.

### Limitations

- **Return-type annotation only fires under a hint.**
  The non-hinted return annotation still reports
  `void / Derived` regardless of what the convention
  pass inferred. Wiring the inferred return into
  `annotate_return_type` is a separate concern (the
  `RecoveryFacts.convention.signature.return_register`
  + `types.values` lookup would need to flow through
  the annotation builder, which it currently does
  not). B3.19 is scoped to the *hint* side of the
  fact; the convention side stays on its
  pre-existing trajectory.
- **No retroactive citation of the `args` overlay.**
  A hint whose only effect is `args` (no `rename`,
  no `return`) currently leaves the annotation
  channel unchanged because the channel surfaces
  name and return-type per-function but does not yet
  surface argument lists. That arrives with the
  per-parameter annotation slot, which the B3.4
  module docstring already flags as pending.
- **Single per-function hint.** `find_function`
  returns the first match in source order; if a user
  file contains two hints that both match, only the
  first is cited. The hint loader does not flag
  duplicates today, so this is consistent with the
  rest of the hint pipeline.

Closes: B3.19.
Touches: I-2 (every name / return annotation that was
overridden by a hint now carries the hint's evidence
ID — the `.annot.json` sidecar's provenance closes the
loop from the emitted source back to the hint file),
I-3 (the `Confidence::Source` reported in the sidecar
matches the lattice the TypeMap overlay records, so
the annotation channel and the recovered-fact lattice
agree on which facts came from a user hint), FR-19,
FR-20.

#### B3.20 — Loop-induction and counter naming (2026-06-05)

B3.7 shipped the variable-naming pass with two
heuristics: API-context (`s` from `strlen`, `path` from
`open`) and string-literal slugs (`str_hello_world`).
The `;; naming:` report row carried those two channels
only. Loop-induction (`i` / `j` / `k`), counter
(`count`), and allocator-size (`size`) all sat on the
B3 follow-up shelf with the docstring noting "each
requires extra dataflow reasoning". B3.20 lands the
three deferrals.

The new heuristics walk natural-loop data from
`dac_analysis::loops::LoopForest`. `dac-analysis`
already depends on `dac-recovery` (for `Function` /
`FunctionSet` / `SourceMask`), so a direct dependency
the other way would close the cycle. B3.20 adds a
small POD — [`names::LoopInfo`] — that the CLI builds
from the forest before calling `recover_names`. The
recovery crate sees only the per-header `(depth,
back_edges)` tuple it needs, not the forest itself.
`LoopInfo::default()` disables the heuristic; the
B3.7-era tests use the default and stay green
unchanged.

- **Loop-induction (`InductionCounter`).** For every
  natural loop in the function's [`LoopInfo`], walk
  the header's phi list. A phi whose every back-edge
  incoming is `Add(phi.dst, 1)` (or `Add(1, phi.dst)`)
  is an induction variable. The first qualifying phi
  per loop earns `i` / `j` / `k` / `l` / `m` / `n` by
  nesting depth (depth 0 → `i`, depth 1 → `j`,
  …); anything past the table folds back to `i` and
  lets the disambiguator append `_1` / `_2` / `_3`.
  The pass classifies "first qualifying" by SSA-phi
  order — the lifter sorts phis by variable id, so
  the choice is deterministic across runs (NFR-9).
- **Counter (`Counter`).** A phi at *any* block (loop
  header or not) where at least one incoming is
  `Add(phi.dst, 1)` earns `count` — except when the
  induction pass already claimed the phi. The
  separate increment check is a loose "is any
  incoming the +=1" rather than the strict
  "every back-edge is +=1" the induction pass uses,
  so this catches sibling phis at a loop header
  (`for (int i=0, j=0; ; i++, j++)`) and counters in
  irreducible CFGs the natural-loop pass could not
  name (I-6 — degrade visibly but keep extracting
  facts).
- **Allocator-size (`AllocatorSize`).** A
  [`SsaOp::Call`] whose target resolves to `malloc` /
  `calloc` / `realloc` in the API catalogue gets its
  size-argument slots scanned: index 0 for `malloc`,
  index 1 for `realloc`, both 0 and 1 for `calloc`
  (the catalogue lists `(n, size)`). Each size-arg
  value whose defining op is an arithmetic op
  (`Add` / `Sub` / `Mul` / `Shl` / `Shr`) earns
  `size`. Bare register / constant / parameter loads
  carry no signal and stay on the API-context (`n`
  from the catalogue) path — the test
  `allocator_size_skips_non_arithmetic_arg` pins this
  contract.

Precedence (encoded by `NameSource`'s `Ord` derive):
`StringRef < ApiContext < AllocatorSize < Counter <
InductionCounter`. The induction pass outranks the
counter pass so a loop's primary induction variable
always wins `i` over a sibling phi's `count`. Both
counter and induction outrank `AllocatorSize` and
`ApiContext` — a value that flows into multiple
sites picks the *most specific* role-name; the
disambiguator's `_1` / `_2` suffix preserves
identifier uniqueness inside the function.

### Pointer-typed phis are excluded

A naive shape match (`phi(initial, phi + 1)`) fires
on `void *p; p++` walker patterns: in SSA, advancing a
pointer by 1 byte produces the same Add-and-phi
signature as advancing an integer counter. Naming a
`void *` value `i` is a regression — the API-context
heuristic already gives it the role-specific name
(e.g. `src` from `memcpy`'s 2nd parameter). To prevent
this, B3.20 threads `&TypeMap` into `recover_names`
and skips phis whose recovered type is `Type::Ptr(_)`
in *both* the induction and counter passes. Values
without a recovered type (still `Type::Unknown`)
remain eligible — the heuristic is conservative on
absence, not aggressive. The test
`loop_induction_skips_pointer_typed_phi` pins this:
the same induction-loop SSA function that earns `i`
under an empty TypeMap earns no name when the phi is
tagged `Ptr(Unknown)`.

### CLI plumbing

`dac_cli::lift::lift_one` builds the loops forest
right after `DominatorTree::build` already (`B3.9`
established the order). B3.20 adds:

```rust
let loop_info = loop_info_from_forest(&loops);
let names = recover_names(
    &ssa,
    convention.as_ref().map(|c| &c.signature),
    &ctx.api_resolver,
    &ctx.string_resolver,
    &loop_info,
    &types,
);
```

The `types` argument is the [`TypeMap`] the
propagation pass produced one step earlier. No new
SSA pass; the heuristic is a pure derived view.

### Corpus drift

Coverage on the PE corpus climbed from
`named_values=25 / 1889 (1.32%)` to
`named_values=28 / 1889 (1.48%)` — three new
`i`/`size`-class names in `hello-x86_64.exe`'s
runtime-init code (a `for(;;i++)` loop and two
`malloc`-feeding arithmetic sites). The golden
`hello-pe-o1-c/source.c` refreshed by `cargo xtask
golden update`; the matching diff is bounded to the
three values' declarations and uses inside the
function — no other code lines moved. The smaller
ELF binaries in the corpus (`hello-x86_64`,
`cpp-hierarchy-x86_64`) have no loops in the
function bodies the pipeline currently surfaces, so
coverage stays at `0 / 79` and `0 / 128`. PLT-stub
naming (B3.21) is the lever that lights up
heuristic coverage on those.

### Tests added

- `loop_induction_names_outer_counter_i` —
  canonical `for (i=0; i<n; i++)` synthesised CFG;
  asserts the header phi earns `i` with
  `NameSource::InductionCounter`.
- `nested_loops_name_inner_counter_j` — two
  natural loops nested; asserts outer phi earns
  `i`, inner phi earns `j`.
- `allocator_size_names_arithmetic_arg_size` —
  `malloc(n * 4)` synthesised call; asserts the
  `Mul` result earns `size` with
  `NameSource::AllocatorSize`.
- `allocator_size_skips_non_arithmetic_arg` —
  `malloc(16)` constant; asserts the value earns
  `n` (from the catalogue) via `ApiContext`, not
  `size`.
- `induction_outranks_allocator_size` — a loop
  whose induction phi feeds a `malloc(i + 8)` site;
  asserts the phi earns `i` while the Add result
  earns `size`.
- `counter_falls_back_when_induction_already_claimed`
  — two phis at the same header, both `+= 1`;
  asserts the first earns `i`, the second `count`.
- `loop_induction_skips_unrecognised_phi_shape` —
  a phi whose back-edge increment is `+= 2`, not
  `+= 1`; asserts no induction name fires.
- `calloc_size_argument_is_named_size_when_arithmetic`
  — `calloc(arithmetic_n, arithmetic_size)`;
  asserts both slots earn `size` / `size_1` via
  AllocatorSize.
- `loop_induction_skips_pointer_typed_phi` —
  same SSA shape as the canonical induction loop
  but the phi value is tagged `Ptr(Unknown)`; the
  heuristic must abstain.

### Limitations

- **`+= n` for `n != 1`.** A real strided loop
  (`for (i = 0; i < n; i += 2)`) does not trigger
  the induction heuristic. The PLAN names `+= 1` as
  the qualifying shape explicitly, so the
  abstention is by design; adding strided support
  would need an `Add(phi, Const(k))` walker with
  `k > 0` and matching uses of `k` in the loop
  exit. That is a separate, larger heuristic.
- **No naming inheritance.** When a loop's
  induction value flows into a function argument
  via a call site, the induction name does not
  propagate to the call site argument's local. The
  C backend renders the argument by ValueId; the
  induction name surfaces only at the phi's
  declaration. Threading the induction name through
  the call site arguments would touch the C
  backend's local-rename pass and is deferred.
- **Loop depth past `n`.** Loops nested deeper than
  six levels reuse `i` and rely on the
  disambiguator's `_1` / `_2` suffix. A real-world
  deeper nest is vanishingly rare on the corpus, so
  the table covers the depths we'd actually see.
- **Counter pass ignores write order.** A phi with
  two `+= 1` updates from sibling back-edges is
  still named `count` — the heuristic does not
  count *how many* increments fire per loop
  iteration, only that the phi has one. A doubly-
  incremented counter is rare enough that the
  `count` name is still defensible.

Closes: B3.20.
Touches: FR-N spec §11.1 (the §11.1 list of
naming heuristics now has three of the deferred
items lit up), I-2 (every new
[`NameCandidate`] carries a [`NameSource`] that
records *which* heuristic fired, so the
`--debug` "why this name?" trail can attribute
each rename), I-3 (every candidate's
`Confidence` stays at `Source::Derived` —
heuristic-name surface is never an `Observed`
claim about a recovered fact), NFR-9 (the pass
walks `LoopInfo.headers` (BTreeMap-sorted),
phis in declaration order, and operand vectors
in index order — no clock, no environment, no
filesystem iteration; the determinism gate is
preserved).

#### B3.21 — PLT-stub naming on ELF (2026-06-05)

B3.7 shipped the variable-naming pass with
API-context and string-literal heuristics. The
naming-coverage row in `--emit-report` lit up on
PE (`28 / 1889` at B3.20) but stayed at `0 / 79`
on the canonical ELF fixture: PE binaries surface
PLT-stub VAs as named `Symbol` entries the way
`BinaryImportResolver` consumes them, but ELF's
`.dynsym` lists imported functions as *undefined*
with `address = 0`. A `main` calling `write@plt`
at `0x1030` resolved to no signature, so
`ApiContext` could not fire and the values for
`fd`, `buf`, and `count` stayed `v3` / `v4` /
`v5`.

B3.21 walks the PLT trampoline table at parse
time and threads the resulting `(stub_va,
import_name)` pairs through the same resolver
path PE already uses. Two pieces ship together:

- **PLT walker.** A new module
  `dac-binfmt::plt` exports
  `elf_x86_64_plt_stubs(model, bytes) ->
  Vec<(u64, String)>`. The walker builds a
  `got_va → import_name` index from every
  `RelocationKind::Glob` (which maps both
  `R_X86_64_JUMP_SLOT` and `R_X86_64_GLOB_DAT`),
  then scans every section whose name is
  `.plt`, `.plt.sec`, or `.plt.got` in 8-byte
  strides. Each stride either accepts
  `ff 25 disp32` (`jmp qword ptr [rip +
  disp32]`, 6 bytes) or
  `f3 0f 1e fa ff 25 disp32` (`endbr64; jmp
  …`, 10 bytes), computes the GOT VA the jump
  targets, and records `(stub_va, import_name)`
  when the GOT VA lives in the JUMP_SLOT index.
  The probe stride covers canonical `.plt`
  (16-byte stubs beyond PLT[0]), `.plt.sec`
  (CFI-prefixed 16-byte stubs), and `.plt.got`
  (8-byte stubs); the "must resolve to a
  JUMP_SLOT GOT VA" filter suppresses spurious
  hits at mid-instruction strides and
  explicitly rejects PLT[0] itself (its
  `ff 25` reads from
  `_GLOBAL_OFFSET_TABLE_+0x10` — the runtime
  resolver address — which is not a JUMP_SLOT
  target). Symbol-versioned names
  (`write@GLIBC_2.2.5`) have the `@` suffix
  stripped so the API catalogue lookup hits
  the bare name.
- **Resolver wiring.**
  `BinaryImportResolver::new` in
  `dac_cli::lift` gains a `bytes: &[u8]`
  argument, calls
  `elf_x86_64_plt_stubs(model, bytes)`, and
  merges each `(stub_va, ApiSignature)` into
  `imports_by_va`. PE inputs hit the
  early-return in the walker and behave
  identically — the PE baseline holds exactly
  at `28 / 1889`.

### Pre-existing bridge bug surfaced and fixed

The PLT walker's debug trip-wire surfaced a real
bug in `dac-binfmt::bridge::relocation_symbol`.
Dynamic relocations (from
`obj.dynamic_relocations()`) index into the
`.dynsym` table, while section-attached
relocations (from `obj.sections().relocations()`)
index into the static `.symtab`. The previous
helper walked both lookup maps in fallback order:

```rust
RelocationTarget::Symbol(idx) => static_symbols
    .get(idx)
    .copied()
    .or_else(|| dynamic_symbols.get(idx).copied()),
```

so every `SymbolIndex(n)` from a dynamic
relocation silently matched whichever static
`.symtab` entry shared that ordinal. On
`hello-x86_64`, the dynamic
`R_X86_64_JUMP_SLOT` at `0x4000` (binds the
`write` slot) was being reported as resolving to
symbol `hello.c` — a `File`-kind static entry.
The fix splits the helper into a single-map
lookup
(`relocation_symbol(target, symbols)`) and
routes each relocation through the table that
owns its symbol-index space. No PE input
exercises this path, so the PE corpus is
unaffected; the ELF report is.

### CLI plumbing

`lift_all` already had `bytes: &[u8]` in scope
(it feeds the decoder). The B3.21 patch threads
it one hop further into
`BinaryImportResolver::new(model, bytes)` and
changes nothing else about the call graph. The
resolver still implements `ApiResolver` through
the same `imports_by_va` / `name_index` fields;
PLT stubs land in the same `imports_by_va` map
that PE-side `Symbol::address` entries already
populate, so the `recover_names` /
`propagate_types` consumers need no change.

### Corpus drift

- `tests/golden/hello-elf-o1-c` heuristic-name
  coverage climbed from
  `named_values=0 / 79 (0.00%)` to
  `named_values=3 / 79 (3.80%)` — the three
  positional arguments of `main`'s call to
  `write@plt`: `int32_t fd` (was `v4`),
  `void *buf` (was `v5`), `uint64_t n` (was
  `v3`). The matching type propagations also
  fire: `fd` becomes `int32_t`, `buf` becomes
  `void *` (both seeded by the catalogue,
  where they were previously `int64_t`
  defaults).
- `tests/golden/hello-elf-o1-c-hints` picks up
  the same three names — the hint catalogue did
  not override the `write` call site, so the
  PLT-derived ApiContext names surface
  unchanged.
- PE baseline holds at
  `named_values=28 / 1889 (1.48%)` exactly; the
  PE-side `Symbol::address` → PLT pathway is
  unchanged.
- Stripped ELF (`hello-x86_64-stripped`) shows
  no change. The fixture has zero
  dynamic-symbol names (the unstripped `.dynsym`
  was removed), so the PLT walker has nothing
  to bind stubs to — the "no JUMP_SLOT names →
  no stubs" early return fires. Recovering
  names on a fully stripped binary needs a
  different signal source (vtable scanning,
  byte-pattern signature matching); that is
  the "stripped-binary C++ recovery" residue
  item.

### Tests added

- **PLT walker unit tests
  (`dac-binfmt::plt`).** Twelve in-crate tests
  use a synthetic `BinaryModel` and
  hand-assembled stub bytes —
  `empty_model_returns_no_stubs`,
  `non_elf_returns_no_stubs`,
  `non_x86_64_returns_no_stubs`,
  `binary_without_jump_slots_yields_no_stubs`,
  `canonical_plt_stub_is_named_from_jump_slot`,
  `endbr_prefixed_plt_sec_stub_is_recognised`,
  `plt_got_eight_byte_stubs_are_recognised`,
  `versioned_symbol_name_is_stripped`,
  `non_executable_section_named_dot_plt_is_ignored`,
  `jump_to_unknown_got_va_is_filtered`,
  `negative_displacement_resolves_correctly`,
  `results_are_sorted_and_deduplicated`. Each
  isolates a single property of the walker so
  future regressions point at a specific shape.
- **End-to-end regression test
  (`real_hello_elf_binds_write_plt_stub`).**
  Loads the actual `hello-x86_64` fixture and
  asserts `elf_x86_64_plt_stubs` returns
  exactly `[(0x1030, "write")]`. This is the
  test the PLT-walker debug trip-wire reduced
  to — it catches future regressions to either
  the walker itself or to the
  `bridge::relocation_symbol` static / dynamic
  table split.

### Limitations

- **Other architectures.** The walker is
  x86-64 only. AArch64 PLT stubs (`adrp x16,
  GOT; ldr x17, [x16, #lo]; br x17`) need a
  separate matcher; the early-return on
  `Architecture != X86_64` keeps that work
  scoped to a future arch-specific batch (M5
  AArch64 / FR-N spec §11.1).
- **Mach-O.** The walker rejects every Mach-O
  input at the format gate. Mach-O has stubs
  (`__stubs` / `__stub_helper`) that play the
  same role but use a different opcode shape
  and a different metadata table (indirect
  symbols in `LC_DYSYMTAB`); landing it joins
  the larger "Mach-O parser" residue item.
- **Statically-linked or fully stripped ELF.**
  The walker requires a populated JUMP_SLOT
  relocation table. A statically-linked binary
  has no dynamic relocations; a fully stripped
  one has emptied them. In both cases the
  walker returns no stubs and `--emit-report`
  shows the pre-B3.21 coverage. Pattern-based
  name recovery (libc-function signatures,
  byte-pattern fingerprints) is the
  longer-term answer there and stays on the
  residue shelf.

Closes: B3.21.
Touches: FR-N spec §11.1 (the §11.1 list of
naming heuristics now lights up on ELF too),
I-1 (PLT metadata is read from the binary's own
relocation table and section bytes — the walker
never invents a stub), I-2 (every PLT-derived
[`Symbol`] → [`ApiSignature`] binding traces
back through the `R_X86_64_JUMP_SLOT`
relocation to the binary bytes that defined
it), I-3 (PLT-stub identity is `Observed`
quality — the relocation table is the
authoritative source — but the *names* the
heuristic mints from it remain `Source::Derived`
at the `NameSource::ApiContext` provenance
level, so the confidence lattice stays honest),
NFR-9 (the walker is a pure function of the
[`BinaryModel`] and input bytes; no global
state, no RNG; the resolver merges stubs by
`stub_va` in `BTreeMap` iteration order).

#### B3.22 — Hint-driven call-result naming (2026-06-06)

**Closes the last numbered M3 follow-up batch.** Threads
`Hints::find_function`'s `rename` field through to the
deterministic variable-naming pass so a `[[function]]` hint with
`rename = "send"` applied to a call site flips the call's
destination SSA value to `send`, satisfying B3.22's "done when"
criterion (FR-20).

Before this batch, a `[[function]]` `rename` overrode the
*emitted function symbol* (the recovered function's declaration
became `void send(void)` rather than `void fn_1030(void)` — B3.6
behavior). What it did *not* do was propagate downstream: a
caller's `v9 = ((...))fn_1030)(...)` left the destination value
`v9` un-renamed, so the lifted source carried two different names
for the same logical entity. B3.22 closes that gap.

- **`NameSource::UserHint` variant
  (`crates/dac-recovery/src/names.rs`).** New highest-precedence
  variant on the [`NameSource`] enum — outranks every
  deterministic heuristic (`InductionCounter > Counter >
  AllocatorSize > ApiContext > StringRef`). The
  [`NameSource::name()`] stable identifier returns
  `"user-hint"`. Module docstring's "Heuristics shipping at B3.7,
  B3.20, and B3.22" section grew a sixth bullet describing the
  new pattern; the precedence comment lists `UserHint` at the top.
- **`CallRenameResolver` trait
  (`crates/dac-recovery/src/names.rs`).** New public trait, sibling
  to [`StringResolver`], with a single
  `resolve(target_va: u64) -> Option<&str>` method. A blanket impl
  on `Fn(u64) -> Option<&'static str>` keeps the test scaffold
  ergonomic — every B3.22 unit test inlines its rename map as a
  closure. [`NullCallRenameResolver`] (`Default`) is the no-op
  default that every existing call site of [`recover_names`] in
  the test scaffold migrated to. Keeps `dac-recovery` decoupled
  from `dac-hints` — the CLI threads in a thin adapter at the
  pipeline boundary.
- **New `recover_names` collector
  (`crates/dac-recovery/src/names.rs::collect_user_hint_call_candidate`).**
  Walks every `SsaOp::Call { target: Some(va), .. }`; on a rename
  hit, sanitises the rename through the existing
  [`sanitise_identifier`] helper (same path that handles the C
  keyword list, `@` / `.` stripping, leading-digit rules) and
  proposes a [`NameCandidate`] keyed by the call's `dst`
  [`ValueId`] with `source: NameSource::UserHint` and
  `confidence: Confidence::new(NAME_CONFIDENCE, Source::UserHint)`
  (I-3 — the value carries the same `Source::UserHint` tag that
  B3.6's `apply_function_hint` minted for the type-lattice
  retypings). Parameter values are skipped — the C backend names
  parameters as `argN`, not via [`NameTable`] — and an empty
  sanitised slug abstains so the deterministic heuristics still
  get to name the dst.
- **`recover_names` signature.** Gains a `rename_resolver: &dyn
  CallRenameResolver` parameter between `strings` and `loops`.
  Every existing test in `crates/dac-recovery/src/names.rs::tests`
  threads `&null_renames()` (a new local `NullCallRenameResolver`
  helper) so the deterministic-heuristic baseline stays
  byte-identical.
- **`dac-cli/src/lift.rs::HintRenameResolver`.** New struct,
  sibling to [`BinaryStringResolver`]. Pre-computes a
  `BTreeMap<u64, String>` from `hints.functions` at construction —
  every hint with a `rename` and an address matcher
  (`HintMatcher::Address` or `HintMatcher::Both`) contributes one
  entry; `HintMatcher::Name`-only hints are skipped because the
  rename heuristic operates on call *target VAs*, which the
  recovery pass has no name index for. `lift_one` builds the
  resolver once and threads it into the [`recover_names`] call.
- **`LiftStats::hint_named_values` counter.** New field added next
  to the existing `named_values` total, populated by counting
  `NameSource::UserHint` entries in each `RecoveryFacts::names`
  provenance map. Surfaces in `--emit-report`'s `naming:` row as
  the new `hint=N` column — the recovery report now reads
  `;; naming: named_values=4 / 79 (5.06% heuristic coverage,
  hint=1)` on the hello-elf-o1-c-hints golden, cleanly satisfying
  the "done when" criterion's "citing `NameSource::UserHint` in
  the recovery report" clause.
- **`tests/fixtures/hello-x86_64.hints.toml` gained a second
  hint.** A new `[[function]] address = "0x1030" rename = "send"`
  block targets the `write@plt` trampoline so the golden
  exercises the call-site rename path end-to-end. The existing
  `main` → `user_main` hint is untouched, keeping B3.6's
  function-rename + arg-/return-retyping coverage on the same
  fixture.
- **Goldens refreshed:**
  - `hello-elf-o1-c-hints/source.c`: `int64_t v9 = 0LL;` /
    `v9 = ((...))fn_1030)(fd, ((int64_t)(buf)), n, ...)` becomes
    `int64_t send = 0LL;` /
    `send = ((...))fn_1030)(fd, ((int64_t)(buf)), n, ...)`. The
    PLT stub function declaration block also gains the
    `user_hint: id=2 rename=send` provenance line and renders as
    `void send(void)` (B3.6 path, recapped here for completeness).
  - `hello-elf-o1-c-hints/report.txt`: `user_hints=1` → `2`, and
    the new `naming:` row reads
    `named_values=4 / 79 (5.06% heuristic coverage, hint=1)`.
  - `hello-elf-o0-report/report.txt`: picks up the new column
    `(hint=0)` everywhere, keeping the format stable across
    cases.
- **Tests added (5 new in `dac-recovery::names::tests`, 627
  workspace total, +5 from B3.21's 622).**
  - `user_hint_rename_names_call_result_value` — base case: a
    rename map keyed by the call's target VA renames the call's
    `dst` SSA value to the hinted identifier, with provenance
    `NameSource::UserHint` and confidence source
    `Source::UserHint`.
  - `user_hint_rename_outranks_api_context_on_call_result` —
    precedence: even when the API-context heuristic would have
    proposed `s` (strlen's catalogue parameter name) for the dst,
    the hint's rename wins.
  - `user_hint_rename_disambiguates_repeat_calls` — two calls to
    the same hinted target give two `send`-rooted names
    (`send`, `send_1`) via the existing
    [`finalise_names`] disambiguation pass; no two values share
    a final name.
  - `user_hint_rename_skipped_when_sanitised_to_empty` — a rename
    that contains only punctuation (`@@@`) sanitises to empty;
    the candidate is abandoned and the dst gets no name (so the
    deterministic heuristics aren't blocked from proposing
    later).
  - `user_hint_rename_skips_parameter_dst` — a dst that is also
    listed as a register-arg in the inferred signature is
    skipped, so the C backend's parameter-naming path (which
    names parameters as `argN`) is not double-named by the
    heuristic.
- **Goldens net:** 3 drifts (`o0-report.txt`, `o1-c-hints/source.c`,
  `o1-c-hints/report.txt`), all expected and refreshed via
  `cargo xtask golden update`. 29/32 goldens unchanged, all 32
  match after the refresh.
- **Stability:** the deterministic heuristics' baseline is
  byte-identical to B3.21 — the new resolver injection point is
  null on every existing call site (an empty
  `NullCallRenameResolver`), and the new candidate collector
  abstains when the resolver returns `None` or the rename
  sanitises to empty.

Limitations (folded into the B3 residue shelf):

- **Call-target resolver does not consult hints.** The C
  backend's `lower_call` resolves the target VA via the
  `CNameResolver`, which `build_c_name_resolver` populates from
  `Function::name` alone. So a call to a renamed function still
  prints the *recovered* name (or `fn_<addr>` for unnamed
  stubs) as the call target, while the dst gets the rename. The
  golden's `user_main` body now reads
  `send = (...)fn_1030(...)` — the dst flips but the call
  expression's target stays at `fn_1030`. Closing this needs the
  resolver to take a `&Hints` reference too; deferred because
  it is a B3.6-rename-propagation concern, not a B3.22
  call-result-naming one.

Closes: B3.22, FR-20 (user hint propagates the rename
through to the SSA value the C backend prints), I-3 (the
new candidate carries `Source::UserHint`), NFR-9 (the
resolver is a pure projection of [`Hints`] over [`u64`];
the candidate collector iterates `ssa.blocks` in declared
order; the disambiguation pass walks the candidate map in
ascending `ValueId`).

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
