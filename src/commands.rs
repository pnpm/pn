use clap::Subcommand;
use strum::{Display as StrumDisplay, EnumString};

/// An enum of commands that need to be passed to pnpm
#[derive(Debug, PartialEq, EnumString, StrumDisplay, Subcommand)]
#[strum(ascii_case_insensitive)]
#[strum(serialize_all = "kebab-case")]
pub enum PnpmCommands {
    // commands that pnpm passes to npm
    Access,
    Adduser,
    Bugs,
    Deprecate,
    DistTag,
    Docs,
    Edit,
    Info,
    Login,
    Logout,
    Owner,
    Ping,
    Prefix,
    Profile,
    Pkg,
    Repo,
    S,
    Se,
    Search,
    SetScript,
    Show,
    Star,
    Stars,
    Team,
    Token,
    Unpublish,
    Unstar,
    V,
    Version,
    View,
    Whoami,
    Xmas,

    // completion commands
    InstallCompletion,
    UninstallCompletion,

    // pnpm commands

    // manage deps
    Add,
    /// Installs all dependencies of the project in the current working directory.
    /// When executed inside a workspace, installs all dependencies of all projects.
    #[strum(serialize = "i")] // aliases
    Install,
    /// Updates packages to their latest version based on the specified range.
    /// You can use "*" in package name to update all packages with the same pattern.
    #[strum(serialize = "up")] // aliases
    Update,
    Remove,
    Link,
    Unlink,
    Import,
    Rebuild,
    Prune,
    Fetch,
    InstallTest,
    Dedupe,

    // patch deps
    Patch,
    PatchCommit,
    PatchRemove,

    // review deps
    Audit,
    List,
    Outdated,
    Why,
    Licenses,

    // run scripts
    Dlx,
    Create,

    // manage environments
    Env,

    // misc
    Publish,
    Pack,
    Server,
    Store,
    Root,
    Bin,
    Setup,
    Init,
    Deploy,
    Doctor,
    Config,
}
