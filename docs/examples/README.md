# OCaml-Interop Examples

This directory contains various examples demonstrating the use of the `ocaml-interop` library to call Rust code from OCaml and vice-versa.

## Running the Examples

The method for running each example depends on whether it is primarily an OCaml program calling Rust, or a Rust program calling OCaml.

### OCaml-Driven Examples (using Dune)

Many examples in this directory are OCaml programs that demonstrate calling Rust functions. These are typically integrated with Dune. To build and run these examples, navigate to the root of the `ocaml-interop` project and execute:

```bash
opam exec -- dune test -f
```
This command will compile the necessary Rust libraries and OCaml executables, and then run the OCaml programs. Each such example usually resides in its own subdirectory (e.g., `tuples/`, `records/`) and contains both the Rust and OCaml source code, along with a `dune` file.

### Rust-Driven Examples (using Cargo)

Some examples might be Rust programs that demonstrate calling OCaml functions. These examples are typically run using Cargo. To run these, you would usually navigate to the specific example's Rust project directory and execute:

```bash
cargo test
```
Or a similar `cargo run` command if it's a binary. Refer to the specific `README.md` within such an example's directory for precise instructions if they differ.
