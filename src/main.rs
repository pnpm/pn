use ansi_term::Colour::{Black, Red};
use clap::Parser;
use cli::Cli;
use error::MainError;
mod cli;
mod commands;
mod error;
mod workspace;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.run() {
        dbg!(&err);
        
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
}
