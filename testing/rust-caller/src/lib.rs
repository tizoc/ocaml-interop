// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

extern crate ocaml_interop;

use ocaml_interop::{
    ocaml_call, ocaml_frame, to_ocaml, OCaml, OCamlBytes, OCamlInt, OCamlList, OCamlRuntime,
    ToOCaml, ToRust,
};

mod ocaml {
    use ocaml_interop::{
        impl_to_ocaml_record, impl_to_ocaml_variant, ocaml, OCamlFloat, OCamlInt, OCamlInt32,
        OCamlInt64, OCamlList,
    };

    pub struct TestRecord {
        pub i: i64,
        pub f: f64,
        pub i32: i32,
        pub i64: Box<i64>,
        pub s: String,
        pub t: (i64, f64),
    }

    pub enum Movement {
        Step(i64),
        RotateLeft,
        RotateRight,
    }

    impl_to_ocaml_record! {
        TestRecord {
            i: OCamlInt,
            f: OCamlFloat,
            i32: OCamlInt32,
            i64: OCamlInt64,
            s: String,
            t: (OCamlInt, OCamlFloat),
        }
    }

    impl_to_ocaml_variant! {
        Movement {
            Movement::Step(count: OCamlInt),
            Movement::RotateLeft,
            Movement::RotateRight,
        }
    }

    ocaml! {
        pub fn increment_bytes(bytes: String, first_n: OCamlInt) -> String;
        pub fn increment_ints_list(ints: OCamlList<OCamlInt>) -> OCamlList<OCamlInt>;
        pub fn twice(num: OCamlInt) -> OCamlInt;
        pub fn make_tuple(fst: String, snd: OCamlInt) -> (String, OCamlInt);
        pub fn make_some(value: String) -> Option<String>;
        pub fn make_ok(value: OCamlInt) -> Result<OCamlInt, String>;
        pub fn make_error(value: String) -> Result<OCamlInt, String>;
        pub fn stringify_record(record: TestRecord) -> String;
        pub fn stringify_variant(variant: Movement) -> String;
    }
}

pub fn increment_bytes(cr: &mut OCamlRuntime, bytes: &str, first_n: usize) -> String {
    ocaml_frame!(cr(bytes_root), {
        let bytes = to_ocaml!(cr, bytes, bytes_root);
        let first_n = to_ocaml!(cr, first_n as i64);
        let result = ocaml_call!(ocaml::increment_bytes(cr, cr.get(&bytes), first_n));
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        result.to_rust()
    })
}

pub fn increment_ints_list(cr: &mut OCamlRuntime, ints: &Vec<i64>) -> Vec<i64> {
    let ints = to_ocaml!(cr, ints);
    let result = ocaml_call!(ocaml::increment_ints_list(cr, ints));
    let result: OCaml<OCamlList<OCamlInt>> =
        result.expect("Error in 'increment_ints_list' call result");
    result.to_rust()
}

pub fn twice(cr: &mut OCamlRuntime, num: i64) -> i64 {
    let num = unsafe { OCaml::of_i64_unchecked(num) };
    let result = ocaml_call!(ocaml::twice(cr, num));
    let result: OCaml<OCamlInt> = result.expect("Error in 'twice' call result");
    result.to_rust()
}

pub fn make_tuple(cr: &mut OCamlRuntime, fst: String, snd: i64) -> (String, i64) {
    let num = unsafe { OCaml::of_i64_unchecked(snd) };
    let str = to_ocaml!(cr, fst);
    let result = ocaml_call!(ocaml::make_tuple(cr, str, num));
    let result: OCaml<(String, OCamlInt)> = result.expect("Error in 'make_tuple' call result");
    result.to_rust()
}

pub fn make_some(cr: &mut OCamlRuntime, value: String) -> Option<String> {
    let str = to_ocaml!(cr, value);
    let result = ocaml_call!(ocaml::make_some(cr, str));
    let result: OCaml<Option<String>> = result.expect("Error in 'make_some' call result");
    result.to_rust()
}

pub fn make_ok(cr: &mut OCamlRuntime, value: i64) -> Result<i64, String> {
    let result = to_ocaml!(cr, value);
    let result = ocaml_call!(ocaml::make_ok(cr, result));
    let result: OCaml<Result<OCamlInt, String>> = result.expect("Error in 'make_ok' call result");
    result.to_rust()
}

pub fn make_error(cr: &mut OCamlRuntime, value: String) -> Result<i64, String> {
    let result = to_ocaml!(cr, value);
    let result = ocaml_call!(ocaml::make_error(cr, result));
    let result: OCaml<Result<OCamlInt, String>> =
        result.expect("Error in 'make_error' call result");
    result.to_rust()
}

pub fn verify_record_test(cr: &mut OCamlRuntime, record: ocaml::TestRecord) -> String {
    let ocaml_record = to_ocaml!(cr, record);
    let result = ocaml_call!(ocaml::stringify_record(cr, ocaml_record));
    let result: OCaml<String> = result.expect("Error in 'stringify_record' call result");
    result.to_rust()
}

pub fn verify_variant_test(cr: &mut OCamlRuntime, variant: ocaml::Movement) -> String {
    let ocaml_variant = to_ocaml!(cr, variant);
    let result = ocaml_call!(ocaml::stringify_variant(cr, ocaml_variant));
    let result: OCaml<String> = result.expect("Error in 'stringify_variant' call result");
    result.to_rust()
}

pub fn allocate_alot(cr: &mut OCamlRuntime) -> bool {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for _n in 1..50000 {
        let _x: OCaml<OCamlBytes> = to_ocaml!(cr, vec);
        let _y: OCaml<OCamlBytes> = to_ocaml!(cr, vec);
        let _z: OCaml<OCamlBytes> = to_ocaml!(cr, vec);
        ()
    }
    true
}

// Tests

// NOTE: required because at the moment, no synchronization is done on OCaml calls
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_twice() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(twice(&mut cr, 10), 20);
}

#[test]
#[serial]
fn test_increment_bytes() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(
        increment_bytes(&mut cr, "0000000000000000", 10),
        "1111111111000000"
    );
}

#[test]
#[serial]
fn test_increment_ints_list() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    let ints = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(increment_ints_list(&mut cr, &ints), expected);
}

#[test]
#[serial]
fn test_make_tuple() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(
        make_tuple(&mut cr, "fst".to_owned(), 9),
        ("fst".to_owned(), 9)
    );
}

#[test]
#[serial]
fn test_make_some() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(
        make_some(&mut cr, "some".to_owned()),
        Some("some".to_owned())
    );
}

#[test]
#[serial]
fn test_make_result() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(make_ok(&mut cr, 10), Ok(10));
    assert_eq!(
        make_error(&mut cr, "error".to_owned()),
        Err("error".to_owned())
    );
}

#[test]
#[serial]
fn test_frame_management() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(allocate_alot(&mut cr), true);
}

#[test]
#[serial]
fn test_record_conversion() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    let record = ocaml::TestRecord {
        i: 10,
        f: 5.0,
        i32: 10,
        i64: Box::new(10),
        s: "string".to_owned(),
        t: (10, 5.0),
    };
    let expected = "{ i=10; f=5.00; i32=10; i64=10; s=string; t=(10, 5.00) }".to_owned();
    assert_eq!(verify_record_test(&mut cr, record), expected);
}

#[test]
#[serial]
fn test_variant_conversion() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let mut cr = unsafe { OCamlRuntime::acquire() };
    assert_eq!(
        verify_variant_test(&mut cr, ocaml::Movement::RotateLeft),
        "RotateLeft".to_owned()
    );
    assert_eq!(
        verify_variant_test(&mut cr, ocaml::Movement::RotateRight),
        "RotateRight".to_owned()
    );
    assert_eq!(
        verify_variant_test(&mut cr, ocaml::Movement::Step(10)),
        "Step(10)".to_owned()
    );
}
