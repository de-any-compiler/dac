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
    Block as CBlock, CType, Function as CFunction, Item as CItem, Stmt as CStmt, TranslationUnit,
};
use dac_backend_c::{
    default_includes as c_default_includes, emit as c_emit, lower_function as c_lower_function,
    NameResolver as CNameResolver, Recovered as CRecovered,
};
use dac_backend_cpp::{
    class_recovery::recover_classes as recover_cpp_classes,
    default_includes as cpp_default_includes, emit as cpp_emit, lower_unit as cpp_lower_unit,
};
use dac_binfmt::{load_from_bytes, Architecture, BinaryModel};
use dac_core::{init_tracing, EvidenceGraph};
use dac_hints::{HintError, Hints};
use dac_recovery::{discover_functions, FunctionSet};

use crate::annotations::{
    render_annotations_json, render_function_debug_block, AnnotationDoc, FunctionAnnotation,
    InputStamp, SettingsStamp, ToolStamp,
};
use crate::lift::{lift_all, register_hints, LiftOutcome, LiftStats};
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
            let functions = discover_functions(&model, &bytes, b.decoder.as_ref(), &mut graph);
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
            annotations_doc = build_annotations_doc(&input_label, &model, args, &functions, &graph);
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
            annotations_doc =
                build_annotations_doc(&input_label, &model, args, &empty_set, &empty_graph);
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
    )
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
        )),
        Target::Cpp => Some(render_cpp_unit(
            input_label,
            model,
            functions,
            graph,
            args.debug,
        )),
    }
}

fn render_c_unit(
    input_label: &str,
    model: &BinaryModel,
    functions: &FunctionSet,
    lift_outcomes: Option<&[LiftOutcome]>,
    annotations: &AnnotationDoc,
    debug: bool,
) -> String {
    let resolver = build_c_name_resolver(functions);
    // The orchestrator returns outcomes in the same order as
    // `functions.functions`; the zip below guarantees the i-th entry
    // pairs with the i-th function. When the orchestrator was skipped
    // (no callers requested it), every function degrades to a stub
    // with the same reason — this keeps the matching call sites
    // simple while preserving the I-6 visible-degradation contract.
    let items: Vec<CItem> = functions
        .functions
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let outcome = lift_outcomes.and_then(|os| os.get(i));
            let annot = annotations
                .functions
                .iter()
                .find(|a| a.address == f.address);
            CItem::Function(lower_one_c_function(f, outcome, annot, &resolver, debug))
        })
        .collect();
    let mut includes = c_default_includes();
    includes.insert(
        0,
        format!(
            "/* dac --target c -O1 reconstruction\n   input: {input_label}\n   arch:  {} */",
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
fn lower_one_c_function(
    f: &dac_recovery::Function,
    outcome: Option<&LiftOutcome>,
    annot: Option<&FunctionAnnotation>,
    resolver: &CNameResolver,
    debug: bool,
) -> CFunction {
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
            let recovered = CRecovered::new(
                facts.convention.as_ref().map(|c| &c.signature),
                Some(&facts.types),
                Some(&facts.structs),
                Some(&facts.names),
            );
            let mut lowered = c_lower_function(ssa, sem, resolver, &recovered);
            // `lower_function` derives the name from `sem.function_name`
            // (the recovered symbol); sanitise to keep emitted C
            // syntactically valid for symbols that contain `@`, `.`
            // and friends.
            lowered.name = sanitized;
            // Replace the structurer's auto-generated leading comment
            // (a duplicate of the address line plus the structurer
            // stats) with a single unified head that carries the
            // recovered-function provenance + structuring stats and the
            // recovered convention citation (B3.10). The `--debug`
            // block from the annotation channel is appended when
            // requested.
            lowered.leading_comment = Some(real_body_leading_comment(f, sem, facts, annot, debug));
            lowered
        }
        outcome => {
            let reason = match outcome {
                Some(LiftOutcome::Stub { reason }) => reason.clone(),
                _ => "orchestrator did not run for this function".into(),
            };
            CFunction {
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
            }
        }
    }
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
            "/* dac --target cpp -O1 reconstruction\n   input: {input_label}\n   arch:  {}\n   classes: {} (polymorphic: {}) members: {} free: {} */",
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
