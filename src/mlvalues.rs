use crate::value::caml_string_length;

pub mod tag;

pub type UIntnat = usize;
pub type Intnat = isize;
pub type RawOCaml = isize;

// #define Is_block(x)  (((x) & 1) == 0)
#[inline]
pub fn is_block(x: RawOCaml) -> bool {
    (x & 1) == 0
}

// #define Is_long(x)   (((x) & 1) != 0)
//pub fn is_long(x: RawOCaml) -> bool {
//    (x & 1) != 0
//}

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

pub unsafe fn raw_ocaml_to_vecu8(raw: RawOCaml) -> Vec<u8> {
    assert!(tag_val(raw) == tag::STRING);
    let len = caml_string_length(raw);
    let mut vec: Vec<u8> = Vec::with_capacity(len);
    vec.extend_from_slice(std::slice::from_raw_parts(raw as *const u8, len));
    vec
}

pub unsafe fn raw_ocaml_to_string(raw: RawOCaml) -> String {
    String::from_utf8_unchecked(raw_ocaml_to_vecu8(raw))
}

#[inline]
pub unsafe fn raw_ocaml_of_i64(n: i64) -> RawOCaml {
    ((n << 1) | 1) as RawOCaml
}
