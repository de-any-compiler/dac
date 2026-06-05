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

The numbered M3 critical-path batches (B3.1 – B3.10) are complete,
plus B3.11 – B3.14 — see [CHANGELOG.md](./CHANGELOG.md).
The remaining 8 numbered follow-up batches (B3.15 – B3.22) below
are pre-M4 work: each closes a specific deferral surfaced in a
CHANGELOG entry and can land independently. Heavier residue items
remain in the "B3 residue shelf" at the end of this section.

### B3.15 — Typed-local refinement (B3.10 follow-up)
- Retype SSA locals from `dac-recovery::TypeMap` directly (B3.10
  retyped only parameters and return types). Surfaces the
  pointer / int mixes the lifter's sub-register arithmetic
  produces; pair with per-use casts where the lattice diverges
  from the variable's `width_bits`.
- **Done when:** a corpus function with a recovered pointer
  parameter shows typed locals downstream instead of every
  local landing as `int64_t`.

### B3.16 — Struct typedef surface (B3.10 follow-up, FR-17)
- Grow the C AST a translation-unit-level `struct` typedef node
  (`Item::StructDecl`).
- Plumb `dac-recovery::RecoveredStructs` into the lowering pass so
  each pointer-anchored layout emits a real typedef plus
  `s->field` access in place of the B3.10 `/* recovered field:
  … */` comment.
- **Done when:** the PE corpus golden shows at least one
  `struct {...}` typedef and at least one `s->field_<hex>`
  access where B3.10 emitted only the marker comment.

### B3.17 — Switch-arm resolution (B3.10 follow-up, FR-18)
- Resolve per-entry switch-table targets by reading bytes from
  `.rodata` (and, on PE, walking the relocation table for rebased
  entries). Mint labels at each target block and populate
  `Stmt::Switch::arms` accordingly.
- Anchor labels outside the structurer's recursive walk so the
  label slots survive arm rewriting.
- **Done when:** a corpus function with a jump-table dispatch
  emits a populated `switch` with per-arm `case <const>:` /
  `goto L<n>;` shapes, replacing the B3.10 empty-arms surface.

### B3.18 — `dac-recovery::structs` SSA-source bounds correctness (B3.10 surfaced)
- `lookup_def_op` currently bounds-checks
  `ValueSource::Instruction { block, index }` defensively after
  the PE corpus surfaced an over-bound index. Chase the
  underlying inconsistency in the SSA constructor /
  value-source bookkeeping; remove the defensive guard when the
  invariant holds again.
- **Done when:** the bookkeeping is tightened (unit test covers
  the previously-out-of-bounds case from the PE corpus), and the
  defensive guard becomes a `debug_assert!`.

### B3.19 — Hint provenance in annotations (B3.6 follow-up, FR-19 / FR-20)
- Thread the matched `EvidenceNode::UserHint` ID into
  `annotate_name` / `annotate_return_type` in
  `dac-cli/src/annotations.rs` so the `.annot.json` sidecar
  names the hint that pinned the type.
- **Done when:** running with a `[[function]]` rename hint
  produces an annotations sidecar whose `name` block cites the
  hint's evidence ID instead of the deterministic pipeline's
  classification.

### B3.20 — Loop-induction & counter naming (B3.7 follow-up, spec §11.1)
- Layer per-function dataflow naming on top of
  `dac-recovery::names`. Three heuristics:
    1. Loop-induction counter (`i` / `j` / `k`) — the phi value
       of a natural loop header whose only back-edge increment is
       `+= 1`.
    2. Counter pattern (`count`) — a non-induction value whose
       only mutating op is `+= 1`.
    3. Allocator-size (`size`) — an arithmetic adjacent to a
       `malloc` / `calloc` call where the result feeds the call's
       size argument.
- **Done when:** the ELF or PE corpus produces at least one
  named `i` or `count` value where B3.7 emitted `v<id>`, and
  the report row's heuristic-coverage % climbs against the prior
  baseline.

### B3.21 — PLT-stub naming on ELF (B3.7 surfaced, FR-N spec §11.1)
- Walk the PLT trampoline at `.plt.sec` / `.plt.got` and thread
  the trampoline VA → import-name map into
  `BinaryImportResolver::resolve`. Lights up API-context naming
  (and type-propagation seeds) on unstripped ELF binaries.
- **Done when:** `tests/golden/hello-elf-o1-c` reaches a non-zero
  heuristic-name coverage % (it was 0 / 98 at B3.7) and the
  matching PE coverage baseline holds steady.

### B3.22 — Hint-driven naming (B3.7 follow-up, FR-20)
- Thread `Hints::find_function`'s `rename` into
  `NameTable::values` for the matching call sites' arguments so a
  user hint propagates names downstream the way the user expects
  from the rename field.
- **Done when:** a `[[function]]` hint with `rename = "send"`
  applied to a call site flips the SSA value name and the
  emitted source's identifier to `send`, citing
  `NameSource::UserHint` in the recovery report.

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
- **Hint argument synthesis past the inferred prefix** (B3.6
  follow-up, FR-20). `apply_function_hint` retypes positional
  arguments the convention pass already inferred, but a hint
  whose `args` lists more slots than `int_args` cannot mint
  additional `RegisterArg` entries — the extra slots have no
  SSA-side value to bind. Synthesising them needs the C
  backend to learn a "declared but unused" parameter shape so
  the printed signature can carry the full hinted arity.
- **Struct hint application** (B3.6 follow-up, FR-17 / FR-20).
  `[[struct]]` hints parse and enter the evidence graph, but
  the lowering pass still surfaces struct fields as
  `/* recovered field: … */` comments. Lands once the struct
  typedef surface (B3.16) is in place.
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
