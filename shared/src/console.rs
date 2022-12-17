//! Console module

use crate::{error::Error, GenericErrOr};
use std::io;
use winapi::um::{
    consoleapi::AllocConsole,
    wincon::{FreeConsole, SetConsoleTitleA},
};

/// Handle all functions that update last error when return value is zero
macro_rules! handle_winapi_against_zero {
    ($f: ident, $($arg:tt)*) => {
        (|| unsafe {
            if $f($($arg)*) == 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        })()
    };
    ($f: ident) => {
        handle_winapi_against_zero!($f,)
    };
}

/// Type for console errors
pub type ConsoleErrOr<T> = GenericErrOr<T>;

pub fn alloc_console() -> ConsoleErrOr<()> {
    handle_winapi_against_zero!(AllocConsole)
}

pub fn free_console() -> ConsoleErrOr<()> {
    handle_winapi_against_zero!(FreeConsole)
}

pub fn set_title(title: &str) -> ConsoleErrOr<()> {
    if !title.ends_with('\0') {
        return Err(Error::NoSentinelCharacter.into());
    }

    handle_winapi_against_zero!(SetConsoleTitleA, title.as_ptr() as _)
}
