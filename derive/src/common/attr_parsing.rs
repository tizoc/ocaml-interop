// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::common::{OCamlInteropError, Result};
use syn::{punctuated::Punctuated, Attribute, Expr, Lit, Meta, Token};

/// Struct to hold parsed OCaml attributes from #[ocaml(...)] annotations.
#[derive(Default, Debug)]
pub struct OCamlAttributes {
    pub name: Option<String>,
    pub as_: Option<String>,
    pub tag: Option<String>,
    pub polymorphic_variant: bool,
}

impl OCamlAttributes {
    /// Parse a string attribute value from an expression
    fn parse_string_attribute(attr_name: &str, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Str(lit_str) => Ok(lit_str.value()),
                _ => Err(OCamlInteropError::attribute_error_spanned(
                    Some(attr_name),
                    format!("Expected string literal for '{attr_name}'"),
                    &expr_lit.lit,
                )),
            },
            _ => Err(OCamlInteropError::attribute_error_spanned(
                Some(attr_name),
                format!("Expected literal expression for '{attr_name}'"),
                expr,
            )),
        }
    }

    /// Process a name=value attribute and update the attributes struct
    fn process_named_attribute(
        ocaml_attrs: &mut OCamlAttributes,
        name_value: &syn::MetaNameValue,
    ) -> Result<()> {
        let name = name_value
            .path
            .get_ident()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let value = &name_value.value;
        let attr_str = match name.as_str() {
            "name" => &mut ocaml_attrs.name,
            "as_" => &mut ocaml_attrs.as_,
            "tag" => &mut ocaml_attrs.tag,
            _ => {
                return Err(OCamlInteropError::attribute_error_spanned(
                    Some(name),
                    "Unknown attribute in #[ocaml(...)]",
                    &name_value.path,
                ));
            }
        };

        *attr_str = Some(Self::parse_string_attribute(&name, value)?);
        Ok(())
    }

    /// Parse all OCaml attributes from a list of attributes
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut ocaml_attrs = OCamlAttributes::default();

        // Find and process the #[ocaml(...)] attribute
        for attr in attrs {
            if !attr.path().is_ident("ocaml") {
                continue;
            }

            // Parse the comma-separated meta items inside #[ocaml(...)]
            let nested_metas =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

            // Process each meta item
            for nested_meta in nested_metas {
                match nested_meta {
                    // Handle name=value attributes
                    Meta::NameValue(name_value) => {
                        Self::process_named_attribute(&mut ocaml_attrs, &name_value)?;
                    }

                    // Handle flag attributes
                    Meta::Path(path) => {
                        let name = path
                            .get_ident()
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        match name.as_str() {
                            "polymorphic_variant" => {
                                ocaml_attrs.polymorphic_variant = true;
                            }
                            _ => {
                                return Err(OCamlInteropError::attribute_error_spanned(
                                    Some(name),
                                    "Unknown flag attribute in #[ocaml(...)]",
                                    &path,
                                ));
                            }
                        }
                    }

                    // Handle unsupported attribute formats
                    _ => {
                        return Err(OCamlInteropError::attribute_error_spanned(
                            None::<String>,
                            "Unsupported attribute format inside #[ocaml(...)]",
                            &nested_meta,
                        ));
                    }
                }
            }

            // Only process the first #[ocaml(...)] attribute found
            return Ok(ocaml_attrs);
        }

        // No #[ocaml(...)] attribute found, return default
        Ok(ocaml_attrs)
    }

    // Simple accessor methods
    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }
    pub fn get_tag(&self) -> &Option<String> {
        &self.tag
    }
    pub fn is_polymorphic_variant(&self) -> bool {
        self.polymorphic_variant
    }
}
