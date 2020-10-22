// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::*;
use crate::{error::OCamlFixnumConversionError, memory::GCFrameHandle};
use core::{marker, slice, str};
use ocaml_sys::{caml_string_length, int_val, val_int};

/// Representation of OCaml values inside [`ocaml_frame!`] blocks.
///
/// Should not be instantiated directly, and will usually be the result
/// of [`ocaml_alloc!`] and [`ocaml_call!`] expressions, or the input arguments
/// of functions defined inside [`ocaml_export!`] blocks.
#[derive(Copy, Clone)]
pub struct OCaml<'a, T: 'a> {
    _marker: marker::PhantomData<&'a T>,
    raw: RawOCaml,
}

pub fn make_ocaml<'a, T>(x: RawOCaml) -> OCaml<'a, T> {
    OCaml {
        _marker: Default::default(),
        raw: x,
    }
}

impl<'a, T> OCaml<'a, T> {
    #[doc(hidden)]
    pub unsafe fn new<'gc>(_gc: &'a dyn GCFrameHandle<'gc>, x: RawOCaml) -> OCaml<'a, T> {
        OCaml {
            _marker: Default::default(),
            raw: x,
        }
    }

    #[doc(hidden)]
    pub unsafe fn field<F>(&self, i: UIntnat) -> OCaml<'a, F> {
        assert!(tag_val(self.raw) < tag::NO_SCAN);
        assert!(i < wosize_val(self.raw));
        OCaml {
            _marker: Default::default(),
            raw: *(self.raw as *const RawOCaml).add(i),
        }
    }

    #[doc(hidden)]
    pub fn is_block(&self) -> bool {
        is_block(self.raw)
    }

    #[doc(hidden)]
    pub fn is_long(&self) -> bool {
        is_long(self.raw)
    }

    #[doc(hidden)]
    pub fn tag_value(&self) -> u8 {
        assert!(self.is_block());
        unsafe { tag_val(self.raw) }
    }

    /// Gets the raw representation for this value reference (pointer or int).
    ///
    /// # Safety
    ///
    /// The resulting raw pointer will not be tracked, and may become invalid
    /// after any call into the OCaml runtime. Great care must be taken when
    /// working with these values.
    pub unsafe fn raw(&self) -> RawOCaml {
        self.raw
    }
}

impl OCaml<'static, ()> {
    /// Returns a value that represent OCaml's unit value.
    pub fn unit() -> OCaml<'static, ()> {
        OCaml {
            _marker: Default::default(),
            raw: UNIT,
        }
    }
}

impl<'a> OCaml<'a, String> {
    /// Returns an `[u8]` reference to the internal bytes of this value.
    pub fn as_bytes(&self) -> &'a [u8] {
        let s = self.raw;
        unsafe {
            assert!(tag_val(s) == tag::STRING);
            slice::from_raw_parts(string_val(s), caml_string_length(s))
        }
    }

    /// Returns a `str` reference to the internal bytes of this value.
    ///
    /// # Panics
    ///
    /// Panics if the bytes do not form a valid utf8 string.
    pub fn as_str(&self) -> &'a str {
        str::from_utf8(self.as_bytes()).unwrap()
    }

    /// Returns a `str` reference to the internal bytes of this value.
    ///
    /// # Safety
    ///
    /// No checks are performed to ensure that the returned value is a valid utf8 string.
    pub unsafe fn as_str_unchecked(&self) -> &'a str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

impl<'a> OCaml<'a, OCamlBytes> {
    /// Returns an `[u8]` reference to the internal bytes of this value.
    pub fn as_bytes(&self) -> &'a [u8] {
        let s = self.raw;
        unsafe {
            assert!(tag_val(s) == tag::STRING);
            slice::from_raw_parts(string_val(s), caml_string_length(s))
        }
    }

    /// Returns a `str` reference to the internal bytes of this value.
    ///
    /// # Panics
    ///
    /// Panics if the bytes do not form a valid utf8 string.
    pub fn as_str(&self) -> &'a str {
        str::from_utf8(self.as_bytes()).unwrap()
    }

    /// Returns a `str` reference to the internal bytes of this value.
    ///
    /// # Safety
    ///
    /// No checks are performed to ensure that the returned value is a valid utf8 string.
    pub unsafe fn as_str_unchecked(&self) -> &'a str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

impl<'a> OCaml<'a, OCamlInt> {
    /// Converts an OCaml int to an `i64`.
    pub fn as_i64(&self) -> i64 {
        int_val(self.raw) as i64
    }

    /// Creates an OCaml int from an `i64` without checking that it fits in an OCaml fixnum.
    ///
    /// # Safety
    ///
    /// OCaml ints are represented as 63bits + 1bit tag, so when converting
    /// from an i64, a bit of precision is lost.
    pub unsafe fn of_i64_unchecked(n: i64) -> OCaml<'static, OCamlInt> {
        OCaml {
            _marker: Default::default(),
            raw: val_int(n as isize),
        }
    }

    // Creates an OCaml int from an `i64`.
    //
    // The conversion fails if the `i64` value doesn't fit in an OCaml fixnum and
    // an error is returned instead.
    pub fn of_i64(n: i64) -> Result<OCaml<'static, OCamlInt>, OCamlFixnumConversionError> {
        if n > MAX_FIXNUM as i64 {
            Err(OCamlFixnumConversionError::InputTooBig(n))
        } else if n < MIN_FIXNUM as i64 {
            Err(OCamlFixnumConversionError::InputTooSmall(n))
        } else {
            Ok(OCaml {
                _marker: Default::default(),
                raw: val_int(n as isize),
            })
        }
    }

    /// Creates an OCaml int from an i32.
    pub fn of_i32(n: i32) -> OCaml<'static, OCamlInt> {
        OCaml {
            _marker: Default::default(),
            raw: val_int(n as isize),
        }
    }
}

impl<'a> OCaml<'a, bool> {
    /// Converts an OCaml boolean into a Rust boolean.
    pub fn as_bool(self) -> bool {
        int_val(self.raw) != 0
    }

    /// Creates an OCaml boolean from a Rust boolean.
    pub fn of_bool(b: bool) -> Self {
        OCaml {
            _marker: Default::default(),
            raw: if b { TRUE } else { FALSE },
        }
    }
}

impl<'a, A> OCaml<'a, Option<A>> {
    /// Returns true if this OCaml option value is an OCaml `None`.
    pub fn is_none(&self) -> bool {
        self.raw == NONE
    }

    /// Returns true if this OCaml option value is an OCaml `Some`.
    pub fn is_some(&self) -> bool {
        self.is_block()
    }

    /// Converts an OCaml `Option<T>` value into a Rust `Option<OCaml<T>>`.
    pub fn to_option(&self) -> Option<OCaml<'a, A>> {
        if self.is_none() {
            None
        } else {
            let value: OCaml<A> = unsafe { self.field(0) };
            Some(OCaml {
                _marker: Default::default(),
                raw: value.raw,
            })
        }
    }
}

impl<'a, A, B> OCaml<'a, (A, B)> {
    pub fn fst(&self) -> OCaml<'a, A> {
        unsafe { self.field(0) }
    }

    pub fn snd(&self) -> OCaml<'a, B> {
        unsafe { self.field(1) }
    }
}

impl<'a, A, B, C> OCaml<'a, (A, B, C)> {
    pub fn fst(&self) -> OCaml<'a, A> {
        unsafe { self.field(0) }
    }

    pub fn snd(&self) -> OCaml<'a, B> {
        unsafe { self.field(1) }
    }

    pub fn tuple_3(&self) -> OCaml<'a, C> {
        unsafe { self.field(2) }
    }
}

impl<'a, A, B, C, D> OCaml<'a, (A, B, C, D)> {
    pub fn fst(&self) -> OCaml<'a, A> {
        unsafe { self.field(0) }
    }

    pub fn snd(&self) -> OCaml<'a, B> {
        unsafe { self.field(1) }
    }

    pub fn tuple_3(&self) -> OCaml<'a, C> {
        unsafe { self.field(2) }
    }

    pub fn tuple_4(&self) -> OCaml<'a, D> {
        unsafe { self.field(3) }
    }
}

impl<'a, A> OCaml<'a, OCamlList<A>> {
    /// Returns an OCaml nil (empty list) value.
    pub fn nil() -> Self {
        OCaml {
            _marker: Default::default(),
            raw: EMPTY_LIST,
        }
    }

    /// Returns true if the value is OCaml's nil (empty list).
    pub fn is_empty(&self) -> bool {
        self.raw == EMPTY_LIST
    }

    /// Returns the head of an OCaml list.
    pub fn hd(&self) -> Option<OCaml<'a, A>> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.field(0) })
        }
    }

    /// Returns the tail of an OCaml list.
    pub fn tl(&self) -> Option<OCaml<'a, OCamlList<A>>> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.field(1) })
        }
    }

    /// Returns a tuple of the head and tail of an OCaml list.
    pub fn uncons(&self) -> Option<(OCaml<'a, A>, Self)> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { (self.field(0), self.field(1)) })
        }
    }
}
