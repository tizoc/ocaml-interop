use mlvalues::{Intnat, RawOCaml};
use std::cell::Cell;
use std::marker;
use std::ptr;
use value::{make_ocaml, OCaml};

// Structure representing a block in the list of OCaml's GC local roots.
#[repr(C)]
struct CamlRootsBlock {
    next: *mut CamlRootsBlock,
    ntables: Intnat,
    nitems: Intnat,
    tables: [*mut RawOCaml; 5],
}

impl Default for CamlRootsBlock {
    fn default() -> CamlRootsBlock {
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

    fn caml_alloc_initialized_string(len: usize, contents: *const u8) -> RawOCaml;
}

// OCaml GC frame handle
pub struct GCFrame<'gc> {
    _marker: marker::PhantomData<&'gc i32>,
}

// Token used for allocation functions.
pub struct GCtoken {}

unsafe fn reserve_local_root_cell<'gc>(_gc: &GCFrame<'gc>) -> &'gc Cell<RawOCaml> {
    let block = &mut *caml_local_roots;
    if (block.nitems as usize) < LOCALS_BLOCK_SIZE {
        let locals: &'gc LocalsBlock = &*(block.tables[0] as *const LocalsBlock);
        let pos = block.nitems;
        let cell = &locals[pos as usize];
        block.nitems = pos + 1;
        cell
    } else {
        panic!("Out of local roots");
    }
}

unsafe fn free_local_root_cell(cell: &Cell<RawOCaml>) {
    let block = &mut *caml_local_roots;
    assert!(block.tables[0].offset(block.nitems - 1) == cell.as_ptr());
    block.nitems -= 1;
}

// Runs a block of code inside a fresh GC frame.
// A new local roots block is pushed and then popped, with `body` executed in-between.
// NOTE: right now a max of LOCAL_BLOCK_SIZE GC-tracked variables are supported.
pub fn with_frame<'a, F>(body: F) -> RawOCaml
where
    F: Fn(&mut GCFrame) -> RawOCaml,
{
    let mut gc = GCFrame {
        _marker: Default::default(),
    };
    let locals: LocalsBlock = Default::default();
    unsafe {
        let mut block: CamlRootsBlock = Default::default();
        block.next = caml_local_roots;
        block.ntables = 1;
        block.tables[0] = locals[0].as_ptr();
        caml_local_roots = &mut block;
        let result = body(&mut gc);
        assert!(caml_local_roots == &mut block);
        assert!(block.nitems == 0);
        caml_local_roots = block.next;
        result
    }
}

// A reference to an OCaml value. This location is tracked by the GC.
pub struct OCamlRef<'a, T> {
    cell: &'a Cell<RawOCaml>,
    _marker: marker::PhantomData<Cell<T>>,
}

impl<'a, T> OCamlRef<'a, T> {
    pub fn new<'gc, 'tmp>(gc: &'a GCFrame<'gc>, x: OCaml<'tmp, T>) -> OCamlRef<'gc, T> {
        let cell: &'gc Cell<RawOCaml> = unsafe { reserve_local_root_cell(gc) };
        cell.set(x.into());
        OCamlRef {
            _marker: Default::default(),
            cell: cell,
        }
    }

    pub fn get<'gc, 'tmp>(&'a self, _gc: &'tmp GCFrame<'gc>) -> OCaml<'tmp, T> {
        make_ocaml(self.cell.get())
    }
}

impl<'a, T> Drop for OCamlRef<'a, T> {
    fn drop(&mut self) {
        unsafe { free_local_root_cell(self.cell) }
    }
}

// Intermediary allocation result.
pub struct GCResult<T> {
    raw: RawOCaml,
    _marker: marker::PhantomData<T>,
}

// Allocation result that has been marked by the GC.
pub struct GCMarkedResult<T> {
    raw: RawOCaml,
    _marker: marker::PhantomData<T>,
}

impl<T> GCResult<T> {
    pub fn of(raw: RawOCaml) -> GCResult<T> {
        GCResult {
            _marker: Default::default(),
            raw: raw,
        }
    }

    pub fn mark<'gc>(self, _gc: &mut GCFrame<'gc>) -> GCMarkedResult<T> {
        GCMarkedResult {
            _marker: Default::default(),
            raw: self.raw,
        }
    }
}

impl<T> GCMarkedResult<T> {
    pub fn eval<'a, 'gc: 'a>(self, _gc: &'a GCFrame<'gc>) -> OCaml<'a, T> {
        make_ocaml(self.raw)
    }
}

pub fn alloc_bytes(_token: GCtoken, s: &[u8]) -> GCResult<String> {
    GCResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}

pub fn alloc_string(_token: GCtoken, s: &str) -> GCResult<String> {
    GCResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}
