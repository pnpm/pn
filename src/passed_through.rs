/// Commands that need to be passed to pnpm.
pub const PASSED_THROUGH_COMMANDS: phf::Set<&str> = phf::phf_set! {
    // commands that pnpm passes to npm
    "access",
    "adduser",
    "bugs",
    "deprecate",
    "dist-tag",
    "docs",
    "edit",
    "info",
    "login",
    "logout",
    "owner",
    "ping",
    "prefix",
    "profile",
    "pkg",
    "repo",
    "s",
    "se",
    "search",
    "set-script",
    "show",
    "star",
    "stars",
    "team",
    "token",
    "unpublish",
    "unstar",
    "v",
    "version",
    "view",
    "whoami",
    "xmas",

    // completion commands
    "install-completion",
    "uninstall-completion",

    // pnpm commands

    // manage deps
    "add",
    "i",
    "install",
    "up",
    "update",
    "remove",
    "link",
    "unlink",
    "import",
    "rebuild",
    "prune",
    "fetch",
    "install-test",
    "dedupe",

    // patch deps
    "patch",
    "patch-commit",
    "patch-remove",

    // review deps
    "audit",
    "list",
    "outdated",
    "why",
    "licenses",

    // run scripts
    "dlx",
    "create",

    // manage environments
    "env",

    // misc
    "publish",
    "pack",
    "server",
    "store",
    "root",
    "bin",
    "setup",
    "init",
    "deploy",
    "doctor",
    "config",
};
