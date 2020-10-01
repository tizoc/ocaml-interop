// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::tag;
use crate::mlvalues::{Intnat, MlsizeT, OCamlBytes, OCamlInt32, OCamlInt64, OCamlList, RawOCaml};
use crate::value::{make_ocaml, OCaml};
use std::cell::Cell;
use std::marker;
use std::ptr;

// Structure representing a block in the list of OCaml's GC local roots.
#[repr(C)]
struct CamlRootsBlock {
    next: *mut CamlRootsBlock,
    ntables: Intnat,
    nitems: Intnat,
    tables: [*mut RawOCaml; 5],
}

impl Default for CamlRootsBlock {
    fn default() -> Self {
        CamlRootsBlock {
            next: ptr::null_mut(),
            ntables: 0,
            nitems: 0,
            tables: [ptr::null_mut(); 5],
        }
    }
}

// TODO: like in caml-oxide, local root slots are reserved in a way equivalent
// to CAMLlocalN, with a fixed size of 8 slots. Look into using an hybrid approach
// for when we know there will be less than 5 slots required, and also handle
// more than 8 slots gracefully.
const LOCALS_BLOCK_SIZE: usize = 8;
type LocalsBlock = [Cell<RawOCaml>; LOCALS_BLOCK_SIZE];

extern "C" {
    static mut caml_local_roots: *mut CamlRootsBlock;

    fn caml_alloc_initialized_string(len: MlsizeT, contents: *const u8) -> RawOCaml;
    pub fn caml_alloc(wosize: MlsizeT, tag: tag::Tag) -> RawOCaml;
    fn caml_alloc_tuple(wosize: MlsizeT) -> RawOCaml;
    fn caml_copy_int32(i: i32) -> RawOCaml;
    fn caml_copy_int64(i: i64) -> RawOCaml;
    fn caml_copy_double(d: f64) -> RawOCaml;
    fn caml_modify(block: *mut RawOCaml, val: RawOCaml);
}

// #define Store_field(block, offset, val) do{ \
//     mlsize_t caml__temp_offset = (offset); \
//     value caml__temp_val = (val); \
//     caml_modify (&Field ((block), caml__temp_offset), caml__temp_val); \
//   }while(0)
#[doc(hidden)]
#[inline]
pub unsafe fn store_field(block: RawOCaml, offset: MlsizeT, val: RawOCaml) {
    // TODO: see if all this can be made prettier
    let ptr = block as *mut isize;
    caml_modify(ptr.add(offset), val);
}

pub trait GCFrameHandle<'gc> {}

// OCaml GC frame handle
#[derive(Default)]
pub struct GCFrame<'gc> {
    _marker: marker::PhantomData<&'gc i32>,
    block: CamlRootsBlock,
    locals: LocalsBlock,
}

impl<'gc> GCFrame<'gc> {
    pub fn initialize(&mut self) -> &mut Self {
        self.block.tables[0] = self.locals[0].as_ptr();
        self.block.ntables = 1;
        unsafe {
            self.block.next = caml_local_roots;
            caml_local_roots = &mut self.block;
        };
        self
    }

    #[doc(hidden)]
    pub unsafe fn initialize_empty(&mut self) -> &mut Self {
        self
    }

    /// Returns a GC tracked reference to an OCaml value.
    pub fn keep<T>(&self, value: OCaml<T>) -> OCamlRef<'gc, T> {
        OCamlRef::new(self, value)
    }

    /// Returns a GC tracked reference to an raw OCaml pointer.
    pub fn keep_raw(&self, value: RawOCaml) -> OCamlRawRef<'gc> {
        OCamlRawRef::new(self, value)
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
        assert!(self.block.nitems == 0);
        assert!(self.block.ntables == 1);
        unsafe {
            assert!(caml_local_roots == &mut self.block);
            caml_local_roots = self.block.next;
        }
    }
}

// OCaml GC frame handle
#[derive(Default)]
pub struct GCFrameNoKeep<'gc> {
    _marker: marker::PhantomData<&'gc i32>,
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

unsafe fn reserve_local_root_cell<'gc>(_gc: &GCFrame<'gc>) -> &'gc Cell<RawOCaml> {
    let block = &mut *caml_local_roots;
    if (block.nitems as usize) < LOCALS_BLOCK_SIZE {
        let locals: &'gc LocalsBlock = &*(block.tables[0] as *const LocalsBlock);
        let pos = block.nitems;
        let cell = &locals[pos as usize];
        block.nitems = pos + 1;
        cell
    } else {
        panic!(
            "Out of local roots. Max is LOCALS_BLOCK_SIZE={}",
            LOCALS_BLOCK_SIZE
        );
    }
}

unsafe fn free_local_root_cell(cell: &Cell<RawOCaml>) {
    let block = &mut *caml_local_roots;
    assert!(block.tables[0].offset(block.nitems - 1) == cell.as_ptr());
    block.nitems -= 1;
}

/// `OCamlRef<T>` is a reference to an `OCaml<T>` value that is tracked by the GC.
///
/// Unlike `OCaml<T>` values, it can be re-referenced after OCaml allocations.
pub struct OCamlRef<'a, T> {
    cell: &'a Cell<RawOCaml>,
    _marker: marker::PhantomData<Cell<T>>,
}

/// Like `OCamlRef` but for `OCamlRaw` values.
pub struct OCamlRawRef<'a> {
    cell: &'a Cell<RawOCaml>,
}

impl<'a, T> OCamlRef<'a, T> {
    #[doc(hidden)]
    pub fn new<'gc>(gc: &GCFrame<'gc>, x: OCaml<T>) -> OCamlRef<'gc, T> {
        let cell: &'gc Cell<RawOCaml> = unsafe { reserve_local_root_cell(gc) };
        cell.set(unsafe { x.raw() });
        OCamlRef {
            _marker: Default::default(),
            cell,
        }
    }

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
    #[doc(hidden)]
    pub fn new<'gc>(gc: &GCFrame<'gc>, x: RawOCaml) -> OCamlRawRef<'gc> {
        let cell: &'gc Cell<RawOCaml> = unsafe { reserve_local_root_cell(gc) };
        cell.set(x);
        OCamlRawRef { cell }
    }

    /// Updates the raw value of this GC tracked reference.
    pub fn set_raw(&mut self, x: RawOCaml) {
        self.cell.set(x);
    }

    /// Gets the raw value contained by this reference.
    pub fn get_raw(&self) -> RawOCaml {
        self.cell.get()
    }
}

impl<'a, T> Drop for OCamlRef<'a, T> {
    fn drop(&mut self) {
        unsafe { free_local_root_cell(self.cell) }
    }
}

impl<'a> Drop for OCamlRawRef<'a> {
    fn drop(&mut self) {
        unsafe { free_local_root_cell(self.cell) }
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

pub fn alloc_double(_token: OCamlAllocToken, d: f64) -> OCamlAllocResult<f64> {
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
