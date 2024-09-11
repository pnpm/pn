use crate::{
    error::{MainError, PnError},
    shell_quoted::ShellQuoted,
    NodeManifest,
};
use pipe_trait::Pipe;
use std::{
    env,
    ffi::OsString,
    fs::File,
    io::ErrorKind,
    num::NonZeroI32,
    path::Path,
    process::{Command, Stdio},
};

pub fn run_script(name: &str, command: ShellQuoted, cwd: &Path) -> Result<(), MainError> {
    let path_env = create_path_env()?;
    let status = Command::new("sh")
        .current_dir(cwd)
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
    match status {
        Some(None) => return Ok(()),
        Some(Some(status)) => PnError::ScriptError {
            name: name.to_string(),
            status,
        },
        None => PnError::UnexpectedTermination { command },
    }
    .pipe(MainError::Pn)
    .pipe(Err)
}

pub fn pass_to_pnpm(args: &[String]) -> Result<(), MainError> {
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
            command: ShellQuoted::from_command_and_args("pnpm".into(), args),
        }),
    })
}

pub fn pass_to_sub(command: ShellQuoted) -> Result<(), MainError> {
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

pub fn read_package_manifest(manifest_path: &Path) -> Result<NodeManifest, MainError> {
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

pub fn create_path_env() -> Result<OsString, MainError> {
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
