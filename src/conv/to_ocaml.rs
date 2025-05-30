// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use core::{borrow::Borrow, str};

use ocaml_sys::{caml_alloc_float_array, caml_sys_store_double_field};

use crate::{
    internal::{caml_alloc, store_field},
    memory::{
        alloc_bigarray1, alloc_bytes, alloc_cons, alloc_double, alloc_error, alloc_int32,
        alloc_int64, alloc_ok, alloc_some, alloc_string, alloc_tuple, store_raw_field_at, OCamlRef,
    },
    mlvalues::{
        bigarray::{Array1, BigarrayElt},
        OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, RawOCaml, FALSE, NONE,
        TRUE,
    },
    runtime::OCamlRuntime,
    value::OCaml,
    BoxRoot, OCamlFloatArray, OCamlUniformArray,
};

/// Implements conversion from Rust values into OCaml values.
///
/// # Safety
///
/// Implementing this trait involves unsafe code that interacts with the OCaml runtime.
/// Implementors must ensure the following to uphold Rust's safety guarantees:
///
/// - **Memory Safety**: Returned OCaml values must be valid and correctly represent
///   the memory layout expected by the OCaml runtime. Any misrepresentation can lead
///   to undefined behavior, potentially causing segmentation faults or data corruption.
///
/// - **Handling of OCaml Exceptions**: If the OCaml code can raise exceptions, the implementor
///   must ensure these are appropriately handled. Uncaught OCaml exceptions should not be allowed
///   to propagate into the Rust code, as they are not compatible with Rust's error handling mechanisms.
///
/// Implementors of this trait must have a deep understanding of both Rust's and OCaml's
/// memory models, garbage collection, and runtime behaviors to ensure safe interoperability.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value. Return an already rooted value as [`BoxRoot`]`<T>`.
    fn to_boxroot(&self, cr: &mut OCamlRuntime) -> BoxRoot<T> {
        BoxRoot::new(self.to_ocaml(cr))
    }

    /// Convert to OCaml value.
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T>;
}

unsafe impl<T> ToOCaml<T> for OCamlRef<'_, T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T> {
        unsafe { OCaml::new(cr, self.get_raw()) }
    }
}

unsafe impl<T> ToOCaml<T> for BoxRoot<T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T> {
        self.get(cr)
    }
}

unsafe impl<T> ToOCaml<T> for OCaml<'_, T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, T> {
        unsafe { OCaml::new(cr, self.raw()) }
    }
}

unsafe impl ToOCaml<()> for () {
    fn to_ocaml(&self, _cr: &mut OCamlRuntime) -> OCaml<'static, ()> {
        OCaml::unit()
    }
}

unsafe impl ToOCaml<OCamlInt> for i64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        unsafe { OCaml::new(cr, ((self << 1) | 1i64) as RawOCaml) }
    }
}

unsafe impl ToOCaml<OCamlInt> for i32 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        (*self as i64).to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlInt32> for i32 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt32> {
        alloc_int32(cr, *self)
    }
}

unsafe impl ToOCaml<OCamlInt64> for i64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt64> {
        alloc_int64(cr, *self)
    }
}

unsafe impl ToOCaml<OCamlFloat> for f64 {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlFloat> {
        alloc_double(cr, *self)
    }
}

unsafe impl ToOCaml<bool> for bool {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, bool> {
        unsafe { OCaml::new(cr, if *self { TRUE } else { FALSE }) }
    }
}

// TODO: figure out how to implement all this without so much duplication
// it is not as simple as implementing for Borrow<str/[u8]> because
// of the Box<T> implementation bellow, which causes a trait implementation
// conflict.

unsafe impl ToOCaml<String> for &str {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        alloc_string(cr, self)
    }
}

unsafe impl ToOCaml<OCamlBytes> for &str {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        alloc_bytes(cr, self.as_bytes())
    }
}

unsafe impl ToOCaml<OCamlBytes> for &[u8] {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        alloc_bytes(cr, self)
    }
}

unsafe impl ToOCaml<String> for &[u8] {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        alloc_string(cr, unsafe { str::from_utf8_unchecked(self) })
    }
}

unsafe impl ToOCaml<String> for String {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        self.as_str().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlBytes> for String {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        self.as_str().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<String> for Vec<u8> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        self.as_slice().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlBytes> for Vec<u8> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        self.as_slice().to_ocaml(cr)
    }
}

unsafe impl ToOCaml<OCamlBytes> for Box<[u8]> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlBytes> {
        let slice: &[u8] = self;
        slice.to_ocaml(cr)
    }
}

unsafe impl ToOCaml<Array1<u8>> for Box<[u8]> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Array1<u8>> {
        let slice: &[u8] = self;
        slice.to_ocaml(cr)
    }
}

unsafe impl<A, OCamlA> ToOCaml<OCamlA> for Box<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlA> {
        self.as_ref().to_ocaml(cr)
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<Option<OCamlA>> for Option<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Option<OCamlA>> {
        if let Some(value) = self {
            let ocaml_value = value.to_boxroot(cr);
            alloc_some(cr, &ocaml_value)
        } else {
            unsafe { OCaml::new(cr, NONE) }
        }
    }
}

unsafe impl<A, OCamlA: 'static, Err, OCamlErr: 'static> ToOCaml<Result<OCamlA, OCamlErr>>
    for Result<A, Err>
where
    A: ToOCaml<OCamlA>,
    Err: ToOCaml<OCamlErr>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Result<OCamlA, OCamlErr>> {
        match self {
            Ok(value) => {
                let ocaml_value = value.to_boxroot(cr);
                alloc_ok(cr, &ocaml_value)
            }
            Err(error) => {
                let ocaml_error = error.to_boxroot(cr);
                alloc_error(cr, &ocaml_error)
            }
        }
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<OCamlList<OCamlA>> for Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlList<OCamlA>> {
        let mut result = BoxRoot::new(OCaml::nil(cr));
        for elt in self.iter().rev() {
            let ov = elt.to_boxroot(cr);
            let cons = alloc_cons(cr, &ov, &result);
            result.keep(cons);
        }
        cr.get(&result)
    }
}

unsafe impl<A, OCamlA: 'static> ToOCaml<OCamlUniformArray<OCamlA>> for Vec<A>
where
    A: ToOCaml<OCamlA>,
{
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlUniformArray<OCamlA>> {
        let result = BoxRoot::new(unsafe { OCaml::new(cr, caml_alloc(self.len(), 0)) });

        for (i, elt) in self.iter().enumerate() {
            let ov = elt.to_ocaml(cr);
            unsafe { store_field(result.get_raw(), i, ov.raw()) };
        }

        result.get(cr)
    }
}

unsafe impl ToOCaml<OCamlFloatArray> for Vec<f64> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlFloatArray> {
        let result = unsafe { OCaml::new(cr, caml_alloc_float_array(self.len())) };

        for (i, elt) in self.iter().enumerate() {
            unsafe { caml_sys_store_double_field(result.raw(), i, *elt) };
        }

        result
    }
}

// Tuples

macro_rules! tuple_to_ocaml {
    ($($n:tt: $t:ident => $ot:ident),+) => {
        unsafe impl<$($t),+, $($ot: 'static),+> ToOCaml<($($ot),+)> for ($($t),+)
        where
            $($t: ToOCaml<$ot>),+
        {
            fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, ($($ot),+)> {
                let len = $crate::count_fields!($($t)*);

                    let ocaml_tuple: BoxRoot<($($ot),+)> = BoxRoot::new(unsafe { alloc_tuple(cr, len) });
                    $(
                        unsafe {
                            let field_val = self.$n.to_ocaml(cr).get_raw();
                            store_raw_field_at(cr, &ocaml_tuple, $n, field_val);
                        }
                    )+

                    cr.get(&ocaml_tuple)
            }
        }
    };
}

tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE,
    5: F => OCamlF);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE,
    5: F => OCamlF,
    6: G => OCamlG);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE,
    5: F => OCamlF,
    6: G => OCamlG,
    7: H => OCamlH);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE,
    5: F => OCamlF,
    6: G => OCamlG,
    7: H => OCamlH,
    8: I => OCamlI);
tuple_to_ocaml!(
    0: A => OCamlA,
    1: B => OCamlB,
    2: C => OCamlC,
    3: D => OCamlD,
    4: E => OCamlE,
    5: F => OCamlF,
    6: G => OCamlG,
    7: H => OCamlH,
    8: I => OCamlI,
    9: J => OCamlJ);

// This copies
unsafe impl<A: BigarrayElt> ToOCaml<Array1<A>> for &[A] {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, Array1<A>> {
        alloc_bigarray1(cr, self)
    }
}

// Note: we deliberately don't implement FromOCaml<Array1<A>>,
// because this trait doesn't have a lifetime parameter
// and implementing would force a copy.
impl<A: BigarrayElt> Borrow<[A]> for OCaml<'_, Array1<A>> {
    fn borrow(&self) -> &[A] {
        unsafe {
            let ba = self.custom_ptr_val::<ocaml_sys::bigarray::Bigarray>();
            core::slice::from_raw_parts((*ba).data as *const A, self.len())
        }
    }
}
