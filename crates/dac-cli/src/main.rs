//! `dac` — command-line frontend.
//!
//! Status: B0.5 declares the full CLI surface from spec §10.1. Every flag
//! is parsed and validated today; most do not yet drive behavior and become
//! active milestone by milestone (`--target` / `--emit-*` in M1/M2,
//! `--ai-provider` / `--no-ai` in M4, `--plugin` in M5). `--deterministic`
//! gates `NonDeterministic` passes through the pass manager once the CLI
//! drives a real pipeline (B1.6); the manager-level enforcement is already
//! covered by `dac-core` unit tests.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dac_binfmt::{load_from_bytes, BinaryModel};
use dac_core::init_tracing;

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

    match run(&input) {
        Ok(model) => {
            tracing::info!(
                format = %model.format.name(),
                size = model.size,
                deterministic = args.deterministic,
                path = %input.display(),
                "loaded input"
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                path = %input.display(),
                "failed to load input"
            );
            ExitCode::FAILURE
        }
    }
}

fn run(path: &Path) -> dac_core::Result<BinaryModel> {
    let bytes = std::fs::read(path)?;
    load_from_bytes(&bytes)
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
