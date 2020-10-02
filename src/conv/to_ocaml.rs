// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::{
    OCamlBytes, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml, FALSE, NONE, TRUE,
};
use crate::value::OCaml;
use crate::{
    memory::{
        alloc_bytes, alloc_cons, alloc_double, alloc_int32, alloc_int64, alloc_some, alloc_string,
        alloc_tuple, alloc_tuple_3, alloc_tuple_4, OCamlAllocResult, OCamlAllocToken,
    },
    OCamlRef,
};
use crate::{ocaml_alloc, ocaml_frame, to_ocaml};

/// Implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value.
    ///
    /// Should not be called directly, use `to_ocaml!` macro instead.
    /// If called directly, the call should be wrapped by `ocaml_alloc!`.
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

unsafe impl ToOCaml<f64> for f64 {
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<f64> {
        alloc_double(token, *self)
    }
}

unsafe impl ToOCaml<bool> for bool {
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<bool> {
        OCamlAllocResult::of(if *self { TRUE } else { FALSE })
    }
}

unsafe impl<T> ToOCaml<String> for T
where
    T: AsRef<str>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<String> {
        alloc_string(token, self.as_ref())
    }
}

unsafe impl<T> ToOCaml<OCamlBytes> for T
where
    T: AsRef<[u8]>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlBytes> {
        alloc_bytes(token, self.as_ref())
    }
}

unsafe impl<A, OCamlA> ToOCaml<Option<OCamlA>> for Option<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<Option<OCamlA>> {
        if let Some(value) = self {
            ocaml_frame!(gc, {
                let ocaml_value = &to_ocaml!(gc, value).keep(gc);
                alloc_some(token, ocaml_value)
            })
        } else {
            OCamlAllocResult::of(NONE)
        }
    }
}

unsafe impl<A, B, OCamlA, OCamlB> ToOCaml<(OCamlA, OCamlB)> for (A, B)
where
    A: ToOCaml<OCamlA>,
    B: ToOCaml<OCamlB>,
{
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<(OCamlA, OCamlB)> {
        ocaml_frame!(gc, {
            let fst = &to_ocaml!(gc, self.0).keep(gc);
            let snd = &to_ocaml!(gc, self.1).keep(gc);
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
        ocaml_frame!(gc, {
            let fst = &to_ocaml!(gc, self.0).keep(gc);
            let snd = &to_ocaml!(gc, self.1).keep(gc);
            let elt3 = &to_ocaml!(gc, self.2).keep(gc);
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
        ocaml_frame!(gc, {
            let fst = &to_ocaml!(gc, self.0).keep(gc);
            let snd = &to_ocaml!(gc, self.1).keep(gc);
            let elt3 = &to_ocaml!(gc, self.2).keep(gc);
            let elt4 = &to_ocaml!(gc, self.3).keep(gc);
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
        ocaml_frame!(gc, {
            let result_ref = &mut gc.keep(OCaml::nil());
            for elt in self.iter().rev() {
                let ov = &to_ocaml!(gc, elt).keep(gc);
                let cons = ocaml_alloc!(alloc_cons(gc, ov, result_ref));
                result_ref.set(cons);
            }
            OCamlAllocResult::of_ocaml(gc.get(result_ref))
        })
    }
}
