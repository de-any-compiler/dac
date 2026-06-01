# dac — Architecture

This document describes how dac is structured. It is the technical companion to
[`dac_design_notes_requirements_spec.md`](./dac_design_notes_requirements_spec.md)
(which states *what* must be true) and [PLAN.md](./PLAN.md) (which states
*when* each piece lands). For the *why* behind individual choices, see
[DECISIONS.md](./DECISIONS.md).

---

## 1. Guiding invariants

Every part of dac must hold these invariants. If a feature seems to require
breaking one, the feature is wrong, not the invariant.

| #   | Invariant                                                                                       |
| --- | ----------------------------------------------------------------------------------------------- |
| I-1 | The IR is the source of truth. Disassembly text is a view, not an input to later passes.        |
| I-2 | Every IR node has a provenance edge back to the bytes (or earlier nodes) that produced it.      |
| I-3 | Every recovered fact has a confidence value and a source (Observed / Derived / Speculative / UserHint). |
| I-4 | The deterministic pipeline must run to completion without AI. AI is strictly additive.          |
| I-5 | A pass declares its inputs, outputs, dependencies, and determinism class. The pass manager enforces these. |
| I-6 | Backends never invent semantics. If the source IR cannot be lowered faithfully, the backend annotates and degrades, it does not guess. |
| I-7 | The pipeline is language-agnostic up to the Source IR layer. Target languages are pure backends. |

---

## 2. Crate layout

dac is a Cargo workspace. Each crate has one responsibility, a small public
API, and its own tests.

```
dac/
├── Cargo.toml                  # workspace manifest
├── crates/
│   ├── dac-cli/                # CLI entrypoint (the `dac` binary)
│   ├── dac-core/               # orchestrator, pass manager, evidence graph,
│   │                           # confidence lattice, shared types
│   ├── dac-binfmt/             # ELF / PE / Mach-O parsing (façade)
│   │   ├── elf/
│   │   ├── pe/
│   │   └── macho/
│   ├── dac-arch/               # Architecture trait + registry
│   ├── dac-arch-x86/           # x86 / x86-64 backend
│   ├── dac-arch-aarch64/       # AArch64 backend (M5)
│   ├── dac-ir/                 # all IR layers (Instr, CFG, SSA, Sem, Src)
│   ├── dac-lift/               # decoded instructions → Instruction IR
│   ├── dac-analysis/           # CFG, dominators, SSA, dataflow, types
│   ├── dac-recovery/           # function discovery, names, structs, idioms
│   ├── dac-knowledge/          # calling conventions, libc/Win32 sigs, patterns
│   ├── dac-ai/                 # AI adapter trait + providers (local / remote)
│   ├── dac-backend-c/          # C backend
│   ├── dac-backend-cpp/        # C++ backend
│   ├── dac-verify/             # IR consistency + AI-delta validation passes
│   ├── dac-plugin/             # plugin loading + stable ABI
│   ├── dac-api/                # library-facing public API (re-exports)
│   └── dac-artifact/           # on-disk artifact format (caching, export)
├── xtask/                      # dev tasks: golden tests, fuzz, benches
├── fuzz/                       # cargo-fuzz targets for parsers/decoders
├── examples/                   # sample binaries + walkthroughs
├── docs/                       # rendered docs, design notes
└── tests/                      # end-to-end golden-file tests
```

### Public surface

External integrators consume **only** `dac-api`. `dac-cli` is a thin shell
around `dac-api`. Other crates are implementation detail and may break between
0.x releases.

---

## 3. The pipeline

```
                       ┌──────────────────┐
                       │  Binary bytes    │
                       └────────┬─────────┘
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-binfmt  → BinaryModel (sections, symbols, relocs) ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-arch + dac-lift  → Instruction IR (per-arch)      ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-recovery  → Function boundaries                   ║
   ║  dac-analysis  → CFG → Dominators / Loops              ║
   ║  dac-analysis  → SSA construction                      ║
   ║  dac-analysis  → Dataflow, liveness, def-use           ║
   ║  dac-analysis  → Type lattice + propagation            ║
   ║  dac-recovery  → Stack frames, structs, idioms         ║
   ║                  (consults dac-knowledge)              ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  Semantic IR (typed, structured, language-agnostic)    ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-ai  (optional, -O3 only)  → AI deltas             ║
   ║  dac-verify           → validate deltas vs invariants  ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  Source IR (target-language-neutral AST)               ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-backend-{c,cpp,…}  → emitted source               ║
   ╚════════════════════════════════════════════════════════╝
                                ▼
   ╔════════════════════════════════════════════════════════╗
   ║  dac-artifact  → source + annotations + report + IR    ║
   ╚════════════════════════════════════════════════════════╝
```

The pipeline is *driven* by the pass manager in `dac-core`. The diagram above
shows a default ordering; the pass manager re-orders and parallelizes based on
declared dependencies (NFR-7).

---

## 4. IR layers

dac uses a stack of IRs, not one. Each layer is strictly higher-level than the
one below, and each carries provenance back into the lower layer. Passes are
attached to a specific layer, so a pass can never "skip" abstraction.

| Layer            | Purpose                                          | Owned by         |
| ---------------- | ------------------------------------------------ | ---------------- |
| **Binary model** | Sections, symbols, imports, relocations, strings | `dac-binfmt`     |
| **Instruction IR** | Arch-neutral form of decoded instructions        | `dac-ir::instr`  |
| **CFG IR**       | Basic blocks, edges, dominance, loop nest        | `dac-ir::cfg`    |
| **SSA IR**       | Phi nodes, def-use chains, value numbering       | `dac-ir::ssa`    |
| **Semantic IR**  | Typed locals, calls, structured CF, idioms       | `dac-ir::sem`    |
| **Source IR**    | Language-neutral AST                             | `dac-ir::src`    |
| **Backend AST**  | Target-language final tree                       | each backend     |

### Why custom over LLVM IR

LLVM IR is a *compilation* IR — it assumes the source is the truth and the
goal is fast machine code. dac's IR is a *decompilation* IR — it assumes the
machine code is the truth and the goal is a faithful, uncertain reconstruction
upward. The two have different requirements:

- **Provenance everywhere** (I-2). dac IR nodes carry an `EvidenceId` pointing
  into the evidence graph.
- **Confidence everywhere** (I-3). Type slots and name slots are
  `Confidence<T>` values, not bare `T`.
- **Partial / unknown types** as first-class lattice elements, not as
  `i8*`-style bottom.
- **Idiom slots** for "this is *probably* a `for` loop, with these candidates".

See [DECISIONS.md](./DECISIONS.md) ADR-0002 for the long form.

---

## 5. The evidence graph and confidence lattice

### Evidence graph

Every fact dac derives points back to the evidence that produced it. The
evidence graph lives in `dac-core` and is the substrate that makes the
"explain why" features in section 12 of the spec possible.

Node kinds:

- `Bytes(range)` — a span in the input file
- `Instruction(id)` — a decoded instruction
- `IRNode(id)` — any node in any IR layer
- `KnowledgeFact(id)` — a fact from the knowledge base (e.g. "x86-64 SysV
  passes int args in `rdi, rsi, rdx, rcx, r8, r9`")
- `UserHint(id)` — something the user supplied
- `AISuggestion(id)` — a proposal from an AI provider, with prompt hash

Edges are directed and labeled: "supports", "contradicts", "refines".

This graph is what `--debug` and the future TUI render. It is also what
`--emit-report` serializes.

### Confidence lattice

```rust
pub enum Source {
    Observed,    // present in the binary directly (e.g. symbol)
    Derived,     // produced by deterministic analysis
    Speculative, // produced by AI or heuristic guess
    UserHint,    // supplied by the user
}

pub struct Confidence {
    pub value: f32, // 0.0..=1.0
    pub source: Source,
}
```

Combination rules (lattice meet/join) are defined in `dac-core` and are
deterministic. A `UserHint` outranks `Speculative`. `Observed` outranks
everything else for facts the user has not contradicted.

---

## 6. Pass manager

A pass in dac is:

```rust
pub trait Pass {
    fn id(&self) -> PassId;
    fn inputs(&self) -> &[ArtifactKind];
    fn outputs(&self) -> &[ArtifactKind];
    fn determinism(&self) -> Determinism;
    fn run(&self, ctx: &mut PassContext) -> Result<()>;
}

pub enum Determinism {
    Pure,             // same input bytes → same output bytes, always
    SeededPure,       // pure given a seed (the seed is recorded)
    NonDeterministic, // rejected when --deterministic is set (e.g. remote AI)
}
```

The pass manager:

- topologically orders passes by declared inputs/outputs,
- parallelizes independent passes across cores (NFR-7),
- caches outputs in `dac-artifact` keyed by `(pass_id, input_hash, settings_hash)` (NFR-5),
- rejects `NonDeterministic` passes when `--deterministic` is set (NFR-9),
- records each pass's wall time and memory for `--debug` (NFR-8).

This is the single biggest reason dac is built around a pipeline of small
crates instead of one big binary: it lets the pass manager treat all work
uniformly.

---

## 7. Architecture backend trait

```rust
pub trait Architecture {
    fn name(&self) -> &'static str;
    fn pointer_size(&self) -> usize;
    fn endianness(&self) -> Endianness;

    fn decoder(&self) -> Box<dyn InstructionDecoder>;
    fn lifter(&self)  -> Box<dyn InstructionLifter>;
    fn calling_conventions(&self) -> &[CallingConvention];
    fn register_file(&self) -> &RegisterFile;
}
```

New architectures land by implementing this trait in a new crate
(`dac-arch-foo`) and registering it. The core pipeline does not change. This
satisfies NFR-15.

Initial targets: `x86`, `x86-64`. M5 adds `aarch64`.

---

## 8. Backend contract

```rust
pub trait Backend {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> &BackendCapabilities;

    fn lower(&self, src: &SourceIr, opts: &LoweringOptions) -> Result<BackendAst>;
    fn emit(&self, ast: &BackendAst, out: &mut dyn Write) -> Result<()>;
    fn format(&self, source: &str) -> Result<String>;
}
```

`BackendCapabilities` declares which Source-IR constructs the backend can
express. The pipeline reads this *before* the high-level reconstruction phase,
so recovery does not waste effort recovering constructs the backend cannot
emit (I-7 corollary).

Unsupported constructs degrade with an annotation, never a silent rewrite
(I-6).

---

## 9. AI adapter and the delta protocol

`dac-ai` exposes:

```rust
pub trait AiProvider {
    fn name(&self) -> &'static str;
    fn is_local(&self) -> bool;
    fn propose(&self, prompt: &Prompt, evidence: &EvidenceBundle) -> Result<Vec<Delta>>;
}
```

A `Delta` is a *proposed* change against the Semantic IR — never against the
Instruction IR or lower. Delta kinds (closed set):

- `RenameSymbol { id, new_name }`
- `RetypeSlot { id, new_type }`
- `SuggestStructLayout { id, fields }`
- `SuggestIdiom { region, idiom_kind }`
- `AnnotateRegion { region, comment }`

Every delta carries:
- `confidence: Confidence` (always `Source::Speculative`)
- `prompt_hash`, `model_id`, `seed` for reproducibility (FR-37)
- a list of `EvidenceId` it was conditioned on

`dac-verify` runs every delta through invariant checks (does the rename
collide? does the retype make any IR node inconsistent?) before
`dac-core` applies it. Failing deltas are recorded but not applied. Strict
mode (`--ai-strict`) rejects any delta that would lower confidence on a
previously Observed fact.

Review mode (`--ai-review`) records deltas without applying them, producing a
diff users can inspect (FR-35-ish, section 13.6).

---

## 10. Reproducibility manifest

Every artifact dac emits is accompanied by a `manifest.json`:

```json
{
  "input": { "path": "a.out", "sha256": "..." },
  "tool":  { "version": "0.1.0", "build_id": "..." },
  "settings": { "level": "O2", "deterministic": true, "target": "c" },
  "passes": [ { "id": "ssa-construct", "version": "1.0", "duration_ms": 12 } ],
  "ai":    { "provider": "none" },
  "plugins": [],
  "artifact_hash": "..."
}
```

The artifact hash is reproducible across machines for `--deterministic` runs
(NFR-9, NFR-10, NFR-11).

---

## 11. Caching and incremental analysis

`dac-artifact` is content-addressed storage keyed by
`hash(pass_id || input_hash || settings_hash || plugin_versions)`. Passes that
declare themselves `Pure` or `SeededPure` are cacheable; the pass manager
shortcuts to the cached output when the key hits.

This makes re-runs after a settings tweak fast and gives the future
distributed / CI workflows a natural shape.

---

## 12. Plugin model

Two plugin shapes:

1. **In-tree plugins** — additional crates that depend on `dac-api` and are
   linked into the binary at build time. Most architecture backends and target
   languages start here.
2. **Dynamic plugins** — built against `dac-plugin`'s C ABI, loaded at
   runtime via `--plugin <path>`. Used for closed-source backends and
   third-party integrations.

The dynamic ABI is frozen at M5 (B5.1). Until then, only in-tree plugins are
guaranteed to work.

---

## 13. Determinism, reproducibility, and "strict mode"

- `--deterministic`: rejects any `NonDeterministic` pass. Pins thread count
  to 1 unless the parallel scheduler is proven stable for the pipeline.
- `--ai-strict`: AI deltas may only *raise* confidence on existing facts;
  they may not retype, rename, or restructure facts already classed
  `Observed`.
- `--ai-review`: collect deltas into a separate artifact, do not apply.
- Per-pass determinism class is enforced by the pass manager, not by trust.

---

## 14. Testing strategy (the short version)

| Test layer       | Where                          | What it catches                          |
| ---------------- | ------------------------------ | ---------------------------------------- |
| Unit tests       | each crate                     | Internal invariants                      |
| Golden-file tests | `tests/golden/`               | Drift in emitted source for sample bins  |
| Round-trip tests | per backend                    | Emitted source compiles                  |
| Fuzz             | `fuzz/`                        | Parser/decoder crashes on malformed input |
| Benchmarks       | `xtask bench`                  | Perf regressions on the sample set       |
| End-to-end       | `tests/e2e/`                   | CLI behavior, manifest determinism       |

Detail and the full test plan live in [PLAN.md](./PLAN.md) under each batch.
