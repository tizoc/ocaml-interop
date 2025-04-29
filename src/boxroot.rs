// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, ops::Deref};

use ocaml_boxroot_sys::{
    boxroot_create, boxroot_delete, boxroot_error_string, boxroot_get, boxroot_get_ref,
    boxroot_modify, BoxRoot as PrimitiveBoxRoot,
};

use crate::{memory::OCamlCell, OCaml, OCamlRef, OCamlRuntime};

/// `BoxRoot<T>` is a container for a rooted [`OCaml`]`<T>` value.
pub struct BoxRoot<T: 'static> {
    boxroot: PrimitiveBoxRoot,
    _marker: PhantomData<T>,
}

fn boxroot_fail() -> ! {
    let reason = unsafe { std::ffi::CStr::from_ptr(boxroot_error_string()) }.to_string_lossy();
    panic!("Failed to allocate boxroot, boxroot_error_string() -> {reason}");
}

impl<T> BoxRoot<T> {
    /// Creates a new root from an [`OCaml`]`<T>` value.
    pub fn new(val: OCaml<T>) -> BoxRoot<T> {
        if let Some(boxroot) = unsafe { boxroot_create(val.raw) } {
            BoxRoot {
                boxroot,
                _marker: PhantomData,
            }
        } else {
            boxroot_fail();
        }
    }

    /// Gets the value stored in this root as an [`OCaml`]`<T>`.
    pub fn get<'a>(&self, cr: &'a OCamlRuntime) -> OCaml<'a, T> {
        unsafe { OCaml::new(cr, boxroot_get(self.boxroot)) }
    }

    /// Roots the OCaml value `val`, returning an [`OCamlRef`]`<T>`.
    pub fn keep<'tmp>(&'tmp mut self, val: OCaml<T>) -> OCamlRef<'tmp, T> {
        unsafe {
            if !boxroot_modify(&mut self.boxroot, val.raw) {
                boxroot_fail();
            }
            &*(boxroot_get_ref(self.boxroot) as *const OCamlCell<T>)
        }
    }
}

impl<T> Drop for BoxRoot<T> {
    fn drop(&mut self) {
        unsafe { boxroot_delete(self.boxroot) }
    }
}

impl<T> Deref for BoxRoot<T> {
    type Target = OCamlCell<T>;

    fn deref(&self) -> OCamlRef<T> {
        unsafe { &*(boxroot_get_ref(self.boxroot) as *const OCamlCell<T>) }
    }
}

#[cfg(test)]
mod boxroot_assertions {
    use super::*;
    use static_assertions::assert_not_impl_any;

    // Assert that BoxRoot<T> does not implement Send or Sync for some concrete T.
    // Using a simple type like () or i32 is sufficient.
    assert_not_impl_any!(BoxRoot<()>: Send, Sync);
    assert_not_impl_any!(BoxRoot<i32>: Send, Sync);
}
