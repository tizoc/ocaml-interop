// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::memory::GCFrame;
use crate::mlvalues::tag;
use crate::mlvalues::*;
use std::marker;
use std::slice;
use std::str;

extern "C" {
    pub fn caml_string_length(s: RawOCaml) -> usize;
}

/// Representation of OCaml values inside `ocaml_frame` blocks.
///
/// Should not be instantiated directly, and will usually be the result
/// of `ocaml_alloc!` and `ocaml_call!` expressions, or the input arguments
/// of functions defined inside `ocaml_export!` blocks.
#[repr(transparent)]
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
    pub unsafe fn new<'gc>(_gc: &'a GCFrame<'gc>, x: RawOCaml) -> OCaml<'a, T> {
        OCaml {
            _marker: Default::default(),
            raw: x,
        }
    }

    pub unsafe fn field<F>(&self, i: UIntnat) -> OCaml<'a, F> {
        assert!(tag_val(self.raw) < tag::NO_SCAN || tag_val(self.raw) == tag::CUSTOM);
        assert!(i < wosize_val(self.raw));
        OCaml {
            _marker: Default::default(),
            raw: *(self.raw as *const RawOCaml).add(i),
        }
    }

    pub fn is_block(&self) -> bool {
        is_block(self.raw)
    }

    pub fn is_long(&self) -> bool {
        is_long(self.raw)
    }

    pub unsafe fn raw(self) -> RawOCaml {
        self.raw
    }
}

impl OCaml<'static, ()> {
    pub fn unit() -> OCaml<'static, ()> {
        OCaml {
            _marker: Default::default(),
            raw: UNIT,
        }
    }
}

impl<'a> OCaml<'a, String> {
    pub unsafe fn as_bytes(self) -> &'a [u8] {
        let s = self.raw;
        assert!(tag_val(s) == tag::STRING);
        slice::from_raw_parts(string_val(s), caml_string_length(s))
    }

    pub unsafe fn as_str(self) -> &'a str {
        str::from_utf8(self.as_bytes()).unwrap()
    }

    pub unsafe fn as_str_unchecked(self) -> &'a str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

impl<'a> OCaml<'a, OCamlBytes> {
    pub unsafe fn as_bytes(self) -> &'a [u8] {
        let s = self.raw;
        assert!(tag_val(s) == tag::STRING);
        slice::from_raw_parts(string_val(s), caml_string_length(s))
    }

    pub unsafe fn as_str(self) -> &'a str {
        str::from_utf8(self.as_bytes()).unwrap()
    }

    pub unsafe fn as_str_unchecked(self) -> &'a str {
        str::from_utf8_unchecked(self.as_bytes())
    }
}

impl<'a> OCaml<'a, Intnat> {
    pub fn as_int(self) -> i64 {
        unsafe { raw_ocaml_to_i64(self.raw) }
    }

    pub fn of_int(n: i64) -> Self {
        OCaml {
            _marker: Default::default(),
            raw: unsafe { raw_ocaml_of_i64(n) },
        }
    }
}

impl<'a> OCaml<'a, bool> {
    pub fn as_bool(self) -> bool {
        unsafe { raw_ocaml_to_i64(self.raw) != 0 }
    }

    pub fn of_bool(b: bool) -> Self {
        OCaml {
            _marker: Default::default(),
            raw: if b { TRUE } else { FALSE }
        }
    }
}

impl<'a, A> OCaml<'a, Option<A>> {
    pub fn is_none(&self) -> bool {
        self.raw == NONE
    }

    pub fn is_some(&self) -> bool {
        self.is_block()
    }

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
    pub fn nil() -> Self {
        OCaml {
            _marker: Default::default(),
            raw: EMPTY_LIST,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw == EMPTY_LIST
    }

    pub fn tl(&self) -> Option<OCaml<'a, A>> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.field(0) })
        }
    }

    pub fn hd(&self) -> Option<OCaml<'a, A>> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.field(1) })
        }
    }

    pub fn uncons(&self) -> Option<(OCaml<'a, A>, Self)> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { (self.field(0), self.field(1)) })
        }
    }
}
