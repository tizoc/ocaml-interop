// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::to_ocaml::codegen::expand_to_ocaml;
use pretty_assertions::assert_eq;
use quote::quote;

#[test]
fn test_to_ocaml_simple_struct_placeholder() {
    let input_struct = quote! {
        struct MySimpleStruct {
            #[ocaml(as_ = "OCamlInt")]
            a: i32,
            b: String, // Will use DefaultOCamlMapping
            c: i64,    // Will use DefaultOCamlMapping
        }
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<MySimpleStruct> for MySimpleStruct {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MySimpleStruct> {
                let record_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(3usize, 0))
                });

                let field_a_ocaml: ::ocaml_interop::OCaml<OCamlInt> = self.a.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 0usize, field_a_ocaml.raw());
                }

                let field_b_ocaml: ::ocaml_interop::OCaml< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.b.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 1usize, field_b_ocaml.raw());
                }

                let field_c_ocaml: ::ocaml_interop::OCaml< <i64 as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.c.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 2usize, field_c_ocaml.raw());
                }

                unsafe { ::ocaml_interop::OCaml::new(cr, record_root.get_raw()) }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for simple struct: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_simple_enum() {
    let input_enum = quote! {
        enum MySimpleEnum {
            A,
            B(i32), // Will use DefaultOCamlMapping for i32
            C { val: String }, // Will use DefaultOCamlMapping for String
            D,
        }
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<MySimpleEnum> for MySimpleEnum {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MySimpleEnum> {
                match self {
                    MySimpleEnum::A => {
                        unsafe { ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::make_ocaml_int(0isize)) }
                    }
                    MySimpleEnum::B(ref field0) => {
                        let block_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                            ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(1usize, 0))
                        });
                        let ocaml_field0: ::ocaml_interop::OCaml< <i32 as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = field0.to_ocaml(cr);
                        unsafe {
                            ::ocaml_interop::internal::store_field(block_root.get_raw(), 0usize, ocaml_field0.raw());
                        }
                        unsafe { ::ocaml_interop::OCaml::new(cr, block_root.get_raw()) }
                    }
                    MySimpleEnum::C { val: ref val } => {
                        let block_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                            ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(1usize, 1))
                        });
                        let ocaml_field0: ::ocaml_interop::OCaml< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = val.to_ocaml(cr);
                        unsafe {
                            ::ocaml_interop::internal::store_field(block_root.get_raw(), 0usize, ocaml_field0.raw());
                        }
                        unsafe { ::ocaml_interop::OCaml::new(cr, block_root.get_raw()) }
                    }
                    MySimpleEnum::D => {
                        unsafe { ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::make_ocaml_int(1isize)) }
                    }
                }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for simple enum: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_polymorphic_enum_unit_variants() {
    let input_enum = quote! {
        #[ocaml(polymorphic_variant)]
        enum MyPolyEnum {
            Answer,
            Question,
        }
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<MyPolyEnum> for MyPolyEnum {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MyPolyEnum> {
                match self {
                    MyPolyEnum::Answer => {
                        let polytag = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Answer\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        unsafe { ::ocaml_interop::OCaml::new(cr, polytag) }
                    }
                    MyPolyEnum::Question => {
                        let polytag = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Question\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        unsafe { ::ocaml_interop::OCaml::new(cr, polytag) }
                    }
                }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for polymorphic enum: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();
    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_generic_struct_placeholder() {
    let input_struct = quote! {
        struct MyGenericStruct<T, U: Clone> where U: std::fmt::Debug {
            data: T, // Will use DefaultOCamlMapping (if T implements it, otherwise compile error)
            info: U, // Will use DefaultOCamlMapping (if U implements it, otherwise compile error)
        }
    };
    let expected_impl = quote! {
        unsafe impl<T, U: Clone> ::ocaml_interop::ToOCaml<MyGenericStruct<T, U> > for MyGenericStruct<T, U>
        where
            U: std::fmt::Debug
        {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MyGenericStruct<T, U> > {
                let record_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(2usize, 0))
                });

                let field_data_ocaml: ::ocaml_interop::OCaml< <T as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.data.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 0usize, field_data_ocaml.raw());
                }

                let field_info_ocaml: ::ocaml_interop::OCaml< <U as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.info.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 1usize, field_info_ocaml.raw());
                }

                unsafe { ::ocaml_interop::OCaml::new(cr, record_root.get_raw()) }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for generic struct: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();
    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_unit_struct() {
    let input_struct = quote! {
        struct MyUnitStruct;
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<MyUnitStruct> for MyUnitStruct {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MyUnitStruct> {
                let record_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(0usize, 0))
                });
                unsafe { ::ocaml_interop::OCaml::new(cr, record_root.get_raw()) }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for unit struct: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();
    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_tuple_struct() {
    let input_struct = quote! {
        struct MyTupleStruct(bool, Vec<String>);
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<MyTupleStruct> for MyTupleStruct {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, MyTupleStruct> {
                let record_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                    ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(2usize, 0))
                });

                let field_0_ocaml: ::ocaml_interop::OCaml< <bool as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.0.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 0usize, field_0_ocaml.raw());
                }

                let field_1_ocaml: ::ocaml_interop::OCaml< <Vec<String> as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = self.1.to_ocaml(cr);
                unsafe {
                    ::ocaml_interop::internal::store_field(record_root.get_raw(), 1usize, field_1_ocaml.raw());
                }

                unsafe { ::ocaml_interop::OCaml::new(cr, record_root.get_raw()) }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_struct);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for tuple struct: {:?}",
        actual_impl_result.err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();
    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_enum_with_field_as_attribute() {
    let input_enum = quote! {
        #[ocaml(as_ = "OCamlMovement")]
        pub enum Movement {
            Step(#[ocaml(as_ = "OCamlInt")] i64), // block, OCaml tag 0
            RotateLeft, // unit, OCaml int 0 (raw 1)
            RotateRight, // unit, OCaml int 1 (raw 3)
        }
    };
    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<OCamlMovement> for Movement {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, OCamlMovement> {
                match self {
                    Movement::Step(ref field0) => {
                        let block_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                            ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(1usize, 0))
                        });
                        let ocaml_field0: ::ocaml_interop::OCaml<OCamlInt> = field0.to_ocaml(cr);
                        unsafe {
                            ::ocaml_interop::internal::store_field(block_root.get_raw(), 0usize, ocaml_field0.raw());
                        }
                        unsafe { ::ocaml_interop::OCaml::new(cr, block_root.get_raw()) }
                    }
                    Movement::RotateLeft => {
                        unsafe { ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::make_ocaml_int(0isize)) }
                    }
                    Movement::RotateRight => {
                        unsafe { ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::make_ocaml_int(1isize)) }
                    }
                }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for enum with field attribute: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}

#[test]
fn test_to_ocaml_polymorphic_enum_with_payloads() {
    let input_enum = quote! {
        #[ocaml(polymorphic_variant)]
        pub enum PolymorphicEnum {
            Unit,
            Single(f64),
            Multiple(i64, String),
        }
    };

    let expected_impl = quote! {
        unsafe impl ::ocaml_interop::ToOCaml<PolymorphicEnum> for PolymorphicEnum {
            fn to_ocaml<'a>(
                &self,
                cr: &'a mut ::ocaml_interop::OCamlRuntime
            ) -> ::ocaml_interop::OCaml<'a, PolymorphicEnum> {
                match self {
                    PolymorphicEnum::Unit => {
                        let polytag = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Unit\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        unsafe { ::ocaml_interop::OCaml::new(cr, polytag) }
                    }
                    PolymorphicEnum::Single(ref field0) => {
                        let polytag = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Single\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };
                        let ocaml_payload_field0: ::ocaml_interop::OCaml< <f64 as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = field0.to_ocaml(cr);
                        unsafe {
                            let block = ::ocaml_interop::internal::caml_alloc(
                                2usize,
                                ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT,
                            );
                            ::ocaml_interop::internal::store_field(block, 0, polytag);
                            ::ocaml_interop::internal::store_field(block, 1, ocaml_payload_field0.raw());
                            ::ocaml_interop::OCaml::new(cr, block)
                        }
                    }
                    PolymorphicEnum::Multiple(ref field0, ref field1) => {
                        let polytag = {
                            static mut TAG_HASH: ::ocaml_interop::RawOCaml = 0;
                            static INIT_TAG_HASH: ::std::sync::Once = ::std::sync::Once::new();
                            unsafe {
                                INIT_TAG_HASH.call_once(|| {
                                    TAG_HASH = ::ocaml_interop::internal::caml_hash_variant("Multiple\0".as_ptr());
                                });
                                TAG_HASH
                            }
                        };

                        // Create a tuple for the multiple payloads
                        let tuple_root: ::ocaml_interop::BoxRoot<()> = ::ocaml_interop::BoxRoot::new(unsafe {
                            ::ocaml_interop::internal::alloc_tuple(cr, 2usize)
                        });

                        let ocaml_field0_for_tuple: ::ocaml_interop::OCaml< <i64 as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = field0.to_ocaml(cr);
                        unsafe { ::ocaml_interop::internal::store_field(tuple_root.get_raw(), 0usize, ocaml_field0_for_tuple.raw()); }

                        let ocaml_field1_for_tuple: ::ocaml_interop::OCaml< <String as ::ocaml_interop::DefaultOCamlMapping>::OCamlType> = field1.to_ocaml(cr);
                        unsafe { ::ocaml_interop::internal::store_field(tuple_root.get_raw(), 1usize, ocaml_field1_for_tuple.raw()); }

                        unsafe {
                            let block = ::ocaml_interop::internal::caml_alloc(
                                2usize,
                                ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT,
                            );
                            ::ocaml_interop::internal::store_field(block, 0, polytag);
                            ::ocaml_interop::internal::store_field(block, 1, tuple_root.get_raw());
                            ::ocaml_interop::OCaml::new(cr, block)
                        }
                    }
                }
            }
        }
    };

    let actual_impl_result = expand_to_ocaml(input_enum);
    assert!(
        actual_impl_result.is_ok(),
        "expand_to_ocaml failed for polymorphic enum with payloads: {:?}",
        actual_impl_result.as_ref().err().map(|e| e.to_string())
    );
    let actual_impl = actual_impl_result.unwrap();

    assert_eq!(actual_impl.to_string(), expected_impl.to_string());
}
