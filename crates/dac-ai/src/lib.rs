//! `dac-ai` — AI adapter layer and delta protocol for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §9 in the workspace
//! root.
//!
//! ## Status (B4.2)
//!
//! - [`AiProvider`] trait, [`Delta`] enum, and [`EvidenceBundle`] builder
//!   ship the offline-only substrate so the rest of M4 can be authored
//!   without committing to a real model adapter (FR-32, FR-35,
//!   ARCHITECTURE §9).
//! - [`NullProvider`] is the default; it always returns no deltas and
//!   keeps the I-4 corridor (deterministic pipeline completes without
//!   AI) trivially satisfied.
//! - [`LocalProvider`] is the first real local provider, gated by
//!   `--ai-provider local` (or `--ai-provider local:stub`). The
//!   default backend is a rule-based, in-process delta producer that
//!   needs no external host (FR-35, NFR-21, NFR-22). HTTP-backed
//!   `local:llama` / `local:ollama` adapters are reserved for a
//!   follow-up batch.
//! - [`EchoProvider`] is a test-only fixture that replays a fixed list
//!   of [`Delta`]s on every call; nothing reaches a network.
//! - [`templates`] holds the versioned prompt templates the CLI uses to
//!   build the prompts it hands the provider (spec §13.8 / NFR-10).
//!
//! ## Invariants this crate is responsible for
//!
//! - **I-3 / FR-37.** Every [`Delta`] is constructed through one of the
//!   `Delta::*` helpers, which clamp `confidence.source` to
//!   [`dac_core::Source::Speculative`]. Real-model providers cannot
//!   bypass that — the [`AiProvider::propose`] return type is
//!   `Vec<Delta>`, and `Delta`'s metadata field is `pub(crate)` access
//!   only via these constructors. CLI dispatch checks the source on
//!   ingress as a defence in depth ([`assert_speculative`]).
//! - **I-2.** Every [`Delta`] carries a non-empty list of evidence
//!   handles that the proposer claims to have conditioned on. The
//!   constructors reject an empty bundle.
//! - **FR-37 reproducibility.** Every [`Delta`] carries the
//!   `prompt_hash`, `model_id`, and `seed` of the call that produced
//!   it. The prompt hash is derived from the prompt's `kind_tag` plus
//!   its text via [`prompt_digest`], so identical prompts hash
//!   identically across builds (no random salting).
//!
//! Delta verification (`dac-verify`) lands with B4.3.

#![forbid(unsafe_code)]

mod bundle;
mod delta;
mod error;
mod local;
mod prompt;
mod provider;
pub mod templates;

pub use bundle::EvidenceBundle;
pub use delta::{
    assert_speculative, Delta, DeltaBuildError, DeltaMetadata, ProposerContext, RegionRef, SlotRef,
    StructFieldSuggestion, SymbolRef,
};
pub use error::{ProviderError, ProviderResult};
pub use local::{LocalBackend, LocalProvider};
pub use prompt::{prompt_digest, Prompt, PromptKind};
pub use provider::{
    select_provider, AiProvider, EchoProvider, NullProvider, ProviderSelection, SelectionReason,
};
