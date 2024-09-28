use clap::Parser;
use cli::Cli;
use error::{MainError, PnError};
use pipe_trait::Pipe;
use shell_quoted::ShellQuoted;
use std::{
    env,
    io::{self, Write},
    path::Path,
    process::exit,
};
use yansi::Color::{Black, Red};

mod cli;

use pn::error;
use pn::passed_through;
use pn::shell_quoted;
use pn::utils::*;
use pn::workspace;
use pn::NodeManifest;

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
    let print_and_run_script =
        |manifest: &NodeManifest, name: &str, command: ShellQuoted, cwd: &Path| {
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
                    let command = ShellQuoted::from_command_and_args(command.into(), &args.args);
                    print_and_run_script(&manifest, &name, command, &cwd)
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
                    let command = ShellQuoted::from_command_and_args(command.into(), &args[1..]);
                    return print_and_run_script(&manifest, name, command, &cwd);
                }
            }
            pass_to_sub(ShellQuoted::from_args(args))
        }
    }
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
