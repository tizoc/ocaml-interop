// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop::{OCaml, OCamlRuntime};

// Example with f64 (float)
#[ocaml_interop::export(noalloc)]
pub fn rust_process_float(_cr: &OCamlRuntime, input_float: f64) -> f64 {
    println!("[Rust] rust_process_float received: {}", input_float);
    input_float * 2.0
}

// Example with i64 (int64)
#[ocaml_interop::export(noalloc)]
pub fn rust_process_int64(_cr: &OCamlRuntime, input_i64: i64) -> i64 {
    println!("[Rust] rust_process_int64 received: {}", input_i64);
    input_i64 + 100
}

// Example with bool
#[ocaml_interop::export(noalloc)]
pub fn rust_process_bool(_cr: &OCamlRuntime, input_bool: bool) -> bool {
    println!("[Rust] rust_process_bool received: {}", input_bool);
    !input_bool
}

// Example with unit argument and unit return
#[ocaml_interop::export(noalloc)]
pub fn rust_process_unit_to_unit(_cr: &OCamlRuntime, _input_unit: OCaml<()>) {
    println!("[Rust] rust_process_unit_to_unit called");
    // No explicit return needed for unit, it's implicit.
}

// Example with isize (OCaml int @untagged)
#[ocaml_interop::export(noalloc)]
pub fn rust_process_isize(_cr: &OCamlRuntime, input_isize: isize) -> isize {
    println!("[Rust] rust_process_isize received: {}", input_isize);
    input_isize + 5
}

// Example with i32 (OCaml int32)
#[ocaml_interop::export(noalloc)]
pub fn rust_process_i32(_cr: &OCamlRuntime, input_i32: i32) -> i32 {
    println!("[Rust] rust_process_i32 received: {}", input_i32);
    input_i32 * 2
}

#[ocaml_interop::export(noalloc)]
pub fn rust_combine_primitives_noalloc(_cr: &OCamlRuntime, f: f64, i: i64, b: bool) -> i64 {
    println!(
        "[Rust] rust_combine_primitives_noalloc: f={}, i={}, b={}",
        f, i, b
    );
    let mut result = i;
    if b {
        result += f as i64;
    } else {
        result -= f as i64;
    }
    result
}

// Example with multiple primitive arguments and returning one of them (noalloc friendly)
#[ocaml_interop::export(noalloc)]
pub fn rust_select_i64_noalloc(_cr: &OCamlRuntime, f: f64, i: i64, b: bool) -> i64 {
    println!("[Rust] rust_select_i64_noalloc: f={}, i={}, b={}", f, i, b);
    if b {
        i + (f as i64)
    } else {
        i - (f as i64)
    }
}

// Example returning unit, taking multiple primitives
#[ocaml_interop::export(noalloc)]
pub fn rust_log_primitives_noalloc(_cr: &OCamlRuntime, f: f64, i: i64, b: bool) {
    println!(
        "[Rust] rust_log_primitives_noalloc: float={}, int64={}, bool={}",
        f, i, b
    );
}
