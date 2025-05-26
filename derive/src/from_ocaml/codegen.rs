use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::common::{
    field_processing::{
        generate_struct_field_extractions, get_ocaml_type, make_field_var, TypeDirection,
    },
    format_type,
    parsing::{parse_input, EnumKind, TypeRep, TypeRepData, VariantKind},
    polytag_utils::generate_polytag_hash,
    validation, OCamlInteropError, Result,
};

/// Generates a conversion error message for enum variants
fn generate_conversion_error_message(
    ocaml_type_ident: &proc_macro2::TokenStream,
    type_ident: &syn::Ident,
    ty_generics: syn::TypeGenerics,
    error_type: &str,
) -> String {
    format!(
        "{} when converting OCaml<{}> to {}",
        error_type,
        format_type(quote! { #ocaml_type_ident #ty_generics }.to_string()),
        format_type(quote! { #type_ident #ty_generics }.to_string())
    )
}

/// Builds an if-else chain from a list of conditions
fn build_conditional_chain(conditions: &[TokenStream]) -> TokenStream {
    match conditions {
        [] => quote! {},
        [first] => quote! { if #first },
        [first, rest @ ..] => quote! { if #first #(else if #rest)* },
    }
}

/// Generates an error branch with panic message
fn generate_error_branch(
    type_rep: &TypeRep,
    should_generate: bool,
    error_type: &str,
) -> TokenStream {
    if should_generate {
        let ty_generics = type_rep.generics.split_for_impl().1;
        let type_ident = &type_rep.ident;
        let ocaml_type_ident = &type_rep.ocaml_target_type_ident_ts;
        let message = generate_conversion_error_message(
            ocaml_type_ident,
            type_ident,
            ty_generics,
            error_type,
        );
        quote! {
            else {
                panic!(#message);
            }
        }
    } else {
        quote! {}
    }
}

/// Generates hash computation setup code for variants
fn generate_variant_hash_setup(
    variants: &[&crate::common::parsing::VariantRep],
) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            let tag_str = v.attrs.get_tag().as_ref();
            let poly_tag = generate_polytag_hash(variant_ident, tag_str);
            quote! {
                #[allow(non_snake_case)]
                let #variant_ident = #poly_tag;
            }
        })
        .collect()
}

/// Generates extraction logic for a polymorphic variant with payload
fn generate_polymorphic_variant_extraction(
    variant: &crate::common::parsing::VariantRep,
) -> Result<TokenStream> {
    let variant_ident = &variant.ident;

    match variant.kind {
        VariantKind::Tuple => {
            let fields = &variant.fields;
            let field_extractions = fields
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let ocaml_type = get_ocaml_type(field, TypeDirection::FromOCaml);
                    let idx_lit =
                        syn::LitInt::new(&(idx + 1).to_string(), proc_macro2::Span::call_site());
                    let field_var = make_field_var(idx, None);

                    quote! {
                        let #field_var = unsafe { v.field::<#ocaml_type>(#idx_lit).to_rust() };
                    }
                })
                .collect::<Vec<_>>();

            let field_vars = (0..fields.len())
                .map(|idx| make_field_var(idx, None))
                .collect::<Vec<_>>();

            Ok(quote! {
                unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == #variant_ident {
                    #(#field_extractions)*
                    return Self::#variant_ident(#(#field_vars),*);
                }
            })
        }
        VariantKind::Struct => {
            let fields = &variant.fields;

            if fields.len() > 1 {
                let field_types = fields
                    .iter()
                    .map(|field| get_ocaml_type(field, TypeDirection::FromOCaml))
                    .collect::<Vec<_>>();

                let tuple_type = quote! { (#(#field_types),*) };
                let field_names = fields
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();

                Ok(quote! {
                    unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == #variant_ident {
                        let (#(#field_names),*) = unsafe {
                            v.field::<#tuple_type>(1).to_rust()
                        };
                        return Self::#variant_ident { #(#field_names),* };
                    }
                })
            } else {
                let field_extractions = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let field_ident = field.ident.as_ref().unwrap();
                        let ocaml_type = get_ocaml_type(field, TypeDirection::FromOCaml);
                        let field_idx_lit = syn::LitInt::new(&(idx + 1).to_string(), proc_macro2::Span::call_site());

                        quote! {
                            let #field_ident = unsafe { v.field::<#ocaml_type>(#field_idx_lit).to_rust() };
                        }
                    })
                    .collect::<Vec<_>>();

                let field_assignments = fields
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();

                Ok(quote! {
                    unsafe { v.field::<::ocaml_interop::OCamlInt>(0).raw() } == #variant_ident {
                        #(#field_extractions)*
                        return Self::#variant_ident { #(#field_assignments),* };
                    }
                })
            }
        }
        _ => unreachable!("Unit variants should be handled separately"),
    }
}

/// Generates FromOCaml implementation for a struct
fn codegen_struct_impl(
    type_rep: &TypeRep,
    fields: &[crate::common::parsing::FieldRep],
) -> Result<TokenStream> {
    let type_ident = &type_rep.ident;
    let (impl_generics, ty_generics, where_clause) = type_rep.generics.split_for_impl();
    let ocaml_type_ident_token_stream = &type_rep.ocaml_target_type_ident_ts;

    let field_processing_quotes = generate_struct_field_extractions(fields, quote! { v });

    let struct_constructor = if fields.iter().all(|f| f.ident.is_some()) {
        // Named fields (struct)
        let field_assignments = fields.iter().enumerate().map(|(idx, field_rep)| {
            let field_ident = field_rep.ident.as_ref().unwrap();
            let rust_var_name = Ident::new(&format!("rust_field_{idx}"), field_rep.span);
            quote! { #field_ident: #rust_var_name }
        });

        quote! { Self { #(#field_assignments),* } }
    } else {
        // Tuple fields
        let field_vars = (0..fields.len())
            .map(|idx| Ident::new(&format!("rust_field_{idx}"), proc_macro2::Span::call_site()));

        quote! { Self(#(#field_vars),*) }
    };

    Ok(quote! {
        unsafe impl #impl_generics ::ocaml_interop::FromOCaml<#ocaml_type_ident_token_stream #ty_generics> for #type_ident #ty_generics #where_clause {
            fn from_ocaml(v: ::ocaml_interop::OCaml<#ocaml_type_ident_token_stream #ty_generics>) -> Self {
                unsafe {
                    #(#field_processing_quotes)*
                    #struct_constructor
                }
            }
        }
    })
}

/// Generates the complete body for a polymorphic enum
fn generate_polymorphic_enum_body(
    type_rep: &TypeRep,
    variants: &[crate::common::parsing::VariantRep],
) -> Result<TokenStream> {
    let (unit_variants, payload_variants): (Vec<_>, Vec<_>) =
        variants.iter().partition(|v| v.kind == VariantKind::Unit);

    let unit_variant_setup = generate_variant_hash_setup(&unit_variants);
    let unit_variant_checks: Vec<TokenStream> = unit_variants
        .iter()
        .map(|v| {
            let variant_ident = &v.ident;
            quote! {
                raw_val == #variant_ident {
                    return Self::#variant_ident;
                }
            }
        })
        .collect();

    let unit_variant_chain = build_conditional_chain(&unit_variant_checks);
    let unit_variant_end = generate_error_branch(
        type_rep,
        !unit_variants.is_empty(),
        "Unknown unit polymorphic variant",
    );

    let long_value_branch = if !unit_variants.is_empty() {
        quote! {
            #(#unit_variant_setup)*
            let raw_val = unsafe { v.raw() };
            #unit_variant_chain
            #unit_variant_end
        }
    } else {
        let ty_generics = type_rep.generics.split_for_impl().1;
        let type_ident = &type_rep.ident;
        let ocaml_type_ident = &type_rep.ocaml_target_type_ident_ts;
        let message = generate_conversion_error_message(
            ocaml_type_ident,
            type_ident,
            ty_generics,
            "Unexpected unit variant encountered",
        ) + ". This enum has no unit variants.";
        quote! { panic!(#message); }
    };

    let payload_variant_setup = generate_variant_hash_setup(&payload_variants);
    let payload_variant_extractions: Result<Vec<TokenStream>> = payload_variants
        .iter()
        .map(|v| generate_polymorphic_variant_extraction(v))
        .collect();
    let payload_variant_extractions = payload_variant_extractions?;

    let payload_variant_chain = build_conditional_chain(&payload_variant_extractions);
    let payload_variant_end = generate_error_branch(
        type_rep,
        !payload_variants.is_empty(),
        "Unknown block polymorphic variant tag",
    );

    let ty_generics = type_rep.generics.split_for_impl().1;
    let type_ident = &type_rep.ident;
    let ocaml_type_ident = &type_rep.ocaml_target_type_ident_ts;
    let final_message = generate_conversion_error_message(
        ocaml_type_ident,
        type_ident,
        ty_generics,
        "Invalid OCaml value",
    ) + ": expected a polymorphic variant";

    let polyvar_check = quote! {
        v.is_block_sized(2) && v.tag_value() == ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT
    };

    Ok(quote! {
        if v.is_long() {
            #long_value_branch
        }
        else if #polyvar_check {
            #(#payload_variant_setup)*
            #payload_variant_chain
            #payload_variant_end
        }
        else {
            panic!(#final_message);
        }
    })
}

/// Generates the complete body for a regular enum based on variant composition
fn generate_regular_enum_body(
    _type_rep: &TypeRep,
    variants: &[crate::common::parsing::VariantRep],
) -> Result<TokenStream> {
    let mut unit_variant_arms = Vec::new();
    let mut block_variant_arms = Vec::new();
    let mut unit_variant_idx = 0u8;
    let mut block_variant_idx = 0u8;

    for variant_rep in variants {
        match variant_rep.kind {
            VariantKind::Unit => {
                let variant_ident = &variant_rep.ident;
                let lit = syn::LitInt::new(
                    &unit_variant_idx.to_string(),
                    proc_macro2::Span::call_site(),
                );
                unit_variant_arms.push(quote! { #lit => Self::#variant_ident, });
                unit_variant_idx += 1;
            }
            VariantKind::Tuple | VariantKind::Struct => {
                let variant_ident = &variant_rep.ident;
                let fields = &variant_rep.fields;
                let tag_lit = syn::LitInt::new(
                    &block_variant_idx.to_string(),
                    proc_macro2::Span::call_site(),
                );

                let field_processing = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field_rep)| {
                        let field_var_ident = field_rep.ident.as_ref()
                            .cloned()
                            .unwrap_or_else(|| make_field_var(idx, None));
                        let ocaml_type = get_ocaml_type(field_rep, TypeDirection::FromOCaml);
                        let idx_lit = syn::LitInt::new(&idx.to_string(), proc_macro2::Span::call_site());

                        quote! {
                            let #field_var_ident = unsafe { v.field::<#ocaml_type>(#idx_lit).to_rust() };
                        }
                    })
                    .collect::<Vec<_>>();

                let constructor = match variant_rep.kind {
                    VariantKind::Tuple => {
                        let field_vars = (0..fields.len()).map(|idx| make_field_var(idx, None));
                        quote! { Self::#variant_ident(#(#field_vars),*) }
                    }
                    VariantKind::Struct => {
                        let field_assignments = fields.iter().map(|field_rep| {
                            field_rep
                                .ident
                                .as_ref()
                                .expect("Struct variant fields must have identifiers")
                        });
                        quote! { Self::#variant_ident { #(#field_assignments),* } }
                    }
                    _ => unreachable!("Unit variants should be handled separately"),
                };

                let match_arm = quote! {
                    #tag_lit => {
                        #(#field_processing)*
                        #constructor
                    }
                };

                block_variant_arms.push(match_arm);
                block_variant_idx += 1;
            }
        }
    }

    let has_only_unit_variants = variants.iter().all(|v| v.kind == VariantKind::Unit);

    let else_branch = if has_only_unit_variants {
        quote! { panic!("Expected long value for unit-only enum"); }
    } else {
        quote! {
            match v.tag_value() {
                #(#block_variant_arms)*
                tag => panic!("Unknown block variant tag: {}", tag),
            }
        }
    };

    Ok(quote! {
        if v.is_long() {
            let value = unsafe { ::ocaml_interop::internal::int_val(v.raw()) };
            match value {
                #(#unit_variant_arms)*
                tag => panic!("Unknown unit variant value: {}", tag),
            }
        } else {
            #else_branch
        }
    })
}

/// Generates FromOCaml implementation for an enum
fn codegen_enum_impl(
    type_rep: &TypeRep,
    variants: &[crate::common::parsing::VariantRep],
    kind: &EnumKind,
) -> Result<TokenStream> {
    let type_ident = &type_rep.ident;
    let (impl_generics, ty_generics, where_clause) = type_rep.generics.split_for_impl();
    let ocaml_type_ident_token_stream = &type_rep.ocaml_target_type_ident_ts;

    let body = match kind {
        EnumKind::Regular => generate_regular_enum_body(type_rep, variants)?,

        EnumKind::Polymorphic => generate_polymorphic_enum_body(type_rep, variants)?,
    };

    Ok(quote! {
        unsafe impl #impl_generics ::ocaml_interop::FromOCaml<#ocaml_type_ident_token_stream #ty_generics> for #type_ident #ty_generics #where_clause {
            fn from_ocaml(v: ::ocaml_interop::OCaml<#ocaml_type_ident_token_stream #ty_generics>) -> Self {
                #body
            }
        }
    })
}

/// Main entry point for expanding the FromOCaml derive macro
pub fn expand_from_ocaml(input: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse2::<DeriveInput>(input).map_err(OCamlInteropError::Syn)?;
    let type_rep = parse_input(derive_input)?;

    validation::validate_type_rep(&type_rep)?;

    match &type_rep.data {
        TypeRepData::Struct { fields } => codegen_struct_impl(&type_rep, fields),
        TypeRepData::Enum { variants, kind } => codegen_enum_impl(&type_rep, variants, kind),
    }
}
