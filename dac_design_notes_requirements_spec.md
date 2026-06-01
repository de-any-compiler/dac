# dac — Design Notes, Functional Requirements, and Specification

**dac** = **de-any-compiler**

A cross-platform open-source tool that takes an executable (ELF / PE / Mach-O, starting with ELF and PE) and lifts it into a supported high-level language such as C or C++, with room to support additional compilable languages over time. The project is designed to work at multiple fidelity levels, from machine-close reconstruction to human-oriented reconstruction, with AI-assisted reasoning used primarily at the highest abstraction levels.

---

## 1. Vision

dac aims to be the best open-source tool for:

- **Decompilation**: Recovering readable source-like code from binaries.
- **Reverse engineering**: Exposing program structure, symbols, control flow, data flow, and semantics.
- **Debugging assistance**: Helping users inspect behavior, reconstruct logic, and understand runtime state.
- **Translation**: Emitting compilable code in supported target languages.

The tool should not merely generate pseudocode. It should produce code that is:

- structurally faithful,
- compilable when possible,
- progressively more readable at higher optimization levels,
- and explainable through traceable evidence.

---

## 2. Core Product Principles

1. **Correctness first**: Output must preserve behavior as much as possible before optimizing for readability.
2. **Traceability**: Every inferred construct should be attributable to evidence from the binary, analysis, or AI reasoning.
3. **Progressive abstraction**: Users can choose between machine-close output and human-like output.
4. **Determinism by default**: The same input and settings should produce the same output whenever feasible.
5. **Transparent uncertainty**: When the tool is unsure, it should mark uncertainty instead of hallucinating.
6. **Composable pipeline**: Each phase should be independently testable, replaceable, and cacheable.
7. **Open-source friendly**: Modular architecture, permissive plugin interfaces, reproducible builds, and clear contribution boundaries.
8. **Multi-language future**: The internal pipeline must not be tied to C/C++ output.

---

## 3. Scope

### In scope

- Static binary analysis.
- Symbol recovery when symbols exist.
- Control-flow graph recovery.
- Type recovery and propagation.
- Function boundary detection.
- Data-flow and def-use analysis.
- Decompilation to one or more target languages.
- Optional AI-assisted source reconstruction.
- CLI, library, and plugin interfaces.
- Batch processing and artifact export.
- Debug-style introspection views and explanation output.

### Out of scope for v1

- Perfect source reconstruction for all binaries.
- Full anti-debug / packed malware defeat.
- Executing untrusted binaries inside the main process.
- Guaranteed compilation for every output program.
- Full support for self-modifying code.

---

## 4. Optimization Levels

dac uses an `-O` style mode to control abstraction and readability.

### `-O0` — Machine-close output

Goal: preserve structure close to the disassembly.

Expected behavior:
- Direct translation of instructions into low-level constructs.
- Minimal control-flow restructuring.
- Minimal variable renaming.
- Registers may remain visible or represented as temporaries.
- Pointer arithmetic and explicit memory operations are retained.
- Strong emphasis on faithful compilation and debugging.

Typical users:
- reverse engineers,
- auditors,
- exploit researchers,
- debugging workflows.

### `-O1` — Basic reconstruction

Goal: recover readable low-level code while staying close to behavior.

Expected behavior:
- Basic variable recovery.
- Recovery of simple loops and conditionals.
- Function signatures inferred where possible.
- Some constant folding and expression simplification.
- Basic type propagation.

### `-O2` — Structured decompilation

Goal: emit code that looks like standard hand-written systems code.

Expected behavior:
- Structured control flow.
- Better variable naming heuristics.
- Semantic grouping of local state.
- Struct/enum recovery when supported by evidence.
- Better propagation of types and aliasing information.
- More aggressive expression simplification.
- Reduced register-like noise.

### `-O3` — Human-oriented reconstruction

Goal: output that resembles code written by a human developer.

Expected behavior:
- Semantic naming suggestions.
- Higher-level abstractions such as helper functions, structs, enums, and idiomatic patterns.
- Optional AI-assisted reasoning to reconstruct intent.
- Code comments or annotations may be emitted separately.
- Emphasis on readability and maintainability over literal fidelity.

Important note:
- `-O3` must never silently invent behavior. Any speculative recovery must be marked as inferred, guessed, or low-confidence.

---

## 5. AI Integration Strategy

AI should be used as an **assistive reasoning layer**, not as the source of truth.

### Best-fit AI tasks

- Naming candidate functions, variables, and structs.
- Recognizing common library and API patterns.
- Summarizing blocks of code into human-readable intent.
- Suggesting higher-level abstractions.
- Identifying likely switch statements, state machines, parsers, and serializers.
- Reconstructing idiomatic code patterns from low-level dataflow.

### AI should not be trusted for

- Raw semantics without supporting binary evidence.
- Control-flow or data-flow facts unless validated.
- Security-sensitive transformations without deterministic verification.
- Any transformation that would break compilation or change behavior.

### AI workflow rules

- AI outputs must be treated as hypotheses.
- Every AI-produced suggestion should carry confidence metadata.
- AI suggestions should be validated by static analysis.
- The user can disable AI completely.
- The tool should support offline, local, and remote model providers.
- The tool should store prompts, outputs, and evidence references for reproducibility.

### AI design idea

Use AI only after deterministic passes have produced a strong intermediate representation. The AI should consume:
- normalized SSA/IR fragments,
- recovered types,
- CFG summaries,
- string references,
- API calls,
- cross-references,
- naming context,
- and binary metadata.

This keeps AI reasoning grounded and reduces hallucination.

---

## 6. Target Languages

Initial target languages:
- C
- C++

Future target languages:
- Rust-like output where feasible,
- Zig,
- C#,
- Java,
- Go,
- pseudocode / analysis IR export,
- JSON-based intermediate artifacts for integration.

### Language target requirements

- Each target language should be implemented as a backend.
- The core analysis pipeline should remain language-agnostic.
- Backends must specify what constructs they can express.
- Unsupported constructs should degrade gracefully with annotations.

---

## 7. Architecture Overview

### High-level pipeline

1. **Input ingestion**
2. **Binary format parsing**
3. **Disassembly and lift to IR**
4. **Function discovery**
5. **CFG recovery**
6. **SSA / data-flow analysis**
7. **Type recovery**
8. **Semantic recovery**
9. **High-level reconstruction**
10. **AI-assisted refinement**
11. **Target-language generation**
12. **Verification / validation**
13. **Artifact export**

### Core internal representations

- **Binary model**: sections, segments, imports, exports, relocations, symbols, strings.
- **Instruction IR**: architecture-neutral representation of decoded instructions.
- **CFG IR**: basic blocks, edges, dominance, loops, conditionals.
- **SSA IR**: variables, use-def chains, phi nodes.
- **Semantic IR**: recovered stack variables, heap objects, structs, calls, returns.
- **Source IR**: target-language-neutral abstract source tree.
- **Backend AST**: language-specific final code structure.

### Recommended module boundaries

- `frontend/`
- `binary/`
- `arch/`
- `lift/`
- `analysis/`
- `recovery/`
- `ai/`
- `backends/`
- `verify/`
- `cli/`
- `api/`
- `plugins/`
- `tests/`

---

## 8. Functional Requirements

### 8.1 Input handling

FR-1. The system shall accept one or more input binaries.

FR-2. The system shall detect the file format automatically where possible.

FR-3. The system shall support at least ELF and PE in the initial release.

FR-4. The system shall allow the user to specify architecture overrides when auto-detection fails.

FR-5. The system shall support stripped and unstripped binaries.

FR-6. The system shall preserve import/export information where available.

### 8.2 Analysis and lifting

FR-7. The system shall disassemble supported instruction sets through a modular architecture backend.

FR-8. The system shall lift decoded instructions into an intermediate representation.

FR-9. The system shall recover function boundaries when symbols are absent.

FR-10. The system shall build control-flow graphs for recovered functions.

FR-11. The system shall compute use-def chains and SSA form where applicable.

FR-12. The system shall identify stack variables, globals, heap accesses, and temporaries.

FR-13. The system shall infer calling conventions when possible.

FR-14. The system shall infer parameter lists and return types when evidence exists.

FR-15. The system shall recover constants, strings, and referenced resources.

### 8.3 Type and semantic recovery

FR-16. The system shall propagate types from API signatures, instruction patterns, and memory usage.

FR-17. The system shall recover structs, arrays, enums, and unions when evidence supports them.

FR-18. The system shall identify common idioms such as loops, switches, error handling, and state machines.

FR-19. The system shall annotate uncertain recoveries with confidence levels.

FR-20. The system shall support user-supplied type hints and signatures.

### 8.4 Decompilation output

FR-21. The system shall generate compilable target-language source when possible.

FR-22. The system shall support multiple output verbosity / abstraction levels via `-O0` to `-O3`.

FR-23. The system shall preserve comments or metadata in a separate annotation channel.

FR-24. The system shall optionally emit formatted source code.

FR-25. The system shall emit a structured report of recovery confidence and unresolved constructs.

### 8.5 Debugging and reverse engineering support

FR-26. The system shall expose cross-references from code to data and data to code.

FR-27. The system shall show function summaries and call graphs.

FR-28. The system shall export CFGs and analysis graphs.

FR-29. The system shall expose a trace mode showing which evidence led to each recovery decision.

FR-30. The system shall allow comparison between raw disassembly, IR, and reconstructed source.

FR-31. The system shall support an interactive query interface for symbols, functions, strings, and references.

### 8.6 AI-assisted features

FR-32. The system shall allow AI-assisted naming suggestions.

FR-33. The system shall allow AI-assisted semantic summarization.

FR-34. The system shall allow AI-assisted high-level code reconstruction at `-O3`.

FR-35. The system shall support local and remote model providers through an abstract adapter.

FR-36. The system shall allow AI features to be disabled entirely.

FR-37. The system shall record AI provenance, prompts, outputs, and evidence links.

### 8.7 Export and integration

FR-38. The system shall export decompilation results as source files and structured metadata.

FR-39. The system shall export intermediate artifacts for debugging and reproducibility.

FR-40. The system shall provide a CLI interface.

FR-41. The system shall provide a library interface for integration into other tools.

FR-42. The system shall support plugins for architectures, analysis passes, and backends.

---

## 9. Non-Functional Requirements

### 9.1 Correctness and safety

NFR-1. The tool shall prioritize semantic preservation over readability when the two conflict.

NFR-2. The tool shall never silently introduce behavior not supported by evidence.

NFR-3. The tool shall isolate risky parsing or decoding operations from core orchestration where practical.

NFR-4. The tool shall support safe handling of malformed binaries.

### 9.2 Performance

NFR-5. The tool shall support incremental analysis and caching.

NFR-6. The tool shall scale to large binaries with millions of instructions when resources permit.

NFR-7. The tool shall allow parallel execution across functions or modules when analysis dependencies permit.

NFR-8. The tool shall expose performance metrics for each analysis stage.

### 9.3 Determinism and reproducibility

NFR-9. The same input and settings should produce the same output when deterministic mode is enabled.

NFR-10. The tool shall record tool version, analysis settings, and backend versions in exports.

NFR-11. The tool shall support reproducible build artifacts.

### 9.4 Usability

NFR-12. The CLI shall provide sensible defaults.

NFR-13. The output shall be easy to inspect in both terminal and file-based workflows.

NFR-14. The tool shall surface warnings, confidence levels, and unsupported features clearly.

### 9.5 Extensibility

NFR-15. New architectures shall be addable without changing the core pipeline.

NFR-16. New target languages shall be addable via backend interfaces.

NFR-17. New analysis passes shall be insertable into the pipeline.

NFR-18. The public API shall remain stable or versioned.

### 9.6 Portability

NFR-19. The tool shall run on Linux, macOS, and Windows.

NFR-20. The tool shall avoid dependence on platform-specific assumptions in the core logic.

### 9.7 Security and privacy

NFR-21. The tool shall not require uploading binaries to external services unless explicitly configured.

NFR-22. The tool shall support fully offline operation.

NFR-23. The tool shall not execute input binaries unless explicitly placed in a sandboxed execution mode.

NFR-24. Sensitive analysis data and proprietary binaries shall remain local by default.

---

## 10. Specifications

### 10.1 CLI specification

Suggested top-level command shape:

```bash
dac <input-binary> [options]
```

Suggested options:

- `-O0`, `-O1`, `-O2`, `-O3`
- `--arch <arch>`
- `--format <elf|pe|mach-o|auto>`
- `--target <c|cpp|...>`
- `--output <path>`
- `--emit-ir`
- `--emit-cfg`
- `--emit-report`
- `--emit-annotations`
- `--no-ai`
- `--ai-provider <name>`
- `--deterministic`
- `--threads <n>`
- `--json` for machine-readable results
- `--debug` for verbose traces
- `--plugin <path>`

### 10.2 Output specification

Output should be organized into:

- reconstructed source files,
- annotations / notes,
- analysis report,
- optional graph exports,
- optional intermediate artifacts.

Every emitted artifact should include:
- input hash,
- tool version,
- analysis level,
- target language,
- timestamp,
- and reproducibility metadata.

### 10.3 Confidence model

Every recovery artifact may carry:
- `confidence`: 0.0 to 1.0,
- `source`: binary evidence, heuristic, or AI,
- `explanation`: human-readable rationale,
- `dependencies`: linked analysis facts.

### 10.4 Annotation model

Annotations should distinguish between:
- **observed facts**: directly present in binary,
- **derived facts**: inferred through analysis,
- **speculative facts**: generated by AI or heuristic guess,
- **user hints**: provided externally.

### 10.5 Backend contract

Each backend shall implement:
- code generation from source IR,
- formatting and pretty-printing,
- language-specific expression lowering,
- unsupported construct handling,
- round-trip sanity checks where feasible.

### 10.6 Analysis contract

Each analysis pass shall:
- declare inputs and outputs,
- declare dependencies,
- be cacheable when possible,
- support diagnostics,
- be testable independently.

---

## 11. Heuristics and Recovery Ideas

### 11.1 Naming heuristics

Potential naming sources:
- symbol tables,
- debug info,
- string references,
- API call patterns,
- standard library identification,
- known library signatures,
- AI suggestions constrained by evidence.

### 11.2 Type recovery heuristics

Potential evidence:
- load/store width,
- arithmetic patterns,
- call signatures,
- stack layout,
- pointer chasing,
- structure access offsets,
- array indexing,
- comparison patterns.

### 11.3 Control-flow reconstruction

Desired recoveries:
- if / else,
- while / do-while / for,
- switch / case,
- early returns,
- break / continue,
- short-circuit logic.

### 11.4 Semantic reconstruction

Higher-level intent detection ideas:
- parsers,
- crypto-like routines,
- serialization/deserialization,
- memory allocators,
- object lifecycle patterns,
- finite-state machines,
- command dispatchers,
- error-handling paths.

---

## 12. Debugging Features

dac should be useful not only for decompilation but also for understanding why the code looks the way it does.

Recommended features:
- side-by-side views of disassembly, IR, and output source,
- hover or click-through evidence links,
- graph visualization of CFG and call graph,
- stack-frame and local-variable inspector,
- path-sensitive traces for selected branches,
- evidence ledger showing which facts came from where,
- “why this name?” explanation for guessed identifiers,
- “why this type?” explanation for recovered types.

---

## 13. AI Safety and Quality Guardrails

1. AI suggestions must not overwrite deterministic recovery.
2. AI-derived naming should be optionally prefixed or annotated when confidence is low.
3. AI should never fabricate function boundaries or binary facts.
4. AI output should be validated against IR consistency.
5. A “strict mode” should reject speculative AI changes.
6. A “review mode” should show proposed improvements without applying them.
7. AI prompts should be minimized and scoped to the exact evidence needed.
8. The system should support prompt templates versioned alongside analysis passes.

---

## 14. Open Source Strategy

### Repository structure idea

- `cmd/dac/` — CLI entrypoint
- `pkg/core/` — orchestrator and shared types
- `pkg/binfmt/` — ELF / PE / Mach-O parsing
- `pkg/arch/` — architecture backends
- `pkg/ir/` — intermediate representations
- `pkg/analysis/` — data-flow, SSA, type recovery
- `pkg/recovery/` — symbols, names, structures
- `pkg/ai/` — AI adapter layer
- `pkg/backend/c/`
- `pkg/backend/cpp/`
- `pkg/plugin/`
- `docs/`
- `examples/`
- `tests/`

### Contribution-friendly design

- clear interfaces,
- small testable passes,
- sample binaries for testing,
- golden-file tests for emitted source,
- fuzz tests for parsers and decoders,
- benchmark suite,
- feature flags for unstable passes.

---

## 15. Milestones

### Milestone 1 — Foundation
- ELF + PE parsing
- one architecture backend
- instruction lifting
- basic function recovery
- `-O0` textual output

### Milestone 2 — Core decompilation
- CFG recovery
- basic SSA
- type propagation
- C backend
- debug export artifacts

### Milestone 3 — Usable reverse engineering tool
- call graph
- xrefs
- better variable recovery
- C++ backend
- annotation and confidence model

### Milestone 4 — Human-oriented decompilation
- `-O3` reconstruction
- AI integration
- semantic summaries
- naming suggestions
- review mode

### Milestone 5 — Ecosystem
- plugin marketplace or registry
- more architectures
- more languages
- IDE/editor integrations
- scripted analysis API

---

## 16. Acceptance Criteria

A release of dac should be considered successful if it can:

- decompile common small-to-medium binaries into readable C/C++,
- clearly distinguish inferred from observed facts,
- produce useful machine-close output at `-O0`,
- generate more readable code at higher levels,
- support offline operation,
- let users inspect analysis decisions,
- and remain extensible for new architectures and backends.

---

## 17. Risks

- AI hallucination causing invalid reconstruction.
- Complex architectures requiring large architecture-specific code.
- Decompiled source compiling but diverging semantically.
- Performance issues on large binaries.
- Overfitting heuristics to a narrow class of programs.
- False confidence in recovered types or names.

Mitigations:
- confidence scoring,
- validation passes,
- golden tests,
- fuzzing,
- sandboxed execution mode,
- modular backends,
- strict reproducibility logs.

---

## 18. Recommended Design Decisions

1. Use a **deterministic core** with an optional AI enhancement layer.
2. Make **IR the center of the system**, not the disassembly text.
3. Treat recovery as **probabilistic with evidence**.
4. Keep **target languages as plugins**.
5. Separate **analysis facts** from **rendered source**.
6. Build **debuggability as a first-class feature**.
7. Make `-O3` a reviewable transformation, not a black box.
8. Support **exportable analysis artifacts** from day one.

---

## 19. Definition of Done for the Project Core

The core project is ready when:

- binaries can be loaded and analyzed reproducibly,
- at least one architecture is supported end-to-end,
- at least one target language compiles for simple binaries,
- output can be generated at multiple abstraction levels,
- AI can improve naming and semantic summarization without breaking determinism,
- and the output contains enough evidence to trust or debug the result.

---

## 20. Final Guiding Statement

The long-term goal of dac is to make binary understanding feel less like archaeology and more like code review: fast, explainable, structured, and increasingly human-readable without losing the truth hidden in the executable.

