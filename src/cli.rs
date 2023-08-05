use clap::*;

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
    /// Execute a shell command in scope of a project.
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

/// Runs a defined package script.
#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct RunArgs {
    /// Name of the package script to run.
    pub script: Option<String>, // Not OsString because it would be compared against package.json#scripts

    /// Arguments to pass to the package script.
    pub args: Vec<String>,
}
