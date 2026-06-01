# dac — de-any-compiler

> Lift executables back to readable, compilable source.

**Status:** pre-alpha · pre-code. Currently in the design and planning phase.

dac is an open-source decompiler that turns ELF / PE / Mach-O binaries into
high-level source code (starting with C and C++). It is built around four
ideas:

1. **Correctness first, readability second.** Output is semantically faithful
   before it is pretty.
2. **Evidence on every line.** Every recovered name, type, and construct is
   attributable to a fact in the binary, a heuristic, an AI suggestion, or a
   user hint — and the tool can tell you which.
3. **Progressive abstraction.** `-O0` to `-O3` walks from machine-close output
   up to human-style reconstruction.
4. **Deterministic core, optional AI.** AI assists at high abstraction
   levels but never overrides deterministic recovery, and the whole tool runs
   offline when AI is disabled.

## Quick links

| Doc                                             | What's in it                                                              |
| ----------------------------------------------- | ------------------------------------------------------------------------- |
| [PLAN.md](./PLAN.md)                            | Milestones, batches, sequencing, scope of work                            |
| [ARCHITECTURE.md](./ARCHITECTURE.md)            | Crate layout, IR design, pipeline, plugin model                           |
| [DECISIONS.md](./DECISIONS.md)                  | Architectural decision records (ADRs) — why we picked what we picked      |
| [CHANGELOG.md](./CHANGELOG.md)                  | Completed batches and releases                                            |
| [CONTRIBUTING.md](./CONTRIBUTING.md)            | How to send patches, batch process, test expectations                     |
| [CLAUDE.md](./CLAUDE.md)                        | Orientation kit for AI assistants working in this repo                    |
| [`dac_design_notes_requirements_spec.md`](./dac_design_notes_requirements_spec.md) | Original requirements spec (FR-/NFR-numbered)        |

## Foundational choices

- **Language:** Rust. Memory safety for parsing untrusted binaries, a mature
  binary-analysis ecosystem (`goblin`, `object`, `iced-x86`, `yaxpeax`,
  `capstone`), and Cargo workspaces map cleanly to the modular crate layout.
- **IR:** custom SSA-based decompilation IR. Designed for *recovering*
  programs, not generating them — every node carries provenance and a
  confidence lattice value (see [ARCHITECTURE.md](./ARCHITECTURE.md)).
- **Pipeline:** explicit pass manager. Each pass declares inputs, outputs,
  dependencies, and a determinism class.
- **AI:** isolated adapter layer. AI proposes *deltas* against the IR; deltas
  are validated by deterministic passes before being applied.

## Project status

Not yet usable. The current goal is to land Milestone 1 (foundation: ELF + PE
parsing, x86-64 lift, `-O0` textual output). See [PLAN.md](./PLAN.md) for the
batch list.

## License

Not yet selected. The project is open-source-bound; a permissive license
(Apache-2.0 or MIT/Apache-2.0 dual) will be picked before the first tagged
release. Tracked in [DECISIONS.md](./DECISIONS.md) as ADR-0001.
