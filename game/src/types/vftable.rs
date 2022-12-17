//! Vftable trait module

pub use shared::get_virtual_function;
pub use shared::memory::generic_address::GenericAddress;

pub trait VfTable {
    fn get_virtual_function(&self, index: isize) -> GenericAddress;
}

// TODO: Rewrite this shit.
/// Implements `get_function` using `self.0` for external usage.
/// Also implements `From<*const ()>` and `Default` to facillitate
/// the interop interface and avoid code repetition, by using the
/// knowledge of `$class` being a tuple struct.
#[macro_export]
macro_rules! implement_vftable_holder_utilities {
    ($class:ty) => {
        impl $crate::types::vftable::VfTable for $class {
            fn get_virtual_function(&self, index: isize) -> $crate::types::vftable::GenericAddress {
                $crate::types::vftable::GenericAddress::new(
                    $crate::types::vftable::get_virtual_function!(self.0, index) as *const (),
                )
            }
        }

        impl From<*const ()> for $class {
            fn from(value: *const ()) -> Self {
                Self(value as _)
            }
        }

        impl Default for $class {
            fn default() -> Self {
                Self(std::ptr::null_mut())
            }
        }
    };
}
