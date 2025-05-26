// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

//! Defines the `DefaultOCamlMapping` trait and implementations for default
//! Rust-to-OCaml type mappings used by derive macros.

use crate::{OCamlBytes, OCamlFloat, OCamlInt32, OCamlInt64, OCamlList};

/// A trait to specify the default OCaml type for a given Rust type.
///
/// Derive macros like `ToOCaml` and `FromOCaml` can use this trait to infer
/// the OCaml representation when an `#[ocaml(as_ = "...")]` attribute is not
/// explicitly provided.
pub trait DefaultOCamlMapping {
    /// The default OCaml type that this Rust type maps to.
    type OCamlType;
}

/// A trait to specify the default Rust type for a given OCaml type.
///
/// Derive macros like `FromOCaml` can use this trait to infer
/// the Rust representation when an `#[ocaml(as_ = "...")]` attribute is not
/// explicitly provided for the target Rust type.
pub trait DefaultRustMapping {
    /// The default Rust type that this OCaml type maps to.
    type RustType;
}

macro_rules! define_bidirectional_mapping {
    ($($rust_type:ty => $ocaml_type:ty),+ $(,)?) => {
        $(
            impl DefaultOCamlMapping for $rust_type {
                type OCamlType = $ocaml_type;
            }

            impl DefaultRustMapping for $ocaml_type {
                type RustType = $rust_type;
            }
        )+
    };
}

define_bidirectional_mapping! {
    f64 => OCamlFloat,
    i64 => OCamlInt64,
    i32 => OCamlInt32,
    String => String, // OCaml `string` is represented as Rust `String`
    bool => bool,     // OCaml `bool` is represented as Rust `bool`
    () => (),
    Box<[u8]> => OCamlBytes // Box<[u8]> often maps to a byte array directly
}

impl<T: DefaultOCamlMapping> DefaultOCamlMapping for &T {
    type OCamlType = <T as DefaultOCamlMapping>::OCamlType;
}

macro_rules! impl_bidirectional_mapping_for_tuple {
    ($(($($RustTypeVar:ident),+), ($($OCamlTypeVar:ident),+))+) => {
        $(
            // Rust Tuple -> OCaml Tuple
            impl<$($RustTypeVar),+, $($OCamlTypeVar),+> DefaultOCamlMapping for ($($RustTypeVar),+)
            where
                $($RustTypeVar: DefaultOCamlMapping<OCamlType = $OCamlTypeVar>),+
            {
                type OCamlType = ($($OCamlTypeVar),+);
            }

            // OCaml Tuple -> Rust Tuple
            // Note: The type parameters $RustTypeVar and $OCamlTypeVar are bound at the impl level.
            // The where clauses ensure the correct mappings.
            impl<$($RustTypeVar),+, $($OCamlTypeVar),+> DefaultRustMapping for ($($OCamlTypeVar),+)
            where
                $($OCamlTypeVar: DefaultRustMapping<RustType = $RustTypeVar>),+
            {
                type RustType = ($($RustTypeVar),+);
            }
        )+
    };
}

impl_bidirectional_mapping_for_tuple! { (A, B), (OCamlA, OCamlB) }
impl_bidirectional_mapping_for_tuple! { (A, B, C), (OCamlA, OCamlB, OCamlC) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D), (OCamlA, OCamlB, OCamlC, OCamlD) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D, E), (OCamlA, OCamlB, OCamlC, OCamlD, OCamlE) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D, E, F), (OCamlA, OCamlB, OCamlC, OCamlD, OCamlE, OCamlF) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D, E, F, G), (OCamlA, OCamlB, OCamlC, OCamlD, OCamlE, OCamlF, OCamlG) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D, E, F, G, H), (OCamlA, OCamlB, OCamlC, OCamlD, OCamlE, OCamlF, OCamlG, OCamlH) }
impl_bidirectional_mapping_for_tuple! { (A, B, C, D, E, F, G, H, I), (OCamlA, OCamlB, OCamlC, OCamlD, OCamlE, OCamlF, OCamlG, OCamlH, OCamlI) }

// --- DefaultOCamlMapping for Generic Types ---

// For Box<T>, the OCaml type is the same as T's OCaml type.
// The boxing is a Rust-side memory management detail for this mapping.
impl<T: DefaultOCamlMapping> DefaultOCamlMapping for Box<T> {
    type OCamlType = T::OCamlType;
}

// For Option<T>, we conceptually map to an OCaml option holding T's OCaml type.
// The actual conversion is handled by ToOCaml/FromOCaml impls for Option.
impl<T: DefaultOCamlMapping> DefaultOCamlMapping for Option<T> {
    type OCamlType = Option<T::OCamlType>;
}

// For Result<T, E>, similar conceptual mapping.
impl<T: DefaultOCamlMapping, E: DefaultOCamlMapping> DefaultOCamlMapping for Result<T, E> {
    type OCamlType = Result<T::OCamlType, E::OCamlType>;
}

// For Vec<T>, maps to OCamlList of T's OCaml type.
// Note: Vec<u8> will map to OCamlList<OCamlInt> via this generic impl.
// If OCamlBytes is desired for Vec<u8>, #[ocaml(as_ = "OCamlBytes")] should be used.
impl<T: DefaultOCamlMapping> DefaultOCamlMapping for Vec<T> {
    type OCamlType = OCamlList<T::OCamlType>;
}

// --- DefaultRustMapping for Generic OCaml Types (or conceptual OCaml types) ---

// No direct structural DefaultRustMapping for the OCamlType of a generic Box<RustT>
// because T::OCamlType is not necessarily a "Boxed OCaml" type.
// The DefaultRustMapping for T::OCamlType would yield T, and the FromOCaml derive
// would handle boxing.

// For OCaml's conceptual option type. T is a placeholder for an OCaml type
// that implements DefaultRustMapping.
impl<T: DefaultRustMapping> DefaultRustMapping for Option<T> {
    type RustType = Option<T::RustType>;
}

// For OCaml's conceptual result type. T, E are placeholders for OCaml types.
impl<T: DefaultRustMapping, E: DefaultRustMapping> DefaultRustMapping for Result<T, E> {
    type RustType = Result<T::RustType, E::RustType>;
}

// For OCamlList<T>, where T is a placeholder for an OCaml type.
impl<T: DefaultRustMapping> DefaultRustMapping for OCamlList<T> {
    type RustType = Vec<T::RustType>;
}

// Add more default implementations as needed
