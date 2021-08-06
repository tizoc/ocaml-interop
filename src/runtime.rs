// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_boxroot_sys::{boxroot_setup, boxroot_teardown};
use ocaml_sys::{caml_shutdown};
use std::{marker::PhantomData};

use crate::{memory::OCamlRef, value::OCaml};

/// OCaml runtime handle.
///
/// Should be initialized once at the beginning of the program
/// and the obtained handle passed around.
///
/// Once the handle is dropped, the OCaml runtime will be shutdown.
pub struct OCamlRuntime {
    _private: (),
}

impl OCamlRuntime {
    /// Recover the runtime handle.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the OCaml runtime handle should be obtained once
    /// upon initialization of the OCaml runtime and then passed around. This method exists
    /// to allow the capture of the runtime when it is already known to have been
    /// initialized.
    #[inline(always)]
    pub unsafe fn recover_handle() -> &'static mut Self {
        static mut RUNTIME: OCamlRuntime = OCamlRuntime { _private: () };
        &mut RUNTIME
    }

    /// Create a new runtime handle.
    ///
    /// This method is used internally, do not use directly in code, only when writing tests.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the OCaml runtime handle should be obtained once
    /// upon initialization of the OCaml runtime and then passed around. This method exists
    /// to allow the creation of a runtime after `caml_startup` has been called and the
    /// runtime is initialized. The OCaml runtime will be shutdown when this value is
    /// dropped.
    #[inline(always)]
    pub unsafe fn create() -> Self {
        OCamlRuntime { _private: () }
    }

    /// Release the OCaml runtime lock, call `f`, and re-acquire the OCaml runtime lock.
    pub fn releasing_runtime<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        OCamlBlockingSection::new().perform(f)
    }

    /// Returns the OCaml valued to which this GC tracked reference points to.
    pub fn get<'tmp, T>(&'tmp self, reference: OCamlRef<T>) -> OCaml<'tmp, T> {
        OCaml {
            _marker: PhantomData,
            raw: unsafe { reference.get_raw() },
        }
    }
}

impl Drop for OCamlRuntime {
    fn drop(&mut self) {
        unsafe {
            boxroot_teardown();
            caml_shutdown();
        }
    }
}

struct OCamlBlockingSection {}

impl OCamlBlockingSection {
    fn new() -> Self {
        Self {}
    }

    fn perform<T, F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        unsafe { ocaml_sys::caml_enter_blocking_section() };
        f()
    }
}

impl Drop for OCamlBlockingSection {
    fn drop(&mut self) {
        unsafe { ocaml_sys::caml_leave_blocking_section() };
    }
}

// For initializing from an OCaml-driven program

#[no_mangle]
extern "C" fn ocaml_interop_setup(_unit: crate::RawOCaml) -> crate::RawOCaml {
    unsafe { boxroot_setup() };
    ocaml_sys::UNIT
}

#[no_mangle]
extern "C" fn ocaml_interop_teardown(_unit: crate::RawOCaml) -> crate::RawOCaml {
    unsafe { boxroot_teardown() };
    ocaml_sys::UNIT
}
