use std::{
    error,
    fmt::{self, Display, Formatter},
};

/// Error type for all context errors
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Error {
    /// Object already initialized (often global context)
    AlreadyInitialized,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::AlreadyInitialized => "already initialized",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(TF): {}", match *self {
            Error::AlreadyInitialized => "Encountered call to an installer for an object that's already initialized (most often global context)"
        })
    }
}
