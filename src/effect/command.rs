use mockall::automock;
use std::{ffi::OsStr, io, path::Path, process};

#[automock(
    type StdIn = ();
    type StdOut = ();
    type StdErr = ();
    type Child = ();
    type Error = ();
)]
pub trait Command {
    type StdIn;
    type StdOut;
    type StdErr;
    type Child;
    type Error;

    fn new<Name: AsRef<OsStr> + 'static>(name: Name) -> Self;
    fn with_current_dir<Value: AsRef<Path> + 'static>(self, value: Value) -> Self;
    fn with_env<Name: AsRef<OsStr> + 'static, Value: AsRef<OsStr> + 'static>(
        self,
        name: Name,
        value: Value,
    ) -> Self;
    fn with_arg<Value: AsRef<OsStr> + 'static>(self, value: Value) -> Self;
    fn with_stdin(self, stdin: Self::StdIn) -> Self;
    fn with_stdout(self, stdout: Self::StdOut) -> Self;
    fn with_stderr(self, stdout: Self::StdErr) -> Self;
    fn spawn(self) -> Result<Self::Child, Self::Error>;

    fn with_args<Args>(self, args: Args) -> Self
    where
        Args: IntoIterator + 'static,
        Args::Item: AsRef<OsStr>,
        Self: Sized,
    {
        let mut command = self;
        for arg in args {
            command = command.with_arg(arg);
        }
        command
    }
}

impl Command for process::Command {
    type StdIn = process::Stdio;
    type StdOut = process::Stdio;
    type StdErr = process::Stdio;
    type Child = process::Child;
    type Error = io::Error;

    fn new<Name: AsRef<OsStr>>(name: Name) -> Self {
        process::Command::new(name)
    }

    fn with_current_dir<Value: AsRef<Path>>(mut self, value: Value) -> Self {
        self.current_dir(value);
        self
    }

    fn with_env<Name: AsRef<OsStr>, Value: AsRef<OsStr>>(
        mut self,
        name: Name,
        value: Value,
    ) -> Self {
        self.env(name, value);
        self
    }

    fn with_arg<Value: AsRef<OsStr>>(mut self, value: Value) -> Self {
        self.arg(value);
        self
    }

    fn with_stdin(mut self, stdin: Self::StdIn) -> Self {
        self.stdin(stdin);
        self
    }

    fn with_stdout(mut self, stdout: Self::StdOut) -> Self {
        self.stdout(stdout);
        self
    }

    fn with_stderr(mut self, stderr: Self::StdErr) -> Self {
        self.stderr(stderr);
        self
    }

    fn spawn(mut self) -> Result<Self::Child, Self::Error> {
        process::Command::spawn(&mut self)
    }
}
