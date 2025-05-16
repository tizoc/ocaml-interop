// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

#[cfg(doc)]
use crate::*;

use core::marker::PhantomData;
pub use ocaml_sys::{
    extract_exception, field as field_val, is_block, is_exception_result, is_long, string_val,
    tag_val, wosize_val, Intnat, Uintnat as UIntnat, Value as RawOCaml, EMPTY_LIST, FALSE,
    MAX_FIXNUM, MIN_FIXNUM, NONE, TRUE, UNIT,
};

pub mod bigarray;
pub mod tag;

pub(crate) unsafe fn int32_val(v: RawOCaml) -> i32 {
    let val = unsafe { field_val(v, 1) };
    unsafe { *(val as *const i32) }
}

pub(crate) unsafe fn int64_val(v: RawOCaml) -> i64 {
    let val = unsafe { field_val(v, 1) };
    unsafe { *(val as *const i64) }
}

/// [`OCaml`]`<OCamlList<T>>` is a reference to an OCaml `list` containing
/// values of type `T`.
pub struct OCamlList<A> {
    _marker: PhantomData<A>,
}

/// [`OCaml`]`<OCamlUniformArray<T>>` is a reference to an OCaml array which is
/// guaranteed to not contain unboxed floats. If OCaml was configured with
/// `--disable-flat-float-array` this corresponds to regular `array`s, but if
/// not, `Uniform_array.t` in the `base` library can be used instead.
/// See [Lexifi's blog post on the topic](https://www.lexifi.com/blog/ocaml/about-unboxed-float-arrays/)
/// for more details.
pub struct OCamlUniformArray<A> {
    _marker: PhantomData<A>,
}

/// [`OCaml`]`<OCamlFloatArray<T>>` is a reference to an OCaml `floatarray`
/// which is an array containing `float`s in an unboxed form.
pub struct OCamlFloatArray {}

/// `OCaml<DynBox<T>>` is for passing a value of type `T` to OCaml
///
/// To box a Rust value, use [`OCaml::box_value`][crate::OCaml::box_value].
///
/// **Experimental**
pub struct DynBox<A> {
    _marker: PhantomData<A>,
}

/// [`OCaml`]`<OCamlBytes>` is a reference to an OCaml `bytes` value.
///
/// # Note
///
/// Unlike with [`OCaml`]`<String>`, there is no validation being performed when converting this
/// value into `String`.
pub struct OCamlBytes {}

/// [`OCaml`]`<OCamlInt>` is an OCaml integer (tagged and unboxed) value.
pub type OCamlInt = Intnat;

/// [`OCaml`]`<OCamlInt32>` is a reference to an OCaml `Int32.t` (boxed `int32`) value.
pub struct OCamlInt32 {}

/// [`OCaml`]`<OCamlInt64>` is a reference to an OCaml `Int64.t` (boxed `int64`) value.
pub struct OCamlInt64 {}

/// [`OCaml`]`<OCamlFloat>` is a reference to an OCaml `float` (boxed `float`) value.
pub struct OCamlFloat {}

/// [`OCaml`]`<OCamlException>` is a reference to an OCaml `exn` value.
pub struct OCamlException {}
