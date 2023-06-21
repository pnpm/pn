use std::{
    io::{self, Stdout, Write},
    process,
};

mod child;
mod command;

pub use child::Child;
pub use command::Command;

#[cfg(test)]
pub use child::MockChild;
#[cfg(test)]
pub use command::MockCommand;

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

#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockEffect;

#[cfg(test)]
impl Effect for MockEffect {
    type Command = MockCommand;
    type Stdout = Vec<u8>;

    fn stdout(&self) -> Self::Stdout {
        Vec::new()
    }
}
