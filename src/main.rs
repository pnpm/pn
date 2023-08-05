use clap::Parser;
use cli::Cli;
use error::{MainError, PnError};
use format_buf::format as format_buf;
use indexmap::IndexMap;
use itertools::Itertools;
use os_display::Quoted;
use pipe_trait::Pipe;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    env,
    ffi::OsString,
    fs::File,
    io::{self, ErrorKind, Write},
    num::NonZeroI32,
    path::Path,
    process::{exit, Command, Stdio},
};
use yansi::Color::{Black, Red};

mod cli;
mod error;
mod passed_through;
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
    let cwd_and_manifest = || -> Result<_, MainError> {
        let mut cwd = env::current_dir().expect("Couldn't find the current working directory");
        if cli.workspace_root {
            cwd = workspace::find_workspace_root(&cwd)?;
        }
        let manifest_path = cwd.join("package.json");
        let manifest = read_package_manifest(&manifest_path)?;
        Ok((cwd, manifest))
    };
    let print_and_run_script = |manifest: &NodeManifest, name: &str, command: &str, cwd: &Path| {
        eprintln!(
            "\n> {name}@{version} {cwd}",
            name = &manifest.name,
            version = &manifest.version,
            cwd = dunce::canonicalize(cwd)
                .unwrap_or_else(|_| cwd.to_path_buf())
                .display(),
        );
        eprintln!("> {command}\n");
        run_script(name, command, cwd)
    };
    match cli.command {
        cli::Command::Run(args) => {
            let (cwd, manifest) = cwd_and_manifest()?;
            if let Some(name) = args.script {
                if let Some(command) = manifest.scripts.get(&name) {
                    let command = sh_command(command, &args.args);
                    print_and_run_script(&manifest, &name, &command, &cwd)
                } else {
                    PnError::MissingScript { name }
                        .pipe(MainError::Pn)
                        .pipe(Err)
                }
            } else if manifest.scripts.is_empty() {
                println!("There are no scripts in package.json");
                Ok(())
            } else {
                list_scripts(io::stdout(), manifest.scripts)
                    .map_err(PnError::WriteStdoutError)
                    .map_err(MainError::from)
            }
        }
        cli::Command::Other(args) => {
            let (cwd, manifest) = cwd_and_manifest()?;
            if let Some(name) = args.first() {
                let name = name.as_str();
                if passed_through::PASSED_THROUGH_COMMANDS.contains(name) {
                    return pass_to_pnpm(&args); // args already contain name, no need to prepend
                }
                if let Some(command) = manifest.scripts.get(name) {
                    let command = sh_command(command, &args[1..]);
                    return print_and_run_script(&manifest, name, &command, &cwd);
                }
            }
            pass_to_sub(args.join(" "))
        }
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
        .map_err(PnError::SpawnProcessError)?
        .wait()
        .map_err(PnError::WaitProcessError)?
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

fn sh_command<'a>(command: &'a str, args: &[String]) -> Cow<'a, str> {
    if args.is_empty() {
        return Cow::Borrowed(command);
    }
    let mut command = command.to_string();
    for arg in args {
        let quoted = Quoted::unix(arg); // because pn uses `sh -c` even on Windows
        format_buf!(command, " {quoted}");
    }
    Cow::Owned(command)
}

fn list_scripts(
    mut stdout: impl Write,
    script_map: impl IntoIterator<Item = (String, String)>,
) -> io::Result<()> {
    writeln!(stdout, "Commands available via `pn run`:")?;
    for (name, command) in script_map {
        writeln!(stdout, "  {name}")?;
        writeln!(stdout, "    {command}")?;
    }
    Ok(())
}

fn pass_to_pnpm(args: &[String]) -> Result<(), MainError> {
    let status = Command::new("pnpm")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(PnError::SpawnProcessError)?
        .wait()
        .map_err(PnError::WaitProcessError)?
        .code()
        .map(NonZeroI32::new);
    Err(match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => MainError::Sub(status),
        None => MainError::Pn(PnError::UnexpectedTermination {
            command: format!("pnpm {}", args.iter().join(" ")),
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
        .map_err(PnError::SpawnProcessError)?
        .wait()
        .map_err(PnError::WaitProcessError)?
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
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => PnError::NoPkgManifest {
                file: manifest_path.to_path_buf(),
            },
            _ => PnError::FsError {
                path: manifest_path.to_path_buf(),
                error,
            },
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
        .map_err(PnError::NodeBinPathError)
        .map_err(MainError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_list_scripts() {
        let script_map = [
            ("hello", "echo hello"),
            ("world", "echo world"),
            ("foo", "echo foo"),
            ("bar", "echo bar"),
            ("abc", "echo abc"),
            ("def", "echo def"),
        ]
        .map(|(k, v)| (k.to_string(), v.to_string()));
        let mut buf = Vec::<u8>::new();
        list_scripts(&mut buf, script_map).unwrap();
        let received = String::from_utf8_lossy(&buf);
        let expected = [
            "Commands available via `pn run`:",
            "  hello",
            "    echo hello",
            "  world",
            "    echo world",
            "  foo",
            "    echo foo",
            "  bar",
            "    echo bar",
            "  abc",
            "    echo abc",
            "  def",
            "    echo def",
        ]
        .join("\n");
        assert_eq!(received.trim(), expected.trim());
    }

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
