// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::{
    tag, Intnat, MlsizeT, OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList, RawOCaml,
};
use crate::value::{make_ocaml, OCaml};
pub use ocaml_sys::{
    caml_alloc, local_roots as ocaml_sys_local_roots, set_local_roots as ocaml_sys_set_local_roots,
    store_field,
};
use ocaml_sys::{caml_alloc_tuple, caml_copy_double, caml_copy_int32, caml_copy_int64};
use std::cell::Cell;
use std::marker;
use std::ptr;

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

// This definition is neede for now because ocaml-sys doesn't provide it.
extern "C" {
    fn caml_alloc_initialized_string(len: MlsizeT, contents: *const u8) -> RawOCaml;
}

// Overrides for ocaml-sys functions of the same name but using ocaml-interop's CamlRootBlocks representation.
unsafe fn local_roots() -> *mut CamlRootsBlock {
    ocaml_sys_local_roots() as *mut CamlRootsBlock
}

unsafe fn set_local_roots(roots: *mut CamlRootsBlock) {
    ocaml_sys_set_local_roots(roots as *mut ocaml_sys::CamlRootsBlock)
}

pub trait GCFrameHandle<'gc> {}

// OCaml GC frame handle
#[derive(Default)]
pub struct GCFrame<'gc> {
    _marker: marker::PhantomData<&'gc i32>,
    block: CamlRootsBlock,
}

// OCaml GC frame handle
#[derive(Default)]
pub struct GCFrameNoKeep<'gc> {
    _marker: marker::PhantomData<&'gc i32>,
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

    /// Returns the OCaml valued to which this GC tracked reference points to.
    pub fn get<'tmp, T>(&'tmp self, reference: &OCamlRef<T>) -> OCaml<'tmp, T> {
        make_ocaml(reference.cell.get())
    }

    #[doc(hidden)]
    pub unsafe fn token(&self) -> OCamlAllocToken {
        OCamlAllocToken {}
    }
}

impl<'gc> Drop for GCFrame<'gc> {
    fn drop(&mut self) {
        unsafe {
            assert!(local_roots() == &mut self.block);
            set_local_roots(self.block.next);
        }
    }
}

impl<'gc> GCFrameNoKeep<'gc> {
    pub fn initialize(&mut self) -> &mut Self {
        self
    }

    #[doc(hidden)]
    pub unsafe fn token(&self) -> OCamlAllocToken {
        OCamlAllocToken {}
    }
}

impl<'gc> GCFrameHandle<'gc> for GCFrame<'gc> {}
impl<'gc> GCFrameHandle<'gc> for GCFrameNoKeep<'gc> {}

/// Token used by allocation functions. Used internally.
pub struct OCamlAllocToken {}

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

    pub fn keep<'tmp, T>(&'tmp mut self, val: OCaml<T>) -> OCamlRef<'tmp, T> {
        self.cell.set(unsafe { val.raw() });
        OCamlRef {
            _marker: Default::default(),
            cell: self.cell,
        }
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn keep_raw<'tmp>(&'tmp mut self, val: RawOCaml) -> OCamlRawRef<'tmp> {
        self.cell.set(val);
        OCamlRawRef { cell: self.cell }
    }
}

/// `OCamlRef<T>` is a reference to an [`OCaml`]`<T>` value that is tracked by the GC.
///
/// Unlike [`OCaml`]`<T>` values, it can be re-referenced after OCaml allocations.
pub struct OCamlRef<'a, T> {
    cell: &'a Cell<RawOCaml>,
    _marker: marker::PhantomData<Cell<T>>,
}

/// Like [`OCamlRef`] but for [`RawOCaml`] values.
pub struct OCamlRawRef<'a> {
    cell: &'a Cell<RawOCaml>,
}

impl<'a, T> OCamlRef<'a, T> {
    /// Updates the value of this GC tracked reference.
    pub fn set(&mut self, x: OCaml<T>) {
        self.cell.set(unsafe { x.raw() });
    }

    /// Gets the raw value contained by this reference.
    pub fn get_raw(&self) -> RawOCaml {
        self.cell.get()
    }
}

impl<'a> OCamlRawRef<'a> {
    /// Updates the raw value of this GC tracked reference.
    pub fn set_raw(&mut self, x: RawOCaml) {
        self.cell.set(x);
    }

    /// Gets the raw value contained by this reference.
    pub fn get_raw(&self) -> RawOCaml {
        self.cell.get()
    }
}

/// Intermediary allocation result.
pub struct OCamlAllocResult<T> {
    raw: RawOCaml,
    _marker: marker::PhantomData<T>,
}

/// Allocation result that has been marked by the GC.
pub struct GCMarkedResult<T> {
    raw: RawOCaml,
    _marker: marker::PhantomData<T>,
}

impl<T> OCamlAllocResult<T> {
    pub fn of(raw: RawOCaml) -> OCamlAllocResult<T> {
        OCamlAllocResult {
            _marker: Default::default(),
            raw,
        }
    }

    pub fn of_ocaml(v: OCaml<T>) -> OCamlAllocResult<T> {
        OCamlAllocResult {
            _marker: Default::default(),
            raw: unsafe { v.raw() },
        }
    }

    pub fn mark(self, _gc: &mut dyn GCFrameHandle) -> GCMarkedResult<T> {
        GCMarkedResult {
            _marker: Default::default(),
            raw: self.raw,
        }
    }
}

impl<T> GCMarkedResult<T> {
    pub fn eval<'a, 'gc: 'a>(self, _gc: &'a dyn GCFrameHandle<'gc>) -> OCaml<'a, T> {
        make_ocaml(self.raw)
    }
}

pub fn alloc_bytes(_token: OCamlAllocToken, s: &[u8]) -> OCamlAllocResult<OCamlBytes> {
    OCamlAllocResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}

pub fn alloc_string(_token: OCamlAllocToken, s: &str) -> OCamlAllocResult<String> {
    OCamlAllocResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}

pub fn alloc_int32(_token: OCamlAllocToken, i: i32) -> OCamlAllocResult<OCamlInt32> {
    OCamlAllocResult::of(unsafe { caml_copy_int32(i) })
}

pub fn alloc_int64(_token: OCamlAllocToken, i: i64) -> OCamlAllocResult<OCamlInt64> {
    OCamlAllocResult::of(unsafe { caml_copy_int64(i) })
}

pub fn alloc_double(_token: OCamlAllocToken, d: f64) -> OCamlAllocResult<OCamlFloat> {
    OCamlAllocResult::of(unsafe { caml_copy_double(d) })
}

// TODO: it is possible to directly alter the fields memory upon first allocation of
// small values (like tuples and conses are) without going through `caml_modify` to get
// a little bit of extra performance.

pub fn alloc_some<A>(_token: OCamlAllocToken, value: &OCamlRef<A>) -> OCamlAllocResult<Option<A>> {
    unsafe {
        let ocaml_some = caml_alloc(1, tag::SOME);
        store_field(ocaml_some, 0, value.get_raw());
        OCamlAllocResult::of(ocaml_some)
    }
}

pub fn alloc_tuple<F, S>(
    _token: OCamlAllocToken,
    fst: &OCamlRef<F>,
    snd: &OCamlRef<S>,
) -> OCamlAllocResult<(F, S)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(2);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        OCamlAllocResult::of(ocaml_tuple)
    }
}

pub fn alloc_tuple_3<F, S, T3>(
    _token: OCamlAllocToken,
    fst: &OCamlRef<F>,
    snd: &OCamlRef<S>,
    elt3: &OCamlRef<T3>,
) -> OCamlAllocResult<(F, S, T3)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(3);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        store_field(ocaml_tuple, 2, elt3.get_raw());
        OCamlAllocResult::of(ocaml_tuple)
    }
}

pub fn alloc_tuple_4<F, S, T3, T4>(
    _token: OCamlAllocToken,
    fst: &OCamlRef<F>,
    snd: &OCamlRef<S>,
    elt3: &OCamlRef<T3>,
    elt4: &OCamlRef<T4>,
) -> OCamlAllocResult<(F, S, T3, T4)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(4);
        store_field(ocaml_tuple, 0, fst.get_raw());
        store_field(ocaml_tuple, 1, snd.get_raw());
        store_field(ocaml_tuple, 2, elt3.get_raw());
        store_field(ocaml_tuple, 3, elt4.get_raw());
        OCamlAllocResult::of(ocaml_tuple)
    }
}

pub fn alloc_cons<A>(
    _token: OCamlAllocToken,
    head: &OCamlRef<A>,
    tail: &OCamlRef<OCamlList<A>>,
) -> OCamlAllocResult<OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::LIST);
        store_field(ocaml_cons, 0, head.get_raw());
        store_field(ocaml_cons, 1, tail.get_raw());
        OCamlAllocResult::of(ocaml_cons)
    }
}
