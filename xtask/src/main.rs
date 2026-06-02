//! Developer tasks for the dac workspace.
//!
//! Run with `cargo xtask <subcommand>`. The alias is defined in
//! `.cargo/config.toml`.

#![forbid(unsafe_code)]

mod golden;

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let task = args.next();
    let result = match task.as_deref() {
        Some("ci") => ci(),
        Some("fmt") => fmt(),
        Some("clippy") => clippy(),
        Some("test") => test(),
        Some("golden") => match args.next().as_deref() {
            Some("check") | None => golden::run(golden::Mode::Check),
            Some("update") => golden::run(golden::Mode::Update),
            Some(other) => {
                eprintln!("xtask: unknown `golden` subcommand: {other}");
                eprintln!("xtask: try `golden check` or `golden update`");
                return ExitCode::from(2);
            }
        },
        Some("help" | "--help" | "-h") | None => {
            usage();
            return ExitCode::SUCCESS;
        }
        Some(unknown) => {
            eprintln!("xtask: unknown subcommand: {unknown}");
            usage();
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask: {err}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!(
        "\
Usage: cargo xtask <subcommand>

Subcommands:
  ci              Canonical CI check: fmt + clippy + test + golden check
  fmt             cargo fmt --all --check
  clippy          cargo clippy --workspace --all-targets -- -D warnings
  test            cargo test --workspace
  golden check    Run the corpus through dac and diff against tests/golden/
  golden update   Regenerate tests/golden/ from the current corpus output
  help            Print this message
"
    );
}

pub(crate) fn cargo() -> Command {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    Command::new(cargo)
}

pub(crate) fn run(label: &str, mut cmd: Command) -> Result<(), String> {
    eprintln!("xtask: running {label}");
    let status = cmd
        .status()
        .map_err(|e| format!("failed to spawn `{label}`: {e}"))?;
    if !status.success() {
        return Err(format!("`{label}` failed with {status}"));
    }
    Ok(())
}

fn fmt() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["fmt", "--all", "--check"]);
    run("cargo fmt --all --check", cmd)
}

fn clippy() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    run("cargo clippy --workspace --all-targets", cmd)
}

fn test() -> Result<(), String> {
    let mut cmd = cargo();
    cmd.args(["test", "--workspace"]);
    run("cargo test --workspace", cmd)
}

fn ci() -> Result<(), String> {
    fmt()?;
    clippy()?;
    test()?;
    golden::run(golden::Mode::Check)?;
    Ok(())
}
