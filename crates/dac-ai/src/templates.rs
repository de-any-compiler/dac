//! Versioned prompt templates (spec §13.8).
//!
//! Each [`PromptTemplate`] pins a `(template_id, version)` pair against
//! a closed [`PromptKind`] and a deterministic body. The body is a tiny
//! format-style string with `{name}` placeholders; [`render`] substitutes
//! a caller-provided parameter list and produces a [`Prompt`] whose
//! [`Prompt::digest`] is then folded into every resulting [`crate::Delta`]'s
//! [`crate::DeltaMetadata::prompt_hash`] (FR-37).
//!
//! Templates are looked up by their stable `id` slug (e.g.
//! `"pipeline-summary"` for the orchestrator's one-shot prompt). Bumping
//! a template's wording requires bumping its [`PromptTemplate::version`]
//! field — the version number is folded into the rendered text so a
//! prompt phrased differently does *not* collide with the previous
//! revision in the manifest's prompt-hash field (NFR-10 reproducibility).
//!
//! Spec §13.8: "prompt templates versioned alongside the passes that
//! consume them." Passes consume templates by looking them up here at
//! call sites; passes that change the prompt wording bump the version
//! at the same time as the pass landing — that single coupling is what
//! makes the prompt set reproducible across `dac` releases.

use crate::prompt::{Prompt, PromptKind};

/// One versioned prompt template.
///
/// The body uses a tiny `{name}` placeholder syntax — see [`render`].
/// Placeholders that are not provided by the caller render verbatim
/// (`{unknown}` stays as `{unknown}`) so an honest template author can
/// always grep the corpus for templates whose substitutions misfired.
#[derive(Debug, Clone, Copy)]
pub struct PromptTemplate {
    /// Stable kebab-case slug. Matches the corresponding pass name where
    /// useful — `"pipeline-summary"` is the orchestrator's once-per-run
    /// summary prompt; `"rename-symbol"` is the per-function naming
    /// prompt B4.5 will consume.
    pub id: &'static str,
    /// Template revision. Bumped any time the body changes so old and
    /// new wordings hash differently (FR-37, NFR-10).
    pub version: u32,
    /// Which [`PromptKind`] this template targets. The rendered prompt's
    /// `kind` is taken verbatim from this field.
    pub kind: PromptKind,
    /// The template body with `{name}` placeholders. The version number
    /// is automatically prefixed by [`render`] (`v<version>\n…body…`) so
    /// a deliberate wording change is observable in the prompt hash even
    /// when the body itself happens to render the same characters.
    pub body: &'static str,
}

/// Look up a template by its stable id.
///
/// Returns `None` if no template with that id is registered. Callers
/// that need a template at compile time should hard-code the id; the
/// lookup is intentionally O(n) over the small static registry rather
/// than a hash map so the registry stays trivially auditable.
#[must_use]
pub fn lookup(id: &str) -> Option<&'static PromptTemplate> {
    REGISTERED.iter().find(|t| t.id == id)
}

/// All templates registered with the crate.
#[must_use]
pub fn all() -> &'static [PromptTemplate] {
    REGISTERED
}

/// Render a template against a parameter list, producing a [`Prompt`]
/// the caller can pass to [`crate::AiProvider::propose`].
///
/// Substitution is positional-name based: every `{name}` placeholder in
/// the template body is replaced with the first matching `(name, value)`
/// pair from `params`. Missing placeholders pass through verbatim — a
/// template that references `{arch}` but is rendered without an `arch`
/// param still produces a valid prompt, just with a literal `{arch}`
/// that a reviewer can grep for.
#[must_use]
pub fn render(template: &PromptTemplate, params: &[(&str, &str)]) -> Prompt {
    let mut text = format!("v{}\n{}", template.version, template.body);
    for (name, value) in params {
        let needle = format!("{{{name}}}");
        text = text.replace(&needle, value);
    }
    Prompt::new(template.kind, text)
}

/// The static registry. Add new templates here; bumping `version`
/// when wording changes is the only mechanism that keeps prompt
/// hashes stable across the codebase's lifetime (NFR-10).
const REGISTERED: &[PromptTemplate] = &[
    PromptTemplate {
        id: "pipeline-summary",
        version: 1,
        kind: PromptKind::Annotation,
        body: "dac pipeline summary for {input}\nformat: {format}\narch: {arch}\nsize: {size}",
    },
    PromptTemplate {
        id: "rename-symbol",
        version: 1,
        kind: PromptKind::NameSuggestion,
        body: "Propose a short, human-readable name for the function at {address} in {input}.\nrecovered name: {recovered}",
    },
    PromptTemplate {
        id: "annotate-region",
        version: 1,
        kind: PromptKind::Annotation,
        body: "Annotate region {region} in {input} with a short, human-readable comment.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_registered_template_by_id() {
        let t = lookup("pipeline-summary").expect("registered");
        assert_eq!(t.id, "pipeline-summary");
        assert_eq!(t.kind, PromptKind::Annotation);
        assert_eq!(t.version, 1);
    }

    #[test]
    fn lookup_returns_none_for_unknown_id() {
        assert!(lookup("does-not-exist").is_none());
    }

    #[test]
    fn registry_ids_are_unique_and_kebab_case() {
        let mut seen = std::collections::BTreeSet::new();
        for t in all() {
            assert!(seen.insert(t.id), "duplicate template id: {}", t.id);
            assert!(
                t.id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "non-kebab-case template id: {}",
                t.id,
            );
        }
    }

    #[test]
    fn render_substitutes_named_placeholders() {
        let t = lookup("pipeline-summary").expect("registered");
        let p = render(
            t,
            &[
                ("input", "hello-x86_64"),
                ("format", "ELF"),
                ("arch", "x86-64"),
                ("size", "17296"),
            ],
        );
        assert!(p.text.contains("hello-x86_64"));
        assert!(p.text.contains("ELF"));
        assert!(p.text.contains("x86-64"));
        assert!(p.text.contains("17296"));
        // The version prefix is observable in the rendered prompt so the
        // template revision is part of the prompt-hash input (NFR-10).
        assert!(p.text.starts_with("v1\n"));
    }

    #[test]
    fn render_passes_unknown_placeholders_through() {
        let t = lookup("pipeline-summary").expect("registered");
        // Only `input` substituted — `format`, `arch`, `size` stay as
        // literal `{name}` so a reviewer grepping for unsubstituted
        // placeholders can find them.
        let p = render(t, &[("input", "test")]);
        assert!(p.text.contains("test"));
        assert!(p.text.contains("{format}"));
        assert!(p.text.contains("{arch}"));
        assert!(p.text.contains("{size}"));
    }

    #[test]
    fn render_with_same_params_is_deterministic() {
        let t = lookup("pipeline-summary").expect("registered");
        let params = [("input", "x"), ("format", "ELF")];
        assert_eq!(render(t, &params), render(t, &params));
    }

    #[test]
    fn version_bump_changes_rendered_prompt_hash() {
        let base = PromptTemplate {
            id: "test",
            version: 1,
            kind: PromptKind::Annotation,
            body: "same body",
        };
        let bumped = PromptTemplate { version: 2, ..base };
        let a = render(&base, &[]);
        let b = render(&bumped, &[]);
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn rename_symbol_template_is_name_suggestion_kind() {
        let t = lookup("rename-symbol").expect("registered");
        assert_eq!(t.kind, PromptKind::NameSuggestion);
    }
}
