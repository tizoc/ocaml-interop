extern crate znfe;

use znfe::{
    ocaml_alloc, ocaml_call, ocaml_frame, FromOCaml, Intnat, OCaml, ToOCaml, ToOCamlInteger
};

mod ocaml {
    use znfe::{ocaml, Intnat};

    ocaml!{
        pub fn increment_bytes(bytes: String, first_n: Intnat) -> String;
        pub fn twice(num: Intnat) -> Intnat;
        pub fn make_tuple(fst: String, snd: Intnat) -> (String, Intnat);
    }
}

pub fn increment_bytes(bytes: &str, first_n: usize) -> String {
    ocaml_frame!(gc, {
        let bytes = ocaml_alloc!(bytes.to_ocaml(gc));
        let ref bytes_ref = gc.keep(bytes);
        let first_n = ocaml_alloc!((first_n as i64).to_ocaml(gc));
        let result = ocaml_call!(ocaml::increment_bytes(gc, gc.get(bytes_ref), first_n));
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        String::from_ocaml(result)
    })
}

pub fn twice(num: i64) -> i64 {
    ocaml_frame!(gc, {
        let num = num.to_ocaml_fixnum();
        let result = ocaml_call!(ocaml::twice(gc, num));
        let result: OCaml<Intnat> = result.expect("Error in 'twice' call result");
        i64::from_ocaml(result)
    })
}

pub fn make_tuple(fst: String, snd: i64) -> (String, i64) {
    ocaml_frame!(gc, {
        let num = snd.to_ocaml_fixnum();
        let str = ocaml_alloc!(fst.to_ocaml(gc));
        let result = ocaml_call!(ocaml::make_tuple(gc, str, num));
        let result: OCaml<(String, Intnat)> = result.expect("Error in 'make_tuple' call result");
        <(String, i64)>::from_ocaml(result)
    })
}

// Tests

// NOTE: required because at the moment, no synchronization is done on OCaml calls
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_twice() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(twice(10), 20);
}

#[test]
#[serial]
fn test_increment_bytes() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(increment_bytes("0000000000000000", 10), "1111111111000000");
}

#[test]
#[serial]
fn test_make_tuple() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(make_tuple("fst".to_owned(), 9), ("fst".to_owned(), 9));
}