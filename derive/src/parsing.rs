use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Lit, Meta, Token};
use syn::{FnArg, ItemFn, PathArguments, ReturnType, Type};

use crate::core::ExportedFnData;
use crate::core::InteropTypeDetail;
use crate::core::PrimitiveInteropType;
use crate::core::ProcessedArg;

pub(crate) fn get_interop_type_detail(
    user_type: &syn::Type,
) -> Result<InteropTypeDetail, syn::Error> {
    match user_type {
        Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                Ok(InteropTypeDetail::Unit)
            } else {
                Err(syn::Error::new_spanned(
                    type_tuple,
                    "Tuple types are not directly supported. For multiple return values, consider returning a struct wrapped in OCaml<T>.",
                ))
            }
        }
        Type::Path(type_path) => {
            if type_path.qself.is_some() {
                return Err(syn::Error::new_spanned(
                    type_path,
                    "Qualified self types are not supported.",
                ));
            }
            if let Some(last_segment) = type_path.path.segments.last() {
                let ident_str = last_segment.ident.to_string();
                match ident_str.as_str() {
                    "f64" => Ok(InteropTypeDetail::Primitive {
                        primitive_type: PrimitiveInteropType::F64,
                        original_rust_type: Box::new(user_type.clone()),
                    }),
                    "i64" => Ok(InteropTypeDetail::Primitive {
                        primitive_type: PrimitiveInteropType::I64,
                        original_rust_type: Box::new(user_type.clone()),
                    }),
                    "i32" => Ok(InteropTypeDetail::Primitive {
                        primitive_type: PrimitiveInteropType::I32,
                        original_rust_type: Box::new(user_type.clone()),
                    }),
                    "bool" => Ok(InteropTypeDetail::Primitive {
                        primitive_type: PrimitiveInteropType::Bool,
                        original_rust_type: Box::new(user_type.clone()),
                    }),
                    "isize" => Ok(InteropTypeDetail::Primitive {
                        primitive_type: PrimitiveInteropType::ISize,
                        original_rust_type: Box::new(user_type.clone()),
                    }),
                    "OCaml" => {
                        let inner_type = extract_inner_type_from_path(type_path, "OCaml")?;
                        Ok(InteropTypeDetail::OCaml {
                            wrapper_type: Box::new(user_type.clone()),
                            inner_type,
                        })
                    }
                    "BoxRoot" => {
                        let inner_type = extract_inner_type_from_path(type_path, "BoxRoot")?;
                        Ok(InteropTypeDetail::BoxRoot {
                            inner_type,
                        })
                    }
                    _ => Err(syn::Error::new_spanned(
                        user_type,
                        format!(
                            "Unsupported type identifier: `{ident_str}`. Must be a primitive, OCaml<T>, or BoxRoot<T> (for arguments).",
                        ),
                    )),
                }
            } else {
                Err(syn::Error::new_spanned(
                    type_path,
                    "Invalid type: Type path has no segments.",
                ))
            }
        }
        Type::Reference(type_ref) => Err(syn::Error::new_spanned(
            type_ref,
            "Reference types are not directly supported. Consider using OCaml<T> or BoxRoot<T>.",
        )),
        _ => Err(syn::Error::new_spanned(
            user_type,
            format!(
                "Unsupported type structure: `{}`. Must be a primitive, OCaml<T>, BoxRoot<T> (for arguments), or `()` (for return types).",
                quote!{#user_type}
            ),
        )),
    }
}

// Extracts the inner type from a generic type like OCaml<T> or BoxRoot<T>.
pub(crate) fn extract_inner_type_from_path(
    type_path: &syn::TypePath,
    expected_wrapper_ident: &str,
) -> Result<Box<syn::Type>, syn::Error> {
    let last_segment = type_path.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(
            type_path,
            format!("Type path for {expected_wrapper_ident} must have segments",),
        )
    })?;

    if last_segment.ident != expected_wrapper_ident {
        return Err(syn::Error::new_spanned(
            type_path,
            format!(
                "Expected type wrapper {expected_wrapper_ident}, found {}",
                last_segment.ident
            ),
        ));
    }

    match &last_segment.arguments {
        PathArguments::AngleBracketed(params) => match params.args.first() {
            Some(syn::GenericArgument::Type(inner_ty)) => Ok(Box::new(inner_ty.clone())),
            _ => Err(syn::Error::new_spanned(
                &last_segment.arguments,
                format!("{expected_wrapper_ident}<T> type argument T is missing or not a type",),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            &last_segment.arguments,
            format!("{expected_wrapper_ident}<T> type argument T must be angle bracketed",),
        )),
    }
}

// Parses the first function argument, ensuring it's a syntactically valid OCamlRuntime reference.
// Type and mutability validation against `noalloc` is handled in `validation.rs`.
pub(crate) fn parse_runtime_argument(
    first_arg: Option<&FnArg>,
    fn_inputs_span: proc_macro2::Span,
) -> Result<(syn::PatType, Box<Type>), syn::Error> {
    // Return Box<Type> for consistency
    let fn_arg = first_arg.ok_or_else(|| {
        syn::Error::new(
            fn_inputs_span,
            "Exported functions must take an OCamlRuntime as their first argument.",
        )
    })?;

    let pat_type = match fn_arg {
        FnArg::Typed(pt) => pt.clone(),
        FnArg::Receiver(rec) => {
            return Err(syn::Error::new_spanned(
                rec,
                "OCamlRuntime argument cannot be a receiver (self).",
            ));
        }
    };

    let arg_type_box = pat_type.ty.clone();
    Ok((pat_type, arg_type_box))
}

// Processes a single user-provided (non-runtime) function argument.
pub(crate) fn process_extern_argument(arg_input: &FnArg) -> Result<ProcessedArg, syn::Error> {
    if let FnArg::Typed(pat_type) = arg_input {
        let original_pat = &pat_type.pat;
        let original_ty = &pat_type.ty;

        let interop_detail = get_interop_type_detail(original_ty)?;

        // Match all variants to construct ProcessedArg. Validation for Unit is in validation.rs
        Ok(ProcessedArg {
            pattern: original_pat.clone(),
            type_detail: interop_detail.clone(), // interop_detail can now be Unit
            original_rust_type: Box::new(*original_ty.clone()),
        })
    } else {
        Err(syn::Error::new_spanned(
            arg_input,
            "Receiver arguments (`self`) are not supported in ocaml_interop::export functions.",
        ))
    }
}

// Processes the return type of the user's function.
pub(crate) fn process_return_type(
    original_fn_return_type_ast: &ReturnType,
) -> Result<(InteropTypeDetail, syn::Type), syn::Error> {
    let user_return_type_ast: syn::Type = match original_fn_return_type_ast {
        ReturnType::Default => syn::parse_quote! { () },
        ReturnType::Type(_, ty_box) => (**ty_box).clone(),
    };

    let interop_detail_matched = get_interop_type_detail(&user_return_type_ast)?;

    Ok((interop_detail_matched, user_return_type_ast))
}

pub(crate) fn parse_export_definition(
    attr_ts: proc_macro2::TokenStream,
    input_fn: &ItemFn,
) -> Result<ExportedFnData, syn::Error> {
    let mut bytecode_fn_name_opt: Option<syn::Ident> = None;
    let mut bytecode_meta_span: Option<proc_macro2::Span> = None;
    let mut no_panic_catch = false;
    let mut no_panic_catch_span: Option<proc_macro2::Span> = None;
    let mut noalloc = false;
    let mut noalloc_span: Option<proc_macro2::Span> = None;

    let attribute_parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let parsed_attributes = attribute_parser.parse2(attr_ts)?;

    for meta in parsed_attributes {
        if meta.path().is_ident("bytecode") {
            if bytecode_fn_name_opt.is_some() {
                let mut err = syn::Error::new_spanned(
                    meta.path(),
                    "'bytecode' attribute specified multiple times",
                );
                if let Some(prev_span) = bytecode_meta_span {
                    err.combine(syn::Error::new(
                        prev_span,
                        "previous 'bytecode' attribute here",
                    ));
                }
                return Err(err);
            }
            match meta {
                syn::Meta::NameValue(mnv) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = mnv.value
                    {
                        bytecode_fn_name_opt =
                            Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
                        bytecode_meta_span = Some(mnv.path.span());
                    } else {
                        return Err(syn::Error::new_spanned(mnv.value, "'bytecode' attribute value must be a string literal (e.g., bytecode = \"my_func_byte\")"));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(meta, "'bytecode' attribute must be a name-value pair (e.g., bytecode = \"my_func_byte\")"));
                }
            }
        } else if meta.path().is_ident("no_panic_catch") {
            if no_panic_catch_span.is_some() {
                let mut err = syn::Error::new_spanned(
                    meta.path(),
                    "'no_panic_catch' attribute specified multiple times",
                );
                if let Some(prev_span) = no_panic_catch_span {
                    err.combine(syn::Error::new(
                        prev_span,
                        "previous 'no_panic_catch' attribute here",
                    ));
                }
                return Err(err);
            }
            match meta {
                syn::Meta::Path(path) => {
                    no_panic_catch = true;
                    no_panic_catch_span = Some(path.span());
                }
                _ => {
                    return Err(syn::Error::new_spanned(meta, "'no_panic_catch' attribute should be a bare path (e.g., #[export(no_panic_catch)]), not a name-value or list"));
                }
            }
        } else if meta.path().is_ident("noalloc") {
            if noalloc_span.is_some() {
                let mut err = syn::Error::new_spanned(
                    meta.path(),
                    "'noalloc' attribute specified multiple times",
                );
                if let Some(prev_span) = noalloc_span {
                    err.combine(syn::Error::new(
                        prev_span,
                        "previous 'noalloc' attribute here",
                    ));
                }
                return Err(err);
            }
            match meta {
                syn::Meta::Path(path) => {
                    noalloc = true;
                    noalloc_span = Some(path.span());
                }
                _ => {
                    return Err(syn::Error::new_spanned(meta, "'noalloc' attribute should be a bare path (e.g., #[export(noalloc)]), not a name-value or list"));
                }
            }
        } else {
            return Err(syn::Error::new_spanned(meta.path(), format!(
                "unsupported attribute '{}', only 'bytecode', 'no_panic_catch' and 'noalloc' are supported",
                meta.path().get_ident().map_or_else(|| String::from("?"), |i| i.to_string())
            )));
        }
    }

    let original_fn_ident = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_body = &input_fn.block;
    let original_fn_return_type_ast = &input_fn.sig.output;
    let fn_token_span = input_fn.sig.fn_token.span();
    let fn_inputs_span = fn_inputs.span();

    let native_fn_name = original_fn_ident.clone();
    let visibility = input_fn.vis.clone();
    let original_fn_ident_span = original_fn_ident.span();
    let is_async = input_fn.sig.asyncness.is_some();
    let is_unsafe = input_fn.sig.unsafety.is_some();
    let has_generics = !input_fn.sig.generics.params.is_empty();
    let is_variadic = input_fn.sig.variadic.is_some();
    let abi = input_fn.sig.abi.clone();

    let mut original_fn_args_iter = fn_inputs.iter();

    let (runtime_arg_pat_ref, runtime_arg_ty_ref) =
        parse_runtime_argument(original_fn_args_iter.next(), fn_inputs_span)?;
    let runtime_arg_pat = Box::new(runtime_arg_pat_ref.pat.as_ref().clone());
    let runtime_arg_ty = runtime_arg_ty_ref; // Already a Box<Type>

    let mut processed_args: Vec<ProcessedArg> = Vec::new();
    for arg in original_fn_args_iter {
        match process_extern_argument(arg) {
            Ok(p_arg) => processed_args.push(p_arg),
            Err(e) => return Err(e),
        }
    }

    let (return_interop_detail, user_return_type_ast) =
        process_return_type(original_fn_return_type_ast)?;

    Ok(ExportedFnData {
        bytecode_fn_name_opt,
        _bytecode_meta_span: bytecode_meta_span,
        no_panic_catch,
        _no_panic_catch_span: no_panic_catch_span,
        noalloc,
        _noalloc_span: noalloc_span,
        native_fn_name,
        visibility,
        original_fn_block: fn_body.clone(),
        runtime_arg_pat,
        runtime_arg_ty,
        processed_args,
        return_interop_detail,
        user_return_type_ast,
        _original_fn_ident_span: original_fn_ident_span,
        _fn_token_span: fn_token_span,
        _fn_inputs_span: fn_inputs_span,
        is_async,
        _is_unsafe: is_unsafe,
        has_generics,
        is_variadic,
        abi,
    })
}
