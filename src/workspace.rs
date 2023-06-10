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
    workspace_manifest_location
        .and_then(|x| x.parent().map(Path::to_path_buf))
        .ok_or(MainError::Pn(PnError::NotInWorkspace))
}
