// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::{
    conv::FromOCaml,
    mlvalues::{tag, Intnat, OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList, RawOCaml},
    runtime::OCamlRuntime,
    value::OCaml,
};
use core::{cell::UnsafeCell, marker::PhantomData, ptr};
pub use ocaml_sys::{
    caml_alloc, local_roots as ocaml_sys_local_roots, set_local_roots as ocaml_sys_set_local_roots,
    store_field,
};
use ocaml_sys::{
    caml_alloc_string, caml_alloc_tuple, caml_copy_double, caml_copy_int32, caml_copy_int64,
    string_val,
};

// Structure representing a block in the list of OCaml's GC local roots.
#[repr(C)]
struct CamlRootsBlock {
    next: *mut CamlRootsBlock,
    ntables: Intnat,
    nitems: Intnat,
    local_roots: *mut RawOCaml,
    // NOTE: in C this field is defined instead, but we only need a single pointer
    // tables: [*mut RawOCaml; 5],
}

impl Default for CamlRootsBlock {
    fn default() -> Self {
        CamlRootsBlock {
            next: ptr::null_mut(),
            ntables: 0,
            nitems: 0,
            local_roots: ptr::null_mut(),
            // NOTE: to mirror C's definition this would be
            // tables: [ptr::null_mut(); 5],
        }
    }
}

// Overrides for ocaml-sys functions of the same name but using ocaml-interop's CamlRootBlocks representation.
unsafe fn local_roots() -> *mut CamlRootsBlock {
    ocaml_sys_local_roots() as *mut CamlRootsBlock
}

unsafe fn set_local_roots(roots: *mut CamlRootsBlock) {
    ocaml_sys_set_local_roots(roots as *mut ocaml_sys::CamlRootsBlock)
}

// OCaml GC frame handle
#[derive(Default)]
pub struct GCFrame<'gc> {
    _marker: PhantomData<&'gc i32>,
    block: CamlRootsBlock,
}

// Impl

impl<'gc> GCFrame<'gc> {
    #[doc(hidden)]
    pub fn initialize(&mut self, frame_local_roots: &[UnsafeCell<RawOCaml>]) -> &mut Self {
        self.block.local_roots = frame_local_roots[0].get();
        self.block.ntables = 1;
        unsafe {
            self.block.next = local_roots();
            set_local_roots(&mut self.block);
        };
        self
    }
}

impl<'gc> Drop for GCFrame<'gc> {
    fn drop(&mut self) {
        unsafe {
            assert!(
                local_roots() == &mut self.block,
                "OCaml local roots corrupted"
            );
            set_local_roots(self.block.next);
        }
    }
}

pub struct OCamlRawRoot<'a> {
    cell: &'a UnsafeCell<RawOCaml>,
}

impl<'a> OCamlRawRoot<'a> {
    #[doc(hidden)]
    pub unsafe fn reserve<'gc>(_gc: &GCFrame<'gc>) -> OCamlRawRoot<'gc> {
        assert_eq!(&_gc.block as *const _, local_roots());
        let block = &mut *local_roots();
        let locals: *const UnsafeCell<RawOCaml> =
            &*(block.local_roots as *const UnsafeCell<RawOCaml>);
        let cell = &*locals.offset(block.nitems);
        block.nitems += 1;
        OCamlRawRoot { cell }
    }

    /// Roots an [`OCaml`] value.
    pub fn keep<'tmp, T>(&'tmp mut self, val: OCaml<T>) -> OCamlRef<'tmp, T> {
        unsafe {
            let cell = self.cell.get();
            *cell = val.raw();
            &*(cell as *const OCamlCell<T>)
        }
    }

    /// Roots a [`RawOCaml`] value and attaches a type to it.
    ///
    /// # Safety
    ///
    /// This method is unsafe because there is no way to validate that the [`RawOCaml`] value
    /// is of the correct type.
    pub unsafe fn keep_raw<T>(&mut self, val: RawOCaml) -> OCamlRef<T> {
        let cell = self.cell.get();
        *cell = val;
        &*(cell as *const OCamlCell<T>)
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
    /// This method is unsafe, because the RawOCaml value obtained will not be tracked.
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

pub fn alloc_some<'a, A>(cr: &'a mut OCamlRuntime, value: OCamlRef<A>) -> OCaml<'a, Option<A>> {
    unsafe {
        let ocaml_some = caml_alloc(1, tag::SOME);
        store_field(ocaml_some, 0, value.get_raw());
        OCaml::new(cr, ocaml_some)
    }
}

pub fn alloc_tuple<'a, F, S>(
    cr: &'a mut OCamlRuntime,
    fst: OCamlRef<F>,
    snd: OCamlRef<S>,
) -> OCaml<'a, (F, S)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(2);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        OCaml::new(cr, ocaml_tuple)
    }
}

pub fn alloc_tuple_3<'a, F, S, T3>(
    cr: &'a mut OCamlRuntime,
    fst: OCamlRef<F>,
    snd: OCamlRef<S>,
    elt3: OCamlRef<T3>,
) -> OCaml<'a, (F, S, T3)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(3);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        store_field(ocaml_tuple, 2, elt3.get_raw());
        OCaml::new(cr, ocaml_tuple)
    }
}

pub fn alloc_tuple_4<'a, F, S, T3, T4>(
    cr: &'a mut OCamlRuntime,
    fst: OCamlRef<F>,
    snd: OCamlRef<S>,
    elt3: OCamlRef<T3>,
    elt4: OCamlRef<T4>,
) -> OCaml<'a, (F, S, T3, T4)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(4);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        store_field(ocaml_tuple, 2, elt3.get_raw());
        store_field(ocaml_tuple, 3, elt4.get_raw());
        OCaml::new(cr, ocaml_tuple)
    }
}

pub fn alloc_cons<'a, A>(
    cr: &'a mut OCamlRuntime,
    head: OCamlRef<A>,
    tail: OCamlRef<OCamlList<A>>,
) -> OCaml<'a, OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::CONS);
        store_field(ocaml_cons, 0, head.get_raw());
        store_field(ocaml_cons, 1, tail.get_raw());
        OCaml::new(cr, ocaml_cons)
    }
}
