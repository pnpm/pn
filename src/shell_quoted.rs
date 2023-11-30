use derive_more::{Display, Into};
use os_display::Quoted;
use std::ffi::OsStr;

#[derive(Debug, Display, Into)]
pub struct ShellQuoted(String);

impl AsRef<OsStr> for ShellQuoted {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl ShellQuoted {
    /// `command` will not be quoted
    pub fn from_command(command: String) -> Self {
        Self(command)
    }

    pub fn push_arg<S: AsRef<str>>(&mut self, arg: S) {
        use std::fmt::Write;
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        let quoted = Quoted::unix(arg.as_ref()); // because pn uses `sh -c` even on Windows
        write!(self.0, "{quoted}").expect("string write doesn't panic");
    }

    pub fn from_command_and_args<Args>(command: String, args: Args) -> Self
    where
        Args: IntoIterator,
        Args::Item: AsRef<str>,
    {
        let mut cmd = Self::from_command(command);
        for arg in args {
            cmd.push_arg(arg);
        }
        cmd
    }

    pub fn from_args<Args>(args: Args) -> Self
    where
        Args: IntoIterator,
        Args::Item: AsRef<str>,
    {
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
