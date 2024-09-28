//! pnpm wrapper library.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod passed_through;
pub mod shell_quoted;
pub mod utils;
pub mod workspace;

/// Structure of `package.json`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NodeManifest {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub scripts: IndexMap<String, String>,
}
