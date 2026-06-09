//! Closed enum of [`Delta`] kinds — the only mutations an AI provider
//! is allowed to propose against the Semantic IR (ARCHITECTURE §9).
//!
//! Every delta is constructed through a `Delta::*` helper. The helpers
//! clamp the confidence source to [`dac_core::Source::Speculative`]
//! (I-3) and require a non-empty evidence list (I-2). Real-model
//! providers cannot bypass the helpers because the metadata field is
//! `pub(crate)`-only — `Delta::*` is the only public path in. CLI
//! ingress re-checks via [`assert_speculative`] as defence in depth.

use dac_core::{Confidence, EvidenceId, Source};

use crate::prompt::Prompt;
use crate::EvidenceBundle;

/// Opaque handle to a recovered symbol the model may rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolRef(pub u64);

/// Opaque handle to an SSA slot / value the model may retype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SlotRef(pub u64);

/// Opaque handle to a Semantic IR region (basic block, structured
/// loop body, function body, …) the model may annotate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionRef(pub u64);

/// One proposed struct field for a [`Delta::SuggestStructLayout`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldSuggestion {
    pub name: String,
    pub ty: String,
    pub offset: u64,
}

/// Provenance attached to every [`Delta`] — what the model saw and
/// who produced the answer. Mirrors the FR-37 record-keeping contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DeltaMetadata {
    pub(crate) confidence: Confidence,
    pub(crate) prompt_hash: [u8; 32],
    pub(crate) model_id: String,
    pub(crate) seed: u64,
    pub(crate) evidence: Vec<EvidenceId>,
}

impl DeltaMetadata {
    /// The recovered fact's confidence. Always [`Source::Speculative`].
    #[must_use]
    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// FR-37: the prompt hash the proposer was conditioned on.
    #[must_use]
    pub fn prompt_hash(&self) -> [u8; 32] {
        self.prompt_hash
    }

    /// FR-37: the proposer's stable model id (e.g. `"null"`,
    /// `"echo"`).
    #[must_use]
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// FR-37: seed used to draw the response. `0` for deterministic
    /// providers (null, echo).
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Evidence handles the proposer claims to have conditioned on.
    /// Always non-empty.
    #[must_use]
    pub fn evidence(&self) -> &[EvidenceId] {
        &self.evidence
    }
}

/// Closed set of proposed Semantic IR changes (ARCHITECTURE §9).
///
/// Producing a delta with a non-`Speculative` source would let an AI
/// proposal masquerade as Observed evidence and corrupt the
/// confidence lattice; the helpers reject it. The metadata is
/// `pub(crate)` so that even crates downstream cannot lift it to
/// Observed by editing fields.
#[derive(Debug, Clone, PartialEq)]
pub enum Delta {
    RenameSymbol {
        target: SymbolRef,
        new_name: String,
        meta: DeltaMetadata,
    },
    RetypeSlot {
        target: SlotRef,
        new_type: String,
        meta: DeltaMetadata,
    },
    SuggestStructLayout {
        region: RegionRef,
        fields: Vec<StructFieldSuggestion>,
        meta: DeltaMetadata,
    },
    SuggestIdiom {
        region: RegionRef,
        idiom: String,
        meta: DeltaMetadata,
    },
    AnnotateRegion {
        region: RegionRef,
        comment: String,
        meta: DeltaMetadata,
    },
}

/// Failure during [`Delta`] construction. Every variant is a violation
/// of an invariant the constructors guard so the orchestrator never
/// sees a malformed proposal in normal flow — these are programmer
/// errors at the provider layer, not transport failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaBuildError {
    /// I-3 violation: caller tried to construct a delta with
    /// `Source != Speculative`.
    NonSpeculativeSource(Source),
    /// I-2 violation: caller passed an empty [`EvidenceBundle`].
    EmptyEvidenceBundle,
}

impl std::fmt::Display for DeltaBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonSpeculativeSource(s) => write!(
                f,
                "delta confidence source must be Speculative, got {}",
                s.name(),
            ),
            Self::EmptyEvidenceBundle => {
                write!(f, "delta must cite at least one evidence handle")
            }
        }
    }
}

impl std::error::Error for DeltaBuildError {}

/// Defence-in-depth check that a delta's metadata claims a
/// `Speculative` source.
///
/// Re-checked at CLI ingress so a future binary path that constructs
/// a [`DeltaMetadata`] outside [`make_metadata`] (e.g. via deserialisation
/// in M5) cannot smuggle an Observed-tagged delta into the lattice.
///
/// Returns `Err` instead of panicking so the orchestrator can treat it
/// as a recoverable rejection.
pub fn assert_speculative(delta: &Delta) -> Result<(), DeltaBuildError> {
    let src = delta.meta().confidence().source();
    if src == Source::Speculative {
        Ok(())
    } else {
        Err(DeltaBuildError::NonSpeculativeSource(src))
    }
}

impl Delta {
    /// Borrow the delta's metadata regardless of variant.
    #[must_use]
    pub fn meta(&self) -> &DeltaMetadata {
        match self {
            Self::RenameSymbol { meta, .. }
            | Self::RetypeSlot { meta, .. }
            | Self::SuggestStructLayout { meta, .. }
            | Self::SuggestIdiom { meta, .. }
            | Self::AnnotateRegion { meta, .. } => meta,
        }
    }

    /// Stable kebab-case tag for the variant. Used by manifest /
    /// report rendering and by [`crate::prompt::PromptKind::tag`]
    /// validation in B4.3.
    #[must_use]
    pub const fn kind_tag(&self) -> &'static str {
        match self {
            Self::RenameSymbol { .. } => "rename-symbol",
            Self::RetypeSlot { .. } => "retype-slot",
            Self::SuggestStructLayout { .. } => "suggest-struct-layout",
            Self::SuggestIdiom { .. } => "suggest-idiom",
            Self::AnnotateRegion { .. } => "annotate-region",
        }
    }

    /// Build a [`Delta::RenameSymbol`]. Confidence is clamped to
    /// `Speculative` per I-3.
    pub fn rename_symbol(
        target: SymbolRef,
        new_name: impl Into<String>,
        meta: ProposerContext<'_>,
    ) -> Result<Self, DeltaBuildError> {
        let meta = make_metadata(meta)?;
        Ok(Self::RenameSymbol {
            target,
            new_name: new_name.into(),
            meta,
        })
    }

    /// Build a [`Delta::RetypeSlot`].
    pub fn retype_slot(
        target: SlotRef,
        new_type: impl Into<String>,
        meta: ProposerContext<'_>,
    ) -> Result<Self, DeltaBuildError> {
        let meta = make_metadata(meta)?;
        Ok(Self::RetypeSlot {
            target,
            new_type: new_type.into(),
            meta,
        })
    }

    /// Build a [`Delta::SuggestStructLayout`].
    pub fn suggest_struct_layout(
        region: RegionRef,
        fields: Vec<StructFieldSuggestion>,
        meta: ProposerContext<'_>,
    ) -> Result<Self, DeltaBuildError> {
        let meta = make_metadata(meta)?;
        Ok(Self::SuggestStructLayout {
            region,
            fields,
            meta,
        })
    }

    /// Build a [`Delta::SuggestIdiom`].
    pub fn suggest_idiom(
        region: RegionRef,
        idiom: impl Into<String>,
        meta: ProposerContext<'_>,
    ) -> Result<Self, DeltaBuildError> {
        let meta = make_metadata(meta)?;
        Ok(Self::SuggestIdiom {
            region,
            idiom: idiom.into(),
            meta,
        })
    }

    /// Build a [`Delta::AnnotateRegion`].
    pub fn annotate_region(
        region: RegionRef,
        comment: impl Into<String>,
        meta: ProposerContext<'_>,
    ) -> Result<Self, DeltaBuildError> {
        let meta = make_metadata(meta)?;
        Ok(Self::AnnotateRegion {
            region,
            comment: comment.into(),
            meta,
        })
    }
}

/// Per-call provenance handed to every [`Delta::*`] constructor.
///
/// Borrows the prompt + bundle so providers don't have to clone them
/// per delta. Confidence is passed as a value; the constructor checks
/// the source field and rejects non-Speculative inputs.
#[derive(Debug, Clone, Copy)]
pub struct ProposerContext<'a> {
    pub prompt: &'a Prompt,
    pub evidence: &'a EvidenceBundle,
    pub confidence: Confidence,
    pub model_id: &'a str,
    pub seed: u64,
}

fn make_metadata(ctx: ProposerContext<'_>) -> Result<DeltaMetadata, DeltaBuildError> {
    if ctx.confidence.source() != Source::Speculative {
        return Err(DeltaBuildError::NonSpeculativeSource(
            ctx.confidence.source(),
        ));
    }
    if ctx.evidence.is_empty() {
        return Err(DeltaBuildError::EmptyEvidenceBundle);
    }
    Ok(DeltaMetadata {
        confidence: ctx.confidence,
        prompt_hash: ctx.prompt.digest(),
        model_id: ctx.model_id.to_string(),
        seed: ctx.seed,
        evidence: ctx.evidence.ids().to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{Prompt, PromptKind};
    use dac_core::{EvidenceGraph, EvidenceNode};

    fn fixture() -> (Prompt, EvidenceBundle, Confidence) {
        let mut g = EvidenceGraph::new();
        let e = g.add_node(EvidenceNode::Instruction(42));
        let bundle = EvidenceBundle::from_iter([e]);
        let prompt = Prompt::new(PromptKind::NameSuggestion, "describe sub_1040");
        let conf = Confidence::new(0.7, Source::Speculative);
        (prompt, bundle, conf)
    }

    fn ctx<'a>(
        prompt: &'a Prompt,
        bundle: &'a EvidenceBundle,
        conf: Confidence,
    ) -> ProposerContext<'a> {
        ProposerContext {
            prompt,
            evidence: bundle,
            confidence: conf,
            model_id: "test",
            seed: 0,
        }
    }

    #[test]
    fn rename_symbol_records_full_provenance() {
        let (p, b, c) = fixture();
        let d = Delta::rename_symbol(SymbolRef(7), "checksum", ctx(&p, &b, c)).expect("ok");
        let meta = d.meta();
        assert_eq!(meta.confidence(), c);
        assert_eq!(meta.prompt_hash(), p.digest());
        assert_eq!(meta.model_id(), "test");
        assert_eq!(meta.seed(), 0);
        assert_eq!(meta.evidence(), b.ids());
        assert_eq!(d.kind_tag(), "rename-symbol");
    }

    #[test]
    fn every_variant_has_a_distinct_kind_tag() {
        let (p, b, c) = fixture();
        let renames = Delta::rename_symbol(SymbolRef(1), "x", ctx(&p, &b, c)).expect("ok");
        let retype = Delta::retype_slot(SlotRef(1), "u32", ctx(&p, &b, c)).expect("ok");
        let layout =
            Delta::suggest_struct_layout(RegionRef(1), vec![], ctx(&p, &b, c)).expect("ok");
        let idiom = Delta::suggest_idiom(RegionRef(1), "memcpy", ctx(&p, &b, c)).expect("ok");
        let note = Delta::annotate_region(RegionRef(1), "fast path", ctx(&p, &b, c)).expect("ok");
        let tags = [
            renames.kind_tag(),
            retype.kind_tag(),
            layout.kind_tag(),
            idiom.kind_tag(),
            note.kind_tag(),
        ];
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len(), "tags must be distinct: {tags:?}");
    }

    #[test]
    fn rejects_non_speculative_confidence() {
        let (p, b, _c) = fixture();
        let observed = Confidence::new(1.0, Source::Observed);
        let err = Delta::rename_symbol(SymbolRef(1), "x", ctx(&p, &b, observed)).unwrap_err();
        assert_eq!(err, DeltaBuildError::NonSpeculativeSource(Source::Observed));
    }

    #[test]
    fn rejects_empty_evidence_bundle() {
        let (p, _b, c) = fixture();
        let empty = EvidenceBundle::new();
        let err = Delta::rename_symbol(SymbolRef(1), "x", ctx(&p, &empty, c)).unwrap_err();
        assert_eq!(err, DeltaBuildError::EmptyEvidenceBundle);
    }

    #[test]
    fn assert_speculative_passes_on_well_formed_delta() {
        let (p, b, c) = fixture();
        let d = Delta::annotate_region(RegionRef(1), "note", ctx(&p, &b, c)).expect("ok");
        assert!(assert_speculative(&d).is_ok());
    }

    #[test]
    fn delta_build_error_display_is_human_readable() {
        let e = DeltaBuildError::NonSpeculativeSource(Source::Observed);
        assert!(e.to_string().contains("Speculative"));
        let e = DeltaBuildError::EmptyEvidenceBundle;
        assert!(e.to_string().contains("evidence"));
    }
}
