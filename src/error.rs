use crate::shell_quoted::ShellQuoted;
use std::{env::JoinPathsError, io, num::NonZeroI32, path::PathBuf};

/// Error types emitted by `pn` itself.
#[derive(Debug, thiserror::Error)]
pub enum PnError {
    /// Script not found when running `pn run`.
    #[error("Missing script: {name}")]
    MissingScript { name: String },

    /// Script ran by `pn run` exits with non-zero status code.
    #[error("Command failed with exit code {status}")]
    ScriptError { name: String, status: NonZeroI32 },

    /// Subprocess finishes but without a status code.
    #[error("Command ended unexpectedly: {command}")]
    UnexpectedTermination { command: ShellQuoted },

    /// Fail to spawn a subprocess.
    #[error("Failed to spawn process: {0}")]
    SpawnProcessError(#[source] io::Error),

    /// Fail to wait for the subprocess to finish.
    #[error("Failed to wait for the process: {0}")]
    WaitProcessError(#[source] io::Error),

    /// The program receives --workspace-root outside a workspace.
    #[error("--workspace-root may only be used in a workspace")]
    NotInWorkspace,

    /// No package manifest.
    #[error("File not found: {file:?}")]
    NoPkgManifest { file: PathBuf },

    /// Error related to filesystem operation.
    #[error("{path:?}: {error}")]
    FsError { path: PathBuf, #[source] error: io::Error },

    /// Error emitted by [`lets_find_up`]'s functions.
    #[error("Failed to find {file_name:?} from {start_dir:?} upward: {error}")]
    FindUpError {
        start_dir: PathBuf,
        file_name: &'static str,
        #[source]
        error: io::Error,
    },

    /// An error is encountered when write to stdout.
    #[error("Failed to write to stdout: {0}")]
    WriteStdoutError(io::Error),

    /// Parse JSON error.
    #[error("Failed to parse {file:?}: {message}")]
    ParseJsonError { file: PathBuf, message: String },

    /// Failed to prepend `node_modules/.bin` to `PATH`.
    #[error("Cannot add `node_modules/.bin` to PATH: {0}")]
    NodeBinPathError(JoinPathsError),
}

/// The main error type.
#[derive(Debug, thiserror::Error)]
pub enum MainError {
    /// Errors emitted by `pn` itself.
    #[error(transparent)]
    Pn(#[from] PnError),

    /// The subprocess that takes control exits with non-zero status code.
    #[error("{0}")]
    Sub(NonZeroI32),
}
