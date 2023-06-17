use ansi_term::Colour::{Black, Red};
use clap::Parser;
use cli::Cli;
use error::MainError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
mod cli;
mod commands;
mod error;
mod workspace;

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct NodeManifest {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.run() {
        let status_code = match err {
            MainError::Sub(status) => {
                eprintln!(
                    "{} {}",
                    Black.on(Red).paint("ELIFECYCLE"),
                    Red.paint(format!("Command failed with exit code {}", status.get()))
                );
                status.get()
            }
            MainError::Pn(error) => {
                eprintln!(
                    "{} {}",
                    Black.on(Red).paint("\u{2009}ERROR\u{2009}"),
                    Red.paint(error.to_string())
                );
                1
            }
        };
        std::process::exit(status_code);
    }
    // match run() {
    //     Ok(()) => {}
    //     Err(MainError::Sub(status)) => exit(status.get()),
    //     Err(MainError::Pn(error)) => {
    //         eprintln!(
    //             "{prefix} {error}",
    //             prefix = Black.on(Red).paint("\u{2009}ERROR\u{2009}"),
    //             error = Red.paint(error.to_string()),
    //         );
    //         exit(1);
    //     }
    // }
}
