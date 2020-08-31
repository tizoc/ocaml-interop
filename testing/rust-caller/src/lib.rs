// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

extern crate znfe;

use znfe::{
    ocaml_alloc, ocaml_call, ocaml_frame, Intnat, IntoRust, OCaml, OCamlBytes, OCamlList, ToOCaml,
};

mod ocaml {
    use znfe::internal::{GCResult, GCToken};
    use znfe::{
        ocaml, ocaml_frame, to_ocaml, Intnat, OCaml, OCamlInt32, OCamlInt64, OCamlList, ToOCaml,
    };

    pub struct TestRecord {
        pub i: i64,
        pub f: f64,
        pub i32: i32,
        pub i64: i64,
        pub s: String,
        pub t: (i64, f64),
    }

    unsafe impl ToOCaml<TestRecord> for TestRecord {
        fn to_ocaml(&self, token: GCToken) -> GCResult<TestRecord> {
            ocaml_frame!(gc, {
                let i = OCaml::of_int(self.i);
                let ref f = to_ocaml!(gc, self.f).keep(gc);
                let ref i32 = to_ocaml!(gc, self.i32).keep(gc);
                let ref i64 = to_ocaml!(gc, self.i64).keep(gc);
                let ref s = to_ocaml!(gc, self.s).keep(gc);
                let t = to_ocaml!(gc, self.t);

                unsafe {
                    alloc_test_record(token, i, gc.get(f), gc.get(i32), gc.get(i64), gc.get(s), t)
                }
            })
        }
    }

    ocaml! {
        pub alloc fn alloc_test_record(i: Intnat, f: f64, i32: OCamlInt32, i64: OCamlInt64, s: String, t: (Intnat, f64)) -> TestRecord;
        pub fn increment_bytes(bytes: String, first_n: Intnat) -> String;
        pub fn increment_ints_list(ints: OCamlList<Intnat>) -> OCamlList<Intnat>;
        pub fn twice(num: Intnat) -> Intnat;
        pub fn make_tuple(fst: String, snd: Intnat) -> (String, Intnat);
        pub fn make_some(value: String) -> Option<String>;
        pub fn verify_record(record: TestRecord) -> bool;
    }
}

pub fn increment_bytes(bytes: &str, first_n: usize) -> String {
    ocaml_frame!(gc, {
        let bytes = ocaml_alloc!(bytes.to_ocaml(gc));
        let ref bytes_ref = gc.keep(bytes);
        let first_n = ocaml_alloc!((first_n as i64).to_ocaml(gc));
        let result = ocaml_call!(ocaml::increment_bytes(gc, gc.get(bytes_ref), first_n));
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        result.into_rust()
    })
}

pub fn increment_ints_list(ints: &Vec<i64>) -> Vec<i64> {
    ocaml_frame!(gc nokeep, {
        let ints = ocaml_alloc!(ints.to_ocaml(gc));
        let result = ocaml_call!(ocaml::increment_ints_list(gc, ints));
        let result: OCaml<OCamlList<Intnat>> =
            result.expect("Error in 'increment_ints_list' call result");
        result.into_rust()
    })
}

pub fn twice(num: i64) -> i64 {
    ocaml_frame!(gc nokeep, {
        let num = OCaml::of_int(num);
        let result = ocaml_call!(ocaml::twice(gc, num));
        let result: OCaml<Intnat> = result.expect("Error in 'twice' call result");
        result.into_rust()
    })
}

pub fn make_tuple(fst: String, snd: i64) -> (String, i64) {
    ocaml_frame!(gc nokeep, {
        let num = OCaml::of_int(snd);
        let str = ocaml_alloc!(fst.to_ocaml(gc));
        let result = ocaml_call!(ocaml::make_tuple(gc, str, num));
        let result: OCaml<(String, Intnat)> = result.expect("Error in 'make_tuple' call result");
        result.into_rust()
    })
}

pub fn make_some(value: String) -> Option<String> {
    ocaml_frame!(gc nokeep, {
        let str = ocaml_alloc!(value.to_ocaml(gc));
        let result = ocaml_call!(ocaml::make_some(gc, str));
        let result: OCaml<Option<String>> = result.expect("Error in 'make_some' call result");
        result.into_rust()
    })
}

pub fn verify_record_test(record: ocaml::TestRecord) -> bool {
    ocaml_frame!(gc, {
        let ocaml_record = ocaml_alloc!(record.to_ocaml(gc));
        let result = ocaml_call!(ocaml::verify_record(gc, ocaml_record));
        let result: OCaml<bool> = result.expect("Error in 'verify_record' call result");
        result.into_rust()
    })
}

pub fn allocate_alot() -> bool {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for _n in 1..50000 {
        ocaml_frame!(gc, {
            let _x: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            let _y: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            let _z: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            ()
        });
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
fn test_increment_ints_list() {
    znfe::OCamlRuntime::init_persistent();
    let ints = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(increment_ints_list(&ints), expected);
}

#[test]
#[serial]
fn test_make_tuple() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(make_tuple("fst".to_owned(), 9), ("fst".to_owned(), 9));
}

#[test]
#[serial]
fn test_make_some() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(make_some("some".to_owned()), Some("some".to_owned()));
}

#[test]
#[serial]
fn test_frame_management() {
    znfe::OCamlRuntime::init_persistent();
    assert_eq!(allocate_alot(), true);
}

#[test]
#[serial]
fn test_record_conversion() {
    znfe::OCamlRuntime::init_persistent();
    let record = ocaml::TestRecord {
        i: 10,
        f: 5.0,
        i32: 10,
        i64: 10,
        s: "string".to_owned(),
        t: (10, 5.0),
    };
    assert_eq!(verify_record_test(record), true);
}
