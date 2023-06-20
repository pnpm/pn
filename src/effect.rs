use std::{
    io::{self, Stdout, Write},
    process,
};

mod command;

pub use command::{Command, MockCommand};

pub trait Effect {
    type Command: Command;
    type Stdout: Write;
    fn stdout(&self) -> Self::Stdout;
}

#[derive(Debug)]
pub struct MainEffect;

impl Effect for MainEffect {
    type Command = process::Command;
    type Stdout = Stdout;

    fn stdout(&self) -> Self::Stdout {
        io::stdout()
    }
}

#[derive(Debug, Default)]
pub struct MockEffect;

impl Effect for MockEffect {
    type Command = MockCommand;
    type Stdout = Vec<u8>;

    fn stdout(&self) -> Self::Stdout {
        Vec::new()
    }
}
