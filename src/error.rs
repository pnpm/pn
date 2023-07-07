use derive_more::{Display, From};
use std::{env::JoinPathsError, io, num::NonZeroI32, path::PathBuf};

/// Error types emitted by `pn` itself.
#[derive(Debug, Display)]
pub enum PnError {
    /// Script not found when running `pn run`.
    #[display(fmt = "Missing script: {name}")]
    MissingScript { name: String },

    /// Script ran by `pn run` exits with non-zero status code.
    #[display(fmt = "Command failed with exit code {status}")]
    ScriptError { name: String, status: NonZeroI32 },

    /// Subprocess finishes but without a status code.
    #[display(fmt = "Command {command:?} has ended unexpectedly")]
    UnexpectedTermination { command: String },

    /// Fail to spawn a subprocess.
    #[display(fmt = "Failed to spawn process: {_0}")]
    SpawnProcessError(io::Error),

    /// Fail to wait for the subprocess to finish.
    #[display(fmt = "Failed to wait for the process: {_0}")]
    WaitProcessError(io::Error),

    /// The program receives --workspace-root outside a workspace.
    #[display(fmt = "--workspace-root may only be used in a workspace")]
    NotInWorkspace,

    /// No package manifest.
    #[display(fmt = "File not found: {file:?}")]
    NoPkgManifest { file: PathBuf },

    /// Error related to filesystem operation.
    #[display(fmt = "{path:?}: {error}")]
    FsError { path: PathBuf, error: io::Error },

    /// Error emitted by [`lets_find_up`]'s functions.
    #[display(fmt = "Failed to find {file_name:?} from {start_dir:?} upward: {error}")]
    FindUpError {
        start_dir: PathBuf,
        file_name: &'static str,
        error: io::Error,
    },

    /// An error is encountered when write to stdout.
    #[display(fmt = "Failed to write to stdout: {_0}")]
    WriteStdoutError(io::Error),

    /// Parse JSON error.
    #[display(fmt = "Failed to parse {file:?}: {message}")]
    ParseJsonError { file: PathBuf, message: String },

    /// Failed to prepend `node_modules/.bin` to `PATH`.
    #[display(fmt = "Cannot add `node_modules/.bin` to PATH: {_0}")]
    NodeBinPathError(JoinPathsError),
}

/// The main error type.
#[derive(Debug, Display, From)]
pub enum MainError {
    /// Errors emitted by `pn` itself.
    Pn(PnError),

    /// The subprocess that takes control exits with non-zero status code.
    #[from(ignore)]
    Sub(NonZeroI32),
}
