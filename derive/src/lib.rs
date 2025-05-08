use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Block, FnArg, ItemFn, Pat, PathArguments, ReturnType, Type}; // Removed LitStr and parse_macro_input
use syn::punctuated::Punctuated;
use syn::{Meta, Token, Expr, ExprLit, Lit};
use syn::parse::Parser; // Added this line

// Helper function to extract generic type arguments
fn extract_inner_type_from_path(
    type_path: &syn::TypePath,
    expected_wrapper_ident: &str,
) -> Result<Box<syn::Type>, syn::Error> {
    let last_segment = type_path.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(
            type_path,
            format!(
                "Type path for {} must have segments",
                expected_wrapper_ident
            ),
        )
    })?;

    if last_segment.ident != expected_wrapper_ident {
        return Err(syn::Error::new_spanned(
            type_path,
            format!(
                "Expected type wrapper {}, found {}",
                expected_wrapper_ident, last_segment.ident
            ),
        ));
    }

    match &last_segment.arguments {
        PathArguments::AngleBracketed(params) => match params.args.first() {
            Some(syn::GenericArgument::Type(inner_ty)) => Ok(Box::new(inner_ty.clone())),
            _ => Err(syn::Error::new_spanned(
                &last_segment.arguments,
                format!(
                    "{}<T> type argument T is missing or not a type",
                    expected_wrapper_ident
                ),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            &last_segment.arguments,
            format!(
                "{}<T> type argument T must be angle bracketed",
                expected_wrapper_ident
            ),
        )),
    }
}

// Helper function to validate the runtime argument
fn validate_runtime_argument(arg_input: Option<&FnArg>) -> Result<&Pat, syn::Error> {
    match arg_input {
        Some(FnArg::Typed(pat_type)) => {
            let user_provided_type = &pat_type.ty;
            match &**user_provided_type {
                Type::Reference(type_ref) => {
                    if type_ref.mutability.is_none() {
                        return Err(syn::Error::new_spanned(
                            user_provided_type,
                            "Expected first argument to be a mutable reference (e.g., &mut OCamlRuntime)",
                        ));
                    }
                    match &*type_ref.elem {
                        Type::Path(type_path) => {
                            let path = &type_path.path;
                            let num_segments = path.segments.len();
                            if num_segments == 0 {
                                return Err(syn::Error::new_spanned(
                                    path,
                                    "Type path for OCamlRuntime cannot be empty",
                                ));
                            }
                            let last_segment = path.segments.last().unwrap();
                            if last_segment.ident != "OCamlRuntime" {
                                return Err(syn::Error::new_spanned(
                                    last_segment,
                                    "Expected referenced type to be OCamlRuntime",
                                ));
                            }
                            if num_segments > 1 {
                                let pre_last_segment_idx = num_segments - 2;
                                if path.segments[pre_last_segment_idx].ident != "ocaml_interop" {
                                    return Err(syn::Error::new_spanned(
                                        &path.segments[pre_last_segment_idx],
                                        "OCamlRuntime should be qualified with ocaml_interop (e.g., ocaml_interop::OCamlRuntime) or imported directly",
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &*type_ref.elem,
                                "Expected referenced type to be OCamlRuntime (e.g. &mut ocaml_interop::OCamlRuntime)",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        user_provided_type,
                        "Expected first argument to be a mutable reference (e.g., &mut OCamlRuntime)",
                    ));
                }
            }
            Ok(&pat_type.pat) // Return the pattern if validation passes
        }
        _ => Err(syn::Error::new_spanned(
            arg_input.unwrap_or(&FnArg::Receiver(syn::parse_quote! {self})),
            "Function must have at least one typed argument for the OCaml runtime handle",
        )),
    }
}

// Helper function to process other (non-runtime) arguments
fn process_extern_argument<'a>(
    arg_input: &'a FnArg,
    runtime_arg_pat: &Pat,
) -> Result<
    (
        proc_macro2::TokenStream, // C signature part
        proc_macro2::TokenStream, // Rust initialization part
        Box<Pat>,                 // Argument pattern
        bool,                     // is_f64 flag
    ),
    syn::Error,
> {
    if let FnArg::Typed(pat_type) = arg_input {
        let original_pat = &pat_type.pat;
        let original_ty = &pat_type.ty; // This is Box<Type>

        // 1. Handle f64: always raw, C type is f64
        if let Type::Path(type_path) = &**original_ty {
            if type_path
                .path
                .segments
                .last()
                .map_or(false, |s| s.ident == "f64")
            {
                let sig_part = quote! { #original_pat: f64 };
                let init_part = quote! {}; // No special initialization
                return Ok((sig_part, init_part, original_pat.clone(), true)); // true for is_f64
            }
        }

        // 2. Handle BoxRoot<T> or OCaml<T>
        if let Type::Path(type_path) = &**original_ty {
            let last_segment = type_path.path.segments.last().ok_or_else(|| {
                syn::Error::new_spanned(type_path, "Type path must have segments")
            })?;

            if last_segment.ident == "BoxRoot" {
                // Expects BoxRoot<T>, implies BoxRooting
                match extract_inner_type_from_path(type_path, "BoxRoot") {
                    Ok(inner_type) => {
                        let sig_part = quote! { #original_pat: ::ocaml_interop::RawOCaml };
                        let init_part = quote! {
                            let #original_pat : #original_ty = ::ocaml_interop::BoxRoot::new(unsafe {
                                ::ocaml_interop::OCaml::<#inner_type>::new(#runtime_arg_pat, #original_pat)
                            });
                        };
                        Ok((sig_part, init_part, original_pat.clone(), false)) // false for is_f64
                    }
                    Err(e) => Err(syn::Error::new_spanned(
                        original_ty,
                        format!(
                            "Argument `{}`: Error parsing BoxRoot<T>: {}. Expected BoxRoot<T>, OCaml<T>, or f64.",
                            quote! {#original_pat}.to_string(),
                            e
                        ),
                    )),
                }
            } else if last_segment.ident == "OCaml" {
                // Expects OCaml<T>, implies raw handling (no BoxRoot by macro)
                match extract_inner_type_from_path(type_path, "OCaml") {
                    Ok(inner_type) => {
                        let sig_part = quote! { #original_pat: ::ocaml_interop::RawOCaml };
                        let init_part = quote! {
                            let #original_pat : #original_ty = unsafe { ::ocaml_interop::OCaml::<#inner_type>::new(#runtime_arg_pat, #original_pat) };
                        };
                        Ok((sig_part, init_part, original_pat.clone(), false)) // false for is_f64
                    }
                    Err(e) => Err(syn::Error::new_spanned(
                        original_ty,
                        format!(
                            "Argument `{}`: Error parsing OCaml<T>: {}. Expected BoxRoot<T>, OCaml<T>, or f64.",
                            quote! {#original_pat}.to_string(),
                            e
                        ),
                    )),
                }
            } else {
                // Type is a Path, but not f64, BoxRoot, or OCaml
                Err(syn::Error::new_spanned(
                    original_ty,
                    format!(
                        "Argument `{}` (type `{}`): type must be f64, BoxRoot<T>, or OCaml<T>. Found `{}`.",
                        quote! {#original_pat}.to_string(),
                        quote! {#original_ty}.to_string(),
                        last_segment.ident
                    ),
                ))
            }
        } else {
            // Type is not a Type::Path (e.g., reference, tuple, etc.) and not f64.
            Err(syn::Error::new_spanned(
                original_ty,
                format!(
                    "Argument `{}` (type `{}`): type must be f64, BoxRoot<T>, or OCaml<T>. It is not a simple path type.",
                    quote! {#original_pat}.to_string(),
                    quote! {#original_ty}.to_string()
                ),
            ))
        }
    } else {
        Err(syn::Error::new_spanned(
            arg_input,
            "Receiver arguments (self) are not supported in ocaml_interop::export functions",
        ))
    }
}

// Helper function to process return type
fn process_return_type(
    original_fn_return_type_ast: &ReturnType,
    fn_body: &Block,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match original_fn_return_type_ast {
        ReturnType::Default => {
            // User's function returns () implicitly.
            (
                quote! { -> ::ocaml_interop::RawOCaml }, // FFI signature still returns RawOCaml
                quote! {
                    #fn_body; // Execute the user's code block for its side effects.
                    ::ocaml_interop::internal::UNIT // Return OCaml's unit value.
                },
            )
        }
        ReturnType::Type(_, ty_box) => {
            // ty_box is Box<Type>
            let user_return_type = &**ty_box; // This is the Type user wrote, e.g., f64 or OCaml<Something>
            let is_f64_return = match user_return_type {
                Type::Path(type_path) => type_path
                    .path
                    .segments
                    .last()
                    .map_or(false, |s| s.ident == "f64"),
                _ => false,
            };

            if is_f64_return {
                (
                    quote! { -> f64 },
                    quote! {
                        let result_from_body: f64 = #fn_body;
                        result_from_body // Directly return f64
                    },
                )
            } else {
                // Assume OCaml<T> for other return types
                (
                    quote! { -> ::ocaml_interop::RawOCaml },
                    quote! {
                        let result_from_body: #user_return_type = #fn_body;
                        let final_result_for_ocaml: ::ocaml_interop::OCaml<_> = result_from_body;
                        unsafe { final_result_for_ocaml.raw() }
                    },
                )
            }
        }
    }
}

// Renamed from original `export` and modified to use proc_macro2 types
fn export_internal_logic(attr_ts: proc_macro2::TokenStream, item_ts: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_fn = syn::parse2::<ItemFn>(item_ts)?;

    let mut bytecode_fn_name_opt: Option<syn::Ident> = None;
    let mut bytecode_meta_span: Option<proc_macro2::Span> = None;
    let mut no_panic_catch = false;
    let mut no_panic_catch_span: Option<proc_macro2::Span> = None;

    // Correctly parse attribute arguments for syn 2.0
    let attribute_parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed_attributes = attribute_parser.parse2(attr_ts)?;

    for meta in parsed_attributes {
        if meta.path().is_ident("bytecode") {
            if bytecode_fn_name_opt.is_some() {
                let mut err = syn::Error::new_spanned(
                    &meta.path(),
                    "\'bytecode\' attribute specified multiple times",
                );
                if let Some(prev_span) = bytecode_meta_span {
                    err.combine(syn::Error::new(
                        prev_span,
                        "previous \'bytecode\' attribute here",
                    ));
                }
                return Err(err);
            }
            match meta {
                syn::Meta::NameValue(mnv) => {
                    if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = mnv.value {
                        bytecode_fn_name_opt = Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                        bytecode_meta_span = Some(mnv.path.span());
                    } else {
                        return Err(syn::Error::new_spanned(mnv.value, "\'bytecode\' attribute value must be a string literal (e.g., bytecode = \\\"my_func_byte\\\")"));
                    }
                }
                _ => {
                     return Err(syn::Error::new_spanned(meta, "\'bytecode\' attribute must be a name-value pair (e.g., bytecode = \\\"my_func_byte\\\")"));
                }
            }
        } else if meta.path().is_ident("no_panic_catch") {
            if no_panic_catch_span.is_some() {
                return Err(syn::Error::new_spanned(
                    &meta.path(),
                    "\'no_panic_catch\' attribute specified multiple times",
                ));
            }
            match meta {
                syn::Meta::Path(_) => {
                    no_panic_catch = true;
                    no_panic_catch_span = Some(meta.path().span());
                }
                _ => {
                    return Err(syn::Error::new_spanned(meta, "\'no_panic_catch\' attribute should be a bare path (e.g., #[export(no_panic_catch)]), not a name-value or list"));
                }
            }
        } else {
            return Err(syn::Error::new_spanned(meta.path(), format!(
                "unsupported attribute \'{}\', only \'bytecode\' and \'no_panic_catch\' are supported",
                meta.path()
                    .get_ident()
                    .map_or_else(|| "?".to_string(), |i| i.to_string())
            )));
        }
    }

    let original_fn_ident = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;
    let original_fn_return_type_ast = &input_fn.sig.output;

    let native_fn_name = original_fn_ident.clone();

    let mut original_fn_args_iter = fn_inputs.iter();

    // 1. Runtime argument: Use the helper function for validation
    let runtime_arg_pat = match validate_runtime_argument(original_fn_args_iter.next()) {
        Ok(pat) => pat,
        Err(err) => return Err(err),
    };

    // 2. Process other arguments for extern "C" signature and BoxRooting
    let mut extern_c_fn_params_sig_parts = Vec::new();
    let mut boxroot_initializations = Vec::new();
    // let mut extern_arg_pats_for_bytecode = Vec::new(); // To store arg patterns for bytecode wrapper
    let mut processed_args_info: Vec<(Box<Pat>, bool)> = Vec::new();

    for arg in original_fn_args_iter {
        match process_extern_argument(arg, runtime_arg_pat) {
            Ok((sig_part, init_part, arg_pat, is_f64_flag)) => {
                extern_c_fn_params_sig_parts.push(sig_part);
                if !init_part.is_empty() {
                    // Only add if there's an initialization (i.e., not f64)
                    boxroot_initializations.push(init_part);
                }
                // extern_arg_pats_for_bytecode.push(arg_pat.clone()); // Store the pattern
                processed_args_info.push((arg_pat, is_f64_flag));
            }
            Err(err) => return Err(err),
        }
    }

    // 3. Determine extern "C" function return type and final conversion logic
    let (extern_fn_return_type_sig, final_return_expression_logic) =
        process_return_type(original_fn_return_type_ast, fn_body);

    // 4. Assemble the full expanded function
    let main_logic_block = quote! {
        let #runtime_arg_pat = unsafe { &mut ::ocaml_interop::internal::recover_runtime_handle_mut() };

        #(#boxroot_initializations)*

        #final_return_expression_logic
    };

    let panic_handled_logic = if no_panic_catch {
        main_logic_block // No panic catching
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
                        ::std::unreachable!(
                            "process_panic_payload_and_raise_ocaml_exception should not return"
                        );
                    }
                }
            }
        }
    };

    let native_fn_impl = quote! {
        #[no_mangle]
        pub extern "C" fn #native_fn_name(#(#extern_c_fn_params_sig_parts),*) #extern_fn_return_type_sig {
            #panic_handled_logic
        }
    };

    let mut all_generated_code = vec![native_fn_impl];

    // Bytecode function generation is now conditional and uses the name from the attribute
    if let Some(bytecode_fn_name) = bytecode_fn_name_opt {
        let mut arg_setup_code = Vec::new();
        let mut call_args_for_native_fn = Vec::new();
        let arg_count = processed_args_info.len();

        for (i, (arg_pat, is_f64_arg)) in processed_args_info.iter().enumerate() {
            let raw_val_ident =
                syn::Ident::new(&format!("__ocaml_interop_arg_{}", i), arg_pat.span());

            arg_setup_code.push(quote_spanned! {arg_pat.span()=>
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                let #raw_val_ident = unsafe { ::core::ptr::read(argv.add(#i)) };
            });

            if *is_f64_arg {
                arg_setup_code.push(quote_spanned! {arg_pat.span()=>
                    let #arg_pat = ::ocaml_interop::internal::float_val(#raw_val_ident);
                });
            } else {
                arg_setup_code.push(quote_spanned! {arg_pat.span()=>
                    let #arg_pat = #raw_val_ident;
                });
            }
            call_args_for_native_fn.push(quote_spanned! {arg_pat.span()=> #arg_pat });
        }

        let bytecode_fn_def = quote! {
            #[no_mangle]
            pub extern "C" fn #bytecode_fn_name(argv: *mut ::ocaml_interop::RawOCaml, argn: ::std::os::raw::c_int) #extern_fn_return_type_sig {
                #(#arg_setup_code)*

                if cfg!(debug_assertions) {
                    if (argn as usize) != #arg_count {
                        panic!(
                            "Bytecode function '{}' called with incorrect number of arguments: expected {}, got {}.",
                            stringify!(#bytecode_fn_name),
                            #arg_count,
                            argn
                        );
                    }
                }
                // Call the native FFI function
                #native_fn_name(#(#call_args_for_native_fn),*)
            }
        };
        all_generated_code.push(bytecode_fn_def);
    }

    let expanded = quote! {
        #(#all_generated_code)*
    };

    Ok(expanded)
}

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ts2 = proc_macro2::TokenStream::from(attr);
    let item_ts2 = proc_macro2::TokenStream::from(item);

    match export_internal_logic(attr_ts2, item_ts2) {
        Ok(generated_code) => TokenStream::from(generated_code),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

#[cfg(test)]
mod tests;
