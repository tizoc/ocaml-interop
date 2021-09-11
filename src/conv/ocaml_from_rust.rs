// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::{BoxRoot, OCaml, OCamlRuntime};

use super::ToOCaml;

/// Implements conversion from Rust values into OCaml values.
///
/// This trait enables the definition of conversions for types
/// that have been defined in other crates.
///
/// It contains a default implementation for types that implement [`ToOCaml`].
pub unsafe trait OCamlFromRust<RustT>
where
    Self: Sized,
{
    /// Convert from Rust value.
    fn ocaml_from_rust<'gc>(cr: &'gc mut OCamlRuntime, v: &RustT) -> OCaml<'gc, Self>;

    /// Convert from Rust value. Return an already rooted value as [`BoxRoot`]`<Self>`.
    fn boxroot_from_rust<'gc>(cr: &'gc mut OCamlRuntime, v: &RustT) -> BoxRoot<Self> {
        BoxRoot::new(Self::ocaml_from_rust(cr, v))
    }
}

unsafe impl<RustT, OCamlT> OCamlFromRust<RustT> for OCamlT
where
    RustT: ToOCaml<OCamlT>,
{
    fn ocaml_from_rust<'gc>(cr: &'gc mut OCamlRuntime, v: &RustT) -> OCaml<'gc, OCamlT> {
        v.to_ocaml(cr)
    }
}
