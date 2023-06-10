use ansi_term::Color::{Black, Red};
use clap::*;
use cli::{Cli, ExecCommand, PassedThroughArgs, PassedThroughCommand};
use error::{MainError, PnError};
use pipe_trait::Pipe;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs::File,
    num::NonZeroI32,
    path::Path,
    process::{exit, Command, Stdio},
};

mod cli;
mod error;
mod workspace;

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
        Err(MainError::Sub(status)) => exit(status.get()),
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
    let Cli {
        workspace_root,
        passed_through,
        exec,
    } = Cli::parse();
    match (passed_through, exec) {
        (None, None) => {
            panic!("Must specify a command") // TODO: define a proper error message.
        }
        (Some(_), Some(_)) => {
            unreachable!("Clap shouldn't allow this")
        }
        (None, Some(ExecCommand::Run(args))) => {
            let mut cwd = env::current_dir().expect("Couldn't find the current working directory");
            if workspace_root {
                cwd = workspace::find_workspace_root(&cwd)?;
            }
            let manifest_path = cwd.join("package.json");
            let manifest = manifest_path
                .pipe(File::open)
                .map_err(MainError::from_dyn)?
                .pipe(serde_json::de::from_reader::<_, NodeManifest>)
                .map_err(MainError::from_dyn)?;
            if let Some(command) = manifest.scripts.get(&args.script) {
                eprintln!("> {:?}", command);
                run_script(args.script, command.clone(), &cwd)
            } else {
                PnError::MissingScript { name: args.script }
                    .pipe(MainError::Pn)
                    .pipe(Err)
            }
        }
        (None, Some(ExecCommand::Other(args))) => pass_to_sub((&*args.join(" ")).into()),
        (Some(PassedThroughArgs { cmd, mut args }), None) => {
            let cmd = match cmd {
                PassedThroughCommand::Install => "install".to_string(),
                PassedThroughCommand::Update => "update".to_string(),
            };
            args.insert(0, cmd);
            pass_to_pnpm(&args)
        }
    }
}

fn run_script(name: String, command: String, cwd: &Path) -> Result<(), MainError> {
    let mut path_env = OsString::from("node_modules/.bin");
    if let Some(path) = env::var_os("PATH") {
        path_env.push(":");
        path_env.push(path);
    }
    let status = Command::new("sh")
        .current_dir(cwd)
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
        Some(Some(status)) => MainError::Sub(status),
        None => MainError::Pn(PnError::UnexpectedTermination {
            command: format!("pnpm {}", args.join(" ")),
        }),
    })
}

fn pass_to_sub(command: String) -> Result<(), MainError> {
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
    Err(match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => MainError::Sub(status),
        None => MainError::Pn(PnError::UnexpectedTermination { command }),
    })
}
