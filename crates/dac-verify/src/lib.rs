//! `dac-verify` — IR consistency and AI-delta verification passes for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §9 in the workspace
//! root.
//!
//! ## Status (B4.3)
//!
//! - [`verify_delta`] judges a single [`dac_ai::Delta`] against a
//!   [`KnownFacts`] snapshot and returns a structured
//!   [`VerifyOutcome`]. Per-variant invariants cover the closed set of
//!   delta kinds (spec §13.4):
//!     - `RenameSymbol`: identifier validity, name-collision against
//!       any other symbol in the world.
//!     - `RetypeSlot`: "no pointer evidence" rejection when the slot's
//!       recovered type is non-pointer.
//!     - `SuggestStructLayout`: non-empty, ascending offsets, valid
//!       field identifiers.
//!     - `SuggestIdiom`: non-empty idiom tag.
//!     - `AnnotateRegion`: non-empty comment.
//! - [`VerifyMode::Strict`] (`--ai-strict`) additionally drops any
//!   delta whose target is recorded as [`dac_core::Source::Observed`]
//!   (ARCHITECTURE §13). Lenient mode lets these through; the
//!   confidence lattice's join semantics keep the Observed fact in
//!   place regardless, but strict mode surfaces the rejection up front
//!   so the orchestrator never applies a proposal that would have been
//!   shadowed.
//! - [`KnownFacts`] is the world model the orchestrator populates from
//!   recovered state before judging deltas. Empty by default; an empty
//!   world rejects every proposal as [`UnknownTarget`](DeltaRejection::UnknownTarget),
//!   which is the safe default until B4.4 / B4.5 wire the world from
//!   real recovered facts.
//!
//! ## Invariants this crate is responsible for
//!
//! - **I-3 enforcement at the apply boundary.** `dac-ai` already
//!   clamps every delta's `confidence.source` to `Speculative`; the
//!   verifier additionally rejects strict-mode overwrites of Observed
//!   facts so the lattice never has to silently absorb a "would
//!   shadow" Speculative proposal.
//! - **I-5 (validate before mutation).** `verify_delta` is the single
//!   public path a delta walks through before any IR mutation. Failing
//!   deltas are recorded but never applied (ARCHITECTURE §9).
//! - **NFR-9 (determinism).** Every check is a pure function of the
//!   delta and world. No I/O, no global state, no PRNG, no time
//!   reads. The pass manager treats [`verify_delta`] as `Pure`.

#![forbid(unsafe_code)]

mod verify;
mod world;

pub use verify::{verify_delta, DeltaRejection, TargetKind, VerifyMode, VerifyOutcome};
pub use world::{KnownFacts, KnownRegion, KnownSlot, KnownSymbol, SlotType};
