//! MatSystemSurface type module

#![allow(non_camel_case_types)]

pub use crate::implement_vftable_holder_utilities;
use shared::call_virtual_function;
use std::ffi::{c_int, c_uchar};

/// Game CMatSystemSurface class partial implementation for usage
#[repr(C)]
#[repr(packed(1))]
pub struct CMatSystemSurface {}

/// Rust-end structure for CMatSystemSurface
pub struct MatSystemSurface(*mut CMatSystemSurface);

/// Indices enumerator for CMatSystemSurface
#[repr(isize)]
pub enum MatSystemSurfaceIndices {
    SetDrawColor = 11,
    PaintTraverse = 88,
}

// Game function types
pub type SetDrawColor_t =
    unsafe extern "thiscall" fn(*const usize, c_uchar, c_uchar, c_uchar, c_int) -> c_int;

impl MatSystemSurface {
    pub fn set_draw_color(&self, r: c_uchar, g: c_uchar, b: c_uchar, a: c_int) -> c_int {
        call_virtual_function!(
            self.0,
            MatSystemSurfaceIndices::SetDrawColor,
            SetDrawColor_t,
            r,
            g,
            b,
            a
        )
    }
}

implement_vftable_holder_utilities!(MatSystemSurface);
