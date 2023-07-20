use clap::*;
use std::ffi::OsString;

use crate::commands::PnpmCommands;

#[derive(Debug, Parser)]
#[clap(author, version, about, rename_all = "kebab-case")]
pub struct Cli {
    /// Run the command on the root workspace project.
    #[clap(short, long)]
    pub workspace_root: bool,
    /// Command to execute.
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[clap(rename_all = "kebab-case")]
pub enum Command {
    /// Runs a defined package script.
    #[clap(alias = "run-script")]
    Run(RunArgs),
    #[clap(flatten)]
    PnpmCommands(PnpmCommands),
    /// Execute a shell command in scope of a project.
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct PassedThroughArgs {
    pub args: Vec<OsString>,
}

// Generate passed thro args from `env::args_os()`. `PnpmCommand` can be formatted to string
// to get the command name, so all we need is the remaining args. This is better than maintaining
// a `PassedThroughArgs` struct for all variants of `PnpmCommand` and also wouldnt bloat the
// size of the enum
impl std::default::Default for PassedThroughArgs {
    fn default() -> Self {
        let mut os_args = std::env::args_os();
        os_args.next(); // skip the bin name
        os_args.next(); // skip the command name
        Self {
            args: os_args.collect(),
        }
    }
}

/// Runs a defined package script.
#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct RunArgs {
    /// Name of the package script to run.
    pub script: Option<String>, // Not OsString because it would be compared against package.json#scripts

    /// Arguments to pass to the package script.
    pub args: Vec<OsString>,
}
