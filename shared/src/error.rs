use std::{
    error,
    fmt::{self, Display, Formatter},
};

/// Error type for all shared errors
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Encountered null pointer attempting to do some operation
    NullPointer,
    /// Working with C strings, there's no null terminator, or any sort of expected sentinel character
    NoSentinelCharacter,
    /// Can't find something, generally in memory scanning
    CantFind,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NullPointer => "null pointer",
            Error::NoSentinelCharacter => "no sentinel character",
            Error::CantFind => "failed a find operation",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(Shared): {}",
            match *self {
                Error::NullPointer => "Encountered a null pointer (most often happens in dereferencing operation)",
                Error::NoSentinelCharacter => "Encountered a string interacting with a C API, without a sentinel character (most often null terminator)",
                Error::CantFind => "Failed to perform a find operation (most often memory related)",
            }
        )
    }
}
