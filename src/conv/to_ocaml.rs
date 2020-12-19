// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use core::str;
use ocaml_sys::{caml_alloc, store_field};

use crate::{
    memory::{
        alloc_bytes, alloc_cons, alloc_double, alloc_int32, alloc_int64, alloc_some, alloc_string,
        alloc_tuple, alloc_tuple_3, alloc_tuple_4, OCamlAllocResult, OCamlRooted,
    },
    mlvalues::{
        tag, OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml, FALSE,
        NONE, TRUE,
    },
    ocaml_alloc, ocaml_frame,
    runtime::OCamlAllocToken,
    to_ocaml,
    value::OCaml,
};

/// Implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value.
    ///
    /// Should not be called directly, use [`to_ocaml!`] macro instead.
    /// If called directly, the call should be wrapped by [`ocaml_alloc!`].
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<T>;
}

unsafe impl<'a, T> ToOCaml<T> for OCamlRooted<'a, T> {
    fn to_ocaml(&self, _token: OCamlAllocToken) -> OCamlAllocResult<T> {
        OCamlAllocResult::of(unsafe { self.get_raw() })
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
        let cr = unsafe { &mut token.recover_runtime_handle() };
        if let Some(value) = self {
            ocaml_frame!(cr, (root), {
                let ocaml_value = to_ocaml!(cr, value, root);
                alloc_some(unsafe { cr.token() }, &ocaml_value)
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
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<Result<OCamlA, OCamlErr>> {
        let cr = unsafe { &mut token.recover_runtime_handle() };
        match self {
            Ok(value) => ocaml_frame!(cr, (root), {
                let ocaml_value = to_ocaml!(cr, value, root);
                let ocaml_ok = unsafe { caml_alloc(1, tag::TAG_OK) };
                unsafe { store_field(ocaml_ok, 0, ocaml_value.get_raw()) };
                OCamlAllocResult::of(ocaml_ok)
            }),
            Err(error) => ocaml_frame!(cr, (root), {
                let ocaml_error = to_ocaml!(cr, error, root);
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
        let cr = unsafe { &mut token.recover_runtime_handle() };
        ocaml_frame!(cr, (fst, snd), {
            let fst = to_ocaml!(cr, self.0, fst);
            let snd = to_ocaml!(cr, self.1, snd);
            alloc_tuple(unsafe { cr.token() }, &fst, &snd)
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
        let cr = unsafe { &mut token.recover_runtime_handle() };
        ocaml_frame!(cr, (fst, snd, elt3), {
            let fst = to_ocaml!(cr, self.0, fst);
            let snd = to_ocaml!(cr, self.1, snd);
            let elt3 = to_ocaml!(cr, self.2, elt3);
            alloc_tuple_3(unsafe { cr.token() }, &fst, &snd, &elt3)
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
        let cr = unsafe { &mut token.recover_runtime_handle() };
        ocaml_frame!(cr, (fst, snd, elt3, elt4), {
            let fst = to_ocaml!(cr, self.0, fst);
            let snd = to_ocaml!(cr, self.1, snd);
            let elt3 = to_ocaml!(cr, self.2, elt3);
            let elt4 = to_ocaml!(cr, self.3, elt4);
            alloc_tuple_4(unsafe { cr.token() }, &fst, &snd, &elt3, &elt4)
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
    fn to_ocaml(&self, token: OCamlAllocToken) -> OCamlAllocResult<OCamlList<OCamlA>> {
        let cr = unsafe { &mut token.recover_runtime_handle() };
        ocaml_frame!(cr, (result_root, ov_root), {
            let mut result_root = result_root.keep(OCaml::nil());
            for elt in self.iter().rev() {
                let ov = to_ocaml!(cr, elt, ov_root);
                let cons = ocaml_alloc!(alloc_cons(cr, &ov, &result_root));
                result_root.set(cons);
            }
            OCamlAllocResult::of_ocaml(cr.get(&result_root))
        })
    }
}
