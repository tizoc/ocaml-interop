// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use proc_macro2::Span;

use crate::common::parsing::{EnumKind, FieldRep, TypeRep, TypeRepData, VariantRep};
use crate::common::{OCamlInteropError, Result};

/// Creates an error for when an attribute is used in the wrong context
pub fn invalid_attribute_use(
    attribute_name: impl Into<String>,
    context: impl Into<String>,
    expected_context: impl Into<String>,
    span: Span,
) -> OCamlInteropError {
    let attr_name = attribute_name.into();
    OCamlInteropError::validation_error(
        format!(
            "#[ocaml({attr_name})] is only applicable on {}, but was used on {}.",
            expected_context.into(),
            context.into()
        ),
        span,
        Some(format!("attribute validation: {attr_name}")),
    )
}

/// Validates that the `#[ocaml(polymorphic_variant)]` attribute is only used on enums
pub fn validate_polymorphic_kind(type_rep: &TypeRep) -> Result<()> {
    if type_rep.attrs.is_polymorphic_variant()
        && !matches!(&type_rep.data, TypeRepData::Enum { .. })
    {
        return Err(invalid_attribute_use(
            "polymorphic_variant".to_string(),
            "non-enum type",
            "enum types only",
            type_rep.ident.span(),
        ));
    }
    Ok(())
}

/// Validates that the `#[ocaml(tag="...")]` attribute is only used on polymorphic enum variants
pub fn validate_tag_attribute(variant_rep: &VariantRep, enum_kind: EnumKind) -> Result<()> {
    if let Some(tag) = &variant_rep.attrs.tag {
        if enum_kind != EnumKind::Polymorphic {
            return Err(invalid_attribute_use(
                format!("tag=\"{tag}\""),
                format!("variant '{}' in a regular enum", variant_rep.ident),
                "variants of a polymorphic enum (i.e. enum has #[ocaml(polymorphic_variant)])",
                variant_rep.span,
            ));
        }
    }
    Ok(())
}

/// Validates field attributes
pub fn validate_field_attributes(field_rep: &FieldRep) -> Result<()> {
    // Check that as_ attribute was successfully parsed if present
    if field_rep.ocaml_as_type_override_ts.is_none() && field_rep.attrs.as_.is_some() {
        // This implies `as_` was present but failed to parse to TokenStream.
        // Error should have been generated during parse_field already.
    }

    // Additional field validation could be added here

    Ok(())
}

/// Validates that enums have at least one variant
pub fn validate_enum_has_variants(type_rep: &TypeRep) -> Result<()> {
    if let TypeRepData::Enum { variants, .. } = &type_rep.data {
        if variants.is_empty() {
            return Err(OCamlInteropError::validation_error(
                "Empty enums are not supported in OCaml interop".to_string(),
                type_rep.ident.span(),
                Some("enum must have at least one variant".to_string()),
            ));
        }
    }
    Ok(())
}

/// General validation for a type representation
pub fn validate_type_rep(type_rep: &TypeRep) -> Result<()> {
    // 1. Validate container-level attributes
    validate_polymorphic_kind(type_rep)?;

    // 2. Validate that enums have variants
    validate_enum_has_variants(type_rep)?;

    // 3. Validate type-specific elements
    match &type_rep.data {
        TypeRepData::Enum { variants, kind, .. } => {
            for variant in variants {
                validate_tag_attribute(variant, *kind)?;

                for field in &variant.fields {
                    validate_field_attributes(field)?;
                }
            }
        }
        TypeRepData::Struct { fields } => {
            for field in fields {
                validate_field_attributes(field)?;
            }
        }
    }

    Ok(())
}
