use std::marker;

pub mod tag;

pub type UIntnat = usize;
pub type Intnat = isize;
pub type RawOCaml = isize;
pub type MlsizeT = UIntnat;

pub struct OCamlList<A> {
    _marker: marker::PhantomData<A>,
}

// #define Val_unit Val_int(0)
pub const UNIT: RawOCaml = unsafe { raw_ocaml_of_i64(0) };

// #define Val_none Val_int(0)
pub const NONE: RawOCaml = unsafe { raw_ocaml_of_i64(0) };

// #define Val_emptylist Val_int(0)
pub const EMPTY_LIST: RawOCaml = unsafe { raw_ocaml_of_i64(0) };

// #define Is_block(x)  (((x) & 1) == 0)
#[inline]
pub fn is_block(x: RawOCaml) -> bool {
    (x & 1) == 0
}

// #define Is_long(x)   (((x) & 1) != 0)
pub fn is_long(x: RawOCaml) -> bool {
    (x & 1) != 0
}

// #define Is_exception_result(v) (((v) & 3) == 2)
pub const fn is_exception_result(val: RawOCaml) -> bool {
    val & 3 == 2
}

// #define Extract_exception(v) ((v) & ~3)
pub const fn extract_exception(val: RawOCaml) -> RawOCaml {
    val & !3
}

// #define Hp_val(val) (((header_t *) (val)) - 1)
#[inline]
pub unsafe fn hd_val(x: RawOCaml) -> UIntnat {
    assert!(is_block(x));
    *(x as *const UIntnat).offset(-1)
}

#[inline]
pub unsafe fn wosize_val(x: RawOCaml) -> UIntnat {
    hd_val(x) >> 10
}

// #ifdef ARCH_BIG_ENDIAN
// #define Tag_val(val) (((unsigned char *) (val)) [-1])
#[cfg(target_endian = "big")]
#[inline]
pub unsafe fn tag_val(x: RawOCaml) -> tag::Tag {
    *(x as *const u8).offset(-1)
}

// #else
// #define Tag_val(val) (((unsigned char *) (val)) [-sizeof(value)])
#[cfg(target_endian = "little")]
#[inline]
pub unsafe fn tag_val(x: RawOCaml) -> tag::Tag {
    *(x as *const u8).offset(-(core::mem::size_of::<RawOCaml>() as isize))
}

// #define Bp_val(v) ((char *) (v))
#[inline]
unsafe fn bp_val(val: RawOCaml) -> *mut u8 {
    assert!(is_block(val));
    val as *mut u8
}

// #define String_val(x) ((const char *) Bp_val(x))
#[inline]
pub unsafe fn string_val(val: RawOCaml) -> *mut u8 {
    bp_val(val)
}

#[inline]
pub unsafe fn raw_ocaml_to_i64(raw: RawOCaml) -> i64 {
    assert!(!is_block(raw));
    (raw >> 1) as i64
}

#[inline]
pub const unsafe fn raw_ocaml_of_i64(n: i64) -> RawOCaml {
    ((n << 1) | 1) as RawOCaml
}
