use super::error::{MainError, PnError};
use lets_find_up::*;
use std::path::{Path, PathBuf};

const WORKSPACE_MANIFEST_FILENAME: &str = "pnpm-workspace.yaml";

pub fn find_workspace_root(cwd: &dyn AsRef<Path>) -> Result<PathBuf, MainError> {
    let workspace_manifest_location = find_up_with(
        WORKSPACE_MANIFEST_FILENAME,
        FindUpOptions {
            kind: FindUpKind::File,
            cwd: cwd.as_ref(),
        },
    )
    .map_err(MainError::from_dyn)?;
    match workspace_manifest_location {
        Some(path) => Ok(path
            .parent()
            .ok_or(MainError::Pn(PnError::NotInWorkspace))?
            .to_path_buf()),
        None => Err(MainError::Pn(PnError::NotInWorkspace)),
    }
}
