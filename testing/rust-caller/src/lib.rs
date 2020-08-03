extern crate znfe;

use znfe::{ocaml_alloc, ocaml_call, ocaml_frame, FromOCaml, Intnat, OCaml, ToOCaml, ToOCamlInteger};

mod ocaml {
    use lazy_static::lazy_static;
    use znfe::OCamlClosure;

    lazy_static! {
        pub static ref TWICE: OCamlClosure =
            OCamlClosure::named("twice").expect("Missing 'twice' function");
        pub static ref INCREMENT_BYTES: OCamlClosure =
            OCamlClosure::named("increment_bytes").expect("Missing 'increment_bytes' function");
    }
}

pub fn increment_bytes(bytes: &str, first_n: usize) -> String {
    ocaml_frame!(gc, {
        let bytes = ocaml_alloc! {bytes.to_ocaml(gc)};
        let ref bytes_ref = gc.keep(bytes);
        let first_n = ocaml_alloc! {(first_n as i64).to_ocaml(gc)};
        let result = ocaml_call! {ocaml::INCREMENT_BYTES(gc, gc.get(bytes_ref), first_n)};
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        String::from_ocaml(result)
    })
}

pub fn twice(num: i64) -> i64 {
    ocaml_frame!(gc, {
        let num = num.to_ocaml_fixnum();
        let result = ocaml_call! {ocaml::TWICE(gc, num)};
        let result: OCaml<Intnat> = result.expect("Error in 'twice' call result");
        i64::from_ocaml(result)
    })
}

// Tests

// NOTE: required because at the moment, no synchronization is done on OCaml calls
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_twice() {
    znfe::init_runtime();
    assert_eq!(twice(10), 20);
}

#[test]
#[serial]
fn test_increment_bytes() {
    znfe::init_runtime();
    assert_eq!(increment_bytes("0000000000000000", 10), "1111111111000000");
}
