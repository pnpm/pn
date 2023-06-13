use ansi_term::Color::{Black, Red};
use clap::Parser;
use cli::{Cli, PassedThroughArgs};
use error::{MainError, PnError};
use itertools::Itertools;
use pipe_trait::Pipe;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs::File,
    io::ErrorKind,
    num::NonZeroI32,
    path::Path,
    process::{exit, Command, Stdio},
};

mod cli;
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
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Run(args) => {
            let mut cwd = env::current_dir().expect("Couldn't find the current working directory");
            if cli.workspace_root {
                cwd = workspace::find_workspace_root(&cwd)?;
            }
            let manifest_path = cwd.join("package.json");
            let manifest = read_package_manifest(&manifest_path)?;
            if let Some(command) = manifest.scripts.get(&args.script) {
                eprintln!("> {command}");
                run_script(&args.script, command, &cwd)
            } else {
                PnError::MissingScript { name: args.script }
                    .pipe(MainError::Pn)
                    .pipe(Err)
            }
        }
        cli::Command::Install(args) => handle_passed_through("install", args),
        cli::Command::Update(args) => handle_passed_through("update", args),
        cli::Command::Other(args) => pass_to_sub(args.join(" ")),
    }
}

fn run_script(name: &str, command: &str, cwd: &Path) -> Result<(), MainError> {
    let mut path_env = OsString::from("node_modules/.bin");
    if let Some(path) = env::var_os("PATH") {
        path_env.push(":");
        path_env.push(path);
    }
    let status = Command::new("sh")
        .current_dir(cwd)
        .env("PATH", path_env)
        .arg("-c")
        .arg(command)
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
        Some(Some(status)) => PnError::ScriptError {
            name: name.to_string(),
            status,
        },
        None => PnError::UnexpectedTermination {
            command: command.to_string(),
        },
    }
    .pipe(MainError::Pn)
    .pipe(Err)
}

fn handle_passed_through(command: &str, args: PassedThroughArgs) -> Result<(), MainError> {
    let PassedThroughArgs { mut args } = args;
    args.insert(0, command.into());
    pass_to_pnpm(&args)
}

fn pass_to_pnpm(args: &[OsString]) -> Result<(), MainError> {
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
            command: format!(
                "pnpm {}",
                args.iter().map(|x| x.to_string_lossy()).join(" "),
            ),
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

fn read_package_manifest(manifest_path: &Path) -> Result<NodeManifest, MainError> {
    manifest_path
        .pipe(File::open)
        .map_err(|err| match err.kind() {
            ErrorKind::NotFound => MainError::Pn(PnError::NoPkgManifest {
                file: manifest_path.to_path_buf(),
            }),
            _ => MainError::from_dyn(err),
        })?
        .pipe(serde_json::de::from_reader::<_, NodeManifest>)
        .map_err(MainError::from_dyn)
}

#[test]
fn test_read_package_manifest() {
    use serde_json::to_string_pretty;
    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let package_json_path = temp_dir.path().join("package.json");
    fs::write(
        &package_json_path,
        r#"{"scripts": {"test": "echo hello world"}}"#,
    )
    .unwrap();

    let package_manifest = read_package_manifest(&package_json_path).unwrap();

    let received = serde_json::to_string_pretty(&package_manifest).unwrap();
    let expected = serde_json::json!({
        "scripts": {
            "test": "echo hello world"
        }
    })
    .pipe_ref(to_string_pretty)
    .unwrap();

    assert_eq!(received, expected);
}
