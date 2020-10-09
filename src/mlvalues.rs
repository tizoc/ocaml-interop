// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

pub use ocaml_sys::{
    extract_exception, field as field_val, is_block, is_exception_result, is_long, string_val,
    tag_val, wosize_val, Intnat, Uintnat as UIntnat, Value as RawOCaml,
    EMPTY_LIST, FALSE, TRUE, UNIT,
};
use core::marker;

pub mod tag;

/// `OCaml<OCamlList<T>>` is a reference to an OCaml `list` containing
/// values of type `T`.
pub struct OCamlList<A> {
    _marker: marker::PhantomData<A>,
}

/// `OCaml<OCamlBytes>` is a reference to an OCaml `bytes` value.
///
/// # Note
///
/// Unlike with `OCaml<String>`, there is no validation being performed when converting this
/// value into `String`.
pub struct OCamlBytes {}

/// `OCaml<OCamlInt>` is an OCaml integer (tagged and unboxed) value.
pub type OCamlInt = Intnat;

/// `OCaml<OCamlInt32>` is a reference to an OCaml `Int32.t` (boxed `int32`) value.
pub struct OCamlInt32 {}

/// `OCaml<OCamlInt64>` is a reference to an OCaml `Int64.t` (boxed `int64`) value.
pub struct OCamlInt64 {}

/// `OCaml<OCamlFloat>` is a reference to an OCaml `float` (boxed `float`) value.
pub struct OCamlFloat {}

// #define Val_none Val_int(0)
pub const NONE: RawOCaml = unsafe { raw_ocaml_of_i64(0) };

// #define Max_long (((intnat)1 << (8 * sizeof(value) - 2)) - 1)
pub const MAX_FIXNUM: isize = (1 << (8 * core::mem::size_of::<RawOCaml>() - 2)) - 1;

// #define Min_long (-((intnat)1 << (8 * sizeof(value) - 2)))
pub const MIN_FIXNUM: isize = -(1 << (8 * core::mem::size_of::<RawOCaml>() - 2));

#[doc(hidden)]
#[inline]
pub unsafe fn raw_ocaml_to_i64(raw: RawOCaml) -> i64 {
    assert!(!is_block(raw));
    (raw >> 1) as i64
}

#[inline]
pub const unsafe fn raw_ocaml_of_i64(n: i64) -> RawOCaml {
    ((n << 1) | 1) as RawOCaml
}
