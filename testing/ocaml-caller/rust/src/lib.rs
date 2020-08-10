use znfe::{ocaml_alloc, ocaml_export, FromOCaml, Intnat, OCaml, ToOCaml, ToOCamlInteger};

ocaml_export! {
    fn rust_twice(_gc, num: OCaml<Intnat>) -> OCaml<Intnat> {
        let num = i64::from_ocaml(num);
        (num * 2).to_ocaml_fixnum()
    }

    fn rust_increment_bytes(gc, bytes: OCaml<String>, first_n: OCaml<Intnat>) -> OCaml<String> {
        let first_n = i64::from_ocaml(first_n) as usize;
        let mut vec = Vec::from_ocaml(bytes);

        for i in 0..first_n {
            vec[i] += 1;
        }

        ocaml_alloc!(vec.to_ocaml(gc))
    }

    fn rust_make_tuple(gc, fst: OCaml<String>, snd: OCaml<Intnat>) -> OCaml<(String, Intnat)> {
        let fst = String::from_ocaml(fst);
        let snd = i64::from_ocaml(snd);
        let tuple = (&fst, snd);
        ocaml_alloc!(tuple.to_ocaml(gc))
    }
}
