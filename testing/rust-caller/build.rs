// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let ocaml_callable_dir = "./ocaml";
    let dune_dir = "../../_build/default/testing/rust-caller/ocaml";
    Command::new("dune")
        .args(&["build", &format!("{}/callable.exe.o", ocaml_callable_dir)])
        .status()
        .expect("Dune failed");
    Command::new("rm")
        .args(&["-f", &format!("{}/libcallable.a", out_dir)])
        .status()
        .expect("rm failed");
    Command::new("rm")
        .args(&["-f", &format!("{}/libcallable.o", out_dir)])
        .status()
        .expect("rm failed");
    Command::new("cp")
        .args(&[
            &format!("{}/callable.exe.o", dune_dir),
            &format!("{}/libcallable.o", out_dir),
        ])
        .status()
        .expect("File copy failed.");
    Command::new("ar")
        .args(&[
            "qs",
            &format!("{}/libcallable.a", out_dir),
            &format!("{}/libcallable.o", out_dir),
        ])
        .status()
        .expect("ar failed");

    println!("cargo:rerun-if-changed={}/callable.ml", ocaml_callable_dir);
    println!("cargo:rerun-if-changed={}/dune", ocaml_callable_dir);
    println!("cargo:rustc-link-search={}", out_dir);
    println!("cargo:rustc-link-lib=static=callable");
}
