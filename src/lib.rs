// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

#![doc(html_root_url = "https://docs.rs/ocaml-interop/0.11.2")]

//! _Zinc-iron alloy coating is used in parts that need very good corrosion protection._
//!
//! **API IS CONSIDERED UNSTABLE AT THE MOMENT AND IS LIKELY TO CHANGE IN THE FUTURE**
//!
//! **IMPORTANT: Starting with version `0.11.0` only OCaml 5.x is supported**
//!
//! [ocaml-interop](https://github.com/tizoc/ocaml-interop) is an OCaml<->Rust FFI with an emphasis
//! on safety inspired by [caml-oxide](https://github.com/stedolan/caml-oxide),
//! [ocaml-rs](https://github.com/zshipko/ocaml-rs) and [CAMLroot](https://arxiv.org/abs/1812.04905).
//! 
//! ## Table of Contents
//!
//! - [Usage](#usage)
//!   * [Runtime Initialization and Management](#runtime-initialization-and-management)
//!   * [Acquiring and Using the OCaml Runtime Handle](#acquiring-and-using-the-ocaml-runtime-handle)
//!   * [OCaml value representation](#ocaml-value-representation)
//!   * [Converting between OCaml and Rust data](#converting-between-ocaml-and-rust-data)
//!   * [Calling convention](#calling-convention)
//!   * [OCaml exceptions](#ocaml-exceptions)
//!   * [Calling into OCaml from Rust](#calling-into-ocaml-from-rust)
//!   * [Calling into Rust from OCaml](#calling-into-rust-from-ocaml)
//! - [User Guides](user_guides)
//! - [References and links](#references-and-links)
//!
//! ## Usage
//!
//! This section provides a high-level overview of `ocaml-interop`. For detailed explanations,
//! tutorials, and best practices, please refer to the [User Guides module](user_guides).
//!
//! ### Runtime Initialization and Management
//!
//! Proper initialization and management of the OCaml runtime is crucial, especially when Rust
//! code drives the execution. This involves using [`OCamlRuntime::init`] and managing its
//! lifecycle with [`OCamlRuntimeStartupGuard`].
//!
//! For detailed information, see
//! [OCaml Runtime (Part 5)](user_guides::part5_managing_the_ocaml_runtime_for_rust_driven_programs).
//!
//! ### Acquiring and Using the OCaml Runtime Handle
//!
//! Most interop operations require an OCaml runtime handle (`cr: &mut OCamlRuntime`).
//! This handle is obtained differently depending on whether Rust calls OCaml or OCaml calls Rust.
//!
//! See these guides for more details:
//! - [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts)
//! - [OCaml Runtime (Part 5)](user_guides::part5_managing_the_ocaml_runtime_for_rust_driven_programs)
//!
//! ### OCaml value representation
//!
//! OCaml values are represented in Rust using types like [`OCaml<'gc, T>`](OCaml),
//! [`BoxRoot<T>`](BoxRoot), and [`OCamlRef<'a, T>`](OCamlRef), each with specific roles
//! in memory management and GC interaction.
//!
//! Learn more in [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts).
//!
//! ### Converting between OCaml and Rust data
//!
//! The traits [`FromOCaml`] and [`ToOCaml`] facilitate data conversion
//! between Rust and OCaml types.
//!
//! For conversion details and examples, refer to
//! [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts), as well as the guides
//! on exporting and invoking functions.
//!
//! ### Calling convention
//!
//! `ocaml-interop` uses a caller-rooted argument convention for safety, where the caller is
//! responsible for ensuring arguments are rooted before a function call.
//!
//! This is explained further in [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts).
//!
//! ### OCaml exceptions
//!
//! By default, Rust panics in exported functions are caught and translated to OCaml exceptions.
//! Conversely, OCaml exceptions raised during calls from Rust will result in Rust panics.
//!
//! For error handling strategies, see
//! [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts) and
//! [Part 6: Advanced Topics](user_guides::part6_advanced_topics).
//!
//! ### Calling into OCaml from Rust
//!
//! To call OCaml functions from Rust, they typically need to be registered in OCaml
//! (e.g., using `Callback.register`) and then declared in Rust using the [`ocaml!`] macro.
//! This setup allows Rust to find and invoke these OCaml functions.
//!
//! For a comprehensive guide on calling OCaml functions from Rust,
//! including detailed examples and best practices, please see:
//! [Invoking OCaml Functions (Part 4)](user_guides::part4_invoking_ocaml_functions_from_rust).
//!
//! ### Calling into Rust from OCaml
//!
//! Rust functions can be exposed to OCaml using the [`#[ocaml_interop::export]`](export)
//! procedural macro, which handles FFI boilerplate, type marshalling, and panic safety.
//!
//! Attributes like `no_panic_catch`, `bytecode`, and `noalloc` allow customization.
//!
//! For a detailed guide, see
//! [Exporting Rust Functions (Part 3)](user_guides::part3_exporting_rust_functions_to_ocaml).
//!
//! ## References and links
//!
//! - OCaml Manual: [Chapter 20  Interfacing C with OCaml](https://caml.inria.fr/pub/docs/manual-ocaml/intfc.html).
//! - [Safely Mixing OCaml and Rust](https://docs.google.com/viewer?a=v&pid=sites&srcid=ZGVmYXVsdGRvbWFpbnxtbHdvcmtzaG9wcGV8Z3g6NDNmNDlmNTcxMDk1YTRmNg) paper by Stephen Dolan.
//! - [Safely Mixing OCaml and Rust](https://www.youtube.com/watch?v=UXfcENNM_ts) talk by Stephen Dolan.
//! - [CAMLroot: revisiting the OCaml FFI](https://arxiv.org/abs/1812.04905).
//! - [caml-oxide](https://github.com/stedolan/caml-oxide), the code from that paper.
//! - [ocaml-rs](https://github.com/zshipko/ocaml-rs), another OCaml<->Rust FFI library.

mod boxroot;
mod closure;
mod conv;
mod error;
mod macros;
mod memory;
mod mlvalues;
mod runtime;
mod value;
#[doc = include_str!("../docs/README.md")]
pub mod user_guides;

pub use crate::boxroot::BoxRoot;

pub use crate::closure::{OCamlFn1, OCamlFn2, OCamlFn3, OCamlFn4, OCamlFn5};
pub use crate::conv::{FromOCaml, ToOCaml};
pub use crate::memory::alloc_cons as cons;
pub use crate::memory::OCamlRef;
pub use crate::memory::{alloc_error, alloc_ok};
pub use crate::mlvalues::{
    bigarray, DynBox, OCamlBytes, OCamlException, OCamlFloat, OCamlFloatArray, OCamlInt,
    OCamlInt32, OCamlInt64, OCamlList, OCamlUniformArray, RawOCaml,
};
pub use crate::runtime::{OCamlRuntime, OCamlRuntimeStartupGuard};
pub use crate::value::OCaml;

/// Exports a Rust function to OCaml.
///
/// This procedural macro handles the complexities of the OCaml Foreign Function Interface (FFI),
/// allowing Rust functions to be called from OCaml code. It generates the necessary
/// `extern "C"` wrapper function and manages type conversions and memory safety.
///
/// ## Basic Usage
///
/// ```rust
/// use ocaml_interop::{OCaml, OCamlRuntime, OCamlBytes, OCamlInt, ToOCaml};
///
/// #[ocaml_interop::export]
/// fn process_bytes(cr: &mut OCamlRuntime, data: OCaml<OCamlBytes>) -> OCaml<OCamlInt> {
///     let byte_slice: &[u8] = &data.to_rust::<Vec<u8>>();
///     let length = byte_slice.len() as i64;
///     length.to_ocaml(cr)
/// }
/// ```
///
/// The macro generates an `extern "C"` function with the same identifier as the Rust function
/// (e.g., `process_bytes` in the example above).
///
/// ## Key Features
///
/// *   **Automatic FFI Boilerplate:** Generates the `extern "C"` wrapper and handles argument/return
///     value marshalling.
/// *   **Type Safety:** Utilizes types like [`OCaml<T>`] and [`BoxRoot<T>`] to provide safe
///     abstractions over OCaml values.
/// *   **Argument Handling:**
///     *   The first argument *must* be [`&mut OCamlRuntime`] (or [`&OCamlRuntime`] if `noalloc`
///         is used).
///     *   [`OCaml<'gc, T>`]: For OCaml values passed as arguments. These are *not* automatically
///         rooted by the macro. Their lifetime `'gc` is tied to the current function call's
///         [`OCamlRuntime`] scope. Root them explicitly (e.g., with [`BoxRoot<T>`]) if they need to
///         persist beyond this scope or be re-passed to OCaml.
///     *   [`BoxRoot<T>`]: If an argument is declared as `BoxRoot<T>`, the macro automatically
///         roots the incoming OCaml value before your function body executes. This ensures the value
///         is valid throughout the function, even across further OCaml calls.
///     *   **Direct Primitive Types:** Supports direct mapping for Rust primitive types like `f64`,
///         `i64`, `i32`, `bool`, and `isize` as arguments. The OCaml `external` declaration
///         must use corresponding `[@@unboxed]` or `[@untagged]` attributes.
/// *   **Return Types:**
///     *   Typically, functions return [`OCaml<T>`].
///     *   Direct primitive types (see above) can also be returned.
/// *   **Panic Handling:**
///     *   By default, Rust panics are caught and raised as an OCaml exception (`RustPanic of string`
///         if registered, otherwise `Failure`).
///     *   This can be disabled with `#[ocaml_interop::export(no_panic_catch)]`. Use with caution.
/// *   **Bytecode Function Generation:**
///     *   Use `#[ocaml_interop::export(bytecode = "my_ocaml_bytecode_function_name")]` to generate
///         a wrapper for OCaml bytecode compilation.
///     *   The OCaml `external` declaration should then specify both the bytecode and native
///         function names: `external rust_fn : int -> int = "bytecode_stub_name" "native_c_stub_name"`.
/// *   **`noalloc` Attribute:**
///     *   `#[ocaml_interop::export(noalloc)]` for functions that must not trigger OCaml GC
///         allocations.
///     *   Requires the runtime argument to be `cr: &OCamlRuntime` (immutable).
///     *   Implies `no_panic_catch`. Panics in `noalloc` functions lead to undefined behavior.
///     *   The corresponding OCaml `external` **must** be annotated with `[@@noalloc]`.
///     *   The user is responsible for ensuring no OCaml allocations occur in the Rust function body.
///
/// ## Argument and Return Value Conventions
///
/// The macro handles the conversion between OCaml's representation and Rust types.
///
/// ### OCaml Values
///
/// *   [`OCaml<T>`]: Represents an OCaml value that is not yet rooted. Its lifetime is tied to the
///     current OCaml runtime scope.
/// *   [`BoxRoot<T>`]: Represents an OCaml value that has been rooted and is protected from the OCaml
///     garbage collector. The `#[ocaml_interop::export]` macro can automatically root arguments
///     if they are specified as `BoxRoot<T>`.
///
/// ### Direct Primitive Type Mapping
///
/// For performance, certain Rust primitive types can be directly mapped to unboxed or untagged
/// OCaml types. This avoids boxing overhead.
///
/// | Rust Type | OCaml Type        | OCaml `external` Attribute(s) Needed |
/// | :-------- | :---------------- | :----------------------------------- |
/// | `f64`     | `float`           | `[@@unboxed]` (or on arg/ret type)   |
/// | `i64`     | `int64`           | `[@@unboxed]` (or on arg/ret type)   |
/// | `i32`     | `int32`           | `[@@unboxed]` (or on arg/ret type)   |
/// | `bool`    | `bool`            | `[@untagged]` (or on arg/ret type)   |
/// | `isize`   | `int`             | `[@untagged]` (or on arg/ret type)   |
/// | `()`      | `unit`            | (Usually implicit for return)        |
///
/// **Example (OCaml `external` for direct primitives):**
/// ```ocaml
/// external process_primitive_values :
///   (int [@untagged]) ->
///   (bool [@untagged]) ->
///   (float [@unboxed]) ->
///   (int32 [@unboxed]) =
///   "" "process_primitive_values"
/// ```
///
/// For more detailed information, refer to the user guides, particularly 
/// [Exporting Rust Functions (Part 3)](user_guides::part3_exporting_rust_functions_to_ocaml)
///
/// [`OCaml<T>`]: OCaml
/// [`OCaml<'gc, T>`]: OCaml
/// [`BoxRoot<T>`]: BoxRoot
/// [`&mut OCamlRuntime`]: OCamlRuntime
/// [`&OCamlRuntime`]: OCamlRuntime
pub use ocaml_interop_derive::export;

#[doc(hidden)]
pub mod internal {
    pub use crate::closure::OCamlClosure;
    pub use crate::memory::{alloc_tuple, caml_alloc, store_field};
    pub use crate::mlvalues::tag;
    pub use crate::mlvalues::UNIT;
    pub use crate::runtime::internal::{recover_runtime_handle, recover_runtime_handle_mut};
    pub use ocaml_boxroot_sys::boxroot_teardown;
    pub use ocaml_sys::caml_hash_variant;
    use std::ffi::CString;
    use std::sync::OnceLock;

    // To bypass ocaml_sys::int_val unsafe declaration
    pub fn int_val(val: super::RawOCaml) -> isize {
        unsafe { ocaml_sys::int_val(val) }
    }

    // To bypass ocaml_sys::caml_sys_double_val unsafe declaration
    pub fn float_val(val: super::RawOCaml) -> f64 {
        unsafe { ocaml_sys::caml_sys_double_val(val) }
    }

    pub fn int32_val(val: super::RawOCaml) -> i32 {
        unsafe { crate::mlvalues::int32_val(val) }
    }

    pub fn int64_val(val: super::RawOCaml) -> i64 {
        unsafe { crate::mlvalues::int64_val(val) }
    }

    pub fn alloc_int32(val: i32) -> super::RawOCaml {
        unsafe { ocaml_sys::caml_copy_int32(val) }
    }

    pub fn alloc_int64(val: i64) -> super::RawOCaml {
        unsafe { ocaml_sys::caml_copy_int64(val) }
    }

    pub fn alloc_float(val: f64) -> super::RawOCaml {
        unsafe { ocaml_sys::caml_copy_double(val) }
    }

    pub fn make_ocaml_bool(val: bool) -> super::RawOCaml {
        unsafe { ocaml_sys::val_int(val as isize) }
    }

    pub fn make_ocaml_int(val: isize) -> super::RawOCaml {
        unsafe { ocaml_sys::val_int(val) }
    }

    // Static storage for the OCaml exception constructor for Rust panics.
    // This will hold the OCaml value for an exception like `exception RustPanic of string`.
    static RUST_PANIC_EXCEPTION_CONSTRUCTOR: OnceLock<Option<super::RawOCaml>> = OnceLock::new();

    /// Retrieves the OCaml exception constructor for `RustPanic`.
    ///
    /// This function attempts to get a reference to an OCaml exception constructor
    /// that should be registered from the OCaml side using a name (e.g., "rust_panic_exn").
    /// Example OCaml registration:
    /// ```ocaml
    /// exception RustPanic of string
    /// let () = Callback.register_exception "rust_panic_exn" (RustPanic "")
    /// ```
    /// Returns `None` if the exception is not found (i.e., not registered by the OCaml code).
    ///
    /// # Safety
    /// This function is unsafe because `ocaml_sys::caml_named_value` interacts with the OCaml
    /// runtime and must be called when the OCaml runtime lock is held and the runtime is initialized.
    unsafe fn get_rust_panic_exception_constructor() -> Option<super::RawOCaml> {
        if let Some(constructor_val_opt) = RUST_PANIC_EXCEPTION_CONSTRUCTOR.get() {
            return *constructor_val_opt;
        }

        let exn_name_cstr = CString::new("rust_panic_exn").unwrap();
        let constructor_ptr = ocaml_sys::caml_named_value(exn_name_cstr.as_ptr());

        if constructor_ptr.is_null() {
            RUST_PANIC_EXCEPTION_CONSTRUCTOR.set(None).ok();

            None
        } else {
            let constructor_val = *constructor_ptr;

            RUST_PANIC_EXCEPTION_CONSTRUCTOR
                .set(Some(constructor_val))
                .ok();

            Some(constructor_val)
        }
    }

    /// # Safety
    /// This function is intended to be called from the `#[export]` macro when a Rust panic is caught.
    /// It attempts to raise a custom OCaml exception (e.g., `RustPanic of string`) with the provided message.
    /// If the custom exception is not registered on the OCaml side, it falls back to `caml_failwith`.
    /// This function will likely not return to the Rust caller in the traditional sense,
    /// as it transfers control to the OCaml runtime's exception handling mechanism.
    /// The OCaml runtime must be initialized, and the current thread must hold the domain lock.
    pub unsafe fn raise_rust_panic_exception(msg: &str) {
        let c_msg = CString::new(msg).unwrap_or_else(|_| {
            CString::new("Rust panic: Invalid message content (e.g., null bytes)").unwrap()
        });

        match get_rust_panic_exception_constructor() {
            Some(rust_panic_exn_constructor) => {
                // Raise the custom OCaml exception `RustPanic "message"`.
                ocaml_sys::caml_raise_with_string(
                    rust_panic_exn_constructor,
                    c_msg.as_ptr() as *const ocaml_sys::Char,
                );
            }
            None => {
                // Fallback to caml_failwith if the custom exception is not found.
                ocaml_sys::caml_failwith(c_msg.as_ptr() as *const ocaml_sys::Char);
            }
        }

        // caml_raise_with_string or caml_failwith should not return. If they do, it's an issue.
        std::process::abort(); // As a last resort if OCaml exception raising returns.
    }

    pub unsafe fn process_panic_payload_and_raise_ocaml_exception(
        panic_payload: Box<dyn ::std::any::Any + Send>,
    ) {
        let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Rust panic occurred, but unable to extract panic message."
        };
        raise_rust_panic_exception(msg);
        unreachable!(
            "raise_rust_panic_exception should have already transferred control or aborted."
        );
    }
}

#[doc(hidden)]
#[cfg(doctest)]
pub mod compile_fail_tests;

#[doc(hidden)]
#[cfg(test)]
mod compile_ok_tests;
