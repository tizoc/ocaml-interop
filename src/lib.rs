mod closure;
mod conv;
mod error;
mod macros;
mod memory;
mod mlvalues;
mod runtime;
mod value;

pub use crate::closure::OCamlResult;
pub use crate::conv::{FromOCaml, ToOCaml, ToOCamlInteger};
pub use crate::error::Error;
pub use crate::memory::OCamlRef;
pub use crate::mlvalues::{Intnat, RawOCaml};
pub use crate::runtime::OCamlRuntime;
pub use crate::value::OCaml;

pub mod internal {
    pub use crate::closure::OCamlClosure;
    pub use crate::memory::{GCFrame, GCResult, GCToken};
}

#[cfg(doctest)]
pub mod compile_fail_tests;
