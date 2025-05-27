// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, Meta, Token};

use crate::export::core::{ExportedFnData, ProcessedArg};
// Import shared parsing utilities from common
use crate::common::parsing::{
    check_duplicate_attr, parse_string_literal_attribute, process_extern_argument,
    process_flag_attribute, process_return_type,
};

/// Parse export attributes like bytecode, no_panic_catch, noalloc
fn parse_export_attributes(
    attr_ts: proc_macro2::TokenStream,
) -> Result<(Option<syn::Ident>, bool, bool), syn::Error> {
    let mut bytecode_fn_name_opt: Option<syn::Ident> = None;
    let mut no_panic_catch = false;
    let mut noalloc = false;

    // Keep track of spans only for error reporting during parsing
    let mut bytecode_meta_span: Option<proc_macro2::Span> = None;
    let mut no_panic_catch_span: Option<proc_macro2::Span> = None;
    let mut noalloc_span: Option<proc_macro2::Span> = None;

    let attribute_parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed_attributes = attribute_parser.parse2(attr_ts)?;

    for meta in parsed_attributes {
        if meta.path().is_ident("bytecode") {
            // Check for duplicate attribute
            check_duplicate_attr("bytecode", meta.path(), &bytecode_meta_span)?;

            // Process the bytecode attribute
            match meta {
                syn::Meta::NameValue(mnv) => {
                    let bytecode_name = parse_string_literal_attribute(&mnv, "bytecode")?;
                    bytecode_fn_name_opt = Some(syn::Ident::new(&bytecode_name, mnv.path.span()));
                    bytecode_meta_span = Some(mnv.path.span());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "'bytecode' attribute must be a name-value pair (e.g., bytecode = \"my_func_byte\")"
                    ));
                }
            }
        } else if meta.path().is_ident("no_panic_catch") {
            // Check for duplicate attribute
            check_duplicate_attr("no_panic_catch", meta.path(), &no_panic_catch_span)?;

            // Process the no_panic_catch flag attribute
            let (value, span) = process_flag_attribute(&meta, "no_panic_catch")?;
            no_panic_catch = value;
            no_panic_catch_span = span;
        } else if meta.path().is_ident("noalloc") {
            // Check for duplicate attribute
            check_duplicate_attr("noalloc", meta.path(), &noalloc_span)?;

            // Process the noalloc flag attribute
            let (value, span) = process_flag_attribute(&meta, "noalloc")?;
            noalloc = value;
            noalloc_span = span;
        } else {
            return Err(syn::Error::new_spanned(meta.path(), format!(
                "unsupported attribute '{}', only 'bytecode', 'no_panic_catch' and 'noalloc' are supported",
                meta.path().get_ident().map_or_else(|| String::from("?"), |i| i.to_string())
            )));
        }
    }

    Ok((bytecode_fn_name_opt, no_panic_catch, noalloc))
}

/// Main entry point for parsing export function definitions
pub(crate) fn parse_export_definition(
    attr_ts: proc_macro2::TokenStream,
    input_fn: &ItemFn,
) -> Result<ExportedFnData, syn::Error> {
    let (bytecode_fn_name_opt, no_panic_catch, noalloc) = parse_export_attributes(attr_ts)?;

    let native_fn_name = input_fn.sig.ident.clone();
    let visibility = input_fn.vis.clone();
    let function_body = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    let has_generics = !input_fn.sig.generics.params.is_empty();
    let is_variadic = input_fn.sig.variadic.is_some();
    let abi = input_fn.sig.abi.clone();

    let fn_inputs = &input_fn.sig.inputs;
    let original_fn_return_type_ast = &input_fn.sig.output;
    let fn_inputs_span = fn_inputs.span();
    let original_fn_ident_span = input_fn.sig.ident.span();

    let mut original_fn_args_iter = fn_inputs.iter();

    let fn_arg = original_fn_args_iter.next().ok_or_else(|| {
        syn::Error::new(
            fn_inputs_span,
            "Exported functions must take an OCamlRuntime as their first argument.",
        )
    })?;

    let (runtime_arg_pat, runtime_arg_ty) = match fn_arg {
        FnArg::Typed(pt) => (Box::new(pt.pat.as_ref().clone()), pt.ty.clone()),
        FnArg::Receiver(rec) => {
            return Err(syn::Error::new_spanned(
                rec,
                "OCamlRuntime argument cannot be a receiver (self).",
            ))
        }
    };

    let mut processed_args: Vec<ProcessedArg> = Vec::new();
    for arg in original_fn_args_iter {
        processed_args.push(process_extern_argument(arg)?);
    }

    let (return_interop_detail, user_return_type_ast) =
        process_return_type(original_fn_return_type_ast)?;

    Ok(ExportedFnData {
        bytecode_fn_name_opt,
        no_panic_catch,
        noalloc,
        native_fn_name,
        visibility,
        original_fn_block: function_body,
        runtime_arg_pat,
        runtime_arg_ty,
        processed_args,
        return_interop_detail,
        user_return_type_ast,
        is_async,
        has_generics,
        is_variadic,
        abi,
        original_fn_ident_span,
    })
}
