// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use crate::conv::FromOCaml;
use crate::value::OCaml;

/// Counterpart to [`FromOCaml`], usually more convenient to use.
pub trait ToRust<T>: Sized {
    /// Convert into a Rust value.
    fn to_rust(&self) -> T;
}

impl<'a, OCamlT, RustT> ToRust<RustT> for OCaml<'a, OCamlT>
where
    RustT: FromOCaml<OCamlT>,
{
    fn to_rust(&self) -> RustT {
        RustT::from_ocaml(self)
    }
}
