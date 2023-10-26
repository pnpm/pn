use os_display::Quoted;
use std::{
    ffi::OsStr,
    fmt::{self, Write as _},
};

#[derive(Debug, Clone, Default)]
pub struct ShellQuoted(String);

impl fmt::Display for ShellQuoted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<OsStr> for ShellQuoted {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl From<ShellQuoted> for String {
    fn from(value: ShellQuoted) -> Self {
        value.0
    }
}

impl ShellQuoted {
    /// `command` will not be quoted
    pub fn from_command(command: String) -> Self {
        Self(command)
    }

    pub fn push_arg<S: AsRef<str>>(&mut self, arg: S) {
        let delim = if self.0.is_empty() { "" } else { " " };
        let quoted = Quoted::unix(arg.as_ref()); // because pn uses `sh -c` even on Windows
        write!(&mut self.0, "{delim}{quoted}").expect("String write doesn't panic");
    }

    // convenience methods based on usage

    pub fn from_command_and_args<S: AsRef<str>, I: IntoIterator<Item = S>>(
        command: String,
        args: I,
    ) -> Self {
        let mut cmd = Self::from_command(command);
        for arg in args {
            cmd.push_arg(arg);
        }
        cmd
    }

    pub fn from_args<S: AsRef<str>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::from_command_and_args(String::default(), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_command_and_args() {
        let command = ShellQuoted::from_command_and_args(
            "echo hello world".into(),
            ["abc", ";ls /etc", "ghi jkl", "\"", "'"],
        );
        assert_eq!(
            command.to_string(),
            r#"echo hello world 'abc' ';ls /etc' 'ghi jkl' '"' "'""#
        );
    }
}