use crate::error::PnError;
use crate::{commands, commands::CommandTrait, error::MainError};
use clap::{Args, Parser, Subcommand};
use itertools::Itertools;
use pipe_trait::Pipe;
use std::{env, ffi::OsString, num::NonZeroI32, path::Path, process, process::Stdio};

#[derive(Debug, Parser)]
#[clap(author, version, about, rename_all = "kebab-case")]
pub struct Cli {
    #[clap(flatten)]
    pub config: Config,

    /// Command to execute.
    #[clap(subcommand)]
    pub command: Command,
}

/// Global configurations to be passed down to subcommands
#[derive(Parser, Debug)]
pub struct Config {
    /// Run the command on the root workspace project
    #[clap(short, long)]
    pub workspace_root: bool,
}

#[derive(Debug, Subcommand)]
#[clap(rename_all = "kebab-case")]
pub enum Command {
    /// Runs a defined package script.
    #[clap(alias = "run-script")]
    Run(commands::run::Run),

    /// Installs all dependencies of the project in the current working directory.
    /// When executed inside a workspace, installs all dependencies of all projects.
    #[clap(alias = "i")]
    Install(commands::PassedThroughArgs),

    /// Updates packages to their latest version based on the specified range.
    /// You can use "*" in package name to update all packages with the same pattern.
    #[clap(alias = "up")]
    Update(commands::PassedThroughArgs),

    /// Execute a shell command in scope of a project.
    #[clap(external_subcommand)]
    Other(Vec<String>),
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

impl Cli {
    pub fn run(self) -> Result<(), MainError> {
        match self.command {
            Command::Run(e) => e.run(self.config),
            Command::Install(args) => handle_passed_through("install", args),
            Command::Update(args) => handle_passed_through("update", args),
            Command::Other(args) => pass_to_sub(args.join(" ")),
        }
    }
}

fn handle_passed_through(
    command: &str,
    args: commands::PassedThroughArgs,
) -> Result<(), MainError> {
    let commands::PassedThroughArgs { mut args } = args;
    args.insert(0, command.into());
    pass_to_pnpm(&args)
}

fn pass_to_pnpm(args: &[OsString]) -> Result<(), MainError> {
    let status = process::Command::new("pnpm")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(MainError::from_dyn)?
        .wait()
        .map_err(MainError::from_dyn)?
        .code()
        .map(NonZeroI32::new);
    Err(match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => MainError::Sub(status),
        None => MainError::Pn(PnError::UnexpectedTermination {
            command: format!(
                "pnpm {}",
                args.iter().map(|x| x.to_string_lossy()).join(" "),
            ),
        }),
    })
}

fn pass_to_sub(command: String) -> Result<(), MainError> {
    let path_env = create_path_env()?;
    let status = process::Command::new("sh")
        .env("PATH", path_env)
        .arg("-c")
        .arg(&command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(MainError::from_dyn)?
        .wait()
        .map_err(MainError::from_dyn)?
        .code()
        .map(NonZeroI32::new);
    Err(match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => MainError::Sub(status),
        None => MainError::Pn(PnError::UnexpectedTermination { command }),
    })
}

pub fn create_path_env() -> Result<OsString, MainError> {
    let existing_paths = env::var_os("PATH");
    let existing_paths = existing_paths.iter().flat_map(env::split_paths);
    Path::new("node_modules")
        .join(".bin")
        .pipe(std::iter::once)
        .chain(existing_paths)
        .pipe(env::join_paths)
        .map_err(|error| MainError::Pn(PnError::NodeBinPathError { error }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_path_env() {
        let bin_path = Path::new("node_modules").join(".bin");
        let path_env = create_path_env().expect("prepend 'node_modules/.bin' to PATH");

        let first_path = env::split_paths(&path_env).next();
        assert_eq!(first_path, Some(bin_path));
    }
}
