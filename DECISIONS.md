# dac — Architectural Decision Records (ADRs)

This file records significant design choices. Each ADR follows the form:

> **Context** → **Decision** → **Consequences** → **Alternatives considered**.

ADRs are append-only. If a decision is reversed, write a new ADR that
supersedes the old one; do not edit history.

---

## ADR-0001 — License: Apache-2.0

**Status:** Accepted, 2026-06-01.

**Context.** dac is open-source-bound; the license needed to be chosen
before any code merged so that contributions land under a known license.
Options considered: Apache-2.0 (single license), MIT/Apache-2.0 dual,
MPL-2.0, GPL-3.0.

**Decision.** Apache-2.0. The canonical text is at
[`LICENSE`](./LICENSE) in the workspace root. Every crate's `Cargo.toml`
declares `license = "Apache-2.0"` via `workspace.package`.

**Consequences.**

- Patent grant covers contributors and downstream embedders, which matters
  for a tool that touches reverse-engineered binaries.
- Compatible with the major Rust crates in the ecosystem we plan to depend
  on (`goblin`, `object`, `iced-x86`, `yaxpeax`, `capstone-rs`).
- Section 5 of the license (Contributions) means we do not yet need a
  separate CLA; sign-off (`git commit -s`) is sufficient for now.
- A future move to MIT/Apache-2.0 dual licensing is still possible without
  contributor re-permission since Apache-2.0 covers the redistribution case.

**Alternatives considered.**

- **MIT/Apache-2.0 dual.** The Rust ecosystem default. Slightly more
  permissive. Rejected for now to keep the licensing story simple and to
  retain the patent grant unambiguously.
- **MPL-2.0.** Weak copyleft at the file level. Rejected because it
  complicates embedding in proprietary tools — dac is meant to be a
  foundation other tools embed.
- **GPL-3.0.** Rejected for the same reason: it would prevent commercial
  embedding and shrink the audience.

---

## ADR-0002 — Implementation language: Rust + custom SSA decompilation IR

**Status:** Accepted, 2026-06-01.

**Context.** Two coupled choices must be made together: the implementation
language and the shape of the central IR. The project's invariants (every
node has provenance, every fact has confidence, deterministic core,
language-agnostic up to source IR) constrain both.

**Decision.**

1. dac is written in **Rust** (stable channel, pinned via
   `rust-toolchain.toml`).
2. The central IR is a **custom SSA-based decompilation IR** with provenance
   and confidence as first-class node attributes.

**Consequences.**

- Cargo workspaces map cleanly to the modular crate layout in
  `ARCHITECTURE.md` §2.
- Memory safety covers the most dangerous code paths (parsers for untrusted
  binaries) without per-pass effort (NFR-4).
- Mature crates available: `goblin`/`object` (binfmt), `iced-x86`,
  `yaxpeax`, `capstone-rs` (decoders). The exact choices are separate ADRs.
- The IR is more work to build than reusing LLVM IR, but LLVM IR is a
  *compilation* IR — provenance, confidence, partial types, and idiom slots
  are not first-class there. Shoehorning would compromise invariants I-2,
  I-3, I-6.
- Rust compile times will hurt iteration. Mitigated by the small-crate
  layout (incremental rebuilds touch few crates).

**Alternatives considered.**

- **Go.** Better build times, matches the `cmd/`+`pkg/` layout the spec
  sketches. Disassembly/IR ecosystem is thin (mostly Capstone bindings); we
  would write more from scratch.
- **C++ with CMake.** Direct LLVM access. Slow to iterate in, hardest to
  keep memory-safe on malformed inputs.
- **Zig.** Comptime is attractive for arch backends. Ecosystem is too
  immature for this scope right now.
- **Reuse LLVM IR.** Mature, well-tooled — but it is the wrong shape (see
  ARCHITECTURE.md §4).
- **Build on RetDec / remill / BAP.** Faster start, but inherits design
  constraints and licensing of upstream, and undermines invariant I-7
  (language-agnostic pipeline).

---

## ADR-0003 — Binary parser library: TBD (placeholder)

**Status:** Open. Closes in B1.1.

**Context.** Need to pick between `goblin` (single crate, multi-format) and
`object` (split crates, lower-level). Both are mature.

**Decision.** Deferred. The shortlist:
- `goblin` — easier ergonomics, covers ELF / PE / Mach-O.
- `object` — more flexible, used widely in the Rust toolchain.

**Consequences.** Either is replaceable behind the `dac-binfmt` façade.
The cost of switching later is moderate, so this is not a blocking decision.

---

## ADR-0004 — x86 decoder library: TBD (placeholder)

**Status:** Open. Closes in B1.3.

**Context.** Shortlist: `iced-x86` (Intel-style, fast, well-maintained),
`yaxpeax-x86` (more general, decode + encode), `capstone-rs` (FFI to
Capstone, broad arch coverage).

**Decision.** Deferred. Working assumption: `iced-x86` for x86 because of
its decoder accuracy and instruction-info API, with `yaxpeax` as the fallback
if licensing or feature gaps appear.

**Consequences.** The architecture trait (`dac-arch`) hides the choice from
the rest of the pipeline. Switching later is local.

---

## How to add an ADR

1. Pick the next free number.
2. Write `Context → Decision → Consequences → Alternatives` — do not skip
   alternatives, they are how future-you understands the trade-off.
3. Mark status: `Accepted`, `Open`, `Superseded by ADR-NNNN`.
4. Cross-link from `ARCHITECTURE.md` or `PLAN.md` where relevant.
