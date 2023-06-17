use clap::*;
use std::ffi::OsString;

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
    /// Installs all dependencies of the project in the current working directory.
    /// When executed inside a workspace, installs all dependencies of all projects.
    #[clap(alias = "i")]
    Install(PassedThroughArgs),
    /// Updates packages to their latest version based on the specified range.
    /// You can use "*" in package name to update all packages with the same pattern.
    #[clap(alias = "up")]
    Update(PassedThroughArgs),
    /// Execute a shell command in scope of a project.
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct PassedThroughArgs {
    pub args: Vec<OsString>,
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
