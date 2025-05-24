// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! Basic usage examples of the OCaml value inspector.

#[cfg(test)]
use ocaml_interop::*;
use ocaml_interop_inspect::inspect_raw_value;

// Example function that demonstrates how to use the inspector for debugging
#[cfg(test)]
fn debug_ocaml_conversion<T>(cr: &mut OCamlRuntime, value: OCaml<'_, T>, description: &str) {
    println!("=== Debugging OCaml value: {} ===", description);

    // Get the raw OCaml value and inspect it
    let inspection = unsafe { inspect_raw_value(value.raw()) };

    // Show the full detailed representation
    println!("Full details: {}", inspection);

    // Show the compact representation for quick viewing
    println!("Compact: {}", inspection.compact());

    // Show the Debug representation with extra details
    println!("Debug: {:?}", inspection);

    // Check the type
    println!("Type: {}", inspection.repr().type_name());

    println!();
}

// Example function showing how to use the inspector in error handling
#[cfg(test)]
fn safe_conversion_with_debugging<T, R>(
    cr: &mut OCamlRuntime,
    value: OCaml<'_, T>,
) -> Result<R, String>
where
    R: FromOCaml<T>,
{
    // First, inspect the value to understand its structure
    let inspection = unsafe { inspect_raw_value(value.raw()) };

    // Try the conversion
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| value.to_rust::<R>())) {
        Ok(result) => {
            println!("✓ Conversion successful for: {}", inspection.compact());
            Ok(result)
        }
        Err(_) => {
            let error_msg = format!(
                "✗ Conversion failed for value with structure: {}\nFull details: {}",
                inspection.compact(),
                inspection
            );
            Err(error_msg)
        }
    }
}

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
