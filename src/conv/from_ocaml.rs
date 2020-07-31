use mlvalues::Intnat;
use value::OCaml;

/// `FromOCaml` implements conversion from OCaml values into Rust values.
pub unsafe trait FromOCaml<T> {
    /// Convert from OCaml value
    fn from_ocaml(v: OCaml<T>) -> Self;
}

unsafe impl FromOCaml<Intnat> for i64 {
    fn from_ocaml(v: OCaml<Intnat>) -> Self {
        v.as_int()
    }
}

unsafe impl FromOCaml<String> for Vec<u8> {
    fn from_ocaml(v: OCaml<String>) -> Self {
        let raw_bytes = v.as_bytes();
        let mut vec: Vec<u8> = Vec::with_capacity(raw_bytes.len());
        vec.extend_from_slice(raw_bytes);
        vec
    }
}

unsafe impl FromOCaml<String> for String {
    fn from_ocaml(v: OCaml<String>) -> Self {
        v.as_str().to_owned()
    }
}
