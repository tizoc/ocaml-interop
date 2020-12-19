// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::error::OCamlException;
use crate::mlvalues::tag;
use crate::mlvalues::{extract_exception, is_exception_result, tag_val, RawOCaml};
use crate::value::OCaml;
use crate::{OCamlRooted, OCamlRuntime};
use ocaml_sys::{
    caml_callback2_exn, caml_callback3_exn, caml_callbackN_exn, caml_callback_exn, caml_named_value,
};

#[derive(Copy, Clone)]
pub struct OCamlClosure(*const RawOCaml);

unsafe impl Sync for OCamlClosure {}

fn get_named(name: &str) -> Option<*const RawOCaml> {
    unsafe {
        let s = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };
        let named = caml_named_value(s.as_ptr());
        if named.is_null() {
            return None;
        }

        if tag_val(*named) != tag::CLOSURE {
            return None;
        }

        Some(named)
    }
}

/// OCaml function that accepts one argument.
pub type OCamlFn1<'a, A, Ret> = unsafe fn(&'a mut OCamlRuntime, OCaml<A>) -> OCaml<'a, Ret>;
/// OCaml function that accepts two arguments.
pub type OCamlFn2<'a, A, B, Ret> =
    unsafe fn(&'a mut OCamlRuntime, OCaml<A>, OCaml<B>) -> OCaml<'a, Ret>;
/// OCaml function that accepts three arguments.
pub type OCamlFn3<'a, A, B, C, Ret> =
    unsafe fn(&'a mut OCamlRuntime, OCaml<A>, OCaml<B>, OCaml<C>) -> OCaml<'a, Ret>;
/// OCaml function that accepts four arguments.
pub type OCamlFn4<'a, A, B, C, D, Ret> =
    unsafe fn(&'a mut OCamlRuntime, OCaml<A>, OCaml<B>, OCaml<C>, OCaml<D>) -> OCaml<'a, Ret>;
/// OCaml function that accepts five arguments.
pub type OCamlFn5<'a, A, B, C, D, E, Ret> = unsafe fn(
    &'a mut OCamlRuntime,
    OCaml<A>,
    OCaml<B>,
    OCaml<C>,
    OCaml<D>,
    OCaml<E>,
) -> OCaml<'a, Ret>;

impl OCamlClosure {
    pub fn named(name: &str) -> Option<OCamlClosure> {
        get_named(name).map(OCamlClosure)
    }

    pub fn call<'a, T, R>(&self, cr: &'a mut OCamlRuntime, arg: &OCamlRooted<T>) -> OCaml<'a, R> {
        let result = unsafe { caml_callback_exn(*self.0, arg.get_raw()) };
        self.handle_call_result(cr, result)
    }

    pub fn call2<'a, T, U, R>(
        &self,
        cr: &'a mut OCamlRuntime,
        arg1: &OCamlRooted<T>,
        arg2: &OCamlRooted<U>,
    ) -> OCaml<'a, R> {
        let result = unsafe { caml_callback2_exn(*self.0, arg1.get_raw(), arg2.get_raw()) };
        self.handle_call_result(cr, result)
    }

    pub fn call3<'a, T, U, V, R>(
        &self,
        cr: &'a mut OCamlRuntime,
        arg1: &OCamlRooted<T>,
        arg2: &OCamlRooted<U>,
        arg3: &OCamlRooted<V>,
    ) -> OCaml<'a, R> {
        let result =
            unsafe { caml_callback3_exn(*self.0, arg1.get_raw(), arg2.get_raw(), arg3.get_raw()) };
        self.handle_call_result(cr, result)
    }

    pub fn call_n<'a, R>(&self, cr: &'a mut OCamlRuntime, args: &mut [RawOCaml]) -> OCaml<'a, R> {
        let len = args.len();
        let result = unsafe { caml_callbackN_exn(*self.0, len, args.as_mut_ptr()) };
        self.handle_call_result(cr, result)
    }

    #[inline]
    fn handle_call_result<'a, R>(
        &self,
        cr: &'a mut OCamlRuntime,
        result: RawOCaml,
    ) -> OCaml<'a, R> {
        if is_exception_result(result) {
            let ex = OCamlException::of(extract_exception(result));
            panic!("OCaml exception: {:?}", ex)
        } else {
            unsafe { OCaml::new(cr, result) }
        }
    }
}
