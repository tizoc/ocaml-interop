// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! Value representation types for OCaml value inspection.

use std::fmt;

/// Represents the different types of OCaml values.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueRepr {
    /// An immediate value (integer, boolean, unit, etc.)
    Immediate {
        /// The raw value as an integer
        value: isize,
        /// Human-readable interpretation of the value
        interpretation: String,
    },
    /// A block value (tuple, record, variant, array, etc.)
    Block {
        /// The tag of the block
        tag: u8,
        /// The size (number of fields) of the block
        size: usize,
        /// The fields of the block
        fields: Vec<ValueRepr>,
        /// Human-readable interpretation of the block type
        interpretation: String,
    },
    /// A string value (special case of block with string tag)
    String {
        /// The actual string content
        content: String,
        /// The byte length of the string
        byte_length: usize,
    },
    /// A custom block (used for boxed values, bigarrays, etc.)
    Custom {
        /// The tag of the custom block
        tag: u8,
        /// The size of the custom block
        size: usize,
        /// A description of what this custom block likely represents
        description: String,
    },
    /// An error occurred while trying to inspect the value
    Error {
        /// Description of the error
        message: String,
    },
}

impl ValueRepr {
    /// Get a short description of the value type
    pub fn type_name(&self) -> &'static str {
        match self {
            ValueRepr::Immediate { .. } => "immediate",
            ValueRepr::Block { .. } => "block",
            ValueRepr::String { .. } => "string",
            ValueRepr::Custom { .. } => "custom",
            ValueRepr::Error { .. } => "error",
        }
    }

    /// Check if this is an immediate value
    pub fn is_immediate(&self) -> bool {
        matches!(self, ValueRepr::Immediate { .. })
    }

    /// Check if this is a block value
    pub fn is_block(&self) -> bool {
        matches!(self, ValueRepr::Block { .. })
    }

    /// Check if this is a string value
    pub fn is_string(&self) -> bool {
        matches!(self, ValueRepr::String { .. })
    }

    /// Check if this is a custom block
    pub fn is_custom(&self) -> bool {
        matches!(self, ValueRepr::Custom { .. })
    }

    /// Check if this represents an error
    pub fn is_error(&self) -> bool {
        matches!(self, ValueRepr::Error { .. })
    }
}

impl fmt::Display for ValueRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRepr::Immediate {
                value,
                interpretation,
            } => {
                write!(f, "Immediate({}): {}", value, interpretation)
            }
            ValueRepr::Block {
                tag,
                size,
                fields,
                interpretation,
            } => {
                write!(f, "Block(tag={}, size={}): {}", tag, size, interpretation)?;
                if !fields.is_empty() {
                    write!(f, " [")?;
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", i, field)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            ValueRepr::String {
                content,
                byte_length,
            } => {
                // Show escaped string content, truncated if too long
                let display_content = if content.len() > 50 {
                    format!("{}...", &content[..50])
                } else {
                    content.clone()
                };
                write!(f, "String({} bytes): {:?}", byte_length, display_content)
            }
            ValueRepr::Custom {
                tag,
                size,
                description,
            } => {
                write!(f, "Custom(tag={}, size={}): {}", tag, size, description)
            }
            ValueRepr::Error { message } => {
                write!(f, "Error: {}", message)
            }
        }
    }
}
