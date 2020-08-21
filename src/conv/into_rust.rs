use crate::value::OCaml;
use crate::conv::FromOCaml;

/// Counterpart to `FromOCaml`, usually more comfortable to use.
pub trait IntoRust<T>: Sized {
    /// Convert into a Rust value.
    fn into_rust(self) -> T;
}

impl<'a, T, U> IntoRust<U> for OCaml<'a, T>
where
    U: FromOCaml<T>,
{
    fn into_rust(self) -> U {
        U::from_ocaml(self)
    }
}
