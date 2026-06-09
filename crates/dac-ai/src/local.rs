//! [`LocalProvider`] — the first non-null [`crate::AiProvider`] in the
//! dac workspace.
//!
//! B4.2 introduces a real local provider behind `--ai-provider local`.
//! Its first backend is [`LocalBackend::Stub`]: a rule-based, in-process
//! delta producer that runs without any external dependency, returns
//! the same deltas for the same prompt+bundle pair, and never touches a
//! network. It is the always-available local provider that satisfies
//! the FR-35 "adapter to a local provider" contract on every CI host.
//!
//! The HTTP-backed `llama.cpp` and `ollama` adapters are *out of scope*
//! at this batch — they require a live server on the host and a runtime
//! detector that gates them on availability. The plumbing here is
//! shaped so a later batch can plug in additional [`LocalBackend`]
//! variants without revising the [`crate::AiProvider`] surface or the
//! CLI dispatch table.
//!
//! ## Determinism (NFR-9)
//!
//! Every backend that ships at B4.2 is purely a function of its inputs
//! — no time, no PRNG, no environment reads. `--deterministic` does not
//! need to special-case the local provider; the corridor is preserved
//! by construction.
//!
//! ## Why deltas not text
//!
//! The provider returns [`crate::Delta`] values, not text. This keeps
//! the AI surface honest about what kinds of changes are allowed (only
//! the closed [`crate::Delta`] enum) and routes every proposal through
//! the same metadata-guarded constructor that clamps the
//! [`dac_core::Source`] to `Speculative` (I-3). The stub backend below
//! shows the minimum surface a real model adapter has to satisfy: read
//! the prompt's kind, consume the evidence bundle, and produce a delta
//! whose metadata cites the same handles the bundle carried.

use dac_core::{Confidence, Source};

use crate::delta::{Delta, DeltaBuildError, ProposerContext, RegionRef, SymbolRef};
use crate::prompt::{Prompt, PromptKind};
use crate::provider::AiProvider;
use crate::{EvidenceBundle, ProviderError, ProviderResult};

/// The dac local-AI provider.
///
/// Wraps a closed [`LocalBackend`] enum so the orchestrator can resolve
/// `--ai-provider local`, `--ai-provider local:stub`, and (in future
/// batches) `--ai-provider local:llama` / `--ai-provider local:ollama`
/// onto the same provider type. Every backend reports itself as local
/// for [`AiProvider::is_local`] — that is what `--deterministic` keys
/// off when it rejects remote providers in B4.6.
#[derive(Debug, Clone)]
pub struct LocalProvider {
    backend: LocalBackend,
}

impl LocalProvider {
    /// The stable name suffix for the default rule-based stub backend.
    /// Visible in logs and in the manifest's `ai.provider` field when
    /// the caller passes `--ai-provider local:stub`.
    pub const STUB_NAME: &'static str = "local:stub";

    /// Build the deterministic rule-based local provider. This is the
    /// backend `--ai-provider local` resolves to today; HTTP-backed
    /// llama.cpp / ollama adapters are deferred.
    #[must_use]
    pub fn stub() -> Self {
        Self {
            backend: LocalBackend::Stub,
        }
    }

    /// Borrow the backend currently in use. Useful for tests that want
    /// to assert the variant without round-tripping through
    /// [`AiProvider::name`].
    #[must_use]
    pub fn backend(&self) -> LocalBackend {
        self.backend
    }
}

/// Closed set of [`LocalProvider`] backends.
///
/// `Stub` is the only variant that ships at B4.2. Future variants will
/// adapt the same trait surface to a local-host HTTP server (llama.cpp
/// in OpenAI-API mode at `http://localhost:8080/v1/chat/completions`,
/// ollama at `http://localhost:11434/api/chat`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalBackend {
    /// Rule-based deterministic backend. Always available. Returns
    /// content-addressed deltas that depend only on the prompt + bundle
    /// — see [`AiProvider::propose`] below.
    Stub,
}

impl LocalBackend {
    /// Name suffix the backend reports to the orchestrator. Joined with
    /// `local:` to form the provider name surfaced through
    /// [`AiProvider::name`].
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Stub => "stub",
        }
    }
}

impl AiProvider for LocalProvider {
    fn name(&self) -> &str {
        match self.backend {
            LocalBackend::Stub => Self::STUB_NAME,
        }
    }

    fn is_local(&self) -> bool {
        true
    }

    /// Propose deltas for the given prompt + bundle.
    ///
    /// The stub backend's contract is:
    ///
    /// - An empty bundle returns `Ok(vec![])` — no evidence, no
    ///   proposal. This matches the existing [`crate::NullProvider`]
    ///   shape so a caller that hands the provider nothing still gets a
    ///   clean run (and the orchestrator's "did you receive deltas?"
    ///   path is exercised on both providers).
    /// - A non-empty bundle produces exactly one delta, keyed off
    ///   [`Prompt::kind`]:
    ///   * [`PromptKind::NameSuggestion`] →
    ///     [`Delta::RenameSymbol`] proposing `dac_local_sub_<id>` where
    ///     `<id>` is the FNV-1a folded prompt-text hash truncated to
    ///     the bottom 32 bits.
    ///   * [`PromptKind::Annotation`] (and every other variant) →
    ///     [`Delta::AnnotateRegion`] whose comment carries the prompt's
    ///     digest as a 16-hex-character prefix so a reviewer can match
    ///     the comment back to the originating prompt without
    ///     consulting the manifest.
    ///
    /// Both branches pass the bundle's handles through to
    /// [`crate::DeltaMetadata::evidence`], which keeps the I-2
    /// provenance chain intact.
    fn propose(&self, prompt: &Prompt, evidence: &EvidenceBundle) -> ProviderResult<Vec<Delta>> {
        match self.backend {
            LocalBackend::Stub => stub_propose(prompt, evidence),
        }
    }
}

fn stub_propose(prompt: &Prompt, evidence: &EvidenceBundle) -> ProviderResult<Vec<Delta>> {
    if evidence.is_empty() {
        return Ok(Vec::new());
    }
    let confidence = Confidence::new(0.35, Source::Speculative);
    let ctx = ProposerContext {
        prompt,
        evidence,
        confidence,
        model_id: LocalProvider::STUB_NAME,
        seed: 0,
    };
    let delta = match prompt.kind {
        PromptKind::NameSuggestion => Delta::rename_symbol(
            SymbolRef(prompt_id_lo(prompt)),
            format!("dac_local_sub_{:08x}", prompt_id_lo(prompt) as u32),
            ctx,
        ),
        // Every non-rename prompt resolves to an annotation. Real
        // backends will branch on every closed `PromptKind` variant
        // once the corresponding delta family lands; the stub keeps the
        // surface uniform so the orchestrator's "any provider, any
        // prompt" contract holds at the type level.
        PromptKind::StructLayout
        | PromptKind::Retype
        | PromptKind::Idiom
        | PromptKind::Annotation => Delta::annotate_region(
            RegionRef(prompt_id_lo(prompt)),
            format!(
                "dac local-stub annotation [prompt {:016x}] kind={}",
                prompt_id_lo(prompt),
                prompt.kind.tag(),
            ),
            ctx,
        ),
    }
    .map_err(build_error_to_provider_error)?;
    Ok(vec![delta])
}

/// Truncate the prompt's 32-byte digest to its bottom 8 bytes for use
/// as a stable per-prompt 64-bit id. The full 256-bit digest stays in
/// [`crate::DeltaMetadata::prompt_hash`] for FR-37 — this 64-bit id is
/// purely an ergonomic anchor for the local stub's body strings.
fn prompt_id_lo(prompt: &Prompt) -> u64 {
    let d = prompt.digest();
    u64::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]])
}

fn build_error_to_provider_error(err: DeltaBuildError) -> ProviderError {
    // The local stub passes through `ProposerContext`s it builds
    // itself, so a `DeltaBuildError` here would be a programmer error
    // in the stub's own code path. Surfacing it as `Transport` keeps
    // the public API honest about which side of the trait failed —
    // the model didn't refuse; the adapter dropped the request — and
    // lets the orchestrator log a single field instead of branching on
    // the build-error variant.
    ProviderError::Transport(format!("local stub failed to construct delta: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::Prompt;
    use dac_core::{EvidenceGraph, EvidenceNode};

    fn fixture_bundle() -> EvidenceBundle {
        let mut g = EvidenceGraph::new();
        EvidenceBundle::from_iter([
            g.add_node(EvidenceNode::Instruction(0x1040)),
            g.add_node(EvidenceNode::Bytes { start: 0, end: 64 }),
        ])
    }

    #[test]
    fn stub_provider_advertises_local_and_kebab_name() {
        let p = LocalProvider::stub();
        assert_eq!(p.name(), "local:stub");
        assert!(p.is_local());
        assert_eq!(p.backend(), LocalBackend::Stub);
        assert_eq!(LocalBackend::Stub.tag(), "stub");
    }

    #[test]
    fn stub_returns_empty_for_empty_bundle() {
        let p = LocalProvider::stub();
        let prompt = Prompt::new(PromptKind::Annotation, "hello");
        let deltas = p.propose(&prompt, &EvidenceBundle::new()).expect("ok");
        assert!(
            deltas.is_empty(),
            "empty bundle must produce zero deltas, got {deltas:?}",
        );
    }

    #[test]
    fn stub_produces_rename_for_name_suggestion_prompts() {
        let p = LocalProvider::stub();
        let prompt = Prompt::new(PromptKind::NameSuggestion, "name sub_1040");
        let bundle = fixture_bundle();
        let deltas = p.propose(&prompt, &bundle).expect("ok");
        assert_eq!(deltas.len(), 1);
        match &deltas[0] {
            Delta::RenameSymbol { new_name, meta, .. } => {
                assert!(
                    new_name.starts_with("dac_local_sub_"),
                    "name should start with local-stub prefix: {new_name}",
                );
                assert_eq!(meta.evidence(), bundle.ids());
                assert_eq!(meta.model_id(), "local:stub");
                assert_eq!(meta.confidence().source(), Source::Speculative);
            }
            other => panic!("expected RenameSymbol, got {other:?}"),
        }
    }

    #[test]
    fn stub_produces_annotation_for_other_prompt_kinds() {
        let p = LocalProvider::stub();
        let bundle = fixture_bundle();
        for kind in [
            PromptKind::Annotation,
            PromptKind::Idiom,
            PromptKind::Retype,
            PromptKind::StructLayout,
        ] {
            let prompt = Prompt::new(kind, format!("test for {}", kind.tag()));
            let deltas = p.propose(&prompt, &bundle).expect("ok");
            assert_eq!(deltas.len(), 1, "kind {kind:?} must produce 1 delta");
            match &deltas[0] {
                Delta::AnnotateRegion { comment, meta, .. } => {
                    assert!(
                        comment.contains(kind.tag()),
                        "annotation must reference its kind tag: {comment}",
                    );
                    assert_eq!(meta.evidence(), bundle.ids());
                    assert_eq!(meta.confidence().source(), Source::Speculative);
                }
                other => panic!("expected AnnotateRegion for {kind:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn stub_is_deterministic_across_calls() {
        let p = LocalProvider::stub();
        let prompt = Prompt::new(PromptKind::NameSuggestion, "deterministic check");
        let bundle = fixture_bundle();
        let a = p.propose(&prompt, &bundle).expect("ok");
        let b = p.propose(&prompt, &bundle).expect("ok");
        assert_eq!(a, b);
    }

    #[test]
    fn stub_differs_for_different_prompt_text() {
        let p = LocalProvider::stub();
        let bundle = fixture_bundle();
        let a = p
            .propose(&Prompt::new(PromptKind::NameSuggestion, "first"), &bundle)
            .expect("ok");
        let b = p
            .propose(&Prompt::new(PromptKind::NameSuggestion, "second"), &bundle)
            .expect("ok");
        assert_ne!(a, b, "different prompt text must produce different deltas",);
    }

    #[test]
    fn stub_prompt_hash_is_recorded_in_metadata() {
        let p = LocalProvider::stub();
        let prompt = Prompt::new(PromptKind::Annotation, "carry-hash check");
        let bundle = fixture_bundle();
        let deltas = p.propose(&prompt, &bundle).expect("ok");
        let meta = deltas[0].meta();
        assert_eq!(meta.prompt_hash(), prompt.digest());
    }
}
