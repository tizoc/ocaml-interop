// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop_dune_builder::DuneBuilder;

fn main() {
    let ocaml_dir = "ocaml";

    // Rebuild if the OCaml source code changes
    println!("cargo:rerun-if-changed={}/test_types.ml", ocaml_dir);
    println!("cargo:rerun-if-changed={}/dune", ocaml_dir);

    // Build the OCaml code using dune
    let dune_builder = DuneBuilder::new(ocaml_dir);
    let objects = dune_builder.build("test_types.exe.o");

    let mut build = cc::Build::new();
    for object in objects {
        build.object(object);
    }

    build.compile("callable_ocaml");
}
