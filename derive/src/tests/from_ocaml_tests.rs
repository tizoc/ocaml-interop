// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

#[cfg(test)]
mod tests {
    use crate::common::parsing::{parse_input, EnumKind, VariantKind};
    use crate::from_ocaml::codegen::expand_from_ocaml;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_regular_enum() {
        let input_enum = quote! {
            enum MyEnum {
                A,
                B(i32),
                C { val: String },
            }
        };

        let derive_input = syn::parse2(input_enum.clone()).unwrap();
        let type_rep = parse_input(derive_input).unwrap();

        // Check that the enum is parsed correctly
        match &type_rep.data {
            crate::common::parsing::TypeRepData::Enum { variants, kind } => {
                assert_eq!(variants.len(), 3, "Should have 3 variants");
                assert_eq!(*kind, EnumKind::Regular, "Should be a regular enum");

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
            }
            _ => panic!("Expected enum type"),
        }

        // Check that it expands correctly
        let actual_impl_result = expand_from_ocaml(input_enum);
        assert!(
            actual_impl_result.is_ok(),
            "expand_from_ocaml failed for regular enum: {:?}",
            actual_impl_result.err().map(|e| e.to_string())
        );

        // Extract the implementation
        let actual_impl = actual_impl_result.unwrap();

        // Check that it contains the key parts we expect
        let actual_impl_str = actual_impl.to_string();
        assert!(
            actual_impl_str.contains("unsafe impl :: ocaml_interop :: FromOCaml < MyEnum >"),
            "Should generate FromOCaml impl"
        );
        assert!(
            actual_impl_str.contains("Self :: A"),
            "Should handle A variant"
        );
        assert!(
            actual_impl_str.contains("Self :: B"),
            "Should handle B variant"
        );
        assert!(
            actual_impl_str.contains("Self :: C"),
            "Should handle C variant"
        );
    }

    #[test]
    fn test_polymorphic_enum() {
        let input_enum = quote! {
            #[ocaml(polymorphic_variant)]
            enum MyPolyEnum {
                Unit,
                WithPayload(i32),
            }
        };

        let derive_input = syn::parse2(input_enum.clone()).unwrap();
        let type_rep = parse_input(derive_input).unwrap();

        // Check that the enum is parsed correctly
        match &type_rep.data {
            crate::common::parsing::TypeRepData::Enum { variants, kind } => {
                assert_eq!(variants.len(), 2, "Should have 2 variants");
                assert_eq!(*kind, EnumKind::Polymorphic, "Should be a polymorphic enum");

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

        // Check that it expands correctly
        let actual_impl_result = expand_from_ocaml(input_enum);
        assert!(
            actual_impl_result.is_ok(),
            "expand_from_ocaml failed for polymorphic enum: {:?}",
            actual_impl_result.err().map(|e| e.to_string())
        );

        // Extract the implementation
        let actual_impl = actual_impl_result.unwrap();

        // Check that it contains the key parts we expect
        let actual_impl_str = actual_impl.to_string();
        assert!(
            actual_impl_str.contains("unsafe impl :: ocaml_interop :: FromOCaml < MyPolyEnum >"),
            "Should generate FromOCaml impl"
        );
        assert!(
            actual_impl_str.contains("Self :: Unit"),
            "Should handle Unit variant"
        );
        assert!(
            actual_impl_str.contains("Self :: WithPayload"),
            "Should handle WithPayload variant"
        );
        assert!(
            actual_impl_str.contains("caml_hash_variant"),
            "Should use caml_hash_variant for polymorphic variants"
        );
    }

    #[test]
    fn test_struct_full_expansion() {
        let input = parse_quote! {
            struct Point {
                x: f64,
                y: f64,
            }
        };

        let expected = quote! {
            unsafe impl :: ocaml_interop :: FromOCaml < Point > for Point {
                fn from_ocaml(v: :: ocaml_interop :: OCaml < Point >) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::< <f64 as :: ocaml_interop :: DefaultOCamlMapping>::OCamlType >(0).to_rust();
                        let rust_field_1 = v.field::< <f64 as :: ocaml_interop :: DefaultOCamlMapping>::OCamlType >(1).to_rust();
                        Self {
                            x: rust_field_0,
                            y: rust_field_1
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_tuple_struct_full_expansion() {
        let input = parse_quote! {
            struct Pair(String, #[ocaml(as_ = "OCamlInt")] i32);
        };

        let expected = quote! {
            unsafe impl ::ocaml_interop::FromOCaml<Pair> for Pair {
                fn from_ocaml(v: ::ocaml_interop::OCaml<Pair>) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(0).to_rust();
                        let rust_field_1 = v.field::<OCamlInt>(1).to_rust();
                        Self(rust_field_0, rust_field_1)
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_with_type_annotations_full_expansion() {
        let input = parse_quote! {
            struct CustomType {
                #[ocaml(as_ = "OCamlInt")]
                count: i32,
                name: String,
            }
        };

        let expected = quote! {
            unsafe impl ::ocaml_interop :: FromOCaml<CustomType> for CustomType {
                fn from_ocaml(v: ::ocaml_interop::OCaml<CustomType>) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::<OCamlInt>(0).to_rust();
                        let rust_field_1 = v.field::< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust();
                        Self {
                            count: rust_field_0,
                            name: rust_field_1
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_unit_enum_full_expansion() {
        let input = parse_quote! {
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let expected = quote! {
            unsafe impl :: ocaml_interop :: FromOCaml < Color > for Color {
                fn from_ocaml(v: :: ocaml_interop :: OCaml < Color >) -> Self {
                    if v.is_long() {
                        // Handle unit variants
                        let value = unsafe { :: ocaml_interop :: internal :: int_val(v.raw()) };
                        match value {
                            0 => Self::Red,
                            1 => Self::Green,
                            2 => Self::Blue,
                            tag => panic!("Unknown unit variant value: {}", tag),
                        }
                    } else {
                        // This branch would handle block variants, but Color has none
                        panic!("Expected long value for unit-only enum");
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_mixed_enum_full_expansion() {
        let input = parse_quote! {
            enum ExtendedColor {
                Red,
                Green,
                Blue,
                RGB(i64, i64, i64),
            }
        };

        let expected = quote! {
            unsafe impl :: ocaml_interop :: FromOCaml < ExtendedColor > for ExtendedColor {
                fn from_ocaml(v: :: ocaml_interop :: OCaml < ExtendedColor >) -> Self {
                    if v.is_long() {
                        // Integer value (unit variant).
                        let value = unsafe { :: ocaml_interop :: internal :: int_val(v.raw()) };
                        match value {
                            0 => Self::Red,
                            1 => Self::Green,
                            2 => Self::Blue,
                            tag => panic!("Unknown unit variant value: {}", tag),
                        }
                    } else {
                        // Block value (variant with payload)
                        match v.tag_value() {
                            0 => {
                                let field0 = unsafe { v.field :: < < i64 as :: ocaml_interop :: DefaultOCamlMapping > :: OCamlType >(0).to_rust() };
                                let field1 = unsafe { v.field :: < < i64 as :: ocaml_interop :: DefaultOCamlMapping > :: OCamlType >(1).to_rust() };
                                let field2 = unsafe { v.field :: < < i64 as :: ocaml_interop :: DefaultOCamlMapping > :: OCamlType >(2).to_rust() };
                                Self::RGB(field0, field1, field2)
                            }
                            tag => panic!("Unknown block variant tag: {}", tag),
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_polymorphic_movement_enum_full_expansion() {
        let input = parse_quote! {
            #[ocaml(polymorphic_variant)]
            enum Movement {
                Step {
                    #[ocaml(as_ = "OCamlInt")]
                    count: i32
                },
                RotateLeft,
                RotateRight,
                SaveLogMessage {
                    filename: String,
                    message: String
                }
            }
        };

        let expected = quote! {
            unsafe impl :: ocaml_interop :: FromOCaml < Movement > for Movement {
                fn from_ocaml(v: :: ocaml_interop :: OCaml < Movement >) -> Self {
                    // First check if it's a long value (unit variant)
                    if v.is_long() {
                        // Compute hashes for unit variants
                        #[allow(non_snake_case)]
                        let RotateLeft = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("RotateLeft\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        #[allow(non_snake_case)]
                        let RotateRight = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("RotateRight\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        // Compare with raw value
                        let raw_val = unsafe { v.raw() };
                        if raw_val == RotateLeft {
                            return Self::RotateLeft;
                        } else if raw_val == RotateRight {
                            return Self::RotateRight;
                        } else {
                            panic!("Unknown unit polymorphic variant when converting OCaml<Movement> to Movement");
                        }
                    }
                    // Then check if it's the right kind of block for variants with payload
                    else if v.is_block_sized(2) && v.tag_value() == ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT {
                        // Compute hash for block variant
                        #[allow(non_snake_case)]
                        let Step = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Step\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        #[allow(non_snake_case)]
                        let SaveLogMessage = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("SaveLogMessage\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        // Compare tag field with expected hash
                        if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == Step {
                            let count = unsafe { v.field::<OCamlInt>(1).to_rust() };
                            return Self::Step { count };
                        } else if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == SaveLogMessage {
                            let (filename, message) = unsafe {
                                v.field::<(
                                    <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType,
                                    <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType
                                )>(1).to_rust()
                            };
                            return Self::SaveLogMessage { filename, message };
                        } else {
                            panic!("Unknown block polymorphic variant tag when converting OCaml<Movement> to Movement");
                        }
                    }
                    else {
                        panic!("Invalid OCaml value when converting OCaml<Movement> to Movement: expected a polymorphic variant");
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_unit_struct_full_expansion() {
        let input = parse_quote! {
            struct EmptyStruct;
        };

        let expected = quote! {
            unsafe impl :: ocaml_interop :: FromOCaml < EmptyStruct > for EmptyStruct {
                fn from_ocaml(v: :: ocaml_interop :: OCaml < EmptyStruct >) -> Self {
                    unsafe {
                        Self { }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_empty_enum_is_rejected() {
        let input = parse_quote! {
            enum EmptyEnum {}
        };

        // The empty enum should be rejected during validation
        let result = expand_from_ocaml(input);
        assert!(result.is_err(), "Empty enum should be rejected");

        // Check that the error message mentions empty enums not being supported
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Empty enums are not supported"),
            "Error message should mention that empty enums are not supported, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_struct_with_lifetime_param() {
        let input = parse_quote! {
            struct Reference<'a> {
                data: &'a str,
                count: i32,
            }
        };

        let expected = quote! {
            unsafe impl<'a> ::ocaml_interop::FromOCaml<Reference<'a> > for Reference<'a> {
                fn from_ocaml(v: ::ocaml_interop::OCaml<Reference<'a> >) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::< <&'a str as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(0).to_rust();
                        let rust_field_1 = v.field::< <i32 as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust();
                        Self {
                            data: rust_field_0,
                            count: rust_field_1
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_with_type_params_and_constraints() {
        let input = parse_quote! {
            struct GenericData<T, U>
            where
                T: Clone + Default,
                U: AsRef<str> + 'static
            {
                primary: T,
                secondary: U,
            }
        };

        let expected = quote! {
            unsafe impl<T, U> ::ocaml_interop::FromOCaml<GenericData<T, U> > for GenericData<T, U>
            where
                T: Clone + Default,
                U: AsRef<str> + 'static
            {
                fn from_ocaml(v: ::ocaml_interop::OCaml<GenericData<T, U> >) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::< <T as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(0).to_rust();
                        let rust_field_1 = v.field::< <U as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust();
                        Self {
                            primary: rust_field_0,
                            secondary: rust_field_1
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_struct_with_nested_generic_types() {
        let input = parse_quote! {
            struct Container<T> {
                items: Vec<Option<T>>,
                count: usize,
            }
        };

        let expected = quote! {
            unsafe impl<T> ::ocaml_interop::FromOCaml<Container<T> > for Container<T> {
                fn from_ocaml(v: ::ocaml_interop::OCaml<Container<T> >) -> Self {
                    unsafe {
                        let rust_field_0 = v.field::< <Vec<Option<T> > as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(0).to_rust();
                        let rust_field_1 = v.field::< <usize as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust();
                        Self {
                            items: rust_field_0,
                            count: rust_field_1
                        }
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_polymorphic_enum_with_type_param() {
        let input = parse_quote! {
            #[ocaml(polymorphic_variant)]
            enum Result<T, E> {
                Ok(T),
                Err(E)
            }
        };

        let expected = quote! {
            unsafe impl<T, E> ::ocaml_interop::FromOCaml<Result<T, E> > for Result<T, E> {
                fn from_ocaml(v: ::ocaml_interop::OCaml<Result<T, E> >) -> Self {
                    // First check if it's a long value (unit variant)
                    if v.is_long() {
                        panic!("Unexpected unit variant encountered when converting OCaml<Result<T, E>> to Result<T, E>. This enum has no unit variants.");
                    }
                    // Then check if it's the right kind of block for variants with payload
                    else if v.is_block_sized(2) && v.tag_value() == ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT {
                        // Compute hash for block variant
                        #[allow(non_snake_case)]
                        let Ok = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Ok\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        #[allow(non_snake_case)]
                        let Err = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Err\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        // Compare tag field with expected hash
                        if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == Ok {
                            let field0 = unsafe { v.field::< <T as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust() };
                            return Self::Ok(field0);
                        } else if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == Err {
                            let field0 = unsafe { v.field::< <E as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust() };
                            return Self::Err(field0);
                        } else {
                            panic!("Unknown block polymorphic variant tag when converting OCaml<Result<T, E>> to Result<T, E>");
                        }
                    }
                    else {
                        panic!("Invalid OCaml value when converting OCaml<Result<T, E>> to Result<T, E>: expected a polymorphic variant");
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_polymorphic_enum_with_custom_tags() {
        let input = parse_quote! {
            #[ocaml(polymorphic_variant)]
            enum CustomTagged {
                #[ocaml(tag = "success")]
                Success(String),
                #[ocaml(tag = "error")]
                Error { message: String }
            }
        };

        let expected = quote! {
            unsafe impl ::ocaml_interop::FromOCaml<CustomTagged> for CustomTagged {
                fn from_ocaml(v: ::ocaml_interop::OCaml<CustomTagged>) -> Self {
                    // First check if it's a long value (unit variant)
                    if v.is_long() {
                        panic!("Unexpected unit variant encountered when converting OCaml<CustomTagged> to CustomTagged. This enum has no unit variants.");
                    }
                    // Then check if it's the right kind of block for variants with payload
                    else if v.is_block_sized(2) && v.tag_value() == ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT {
                        // Compute hash for block variant
                        #[allow(non_snake_case)]
                        let Success = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("success\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        #[allow(non_snake_case)]
                        let Error = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("error\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        // Compare tag field with expected hash
                        if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == Success {
                            let field0 = unsafe { v.field::< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust() };
                            return Self::Success(field0);
                        } else if unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == Error {
                            let message = unsafe { v.field::< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType >(1).to_rust() };
                            return Self::Error { message };
                        } else {
                            panic!("Unknown block polymorphic variant tag when converting OCaml<CustomTagged> to CustomTagged");
                        }
                    }
                    else {
                        panic!("Invalid OCaml value when converting OCaml<CustomTagged> to CustomTagged: expected a polymorphic variant");
                    }
                }
            }
        };

        let actual = expand_from_ocaml(input).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
