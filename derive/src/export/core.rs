// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use quote::quote;
use syn::Pat;
use syn::Type;

// Holds all parsed data from the macro input, used to feed the expansion phase.
pub(crate) struct ExportedFnData {
    // Attributes
    pub(crate) bytecode_fn_name_opt: Option<syn::Ident>,

    pub(crate) no_panic_catch: bool,
    pub(crate) noalloc: bool,

    // Original function elements
    pub(crate) native_fn_name: syn::Ident, // Name of the original Rust function, used for the extern "C" native fn
    pub(crate) visibility: syn::Visibility, // Original visibility, for bytecode stub
    pub(crate) original_fn_block: Box<syn::Block>, // The original function's body
    pub(crate) is_async: bool,
    pub(crate) has_generics: bool,
    pub(crate) is_variadic: bool,
    pub(crate) abi: Option<syn::Abi>,

    // Runtime argument details
    pub(crate) runtime_arg_pat: Box<Pat>,
    pub(crate) runtime_arg_ty: Box<Type>,

    // Processed user-provided arguments (details for C signature, Rust initialization, type info)
    pub(crate) processed_args: Vec<ProcessedArg>, // User args, excluding runtime

    // Return type processing results
    pub(crate) return_interop_detail: InteropTypeDetail, // InteropTypeDetail of the user's function return type
    pub(crate) user_return_type_ast: syn::Type, // The syn::Type of the original function's return

    // Spans for error reporting
    pub(crate) original_fn_ident_span: proc_macro2::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimitiveInteropType {
    F64,
    I64,
    I32,
    Bool,
    ISize,
}

#[derive(Clone)]
pub(crate) enum InteropTypeDetail {
    Unit, // Represents Rust's () type, primarily for return types
    Primitive {
        primitive_type: PrimitiveInteropType,
        original_rust_type: Box<syn::Type>, // e.g., syn::Type for "f64"
    },
    OCaml {
        wrapper_type: Box<syn::Type>, // e.g., syn::Type for "OCaml<String>"
        inner_type: Box<syn::Type>,   // e.g., syn::Type for "String"
    },
    BoxRoot {
        // Only for arguments
        inner_type: Box<syn::Type>, // e.g., syn::Type for "MyStruct"
    },
}

impl InteropTypeDetail {
    pub(crate) fn is_primitive(&self) -> bool {
        matches!(self, InteropTypeDetail::Primitive { .. })
    }

    pub(crate) fn get_ocaml_to_rust_fn_name(&self) -> Result<String, syn::Error> {
        match self {
            InteropTypeDetail::Primitive { primitive_type, .. } => Ok(match primitive_type {
                PrimitiveInteropType::F64 => "float_val".to_string(),
                PrimitiveInteropType::I64 => "int64_val".to_string(),
                PrimitiveInteropType::I32 => "int32_val".to_string(),
                PrimitiveInteropType::Bool => "bool_val".to_string(),
                PrimitiveInteropType::ISize => "int_val".to_string(),
            }),
            _ => Err(syn::Error::new(
                // This function should ideally only be called on primitives.
                // A more specific span could be used if available from the caller context.
                proc_macro2::Span::call_site(),
                "Internal error: get_ocaml_to_rust_fn_name called on non-primitive type",
            )),
        }
    }

    pub(crate) fn get_rust_to_ocaml_fn_name(&self) -> Result<String, syn::Error> {
        match self {
            InteropTypeDetail::Primitive { primitive_type, .. } => Ok(match primitive_type {
                PrimitiveInteropType::F64 => "alloc_float".to_string(),
                PrimitiveInteropType::I64 => "alloc_int64".to_string(),
                PrimitiveInteropType::I32 => "alloc_int32".to_string(),
                PrimitiveInteropType::Bool => "make_ocaml_bool".to_string(),
                PrimitiveInteropType::ISize => "make_ocaml_int".to_string(),
            }),
            _ => Err(syn::Error::new(
                // This function should ideally only be called on primitives.
                // A more specific span could be used if available from the caller context.
                proc_macro2::Span::call_site(),
                "Internal error: get_rust_to_ocaml_fn_name called on non-primitive type",
            )),
        }
    }

    pub(crate) fn get_conversion_module_path_tokens(&self) -> proc_macro2::TokenStream {
        // Path for primitive type conversion functions.
        quote! { ::ocaml_interop::internal:: }
    }
}

// Holds processed information for a single user-provided function argument.
pub(crate) struct ProcessedArg {
    pub(crate) pattern: Box<Pat>,
    pub(crate) type_detail: InteropTypeDetail,
    pub(crate) original_rust_type: Box<syn::Type>,
}
