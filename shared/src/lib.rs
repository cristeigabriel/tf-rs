#![feature(slice_pattern)]
#![feature(strict_provenance)]
#![feature(unchecked_math)]
#![feature(abi_thiscall)]

#[cfg(not(target_os = "windows"))]
compile_error!("Windows support only");

pub mod console;
pub mod error;
pub mod memory;
use std::error::Error;
pub use std::ffi::{c_schar, c_uchar};
pub use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, LPVOID};
use winapi::um::winbase::lstrlenA;

/// Allow for error decayal so we can work with multiple error types
pub type GenericErrOr<T> = Result<T, Box<dyn Error>>;

/// Wrap windows shared library entry point to only take a closure
#[macro_export]
macro_rules! entry_point {
    ($f: expr) => {
        #[no_mangle]
        unsafe extern "stdcall" fn DllMain(
            module: HMODULE,
            reason_for_call: DWORD,
            _: LPVOID,
        ) -> BOOL {
            // Coerce values
            $f(module, reason_for_call as u32).into()
        }
    };
}

/// Syntactical sugar for `*const c_schar`
#[macro_export]
macro_rules! c_str {
    ($str:literal) => {
        $str.as_ptr() as *const $crate::c_schar
    };
}

/// Turns null-terminated UTF-8 array `raw` to `String`, if not nil or others
///
/// # Example
///
/// ```rust
/// if let Some(rs) = read_c_string(&['a', 'b', 'c', '\0'].as_ptr() as _) {
///     println!("Hello {}!", rs)
/// }
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn read_c_string(raw: *const c_uchar) -> Option<String> {
    unsafe {
        // Verify if it's null or begins with sentinel character
        if raw.is_null() || *raw == b'\0' {
            return None;
        }

        let string_length = lstrlenA(raw as _);
        if let Ok(result) = std::str::from_utf8(
            (0..string_length)
                .map(|i| *raw.offset(i as isize))
                .collect::<Vec<u8>>()
                .as_ref(),
        ) {
            Some(result.to_owned())
        } else {
            None
        }
    }
}
