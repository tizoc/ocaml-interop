// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use proc_macro2::Span;
use std::fmt;
use syn::spanned::Spanned;

#[derive(Debug)]
pub enum OCamlInteropError {
    Message { message: String, span: Span },
    Syn(syn::Error),
}

impl OCamlInteropError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        OCamlInteropError::Message {
            message: message.into(),
            span,
        }
    }

    pub fn new_spanned(message: impl Into<String>, item: &impl Spanned) -> Self {
        Self::new(message, item.span())
    }

    // Specialized errors that format messages appropriately
    pub fn type_error_spanned(message: impl Into<String>, item: &impl Spanned) -> Self {
        Self::new_spanned(message, item)
    }

    pub fn attribute_error_spanned(
        name: Option<impl Into<String>>,
        message: impl Into<String>,
        item: &impl Spanned,
    ) -> Self {
        let msg = if let Some(n) = name {
            format!("Attribute '{}': {}", n.into(), message.into())
        } else {
            message.into()
        };
        Self::new_spanned(msg, item)
    }

    pub fn validation_error(
        message: impl Into<String>,
        span: Span,
        context: Option<impl Into<String>>,
    ) -> Self {
        let msg = if let Some(ctx) = context {
            format!("{}: {}", ctx.into(), message.into())
        } else {
            message.into()
        };
        Self::new(msg, span)
    }

    pub fn validation_error_spanned(
        message: impl Into<String>,
        item: &impl Spanned,
        context: Option<impl Into<String>>,
    ) -> Self {
        Self::validation_error(message, item.span(), context)
    }

    pub fn span(&self) -> Span {
        match self {
            OCamlInteropError::Message { span, .. } => *span,
            OCamlInteropError::Syn(err) => err.span(),
        }
    }

    pub fn to_syn_error(&self) -> syn::Error {
        match self {
            OCamlInteropError::Syn(err) => err.clone(),
            OCamlInteropError::Message { message, span } => syn::Error::new(*span, message),
        }
    }

    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        self.to_syn_error().to_compile_error()
    }

    pub fn into_syn_error(self) -> syn::Error {
        match self {
            OCamlInteropError::Syn(err) => err,
            OCamlInteropError::Message { message, span } => syn::Error::new(span, message),
        }
    }

    // into_err removed as it was just a wrapper around Err
}

impl fmt::Display for OCamlInteropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OCamlInteropError::Message { message, .. } => write!(f, "{message}"),
            OCamlInteropError::Syn(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for OCamlInteropError {}

impl From<syn::Error> for OCamlInteropError {
    fn from(err: syn::Error) -> Self {
        OCamlInteropError::Syn(err)
    }
}

impl From<OCamlInteropError> for syn::Error {
    fn from(err: OCamlInteropError) -> Self {
        err.to_syn_error()
    }
}

/// A specialized Result type for OCamlInteropDerive
pub type Result<T> = std::result::Result<T, OCamlInteropError>;
