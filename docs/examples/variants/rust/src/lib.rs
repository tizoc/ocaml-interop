// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop::{FromOCaml, OCaml, OCamlInt, OCamlRuntime, ToOCaml};

// Rust enum mirroring the OCaml variant type.
// Order of variants must match OCaml definition.
#[derive(Debug, PartialEq, Clone, FromOCaml, ToOCaml)]
#[ocaml(as_ = "OCamlStatus")]
pub enum Status {
    Ok,
    Error(String),
    Retrying(#[ocaml(as_ = "OCamlInt")] i64),
}

// Rust marker type for the OCaml `status` variant.
// This is used in type signatures like OCaml<OCamlStatus>.
pub enum OCamlStatus {}

// Exported Rust function that takes an OCaml `status`,
// converts it to Rust `Status`, processes it, and returns a string.
#[ocaml_interop::export]
pub fn rust_process_status(cr: &mut OCamlRuntime, status_val: OCaml<OCamlStatus>) -> OCaml<String> {
    let status_rust: Status = status_val.to_rust();
    let result_string = match status_rust {
        Status::Ok => "Rust received: Ok".to_string(),
        Status::Error(s) => format!("Rust received: Error(\"{}\")", s),
        Status::Retrying(n) => format!("Rust received: Retrying({})", n),
    };
    result_string.to_ocaml(cr)
}

// Exported Rust function that creates and returns an OCaml `Ok` variant.
#[ocaml_interop::export]
pub fn rust_create_status_ok(cr: &mut OCamlRuntime, _unused: OCaml<()>) -> OCaml<OCamlStatus> {
    Status::Ok.to_ocaml(cr)
}

// Exported Rust function that creates and returns an OCaml `Error` variant.
#[ocaml_interop::export]
pub fn rust_create_status_error(
    cr: &mut OCamlRuntime,
    message: OCaml<String>,
) -> OCaml<OCamlStatus> {
    Status::Error(message.to_rust()).to_ocaml(cr)
}

// Exported Rust function that creates and returns an OCaml `Retrying` variant.
#[ocaml_interop::export]
pub fn rust_create_status_retrying(
    cr: &mut OCamlRuntime,
    count: OCaml<OCamlInt>,
) -> OCaml<OCamlStatus> {
    Status::Retrying(count.to_rust()).to_ocaml(cr)
}
