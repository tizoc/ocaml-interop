// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::{
    conv::FromOCaml,
    mlvalues::{tag, Intnat, OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList, RawOCaml},
    runtime::OCamlRuntime,
    value::OCaml,
};
use core::{cell::Cell, marker::PhantomData, ptr};
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
    pub fn initialize(&mut self, frame_local_roots: &[Cell<RawOCaml>]) -> &mut Self {
        self.block.local_roots = frame_local_roots[0].as_ptr();
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

pub struct OCamlRoot<'a> {
    cell: &'a Cell<RawOCaml>,
}

impl<'a> OCamlRoot<'a> {
    #[doc(hidden)]
    pub unsafe fn reserve<'gc>(_gc: &GCFrame<'gc>) -> OCamlRoot<'gc> {
        assert_eq!(&_gc.block as *const _, local_roots());
        let block = &mut *local_roots();
        let locals: *const Cell<RawOCaml> = &*(block.local_roots as *const Cell<RawOCaml>);
        let cell = &*locals.offset(block.nitems);
        block.nitems += 1;
        OCamlRoot { cell }
    }

    /// Roots an [`OCaml`] value.
    pub fn keep<'tmp, T>(&'tmp mut self, val: OCaml<T>) -> OCamlRooted<'tmp, T> {
        self.cell.set(unsafe { val.raw() });
        OCamlRooted {
            _marker: PhantomData,
            cell: self.cell,
        }
    }

    /// Roots a [`RawOCaml`] value and attaches a type to it.
    ///
    /// # Safety
    ///
    /// This method is unsafe because there is no way to validate that the [`RawOCaml`] value
    /// is of the correct type.
    pub unsafe fn keep_raw<T>(&mut self, val: RawOCaml) -> OCamlRooted<T> {
        self.cell.set(val);
        OCamlRooted {
            _marker: PhantomData,
            cell: self.cell,
        }
    }
}

/// An `OCamlRooted<T>` value is the result of rooting [`OCaml`]`<T>` value using a root variable.
///
/// Rooted values can be used to recover a fresh reference to an [`OCaml`]`<T>` value what would
/// otherwise become stale after a call to the OCaml runtime.
pub struct OCamlRooted<'a, T> {
    pub(crate) cell: &'a Cell<RawOCaml>,
    _marker: PhantomData<Cell<T>>,
}

impl<'a, T> OCamlRooted<'a, T> {
    /// Converts this value into a Rust value.
    pub fn to_rust<RustT>(&self, cr: &OCamlRuntime) -> RustT
    where
        RustT: FromOCaml<T>,
    {
        RustT::from_ocaml(cr.get(self))
    }

    /// Updates the value of this GC tracked reference.
    pub fn set(&mut self, x: OCaml<T>) {
        self.cell.set(unsafe { x.raw() });
    }

    /// Gets the raw value contained by this reference.
    pub unsafe fn get_raw(&self) -> RawOCaml {
        self.cell.get()
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

pub fn alloc_int32<'a>(cr: &'a mut OCamlRuntime, i: i32) -> OCaml<'a, OCamlInt32> {
    unsafe { OCaml::new(cr, caml_copy_int32(i)) }
}

pub fn alloc_int64<'a>(cr: &'a mut OCamlRuntime, i: i64) -> OCaml<'a, OCamlInt64> {
    unsafe { OCaml::new(cr, caml_copy_int64(i)) }
}

pub fn alloc_double<'a>(cr: &'a mut OCamlRuntime, d: f64) -> OCaml<'a, OCamlFloat> {
    unsafe { OCaml::new(cr, caml_copy_double(d)) }
}

// TODO: it is possible to directly alter the fields memory upon first allocation of
// small values (like tuples and conses are) without going through `caml_modify` to get
// a little bit of extra performance.

pub fn alloc_some<'a, A>(cr: &'a mut OCamlRuntime, value: &OCamlRooted<A>) -> OCaml<'a, Option<A>> {
    unsafe {
        let ocaml_some = caml_alloc(1, tag::SOME);
        store_field(ocaml_some, 0, value.get_raw());
        OCaml::new(cr, ocaml_some)
    }
}

pub fn alloc_tuple<'a, F, S>(
    cr: &'a mut OCamlRuntime,
    fst: &OCamlRooted<F>,
    snd: &OCamlRooted<S>,
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
    fst: &OCamlRooted<F>,
    snd: &OCamlRooted<S>,
    elt3: &OCamlRooted<T3>,
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
    fst: &OCamlRooted<F>,
    snd: &OCamlRooted<S>,
    elt3: &OCamlRooted<T3>,
    elt4: &OCamlRooted<T4>,
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
    head: &OCamlRooted<A>,
    tail: &OCamlRooted<OCamlList<A>>,
) -> OCaml<'a, OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::CONS);
        store_field(ocaml_cons, 0, head.get_raw());
        store_field(ocaml_cons, 1, tail.get_raw());
        OCaml::new(cr, ocaml_cons)
    }
}
