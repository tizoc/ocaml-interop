use znfe::{ocaml_alloc, ocaml_export, FromOCaml, Intnat, OCaml, OCamlBytes, OCamlList, ToOCaml};

ocaml_export! {
    fn rust_twice(_gc, num: OCaml<Intnat>) -> OCaml<Intnat> {
        let num = i64::from_ocaml(num);
        OCaml::of_int(num * 2)
    }

    fn rust_increment_bytes(gc, bytes: OCaml<OCamlBytes>, first_n: OCaml<Intnat>) -> OCaml<OCamlBytes> {
        let first_n = i64::from_ocaml(first_n) as usize;
        let mut vec = Vec::from_ocaml(bytes);

        for i in 0..first_n {
            vec[i] += 1;
        }

        ocaml_alloc!(vec.to_ocaml(gc))
    }

    fn rust_increment_ints_list(gc, ints: OCaml<OCamlList<Intnat>>) -> OCaml<OCamlList<Intnat>> {
        let mut vec = <Vec<i64>>::from_ocaml(ints);

        for i in 0..vec.len() {
            vec[i] += 1;
        }

        ocaml_alloc!(vec.to_ocaml(gc))
    }

    fn rust_make_tuple(gc, fst: OCaml<String>, snd: OCaml<Intnat>) -> OCaml<(String, Intnat)> {
        let fst = String::from_ocaml(fst);
        let snd = i64::from_ocaml(snd);
        let tuple = (fst, snd);
        ocaml_alloc!(tuple.to_ocaml(gc))
    }

    fn rust_make_some(gc, value: OCaml<String>) -> OCaml<Option<String>> {
        let value = String::from_ocaml(value);
        let some_value = Some(value);
        ocaml_alloc!(some_value.to_ocaml(gc))
    }
}
