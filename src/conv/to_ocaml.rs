use crate::memory::{alloc_bytes, alloc_string, GCResult, GCToken};
use crate::mlvalues::{Intnat, RawOCaml};
use crate::value::OCaml;

/// `ToOCaml` implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value
    fn to_ocaml(self, gc: GCToken) -> GCResult<T>;
}

/// `ToOCamlInteger` implements conversion from Rust integers into OCaml values.
pub unsafe trait ToOCamlInteger {
    fn to_ocaml_fixnum(self) -> OCaml<'static, Intnat>;
    // TODO: Int32.t and Int64.t
}

unsafe impl ToOCaml<Intnat> for i64 {
    fn to_ocaml(self, _token: GCToken) -> GCResult<Intnat> {
        GCResult::of(((self << 1) | 1) as RawOCaml)
    }
}

unsafe impl ToOCamlInteger for i64 {
    fn to_ocaml_fixnum(self) -> OCaml<'static, Intnat> {
        OCaml::of_int(self)
    }
}

unsafe impl ToOCaml<String> for &Vec<u8> {
    fn to_ocaml<'a, 'gc>(self, token: GCToken) -> GCResult<String> {
        alloc_bytes(token, self.as_slice())
    }
}

unsafe impl ToOCaml<String> for &String {
    fn to_ocaml<'a, 'gc>(self, token: GCToken) -> GCResult<String> {
        alloc_string(token, self.as_str())
    }
}

unsafe impl ToOCaml<String> for &str {
    fn to_ocaml(self, token: GCToken) -> GCResult<String> {
        alloc_string(token, self)
    }
}
