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

### B1.2 — PE parser
- Same shape as B1.1 for PE.
- Fuzz target `fuzz_pe_parse`.
- **Done when:** PE parser handles a sample `notepad.exe` and a stripped
  console exe; auto-detection (FR-2) picks ELF vs PE correctly.

### B1.3 — Architecture trait + x86-64 decoder
- `dac-arch` defines `Architecture`, `InstructionDecoder`, `RegisterFile`.
- `dac-arch-x86` implements decode for x86 and x86-64 (use `iced-x86` —
  ADR-0004).
- Snapshot tests against known instruction sequences.
- **Done when:** decoder round-trips a real `.text` section.

### B1.4 — Instruction IR + x86-64 lifter
- `dac-ir::instr`: arch-neutral instruction representation with provenance
  (I-2).
- `dac-arch-x86` lifter for the common subset (movs, arithmetic, control
  transfers, calls, returns).
- Coverage report: which opcodes are lifted vs not.
- Unsupported opcodes lift to an `Opaque` node so later passes still see
  CFG edges (I-6).
- **Done when:** lifter coverage on the sample corpus's `.text` is ≥ 95%
  by instruction count.

### B1.5 — Function discovery
- `dac-recovery::functions`: discover functions from symbols, from entry
  point, from call edges, and from prologue heuristics (FR-9).
- Function boundary results are facts in the evidence graph with sources.
- **Done when:** on the sample corpus, function discovery matches symbol
  tables ≥ 98% (stripped numbers tracked but not gated).

### B1.6 — `-O0` text backend
- Not a "real" backend — emits an annotated listing of lifted IR.
- Output is grouped per function, shows IR with byte-offset annotations.
- `--emit-report` produces the analysis report from FR-25.
- Reproducibility manifest emitted (NFR-10).
- **Done when:** running `dac sample.elf -O0` produces stable output across
  re-runs (golden test).

---

## Milestone 2 — Core decompilation (matches spec §15 M2)

Goal: real C output for small programs at `-O1`.

### B2.1 — CFG construction
- `dac-analysis::cfg`: basic blocks, edges, entry/exit, unreachable
  detection (FR-10).
- `--emit-cfg` exports DOT.
- **Done when:** CFG matches a hand-checked reference for 10 sample
  functions.

### B2.2 — Dominators + loop nest
- Dominator tree, post-dominators, natural loops, loop nest forest.
- Reducibility detection (irreducible CFGs flagged, handled with `goto`
  fallback in the backend).
- **Done when:** loop tree matches reference on the corpus.

### B2.3 — SSA construction
- Pruned SSA with phi nodes (FR-11).
- `dac-ir::ssa`.
- Value numbering for trivial CSE.
- **Done when:** SSA round-trip preserves observable semantics on a
  test-vector of small functions.

### B2.4 — Dataflow + liveness
- Use-def, def-use chains; liveness; reaching definitions (FR-11).
- Stack-frame analysis: identify stack locals (FR-12).
- **Done when:** stack-local recovery has unit tests for SysV x86-64 and
  Win64 stack patterns.

### B2.5 — Calling convention inference
- Use `dac-knowledge` calling-convention table (SysV x86-64, Win64) to
  match observed register usage (FR-13).
- Confidence based on how many signature constraints are satisfied.
- **Done when:** convention inference matches symbol-derived ground truth
  ≥ 95% on the sample corpus.

### B2.6 — Type lattice + propagation
- Initial type lattice: `Unknown`, `IntN`, `Ptr<T>`, `Struct{…}`, `Array<T,n>`,
  `Top`.
- Propagation from API signatures in `dac-knowledge` (libc, Win32 minimal
  set) and from load/store widths (FR-14, FR-16).
- **Done when:** types are recovered for ≥ 70% of locals in the corpus.

### B2.7 — Semantic IR + structuring
- `dac-ir::sem`.
- Structuring algorithm (Cifuentes-style or no-more-gotos) producing `if`,
  `while`, `for`, `switch`, early returns (FR-18, spec §11.3).
- Goto fallback for irreducible CFGs.
- **Done when:** structuring is goto-free on the sample corpus for at least
  the simple functions; metrics reported.

### B2.8 — C backend (`-O1`)
- `dac-backend-c`: Source IR → C AST → formatted source (FR-21).
- Compilability check: emitted C is fed to a system C compiler in CI
  (round-trip sanity, ARCHITECTURE §8).
- `--target c` works end-to-end.
- **Done when:** at least 5 sample binaries decompile to compilable C and
  run with matching behavior on a smoke test.

### B2.9 — Golden test infrastructure
- `tests/golden/` with corpus binaries + expected output.
- `xtask golden update` regenerates goldens.
- CI fails on drift.
- **Done when:** all goldens stable, drift gated in CI.

---

## Milestone 3 — Usable RE tool (matches spec §15 M3)

Goal: dac is genuinely useful to a reverse engineer.

### B3.1 — Call graph + xrefs
- Whole-program call graph (FR-27).
- Cross-references: code↔code, code↔data, data↔code (FR-26).
- CLI query interface for symbols/strings/refs (FR-31).
- **Done when:** `dac sample.elf --xrefs sym` prints sane results.

### B3.2 — Struct and array recovery
- Cluster offsets into struct field candidates; promote to structs when
  evidence supports (FR-17).
- Array detection from indexed access patterns.
- **Done when:** recovers known structs on a hand-built test binary.

### B3.3 — Idiom recognition
- Switch tables, error-handling patterns, ref-counting, simple state
  machines (FR-18, spec §11.4).
- Each idiom is a pass that proposes annotations; non-matches do not
  rewrite the IR.
- **Done when:** switch recovery handles compiler-emitted jump tables on
  x86-64.

### B3.4 — Annotation channel and confidence reporting
- All recovered facts surface their confidence and source (FR-19, FR-25,
  spec §10.4).
- `--emit-annotations` writes a separate annotation file alongside source
  (FR-23).
- "Why this name?" / "Why this type?" rendered in `--debug` output (spec
  §12).
- **Done when:** every name and type in emitted C is traceable through the
  evidence graph in `--debug`.

### B3.5 — C++ backend (`-O2`)
- `dac-backend-cpp`: class recovery from vtables, ctor/dtor patterns,
  member function naming (FR-21).
- **Done when:** a sample C++ binary with a small class hierarchy decompiles
  to C++ that compiles.

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
