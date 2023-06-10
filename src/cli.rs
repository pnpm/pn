use clap::*;
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[clap(author, version, about, rename_all = "kebab-case")]
pub struct Cli {
    /// Run the command on the root workspace project.
    #[clap(short, long)]
    pub workspace_root: bool,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
#[clap(rename_all = "kebab-case")]
pub enum Command {
    #[clap(alias = "run-script")]
    Run(RunArgs),
    #[clap(alias = "i")]
    Install(PassedThroughArgs),
    #[clap(alias = "up")]
    Update(PassedThroughArgs),
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct PassedThroughArgs {
    pub args: Vec<String>,
}

/// Runs a defined package script.
#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct RunArgs {
    /// Name of the package script to run.
    pub script: String, // Not OsString because it would be compared against package.json#scripts

    /// Arguments to pass to the package script.
    pub args: Vec<OsString>,
}
