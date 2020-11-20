// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::memory::OCamlAllocResult;
use crate::mlvalues::tag;
use crate::mlvalues::{extract_exception, is_exception_result, tag_val, RawOCaml};
use crate::value::OCaml;
use crate::{
    error::{OCamlError, OCamlException},
    OCamlAllocToken,
};
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

/// The result of calls to OCaml functions. Can be a value or an error.
pub type OCamlResult<T> = Result<OCamlAllocResult<T>, OCamlError>;

/// OCaml function that accepts one argument.
pub type OCamlFn1<A, Ret> = unsafe fn(OCamlAllocToken, OCaml<A>) -> OCamlResult<Ret>;
/// OCaml function that accepts two arguments.
pub type OCamlFn2<A, B, Ret> = unsafe fn(OCamlAllocToken, OCaml<A>, OCaml<B>) -> OCamlResult<Ret>;
/// OCaml function that accepts three arguments.
pub type OCamlFn3<A, B, C, Ret> =
    unsafe fn(OCamlAllocToken, OCaml<A>, OCaml<B>, OCaml<C>) -> OCamlResult<Ret>;
/// OCaml function that accepts four arguments.
pub type OCamlFn4<A, B, C, D, Ret> =
    unsafe fn(OCamlAllocToken, OCaml<A>, OCaml<B>, OCaml<C>, OCaml<D>) -> OCamlResult<Ret>;
/// OCaml function that accepts five arguments.
pub type OCamlFn5<A, B, C, D, E, Ret> = unsafe fn(
    OCamlAllocToken,
    OCaml<A>,
    OCaml<B>,
    OCaml<C>,
    OCaml<D>,
    OCaml<E>,
) -> OCamlResult<Ret>;

impl OCamlClosure {
    pub fn named(name: &str) -> Option<OCamlClosure> {
        get_named(name).map(OCamlClosure)
    }

    pub fn call<T, R>(&self, _token: OCamlAllocToken, arg: OCaml<T>) -> OCamlResult<R> {
        let result = unsafe { caml_callback_exn(*self.0, arg.raw()) };
        self.handle_result(result)
    }

    pub fn call2<T, U, R>(
        &self,
        _token: OCamlAllocToken,
        arg1: OCaml<T>,
        arg2: OCaml<U>,
    ) -> OCamlResult<R> {
        let result = unsafe { caml_callback2_exn(*self.0, arg1.raw(), arg2.raw()) };
        self.handle_result(result)
    }

    pub fn call3<T, U, V, R>(
        &self,
        _token: OCamlAllocToken,
        arg1: OCaml<T>,
        arg2: OCaml<U>,
        arg3: OCaml<V>,
    ) -> OCamlResult<R> {
        let result = unsafe { caml_callback3_exn(*self.0, arg1.raw(), arg2.raw(), arg3.raw()) };
        self.handle_result(result)
    }

    pub fn call_n<R>(&self, _token: OCamlAllocToken, args: &mut [RawOCaml]) -> OCamlResult<R> {
        let len = args.len();
        let result = unsafe { caml_callbackN_exn(*self.0, len, args.as_mut_ptr()) };
        self.handle_result(result)
    }

    #[inline]
    fn handle_result<R>(self, result: RawOCaml) -> OCamlResult<R> {
        if is_exception_result(result) {
            let ex = extract_exception(result);
            Err(OCamlError::Exception(OCamlException::of(ex)))
        } else {
            let gv = OCamlAllocResult::of(result);
            Ok(gv)
        }
    }
}
