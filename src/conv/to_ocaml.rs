use crate::memory::{
    alloc_bytes, alloc_cons, alloc_double, alloc_int32, alloc_some, alloc_string, alloc_tuple,
    alloc_tuple_3, alloc_tuple_4, GCResult, GCToken,
};
use crate::mlvalues::{Intnat, OCamlBytes, OCamlInt32, OCamlList, RawOCaml, FALSE, NONE, TRUE};
use crate::value::OCaml;
use crate::{ocaml_alloc, ocaml_frame};

/// `ToOCaml` implements conversion from Rust values into OCaml values.
pub unsafe trait ToOCaml<T> {
    /// Convert to OCaml value
    fn to_ocaml(&self, gc: GCToken) -> GCResult<T>;
}

unsafe impl ToOCaml<Intnat> for i64 {
    fn to_ocaml(&self, _token: GCToken) -> GCResult<Intnat> {
        GCResult::of(((self << 1) | 1) as RawOCaml)
    }
}

unsafe impl ToOCaml<Intnat> for i32 {
    fn to_ocaml(&self, token: GCToken) -> GCResult<Intnat> {
        (*self as i64).to_ocaml(token)
    }
}

unsafe impl ToOCaml<OCamlInt32> for i32 {
    fn to_ocaml(&self, token: GCToken) -> GCResult<OCamlInt32> {
        alloc_int32(token, *self)
    }
}

unsafe impl ToOCaml<f64> for f64 {
    fn to_ocaml(&self, token: GCToken) -> GCResult<f64> {
        alloc_double(token, *self)
    }
}

unsafe impl ToOCaml<bool> for bool {
    fn to_ocaml(&self, _token: GCToken) -> GCResult<bool> {
        GCResult::of(if *self { TRUE } else { FALSE })
    }
}

unsafe impl ToOCaml<String> for Vec<u8> {
    fn to_ocaml(&self, token: GCToken) -> GCResult<String> {
        let s = unsafe { std::str::from_utf8_unchecked(self.as_slice()) };
        alloc_string(token, s)
    }
}

unsafe impl ToOCaml<String> for &Vec<u8> {
    fn to_ocaml(&self, token: GCToken) -> GCResult<String> {
        let s = unsafe { std::str::from_utf8_unchecked(self.as_slice()) };
        alloc_string(token, s)
    }
}

unsafe impl ToOCaml<String> for String {
    fn to_ocaml(&self, token: GCToken) -> GCResult<String> {
        alloc_string(token, self.as_str())
    }
}

unsafe impl ToOCaml<String> for &String {
    fn to_ocaml(&self, token: GCToken) -> GCResult<String> {
        alloc_string(token, self.as_str())
    }
}

unsafe impl ToOCaml<String> for &str {
    fn to_ocaml(&self, token: GCToken) -> GCResult<String> {
        alloc_string(token, self)
    }
}

unsafe impl ToOCaml<OCamlBytes> for Vec<u8> {
    fn to_ocaml(&self, token: GCToken) -> GCResult<OCamlBytes> {
        alloc_bytes(token, self.as_slice())
    }
}

unsafe impl ToOCaml<OCamlBytes> for String {
    fn to_ocaml(&self, token: GCToken) -> GCResult<OCamlBytes> {
        alloc_bytes(token, self.as_bytes())
    }
}

unsafe impl ToOCaml<OCamlBytes> for str {
    fn to_ocaml(&self, token: GCToken) -> GCResult<OCamlBytes> {
        alloc_bytes(token, self.as_bytes())
    }
}

unsafe impl<A, ToA> ToOCaml<Option<ToA>> for Option<A>
where
    A: ToOCaml<ToA>,
{
    fn to_ocaml(&self, token: GCToken) -> GCResult<Option<ToA>> {
        if let Some(value) = self {
            ocaml_frame!(gc, {
                let ocaml_value = ocaml_alloc!(value.to_ocaml(gc));
                alloc_some(token, ocaml_value)
            })
        } else {
            GCResult::of(NONE)
        }
    }
}

unsafe impl<A, B, ToA, ToB> ToOCaml<(ToA, ToB)> for (A, B)
where
    A: ToOCaml<ToA>,
    B: ToOCaml<ToB>,
{
    fn to_ocaml(&self, token: GCToken) -> GCResult<(ToA, ToB)> {
        ocaml_frame!(gc, {
            let fst = ocaml_alloc!((self.0).to_ocaml(gc));
            let ref fst_ref = gc.keep(fst);
            let snd = ocaml_alloc!((self.1).to_ocaml(gc));
            alloc_tuple(token, gc.get(fst_ref), snd)
        })
    }
}

unsafe impl<A, B, C, ToA, ToB, ToC> ToOCaml<(ToA, ToB, ToC)> for (A, B, C)
where
    A: ToOCaml<ToA>,
    B: ToOCaml<ToB>,
    C: ToOCaml<ToC>,
{
    fn to_ocaml(&self, token: GCToken) -> GCResult<(ToA, ToB, ToC)> {
        ocaml_frame!(gc, {
            let fst = ocaml_alloc!((self.0).to_ocaml(gc));
            let ref fst_ref = gc.keep(fst);
            let snd = ocaml_alloc!((self.1).to_ocaml(gc));
            let ref snd_ref = gc.keep(snd);
            let elt3 = ocaml_alloc!((self.2).to_ocaml(gc));
            alloc_tuple_3(token, gc.get(fst_ref), gc.get(snd_ref), elt3)
        })
    }
}

unsafe impl<A, B, C, D, ToA, ToB, ToC, ToD> ToOCaml<(ToA, ToB, ToC, ToD)> for (A, B, C, D)
where
    A: ToOCaml<ToA>,
    B: ToOCaml<ToB>,
    C: ToOCaml<ToC>,
    D: ToOCaml<ToD>,
{
    fn to_ocaml(&self, token: GCToken) -> GCResult<(ToA, ToB, ToC, ToD)> {
        ocaml_frame!(gc, {
            let fst = ocaml_alloc!((self.0).to_ocaml(gc));
            let ref fst_ref = gc.keep(fst);
            let snd = ocaml_alloc!((self.1).to_ocaml(gc));
            let ref snd_ref = gc.keep(snd);
            let elt3 = ocaml_alloc!((self.2).to_ocaml(gc));
            let ref elt3_ref = gc.keep(elt3);
            let elt4 = ocaml_alloc!((self.3).to_ocaml(gc));
            alloc_tuple_4(
                token,
                gc.get(fst_ref),
                gc.get(snd_ref),
                gc.get(elt3_ref),
                elt4,
            )
        })
    }
}

unsafe impl<A, ToA> ToOCaml<OCamlList<ToA>> for Vec<A>
where
    A: ToOCaml<ToA>,
{
    fn to_ocaml(&self, _token: GCToken) -> GCResult<OCamlList<ToA>> {
        ocaml_frame!(gc, {
            let ref mut result_ref = gc.keep(OCaml::nil());
            for elt in self.iter().rev() {
                let ov = ocaml_alloc!(elt.to_ocaml(gc));
                let cons = ocaml_alloc!(alloc_cons(gc, ov, gc.get(result_ref)));
                result_ref.set(cons);
            }
            GCResult::of_ocaml(gc.get(result_ref))
        })
    }
}
