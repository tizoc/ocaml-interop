use quote::quote;
use syn::{Error, Type};

use crate::core::{ExportedFnData, InteropTypeDetail};

// Performs validation checks on the fully parsed ExportedFnData.
pub fn validate_parsed_data(data: &ExportedFnData) -> Result<(), Error> {
    // Validate the first argument is a syntactically correct OCamlRuntime reference
    match &*data.runtime_arg_ty {
        Type::Reference(type_ref) => {
            // Check if the referenced type is OCamlRuntime
            if let Type::Path(type_path) = &*type_ref.elem {
                let path_str = quote!(#type_path).to_string();
                if !(path_str == "OCamlRuntime"
                    || path_str == "ocaml_interop :: OCamlRuntime"
                    || path_str == ":: ocaml_interop :: OCamlRuntime")
                {
                    return Err(Error::new_spanned(
                        &data.runtime_arg_ty,
                        "Exported functions must take an OCamlRuntime reference (e.g., `rt: &OCamlRuntime` or `rt: &mut OCamlRuntime`) as their first argument.",
                    ));
                }
            } else {
                // Referenced type is not a path, so not OCamlRuntime
                return Err(Error::new_spanned(
                    &data.runtime_arg_ty,
                    "Exported functions must take an OCamlRuntime reference (e.g., `rt: &OCamlRuntime` or `rt: &mut OCamlRuntime`) as their first argument.",
                ));
            }

            // Validate OCamlRuntime argument mutability based on noalloc
            let is_mutable_runtime = type_ref.mutability.is_some();
            if data.noalloc {
                if is_mutable_runtime {
                    return Err(Error::new_spanned(
                        &data.runtime_arg_ty,
                        "When `noalloc` is used, OCaml runtime argument must be an immutable reference (e.g., &OCamlRuntime)",
                    ));
                }
            } else {
                // Default case (allocations allowed, noalloc is false)
                if !is_mutable_runtime {
                    return Err(Error::new_spanned(
                        &data.runtime_arg_ty,
                        "OCaml runtime argument must be a mutable reference (e.g., &mut OCamlRuntime). Use `#[export(noalloc)]` for an immutable reference.",
                    ));
                }
            }
        }
        _ => {
            // Not a reference type at all
            return Err(Error::new_spanned(
                &data.runtime_arg_ty,
                "Exported functions must take an OCamlRuntime reference (e.g., `rt: &OCamlRuntime` or `rt: &mut OCamlRuntime`) as their first argument.",
            ));
        }
    }

    // Validate that there is at least one non-runtime argument
    if data.processed_args.is_empty() {
        return Err(syn::Error::new(
            data._original_fn_ident_span, // Using the function name's span for this error
            "OCaml functions must take at least one argument in addition to the OCamlRuntime.",
        ));
    }

    // Validate that BoxRoot<T> is not used as a return type
    if let InteropTypeDetail::BoxRoot { .. } = &data.return_interop_detail {
        return Err(Error::new_spanned(
            &data.user_return_type_ast, // Span of the user's return type
            "BoxRoot<T> cannot be used as a return type directly. Return OCaml<T> instead.",
        ));
    }

    // Validate that Unit type `()` is not used as an argument type
    for arg in &data.processed_args {
        if let InteropTypeDetail::Unit = &arg.type_detail {
            return Err(Error::new_spanned(
                &arg.original_rust_type,
                "Unit type `()` is not a supported argument type directly. Use OCaml<()> if needed for placeholder.",
            ));
        }
    }

    if data.noalloc {
        // Disallow BoxRoot<T> arguments when noalloc is used.
        for arg in &data.processed_args {
            if let InteropTypeDetail::BoxRoot { .. } = &arg.type_detail {
                return Err(Error::new_spanned(
                    &arg.original_rust_type,
                    "`BoxRoot<T>` arguments are not allowed when `noalloc` is used because BoxRoot implies allocation for rooting.",
                ));
            }
        }
    }

    // Validate function visibility (must be public)
    if !matches!(data.visibility, syn::Visibility::Public(_)) {
        return Err(Error::new_spanned(
            &data.native_fn_name, // Span of the function name
            "Exported functions must be public (`pub`).",
        ));
    }

    // Validate function is not async
    if data.is_async {
        return Err(Error::new_spanned(
            &data.native_fn_name,
            "Exported functions cannot be `async`.",
        ));
    }

    // Validate function is not generic
    if data.has_generics {
        return Err(Error::new_spanned(
            &data.native_fn_name,
            "Exported functions cannot have generic parameters.",
        ));
    }

    // Validate function is not variadic
    if data.is_variadic {
        return Err(Error::new_spanned(
            &data.native_fn_name,
            "Exported functions cannot be variadic (e.g., `...`).",
        ));
    }

    // Validate ABI is not specified or is "C" or "C-unwind"
    if let Some(abi) = &data.abi {
        if let Some(name) = &abi.name {
            let abi_name = name.value();
            if abi_name != "C" && abi_name != "C-unwind" {
                return Err(Error::new_spanned(
                    abi,
                    "Exported functions must use `extern \"C\"` or `extern \"C-unwind\"` ABI. Other ABIs are not supported.",
                ));
            }
        }
    }

    Ok(())
}
