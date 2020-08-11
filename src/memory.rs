use crate::mlvalues::tag;
use crate::mlvalues::{Intnat, MlsizeT, OCamlList, RawOCaml};
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
    fn caml_alloc(wosize: MlsizeT, tag: tag::Tag) -> RawOCaml;
    fn caml_alloc_tuple(wosize: MlsizeT) -> RawOCaml;
    fn caml_modify(block: *mut RawOCaml, val: RawOCaml);
}

// #define Store_field(block, offset, val) do{ \
//     mlsize_t caml__temp_offset = (offset); \
//     value caml__temp_val = (val); \
//     caml_modify (&Field ((block), caml__temp_offset), caml__temp_val); \
//   }while(0)
#[inline]
unsafe fn store_field(block: RawOCaml, offset: MlsizeT, val: RawOCaml) {
    // TODO: see if all this can be made prettier
    let ptr = block as *mut isize;
    caml_modify(ptr.add(offset), val);
}

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

    pub fn keep<T>(&self, value: OCaml<T>) -> OCamlRef<'gc, T> {
        OCamlRef::new(self, value)
    }

    pub fn get<'tmp, T>(&'tmp self, reference: &OCamlRef<T>) -> OCaml<'tmp, T> {
        make_ocaml(reference.cell.get())
    }

    pub unsafe fn token(&self) -> GCToken {
        GCToken {}
    }
}

impl<'gc> Drop for GCFrame<'gc> {
    fn drop(&mut self) {
        assert!(self.block.nitems == 0);
        // In case this happens whith a GCFrame that was not initialized.
        // We don't want to mess with caml_local_roots in that case.
        if !self.block.next.is_null() {
            unsafe {
                assert!(caml_local_roots == &mut self.block);
                caml_local_roots = self.block.next;
            }
        }
    }
}

// Token used for allocation functions.
pub struct GCToken {}

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

// A reference to an OCaml value. This location is tracked by the GC.
pub struct OCamlRef<'a, T> {
    cell: &'a Cell<RawOCaml>,
    _marker: marker::PhantomData<Cell<T>>,
}

impl<'a, T> OCamlRef<'a, T> {
    pub fn new<'gc>(gc: &GCFrame<'gc>, x: OCaml<T>) -> OCamlRef<'gc, T> {
        let cell: &'gc Cell<RawOCaml> = unsafe { reserve_local_root_cell(gc) };
        cell.set(unsafe { x.raw() });
        OCamlRef {
            _marker: Default::default(),
            cell,
        }
    }

    pub fn set(&mut self, x: OCaml<T>) {
        self.cell.set(unsafe { x.raw() });
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
            raw,
        }
    }

    pub fn of_ocaml(v: OCaml<T>) -> GCResult<T> {
        GCResult {
            _marker: Default::default(),
            raw: unsafe { v.raw() },
        }
    }

    pub fn mark(self, _gc: &mut GCFrame) -> GCMarkedResult<T> {
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

pub fn alloc_bytes(_token: GCToken, s: &[u8]) -> GCResult<String> {
    GCResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}

pub fn alloc_string(_token: GCToken, s: &str) -> GCResult<String> {
    GCResult::of(unsafe { caml_alloc_initialized_string(s.len(), s.as_ptr()) })
}

// TODO: it is possible to directly alter the fields memory upon first allocation of
// small values (like tuples and conses are) without going through `caml_modify` to get
// a little bit of extra performance.

pub fn alloc_tuple<F, S>(_token: GCToken, fst: OCaml<F>, snd: OCaml<S>) -> GCResult<(F, S)> {
    unsafe {
        let ocaml_tuple = caml_alloc_tuple(2);
        store_field(ocaml_tuple, 0, fst.raw());
        store_field(ocaml_tuple, 1, snd.raw());
        GCResult::of(ocaml_tuple)
    }
}

pub fn alloc_cons<A>(
    _token: GCToken,
    head: OCaml<A>,
    tail: OCaml<OCamlList<A>>,
) -> GCResult<OCamlList<A>> {
    unsafe {
        let ocaml_cons = caml_alloc(2, tag::LIST);
        store_field(ocaml_cons, 0, head.raw());
        store_field(ocaml_cons, 1, tail.raw());
        GCResult::of(ocaml_cons)
    }
}
