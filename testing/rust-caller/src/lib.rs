// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

extern crate ocaml_interop;

#[cfg(test)]
use ocaml_interop::cons;
use ocaml_interop::{OCaml, OCamlBytes, OCamlRuntime, ToOCaml};
#[cfg(test)]
use std::borrow::Borrow;

#[cfg(test)]
use ocaml_interop::{bigarray, BoxRoot};

mod ocaml {
    use ocaml_interop::*;

    #[derive(ToOCaml)]
    pub struct TestRecord {
        #[ocaml(as_ = "OCamlInt")]
        pub i: i64,
        pub f: f64,
        pub i32: i32,
        pub i64: Box<i64>,
        pub s: String,
        #[ocaml(as_ = "(OCamlInt, OCamlFloat)")]
        pub t: (i64, f64),
    }

    #[derive(ToOCaml)]
    pub enum Movement {
        Step(#[ocaml(as_ = "OCamlInt")] i64),
        RotateLeft,
        RotateRight,
    }

    #[derive(ToOCaml)]
    #[ocaml(polymorphic_variant)]
    pub enum PolymorphicEnum {
        Unit,
        Single(f64),
        Multiple(#[ocaml(as_ = "OCamlInt")] i64, String),
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
        pub fn stringify_polymorphic_variant(pvariant: PolymorphicEnum) -> String;
        pub fn raises_message_exception(message: String);
        pub fn raises_nonmessage_exception(unit: ());
        pub fn raises_nonblock_exception(unit: ());
        pub fn gc_compact(unit: ());
        pub fn reverse_list_and_compact(list: OCamlList<DynBox<u16>>)
            -> OCamlList<DynBox<u16>>;
        pub fn double_u16_array(array: bigarray::Array1<u16>);
    }
}

pub fn increment_bytes(cr: &mut OCamlRuntime, bytes: &str, first_n: usize) -> String {
    let result = ocaml::increment_bytes(cr, bytes, first_n as i64);
    result.to_rust(cr)
}

pub fn increment_ints_list(cr: &mut OCamlRuntime, ints: &Vec<i64>) -> Vec<i64> {
    let result = ocaml::increment_ints_list(cr, ints);
    result.to_rust(cr)
}

pub fn twice(cr: &mut OCamlRuntime, num: i64) -> i64 {
    let result = ocaml::twice(cr, num);
    result.to_rust(cr)
}

pub fn make_tuple(cr: &mut OCamlRuntime, fst: String, snd: i64) -> (String, i64) {
    let result = ocaml::make_tuple(cr, &fst, snd);
    result.to_rust(cr)
}

pub fn make_some(cr: &mut OCamlRuntime, value: String) -> Option<String> {
    let result = ocaml::make_some(cr, &value);
    result.to_rust(cr)
}

pub fn make_ok(cr: &mut OCamlRuntime, value: i64) -> Result<i64, String> {
    let result = ocaml::make_ok(cr, value);
    result.to_rust(cr)
}

pub fn make_error(cr: &mut OCamlRuntime, value: String) -> Result<i64, String> {
    let result = value.to_boxroot(cr);
    let result = ocaml::make_error(cr, &result);
    result.to_rust(cr)
}

pub fn verify_record_test(cr: &mut OCamlRuntime, record: ocaml::TestRecord) -> String {
    let ocaml_record = record.to_boxroot(cr);
    let result = ocaml::stringify_record(cr, &ocaml_record);
    result.to_rust(cr)
}

pub fn verify_variant_test(cr: &mut OCamlRuntime, variant: ocaml::Movement) -> String {
    let ocaml_variant = variant.to_boxroot(cr);
    let result = ocaml::stringify_variant(cr, &ocaml_variant);
    result.to_rust(cr)
}

pub fn verify_polymorphic_variant_test(
    cr: &mut OCamlRuntime,
    variant: ocaml::PolymorphicEnum,
) -> String {
    let ocaml_variant = variant.to_boxroot(cr);
    let result = ocaml::stringify_polymorphic_variant(cr, &ocaml_variant);
    result.to_rust(cr)
}

pub fn allocate_alot(cr: &mut OCamlRuntime) -> bool {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for _n in 1..50000 {
        let _x: OCaml<OCamlBytes> = vec.to_ocaml(cr);
        let _y: OCaml<OCamlBytes> = vec.to_ocaml(cr);
        let _z: OCaml<OCamlBytes> = vec.to_ocaml(cr);
    }
    true
}

// Tests

#[cfg(test)]
fn ensure_ocaml_runtime_initialized() {
    use ocaml_interop::OCamlRuntimeStartupGuard;

    static INIT: std::sync::Once = std::sync::Once::new();
    static mut OCAML_RUNTIME: Option<OCamlRuntimeStartupGuard> = None;

    INIT.call_once(|| {
        let guard = OCamlRuntime::init().expect("Failed to initialize OCaml runtime");
        unsafe {
            OCAML_RUNTIME = Some(guard);
        }
    });
}

#[cfg(test)]
fn with_domain_lock<F, T>(f: F) -> T
where
    F: FnOnce(&mut OCamlRuntime) -> T,
{
    ensure_ocaml_runtime_initialized();

    OCamlRuntime::with_domain_lock(f)
}

#[test]
fn test_twice() {
    with_domain_lock(|cr| {
        assert_eq!(twice(cr, 10), 20);
    });
}

#[test]
fn test_increment_bytes() {
    with_domain_lock(|cr| {
        assert_eq!(
            increment_bytes(cr, "0000000000000000", 10),
            "1111111111000000"
        );
    });
}

#[test]
fn test_increment_ints_list() {
    with_domain_lock(|cr| {
        let ints = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(increment_ints_list(cr, &ints), expected);
    });
}

#[test]
fn test_make_tuple() {
    with_domain_lock(|cr| {
        assert_eq!(make_tuple(cr, "fst".to_owned(), 9), ("fst".to_owned(), 9));
    });
}

#[test]
fn test_make_some() {
    with_domain_lock(|cr| {
        assert_eq!(make_some(cr, "some".to_owned()), Some("some".to_owned()));
    });
}

#[test]
fn test_make_result() {
    with_domain_lock(|cr| {
        assert_eq!(make_ok(cr, 10), Ok(10));
        assert_eq!(make_error(cr, "error".to_owned()), Err("error".to_owned()));
    });
}

#[test]
fn test_frame_management() {
    with_domain_lock(|cr| {
        assert!(allocate_alot(cr));
    });
}

#[test]
fn test_record_conversion() {
    with_domain_lock(|cr| {
        let record = ocaml::TestRecord {
            i: 10,
            f: 5.0,
            i32: 10,
            i64: Box::new(10),
            s: "string".to_owned(),
            t: (10, 5.0),
        };
        let expected = "{ i=10; f=5.00; i32=10; i64=10; s=string; t=(10, 5.00) }".to_owned();
        assert_eq!(verify_record_test(cr, record), expected);
    });
}

#[test]
fn test_variant_conversion() {
    with_domain_lock(|cr| {
        assert_eq!(
            verify_variant_test(cr, ocaml::Movement::RotateLeft),
            "RotateLeft".to_owned()
        );
        assert_eq!(
            verify_variant_test(cr, ocaml::Movement::RotateRight),
            "RotateRight".to_owned()
        );
        assert_eq!(
            verify_variant_test(cr, ocaml::Movement::Step(10)),
            "Step(10)".to_owned()
        );
    });
}

#[test]
fn test_polymorphic_variant_conversion() {
    with_domain_lock(|cr| {
        assert_eq!(
            verify_polymorphic_variant_test(cr, ocaml::PolymorphicEnum::Unit),
            "Unit".to_owned()
        );
        assert_eq!(
            verify_polymorphic_variant_test(cr, ocaml::PolymorphicEnum::Single(10.0)),
            "Single(10.00)".to_owned()
        );
        assert_eq!(
            verify_polymorphic_variant_test(
                cr,
                ocaml::PolymorphicEnum::Multiple(10, "text".to_string())
            ),
            "Multiple(10, text)".to_owned()
        );
    });
}

#[test]
fn test_bigarray() {
    with_domain_lock(|cr| {
        let arr: Vec<u16> = (0..16).collect();

        let arr_ocaml: BoxRoot<bigarray::Array1<_>> = arr.as_slice().to_boxroot(cr);
        ocaml::double_u16_array(cr, &arr_ocaml);
        assert_eq!(
            cr.get(&arr_ocaml).as_slice(),
            (0..16u16).map(|i| i * 2).collect::<Vec<_>>().as_slice()
        );
    });
}

#[test]
fn test_exception_handling_with_message() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    with_domain_lock(|cr| {
        let result = catch_unwind(AssertUnwindSafe(move || {
            let message = "my-error-message".to_boxroot(cr);
            ocaml::raises_message_exception(cr, &message);
        }));
        assert_eq!(
            result
                .err()
                .map(|err| err.downcast_ref::<String>().unwrap().clone())
                .unwrap(),
            "OCaml exception, message: Some(\"my-error-message\")"
        );
    });
}

#[test]
fn test_exception_handling_without_message() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    with_domain_lock(|cr| {
        let result = catch_unwind(AssertUnwindSafe(move || {
            ocaml::raises_nonmessage_exception(cr, ());
        }));
        assert_eq!(
            result
                .err()
                .map(|err| err.downcast_ref::<String>().unwrap().clone())
                .unwrap(),
            "OCaml exception, message: None"
        );
    });
}

#[test]
fn test_exception_handling_nonblock_exception() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    with_domain_lock(|cr| {
        let result = catch_unwind(AssertUnwindSafe(move || {
            ocaml::raises_nonblock_exception(cr, ());
        }));
        assert_eq!(
            result
                .err()
                .map(|err| err.downcast_ref::<String>().unwrap().clone())
                .unwrap(),
            "OCaml exception, message: None"
        );
    });
}

#[test]
fn test_dynbox() {
    with_domain_lock(|cr| {
        let mut list = OCaml::nil(cr).root();
        let mut l2;
        // Note: building a list with cons will build it in reverse order
        for e in (0u16..4).rev() {
            let boxed = OCaml::box_value(cr, e).root();
            list = cons(cr, &boxed, &list).root();
        }
        l2 = ocaml::reverse_list_and_compact(cr, &list);
        let mut vec2: Vec<u16> = vec![];
        while let Some((hd, tl)) = cr.get(&l2).uncons() {
            l2 = tl.root();
            vec2.push(*hd.borrow());
        }
        // The next call will drop the boxes through the OCaml finalizer
        ocaml::gc_compact(cr, OCaml::unit().as_ref());
        assert_eq!(vec2, vec![3, 2, 1, 0]);
    });
}

#[test]
fn test_threads() {
    let mut handles = Vec::new();

    for _ in 0..100 {
        let handle = std::thread::spawn(move || {
            with_domain_lock(|cr: &mut OCamlRuntime| allocate_alot(cr));
        });

        handles.push(handle);
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    for handle in &handles {
        assert!(handle.is_finished());
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_flexible_parameters_rust_values() {
    // Test that OCaml functions can accept direct Rust values
    with_domain_lock(|cr| {
        // Test with direct primitive values
        assert_eq!(twice(cr, 42), 84);

        // Test with direct string
        let result = ocaml::make_tuple(cr, "hello", 123);
        let (s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(s, "hello");
        assert_eq!(n, 123);

        // Test with direct string slice
        assert_eq!(increment_bytes(cr, "0000", 3), "1110");

        // Test with direct Vec
        let ints = vec![5, 10, 15];
        let expected = vec![6, 11, 16];
        assert_eq!(increment_ints_list(cr, &ints), expected);
    });
}

#[test]
fn test_flexible_parameters_ocaml_refs() {
    // Test that OCaml functions still work with OCamlRef arguments
    with_domain_lock(|cr| {
        // Test with boxroot references
        let num_boxroot = 42i64.to_boxroot(cr);
        let result = ocaml::twice(cr, &num_boxroot);
        let result_val: i64 = result.to_rust(cr);
        assert_eq!(result_val, 84);

        let string_boxroot = "world".to_string().to_boxroot(cr);
        let num_boxroot2 = 456i64.to_boxroot(cr);
        let result = ocaml::make_tuple(cr, &string_boxroot, &num_boxroot2);
        let (s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(s, "world");
        assert_eq!(n, 456);
    });
}

#[test]
fn test_flexible_parameters_mixed() {
    // Test mixing direct Rust values and OCamlRef arguments
    with_domain_lock(|cr| {
        let string_boxroot = "mixed".to_string().to_boxroot(cr);
        let result = ocaml::make_tuple(cr, &string_boxroot, 789);
        let (s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(s, "mixed");
        assert_eq!(n, 789);

        let num_boxroot = 99i64.to_boxroot(cr);
        let result = ocaml::make_tuple(cr, "direct", &num_boxroot);
        let (s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(s, "direct");
        assert_eq!(n, 99);
    });
}

#[test]
fn test_flexible_parameters_borrowed_values() {
    // Test with borrowed Rust values
    with_domain_lock(|cr| {
        let s = String::from("borrowed");
        let result = ocaml::make_tuple(cr, &s, 321);
        let (result_s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(result_s, "borrowed");
        assert_eq!(n, 321);

        let owned_string = "owned".to_string();
        let result = ocaml::make_tuple(cr, owned_string, 654);
        let (result_s, n): (String, i64) = result.to_rust(cr);
        assert_eq!(result_s, "owned");
        assert_eq!(n, 654);
    });
}

#[test]
fn test_flexible_parameters_unit_values() {
    // Test with unit values and OCaml::unit()
    with_domain_lock(|cr| {
        // Test with unit literal
        ocaml::gc_compact(cr, ());

        // Test with OCaml::unit() properly
        let unit_val = OCaml::unit();
        ocaml::gc_compact(cr, unit_val.as_ref());

        // Test with owned OCaml::unit()
        ocaml::gc_compact(cr, OCaml::unit());
    });
}

#[test]
fn test_flexible_parameters_complex_types() {
    // Test with complex convertible types
    with_domain_lock(|cr| {
        // Test with Vec reference for list conversion
        let numbers = vec![100i64, 200, 300];
        let result = ocaml::increment_ints_list(cr, &numbers);
        let result_vec: Vec<i64> = result.to_rust(cr);
        assert_eq!(result_vec, vec![101, 201, 301]);

        // Test with owned Vec
        let numbers_owned = vec![1000i64, 2000, 3000];
        let result = ocaml::increment_ints_list(cr, numbers_owned);
        let result_vec: Vec<i64> = result.to_rust(cr);
        assert_eq!(result_vec, vec![1001, 2001, 3001]);
    });
}

#[test]
fn test_flexible_parameters_option_result() {
    // Test with Option and Result values
    with_domain_lock(|cr| {
        // Test Option with direct string
        let result = ocaml::make_some(cr, "option_test");
        let result_opt: Option<String> = result.to_rust(cr);
        assert_eq!(result_opt, Some("option_test".to_string()));

        // Test Result with direct values
        let result = ocaml::make_ok(cr, 999);
        let result_res: Result<i64, String> = result.to_rust(cr);
        assert_eq!(result_res, Ok(999));

        let result = ocaml::make_error(cr, "error_test");
        let result_res: Result<i64, String> = result.to_rust(cr);
        assert_eq!(result_res, Err("error_test".to_string()));
    });
}

#[test]
fn test_flexible_parameters_records_variants() {
    // Test with record and variant types using flexible parameters
    with_domain_lock(|cr| {
        let record = ocaml::TestRecord {
            i: 42,
            f: 3.14,
            i32: 123,
            i64: Box::new(456),
            s: "flexible".to_owned(),
            t: (789, 2.71),
        };

        // Test passing record directly (will be converted via ToOCaml)
        let result = ocaml::stringify_record(cr, record);
        let expected = "{ i=42; f=3.14; i32=123; i64=456; s=flexible; t=(789, 2.71) }";
        let result_str: String = result.to_rust(cr);
        assert_eq!(result_str, expected);

        // Test passing variant directly
        let variant = ocaml::Movement::Step(555);
        let result = ocaml::stringify_variant(cr, variant);
        let result_str: String = result.to_rust(cr);
        assert_eq!(result_str, "Step(555)");

        // Test passing polymorphic variant directly
        let pvariant = ocaml::PolymorphicEnum::Multiple(777, "flexible_param".to_string());
        let result = ocaml::stringify_polymorphic_variant(cr, pvariant);
        let result_str: String = result.to_rust(cr);
        assert_eq!(result_str, "Multiple(777, flexible_param)");
    });
}

#[test]
fn test_flexible_parameters_backward_compatibility() {
    // Ensure existing code patterns still work
    with_domain_lock(|cr| {
        // Original pattern with explicit to_boxroot
        let value = "legacy".to_string();
        let boxroot = value.to_boxroot(cr);
        let result = ocaml::make_some(cr, &boxroot);
        let result_opt: Option<String> = result.to_rust(cr);
        assert_eq!(result_opt, Some("legacy".to_string()));

        // Original pattern with explicit conversion and rooting
        let record = ocaml::TestRecord {
            i: 1,
            f: 1.0,
            i32: 1,
            i64: Box::new(1),
            s: "legacy".to_owned(),
            t: (1, 1.0),
        };
        let ocaml_record = record.to_boxroot(cr);
        let result = ocaml::stringify_record(cr, &ocaml_record);
        let expected = "{ i=1; f=1.00; i32=1; i64=1; s=legacy; t=(1, 1.00) }";
        let result_str: String = result.to_rust(cr);
        assert_eq!(result_str, expected);
    });
}
