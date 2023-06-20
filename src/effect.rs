use std::{
    io::{Stdout, Write},
    process,
};

mod command;

pub use command::{Command, MockCommand};

pub trait Effect {
    type Command: Command;
    type Stdout: Write;
}

#[derive(Debug)]
pub struct MainEffect {
    pub stdout: Stdout,
}

impl Effect for MainEffect {
    type Command = process::Command;
    type Stdout = Stdout;
}

#[derive(Debug, Default)]
pub struct MockEffect {
    pub stdout: Vec<u8>,
}

impl Effect for MockEffect {
    type Command = MockCommand;
    type Stdout = Vec<u8>;
}
