// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::error::{OCamlError, OCamlException};
use crate::memory::{GCResult, GCToken};
use crate::mlvalues::tag;
use crate::mlvalues::{extract_exception, is_exception_result, tag_val, RawOCaml};
use crate::value::OCaml;

extern "C" {
    fn caml_named_value(name: *const i8) -> *const RawOCaml;

    // fn caml_callback(closure: RawOCaml, arg1: RawOCaml) -> RawOCaml;
    // fn caml_callback2(closure: RawOCaml, arg1: RawOCaml, arg2: RawOCaml) -> RawOCaml;
    // fn caml_callback3(
    //     closure: RawOCaml,
    //     arg1: RawOCaml,
    //     arg2: RawOCaml,
    //     arg3: RawOCaml,
    // ) -> RawOCaml;
    // fn caml_callbackN(closure: RawOCaml, narg: usize, args: *mut RawOCaml) -> RawOCaml;

    fn caml_callback_exn(closure: RawOCaml, arg1: RawOCaml) -> RawOCaml;
    fn caml_callback2_exn(closure: RawOCaml, arg1: RawOCaml, arg2: RawOCaml) -> RawOCaml;
    fn caml_callback3_exn(
        closure: RawOCaml,
        arg1: RawOCaml,
        arg2: RawOCaml,
        arg3: RawOCaml,
    ) -> RawOCaml;
    fn caml_callbackN_exn(closure: RawOCaml, narg: usize, args: *mut RawOCaml) -> RawOCaml;
}

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
pub type OCamlResult<T> = Result<GCResult<T>, OCamlError>;

/// OCaml function that accepts one argument.
pub type OCamlFn1<A, Ret> = unsafe fn(GCToken, OCaml<A>) -> OCamlResult<Ret>;
/// OCaml function that accepts two arguments.
pub type OCamlFn2<A, B, Ret> = unsafe fn(GCToken, OCaml<A>, OCaml<B>) -> OCamlResult<Ret>;
/// OCaml function that accepts three arguments.
pub type OCamlFn3<A, B, C, Ret> = unsafe fn(GCToken, OCaml<A>, OCaml<B>, OCaml<C>) -> OCamlResult<Ret>;
/// OCaml function that accepts four arguments.
pub type OCamlFn4<A, B, C, D, Ret> = unsafe fn(GCToken, OCaml<A>, OCaml<B>, OCaml<C>, OCaml<D>) -> OCamlResult<Ret>;
/// OCaml function that accepts five arguments.
pub type OCamlFn5<A, B, C, D, E, Ret> = unsafe fn(GCToken, OCaml<A>, OCaml<B>, OCaml<C>, OCaml<D>, OCaml<E>) -> OCamlResult<Ret>;

impl OCamlClosure {
    pub fn named(name: &str) -> Option<OCamlClosure> {
        get_named(name).map(OCamlClosure)
    }

    pub fn call<T, R>(&self, _token: GCToken, arg: OCaml<T>) -> OCamlResult<R> {
        let result = unsafe { caml_callback_exn(*self.0, arg.raw()) };
        self.handle_result(result)
    }

    pub fn call2<T, U, R>(
        &self,
        _token: GCToken,
        arg1: OCaml<T>,
        arg2: OCaml<U>,
    ) -> OCamlResult<R> {
        let result = unsafe { caml_callback2_exn(*self.0, arg1.raw(), arg2.raw()) };
        self.handle_result(result)
    }

    pub fn call3<T, U, V, R>(
        &self,
        _token: GCToken,
        arg1: OCaml<T>,
        arg2: OCaml<U>,
        arg3: OCaml<V>,
    ) -> OCamlResult<R> {
        let result = unsafe { caml_callback3_exn(*self.0, arg1.raw(), arg2.raw(), arg3.raw()) };
        self.handle_result(result)
    }

    pub fn call_n<R>(&self, _token: GCToken, args: &mut [RawOCaml]) -> OCamlResult<R> {
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
            let gv = GCResult::of(result);
            Ok(gv)
        }
    }
}
