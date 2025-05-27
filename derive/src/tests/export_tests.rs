// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::export_internal_logic;

use pretty_assertions::assert_eq;
use quote::quote;

#[test]
fn test_simple_function_no_args_no_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn simple_test_fn(cr: &mut OCamlRuntime, _unused: OCaml<()>) {
            println!("Test");
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn simple_test_fn(_unused: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let _unused: OCaml<()> = unsafe { ::ocaml_interop::OCaml::<()>::new(cr, _unused) };
                {
                    println!("Test");
                };
                ::ocaml_interop::internal::UNIT
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_f64_arg_and_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn multiply_by_two(cr: &mut OCamlRuntime, num: f64) -> f64 {
            num * 2.0
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn multiply_by_two(num: f64) -> f64 {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let result_from_body: f64 = {
                    num * 2.0
                };
                result_from_body
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_ocaml_string_arg_and_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn greet(cr: &mut OCamlRuntime, name: OCaml<String>) -> OCaml<String> {
            let rust_name: String = name.to_rust();
            let greeting = format!("Hello, {}!", rust_name);
            OCaml::of_rust(cr, &greeting)
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn greet(name: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let result_from_body: OCaml<String> = {
                    let rust_name: String = name.to_rust();
                    let greeting = format!("Hello, {}!", rust_name);
                    OCaml::of_rust(cr, &greeting)
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_boxroot_ocaml_string_arg() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn get_string_len(cr: &mut OCamlRuntime, s: BoxRoot<String>) -> OCaml<isize> {
            let rust_s: String = s.to_rust(cr);
            OCaml::of_rust(cr, &(rust_s.len() as isize))
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn get_string_len(s: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let s: BoxRoot<String> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::<String>::new(cr, s)
                });
                let result_from_body: OCaml<isize> = {
                    let rust_s: String = s.to_rust(cr);
                    OCaml::of_rust(cr, &(rust_s.len() as isize))
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_no_panic_catch_attribute() {
    let attributes = quote! { no_panic_catch };
    let input_function = quote! {
        pub fn function_that_might_panic(cr: &mut OCamlRuntime, _unused: OCaml<()>) {
            // This function could panic, but with no_panic_catch, it won't be caught by the macro.
            println!("Potentially panicking function");
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn function_that_might_panic(_unused: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
            let _unused: OCaml<()> = unsafe { ::ocaml_interop::OCaml::<()>::new(cr, _unused) };
            {
                println!("Potentially panicking function");
            };
            ::ocaml_interop::internal::UNIT
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_bytecode_attribute_simple() {
    let attributes = quote! { bytecode = "test_fn_byte" };
    let input_function = quote! {
        pub fn test_fn(cr: &mut OCamlRuntime, _unused: OCaml<()>) {
            println!("Test");
        }
    };

    let expected_native_part = quote! {
        #[no_mangle]
        pub extern "C" fn test_fn(_unused: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let _unused: OCaml<()> = unsafe { ::ocaml_interop::OCaml::<()>::new(cr, _unused) };
                {
                    println!("Test");
                };
                ::ocaml_interop::internal::UNIT
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let expected_bytecode_part = quote! {
        #[no_mangle]
        pub extern "C" fn test_fn_byte(argv: *mut ::ocaml_interop::RawOCaml, argn: ::std::os::raw::c_int) -> ::ocaml_interop::RawOCaml {
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_0 = unsafe { ::core::ptr::read(argv.add(0usize)) };
            let _unused = __ocaml_interop_arg_0;
            if cfg!(debug_assertions) {
                if (argn as usize) != 1usize {
                    panic!(
                        "Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
                        stringify!(test_fn_byte),
                        1usize,
                        argn
                    );
                }
            }
            let result = test_fn(_unused);
            result
        }
    };

    let expected_expansion = quote! {
        #expected_native_part
        #expected_bytecode_part
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_bytecode_attribute_with_int_arg_and_return() {
    let attributes = quote! { bytecode = "add_one_byte" };
    let input_function = quote! {
        pub fn add_one(cr: &mut OCamlRuntime, num: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
            let rust_num: i64 = num.to_rust();
            OCaml::of_rust(cr, &(rust_num + 1))
        }
    };

    let expected_native_part = quote! {
        #[no_mangle]
        pub extern "C" fn add_one(num: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let num: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, num) };
                let result_from_body: OCaml<OCamlInt> = {
                    let rust_num: i64 = num.to_rust();
                    OCaml::of_rust(cr, &(rust_num + 1))
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let expected_bytecode_part = quote! {
        #[no_mangle]
        pub extern "C" fn add_one_byte(argv: *mut ::ocaml_interop::RawOCaml, argn: ::std::os::raw::c_int) -> ::ocaml_interop::RawOCaml {
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_0 = unsafe { ::core::ptr::read(argv.add(0usize)) };
            let num = __ocaml_interop_arg_0;
            if cfg!(debug_assertions) {
                if (argn as usize) != 1usize {
                    panic!(
                        "Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
                        stringify!(add_one_byte),
                        1usize,
                        argn
                    );
                }
            }
            let result = add_one(num);
            result
        }
    };

    let expected_expansion = quote! {
        #expected_native_part
        #expected_bytecode_part
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_multiple_args_mixed_types() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn process_data(
            cr: &mut OCamlRuntime,
            name: OCaml<String>,
            count: OCaml<OCamlInt>,
            is_active: OCaml<bool>,
            score: f64
        ) -> OCaml<String> {
            let rust_name: String = name.to_rust();
            let rust_count: i64 = count.to_rust();
            let rust_is_active: bool = is_active.to_rust();
            let result_string = format!(
                "Processed: {} with count {}, active: {}, score: {}",
                rust_name,
                rust_count,
                rust_is_active,
                score
            );
            OCaml::of_rust(cr, &result_string)
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn process_data(
            name: ::ocaml_interop::RawOCaml,
            count: ::ocaml_interop::RawOCaml,
            is_active: ::ocaml_interop::RawOCaml,
            score: f64
        ) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let count: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, count) };
                let is_active: OCaml<bool> = unsafe { ::ocaml_interop::OCaml::<bool>::new(cr, is_active) };
                let result_from_body: OCaml<String> = {
                    let rust_name: String = name.to_rust();
                    let rust_count: i64 = count.to_rust();
                    let rust_is_active: bool = is_active.to_rust();
                    let result_string = format!(
                        "Processed: {} with count {}, active: {}, score: {}",
                        rust_name,
                        rust_count,
                        rust_is_active,
                        score
                    );
                    OCaml::of_rust(cr, &result_string)
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_complex_ocaml_types_list_option() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn process_string_list(
            cr: &mut OCamlRuntime,
            list: OCaml<OCamlList<String>>,
            threshold: OCaml<OCamlInt>
        ) -> OCaml<Option<i64>> {
            let rust_list: Vec<String> = list.to_rust();
            let rust_threshold: i64 = threshold.to_rust();
            if rust_list.len() > rust_threshold as usize {
                OCaml::of_rust(cr, &Some(rust_list.len() as i64))
            } else {
                OCaml::of_rust(cr, &None)
            }
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn process_string_list(
            list: ::ocaml_interop::RawOCaml,
            threshold: ::ocaml_interop::RawOCaml
        ) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let list: OCaml<OCamlList<String> > = unsafe { ::ocaml_interop::OCaml::<OCamlList<String> >::new(cr, list) };
                let threshold: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, threshold) };
                let result_from_body: OCaml<Option<i64> > = {
                    let rust_list: Vec<String> = list.to_rust();
                    let rust_threshold: i64 = threshold.to_rust();
                    if rust_list.len() > rust_threshold as usize {
                        OCaml::of_rust(cr, &Some(rust_list.len() as i64))
                    } else {
                        OCaml::of_rust(cr, &None)
                    }
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_fully_qualified_paths_args_and_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn process_fully_qualified(
            cr: &mut ::ocaml_interop::OCamlRuntime,
            name: ::ocaml_interop::OCaml<String>,
            count: ::ocaml_interop::BoxRoot<::ocaml_interop::OCamlInt>
        ) -> ::ocaml_interop::OCaml<String> {
            let rust_name: String = name.to_rust();
            let rust_count: i64 = count.to_rust(cr); // BoxRoot needs cr for to_rust
            let result_string = format!(
                "Name: {}, Count: {}",
                rust_name,
                rust_count
            );
            ::ocaml_interop::OCaml::of_rust(cr, &result_string)
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn process_fully_qualified(
            name: ::ocaml_interop::RawOCaml,
            count: ::ocaml_interop::RawOCaml
        ) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut ::ocaml_interop::OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: ::ocaml_interop::OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let count: ::ocaml_interop::BoxRoot<::ocaml_interop::OCamlInt> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::<::ocaml_interop::OCamlInt>::new(cr, count)
                });
                let result_from_body: ::ocaml_interop::OCaml<String> = {
                    let rust_name: String = name.to_rust();
                    let rust_count: i64 = count.to_rust(cr);
                    let result_string = format!("Name: {}, Count: {}", rust_name, rust_count);
                    ::ocaml_interop::OCaml::of_rust(cr, &result_string)
                };
                unsafe { result_from_body.raw() }
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_error_no_args_other_than_runtime() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn only_runtime_arg_fn(cr: &mut OCamlRuntime) {
            // This function should fail to compile because it lacks
            // a non-runtime argument.
            println!("This should not compile successfully.");
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);

    assert!(
        actual_expansion_result.is_err(),
        "Macro expansion should have failed for a function with only a runtime argument."
    );

    if let Err(e) = actual_expansion_result {
        let error_message = e.to_string();
        assert!(
            error_message.contains(
                "OCaml functions must take at least one argument in addition to the OCamlRuntime"
            ),
            "Error message did not contain the expected text. Got: {}",
            error_message
        );
    }
}

#[test]
fn test_noalloc_attribute_simple() {
    let attributes = quote! { noalloc };
    let input_function = quote! {
        pub fn simple_noalloc_fn(cr: &OCamlRuntime, _unused: OCaml<()>) {
            // This function is marked noalloc
            // OCaml values should not be allocated here.
            // Panics are not caught.
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn simple_noalloc_fn(_unused: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let cr : &OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle() };
            let _unused: OCaml<()> = unsafe { ::ocaml_interop::OCaml::<()>::new(cr, _unused) };
            {
                // This function is marked noalloc
                // OCaml values should not be allocated here.
                // Panics are not caught.
            };
            ::ocaml_interop::internal::UNIT
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed for noalloc: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_error_noalloc_with_mutable_runtime() {
    let attributes = quote! { noalloc };
    let input_function = quote! {
        pub fn noalloc_mutable_fail(cr: &mut OCamlRuntime, _unused: OCaml<()>) {
            // This should fail validation
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_err(),
        "Macro expansion should have failed for noalloc with &mut OCamlRuntime"
    );

    if let Err(e) = actual_expansion_result {
        let error_message = e.to_string();
        assert!(
            error_message.contains("When `noalloc` is used, OCaml runtime argument must be an immutable reference (e.g., &OCamlRuntime)"),
            "Error message did not contain the expected text for noalloc with mutable runtime. Got: {}",
            error_message
        );
    }
}

#[test]
fn test_error_alloc_with_immutable_runtime() {
    let attributes = quote! {}; // No noalloc
    let input_function = quote! {
        pub fn alloc_immutable_fail(cr: &OCamlRuntime, _unused: OCaml<()>) {
            // This should fail validation
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_err(),
        "Macro expansion should have failed for alloc (default) with &OCamlRuntime"
    );

    if let Err(e) = actual_expansion_result {
        let error_message = e.to_string();
        assert!(
            error_message.contains("OCaml runtime argument must be a mutable reference (e.g., &mut OCamlRuntime). Use `#[export(noalloc)]` for an immutable reference."),
            "Error message did not contain the expected text for alloc with immutable runtime. Got: {}",
            error_message
        );
    }
}

#[test]
fn test_i64_arg_and_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn process_i64(cr: &mut OCamlRuntime, val: i64) -> i64 {
            val * 2
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn process_i64(val: i64) -> i64 {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let result_from_body: i64 = {
                    val * 2
                };
                result_from_body
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_bool_arg_and_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn process_bool(cr: &mut OCamlRuntime, val: bool) -> bool {
            !val
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn process_bool(val: bool) -> bool {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr : &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let result_from_body: bool = {
                    !val
                };
                result_from_body
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}

#[test]
fn test_bytecode_all_primitives_args_and_return() {
    let attributes = quote! { bytecode = "process_all_primitives_byte" };
    let input_function = quote! {
        pub fn process_all_primitives(
            cr: &mut OCamlRuntime,
            arg_i64: i64,
            arg_f64: f64,
            arg_bool: bool,
            arg_isize: isize,
            arg_i32: i32,
            arg_ocaml_int: OCaml<OCamlInt>
        ) -> i64 {
            let ocaml_int_val: i64 = arg_ocaml_int.to_rust();
            arg_i64 + (arg_f64 as i64) + (if arg_bool { 1 } else { 0 }) + (arg_isize as i64) + (arg_i32 as i64) + ocaml_int_val
        }
    };

    let expected_native_part = quote! {
        #[no_mangle]
        pub extern "C" fn process_all_primitives(
            arg_i64: i64,
            arg_f64: f64,
            arg_bool: bool,
            arg_isize: isize,
            arg_i32: i32,
            arg_ocaml_int: ::ocaml_interop::RawOCaml
        ) -> i64 {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr: &mut OCamlRuntime = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let arg_ocaml_int: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, arg_ocaml_int) };
                let result_from_body: i64 = {
                    let ocaml_int_val: i64 = arg_ocaml_int.to_rust();
                    arg_i64 + (arg_f64 as i64) + (if arg_bool { 1 } else { 0 }) + (arg_isize as i64) + (arg_i32 as i64) + ocaml_int_val
                };
                result_from_body
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let expected_bytecode_part = quote! {
        #[no_mangle]
        pub extern "C" fn process_all_primitives_byte(
            argv: *mut ::ocaml_interop::RawOCaml,
            argn: ::std::os::raw::c_int
        ) -> ::ocaml_interop::RawOCaml {
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_0 = unsafe { ::core::ptr::read(argv.add(0usize)) };
            let arg_i64 = ::ocaml_interop::internal::int64_val(__ocaml_interop_arg_0);
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_1 = unsafe { ::core::ptr::read(argv.add(1usize)) };
            let arg_f64 = ::ocaml_interop::internal::float_val(__ocaml_interop_arg_1);
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_2 = unsafe { ::core::ptr::read(argv.add(2usize)) };
            let arg_bool = ::ocaml_interop::internal::bool_val(__ocaml_interop_arg_2);
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_3 = unsafe { ::core::ptr::read(argv.add(3usize)) };
            let arg_isize = ::ocaml_interop::internal::int_val(__ocaml_interop_arg_3);
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_4 = unsafe { ::core::ptr::read(argv.add(4usize)) };
            let arg_i32 = ::ocaml_interop::internal::int32_val(__ocaml_interop_arg_4);
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            let __ocaml_interop_arg_5 = unsafe { ::core::ptr::read(argv.add(5usize)) };
            let arg_ocaml_int = __ocaml_interop_arg_5;

            if cfg!(debug_assertions) {
                if (argn as usize) != 6usize {
                    panic!(
                        "Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
                        stringify!(process_all_primitives_byte),
                        6usize,
                        argn
                    );
                }
            }
            let result = process_all_primitives(arg_i64, arg_f64, arg_bool, arg_isize, arg_i32, arg_ocaml_int);
            ::ocaml_interop::internal::alloc_int64(result)
        }
    };

    let expected_expansion = quote! {
        #expected_native_part
        #expected_bytecode_part
    };

    let actual_expansion_result = export_internal_logic(attributes, input_function);
    assert!(
        actual_expansion_result.is_ok(),
        "Macro expansion failed: {:?}",
        actual_expansion_result.err().map(|e| e.to_string())
    );
    let actual_expansion = actual_expansion_result.unwrap();

    assert_eq!(actual_expansion.to_string(), expected_expansion.to_string());
}
