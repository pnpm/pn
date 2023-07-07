use super::error::{MainError, PnError};
use lets_find_up::*;
use std::path::{Path, PathBuf};

const WORKSPACE_MANIFEST_FILENAME: &str = "pnpm-workspace.yaml";

pub fn find_workspace_root(cwd: &Path) -> Result<PathBuf, MainError> {
    let options = FindUpOptions {
        kind: FindUpKind::File,
        cwd,
    };
    find_up_with(WORKSPACE_MANIFEST_FILENAME, options)
        .map_err(|error| PnError::FindUpError {
            start_dir: cwd.to_path_buf(),
            file_name: WORKSPACE_MANIFEST_FILENAME,
            error,
        })?
        .and_then(|x| x.parent().map(Path::to_path_buf))
        .ok_or(MainError::Pn(PnError::NotInWorkspace))
}
