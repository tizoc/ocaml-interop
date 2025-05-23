// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! # OCaml Interop Inspect
//!
//! This crate provides utilities for inspecting OCaml runtime values to help debug
//! conversions between OCaml and Rust. It offers human-readable representations
//! of OCaml values that show their internal structure including whether they are
//! immediate values or blocks, their tags, sizes, and field contents.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ocaml_sys::Value as RawOCaml;
//! use ocaml_interop_inspect::inspect_raw_value;
//!
//! // When you have access to a raw OCaml value:
//! unsafe {
//!     let raw_value: RawOCaml = /* some OCaml value */;
//!     let inspection = inspect_raw_value(raw_value);
//!     println!("OCaml value structure: {}", inspection);
//!     // Or for debugging output:
//!     println!("OCaml value structure: {:?}", inspection);
//! }
//! ```
//!
//! When using with ocaml-interop, you can get the raw value using `value.raw()`:
//!
//! ```rust,ignore
//! use ocaml_interop::*;
//! use ocaml_interop_inspect::inspect_raw_value;
//!
//! fn debug_conversion<T>(value: OCaml<'_, T>) {
//!     let inspection = unsafe { inspect_raw_value(value.raw()) };
//!     println!("OCaml value structure: {}", inspection);
//! }
//! ```

// We only need RawOCaml from ocaml-sys
use ocaml_sys::Value as RawOCaml;

pub mod inspector;
pub mod value_repr;

pub use inspector::ValueInspector;
pub use value_repr::ValueRepr;

/// Inspect a raw OCaml value directly.
///
/// This function analyzes the raw representation of an OCaml value and returns
/// a `ValueInspector` that can be displayed to show the value's structure,
/// including whether it's immediate or a block, its tag, size, and contents.
///
/// # Example
///
/// ```rust,ignore
/// use ocaml_sys::Value as RawOCaml;
/// use ocaml_interop_inspect::inspect_raw_value;
///
/// unsafe {
///     let raw_value: RawOCaml = /* some OCaml value */;
///     let inspection = inspect_raw_value(raw_value);
///     println!("Value structure: {}", inspection);
/// }
/// ```
///
/// # Safety
///
/// The caller must ensure that the `RawOCaml` value is valid.
pub unsafe fn inspect_raw_value(raw: RawOCaml) -> ValueInspector {
    ValueInspector::inspect(raw)
}
