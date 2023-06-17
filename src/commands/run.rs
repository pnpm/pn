use super::CommandTrait;
use crate::{
    cli::{create_path_env, Config},
    error::{MainError, PnError},
    workspace,
};
use pipe_trait::Pipe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::ErrorKind;
use std::num::NonZeroI32;
use std::path::Path;
use std::{process, process::Stdio};

/// Runs a defined package script.
#[derive(Debug, clap::Args)]
#[clap(rename_all = "kebab-case")]
pub struct Run {
    /// Name of the package script to run.
    pub script: String, // Not OsString because it would be compared against package.json#scripts

    /// Arguments to pass to the package script.
    pub args: Vec<OsString>,
}

impl CommandTrait for Run {
    fn run(self, config: Config) -> Result<(), MainError> {
        let mut cwd = std::env::current_dir().expect("Couldn't find the current working directory");
        if config.workspace_root {
            cwd = workspace::find_workspace_root(&cwd)?;
        }
        let manifest_path = cwd.join("package.json");
        let manifest = read_package_manifest(&manifest_path)?;
        if let Some(command) = manifest.scripts.get(&self.script) {
            eprintln!(
                "\n> {pkg_name}@{pkg_version} {script} {path}",
                pkg_name = &manifest.name,
                pkg_version = &manifest.version,
                script = &self.script,
                path = cwd.display(),
            );
            eprintln!("> {command}\n");
            return run_script(&self.script, command, &cwd);
        } else {
            let e: Result<(), MainError> = PnError::MissingScript { name: self.script }
                .pipe(MainError::Pn)
                .pipe(Err);
            return e;
        }
    }
}

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeManifest {
    #[serde(default)]
    name: String,

    #[serde(default)]
    scripts: HashMap<String, String>,

    #[serde(default)]
    version: String,
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

fn run_script(name: &str, command: &str, cwd: &Path) -> Result<(), MainError> {
    let path_env = create_path_env()?;
    let status = process::Command::new("sh")
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_read_package_manifest_ok() {
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let package_json_path = temp_dir.path().join("package.json");
        fs::write(
            &package_json_path,
            r#"{"name": "", "scripts": {"test": "echo hello world"}, "version": ""}"#,
        )
        .unwrap();

        let package_manifest = read_package_manifest(&package_json_path).unwrap();

        let received = serde_json::to_string_pretty(&package_manifest).unwrap();
        let expected = serde_json::json!({
            "name": "",
            "version": "",
            "scripts": {
                "test": "echo hello world"
            }
        })
        .pipe_ref(serde_json::to_string_pretty)
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
}
