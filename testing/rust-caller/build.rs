// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

// DuneBuilder is a helper for building OCaml code with dune and collecting object
// files for linking with Rust.
use ocaml_interop_dune_builder::DuneBuilder;

fn main() {
    // Relative path to the OCaml source directory
    // It is used to determine the location of the dune project and the build directory.
    // The path is relative to the crate's manifest directory.
    // In this case, it is "ocaml", which is the directory containing the OCaml code.
    let ocaml_callable_dir = "ocaml";

    // Rebuild if the OCaml source code changes.
    println!("cargo:rerun-if-changed={}/callable.ml", ocaml_callable_dir);
    println!("cargo:rerun-if-changed={}/dune", ocaml_callable_dir);

    // Build the OCaml code using dune and collect the object files for linking with Rust.
    let dune_builder = DuneBuilder::new(ocaml_callable_dir);
    let objects = dune_builder.build("callable.exe.o");

    let mut build = cc::Build::new();
    for object in objects {
        build.object(object);
    }

    build.compile("callable_ocaml");
}
