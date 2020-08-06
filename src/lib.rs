mod closure;
mod conv;
mod error;
mod macros;
mod memory;
mod mlvalues;
mod runtime;
mod value;

pub use crate::closure::{OCamlClosure, OCamlResult};
pub use crate::conv::{FromOCaml, ToOCaml, ToOCamlInteger};
pub use crate::error::Error;
pub use crate::memory::{GCFrame, GCResult, GCToken, OCamlRef};
pub use crate::mlvalues::{Intnat, RawOCaml};
pub use crate::runtime::init as init_ocaml_runtime;
pub use crate::runtime::shutdown as shutdown_ocaml_runtime;
pub use crate::value::OCaml;


#[cfg(doctest)]
pub mod compile_fail_tests;