use mlvalues::{Intnat, OCamlBytes, OCamlInt32, OCamlList};
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

unsafe impl FromOCaml<OCamlInt32> for i32 {
    fn from_ocaml(v: OCaml<OCamlInt32>) -> Self {
        let val: OCaml<i32> = unsafe { v.field(1) };
        unsafe { *(val.raw() as *const i32) }
    }
}

unsafe impl FromOCaml<bool> for bool {
    fn from_ocaml(v: OCaml<bool>) -> Self {
        v.as_bool()
    }
}

unsafe impl FromOCaml<f64> for f64 {
    fn from_ocaml(v: OCaml<f64>) -> Self {
        unsafe { *(v.raw() as *const f64) }
    }
}

unsafe impl FromOCaml<String> for Vec<u8> {
    fn from_ocaml(v: OCaml<String>) -> Self {
        let raw_bytes = unsafe { v.as_bytes() };
        let mut vec: Vec<u8> = Vec::with_capacity(raw_bytes.len());
        vec.extend_from_slice(raw_bytes);
        vec
    }
}

unsafe impl FromOCaml<String> for String {
    fn from_ocaml(v: OCaml<String>) -> Self {
        unsafe { v.as_str() }.to_owned()
    }
}

unsafe impl FromOCaml<OCamlBytes> for Vec<u8> {
    fn from_ocaml(v: OCaml<OCamlBytes>) -> Self {
        let raw_bytes = unsafe { v.as_bytes() };
        let mut vec: Vec<u8> = Vec::with_capacity(raw_bytes.len());
        vec.extend_from_slice(raw_bytes);
        vec
    }
}

unsafe impl FromOCaml<OCamlBytes> for String {
    fn from_ocaml(v: OCaml<OCamlBytes>) -> Self {
        unsafe { v.as_str_unchecked() }.to_owned()
    }
}

unsafe impl<A, FromA> FromOCaml<Option<FromA>> for Option<A>
where
    A: FromOCaml<FromA>,
{
    fn from_ocaml(v: OCaml<Option<FromA>>) -> Self {
        if let Some(value) = v.to_option() {
            Some(A::from_ocaml(value))
        } else {
            None
        }
    }
}

unsafe impl<A, B, FromA, FromB> FromOCaml<(FromA, FromB)> for (A, B)
where
    A: FromOCaml<FromA>,
    B: FromOCaml<FromB>,
{
    fn from_ocaml(v: OCaml<(FromA, FromB)>) -> Self {
        (A::from_ocaml(v.fst()), B::from_ocaml(v.snd()))
    }
}

unsafe impl<A, B, C, FromA, FromB, FromC> FromOCaml<(FromA, FromB, FromC)> for (A, B, C)
where
    A: FromOCaml<FromA>,
    B: FromOCaml<FromB>,
    C: FromOCaml<FromC>,
{
    fn from_ocaml(v: OCaml<(FromA, FromB, FromC)>) -> Self {
        (
            A::from_ocaml(v.fst()),
            B::from_ocaml(v.snd()),
            C::from_ocaml(v.tuple_3()),
        )
    }
}

unsafe impl<A, B, C, D, FromA, FromB, FromC, FromD> FromOCaml<(FromA, FromB, FromC, FromD)>
    for (A, B, C, D)
where
    A: FromOCaml<FromA>,
    B: FromOCaml<FromB>,
    C: FromOCaml<FromC>,
    D: FromOCaml<FromD>,
{
    fn from_ocaml(v: OCaml<(FromA, FromB, FromC, FromD)>) -> Self {
        (
            A::from_ocaml(v.fst()),
            B::from_ocaml(v.snd()),
            C::from_ocaml(v.tuple_3()),
            D::from_ocaml(v.tuple_4()),
        )
    }
}

unsafe impl<A, FromA> FromOCaml<OCamlList<FromA>> for Vec<A>
where
    A: FromOCaml<FromA>,
{
    fn from_ocaml(v: OCaml<OCamlList<FromA>>) -> Self {
        // TODO: pre-calculate actual required capacity?
        let mut vec = Vec::new();
        let mut current = v;
        while let Some((hd, tl)) = current.uncons() {
            current = tl;
            vec.push(A::from_ocaml(hd));
        }
        vec
    }
}
