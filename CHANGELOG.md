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
