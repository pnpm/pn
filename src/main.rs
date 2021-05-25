use ansi_term::Color::{Black, Red};
use derive_more::Display;
use pipe_trait::Pipe;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    error::Error,
    ffi::OsString,
    fs::File,
    num::NonZeroI32,
    process::{exit, Command, Stdio},
};

/// Error types emitted by `pn` itself.
#[derive(Debug, Display)]
enum PnError {
    /// Script not found when running `pn run`.
    #[display(fmt = "Missing script {:?}", name)]
    MissingScript { name: String },
    /// Script ran by `pn run` exits with non-zero status code.
    #[display(fmt = "Script {:?} exits with non-zero status code {}", name, status)]
    ScriptError { name: String, status: NonZeroI32 },
    /// Subprocess finishes but without a status code.
    #[display(fmt = "Command {:?} has ended unexpectedly", command)]
    UnexpectedTermination { command: String },
    /// Other errors.
    #[display(fmt = "{}", error)]
    Other { error: Box<dyn Error> },
}

/// The main error type.
#[derive(Debug)]
enum MainError {
    /// Errors emitted by `pn` itself.
    Pn(PnError),
    /// The `pnpm` subprocess exits with non-zero status code.
    Pnpm(NonZeroI32),
}

impl MainError {
    fn from_dyn(error: impl Error + 'static) -> Self {
        MainError::Pn(PnError::Other {
            error: Box::new(error),
        })
    }
}

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct NodeManifest {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(MainError::Pnpm(status)) => exit(status.get()),
        Err(MainError::Pn(error)) => {
            eprintln!(
                "{prefix} {error}",
                prefix = Black.on(Red).paint("\u{2009}ERROR\u{2009}"),
                error = Red.paint(error.to_string()),
            );
            exit(1);
        }
    }
}

fn run() -> Result<(), MainError> {
    let args: Vec<String> = std::env::args().collect();
    match &*args[1] {
        "run" | "run-script" => {
            let manifest = "package.json"
                .pipe(File::open)
                .map_err(MainError::from_dyn)?
                .pipe(serde_json::de::from_reader::<_, NodeManifest>)
                .map_err(MainError::from_dyn)?;
            let name = args[2].clone();
            if let Some(command) = manifest.scripts.get(&name) {
                eprintln!("> {:?}", command);
                run_script(name, command.clone())?;
                Ok(())
            } else {
                PnError::MissingScript { name }
                    .pipe(MainError::Pn)
                    .pipe(Err)
            }
        }
        "install" | "i" | "update" | "up" => {
            pass_to_pnpm(&args[1..])?;
            Ok(())
        }
        _ => {
            // run_script(&args[1..].join(" "))?;
            // Ok(())
            panic!("What does this part mean, @zkochan? Why doesn't pn just pass this to pnpm to handle?")
        }
    }
}

fn run_script(name: String, command: String) -> Result<(), MainError> {
    let mut path_env = OsString::from("node_modules/.bin");
    if let Some(path) = env::var_os("PATH") {
        path_env.push(":");
        path_env.push(path);
    }
    let status = Command::new("sh")
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
    match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => PnError::ScriptError { name, status },
        None => PnError::UnexpectedTermination { command },
    }
    .pipe(MainError::Pn)
    .pipe(Err)
}

fn pass_to_pnpm(args: &[String]) -> Result<(), MainError> {
    let status = Command::new("pnpm")
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
        Some(Some(status)) => MainError::Pnpm(status),
        None => MainError::Pn(PnError::UnexpectedTermination {
            command: format!("pnpm {}", args.join(" ")),
        }),
    })
}
