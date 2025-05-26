// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Error, Pat};

use crate::export::{core::ExportedFnData, core::InteropTypeDetail};

/// Generate function parameter signature and initialization code for arguments
fn process_function_arguments(
    data: &ExportedFnData,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>), Error> {
    let mut extern_c_fn_params_sig_parts = Vec::new();
    let mut initializations = Vec::new();

    for arg in &data.processed_args {
        let original_pat = &arg.pattern;
        let original_rust_ty = &arg.original_rust_type;

        match &arg.type_detail {
            InteropTypeDetail::Primitive { .. } => {
                let sig_part =
                    quote_spanned! {original_rust_ty.span()=> #original_pat: #original_rust_ty };
                extern_c_fn_params_sig_parts.push(sig_part);
                // No special initialization for primitives
            }
            InteropTypeDetail::OCaml { inner_type, .. } => {
                let sig_part = quote_spanned! {original_rust_ty.span()=> #original_pat: ::ocaml_interop::RawOCaml };
                extern_c_fn_params_sig_parts.push(sig_part);

                let runtime_arg_pat = &data.runtime_arg_pat;
                let init_part = quote_spanned! {original_rust_ty.span()=>
                    let #original_pat : #original_rust_ty = unsafe { ::ocaml_interop::OCaml::<#inner_type>::new(#runtime_arg_pat, #original_pat) };
                };
                initializations.push(init_part);
            }
            InteropTypeDetail::BoxRoot { inner_type, .. } => {
                let sig_part = quote_spanned! {original_rust_ty.span()=> #original_pat: ::ocaml_interop::RawOCaml };
                extern_c_fn_params_sig_parts.push(sig_part);

                let runtime_arg_pat = &data.runtime_arg_pat;
                let init_part = quote_spanned! {original_rust_ty.span()=>
                    let #original_pat : #original_rust_ty = ::ocaml_interop::BoxRoot::new(unsafe {
                        ::ocaml_interop::OCaml::<#inner_type>::new(#runtime_arg_pat, #original_pat)
                    });
                };
                initializations.push(init_part);
            }
            InteropTypeDetail::Unit => {
                // This should be caught during parsing
                return Err(Error::new_spanned(
                    original_rust_ty,
                    "Internal error: Unit type encountered for argument in expansion phase.",
                ));
            }
        }
    }

    Ok((extern_c_fn_params_sig_parts, initializations))
}

/// Generate return type signature and expression logic
fn process_return_type(data: &ExportedFnData) -> Result<(TokenStream, TokenStream), Error> {
    let user_return_type_ast = &data.user_return_type_ast;
    let fn_body = &data.original_fn_block;

    match &data.return_interop_detail {
        InteropTypeDetail::Unit => Ok((
            quote_spanned! {user_return_type_ast.span()=> -> ::ocaml_interop::RawOCaml },
            quote_spanned! {fn_body.span()=>
                #fn_body; // Execute for side effects
                ::ocaml_interop::internal::UNIT // Return OCaml's unit
            },
        )),
        InteropTypeDetail::Primitive {
            original_rust_type, ..
        } => {
            let ort_for_sig = original_rust_type.clone();
            Ok((
                quote_spanned! {user_return_type_ast.span()=> -> #ort_for_sig },
                quote_spanned! {fn_body.span()=>
                    let result_from_body: #original_rust_type = #fn_body;
                    result_from_body
                },
            ))
        }
        InteropTypeDetail::OCaml { wrapper_type, .. } => Ok((
            quote_spanned! {user_return_type_ast.span()=> -> ::ocaml_interop::RawOCaml },
            quote_spanned! {fn_body.span()=>
                let result_from_body: #wrapper_type = #fn_body;
                unsafe { result_from_body.raw() } // OCaml<T>.raw() returns RawOCaml
            },
        )),
        InteropTypeDetail::BoxRoot { .. } => {
            // This should have been caught by validation
            Err(Error::new_spanned(
                user_return_type_ast,
                "Internal error: BoxRoot<T> should not be possible as a return type in expansion phase.",
            ))
        }
    }
}

/// Process a single argument for bytecode function generation
fn process_bytecode_argument(
    p_arg: &crate::export::core::ProcessedArg,
    index: usize,
) -> Result<(Vec<TokenStream>, TokenStream), Error> {
    let mut preparations = Vec::new();
    let raw_val_ident = format_ident!("__ocaml_interop_arg_{}", index);
    let target_var_ident = match &*p_arg.pattern {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        _ => format_ident!("__ocaml_interop_converted_arg_{}", index),
    };

    preparations.push(quote! {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        let #raw_val_ident = unsafe { ::core::ptr::read(argv.add(#index)) };
    });

    match &p_arg.type_detail {
        InteropTypeDetail::OCaml { .. } | InteropTypeDetail::BoxRoot { .. } => {
            preparations.push(quote! {
                let #target_var_ident = #raw_val_ident;
            });
        }
        InteropTypeDetail::Primitive { .. } => {
            let conversion_fn_name_str = p_arg.type_detail.get_ocaml_to_rust_fn_name()?;
            let conversion_fn_name = format_ident!("{}", conversion_fn_name_str);
            let conversion_path = p_arg.type_detail.get_conversion_module_path_tokens();
            preparations.push(quote! {
                let #target_var_ident = #conversion_path #conversion_fn_name(#raw_val_ident);
            });
        }
        InteropTypeDetail::Unit => {
            return Err(Error::new_spanned(
                &*p_arg.pattern,
                "Internal error: Unit type encountered for argument in bytecode generation.",
            ));
        }
    }

    Ok((preparations, quote! { #target_var_ident }))
}

/// Generate bytecode version of the function if requested
fn generate_bytecode_function(
    data: &ExportedFnData,
    bytecode_fn_name: &syn::Ident,
) -> Result<TokenStream, Error> {
    let native_fn_name = &data.native_fn_name;
    let visibility = &data.visibility;
    let arg_count = data.processed_args.len();

    let mut bytecode_arg_preparations = Vec::new();
    let mut native_call_args_for_bytecode = Vec::new();

    for (i, p_arg) in data.processed_args.iter().enumerate() {
        let (mut preps, arg) = process_bytecode_argument(p_arg, i)?;
        bytecode_arg_preparations.append(&mut preps);
        native_call_args_for_bytecode.push(arg);
    }

    let return_conversion_logic = if data.return_interop_detail.is_primitive() {
        let alloc_fn_name_str = data.return_interop_detail.get_rust_to_ocaml_fn_name()?;
        let alloc_fn_name = format_ident!("{}", alloc_fn_name_str);
        let conversion_path = data
            .return_interop_detail
            .get_conversion_module_path_tokens();
        quote! { #conversion_path #alloc_fn_name(result) }
    } else {
        quote! { result }
    };

    let arity_check_panic = quote! {
        panic!("Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
               stringify!(#bytecode_fn_name), #arg_count, argn);
    };

    Ok(quote! {
        #[no_mangle]
        #visibility extern "C" fn #bytecode_fn_name(
            argv: *mut ::ocaml_interop::RawOCaml,
            argn: ::std::os::raw::c_int
        ) -> ::ocaml_interop::RawOCaml {
            #(#bytecode_arg_preparations)*

            if cfg!(debug_assertions) {
                if (argn as usize) != #arg_count {
                    #arity_check_panic
                }
            }

            let result = #native_fn_name(#(#native_call_args_for_bytecode),*);
            #return_conversion_logic
        }
    })
}

/// Main entry point for generating export function code
pub(crate) fn expand_function_from_data(data: &ExportedFnData) -> Result<TokenStream, Error> {
    // Step 1: Generate runtime handle recovery code
    let runtime_arg_pat = &data.runtime_arg_pat;
    let runtime_arg_ty = &data.runtime_arg_ty;
    let runtime_handle_recovery = if data.noalloc {
        quote! { let #runtime_arg_pat : #runtime_arg_ty = unsafe { ::ocaml_interop::internal::recover_runtime_handle() }; }
    } else {
        quote! { let #runtime_arg_pat : #runtime_arg_ty = unsafe { ::ocaml_interop::internal::recover_runtime_handle_mut() }; }
    };

    // Step 2: Process function arguments
    let (extern_c_fn_params_sig_parts, initializations) = process_function_arguments(data)?;

    // Step 3: Process return type
    let (extern_c_return_type_sig, final_return_expression_logic) = process_return_type(data)?;

    // Step 4: Build the main logic block
    let main_logic_block = quote! {
        #runtime_handle_recovery
        #(#initializations)*
        #final_return_expression_logic
    };

    // Step 5: Apply panic handling if needed
    let final_panic_handled_logic = if data.noalloc || data.no_panic_catch {
        main_logic_block.clone() // No panic catching
    } else {
        // Default: catch panics
        quote! {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                #main_logic_block
            }));
            match result {
                Ok(value) => value,
                Err(panic_payload) => {
                    unsafe {
                        ::ocaml_interop::internal::process_panic_payload_and_raise_ocaml_exception(panic_payload);
                        // This part should be unreachable as the above function raises an OCaml exception and doesn't return.
                        ::std::unreachable!("process_panic_payload_and_raise_ocaml_exception should not return");
                    }
                }
            }
        }
    };

    // Step 6: Generate the native function implementation
    let visibility = &data.visibility;
    let native_fn_name = &data.native_fn_name;
    let native_fn_impl = quote! {
        #[no_mangle]
        #visibility extern "C" fn #native_fn_name(#(#extern_c_fn_params_sig_parts),*) #extern_c_return_type_sig {
            #final_panic_handled_logic
        }
    };

    // Step 7: Build result collection, starting with the native function
    let mut all_generated_code = vec![native_fn_impl];

    // Step 8: Generate bytecode function if needed
    if let Some(bytecode_fn_name) = &data.bytecode_fn_name_opt {
        let bytecode_fn_impl = generate_bytecode_function(data, bytecode_fn_name)?;
        all_generated_code.push(bytecode_fn_impl);
    }

    Ok(quote! { #(#all_generated_code)* })
}
