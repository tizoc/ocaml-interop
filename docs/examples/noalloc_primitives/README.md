# Noalloc Primitives Example

This example demonstrates the use of the `#[ocaml_interop::export(noalloc)]` attribute
with primitive types that are passed without boxing/tagging between OCaml and Rust:
- `f64` (OCaml `float`)
- `i32` (OCaml `int32`)
- `i64` (OCaml `int64`)
- `isize` (OCaml `int`)
- `bool` (OCaml `bool`)
- `()` (OCaml `unit`)

Functions marked with `noalloc` use an immutable reference to the OCaml runtime (`&OCamlRuntime`)
and are expected not to allocate new OCaml values that would require garbage collection.
