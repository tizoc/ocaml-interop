// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use ocaml_sys::{caml_shutdown, caml_startup};
use std::marker::PhantomData;

use crate::{memory::GCFrame, memory::OCamlRef, value::OCaml};

/// OCaml runtime handle.
pub struct OCamlRuntime {
    _private: (),
}

impl OCamlRuntime {
    /// Initializes the OCaml runtime and returns an OCaml runtime handle.
    pub fn init() -> Self {
        OCamlRuntime::init_persistent();
        OCamlRuntime { _private: () }
    }

    /// Initializes the OCaml runtime.
    pub fn init_persistent() {
        let arg0 = "ocaml\0".as_ptr() as *const i8;
        let c_args = vec![arg0, core::ptr::null()];
        unsafe { caml_startup(c_args.as_ptr()) }
    }

    /// Recover the runtime handle.
    ///
    /// This method is used internally, do not use directly in code, only when writing tests.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the OCaml runtime handle should be obtained once
    /// upon initialization of the OCaml runtime and then passed around. This method exists
    /// only to ease the authoring of tests.
    pub unsafe fn recover_handle() -> &'static mut Self {
        static mut RUNTIME: OCamlRuntime = OCamlRuntime { _private: () };
        &mut RUNTIME
    }

    /// Release the OCaml runtime lock, call `f`, and re-acquire the OCaml runtime lock.
    pub fn releasing_runtime<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        OCamlBlockingSection::new().perform(f)
    }

    #[doc(hidden)]
    pub fn open_frame<'a, 'gc>(&'a self) -> GCFrame<'gc> {
        Default::default()
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
        unsafe { caml_shutdown() }
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
