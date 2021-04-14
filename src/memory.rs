// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::{
    conv::FromOCaml,
    mlvalues::{tag, OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList, RawOCaml},
    runtime::OCamlRuntime,
    value::OCaml,
};
use core::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    pin::Pin,
};
pub use ocaml_sys::{caml_alloc, store_field};
use ocaml_sys::{
    caml_alloc_string, caml_alloc_tuple, caml_copy_double, caml_copy_int32, caml_copy_int64, string_val,
};

/// A global root for keeping OCaml values alive and tracked
///
/// This allows keeping a value around when exiting the stack frame.
///
/// See [`OCaml::register_global_root`].
pub struct OCamlGlobalRoot<T> {
    pub(crate) cell: Pin<Box<Cell<RawOCaml>>>,
    _marker: PhantomData<Cell<T>>,
}

impl<T> std::fmt::Debug for OCamlGlobalRoot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OCamlGlobalRoot({:#x})", self.cell.get())
    }
}

impl<T> OCamlGlobalRoot<T> {
    // NOTE: we require initialisation here, unlike OCamlRoot which delays it
    // This is because we register with the GC in the constructor,
    // for easy pairing with Drop, and registering without initializing
    // would break OCaml runtime invariants.
    // Always registering with UNIT (like for GCFrame initialisation)
    // would also work, but for OCamlGenerationalRoot that would
    // make things slower (updating requires notifying the GC),
    // and it's better if the API is the same for both kinds of global roots.
    pub(crate) fn new(val: OCaml<T>) -> Self {
        let r = Self {
            cell: Box::pin(Cell::new(val.raw)),
            _marker: PhantomData,
        };
        unsafe { ocaml_sys::caml_register_global_root(r.cell.as_ptr()) };
        r
    }

    /// Access the rooted value
    pub fn get_ref(&self) -> OCamlRef<T> {
        unsafe { OCamlCell::create_ref(self.cell.as_ptr()) }
    }

    /// Replace the rooted value
    pub fn set(&self, val: OCaml<T>) {
        self.cell.replace(val.raw);
    }
}

impl<T> Drop for OCamlGlobalRoot<T> {
    fn drop(&mut self) {
        unsafe { ocaml_sys::caml_remove_global_root(self.cell.as_ptr()) };
    }
}

/// A global, GC-friendly root for keeping OCaml values alive and tracked
///
/// This allows keeping a value around when exiting the stack frame.
///
/// Unlike with [`OCamlGlobalRoot`], the GC doesn't have to walk
/// referenced values on every minor collection.  This makes collection
/// faster, except if the value is short-lived and frequently updated.
///
/// See [`OCaml::register_generational_root`].
pub struct OCamlGenerationalRoot<T> {
    pub(crate) cell: Pin<Box<Cell<RawOCaml>>>,
    _marker: PhantomData<Cell<T>>,
}

impl<T> std::fmt::Debug for OCamlGenerationalRoot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OCamlGenerationalRoot({:#x})", self.cell.get())
    }
}

impl<T> OCamlGenerationalRoot<T> {
    pub(crate) fn new(val: OCaml<T>) -> Self {
        let r = Self {
            cell: Box::pin(Cell::new(val.raw)),
            _marker: PhantomData,
        };
        unsafe { ocaml_sys::caml_register_generational_global_root(r.cell.as_ptr()) };
        r
    }

    /// Access the rooted value
    pub fn get_ref(&self) -> OCamlRef<T> {
        unsafe { OCamlCell::create_ref(self.cell.as_ptr()) }
    }

    /// Replace the rooted value
    pub fn set(&self, val: OCaml<T>) {
        unsafe { ocaml_sys::caml_modify_generational_global_root(self.cell.as_ptr(), val.raw) };
        debug_assert_eq!(self.cell.get(), val.raw);
    }
}

impl<T> Drop for OCamlGenerationalRoot<T> {
    fn drop(&mut self) {
        unsafe { ocaml_sys::caml_remove_generational_global_root(self.cell.as_ptr()) };
    }
}

pub struct OCamlCell<T> {
    cell: UnsafeCell<RawOCaml>,
    _marker: PhantomData<T>,
}

static_assertions::assert_eq_size!(OCamlCell<bool>, OCaml<'static, bool>, RawOCaml);

/// An `OCamlRef<T>` is a reference to a location containing a [`OCaml`]`<T>` value.
///
/// Usually obtained as the result of rooting an OCaml value.
pub type OCamlRef<'a, T> = &'a OCamlCell<T>;

impl<T> OCamlCell<T> {
    #[doc(hidden)]
    pub unsafe fn create_ref<'a>(val: *const RawOCaml) -> OCamlRef<'a, T> {
        &*(val as *const OCamlCell<T>)
    }

    /// Converts this value into a Rust value.
    pub fn to_rust<RustT>(&self, cr: &OCamlRuntime) -> RustT
    where
        RustT: FromOCaml<T>,
    {
        RustT::from_ocaml(cr.get(self))
    }

    /// Borrows the raw value contained in this root.
    ///
    /// # Safety
    ///
    /// The [`RawOCaml`] value obtained may become invalid after the OCaml GC runs.
    pub unsafe fn get_raw(&self) -> RawOCaml {
        *self.cell.get()
    }
}

pub fn alloc_bytes<'a>(cr: &'a mut OCamlRuntime, s: &[u8]) -> OCaml<'a, OCamlBytes> {
    unsafe {
        let len = s.len();
        let value = caml_alloc_string(len);
        let ptr = string_val(value);
        core::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
        OCaml::new(cr, value)
    }
}

pub fn alloc_string<'a>(cr: &'a mut OCamlRuntime, s: &str) -> OCaml<'a, String> {
    unsafe {
        let len = s.len();
        let value = caml_alloc_string(len);
        let ptr = string_val(value);
        core::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
        OCaml::new(cr, value)
    }
}

pub fn alloc_int32(cr: &mut OCamlRuntime, i: i32) -> OCaml<OCamlInt32> {
    unsafe { OCaml::new(cr, caml_copy_int32(i)) }
}

pub fn alloc_int64(cr: &mut OCamlRuntime, i: i64) -> OCaml<OCamlInt64> {
    unsafe { OCaml::new(cr, caml_copy_int64(i)) }
}

pub fn alloc_double(cr: &mut OCamlRuntime, d: f64) -> OCaml<OCamlFloat> {
    unsafe { OCaml::new(cr, caml_copy_double(d)) }
}

// TODO: it is possible to directly alter the fields memory upon first allocation of
// small values (like tuples and conses are) without going through `caml_modify` to get
// a little bit of extra performance.

pub fn alloc_some<'a, 'b, A>(
    cr: &'a mut OCamlRuntime,
    value: OCamlRef<'b, A>,
) -> OCaml<'a, Option<A>> {
    unsafe {
        let ocaml_some = caml_alloc(1, tag::SOME);
        store_field(ocaml_some, 0, value.get_raw());
        OCaml::new(cr, ocaml_some)
    }
}

#[doc(hidden)]
pub unsafe fn alloc_tuple<T>(cr: &mut OCamlRuntime, size: usize) -> OCaml<T> {
    let ocaml_tuple = caml_alloc_tuple(size);
    OCaml::new(cr, ocaml_tuple)
}

pub fn alloc_cons<'a, 'b, A>(
    cr: &'a mut OCamlRuntime,
    head: OCamlRef<'b, A>,
    tail: OCamlRef<'b, OCamlList<A>>,
) -> OCaml<'a, OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::CONS);
        store_field(ocaml_cons, 0, head.get_raw());
        store_field(ocaml_cons, 1, tail.get_raw());
        OCaml::new(cr, ocaml_cons)
    }
}
