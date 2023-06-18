use clap::Parser;
use cli::Cli;
use error::MainError;
use yansi::Paint;
mod cli;
mod commands;
mod error;
mod workspace;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.run() {
        let status_code = match err {
            MainError::Sub(status) => status.get(),
            MainError::Pn(error) => {
                eprintln!(
                    "{} {}",
                    Paint::black("\u{2009}ERROR\u{2009}").bg(yansi::Color::Red),
                    Paint::red(error.to_string())
                );
                1
            }
        };
        std::process::exit(status_code);
    }
}
