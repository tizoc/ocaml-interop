// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! Core value inspection functionality.

use crate::value_repr::ValueRepr;
use ocaml_sys::{
    field as field_val, is_block, is_long, tag_val, wosize_val, Value as RawOCaml, CLOSURE,
    DOUBLE_ARRAY, NO_SCAN, STRING, TAG_CONS, TAG_SOME,
};
use std::fmt;

/// Maximum depth for recursive inspection to avoid infinite loops
const MAX_DEPTH: usize = 8;

/// Maximum number of fields to display for blocks to keep output readable
const MAX_FIELDS_DISPLAY: usize = 10;

/// A value inspector that can analyze and display OCaml runtime values.
#[derive(Clone)]
pub struct ValueInspector {
    repr: ValueRepr,
}

impl ValueInspector {
    /// Inspect a raw OCaml value and create a new inspector.
    ///
    /// This function analyzes the given raw OCaml value and creates a
    /// representation that shows its internal structure.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `RawOCaml` value is valid and points
    /// to a properly initialized OCaml value.
    pub unsafe fn inspect(raw: RawOCaml) -> Self {
        let repr = Self::inspect_value_recursive(raw, 0);
        ValueInspector { repr }
    }

    /// Get the value representation.
    pub fn repr(&self) -> &ValueRepr {
        &self.repr
    }

    /// Get a compact single-line representation of the value.
    pub fn compact(&self) -> String {
        self.compact_repr(&self.repr, 0)
    }

    /// Recursively inspect an OCaml value with depth tracking.
    unsafe fn inspect_value_recursive(raw: RawOCaml, depth: usize) -> ValueRepr {
        if depth > MAX_DEPTH {
            return ValueRepr::Error {
                message: format!("Maximum inspection depth ({}) exceeded", MAX_DEPTH),
            };
        }

        if is_long(raw) {
            // Immediate value
            Self::inspect_immediate(raw)
        } else if is_block(raw) {
            // Block value
            Self::inspect_block(raw, depth)
        } else {
            ValueRepr::Error {
                message: "Invalid OCaml value: neither immediate nor block".to_string(),
            }
        }
    }

    /// Inspect an immediate (non-block) value.
    unsafe fn inspect_immediate(raw: RawOCaml) -> ValueRepr {
        let value = raw as isize;

        // OCaml integers are represented as (value << 1) | 1
        let interpretation = if value & 1 == 1 {
            // This is a tagged integer
            let int_val = value >> 1;

            // Special handling for known OCaml constants
            if raw == ocaml_sys::TRUE {
                format!("integer {} (likely boolean true)", int_val)
            } else if raw == 0x1 {
                // UNIT, FALSE, NONE, EMPTY_LIST all have value 0x1
                // These all decode to integer 0, so we list all possibilities
                "integer 0 (could be: unit, boolean false, empty list, None)".to_string()
            } else {
                format!("integer {}", int_val)
            }
        } else {
            // This should not happen for immediate values in practice
            format!("raw immediate value {:#x}", value)
        };

        ValueRepr::Immediate {
            value,
            interpretation,
        }
    }

    /// Inspect a block value.
    unsafe fn inspect_block(raw: RawOCaml, depth: usize) -> ValueRepr {
        let tag = tag_val(raw);
        let size = wosize_val(raw);

        match tag {
            // String tag
            t if t == STRING => Self::inspect_string(raw),

            // Double array tag (float array)
            t if t == DOUBLE_ARRAY => Self::inspect_float_array(raw),

            // Closure tag
            t if t == CLOSURE => ValueRepr::Custom {
                tag,
                size,
                description: "function closure".to_string(),
            },

            // Custom blocks (for custom C types, bigarrays, etc.)
            t if t >= NO_SCAN => ValueRepr::Custom {
                tag,
                size,
                description: Self::describe_custom_block(tag),
            },

            // Regular block (variant, tuple, record, etc.)
            _ => Self::inspect_regular_block(raw, tag, size, depth),
        }
    }

    /// Inspect a string value.
    unsafe fn inspect_string(raw: RawOCaml) -> ValueRepr {
        // For string blocks, the length is stored in the header
        // We'll get the word size and estimate the string length
        let size = wosize_val(raw);
        let estimated_byte_length = size * std::mem::size_of::<usize>();

        // For safety, we'll just indicate it's a string without trying to read the content
        // This avoids calling OCaml runtime functions that might not be available
        let content = format!("<string block, ~{} bytes>", estimated_byte_length);

        ValueRepr::String {
            content,
            byte_length: estimated_byte_length,
        }
    }

    /// Inspect a float array (double array).
    unsafe fn inspect_float_array(raw: RawOCaml) -> ValueRepr {
        let size = wosize_val(raw);
        let interpretation = format!("float array of {} elements", size);

        // For float arrays, we could extract the actual values but for now
        // we'll just show it as a custom block
        ValueRepr::Block {
            tag: DOUBLE_ARRAY,
            size,
            fields: vec![], // We don't recurse into float array elements for now
            interpretation,
        }
    }

    /// Inspect a regular block (variant, tuple, record, etc.).
    unsafe fn inspect_regular_block(
        raw: RawOCaml,
        tag: u8,
        size: usize,
        depth: usize,
    ) -> ValueRepr {
        let interpretation = Self::interpret_block_type(tag, size);

        // Recursively inspect fields, but limit the number to avoid excessive output
        let fields_to_inspect = std::cmp::min(size, MAX_FIELDS_DISPLAY);
        let mut fields = Vec::with_capacity(fields_to_inspect);

        for i in 0..fields_to_inspect {
            let field_raw = unsafe { *(field_val(raw, i) as *const RawOCaml) };
            let field_repr = Self::inspect_value_recursive(field_raw, depth + 1);
            fields.push(field_repr);
        }

        // If we truncated fields, add a note
        if size > MAX_FIELDS_DISPLAY {
            fields.push(ValueRepr::Error {
                message: format!("... and {} more fields", size - MAX_FIELDS_DISPLAY),
            });
        }

        ValueRepr::Block {
            tag,
            size,
            fields,
            interpretation,
        }
    }

    /// Interpret the type of a block based on its tag and size.
    fn interpret_block_type(tag: u8, size: usize) -> String {
        match tag {
            0 => match size {
                0 => "variant constructor or unit tuple".to_string(),
                1 => "single-field record or tuple".to_string(),
                2 => "pair tuple or record".to_string(),
                n => format!("tuple/record with {} fields", n),
            },
            t if t == TAG_CONS => "list cons cell (::)".to_string(),
            t if t == TAG_SOME => "option Some".to_string(),
            1..=255 => format!("variant constructor with tag {}", tag),
        }
    }

    /// Describe a custom block based on its tag.
    fn describe_custom_block(tag: u8) -> String {
        match tag {
            t if t == NO_SCAN => "abstract/custom block".to_string(),
            t if t == 255 => "bigarray or similar".to_string(),
            _ => format!("custom block (tag {})", tag),
        }
    }

    /// Create a compact representation that fits on one line.
    fn compact_repr(&self, repr: &ValueRepr, depth: usize) -> String {
        const MAX_COMPACT_DEPTH: usize = 3;

        if depth > MAX_COMPACT_DEPTH {
            return "...".to_string();
        }

        match repr {
            ValueRepr::Immediate { interpretation, .. } => interpretation.clone(),
            ValueRepr::String {
                content,
                byte_length,
            } => {
                if content.len() > 20 {
                    format!("\"{}...\" ({} bytes)", &content[..20], byte_length)
                } else {
                    format!("\"{}\"", content)
                }
            }
            ValueRepr::Block {
                tag,
                size,
                fields,
                interpretation,
            } => {
                if fields.is_empty() {
                    format!("{} (tag={}, size={})", interpretation, tag, size)
                } else if fields.len() == 1 {
                    format!(
                        "{} [{}]",
                        interpretation,
                        self.compact_repr(&fields[0], depth + 1)
                    )
                } else {
                    let field_reprs: Vec<String> = fields
                        .iter()
                        .take(3)
                        .map(|f| self.compact_repr(f, depth + 1))
                        .collect();
                    let fields_str = if fields.len() > 3 {
                        format!("{}, ...", field_reprs.join(", "))
                    } else {
                        field_reprs.join(", ")
                    };
                    format!("{} [{}]", interpretation, fields_str)
                }
            }
            ValueRepr::Custom {
                description,
                tag,
                size,
            } => {
                format!("{} (tag={}, size={})", description, tag, size)
            }
            ValueRepr::Error { message } => {
                format!("Error: {}", message)
            }
        }
    }
}

impl fmt::Display for ValueInspector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl fmt::Debug for ValueInspector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueInspector")
            .field("repr", &self.repr)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_value_interpretation() {
        // Test OCaml integer encoding
        // OCaml integers are encoded as (value << 1) | 1
        let ocaml_int_5 = (5 << 1) | 1; // Should be 11 in binary

        unsafe {
            let inspector = ValueInspector::inspect(ocaml_int_5 as RawOCaml);
            match inspector.repr() {
                ValueRepr::Immediate {
                    value,
                    interpretation,
                } => {
                    assert_eq!(*value, ocaml_int_5 as isize);
                    assert!(interpretation.contains("integer 5"));
                }
                _ => panic!("Expected immediate value"),
            }
        }
    }

    #[test]
    fn test_unit_value() {
        unsafe {
            let inspector = ValueInspector::inspect(ocaml_sys::UNIT);
            match inspector.repr() {
                ValueRepr::Immediate { interpretation, .. } => {
                    // Unit is one of the possibilities for 0x1
                    assert!(interpretation.contains("unit"));
                }
                _ => panic!("Expected immediate value for unit"),
            }
        }
    }

    #[test]
    fn test_boolean_values() {
        unsafe {
            let true_inspector = ValueInspector::inspect(ocaml_sys::TRUE);
            match true_inspector.repr() {
                ValueRepr::Immediate { interpretation, .. } => {
                    assert!(interpretation.contains("true"));
                }
                _ => panic!("Expected immediate value for true"),
            }

            let false_inspector = ValueInspector::inspect(ocaml_sys::FALSE);
            match false_inspector.repr() {
                ValueRepr::Immediate { interpretation, .. } => {
                    // False is one of the possibilities for 0x1
                    assert!(interpretation.contains("false"));
                }
                _ => panic!("Expected immediate value for false"),
            }
        }
    }

    #[test]
    fn test_compact_representation() {
        unsafe {
            let inspector = ValueInspector::inspect((42 << 1) | 1);
            let compact = inspector.compact();
            assert!(compact.contains("42"));
            assert!(!compact.contains('\n')); // Should be single line
        }
    }
}
