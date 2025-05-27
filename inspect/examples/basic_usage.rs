// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! Basic usage examples of the OCaml value inspector.

use ocaml_interop_inspect::inspect_raw_value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_inspect_integer() {
        // This example shows how to inspect OCaml integers
        let inspection = unsafe {
            inspect_raw_value((42_isize << 1) | 1) // OCaml integer encoding
        };

        assert!(inspection.repr().is_immediate());
        assert!(inspection.compact().contains("integer 42"));
        println!("Integer inspection: {}", inspection);
    }

    #[test]
    fn example_inspect_unit() {
        let inspection = unsafe { inspect_raw_value(ocaml_sys::UNIT) };

        assert!(inspection.repr().is_immediate());
        assert!(inspection.compact().contains("unit"));
        println!("Unit inspection: {}", inspection);
    }

    #[test]
    fn example_inspect_boolean() {
        let true_inspection = unsafe { inspect_raw_value(ocaml_sys::TRUE) };

        let false_inspection = unsafe { inspect_raw_value(ocaml_sys::FALSE) };

        assert!(true_inspection.repr().is_immediate());
        assert!(false_inspection.repr().is_immediate());
        assert!(true_inspection.compact().contains("true"));
        assert!(false_inspection.compact().contains("false"));

        println!("True inspection: {}", true_inspection);
        println!("False inspection: {}", false_inspection);
    }
}

fn main() {
    println!("OCaml Value Inspector Examples");
    println!("==============================");

    // Examples of inspecting different types of OCaml values

    // Integer inspection
    println!("1. Integer values:");
    let int_inspection = unsafe { inspect_raw_value((123_isize << 1) | 1) };
    println!("   {}", int_inspection);
    println!("   Compact: {}", int_inspection.compact());

    // Boolean inspection
    println!("\n2. Boolean values:");
    let true_inspection = unsafe { inspect_raw_value(ocaml_sys::TRUE) };
    let false_inspection = unsafe { inspect_raw_value(ocaml_sys::FALSE) };
    println!("   True: {}", true_inspection);
    println!("   False: {}", false_inspection);

    // Unit inspection
    println!("\n3. Unit value:");
    let unit_inspection = unsafe { inspect_raw_value(ocaml_sys::UNIT) };
    println!("   {}", unit_inspection);

    // Empty list inspection
    println!("\n4. Empty list:");
    let empty_list_inspection = unsafe { inspect_raw_value(ocaml_sys::EMPTY_LIST) };
    println!("   {}", empty_list_inspection);

    // None option inspection
    println!("\n5. None option:");
    let none_inspection = unsafe { inspect_raw_value(ocaml_sys::NONE) };
    println!("   {}", none_inspection);

    println!("\nFor block values (tuples, records, variants), you'll need to");
    println!("create them through the OCaml runtime within a proper context.");
    println!("This example shows the basic immediate value inspection capabilities.");
}
