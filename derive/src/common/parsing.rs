use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    spanned::Spanned, Data, DeriveInput, Expr, ExprLit, Fields, FnArg, Generics, Ident, Lit,
    PathArguments, ReturnType, Type,
};

use crate::{
    common::{
        attr_parsing::OCamlAttributes,
        error::{OCamlInteropError, Result},
    },
    export::core::{InteropTypeDetail, PrimitiveInteropType, ProcessedArg},
};

/// Represents the kind of variant in an enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantKind {
    /// A unit variant without any fields (e.g., `MyEnum::Unit`)
    Unit,

    /// A tuple variant with unnamed fields (e.g., `MyEnum::Tuple(i32, String)`)
    Tuple,

    /// A struct variant with named fields (e.g., `MyEnum::Struct { field: i32 }`)
    Struct,
}

/// Represents the kind of enum for OCaml interop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumKind {
    /// A regular OCaml enum, represented as tagged integers or blocks
    Regular,

    /// A polymorphic variant, represented as hash values
    Polymorphic,
}

pub struct FieldRep {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub attrs: OCamlAttributes,
    pub ocaml_as_type_override_ts: Option<TokenStream>,
    pub span: Span,
}

pub struct VariantRep {
    pub ident: Ident,
    pub fields: Vec<FieldRep>,
    pub attrs: OCamlAttributes,
    pub kind: VariantKind,
    pub span: Span,
}

pub enum TypeRepData {
    Struct {
        fields: Vec<FieldRep>,
    },
    Enum {
        variants: Vec<VariantRep>,
        kind: EnumKind,
    },
}

pub struct TypeRep {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: OCamlAttributes,
    pub data: TypeRepData,
    pub ocaml_target_type_ident_ts: TokenStream,
}

// --- Parse Stage ---

pub fn parse_field(field: &syn::Field) -> Result<FieldRep> {
    let attrs = OCamlAttributes::from_attrs(&field.attrs)?;

    let ocaml_as_type_override_ts = if let Some(as_val_str) = &attrs.as_ {
        as_val_str.parse::<TokenStream>().map(Some).map_err(|e| {
            OCamlInteropError::type_error_spanned(
                format!("Failed to parse OCaml type override '{as_val_str}': {e}"),
                field,
            )
        })?
    } else {
        None
    };

    Ok(FieldRep {
        ident: field.ident.clone(),
        ty: field.ty.clone(),
        attrs,
        ocaml_as_type_override_ts,
        span: field.span(),
    })
}

fn parse_fields_common(fields: &syn::Fields) -> Result<Vec<FieldRep>> {
    match fields {
        Fields::Named(named_fields) => named_fields
            .named
            .iter()
            .map(parse_field)
            .collect::<Result<Vec<_>>>(),
        Fields::Unnamed(unnamed_fields) => unnamed_fields
            .unnamed
            .iter()
            .map(parse_field)
            .collect::<Result<Vec<_>>>(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn parse_enum_variant(variant: &syn::Variant) -> Result<VariantRep> {
    let variant_attrs = OCamlAttributes::from_attrs(&variant.attrs)?;

    let fields_rep = parse_fields_common(&variant.fields)?;

    let variant_kind = match &variant.fields {
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                VariantKind::Unit
            } else {
                VariantKind::Struct
            }
        }
        Fields::Unnamed(fields) => {
            if fields.unnamed.is_empty() {
                VariantKind::Unit
            } else {
                VariantKind::Tuple
            }
        }
        Fields::Unit => VariantKind::Unit,
    };

    Ok(VariantRep {
        ident: variant.ident.clone(),
        fields: fields_rep,
        attrs: variant_attrs,
        kind: variant_kind,
        span: variant.span(),
    })
}

fn parse_struct_data(data_struct: &syn::DataStruct) -> Result<TypeRepData> {
    let fields_rep = parse_fields_common(&data_struct.fields)?;
    Ok(TypeRepData::Struct { fields: fields_rep })
}

fn parse_enum_data(data_enum: &syn::DataEnum, type_attrs: &OCamlAttributes) -> Result<TypeRepData> {
    let enum_kind = if type_attrs.is_polymorphic_variant() {
        EnumKind::Polymorphic
    } else {
        EnumKind::Regular
    };

    let variants_rep = data_enum
        .variants
        .iter()
        .map(parse_enum_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(TypeRepData::Enum {
        variants: variants_rep,
        kind: enum_kind,
    })
}

pub fn parse_input(derive_input: DeriveInput) -> Result<TypeRep> {
    let type_ident = derive_input.ident.clone();
    let type_attrs = OCamlAttributes::from_attrs(&derive_input.attrs)?;

    let ocaml_target_type_ident_ts = match &type_attrs.as_ {
        Some(as_val_str) => as_val_str.parse::<TokenStream>().map_err(|e| {
            OCamlInteropError::attribute_error_spanned(
                Some("as_"),
                format!("Failed to parse type string '{as_val_str}' from attribute #[ocaml(as_ = \"...\")]: {e}"),
                &derive_input,
            )
        })?,
        None => {
            // Default to the same name as the Rust type
            quote! { #type_ident }
        }
    };

    let type_rep_data = match &derive_input.data {
        Data::Struct(data_struct) => parse_struct_data(data_struct)?,
        Data::Enum(data_enum) => parse_enum_data(data_enum, &type_attrs)?,
        Data::Union(data_union) => {
            return Err(OCamlInteropError::type_error_spanned(
                "Unions are not supported by OCaml interop derive macros",
                &data_union.union_token,
            ));
        }
    };

    Ok(TypeRep {
        ident: type_ident,
        generics: derive_input.generics,
        attrs: type_attrs,
        data: type_rep_data,
        ocaml_target_type_ident_ts,
    })
}

// --- Shared Type Parsing Utilities ---

/// Maps a Rust primitive type to the corresponding OCaml interop primitive type
pub fn get_primitive_type(
    type_name: &str,
    user_type: &syn::Type,
) -> std::result::Result<PrimitiveInteropType, syn::Error> {
    use PrimitiveInteropType::*;
    match type_name {
        "f64" => Ok(F64),
        "i64" => Ok(I64),
        "i32" => Ok(I32),
        "bool" => Ok(Bool),
        "isize" => Ok(ISize),
        _ => Err(syn::Error::new_spanned(
            user_type,
            format!("Unsupported primitive type: `{type_name}`"),
        )),
    }
}

/// Extracts the inner type from a generic type like OCaml<T> or BoxRoot<T>
pub fn extract_inner_type_from_path(
    type_path: &syn::TypePath,
    expected_wrapper_ident: &str,
) -> std::result::Result<Box<syn::Type>, syn::Error> {
    let last_segment = type_path.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(
            type_path,
            format!("Type path for {expected_wrapper_ident} must have segments"),
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
        PathArguments::AngleBracketed(params) => {
            if let Some(syn::GenericArgument::Type(inner_ty)) = params.args.first() {
                Ok(Box::new(inner_ty.clone()))
            } else {
                Err(syn::Error::new_spanned(
                    &last_segment.arguments,
                    format!("{expected_wrapper_ident}<T> type argument T is missing or not a type"),
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            &last_segment.arguments,
            format!("{expected_wrapper_ident}<T> type argument T must be angle bracketed"),
        )),
    }
}

/// Processes a path type (like primitives, OCaml<T>, BoxRoot<T>)
pub fn process_path_type(
    type_path: &syn::TypePath,
    user_type: &syn::Type,
) -> std::result::Result<InteropTypeDetail, syn::Error> {
    if type_path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            type_path,
            "Qualified self types are not supported.",
        ));
    }

    let last_segment = type_path.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(type_path, "Invalid type: Type path has no segments.")
    })?;

    let ident_str = last_segment.ident.to_string();

    // Handle primitive types
    if matches!(ident_str.as_str(), "f64" | "i64" | "i32" | "bool" | "isize") {
        let primitive_type = get_primitive_type(&ident_str, user_type)?;
        return Ok(InteropTypeDetail::Primitive {
            primitive_type,
            original_rust_type: Box::new(user_type.clone()),
        });
    }

    // Handle wrapper types
    match ident_str.as_str() {
        "OCaml" => {
            let inner_type = extract_inner_type_from_path(type_path, "OCaml")?;
            Ok(InteropTypeDetail::OCaml {
                wrapper_type: Box::new(user_type.clone()),
                inner_type,
            })
        }
        "BoxRoot" => {
            let inner_type = extract_inner_type_from_path(type_path, "BoxRoot")?;
            Ok(InteropTypeDetail::BoxRoot { inner_type })
        }
        _ => Err(syn::Error::new_spanned(
            user_type,
            format!(
                "Unsupported type identifier: `{ident_str}`. Must be a primitive, OCaml<T>, or BoxRoot<T> (for arguments).",
            ),
        )),
    }
}

/// Determines the interoperation type details for a given Rust type
pub fn get_interop_type_detail(
    user_type: &syn::Type,
) -> std::result::Result<InteropTypeDetail, syn::Error> {
    match user_type {
        // Handle unit type - ()
        Type::Tuple(type_tuple) if type_tuple.elems.is_empty() => {
            Ok(InteropTypeDetail::Unit)
        }

        // Handle other tuple types - (A, B, ...)
        Type::Tuple(type_tuple) => {
            Err(syn::Error::new_spanned(
                type_tuple,
                "Tuple types are not directly supported. For multiple return values, consider returning a struct wrapped in OCaml<T>.",
            ))
        }

        // Handle path types - primitives, OCaml<T>, BoxRoot<T>
        Type::Path(type_path) => {
            process_path_type(type_path, user_type)
        }

        // Handle reference types
        Type::Reference(type_ref) => Err(syn::Error::new_spanned(
            type_ref,
            "Reference types are not directly supported. Consider using OCaml<T> or BoxRoot<T>.",
        )),

        // Handle all other types
        _ => Err(syn::Error::new_spanned(
            user_type,
            format!(
                "Unsupported type structure: `{}`. Must be a primitive, OCaml<T>, BoxRoot<T> (for arguments), or `()` (for return types).",
                quote!{#user_type}
            ),
        )),
    }
}

/// Processes a single user-provided (non-runtime) function argument
pub fn process_extern_argument(arg_input: &FnArg) -> std::result::Result<ProcessedArg, syn::Error> {
    if let FnArg::Typed(pat_type) = arg_input {
        let original_pat = &pat_type.pat;
        let original_ty = &pat_type.ty;

        let interop_detail = get_interop_type_detail(original_ty)?;

        Ok(ProcessedArg {
            pattern: original_pat.clone(),
            type_detail: interop_detail,
            original_rust_type: (*original_ty).clone(),
        })
    } else {
        Err(syn::Error::new_spanned(
            arg_input,
            "Receiver arguments (`self`) are not supported in ocaml_interop::export functions.",
        ))
    }
}

/// Processes the return type of the user's function
pub fn process_return_type(
    original_fn_return_type_ast: &ReturnType,
) -> std::result::Result<(InteropTypeDetail, syn::Type), syn::Error> {
    let user_return_type_ast: syn::Type = match original_fn_return_type_ast {
        ReturnType::Default => syn::parse_quote! { () },
        ReturnType::Type(_, ty_box) => (**ty_box).clone(),
    };

    let interop_detail = get_interop_type_detail(&user_return_type_ast)?;
    Ok((interop_detail, user_return_type_ast))
}

/// Generic helper function to check for duplicate attributes
pub fn check_duplicate_attr(
    attr_name: &str,
    meta_path: &syn::Path,
    span: &Option<proc_macro2::Span>,
) -> std::result::Result<(), syn::Error> {
    if let Some(prev_span) = span {
        let mut err = syn::Error::new_spanned(
            meta_path,
            format!("'{attr_name}' attribute specified multiple times"),
        );
        err.combine(syn::Error::new(
            *prev_span,
            format!("previous '{attr_name}' attribute here"),
        ));
        return Err(err);
    }
    Ok(())
}

/// Generic helper function to process a flag attribute (bare path like #[attr(flag)])
/// Returns (flag_value, span) for tracking duplicates
pub fn process_flag_attribute(
    meta: &syn::Meta,
    attr_name: &str,
) -> std::result::Result<(bool, Option<proc_macro2::Span>), syn::Error> {
    match meta {
        syn::Meta::Path(path) => Ok((true, Some(path.span()))),
        _ => Err(syn::Error::new_spanned(
            meta,
            format!("'{attr_name}' attribute should be a bare path (e.g., #[attr({attr_name})]), not a name-value or list")
        )),
    }
}

/// Generic helper function to parse a string literal from a name-value attribute
/// Returns the string value from expressions like attr = "value"
pub fn parse_string_literal_attribute(
    meta_name_value: &syn::MetaNameValue,
    attr_name: &str,
) -> std::result::Result<String, syn::Error> {
    match &meta_name_value.value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) => Ok(lit_str.value()),
        _ => Err(syn::Error::new_spanned(
            &meta_name_value.value,
            format!(
                "'{attr_name}' attribute value must be a string literal (e.g., {attr_name} = \"value\")"
            ),
        )),
    }
}
