#![feature(abi_thiscall)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(not(target_os = "windows"))]
compile_error!("Windows support only");

pub mod types;
