mod closure;
mod conv;
mod error;
mod macros;
mod memory;
mod mlvalues;
mod runtime;
mod value;

pub use crate::closure::{OCamlFn1, OCamlFn2, OCamlFn3, OCamlFn4, OCamlFn5, OCamlResult};
pub use crate::conv::{FromOCaml, ToOCaml, IntoRust};
pub use crate::error::{OCamlError, OCamlException};
pub use crate::memory::OCamlRef;
pub use crate::mlvalues::{Intnat, OCamlBytes, OCamlInt32, OCamlList, RawOCaml};
pub use crate::runtime::OCamlRuntime;
pub use crate::value::OCaml;

pub mod internal {
    pub use crate::closure::OCamlClosure;
    pub use crate::memory::{GCFrame, GCResult, GCToken};
}

#[cfg(doctest)]
pub mod compile_fail_tests;
