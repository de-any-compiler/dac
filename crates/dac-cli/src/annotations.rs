//! Annotation channel (FR-19, FR-23, FR-25, spec §10.4 / §12).
//!
//! Surfaces every name and type that appears in dac's emitted source
//! together with the confidence, source class, evidence-graph trail,
//! and a short human-readable explanation. Two views ship:
//!
//! - [`render_annotations_json`] — deterministic JSON written to
//!   `<output>.annot.json` when `--emit-annotations` is set (spec
//!   §10.2 "annotations / notes" artifact). Hand-rolled writer with
//!   fixed key order so the manifest pattern from
//!   [`crate::manifest::render_manifest_json`] holds: identical inputs
//!   → byte-identical output.
//! - [`render_annotations_debug`] — plain-text "Why this name?" /
//!   "Why this type?" block embedded into each emitted C function's
//!   leading comment when `--debug` is passed (spec §12 trace mode).
//!
//! ## What gets annotated at B3.4
//!
//! The emitted C unit today has two kinds of surface fact per
//! discovered function: the function **name** and its **return type**.
//! Both get a [`FactAnnotation`]:
//!
//! - **Name.** When the function carries a symbol-table name
//!   ([`SourceMask::SYMBOL`] set), the annotation is
//!   [`Source::Observed`] with the symbol confidence and an evidence
//!   chain of the function's [`EvidenceId`] plus every supporting
//!   predecessor in the [`EvidenceGraph`] (the byte-span node
//!   `dac-recovery::functions` minted under it). When the name is the
//!   synthesized `fn_<hex>` fallback, the annotation is
//!   [`Source::Derived`] with value `0.0` — the address-based label
//!   carries no semantic content.
//! - **Return type.** All recovered functions render with
//!   `CType::Void` until the signature-inference batch lands (B3.6).
//!   The annotation is therefore [`Source::Derived`] with value `0.0`
//!   and an explanation that records the pending status, so a reader
//!   inspecting the `--debug` trail or the `.annot.json` sidecar can
//!   see *why* the type is `void` (and that it is not "observed
//!   void", which is meaningful).
//!
//! Stack-frame locals (B2.4), inferred calling-convention parameter
//! lists (B2.5), propagated value types (B2.6), recovered struct
//! / array layouts (B3.2), and switch-table idioms (B3.3) all exist
//! in `dac-recovery`'s side tables but do not yet surface in the
//! emitted C — the lifter → `RawFunction` bridge needed to drive the
//! structurer is still pending. Until those facts land in
//! `TranslationUnit`, annotating them here would describe artifacts
//! the reader cannot find in the `.c` sidecar. They slot into the
//! [`FunctionAnnotation`] struct as additional fields when the
//! corresponding emission lands.
//!
//! ## Evidence-graph linkage (I-2)
//!
//! [`AnnotationDoc::build`] walks the [`EvidenceGraph`] once to fold a
//! reverse index keyed by node id, so each annotated fact can render
//! every node that supports it (`Supports` edges only — `Contradicts`
//! and `Refines` edges are deliberately omitted from the user-visible
//! trail at this batch; they will land alongside the AI delta
//! protocol in M4).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use dac_binfmt::BinaryModel;
use dac_core::{Confidence, EdgeKind, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer, Source};
use dac_hints::{FunctionHint, Hints};
use dac_recovery::{FunctionSet, SourceMask, SYMBOL_CONFIDENCE};

use crate::lift::USER_HINT_CONFIDENCE;

/// Top-level annotation document.
#[derive(Debug, Clone)]
pub(crate) struct AnnotationDoc {
    pub tool: ToolStamp,
    pub input: InputStamp,
    pub settings: SettingsStamp,
    pub evidence: EvidenceSummary,
    pub functions: Vec<FunctionAnnotation>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolStamp {
    pub name: String,
    pub version: String,
    pub build_id: String,
}

#[derive(Debug, Clone)]
pub(crate) struct InputStamp {
    pub path: String,
    pub format: String,
    pub architecture: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct SettingsStamp {
    pub level: String,
    pub target: String,
    pub debug: bool,
}

/// Coarse summary of the evidence graph: total node count plus a
/// histogram by [`EvidenceNode`] variant. Reproducible at byte level
/// because the histogram is rendered in fixed key order.
#[derive(Debug, Clone, Default)]
pub(crate) struct EvidenceSummary {
    pub node_count: u64,
    pub bytes: u64,
    pub instruction: u64,
    pub ir_node: u64,
    pub knowledge_fact: u64,
    pub user_hint: u64,
    pub ai_suggestion: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionAnnotation {
    pub address: u64,
    pub end: Option<u64>,
    pub signals: SourceMask,
    pub name: FactAnnotation,
    pub return_type: FactAnnotation,
}

/// A single annotated fact: the surfaced value plus its provenance.
#[derive(Debug, Clone)]
pub(crate) struct FactAnnotation {
    pub value: String,
    pub confidence: Confidence,
    pub explanation: String,
    pub evidence: Vec<EvidenceRef>,
}

/// A single entry in the rendered evidence trail.
///
/// Mirrors the [`EvidenceNode`] variants so the JSON sidecar carries
/// enough structure for a reader to cross-reference back into the
/// graph without re-walking it.
#[derive(Debug, Clone)]
pub(crate) struct EvidenceRef {
    pub id: u32,
    pub kind: EvidenceRefKind,
}

#[derive(Debug, Clone)]
pub(crate) enum EvidenceRefKind {
    Bytes { start: u64, end: u64 },
    Instruction { id: u64 },
    IrNode { layer: IrLayer, node_id: u64 },
    KnowledgeFact { id: u64 },
    UserHint { id: u64 },
    AiSuggestion { prompt_hash: [u8; 32] },
}

impl EvidenceRefKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Bytes { .. } => "bytes",
            Self::Instruction { .. } => "instruction",
            Self::IrNode { .. } => "ir_node",
            Self::KnowledgeFact { .. } => "knowledge_fact",
            Self::UserHint { .. } => "user_hint",
            Self::AiSuggestion { .. } => "ai_suggestion",
        }
    }
}

impl AnnotationDoc {
    /// Build the document from the inputs that drove the current run.
    ///
    /// `level` / `target` / `debug` track the active CLI flags so the
    /// header in the JSON sidecar describes the exact configuration
    /// the artifact came out of.
    pub(crate) fn build(
        tool: ToolStamp,
        input: InputStamp,
        settings: SettingsStamp,
        model: &BinaryModel,
        functions: &FunctionSet,
        graph: &EvidenceGraph,
        hints: &Hints,
    ) -> Self {
        let preds = predecessor_index(graph);
        let evidence = summarize_graph(graph);
        let format_label = model.format.name();
        let mut function_annotations = Vec::with_capacity(functions.functions.len());
        for f in &functions.functions {
            // B3.19: surface the matched `[[function]]` hint in the
            // sidecar so a reader can see *which* hint pinned the name
            // / return type. The hint catalogue's `evidence` field is
            // populated by `register_hints` (lift.rs) before
            // `AnnotationDoc::build` runs.
            let hint = hints.find_function(f.address, f.name.as_deref());
            function_annotations.push(FunctionAnnotation {
                address: f.address,
                end: f.end,
                signals: f.sources,
                name: annotate_name(f, format_label, graph, &preds, hint),
                return_type: annotate_return_type(hint),
            });
        }
        let mut notes = Vec::new();
        if functions.functions.is_empty() {
            notes.push(
                "no functions discovered (architecture backend unavailable or empty input)".into(),
            );
        }
        Self {
            tool,
            input,
            settings,
            evidence,
            functions: function_annotations,
            notes,
        }
    }
}

fn annotate_name(
    f: &dac_recovery::Function,
    format_label: &str,
    graph: &EvidenceGraph,
    preds: &BTreeMap<u32, Vec<EvidenceId>>,
    hint: Option<&FunctionHint>,
) -> FactAnnotation {
    // B3.19: a `[[function]]` hint with `rename` set wins over the
    // observed symbol — the C backend's `lower_one_c_function` puts
    // the hint's name on the emitted symbol, so the annotation has
    // to agree. The evidence chain anchors on the hint's
    // `EvidenceNode::UserHint` node so a reader can trace the
    // override straight to the hint catalogue.
    if let Some(h) = hint {
        if let Some(rename) = &h.rename {
            let conf = Confidence::new(USER_HINT_CONFIDENCE, Source::UserHint);
            let why = format!(
                "user hint at line {line} renamed function at {addr:#018x} to '{rename}'",
                line = h.line,
                addr = f.address,
            );
            return FactAnnotation {
                value: rename.clone(),
                confidence: conf,
                explanation: why,
                evidence: hint_evidence_chain(graph, preds, h),
            };
        }
    }
    let has_symbol = f.sources.contains(SourceMask::SYMBOL);
    let (value, confidence, explanation) = if let (true, Some(name)) = (has_symbol, &f.name) {
        let conf = Confidence::new(SYMBOL_CONFIDENCE, Source::Observed);
        let why = format!(
            "{format_label} symbol-table entry for .text address {addr:#018x}",
            addr = f.address,
        );
        (name.clone(), conf, why)
    } else {
        let synthesized = format!("fn_{:x}", f.address);
        let conf = Confidence::new(0.0, Source::Derived);
        let why = format!(
            "synthesized from function start address; no symbol-table entry at {:#018x}",
            f.address,
        );
        (synthesized, conf, why)
    };
    let evidence = evidence_chain(graph, preds, f.evidence);
    FactAnnotation {
        value,
        confidence,
        explanation,
        evidence,
    }
}

fn annotate_return_type(hint: Option<&FunctionHint>) -> FactAnnotation {
    // B3.19: when a `[[function]]` hint pinned `return`, the C
    // backend's `pick_return_type` consults the hint-seeded TypeMap
    // entry, so the annotation surfaces the hinted type with
    // `Source::UserHint` and cites the hint node.
    if let Some(h) = hint {
        if let Some(ret_ty) = &h.return_ty {
            return FactAnnotation {
                value: format!("{ret_ty}"),
                confidence: Confidence::new(USER_HINT_CONFIDENCE, Source::UserHint),
                explanation: format!(
                    "user hint at line {line} pinned the return type to {ret_ty}",
                    line = h.line,
                ),
                evidence: hint_evidence_refs(h),
            };
        }
    }
    FactAnnotation {
        value: "void".to_string(),
        confidence: Confidence::new(0.0, Source::Derived),
        explanation:
            "default void return; calling-convention return-value inference lands with B3.6"
                .to_string(),
        evidence: Vec::new(),
    }
}

/// Resolve the evidence chain rooted at the hint's
/// [`EvidenceNode::UserHint`] node. Falls back to a single
/// synthesised `UserHint` ref when the hint catalogue was not
/// registered against the graph (test paths). Hints have no
/// `Supports`-predecessors today, so the BFS terminates at the
/// hint node itself; the helper exists so a later batch can add
/// supporting edges (e.g. citing the bytes of the hint file) and
/// the annotation channel will pick them up automatically.
fn hint_evidence_chain(
    graph: &EvidenceGraph,
    preds: &BTreeMap<u32, Vec<EvidenceId>>,
    hint: &FunctionHint,
) -> Vec<EvidenceRef> {
    match hint.evidence {
        Some(id) => evidence_chain(graph, preds, id),
        None => hint_evidence_refs(hint),
    }
}

/// Fallback evidence list used when the hint was not registered
/// against the [`EvidenceGraph`] — the `id` field still tracks the
/// hint's [`dac_hints::HintId`] so the sidecar carries the same
/// payload the report's `user_hint` summary cites.
fn hint_evidence_refs(hint: &FunctionHint) -> Vec<EvidenceRef> {
    vec![EvidenceRef {
        id: hint.evidence.map(|e| e.as_u32()).unwrap_or(0),
        kind: EvidenceRefKind::UserHint { id: hint.id },
    }]
}

/// Walk the [`EvidenceGraph`] once and build a reverse index of
/// `Supports`-typed predecessors per node. Iteration matches insertion
/// order so the index is deterministic.
fn predecessor_index(graph: &EvidenceGraph) -> BTreeMap<u32, Vec<EvidenceId>> {
    let mut idx: BTreeMap<u32, Vec<EvidenceId>> = BTreeMap::new();
    for (id, _) in graph.iter() {
        for edge in graph.out_edges(id) {
            if edge.kind == EdgeKind::Supports {
                idx.entry(edge.target.as_u32()).or_default().push(id);
            }
        }
    }
    idx
}

/// Resolve the evidence chain that backs a fact: the fact's own
/// `EvidenceId` followed by every `Supports`-predecessor reachable
/// transitively. The chain is deduplicated and walked breadth-first so
/// the order is deterministic.
fn evidence_chain(
    graph: &EvidenceGraph,
    preds: &BTreeMap<u32, Vec<EvidenceId>>,
    root: EvidenceId,
) -> Vec<EvidenceRef> {
    let mut visited: BTreeSet<u32> = BTreeSet::new();
    let mut frontier: Vec<EvidenceId> = vec![root];
    let mut chain: Vec<EvidenceRef> = Vec::new();
    while let Some(id) = frontier.pop() {
        if !visited.insert(id.as_u32()) {
            continue;
        }
        if let Some(node) = graph.node(id) {
            chain.push(node_to_ref(id, node));
        }
        if let Some(parents) = preds.get(&id.as_u32()) {
            // Push in reverse so the next pops happen in insertion order.
            for parent in parents.iter().rev() {
                frontier.push(*parent);
            }
        }
    }
    chain
}

fn node_to_ref(id: EvidenceId, node: &EvidenceNode) -> EvidenceRef {
    let kind = match node {
        EvidenceNode::Bytes { start, end } => EvidenceRefKind::Bytes {
            start: *start,
            end: *end,
        },
        EvidenceNode::Instruction(n) => EvidenceRefKind::Instruction { id: *n },
        EvidenceNode::IrNode { layer, id } => EvidenceRefKind::IrNode {
            layer: *layer,
            node_id: *id,
        },
        EvidenceNode::KnowledgeFact(n) => EvidenceRefKind::KnowledgeFact { id: *n },
        EvidenceNode::UserHint(n) => EvidenceRefKind::UserHint { id: *n },
        EvidenceNode::AiSuggestion { prompt_hash } => EvidenceRefKind::AiSuggestion {
            prompt_hash: *prompt_hash,
        },
    };
    EvidenceRef {
        id: id.as_u32(),
        kind,
    }
}

fn summarize_graph(graph: &EvidenceGraph) -> EvidenceSummary {
    let mut s = EvidenceSummary {
        node_count: graph.node_count() as u64,
        ..EvidenceSummary::default()
    };
    for (_, node) in graph.iter() {
        match node {
            EvidenceNode::Bytes { .. } => s.bytes += 1,
            EvidenceNode::Instruction(_) => s.instruction += 1,
            EvidenceNode::IrNode { .. } => s.ir_node += 1,
            EvidenceNode::KnowledgeFact(_) => s.knowledge_fact += 1,
            EvidenceNode::UserHint(_) => s.user_hint += 1,
            EvidenceNode::AiSuggestion { .. } => s.ai_suggestion += 1,
        }
    }
    s
}

/// Serialize an [`AnnotationDoc`] to deterministic JSON.
///
/// Hand-rolled writer with fixed key order — same contract as
/// [`crate::manifest::render_manifest_json`]. Confidence values are
/// rendered with `{:.3}` so byte-stability does not depend on the
/// host's floating-point default precision.
pub(crate) fn render_annotations_json(doc: &AnnotationDoc) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    write_tool(&mut out, &doc.tool);
    write_input(&mut out, &doc.input);
    write_settings(&mut out, &doc.settings);
    write_evidence_summary(&mut out, &doc.evidence);
    write_functions(&mut out, &doc.functions);
    write_notes(&mut out, &doc.notes);
    out.push_str("}\n");
    out
}

fn write_tool(out: &mut String, tool: &ToolStamp) {
    let _ = writeln!(out, "  \"tool\": {{");
    let _ = writeln!(out, "    \"name\": {},", json_string(&tool.name));
    let _ = writeln!(out, "    \"version\": {},", json_string(&tool.version));
    let _ = writeln!(out, "    \"build_id\": {}", json_string(&tool.build_id));
    out.push_str("  },\n");
}

fn write_input(out: &mut String, input: &InputStamp) {
    let _ = writeln!(out, "  \"input\": {{");
    let _ = writeln!(out, "    \"path\": {},", json_string(&input.path));
    let _ = writeln!(out, "    \"format\": {},", json_string(&input.format));
    let _ = writeln!(
        out,
        "    \"architecture\": {},",
        json_string(&input.architecture)
    );
    let _ = writeln!(out, "    \"size\": {}", input.size);
    out.push_str("  },\n");
}

fn write_settings(out: &mut String, s: &SettingsStamp) {
    let _ = writeln!(out, "  \"settings\": {{");
    let _ = writeln!(out, "    \"level\": {},", json_string(&s.level));
    let _ = writeln!(out, "    \"target\": {},", json_string(&s.target));
    let _ = writeln!(out, "    \"debug\": {}", json_bool(s.debug));
    out.push_str("  },\n");
}

fn write_evidence_summary(out: &mut String, e: &EvidenceSummary) {
    let _ = writeln!(out, "  \"evidence\": {{");
    let _ = writeln!(out, "    \"node_count\": {},", e.node_count);
    let _ = writeln!(out, "    \"by_kind\": {{");
    // Fixed alphabetical key order for byte-stability.
    let _ = writeln!(out, "      \"ai_suggestion\": {},", e.ai_suggestion);
    let _ = writeln!(out, "      \"bytes\": {},", e.bytes);
    let _ = writeln!(out, "      \"instruction\": {},", e.instruction);
    let _ = writeln!(out, "      \"ir_node\": {},", e.ir_node);
    let _ = writeln!(out, "      \"knowledge_fact\": {},", e.knowledge_fact);
    let _ = writeln!(out, "      \"user_hint\": {}", e.user_hint);
    out.push_str("    }\n");
    out.push_str("  },\n");
}

fn write_functions(out: &mut String, fns: &[FunctionAnnotation]) {
    if fns.is_empty() {
        out.push_str("  \"functions\": [],\n");
        return;
    }
    out.push_str("  \"functions\": [\n");
    for (i, f) in fns.iter().enumerate() {
        write_function(out, f, i + 1 == fns.len());
    }
    out.push_str("  ],\n");
}

fn write_function(out: &mut String, f: &FunctionAnnotation, last: bool) {
    out.push_str("    {\n");
    let _ = writeln!(out, "      \"address\": \"{:#018x}\",", f.address);
    let end_value = match f.end {
        Some(e) => format!("\"{e:#018x}\""),
        None => "null".to_string(),
    };
    let _ = writeln!(out, "      \"end\": {end_value},");
    write_signals(out, f.signals);
    write_fact(out, "name", &f.name, false);
    write_fact(out, "return_type", &f.return_type, true);
    let close = if last { "    }\n" } else { "    },\n" };
    out.push_str(close);
}

fn write_signals(out: &mut String, mask: SourceMask) {
    let labels = signal_labels(mask);
    out.push_str("      \"signals\": [");
    for (i, l) in labels.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&json_string(l));
    }
    out.push_str("],\n");
}

fn signal_labels(mask: SourceMask) -> Vec<&'static str> {
    // Alphabetical for byte-stability across runs.
    let mut labels: Vec<&'static str> = Vec::new();
    if mask.contains(SourceMask::CALL) {
        labels.push("call");
    }
    if mask.contains(SourceMask::ENTRY) {
        labels.push("entry");
    }
    if mask.contains(SourceMask::PROLOGUE) {
        labels.push("prologue");
    }
    if mask.contains(SourceMask::SYMBOL) {
        labels.push("symbol");
    }
    labels
}

fn write_fact(out: &mut String, key: &str, fact: &FactAnnotation, last: bool) {
    let _ = writeln!(out, "      \"{key}\": {{");
    let _ = writeln!(out, "        \"value\": {},", json_string(&fact.value));
    let _ = writeln!(out, "        \"confidence\": {{");
    let _ = writeln!(out, "          \"value\": {:.3},", fact.confidence.value());
    let _ = writeln!(
        out,
        "          \"source\": {}",
        json_string(fact.confidence.source().name())
    );
    out.push_str("        },\n");
    let _ = writeln!(
        out,
        "        \"explanation\": {},",
        json_string(&fact.explanation)
    );
    write_evidence_list(out, &fact.evidence);
    let close = if last { "      }\n" } else { "      },\n" };
    out.push_str(close);
}

fn write_evidence_list(out: &mut String, evidence: &[EvidenceRef]) {
    if evidence.is_empty() {
        out.push_str("        \"evidence\": []\n");
        return;
    }
    out.push_str("        \"evidence\": [\n");
    for (i, e) in evidence.iter().enumerate() {
        let last = i + 1 == evidence.len();
        write_evidence_ref(out, e, last);
    }
    out.push_str("        ]\n");
}

fn write_evidence_ref(out: &mut String, e: &EvidenceRef, last: bool) {
    out.push_str("          {");
    let _ = write!(out, "\"id\": {}, \"kind\": \"{}\"", e.id, e.kind.name());
    match &e.kind {
        EvidenceRefKind::Bytes { start, end } => {
            let _ = write!(
                out,
                ", \"start\": \"{start:#018x}\", \"end\": \"{end:#018x}\""
            );
        }
        EvidenceRefKind::Instruction { id } => {
            let _ = write!(out, ", \"instruction_id\": {id}");
        }
        EvidenceRefKind::IrNode { layer, node_id } => {
            let _ = write!(
                out,
                ", \"layer\": \"{}\", \"node_id\": {node_id}",
                ir_layer_name(*layer),
            );
        }
        EvidenceRefKind::KnowledgeFact { id } => {
            let _ = write!(out, ", \"fact_id\": {id}");
        }
        EvidenceRefKind::UserHint { id } => {
            let _ = write!(out, ", \"hint_id\": {id}");
        }
        EvidenceRefKind::AiSuggestion { prompt_hash } => {
            let _ = write!(out, ", \"prompt_hash\": \"{}\"", hex_hash(prompt_hash));
        }
    }
    if last {
        out.push_str("}\n");
    } else {
        out.push_str("},\n");
    }
}

fn write_notes(out: &mut String, notes: &[String]) {
    if notes.is_empty() {
        out.push_str("  \"notes\": []\n");
        return;
    }
    out.push_str("  \"notes\": [\n");
    for (i, n) in notes.iter().enumerate() {
        let comma = if i + 1 == notes.len() { "" } else { "," };
        let _ = writeln!(out, "    {}{}", json_string(n), comma);
    }
    out.push_str("  ]\n");
}

/// Render a per-function annotation block as plain text suitable for
/// embedding in a C source comment. Used by `--debug` to surface
/// "Why this name?" / "Why this return type?" inline (spec §12).
///
/// The output is line-oriented and does not contain `*/`, so it is
/// safe to drop into a `/* … */` block.
pub(crate) fn render_function_debug_block(f: &FunctionAnnotation) -> String {
    let mut out = String::new();
    out.push_str("Why this name?\n");
    fact_debug_lines(&mut out, &f.name);
    out.push_str("Why this return type?\n");
    fact_debug_lines(&mut out, &f.return_type);
    out
}

fn fact_debug_lines(out: &mut String, fact: &FactAnnotation) {
    let _ = writeln!(
        out,
        "  value:       {} ({}/{:.3})",
        fact.value,
        fact.confidence.source().name(),
        fact.confidence.value(),
    );
    let _ = writeln!(out, "  explanation: {}", fact.explanation);
    if fact.evidence.is_empty() {
        out.push_str("  evidence:    (none)\n");
    } else {
        out.push_str("  evidence:    ");
        for (i, e) in fact.evidence.iter().enumerate() {
            if i > 0 {
                out.push_str("; ");
            }
            evidence_ref_inline(out, e);
        }
        out.push('\n');
    }
}

fn evidence_ref_inline(out: &mut String, e: &EvidenceRef) {
    let _ = write!(out, "#{} {}", e.id, e.kind.name());
    match &e.kind {
        EvidenceRefKind::Bytes { start, end } => {
            let _ = write!(out, "[{start:#x}..{end:#x})");
        }
        EvidenceRefKind::Instruction { id } => {
            let _ = write!(out, "[id={id}]");
        }
        EvidenceRefKind::IrNode { layer, node_id } => {
            let _ = write!(out, "[{}, id={node_id}]", ir_layer_name(*layer));
        }
        EvidenceRefKind::KnowledgeFact { id } => {
            let _ = write!(out, "[id={id}]");
        }
        EvidenceRefKind::UserHint { id } => {
            let _ = write!(out, "[id={id}]");
        }
        EvidenceRefKind::AiSuggestion { prompt_hash } => {
            let _ = write!(out, "[prompt={}]", short_hash(prompt_hash));
        }
    }
}

fn ir_layer_name(layer: IrLayer) -> &'static str {
    match layer {
        IrLayer::Instruction => "instruction",
        IrLayer::Cfg => "cfg",
        IrLayer::Ssa => "ssa",
        IrLayer::Semantic => "semantic",
        IrLayer::Source => "source",
    }
}

fn hex_hash(h: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in h {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn short_hash(h: &[u8; 32]) -> String {
    let mut s = String::with_capacity(16);
    for b in h.iter().take(8) {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn json_bool(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_arch::{DecodeError, DecodedInstruction, InstructionDecoder};
    use dac_binfmt::{
        Architecture, BinaryFormat, Bits, Endian, Permissions, Section, SectionKind, Symbol,
        SymbolBinding, SymbolKind, SymbolSource,
    };
    use dac_recovery::discover_functions;

    fn text_symbol(name: &str, address: u64, size: u64) -> Symbol {
        Symbol {
            name: name.to_string(),
            address,
            size,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: Some(0),
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    struct NullDecoder;
    impl InstructionDecoder for NullDecoder {
        fn decode_one(
            &self,
            _bytes: &[u8],
            _address: u64,
        ) -> Result<DecodedInstruction, DecodeError> {
            Err(DecodeError::Truncated { offset: 0 })
        }
        fn iter<'a>(
            &'a self,
            _bytes: &'a [u8],
            _address: u64,
        ) -> Box<dyn Iterator<Item = DecodedInstruction> + 'a> {
            Box::new(std::iter::empty())
        }
    }

    fn base_model() -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: Some(0x1000),
            size: 0x100,
            sections: vec![Section {
                name: ".text".to_string(),
                address: 0x1000,
                size: 0x100,
                file_offset: Some(0),
                perms: Permissions {
                    readable: true,
                    writable: false,
                    executable: true,
                },
                kind: SectionKind::Text,
            }],
            segments: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
            needed_libraries: Vec::new(),
        }
    }

    fn stamp_pair() -> (ToolStamp, InputStamp, SettingsStamp) {
        (
            ToolStamp {
                name: "dac".to_string(),
                version: "0.1.0".to_string(),
                build_id: "dev".to_string(),
            },
            InputStamp {
                path: "fixture".to_string(),
                format: "ELF".to_string(),
                architecture: "x86-64".to_string(),
                size: 0x100,
            },
            SettingsStamp {
                level: "O1".to_string(),
                target: "c".to_string(),
                debug: false,
            },
        )
    }

    fn build_doc(model: &BinaryModel) -> AnnotationDoc {
        build_doc_with_hints(model, Hints::new())
    }

    fn build_doc_with_hints(model: &BinaryModel, hints: Hints) -> AnnotationDoc {
        let bytes = vec![0u8; 0x100];
        let mut graph = EvidenceGraph::new();
        let set = discover_functions(model, &bytes, &NullDecoder, &mut graph);
        // Mirror the CLI's `register_hints`: every hint gets a
        // `EvidenceNode::UserHint` minted in the same graph so the
        // annotation channel can cite the exact node.
        let mut hints = hints;
        for h in hints.functions.iter_mut() {
            h.evidence = Some(graph.add_node(EvidenceNode::UserHint(h.id)));
        }
        for h in hints.structs.iter_mut() {
            h.evidence = Some(graph.add_node(EvidenceNode::UserHint(h.id)));
        }
        let (tool, input, settings) = stamp_pair();
        AnnotationDoc::build(tool, input, settings, model, &set, &graph, &hints)
    }

    fn function_hint(
        id: u64,
        matcher: dac_hints::HintMatcher,
        rename: Option<&str>,
        return_ty: Option<dac_hints::HintType>,
    ) -> FunctionHint {
        FunctionHint {
            id,
            line: 7,
            matcher,
            rename: rename.map(str::to_string),
            return_ty,
            args: None,
            evidence: None,
        }
    }

    #[test]
    fn symbol_derived_name_renders_as_observed_with_evidence_chain() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let doc = build_doc(&model);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("function discovered");
        assert_eq!(f.name.value, "main");
        assert_eq!(f.name.confidence.source(), Source::Observed);
        assert!(f.name.confidence.value() >= 0.9);
        assert!(
            f.name.explanation.contains("symbol-table entry"),
            "expected symbol-table explanation, got: {}",
            f.name.explanation
        );
        assert!(
            f.name.evidence.len() >= 2,
            "expected ir-node + supporting bytes node in chain"
        );
        let has_bytes = f
            .name
            .evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceRefKind::Bytes { .. }));
        let has_ir = f
            .name
            .evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceRefKind::IrNode { .. }));
        assert!(has_bytes && has_ir);
    }

    #[test]
    fn synthesized_name_is_derived_with_zero_value() {
        let model = base_model();
        let doc = build_doc(&model);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("entry-point function discovered");
        assert_eq!(f.name.value, "fn_1000");
        assert_eq!(f.name.confidence.source(), Source::Derived);
        assert_eq!(f.name.confidence.value(), 0.0);
        assert!(f.name.explanation.contains("synthesized"));
    }

    #[test]
    fn return_type_is_void_derived_pending_signature_inference() {
        let model = base_model();
        let doc = build_doc(&model);
        let f = doc
            .functions
            .first()
            .expect("at least one function discovered");
        assert_eq!(f.return_type.value, "void");
        assert_eq!(f.return_type.confidence.source(), Source::Derived);
        assert_eq!(f.return_type.confidence.value(), 0.0);
        assert!(f.return_type.explanation.contains("B3.6"));
        assert!(f.return_type.evidence.is_empty());
    }

    #[test]
    fn render_annotations_json_is_byte_stable_across_calls() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let doc = build_doc(&model);
        let a = render_annotations_json(&doc);
        let b = render_annotations_json(&doc);
        assert_eq!(a, b);
    }

    #[test]
    fn render_annotations_json_carries_every_top_level_section() {
        let model = base_model();
        let doc = build_doc(&model);
        let s = render_annotations_json(&doc);
        assert!(s.contains("\"tool\""));
        assert!(s.contains("\"input\""));
        assert!(s.contains("\"settings\""));
        assert!(s.contains("\"evidence\""));
        assert!(s.contains("\"functions\""));
        assert!(s.contains("\"notes\""));
    }

    #[test]
    fn evidence_summary_counts_match_graph() {
        let model = base_model();
        let bytes = vec![0u8; 0x100];
        let mut graph = EvidenceGraph::new();
        let _ = discover_functions(&model, &bytes, &NullDecoder, &mut graph);
        let summary = summarize_graph(&graph);
        assert_eq!(summary.node_count, graph.node_count() as u64);
        assert_eq!(
            summary.bytes
                + summary.instruction
                + summary.ir_node
                + summary.knowledge_fact
                + summary.user_hint
                + summary.ai_suggestion,
            summary.node_count,
        );
    }

    #[test]
    fn debug_block_renders_why_this_name_and_why_this_type() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let doc = build_doc(&model);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("function");
        let block = render_function_debug_block(f);
        assert!(block.contains("Why this name?"));
        assert!(block.contains("Why this return type?"));
        assert!(block.contains("value:"));
        assert!(block.contains("explanation:"));
        assert!(block.contains("evidence:"));
        assert!(block.contains("main"));
        assert!(!block.contains("*/"), "must be safe in /* … */");
    }

    #[test]
    fn empty_function_set_produces_an_explanatory_note() {
        // The base model has an entry point, so discover_functions still
        // finds at least one function. Force an empty set by using a
        // model with no entry and no symbols and no executable bytes.
        let mut model = base_model();
        model.entry = None;
        model.sections.clear();
        let doc = build_doc(&model);
        assert!(doc.functions.is_empty());
        assert_eq!(doc.notes.len(), 1);
        let json = render_annotations_json(&doc);
        assert!(json.contains("\"functions\": []"));
        assert!(json.contains("\"notes\": ["));
    }

    #[test]
    fn signals_list_is_alphabetical() {
        let mut mask = SourceMask::empty();
        mask.insert(SourceMask::SYMBOL);
        mask.insert(SourceMask::ENTRY);
        mask.insert(SourceMask::CALL);
        assert_eq!(signal_labels(mask), vec!["call", "entry", "symbol"]);
    }

    #[test]
    fn json_string_escapes_quote_and_control() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("\n"), "\"\\n\"");
        assert_eq!(json_string("\x05"), "\"\\u0005\"");
    }

    #[test]
    fn evidence_chain_terminates_on_cycle() {
        // The append-only graph permits self-loops; the BFS in
        // `evidence_chain` must dedup by node id so cycles do not loop.
        let mut g = EvidenceGraph::new();
        let a = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        let b = g.add_node(EvidenceNode::Bytes { start: 0, end: 4 });
        assert!(g.add_edge(b, a, EdgeKind::Supports));
        assert!(g.add_edge(a, b, EdgeKind::Supports));
        let preds = predecessor_index(&g);
        let chain = evidence_chain(&g, &preds, a);
        // a (root) + b (predecessor) — exactly two entries, no cycle.
        assert_eq!(chain.len(), 2);
    }

    // B3.19: a `[[function]]` rename hint shows up in the
    // annotation channel as the overriding name with
    // `Source::UserHint` confidence, an explanation citing the
    // hint's source line, and an evidence chain anchored on the
    // hint's `EvidenceNode::UserHint` node.
    #[test]
    fn function_hint_rename_pins_name_with_user_hint_source() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let mut hints = Hints::new();
        hints.functions.push(function_hint(
            42,
            dac_hints::HintMatcher::Address(0x1000),
            Some("user_main"),
            None,
        ));
        let doc = build_doc_with_hints(&model, hints);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("function discovered");
        assert_eq!(f.name.value, "user_main", "hint rename must win");
        assert_eq!(f.name.confidence.source(), Source::UserHint);
        assert!((f.name.confidence.value() - USER_HINT_CONFIDENCE).abs() < f32::EPSILON);
        assert!(
            f.name.explanation.contains("user hint"),
            "explanation must cite the hint, got: {}",
            f.name.explanation,
        );
        // Chain rooted at the hint's UserHint node — no
        // symbol-table evidence appears because the hint *replaces*
        // the symbol-table observation.
        let has_user_hint = f
            .name
            .evidence
            .iter()
            .any(|e| matches!(e.kind, EvidenceRefKind::UserHint { .. }));
        assert!(has_user_hint, "evidence chain must cite the user hint");
        let has_symbol_chain = f.name.evidence.iter().any(|e| {
            matches!(
                e.kind,
                EvidenceRefKind::Bytes { .. } | EvidenceRefKind::IrNode { .. }
            )
        });
        assert!(
            !has_symbol_chain,
            "hint-cited name must not carry the symbol-table chain underneath",
        );
    }

    // B3.19: a `[[function]]` `return` hint shows up in the
    // annotation channel as the overriding return type with
    // `Source::UserHint` confidence and an evidence chain rooted
    // on the hint node.
    #[test]
    fn function_hint_return_pins_return_type_with_user_hint_source() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let mut hints = Hints::new();
        hints.functions.push(function_hint(
            7,
            dac_hints::HintMatcher::Address(0x1000),
            None,
            Some(dac_hints::HintType::Int {
                width_bits: 32,
                signed: Some(true),
            }),
        ));
        let doc = build_doc_with_hints(&model, hints);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("function discovered");
        assert_eq!(f.return_type.value, "int32_t");
        assert_eq!(f.return_type.confidence.source(), Source::UserHint);
        assert!((f.return_type.confidence.value() - USER_HINT_CONFIDENCE).abs() < f32::EPSILON);
        assert!(f.return_type.explanation.contains("user hint"));
        assert!(matches!(
            f.return_type
                .evidence
                .first()
                .expect("hint evidence ref")
                .kind,
            EvidenceRefKind::UserHint { .. }
        ));
    }

    // B3.19: a hint without `rename` / `return` overrides leaves
    // both fact annotations on the deterministic-pipeline path.
    // The catch is the matched-but-passive hint must not be
    // accidentally cited.
    #[test]
    fn function_hint_without_overrides_does_not_alter_annotations() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let mut hints = Hints::new();
        hints.functions.push(function_hint(
            1,
            dac_hints::HintMatcher::Address(0x1000),
            None,
            None,
        ));
        let doc = build_doc_with_hints(&model, hints);
        let f = doc
            .functions
            .iter()
            .find(|f| f.address == 0x1000)
            .expect("function discovered");
        assert_eq!(f.name.value, "main");
        assert_eq!(f.name.confidence.source(), Source::Observed);
        assert_eq!(f.return_type.value, "void");
        assert_eq!(f.return_type.confidence.source(), Source::Derived);
    }

    // B3.19: the JSON sidecar must carry the hint's UserHint
    // evidence ref under the name fact when a rename applies.
    #[test]
    fn rendered_json_cites_user_hint_id_under_name() {
        let mut model = base_model();
        model.symbols.push(text_symbol("main", 0x1000, 0x10));
        let mut hints = Hints::new();
        hints.functions.push(function_hint(
            99,
            dac_hints::HintMatcher::Address(0x1000),
            Some("user_main"),
            None,
        ));
        let doc = build_doc_with_hints(&model, hints);
        let json = render_annotations_json(&doc);
        assert!(json.contains("\"kind\": \"user_hint\""));
        assert!(json.contains("\"hint_id\": 99"));
        assert!(json.contains("\"value\": \"user_main\""));
    }
}
