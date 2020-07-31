use crate::memory::GCFrame;
use crate::mlvalues::*;
use crate::mlvalues::tag;
use std::marker;
use std::slice;
use std::str;

extern "C" {
    pub fn caml_string_length(s: RawOCaml) -> usize;
}

#[repr(transparent)]
pub struct OCaml<'a, T: 'a> {
    _marker: marker::PhantomData<&'a T>,
    raw: RawOCaml,
}

impl<'a, T> Copy for OCaml<'a, T> {}

impl<'a, T> Clone for OCaml<'a, T> {
    fn clone(&self) -> OCaml<'a, T> {
        OCaml {
            _marker: Default::default(),
            raw: self.raw,
        }
    }
}

impl<'a, T> Into<RawOCaml> for OCaml<'a, T> {
    fn into(self) -> RawOCaml {
        self.raw
    }
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

    pub unsafe fn field<F>(self, i: UIntnat) -> OCaml<'a, F> {
        assert!(tag_val(self.raw) < tag::NO_SCAN);
        assert!(i < wosize_val(self.raw));
        OCaml {
            _marker: Default::default(),
            raw: *(self.raw as *const RawOCaml).add(i),
        }
    }

    pub fn is_block(self) -> bool {
        is_block(self.raw)
    }
}

impl<'a> OCaml<'a, String> {
    pub fn as_bytes(self) -> &'a [u8] {
        let s = self.raw;
        assert!(unsafe { tag_val(s) } == tag::STRING);
        unsafe { slice::from_raw_parts(string_val(s), caml_string_length(s)) }
    }

    pub fn as_str(self) -> &'a str {
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
