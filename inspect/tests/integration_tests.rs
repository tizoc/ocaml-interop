// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop_inspect::{inspect_raw_value, ValueRepr};

#[test]
fn test_immediate_values() {
    // Test various immediate values

    // Integer values
    let int_zero = unsafe { inspect_raw_value((0_isize << 1) | 1) };
    match int_zero.repr() {
        ValueRepr::Immediate {
            value,
            interpretation,
        } => {
            assert_eq!(*value, 1); // OCaml encoding: (0 << 1) | 1 = 1
            assert!(interpretation.contains("integer 0"));
        }
        _ => panic!("Expected immediate value for zero"),
    }

    let int_positive = unsafe { inspect_raw_value((42_isize << 1) | 1) };
    match int_positive.repr() {
        ValueRepr::Immediate {
            value,
            interpretation,
        } => {
            assert_eq!(*value, 85); // OCaml encoding: (42 << 1) | 1 = 85
            assert!(interpretation.contains("integer 42"));
        }
        _ => panic!("Expected immediate value for 42"),
    }

    let int_negative = unsafe { inspect_raw_value((-10_isize << 1) | 1) };
    match int_negative.repr() {
        ValueRepr::Immediate {
            value,
            interpretation,
        } => {
            assert_eq!(*value, -19); // OCaml encoding: (-10 << 1) | 1 = -19
            assert!(interpretation.contains("integer -10"));
        }
        _ => panic!("Expected immediate value for -10"),
    }
}

#[test]
fn test_special_values() {
    // Test special OCaml values

    let unit_val = unsafe { inspect_raw_value(ocaml_sys::UNIT) };
    match unit_val.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("unit"));
        }
        _ => panic!("Expected immediate value for unit"),
    }

    let true_val = unsafe { inspect_raw_value(ocaml_sys::TRUE) };
    match true_val.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("true"));
        }
        _ => panic!("Expected immediate value for true"),
    }

    let false_val = unsafe { inspect_raw_value(ocaml_sys::FALSE) };
    match false_val.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("false"));
        }
        _ => panic!("Expected immediate value for false"),
    }

    let empty_list = unsafe { inspect_raw_value(ocaml_sys::EMPTY_LIST) };
    match empty_list.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("empty list"));
        }
        _ => panic!("Expected immediate value for empty list"),
    }

    let none_val = unsafe { inspect_raw_value(ocaml_sys::NONE) };
    match none_val.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("None"));
        }
        _ => panic!("Expected immediate value for None"),
    }
}

#[test]
fn test_value_type_checks() {
    let int_val = unsafe { inspect_raw_value((123_isize << 1) | 1) };
    assert!(int_val.repr().is_immediate());
    assert!(!int_val.repr().is_block());
    assert!(!int_val.repr().is_string());
    assert!(!int_val.repr().is_custom());
    assert!(!int_val.repr().is_error());
    assert_eq!(int_val.repr().type_name(), "immediate");
}

#[test]
fn test_compact_representation() {
    let int_val = unsafe { inspect_raw_value((999_isize << 1) | 1) };
    let compact = int_val.compact();

    // Compact representation should be concise and contain the essential info
    assert!(compact.contains("999"));
    assert!(!compact.contains('\n')); // Should be single line

    // Test that compact is shorter than full representation
    let full = format!("{}", int_val);
    assert!(compact.len() <= full.len());
}

#[test]
fn test_display_and_debug() {
    let int_val = unsafe { inspect_raw_value((456_isize << 1) | 1) };

    // Test Display trait
    let display_str = format!("{}", int_val);
    assert!(display_str.contains("456"));

    // Test Debug trait
    let debug_str = format!("{:?}", int_val);
    assert!(debug_str.contains("ValueInspector"));
}

#[test]
fn test_value_repr_display() {
    let int_val = unsafe { inspect_raw_value((789_isize << 1) | 1) };
    let display_str = format!("{}", int_val.repr());
    assert!(display_str.contains("Immediate"));
    assert!(display_str.contains("789"));
}

#[test]
fn test_large_integers() {
    // Test edge cases for OCaml integer representation

    // Maximum positive value that fits in OCaml int (platform dependent, but test a reasonably large one)
    let large_positive = unsafe { inspect_raw_value((1000000_isize << 1) | 1) };
    match large_positive.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("integer 1000000"));
        }
        _ => panic!("Expected immediate value for large positive"),
    }

    // Large negative value
    let large_negative = unsafe { inspect_raw_value((-500000_isize << 1) | 1) };
    match large_negative.repr() {
        ValueRepr::Immediate { interpretation, .. } => {
            assert!(interpretation.contains("integer -500000"));
        }
        _ => panic!("Expected immediate value for large negative"),
    }
}

#[test]
fn test_error_representation() {
    // Test the error representation type
    let error_repr = ValueRepr::Error {
        message: "Test error message".to_string(),
    };

    assert!(error_repr.is_error());
    assert!(!error_repr.is_immediate());
    assert_eq!(error_repr.type_name(), "error");

    let display_str = format!("{}", error_repr);
    assert!(display_str.contains("Error: Test error message"));
}

#[test]
fn test_inspector_clone() {
    let int_val = unsafe { inspect_raw_value((42_isize << 1) | 1) };
    let cloned = int_val.clone();

    // Ensure the clone has the same representation
    match (int_val.repr(), cloned.repr()) {
        (
            ValueRepr::Immediate {
                value: v1,
                interpretation: i1,
            },
            ValueRepr::Immediate {
                value: v2,
                interpretation: i2,
            },
        ) => {
            assert_eq!(v1, v2);
            assert_eq!(i1, i2);
        }
        _ => panic!("Expected both to be immediate values"),
    }
}

#[test]
fn test_multiple_inspections() {
    // Test that multiple inspections of the same value produce consistent results
    let raw_val = (100_isize << 1) | 1;

    let inspect1 = unsafe { inspect_raw_value(raw_val) };
    let inspect2 = unsafe { inspect_raw_value(raw_val) };

    assert_eq!(format!("{}", inspect1), format!("{}", inspect2));
    assert_eq!(inspect1.compact(), inspect2.compact());
}
