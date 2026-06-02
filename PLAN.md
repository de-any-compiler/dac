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

**Recommended execution order before M4:** B3.10 → B3.6 → B3.7.
Rationale: every M2/M3 recovery pass exists in `dac-recovery` /
`dac-analysis` today; B3.8 + B3.9 already wired the
`InstructionIr → RawFunction → SsaFunction → SemFunction →
C AST` bridge into `--target c -O1`. B3.10 surfaces the recovered
calling conventions / types / switch tables / struct fields in the
emitted source — which is what gives B3.6's hints a typed surface
to retype, and B3.7's names a real local-binding to rename.

### B3.6 — User hints / signatures
- TOML or JSON file: per-function signatures, struct definitions, type
  hints (FR-20).
- Hints enter the evidence graph as `Source::UserHint`.
- **Done when:** providing a hint changes the recovered type as expected and
  is reflected in the confidence report.

### B3.7 — Variable naming heuristics
- Name candidates from API context, string usage, common patterns (spec
  §11.1).
- Deterministic only at this milestone — no AI yet.
- **Done when:** generated names beat `v1, v2, v3` on the corpus per a
  measurable rubric (heuristic-name coverage %).

### B3.10 — Recovery facts → emitted source
- Close the "facts in `dac-recovery` side tables don't surface in the
  emitted source" debt explicitly recorded across B2.5 / B2.6 / B3.2 /
  B3.3 / B3.4 CHANGELOG entries.
- Thread `dac-recovery::infer_calling_convention` →
  `pick_best` → `InferredSignature` into the C / C++ lowering pass:
  functions emit `int f(int arg0, char *arg1)` instead of
  `void f(void)`. The chosen convention's name + score lands in the
  annotation channel so the leading comment cites it.
- Thread the recovered `TypeMap` (B2.6) so each `vN` local carries its
  recovered type instead of the `int64_t` fallback.
- Lower `dac-recovery::idioms::SwitchTableIdiom` (B3.3) into
  `Stmt::Switch` arms — resolves the deferred jump-table follow-up
  from the B3.3 CHANGELOG entry. Per-entry resolution reads the
  binary's `.rodata` (and where applicable, the relocation table).
- Lower `dac-recovery::structs::RecoveredStructs` field accesses as
  `s->field` / `s.field` in emitted C / C++ instead of
  `*(int*)(s+8)`.
- Closes: FR-14 (parameter / return inference reflected in source),
  FR-16 (type propagation), FR-17 (struct / array surface in emitted
  source), FR-18 (switch idiom lowered, not just recorded).
- **Done when:** a function in the corpus with a recovered convention
  emits a typed signature; a function with a recovered switch table
  emits `switch (…)` instead of a goto chain; a function with
  recovered struct field offsets emits `s->field`.

### B3 follow-up shelf

Items recorded across the M2 / M3 CHANGELOG entries as deferred but
not yet promoted to numbered batches. Each is well-scoped enough to
become a `B3.<n>` later if we choose to land it before M4 closes;
they are listed here so they stay visible.

- **Base-class recovery** (B3.5 deferral). Typeinfo-relocation walker
  that reads `__si_class_type_info` / `__vmi_class_type_info` shapes
  out of `.data.rel.ro` and populates `Class::bases`.
- **Stripped-binary C++ recovery** (B3.5 deferral). Byte-level vtable
  scanner across `.data.rel.ro` reservation patterns for the
  no-`_Z…`-symbols case.
- **Namespace lowering** (B3.5 deferral). Emit `namespace { … }` from
  the already-recovered `Class::scope_chain` instead of flattening
  into the leading comment.
- **Variadic + syscall conventions** (B2.5 deferral). SysV's
  "rax = vector-arg count" and Linux `syscall` argument layouts
  (`rdi, rsi, rdx, r10, r8, r9`).
- **Error-handling / ref-counting / state-machine idioms** (B3.3
  deferrals). Each needs additional substrate: errno tables in
  `dac-knowledge`, atomic/lock-prefix modelling at the SSA layer,
  phi-of-state-constants tracking respectively.
- **Union recovery, nested-struct chasing** (B3.2 deferrals).
- **Subreg-aliasing correctness in the bridge** (B3.8 follow-up).
  Refine the lossy full-register-write rule into the precise x86_64
  model (32-bit writes zero the upper 32; 16/8-bit writes preserve).
- **C++ body lowering** (B3.9 follow-up). The C++ AST in
  `dac-backend-cpp::ast` only describes class hierarchies and
  free-function signatures; member and free-function *bodies*
  remain `/* dac C++ stub */` stubs because the AST has no
  block/stmt node yet. Extending the AST (plus the matching
  emit/lowering rules) lets the C-side `SsaFunction → SemFunction`
  shape feed the C++ printer the same way it now feeds the C
  printer.
- **Mach-O parser** (FR-3). The format is detected and the model has a
  `BinaryFormat::MachO` variant, but no parser populates it.

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
- Pick one of: Rust-like, Zig, Go, pseudocode (spec §6).
- Implements the full `Backend` contract.
- **Done when:** at least one new language emits compilable output on a
  toy corpus.

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
