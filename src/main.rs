use clap::Parser;
use cli::{Cli, PassedThroughArgs};
use error::{MainError, PnError};
use indexmap::IndexMap;
use itertools::Itertools;
use pipe_trait::Pipe;
use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsString,
    fs::File,
    io::ErrorKind,
    num::NonZeroI32,
    path::Path,
    process::{exit, Command, Stdio},
};
use yansi::Color::{Black, Red};

mod cli;
mod error;
mod workspace;

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct NodeManifest {
    #[serde(default)]
    name: String,

    #[serde(default)]
    version: String,

    #[serde(default)]
    scripts: IndexMap<String, String>,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(MainError::Sub(status)) => exit(status.get()),
        Err(MainError::Pn(error)) => {
            eprintln!(
                "{prefix} {error}",
                prefix = Black.paint("\u{2009}ERROR\u{2009}").bg(Red),
                error = Red.paint(error),
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
            if let Some(name) = args.script {
                if let Some(command) = manifest.scripts.get(&name) {
                    eprintln!(
                        "\n> {}@{} {}",
                        &manifest.name,
                        &manifest.version,
                        &cwd.display()
                    );
                    eprintln!("> {command}\n");
                    run_script(&name, command, &cwd)
                } else {
                    PnError::MissingScript { name }
                        .pipe(MainError::Pn)
                        .pipe(Err)
                }
            } else if manifest.scripts.is_empty() {
                println!("There are no scripts in package.json");
                Ok(())
            } else {
                println!("Commands available via `pn run`:");
                for (name, command) in manifest.scripts {
                    println!("  {name}");
                    println!("    {command}");
                }
                Ok(())
            }
        }
        cli::Command::Install(args) => handle_passed_through("install", args),
        cli::Command::Update(args) => handle_passed_through("update", args),
        cli::Command::Other(args) => pass_to_sub(args.join(" ")),
    }
}

fn run_script(name: &str, command: &str, cwd: &Path) -> Result<(), MainError> {
    let path_env = create_path_env()?;
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
    let path_env = create_path_env()?;
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
        .map_err(|err| {
            MainError::Pn(PnError::ParseJsonError {
                file: manifest_path.to_path_buf(),
                message: err.to_string(),
            })
        })
}

fn create_path_env() -> Result<OsString, MainError> {
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
    use serde_json::json;

    #[test]
    fn test_read_package_manifest_ok() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        fs::write(
            &package_json_path,
            r#"{"scripts": {"test": "echo hello world"}}"#,
        )
        .unwrap();

        let received = read_package_manifest(&package_json_path).unwrap();

        let expected: NodeManifest = json!({
            "scripts": {
                "test": "echo hello world"
            }
        })
        .pipe(serde_json::from_value)
        .unwrap();

        assert_eq!(received, expected);
    }

    #[test]
    fn test_read_package_manifest_error() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        fs::write(
            &package_json_path,
            r#"{"scripts": {"test": "echo hello world",}}"#,
        )
        .unwrap();

        let received_error = read_package_manifest(&package_json_path).unwrap_err();
        dbg!(&received_error);
        assert!(matches!(
            received_error,
            MainError::Pn(PnError::ParseJsonError { .. }),
        ));

        let received_message = received_error.to_string();
        eprintln!("MESSAGE:\n{received_message}\n");
        let expected_message =
            format!("Failed to parse {package_json_path:?}: trailing comma at line 1 column 41",);
        assert_eq!(received_message, expected_message);
    }

    #[test]
    fn test_create_path_env() {
        let bin_path = Path::new("node_modules").join(".bin");
        let path_env = create_path_env().expect("prepend 'node_modules/.bin' to PATH");

        let first_path = env::split_paths(&path_env).next();
        assert_eq!(first_path, Some(bin_path));
    }
}
