// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_boxroot_sys::boxroot_teardown;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

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
    /// Initializes the OCaml runtime and returns an OCaml runtime handle.
    ///
    /// Once the handle is dropped, the OCaml runtime will be shutdown.
    pub fn init() -> Self {
        Self::init_persistent();
        Self { _private: () }
    }

    /// Initializes the OCaml runtime.
    ///
    /// After the first invocation, this method does nothing.
    pub fn init_persistent() {
        #[cfg(not(feature = "no-caml-startup"))]
        {
            static INIT: std::sync::Once = std::sync::Once::new();

            INIT.call_once(|| {
                let arg0 = "ocaml\0".as_ptr() as *const ocaml_sys::Char;
                let c_args = [arg0, core::ptr::null()];
                unsafe {
                    ocaml_sys::caml_startup(c_args.as_ptr());
                    ocaml_boxroot_sys::boxroot_setup_systhreads();
                }
            })
        }
        #[cfg(feature = "no-caml-startup")]
        panic!("Rust code that is called from an OCaml program should not try to initialize the runtime.");
    }

    #[inline(always)]
    pub(crate) unsafe fn recover_handle_mut() -> &'static mut Self {
        static mut RUNTIME: OCamlRuntime = OCamlRuntime { _private: () };
        &mut RUNTIME
    }

    #[inline(always)]
    pub(crate) unsafe fn recover_handle() -> &'static Self {
        Self::recover_handle_mut()
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

    pub fn acquire_lock() -> OCamlDomainLock {
        OCamlDomainLock::new()
    }
}

impl Drop for OCamlRuntime {
    fn drop(&mut self) {
        unsafe {
            boxroot_teardown();
            ocaml_sys::caml_shutdown();
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

pub struct OCamlDomainLock {
    _private: (),
}

extern "C" {
    pub fn caml_c_thread_register() -> isize;
    pub fn caml_c_thread_unregister() -> isize;
}

impl OCamlDomainLock {
    #[inline(always)]
    fn new() -> Self {
        unsafe {
            caml_c_thread_register();
            ocaml_sys::caml_leave_blocking_section();
        };
        Self { _private: () }
    }

    #[inline(always)]
    pub(crate) fn recover_handle<'a>(&self) -> &'a OCamlRuntime {
        unsafe { OCamlRuntime::recover_handle() }
    }

    #[inline(always)]
    pub(crate) fn recover_handle_mut<'a>(&self) -> &'a mut OCamlRuntime {
        unsafe { OCamlRuntime::recover_handle_mut() }
    }
}

impl Drop for OCamlDomainLock {
    fn drop(&mut self) {
        unsafe {
            ocaml_sys::caml_enter_blocking_section();
            caml_c_thread_unregister();
        };
    }
}

impl Deref for OCamlDomainLock {
    type Target = OCamlRuntime;

    fn deref(&self) -> &OCamlRuntime {
        self.recover_handle()
    }
}

impl DerefMut for OCamlDomainLock {
    fn deref_mut(&mut self) -> &mut OCamlRuntime {
        self.recover_handle_mut()
    }
}

// For initializing from an OCaml-driven program

#[no_mangle]
extern "C" fn ocaml_interop_setup(_unit: crate::RawOCaml) -> crate::RawOCaml {
    ocaml_sys::UNIT
}

#[no_mangle]
extern "C" fn ocaml_interop_teardown(_unit: crate::RawOCaml) -> crate::RawOCaml {
    unsafe { boxroot_teardown() };
    ocaml_sys::UNIT
}
