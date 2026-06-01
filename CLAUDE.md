# CLAUDE.md — guidance for AI assistants working in this repo

This file is the orientation kit for any Claude (or other AI) session
operating on dac. Read it before editing anything. It is short on purpose —
the long form lives in [README.md](./README.md),
[ARCHITECTURE.md](./ARCHITECTURE.md), [PLAN.md](./PLAN.md),
[DECISIONS.md](./DECISIONS.md), and
[CONTRIBUTING.md](./CONTRIBUTING.md).

---

## What dac is

A Rust workspace that lifts ELF / PE / Mach-O binaries to high-level source
(C / C++ first). Decompiler, not transpiler: the binary is ground truth.

## How work is organized

- **Spec → invariants → milestones → batches.**
  - `dac_design_notes_requirements_spec.md` has numbered requirements
    (`FR-N`, `NFR-N`).
  - `ARCHITECTURE.md` has numbered invariants (`I-N`).
  - `PLAN.md` has milestones M0–M5 with batches `B<milestone>.<n>`.
  - `CHANGELOG.md` records finished batches under `[Unreleased]`.
- **Always cite IDs.** Commits, PRs, and ADRs reference the FR/NFR/I numbers
  they close. This is the only thing that keeps the spec connected to the
  code over time.
- **Finished batches move.** When a batch lands, its `PLAN.md` entry moves
  to `CHANGELOG.md` (don't duplicate).

## Non-negotiables

These are load-bearing for the whole project. If a task seems to require
breaking one, the task is wrong — flag it and stop.

1. **The IR is the source of truth** (I-1). Disassembly text is a view, not
   an input to later passes.
2. **Every IR node carries provenance** (I-2) — an `EvidenceId` back to the
   bytes (or earlier nodes) that produced it.
3. **Every recovered fact carries a `Confidence` and a `Source`** (I-3) —
   `Observed`, `Derived`, `Speculative`, or `UserHint`.
4. **The deterministic pipeline runs to completion without AI** (I-4). AI is
   strictly additive and lives behind `dac-ai`.
5. **AI ships as a delta protocol**, not as free-form text. Closed enum of
   delta kinds, validated by `dac-verify` before any IR mutation.
6. **`--deterministic` is enforced by the pass manager** (NFR-9), not by
   trust. Every pass declares a `Determinism` class
   (`Pure` / `SeededPure` / `NonDeterministic`).
7. **Backends never invent semantics** (I-6). If Source IR can't be lowered
   faithfully, the backend annotates and degrades.

## Workflow conventions

- **Pick the next open batch from `PLAN.md`** before starting code. If the
  user asks for work that isn't in `PLAN.md`, push back and propose adding
  it as a batch.
- **Branch names:** `b<milestone>.<n>-<slug>`, e.g. `b1.3-x86-decoder`.
- **One canonical CI command:** `cargo xtask ci`. It runs fmt + clippy + test.
  Run it before declaring a batch done.
- **No `unwrap()` / `expect()` in non-test code** outside `dac-cli`'s
  startup path. Anything touching input bytes returns `Result`.
- **Tests per batch kind:** see the table in
  [CONTRIBUTING.md](./CONTRIBUTING.md#tests-we-expect-with-a-batch). Parsers
  and decoders ship with a `cargo-fuzz` target; passes ship with goldens;
  backends ship with a compile round-trip.
- **ADRs.** If a choice would take more than two sentences to justify, write
  an ADR in [DECISIONS.md](./DECISIONS.md). Don't skip the *Alternatives*
  section — that's what future-you will want to read.

## What to do when uncertain

- About a design choice: check `ARCHITECTURE.md` first, then `DECISIONS.md`,
  then ask the user. Do not invent a convention silently.
- About requirements: the spec is authoritative. Cite the FR/NFR ID and
  quote the language.
- About scope: when in doubt, do less. Bug fixes don't need surrounding
  cleanup; one-shot operations don't need helpers.

## What *not* to do

- Don't add a pass that bypasses the pass manager.
- Don't let an AI delta touch the Instruction IR or below — Semantic IR only.
- Don't raise a `Speculative` fact's `Confidence` source to `Observed`.
- Don't introduce a `NonDeterministic` pass without an explicit reviewer ack
  in the PR (see `CONTRIBUTING.md`).
- Don't add new top-level docs without updating `README.md`'s "Quick links"
  table.

## Useful commands

```bash
cargo xtask ci                # canonical CI check
cargo xtask test              # tests only
cargo build --workspace       # build everything
cargo test -p dac-core        # tests for a single crate
```

## When the user says "kick off B<x.y>"

It means: start the batch in PLAN.md, write the code that satisfies its
deliverables, run `cargo xtask ci`, commit, and move the batch entry from
`PLAN.md` to `CHANGELOG.md` (under `[Unreleased]`).
