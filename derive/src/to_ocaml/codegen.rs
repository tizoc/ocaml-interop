use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::common::{
    field_processing::{
        create_ocaml_block, create_ocaml_tuple, create_polymorphic_variant_block,
        generate_field_conversion_and_storage, generate_struct_field_conversions, get_ocaml_type,
        make_field_var, TypeDirection,
    },
    parsing::{parse_input, EnumKind, TypeRep, TypeRepData, VariantKind},
    polytag_utils::generate_polytag_hash,
    validation, OCamlInteropError, Result,
};

/// Generates a unit variant match arm for both regular and polymorphic enums
fn generate_unit_variant_arm(
    type_ident: &Ident,
    variant_rep: &crate::common::parsing::VariantRep,
    unit_idx: Option<u8>,
) -> TokenStream {
    let variant_ident = &variant_rep.ident;

    if let Some(idx) = unit_idx {
        // Regular enum unit variant
        let lit_int = syn::LitInt::new(&format!("{idx}isize"), Span::call_site());
        quote! {
            #type_ident::#variant_ident => {
                unsafe { ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::make_ocaml_int(#lit_int)) }
            }
        }
    } else {
        // Polymorphic variant unit
        let polytag =
            generate_polytag_hash(&variant_rep.ident, variant_rep.attrs.get_tag().as_ref());
        quote! {
            #type_ident::#variant_ident => {
                let polytag = #polytag;
                unsafe { ::ocaml_interop::OCaml::new(cr, polytag) }
            }
        }
    }
}

/// Generate field patterns and identifiers for variant destructuring
fn generate_variant_pattern(
    variant_rep: &crate::common::parsing::VariantRep,
) -> (TokenStream, Vec<Ident>) {
    let (field_pats, field_idents): (Vec<_>, Vec<_>) = match variant_rep.kind {
        VariantKind::Tuple => variant_rep
            .fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let field_ident = make_field_var(idx, Some(field.span));
                (quote! { ref #field_ident }, field_ident)
            })
            .unzip(),
        VariantKind::Struct => variant_rep
            .fields
            .iter()
            .map(|field| {
                let field_ident = field
                    .ident
                    .as_ref()
                    .expect("Named field in struct variant must have an identifier")
                    .clone();
                (quote! { #field_ident: ref #field_ident }, field_ident)
            })
            .unzip(),
        _ => unreachable!(),
    };

    let pattern = match variant_rep.kind {
        VariantKind::Tuple => quote! { ( #(#field_pats),* ) },
        VariantKind::Struct => quote! { { #(#field_pats),* } },
        _ => unreachable!(),
    };

    (pattern, field_idents)
}

/// Generate polymorphic variant payload logic for variants with fields
fn generate_poly_variant_payload(
    variant_rep: &crate::common::parsing::VariantRep,
    field_idents: &[Ident],
) -> (Vec<TokenStream>, TokenStream) {
    let field_count = variant_rep.fields.len();
    let mut setup_ops = Vec::new();

    let ocaml_payload_value_expr = if field_count == 1 {
        // Single field case - direct conversion
        let field_rep = &variant_rep.fields[0];
        let field_ident = &field_idents[0];
        let ocaml_payload_var = Ident::new(&format!("ocaml_payload_{field_ident}"), field_rep.span);

        let ocaml_type = get_ocaml_type(field_rep, TypeDirection::ToOCaml);
        setup_ops.push(quote! {
            let #ocaml_payload_var: ::ocaml_interop::OCaml<#ocaml_type> = #field_ident.to_ocaml(cr);
        });

        quote! { #ocaml_payload_var.raw() }
    } else {
        // Multiple fields case - tuple conversion
        let tuple_root_var = Ident::new("tuple_root", variant_rep.span);
        let tuple_block = create_ocaml_tuple(field_count);

        setup_ops.push(quote! {
            let #tuple_root_var: ::ocaml_interop::BoxRoot<()> = #tuple_block;
        });

        let tuple_conversions = generate_field_conversions_with_suffix(
            &variant_rep.fields,
            field_idents,
            quote! { #tuple_root_var.get_raw() },
            "for_tuple",
        );

        setup_ops.extend(tuple_conversions);
        quote! { #tuple_root_var.get_raw() }
    };

    (setup_ops, ocaml_payload_value_expr)
}

/// Generate field conversion operations for variant processing
fn generate_field_conversions(
    fields: &[crate::common::parsing::FieldRep],
    field_idents: &[Ident],
    container_expr: TokenStream,
) -> Vec<TokenStream> {
    generate_field_conversions_with_suffix(fields, field_idents, container_expr, "")
}

/// Generate field conversion operations with custom variable suffix
fn generate_field_conversions_with_suffix(
    fields: &[crate::common::parsing::FieldRep],
    field_idents: &[Ident],
    container_expr: TokenStream,
    suffix: &str,
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(field_idents.iter())
        .enumerate()
        .map(|(idx, (field_rep, field_ident))| {
            let var_name = if suffix.is_empty() {
                format!("ocaml_field{idx}")
            } else {
                format!("ocaml_field{idx}_{suffix}")
            };
            let ocaml_field_var = Ident::new(&var_name, field_rep.span);
            generate_field_conversion_and_storage(
                field_rep,
                quote! { #field_ident },
                &ocaml_field_var,
                container_expr.clone(),
                idx,
            )
        })
        .collect()
}

/// Generate a regular enum variant arm with block allocation and field processing
fn generate_regular_block_variant_arm(
    type_ident: &Ident,
    variant_rep: &crate::common::parsing::VariantRep,
    block_variant_idx: u8,
) -> TokenStream {
    let (pattern, field_idents) = generate_variant_pattern(variant_rep);
    let variant_ident = &variant_rep.ident;

    let block_root_var = Ident::new("block_root", variant_rep.span);
    let block_allocation = create_ocaml_block(variant_rep.fields.len(), block_variant_idx as usize);
    let field_processing = generate_field_conversions(
        &variant_rep.fields,
        &field_idents,
        quote! { #block_root_var.get_raw() },
    );

    quote! {
        #type_ident::#variant_ident #pattern => {
            let #block_root_var: ::ocaml_interop::BoxRoot<()> = #block_allocation;
            #(#field_processing)*
            unsafe { ::ocaml_interop::OCaml::new(cr, #block_root_var.get_raw()) }
        }
    }
}

fn codegen_struct_impl(
    type_rep: &TypeRep,
    fields: &[crate::common::parsing::FieldRep],
) -> TokenStream {
    let type_ident = &type_rep.ident;
    let (impl_generics, ty_generics, where_clause) = type_rep.generics.split_for_impl();
    let ocaml_type_ident_token_stream = &type_rep.ocaml_target_type_ident_ts;

    let record_var = Ident::new("record_root", proc_macro2::Span::call_site());
    let record_allocation = create_ocaml_block(fields.len(), 0); // Tag 0 for records/structs

    let container_expr = quote! { #record_var.get_raw() };
    let field_processing_quotes =
        generate_struct_field_conversions(fields, container_expr, |idx, field_rep| {
            if let Some(ident) = &field_rep.ident {
                quote! { self.#ident }
            } else {
                let i = syn::Index::from(idx);
                quote! { self.#i }
            }
        });

    let body = quote! {
        let #record_var: ::ocaml_interop::BoxRoot<()> = #record_allocation;
        #(#field_processing_quotes)*
        unsafe { ::ocaml_interop::OCaml::new(cr, #record_var.get_raw()) }
    };

    quote! {
        unsafe impl #impl_generics ::ocaml_interop::ToOCaml<#ocaml_type_ident_token_stream #ty_generics> for #type_ident #ty_generics #where_clause {
            fn to_ocaml<'a>(&self, cr: &'a mut ::ocaml_interop::OCamlRuntime) -> ::ocaml_interop::OCaml<'a, #ocaml_type_ident_token_stream #ty_generics> {
                #body
            }
        }
    }
}

fn codegen_enum_impl(
    type_rep: &TypeRep,
    variants: &[crate::common::parsing::VariantRep],
    kind: &EnumKind,
) -> TokenStream {
    let type_ident = &type_rep.ident;
    let (impl_generics, ty_generics, where_clause) = type_rep.generics.split_for_impl();
    let ocaml_type_ident_token_stream = &type_rep.ocaml_target_type_ident_ts;

    let match_arms = match kind {
        EnumKind::Regular => generate_regular_enum_arms(type_ident, variants),
        EnumKind::Polymorphic => generate_polymorphic_enum_arms(type_ident, variants),
    };

    quote! {
        unsafe impl #impl_generics ::ocaml_interop::ToOCaml<#ocaml_type_ident_token_stream #ty_generics> for #type_ident #ty_generics #where_clause {
            fn to_ocaml<'a>(&self, cr: &'a mut ::ocaml_interop::OCamlRuntime) -> ::ocaml_interop::OCaml<'a, #ocaml_type_ident_token_stream #ty_generics> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

/// Generate match arms for regular enums
fn generate_regular_enum_arms(
    type_ident: &Ident,
    variants: &[crate::common::parsing::VariantRep],
) -> Vec<TokenStream> {
    let mut unit_variant_idx = 0u8;
    let mut block_variant_idx = 0u8;

    variants
        .iter()
        .map(|variant_rep| match variant_rep.kind {
            VariantKind::Unit => {
                let arm =
                    generate_unit_variant_arm(type_ident, variant_rep, Some(unit_variant_idx));
                unit_variant_idx += 1;
                arm
            }
            VariantKind::Tuple | VariantKind::Struct => {
                let arm =
                    generate_regular_block_variant_arm(type_ident, variant_rep, block_variant_idx);
                block_variant_idx += 1;
                arm
            }
        })
        .collect()
}

/// Generate match arms for polymorphic enums
fn generate_polymorphic_enum_arms(
    type_ident: &Ident,
    variants: &[crate::common::parsing::VariantRep],
) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|variant_rep| {
            let variant_ident = &variant_rep.ident;
            match variant_rep.kind {
                VariantKind::Unit => generate_unit_variant_arm(type_ident, variant_rep, None),
                VariantKind::Tuple | VariantKind::Struct => {
                    let (pattern, field_idents) = generate_variant_pattern(variant_rep);
                    let polytag = generate_polytag_hash(
                        &variant_rep.ident,
                        variant_rep.attrs.get_tag().as_ref(),
                    );
                    let (setup_ops, ocaml_payload_value_expr) =
                        generate_poly_variant_payload(variant_rep, &field_idents);
                    let poly_block = create_polymorphic_variant_block(
                        quote! { polytag },
                        ocaml_payload_value_expr,
                    );

                    quote! {
                        #type_ident::#variant_ident #pattern => {
                            let polytag = #polytag;
                            #(#setup_ops)*
                            #poly_block
                        }
                    }
                }
            }
        })
        .collect()
}

fn codegen_to_ocaml(type_rep: &TypeRep) -> TokenStream {
    match &type_rep.data {
        TypeRepData::Struct { fields } => codegen_struct_impl(type_rep, fields),
        TypeRepData::Enum { variants, kind } => codegen_enum_impl(type_rep, variants, kind),
    }
}

pub fn expand_to_ocaml(input: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse2::<DeriveInput>(input).map_err(OCamlInteropError::Syn)?;
    let type_rep = parse_input(derive_input)?;

    validation::validate_type_rep(&type_rep)?;

    Ok(codegen_to_ocaml(&type_rep))
}
