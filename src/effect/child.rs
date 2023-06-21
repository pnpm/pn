#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock(
    type ExitStatus = ();
    type Error = ();
))]
pub trait Child {
    type ExitStatus;
    type Error;
    fn wait(self) -> Result<Self::ExitStatus, Self::Error>;
}
