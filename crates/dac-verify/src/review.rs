//! Review-mode artifact: collect every AI proposal as a structured
//! before/after diff record (spec §13.6, FR-33).
//!
//! `--ai-review` collects each [`Delta`] together with the verifier's
//! [`VerifyOutcome`] and a snapshot of the target's recorded state
//! from a [`KnownFacts`] world into a [`ReviewLog`]. The log is then
//! rendered to a human-readable, deterministic plaintext block by
//! [`render_review`]. The orchestrator never applies the proposals;
//! the artifact is purely informational (ARCHITECTURE §13).
//!
//! The renderer is a pure function of the log — no I/O, no global
//! state, no PRNG, no time reads (NFR-9). Repeated runs on the same
//! pipeline produce byte-identical output, which is what
//! `--ai-review` ships against in its done-when criterion.

use dac_ai::{Delta, RegionRef, SlotRef, StructFieldSuggestion, SymbolRef};
use dac_core::{EvidenceId, Source};

use crate::verify::{DeltaRejection, TargetKind, VerifyMode, VerifyOutcome};
use crate::world::{KnownFacts, SlotType};

/// Which kind of handle an entry refers to, plus the raw id so the
/// rendered header can cite it (`SymbolRef(7)`, `SlotRef(3)`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetDescriptor {
    Symbol(SymbolRef),
    Slot(SlotRef),
    Region(RegionRef),
}

impl TargetDescriptor {
    /// Stable kebab-case tag matching [`TargetKind::tag`].
    #[must_use]
    pub const fn kind(self) -> TargetKind {
        match self {
            Self::Symbol(_) => TargetKind::Symbol,
            Self::Slot(_) => TargetKind::Slot,
            Self::Region(_) => TargetKind::Region,
        }
    }
}

/// Snapshot of the target's recorded state at the moment the delta
/// was judged. Lets the renderer print the diff's `-` side without
/// needing the world at render time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentState {
    /// Target is not in the world model. The diff renders the `-`
    /// side as `(unknown target)` so the reviewer can see the
    /// rejection grounds at a glance.
    Unknown,
    Symbol {
        name: String,
        source: Source,
    },
    Slot {
        ty: SlotType,
        source: Source,
    },
    Region {
        source: Source,
    },
}

impl CurrentState {
    fn from_world(world: &KnownFacts, target: TargetDescriptor) -> Self {
        match target {
            TargetDescriptor::Symbol(id) => match world.symbol(id) {
                Some(s) => Self::Symbol {
                    name: s.name.clone(),
                    source: s.source,
                },
                None => Self::Unknown,
            },
            TargetDescriptor::Slot(id) => match world.slot(id) {
                Some(s) => Self::Slot {
                    ty: s.ty.clone(),
                    source: s.source,
                },
                None => Self::Unknown,
            },
            TargetDescriptor::Region(id) => match world.region(id) {
                Some(r) => Self::Region { source: r.source },
                None => Self::Unknown,
            },
        }
    }
}

/// The proposed change, decoupled from the [`Delta`] enum so the
/// renderer does not need to import `dac_ai` to format an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposedChange {
    Rename { new_name: String },
    Retype { new_type: String },
    StructLayout { fields: Vec<ProposedField> },
    Idiom { tag: String },
    Annotation { comment: String },
}

/// One proposed struct field, mirroring [`StructFieldSuggestion`] in
/// the rendered output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedField {
    pub name: String,
    pub ty: String,
    pub offset: u64,
}

/// Outcome of the verifier's judgement for a single entry.
///
/// Reject carries the rejection's stable tag plus a short detail
/// string so the rendered output can group by tag without re-deriving
/// it from the structured variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewOutcome {
    Accept,
    Reject { tag: &'static str, detail: String },
}

impl ReviewOutcome {
    fn from_verify(outcome: &VerifyOutcome) -> Self {
        match outcome {
            VerifyOutcome::Accept => Self::Accept,
            VerifyOutcome::Reject(reason) => Self::Reject {
                tag: reason.tag(),
                detail: rejection_detail(reason),
            },
        }
    }

    /// Stable kebab-case tag — `accept` or the rejection's `tag()`.
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject { tag, .. } => tag,
        }
    }
}

fn rejection_detail(reason: &DeltaRejection) -> String {
    match reason {
        DeltaRejection::UnknownTarget { kind } => format!("{} not in world", kind.tag()),
        DeltaRejection::NameCollision { existing, name } => {
            format!("name `{}` already owned by SymbolRef({})", name, existing.0)
        }
        DeltaRejection::InvalidIdentifier(name) => format!("`{name}` is not a C identifier"),
        DeltaRejection::RetypeNoPointerEvidence { current, requested } => format!(
            "current type {} has no pointer evidence; requested `{}`",
            current.tag(),
            requested
        ),
        DeltaRejection::InvalidStructLayout(msg) => msg.clone(),
        DeltaRejection::EmptyAnnotation => "annotation comment is empty".to_string(),
        DeltaRejection::EmptyIdiom => "idiom tag is empty".to_string(),
        DeltaRejection::StrictModeBlocksObserved { kind, source } => format!(
            "strict mode: {} target's source is {}",
            kind.tag(),
            source.name()
        ),
    }
}

/// One reviewed delta: who proposed it, what it proposed, what the
/// world currently records for the target, and the verifier's verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewEntry {
    pub kind: &'static str,
    pub target: TargetDescriptor,
    pub current: CurrentState,
    pub proposed: ProposedChange,
    pub confidence_score: f32,
    pub confidence_source: Source,
    pub evidence: Vec<EvidenceId>,
    pub outcome: ReviewOutcome,
}

/// The accumulated review log for one `--ai-review` run.
///
/// Entries are recorded in the order [`Self::record`] is called — the
/// orchestrator drives them through in the order the provider returned
/// them, which is itself deterministic for every shipping provider
/// (NFR-9). The log is otherwise inert: it never applies a delta and
/// never mutates the world.
#[derive(Debug, Clone)]
pub struct ReviewLog {
    provider: String,
    mode: VerifyMode,
    entries: Vec<ReviewEntry>,
    accepted: usize,
    rejected: usize,
}

impl ReviewLog {
    /// Build an empty log keyed off the provider name and mode.
    #[must_use]
    pub fn new(provider: impl Into<String>, mode: VerifyMode) -> Self {
        Self {
            provider: provider.into(),
            mode,
            entries: Vec::new(),
            accepted: 0,
            rejected: 0,
        }
    }

    /// Append one delta + outcome pair to the log. The world is
    /// consulted at append time so the rendered diff can show the
    /// recorded state of the target without consulting the world
    /// again at render time.
    pub fn record(&mut self, delta: &Delta, world: &KnownFacts, outcome: &VerifyOutcome) {
        let (target, proposed) = describe_delta(delta);
        let current = CurrentState::from_world(world, target);
        let review_outcome = ReviewOutcome::from_verify(outcome);
        match review_outcome {
            ReviewOutcome::Accept => self.accepted += 1,
            ReviewOutcome::Reject { .. } => self.rejected += 1,
        }
        let meta = delta.meta();
        self.entries.push(ReviewEntry {
            kind: delta.kind_tag(),
            target,
            current,
            proposed,
            confidence_score: meta.confidence().value(),
            confidence_source: meta.confidence().source(),
            evidence: meta.evidence().to_vec(),
            outcome: review_outcome,
        });
    }

    /// Borrow the entries in record order.
    #[must_use]
    pub fn entries(&self) -> &[ReviewEntry] {
        &self.entries
    }

    /// Provider name the log was opened for (e.g. `local:stub`).
    #[must_use]
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Verifier mode (lenient / strict) the log was opened for.
    #[must_use]
    pub fn mode(&self) -> VerifyMode {
        self.mode
    }

    /// Total number of deltas the verifier accepted.
    #[must_use]
    pub fn accepted(&self) -> usize {
        self.accepted
    }

    /// Total number of deltas the verifier rejected.
    #[must_use]
    pub fn rejected(&self) -> usize {
        self.rejected
    }

    /// Total number of deltas the log carries.
    #[must_use]
    pub fn total(&self) -> usize {
        self.entries.len()
    }
}

fn describe_delta(delta: &Delta) -> (TargetDescriptor, ProposedChange) {
    match delta {
        Delta::RenameSymbol {
            target, new_name, ..
        } => (
            TargetDescriptor::Symbol(*target),
            ProposedChange::Rename {
                new_name: new_name.clone(),
            },
        ),
        Delta::RetypeSlot {
            target, new_type, ..
        } => (
            TargetDescriptor::Slot(*target),
            ProposedChange::Retype {
                new_type: new_type.clone(),
            },
        ),
        Delta::SuggestStructLayout { region, fields, .. } => (
            TargetDescriptor::Region(*region),
            ProposedChange::StructLayout {
                fields: fields.iter().map(field_from_suggestion).collect(),
            },
        ),
        Delta::SuggestIdiom { region, idiom, .. } => (
            TargetDescriptor::Region(*region),
            ProposedChange::Idiom { tag: idiom.clone() },
        ),
        Delta::AnnotateRegion {
            region, comment, ..
        } => (
            TargetDescriptor::Region(*region),
            ProposedChange::Annotation {
                comment: comment.clone(),
            },
        ),
    }
}

fn field_from_suggestion(s: &StructFieldSuggestion) -> ProposedField {
    ProposedField {
        name: s.name.clone(),
        ty: s.ty.clone(),
        offset: s.offset,
    }
}

/// Render a [`ReviewLog`] as a deterministic plaintext block.
///
/// Format (locked by the renderer's tests + B4.4 golden):
///
/// ```text
/// ;; dac --ai-review (spec §13.6)
/// ;; provider: <name>
/// ;; mode:     <lenient|strict>
/// ;; deltas:   total=<n> accepted=<a> rejected=<r>
///
/// ;; delta 1: <kind> on <target>
/// ;;   confidence: <0.00>..<1.00> (<source>)
/// ;;   evidence:   [id=<n>, ...]
/// ;;   outcome:    <accept|reject:<tag> — <detail>>
/// - <current side, kind-specific>
/// + <proposed side, kind-specific>
/// ```
///
/// The header always renders even when the log is empty so a reader
/// can tell "review mode was active, no proposals were returned"
/// apart from "review mode was off".
#[must_use]
pub fn render_review(log: &ReviewLog) -> String {
    let mut out = String::new();
    out.push_str(";; dac --ai-review (spec §13.6)\n");
    out.push_str(&format!(";; provider: {}\n", log.provider()));
    out.push_str(&format!(";; mode:     {}\n", log.mode().tag()));
    out.push_str(&format!(
        ";; deltas:   total={} accepted={} rejected={}\n",
        log.total(),
        log.accepted(),
        log.rejected()
    ));
    for (i, e) in log.entries().iter().enumerate() {
        out.push('\n');
        out.push_str(&format!(
            ";; delta {idx}: {kind} on {target}\n",
            idx = i + 1,
            kind = e.kind,
            target = render_target(e.target)
        ));
        out.push_str(&format!(
            ";;   confidence: {:.2} ({})\n",
            e.confidence_score,
            e.confidence_source.name()
        ));
        out.push_str(&format!(
            ";;   evidence:   {}\n",
            render_evidence(&e.evidence)
        ));
        out.push_str(&format!(
            ";;   outcome:    {}\n",
            render_outcome(&e.outcome)
        ));
        render_diff(&mut out, &e.current, &e.proposed);
    }
    out
}

fn render_target(target: TargetDescriptor) -> String {
    match target {
        TargetDescriptor::Symbol(id) => format!("SymbolRef({})", id.0),
        TargetDescriptor::Slot(id) => format!("SlotRef({})", id.0),
        TargetDescriptor::Region(id) => format!("RegionRef({})", id.0),
    }
}

fn render_evidence(ev: &[EvidenceId]) -> String {
    if ev.is_empty() {
        return "[]".to_string();
    }
    let mut s = String::from("[");
    for (i, id) in ev.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("id={}", id.as_u32()));
    }
    s.push(']');
    s
}

fn render_outcome(outcome: &ReviewOutcome) -> String {
    match outcome {
        ReviewOutcome::Accept => "accept".to_string(),
        ReviewOutcome::Reject { tag, detail } => format!("reject:{tag} — {detail}"),
    }
}

fn render_diff(out: &mut String, current: &CurrentState, proposed: &ProposedChange) {
    match (current, proposed) {
        (CurrentState::Symbol { name, source }, ProposedChange::Rename { new_name }) => {
            out.push_str(&format!("-   name: {name} ({})\n", source.name()));
            out.push_str(&format!("+   name: {new_name}\n"));
        }
        (CurrentState::Slot { ty, source }, ProposedChange::Retype { new_type }) => {
            out.push_str(&format!("-   type: {} ({})\n", ty.tag(), source.name()));
            out.push_str(&format!("+   type: {new_type}\n"));
        }
        (CurrentState::Region { source }, ProposedChange::Annotation { comment }) => {
            out.push_str(&format!("-   comment: (none) ({})\n", source.name()));
            out.push_str(&format!("+   comment: {comment}\n"));
        }
        (CurrentState::Region { source }, ProposedChange::Idiom { tag }) => {
            out.push_str(&format!("-   idiom: (none) ({})\n", source.name()));
            out.push_str(&format!("+   idiom: {tag}\n"));
        }
        (CurrentState::Region { source }, ProposedChange::StructLayout { fields }) => {
            out.push_str(&format!("-   layout: (none) ({})\n", source.name()));
            out.push_str("+   layout:\n");
            for f in fields {
                out.push_str(&format!("+     {}: {} @ {}\n", f.name, f.ty, f.offset));
            }
        }
        // The target was not in the world. Render the `-` side as
        // `(unknown target)` and the `+` side as the proposed change
        // so the reviewer can still see what the provider asked for.
        (CurrentState::Unknown, change) => {
            out.push_str("-   (unknown target)\n");
            render_unknown_proposed(out, change);
        }
        // Type mismatches between current and proposed (e.g. a
        // RenameSymbol whose target the verifier looked up as a Slot)
        // are programmer bugs at the recorder boundary — the
        // `describe_delta`/`from_world` pair maps each delta variant
        // to the matching world lookup. Render a defensive line so
        // the artifact stays parseable if a future variant slips
        // through.
        (current, proposed) => {
            out.push_str(&format!("-   (mismatch: {current:?})\n"));
            out.push_str(&format!("+   (mismatch: {proposed:?})\n"));
        }
    }
}

fn render_unknown_proposed(out: &mut String, change: &ProposedChange) {
    match change {
        ProposedChange::Rename { new_name } => {
            out.push_str(&format!("+   name: {new_name}\n"));
        }
        ProposedChange::Retype { new_type } => {
            out.push_str(&format!("+   type: {new_type}\n"));
        }
        ProposedChange::Annotation { comment } => {
            out.push_str(&format!("+   comment: {comment}\n"));
        }
        ProposedChange::Idiom { tag } => {
            out.push_str(&format!("+   idiom: {tag}\n"));
        }
        ProposedChange::StructLayout { fields } => {
            out.push_str("+   layout:\n");
            for f in fields {
                out.push_str(&format!("+     {}: {} @ {}\n", f.name, f.ty, f.offset));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_ai::{
        EvidenceBundle, Prompt, PromptKind, ProposerContext, RegionRef, SlotRef,
        StructFieldSuggestion, SymbolRef,
    };
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode};

    fn ctx_fixture() -> (Prompt, EvidenceBundle, Confidence) {
        let mut g = EvidenceGraph::new();
        let e = g.add_node(EvidenceNode::Instruction(42));
        let bundle = EvidenceBundle::from_iter([e]);
        let prompt = Prompt::new(PromptKind::NameSuggestion, "test prompt");
        let confidence = Confidence::new(0.5, Source::Speculative);
        (prompt, bundle, confidence)
    }

    fn make_rename(target: SymbolRef, new_name: &str) -> Delta {
        let (p, b, c) = ctx_fixture();
        Delta::rename_symbol(
            target,
            new_name,
            ProposerContext {
                prompt: &p,
                evidence: &b,
                confidence: c,
                model_id: "test",
                seed: 0,
            },
        )
        .expect("ok")
    }

    fn make_retype(target: SlotRef, new_type: &str) -> Delta {
        let (p, b, c) = ctx_fixture();
        Delta::retype_slot(
            target,
            new_type,
            ProposerContext {
                prompt: &p,
                evidence: &b,
                confidence: c,
                model_id: "test",
                seed: 0,
            },
        )
        .expect("ok")
    }

    fn make_annotate(region: RegionRef, comment: &str) -> Delta {
        let (p, b, c) = ctx_fixture();
        Delta::annotate_region(
            region,
            comment,
            ProposerContext {
                prompt: &p,
                evidence: &b,
                confidence: c,
                model_id: "test",
                seed: 0,
            },
        )
        .expect("ok")
    }

    fn make_idiom(region: RegionRef, tag: &str) -> Delta {
        let (p, b, c) = ctx_fixture();
        Delta::suggest_idiom(
            region,
            tag,
            ProposerContext {
                prompt: &p,
                evidence: &b,
                confidence: c,
                model_id: "test",
                seed: 0,
            },
        )
        .expect("ok")
    }

    fn make_struct(region: RegionRef, fields: Vec<StructFieldSuggestion>) -> Delta {
        let (p, b, c) = ctx_fixture();
        Delta::suggest_struct_layout(
            region,
            fields,
            ProposerContext {
                prompt: &p,
                evidence: &b,
                confidence: c,
                model_id: "test",
                seed: 0,
            },
        )
        .expect("ok")
    }

    #[test]
    fn empty_log_still_renders_header() {
        let log = ReviewLog::new("null", VerifyMode::Lenient);
        let rendered = render_review(&log);
        assert!(rendered.contains(";; dac --ai-review"));
        assert!(rendered.contains(";; provider: null"));
        assert!(rendered.contains(";; mode:     lenient"));
        assert!(rendered.contains(";; deltas:   total=0 accepted=0 rejected=0"));
    }

    #[test]
    fn accept_rename_records_before_and_after() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let delta = make_rename(SymbolRef(7), "checksum");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&delta, &world, &VerifyOutcome::Accept);
        assert_eq!(log.accepted(), 1);
        assert_eq!(log.rejected(), 0);
        let rendered = render_review(&log);
        assert!(rendered.contains(";; delta 1: rename-symbol on SymbolRef(7)"));
        assert!(rendered.contains("outcome:    accept"));
        assert!(rendered.contains("-   name: sub_1040 (derived)"));
        assert!(rendered.contains("+   name: checksum"));
    }

    #[test]
    fn unknown_target_renders_unknown_marker() {
        let world = KnownFacts::new();
        let delta = make_rename(SymbolRef(7), "checksum");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(
            &delta,
            &world,
            &VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Symbol,
            }),
        );
        let rendered = render_review(&log);
        assert!(rendered.contains("outcome:    reject:unknown-target"));
        assert!(rendered.contains("-   (unknown target)"));
        assert!(rendered.contains("+   name: checksum"));
    }

    #[test]
    fn retype_diff_uses_slot_type_tag() {
        let mut world = KnownFacts::new();
        world.insert_slot(
            SlotRef(3),
            SlotType::Integer { width_bits: 32 },
            Source::Derived,
        );
        let delta = make_retype(SlotRef(3), "uint8_t *");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(
            &delta,
            &world,
            &VerifyOutcome::Reject(DeltaRejection::RetypeNoPointerEvidence {
                current: SlotType::Integer { width_bits: 32 },
                requested: "uint8_t *".to_string(),
            }),
        );
        let rendered = render_review(&log);
        assert!(rendered.contains("-   type: integer (derived)"));
        assert!(rendered.contains("+   type: uint8_t *"));
        assert!(rendered.contains("reject:retype-no-pointer-evidence"));
    }

    #[test]
    fn struct_layout_renders_each_field() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = make_struct(
            RegionRef(5),
            vec![
                StructFieldSuggestion {
                    name: "len".to_string(),
                    ty: "uint32_t".to_string(),
                    offset: 0,
                },
                StructFieldSuggestion {
                    name: "data".to_string(),
                    ty: "uint8_t".to_string(),
                    offset: 4,
                },
            ],
        );
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&delta, &world, &VerifyOutcome::Accept);
        let rendered = render_review(&log);
        assert!(rendered.contains("-   layout: (none) (derived)"));
        assert!(rendered.contains("+   layout:"));
        assert!(rendered.contains("+     len: uint32_t @ 0"));
        assert!(rendered.contains("+     data: uint8_t @ 4"));
    }

    #[test]
    fn idiom_diff_uses_tag_line() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = make_idiom(RegionRef(5), "memcpy");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&delta, &world, &VerifyOutcome::Accept);
        let rendered = render_review(&log);
        assert!(rendered.contains("-   idiom: (none) (derived)"));
        assert!(rendered.contains("+   idiom: memcpy"));
    }

    #[test]
    fn annotation_diff_uses_comment_line() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = make_annotate(RegionRef(5), "fast path");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&delta, &world, &VerifyOutcome::Accept);
        let rendered = render_review(&log);
        assert!(rendered.contains("-   comment: (none) (derived)"));
        assert!(rendered.contains("+   comment: fast path"));
    }

    #[test]
    fn record_counts_accept_and_reject() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let d1 = make_rename(SymbolRef(7), "checksum");
        let d2 = make_rename(SymbolRef(99), "missing");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&d1, &world, &VerifyOutcome::Accept);
        log.record(
            &d2,
            &world,
            &VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Symbol,
            }),
        );
        assert_eq!(log.total(), 2);
        assert_eq!(log.accepted(), 1);
        assert_eq!(log.rejected(), 1);
    }

    #[test]
    fn render_is_deterministic_across_two_runs() {
        // The renderer is a pure function: same log → same output.
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        world.insert_region(RegionRef(5), Source::Observed);
        let d1 = make_rename(SymbolRef(7), "checksum");
        let d2 = make_annotate(RegionRef(5), "fast path");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Strict);
        log.record(&d1, &world, &VerifyOutcome::Accept);
        log.record(
            &d2,
            &world,
            &VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
                kind: TargetKind::Region,
                source: Source::Observed,
            }),
        );
        let first = render_review(&log);
        let second = render_review(&log);
        assert_eq!(first, second);
    }

    #[test]
    fn outcome_tag_matches_rejection_tag() {
        let world = KnownFacts::new();
        let delta = make_rename(SymbolRef(7), "checksum");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(
            &delta,
            &world,
            &VerifyOutcome::Reject(DeltaRejection::InvalidIdentifier("3foo".to_string())),
        );
        let entry = &log.entries()[0];
        assert_eq!(entry.outcome.tag(), "invalid-identifier");
    }

    #[test]
    fn target_descriptor_kind_matches_target_kind() {
        assert_eq!(
            TargetDescriptor::Symbol(SymbolRef(0)).kind(),
            TargetKind::Symbol
        );
        assert_eq!(TargetDescriptor::Slot(SlotRef(0)).kind(), TargetKind::Slot);
        assert_eq!(
            TargetDescriptor::Region(RegionRef(0)).kind(),
            TargetKind::Region
        );
    }

    #[test]
    fn evidence_renders_as_bracketed_id_list() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let delta = make_rename(SymbolRef(7), "checksum");
        let mut log = ReviewLog::new("local:stub", VerifyMode::Lenient);
        log.record(&delta, &world, &VerifyOutcome::Accept);
        let rendered = render_review(&log);
        assert!(rendered.contains("evidence:   [id="));
    }

    #[test]
    fn strict_mode_renders_strict_in_header() {
        let log = ReviewLog::new("null", VerifyMode::Strict);
        let rendered = render_review(&log);
        assert!(rendered.contains(";; mode:     strict"));
    }
}
