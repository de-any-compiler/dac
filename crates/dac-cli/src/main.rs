//! `dac` — command-line frontend.
//!
//! Status: B0.2 wired the binary to `dac-binfmt::load_from_bytes` via
//! `dac-core`'s `Error` and tracing. B0.4 adds `--deterministic` (NFR-9);
//! the flag is accepted and surfaced through tracing today, and gates
//! `NonDeterministic` passes through the pass manager once the CLI drives
//! real pipelines (B1.6). Full CLI surface (`-O`, `--target`, `--emit-*`,
//! plugins, …) lands with B0.5.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dac_binfmt::{load_from_bytes, BinaryModel};
use dac_core::init_tracing;

fn main() -> ExitCode {
    let args = match parse_args(std::env::args_os().skip(1)) {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("dac: {msg}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    if args.show_help {
        print_usage();
        return ExitCode::SUCCESS;
    }

    init_tracing(args.json);

    let Some(input) = args.input else {
        eprintln!("dac: missing input binary path");
        print_usage();
        return ExitCode::from(2);
    };

    if args.deterministic {
        tracing::info!("deterministic mode requested");
    }

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

#[derive(Debug, Default)]
struct Args {
    input: Option<PathBuf>,
    json: bool,
    deterministic: bool,
    show_help: bool,
}

fn parse_args<I>(iter: I) -> std::result::Result<Args, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = Args::default();
    for arg in iter {
        let s = arg.to_string_lossy();
        match s.as_ref() {
            "--json" => args.json = true,
            "--deterministic" => args.deterministic = true,
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

fn print_usage() {
    eprintln!(
        "\
Usage: dac <input> [--json] [--deterministic] [--help]

  <input>           Path to the binary to analyze.
  --json            Emit machine-readable JSON diagnostics.
  --deterministic   Reject any pipeline that registers a non-deterministic
                    pass (NFR-9).
  --help            Print this message.
"
    );
}
