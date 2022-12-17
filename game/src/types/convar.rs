//! Cvar type module

#![allow(non_camel_case_types)]

pub use crate::implement_vftable_holder_utilities;
use shared::call_virtual_function;
use std::ffi::c_schar;

/// Game CCvar class partial implementation for usage
#[repr(C)]
#[repr(packed(1))]
pub struct CCvar {}

/// Rust-end structure for CCvar
pub struct Cvar(*mut CCvar);

/// Indices enumerator for CCvar
#[repr(isize)]
pub enum CvarIndices {
    ConsolePrint = 24,
}

// Game function types
type ConsolePrint_t = unsafe extern "cdecl" fn(*const usize, *const c_schar);

impl Cvar {
    pub fn console_print(&self, text: *const c_schar) {
        call_virtual_function!(self.0, CvarIndices::ConsolePrint, ConsolePrint_t, text);
    }
}

implement_vftable_holder_utilities!(Cvar);
