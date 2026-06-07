# dac — Project Plan

This is the execution plan. It maps the requirements in
[`dac_design_notes_requirements_spec.md`](./dac_design_notes_requirements_spec.md)
into milestones and batches.

## How to read this plan

- **Milestone** = a user-visible capability ("we can decompile an x86-64 ELF
  to `-O0` C").
- **Batch** = a self-contained PR-sized unit of work. Each batch has an ID
  (`B<milestone>.<n>`), a goal, deliverables, the FR/NFR/Invariant IDs it
  satisfies, and "done when" criteria. Batches are designed to merge
  independently and are the unit recorded in [CHANGELOG.md](./CHANGELOG.md).
- **Finished batches move to CHANGELOG.md.** This file stays focused on what
  is upcoming.

Numbers in `(FR-X, NFR-Y, I-Z)` reference the requirements spec and the
invariants in [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## Milestone 0 — Project skeleton

Goal: a Cargo workspace that builds, tests, lints, and has the basic
plumbing every later batch will use. No analysis yet.

*All M0 batches complete — see [CHANGELOG.md](./CHANGELOG.md).*

---

## Milestone 1 — Foundation (matches spec §15 M1)

Goal: load an ELF or PE, decode x86-64, lift to Instruction IR, recover
functions, and emit `-O0` textual output that is a faithful annotated
disassembly-style listing.

*All M1 batches complete — see [CHANGELOG.md](./CHANGELOG.md).*

---

## Milestone 2 — Core decompilation (matches spec §15 M2)

*All M2 batches complete — see [CHANGELOG.md](./CHANGELOG.md).*

---

## Milestone 3 — Usable RE tool (matches spec §15 M3)

Goal: dac is genuinely useful to a reverse engineer.

B3.1 – B3.22, B3.23 – B3.31, and B3.32 are complete — see
[CHANGELOG.md](./CHANGELOG.md). B3.33 – B3.35 are the remaining
polish-wave batches that lift the most visible non-residue gaps
before M3 closes: canonical typedef preservation in extern decls,
void-return inference for runtime helpers, and i386 dispatch
wiring. The "B3 residue shelf" further down tracks heavier
residue items that stay deferred past M3 and are sized as
separate milestones rather than numbered batches.

### Sequencing

Within the remaining B3.33 – B3.35 wave, B3.33 (typedef
preservation in extern decls) is independent of the others. B3.34
(void-return inference) is also independent of B3.33. B3.35 (i386
dispatch wiring) unblocks the GOTHIC.EXE-class of PE fixtures and
is independent of the other two but lower-priority for
hello-fixture polish.

### B3.33 — Canonical typedef preservation in extern decls

- `CType::Typedef(name, underlying)` and `CType::Const(Box<CType>)`
  variants added so the C printer can render `ssize_t`, `size_t`,
  `const void *`, etc., verbatim instead of collapsing to
  `int64_t` / `uint64_t` / `void *`.
- The lowering pass consults `dac-knowledge` for known API
  signatures and opts the extern decl into the canonical typedefs.
- Adds `#include <sys/types.h>` (or equivalent) when the rendered
  decl uses a typedef outside `stddef.h` / `stdint.h`.
- **Done when:** the extern declaration for `write` on the
  `hello-x86_64` fixture reads exactly
  `extern ssize_t write(int fd, const void *buf, size_t n);` and
  the emitted source still compiles round-trip. (FR-21)

### B3.34 — Void-return inference for unobserved-rax helpers

- New whole-callgraph pass (`Pure` determinism class): for every
  function whose return value is dropped by every caller (no SSA
  use of the returned register across the callgraph), the
  recovered signature's return type is demoted to `void`.
- Confidence `Derived`; the original return-type evidence remains
  in the graph.
- **Done when:** `_init`, `_fini`, `register_tm_clones`,
  `deregister_tm_clones`, `__do_global_dtors_aux` emit with a
  `void` return type on the `hello-x86_64` fixture; no behaviour
  change for functions whose return value is observed by at least
  one caller. (FR-21)

### B3.35 — i386 dispatch wiring (PE focus)

- `Architecture::I386 => Some(Backend { … })` added to the dispatch
  in `dac-cli::main.rs` so the existing `I386` zero-sized type,
  its decoder, and its register file (all already present in
  `dac-arch-x86`) route through the same pipeline as `X86_64`.
- 32-bit ABI plumbing (cdecl / stdcall on i386 PE) where the
  existing x86-64 SysV / Win64 conventions don't already cover it.
- The compile round-trip gate is *not* a done-when for this batch
  — i386 corpus quality is a residue-shelf concern.
- **Done when:** `dac --target c <i386 PE fixture>` produces a
  listing with at least one recovered function instead of the
  current "no architecture backend available" stub, and the
  manifest correctly reports `architecture: i386`. (FR-3, FR-21)

### B3 residue shelf

The deferrals below stay on the shelf rather than landing as
numbered batches before M4 opens. Each is either large enough to
read as a separate milestone (Mach-O parser, C++ body lowering,
idiom cluster) or depends on a numbered batch already queued
above.

- **Stripped-binary C++ recovery** (B3.5 deferral). Byte-level
  vtable scanner across `.data.rel.ro` reservation patterns for the
  no-`_Z…`-symbols case.
- **Error-handling / ref-counting / state-machine idioms** (B3.3
  deferrals). Each needs additional substrate: errno tables in
  `dac-knowledge`, atomic / lock-prefix modelling at the SSA layer,
  phi-of-state-constants tracking respectively.
- **Union recovery, nested-struct chasing** (B3.2 deferrals).
- **C++ body lowering** (B3.9 follow-up). The C++ AST in
  `dac-backend-cpp::ast` only describes class hierarchies and
  free-function signatures; member and free-function *bodies*
  remain `/* dac C++ stub */` stubs because the AST has no
  block / stmt node yet. Extending the AST (plus the matching
  emit / lowering rules) lets the C-side `SsaFunction → SemFunction`
  shape feed the C++ printer the same way it now feeds the C
  printer.
- **Struct hint application** (B3.6 follow-up, FR-17 / FR-20).
  `[[struct]]` hints parse and enter the evidence graph, but
  `apply_struct_hint` still has no path to retype recovered
  layouts. B3.16 landed the typedef substrate the lowering
  pass needs; wiring the hint into the struct typedef table
  remains.
- **Mach-O parser** (FR-3). The format is detected and the model
  has a `BinaryFormat::MachO` variant, but no parser populates it.

---

## Milestone 4 — Human-oriented reconstruction (matches spec §15 M4)

Goal: `-O3` with AI assistance, review mode, and strict mode.

### B4.1 — AI adapter trait + offline default
- `dac-ai`: `AiProvider` trait, `Delta` enum, `EvidenceBundle` builder
  (ARCHITECTURE §9, FR-32, FR-35).
- A "null" provider that always returns no deltas (default).
- An "echo" provider for tests.
- **Done when:** AI plumbing exists end-to-end with zero real model calls.

### B4.2 — Local model provider (llama.cpp / ollama)
- Adapter to a local provider (FR-35, NFR-21, NFR-22).
- Prompt templates versioned alongside passes (spec §13.8).
- **Done when:** running with a local model produces deltas on the sample
  corpus and `--no-ai` produces identical output to M3.

### B4.3 — Delta verification (`dac-verify`)
- IR consistency checks for every delta type (spec §13.4).
- Strict mode (`--ai-strict`) drops any delta that would reduce confidence
  on an Observed fact.
- **Done when:** verification rejects a hand-crafted "rename to colliding
  symbol" delta and a "retype int→ptr without evidence" delta.

### B4.4 — Review mode
- `--ai-review` collects deltas as a side artifact without applying them
  (FR-33, spec §13.6).
- Diff renderer for proposed changes.
- **Done when:** review-mode output is human-readable and stable.

### B4.5 — `-O3` semantic reconstruction
- AI is consulted only at `-O3` and only after deterministic passes complete
  (spec §5).
- Naming suggestions, idiom suggestions, region summaries.
- Confidence-aware rendering: low-confidence AI names get a prefix
  (configurable) or annotation.
- **Done when:** `-O3` produces meaningfully more readable C on the corpus
  than `-O2`, with the strict-mode invariant preserved.

### B4.6 — Remote model provider (opt-in)
- Adapter for at least one remote API.
- Off by default (NFR-21). `--ai-provider remote:<name>` opts in.
- Provenance: prompt, response, model id, seed recorded (FR-37).
- **Done when:** the remote adapter passes the same delta-verification tests
  as the local one, and `--deterministic` rejects it.

---

## Milestone 5 — Ecosystem (matches spec §15 M5)

Goal: dac is contributable, extensible, and integratable.

### B5.1 — Dynamic plugin ABI
- Freeze `dac-plugin` C ABI for architectures and backends (FR-42, NFR-15,
  NFR-16, NFR-18).
- Versioned: minor version compatibility guaranteed.
- **Done when:** an out-of-tree sample plugin compiled separately can be
  loaded with `--plugin`.

### B5.2 — AArch64 architecture
- `dac-arch-aarch64`: decoder + lifter for the common subset.
- Validates the plugin/architecture boundary.
- **Done when:** corpus includes an AArch64 ELF that decompiles end-to-end.

### B5.3 — Additional target language backend
Promoted to its own milestone — see
[Milestone 6](#milestone-6--additional-target-language-backend-go).
B5.3 stayed a one-batch placeholder for too long; honest scoping
showed a new-language backend is milestone-sized (type-system
bridge + lowering + idiom mapping + compile gate + golden
coverage), not a single PR. Numbering kept stable so existing
references survive.

### B5.4 — Public scripted analysis API
- Stable `dac-api` surface; `0.x` → `1.0` policy.
- Examples in `examples/` (FR-41).
- **Done when:** an external project depends on `dac-api` and uses it
  without touching internal crates.

### B5.5 — IDE/editor integration (proof of concept)
- LSP-like server exposing the evidence graph and "why this name/type?"
  for editor display (spec §15 M5).
- **Done when:** a minimal VS Code extension renders an annotation overlay
  on emitted source.

---

## Milestone 6 — Additional target language backend (Go)

Goal: a `--target go` mode that produces Go source from recovered IR
with the same evidence-grounding and determinism guarantees as
`--target c`. The Go backend stresses the language-neutrality of
the IR (every assumption that leaked C-isms into the IR shows up
here).

### B6.1 — Go AST + emit skeleton
- New `dac-backend-go` crate: `TranslationUnit`, `Item`, `Stmt`,
  `Expr` AST mirroring `dac-backend-c::ast`'s shape; per-node
  provenance via `EvidenceId` (I-2).
- Deterministic printer (NFR-9): stable iteration order, no
  hashmap leak into output ordering.
- **Done when:** an empty `dac --target go <fixture>` invocation
  produces a valid `package main` translation unit with the dac
  banner and zero recovered functions.

### B6.2 — Type system mapping (C ↔ Go)
- `CType → GoType` bridge: physical-width ints become `int8` /
  `uint16` / `int32` / `uint64` / …; pointers become `*T` where
  `T` is a Go-friendly element; opaque/recovered structs become
  `struct { … }` with field names preserved.
- Canonical typedef preservation: `size_t` → `uintptr`,
  `ssize_t` → `int`, `const T *` → `*T` (Go has no const) with a
  doc comment recording the loss. ADR in
  [DECISIONS.md](./DECISIONS.md) capturing the const-loss
  decision.
- **Done when:** every type the C printer emits has a defined Go
  counterpart, and the bridge ships with goldens covering
  primitives, pointers, structs, and arrays.

### B6.3 — Function signature + body lowering
- `SemFunction → GoFunction` reusing the structurer's output
  (the structured tree is language-agnostic; this batch is what
  proves that).
- `if` / `for` / `switch` lower to native Go forms. The
  structurer's fallback `goto` blocks survive verbatim (Go
  supports `goto`).
- **Done when:** the `hello-x86_64` fixture decompiles to Go
  that `gofmt`s cleanly and invokes a `syscall.Write(1, buf, 6)`
  (or `os.Stdout.Write`) equivalent.

### B6.4 — Idiom mapping (defer, error returns)
- Recognise C error-return idioms (`errno`-style negative-return,
  `NULL`-return-on-failure) and surface them as Go `(T, error)`
  return shapes where the evidence supports it.
- Stack-cleanup patterns (e.g. `free` at every function exit)
  surface as `defer`.
- **Done when:** at least one corpus fixture demonstrates each
  idiom translation with provenance preserved.

### B6.5 — Compile round-trip gate
- `go vet` + `go build` gate on every emitted unit in CI.
- Silently skipped when `go` is not on PATH (mirrors B3.5's C++
  skip contract).
- **Done when:** every corpus fixture round-trips through
  `go build` on CI hosts that have Go installed.

### B6.6 — Determinism + golden coverage
- Two-run determinism diff for the Go backend.
- Golden files mirror the C backend's coverage for the same
  fixtures.
- **Done when:** `cargo xtask ci` reports zero divergence across
  two runs of any Go-backend golden case, and the golden corpus
  covers every fixture the C backend covers.

### B6.7 — Go-specific recovery (optional follow-up)
- Surfacing recovered shapes in their idiomatic Go forms once the
  M4 AI work has landed: e.g. lifting recovered C `Result`-style
  tagged unions into `(T, error)` pairs with AI-provided naming.
- **Done when:** at least one fixture demonstrates an AI-driven
  Go-only idiom surfacing not present in the C output.

---

## Cross-cutting work (continuous, not a milestone)

These are ongoing concerns that every batch must respect. They are not
batches — they are review checkboxes.

- **Determinism.** Every new pass declares its `Determinism` class. CI runs
  the deterministic corpus twice and diffs manifests.
- **Fuzzing.** Every new parser or decoder ships with a fuzz target.
- **Benchmarks.** `xtask bench` tracks per-pass wall time on a fixed corpus.
- **Docs.** Each crate has a `README.md` summarizing its role and public
  types.
- **ADRs.** Non-obvious decisions get an ADR in [DECISIONS.md](./DECISIONS.md).
- **Spec traceability.** Each batch closes specific FR/NFR/I numbers. The
  CHANGELOG records which.

---

## Risk register

These map onto spec §17 with concrete mitigations per phase.

| Risk                                  | Mitigation                                                                       |
| ------------------------------------- | -------------------------------------------------------------------------------- |
| AI hallucination breaks recovery      | Delta protocol, strict mode, evidence-grounded prompts (M4)                      |
| Decompiled C compiles but diverges    | Round-trip sanity tests in CI (B2.8)                                             |
| Perf collapse on large binaries       | Pass-level caching, parallelism, per-pass benches (B0.4, cross-cutting)          |
| Heuristic overfitting to sample corpus | Corpus growth as part of every milestone; rubric metrics, not anecdote (B3.7)    |
| False confidence in names/types       | Confidence lattice, source attribution, `--ai-strict`, user hints (B0.3, B4.3)   |
| Plugin ABI churn                       | Freeze at M5; in-tree only until then (B5.1)                                     |

---

## Definition of "done" for the project core

Mirrors spec §19. The core is done when:

- Binaries load and analyze reproducibly (M0 + M1).
- ≥1 architecture is end-to-end (M1, M2; AArch64 added in M5).
- ≥1 target language compiles for simple binaries (M2 C, M3 C++).
- Output exists at `-O0` through `-O3` (M1 → M4).
- AI improves naming and summaries without breaking determinism (M4).
- Output carries enough evidence to trust or debug (M3, spec §12).
