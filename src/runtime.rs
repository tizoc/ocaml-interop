// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use ocaml_sys::{caml_shutdown, caml_startup};
use std::{marker::PhantomData, panic::{catch_unwind, resume_unwind, UnwindSafe}};

use crate::{memory::GCFrame, value::make_ocaml, OCaml, OCamlRef};

/// OCaml runtime handle.
pub struct OCamlRuntime {}

impl OCamlRuntime {
    /// Initializes the OCaml runtime and returns a handle, that once dropped
    /// will perform the necessary cleanup.
    pub fn init() -> Self {
        OCamlRuntime::init_persistent();
        OCamlRuntime {}
    }

    /// Initializes the OCaml runtime.
    pub fn init_persistent() {
        let arg0 = "ocaml".as_ptr() as *const i8;
        let c_args = vec![arg0, core::ptr::null()];
        unsafe { caml_startup(c_args.as_ptr()) }
    }

    #[doc(hidden)]
    pub unsafe fn acquire() -> Self {
        OCamlRuntime {}
    }

    /// Release the OCaml runtime lock, call `f`, and re-acquire the OCaml runtime lock.
    ///
    /// TODO: document
    pub fn in_blocking_section<T, F>(&mut self, f: F) -> T
    where
        F: UnwindSafe + FnOnce() -> T,
    {
        unsafe { ocaml_sys::caml_enter_blocking_section() };

        let result = catch_unwind(|| f());

        unsafe { ocaml_sys::caml_leave_blocking_section() };

        match result {
            Err(err) => resume_unwind(err),
            Ok(result) => result,
        }
    }

    /// Performs the necessary cleanup and shuts down the OCaml runtime.
    pub fn shutdown_persistent() {
        unsafe { caml_shutdown() }
    }

    pub unsafe fn token(&self) -> OCamlAllocToken {
        OCamlAllocToken {
            _marker: PhantomData
        }
    }

    pub fn open_frame<'a, 'gc>(&'a self) -> GCFrame<'gc> {
        Default::default()
    }

    /// Returns the OCaml valued to which this GC tracked reference points to.
    pub fn get<'tmp, T>(&'tmp self, reference: &OCamlRef<T>) -> OCaml<'tmp, T> {
        make_ocaml(reference.cell.get())
    }
}

/// Token used by allocation functions. Used internally.
pub struct OCamlAllocToken<'a> {
    _marker: PhantomData<&'a i32>,
}

impl<'a> OCamlAllocToken<'a> {
    pub unsafe fn acquire_runtime(self) -> OCamlRuntime {
        OCamlRuntime::acquire()
    }
}
