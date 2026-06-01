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

## ADR-0003 — Binary parser library: `object`

**Status:** Accepted (closed in B1.1, 2026-06-01).

**Context.** `dac-binfmt` needs to parse ELF, PE, and Mach-O. Two mature
Rust crates fit: `goblin` (single crate, format-specific types per format)
and `object` (trait-based, uniform read API). The choice is load-bearing
because every later layer (lifters, recovery passes, backends) reads
through whatever vocabulary `dac-binfmt` exposes.

**Decision.** Use `object` (version `0.36`, `read` + `std` features only).

**Reasoning.** Three properties make `object` the better fit for dac:

1. **Trait-uniform reads.** `Object`, `ObjectSection`, `ObjectSegment`,
   `ObjectSymbol`, and the relocation traits expose the same shape across
   ELF / PE / Mach-O. That maps almost 1:1 onto `BinaryModel`'s
   format-agnostic vocabulary, so PE (B1.2) and Mach-O reuse the same
   bridge code instead of growing a parallel path.
2. **Rustc / cargo lineage.** `object` is the parser used by the Rust
   toolchain itself. It has been adversarially exercised on every linker
   input the Rust ecosystem has seen, which is the strongest available
   answer to NFR-4 (safe handling of malformed binaries).
3. **`#![no_std]`-friendly with `default-features = false`.** dac is `std`
   today, but keeping the parser core `no_std` capable matters for the
   embedded/firmware use case the spec leaves room for.

**Alternatives considered.**
- **`goblin`** — simpler call surface, but every format gets its own
  type, so the façade ends up reimplementing the trait-uniformity that
  `object` already provides. The split would push format-specific glue
  into PE (B1.2) and beyond.
- **Hand-rolled parsers.** Strongest invariants in theory; in practice a
  decompiler-grade ELF/PE/Mach-O parser is a year of work and is exactly
  what `object` already is. Rejected as scope.

**Consequences.**
- `object` types never leak past `dac-binfmt`. Downstream crates depend
  only on the `BinaryModel` vocabulary, so swapping parsers later is
  contained to one crate.
- `object::Object::dynamic_relocations()` is the source of truth for
  shared-library / executable relocations. Static (`.o`) relocations
  arrive through per-section `relocations()`; the model accommodates
  both by making `Relocation::section` optional and using `offset` for
  either a section-relative offset or a virtual address.
- The crate's `read` feature set is enough; `write` and the format-
  specific compile-time features stay off.

---

## ADR-0004 — x86 decoder library: `iced-x86`

**Status:** Accepted (closed in B1.3, 2026-06-01).

**Context.** `dac-arch-x86` needs an x86 / x86-64 decoder. Three Rust-
visible options were shortlisted:

- **`iced-x86`** — Intel-style decoder + formatter, instruction-info
  (`FlowControl`, `near_branch_target`, register reads/writes), fast.
- **`yaxpeax-x86`** — Rust-native decoder + encoder, more uniform across
  the `yaxpeax-arch` family.
- **`capstone-rs`** — FFI binding to Capstone, broad architecture
  coverage, battle-tested.

**Decision.** Use `iced-x86` (`1.21`, features `std`, `decoder`,
`instr_info`, `intel`).

**Reasoning.**

1. **Flow-control + branch-target metadata is first-class.** iced exposes
   `instr.flow_control()` and `instr.near_branch_target()` directly, and
   they map cleanly onto dac-arch's [`ControlFlow`] (`Sequential`,
   `ConditionalBranch { target }`, `IndirectCall`, …). CFG construction in
   B2.1 picks this up unmodified. `yaxpeax-x86` exposes equivalent
   information but the projection takes more work; `capstone-rs` wraps
   the C `cs_detail` API behind FFI.
2. **Explicit invalid-encoding reporting.** iced returns an `Instruction`
   with `is_invalid()` set on unrecognized bytes — the trait surface can
   surface that as `valid: false` with `(bad)` text, which matches I-6
   ("degrade, don't invent").
3. **Rust-native.** `capstone-rs` requires shipping a system C dependency
   and an FFI boundary at every build site. iced-x86 keeps
   `dac-arch-x86` a pure Rust crate, so cross-compiling stays a `cargo
   build --target ...` away.
4. **Coverage breadth.** iced covers every modern Intel/AMD opcode
   including AVX-512, AMX, the 2024 APX extensions, plus 16/32/64-bit
   modes — far more than dac needs at M1 but enough that we will not
   outgrow the decoder.
5. **License.** `iced-x86` is MIT — compatible with dac's Apache-2.0
   (ADR-0001).

**Alternatives considered.**

- **`yaxpeax-x86`.** Strong design and Rust-native, and the multi-arch
  family is attractive for when AArch64 lands (M5). The deciding
  factor was the more direct flow-control surface in iced; if `dac-arch`
  ever needs to host two decoders side-by-side for cross-checking,
  yaxpeax is the obvious second.
- **`capstone-rs`.** Battle-tested across architectures and the canonical
  choice for many RE tools. Rejected here because the FFI + libcapstone
  build dependency does not pay for itself on an x86-only target, and
  the instruction-info surface is wrapped behind `cs_detail` rather than
  exposed as Rust enums.
- **Hand-rolled.** Out of scope by a wide margin; a decompiler-grade
  x86-64 decoder is months of careful work and is exactly what iced
  already is.

**Consequences.**

- `iced_x86` types stay inside `dac-arch-x86::decoder`. Downstream crates
  depend only on `dac_arch::{InstructionDecoder, DecodedInstruction,
  ControlFlow, …}`, so swapping decoders later is contained to one
  module.
- The B1.4 lifter is free to consume iced's `Instruction` directly for
  accurate operand semantics, while still emitting the arch-neutral
  Instruction IR.
- iced uses internal `unsafe` for performance; that is allowed by
  workspace lints (`unsafe_code = "warn"` only checks first-party code,
  and `dac-arch-x86` itself stays `#![forbid(unsafe_code)]`).

---

## How to add an ADR

1. Pick the next free number.
2. Write `Context → Decision → Consequences → Alternatives` — do not skip
   alternatives, they are how future-you understands the trade-off.
3. Mark status: `Accepted`, `Open`, `Superseded by ADR-NNNN`.
4. Cross-link from `ARCHITECTURE.md` or `PLAN.md` where relevant.
