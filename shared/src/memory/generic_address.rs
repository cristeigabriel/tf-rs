//! Generic address module

use crate::{error::Error, GenericErrOr};
use std::marker::PhantomData;

/// Type for generic address errors
pub type GenericAddressErrOr<T> = GenericErrOr<T>;

/// Generic address holder, implements basic operations,
/// preserves potentailly wanted type but also allows for
/// casting unrelated to it. Void by default, so it can be ignored, where
/// that fits better
#[derive(Debug, Clone, Copy)]
pub struct GenericAddress<T = ()> {
    resource: *const (),
    /// Exists so we can hold T
    phantom_data: PhantomData<T>,
}

impl<T> GenericAddress<T> {
    pub fn new(resource: *const ()) -> Self {
        Self {
            resource,
            phantom_data: PhantomData::default(),
        }
    }

    /// Get `resource` address
    pub fn exposed_addr(&self) -> usize {
        self.resource.expose_addr()
    }

    /// Get `resource`
    pub fn get_resource(&self) -> *const () {
        self.resource
    }

    /// Get `resource` handle as `*const R`
    #[allow(invalid_type_param_default)]
    pub fn get_ptr<R = T>(&self) -> *const R {
        self.resource as _
    }

    /// Get `resource` handle as `*mut T`
    #[allow(invalid_type_param_default)]
    pub fn get_ptr_mut<R = T>(&self) -> *mut R {
        self.resource as _
    }

    /// Get `resource` handle as `R`, if not null
    ///
    /// # Safety
    ///
    /// Copies `size_of::<R>()` bytes from `&self.get_ptr()`, therefore, make sure
    /// that the reads are safe on your end
    pub fn transmute<R>(&self) -> GenericAddressErrOr<R> {
        if self.resource.is_null() {
            Err(Error::NullPointer.into())
        } else {
            Ok(unsafe { std::mem::transmute_copy::<*const T, R>(&self.get_ptr()) })
        }
    }

    /// Offset from `resource` by `offset` bytes
    ///
    /// # Safety
    ///
    /// Unwraps `checked_add_signed`
    pub fn offset(self, offset: isize) -> GenericAddressErrOr<Self> {
        let add = self.exposed_addr().checked_add_signed(offset).unwrap();
        if add == 0 {
            Err(Error::NullPointer.into())
        } else {
            Ok(Self::from(add))
        }
    }

    /// Dereference `resource` `branches` times
    pub fn deref(mut self, mut branches: usize) -> GenericAddressErrOr<Self> {
        while branches > 0 {
            self.resource = unsafe { *(self.resource as *const _) };
            if self.resource.is_null() {
                return Err(Error::NullPointer.into());
            }

            branches -= 1;
        }

        Ok(self)
    }
}

impl<T> From<usize> for GenericAddress<T> {
    fn from(value: usize) -> Self {
        Self::new(value as _)
    }
}
