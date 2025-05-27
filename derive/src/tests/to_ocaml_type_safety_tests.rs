// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::common::parsing::{parse_input, EnumKind, VariantKind};
use crate::to_ocaml::codegen::expand_to_ocaml;
use pretty_assertions::assert_eq;
use quote::quote;

#[test]
fn test_struct_with_type_safe_components() {
    let input_struct = quote! {
        struct MySimpleStruct {
            #[ocaml(as_ = "OCamlInt")]
            a: i32,
            b: String,
            c: i64,
        }
    };

    let derive_input = syn::parse2(input_struct.clone()).unwrap();
    let type_rep = parse_input(derive_input).unwrap();

    // Check that we correctly parsed the struct
    match &type_rep.data {
        crate::common::parsing::TypeRepData::Struct { fields } => {
            assert_eq!(fields.len(), 3, "Should have 3 fields");
            assert_eq!(fields[0].ident.as_ref().unwrap().to_string(), "a");
            assert_eq!(fields[1].ident.as_ref().unwrap().to_string(), "b");
            assert_eq!(fields[2].ident.as_ref().unwrap().to_string(), "c");
        }
        _ => panic!("Expected struct type"),
    }

    // Now check that it expands correctly
    let actual_impl_result = expand_to_ocaml(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for simple struct: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );

    // Extract the implementation
    let actual_impl = actual_impl_result.unwrap();

    // Just check that it contains the key parts we expect
    let actual_impl_str = actual_impl.to_string();
    assert!(
        actual_impl_str.contains("unsafe impl :: ocaml_interop :: ToOCaml < MySimpleStruct >"),
        "Should generate ToOCaml impl"
    );
    assert!(
        actual_impl_str.contains("field_a_ocaml"),
        "Should generate field_a_ocaml"
    );
    assert!(
        actual_impl_str.contains("field_b_ocaml"),
        "Should generate field_b_ocaml"
    );
    assert!(
        actual_impl_str.contains("field_c_ocaml"),
        "Should generate field_c_ocaml"
    );
}

#[test]
fn test_regular_enum_with_enums_replacing_boolean_flags() {
    let input_enum = quote! {
        enum MySimpleEnum {
            A,
            B(i32),
            C { val: String },
            D,
        }
    };

    let derive_input = syn::parse2(input_enum.clone()).unwrap();
    let type_rep = parse_input(derive_input).unwrap();

    // Check that we correctly parsed the enum
    match &type_rep.data {
        crate::common::parsing::TypeRepData::Enum { variants, kind, .. } => {
            assert_eq!(variants.len(), 4, "Should have 4 variants");

            // Check that the enum kind is Regular (not polymorphic)
            assert_eq!(kind, &EnumKind::Regular);

            // Check variant A: Unit variant
            assert_eq!(variants[0].ident.to_string(), "A");
            assert_eq!(variants[0].kind, VariantKind::Unit);

            // Check variant B: Tuple variant
            assert_eq!(variants[1].ident.to_string(), "B");
            assert_eq!(variants[1].kind, VariantKind::Tuple);
            assert_eq!(variants[1].fields.len(), 1);

            // Check variant C: Struct variant
            assert_eq!(variants[2].ident.to_string(), "C");
            assert_eq!(variants[2].kind, VariantKind::Struct);
            assert_eq!(variants[2].fields.len(), 1);
            assert_eq!(
                variants[2].fields[0].ident.as_ref().unwrap().to_string(),
                "val"
            );

            // Check variant D: Unit variant
            assert_eq!(variants[3].ident.to_string(), "D");
            assert_eq!(variants[3].kind, VariantKind::Unit);
        }
        _ => panic!("Expected enum type"),
    }

    // Now check that it expands correctly
    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for simple enum: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );

    // Extract the implementation
    let actual_impl = actual_impl_result.unwrap();

    // Check that it contains the key parts we expect
    let actual_impl_str = actual_impl.to_string();
    assert!(
        actual_impl_str.contains("unsafe impl :: ocaml_interop :: ToOCaml < MySimpleEnum >"),
        "Should generate ToOCaml impl"
    );
    assert!(
        actual_impl_str.contains("MySimpleEnum :: A =>"),
        "Should handle A variant"
    );
    assert!(
        actual_impl_str.contains("MySimpleEnum :: B ("),
        "Should handle B variant"
    );
    assert!(
        actual_impl_str.contains("MySimpleEnum :: C"),
        "Should handle C variant"
    );
    assert!(
        actual_impl_str.contains("MySimpleEnum :: D =>"),
        "Should handle D variant"
    );
}

#[test]
fn test_polymorphic_enum_with_enum_kind() {
    let input_enum = quote! {
        #[ocaml(polymorphic_variant)]
        enum MyPolyEnum {
            Unit,
            WithPayload(i32),
        }
    };

    let derive_input = syn::parse2(input_enum.clone()).unwrap();
    let type_rep = parse_input(derive_input).unwrap();

    // Check that we correctly parsed the enum
    match &type_rep.data {
        crate::common::parsing::TypeRepData::Enum { variants, kind, .. } => {
            assert_eq!(variants.len(), 2, "Should have 2 variants");

            // Check that the enum kind is Polymorphic
            assert_eq!(kind, &EnumKind::Polymorphic);

            // Check variant Unit: Unit variant
            assert_eq!(variants[0].ident.to_string(), "Unit");
            assert_eq!(variants[0].kind, VariantKind::Unit);

            // Check variant WithPayload: Tuple variant
            assert_eq!(variants[1].ident.to_string(), "WithPayload");
            assert_eq!(variants[1].kind, VariantKind::Tuple);
            assert_eq!(variants[1].fields.len(), 1);
        }
        _ => panic!("Expected enum type"),
    }

    // Now check that it expands correctly
    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for polymorphic enum: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );

    // Extract the implementation
    let actual_impl = actual_impl_result.unwrap();

    // Check that it contains the key parts we expect
    let actual_impl_str = actual_impl.to_string();
    assert!(
        actual_impl_str.contains("unsafe impl :: ocaml_interop :: ToOCaml < MyPolyEnum >"),
        "Should generate ToOCaml impl"
    );
    assert!(
        actual_impl_str.contains("MyPolyEnum :: Unit =>"),
        "Should handle Unit variant"
    );
    assert!(
        actual_impl_str.contains("MyPolyEnum :: WithPayload ("),
        "Should handle WithPayload variant"
    );
    assert!(
        actual_impl_str.contains("caml_hash_variant"),
        "Should use caml_hash_variant for polymorphic variants"
    );
}
