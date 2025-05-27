// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! This example demonstrates how to use the ocaml-interop-inspect crate
//! with an actual OCaml runtime.
//!
//! It shows how to:
//! 1. Initialize the OCaml runtime
//! 2. Call OCaml functions that create various data types
//! 3. Inspect the values returned from OCaml using the inspect_raw_value function
//!
//! The OCaml code defines various types (records, variants, lists, polymorphic variants)
//! that are created and returned to Rust for inspection.
//!
//! For each OCaml value, the example shows:
//! - The OCaml literal representation (as shown in the INSPECTING: line)
//! - The detailed memory structure (blocks, tags, fields)
//! - A compact view summarizing the structure
//! - A debug view showing the internal representation
//!
//! This is useful for understanding how OCaml values are represented in memory
//! and how the ocaml-interop-inspect crate helps visualize this representation.

use ocaml_interop::*;
use ocaml_interop_inspect::inspect_raw_value;

// Define a struct to represent unknown OCaml values
// This is used as a placeholder return type for OCaml functions
struct UnknownOCamlValue;

mod ocaml {
    use super::UnknownOCamlValue;
    use ocaml_interop::*;

    ocaml! {
        pub fn create_test_record(unit: ()) -> UnknownOCamlValue;
        pub fn create_empty_variant(unit: ()) -> UnknownOCamlValue;
        pub fn create_int_variant(unit: ()) -> UnknownOCamlValue;
        pub fn create_string_variant(unit: ()) -> UnknownOCamlValue;
        pub fn create_tuple_variant(unit: ()) -> UnknownOCamlValue;
        pub fn create_complex_variant(unit: ()) -> UnknownOCamlValue;
        pub fn create_list_of_variants(unit: ()) -> UnknownOCamlValue;
        pub fn create_poly_none(unit: ()) -> UnknownOCamlValue;
        pub fn create_poly_int(unit: ()) -> UnknownOCamlValue;
        pub fn create_poly_tuple(unit: ()) -> UnknownOCamlValue;
    }
}

fn print_separator() {
    println!("\n{}", "=".repeat(80));
}

fn inspect_value(raw_value: RawOCaml, name: &str) {
    print_separator();
    println!("INSPECTING: {}", name);
    println!();

    // Inspect the raw OCaml value directly
    let inspection = unsafe { inspect_raw_value(raw_value) };

    // Print the detailed representation
    println!("DETAILED STRUCTURE:");
    println!("{}", inspection);

    // Print the compact view
    println!("\nCOMPACT VIEW:");
    println!("{}", inspection.compact());

    // Print the debug representation
    println!("\nDEBUG VIEW:");
    println!("{:?}", inspection);
}

fn main() {
    // Step 1: Initialize the OCaml runtime
    // This starts the OCaml runtime and allows us to call OCaml functions from Rust
    // The guard ensures that the OCaml runtime is properly shut down when we're done
    let _guard = OCamlRuntime::init().expect("Failed to initialize OCaml runtime");

    println!("OCaml Interop Inspect - Runtime Example");
    println!("======================================");
    println!("This example shows how to inspect various OCaml values using ocaml-interop-inspect.");

    // Step 2: Acquire the domain lock before interacting with OCaml
    // This is necessary for thread safety when calling OCaml functions
    OCamlRuntime::with_domain_lock(|cr| {
        // Test record
        let test_record = ocaml::create_test_record(cr, ());
        unsafe {
            inspect_value(test_record.get_raw(),
            "{ int_field = 42; float_field = 3.14159; string_field = \"Hello, OCaml!\"; bool_field = true; tuple_field = (123, \"tuple element\", 45.67) }");
        }

        // Empty variant
        let empty_variant = ocaml::create_empty_variant(cr, ());
        unsafe {
            inspect_value(empty_variant.get_raw(), "Empty");
        }

        // Int variant
        let int_variant = ocaml::create_int_variant(cr, ());
        unsafe {
            inspect_value(int_variant.get_raw(), "WithInt 42");
        }

        // String variant
        let string_variant = ocaml::create_string_variant(cr, ());
        unsafe {
            inspect_value(string_variant.get_raw(), "WithString \"variant string\"");
        }

        // Tuple variant
        let tuple_variant = ocaml::create_tuple_variant(cr, ());
        unsafe {
            inspect_value(tuple_variant.get_raw(), "WithTuple (42, 3.14)");
        }

        // Complex variant
        let complex_variant = ocaml::create_complex_variant(cr, ());
        unsafe {
            inspect_value(complex_variant.get_raw(),
            "Complex { int_field = 42; float_field = 3.14159; string_field = \"Hello, OCaml!\"; bool_field = true; tuple_field = (123, \"tuple element\", 45.67) }");
        }

        // List of variants
        let list_of_variants = ocaml::create_list_of_variants(cr, ());
        unsafe {
            inspect_value(
                list_of_variants.get_raw(),
                "[Empty; WithInt 42; WithString \"variant in list\"; WithTuple (99, 99.99)]",
            );
        }

        // Polymorphic variants
        let poly_none = ocaml::create_poly_none(cr, ());
        unsafe {
            inspect_value(poly_none.get_raw(), "`None");
        }

        let poly_int = ocaml::create_poly_int(cr, ());
        unsafe {
            inspect_value(poly_int.get_raw(), "`Int 42");
        }

        let poly_tuple = ocaml::create_poly_tuple(cr, ());
        unsafe {
            inspect_value(poly_tuple.get_raw(), "`Tuple (42, \"poly tuple\")");
        }
    });

    print_separator();
    println!("Example completed successfully!");
}
