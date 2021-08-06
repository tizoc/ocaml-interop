// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT
use std::sync::Once;
use ocaml_sys::{caml_startup};
use ocaml_interop::OCamlRuntime;

/// Initializes the OCaml runtime and returns an OCaml runtime handle.
///
/// Once the handle is dropped, the OCaml runtime will be shutdown.
pub fn init() -> OCamlRuntime {
    init_persistent();
    unsafe { ocaml_interop::OCamlRuntime::create() }
}

/// Initializes the OCaml runtime.
///
/// After the first invocation, this method does nothing.
pub fn init_persistent() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let arg0 = "ocaml\0".as_ptr() as *const ocaml_sys::Char;
        let c_args = vec![arg0, core::ptr::null()];
        unsafe {
            caml_startup(c_args.as_ptr());
        }
    })
}
