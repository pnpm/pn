use crate::{cli::Config, error::MainError};
use clap::Args;
use std::ffi::OsString;

pub mod run;

/// Arguments to pass to pnpm
#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct PassedThroughArgs {
    pub args: Vec<OsString>,
}

/// Trait for all subcommands to be implemented
pub trait CommandTrait: Sized {
    fn run(self, config: Config) -> Result<(), MainError>;
}
