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
        pub fn multiply_by_two(cr: &mut ::ocaml_interop::OCamlRuntime, num: f64) -> f64 {
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
        pub fn greet(cr: &mut ::ocaml_interop::OCamlRuntime, name: ::ocaml_interop::OCaml<String>) -> ::ocaml_interop::OCaml<String> {
            let rust_name: String = name.to_rust(cr);
            let greeting = format!("Hello, {}!", rust_name);
            ::ocaml_interop::OCaml::of_rust(cr, &greeting)
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn greet(name: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let name: ::ocaml_interop::OCaml<String> = unsafe { ::ocaml_interop::OCaml::<String>::new(cr, name) };
                let result_from_body: ::ocaml_interop::OCaml<String> = {
                    let rust_name: String = name.to_rust(cr);
                    let greeting = format!("Hello, {}!", rust_name);
                    ::ocaml_interop::OCaml::of_rust(cr, &greeting)
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
        pub fn get_string_len(cr: &mut ::ocaml_interop::OCamlRuntime, s: ::ocaml_interop::BoxRoot<String>) -> ::ocaml_interop::OCaml<isize> {
            let rust_s: String = s.to_rust(cr);
            ::ocaml_interop::OCaml::of_rust(cr, &(rust_s.len() as isize))
        }
    };

    let expected_expansion = quote! {
        #[no_mangle]
        pub extern "C" fn get_string_len(s: ::ocaml_interop::RawOCaml) -> ::ocaml_interop::RawOCaml {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let cr = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };
                let s: ::ocaml_interop::BoxRoot<String> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::<String>::new(cr, s)
                });
                let result_from_body: ::ocaml_interop::OCaml<isize> = {
                    let rust_s: String = s.to_rust(cr);
                    ::ocaml_interop::OCaml::of_rust(cr, &(rust_s.len() as isize))
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
        pub fn function_that_might_panic(cr: &mut ::ocaml_interop::OCamlRuntime) {
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
        pub fn test_fn(cr: &mut ::ocaml_interop::OCamlRuntime) {
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
