//! [`AiProvider`] trait, the two offline providers ([`NullProvider`]
//! default + [`EchoProvider`] test fixture), and the [`select_provider`]
//! dispatch table that maps `--no-ai` / `--ai-provider` onto a concrete
//! provider.
//!
//! B4.2 wires [`crate::LocalProvider`] into the dispatch table as the
//! first real local provider — see the table in [`select_provider`].
//! Remote-API providers land in B4.6; until then every non-local /
//! non-null name resolves to [`NullProvider`] with a
//! [`SelectionReason::Fallback`] tag so the orchestrator can warn once
//! and keep the deterministic pipeline (I-4) intact.

use crate::local::LocalProvider;
use crate::{Delta, EvidenceBundle, Prompt, ProviderResult};

/// What an AI adapter looks like from `dac-core`'s perspective.
///
/// Providers are pure proposers: they receive a [`Prompt`] + an
/// [`EvidenceBundle`] and return a [`Vec<Delta>`]. They never touch
/// the IR directly (I-4) and never mutate global state. Implementors
/// are `Send + Sync` so the orchestrator can hand them off to a thread
/// pool in B4.5 without re-engineering the trait.
pub trait AiProvider: Send + Sync + std::fmt::Debug {
    /// Stable identifier for logs, manifest, and FR-37 provenance.
    /// One-word kebab-case (`"null"`, `"echo"`, `"local:llama"`).
    fn name(&self) -> &str;

    /// `true` iff the provider runs entirely on-host (no network).
    /// `--deterministic` rejects non-local providers in B4.6.
    fn is_local(&self) -> bool;

    /// Propose zero or more [`Delta`]s for the given prompt + bundle.
    ///
    /// Returning `Ok(vec![])` is the contract for "I have nothing
    /// to add" — that path is always available and is what
    /// [`NullProvider`] always does.
    fn propose(&self, prompt: &Prompt, evidence: &EvidenceBundle) -> ProviderResult<Vec<Delta>>;
}

/// The default provider. Always returns an empty proposal list so the
/// I-4 corridor (deterministic pipeline runs to completion without
/// AI) is trivially satisfied even when the CLI dispatch wires a
/// provider in.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullProvider;

impl NullProvider {
    /// Stable provider name suitable for the manifest's
    /// `ai.provider` field.
    pub const NAME: &'static str = "null";
}

impl AiProvider for NullProvider {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn is_local(&self) -> bool {
        true
    }

    fn propose(&self, _prompt: &Prompt, _evidence: &EvidenceBundle) -> ProviderResult<Vec<Delta>> {
        Ok(Vec::new())
    }
}

/// Test fixture that replays a fixed list of [`Delta`]s on every
/// [`AiProvider::propose`] call. Useful for downstream pipeline
/// authors who want a stub that produces *non-empty* output without
/// touching a model.
///
/// The replayed deltas are cloned per call, so the provider can be
/// reused across multiple propose calls and the caller still owns
/// each returned vector independently.
#[derive(Debug, Clone)]
pub struct EchoProvider {
    name: String,
    deltas: Vec<Delta>,
}

impl EchoProvider {
    /// Provider name suitable for the manifest. Same kebab-case
    /// convention as the rest of the AI surface.
    pub const NAME: &'static str = "echo";

    /// Build an echo provider that replays `deltas` verbatim. The
    /// provider name is fixed to [`EchoProvider::NAME`]; tests that
    /// want a custom label call [`EchoProvider::with_name`].
    #[must_use]
    pub fn new(deltas: Vec<Delta>) -> Self {
        Self {
            name: Self::NAME.to_string(),
            deltas,
        }
    }

    /// Override the reported provider name. Useful when a test wants
    /// to see "model_id" propagate into the manifest.
    #[must_use]
    pub fn with_name(name: impl Into<String>, deltas: Vec<Delta>) -> Self {
        Self {
            name: name.into(),
            deltas,
        }
    }

    /// Number of deltas this provider returns per call.
    #[must_use]
    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    /// `true` iff the provider would return zero deltas.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }
}

impl AiProvider for EchoProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_local(&self) -> bool {
        true
    }

    fn propose(&self, _prompt: &Prompt, _evidence: &EvidenceBundle) -> ProviderResult<Vec<Delta>> {
        Ok(self.deltas.clone())
    }
}

/// Outcome of resolving a CLI `--ai-provider` argument plus `--no-ai`
/// flag into a concrete [`AiProvider`].
///
/// Returned by [`select_provider`]. The orchestrator inspects the tag
/// to decide whether to warn (Fallback) or stay silent (Selected) and
/// what `ai.provider` value to record in the manifest.
#[derive(Debug)]
pub struct ProviderSelection {
    /// The provider the orchestrator should use.
    pub provider: Box<dyn AiProvider>,
    /// Why this provider was chosen, suitable for a `tracing::info!`
    /// field. One-word kebab-case.
    pub reason: SelectionReason,
    /// What the caller asked for, if anything. `None` mirrors the
    /// CLI's `--ai-provider` being absent. Echoed back so the
    /// orchestrator can log "fallback from `local:llama`".
    pub requested: Option<String>,
}

/// Why [`select_provider`] returned a particular provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReason {
    /// `--no-ai` was set. The provider is unconditionally
    /// [`NullProvider`].
    Disabled,
    /// `--ai-provider` was absent and `--no-ai` was unset. Default
    /// to [`NullProvider`].
    Default,
    /// Caller asked for a known provider and we routed to it.
    Selected,
    /// Caller asked for a provider that doesn't exist yet (B4.2
    /// hasn't landed local model adapters). The orchestrator should
    /// warn so the user knows their request was downgraded.
    Fallback,
}

impl SelectionReason {
    /// Short kebab-case tag for logs.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Default => "default",
            Self::Selected => "selected",
            Self::Fallback => "fallback",
        }
    }
}

/// Map the CLI's `--no-ai` + `--ai-provider` flags onto a concrete
/// provider.
///
/// The mapping table:
///
/// | `no_ai` | `requested`                              | result          | reason     |
/// |---------|------------------------------------------|-----------------|------------|
/// | `true`  | _any_                                    | `Null`          | `Disabled` |
/// | `false` | `None`                                   | `Null`          | `Default`  |
/// | `false` | `Some("null" / "none")`                  | `Null`          | `Selected` |
/// | `false` | `Some("local" / "local:stub")`           | `LocalProvider` | `Selected` |
/// | `false` | `Some(other, incl. `local:llama`, etc.)` | `Null`          | `Fallback` |
///
/// `local` (no suffix) is the alias for the default local backend; at
/// B4.2 that is the rule-based stub. Future HTTP-backed local adapters
/// (B4.6 follow-up) will be routed by suffix (`local:llama`,
/// `local:ollama`).
#[must_use]
pub fn select_provider(no_ai: bool, requested: Option<&str>) -> ProviderSelection {
    if no_ai {
        return ProviderSelection {
            provider: Box::new(NullProvider),
            reason: SelectionReason::Disabled,
            requested: requested.map(str::to_string),
        };
    }
    match requested {
        None => ProviderSelection {
            provider: Box::new(NullProvider),
            reason: SelectionReason::Default,
            requested: None,
        },
        Some(name) if name.eq_ignore_ascii_case("null") || name.eq_ignore_ascii_case("none") => {
            ProviderSelection {
                provider: Box::new(NullProvider),
                reason: SelectionReason::Selected,
                requested: Some(name.to_string()),
            }
        }
        Some(name)
            if name.eq_ignore_ascii_case("local") || name.eq_ignore_ascii_case("local:stub") =>
        {
            ProviderSelection {
                provider: Box::new(LocalProvider::stub()),
                reason: SelectionReason::Selected,
                requested: Some(name.to_string()),
            }
        }
        Some(name) => {
            // Local HTTP adapters (`local:llama`, `local:ollama`) and
            // remote APIs are reserved for B4.6. They land here as
            // `Fallback` so the orchestrator can warn the user once
            // that their requested provider was downgraded to null
            // (NFR-9: the determinism corridor stays intact because
            // the actual provider is always `NullProvider` in this
            // branch).
            ProviderSelection {
                provider: Box::new(NullProvider),
                reason: SelectionReason::Fallback,
                requested: Some(name.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{Prompt, PromptKind};
    use crate::{Delta, ProposerContext, SymbolRef};
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode, Source};

    fn fixture_prompt() -> Prompt {
        Prompt::new(PromptKind::NameSuggestion, "describe sub_1040")
    }

    fn fixture_bundle() -> EvidenceBundle {
        let mut g = EvidenceGraph::new();
        EvidenceBundle::from_iter([g.add_node(EvidenceNode::Instruction(42))])
    }

    fn fixture_delta() -> Delta {
        let prompt = fixture_prompt();
        let bundle = fixture_bundle();
        Delta::rename_symbol(
            SymbolRef(7),
            "checksum",
            ProposerContext {
                prompt: &prompt,
                evidence: &bundle,
                confidence: Confidence::new(0.5, Source::Speculative),
                model_id: "test",
                seed: 0,
            },
        )
        .expect("constructable")
    }

    #[test]
    fn null_provider_advertises_correctly() {
        let p = NullProvider;
        assert_eq!(p.name(), "null");
        assert!(p.is_local());
    }

    #[test]
    fn null_provider_returns_empty_proposals_for_any_input() {
        let p = NullProvider;
        let prompt = fixture_prompt();
        let bundle = fixture_bundle();
        let deltas = p.propose(&prompt, &bundle).expect("ok");
        assert!(deltas.is_empty());
    }

    #[test]
    fn null_provider_returns_empty_for_empty_bundle_too() {
        let p = NullProvider;
        let prompt = fixture_prompt();
        let empty = EvidenceBundle::new();
        assert!(p.propose(&prompt, &empty).expect("ok").is_empty());
    }

    #[test]
    fn echo_provider_replays_canned_deltas() {
        let provider = EchoProvider::new(vec![fixture_delta()]);
        assert_eq!(provider.name(), "echo");
        assert!(provider.is_local());
        let prompt = fixture_prompt();
        let bundle = fixture_bundle();
        let first = provider.propose(&prompt, &bundle).expect("ok");
        let second = provider.propose(&prompt, &bundle).expect("ok");
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(provider.len(), 1);
        assert!(!provider.is_empty());
    }

    #[test]
    fn echo_provider_with_custom_name_round_trips() {
        let provider = EchoProvider::with_name("offline:v1", vec![]);
        assert_eq!(provider.name(), "offline:v1");
        assert!(provider.is_empty());
    }

    #[test]
    fn select_provider_returns_null_when_no_ai_set() {
        let sel = select_provider(true, Some("local:llama"));
        assert_eq!(sel.reason, SelectionReason::Disabled);
        assert_eq!(sel.provider.name(), "null");
        assert_eq!(sel.requested.as_deref(), Some("local:llama"));
        assert_eq!(sel.reason.tag(), "disabled");
    }

    #[test]
    fn select_provider_returns_null_by_default() {
        let sel = select_provider(false, None);
        assert_eq!(sel.reason, SelectionReason::Default);
        assert_eq!(sel.provider.name(), "null");
        assert!(sel.requested.is_none());
    }

    #[test]
    fn select_provider_routes_null_and_none_to_null_as_selected() {
        for name in ["null", "NULL", "none", "None"] {
            let sel = select_provider(false, Some(name));
            assert_eq!(sel.reason, SelectionReason::Selected);
            assert_eq!(sel.provider.name(), "null");
            assert_eq!(sel.requested.as_deref(), Some(name));
        }
    }

    #[test]
    fn select_provider_falls_back_when_unknown_provider_requested() {
        let sel = select_provider(false, Some("local:llama"));
        assert_eq!(sel.reason, SelectionReason::Fallback);
        assert_eq!(sel.provider.name(), "null");
        assert_eq!(sel.requested.as_deref(), Some("local:llama"));
    }

    #[test]
    fn select_provider_routes_local_alias_to_local_stub() {
        // B4.2: `local` (no suffix) and `local:stub` both resolve to the
        // rule-based local provider. The selected reason fires (not
        // Default and not Fallback) so the orchestrator stays silent.
        for name in ["local", "LOCAL", "local:stub", "LOCAL:STUB"] {
            let sel = select_provider(false, Some(name));
            assert_eq!(sel.reason, SelectionReason::Selected, "name = {name}");
            assert_eq!(sel.provider.name(), "local:stub", "name = {name}");
            assert!(sel.provider.is_local(), "name = {name}");
            assert_eq!(sel.requested.as_deref(), Some(name));
        }
    }

    #[test]
    fn select_provider_keeps_local_llama_on_fallback_until_b4_6() {
        // `local:llama` / `local:ollama` are reserved for B4.6's HTTP
        // adapter — at B4.2 they still downgrade to null so the run
        // succeeds. The orchestrator logs a warn so the user knows.
        for name in ["local:llama", "local:ollama"] {
            let sel = select_provider(false, Some(name));
            assert_eq!(sel.reason, SelectionReason::Fallback, "name = {name}");
            assert_eq!(sel.provider.name(), "null", "name = {name}");
        }
    }

    #[test]
    fn select_provider_local_routes_through_no_ai() {
        // `--no-ai` outranks every requested provider, including the
        // new `local` alias. The provider remains `null` and the
        // requested name is preserved for the warn message.
        let sel = select_provider(true, Some("local"));
        assert_eq!(sel.reason, SelectionReason::Disabled);
        assert_eq!(sel.provider.name(), "null");
        assert_eq!(sel.requested.as_deref(), Some("local"));
    }

    #[test]
    fn ai_provider_is_object_safe() {
        // If this compiles, the trait is dyn-compatible. The B4.5
        // orchestrator stores providers behind `Box<dyn AiProvider>`,
        // so the bound is load-bearing.
        let providers: Vec<Box<dyn AiProvider>> =
            vec![Box::new(NullProvider), Box::new(EchoProvider::new(vec![]))];
        for p in providers {
            assert!(p.is_local());
        }
    }

    #[test]
    fn selection_reason_tags_are_distinct_kebab_case() {
        let tags = [
            SelectionReason::Disabled.tag(),
            SelectionReason::Default.tag(),
            SelectionReason::Selected.tag(),
            SelectionReason::Fallback.tag(),
        ];
        for t in tags {
            assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len());
    }
}
