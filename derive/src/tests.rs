use super::*; // To import export_internal_logic and other items from lib.rs
use pretty_assertions::assert_eq;
use quote::quote;

#[test]
fn test_simple_function_no_args_no_return() {
    let attributes = quote! {};
    let input_function = quote! {
        pub fn simple_test_fn(cr: &mut ::ocaml_interop::OCamlRuntime) {
            println!("Test");
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn simple_test_fn() -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let result_from_body: OCaml<String> = {
                    let rust_name: String = name.to_rust();
                    let greeting = format!("Hello, {}!", rust_name);
                    OCaml::of_rust(cr, &greeting)
                };
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let s: BoxRoot<String> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::<String>::new(cr, s)
                });
                let result_from_body: OCaml<isize> = {
                    let rust_s: String = s.to_rust(cr);
                    OCaml::of_rust(cr, &(rust_s.len() as isize))
                };
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
        pub fn function_that_might_panic(cr: &mut OCamlRuntime) {
            // This function could panic, but with no_panic_catch, it won\'t be caught by the macro.
            println!("Potentially panicking function");
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn function_that_might_panic() -> ::ocaml_interop::RawOCaml {
            let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
        pub fn test_fn(cr: &mut OCamlRuntime) {
            println!("Test");
        }
    };

    let expected_native_part = quote! {
        #[no_mangle]
        pub extern "C" fn test_fn() -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
            if cfg!(debug_assertions) {
                if (argn as usize) != 0usize {
                    panic!(
                        "Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
                        stringify!(test_fn_byte),
                        0usize,
                        argn
                    );
                }
            }
            test_fn()
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let num: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, num) };
                let result_from_body: OCaml<OCamlInt> = {
                    let rust_num: i64 = num.to_rust();
                    OCaml::of_rust(cr, &(rust_num + 1))
                };
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
            add_one(num)
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
            is_active: OCaml<OCamlBool>,
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let count: OCaml<OCamlInt> = unsafe { ::ocaml_interop::OCaml::<OCamlInt>::new(cr, count) };
                let is_active: OCaml<OCamlBool> = unsafe { ::ocaml_interop::OCaml::<OCamlBool>::new(cr, is_active) };
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
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
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
                let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                unsafe { final_result_for_ocaml.raw() }
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
