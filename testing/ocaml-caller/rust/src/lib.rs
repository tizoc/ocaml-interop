use znfe::{ocaml_alloc, ocaml_frame, FromOCaml, Intnat, OCaml, RawOCaml, ToOCaml, ToOCamlInteger};

#[no_mangle]
pub extern "C" fn rust_twice(num: OCaml<'static, Intnat>) -> RawOCaml {
    let num = i64::from_ocaml(num);
    unsafe { (num * 2).to_ocaml_fixnum().raw() }
}

#[no_mangle]
pub extern "C" fn rust_increment_bytes(bytes: OCaml<String>, first_n: OCaml<'static, Intnat>) -> RawOCaml {
    let first_n = i64::from_ocaml(first_n) as usize;
    let mut vec = Vec::from_ocaml(bytes);

    for i in 0..first_n {
        vec[i] += 1;
    }

    ocaml_frame!(gc, {
        let output = ocaml_alloc!(vec.to_ocaml(gc));
        unsafe { output.raw() }
    })
}
