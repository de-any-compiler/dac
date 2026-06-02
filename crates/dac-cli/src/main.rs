//! `dac` — command-line frontend.
//!
//! Status: B0.5 declared the full CLI surface from spec §10.1. B1.6 wires
//! `-O0` end-to-end: the binary is loaded through `dac-binfmt`, functions
//! are discovered through `dac-recovery`, and a deterministic annotated
//! listing (plus a reproducibility manifest per NFR-10 and an optional
//! analysis report per FR-25) is emitted on the user-selected output
//! path. `--target`, `--emit-ir`, `--emit-cfg`, `--emit-annotations`,
//! `--ai-provider`, `--no-ai`, and `--plugin` are still parsed but
//! become live milestone by milestone (M2 / M4 / M5). `--deterministic`
//! is surfaced on the manifest today; manager-level enforcement remains
//! covered by `dac-core` unit tests.

#![forbid(unsafe_code)]

mod listing;
mod manifest;
mod report;

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dac_analysis::cfg::{build_cfgs, render_dot_all};
use dac_arch::{Architecture as _, InstructionDecoder, InstructionLifter};
use dac_arch_x86::X86_64;
use dac_binfmt::{load_from_bytes, Architecture, BinaryModel};
use dac_core::{init_tracing, EvidenceGraph};
use dac_recovery::discover_functions;

use crate::listing::{render_listing, ListingOptions};
use crate::manifest::{
    render_manifest_json, Manifest, ManifestInput, ManifestSettings, ManifestTool,
};
use crate::report::{render_report_text, Report};

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
        plugin = ?args.plugin,
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

    // The pipeline picks a backend lazily so that formats without an
    // arch backend (everything but x86-64 today) still produce the
    // manifest + a minimal listing rather than failing.
    let backend = pick_backend(&model);
    let listing_text;
    let report_text;
    let cfg_text;
    match &backend {
        Some(b) => {
            let mut graph = EvidenceGraph::new();
            let functions = discover_functions(&model, &bytes, b.decoder.as_ref(), &mut graph);
            listing_text = render_listing(
                &input_label,
                &model,
                &bytes,
                b.decoder.as_ref(),
                b.lifter.as_ref(),
                &functions,
                &ListingOptions::default(),
            );
            if args.emit_report {
                let r = Report::build(
                    &model,
                    &bytes,
                    b.decoder.as_ref(),
                    b.lifter.as_ref(),
                    &functions,
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
            let _ = functions; // evidence handles already wired
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
        }
    }

    emit_outputs(
        args.output.as_deref(),
        &listing_text,
        &manifest_json,
        report_text.as_deref(),
        cfg_text.as_deref(),
    )
}

struct Backend {
    decoder: Box<dyn InstructionDecoder>,
    lifter: Box<dyn InstructionLifter>,
}

fn pick_backend(model: &BinaryModel) -> Option<Backend> {
    match model.architecture {
        Architecture::X86_64 => Some(Backend {
            decoder: X86_64.decoder(),
            lifter: X86_64.lifter(),
        }),
        _ => None,
    }
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

/// Emit the listing, manifest, and optional report / CFG.
///
/// - No `--output`: listing → stdout, manifest → stdout (delimited),
///   report (if any) → stdout (delimited), CFG (if any) → stdout
///   (delimited).
/// - With `--output <path>`: listing → `<path>`, manifest →
///   `<path>.manifest.json`, report → `<path>.report.txt`, CFG →
///   `<path>.cfg.dot`.
fn emit_outputs(
    output: Option<&Path>,
    listing: &str,
    manifest: &str,
    report: Option<&str>,
    cfg: Option<&str>,
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
    no_ai: bool,
    ai_provider: Option<String>,
    deterministic: bool,
    threads: Option<u32>,
    json: bool,
    debug: bool,
    plugin: Option<PathBuf>,
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
