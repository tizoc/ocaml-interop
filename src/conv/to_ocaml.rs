// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use ocaml_sys::{caml_alloc, store_field};

use crate::{
    memory::{
        alloc_bytes, alloc_cons, alloc_double, alloc_int32, alloc_int64, alloc_some, alloc_string,
        alloc_tuple, alloc_tuple_3, alloc_tuple_4, OCamlAllocResult, OCamlAllocToken,
    },
    OCamlRef,
};
use crate::{mlvalues::tag, value::OCaml};
use crate::{
    mlvalues::{
        OCamlBytes, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml, FALSE, NONE, TRUE,
    },
    OCamlFloat,
};
use crate::{ocaml_alloc, ocaml_frame, to_ocaml};
use core::str;

/// Implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value.
    ///
    /// Should not be called directly, use [`to_ocaml!`] macro instead.
    /// If called directly, the call should be wrapped by [`ocaml_alloc!`].
    fn to_ocaml(&self, gc: OCamlAllocToken) -> OCamlAllocResult<T>;
}

unsafe impl<'a, T> ToOCaml<T> for OCamlRef<'a, T> {
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<T> {
        OCamlAllocResult::of(self.get_raw())
    }
}

unsafe impl ToOCaml<OCamlInt> for i64 {
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<OCamlInt> {
        OCamlAllocResult::of(((self << 1) | 1) as RawOCaml)
    }
}

unsafe impl ToOCaml<OCamlInt> for i32 {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlInt> {
        (*self as i64).to_ocaml(token)
    }
}

unsafe impl ToOCaml<OCamlInt32> for i32 {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlInt32> {
        alloc_int32(token, *self)
    }
}

unsafe impl ToOCaml<OCamlInt64> for i64 {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlInt64> {
        alloc_int64(token, *self)
    }
}

unsafe impl ToOCaml<OCamlFloat> for f64 {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlFloat> {
        alloc_double(token, *self)
    }
}

unsafe impl ToOCaml<bool> for bool {
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<bool> {
        OCamlAllocResult::of(if *self { TRUE } else { FALSE })
    }
}

unsafe impl ToOCaml<String> for &str {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<String> {
        alloc_string(token, self)
    }
}

unsafe impl ToOCaml<OCamlBytes> for &str {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlBytes> {
        alloc_bytes(token, self.as_bytes())
    }
}

unsafe impl ToOCaml<OCamlBytes> for &[u8] {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlBytes> {
        alloc_bytes(token, self)
    }
}

unsafe impl ToOCaml<String> for &[u8] {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<String> {
        alloc_string(token, unsafe { str::from_utf8_unchecked(self) })
    }
}

unsafe impl ToOCaml<String> for String {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<String> {
        self.as_str().to_ocaml(token)
    }
}

unsafe impl ToOCaml<OCamlBytes> for String {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlBytes> {
        self.as_str().to_ocaml(token)
    }
}

unsafe impl ToOCaml<String> for Vec<u8> {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<String> {
        self.as_slice().to_ocaml(token)
    }
}

unsafe impl ToOCaml<OCamlBytes> for Vec<u8> {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlBytes> {
        self.as_slice().to_ocaml(token)
    }
}

unsafe impl<A, OCamlA> ToOCaml<OCamlA> for Box<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlA> {
        self.as_ref().to_ocaml(token)
    }
}

unsafe impl<A, OCamlA> ToOCaml<Option<OCamlA>> for Option<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<Option<OCamlA>> {
        if let Some(value) = self {
            ocaml_frame!(gc(ocaml_value), {
                let ocaml_value = &ocaml_value.keep(to_ocaml!(gc, value));
                alloc_some(token, ocaml_value)
            })
        } else {
            OCamlAllocResult::of(NONE)
        }
    }
}

unsafe impl<A, OCamlA, Err, OCamlErr> ToOCaml<Result<OCamlA, OCamlErr>> for Result<A, Err>
where
    A: ToOCaml<OCamlA>,
    Err: ToOCaml<OCamlErr>,
{
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<Result<OCamlA, OCamlErr>> {
        match self {
            Ok(value) => ocaml_frame!(gc(ocaml_value), {
                let ocaml_value = to_ocaml!(gc, value, ocaml_value);
                let ocaml_ok = unsafe { caml_alloc(1, tag::TAG_OK) };
                unsafe { store_field(ocaml_ok, 0, ocaml_value.get_raw()) };
                OCamlAllocResult::of(ocaml_ok)
            }),
            Err(error) => ocaml_frame!(gc(ocaml_error), {
                let ocaml_error = to_ocaml!(gc, error, ocaml_error);
                let ocaml_err = unsafe { caml_alloc(1, tag::TAG_ERROR) };
                unsafe { store_field(ocaml_err, 0, ocaml_error.get_raw()) };
                OCamlAllocResult::of(ocaml_err)
            }),
        }
    }
}

unsafe impl<A, B, OCamlA, OCamlB> ToOCaml<(OCamlA, OCamlB)> for (A, B)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<(OCamlA, OCamlB)> {
        ocaml_frame!(gc(fst, snd), {
            let fst = &fst.keep(to_ocaml!(gc, self.0));
            let snd = &snd.keep(to_ocaml!(gc, self.1));
            alloc_tuple(token, fst, snd)
        })
    }
}

unsafe impl<A, B, C, OCamlA, OCamlB, OCamlC> ToOCaml<(OCamlA, OCamlB, OCamlC)> for (A, B, C)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
    C: ToOCaml<OCamlC>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<(OCamlA, OCamlB, OCamlC)> {
        ocaml_frame!(gc(fst, snd, elt3), {
            let fst = &fst.keep(to_ocaml!(gc, self.0));
            let snd = &snd.keep(to_ocaml!(gc, self.1));
            let elt3 = &elt3.keep(to_ocaml!(gc, self.2));
            alloc_tuple_3(token, fst, snd, elt3)
        })
    }
}

unsafe impl<A, B, C, D, OCamlA, OCamlB, OCamlC, OCamlD> ToOCaml<(OCamlA, OCamlB, OCamlC, OCamlD)>
    for (A, B, C, D)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
    C: ToOCaml<OCamlC>,
    D: ToOCaml<OCamlD>,
{
    fn to_ocaml(
        &self,
        token: OCamlAllocToken,
    ) -> OCamlAllocResult<(OCamlA, OCamlB, OCamlC, OCamlD)> {
        ocaml_frame!(gc(fst, snd, elt3, elt4), {
            let fst = &fst.keep(to_ocaml!(gc, self.0));
            let snd = &snd.keep(to_ocaml!(gc, self.1));
            let elt3 = &elt3.keep(to_ocaml!(gc, self.2));
            let elt4 = &elt4.keep(to_ocaml!(gc, self.3));
            alloc_tuple_4(token, fst, snd, elt3, elt4)
        })
    }
}

unsafe impl<A, OCamlA> ToOCaml<OCamlList<OCamlA>> for Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlList<OCamlA>> {
        (&self).to_ocaml(token)
    }
}

unsafe impl<A, OCamlA> ToOCaml<OCamlList<OCamlA>> for &Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<OCamlList<OCamlA>> {
        ocaml_frame!(gc(result_ref, ov_ref), {
            let mut result_ref = result_ref.keep(OCaml::nil());
            for elt in self.iter().rev() {
                let ov = &ov_ref.keep(to_ocaml!(gc, elt));
                let cons = ocaml_alloc!(alloc_cons(gc, ov, &result_ref));
                result_ref.set(cons);
            }
            OCamlAllocResult::of_ocaml(gc.get(&result_ref))
        })
    }
}
