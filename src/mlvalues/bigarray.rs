use core::marker::PhantomData;

/// Bigarray kind
///
/// # Safety
///
/// This is unsafe to implement, because it allows casts
/// to the implementing type (through `OCaml<Array1<T>>::as_slice()`).
///
/// To make this safe, the type implementing this trait must be
/// safe to transmute from OCaml data with the relevant KIND.
//
// Using a trait for this means a Rust type can only be matched to a
// single kind.  There are enough Rust types that this isn't a problem
// in practice.
pub unsafe trait BigarrayElt: Copy {
    /// OCaml bigarray type identifier
    const KIND: i32;
}

// TODO:
// assert that size_of::<$t>() matches caml_ba_element_size[$k]
// Not sure we can check this at compile time,
// when caml_ba_element_size is a C symbol
macro_rules! make_kind {
    ($t:ty, $k:ident) => {
        unsafe impl BigarrayElt for $t {
            const KIND: i32 = ocaml_sys::bigarray::Kind::$k as i32;
        }
    };
}

// In kind order
// Skips some kinds OCaml supports: caml_int, complex32, complex64
make_kind!(f32, FLOAT32);
make_kind!(f64, FLOAT64);
make_kind!(i8, SINT8);
make_kind!(u8, UINT8);
make_kind!(i16, SINT16);
make_kind!(u16, UINT16);
make_kind!(i32, INT32);
make_kind!(i64, INT64);
make_kind!(isize, NATIVE_INT);
make_kind!(char, CHAR);

pub struct Array1<A: BigarrayElt> {
    _marker: PhantomData<A>,
}
