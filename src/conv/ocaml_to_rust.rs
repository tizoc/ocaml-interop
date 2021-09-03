// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::OCaml;

use super::FromOCaml;

/// Implements conversion from OCaml values into Rust values.
///
/// This trait enables the definition of conversions for types
/// that have been defined in other crates.
///
/// It contains a default implementation for types that implement [`FromOCaml`].
pub unsafe trait OCamlToRust<RustT>
where
    Self: Sized,
{
    /// Convert to Rust value.
    fn ocaml_to_rust(v: OCaml<Self>) -> RustT;
}

unsafe impl<RustT, OCamlT> OCamlToRust<RustT> for OCamlT
where
    RustT: FromOCaml<OCamlT>,
{
    fn ocaml_to_rust(v: OCaml<Self>) -> RustT {
        RustT::from_ocaml(v)
    }
}
