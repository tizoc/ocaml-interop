// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::ocaml_describer::codegen::expand_ocaml_describer;
use pretty_assertions::assert_eq;
use quote::quote;

#[test]
fn test_ocaml_describer_simple_struct_default_name() {
    let input_struct = quote! {
        struct MySimpleStruct;
    };
    let expected_impl = quote! {
        impl ocaml_interop::OCamlDescriber for MySimpleStruct {
            fn ocaml_type_name() -> String {
                "my_simple_struct".to_string()
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_ocaml_describer_simple_struct_custom_name() {
    let input_struct = quote! {
        #[ocaml(name = "custom_name_for_struct")]
        struct MyRenamedStruct;
    };
    let expected_impl = quote! {
        impl ocaml_interop::OCamlDescriber for MyRenamedStruct {
            fn ocaml_type_name() -> String {
                "custom_name_for_struct".to_string()
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_ocaml_describer_generic_struct_one_param() {
    let input_struct = quote! {
        struct MyGenericStruct<A>;
    };
    let expected_impl = quote! {
        impl<A> ocaml_interop::OCamlDescriber for MyGenericStruct<A>
        where A: ocaml_interop::OCamlDescriber
        {
            fn ocaml_type_name() -> String {
                format!("{} {}", <A as ocaml_interop::OCamlDescriber>::ocaml_type_name(), "my_generic_struct")
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_ocaml_describer_generic_struct_multiple_params() {
    let input_struct = quote! {
        struct MyMultiGeneric<A, B>;
    };
    let expected_impl = quote! {
        impl<A, B> ocaml_interop::OCamlDescriber for MyMultiGeneric<A, B>
        where
            A: ocaml_interop::OCamlDescriber,
            B: ocaml_interop::OCamlDescriber
        {
            fn ocaml_type_name() -> String {
                format!("({}) {}",
                    vec![
                        <A as ocaml_interop::OCamlDescriber>::ocaml_type_name(),
                        <B as ocaml_interop::OCamlDescriber>::ocaml_type_name()
                    ].join(", "),
                    "my_multi_generic"
                )
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_ocaml_describer_generic_struct_existing_where_clause() {
    let input_struct = quote! {
        struct MyComplexGeneric<T> where T: Clone;
    };
    let expected_impl = quote! {
        impl<T> ocaml_interop::OCamlDescriber for MyComplexGeneric<T>
        where
            T: Clone,
            T: ocaml_interop::OCamlDescriber
        {
            fn ocaml_type_name() -> String {
                format!("{} {}", <T as ocaml_interop::OCamlDescriber>::ocaml_type_name(), "my_complex_generic")
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_ocaml_describer_lifetime_and_type_param() {
    let input_struct = quote! {
        struct MyLifetimeStruct<'a, T> {
            data: &'a T,
        }
    };
    let expected_impl = quote! {
        impl<'a, T> ocaml_interop::OCamlDescriber for MyLifetimeStruct<'a, T>
        where T: ocaml_interop::OCamlDescriber
        {
            fn ocaml_type_name() -> String {
                format!("{} {}", <T as ocaml_interop::OCamlDescriber>::ocaml_type_name(), "my_lifetime_struct")
            }
        }
    };

    let actual_impl_result = expand_ocaml_describer(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "OCamlDescriber expansion failed: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}
