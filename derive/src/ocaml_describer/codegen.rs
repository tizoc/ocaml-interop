// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

use crate::common::attr_parsing::OCamlAttributes;

/// Add OCamlDescriber trait bound to a where clause for generic type parameters
fn add_ocaml_describer_bounds(
    generics: &syn::Generics,
    mut where_clause: syn::WhereClause,
) -> syn::WhereClause {
    for param in generics.type_params() {
        let param_ident = &param.ident;
        where_clause
            .predicates
            .push(syn::parse_quote! { #param_ident: ocaml_interop::OCamlDescriber });
    }
    where_clause
}

/// Generate the OCaml type name code based on generic parameters
fn generate_type_name_code(ocaml_type_name_str: &str, generics: &syn::Generics) -> TokenStream {
    let param_count = generics.type_params().count();

    match param_count {
        0 => quote! { #ocaml_type_name_str.to_string() },

        1 => {
            // Single parameter case: "param_name type_name"
            let param_ident = &generics.type_params().next().unwrap().ident;
            quote! {
                format!(
                    "{} {}",
                    <#param_ident as ocaml_interop::OCamlDescriber>::ocaml_type_name(),
                    #ocaml_type_name_str
                )
            }
        }

        _ => {
            // Multiple parameters case: "(param1_name, param2_name, ...) type_name"
            let param_name_calls = generics.type_params().map(|param| {
                let param_ident = &param.ident;
                quote! { <#param_ident as ocaml_interop::OCamlDescriber>::ocaml_type_name() }
            });

            quote! {
                format!("({}) {}",
                    vec![#(#param_name_calls),*].join(", "),
                    #ocaml_type_name_str
                )
            }
        }
    }
}

pub fn expand_ocaml_describer(input: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse2::<DeriveInput>(input)?;
    let type_ident = &derive_input.ident;

    // Parse #[ocaml(...)] attributes on the type
    let type_attrs = OCamlAttributes::from_attrs(&derive_input.attrs)?;

    // Get OCaml type name from attribute or convert from Rust name
    let ocaml_type_name_str = type_attrs
        .get_name()
        .clone()
        .unwrap_or_else(|| type_ident.to_string().to_snake_case());

    // Handle generics
    let generics = &derive_input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add OCamlDescriber bound to generic type parameters
    let where_clause = add_ocaml_describer_bounds(
        generics,
        where_clause
            .cloned()
            .unwrap_or_else(|| syn::parse_quote! { where }),
    );

    // Generate the implementation
    let final_ocaml_name_code = generate_type_name_code(&ocaml_type_name_str, generics);

    let expanded = quote! {
        impl #impl_generics ocaml_interop::OCamlDescriber for #type_ident #ty_generics #where_clause {
            fn ocaml_type_name() -> String {
                #final_ocaml_name_code
            }
        }
    };

    Ok(expanded)
}
