use clap::Args;
use std::ffi::OsString;

pub mod run;

/// Arguments to pass to pnpm
#[derive(Debug, Args)]
#[clap(rename_all = "kebab-case")]
pub struct PassedThroughArgs {
    pub args: Vec<OsString>,
}
