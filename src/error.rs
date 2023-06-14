use derive_more::Display;
use std::{env::JoinPathsError, error::Error, num::NonZeroI32};

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

    /// The program receives --workspace-root outside a workspace.
    #[display(fmt = "--workspace-root may only be used in a workspace")]
    NotInWorkspace,

    /// Failed to prepend `node_modules/.bin` to `PATH`.
    #[display(fmt = "Cannot add `node_modules/.bin` to PATH: {error}")]
    NodeBinPathError { error: JoinPathsError },

    /// Other errors.
    #[display(fmt = "{error}")]
    Other { error: Box<dyn Error> },
}

/// The main error type.
#[derive(Debug)]
pub enum MainError {
    /// Errors emitted by `pn` itself.
    Pn(PnError),

    /// The subprocess that takes control exits with non-zero status code.
    Sub(NonZeroI32),
}

impl MainError {
    pub fn from_dyn(error: impl Error + 'static) -> Self {
        MainError::Pn(PnError::Other {
            error: Box::new(error),
        })
    }
}
