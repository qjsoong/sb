#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! `sb`, a minimal static blog generator built on `mdk`.

mod build;
mod config;
mod post;
mod template;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

/// A minimal static blog generator built on mdk.
#[derive(Debug, Parser)]
#[command(name = "sb", version, about, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate the site into `public/`
    Build(BuildArgs),
}

#[derive(Debug, Args)]
struct BuildArgs {
    /// Site directory, defaults to the current directory
    #[arg(value_name = "DIR", default_value = ".")]
    directory: PathBuf,

    /// Include posts marked `draft: true`
    #[arg(long)]
    drafts: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match &cli.command {
        Command::Build(args) => run_build(&args.directory, args.drafts),
    }
}

fn run_build(directory: &Path, drafts: bool) -> ExitCode {
    match build::build(directory, &build::Options { drafts }) {
        Ok(report) => {
            eprintln!("sb: {}", report.summary());
            ExitCode::SUCCESS
        }
        Err(build::Error::Posts(messages)) => {
            for message in &messages {
                eprintln!("error: {message}");
            }
            ExitCode::from(1)
        }
        Err(build::Error::Fatal(error)) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}
