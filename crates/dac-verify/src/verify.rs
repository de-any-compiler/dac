//! Per-delta verification (spec §13.4, §13.5).
//!
//! [`verify_delta`] judges a single [`Delta`] against a [`KnownFacts`]
//! snapshot of recovered state. It returns either [`VerifyOutcome::Accept`]
//! or [`VerifyOutcome::Reject`] with a structured reason. The
//! orchestrator then decides what to do with rejected proposals
//! (typically: count + log; in `--ai-review` mode, write them to the
//! review artifact).
//!
//! The check is intentionally compositional: each delta variant has
//! its own private helper and the public [`verify_delta`] just
//! dispatches. New invariants can be added by extending the variant
//! helper without disturbing the others.

use dac_ai::{Delta, RegionRef, SlotRef, StructFieldSuggestion, SymbolRef};
use dac_core::Source;

use crate::world::{KnownFacts, SlotType};

/// Whether strict mode is engaged.
///
/// Strict mode (`--ai-strict`, ARCHITECTURE §13) rejects any delta
/// whose target is already recorded as [`Source::Observed`]. In
/// lenient mode the same delta would still be accepted (the
/// confidence lattice keeps the Observed fact in place by way of the
/// join semantics — a Speculative rename can never lower an Observed
/// name's confidence — but strict mode drops them up front so the
/// orchestrator never has to apply a delta it would have shadowed
/// anyway).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyMode {
    Lenient,
    Strict,
}

impl VerifyMode {
    /// Stable kebab-case tag for log fields.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Lenient => "lenient",
            Self::Strict => "strict",
        }
    }

    /// `true` iff strict-mode checks are engaged.
    #[must_use]
    pub const fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }
}

/// Result of running [`verify_delta`] on a single proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    Accept,
    Reject(DeltaRejection),
}

impl VerifyOutcome {
    /// `true` iff the verifier accepted the delta.
    #[must_use]
    pub const fn is_accept(&self) -> bool {
        matches!(self, Self::Accept)
    }

    /// `true` iff the verifier rejected the delta.
    #[must_use]
    pub const fn is_reject(&self) -> bool {
        matches!(self, Self::Reject(_))
    }
}

/// Why a delta was rejected.
///
/// Closed set so the orchestrator can render structured log fields
/// and so review-mode output can group rejections by kind. Each
/// variant carries enough context for a human to understand the
/// rejection without consulting the original delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaRejection {
    /// The delta targets a symbol / slot / region the world model
    /// has no record of. Safe default: the verifier cannot reason
    /// about consistency with a fact it has not seen.
    UnknownTarget { kind: TargetKind },
    /// The rename would give the target a name that another symbol
    /// already owns. The `existing` field names the colliding owner
    /// so the rejection log can cite it.
    NameCollision { existing: SymbolRef, name: String },
    /// The proposed new name is syntactically unusable as a C
    /// identifier (empty, leading digit, contains whitespace, …).
    /// Caught here rather than in the constructor so a bug in the
    /// provider surfaces as a structured rejection, not a panic.
    InvalidIdentifier(String),
    /// The retype asks for a pointer type but the slot has no
    /// observed pointer evidence (its recovered type is `Integer`,
    /// `Float`, or `Aggregate`). This is the "retype int→ptr without
    /// evidence" case PLAN.md calls out.
    RetypeNoPointerEvidence {
        current: SlotType,
        requested: String,
    },
    /// The proposed struct layout is structurally invalid: empty,
    /// overlapping fields, or fields out of offset order.
    InvalidStructLayout(String),
    /// The annotation comment is empty — nothing to surface.
    EmptyAnnotation,
    /// The idiom suggestion's tag is empty.
    EmptyIdiom,
    /// Strict mode: the target's recorded source is `Observed`, so
    /// any Speculative delta would (in lenient mode) be shadowed by
    /// the lattice anyway. Strict mode drops the proposal up front.
    StrictModeBlocksObserved { kind: TargetKind, source: Source },
}

impl DeltaRejection {
    /// Stable kebab-case tag — useful for log fields and review-mode
    /// grouping.
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::UnknownTarget { .. } => "unknown-target",
            Self::NameCollision { .. } => "name-collision",
            Self::InvalidIdentifier(_) => "invalid-identifier",
            Self::RetypeNoPointerEvidence { .. } => "retype-no-pointer-evidence",
            Self::InvalidStructLayout(_) => "invalid-struct-layout",
            Self::EmptyAnnotation => "empty-annotation",
            Self::EmptyIdiom => "empty-idiom",
            Self::StrictModeBlocksObserved { .. } => "strict-mode-blocks-observed",
        }
    }
}

/// Which kind of handle a rejection refers to. Lets the rejection
/// reason stay variant-free while preserving the "symbol vs slot vs
/// region" distinction in logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Symbol,
    Slot,
    Region,
}

impl TargetKind {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::Slot => "slot",
            Self::Region => "region",
        }
    }
}

/// Judge a single [`Delta`] against the [`KnownFacts`] snapshot.
///
/// Returns [`VerifyOutcome::Accept`] iff every applicable invariant
/// holds. The function is pure (no I/O, no mutation, no global
/// state); the orchestrator can call it from any pass without
/// declaring a determinism class beyond `Pure`.
#[must_use]
pub fn verify_delta(delta: &Delta, world: &KnownFacts, mode: VerifyMode) -> VerifyOutcome {
    match delta {
        Delta::RenameSymbol {
            target, new_name, ..
        } => verify_rename(*target, new_name, world, mode),
        Delta::RetypeSlot {
            target, new_type, ..
        } => verify_retype(*target, new_type, world, mode),
        Delta::SuggestStructLayout { region, fields, .. } => {
            verify_struct_layout(*region, fields, world, mode)
        }
        Delta::SuggestIdiom { region, idiom, .. } => verify_idiom(*region, idiom, world, mode),
        Delta::AnnotateRegion {
            region, comment, ..
        } => verify_annotation(*region, comment, world, mode),
    }
}

fn verify_rename(
    target: SymbolRef,
    new_name: &str,
    world: &KnownFacts,
    mode: VerifyMode,
) -> VerifyOutcome {
    if !is_valid_identifier(new_name) {
        return VerifyOutcome::Reject(DeltaRejection::InvalidIdentifier(new_name.to_string()));
    }
    let known = match world.symbol(target) {
        Some(s) => s,
        None => {
            return VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Symbol,
            });
        }
    };
    if mode.is_strict() && known.source == Source::Observed {
        return VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
            kind: TargetKind::Symbol,
            source: known.source,
        });
    }
    // Renaming to the symbol's own current name is a no-op — accept
    // it so providers don't have to special-case it. The collision
    // check only fires when a *different* symbol already owns the
    // proposed name.
    if let Some(existing) = world.symbol_by_name(new_name) {
        if existing != target {
            return VerifyOutcome::Reject(DeltaRejection::NameCollision {
                existing,
                name: new_name.to_string(),
            });
        }
    }
    VerifyOutcome::Accept
}

fn verify_retype(
    target: SlotRef,
    new_type: &str,
    world: &KnownFacts,
    mode: VerifyMode,
) -> VerifyOutcome {
    let known = match world.slot(target) {
        Some(s) => s,
        None => {
            return VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Slot,
            });
        }
    };
    if mode.is_strict() && known.source == Source::Observed {
        return VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
            kind: TargetKind::Slot,
            source: known.source,
        });
    }
    if looks_like_pointer_type(new_type) && !known.ty.is_pointer() && known.ty != SlotType::Unknown
    {
        return VerifyOutcome::Reject(DeltaRejection::RetypeNoPointerEvidence {
            current: known.ty.clone(),
            requested: new_type.to_string(),
        });
    }
    VerifyOutcome::Accept
}

fn verify_struct_layout(
    region: RegionRef,
    fields: &[StructFieldSuggestion],
    world: &KnownFacts,
    mode: VerifyMode,
) -> VerifyOutcome {
    let known = match world.region(region) {
        Some(r) => r,
        None => {
            return VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Region,
            });
        }
    };
    if mode.is_strict() && known.source == Source::Observed {
        return VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
            kind: TargetKind::Region,
            source: known.source,
        });
    }
    if fields.is_empty() {
        return VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(
            "struct layout must have at least one field".to_string(),
        ));
    }
    // Fields must be in strictly ascending offset order so the
    // recovered struct mirrors a real C layout. Two fields at the
    // same offset means an overlap; that is a hard rejection (the
    // backend cannot lower it without an `union` it never invented).
    let mut prev_offset: Option<u64> = None;
    for f in fields {
        if !is_valid_identifier(&f.name) {
            return VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(format!(
                "field name `{}` is not a valid identifier",
                f.name
            )));
        }
        if f.ty.trim().is_empty() {
            return VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(format!(
                "field `{}` has empty type",
                f.name
            )));
        }
        if let Some(prev) = prev_offset {
            if f.offset <= prev {
                return VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(format!(
                    "field `{}` offset {} does not exceed previous offset {}",
                    f.name, f.offset, prev
                )));
            }
        }
        prev_offset = Some(f.offset);
    }
    VerifyOutcome::Accept
}

fn verify_idiom(
    region: RegionRef,
    idiom: &str,
    world: &KnownFacts,
    mode: VerifyMode,
) -> VerifyOutcome {
    let known = match world.region(region) {
        Some(r) => r,
        None => {
            return VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Region,
            });
        }
    };
    if mode.is_strict() && known.source == Source::Observed {
        return VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
            kind: TargetKind::Region,
            source: known.source,
        });
    }
    if idiom.trim().is_empty() {
        return VerifyOutcome::Reject(DeltaRejection::EmptyIdiom);
    }
    VerifyOutcome::Accept
}

fn verify_annotation(
    region: RegionRef,
    comment: &str,
    world: &KnownFacts,
    mode: VerifyMode,
) -> VerifyOutcome {
    let known = match world.region(region) {
        Some(r) => r,
        None => {
            return VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Region,
            });
        }
    };
    if mode.is_strict() && known.source == Source::Observed {
        return VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
            kind: TargetKind::Region,
            source: known.source,
        });
    }
    if comment.trim().is_empty() {
        return VerifyOutcome::Reject(DeltaRejection::EmptyAnnotation);
    }
    VerifyOutcome::Accept
}

/// Cheap, dependency-free C-identifier check. Matches `[A-Za-z_][A-Za-z0-9_]*`.
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if first != '_' && !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Heuristic: does this type spelling describe a pointer? Used as the
/// trigger for the "retype int→ptr without evidence" check.
///
/// The check is intentionally syntactic — the verifier does not own a
/// type parser. Anything ending in `*` (after trimming) or containing
/// `* ` or `*const`/`*restrict` qualifiers counts. The full C type
/// system check lives in `dac-ir`; we only need the trigger.
fn looks_like_pointer_type(s: &str) -> bool {
    let trimmed = s.trim();
    trimmed.ends_with('*') || trimmed.contains("* ") || trimmed.contains("*const")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_ai::{
        Delta, EvidenceBundle, Prompt, PromptKind, ProposerContext, RegionRef, SlotRef,
        StructFieldSuggestion, SymbolRef,
    };
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode};

    struct Fixture {
        prompt: Prompt,
        bundle: EvidenceBundle,
        confidence: Confidence,
    }

    fn fixture() -> Fixture {
        let mut g = EvidenceGraph::new();
        let e = g.add_node(EvidenceNode::Instruction(1));
        let bundle = EvidenceBundle::from_iter([e]);
        let prompt = Prompt::new(PromptKind::NameSuggestion, "name sub_1040");
        let confidence = Confidence::new(0.5, Source::Speculative);
        Fixture {
            prompt,
            bundle,
            confidence,
        }
    }

    fn ctx<'a>(f: &'a Fixture) -> ProposerContext<'a> {
        ProposerContext {
            prompt: &f.prompt,
            evidence: &f.bundle,
            confidence: f.confidence,
            model_id: "test",
            seed: 0,
        }
    }

    fn rename(name: &str) -> Delta {
        let f = fixture();
        Delta::rename_symbol(SymbolRef(7), name, ctx(&f)).expect("ok")
    }

    fn retype(ty: &str) -> Delta {
        let f = fixture();
        Delta::retype_slot(SlotRef(3), ty, ctx(&f)).expect("ok")
    }

    fn annotate(region: RegionRef, comment: &str) -> Delta {
        let f = fixture();
        Delta::annotate_region(region, comment, ctx(&f)).expect("ok")
    }

    fn idiom(region: RegionRef, idiom: &str) -> Delta {
        let f = fixture();
        Delta::suggest_idiom(region, idiom, ctx(&f)).expect("ok")
    }

    fn struct_layout(region: RegionRef, fields: Vec<StructFieldSuggestion>) -> Delta {
        let f = fixture();
        Delta::suggest_struct_layout(region, fields, ctx(&f)).expect("ok")
    }

    // ── PLAN.md "done when" case 1: rename to colliding symbol ─────────────

    #[test]
    fn rejects_rename_to_colliding_symbol() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        world.insert_symbol(SymbolRef(8), "checksum", Source::Derived);
        let outcome = verify_delta(&rename("checksum"), &world, VerifyMode::Lenient);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::NameCollision {
                existing: SymbolRef(8),
                name: "checksum".to_string(),
            })
        );
    }

    // ── PLAN.md "done when" case 2: retype int → ptr without evidence ──────

    #[test]
    fn rejects_retype_int_to_ptr_without_evidence() {
        let mut world = KnownFacts::new();
        world.insert_slot(
            SlotRef(3),
            SlotType::Integer { width_bits: 32 },
            Source::Derived,
        );
        let outcome = verify_delta(&retype("uint8_t *"), &world, VerifyMode::Lenient);
        match outcome {
            VerifyOutcome::Reject(DeltaRejection::RetypeNoPointerEvidence {
                current,
                requested,
            }) => {
                assert_eq!(current, SlotType::Integer { width_bits: 32 });
                assert_eq!(requested, "uint8_t *");
            }
            other => panic!("expected RetypeNoPointerEvidence, got {other:?}"),
        }
    }

    // ── Strict mode ─────────────────────────────────────────────────────────

    #[test]
    fn strict_mode_blocks_rename_against_observed_symbol() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "main", Source::Observed);
        let outcome = verify_delta(&rename("foo"), &world, VerifyMode::Strict);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::StrictModeBlocksObserved {
                kind: TargetKind::Symbol,
                source: Source::Observed,
            })
        );
    }

    #[test]
    fn lenient_mode_accepts_rename_against_observed_symbol() {
        // Lenient mode lets the delta through; the lattice protects
        // the Observed name at apply time.
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "main", Source::Observed);
        let outcome = verify_delta(&rename("foo"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn strict_mode_does_not_block_speculative_target() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let outcome = verify_delta(&rename("checksum"), &world, VerifyMode::Strict);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    // ── Unknown targets ─────────────────────────────────────────────────────

    #[test]
    fn rejects_delta_against_unknown_symbol() {
        let world = KnownFacts::new();
        let outcome = verify_delta(&rename("checksum"), &world, VerifyMode::Lenient);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Symbol,
            })
        );
    }

    #[test]
    fn rejects_delta_against_unknown_slot() {
        let world = KnownFacts::new();
        let outcome = verify_delta(&retype("uint8_t *"), &world, VerifyMode::Lenient);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Slot,
            })
        );
    }

    #[test]
    fn rejects_delta_against_unknown_region() {
        let world = KnownFacts::new();
        let outcome = verify_delta(&annotate(RegionRef(5), "note"), &world, VerifyMode::Lenient);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::UnknownTarget {
                kind: TargetKind::Region,
            })
        );
    }

    // ── Rename accept paths ────────────────────────────────────────────────

    #[test]
    fn accepts_rename_to_unique_valid_name() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let outcome = verify_delta(&rename("checksum"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn accepts_rename_to_own_current_name_as_noop() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let outcome = verify_delta(&rename("sub_1040"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn rejects_rename_to_empty_identifier() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let outcome = verify_delta(&rename(""), &world, VerifyMode::Lenient);
        assert!(matches!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::InvalidIdentifier(_))
        ));
    }

    #[test]
    fn rejects_rename_to_identifier_starting_with_digit() {
        let mut world = KnownFacts::new();
        world.insert_symbol(SymbolRef(7), "sub_1040", Source::Derived);
        let outcome = verify_delta(&rename("3foo"), &world, VerifyMode::Lenient);
        assert!(matches!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::InvalidIdentifier(_))
        ));
    }

    // ── Retype accept paths ────────────────────────────────────────────────

    #[test]
    fn accepts_retype_when_slot_already_observed_as_pointer() {
        let mut world = KnownFacts::new();
        world.insert_slot(SlotRef(3), SlotType::Pointer, Source::Derived);
        let outcome = verify_delta(&retype("uint8_t *"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn accepts_retype_int_to_int_with_different_width() {
        let mut world = KnownFacts::new();
        world.insert_slot(
            SlotRef(3),
            SlotType::Integer { width_bits: 32 },
            Source::Derived,
        );
        let outcome = verify_delta(&retype("int64_t"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn accepts_retype_against_unknown_slot_type() {
        // Slot known to world but no type evidence yet → any retype
        // is plausible.
        let mut world = KnownFacts::new();
        world.insert_slot(SlotRef(3), SlotType::Unknown, Source::Derived);
        let outcome = verify_delta(&retype("uint8_t *"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    // ── Struct layout ──────────────────────────────────────────────────────

    fn field(name: &str, ty: &str, offset: u64) -> StructFieldSuggestion {
        StructFieldSuggestion {
            name: name.to_string(),
            ty: ty.to_string(),
            offset,
        }
    }

    #[test]
    fn accepts_struct_with_ascending_field_offsets() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = struct_layout(
            RegionRef(5),
            vec![field("len", "uint32_t", 0), field("data", "uint8_t", 4)],
        );
        let outcome = verify_delta(&delta, &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn rejects_struct_with_no_fields() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = struct_layout(RegionRef(5), vec![]);
        assert!(matches!(
            verify_delta(&delta, &world, VerifyMode::Lenient),
            VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(_))
        ));
    }

    #[test]
    fn rejects_struct_with_overlapping_fields() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = struct_layout(
            RegionRef(5),
            vec![field("a", "uint32_t", 4), field("b", "uint32_t", 4)],
        );
        assert!(matches!(
            verify_delta(&delta, &world, VerifyMode::Lenient),
            VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(_))
        ));
    }

    #[test]
    fn rejects_struct_with_field_offsets_out_of_order() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = struct_layout(
            RegionRef(5),
            vec![field("a", "uint32_t", 8), field("b", "uint32_t", 0)],
        );
        assert!(matches!(
            verify_delta(&delta, &world, VerifyMode::Lenient),
            VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(_))
        ));
    }

    #[test]
    fn rejects_struct_with_invalid_field_identifier() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let delta = struct_layout(RegionRef(5), vec![field("3bad", "uint32_t", 0)]);
        assert!(matches!(
            verify_delta(&delta, &world, VerifyMode::Lenient),
            VerifyOutcome::Reject(DeltaRejection::InvalidStructLayout(_))
        ));
    }

    // ── Annotation / idiom ─────────────────────────────────────────────────

    #[test]
    fn accepts_non_empty_annotation() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let outcome = verify_delta(
            &annotate(RegionRef(5), "fast path"),
            &world,
            VerifyMode::Lenient,
        );
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn rejects_whitespace_only_annotation() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let outcome = verify_delta(&annotate(RegionRef(5), "   "), &world, VerifyMode::Lenient);
        assert_eq!(
            outcome,
            VerifyOutcome::Reject(DeltaRejection::EmptyAnnotation)
        );
    }

    #[test]
    fn accepts_non_empty_idiom() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let outcome = verify_delta(&idiom(RegionRef(5), "memcpy"), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Accept);
    }

    #[test]
    fn rejects_empty_idiom() {
        let mut world = KnownFacts::new();
        world.insert_region(RegionRef(5), Source::Derived);
        let outcome = verify_delta(&idiom(RegionRef(5), ""), &world, VerifyMode::Lenient);
        assert_eq!(outcome, VerifyOutcome::Reject(DeltaRejection::EmptyIdiom));
    }

    // ── Mode helpers ───────────────────────────────────────────────────────

    #[test]
    fn verify_mode_tags_are_kebab_case() {
        for m in [VerifyMode::Lenient, VerifyMode::Strict] {
            let t = m.tag();
            assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
        assert!(!VerifyMode::Lenient.is_strict());
        assert!(VerifyMode::Strict.is_strict());
    }

    #[test]
    fn rejection_tags_are_distinct_and_kebab_case() {
        let cases = [
            DeltaRejection::UnknownTarget {
                kind: TargetKind::Symbol,
            },
            DeltaRejection::NameCollision {
                existing: SymbolRef(0),
                name: String::new(),
            },
            DeltaRejection::InvalidIdentifier(String::new()),
            DeltaRejection::RetypeNoPointerEvidence {
                current: SlotType::Unknown,
                requested: String::new(),
            },
            DeltaRejection::InvalidStructLayout(String::new()),
            DeltaRejection::EmptyAnnotation,
            DeltaRejection::EmptyIdiom,
            DeltaRejection::StrictModeBlocksObserved {
                kind: TargetKind::Symbol,
                source: Source::Observed,
            },
        ];
        let mut tags = Vec::new();
        for c in &cases {
            let t = c.tag();
            assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
            tags.push(t);
        }
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len());
    }

    #[test]
    fn target_kind_tags_are_distinct() {
        let tags = [
            TargetKind::Symbol.tag(),
            TargetKind::Slot.tag(),
            TargetKind::Region.tag(),
        ];
        let unique: std::collections::BTreeSet<_> = tags.iter().copied().collect();
        assert_eq!(tags.len(), unique.len());
    }

    // ── Outcome helpers ────────────────────────────────────────────────────

    #[test]
    fn outcome_helpers_match_variant() {
        assert!(VerifyOutcome::Accept.is_accept());
        assert!(!VerifyOutcome::Accept.is_reject());
        let r = VerifyOutcome::Reject(DeltaRejection::EmptyAnnotation);
        assert!(r.is_reject());
        assert!(!r.is_accept());
    }

    // ── Pointer-shape heuristic ────────────────────────────────────────────

    #[test]
    fn pointer_heuristic_catches_common_spellings() {
        assert!(looks_like_pointer_type("uint8_t*"));
        assert!(looks_like_pointer_type("uint8_t *"));
        assert!(looks_like_pointer_type("uint8_t * const"));
        assert!(looks_like_pointer_type("const char *"));
        assert!(looks_like_pointer_type("void *const"));
        assert!(!looks_like_pointer_type("uint32_t"));
        assert!(!looks_like_pointer_type("int64_t"));
        assert!(!looks_like_pointer_type("struct foo"));
    }

    #[test]
    fn identifier_heuristic_catches_common_invalid() {
        assert!(is_valid_identifier("checksum"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("a1"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("1foo"));
        assert!(!is_valid_identifier("with space"));
        assert!(!is_valid_identifier("with-dash"));
    }
}
