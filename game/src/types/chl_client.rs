//! CHLClient type module

#![allow(non_camel_case_types)]

pub use crate::implement_vftable_holder_utilities;

/// Game CHLClient class partial implementation for usage
#[repr(C)]
#[repr(packed(1))]
pub struct CCHLClient {}

/// Rust-end structure for CCHLClient
pub struct CHLClient(*mut CCHLClient);

/// Indices enumerator for CCHLClient
#[repr(isize)]
pub enum CHLClientIndices {
    FrameStageNotify = 35,
}

impl CHLClient {}

implement_vftable_holder_utilities!(CHLClient);
