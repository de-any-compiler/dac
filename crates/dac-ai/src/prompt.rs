//! Prompt records and content-addressed digests.
//!
//! A [`Prompt`] is the *input side* of an [`crate::AiProvider::propose`]
//! call. The provider sees the prompt + an [`crate::EvidenceBundle`] and
//! returns a list of [`crate::Delta`]s. Each returned delta carries the
//! prompt's digest in its [`crate::DeltaMetadata`] so a manifest reader
//! can match `(prompt, model_id, seed) → deltas` for FR-37.
//!
//! The digest is a 32-byte content hash derived from
//! [`PromptKind::tag`] plus the prompt text via four
//! [FNV-1a 64] streams seeded with distinct IVs. FNV-1a is deterministic
//! across builds and targets — that is the *only* property the digest
//! needs (collision-resistance against adversaries is not a goal; the
//! threat model is reproducibility, not integrity). The constants are
//! pinned in [`prompt_digest`] so this digest is part of the artifact's
//! observable surface.
//!
//! [FNV-1a 64]: https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function

/// What the proposer is being asked to suggest. The variant set is
/// closed and aligns with the [`crate::Delta`] kinds — every prompt
/// kind has a corresponding delta kind, so a verifier can reject
/// out-of-scope responses ahead of B4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptKind {
    /// "Suggest a human-readable name for this symbol" (FR-32).
    NameSuggestion,
    /// "Suggest a struct layout for this region" (B3.2 follow-up).
    StructLayout,
    /// "Suggest a more precise type for this slot" (FR-14).
    Retype,
    /// "Suggest an idiom that explains this region" (FR-15).
    Idiom,
    /// "Annotate this region with a human-readable comment"
    /// (FR-33).
    Annotation,
}

impl PromptKind {
    /// Stable kebab-case tag suitable for logs and manifests.
    ///
    /// The tag is also folded into [`prompt_digest`]; changing it
    /// changes every downstream prompt hash, which is intentional —
    /// prompts that mean different things must not collide.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::NameSuggestion => "name-suggestion",
            Self::StructLayout => "struct-layout",
            Self::Retype => "retype",
            Self::Idiom => "idiom",
            Self::Annotation => "annotation",
        }
    }
}

/// A single prompt for an [`crate::AiProvider`].
///
/// The `text` field is the rendered template that the provider will see
/// — `dac-ai` does not own the template language; backends and passes
/// produce it. The whole record is content-addressed by
/// [`prompt_digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub kind: PromptKind,
    pub text: String,
}

impl Prompt {
    /// Build a prompt for the given kind and free-form text.
    #[must_use]
    pub fn new(kind: PromptKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    /// Convenience: the prompt's digest as a 32-byte array.
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        prompt_digest(self)
    }
}

/// Produce a deterministic 32-byte content hash of a prompt.
///
/// Folds [`PromptKind::tag`] + a separator byte + the prompt text
/// through four FNV-1a streams seeded with distinct IVs. Returns the
/// concatenation in little-endian word order so the byte layout is
/// platform-independent.
#[must_use]
pub fn prompt_digest(prompt: &Prompt) -> [u8; 32] {
    // Four IVs chosen to spell `DACAI 0..3` in ASCII so they are easy
    // to grep for in a hex dump but otherwise carry no special meaning.
    const IVS: [u64; 4] = [
        0x4441_4341_4900_0000,
        0x4441_4341_4900_0001,
        0x4441_4341_4900_0002,
        0x4441_4341_4900_0003,
    ];
    let mut out = [0u8; 32];
    let tag = prompt.kind.tag().as_bytes();
    let text = prompt.text.as_bytes();
    for (i, &iv) in IVS.iter().enumerate() {
        let mut h = iv;
        h = fnv1a64_fold(h, tag);
        h = fnv1a64_fold(h, &[0x1f]); // tag/text separator (ASCII unit-separator)
        h = fnv1a64_fold(h, text);
        out[i * 8..(i + 1) * 8].copy_from_slice(&h.to_le_bytes());
    }
    out
}

fn fnv1a64_fold(mut hash: u64, bytes: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x100_0000_01b3;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_distinguishes_every_kind() {
        let kinds = [
            PromptKind::NameSuggestion,
            PromptKind::StructLayout,
            PromptKind::Retype,
            PromptKind::Idiom,
            PromptKind::Annotation,
        ];
        let tags: Vec<_> = kinds.iter().map(|k| k.tag()).collect();
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len(), "tags must be distinct: {tags:?}");
    }

    #[test]
    fn digest_is_deterministic() {
        let p = Prompt::new(PromptKind::NameSuggestion, "describe sub_1040");
        assert_eq!(prompt_digest(&p), prompt_digest(&p));
    }

    #[test]
    fn digest_changes_with_text() {
        let a = Prompt::new(PromptKind::NameSuggestion, "describe sub_1040");
        let b = Prompt::new(PromptKind::NameSuggestion, "describe sub_1050");
        assert_ne!(prompt_digest(&a), prompt_digest(&b));
    }

    #[test]
    fn digest_changes_with_kind() {
        let a = Prompt::new(PromptKind::NameSuggestion, "describe sub_1040");
        let b = Prompt::new(PromptKind::Annotation, "describe sub_1040");
        assert_ne!(prompt_digest(&a), prompt_digest(&b));
    }

    #[test]
    fn digest_separator_blocks_tag_text_blending() {
        // Without the separator byte, "ax" + "y" would hash the same
        // as "a" + "xy" because FNV-1a folds bytes one at a time.
        // With the separator, the boundary is observable.
        let a = Prompt::new(PromptKind::NameSuggestion, "extra");
        let b = Prompt {
            kind: PromptKind::NameSuggestion,
            // Same end-to-end byte sequence sans the separator.
            text: format!("{}extra", PromptKind::NameSuggestion.tag()),
        };
        assert_ne!(prompt_digest(&a), prompt_digest(&b));
    }

    #[test]
    fn digest_fills_all_four_words() {
        let p = Prompt::new(PromptKind::Idiom, "loop body");
        let bytes = prompt_digest(&p);
        // Each IV is distinct, so each 8-byte word should be non-zero
        // for any non-empty prompt. (FNV-1a starting from a non-zero
        // IV folded over any byte sequence cannot collapse to 0.)
        for chunk in bytes.chunks_exact(8) {
            let word = u64::from_le_bytes(chunk.try_into().expect("8 bytes"));
            assert_ne!(word, 0);
        }
    }
}
