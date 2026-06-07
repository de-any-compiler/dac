//! `dac` — command-line frontend.
//!
//! Status: B0.5 declared the full CLI surface from spec §10.1. B1.6 wires
//! `-O0` end-to-end: the binary is loaded through `dac-binfmt`, functions
//! are discovered through `dac-recovery`, and a deterministic annotated
//! listing (plus a reproducibility manifest per NFR-10 and an optional
//! analysis report per FR-25) is emitted on the user-selected output
//! path. `--emit-ir`, `--emit-annotations`, `--ai-provider`, `--no-ai`,
//! and `--plugin` are still parsed but become live milestone by
//! milestone (M2 / M4 / M5). `--deterministic` is surfaced on the
//! manifest today; manager-level enforcement remains covered by
//! `dac-core` unit tests.
//!
//! B2.8 wires `--target c` at `-O1`+ end-to-end through
//! `dac-backend-c`: a C translation unit lands at `<output>.c`
//! alongside the listing. The per-function body is a stub until the
//! lifter → `RawFunction` bridge (the InstructionIR → SSA wiring) is
//! a batch in PLAN.md — see [`render_c_unit`].

#![forbid(unsafe_code)]

mod annotations;
mod lift;
mod listing;
mod manifest;
mod report;
mod xrefs;

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dac_analysis::cfg::{build_cfgs, render_dot_all};
use dac_analysis::{build_call_graph, build_xref_index, render_callgraph_dot, resolve_subject};
use dac_arch::{Architecture as _, InstructionDecoder, InstructionLifter, RegisterFile};
use dac_arch_x86::X86_64;
use dac_backend_c::ast::{
    Block as CBlock, CType, Expr as CExpr, ExternDecl as CExternDecl, Function as CFunction,
    Item as CItem, Param as CParam, Stmt as CStmt, TranslationUnit,
};
use dac_backend_c::{
    default_includes as c_default_includes, emit as c_emit,
    lower_function_with_options as c_lower_function_with_options, LowerOptions as CLowerOptions,
    LoweredFunction, NameResolver as CNameResolver, Recovered as CRecovered,
    StringTable as CStringTable,
};
use dac_backend_cpp::{
    class_recovery::recover_classes as recover_cpp_classes,
    default_includes as cpp_default_includes, emit as cpp_emit, lower_unit as cpp_lower_unit,
};
use dac_binfmt::{load_from_bytes, Architecture, BinaryModel};
use dac_core::{init_tracing, EvidenceGraph};
use dac_hints::{HintError, Hints};
use dac_recovery::{detect_thunks, discover_functions, FunctionSet, FunctionTaxonomy};

use crate::annotations::{
    render_annotations_json, render_function_debug_block, AnnotationDoc, FunctionAnnotation,
    InputStamp, SettingsStamp, ToolStamp,
};
use crate::lift::{lift_all, register_hints, InferredReturn, LiftOutcome, LiftStats};
use crate::listing::{render_listing, ListingOptions};
use crate::manifest::{
    render_manifest_json, Manifest, ManifestInput, ManifestSettings, ManifestTool,
};
use crate::report::{render_report_text, Report};
use crate::xrefs::render_xrefs_report;

const HELP_TEXT: &str = include_str!("../tests/snapshots/help.txt");

const BUILD_ID: &str = match option_env!("DAC_BUILD_ID") {
    Some(s) => s,
    None => "dev",
};

fn main() -> ExitCode {
    let args = match parse_args(std::env::args_os().skip(1)) {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("dac: {msg}");
            print_usage_hint();
            return ExitCode::from(2);
        }
    };

    if args.show_help {
        print!("{HELP_TEXT}");
        return ExitCode::SUCCESS;
    }
    if args.show_version {
        println!("dac {} ({BUILD_ID})", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    init_tracing(args.json, args.debug);

    let Some(input) = args.input.clone() else {
        eprintln!("dac: missing input binary path");
        print_usage_hint();
        return ExitCode::from(2);
    };

    if args.deterministic {
        tracing::info!("deterministic mode requested");
    }

    tracing::debug!(
        opt = args.opt.as_str(),
        format = args.format.as_str(),
        target = args.target.as_str(),
        json = args.json,
        debug = args.debug,
        deterministic = args.deterministic,
        no_ai = args.no_ai,
        ai_provider = ?args.ai_provider,
        threads = ?args.threads,
        arch = ?args.arch,
        output = ?args.output,
        emit_ir = args.emit_ir,
        emit_cfg = args.emit_cfg,
        emit_report = args.emit_report,
        emit_annotations = args.emit_annotations,
        emit_callgraph = args.emit_callgraph,
        xrefs = ?args.xrefs_subject,
        plugin = ?args.plugin,
        hints = ?args.hints,
        "parsed cli arguments"
    );

    match run_pipeline(&input, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(
                error = %err,
                path = %input.display(),
                "pipeline failed"
            );
            ExitCode::FAILURE
        }
    }
}

fn run_pipeline(path: &Path, args: &Args) -> dac_core::Result<()> {
    let bytes = std::fs::read(path)?;
    let model = load_from_bytes(&bytes)?;
    tracing::info!(
        format = %model.format.name(),
        arch = %model.architecture.name(),
        size = model.size,
        sections = model.sections.len(),
        segments = model.segments.len(),
        symbols = model.symbols.len(),
        imports = model.imports.len(),
        exports = model.exports.len(),
        relocations = model.relocations.len(),
        strings = model.strings.len(),
        needed_libraries = model.needed_libraries.len(),
        entry = ?model.entry,
        deterministic = args.deterministic,
        path = %path.display(),
        "loaded input"
    );

    let input_label = path.to_string_lossy().into_owned();
    let manifest = build_manifest(&input_label, &model, args);
    let manifest_json = render_manifest_json(&manifest);

    let hints = match args.hints.as_deref() {
        Some(p) => Hints::load_from_path(p).map_err(hints_error_to_core)?,
        None => Hints::new(),
    };

    // The pipeline picks a backend lazily so that formats without an
    // arch backend (everything but x86-64 today) still produce the
    // manifest + a minimal listing rather than failing.
    let backend = pick_backend(&model);
    let listing_text;
    let report_text;
    let cfg_text;
    let source_text;
    let callgraph_text;
    let xrefs_text;
    let annotations_doc;
    match &backend {
        Some(b) => {
            let mut graph = EvidenceGraph::new();
            let mut functions = discover_functions(&model, &bytes, b.decoder.as_ref(), &mut graph);
            // B3.25: reclassify `[endbr64?]; jmp <known function>`
            // bodies as `FunctionKind::Thunk { target }` so the C
            // backend can render them as one-line tail calls instead
            // of structuring-fallback stubs (FR-21).
            detect_thunks(&mut functions, &model, &bytes, b.decoder.as_ref());
            // B3.6: register every loaded hint as an
            // `EvidenceNode::UserHint` in the same graph so the
            // annotation channel can cite them and the per-binary
            // user_hint summary in the report matches the graph's
            // by-kind histogram.
            let hints = register_hints(hints, &mut graph);
            listing_text = render_listing(
                &input_label,
                &model,
                &bytes,
                b.decoder.as_ref(),
                b.lifter.as_ref(),
                &functions,
                &ListingOptions::default(),
            );
            // The B3.9 orchestrator runs the per-function lift exactly
            // once and shares the outcomes with both the `--target c`
            // source emitter and the optional `--emit-report` so the
            // two views agree on which functions ended up with a real
            // body. `-O0` and `--target cpp` paths don't need it; the
            // call is cheap enough on the corpus that the
            // unconditional cost is worth the simplicity.
            let lift_outcomes = if args.opt != OptLevel::O0 || args.emit_report {
                Some(lift_all(
                    &functions,
                    &model,
                    &bytes,
                    b.decoder.as_ref(),
                    b.lifter.as_ref(),
                    b.register_file,
                    &hints,
                ))
            } else {
                None
            };
            let lift_stats = lift_outcomes
                .as_deref()
                .map(LiftStats::from)
                .unwrap_or_default();
            if args.emit_report {
                let r = Report::build(
                    &model,
                    &bytes,
                    b.decoder.as_ref(),
                    b.lifter.as_ref(),
                    &functions,
                    lift_stats,
                );
                report_text = Some(render_report_text(&r));
            } else {
                report_text = None;
            }
            cfg_text = if args.emit_cfg {
                let cfgs = build_cfgs(&functions.functions, &model, &bytes, b.decoder.as_ref());
                Some(render_dot_all(&cfgs))
            } else {
                None
            };
            let inferred_returns =
                inferred_returns_from_outcomes(lift_outcomes.as_deref(), functions.functions.len());
            annotations_doc = build_annotations_doc(
                &input_label,
                &model,
                args,
                &functions,
                &graph,
                &hints,
                &inferred_returns,
            );
            source_text = render_source_text(
                args,
                &input_label,
                &model,
                &functions,
                lift_outcomes.as_deref(),
                &annotations_doc,
                Some(&mut graph),
            );
            callgraph_text = if args.emit_callgraph {
                let cg = build_call_graph(&model, &bytes, b.decoder.as_ref(), &functions);
                Some(render_callgraph_dot(&cg, &input_label))
            } else {
                None
            };
            xrefs_text = match &args.xrefs_subject {
                Some(raw) => {
                    let xidx = build_xref_index(&model, &bytes, b.decoder.as_ref(), &functions);
                    match resolve_subject(raw, &model, &functions) {
                        Some((va, name)) => Some(render_xrefs_report(
                            raw,
                            va,
                            name.as_deref(),
                            &xidx,
                            &model,
                            &functions,
                        )),
                        None => Some(unresolved_xrefs_subject_text(raw)),
                    }
                }
                None => None,
            };
        }
        None => {
            listing_text = unsupported_arch_listing(&input_label, &model);
            report_text = if args.emit_report {
                Some(unsupported_arch_report(&model))
            } else {
                None
            };
            cfg_text = if args.emit_cfg {
                Some(unsupported_arch_cfg(&model))
            } else {
                None
            };
            let empty_set = FunctionSet {
                functions: Vec::new(),
                stats: Default::default(),
            };
            let mut empty_graph = EvidenceGraph::new();
            let empty_hints = Hints::new();
            annotations_doc = build_annotations_doc(
                &input_label,
                &model,
                args,
                &empty_set,
                &empty_graph,
                &empty_hints,
                &[],
            );
            source_text = render_source_text(
                args,
                &input_label,
                &model,
                &empty_set,
                None,
                &annotations_doc,
                Some(&mut empty_graph),
            );
            callgraph_text = if args.emit_callgraph {
                Some(unsupported_arch_callgraph(&model))
            } else {
                None
            };
            xrefs_text = args
                .xrefs_subject
                .as_deref()
                .map(unresolved_xrefs_subject_text);
        }
    }

    let annotations_text = if args.emit_annotations {
        Some(render_annotations_json(&annotations_doc))
    } else {
        None
    };

    let source_suffix = match args.target {
        Target::C => ".c",
        Target::Cpp => ".cpp",
    };

    emit_outputs(
        args.output.as_deref(),
        &listing_text,
        &manifest_json,
        report_text.as_deref(),
        cfg_text.as_deref(),
        source_text.as_deref(),
        source_suffix,
        callgraph_text.as_deref(),
        xrefs_text.as_deref(),
        annotations_text.as_deref(),
    )
}

fn build_annotations_doc(
    input_label: &str,
    model: &BinaryModel,
    args: &Args,
    functions: &FunctionSet,
    graph: &EvidenceGraph,
    hints: &Hints,
    inferred_returns: &[Option<InferredReturn>],
) -> AnnotationDoc {
    AnnotationDoc::build(
        ToolStamp {
            name: "dac".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_id: BUILD_ID.to_string(),
        },
        InputStamp {
            path: input_label.to_string(),
            format: model.format.name().to_string(),
            architecture: model.architecture.name().to_string(),
            size: model.size as u64,
        },
        SettingsStamp {
            level: args.opt.as_str().to_string(),
            target: args.target.as_str().to_string(),
            debug: args.debug,
        },
        model,
        functions,
        graph,
        hints,
        inferred_returns,
    )
}

/// Project lift outcomes to the B3.29 inferred-return slice the
/// annotation channel expects. Entries are ordered to match
/// `functions.functions`; stub outcomes (and missing outcomes) carry
/// `None` so the annotator's hint-then-inference path degrades to
/// the unavailable-inference placeholder.
fn inferred_returns_from_outcomes(
    outcomes: Option<&[LiftOutcome]>,
    functions_len: usize,
) -> Vec<Option<InferredReturn>> {
    match outcomes {
        Some(os) => os
            .iter()
            .map(|o| match o {
                LiftOutcome::Real { facts, .. } => Some(facts.inferred_return),
                LiftOutcome::Stub { .. } => None,
            })
            .collect(),
        None => vec![None; functions_len],
    }
}

fn unresolved_xrefs_subject_text(subject: &str) -> String {
    format!(
        ";; dac --xrefs report (FR-26, FR-31)\n;; subject:   {subject}\n;; (unresolved: no matching symbol or address)\n",
    )
}

fn unsupported_arch_callgraph(model: &BinaryModel) -> String {
    format!(
        "digraph \"callgraph_unsupported_arch_{}\" {{\n}}\n",
        sanitize_dot_ident(model.architecture.name())
    )
}

/// Decide whether the `--target` / `-O` combination wants a C
/// translation unit, and if so build it.
///
/// B3.9 wires the C backend end-to-end at `-O1` (and above): the
/// per-function pipeline runs `build_cfg → lift_function →
/// construct_ssa → structure → lower_function` for every recovered
/// function. Functions whose CFG cannot be built (no recovered `end`
/// address, byte range outside an executable section, empty after
/// decode) fall back to a stub body whose leading comment records
/// the reason — I-6: the unit is still valid C so the round-trip
/// compile gate (`dac_backend_c::try_compile`) holds on the corpus.
///
/// `--target cpp` continues to emit class-shaped stubs because the
/// C++ AST does not model bodies. Extending the AST to carry lowered
/// bodies is on the B3 follow-up shelf in PLAN.md.
fn render_source_text(
    args: &Args,
    input_label: &str,
    model: &BinaryModel,
    functions: &FunctionSet,
    lift_outcomes: Option<&[LiftOutcome]>,
    annotations: &AnnotationDoc,
    graph: Option<&mut EvidenceGraph>,
) -> Option<String> {
    if args.opt == OptLevel::O0 {
        return None;
    }
    match args.target {
        Target::C => Some(render_c_unit(
            input_label,
            model,
            functions,
            lift_outcomes,
            annotations,
            args.debug,
            args.hide_crt,
            args.opt,
        )),
        Target::Cpp => Some(render_cpp_unit(
            input_label,
            model,
            functions,
            graph,
            args.debug,
            args.opt,
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_c_unit(
    input_label: &str,
    model: &BinaryModel,
    functions: &FunctionSet,
    lift_outcomes: Option<&[LiftOutcome]>,
    annotations: &AnnotationDoc,
    debug: bool,
    hide_crt: bool,
    opt: OptLevel,
) -> String {
    let resolver = build_c_name_resolver(functions);
    // B3.32: build the absolute-address → recovered-text index from
    // the per-section `StringRef` entries the binfmt scanner already
    // populated. The C lowering pass consults this through
    // `Recovered::strings` to swap call-arg constants whose value
    // matches a known string pointer with the literal text the
    // pointer references.
    let string_table: CStringTable = build_c_string_table(model);
    // The orchestrator returns outcomes in the same order as
    // `functions.functions`; the zip below guarantees the i-th entry
    // pairs with the i-th function. When the orchestrator was skipped
    // (no callers requested it), every function degrades to a stub
    // with the same reason — this keeps the matching call sites
    // simple while preserving the I-6 visible-degradation contract.
    //
    // B3.16: each lowered function may carry struct typedefs the
    // pointer-anchored recovery promoted. We collect them all,
    // deduplicate by typedef name, and prepend the resulting
    // `StructDecl` items so each typedef is in scope at every
    // function that references it.
    let mut typedefs: std::collections::BTreeMap<String, dac_backend_c::ast::StructDecl> =
        std::collections::BTreeMap::new();
    // B3.23: PLT-bound functions surface as `extern <sig> name(...);`
    // forward declarations instead of bodies. Collect them separately
    // so the rendered translation unit starts with imports, then
    // typedefs, then user-function bodies — a stable ordering callers
    // (and the golden tests) can rely on.
    let mut extern_items: Vec<CItem> = Vec::new();
    let mut function_items: Vec<CItem> = Vec::with_capacity(functions.functions.len());
    for (i, f) in functions.functions.iter().enumerate() {
        let outcome = lift_outcomes.and_then(|os| os.get(i));
        let annot = annotations
            .functions
            .iter()
            .find(|a| a.address == f.address);
        if let dac_recovery::FunctionKind::PltStub { import } = &f.kind {
            extern_items.push(CItem::ExternDecl(lower_plt_stub_extern(f, import, debug)));
            continue;
        }
        // B3.30: `--hide-crt` collapses every CRT-tagged body (whether
        // it lowered as a thunk or a real body) to an `extern <sig>
        // name(...);` forward declaration so the rendered translation
        // unit is dominated by user code (FR-21). The recovered
        // signature still drives the extern's return type and
        // parameter list, so callers continue to type-check.
        if hide_crt && f.taxonomy() == FunctionTaxonomy::CrtSupport {
            extern_items.push(CItem::ExternDecl(hidden_crt_extern_decl(f, outcome, debug)));
            continue;
        }
        if let dac_recovery::FunctionKind::Thunk { target } = &f.kind {
            let mut thunk = lower_thunk_function(f, *target, &resolver, debug);
            thunk.leading_comment = prepend_crt_banner(thunk.leading_comment, f);
            function_items.push(CItem::Function(thunk));
            continue;
        }
        let mut lowered = lower_one_c_function(f, outcome, annot, &resolver, &string_table, debug);
        for decl in lowered.struct_decls {
            typedefs.entry(decl.name.clone()).or_insert(decl);
        }
        lowered.function.leading_comment = prepend_crt_banner(lowered.function.leading_comment, f);
        function_items.push(CItem::Function(lowered.function));
    }
    let mut items: Vec<CItem> = extern_items;
    items.extend(typedefs.into_values().map(CItem::StructDecl));
    items.extend(function_items);
    let mut includes = c_default_includes();
    includes.insert(
        0,
        format!(
            "/* dac --target c -{} reconstruction\n   input: {input_label}\n   arch:  {} */",
            opt.as_str(),
            model.architecture.name()
        ),
    );
    let unit = TranslationUnit { includes, items };
    c_emit(&unit)
}

/// Build the C-side [`CNameResolver`] from the recovered function
/// set. Every recovered function whose recovered name (or synthesised
/// `fn_<addr>` placeholder) sanitises to a valid C identifier lands
/// in the resolver keyed by its virtual address; the C backend then
/// lowers `Call { target: Some(addr), … }` to a named call instead of
/// the `((void (*)())0xNN)(…)` fallback.
fn build_c_name_resolver(functions: &FunctionSet) -> CNameResolver {
    let mut r = CNameResolver::new();
    for f in &functions.functions {
        let raw = f
            .name
            .clone()
            .unwrap_or_else(|| format!("fn_{:x}", f.address));
        let name = sanitize_c_identifier(&raw);
        r.insert(f.address, name);
    }
    r
}

/// Build the [`CStringTable`] from `BinaryModel::strings` (B3.32).
///
/// Each `StringRef` is keyed by its absolute virtual address
/// (`section.address + offset`). The C lowering pass reads through
/// `Recovered::strings` and rewrites `Operand::Const(c)` operands
/// whose `c` matches a key into [`CExpr::StringLit`] so the
/// recovered text surfaces next to the call instead of the bare
/// integer the SSA carries. Entries whose `section` index is out
/// of range (defensive — the binfmt scanner only populates from
/// the `sections` array it built) are dropped silently.
fn build_c_string_table(model: &BinaryModel) -> CStringTable {
    let mut t = CStringTable::new();
    for sref in &model.strings {
        let Some(section) = model.sections.get(sref.section) else {
            continue;
        };
        let address = section.address.saturating_add(sref.offset);
        t.entry(address).or_insert_with(|| sref.value.clone());
    }
    t
}

/// Lower a single recovered function to a C AST [`CFunction`].
///
/// `outcome` is the per-function orchestrator result (B3.9). On
/// [`LiftOutcome::Real`] we delegate to
/// [`dac_backend_c::lower_function`] and post-process the result's
/// name through [`sanitize_c_identifier`] so symbols like `_start`
/// or PLT thunks still produce valid C identifiers. On
/// [`LiftOutcome::Stub`] (or a missing outcome) we keep the B2.8
/// stub-body shape — empty params, `void` return, a single
/// `lifter→SSA bridge pending` comment — and surface the degradation
/// reason in the leading comment so a reader knows *why* the body is
/// stubbed (I-6).
#[allow(clippy::too_many_arguments)]
fn lower_one_c_function(
    f: &dac_recovery::Function,
    outcome: Option<&LiftOutcome>,
    annot: Option<&FunctionAnnotation>,
    resolver: &CNameResolver,
    strings: &CStringTable,
    debug: bool,
) -> LoweredFunction {
    // B3.6: an applied `rename` hint takes precedence over the
    // recovered symbol. The sanitiser still runs so a hint that
    // accidentally contains `@` or `.` cannot break the round-trip
    // compile gate.
    let renamed = outcome.and_then(|o| match o {
        LiftOutcome::Real { facts, .. } => facts.user_hint.as_ref().and_then(|h| h.rename.clone()),
        LiftOutcome::Stub { .. } => None,
    });
    let raw_name = renamed.unwrap_or_else(|| {
        f.name
            .clone()
            .unwrap_or_else(|| format!("fn_{:x}", f.address))
    });
    let sanitized = sanitize_c_identifier(&raw_name);
    match outcome {
        Some(LiftOutcome::Real { ssa, sem, facts }) => {
            // B3.29: materialise the inferred return-type override
            // here so the borrowed `&CType` survives the lowering
            // call. The canonical channel still wins at lowering
            // time when both are set; the inference fallback covers
            // every non-canonical callee.
            let inferred_return_ty = facts.inferred_return.to_c_type();
            let recovered = CRecovered::with_canonical(
                facts.convention.as_ref().map(|c| &c.signature),
                Some(&facts.types),
                Some(&facts.structs),
                Some(&facts.names),
                facts.canonical_signature.as_ref(),
            )
            .with_inferred_return(Some(&inferred_return_ty))
            .with_strings(Some(strings));
            let mut lowered = c_lower_function_with_options(
                ssa,
                sem,
                resolver,
                &recovered,
                CLowerOptions { debug },
            );
            // `lower_function_with_typedefs` derives the name from
            // `sem.function_name` (the recovered symbol); sanitise to
            // keep emitted C syntactically valid for symbols that
            // contain `@`, `.` and friends.
            lowered.function.name = sanitized;
            // Replace the structurer's auto-generated leading comment
            // (a duplicate of the address line plus the structurer
            // stats) with a single unified head that carries the
            // recovered-function provenance + structuring stats and the
            // recovered convention citation (B3.10). The `--debug`
            // block from the annotation channel is appended when
            // requested.
            lowered.function.leading_comment =
                Some(real_body_leading_comment(f, sem, facts, annot, debug));
            lowered
        }
        outcome => {
            let reason = match outcome {
                Some(LiftOutcome::Stub { reason }) => reason.clone(),
                _ => "orchestrator did not run for this function".into(),
            };
            LoweredFunction {
                function: CFunction {
                    name: sanitized,
                    return_type: CType::Void,
                    params: Vec::new(),
                    locals: Vec::new(),
                    body: CBlock {
                        stmts: vec![CStmt::Comment(format!(
                            "lifter→SSA bridge pending: {reason}"
                        ))],
                    },
                    leading_comment: Some(stub_body_leading_comment(f, annot, debug)),
                },
                struct_decls: Vec::new(),
            }
        }
    }
}

/// Lower a PLT-bound stub function (B3.23) to a C `extern` forward
/// declaration. Looks the import up in the `dac-knowledge` API
/// catalogue: when matched, the recovered signature drives the
/// rendered `return_type` and `params`; otherwise the declaration
/// falls back to `int64_t name(void)` so callers compile.
fn lower_plt_stub_extern(f: &dac_recovery::Function, import: &str, debug: bool) -> CExternDecl {
    let signature = dac_knowledge::lookup_api_signature(import);
    let return_type = signature
        .and_then(|s| dac_backend_c::map_ir_type(&s.return_ty))
        .unwrap_or(CType::Int {
            width_bits: 64,
            signed: true,
        });
    let params: Vec<CParam> = signature
        .map(|s| {
            s.parameters
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let ty = dac_backend_c::map_ir_type(&p.ty).unwrap_or(CType::Int {
                        width_bits: 64,
                        signed: true,
                    });
                    let name = if p.name.is_empty() {
                        format!("arg{i}")
                    } else {
                        p.name.to_string()
                    };
                    CParam { name, ty }
                })
                .collect()
        })
        .unwrap_or_default();
    let is_variadic = signature.map(|s| s.is_variadic).unwrap_or(false);
    let leading_comment = Some(plt_stub_leading_comment(
        f,
        import,
        signature.is_some(),
        debug,
    ));
    CExternDecl {
        name: sanitize_c_identifier(import),
        return_type,
        params,
        is_variadic,
        leading_comment,
    }
}

/// Lower a CRT-tagged function (B3.30, FR-21) to an `extern <sig>
/// name(...);` forward declaration when `--hide-crt` is set.
///
/// The signature comes from the orchestrator's recovered facts: when
/// a real body lifted the canonical / convention / inferred-return
/// pipeline produces a [`CRecovered`] the same way the body-lowering
/// path consumes, so the hidden declaration's return type and
/// parameter list match what the body would have rendered. The
/// leading comment carries the CRT banner ("runtime support
/// (<runtime>) — not user code") followed by the recovered address
/// range so a reviewer running `-Ohide-crt` can still trace the
/// elided body back to its bytes.
fn hidden_crt_extern_decl(
    f: &dac_recovery::Function,
    outcome: Option<&LiftOutcome>,
    debug: bool,
) -> CExternDecl {
    let raw_name = f
        .name
        .clone()
        .unwrap_or_else(|| format!("fn_{:x}", f.address));
    let sanitized = sanitize_c_identifier(&raw_name);
    // Default `int64_t name(void);` shape covers thunks (no recovered
    // signature) and the unsupported-arch path. Real-body outcomes
    // overwrite both fields with the convention-inferred / canonical
    // signature.
    let mut return_type = CType::Int {
        width_bits: 64,
        signed: true,
    };
    let mut params: Vec<CParam> = Vec::new();
    if let Some(LiftOutcome::Real { facts, .. }) = outcome {
        if let Some(canon) = facts.canonical_signature.as_ref() {
            if let Some(ty) = canon.return_type.clone() {
                return_type = ty;
            }
            params = canon
                .params
                .iter()
                .map(|p| CParam {
                    name: p.name.clone(),
                    ty: p.ty.clone(),
                })
                .collect();
        } else if let Some(c) = facts.convention.as_ref() {
            params = c
                .signature
                .int_args
                .iter()
                .enumerate()
                .map(|(i, _)| CParam {
                    name: format!("arg{i}"),
                    ty: CType::Int {
                        width_bits: 64,
                        signed: true,
                    },
                })
                .collect();
            return_type = facts.inferred_return.to_c_type();
        }
    }
    let mut leading = format!(
        "runtime support ({label}) — not user code\n\
         dac-recovered CRT helper (body hidden by --hide-crt)\n\
         address: {addr:#x}\n\
         end: {end}\n\
         confidence: {conf:.2} ({src:?})",
        label = f
            .name
            .as_deref()
            .and_then(dac_knowledge::lookup_crt_entry)
            .map(|e| e.runtime.label())
            .unwrap_or("CRT scaffolding"),
        addr = f.address,
        end = f
            .end
            .map(|e| format!("{e:#x}"))
            .unwrap_or_else(|| "?".to_string()),
        conf = f.confidence.value(),
        src = f.confidence.source(),
    );
    if let Some(role) = f
        .name
        .as_deref()
        .and_then(dac_knowledge::lookup_crt_entry)
        .map(|e| e.role)
    {
        leading.push_str(&format!("\nrole: {role}"));
    }
    if debug {
        leading.push_str("\nsignal: CRT");
    }
    CExternDecl {
        name: sanitized,
        return_type,
        params,
        is_variadic: false,
        leading_comment: Some(leading),
    }
}

/// Build the `/* … */` block that precedes a PLT-stub `extern`
/// declaration. Mirrors the shape of [`stub_body_leading_comment`]
/// so the rendered translation unit stays uniform: address range,
/// joined-discovery confidence, plus a one-line trampoline / import
/// pair so a reviewer can trace the binding back to the relocation
/// table.
fn plt_stub_leading_comment(
    f: &dac_recovery::Function,
    import: &str,
    signature_known: bool,
    debug: bool,
) -> String {
    let mut s = format!(
        "dac-recovered PLT stub\n\
         address: {:#x}\n\
         end: {}\n\
         confidence: {:.2} ({:?})\n\
         import: {} (signature: {})",
        f.address,
        f.end
            .map(|e| format!("{e:#x}"))
            .unwrap_or_else(|| "?".to_string()),
        f.confidence.value(),
        f.confidence.source(),
        import,
        if signature_known {
            "dac-knowledge"
        } else {
            "unknown — fell back to int64_t(void)"
        },
    );
    if debug {
        s.push_str("\nsignal: PLT");
    }
    s
}

/// Lower a forwarding thunk (B3.25, FR-21) to a C function whose body
/// is a single tail-call to the recovered target. The thunk's own
/// signature collapses to `void (void)` — a thunk's call-site
/// arguments pass through registers without dac having to materialise
/// them, and the void return reflects that the thunk itself never
/// writes a return register the caller can observe.
///
/// When the target name resolves through `resolver`, the call
/// emits as `target_name();`; otherwise it falls back to the
/// existing [`CExpr::Call`] cast through an `AddrLit` so the
/// emitter still produces compileable C (the function-pointer cast
/// dodges the implicit-declaration error a bare numeric call would
/// trip on the round-trip gate).
fn lower_thunk_function(
    f: &dac_recovery::Function,
    target: u64,
    resolver: &CNameResolver,
    debug: bool,
) -> CFunction {
    let raw_name = f
        .name
        .clone()
        .unwrap_or_else(|| format!("fn_{:x}", f.address));
    let sanitized = sanitize_c_identifier(&raw_name);
    let target_name = resolver.get(&target).cloned();
    let call = match &target_name {
        Some(name) => CExpr::DirectCall {
            name: name.clone(),
            args: Vec::new(),
        },
        None => CExpr::Call {
            target: Box::new(CExpr::AddrLit(target)),
            args: Vec::new(),
        },
    };
    let body = CBlock {
        stmts: vec![CStmt::ExprStmt(call)],
    };
    CFunction {
        name: sanitized,
        return_type: CType::Void,
        params: Vec::new(),
        locals: Vec::new(),
        body,
        leading_comment: Some(thunk_leading_comment(
            f,
            target,
            target_name.as_deref(),
            debug,
        )),
    }
}

/// Build the CRT banner line for a function classified as
/// [`FunctionTaxonomy::CrtSupport`]. Returns `None` for every other
/// taxonomy so callers can prepend without branching.
///
/// The banner reads `runtime support (<runtime>) — not user code`
/// where `<runtime>` comes from [`dac_knowledge::CrtRuntime::label`]
/// — `glibc startup` on ELF, `mingw-w64 startup` on PE. A function
/// classified `CrtSupport` whose catalogue entry is no longer
/// resolvable (renamed, removed) degrades to the generic
/// `CRT scaffolding` label so the banner stays informative without
/// the catalogue having to be exhaustive (I-6).
fn crt_banner_line(f: &dac_recovery::Function) -> Option<String> {
    if f.taxonomy() != FunctionTaxonomy::CrtSupport {
        return None;
    }
    let runtime_label = f
        .name
        .as_deref()
        .and_then(dac_knowledge::lookup_crt_entry)
        .map(|e| e.runtime.label())
        .unwrap_or("CRT scaffolding");
    Some(format!("runtime support ({runtime_label}) — not user code"))
}

/// Prepend the B3.30 CRT banner to a function's leading comment when
/// the recovered taxonomy is [`FunctionTaxonomy::CrtSupport`]. Other
/// taxonomies pass through unchanged.
fn prepend_crt_banner(existing: Option<String>, f: &dac_recovery::Function) -> Option<String> {
    let Some(banner) = crt_banner_line(f) else {
        return existing;
    };
    Some(match existing {
        Some(rest) => format!("{banner}\n{rest}"),
        None => banner,
    })
}

/// Build the `/* … */` block that precedes a forwarding-thunk
/// function body. Mirrors the shape of [`plt_stub_leading_comment`]
/// and [`stub_body_leading_comment`] so the rendered translation
/// unit stays uniform: address range, joined-discovery confidence,
/// plus a one-line `tail-call → <target>` pair so a reviewer can
/// trace the binding back to the recovered call edge.
fn thunk_leading_comment(
    f: &dac_recovery::Function,
    target: u64,
    target_name: Option<&str>,
    debug: bool,
) -> String {
    let target_display = match target_name {
        Some(name) => format!("{name} ({target:#x})"),
        None => format!("{target:#x}"),
    };
    let mut s = format!(
        "dac-recovered forwarding thunk\n\
         address: {:#x}\n\
         end: {}\n\
         confidence: {:.2} ({:?})\n\
         tail-call: {target_display}",
        f.address,
        f.end
            .map(|e| format!("{e:#x}"))
            .unwrap_or_else(|| "?".to_string()),
        f.confidence.value(),
        f.confidence.source(),
    );
    if debug {
        s.push_str("\nsignal: THUNK");
    }
    s
}

/// Build the leading comment for a recovered function whose body
/// degraded to a stub (no recovered end, `build_cfg` failed, …).
/// Lists the recovered address range and joined-discovery
/// confidence; with `--debug` embeds the per-fact "Why this name?"
/// / "Why this return type?" trail from the annotation channel
/// (spec §12, FR-25).
fn stub_body_leading_comment(
    f: &dac_recovery::Function,
    annot: Option<&FunctionAnnotation>,
    debug: bool,
) -> String {
    let mut s = format!(
        "dac-recovered function\n\
         address: {:#x}\n\
         end: {}\n\
         confidence: {:.2} ({:?})",
        f.address,
        f.end
            .map(|e| format!("{e:#x}"))
            .unwrap_or_else(|| "?".to_string()),
        f.confidence.value(),
        f.confidence.source()
    );
    if debug {
        if let Some(a) = annot {
            s.push_str("\n\n");
            s.push_str(&render_function_debug_block(a));
        }
    }
    s
}

/// Build the leading comment for a recovered function whose body
/// was lifted end-to-end. Combines the recovered-function header
/// (address / end / discovery confidence) with the structurer's
/// own stats (source blocks visited, gotos emitted, label count,
/// irreducible flag), the B3.10 recovery facts (convention name +
/// score, struct / switch counts), and — under `--debug` — the
/// annotation channel's "Why this name? / type?" trail.
fn real_body_leading_comment(
    f: &dac_recovery::Function,
    sem: &dac_ir::sem::SemFunction,
    facts: &crate::lift::RecoveryFacts,
    annot: Option<&FunctionAnnotation>,
    debug: bool,
) -> String {
    let mut s = format!(
        "dac-recovered function\n\
         address: {:#x}\n\
         end: {}\n\
         confidence: {:.2} ({:?})\n\
         source_blocks: {}\n\
         goto_count: {}\n\
         label_count: {}\n\
         irreducible: {}",
        f.address,
        f.end
            .map(|e| format!("{e:#x}"))
            .unwrap_or_else(|| "?".to_string()),
        f.confidence.value(),
        f.confidence.source(),
        sem.stats.source_blocks,
        sem.stats.goto_count,
        sem.stats.label_count,
        sem.stats.irreducible,
    );
    // B3.10: surface the recovered convention + side-table counts so
    // the reader sees what FR-13 / FR-14 / FR-17 / FR-18 produced for
    // this function (or didn't).
    match &facts.convention {
        Some(c) => {
            let regs: Vec<&str> = c.signature.int_args.iter().map(|a| a.register).collect();
            let arg_list = if regs.is_empty() {
                "(no register args)".to_string()
            } else {
                regs.join(",")
            };
            let return_desc = c.signature.return_register.unwrap_or("none");
            s.push_str(&format!(
                "\nconvention: {} (score {:.2})\n\
                 args: {arg_list}\n\
                 return_reg: {return_desc}",
                c.convention_name,
                c.confidence.value(),
            ));
        }
        None => s.push_str("\nconvention: (none inferred)"),
    }
    s.push_str(&format!(
        "\nstack_locals: {}\n\
         struct_layouts: pointer={} stack={}\n\
         switch_tables: {}",
        facts.stack_frame.locals.len(),
        facts.structs.pointer_structs.len(),
        facts.structs.stack_structs.len(),
        facts.idioms.switch_tables.len(),
    ));
    // B3.6: surface the applied user hint so a reader of the .c
    // sidecar sees that the printed signature was pinned by
    // `--hints`, not inferred from the bytes.
    if let Some(h) = facts.user_hint.as_ref() {
        let rename_desc = h.rename.as_deref().unwrap_or("(none)");
        s.push_str(&format!(
            "\nuser_hint: id={} rename={} return_override={} args_override={}",
            h.id, rename_desc, h.return_overridden, h.args_overridden,
        ));
    }
    if debug {
        if let Some(a) = annot {
            s.push_str("\n\n");
            s.push_str(&render_function_debug_block(a));
        }
    }
    s
}

/// Render the `--target cpp` translation unit at `-O1`+.
///
/// Runs symbol-driven class recovery on the binary's symbol table
/// (B3.5, FR-21), feeds the recovered table plus the recovered
/// function set through `dac-backend-cpp::lower_unit`, and prepends a
/// banner comment + the canonical C++ includes. The translation
/// unit's per-member / per-free-function bodies remain stubs at
/// B3.9 because the C++ AST in `dac-backend-cpp::ast` does not
/// model function bodies — extending it to thread the SSA →
/// SemFunction shape the C side now consumes is on the B3
/// follow-up shelf in PLAN.md. See [`render_c_unit`] for the C-side
/// path where B3.9 wired the bridge through end-to-end.
fn render_cpp_unit(
    input_label: &str,
    model: &BinaryModel,
    functions: &FunctionSet,
    graph: Option<&mut EvidenceGraph>,
    _debug: bool,
    opt: OptLevel,
) -> String {
    let mut local_graph = EvidenceGraph::new();
    let g: &mut EvidenceGraph = match graph {
        Some(g) => g,
        None => &mut local_graph,
    };
    let classes = recover_cpp_classes(model, functions, g);
    let mut unit = cpp_lower_unit(&classes, functions);
    let mut includes = cpp_default_includes();
    includes.insert(
        0,
        format!(
            "/* dac --target cpp -{} reconstruction\n   input: {input_label}\n   arch:  {}\n   classes: {} (polymorphic: {}) members: {} free: {} */",
            opt.as_str(),
            model.architecture.name(),
            classes.stats.classes,
            classes.stats.polymorphic_classes,
            classes.stats.member_functions,
            classes.stats.free_functions,
        ),
    );
    unit.includes = includes;
    cpp_emit(&unit)
}

/// Map a recovered symbol name to a C identifier. Replaces any
/// character that is not `[A-Za-z0-9_]` with `_`, and prefixes a
/// leading digit with `f_` so the result is always a valid identifier.
fn sanitize_c_identifier(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars();
    if let Some(first) = chars.next() {
        if first.is_ascii_digit() {
            out.push_str("f_");
            out.push(first);
        } else if first.is_ascii_alphabetic() || first == '_' {
            out.push(first);
        } else {
            out.push('_');
        }
    } else {
        return "anon".to_string();
    }
    for c in chars {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    out
}

struct Backend {
    decoder: Box<dyn InstructionDecoder>,
    lifter: Box<dyn InstructionLifter>,
    /// Held as `'static` so the lift orchestrator can thread it into
    /// `dac-lift::lift_function` without dragging an `Architecture`
    /// lifetime parameter through the CLI's plumbing. The trait method
    /// returns `&Self::Output` with elided lifetimes; calling the
    /// underlying lazy-init function directly recovers the `'static`
    /// the implementation actually has.
    register_file: &'static RegisterFile,
}

fn pick_backend(model: &BinaryModel) -> Option<Backend> {
    match model.architecture {
        Architecture::X86_64 => Some(Backend {
            decoder: X86_64.decoder(),
            lifter: X86_64.lifter(),
            register_file: x86_64_register_file_static(),
        }),
        _ => None,
    }
}

/// Borrow the x86-64 register file with its real `'static` lifetime.
///
/// `Architecture::register_file` returns `&RegisterFile` with the
/// elided `&self` lifetime — the trait can't promise `'static` because
/// not every arch's register file is interned. The x86-64 backend's
/// register file is `OnceLock`-backed and the `Architecture` value
/// itself is zero-sized, so we promote a `X86_64` to `static` and
/// hand back the trait-method's borrow; the borrow checker sees
/// `&'static self` and therefore `&'static RegisterFile`.
fn x86_64_register_file_static() -> &'static RegisterFile {
    static ARCH: X86_64 = X86_64;
    ARCH.register_file()
}

fn unsupported_arch_listing(input_name: &str, model: &BinaryModel) -> String {
    let mut s = String::new();
    s.push_str(";; dac -O0 annotated listing\n");
    s.push_str(&format!(";; input:     {input_name}\n"));
    s.push_str(&format!(";; format:    {}\n", model.format.name()));
    s.push_str(&format!(";; arch:      {}\n", model.architecture.name()));
    s.push_str(";; (no architecture backend available; listing skipped)\n");
    s
}

fn unsupported_arch_report(model: &BinaryModel) -> String {
    let mut s = String::new();
    s.push_str(";; dac analysis report (FR-25)\n");
    s.push_str(&format!(";; arch:      {}\n", model.architecture.name()));
    s.push_str(";; (no architecture backend available; report skipped)\n");
    s
}

fn unsupported_arch_cfg(model: &BinaryModel) -> String {
    // Emit a valid (empty) DOT digraph so downstream tooling that pipes
    // `dac --emit-cfg` into `dot` / `graphviz` does not fail to parse.
    // The graph name records why the file is empty.
    format!(
        "digraph \"unsupported_arch_{}\" {{\n}}\n",
        sanitize_dot_ident(model.architecture.name())
    )
}

fn sanitize_dot_ident(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn build_manifest(input_path: &str, model: &BinaryModel, args: &Args) -> Manifest {
    Manifest {
        tool: ManifestTool {
            name: "dac".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_id: BUILD_ID.to_string(),
        },
        input: ManifestInput {
            path: input_path.to_string(),
            size: model.size as u64,
            format: model.format.name().to_string(),
            architecture: model.architecture.name().to_string(),
        },
        settings: ManifestSettings {
            level: args.opt.as_str().to_string(),
            target: args.target.as_str().to_string(),
            deterministic: args.deterministic,
            no_ai: args.no_ai,
            emit_ir: args.emit_ir,
            emit_cfg: args.emit_cfg,
            emit_report: args.emit_report,
            emit_annotations: args.emit_annotations,
            threads: args.threads,
        },
        ai_provider: args.ai_provider.clone(),
        plugins: args
            .plugin
            .as_ref()
            .map(|p| vec![p.display().to_string()])
            .unwrap_or_default(),
    }
}

/// Emit the listing, manifest, optional report / CFG / callgraph /
/// xrefs / annotation sidecars, and (for `--target c` at `-O1`+) the
/// lowered C translation unit.
///
/// - No `--output`: listing → stdout, then delimited blocks for
///   manifest, report, CFG, source, callgraph, xrefs, annotations.
/// - With `--output <path>`: listing → `<path>`, manifest →
///   `<path>.manifest.json`, report → `<path>.report.txt`, CFG →
///   `<path>.cfg.dot`, reconstructed source → `<path>.c`,
///   callgraph → `<path>.callgraph.dot`, xrefs → `<path>.xrefs.txt`,
///   annotations → `<path>.annot.json`.
#[allow(clippy::too_many_arguments)]
fn emit_outputs(
    output: Option<&Path>,
    listing: &str,
    manifest: &str,
    report: Option<&str>,
    cfg: Option<&str>,
    source: Option<&str>,
    source_suffix: &str,
    callgraph: Option<&str>,
    xrefs: Option<&str>,
    annotations: Option<&str>,
) -> dac_core::Result<()> {
    match output {
        None => {
            let stdout = io::stdout();
            let mut h = stdout.lock();
            h.write_all(listing.as_bytes())?;
            h.write_all(b"\n;; ---- manifest (NFR-10) ----\n")?;
            h.write_all(manifest.as_bytes())?;
            if let Some(r) = report {
                h.write_all(b"\n;; ---- analysis report (FR-25) ----\n")?;
                h.write_all(r.as_bytes())?;
            }
            if let Some(c) = cfg {
                h.write_all(b"\n;; ---- cfg (FR-28) ----\n")?;
                h.write_all(c.as_bytes())?;
            }
            if let Some(s) = source {
                h.write_all(b"\n;; ---- target source (FR-21) ----\n")?;
                h.write_all(s.as_bytes())?;
            }
            if let Some(g) = callgraph {
                h.write_all(b"\n;; ---- callgraph (FR-27) ----\n")?;
                h.write_all(g.as_bytes())?;
            }
            if let Some(x) = xrefs {
                h.write_all(b"\n;; ---- xrefs (FR-26, FR-31) ----\n")?;
                h.write_all(x.as_bytes())?;
            }
            if let Some(a) = annotations {
                h.write_all(b"\n;; ---- annotations (FR-19, FR-23, FR-25) ----\n")?;
                h.write_all(a.as_bytes())?;
            }
            Ok(())
        }
        Some(path) => {
            write_file(path, listing)?;
            let manifest_path = sidecar(path, ".manifest.json");
            write_file(&manifest_path, manifest)?;
            if let Some(r) = report {
                let report_path = sidecar(path, ".report.txt");
                write_file(&report_path, r)?;
            }
            if let Some(c) = cfg {
                let cfg_path = sidecar(path, ".cfg.dot");
                write_file(&cfg_path, c)?;
            }
            if let Some(s) = source {
                let source_path = sidecar(path, source_suffix);
                write_file(&source_path, s)?;
            }
            if let Some(g) = callgraph {
                let cg_path = sidecar(path, ".callgraph.dot");
                write_file(&cg_path, g)?;
            }
            if let Some(x) = xrefs {
                let xrefs_path = sidecar(path, ".xrefs.txt");
                write_file(&xrefs_path, x)?;
            }
            if let Some(a) = annotations {
                let annot_path = sidecar(path, ".annot.json");
                write_file(&annot_path, a)?;
            }
            Ok(())
        }
    }
}

fn sidecar(base: &Path, suffix: &str) -> PathBuf {
    let mut s = base.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

fn write_file(path: &Path, contents: &str) -> dac_core::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(contents.as_bytes())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum OptLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::O0 => "O0",
            Self::O1 => "O1",
            Self::O2 => "O2",
            Self::O3 => "O3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Format {
    #[default]
    Auto,
    Elf,
    Pe,
    MachO,
}

impl Format {
    fn parse(s: &str) -> std::result::Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "elf" => Ok(Self::Elf),
            "pe" => Ok(Self::Pe),
            "mach-o" => Ok(Self::MachO),
            other => Err(format!(
                "invalid --format value `{other}`; expected one of: elf, pe, mach-o, auto"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Elf => "elf",
            Self::Pe => "pe",
            Self::MachO => "mach-o",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Target {
    #[default]
    C,
    Cpp,
}

impl Target {
    fn parse(s: &str) -> std::result::Result<Self, String> {
        match s {
            "c" => Ok(Self::C),
            "cpp" => Ok(Self::Cpp),
            other => Err(format!(
                "invalid --target value `{other}`; expected one of: c, cpp"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }
}

#[derive(Debug, Default)]
struct Args {
    input: Option<PathBuf>,
    opt: OptLevel,
    arch: Option<String>,
    format: Format,
    target: Target,
    output: Option<PathBuf>,
    emit_ir: bool,
    emit_cfg: bool,
    emit_report: bool,
    emit_annotations: bool,
    emit_callgraph: bool,
    xrefs_subject: Option<String>,
    no_ai: bool,
    ai_provider: Option<String>,
    deterministic: bool,
    threads: Option<u32>,
    json: bool,
    debug: bool,
    plugin: Option<PathBuf>,
    hints: Option<PathBuf>,
    /// B3.30: collapse CRT-tagged function bodies to `extern <sig>
    /// name(...);` forward declarations so the rendered translation
    /// unit is dominated by user code (FR-21).
    hide_crt: bool,
    show_help: bool,
    show_version: bool,
}

fn parse_args<I>(iter: I) -> std::result::Result<Args, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = Args::default();
    let mut it = iter.into_iter();
    while let Some(arg) = it.next() {
        let raw = arg.to_string_lossy();
        match raw.as_ref() {
            "-O0" => args.opt = OptLevel::O0,
            "-O1" => args.opt = OptLevel::O1,
            "-O2" => args.opt = OptLevel::O2,
            "-O3" => args.opt = OptLevel::O3,
            "--arch" => args.arch = Some(take_value("--arch", &mut it)?),
            "--format" => args.format = Format::parse(&take_value("--format", &mut it)?)?,
            "--target" => args.target = Target::parse(&take_value("--target", &mut it)?)?,
            "--output" => args.output = Some(PathBuf::from(take_os_value("--output", &mut it)?)),
            "--emit-ir" => args.emit_ir = true,
            "--emit-cfg" => args.emit_cfg = true,
            "--emit-report" => args.emit_report = true,
            "--emit-annotations" => args.emit_annotations = true,
            "--callgraph" => args.emit_callgraph = true,
            "--xrefs" => args.xrefs_subject = Some(take_value("--xrefs", &mut it)?),
            "--no-ai" => args.no_ai = true,
            "--ai-provider" => args.ai_provider = Some(take_value("--ai-provider", &mut it)?),
            "--deterministic" => args.deterministic = true,
            "--threads" => {
                let raw = take_value("--threads", &mut it)?;
                let n: u32 = raw
                    .parse()
                    .map_err(|_| format!("invalid --threads value `{raw}`"))?;
                if n == 0 {
                    return Err("--threads must be >= 1".into());
                }
                args.threads = Some(n);
            }
            "--json" => args.json = true,
            "--debug" => args.debug = true,
            "--plugin" => args.plugin = Some(PathBuf::from(take_os_value("--plugin", &mut it)?)),
            "--hints" => args.hints = Some(PathBuf::from(take_os_value("--hints", &mut it)?)),
            "--hide-crt" => args.hide_crt = true,
            "--version" | "-V" => args.show_version = true,
            "--help" | "-h" => args.show_help = true,
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {other}"));
            }
            _ => {
                if args.input.is_some() {
                    return Err("multiple input paths not supported".into());
                }
                args.input = Some(PathBuf::from(arg));
            }
        }
    }
    Ok(args)
}

fn take_value<I>(flag: &str, rest: &mut I) -> std::result::Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    let next = rest
        .next()
        .ok_or_else(|| format!("{flag} requires a value"))?;
    Ok(next.to_string_lossy().into_owned())
}

fn take_os_value<I>(flag: &str, rest: &mut I) -> std::result::Result<OsString, String>
where
    I: Iterator<Item = OsString>,
{
    rest.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn print_usage_hint() {
    eprintln!("dac: try `dac --help` for usage.");
}

/// Map a [`HintError`] onto a [`dac_core::Error`] so `run_pipeline`'s
/// signature stays unchanged. The error message format matches the
/// rest of the CLI's user-facing diagnostics.
fn hints_error_to_core(err: HintError) -> dac_core::Error {
    let msg = err.message();
    tracing::error!(error = %msg, "failed to load hints file");
    dac_core::Error::Other(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_core::{Confidence, EvidenceGraph, EvidenceNode, IrLayer, Source};

    fn plt_stub_function(import: &str, address: u64) -> dac_recovery::Function {
        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        dac_recovery::Function {
            address,
            end: Some(address + 0x10),
            name: Some(import.to_string()),
            confidence: Confidence::new(1.0, Source::Observed),
            sources: dac_recovery::SourceMask::PLT,
            evidence: ev,
            kind: dac_recovery::FunctionKind::PltStub {
                import: import.to_string(),
            },
        }
    }

    /// A PLT stub whose import lives in the `dac-knowledge` catalogue
    /// surfaces the recovered signature: the return type and each
    /// positional parameter come from the catalogue entry, so the
    /// rendered declaration reads like the libc / Win32 header.
    #[test]
    fn b3_23_lower_plt_stub_extern_uses_dac_knowledge_signature() {
        let f = plt_stub_function("write", 0x1030);
        let e = lower_plt_stub_extern(&f, "write", false);
        assert_eq!(e.name, "write");
        // `write` in dac-knowledge returns `ssize_t` (int64_t in C),
        // and takes (int fd, void *buf, size_t n).
        assert!(matches!(
            e.return_type,
            CType::Int {
                width_bits: 64,
                signed: true
            }
        ));
        assert_eq!(e.params.len(), 3);
        assert_eq!(e.params[0].name, "fd");
        assert!(matches!(
            e.params[0].ty,
            CType::Int {
                width_bits: 32,
                signed: true
            }
        ));
        assert!(matches!(e.params[1].ty, CType::Ptr(_)));
        assert!(matches!(
            e.params[2].ty,
            CType::Int {
                width_bits: 64,
                signed: false
            }
        ));
        assert!(!e.is_variadic);
    }

    /// An import the catalogue doesn't know about degrades visibly
    /// (I-6): a single `int64_t name(void);` declaration and a
    /// leading-comment marker so a reader sees the fallback fired.
    #[test]
    fn b3_23_lower_plt_stub_extern_falls_back_for_unknown_imports() {
        let f = plt_stub_function("__nosuch_runtime_helper", 0x2000);
        let e = lower_plt_stub_extern(&f, "__nosuch_runtime_helper", false);
        assert_eq!(e.name, "__nosuch_runtime_helper");
        assert!(matches!(
            e.return_type,
            CType::Int {
                width_bits: 64,
                signed: true
            }
        ));
        assert!(e.params.is_empty());
        assert!(!e.is_variadic);
        let comment = e.leading_comment.expect("leading comment present");
        assert!(
            comment.contains("unknown — fell back to int64_t(void)"),
            "missing fallback marker: {comment:?}"
        );
    }

    /// Variadic catalog entries (`printf`) propagate `is_variadic`
    /// into the rendered declaration so the emitter writes
    /// `extern int32_t printf(... , ...);`.
    #[test]
    fn b3_23_lower_plt_stub_extern_propagates_variadic_flag() {
        let f = plt_stub_function("printf", 0x3000);
        let e = lower_plt_stub_extern(&f, "printf", false);
        assert!(e.is_variadic);
    }

    /// The `--debug` knob appends a `signal: PLT` row to the leading
    /// comment so a reviewer can see which discoverer signal pinned
    /// the binding without re-running with `--emit-annotations`.
    #[test]
    fn b3_23_lower_plt_stub_extern_debug_appends_signal_row() {
        let f = plt_stub_function("write", 0x1030);
        let e_default = lower_plt_stub_extern(&f, "write", false);
        let e_debug = lower_plt_stub_extern(&f, "write", true);
        assert!(!e_default
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: PLT"));
        assert!(e_debug
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: PLT"));
    }

    // ---- B3.25 forwarding-thunk lowering ---------------------------

    fn thunk_function(name: &str, address: u64, target: u64) -> dac_recovery::Function {
        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        dac_recovery::Function {
            address,
            end: Some(address + 0x09),
            name: Some(name.to_string()),
            confidence: Confidence::new(1.0, Source::Observed),
            sources: dac_recovery::SourceMask::SYMBOL,
            evidence: ev,
            kind: dac_recovery::FunctionKind::Thunk { target },
        }
    }

    /// A thunk whose target resolves through the name resolver
    /// lowers to a one-line clean direct call. The function
    /// signature collapses to `void(void)` and the leading comment
    /// carries the target's name + address so a reviewer sees the
    /// recognised pattern.
    #[test]
    fn b3_25_lower_thunk_function_uses_clean_direct_call() {
        let f = thunk_function("frame_dummy", 0x1150, 0x10c0);
        let mut resolver = CNameResolver::new();
        resolver.insert(0x10c0, "register_tm_clones".to_string());
        let cf = lower_thunk_function(&f, 0x10c0, &resolver, false);
        assert_eq!(cf.name, "frame_dummy");
        assert!(matches!(cf.return_type, CType::Void));
        assert!(cf.params.is_empty());
        assert_eq!(cf.body.stmts.len(), 1);
        match &cf.body.stmts[0] {
            CStmt::ExprStmt(CExpr::DirectCall { name, args }) => {
                assert_eq!(name, "register_tm_clones");
                assert!(args.is_empty());
            }
            other => panic!("expected DirectCall, got {other:?}"),
        }
        let comment = cf.leading_comment.expect("leading comment");
        assert!(comment.contains("forwarding thunk"));
        assert!(comment.contains("tail-call: register_tm_clones (0x10c0)"));
        // `signal: THUNK` is debug-only.
        assert!(!comment.contains("signal: THUNK"));
    }

    /// A thunk whose target is *not* in the resolver falls back to
    /// the [`CExpr::Call`] / [`CExpr::AddrLit`] cast pattern so the
    /// emitter still produces a compileable forwarding call. The
    /// leading comment still names the target VA so a reviewer can
    /// trace the binding.
    #[test]
    fn b3_25_lower_thunk_function_falls_back_to_addrlit_when_unresolved() {
        let f = thunk_function("opaque_thunk", 0x9000, 0x9abc);
        let resolver = CNameResolver::new();
        let cf = lower_thunk_function(&f, 0x9abc, &resolver, false);
        assert_eq!(cf.body.stmts.len(), 1);
        match &cf.body.stmts[0] {
            CStmt::ExprStmt(CExpr::Call { target, args }) => {
                assert!(args.is_empty());
                assert_eq!(**target, CExpr::AddrLit(0x9abc));
            }
            other => panic!("expected fallback Call(AddrLit), got {other:?}"),
        }
        let comment = cf.leading_comment.expect("leading comment");
        assert!(comment.contains("tail-call: 0x9abc"));
    }

    /// The `--debug` knob appends a `signal: THUNK` row so a reader
    /// of the per-function header sees which detector ran without
    /// digging into the annotation channel — same convention as the
    /// B3.23 PLT-stub leading comment.
    #[test]
    fn b3_25_lower_thunk_function_debug_appends_signal_row() {
        let f = thunk_function("atexit", 0x1460, 0x29c8);
        let mut resolver = CNameResolver::new();
        resolver.insert(0x29c8, "_crt_atexit".to_string());
        let default = lower_thunk_function(&f, 0x29c8, &resolver, false);
        let debug = lower_thunk_function(&f, 0x29c8, &resolver, true);
        assert!(!default
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: THUNK"));
        assert!(debug
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: THUNK"));
    }

    // ---- B3.30 CRT taxonomy + banner + --hide-crt ----------------

    fn user_function(name: &str, address: u64) -> dac_recovery::Function {
        let mut g = EvidenceGraph::new();
        let ev = g.add_node(EvidenceNode::IrNode {
            layer: IrLayer::Cfg,
            id: 0,
        });
        dac_recovery::Function {
            address,
            end: Some(address + 0x40),
            name: Some(name.to_string()),
            confidence: Confidence::new(1.0, Source::Observed),
            sources: dac_recovery::SourceMask::SYMBOL,
            evidence: ev,
            kind: dac_recovery::FunctionKind::User,
        }
    }

    /// A glibc CRT helper gets the runtime-specific banner line. The
    /// label tracks [`dac_knowledge::CrtRuntime::label`] so a PE-side
    /// fixture rendered the same way reads `mingw-w64 startup`.
    #[test]
    fn b3_30_crt_banner_line_uses_runtime_label_for_glibc_helper() {
        let f = user_function("_init", 0x1000);
        let line = crt_banner_line(&f).expect("CRT banner present");
        assert_eq!(line, "runtime support (glibc startup) — not user code");
    }

    /// A MinGW CRT helper gets the matching banner. The runtime label
    /// is the user-facing differentiator between the two glibc /
    /// MinGW startup families.
    #[test]
    fn b3_30_crt_banner_line_uses_runtime_label_for_mingw_helper() {
        let f = user_function("__tmainCRTStartup", 0x140001000);
        let line = crt_banner_line(&f).expect("CRT banner present");
        assert_eq!(line, "runtime support (mingw-w64 startup) — not user code");
    }

    /// A user-code function has no CRT banner; the helper returns
    /// `None` so call sites can prepend unconditionally without
    /// extra branching.
    #[test]
    fn b3_30_crt_banner_line_returns_none_for_user_function() {
        let f = user_function("my_business_logic", 0x4000);
        assert!(crt_banner_line(&f).is_none());
    }

    /// The prepend helper threads the banner ahead of every other
    /// line so the runtime-support marker is the first thing a
    /// reviewer sees when scrolling the source.
    #[test]
    fn b3_30_prepend_crt_banner_inserts_banner_at_top_of_existing_comment() {
        let f = user_function("_start", 0x1060);
        let prepended =
            prepend_crt_banner(Some("dac-recovered function\naddress: 0x1060".into()), &f)
                .expect("non-None");
        let mut lines = prepended.lines();
        assert_eq!(
            lines.next(),
            Some("runtime support (glibc startup) — not user code")
        );
        assert_eq!(lines.next(), Some("dac-recovered function"));
        assert_eq!(lines.next(), Some("address: 0x1060"));
    }

    /// A user-code prepend is a no-op: the helper returns the
    /// original comment so an unrelated batch tweaking the leading
    /// block does not have to special-case the CRT path.
    #[test]
    fn b3_30_prepend_crt_banner_is_noop_for_user_function() {
        let f = user_function("main", 0x1040);
        let original = Some("dac-recovered function\naddress: 0x1040".to_string());
        let result = prepend_crt_banner(original.clone(), &f);
        assert_eq!(result, original);
    }

    /// `hidden_crt_extern_decl` collapses a thunk-shaped CRT body
    /// (`frame_dummy` in the hello-x86_64 fixture) into a single
    /// `extern <ret> name(<params>);` forward declaration with the
    /// CRT banner prepended to its leading comment.
    #[test]
    fn b3_30_hidden_crt_extern_decl_collapses_thunk_helper_to_extern() {
        let f = thunk_function("frame_dummy", 0x1150, 0x10c0);
        let e = hidden_crt_extern_decl(&f, None, false);
        assert_eq!(e.name, "frame_dummy");
        // No recovered facts → default `int64_t name(void);` shape.
        assert!(matches!(
            e.return_type,
            CType::Int {
                width_bits: 64,
                signed: true
            }
        ));
        assert!(e.params.is_empty());
        assert!(!e.is_variadic);
        let comment = e.leading_comment.expect("leading comment present");
        assert!(
            comment.contains("runtime support (glibc startup) — not user code"),
            "expected CRT banner in:\n{comment}"
        );
        assert!(
            comment.contains("body hidden by --hide-crt"),
            "expected hide-crt marker in:\n{comment}"
        );
        assert!(comment.contains("address: 0x1150"));
        // The catalogue role line gives a reader the *why* behind
        // the elision without re-running with `--debug`.
        assert!(
            comment.contains("role: "),
            "expected role line in:\n{comment}"
        );
    }

    /// MinGW startup helpers collapse with the matching banner label.
    /// `__tmainCRTStartup` is the shared CRT entry on PE binaries and
    /// must read "mingw-w64 startup".
    #[test]
    fn b3_30_hidden_crt_extern_decl_uses_mingw_label_for_pe_helper() {
        let f = user_function("__tmainCRTStartup", 0x140001000);
        let e = hidden_crt_extern_decl(&f, None, false);
        let comment = e.leading_comment.expect("leading comment present");
        assert!(comment.contains("runtime support (mingw-w64 startup) — not user code"));
    }

    /// `--debug` appends the `signal: CRT` row to the elided helper's
    /// leading comment so a reviewer running with both `--hide-crt`
    /// and `--debug` still sees which detector pinned the elision.
    #[test]
    fn b3_30_hidden_crt_extern_decl_debug_appends_signal_row() {
        let f = user_function("_init", 0x1000);
        let default = hidden_crt_extern_decl(&f, None, false);
        let debug = hidden_crt_extern_decl(&f, None, true);
        assert!(!default
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: CRT"));
        assert!(debug
            .leading_comment
            .as_deref()
            .unwrap()
            .contains("signal: CRT"));
    }
}
