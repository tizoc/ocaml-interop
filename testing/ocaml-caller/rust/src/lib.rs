use znfe::{with_frame, alloc_ocaml, FromOCaml, Intnat, OCaml, RawOCaml, ToOCaml, ToOCamlInteger};

#[no_mangle]
pub fn rust_twice(num: OCaml<'static, Intnat>) -> RawOCaml {
    let num = i64::from_ocaml(num);
    (num * 2).to_ocaml_fixnum().into()
}

#[no_mangle]
pub fn rust_increment_bytes(bytes: OCaml<String>, first_n: OCaml<'static, Intnat>) -> RawOCaml {
    let first_n = i64::from_ocaml(first_n) as usize;
    let mut vec = Vec::from_ocaml(bytes);

    for i in 0..first_n {
        vec[i] += 1;
    }

    with_frame(|gc| {
        let output = alloc_ocaml! {vec.to_ocaml(gc)};
        output.into()
    })
}
