// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::conv::FromOCaml;
use crate::value::OCaml;

/// Counterpart to `FromOCaml`, usually more convenient to use.
pub trait IntoRust<T>: Sized {
    /// Convert into a Rust value.
    fn into_rust(self) -> T;
}

impl<'a, OCamlT, RustT> IntoRust<RustT> for OCaml<'a, OCamlT>
where
    RustT: FromOCaml<OCamlT>,
{
    fn into_rust(self) -> RustT {
        RustT::from_ocaml(self)
    }
}
