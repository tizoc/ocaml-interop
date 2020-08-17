use OCaml;
use FromOCaml;

pub trait IntoRust<T>: Sized {
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
