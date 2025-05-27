// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

extern crate proc_macro;
use proc_macro::TokenStream;
use syn::ItemFn;

mod common;
mod export;
mod from_ocaml;
mod ocaml_describer;
mod to_ocaml;

#[cfg(test)]
mod tests {
    mod export_tests;
    mod from_ocaml_tests;
    mod ocaml_describer_tests;
    mod to_ocaml_tests;
    mod to_ocaml_type_safety_tests;
}

fn export_internal_logic(
    attr_ts: proc_macro2::TokenStream,
    item_ts: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_fn = syn::parse2::<ItemFn>(item_ts.clone()).map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("Failed to parse input item as a function: {e}. Input was: {item_ts}",),
        )
    })?;

    let parsed_data = export::parsing::parse_export_definition(attr_ts, &input_fn)?;

    export::validation::validate_parsed_data(&parsed_data)?;

    export::codegen::expand_function_from_data(&parsed_data)
}

// --- Proc Macro Entry Point ---
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ts2 = proc_macro2::TokenStream::from(attr);
    let item_ts2 = proc_macro2::TokenStream::from(item);

    match export_internal_logic(attr_ts2, item_ts2) {
        Ok(generated_code) => TokenStream::from(generated_code),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

#[proc_macro_derive(OCamlDescriber, attributes(ocaml))]
pub fn ocaml_describer_derive(input: TokenStream) -> TokenStream {
    // Convert proc_macro::TokenStream to proc_macro2::TokenStream for internal use
    let input_pm2 = proc_macro2::TokenStream::from(input);
    ocaml_describer::codegen::expand_ocaml_describer(input_pm2)
        .unwrap_or_else(|err| err.to_compile_error())
        .into() // Convert proc_macro2::TokenStream back to proc_macro::TokenStream
}

#[proc_macro_derive(ToOCaml, attributes(ocaml))]
pub fn to_ocaml_derive(input: TokenStream) -> TokenStream {
    // Convert proc_macro::TokenStream to proc_macro2::TokenStream for internal use
    let input_pm2 = proc_macro2::TokenStream::from(input);
    to_ocaml::codegen::expand_to_ocaml(input_pm2)
        .unwrap_or_else(|err| err.to_compile_error())
        .into() // Convert proc_macro2::TokenStream back to proc_macro::TokenStream
}

#[proc_macro_derive(FromOCaml, attributes(ocaml))]
pub fn from_ocaml_derive(input: TokenStream) -> TokenStream {
    // Convert proc_macro::TokenStream to proc_macro2::TokenStream for internal use
    let input_pm2 = proc_macro2::TokenStream::from(input);
    from_ocaml::codegen::expand_from_ocaml(input_pm2)
        .unwrap_or_else(|err| err.to_compile_error())
        .into() // Convert proc_macro2::TokenStream back to proc_macro::TokenStream
}
