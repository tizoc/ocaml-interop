// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::common::parsing::FieldRep;

pub fn make_field_var(idx: usize, span: Option<Span>) -> Ident {
    let span = span.unwrap_or_else(Span::call_site);
    Ident::new(&format!("field{idx}"), span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDirection {
    ToOCaml,
    FromOCaml,
}

pub fn get_ocaml_type(field_rep: &FieldRep, _direction: TypeDirection) -> TokenStream {
    match &field_rep.ocaml_as_type_override_ts {
        Some(as_ts) => quote! { #as_ts },
        None => {
            let field_type = &field_rep.ty;
            quote! { <#field_type as ::ocaml_interop::DefaultOCamlMapping>::OCamlType }
        }
    }
}

pub fn generate_field_conversion_and_storage(
    field_rep: &FieldRep,
    field_access: TokenStream,
    var_name: &Ident,
    container_expr: TokenStream,
    field_idx: usize,
) -> TokenStream {
    let ocaml_type = get_ocaml_type(field_rep, TypeDirection::ToOCaml);
    let idx_lit = syn::LitInt::new(&format!("{field_idx}usize"), Span::call_site());

    quote! {
        let #var_name: ::ocaml_interop::OCaml<#ocaml_type> = #field_access.to_ocaml(cr);
        unsafe { ::ocaml_interop::internal::store_field(#container_expr, #idx_lit, #var_name.raw()); }
    }
}

pub fn generate_struct_field_extractions(
    fields: &[FieldRep],
    ocaml_value_expr: TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field_rep)| {
            let var_name = Ident::new(&format!("rust_field_{idx}"), field_rep.span);
            let ocaml_type = get_ocaml_type(field_rep, TypeDirection::FromOCaml);
            let idx_lit = syn::LitInt::new(&format!("{idx}"), Span::call_site());

            quote! {
                let #var_name = #ocaml_value_expr.field::<#ocaml_type>(#idx_lit).to_rust();
            }
        })
        .collect()
}

pub fn generate_struct_field_conversions(
    fields: &[FieldRep],
    container_expr: TokenStream,
    self_access_pattern: impl Fn(usize, &FieldRep) -> TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field_rep)| {
            let field_access = self_access_pattern(idx, field_rep);
            let var_name = if let Some(ident) = &field_rep.ident {
                Ident::new(&format!("field_{ident}_ocaml"), field_rep.span)
            } else {
                Ident::new(&format!("field_{idx}_ocaml"), field_rep.span)
            };

            generate_field_conversion_and_storage(
                field_rep,
                field_access,
                &var_name,
                container_expr.clone(),
                idx,
            )
        })
        .collect()
}

pub fn create_ocaml_block(size: usize, tag: usize) -> TokenStream {
    let size_lit = syn::LitInt::new(&format!("{size}usize"), Span::call_site());
    let tag_lit = syn::LitInt::new(&format!("{tag}"), Span::call_site());

    quote! {
        ::ocaml_interop::BoxRoot::new(unsafe {
            ::ocaml_interop::OCaml::new(cr, ::ocaml_interop::internal::caml_alloc(#size_lit, #tag_lit))
        })
    }
}

pub fn create_ocaml_tuple(size: usize) -> TokenStream {
    let size_lit = syn::LitInt::new(&format!("{size}usize"), Span::call_site());

    quote! {
        ::ocaml_interop::BoxRoot::new(unsafe {
            ::ocaml_interop::internal::alloc_tuple(cr, #size_lit)
        })
    }
}

pub fn create_polymorphic_variant_block(
    polytag_expr: TokenStream,
    payload_expr: TokenStream,
) -> TokenStream {
    quote! {
        unsafe {
            let block = ::ocaml_interop::internal::caml_alloc(
                2usize, // Size is 2: [hash, payload]
                ::ocaml_interop::internal::tag::TAG_POLYMORPHIC_VARIANT,
            );
            ::ocaml_interop::internal::store_field(block, 0, #polytag_expr);
            ::ocaml_interop::internal::store_field(block, 1, #payload_expr);
            ::ocaml_interop::OCaml::new(cr, block)
        }
    }
}
