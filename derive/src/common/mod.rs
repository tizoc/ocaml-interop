// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

pub mod attr_parsing;
pub mod error;
pub mod field_processing;
pub mod parsing;
pub mod polytag_utils;
pub mod validation;

pub use error::{OCamlInteropError, Result};

pub fn format_type(ty: String) -> String {
    ty.replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("> ", ">")
        .replace(" ,", ",")
}
